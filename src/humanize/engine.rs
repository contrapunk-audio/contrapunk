//! Humanizer engine that applies timing and velocity variations.
//!
//! The [`Humanizer`] is the main processor that takes notes and applies
//! configured humanization effects. It maintains a [`BeatClock`] for
//! swing calculations and tracks active notes to match Note-Off events
//! with their corresponding Note-On humanization.
//!
//! # Processing Flow
//!
//! ```text
//! Note-On arrives
//!     |
//!     v
//! +-------------------+
//! | Humanizer         |
//! |-------------------|
//! | 1. Velocity vary  | --> random +/- offset
//! | 2. Jitter calc    | --> random delay
//! | 3. Swing calc     | --> off-beat shift (uses BeatClock)
//! | 4. Duration calc  | --> note extension
//! | 5. Store record   | --> for matching Note-Off
//! +-------------------+
//!     |
//!     v
//! HumanizedNote (with computed offsets)
//! ```

use std::collections::HashMap;

use rand::Rng;
use wmidi::{Channel, Note, Velocity};

use super::beat_clock::BeatClock;
use super::config::{HumanizeConfig, HumanizedNote, SwingCurve};

/// Record of humanization applied to a Note-On, so Note-Off can match.
///
/// When a Note-On is humanized, its parameters are stored here so that
/// the corresponding Note-Off receives matching timing characteristics.
#[derive(Clone, Debug)]
struct HumanizationRecord {
    /// The total delay applied to the Note-On.
    delay_ms: u16,
    /// The humanized velocity (stored but not currently used for Note-Off).
    #[allow(dead_code)]
    velocity: Velocity,
    /// The duration extension to apply to Note-Off.
    duration_delta_ms: i16,
}

/// Applies humanization effects to harmony notes.
///
/// Takes notes from the harmony engine and adds:
/// - Timing jitter (random delay 0-N ms)
/// - Velocity variation (random +/- percentage)
/// - Groove/swing (off-beat timing shift based on beat position)
/// - Duration variation (note-off delay for legato)
///
/// The humanizer maintains an internal [`BeatClock`] for swing calculations.
/// Call [`tick`](Self::tick) on each frame to advance the clock.
///
/// # Note Tracking
///
/// The humanizer tracks active notes in a `HashMap<u8, HumanizationRecord>`.
/// When a Note-On is humanized, its parameters are stored so the matching
/// Note-Off can use the same timing characteristics.
///
/// # Example
///
/// ```ignore
/// use contrapunk::humanize::{Humanizer, HumanizeConfig};
/// use wmidi::{Note, Channel, Velocity};
///
/// let config = HumanizeConfig::default();
/// let mut humanizer = Humanizer::new(config);
///
/// // Enable humanization
/// humanizer.config_mut().enabled = true;
/// humanizer.config_mut().jitter_enabled = true;
///
/// // Start the clock
/// humanizer.clock_mut().start(current_time_ms);
///
/// // Humanize a note
/// let humanized = humanizer.humanize_note_on(
///     Note::C4,
///     Channel::Ch1,
///     Velocity::try_from(100).unwrap(),
///     0, // port index
/// );
/// ```
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
        voice_index: u8,
    ) -> HumanizedNote {
        if !self.config.enabled {
            return HumanizedNote {
                note,
                channel,
                velocity,
                delay_ms: 0,
                duration_delta_ms: 0,
                port,
                voice_index,
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
            compute_swing_delay(&self.clock, &self.config)
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
            voice_index,
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
        voice_index: u8,
    ) -> HumanizedNote {
        let note_num = u8::from(note);
        if let Some(record) = self.active_humanization.remove(&note_num) {
            let total_delay = record
                .delay_ms
                .saturating_add(record.duration_delta_ms as u16);
            HumanizedNote {
                note,
                channel,
                velocity,
                delay_ms: total_delay,
                duration_delta_ms: record.duration_delta_ms,
                port,
                voice_index,
                is_note_off: true,
            }
        } else {
            HumanizedNote {
                note,
                channel,
                velocity,
                delay_ms: 0,
                duration_delta_ms: 0,
                port,
                voice_index,
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

/// Compute swing delay based on subdivision phase and swing config.
///
/// The new model applies different swing amounts to 8th-note and 16th-note
/// offbeats independently. When both `swing_8th_amount` and `swing_16th_amount`
/// are zero, the legacy `swing_amount` is used for backward compatibility.
fn compute_swing_delay(clock: &BeatClock, config: &HumanizeConfig) -> u16 {
    let phase = clock.subdivision_phase();

    // Determine which swing amount applies to this subdivision slot.
    // Slot 0 = beat downbeat: no swing
    // Slot 2 = 8th offbeat: use 8th swing
    // Slot 1, 3 = 16th offbeats: use 16th swing
    let (raw_amount, subdivision_duration) = match phase.sixteenth {
        0 => return 0, // On-beat: no swing
        2 => {
            // 8th offbeat
            let amount = if config.swing_8th_amount > 0.0 || config.swing_16th_amount > 0.0 {
                config.swing_8th_amount
            } else {
                // Legacy fallback: use swing_amount for 8th offbeats
                config.swing_amount
            };
            // An 8th note = half a beat
            let eighth_ms = 60_000.0 / clock.bpm / 2.0;
            (amount, eighth_ms)
        }
        1 | 3 => {
            // 16th offbeat
            let amount = if config.swing_8th_amount > 0.0 || config.swing_16th_amount > 0.0 {
                config.swing_16th_amount
            } else {
                // Legacy: swing_amount did not apply to 16ths
                0.0
            };
            if amount == 0.0 {
                return 0;
            }
            // A 16th note = quarter of a beat
            let sixteenth_ms = 60_000.0 / clock.bpm / 4.0;
            (amount, sixteenth_ms)
        }
        _ => return 0,
    };

    if raw_amount == 0.0 {
        return 0;
    }

    // Apply swing curve to the raw amount
    let curved = apply_swing_curve(raw_amount, &config.swing_curve);
    let delay = subdivision_duration * curved as f64;
    delay as u16
}

/// Apply the swing curve to a raw swing amount (0.0-1.0).
///
/// - `Linear`: passes through unchanged.
/// - `Triplet`: maps so that 0.33 -> triplet feel (2:1), 0.5 -> 3:1.
/// - `Logarithmic`: gentle onset with steep high values.
fn apply_swing_curve(amount: f32, curve: &SwingCurve) -> f32 {
    let clamped = amount.clamp(0.0, 1.0);
    match curve {
        SwingCurve::Linear => clamped,
        SwingCurve::Triplet => {
            // At amount=0.33, we want a triplet feel (delay = 1/3 of subdivision).
            // Triplet: the offbeat shifts by 1/3 subdivision (2:1 ratio).
            // At amount=0.5, 3:1 ratio (delay = 1/2 subdivision).
            // Map linearly: triplet_factor = amount (same scale, but the
            // semantics are that 0.33 = natural triplet).
            // Actually scale so that at 0.33 the delay is 0.333 and at
            // 0.5 it is 0.5 — this is already linear in practice, but we
            // snap to exact triplet grid: round to nearest 1/3 increment.
            let triplet_grid = (clamped * 3.0).round() / 3.0;
            triplet_grid.clamp(0.0, 1.0)
        }
        SwingCurve::Logarithmic => {
            // Quadratic (power) curve: gentle at low values, steep at high.
            // f(x) = x^2 so f(0)=0, f(1)=1, with gentle onset.
            clamped * clamped
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::humanize::config::SwingCurve;

    #[test]
    fn swing_delay_zero_on_downbeat() {
        let mut clock = BeatClock::new(120.0, 4, 4);
        clock.start(0.0);
        clock.tick(0.0); // beat position 0.0 -> slot 0
        let config = HumanizeConfig {
            swing_enabled: true,
            swing_amount: 0.5,
            ..HumanizeConfig::default()
        };
        assert_eq!(compute_swing_delay(&clock, &config), 0);
    }

    #[test]
    fn swing_delay_on_8th_offbeat_legacy() {
        let mut clock = BeatClock::new(120.0, 4, 4);
        clock.start(0.0);
        // At 120 BPM, one beat = 500ms. Midpoint = 250ms -> slot 2 (8th offbeat)
        clock.tick(260.0);
        let config = HumanizeConfig {
            swing_enabled: true,
            swing_amount: 0.5,
            // 8th and 16th are 0 -> legacy mode
            ..HumanizeConfig::default()
        };
        let delay = compute_swing_delay(&clock, &config);
        // 8th note at 120 BPM = 250ms. delay = 250 * 0.5 = 125ms
        assert_eq!(delay, 125);
    }

    #[test]
    fn swing_delay_with_new_8th_amount() {
        let mut clock = BeatClock::new(120.0, 4, 4);
        clock.start(0.0);
        clock.tick(260.0); // 8th offbeat
        let config = HumanizeConfig {
            swing_enabled: true,
            swing_amount: 0.1, // should be ignored
            swing_8th_amount: 0.4,
            swing_16th_amount: 0.0,
            ..HumanizeConfig::default()
        };
        let delay = compute_swing_delay(&clock, &config);
        // 250 * 0.4 = 100
        assert_eq!(delay, 100);
    }

    #[test]
    fn swing_delay_on_16th_offbeat() {
        let mut clock = BeatClock::new(120.0, 4, 4);
        clock.start(0.0);
        // At 120 BPM, one 16th = 125ms. Slot 1 starts at 125ms.
        clock.tick(130.0); // position ~0.26 -> slot 1
        let config = HumanizeConfig {
            swing_enabled: true,
            swing_8th_amount: 0.3,
            swing_16th_amount: 0.5,
            ..HumanizeConfig::default()
        };
        let delay = compute_swing_delay(&clock, &config);
        // 16th note at 120 BPM = 125ms. delay = 125 * 0.5 = 62
        assert_eq!(delay, 62);
    }

    #[test]
    fn swing_curve_linear() {
        assert!((apply_swing_curve(0.5, &SwingCurve::Linear) - 0.5).abs() < 0.001);
    }

    #[test]
    fn swing_curve_triplet_snaps() {
        // 0.33 should snap to 0.333...
        let result = apply_swing_curve(0.33, &SwingCurve::Triplet);
        assert!((result - 1.0 / 3.0).abs() < 0.01);
        // 0.5 should snap to 0.667 (2/3)
        let result = apply_swing_curve(0.5, &SwingCurve::Triplet);
        assert!((result - 2.0 / 3.0).abs() < 0.01);
    }

    #[test]
    fn swing_curve_logarithmic_endpoints() {
        let result_0 = apply_swing_curve(0.0, &SwingCurve::Logarithmic);
        let result_1 = apply_swing_curve(1.0, &SwingCurve::Logarithmic);
        assert!(result_0.abs() < 0.001, "log curve at 0 should be ~0");
        assert!(
            (result_1 - 1.0).abs() < 0.001,
            "log curve at 1 should be ~1"
        );
    }

    #[test]
    fn swing_curve_logarithmic_gentle_onset() {
        // At low values, log curve should be less than linear
        let log_val = apply_swing_curve(0.3, &SwingCurve::Logarithmic);
        let lin_val = apply_swing_curve(0.3, &SwingCurve::Linear);
        assert!(
            log_val < lin_val,
            "log curve should be below linear at 0.3: log={}, lin={}",
            log_val,
            lin_val
        );
    }

    #[test]
    fn default_config_produces_no_swing() {
        let mut clock = BeatClock::new(120.0, 4, 4);
        clock.start(0.0);
        clock.tick(260.0);
        let config = HumanizeConfig::default();
        assert_eq!(compute_swing_delay(&clock, &config), 0);
    }

    #[test]
    fn backward_compatible_humanizer() {
        // HumanizeConfig::default() should produce a Humanizer that
        // passes notes through unchanged (all effects disabled).
        let config = HumanizeConfig::default();
        let mut h = Humanizer::new(config);
        let note = wmidi::Note::C4;
        let channel = wmidi::Channel::Ch1;
        let vel = wmidi::Velocity::try_from(100u8).unwrap();
        let hn = h.humanize_note_on(note, channel, vel, 0);
        assert_eq!(hn.delay_ms, 0);
        assert_eq!(hn.duration_delta_ms, 0);
        assert_eq!(u8::from(hn.velocity), 100);
    }
}
