//! Small linear ADSR used as Elixir's Chapter 2 amplitude trajectory.

use crate::AmpEnvelope;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Stage {
    Idle,
    Attack,
    Decay,
    Sustain,
    Release,
}

pub(crate) struct Envelope {
    stage: Stage,
    value: f32,
    stage_elapsed: f32,
    release_start: f32,
    forced_release_secs: f32,
}

impl Envelope {
    pub const fn new() -> Self {
        Self {
            stage: Stage::Idle,
            value: 0.0,
            stage_elapsed: 0.0,
            release_start: 0.0,
            forced_release_secs: -1.0,
        }
    }

    pub fn note_on(&mut self) {
        self.stage = Stage::Attack;
        self.value = 0.0;
        self.stage_elapsed = 0.0;
        self.release_start = 0.0;
        self.forced_release_secs = -1.0;
    }

    pub fn note_off(&mut self) {
        if self.stage != Stage::Idle && self.stage != Stage::Release {
            self.stage = Stage::Release;
            self.stage_elapsed = 0.0;
            self.release_start = self.value;
            self.forced_release_secs = -1.0;
        }
    }

    pub fn force_release(&mut self, seconds: f32) {
        if self.stage != Stage::Idle {
            self.stage = Stage::Release;
            self.stage_elapsed = 0.0;
            self.release_start = self.value;
            self.forced_release_secs = seconds.max(0.0);
        }
    }

    #[inline]
    pub fn tick(&mut self, parameters: AmpEnvelope, sample_rate: f32) -> f32 {
        let dt = 1.0 / sample_rate.max(1.0);
        match self.stage {
            Stage::Idle => self.value = 0.0,
            Stage::Attack => {
                if parameters.attack_secs <= 0.0 {
                    self.value = 1.0;
                    self.stage = Stage::Decay;
                    self.stage_elapsed = 0.0;
                } else {
                    self.stage_elapsed += dt;
                    self.value = (self.stage_elapsed / parameters.attack_secs).min(1.0);
                    if self.value >= 1.0 {
                        self.stage = Stage::Decay;
                        self.stage_elapsed = 0.0;
                    }
                }
            }
            Stage::Decay => {
                if parameters.decay_secs <= 0.0 {
                    self.value = parameters.sustain_level;
                    self.stage = Stage::Sustain;
                } else {
                    self.stage_elapsed += dt;
                    let progress = (self.stage_elapsed / parameters.decay_secs).min(1.0);
                    self.value = 1.0 + (parameters.sustain_level - 1.0) * progress;
                    if progress >= 1.0 {
                        self.stage = Stage::Sustain;
                    }
                }
            }
            Stage::Sustain => self.value = parameters.sustain_level,
            Stage::Release => {
                let release_secs = if self.forced_release_secs >= 0.0 {
                    self.forced_release_secs
                } else {
                    parameters.release_secs
                };
                if release_secs <= 0.0 {
                    self.value = 0.0;
                    self.stage = Stage::Idle;
                } else {
                    self.stage_elapsed += dt;
                    let progress = (self.stage_elapsed / release_secs).min(1.0);
                    self.value = self.release_start * (1.0 - progress);
                    if progress >= 1.0 {
                        self.value = 0.0;
                        self.stage = Stage::Idle;
                    }
                }
            }
        }
        self.value
    }

    pub fn is_idle(&self) -> bool {
        self.stage == Stage::Idle
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adsr_reaches_each_declared_level() {
        let mut envelope = Envelope::new();
        let parameters = AmpEnvelope {
            attack_secs: 0.01,
            decay_secs: 0.01,
            sustain_level: 0.4,
            release_secs: 0.02,
            ..AmpEnvelope::default()
        };
        envelope.note_on();
        for _ in 0..10 {
            envelope.tick(parameters, 1_000.0);
        }
        assert!((envelope.value - 1.0).abs() < 1.0e-6);
        for _ in 0..10 {
            envelope.tick(parameters, 1_000.0);
        }
        assert!((envelope.value - 0.4).abs() < 1.0e-6);
        envelope.note_off();
        for _ in 0..20 {
            envelope.tick(parameters, 1_000.0);
        }
        assert!(envelope.is_idle());
        assert_eq!(envelope.value, 0.0);
    }

    #[test]
    fn release_starts_from_the_current_trajectory_value() {
        let mut envelope = Envelope::new();
        let parameters = AmpEnvelope {
            attack_secs: 1.0,
            release_secs: 1.0,
            ..AmpEnvelope::default()
        };
        envelope.note_on();
        for _ in 0..250 {
            envelope.tick(parameters, 1_000.0);
        }
        let before = envelope.value;
        envelope.note_off();
        envelope.tick(parameters, 1_000.0);
        assert!(envelope.value < before);
        assert!(envelope.value > 0.0);
    }
}
