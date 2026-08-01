//! Companion orchestrator — runs Lanes in 3-phase order.
//!
//! Owns the `Vec<Box<dyn Lane>>` and the shared `WorldState`. Each
//! router-loop iteration calls `tick()`, which runs Sense lanes
//! (write WorldState), then Mutate lanes (write HarmonyEngine), then
//! Decide lanes (emit DispatchOps). Each input event triggers
//! `on_input()`, which dispatches to filter-matching lanes and tells
//! the input pipeline whether to skip the default harmonize.
//!
//! Phase 1.4 ships the orchestrator with no concrete Lanes. The
//! orchestrator's contract is testable on its own with a `TestLane`
//! fixture (see tests).

#![allow(dead_code)]

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};

use contrapunk_harmony::HarmonyEngine;

use super::lane::{EngineMutation, InputEvent, Lane, LanePhase, LaneState, WorldWrite};
use super::world::WorldState;
use super::DispatchOp;

/// Result of `Companion::on_input` returned to the input pipeline.
///
/// `ops` are the dispatch ops emitted by all matched Lanes.
/// `suppress_default = true` means at least one Lane wants the input
/// pipeline to skip the default `harmonize_note_on` for this event.
pub struct CompanionInputResult {
    pub ops: Vec<DispatchOp>,
    pub suppress_default: bool,
}

/// Persisted snapshot of the whole companion. Written into the rig
/// JSON; restored on rig load. Lane order is preserved so phase
/// ordering is stable across save/load.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CompanionState {
    pub enabled: bool,
    #[serde(default = "default_phrase_gap_beats")]
    pub phrase_gap_beats: f64,
    pub lanes: Vec<LaneState>,
}

fn default_phrase_gap_beats() -> f64 {
    crate::phrase::DEFAULT_PHRASE_GAP_BEATS
}

impl Default for CompanionState {
    fn default() -> Self {
        Self {
            enabled: false,
            phrase_gap_beats: default_phrase_gap_beats(),
            lanes: Vec::new(),
        }
    }
}

/// The companion. Owned by the router thread; UI commands flow in
/// through Tauri command handlers that take a brief lock.
pub struct Companion {
    /// Master enable. When false, `tick` and `on_input` short-circuit
    /// to empty results — Lanes don't run, no allocations.
    pub enabled: AtomicBool,

    /// Shared observable.
    pub world: Arc<WorldState>,

    /// Lanes in declaration order. The orchestrator filters by phase
    /// at iteration time rather than maintaining three sub-vectors —
    /// the Lane count stays small (<20) so the linear scan cost is
    /// noise.
    pub lanes: Vec<Box<dyn Lane>>,
}

impl Companion {
    /// Construct an empty companion sharing the given `WorldState`.
    /// Phase 1.4 callers register no Lanes; later phases append
    /// `LooperLane` etc. via `lanes.push(...)` or a registry helper.
    pub fn new(world: Arc<WorldState>) -> Self {
        Self {
            enabled: AtomicBool::new(false),
            world,
            lanes: Vec::new(),
        }
    }

    /// Snapshot the global hold mode. Cheap (Mutex lock + Copy).
    /// Delegates to the single owner on `WorldState` — lanes also
    /// read from there during their NoteOff handling. Keeping one
    /// source of truth avoids consistency bugs.
    pub fn global_hold_mode(&self) -> crate::lane::HoldMode {
        *self
            .world
            .global_hold_mode
            .lock()
            .expect("global_hold_mode mutex poisoned")
    }

    /// Replace the global hold mode default. The new value applies to
    /// the next NoteOff seen by any inheriting lane/voice, including
    /// emissions buffered before this setter ran. Hold resolution is
    /// intentionally deferred until the source note is released.
    pub fn set_global_hold_mode(&self, mode: crate::lane::HoldMode) {
        if let Ok(mut g) = self.world.global_hold_mode.lock() {
            *g = mode;
        }
    }

    pub fn set_phrase_gap_beats(&self, beats: f64) -> Result<(), String> {
        self.world.set_phrase_gap_beats(beats)
    }

    pub fn phrase_snapshot(&self) -> crate::phrase::PhraseSnapshot {
        self.world.phrase_snapshot()
    }

    /// Update phrase ownership without running arrangement lanes.
    pub fn observe_phrase_input(&self, event: InputEvent) {
        self.world.observe_phrase_input(event);
    }

    /// Advance phrase timeout state without running arrangement lanes.
    pub fn advance_phrase(&self) {
        self.world.advance_phrase();
    }

    /// Clear every lane's in-flight runtime state while preserving its
    /// configuration. This is allocation-free and safe for panic/stop
    /// handling on the processing thread.
    pub fn reset_runtime(&mut self) {
        self.world.reset_phrase_runtime();
        for lane in &mut self.lanes {
            lane.reset_runtime();
        }
    }

    /// Run one iteration: Sense → Mutate → Decide. Returns the
    /// concatenated dispatch ops from all Decide lanes.
    ///
    /// `engine` is passed in (rather than reaching into
    /// `world.engine_snapshot`) so tests can pass an isolated engine
    /// without standing up a full `WorldState::new` wiring.
    pub fn tick(&mut self, engine: &Mutex<HarmonyEngine>) -> Vec<DispatchOp> {
        self.world.advance_phrase();
        if !self.enabled.load(Ordering::Acquire) {
            return Vec::new();
        }

        // Phase 1: Sense — write WorldState.
        for lane in self
            .lanes
            .iter_mut()
            .filter(|l| l.phase() == LanePhase::Sense)
        {
            let out = lane.tick(&self.world);
            apply_world_writes(&self.world, out.world_writes);
        }

        // Phase 2: Mutate — write HarmonyEngine.
        for lane in self
            .lanes
            .iter_mut()
            .filter(|l| l.phase() == LanePhase::Mutate)
        {
            let out = lane.tick(&self.world);
            if !out.engine_mutations.is_empty() {
                let mut e = engine.lock().expect("engine mutex poisoned");
                for m in out.engine_mutations {
                    apply_engine_mutation(&mut e, m);
                }
            }
        }

        // Phase 3: Decide — emit dispatch ops.
        let mut all_ops = Vec::new();
        for lane in self
            .lanes
            .iter_mut()
            .filter(|l| l.phase() == LanePhase::Decide)
        {
            let out = lane.tick(&self.world);
            all_ops.extend(out.ops);
        }
        all_ops
    }

    /// Variant of `tick` that tags each emitted DispatchOp with the
    /// lane type and stable voice slot used by routing, Slide, and the UI.
    pub fn tick_tagged(
        &mut self,
        engine: &Mutex<HarmonyEngine>,
    ) -> Vec<(&'static str, u8, DispatchOp)> {
        self.world.advance_phrase();
        if !self.enabled.load(Ordering::Acquire) {
            return Vec::new();
        }

        // Sense + Mutate are identical to `tick`; only Decide
        // emissions get the lane_id stamp.
        for lane in self
            .lanes
            .iter_mut()
            .filter(|l| l.phase() == LanePhase::Sense)
        {
            let out = lane.tick(&self.world);
            apply_world_writes(&self.world, out.world_writes);
        }
        for lane in self
            .lanes
            .iter_mut()
            .filter(|l| l.phase() == LanePhase::Mutate)
        {
            let out = lane.tick(&self.world);
            if !out.engine_mutations.is_empty() {
                let mut e = engine.lock().expect("engine mutex poisoned");
                for m in out.engine_mutations {
                    apply_engine_mutation(&mut e, m);
                }
            }
        }

        let mut tagged = Vec::new();
        for lane in self
            .lanes
            .iter_mut()
            .filter(|l| l.phase() == LanePhase::Decide)
        {
            let lane_id = lane.type_id();
            for (voice_slot, op) in lane.tick_slotted(&self.world) {
                tagged.push((lane_id, voice_slot, op));
            }
        }
        tagged
    }

    /// Dispatch one input event to filter-matching Lanes. Shared phrase
    /// ownership is updated exactly once before lane fan-out so every lane
    /// observes the same post-event state.
    pub fn on_input(
        &mut self,
        ev: InputEvent,
        engine: &Mutex<HarmonyEngine>,
    ) -> CompanionInputResult {
        self.world.observe_phrase_input(ev);
        if !self.enabled.load(Ordering::Acquire) {
            return CompanionInputResult {
                ops: Vec::new(),
                suppress_default: false,
            };
        }

        let mut ops = Vec::new();
        let mut suppress_default = false;

        for lane in self.lanes.iter_mut() {
            if !lane.input_filter().matches(&ev) {
                continue;
            }
            let out = lane.on_input(ev, &self.world);
            ops.extend(out.ops);
            if out.suppress_default {
                suppress_default = true;
            }
            apply_world_writes(&self.world, out.world_writes);
            if !out.engine_mutations.is_empty() {
                let mut e = engine.lock().expect("engine mutex poisoned");
                for m in out.engine_mutations {
                    apply_engine_mutation(&mut e, m);
                }
            }
        }

        CompanionInputResult {
            ops,
            suppress_default,
        }
    }

    /// Variant of `on_input` that tags each emitted DispatchOp with
    /// the `lane.type_id()` that produced it. Same role as
    /// `tick_tagged` — surfaces per-lane attribution to the UI.
    pub fn on_input_tagged(
        &mut self,
        ev: InputEvent,
        engine: &Mutex<HarmonyEngine>,
    ) -> (Vec<(&'static str, u8, DispatchOp)>, bool) {
        self.world.observe_phrase_input(ev);
        if !self.enabled.load(Ordering::Acquire) {
            return (Vec::new(), false);
        }

        let mut tagged: Vec<(&'static str, u8, DispatchOp)> = Vec::new();
        let mut suppress_default = false;

        for lane in self.lanes.iter_mut() {
            if !lane.input_filter().matches(&ev) {
                continue;
            }
            let lane_id = lane.type_id();
            let out = lane.on_input(ev, &self.world);
            for op in out.ops {
                tagged.push((lane_id, 0, op));
            }
            if out.suppress_default {
                suppress_default = true;
            }
            apply_world_writes(&self.world, out.world_writes);
            if !out.engine_mutations.is_empty() {
                let mut e = engine.lock().expect("engine mutex poisoned");
                for m in out.engine_mutations {
                    apply_engine_mutation(&mut e, m);
                }
            }
        }

        (tagged, suppress_default)
    }

    /// Apply a partial JSON state blob to the first Lane matching the
    /// given `type_id`. Used by per-Lane Tauri command handlers to
    /// update specific config fields without needing to downcast the
    /// `Box<dyn Lane>` to a concrete type. The Lane's
    /// `deserialize_state` is expected to merge (skip missing fields)
    /// rather than fully replace; see `CanonLane::deserialize_state`
    /// for the canonical pattern.
    pub fn configure_lane(
        &mut self,
        type_id: &str,
        state: serde_json::Value,
    ) -> Result<(), String> {
        for lane in self.lanes.iter_mut() {
            if lane.type_id() == type_id {
                return lane.deserialize_state(state);
            }
        }
        Err(format!("No lane with type_id '{}' registered", type_id))
    }

    /// Read the current state of the first Lane matching the type_id.
    /// Returns `None` if no such Lane is registered.
    pub fn lane_state(&self, type_id: &str) -> Option<serde_json::Value> {
        self.lanes
            .iter()
            .find(|l| l.type_id() == type_id)
            .map(|l| l.serialize_state())
    }

    /// Snapshot every Lane's state. Order matches `lanes`.
    pub fn save(&self) -> CompanionState {
        CompanionState {
            enabled: self.enabled.load(Ordering::Acquire),
            phrase_gap_beats: self.phrase_snapshot().gap_beats,
            lanes: self
                .lanes
                .iter()
                .map(|l| LaneState {
                    type_id: l.type_id().to_string(),
                    state: l.serialize_state(),
                })
                .collect(),
        }
    }

    /// Restore Lane states. Matches saved Lane states to live Lanes
    /// by `type_id` in order — Lane count and order in `self.lanes`
    /// must match what was saved. Returns the first restoration
    /// error if any Lane refuses its state.
    pub fn restore(&mut self, state: CompanionState) -> Result<(), String> {
        self.set_phrase_gap_beats(state.phrase_gap_beats)?;
        self.enabled.store(state.enabled, Ordering::Release);

        if state.lanes.len() != self.lanes.len() {
            return Err(format!(
                "lane count mismatch: saved {}, live {}",
                state.lanes.len(),
                self.lanes.len()
            ));
        }

        for (i, saved) in state.lanes.into_iter().enumerate() {
            let lane = &mut self.lanes[i];
            if lane.type_id() != saved.type_id {
                return Err(format!(
                    "lane type mismatch at index {}: saved '{}', live '{}'",
                    i,
                    saved.type_id,
                    lane.type_id()
                ));
            }
            lane.deserialize_state(saved.state)
                .map_err(|e| format!("lane '{}' restore failed: {}", saved.type_id, e))?;
        }
        Ok(())
    }
}

/// Apply a batch of `WorldWrite`s to the shared `WorldState`. Kept as
/// a free function so both `tick` and `on_input` reuse it without a
/// borrow split between `&mut self.lanes` and `&self.world`.
fn apply_world_writes(world: &WorldState, writes: Vec<WorldWrite>) {
    for w in writes {
        match w {
            WorldWrite::UpdateChord(chord) => {
                *world.current_chord.lock().expect("chord mutex poisoned") = chord;
            }
            WorldWrite::UpdateDetectedKey(key) => {
                world
                    .engine_snapshot
                    .lock()
                    .expect("engine mutex poisoned")
                    .set_key(key);
            }
            WorldWrite::UpdateDetectedMode(mode) => {
                world
                    .engine_snapshot
                    .lock()
                    .expect("engine mutex poisoned")
                    .set_mode(mode);
            }
        }
    }
}

/// Apply one `EngineMutation` under an already-held engine lock. The
/// orchestrator hoists the lock acquisition outside the per-mutation
/// loop so a Lane emitting many mutations doesn't churn the mutex.
fn apply_engine_mutation(engine: &mut HarmonyEngine, m: EngineMutation) {
    match m {
        EngineMutation::SetKey(k) => engine.set_key(k),
        EngineMutation::SetMode(mode) => engine.set_mode(mode),
        EngineMutation::SetScale(scale) => engine.set_scale_mode(scale),
        EngineMutation::SetVoiceLeading(style) => engine.set_voice_leading_style(style),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lane::{InputFilter, LaneOutput};
    use crate::world::DetectedChord;
    use contrapunk_harmony::{HarmonyMode, Key};
    use contrapunk_transport::Transport;
    use std::sync::atomic::AtomicUsize;

    /// Generic test Lane. Records every tick and on_input call into a
    /// shared trace and emits the configured output. Lets tests
    /// assert phase ordering, filter dispatch, and suppress_default
    /// propagation without wiring concrete behavior.
    struct TestLane {
        id: &'static str,
        phase: LanePhase,
        filter: InputFilter,
        tick_output: Vec<DispatchOp>,
        engine_mutations: Vec<EngineMutation>,
        world_writes: Vec<WorldWrite>,
        on_input_output: Vec<DispatchOp>,
        suppress_default: bool,
        trace: Arc<Mutex<Vec<String>>>,
        tick_count: Arc<AtomicUsize>,
    }

    impl TestLane {
        fn new(id: &'static str, phase: LanePhase, trace: Arc<Mutex<Vec<String>>>) -> Self {
            Self {
                id,
                phase,
                filter: InputFilter::None,
                tick_output: Vec::new(),
                engine_mutations: Vec::new(),
                world_writes: Vec::new(),
                on_input_output: Vec::new(),
                suppress_default: false,
                trace,
                tick_count: Arc::new(AtomicUsize::new(0)),
            }
        }

        fn with_filter(mut self, f: InputFilter) -> Self {
            self.filter = f;
            self
        }

        fn with_suppress_default(mut self, s: bool) -> Self {
            self.suppress_default = s;
            self
        }
    }

    impl Lane for TestLane {
        fn name(&self) -> &str {
            self.id
        }
        fn type_id(&self) -> &'static str {
            self.id
        }
        fn phase(&self) -> LanePhase {
            self.phase
        }
        fn input_filter(&self) -> InputFilter {
            // Re-construct an equivalent filter — the trait wants an
            // owned value but we need to peek at our config without
            // moving it out. For tests we only use static variants.
            match &self.filter {
                InputFilter::None => InputFilter::None,
                InputFilter::All => InputFilter::All,
                InputFilter::NoteRange { min, max } => InputFilter::NoteRange {
                    min: *min,
                    max: *max,
                },
                InputFilter::Channel(c) => InputFilter::Channel(*c),
                // Predicate isn't trivially clonable; tests don't use it.
                InputFilter::Predicate(_) => InputFilter::None,
            }
        }
        fn tick(&mut self, _world: &WorldState) -> LaneOutput {
            self.trace.lock().unwrap().push(format!("tick:{}", self.id));
            self.tick_count.fetch_add(1, Ordering::Relaxed);
            LaneOutput {
                ops: self.tick_output.clone(),
                engine_mutations: self.engine_mutations.clone(),
                world_writes: self.world_writes.clone(),
                suppress_default: false,
            }
        }
        fn on_input(&mut self, _ev: InputEvent, _world: &WorldState) -> LaneOutput {
            self.trace
                .lock()
                .unwrap()
                .push(format!("on_input:{}", self.id));
            LaneOutput {
                ops: self.on_input_output.clone(),
                engine_mutations: Vec::new(),
                world_writes: Vec::new(),
                suppress_default: self.suppress_default,
            }
        }
        fn serialize_state(&self) -> serde_json::Value {
            serde_json::json!({ "tick_count": self.tick_count.load(Ordering::Relaxed) })
        }
        fn deserialize_state(&mut self, state: serde_json::Value) -> Result<(), String> {
            let count = state
                .get("tick_count")
                .and_then(|v| v.as_u64())
                .ok_or_else(|| "missing tick_count".to_string())?;
            self.tick_count.store(count as usize, Ordering::Relaxed);
            Ok(())
        }
    }

    fn fixture() -> (Companion, Arc<Mutex<HarmonyEngine>>) {
        let transport = Transport::new(48_000);
        let engine = Arc::new(Mutex::new(HarmonyEngine::new(
            Key::C,
            HarmonyMode::PassThrough,
        )));
        let world = WorldState::new(transport, engine.clone());
        (Companion::new(world), engine)
    }

    #[test]
    fn disabled_companion_emits_nothing() {
        let (mut c, engine) = fixture();
        c.lanes.push(Box::new(TestLane::new(
            "decide",
            LanePhase::Decide,
            Arc::new(Mutex::new(Vec::new())),
        )));
        // enabled defaults to false.
        assert_eq!(c.tick(&engine), Vec::<DispatchOp>::new());
    }

    #[test]
    fn tick_runs_phases_in_order_sense_mutate_decide() {
        let (mut c, engine) = fixture();
        c.enabled.store(true, Ordering::Release);
        let trace = Arc::new(Mutex::new(Vec::new()));

        // Mix declaration order on purpose — orchestrator must
        // reorder by phase regardless of how Lanes were registered.
        c.lanes.push(Box::new(TestLane::new(
            "d1",
            LanePhase::Decide,
            trace.clone(),
        )));
        c.lanes.push(Box::new(TestLane::new(
            "s1",
            LanePhase::Sense,
            trace.clone(),
        )));
        c.lanes.push(Box::new(TestLane::new(
            "m1",
            LanePhase::Mutate,
            trace.clone(),
        )));
        c.lanes.push(Box::new(TestLane::new(
            "s2",
            LanePhase::Sense,
            trace.clone(),
        )));

        c.tick(&engine);
        let log = trace.lock().unwrap();
        assert_eq!(
            log.iter().map(String::as_str).collect::<Vec<_>>(),
            vec!["tick:s1", "tick:s2", "tick:m1", "tick:d1"]
        );
    }

    #[test]
    fn tick_concatenates_decide_ops_in_declaration_order() {
        let (mut c, engine) = fixture();
        c.enabled.store(true, Ordering::Release);
        let trace = Arc::new(Mutex::new(Vec::new()));

        let target = crate::voice_output::VoiceOutputTarget::Synth;
        let mut a = TestLane::new("a", LanePhase::Decide, trace.clone());
        a.tick_output = vec![DispatchOp::NoteOn {
            target,
            note: 60,
            velocity: 100,
            channel: 0,
        }];
        let mut b = TestLane::new("b", LanePhase::Decide, trace.clone());
        b.tick_output = vec![DispatchOp::NoteOn {
            target,
            note: 64,
            velocity: 100,
            channel: 0,
        }];

        c.lanes.push(Box::new(a));
        c.lanes.push(Box::new(b));

        let ops = c.tick(&engine);
        assert_eq!(ops.len(), 2);
        match (&ops[0], &ops[1]) {
            (DispatchOp::NoteOn { note: 60, .. }, DispatchOp::NoteOn { note: 64, .. }) => {}
            _ => panic!("ops out of declaration order: {:?}", ops),
        }
    }

    #[test]
    fn mutate_phase_writes_engine() {
        let (mut c, engine) = fixture();
        c.enabled.store(true, Ordering::Release);

        let mut m = TestLane::new("m", LanePhase::Mutate, Arc::new(Mutex::new(Vec::new())));
        m.engine_mutations = vec![EngineMutation::SetKey(Key::F)];
        c.lanes.push(Box::new(m));

        c.tick(&engine);
        assert_eq!(engine.lock().unwrap().key(), Key::F);
    }

    #[test]
    fn sense_phase_writes_world() {
        let (mut c, engine) = fixture();
        c.enabled.store(true, Ordering::Release);

        let world_handle = c.world.clone();
        let mut s = TestLane::new("s", LanePhase::Sense, Arc::new(Mutex::new(Vec::new())));
        s.world_writes = vec![WorldWrite::UpdateChord(DetectedChord {
            root: Some(60),
            quality: None,
            display: "Cmaj".into(),
        })];
        c.lanes.push(Box::new(s));

        c.tick(&engine);
        assert_eq!(
            world_handle.current_chord.lock().unwrap().display.as_str(),
            "Cmaj"
        );
    }

    #[test]
    fn on_input_dispatches_only_to_filter_matching_lanes() {
        let (mut c, engine) = fixture();
        c.enabled.store(true, Ordering::Release);
        let trace = Arc::new(Mutex::new(Vec::new()));

        c.lanes.push(Box::new(
            TestLane::new("none", LanePhase::Decide, trace.clone()).with_filter(InputFilter::None),
        ));
        c.lanes.push(Box::new(
            TestLane::new("all", LanePhase::Decide, trace.clone()).with_filter(InputFilter::All),
        ));
        c.lanes.push(Box::new(
            TestLane::new("range_low", LanePhase::Decide, trace.clone())
                .with_filter(InputFilter::NoteRange { min: 36, max: 47 }),
        ));
        c.lanes.push(Box::new(
            TestLane::new("range_high", LanePhase::Decide, trace.clone())
                .with_filter(InputFilter::NoteRange { min: 60, max: 72 }),
        ));

        c.on_input(
            InputEvent::NoteOn {
                note: 36,
                velocity: 100,
                channel: 0,
            },
            &engine,
        );

        let log = trace.lock().unwrap();
        let calls: Vec<&str> = log.iter().map(String::as_str).collect();
        assert!(calls.contains(&"on_input:all"));
        assert!(calls.contains(&"on_input:range_low"));
        assert!(!calls.contains(&"on_input:none"));
        assert!(!calls.contains(&"on_input:range_high"));
    }

    #[test]
    fn on_input_propagates_suppress_default() {
        let (mut c, engine) = fixture();
        c.enabled.store(true, Ordering::Release);

        c.lanes.push(Box::new(
            TestLane::new("a", LanePhase::Decide, Arc::new(Mutex::new(Vec::new())))
                .with_filter(InputFilter::All)
                .with_suppress_default(false),
        ));
        c.lanes.push(Box::new(
            TestLane::new("b", LanePhase::Decide, Arc::new(Mutex::new(Vec::new())))
                .with_filter(InputFilter::All)
                .with_suppress_default(true),
        ));

        let result = c.on_input(
            InputEvent::NoteOn {
                note: 60,
                velocity: 100,
                channel: 0,
            },
            &engine,
        );
        assert!(
            result.suppress_default,
            "any-Lane suppress should propagate"
        );
    }

    #[test]
    fn on_input_no_suppress_when_no_lane_requests() {
        let (mut c, engine) = fixture();
        c.enabled.store(true, Ordering::Release);
        c.lanes.push(Box::new(
            TestLane::new("a", LanePhase::Decide, Arc::new(Mutex::new(Vec::new())))
                .with_filter(InputFilter::All),
        ));
        let result = c.on_input(
            InputEvent::NoteOn {
                note: 60,
                velocity: 100,
                channel: 0,
            },
            &engine,
        );
        assert!(!result.suppress_default);
    }

    #[test]
    fn save_restore_round_trips_lane_state() {
        let (mut c, _engine) = fixture();
        c.enabled.store(true, Ordering::Release);
        c.set_phrase_gap_beats(7.25).unwrap();
        let trace = Arc::new(Mutex::new(Vec::new()));
        let lane = TestLane::new("counter", LanePhase::Decide, trace.clone());
        lane.tick_count.store(7, Ordering::Relaxed);
        c.lanes.push(Box::new(lane));

        let saved = c.save();
        assert!(saved.enabled);
        assert_eq!(saved.phrase_gap_beats, 7.25);
        assert_eq!(saved.lanes.len(), 1);

        // Build a fresh companion with a same-shape Lane and restore.
        let (mut c2, _) = fixture();
        let lane2 = TestLane::new("counter", LanePhase::Decide, trace.clone());
        c2.lanes.push(Box::new(lane2));
        c2.restore(saved).expect("restore");

        assert!(c2.enabled.load(Ordering::Acquire));
        assert_eq!(c2.phrase_snapshot().gap_beats, 7.25);
        // Roundtrip preserved the tick_count via TestLane's
        // serialize/deserialize.
        assert_eq!(c2.lanes[0].serialize_state()["tick_count"], 7);
    }

    #[test]
    fn restore_rejects_lane_count_mismatch() {
        let (mut c, _engine) = fixture();
        let trace = Arc::new(Mutex::new(Vec::new()));
        c.lanes.push(Box::new(TestLane::new(
            "a",
            LanePhase::Decide,
            trace.clone(),
        )));
        let saved = c.save();

        let (mut c2, _) = fixture();
        // c2 has zero lanes, saved has one.
        let err = c2.restore(saved).unwrap_err();
        assert!(err.contains("lane count mismatch"), "got: {}", err);
    }

    #[test]
    fn restore_rejects_lane_type_mismatch() {
        let (mut c, _engine) = fixture();
        let trace = Arc::new(Mutex::new(Vec::new()));
        c.lanes.push(Box::new(TestLane::new(
            "a",
            LanePhase::Decide,
            trace.clone(),
        )));
        let saved = c.save();

        let (mut c2, _) = fixture();
        c2.lanes.push(Box::new(TestLane::new(
            "b",
            LanePhase::Decide,
            trace.clone(),
        )));
        let err = c2.restore(saved).unwrap_err();
        assert!(err.contains("type mismatch"), "got: {}", err);
    }

    #[test]
    fn old_companion_state_defaults_phrase_gap() {
        let state: CompanionState = serde_json::from_value(serde_json::json!({
            "enabled": false,
            "lanes": []
        }))
        .unwrap();
        assert_eq!(
            state.phrase_gap_beats,
            crate::phrase::DEFAULT_PHRASE_GAP_BEATS
        );
    }

    #[test]
    fn on_input_disabled_short_circuits_but_tracks_phrase_input() {
        let (mut c, engine) = fixture();
        // enabled stays false.
        let trace = Arc::new(Mutex::new(Vec::new()));
        c.lanes.push(Box::new(
            TestLane::new("a", LanePhase::Decide, trace.clone()).with_filter(InputFilter::All),
        ));
        let result = c.on_input(
            InputEvent::NoteOn {
                note: 60,
                velocity: 100,
                channel: 0,
            },
            &engine,
        );
        assert!(result.ops.is_empty());
        assert!(!result.suppress_default);
        assert!(trace.lock().unwrap().is_empty());
        assert_eq!(c.phrase_snapshot().latest_note, Some(60));
    }
}
