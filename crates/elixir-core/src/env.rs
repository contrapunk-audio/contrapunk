//! ADSR envelope (Phase 21.A1).
//!
//! Linear segments. The full DAHDSR + power-curve shaping lands in A3
//! once the modulation matrix is in. For A1 the goal is *audible
//! shape* — no clicks on note-on, no DC tail on note-off.

/// Envelope stage.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EnvStage {
    Idle,
    Attack,
    Decay,
    Sustain,
    Release,
}

/// Plain ADSR. One per voice in A1; A2 introduces SIMD-packed voice
/// envelopes and A3 introduces per-voice multi-stage envelopes.
pub struct AdsrEnvelope {
    stage: EnvStage,
    value: f32,
    sample_rate: f32,
    attack_secs: f32,
    decay_secs: f32,
    sustain_level: f32,
    release_secs: f32,
    /// Snapshot of `value` at the moment of `note_off` so release decays
    /// linearly from wherever the envelope was, never re-jumping to 1.
    release_start: f32,
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
        }
    }

    pub fn set_sample_rate(&mut self, sr: f32) {
        self.sample_rate = sr.max(1.0);
    }

    pub fn set_attack(&mut self, secs: f32) {
        self.attack_secs = secs.max(0.001);
    }
    pub fn set_decay(&mut self, secs: f32) {
        self.decay_secs = secs.max(0.001);
    }
    pub fn set_sustain(&mut self, lvl: f32) {
        self.sustain_level = lvl.clamp(0.0, 1.0);
    }
    pub fn set_release(&mut self, secs: f32) {
        self.release_secs = secs.max(0.001);
    }

    pub fn note_on(&mut self) {
        self.stage = EnvStage::Attack;
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

    pub fn is_idle(&self) -> bool {
        self.stage == EnvStage::Idle
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
                let step = self.release_start / (self.release_secs * self.sample_rate);
                self.value -= step;
                if self.value <= 0.0 {
                    self.value = 0.0;
                    self.stage = EnvStage::Idle;
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
            // attack: ~48 samples, decay: ~240 samples
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
}
