//! Canon Lane — N delayed-entry voices replaying the player's melody.
//!
//! Closes #3. The first concrete `Decide`-phase Lane. Each input note
//! captured by `on_input` is buffered with its trigger beat position
//! and one pending emission is scheduled PER configured canon voice.
//! On each `tick`, the lane drains entries whose delay has elapsed
//! against the current transport beat. Matching NoteOff events follow
//! the same delay so each canon voice preserves the original duration.
//!
//! Per `.planning/research/group-a-core-harmony.md:108-153`:
//! - Beat-clock-driven (NOT user-input-driven). The Lane fires on
//!   `tick`, called every router iteration with the live transport.
//!   This is how the canon plays *between* user note-ons rather than
//!   clumping all delayed emissions on the next trigger.
//! - Per-voice delay buffer is bounded (few seconds of held content)
//!   so no memory concern.
//! - Reuses the existing synthetic-beat / transport infrastructure.
//! - "Multi-voice canons (3-voice canon at +2, +4)? Trivial extension
//!   once 1-voice works." Now wired.
//!
//! Each `CanonVoice` is independently configurable: own delay (in
//! beats) and own diatonic transpose. Transpose routes through
//! `Scale::harmonize_smart`, so out-of-scale input uses the engine's
//! modal-interchange logic when the user has it enabled, and falls
//! back to consonant chromatic intervals otherwise — never naked
//! unison.

#![allow(dead_code)]

use std::collections::{HashMap, VecDeque};

use wmidi::Note;

use super::lane::{InputEvent, InputFilter, Lane, LaneOutput, LanePhase};
use super::world::WorldState;
use super::DispatchOp;
use crate::state::VoiceOutputTarget;

/// Configuration for one canon voice. The lane can hold any number
/// of these; each one independently delays and transposes the
/// player's input.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CanonVoice {
    /// Beats to wait between the player's NoteOn and this voice's
    /// NoteOn. Clamped to [0.0, 8.0].
    pub delay_beats: f32,
    /// Diatonic transpose for this voice's emissions. Clamped to
    /// [-7, 7] degrees.
    pub transpose_degrees: i8,
}

impl CanonVoice {
    pub fn new(delay_beats: f32, transpose_degrees: i8) -> Self {
        Self {
            delay_beats: delay_beats.clamp(0.0, 8.0),
            transpose_degrees: transpose_degrees.clamp(-7, 7),
        }
    }
}

impl Default for CanonVoice {
    fn default() -> Self {
        Self {
            delay_beats: 1.0,
            transpose_degrees: 0,
        }
    }
}

/// One pending NoteOn that will fire at `fire_at` beats (transport
/// total-beats coordinate).
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

/// One canon voice's fire record for a single held input — captures
/// the voice's index (to look up its delay on NoteOff) and the
/// already-transposed pitch (so the matching NoteOff fires the same
/// note).
#[derive(Clone, Copy, Debug, PartialEq)]
struct HeldVoiceFire {
    voice_idx: usize,
    canon_note: u8,
    channel: u8,
}

/// Tracked per held input. `on_beat` is shared across all canon
/// voices for that input; `voices` records what each voice scheduled.
#[derive(Clone, Debug, PartialEq)]
struct HeldEntry {
    on_beat: f64,
    voices: Vec<HeldVoiceFire>,
}

pub struct CanonLane {
    /// Master toggle for this lane independent of the Companion's
    /// master enabled flag. When false, `on_input` and `tick` are
    /// no-ops. Default false: pre-existing user behavior is
    /// preserved when the lane is registered.
    pub enabled: bool,

    /// Configured canon voices. Empty = no canon emissions even when
    /// `enabled` is true. Default contains one voice at delay=1.0,
    /// transpose=0 (the v1 single-voice behavior).
    pub voices: Vec<CanonVoice>,

    /// Output target. Canon emits to its own slot so the dispatcher
    /// can mute / route it independently of the main harmony voices.
    /// Default `Synth` so users hear it without needing MIDI routing.
    pub target: VoiceOutputTarget,

    /// Pending NoteOn emissions, sorted by fire_at ascending (push
    /// order is monotonic in `transport.total_beats()` but voices
    /// have different delays, so we sort by fire_at on each push
    /// to keep the front of the deque the next emission).
    pending_on: VecDeque<PendingOn>,

    /// Pending NoteOff emissions, scanned linearly each tick.
    pending_off: Vec<PendingOff>,

    /// Currently-held inputs. One entry per *input* MIDI note;
    /// each entry's `voices` vec records the per-voice fire info
    /// the lane will need at NoteOff time.
    held: HashMap<u8, HeldEntry>,
}

impl CanonLane {
    pub fn new() -> Self {
        Self {
            enabled: false,
            voices: vec![CanonVoice::default()],
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

    /// Replace the entire voices vector. Capped at 8 entries so a
    /// runaway UI / API caller can't allocate unbounded pending
    /// queues per input note. Empty vector means "no canon
    /// emissions" — the lane stays enabled but produces nothing.
    pub fn set_voices(&mut self, voices: Vec<CanonVoice>) {
        let clamped: Vec<CanonVoice> = voices
            .into_iter()
            .take(8)
            .map(|v| CanonVoice::new(v.delay_beats, v.transpose_degrees))
            .collect();
        self.voices = clamped;
        // Drop any in-flight emissions targeting voices that may no
        // longer exist — simplest correctness path. Notes that were
        // already dispatched are out the door already; this clears
        // the pending pipeline.
        self.pending_on.clear();
        self.pending_off.clear();
        self.held.clear();
    }

    pub fn voices(&self) -> &[CanonVoice] {
        &self.voices
    }

    // === Backward-compat single-voice setters — operate on voices[0]. ===
    //
    // Kept so the existing `canon_set_delay` / `canon_set_transpose`
    // Tauri commands keep working. UI for multi-voice uses
    // `canon_set_voices` directly.

    pub fn set_delay(&mut self, beats: f32) {
        if self.voices.is_empty() {
            self.voices.push(CanonVoice::default());
        }
        self.voices[0].delay_beats = beats.clamp(0.0, 8.0);
    }

    pub fn set_transpose(&mut self, degrees: i8) {
        if self.voices.is_empty() {
            self.voices.push(CanonVoice::default());
        }
        self.voices[0].transpose_degrees = degrees.clamp(-7, 7);
    }

    /// Apply diatonic transpose against the engine's current scale,
    /// routing through `Scale::harmonize_smart` so out-of-scale input
    /// lands somewhere musically defensible. See module docstring.
    fn transpose(&self, input_note: u8, transpose_degrees: i8, world: &WorldState) -> u8 {
        if transpose_degrees == 0 {
            return input_note;
        }
        let Ok(mut engine) = world.engine_snapshot.lock() else {
            return input_note;
        };
        let Ok(note) = Note::try_from(input_note) else {
            return input_note;
        };
        let prefer_above = transpose_degrees > 0;
        match engine
            .scale_mut()
            .harmonize_smart(note, transpose_degrees, prefer_above)
        {
            Some(transposed) => u8::from(transposed),
            None => input_note,
        }
    }

    /// Insert a PendingOn into the deque in fire-order. Linear scan
    /// over the deque from the back — typically O(1) because voices
    /// with longer delays arrive in fire-order naturally, but
    /// out-of-order voices (e.g. v2 with shorter delay than v1)
    /// require a quick scan. Deque size stays small (~N voices ×
    /// active held inputs).
    fn insert_sorted(&mut self, p: PendingOn) {
        let pos = self
            .pending_on
            .iter()
            .rposition(|existing| existing.fire_at <= p.fire_at);
        match pos {
            Some(idx) => self.pending_on.insert(idx + 1, p),
            None => self.pending_on.push_front(p),
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

    fn input_filter(&self) -> InputFilter {
        InputFilter::All
    }

    fn on_input(&mut self, ev: InputEvent, world: &WorldState) -> LaneOutput {
        if !self.enabled || self.voices.is_empty() {
            return LaneOutput::default();
        }
        let now = world.transport.total_beats();
        match ev {
            InputEvent::NoteOn {
                note,
                velocity,
                channel,
            } => {
                let voices_snapshot: Vec<(usize, CanonVoice)> =
                    self.voices.iter().copied().enumerate().collect();
                let mut held_voices: Vec<HeldVoiceFire> = Vec::with_capacity(voices_snapshot.len());
                for (voice_idx, voice) in voices_snapshot {
                    let canon_note = self.transpose(note, voice.transpose_degrees, world);
                    self.insert_sorted(PendingOn {
                        fire_at: now + voice.delay_beats as f64,
                        canon_note,
                        velocity,
                        channel,
                    });
                    held_voices.push(HeldVoiceFire {
                        voice_idx,
                        canon_note,
                        channel,
                    });
                }
                self.held.insert(
                    note,
                    HeldEntry {
                        on_beat: now,
                        voices: held_voices,
                    },
                );
            }
            InputEvent::NoteOff { note, channel: _ } => {
                if let Some(held) = self.held.remove(&note) {
                    let duration = (now - held.on_beat).max(0.0);
                    for fire in held.voices {
                        let Some(voice) = self.voices.get(fire.voice_idx) else {
                            continue; // voice removed mid-flight — drop the off
                        };
                        self.pending_off.push(PendingOff {
                            fire_at: held.on_beat + voice.delay_beats as f64 + duration,
                            canon_note: fire.canon_note,
                            channel: fire.channel,
                        });
                    }
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
        let voices_json: Vec<serde_json::Value> = self
            .voices
            .iter()
            .map(|v| {
                serde_json::json!({
                    "delay_beats": v.delay_beats,
                    "transpose_degrees": v.transpose_degrees,
                })
            })
            .collect();
        serde_json::json!({
            "enabled": self.enabled,
            "voices": voices_json,
            // Back-compat scalar fields — readers that haven't been
            // updated still see voice 0's config under the old keys.
            "delay_beats": self.voices.first().map(|v| v.delay_beats).unwrap_or(1.0),
            "transpose_degrees": self.voices.first().map(|v| v.transpose_degrees).unwrap_or(0),
        })
    }

    fn deserialize_state(&mut self, state: serde_json::Value) -> Result<(), String> {
        if let Some(b) = state.get("enabled").and_then(|v| v.as_bool()) {
            self.enabled = b;
        }
        // Prefer the new `voices` array. Fall back to the old single-
        // voice keys for backward compatibility with older snapshots.
        if let Some(arr) = state.get("voices").and_then(|v| v.as_array()) {
            let voices: Vec<CanonVoice> = arr
                .iter()
                .filter_map(|item| {
                    let delay = item.get("delay_beats").and_then(|v| v.as_f64())? as f32;
                    let trans = item.get("transpose_degrees").and_then(|v| v.as_i64())? as i8;
                    Some(CanonVoice::new(delay, trans))
                })
                .take(8)
                .collect();
            if !voices.is_empty() {
                self.voices = voices;
            }
        } else {
            if let Some(d) = state.get("delay_beats").and_then(|v| v.as_f64()) {
                self.set_delay(d as f32);
            }
            if let Some(t) = state.get("transpose_degrees").and_then(|v| v.as_i64()) {
                self.set_transpose(t as i8);
            }
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
    use std::sync::Arc;

    fn fixture() -> (CanonLane, Arc<WorldState>, Arc<Transport>) {
        let transport = Transport::new(48_000);
        let engine = Arc::new(std::sync::Mutex::new(HarmonyEngine::new(
            Key::C,
            HarmonyMode::PassThrough,
        )));
        let world = WorldState::new(Arc::clone(&transport), engine);
        let lane = CanonLane::new();
        (lane, world, transport)
    }

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
        assert!(lane.tick(&world).ops.is_empty());
    }

    #[test]
    fn single_voice_default_buffers_note_on() {
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
        assert!(lane.tick(&world).ops.is_empty());
        assert_eq!(lane.pending_on.len(), 1);
    }

    #[test]
    fn single_voice_emits_at_delay_boundary() {
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
            DispatchOp::NoteOn { note, .. } => assert_eq!(*note, 60),
            _ => panic!("expected NoteOn"),
        }
    }

    /// Multi-voice canon: two voices at different delays emit two
    /// separate canon notes at the right beats. This is the core
    /// proof that the lane supports the canon-of-canons / fugue-
    /// subject-answer pattern.
    #[test]
    fn multi_voice_canon_emits_per_voice() {
        let (mut lane, world, transport) = fixture();
        lane.set_enabled(true);
        lane.set_voices(vec![CanonVoice::new(1.0, 0), CanonVoice::new(2.0, 0)]);
        lane.on_input(
            InputEvent::NoteOn {
                note: 60,
                velocity: 100,
                channel: 0,
            },
            &world,
        );

        // At beat 0.5: nothing emitted yet (both voices still in
        // future).
        advance_to_beat(&transport, 0.5);
        assert!(lane.tick(&world).ops.is_empty());

        // At beat 1.5: voice 1 (delay=1.0) has matured; voice 2 has
        // not. Expect exactly 1 NoteOn.
        advance_to_beat(&transport, 1.5);
        let out_v1 = lane.tick(&world);
        assert_eq!(out_v1.ops.len(), 1, "voice 1 should have fired alone");

        // At beat 2.5: voice 2 (delay=2.0) matures. Expect 1 more
        // NoteOn.
        advance_to_beat(&transport, 2.5);
        let out_v2 = lane.tick(&world);
        assert_eq!(out_v2.ops.len(), 1, "voice 2 should have fired");
    }

    /// Multi-voice with different transposes: 3 voices at (delay, transpose)
    /// = (1, 0), (1, 2), (1, 4). On a single NoteOn the canon should emit
    /// 3 simultaneous notes at the delay boundary, forming a triad.
    #[test]
    fn multi_voice_canon_triad_from_single_input() {
        let (mut lane, world, transport) = fixture();
        lane.set_enabled(true);
        lane.set_voices(vec![
            CanonVoice::new(1.0, 0), // unison
            CanonVoice::new(1.0, 2), // third
            CanonVoice::new(1.0, 4), // fifth
        ]);
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
        assert_eq!(out.ops.len(), 3, "3-voice canon should emit 3 notes");
        let notes: Vec<u8> = out
            .ops
            .iter()
            .filter_map(|op| match op {
                DispatchOp::NoteOn { note, .. } => Some(*note),
                _ => None,
            })
            .collect();
        // C major triad notes: C(60), E(64), G(67).
        assert!(notes.contains(&60), "should include unison C: {:?}", notes);
        assert!(
            notes.contains(&64),
            "should include diatonic third E: {:?}",
            notes
        );
        assert!(
            notes.contains(&67),
            "should include diatonic fifth G: {:?}",
            notes
        );
    }

    #[test]
    fn note_off_preserves_per_voice_duration() {
        let (mut lane, world, transport) = fixture();
        lane.set_enabled(true);
        lane.set_voices(vec![CanonVoice::new(1.0, 0), CanonVoice::new(2.0, 0)]);
        // Beat 0: NoteOn
        lane.on_input(
            InputEvent::NoteOn {
                note: 60,
                velocity: 100,
                channel: 0,
            },
            &world,
        );
        // Beat 1: NoteOff (held 1 beat)
        advance_to_beat(&transport, 1.0);
        lane.on_input(
            InputEvent::NoteOff {
                note: 60,
                channel: 0,
            },
            &world,
        );
        // Voice 1 NoteOff fires at 0 + 1 + 1 = 2.0
        // Voice 2 NoteOff fires at 0 + 2 + 1 = 3.0
        assert_eq!(lane.pending_off.len(), 2);

        // At beat 1.5: voice 1's NoteOn has matured but its NoteOff
        // (at 2.0) hasn't. Voice 2 hasn't fired NoteOn yet either.
        advance_to_beat(&transport, 1.5);
        let out = lane.tick(&world);
        assert_eq!(out.ops.len(), 1); // voice 1's NoteOn
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

    #[test]
    fn transpose_emits_diatonic_interval() {
        let (mut lane, world, transport) = fixture();
        lane.set_enabled(true);
        lane.set_delay(1.0);
        lane.set_transpose(2);

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
        match &out.ops[0] {
            DispatchOp::NoteOn { note, .. } => assert_eq!(*note, 64),
            _ => panic!("expected NoteOn"),
        }
    }

    #[test]
    fn transpose_uses_modal_interchange_for_chromatic_input() {
        let (mut lane, world, transport) = fixture();
        lane.set_enabled(true);
        lane.set_delay(1.0);
        lane.set_transpose(2);
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
        assert_ne!(emitted, 63, "must not fall back to unison");
        assert!((emitted as i16 - 63).abs() <= 7);
    }

    #[test]
    fn voices_cap_at_eight() {
        let mut lane = CanonLane::new();
        let huge: Vec<CanonVoice> = (0..20)
            .map(|i| CanonVoice::new(0.5 * i as f32, 0))
            .collect();
        lane.set_voices(huge);
        assert_eq!(lane.voices().len(), 8, "must clamp to 8 voices max");
    }

    #[test]
    fn serialize_and_restore_round_trip() {
        let mut lane = CanonLane::new();
        lane.set_enabled(true);
        lane.set_voices(vec![
            CanonVoice::new(1.0, 0),
            CanonVoice::new(2.0, 3),
            CanonVoice::new(0.5, -2),
        ]);
        let snapshot = lane.serialize_state();

        let mut restored = CanonLane::new();
        restored.deserialize_state(snapshot).unwrap();
        assert_eq!(restored.enabled, true);
        assert_eq!(restored.voices.len(), 3);
        assert!((restored.voices[0].delay_beats - 1.0).abs() < 1e-6);
        assert_eq!(restored.voices[1].transpose_degrees, 3);
        assert!((restored.voices[2].delay_beats - 0.5).abs() < 1e-6);
        assert_eq!(restored.voices[2].transpose_degrees, -2);
    }

    /// Old serialized format (single voice keys, no `voices` array)
    /// still deserializes correctly into voices[0]. Guards against
    /// breaking saved configs from before multi-voice landed.
    #[test]
    fn deserialize_legacy_single_voice_format() {
        let mut lane = CanonLane::new();
        let legacy = serde_json::json!({
            "enabled": true,
            "delay_beats": 2.5,
            "transpose_degrees": 4,
        });
        lane.deserialize_state(legacy).unwrap();
        assert_eq!(lane.enabled, true);
        assert_eq!(lane.voices.len(), 1);
        assert!((lane.voices[0].delay_beats - 2.5).abs() < 1e-6);
        assert_eq!(lane.voices[0].transpose_degrees, 4);
    }
}
