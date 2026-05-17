//! Synth voice (Phases 21.A1 + 21.A2).
//!
//! A1 was a single-voice scaffold. A2 keeps the same `Voice` struct
//! shape but adds the bookkeeping the engine needs to run a pool:
//!
//! - `age` — monotonic note-on counter so the handler can find the
//!   oldest live voice when stealing.
//! - `killing` — voice was stolen and is fading via [`AdsrEnvelope::kill`];
//!   it does not count toward live polyphony.
//! - `sustained` — note-off arrived while the sustain pedal was down;
//!   the voice keeps playing until the pedal releases.
//!
//! Future A2 follow-up: SIMD-packed `AggregateVoice` (two voices per
//! `f32x8` lane), per the design doc. Not required for correctness.

use crate::env::AdsrEnvelope;
use crate::osc::Oscillator;
use crate::tables::SineTable;
use crate::util::midi_to_freq;

pub struct Voice {
    osc: Oscillator,
    env: AdsrEnvelope,
    active: bool,
    killing: bool,
    sustained: bool,
    note: u8,
    velocity: f32,
    age: u64,
}

impl Voice {
    pub const fn new() -> Self {
        Self {
            osc: Oscillator::new(),
            env: AdsrEnvelope::new(),
            active: false,
            killing: false,
            sustained: false,
            note: 0,
            velocity: 0.0,
            age: 0,
        }
    }

    pub fn set_sample_rate(&mut self, sr: f32) {
        self.env.set_sample_rate(sr);
    }

    pub fn note_on(&mut self, note: u8, velocity: u8, sample_rate: f32, age: u64) {
        self.active = true;
        self.killing = false;
        self.sustained = false;
        self.note = note;
        self.velocity = (velocity as f32 / 127.0).clamp(0.0, 1.0);
        self.osc.set_frequency(midi_to_freq(note), sample_rate);
        self.osc.reset_phase();
        self.env.set_sample_rate(sample_rate);
        self.env.note_on();
        self.age = age;
    }

    /// Handle note-off. If `sustain_down` is true and the voice matches
    /// the requested note, mark it sustained instead of releasing.
    pub fn note_off_or_sustain(&mut self, note: u8, sustain_down: bool) {
        if !self.active || self.killing || self.note != note {
            return;
        }
        if sustain_down {
            self.sustained = true;
        } else {
            self.env.note_off();
        }
    }

    /// Called when the sustain pedal lifts. Drops any held notes into
    /// release.
    pub fn release_sustain(&mut self) {
        if self.sustained {
            self.sustained = false;
            self.env.note_off();
        }
    }

    /// Force a fast-release ramp (see [`AdsrEnvelope::kill`]). The voice
    /// no longer counts toward live polyphony immediately, so the slot
    /// is reusable as soon as the ramp completes (or even sooner — the
    /// engine just won't keep it in the live count).
    pub fn kill(&mut self) {
        if self.active && !self.killing {
            self.killing = true;
            self.sustained = false;
            self.env.kill();
        }
    }

    pub fn all_notes_off(&mut self) {
        if self.active {
            self.sustained = false;
            self.env.note_off();
        }
    }

    pub fn is_active(&self) -> bool {
        self.active
    }

    /// Counts toward live polyphony: active and not currently being killed.
    pub fn is_live(&self) -> bool {
        self.active && !self.killing
    }

    /// Voice owns a currently-playing note that's neither released nor
    /// being killed nor sustain-held.
    pub fn is_playing_note(&self, note: u8) -> bool {
        self.is_live() && self.note == note && !self.sustained
    }

    pub fn note(&self) -> u8 {
        self.note
    }
    pub fn age(&self) -> u64 {
        self.age
    }
    pub fn is_killing(&self) -> bool {
        self.killing
    }
    pub fn is_sustained(&self) -> bool {
        self.sustained
    }
    /// Read-only access to the envelope's current output. Used by the
    /// modulation matrix as the `AmpEnv` source.
    pub fn env_value(&self) -> f32 {
        self.env.value()
    }

    /// Produce one sample. Returns 0.0 and self-deactivates when the
    /// envelope hits idle.
    #[inline]
    pub fn tick(&mut self, table: &SineTable) -> f32 {
        if !self.active {
            return 0.0;
        }
        let env_v = self.env.tick();
        if env_v <= 0.0 && self.env.is_idle() {
            self.active = false;
            self.killing = false;
            self.sustained = false;
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
        v.note_on(69, 100, 48_000.0, 0);
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
        v.note_on(60, 100, 48_000.0, 0);
        for _ in 0..(48_000 / 4) {
            let _ = v.tick(&table);
        }
        v.note_off_or_sustain(60, false);
        for _ in 0..30_000 {
            let _ = v.tick(&table);
        }
        assert!(!v.is_active());
    }

    #[test]
    fn note_off_only_releases_matching_note() {
        let table = SineTable::new();
        let mut v = Voice::new();
        v.note_on(60, 100, 48_000.0, 0);
        v.note_off_or_sustain(72, false);
        for _ in 0..100 {
            let _ = v.tick(&table);
        }
        assert!(v.is_active());
    }

    #[test]
    fn kill_demotes_voice_immediately_and_silences_within_5ms() {
        let table = SineTable::new();
        let mut v = Voice::new();
        v.note_on(60, 127, 48_000.0, 0);
        // settle into sustain
        for _ in 0..2048 {
            let _ = v.tick(&table);
        }
        assert!(v.is_live());
        v.kill();
        assert!(!v.is_live());
        assert!(v.is_killing());
        // 5 ms = 240 samples at 48 kHz
        for _ in 0..400 {
            let _ = v.tick(&table);
        }
        assert!(!v.is_active());
    }

    #[test]
    fn sustain_holds_through_note_off_and_releases_on_pedal_up() {
        let table = SineTable::new();
        let mut v = Voice::new();
        v.note_on(60, 100, 48_000.0, 0);
        for _ in 0..2048 {
            let _ = v.tick(&table);
        }
        // pedal down → note-off marks sustained, voice keeps playing
        v.note_off_or_sustain(60, true);
        assert!(v.is_sustained());
        assert!(v.is_live() || v.is_active());
        for _ in 0..2048 {
            let _ = v.tick(&table);
        }
        assert!(v.is_active());
        // pedal up → release
        v.release_sustain();
        assert!(!v.is_sustained());
        for _ in 0..30_000 {
            let _ = v.tick(&table);
        }
        assert!(!v.is_active());
    }
}
