//! Loop slots — recorded MIDI replayed through the companion pipeline.
//!
//! Each slot has a source (Input or Output), a length in bars, a state
//! machine (Empty | Armed | Recording | Playing | Stopped), and an
//! optional buffer of beat-offset-keyed events. The slot's `tick`
//! emits dispatch ops based on its state.
//!
//! Build phase: type scaffolding only. Behavior lands in Phase 2.
//!
//! `#[allow(dead_code)]` is applied module-wide because variants/fields
//! are intentionally unconsumed at this increment — Phase 2 wires them.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

/// Whether the slot captures pre-harmony (`Input`) or post-harmony,
/// post-pattern (`Output`) MIDI events. Set at record time and frozen
/// in the buffer; replay branches on `buffer.source` accordingly.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LoopSource {
    /// Captures raw user input before harmony engine. On replay,
    /// re-enters the harmony engine — pattern + harmony apply fresh
    /// each cycle.
    Input,
    /// Captures the harmony-engine output (post-pattern, post-routing).
    /// On replay, bypasses harmony engine and goes direct to dispatch.
    /// Loop is "frozen" in original mode/key.
    Output,
}

/// Slot lifecycle.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LoopState {
    /// No buffer, not armed. UI shows "empty".
    Empty,
    /// User pressed record; waiting for next bar boundary to start
    /// capturing. `armed_at_bar` is the global bar at the moment of
    /// arming; recording starts at `armed_at_bar + 1`.
    Armed { armed_at_bar: u32 },
    /// Capturing events into the buffer. `started_at_bar` is the
    /// global bar when capture began; events store `beat_offset`
    /// relative to this bar.
    Recording { started_at_bar: u32 },
    /// Buffer present, replaying on each transport cycle. The
    /// playback is keyed by `(transport.totalBeat - phase_origin) mod
    /// length_beats`.
    Playing,
    /// Buffer present but not currently playing. UI offers Play / Clear.
    Stopped,
}

/// One recorded MIDI event with beat-relative offset.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LoopEvent {
    /// Offset in beats from the start of the loop. Range `[0, length_beats)`.
    pub beat_offset_us: u64, // microseconds-of-a-beat to keep Eq derivable; equivalent to f64 beats × 1_000_000
    pub kind: LoopEventKind,
}

/// MIDI event kinds captured by the looper. Channel and velocity are
/// preserved so replay matches the original take's musical character.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LoopEventKind {
    NoteOn { note: u8, velocity: u8, channel: u8 },
    NoteOff { note: u8, channel: u8 },
}

/// Recorded buffer for one slot. Source is set at capture time and
/// determines the replay path — never mutates after capture.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LoopBuffer {
    pub source: LoopSource,
    /// Length in beats at record time. Stored explicitly so a later
    /// time-signature change doesn't silently re-interpret the buffer.
    pub length_beats_us: u64,
    pub events: Vec<LoopEvent>,
}

/// One slot in the companion's `loops: Vec<LoopSlot>`. Slots are
/// independent — they play simultaneously, each on its own length
/// modulus. Slot identity (`id`) is stable across UI re-renders so
/// MIDI Learn bindings don't drift when the slot count changes.
#[derive(Clone, Debug)]
pub struct LoopSlot {
    pub id: u32,
    pub state: LoopState,
    pub source: LoopSource,
    pub length_bars: u8,
    pub buffer: Option<LoopBuffer>,
}

impl Default for LoopSlot {
    fn default() -> Self {
        Self {
            id: 0,
            state: LoopState::Empty,
            source: LoopSource::Output,
            length_bars: 2,
            buffer: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_slot_starts_empty() {
        let s = LoopSlot::default();
        assert_eq!(s.state, LoopState::Empty);
        assert!(s.buffer.is_none());
        assert_eq!(s.source, LoopSource::Output);
        assert_eq!(s.length_bars, 2);
    }

    #[test]
    fn loop_source_serializes_lowercase() {
        let json = serde_json::to_string(&LoopSource::Input).unwrap();
        assert_eq!(json, "\"input\"");
        let parsed: LoopSource = serde_json::from_str("\"output\"").unwrap();
        assert_eq!(parsed, LoopSource::Output);
    }
}
