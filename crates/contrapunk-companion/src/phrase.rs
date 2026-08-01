//! Shared, bounded phrase context derived from player input.
//!
//! A phrase is deliberately defined as one input episode separated by a
//! configurable amount of musical silence. This does not pretend to infer
//! compositional phrasing: it gives every companion lane the same exact
//! channel/note ownership, sustain, opening pitch, latest pitch, and boundary.

use serde::Serialize;

use crate::lane::InputEvent;

pub const MIN_PHRASE_GAP_BEATS: f64 = 0.5;
pub const MAX_PHRASE_GAP_BEATS: f64 = 16.0;
pub const DEFAULT_PHRASE_GAP_BEATS: f64 = 2.0;

const MIDI_NOTES: usize = 128;
const MIDI_CHANNELS: usize = 16;
const INPUT_OWNERS: usize = MIDI_NOTES * MIDI_CHANNELS;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PhrasePhase {
    Idle,
    Opening,
    Active,
    Releasing,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct PhraseSnapshot {
    pub id: Option<u64>,
    pub phase: PhrasePhase,
    pub gap_beats: f64,
    pub started_at: Option<f64>,
    pub release_started_at: Option<f64>,
    pub attack_count: u16,
    pub opening_note: Option<u8>,
    pub previous_note: Option<u8>,
    pub latest_note: Option<u8>,
    pub latest_velocity: Option<u8>,
    pub latest_channel: Option<u8>,
    pub input_idle: bool,
}

impl PhraseSnapshot {
    pub fn idle(gap_beats: f64) -> Self {
        Self {
            id: None,
            phase: PhrasePhase::Idle,
            gap_beats,
            started_at: None,
            release_started_at: None,
            attack_count: 0,
            opening_note: None,
            previous_note: None,
            latest_note: None,
            latest_velocity: None,
            latest_channel: None,
            input_idle: true,
        }
    }
}

/// Canonical input ownership and phrase lifecycle. One instance lives in
/// `WorldState`; `Companion` updates it once before fan-out to lanes.
pub struct PhraseContext {
    snapshot: PhraseSnapshot,
    next_id: u64,
    pressed: [u16; INPUT_OWNERS],
    sustained: [u16; INPUT_OWNERS],
    sustain_down: [bool; MIDI_CHANNELS],
    active_owners: u32,
    last_beat: Option<f64>,
}

impl PhraseContext {
    pub fn new() -> Self {
        Self {
            snapshot: PhraseSnapshot::idle(DEFAULT_PHRASE_GAP_BEATS),
            next_id: 0,
            pressed: [0; INPUT_OWNERS],
            sustained: [0; INPUT_OWNERS],
            sustain_down: [false; MIDI_CHANNELS],
            active_owners: 0,
            last_beat: None,
        }
    }

    pub fn snapshot(&self) -> PhraseSnapshot {
        self.snapshot
    }

    pub fn set_gap_beats(&mut self, beats: f64) -> Result<(), String> {
        if !beats.is_finite() || !(MIN_PHRASE_GAP_BEATS..=MAX_PHRASE_GAP_BEATS).contains(&beats) {
            return Err(format!(
                "phrase gap must be between {MIN_PHRASE_GAP_BEATS} and {MAX_PHRASE_GAP_BEATS} beats"
            ));
        }
        self.snapshot.gap_beats = beats;
        Ok(())
    }

    pub fn observe(&mut self, event: InputEvent, now: f64) {
        self.advance(now);
        match event {
            InputEvent::NoteOn {
                note,
                velocity,
                channel,
            } if Self::valid_owner(note, channel) => self.note_on(note, velocity, channel, now),
            InputEvent::NoteOff { note, channel } if Self::valid_owner(note, channel) => {
                self.note_off(note, channel, now)
            }
            InputEvent::Cc {
                number: 64,
                value,
                channel,
            } if (channel as usize) < MIDI_CHANNELS => self.set_sustain(channel, value >= 64, now),
            InputEvent::Cc {
                number: 120 | 123, ..
            } => self.reset_runtime(),
            _ => {}
        }
    }

    pub fn advance(&mut self, now: f64) {
        if self
            .last_beat
            .map(|previous| now + f64::EPSILON < previous)
            .unwrap_or(false)
        {
            self.reset_runtime();
        }
        self.last_beat = Some(now);

        if self.snapshot.phase == PhrasePhase::Releasing
            && self
                .snapshot
                .release_started_at
                .map(|released| now + 1.0e-9 >= released + self.snapshot.gap_beats)
                .unwrap_or(false)
        {
            let gap = self.snapshot.gap_beats;
            self.snapshot = PhraseSnapshot::idle(gap);
        }
    }

    /// Clear physical/runtime ownership while preserving the player's gap.
    pub fn reset_runtime(&mut self) {
        let gap = self.snapshot.gap_beats;
        self.snapshot = PhraseSnapshot::idle(gap);
        self.pressed.fill(0);
        self.sustained.fill(0);
        self.sustain_down.fill(false);
        self.active_owners = 0;
        self.last_beat = None;
    }

    fn note_on(&mut self, note: u8, velocity: u8, channel: u8, now: f64) {
        let owner = Self::owner(note, channel);
        self.pressed[owner] = self.pressed[owner].saturating_add(1);
        self.active_owners = self.active_owners.saturating_add(1);

        if self.snapshot.phase == PhrasePhase::Idle {
            self.next_id = self.next_id.wrapping_add(1);
            if self.next_id == 0 {
                self.next_id = 1;
            }
            self.snapshot.id = Some(self.next_id);
            self.snapshot.started_at = Some(now);
            self.snapshot.opening_note = Some(note);
            self.snapshot.attack_count = 0;
        }

        self.snapshot.attack_count = self.snapshot.attack_count.saturating_add(1);
        self.snapshot.previous_note = self.snapshot.latest_note;
        self.snapshot.latest_note = Some(note);
        self.snapshot.latest_velocity = Some(velocity);
        self.snapshot.latest_channel = Some(channel);
        self.snapshot.release_started_at = None;
        self.snapshot.input_idle = false;
        self.snapshot.phase = if self.snapshot.attack_count == 1 {
            PhrasePhase::Opening
        } else {
            PhrasePhase::Active
        };
    }

    fn note_off(&mut self, note: u8, channel: u8, now: f64) {
        let owner = Self::owner(note, channel);
        if self.pressed[owner] == 0 {
            return;
        }
        self.pressed[owner] -= 1;
        self.active_owners = self.active_owners.saturating_sub(1);
        if self.sustain_down[channel as usize] {
            self.sustained[owner] = self.sustained[owner].saturating_add(1);
            self.active_owners = self.active_owners.saturating_add(1);
        }
        self.update_release(now);
    }

    fn set_sustain(&mut self, channel: u8, down: bool, now: f64) {
        let channel = channel as usize;
        if self.sustain_down[channel] == down {
            return;
        }
        self.sustain_down[channel] = down;
        if !down {
            let start = channel * MIDI_NOTES;
            for owner in start..start + MIDI_NOTES {
                self.active_owners = self
                    .active_owners
                    .saturating_sub(u32::from(self.sustained[owner]));
                self.sustained[owner] = 0;
            }
            self.update_release(now);
        }
    }

    fn update_release(&mut self, now: f64) {
        self.snapshot.input_idle = self.active_owners == 0;
        if self.active_owners == 0 && self.snapshot.phase != PhrasePhase::Idle {
            self.snapshot.phase = PhrasePhase::Releasing;
            self.snapshot.release_started_at.get_or_insert(now);
        }
    }

    fn valid_owner(note: u8, channel: u8) -> bool {
        (note as usize) < MIDI_NOTES && (channel as usize) < MIDI_CHANNELS
    }

    fn owner(note: u8, channel: u8) -> usize {
        debug_assert!(Self::valid_owner(note, channel));
        channel as usize * MIDI_NOTES + note as usize
    }
}

impl Default for PhraseContext {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn on(note: u8, channel: u8) -> InputEvent {
        InputEvent::NoteOn {
            note,
            velocity: 100,
            channel,
        }
    }

    fn off(note: u8, channel: u8) -> InputEvent {
        InputEvent::NoteOff { note, channel }
    }

    #[test]
    fn phrase_survives_short_gap_and_changes_after_configured_gap() {
        let mut phrase = PhraseContext::new();
        phrase.observe(on(60, 0), 0.0);
        let first = phrase.snapshot().id;
        phrase.observe(off(60, 0), 0.25);
        phrase.observe(on(62, 0), 2.0);
        assert_eq!(phrase.snapshot().id, first);

        phrase.observe(off(62, 0), 2.1);
        phrase.advance(4.1);
        assert_eq!(phrase.snapshot().phase, PhrasePhase::Idle);
        phrase.observe(on(67, 0), 4.2);
        assert_ne!(phrase.snapshot().id, first);
        assert_eq!(phrase.snapshot().opening_note, Some(67));
    }

    #[test]
    fn ownership_counts_repeats_and_channels_exactly() {
        let mut phrase = PhraseContext::new();
        phrase.observe(on(60, 0), 0.0);
        phrase.observe(on(60, 0), 0.1);
        phrase.observe(on(60, 1), 0.2);
        phrase.observe(off(60, 0), 0.3);
        phrase.observe(off(60, 1), 0.4);
        assert!(!phrase.snapshot().input_idle);
        phrase.observe(off(60, 0), 0.5);
        assert!(phrase.snapshot().input_idle);
        phrase.observe(off(60, 0), 0.6);
        assert!(phrase.snapshot().input_idle);
    }

    #[test]
    fn sustain_is_counted_per_channel() {
        let mut phrase = PhraseContext::new();
        phrase.observe(on(60, 2), 0.0);
        phrase.observe(
            InputEvent::Cc {
                number: 64,
                value: 127,
                channel: 2,
            },
            0.1,
        );
        phrase.observe(off(60, 2), 0.2);
        assert!(!phrase.snapshot().input_idle);
        phrase.observe(
            InputEvent::Cc {
                number: 64,
                value: 0,
                channel: 1,
            },
            0.3,
        );
        assert!(!phrase.snapshot().input_idle);
        phrase.observe(
            InputEvent::Cc {
                number: 64,
                value: 0,
                channel: 2,
            },
            0.4,
        );
        assert!(phrase.snapshot().input_idle);
    }

    #[test]
    fn gap_validation_and_reset_preserve_configuration() {
        let mut phrase = PhraseContext::new();
        assert!(phrase.set_gap_beats(f64::NAN).is_err());
        assert!(phrase.set_gap_beats(0.49).is_err());
        assert!(phrase.set_gap_beats(16.01).is_err());
        phrase.set_gap_beats(7.25).unwrap();
        phrase.observe(on(64, 0), 0.0);
        phrase.reset_runtime();
        assert_eq!(phrase.snapshot(), PhraseSnapshot::idle(7.25));
    }

    #[test]
    fn invalid_midi_note_and_channel_values_are_ignored() {
        let mut phrase = PhraseContext::new();
        phrase.observe(on(128, 0), 0.0);
        phrase.observe(on(60, 16), 0.1);
        phrase.observe(
            InputEvent::Cc {
                number: 64,
                value: 127,
                channel: 16,
            },
            0.2,
        );
        assert_eq!(
            phrase.snapshot(),
            PhraseSnapshot::idle(DEFAULT_PHRASE_GAP_BEATS)
        );

        phrase.observe(on(60, 0), 0.3);
        assert_eq!(phrase.snapshot().latest_note, Some(60));
    }

    #[test]
    fn rewind_and_all_notes_off_clear_runtime() {
        let mut phrase = PhraseContext::new();
        phrase.observe(on(60, 0), 4.0);
        phrase.advance(1.0);
        assert_eq!(phrase.snapshot().phase, PhrasePhase::Idle);

        phrase.observe(on(62, 0), 2.0);
        phrase.observe(
            InputEvent::Cc {
                number: 123,
                value: 0,
                channel: 0,
            },
            2.1,
        );
        assert_eq!(phrase.snapshot().phase, PhrasePhase::Idle);
    }
}
