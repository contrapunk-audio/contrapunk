use std::collections::HashMap;

use rand::Rng;
use wmidi::{Channel, Note, Velocity};

use super::beat_clock::BeatClock;
use super::config::{HumanizeConfig, HumanizedNote};

/// Record of humanization applied to a Note-On, so Note-Off can match.
#[derive(Clone, Debug)]
struct HumanizationRecord {
    delay_ms: u16,
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
}
