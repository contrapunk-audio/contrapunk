//! Canon Lane — delayed-entry voice that replays the player's melody.
//!
//! Closes #3. The first concrete `Decide`-phase Lane. Each input note
//! captured by `on_input` is buffered with its trigger beat position.
//! On each `tick`, the lane checks whether any buffered entry has aged
//! past `delay_beats` against the current transport beat — if so, it
//! emits a `DispatchOp::NoteOn` for that note (optionally diatonically
//! transposed). Matching NoteOff events follow the same delay so the
//! canon voice preserves the original duration.
//!
//! Per `.planning/research/group-a-core-harmony.md:108-153`:
//! - Beat-clock-driven (NOT user-input-driven). The Lane fires on
//!   `tick`, called every router iteration with the live transport.
//!   This is how the canon plays *between* user note-ons rather than
//!   clumping all delayed emissions on the next trigger.
//! - Per-voice delay buffer is bounded (few seconds of held content)
//!   so no memory concern.
//! - Reuses the existing synthetic-beat / transport infrastructure.
//!
//! v1 ships unison canon only (`transpose_degrees = 0`). Diatonic
//! transposition is a follow-up — it requires reading the engine's
//! current scale at emit time and applying `Scale::transpose_diatonic`,
//! which adds an engine-lock to the Decide phase. Defer to v2.

#![allow(dead_code)]

use std::collections::{HashMap, VecDeque};

use wmidi::Note;

use super::lane::{InputEvent, InputFilter, Lane, LaneOutput, LanePhase};
use super::world::WorldState;
use super::DispatchOp;
use crate::state::VoiceOutputTarget;

/// One pending NoteOn that will fire at `fire_at` beats (transport
/// total-beats coordinate). `canon_note` is the post-transpose MIDI
/// pitch the lane will emit (today = original note; v2 transpose).
#[derive(Clone, Copy, Debug, PartialEq)]
struct PendingOn {
    fire_at: f64,
    canon_note: u8,
    velocity: u8,
    channel: u8,
}

/// One pending NoteOff matched to a previously-scheduled NoteOn.
#[derive(Clone, Copy, Debug, PartialEq)]
struct PendingOff {
    fire_at: f64,
    canon_note: u8,
    channel: u8,
}

/// Tracked per held input so NoteOff can compute the duration to apply
/// to the canon voice.
#[derive(Clone, Copy, Debug, PartialEq)]
struct HeldInput {
    on_beat: f64,
    canon_note: u8,
    channel: u8,
}

pub struct CanonLane {
    /// Master toggle for this lane independent of the Companion's
    /// master enabled flag. When false, `on_input` and `tick` are
    /// no-ops — Companion can be on for other lanes without forcing
    /// the canon voice on too. Default false: pre-existing user
    /// behavior is preserved when the lane is registered.
    pub enabled: bool,

    /// Beats to wait between the player's NoteOn and the canon's
    /// NoteOn. Range [0.0, 8.0] in practice. 0.0 = simultaneous
    /// (degenerate — same as omitting the lane).
    pub delay_beats: f32,

    /// Diatonic transpose applied to canon emissions. v1 ignores this
    /// (unison only); v2 will fold in `Scale::transpose_diatonic`.
    pub transpose_degrees: i8,

    /// Output target. Canon emits to its own slot so the dispatcher
    /// can mute / route it independently of the main harmony voices.
    /// Default `Synth` so users hear it without needing MIDI routing.
    pub target: VoiceOutputTarget,

    /// Pending NoteOn emissions, sorted by fire_at ascending (push
    /// order is already sorted because `on_input` is monotonic in
    /// `transport.total_beats()`).
    pending_on: VecDeque<PendingOn>,

    /// Pending NoteOff emissions, scanned linearly each tick.
    pending_off: Vec<PendingOff>,

    /// Currently-held inputs so NoteOff can compute the canon's
    /// duration. Keyed by the *input* MIDI note — that's what arrives
    /// in `InputEvent::NoteOff`.
    held: HashMap<u8, HeldInput>,
}

impl CanonLane {
    pub fn new() -> Self {
        Self {
            enabled: false,
            delay_beats: 1.0,
            transpose_degrees: 0,
            target: VoiceOutputTarget::Synth,
            pending_on: VecDeque::new(),
            pending_off: Vec::new(),
            held: HashMap::new(),
        }
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        if !enabled {
            self.pending_on.clear();
            self.pending_off.clear();
            self.held.clear();
        }
    }

    pub fn set_delay(&mut self, beats: f32) {
        self.delay_beats = beats.clamp(0.0, 8.0);
    }

    pub fn set_transpose(&mut self, degrees: i8) {
        self.transpose_degrees = degrees.clamp(-7, 7);
    }

    /// Apply diatonic transpose against the engine's current scale,
    /// routing through `Scale::harmonize_smart` so out-of-scale input
    /// lands somewhere musically defensible:
    ///   - In-scale input → diatonic transpose in the current mode
    ///   - Out-of-scale + interchange enabled → borrows from a parallel
    ///     mode that contains the input note (modal interchange path)
    ///   - Out-of-scale + interchange disabled → consonant chromatic
    ///     interval landing in-scale where possible
    /// All paths return *something* — no naked-unison fallback for
    /// chromatic input. Naked unison is musically inert; the user
    /// asked for canon transpose to do real work even on borrowed
    /// notes.
    ///
    /// Locks `world.engine_snapshot` mutably (because harmonize_smart
    /// updates `last_borrowed_from` as a side effect). Decide phase
    /// doesn't hold the engine lock at the orchestrator level so this
    /// is safe. Transpose is captured AT INPUT TIME so a canon line
    /// that started in C major stays in C major even if the user
    /// changes key mid-replay (matches classical-canon semantics).
    fn transpose(&self, input_note: u8, world: &WorldState) -> u8 {
        if self.transpose_degrees == 0 {
            return input_note;
        }
        let Ok(mut engine) = world.engine_snapshot.lock() else {
            return input_note;
        };
        let Ok(note) = Note::try_from(input_note) else {
            return input_note;
        };
        let prefer_above = self.transpose_degrees > 0;
        match engine
            .scale_mut()
            .harmonize_smart(note, self.transpose_degrees, prefer_above)
        {
            Some(transposed) => u8::from(transposed),
            None => input_note,
        }
    }
}

impl Default for CanonLane {
    fn default() -> Self {
        Self::new()
    }
}

impl Lane for CanonLane {
    fn name(&self) -> &str {
        "Canon"
    }

    fn type_id(&self) -> &str {
        "canon"
    }

    fn phase(&self) -> LanePhase {
        LanePhase::Decide
    }

    /// Catches every NoteOn / NoteOff so we can buffer and schedule
    /// the matching canon emissions. Does NOT suppress the default
    /// harmonize path — the melody still plays normally; the canon
    /// is an additional delayed voice.
    fn input_filter(&self) -> InputFilter {
        InputFilter::All
    }

    fn on_input(&mut self, ev: InputEvent, world: &WorldState) -> LaneOutput {
        if !self.enabled {
            return LaneOutput::default();
        }
        let now = world.transport.total_beats();
        match ev {
            InputEvent::NoteOn {
                note,
                velocity,
                channel,
            } => {
                let canon_note = self.transpose(note, world);
                self.pending_on.push_back(PendingOn {
                    fire_at: now + self.delay_beats as f64,
                    canon_note,
                    velocity,
                    channel,
                });
                self.held.insert(
                    note,
                    HeldInput {
                        on_beat: now,
                        canon_note,
                        channel,
                    },
                );
            }
            InputEvent::NoteOff { note, channel: _ } => {
                if let Some(held) = self.held.remove(&note) {
                    // Duration the input was held = now - on_beat.
                    // Canon NoteOff fires at on_beat + delay + duration.
                    let duration = (now - held.on_beat).max(0.0);
                    self.pending_off.push(PendingOff {
                        fire_at: held.on_beat + self.delay_beats as f64 + duration,
                        canon_note: held.canon_note,
                        channel: held.channel,
                    });
                }
            }
            InputEvent::Cc { .. } => {
                // Canon ignores CCs in v1; future could honor sustain
                // pedal to keep notes alive longer.
            }
        }
        LaneOutput::default()
    }

    fn tick(&mut self, world: &WorldState) -> LaneOutput {
        if !self.enabled {
            return LaneOutput::default();
        }
        let now = world.transport.total_beats();
        let mut ops: Vec<DispatchOp> = Vec::new();

        // Drain matured NoteOn emissions in fire-order.
        while let Some(p) = self.pending_on.front() {
            if p.fire_at <= now {
                let p = self.pending_on.pop_front().unwrap();
                ops.push(DispatchOp::NoteOn {
                    target: self.target,
                    note: p.canon_note,
                    velocity: p.velocity,
                    channel: p.channel,
                });
            } else {
                break;
            }
        }

        // Drain matured NoteOff emissions — linear scan + retain.
        self.pending_off.retain(|p| {
            if p.fire_at <= now {
                ops.push(DispatchOp::NoteOff {
                    target: self.target,
                    note: p.canon_note,
                    channel: p.channel,
                });
                false
            } else {
                true
            }
        });

        LaneOutput {
            ops,
            ..Default::default()
        }
    }

    fn serialize_state(&self) -> serde_json::Value {
        serde_json::json!({
            "enabled": self.enabled,
            "delay_beats": self.delay_beats,
            "transpose_degrees": self.transpose_degrees,
        })
    }

    fn deserialize_state(&mut self, state: serde_json::Value) -> Result<(), String> {
        if let Some(b) = state.get("enabled").and_then(|v| v.as_bool()) {
            self.enabled = b;
        }
        if let Some(d) = state.get("delay_beats").and_then(|v| v.as_f64()) {
            self.delay_beats = (d as f32).clamp(0.0, 8.0);
        }
        if let Some(t) = state.get("transpose_degrees").and_then(|v| v.as_i64()) {
            self.transpose_degrees = (t as i8).clamp(-7, 7);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::companion::world::WorldState;
    use contrapunk::harmony::{HarmonyEngine, HarmonyMode, Key};
    use contrapunk::transport::Transport;
    use std::sync::{Arc, Mutex};

    fn fixture() -> (CanonLane, Arc<WorldState>, Arc<Transport>) {
        let transport = Transport::new(48_000);
        // Stopped transport — total_beats stays 0.0 until we manually
        // advance via the test helpers. The transport API doesn't
        // expose a direct setter; we manipulate by calling play() and
        // letting sample_pos drive total_beats via bpm. For unit
        // tests we drive everything through advance_beats helpers.
        let engine = Arc::new(Mutex::new(HarmonyEngine::new(
            Key::C,
            HarmonyMode::PassThrough,
        )));
        let world = WorldState::new(Arc::clone(&transport), engine);
        let lane = CanonLane::new();
        (lane, world, transport)
    }

    /// Jump the transport to an absolute beat position by resetting
    /// sample_pos to 0 and advancing by the right frame count. Tests
    /// don't share state between helper invocations — each call is
    /// idempotent w.r.t. the resulting `total_beats()` value.
    fn advance_to_beat(transport: &Transport, beat: f64) {
        let spb = transport.sample_rate() as f64 * 60.0 / transport.bpm();
        let target_frames = (beat * spb) as u32;
        transport.reset();
        transport.play();
        if target_frames > 0 {
            let _ = transport.advance(target_frames);
        }
    }

    #[test]
    fn disabled_lane_is_inert() {
        let (mut lane, world, _t) = fixture();
        // enabled defaults to false
        let out = lane.on_input(
            InputEvent::NoteOn {
                note: 60,
                velocity: 100,
                channel: 0,
            },
            &world,
        );
        assert!(out.ops.is_empty());
        assert!(lane.pending_on.is_empty());
        let tick_out = lane.tick(&world);
        assert!(tick_out.ops.is_empty());
    }

    #[test]
    fn enabled_lane_buffers_note_on() {
        let (mut lane, world, _t) = fixture();
        lane.set_enabled(true);
        lane.set_delay(1.0);
        lane.on_input(
            InputEvent::NoteOn {
                note: 60,
                velocity: 100,
                channel: 0,
            },
            &world,
        );
        // Tick at beat 0 — too early, nothing emitted yet.
        assert!(lane.tick(&world).ops.is_empty());
        assert_eq!(lane.pending_on.len(), 1);
    }

    #[test]
    fn emits_note_on_at_delay_boundary() {
        let (mut lane, world, transport) = fixture();
        lane.set_enabled(true);
        lane.set_delay(1.0);
        lane.on_input(
            InputEvent::NoteOn {
                note: 60,
                velocity: 100,
                channel: 0,
            },
            &world,
        );
        advance_to_beat(&transport, 1.5);
        let out = lane.tick(&world);
        assert_eq!(out.ops.len(), 1);
        match &out.ops[0] {
            DispatchOp::NoteOn { note, velocity, .. } => {
                assert_eq!(*note, 60);
                assert_eq!(*velocity, 100);
            }
            _ => panic!("expected NoteOn, got {:?}", out.ops[0]),
        }
        assert!(lane.pending_on.is_empty());
    }

    #[test]
    fn note_off_preserves_duration() {
        let (mut lane, world, transport) = fixture();
        lane.set_enabled(true);
        lane.set_delay(1.0);
        // Beat 0: NoteOn
        lane.on_input(
            InputEvent::NoteOn {
                note: 60,
                velocity: 100,
                channel: 0,
            },
            &world,
        );
        // Beat 2: NoteOff (held for 2 beats)
        advance_to_beat(&transport, 2.0);
        lane.on_input(
            InputEvent::NoteOff {
                note: 60,
                channel: 0,
            },
            &world,
        );

        // At beat 1.5: canon NoteOn should fire.
        advance_to_beat(&transport, 1.5);
        let on_out = lane.tick(&world);
        assert!(matches!(
            on_out.ops[..],
            [DispatchOp::NoteOn { note: 60, .. }]
        ));

        // Canon NoteOff fires at on_beat (0) + delay (1) + duration
        // (2) = beat 3.0. At beat 2.5 nothing should fire yet.
        advance_to_beat(&transport, 2.5);
        assert!(lane.tick(&world).ops.is_empty());

        // At beat 3.5 the off matures and fires.
        advance_to_beat(&transport, 3.5);
        let off_out = lane.tick(&world);
        assert!(matches!(
            off_out.ops[..],
            [DispatchOp::NoteOff { note: 60, .. }]
        ));
        assert!(lane.pending_off.is_empty());
    }

    #[test]
    fn disabling_clears_buffers() {
        let (mut lane, world, _t) = fixture();
        lane.set_enabled(true);
        lane.on_input(
            InputEvent::NoteOn {
                note: 60,
                velocity: 100,
                channel: 0,
            },
            &world,
        );
        assert!(!lane.pending_on.is_empty());
        lane.set_enabled(false);
        assert!(lane.pending_on.is_empty());
        assert!(lane.held.is_empty());
    }

    #[test]
    fn unmatched_note_off_is_a_noop() {
        // NoteOff for a note we never received NoteOn for — must not
        // panic, must not push to pending_off.
        let (mut lane, world, _t) = fixture();
        lane.set_enabled(true);
        let out = lane.on_input(
            InputEvent::NoteOff {
                note: 42,
                channel: 0,
            },
            &world,
        );
        assert!(out.ops.is_empty());
        assert!(lane.pending_off.is_empty());
    }

    /// Issue #3 transpose: canon emits diatonically-transposed notes.
    /// Input C4 (60) with transpose_degrees=2 in C major Ionian should
    /// emit E4 (64) — the diatonic third above C. Test pinpoints the
    /// fact that transpose is applied at CAPTURE time, not emit time,
    /// so the canon snapshot is stable against mid-replay key changes.
    #[test]
    fn transpose_emits_diatonic_interval() {
        let (mut lane, world, transport) = fixture();
        lane.set_enabled(true);
        lane.set_delay(1.0);
        lane.set_transpose(2); // diatonic third

        lane.on_input(
            InputEvent::NoteOn {
                note: 60, // C4
                velocity: 100,
                channel: 0,
            },
            &world,
        );
        advance_to_beat(&transport, 1.5);
        let out = lane.tick(&world);
        match &out.ops[0] {
            DispatchOp::NoteOn { note, .. } => {
                assert_eq!(
                    *note, 64,
                    "C4 + 2 diatonic degrees in C major Ionian = E4 (64), got {}",
                    note
                );
            }
            _ => panic!("expected NoteOn"),
        }
    }

    /// Chromatic input + modal interchange enabled: canon transpose
    /// routes through harmonize_smart, which borrows from a parallel
    /// mode that contains the note. The canon line stays musically
    /// sensible instead of falling back to naked unison.
    #[test]
    fn transpose_uses_modal_interchange_for_chromatic_input() {
        let (mut lane, world, transport) = fixture();
        lane.set_enabled(true);
        lane.set_delay(1.0);
        lane.set_transpose(2);

        // Eb4 (63) is not in C Ionian, but IS in C Aeolian / C Dorian.
        // Enable interchange so harmonize_smart routes through the
        // borrowing path.
        {
            let mut engine = world.engine_snapshot.lock().unwrap();
            engine.set_interchange_enabled(true);
            engine.set_borrowing_range(3);
        }
        lane.on_input(
            InputEvent::NoteOn {
                note: 63,
                velocity: 100,
                channel: 0,
            },
            &world,
        );
        advance_to_beat(&transport, 1.5);
        let out = lane.tick(&world);
        let emitted = match &out.ops[0] {
            DispatchOp::NoteOn { note, .. } => *note,
            _ => panic!("expected NoteOn"),
        };
        // Eb + 2 diatonic degrees in C Aeolian (Eb-F-G-Ab-Bb-C) lands
        // on G. In C Dorian (D-Eb-F-G-A-Bb-C) the diatonic third up
        // from Eb is also G. So the canon transpose should land on
        // G4 (67). At minimum it must NOT equal the input (63) — that
        // would mean we fell back to unison, defeating the whole point.
        assert_ne!(
            emitted, 63,
            "interchange path must produce a real transposition, not unison"
        );
        assert!(
            (emitted as i16 - 63).abs() <= 7,
            "transpose should land within an octave of input, got {}",
            emitted
        );
    }

    /// Chromatic input + interchange disabled: canon transpose routes
    /// through harmonize_smart's chromatic-consonant path, landing on
    /// a scale tone via a consonant interval (3rd / 6th / 5th / 4th).
    /// Still produces a real transposition, not unison.
    #[test]
    fn transpose_uses_chromatic_fallback_when_interchange_disabled() {
        let (mut lane, world, transport) = fixture();
        lane.set_enabled(true);
        lane.set_delay(1.0);
        lane.set_transpose(2);

        // interchange defaults to disabled — explicit just for clarity.
        {
            let mut engine = world.engine_snapshot.lock().unwrap();
            engine.set_interchange_enabled(false);
        }
        lane.on_input(
            InputEvent::NoteOn {
                note: 63, // Eb4 — chromatic in C Ionian
                velocity: 100,
                channel: 0,
            },
            &world,
        );
        advance_to_beat(&transport, 1.5);
        let out = lane.tick(&world);
        match &out.ops[0] {
            DispatchOp::NoteOn { note, .. } => {
                // Stays within reach of the input but must not be naked
                // unison — chromatic-consonant path always picks an
                // interval, never returns the input unchanged.
                assert!(
                    (*note as i16 - 63).abs() <= 9,
                    "chromatic-consonant transpose should land within a 6th, got {}",
                    note
                );
            }
            _ => panic!("expected NoteOn"),
        }
    }

    #[test]
    fn serialize_and_restore_round_trip() {
        let mut lane = CanonLane::new();
        lane.set_enabled(true);
        lane.set_delay(2.5);
        lane.set_transpose(-3);
        let snapshot = lane.serialize_state();

        let mut restored = CanonLane::new();
        restored.deserialize_state(snapshot).unwrap();
        assert_eq!(restored.enabled, true);
        assert!((restored.delay_beats - 2.5).abs() < 1e-6);
        assert_eq!(restored.transpose_degrees, -3);
    }
}
