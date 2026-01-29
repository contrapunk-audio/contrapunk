use std::collections::HashMap;

use rand::Rng;
use wmidi::{Channel, Note, Velocity};

use super::beat_clock::BeatClock;
use super::config::{HumanizeConfig, HumanizedNote};

/// Record of humanization applied to a Note-On, so Note-Off can match.
#[derive(Clone, Debug)]
struct HumanizationRecord {
    delay_ms: u16,
    #[allow(dead_code)]
    velocity: Velocity,
    duration_delta_ms: i16,
}

/// Computes humanization (velocity variation, jitter, swing, duration delta) for notes.
pub struct Humanizer {
    config: HumanizeConfig,
    clock: BeatClock,
    active_humanization: HashMap<u8, HumanizationRecord>,
}

impl Humanizer {
    pub fn new(config: HumanizeConfig) -> Self {
        let clock = BeatClock::new(config.bpm, config.beats_per_bar, config.beat_unit);
        Self {
            config,
            clock,
            active_humanization: HashMap::new(),
        }
    }

    pub fn config(&self) -> &HumanizeConfig {
        &self.config
    }

    pub fn config_mut(&mut self) -> &mut HumanizeConfig {
        &mut self.config
    }

    pub fn clock(&self) -> &BeatClock {
        &self.clock
    }

    pub fn clock_mut(&mut self) -> &mut BeatClock {
        &mut self.clock
    }

    /// Tick the internal beat clock.
    pub fn tick(&mut self, now_ms: f64) {
        self.clock.tick(now_ms);
    }

    /// Humanize a Note-On event. Returns a HumanizedNote with computed variations.
    pub fn humanize_note_on(
        &mut self,
        note: Note,
        channel: Channel,
        velocity: Velocity,
        port: usize,
    ) -> HumanizedNote {
        if !self.config.enabled {
            return HumanizedNote {
                note,
                channel,
                velocity,
                delay_ms: 0,
                duration_delta_ms: 0,
                port,
                is_note_off: false,
            };
        }

        let mut rng = rand::thread_rng();

        // Velocity variation
        let humanized_vel = if self.config.velocity_enabled {
            let base = u8::from(velocity) as i16;
            let variation = self.config.velocity_variation as i16;
            let delta = rng.gen_range(-variation..=variation);
            let clamped = (base + delta).clamp(1, 127) as u8;
            Velocity::try_from(clamped).unwrap_or(velocity)
        } else {
            velocity
        };

        // Jitter
        let jitter = if self.config.jitter_enabled {
            rng.gen_range(self.config.jitter_min_ms..=self.config.jitter_max_ms)
        } else {
            0
        };

        // Swing delay
        let swing = if self.config.swing_enabled {
            compute_swing_delay(&self.clock, self.config.swing_amount)
        } else {
            0
        };

        let total_delay = jitter + swing;

        // Duration delta (extension only, positive values)
        let duration_delta_ms = if self.config.duration_enabled {
            rng.gen_range(0..=self.config.duration_variation_ms as i16)
        } else {
            0
        };

        // Store record for matching Note-Off
        let note_num = u8::from(note);
        self.active_humanization.insert(
            note_num,
            HumanizationRecord {
                delay_ms: total_delay,
                velocity: humanized_vel,
                duration_delta_ms,
            },
        );

        HumanizedNote {
            note,
            channel,
            velocity: humanized_vel,
            delay_ms: total_delay,
            duration_delta_ms,
            port,
            is_note_off: false,
        }
    }

    /// Humanize a Note-Off event. Uses stored humanization from corresponding Note-On.
    pub fn humanize_note_off(
        &mut self,
        note: Note,
        channel: Channel,
        velocity: Velocity,
        port: usize,
    ) -> HumanizedNote {
        let note_num = u8::from(note);
        if let Some(record) = self.active_humanization.remove(&note_num) {
            // Note-Off delay = original jitter + duration extension
            let total_delay = record.delay_ms.saturating_add(record.duration_delta_ms as u16);
            HumanizedNote {
                note,
                channel,
                velocity,
                delay_ms: total_delay,
                duration_delta_ms: record.duration_delta_ms,
                port,
                is_note_off: true,
            }
        } else {
            // No matching Note-On record; pass through unchanged
            HumanizedNote {
                note,
                channel,
                velocity,
                delay_ms: 0,
                duration_delta_ms: 0,
                port,
                is_note_off: true,
            }
        }
    }

    /// Update config and sync clock tempo if changed.
    pub fn update_config(&mut self, config: HumanizeConfig) {
        if (self.config.bpm - config.bpm).abs() > f64::EPSILON
            || self.config.beats_per_bar != config.beats_per_bar
            || self.config.beat_unit != config.beat_unit
        {
            self.clock
                .update_tempo(config.bpm, config.beats_per_bar, config.beat_unit);
        }
        self.config = config;
    }
}

/// Compute swing delay based on beat clock position and swing amount.
fn compute_swing_delay(clock: &BeatClock, swing_amount: f32) -> u16 {
    if !clock.is_offbeat() {
        return 0;
    }
    let delay = (60_000.0 / clock.bpm / 2.0) * swing_amount as f64;
    delay as u16
}
