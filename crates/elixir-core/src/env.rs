//! ADSR envelope (Phases 21.A1 + 21.A2).
//!
//! Linear segments. The full DAHDSR + power-curve shaping lands in A3
//! once the modulation matrix is in. A2 adds the kill ramp: a forced
//! ~5 ms release path used when the voice handler steals an active
//! voice. Without it, abrupt voice reuse clicks audibly.

/// Envelope stage.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EnvStage {
    Idle,
    Attack,
    Decay,
    Sustain,
    Release,
}

/// Fixed ramp time used when [`AdsrEnvelope::kill`] is called. 5 ms is
/// short enough to be inaudible as a note but long enough to avoid a
/// click on full-scale signal.
pub const KILL_RELEASE_SECS: f32 = 0.005;

/// Plain ADSR. One per voice in A1; A2 still uses one per voice but the
/// engine now owns a pool of them. A3 introduces per-voice multi-stage
/// envelopes.
pub struct AdsrEnvelope {
    stage: EnvStage,
    value: f32,
    sample_rate: f32,
    attack_secs: f32,
    decay_secs: f32,
    sustain_level: f32,
    release_secs: f32,
    /// Snapshot of `value` at the moment of `note_off` (or `kill`) so
    /// release decays linearly from wherever the envelope was, never
    /// re-jumping to 1.
    release_start: f32,
    /// When true, the current Release stage uses [`KILL_RELEASE_SECS`]
    /// instead of `release_secs`. Cleared the next time the envelope
    /// transitions back to Idle.
    kill_active: bool,
}

impl AdsrEnvelope {
    pub const fn new() -> Self {
        Self {
            stage: EnvStage::Idle,
            value: 0.0,
            sample_rate: 48_000.0,
            attack_secs: 0.005,
            decay_secs: 0.120,
            sustain_level: 0.70,
            release_secs: 0.250,
            release_start: 0.0,
            kill_active: false,
        }
    }

    pub fn set_sample_rate(&mut self, sr: f32) {
        crate::util::set_finite_clamped(&mut self.sample_rate, sr, 1.0, f32::MAX);
    }

    pub fn set_attack(&mut self, secs: f32) {
        crate::util::set_finite_clamped(&mut self.attack_secs, secs, 0.001, f32::MAX);
    }
    pub fn set_decay(&mut self, secs: f32) {
        crate::util::set_finite_clamped(&mut self.decay_secs, secs, 0.001, f32::MAX);
    }
    pub fn set_sustain(&mut self, lvl: f32) {
        crate::util::set_finite_clamped(&mut self.sustain_level, lvl, 0.0, 1.0);
    }
    pub fn set_release(&mut self, secs: f32) {
        crate::util::set_finite_clamped(&mut self.release_secs, secs, 0.001, f32::MAX);
    }

    pub fn note_on(&mut self) {
        self.stage = EnvStage::Attack;
        self.kill_active = false;
    }

    pub fn note_on_reset(&mut self) {
        self.value = 0.0;
        self.release_start = 0.0;
        self.note_on();
    }

    pub fn note_off(&mut self) {
        if matches!(
            self.stage,
            EnvStage::Attack | EnvStage::Decay | EnvStage::Sustain
        ) {
            self.release_start = self.value;
            self.stage = EnvStage::Release;
        }
    }

    /// Force a fast release (`KILL_RELEASE_SECS`) regardless of the
    /// configured release time. Used by the voice handler when stealing
    /// an active voice to make room for a new note.
    pub fn kill(&mut self) {
        if !matches!(self.stage, EnvStage::Idle) {
            self.release_start = self.value;
            self.stage = EnvStage::Release;
            self.kill_active = true;
        }
    }

    pub fn is_idle(&self) -> bool {
        self.stage == EnvStage::Idle
    }
    pub fn is_killing(&self) -> bool {
        self.kill_active
    }
    pub fn stage(&self) -> EnvStage {
        self.stage
    }
    pub fn value(&self) -> f32 {
        self.value
    }

    /// Produce one envelope sample and advance state.
    #[inline]
    pub fn tick(&mut self) -> f32 {
        self.tick_with_legacy_release(false)
    }

    pub fn tick_with_legacy_release(&mut self, legacy_release: bool) -> f32 {
        match self.stage {
            EnvStage::Idle => {}
            EnvStage::Attack => {
                let step = 1.0 / (self.attack_secs * self.sample_rate);
                self.value += step;
                if self.value >= 1.0 {
                    self.value = 1.0;
                    self.stage = EnvStage::Decay;
                }
            }
            EnvStage::Decay => {
                let step = (1.0 - self.sustain_level) / (self.decay_secs * self.sample_rate);
                self.value -= step;
                if self.value <= self.sustain_level {
                    self.value = self.sustain_level;
                    self.stage = EnvStage::Sustain;
                }
            }
            EnvStage::Sustain => {
                // hold
            }
            EnvStage::Release => {
                let release_secs = if self.kill_active {
                    KILL_RELEASE_SECS
                } else {
                    self.release_secs
                };
                let start = if legacy_release && !self.kill_active {
                    1.0
                } else {
                    self.release_start
                };
                let step = start / (release_secs * self.sample_rate);
                self.value -= step;
                if self.value <= 0.0 {
                    self.value = 0.0;
                    self.stage = EnvStage::Idle;
                    self.kill_active = false;
                }
            }
        }
        self.value
    }
}

impl Default for AdsrEnvelope {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn idle_envelope_returns_zero() {
        let mut e = AdsrEnvelope::new();
        for _ in 0..16 {
            assert_eq!(e.tick(), 0.0);
        }
    }

    #[test]
    fn attack_climbs_then_decays_to_sustain() {
        let mut e = AdsrEnvelope::new();
        e.set_sample_rate(48_000.0);
        e.set_attack(0.001);
        e.set_decay(0.005);
        e.set_sustain(0.5);
        e.note_on();
        for _ in 0..(48 + 240 + 8) {
            e.tick();
        }
        assert!((e.value() - 0.5).abs() < 1e-3);
        assert_eq!(e.stage(), EnvStage::Sustain);
    }

    #[test]
    fn release_returns_to_idle() {
        let mut e = AdsrEnvelope::new();
        e.set_sample_rate(48_000.0);
        e.set_attack(0.001);
        e.set_decay(0.001);
        e.set_sustain(1.0);
        e.set_release(0.001);
        e.note_on();
        for _ in 0..200 {
            e.tick();
        }
        e.note_off();
        for _ in 0..200 {
            e.tick();
        }
        assert!(e.is_idle());
        assert_eq!(e.value(), 0.0);
    }

    #[test]
    fn kill_completes_within_kill_release_secs() {
        let sr = 48_000.0f32;
        let mut e = AdsrEnvelope::new();
        e.set_sample_rate(sr);
        e.set_attack(0.001);
        e.set_decay(0.001);
        e.set_sustain(1.0);
        e.set_release(2.0); // long release that kill must override
        e.note_on();
        for _ in 0..200 {
            e.tick();
        }
        assert!((e.value() - 1.0).abs() < 1e-3);
        e.kill();
        assert!(e.is_killing());
        // 5 ms = 240 samples at 48 kHz. Allow a tiny margin.
        for _ in 0..300 {
            e.tick();
        }
        assert!(e.is_idle());
        assert!(!e.is_killing());
        assert_eq!(e.value(), 0.0);
    }
}
