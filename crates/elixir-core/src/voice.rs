//! Single synth voice (Phase 21.A1).
//!
//! Combines an oscillator with an envelope. A2 replaces the single-voice
//! engine field with a SIMD-packed pool, and the steal / priority logic
//! lands there too.

use crate::env::AdsrEnvelope;
use crate::osc::Oscillator;
use crate::tables::SineTable;
use crate::util::midi_to_freq;

pub struct Voice {
    osc: Oscillator,
    env: AdsrEnvelope,
    active: bool,
    note: u8,
    velocity: f32,
}

impl Voice {
    pub const fn new() -> Self {
        Self {
            osc: Oscillator::new(),
            env: AdsrEnvelope::new(),
            active: false,
            note: 0,
            velocity: 0.0,
        }
    }

    pub fn set_sample_rate(&mut self, sr: f32) {
        self.env.set_sample_rate(sr);
    }

    pub fn note_on(&mut self, note: u8, velocity: u8, sample_rate: f32) {
        self.active = true;
        self.note = note;
        self.velocity = (velocity as f32 / 127.0).clamp(0.0, 1.0);
        self.osc.set_frequency(midi_to_freq(note), sample_rate);
        self.osc.reset_phase();
        self.env.set_sample_rate(sample_rate);
        self.env.note_on();
    }

    pub fn note_off(&mut self, note: u8) {
        if self.active && self.note == note {
            self.env.note_off();
        }
    }

    pub fn all_notes_off(&mut self) {
        if self.active {
            self.env.note_off();
        }
    }

    pub fn is_active(&self) -> bool {
        self.active
    }
    pub fn note(&self) -> u8 {
        self.note
    }

    /// Produce one sample. Returns 0.0 and self-deactivates when the
    /// envelope hits idle after release.
    #[inline]
    pub fn tick(&mut self, table: &SineTable) -> f32 {
        if !self.active {
            return 0.0;
        }
        let env_v = self.env.tick();
        if env_v <= 0.0 && self.env.is_idle() {
            self.active = false;
            return 0.0;
        }
        let osc_v = self.osc.tick(table);
        osc_v * env_v * self.velocity
    }
}

impl Default for Voice {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fresh_voice_is_silent_and_inactive() {
        let table = SineTable::new();
        let mut v = Voice::new();
        assert!(!v.is_active());
        for _ in 0..128 {
            assert_eq!(v.tick(&table), 0.0);
        }
    }

    #[test]
    fn note_on_makes_voice_active_and_audible() {
        let table = SineTable::new();
        let mut v = Voice::new();
        v.note_on(69, 100, 48_000.0);
        assert!(v.is_active());
        let mut any_nonzero = false;
        for _ in 0..2048 {
            if v.tick(&table).abs() > 1e-6 {
                any_nonzero = true;
            }
        }
        assert!(any_nonzero);
    }

    #[test]
    fn note_off_sends_voice_to_idle() {
        let table = SineTable::new();
        let mut v = Voice::new();
        v.note_on(60, 100, 48_000.0);
        // run through attack + decay + a bit of sustain
        for _ in 0..(48_000 / 4) {
            let _ = v.tick(&table);
        }
        v.note_off(60);
        // release default is 250 ms → ~12_000 samples; allow margin
        for _ in 0..30_000 {
            let _ = v.tick(&table);
        }
        assert!(!v.is_active());
    }

    #[test]
    fn note_off_only_releases_matching_note() {
        let table = SineTable::new();
        let mut v = Voice::new();
        v.note_on(60, 100, 48_000.0);
        v.note_off(72); // different note: must not affect anything
        for _ in 0..100 {
            let _ = v.tick(&table);
        }
        assert!(v.is_active());
    }
}
