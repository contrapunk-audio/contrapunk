//! Lane abstraction — the unit of companion behavior.
//!
//! Six of the nine jam-pipeline weeks fit cleanly as `Lane` impls
//! (looper, arp, drone, pad, beat machine, motif transposer). Each
//! Lane belongs to one of three phases — Sense, Mutate, Decide —
//! which the orchestrator runs in strict order so every Lane sees a
//! coherent world. Sense writes `WorldState`. Mutate writes
//! `HarmonyEngine`. Decide emits dispatch ops only.
//!
//! Phase 1.4 ships the abstraction with no concrete impls. Phase 2
//! adds `LooperLane`, the first concrete Decide-phase Lane.
//!
//! See `.planning/jam-features-2026/01-companion-architecture.md`
//! § "Lane abstraction" for the full contract.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

use contrapunk_harmony::{HarmonyMode, Key, ScaleMode, VoiceLeadingStyle};

use super::world::{DetectedChord, WorldState};
use super::DispatchOp;

/// What happens to pending lane emissions when the player releases
/// the input note that seeded them.
///
/// Lane emissions are buffered as `PendingOn` entries with a `fire_at`
/// beat. The naive design lets them all fire on schedule even after
/// the player released — which is right for canons (delayed echoes
/// outlive the subject by design) but wrong for performative
/// scenarios where the user expects "lift = stop." Per-lane and per-
/// voice `Option<HoldMode>` overrides let the user pick per slot;
/// the orchestrator-level `Companion.hold_mode` is the default.
///
/// Resolution: `voice.hold_mode || lane.hold_mode || companion.hold_mode`.
///
/// Event-driven: applied at NoteOff time, NOT polled. Each lane's
/// `on_input(NoteOff)` walks its `pending_on` deque, filters by the
/// effective mode + the current `transport.total_beats()`, and emits
/// matching NoteOffs / cancels orphan NoteOns in one pass.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum HoldMode {
    /// Kill every pending emission seeded by the released input note.
    /// Already-sounded voices still go through their natural release
    /// envelope (the synth handles that). Use for performative "lift
    /// = stop" feel.
    Cancel,
    /// Let pending emissions whose `fire_at` lies within `tail_beats`
    /// of the current beat fire normally; cancel anything scheduled
    /// further out. Use for "echoes near in time still ring, far-
    /// future canon entries cancel." `tail_beats = 0.0` is equivalent
    /// to `Cancel`.
    NearFuture { tail_beats: f64 },
    /// Let pending emissions within the CURRENT phrase fire (where
    /// "phrase" is anchored by `CanonLane.sequence_anchor` plus the
    /// transport's beats-per-bar). Cancels anything beyond the next
    /// phrase boundary. Use for "let this musical idea finish, then
    /// stop."
    PhraseEnd,
    /// No cancellation. Every pending emission fires on its scheduled
    /// beat. This is the pre-v1.2 behavior; keeps canon-as-canon
    /// semantics intact (V_K can outlive the player's input by
    /// `delay_beats * time_ratio` beats).
    Forever,
}

impl Default for HoldMode {
    fn default() -> Self {
        // Tail of 1.0 beats — near-term companion echoes ring out
        // after a release, multi-bar augmentation/canon emissions
        // cancel. Most musical of the four for a typical user.
        HoldMode::NearFuture { tail_beats: 1.0 }
    }
}

/// Serialize a HoldMode to a stable JSON object the Tauri / WASM /
/// website state-restore paths can round-trip. Shape:
///   {"kind":"cancel"}
///   {"kind":"near_future","tail_beats":1.0}
///   {"kind":"phrase_end"}
///   {"kind":"forever"}
pub fn hold_mode_to_json(m: HoldMode) -> serde_json::Value {
    match m {
        HoldMode::Cancel => serde_json::json!({ "kind": "cancel" }),
        HoldMode::NearFuture { tail_beats } => {
            serde_json::json!({ "kind": "near_future", "tail_beats": tail_beats })
        }
        HoldMode::PhraseEnd => serde_json::json!({ "kind": "phrase_end" }),
        HoldMode::Forever => serde_json::json!({ "kind": "forever" }),
    }
}

/// Inverse of `hold_mode_to_json`. Unknown / malformed shapes return
/// `None` so callers can fall back to the lane's existing mode rather
/// than a destructive reset.
pub fn hold_mode_from_json(v: &serde_json::Value) -> Option<HoldMode> {
    let kind = v.get("kind").and_then(|s| s.as_str())?;
    match kind {
        "cancel" => Some(HoldMode::Cancel),
        "near_future" => {
            let tail_beats = v
                .get("tail_beats")
                .and_then(|n| n.as_f64())
                .unwrap_or(1.0)
                .clamp(0.0, 32.0);
            Some(HoldMode::NearFuture { tail_beats })
        }
        "phrase_end" => Some(HoldMode::PhraseEnd),
        "forever" => Some(HoldMode::Forever),
        _ => None,
    }
}

/// Which orchestration phase a Lane participates in.
///
/// The orchestrator runs all `Sense` lanes first, then all `Mutate`,
/// then all `Decide`. Within a phase, declaration order in
/// `Companion::lanes` decides ordering — primarily relevant for Sense
/// (auto-key writes the engine before chord-detect reads sounding
/// voices) and Decide (looper feeds drone, etc.).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum LanePhase {
    /// Updates `WorldState` only (auto-key, chord-detect).
    Sense,
    /// Mutates `HarmonyEngine` state (chord-seq).
    Mutate,
    /// Emits dispatch ops only (looper, arp, drone, pad, beat machine).
    Decide,
}

/// Which input events a Lane wants to handle.
///
/// The orchestrator's `on_input` consults `input_filter` per Lane and
/// only calls `on_input` for matching events. Inputs that no Lane
/// claims (or no Lane suppresses) fall through to the default
/// harmonize path. This is what enables split-keyboard / live-channel
/// routing.
pub enum InputFilter {
    /// Tick-only Lane (Drone, Pad). Never claims input.
    None,
    /// Grabs everything.
    All,
    /// Inclusive MIDI note range — used for split-keyboard mode where
    /// a Lane only owns a register (e.g. chord region).
    NoteRange { min: u8, max: u8 },
    /// MIDI channel filter — used for live-channel routing where a
    /// hardware controller layer lands on a dedicated channel.
    Channel(u8),
    /// Caller-supplied predicate. Boxed because trait objects need a
    /// concrete size; `Send + Sync` because Lanes run on the router
    /// thread but may be inspected from the main thread.
    Predicate(Box<dyn Fn(&InputEvent) -> bool + Send + Sync>),
}

impl Default for InputFilter {
    fn default() -> Self {
        InputFilter::None
    }
}

impl InputFilter {
    /// Returns true if this filter matches the given event. The
    /// orchestrator calls this before invoking `Lane::on_input`.
    pub fn matches(&self, ev: &InputEvent) -> bool {
        let (note_opt, channel_opt) = match ev {
            InputEvent::NoteOn { note, channel, .. } => (Some(*note), Some(*channel)),
            InputEvent::NoteOff { note, channel } => (Some(*note), Some(*channel)),
            InputEvent::Cc { channel, .. } => (None, Some(*channel)),
        };
        match self {
            InputFilter::None => false,
            InputFilter::All => true,
            InputFilter::NoteRange { min, max } => {
                note_opt.map(|n| n >= *min && n <= *max).unwrap_or(false)
            }
            InputFilter::Channel(c) => channel_opt.map(|ch| ch == *c).unwrap_or(false),
            InputFilter::Predicate(f) => f(ev),
        }
    }
}

/// A single input event delivered to `Companion::on_input`. Sustain
/// pedal + other CCs flow through `Cc` so Lanes (e.g. ArpeggiatorLane)
/// can intercept pedal-down to keep held chords latched.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InputEvent {
    NoteOn { note: u8, velocity: u8, channel: u8 },
    NoteOff { note: u8, channel: u8 },
    Cc { number: u8, value: u8, channel: u8 },
}

/// One mutation a Mutate-phase Lane wants applied to the engine. The
/// orchestrator applies these immediately under the engine mutex
/// before Decide phase runs, so Decide lanes see post-mutate state.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EngineMutation {
    SetKey(Key),
    SetMode(HarmonyMode),
    SetScale(ScaleMode),
    SetVoiceLeading(VoiceLeadingStyle),
}

/// One write a Sense-phase Lane wants applied to `WorldState`. Kept
/// as an enum (rather than letting Lanes mutate `WorldState` directly)
/// so the orchestrator owns the apply order — guarantees Mutate lanes
/// see a fully-updated world.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WorldWrite {
    UpdateChord(DetectedChord),
    UpdateDetectedKey(Key),
    UpdateDetectedMode(HarmonyMode),
}

/// What a single Lane invocation produced.
///
/// `ops` are emitted by Decide-phase lanes. `engine_mutations` by
/// Mutate-phase lanes. `world_writes` by Sense-phase lanes.
/// `suppress_default` is meaningful only on `on_input` results — it
/// tells the input pipeline to skip default `harmonize_note_on`.
///
/// Lanes generally fill exactly one of `ops` / `engine_mutations` /
/// `world_writes`. The struct stays one-shape so the orchestrator
/// pipeline doesn't branch on phase mid-iteration.
#[derive(Default)]
pub struct LaneOutput {
    pub ops: Vec<DispatchOp>,
    pub engine_mutations: Vec<EngineMutation>,
    pub world_writes: Vec<WorldWrite>,
    pub suppress_default: bool,
}

/// The companion's behavioral unit. See module doc.
pub trait Lane: Send + Sync {
    /// Display name for UI / logs. Stable across instances of the
    /// same Lane *type* (the user-facing label). For rig restoration,
    /// see `type_id`.
    fn name(&self) -> &str;

    /// Stable identifier for rig serialization. Distinct Lanes of the
    /// same type may share `type_id` ("looper") but have different
    /// `name`s ("Loop A", "Loop B"). The Lane registry uses this.
    ///
    /// Returns `&'static str` so the audio-adjacent dispatch path can
    /// tag every emitted op without allocating per-tick. Every shipped
    /// impl already returns a string literal — tightening the trait
    /// makes the no-alloc contract enforced.
    fn type_id(&self) -> &'static str;

    /// Which orchestration phase this Lane runs in.
    fn phase(&self) -> LanePhase;

    /// Which input events this Lane wants. Default `None` (tick-only).
    fn input_filter(&self) -> InputFilter {
        InputFilter::None
    }

    /// Called once per router-loop iteration with the current world.
    /// Sense lanes write to `world_writes`; Mutate lanes to
    /// `engine_mutations`; Decide lanes to `ops`.
    fn tick(&mut self, world: &WorldState) -> LaneOutput;

    /// Runtime voice slot paired with each emitted operation. Most lanes have
    /// one voice; multi-voice lanes override this to preserve UI/settings identity.
    fn tick_slotted(&mut self, world: &WorldState) -> Vec<(u8, DispatchOp)> {
        self.tick(world).ops.into_iter().map(|op| (0, op)).collect()
    }

    /// Called when an input matched by `input_filter()` arrives.
    /// Returning `suppress_default = true` skips the default harmonize
    /// path for this event (used by ArpeggiatorLane and any future
    /// Lane that fully owns the chord register).
    fn on_input(&mut self, _ev: InputEvent, _world: &WorldState) -> LaneOutput {
        LaneOutput::default()
    }

    /// Clear runtime queues and held-note state without changing the
    /// lane's configuration or enabled state. Panic, transport stop,
    /// and input-mode transitions call this to prevent delayed notes
    /// from reappearing after an all-notes-off.
    fn reset_runtime(&mut self) {}

    /// Snapshot this Lane's state for rig persistence. Default empty
    /// (stateless Lane). Concrete impls that own buffers / counters
    /// override this.
    fn serialize_state(&self) -> serde_json::Value {
        serde_json::Value::Null
    }

    /// Restore Lane state. Default no-op. Returning `Err` aborts the
    /// rig restore for this Lane; the orchestrator surfaces the error
    /// rather than silently dropping it.
    fn deserialize_state(&mut self, _state: serde_json::Value) -> Result<(), String> {
        Ok(())
    }
}

/// Persisted snapshot of one Lane's state, addressed by `type_id`.
/// The companion's rig writer collects these into `CompanionState`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LaneState {
    pub type_id: String,
    pub state: serde_json::Value,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn input_filter_none_matches_nothing() {
        let f = InputFilter::None;
        assert!(!f.matches(&InputEvent::NoteOn {
            note: 60,
            velocity: 100,
            channel: 0
        }));
    }

    #[test]
    fn input_filter_all_matches_everything() {
        let f = InputFilter::All;
        assert!(f.matches(&InputEvent::NoteOn {
            note: 60,
            velocity: 100,
            channel: 0
        }));
        assert!(f.matches(&InputEvent::NoteOff {
            note: 60,
            channel: 0
        }));
        assert!(f.matches(&InputEvent::Cc {
            number: 64,
            value: 127,
            channel: 0
        }));
    }

    #[test]
    fn input_filter_note_range_inclusive_bounds() {
        let f = InputFilter::NoteRange { min: 36, max: 47 };
        assert!(f.matches(&InputEvent::NoteOn {
            note: 36,
            velocity: 100,
            channel: 0
        }));
        assert!(f.matches(&InputEvent::NoteOn {
            note: 47,
            velocity: 100,
            channel: 0
        }));
        assert!(!f.matches(&InputEvent::NoteOn {
            note: 48,
            velocity: 100,
            channel: 0
        }));
        assert!(!f.matches(&InputEvent::NoteOn {
            note: 35,
            velocity: 100,
            channel: 0
        }));
        // Cc has no note → no match for note-range filter.
        assert!(!f.matches(&InputEvent::Cc {
            number: 64,
            value: 127,
            channel: 0
        }));
    }

    #[test]
    fn input_filter_channel_targets_one() {
        let f = InputFilter::Channel(2);
        assert!(f.matches(&InputEvent::NoteOn {
            note: 60,
            velocity: 100,
            channel: 2
        }));
        assert!(!f.matches(&InputEvent::NoteOn {
            note: 60,
            velocity: 100,
            channel: 1
        }));
        // Cc carries its own channel.
        assert!(f.matches(&InputEvent::Cc {
            number: 64,
            value: 127,
            channel: 2
        }));
    }

    #[test]
    fn input_filter_predicate_invoked() {
        let f = InputFilter::Predicate(Box::new(|ev| {
            matches!(ev, InputEvent::Cc { number: 64, .. })
        }));
        assert!(f.matches(&InputEvent::Cc {
            number: 64,
            value: 127,
            channel: 0
        }));
        assert!(!f.matches(&InputEvent::Cc {
            number: 7,
            value: 127,
            channel: 0
        }));
        assert!(!f.matches(&InputEvent::NoteOn {
            note: 60,
            velocity: 100,
            channel: 0
        }));
    }

    #[test]
    fn lane_output_default_is_empty() {
        let out = LaneOutput::default();
        assert!(out.ops.is_empty());
        assert!(out.engine_mutations.is_empty());
        assert!(out.world_writes.is_empty());
        assert!(!out.suppress_default);
    }
}
