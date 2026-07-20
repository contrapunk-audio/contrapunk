//! Lightweight guitar-follow feature extraction.
//!
//! v0.1 uses realtime-safe calibrated level features: raw level, adaptive
//! noise floor, normalized performance energy, onset strength, and smoothed
//! density. Pitch/chord transcription stays out of the audio callback.

const MIN_LEVEL: f32 = 1.0e-6;

/// Raw selected-channel input level before musical normalization.
#[derive(Clone, Copy, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct RawInputLevel {
    pub rms: f32,
    pub peak: f32,
    pub rms_db: f32,
    pub peak_db: f32,
    pub channel: usize,
    pub clipping: bool,
}

/// Adaptive calibration state for the current input device/channel.
#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct InputCalibration {
    pub noise_floor: f32,
    pub noise_floor_db: f32,
    pub playing_peak: f32,
    pub playing_peak_db: f32,
    pub calibrated: bool,
}

impl Default for InputCalibration {
    fn default() -> Self {
        Self {
            noise_floor: 0.00003,
            noise_floor_db: amp_to_db(0.00003),
            playing_peak: 0.002,
            playing_peak_db: amp_to_db(0.002),
            calibrated: false,
        }
    }
}

/// Calibrated player features. These are musical inputs, not drum decisions.
#[derive(Clone, Copy, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct PlayerFeatures {
    pub raw: RawInputLevel,
    pub calibration: InputCalibration,
    pub normalized_energy: f32,
    pub energy_fast: f32,
    pub energy_slow: f32,
    pub onset: f32,
    pub density: f32,
    pub confidence: f32,
}

/// Features consumed by the drummer brain.
///
/// The first four fields preserve the original v0.1 host API. `guitar_rms`
/// now represents calibrated performance energy rather than raw electrical RMS.
#[derive(Clone, Copy, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct FollowInput {
    pub guitar_rms: f32,
    pub onset_strength: f32,
    pub strum_density: f32,
    pub confidence: f32,
    pub raw_rms: f32,
    pub raw_peak: f32,
    pub raw_rms_db: f32,
    pub raw_peak_db: f32,
    pub normalized_energy: f32,
    pub energy_fast: f32,
    pub energy_slow: f32,
    pub noise_floor_db: f32,
    pub clipping: bool,
}

impl From<PlayerFeatures> for FollowInput {
    fn from(features: PlayerFeatures) -> Self {
        Self {
            guitar_rms: features.normalized_energy,
            onset_strength: features.onset,
            strum_density: features.density,
            confidence: features.confidence,
            raw_rms: features.raw.rms,
            raw_peak: features.raw.peak,
            raw_rms_db: features.raw.rms_db,
            raw_peak_db: features.raw.peak_db,
            normalized_energy: features.normalized_energy,
            energy_fast: features.energy_fast,
            energy_slow: features.energy_slow,
            noise_floor_db: features.calibration.noise_floor_db,
            clipping: features.raw.clipping,
        }
    }
}

/// Realtime-safe guitar feature follower.
pub struct Follower {
    sample_rate: u32,
    fast_energy: f32,
    slow_energy: f32,
    density: f32,
    onset_hold: f32,
    cooldown_remaining: u32,
    noise_floor: f32,
    playing_peak: f32,
    calibrated_blocks: u32,
}

impl Follower {
    pub fn new(sample_rate: u32) -> Self {
        let calibration = InputCalibration::default();
        Self {
            sample_rate: sample_rate.max(1),
            fast_energy: 0.0,
            slow_energy: 0.0,
            density: 0.0,
            onset_hold: 0.0,
            cooldown_remaining: 0,
            noise_floor: calibration.noise_floor,
            playing_peak: calibration.playing_peak,
            calibrated_blocks: 0,
        }
    }

    pub fn set_sample_rate(&mut self, sample_rate: u32) {
        self.sample_rate = sample_rate.max(1);
    }

    /// Analyze one interleaved f32 audio block.
    pub fn process_interleaved_f32(
        &mut self,
        input: &[f32],
        channels: usize,
        channel: usize,
        input_gain: f32,
    ) -> FollowInput {
        self.process_features_interleaved_f32(input, channels, channel, input_gain)
            .into()
    }

    pub fn process_features_interleaved_f32(
        &mut self,
        input: &[f32],
        channels: usize,
        channel: usize,
        input_gain: f32,
    ) -> PlayerFeatures {
        if channels == 0 || input.is_empty() {
            return PlayerFeatures::default();
        }

        let mut sum = 0.0f32;
        let mut peak = 0.0f32;
        let mut frames = 0usize;
        for frame in input.chunks(channels) {
            let x = frame.get(channel).copied().unwrap_or(0.0) * input_gain;
            let abs = x.abs();
            sum += x * x;
            peak = peak.max(abs);
            frames += 1;
        }

        if frames == 0 {
            return PlayerFeatures::default();
        }

        let rms = (sum / frames as f32).sqrt().clamp(0.0, 1.0);
        let raw = RawInputLevel {
            rms,
            peak: peak.clamp(0.0, 1.0),
            rms_db: amp_to_db(rms),
            peak_db: amp_to_db(peak),
            channel,
            clipping: peak > 0.98,
        };

        let dt = frames as f32 / self.sample_rate as f32;
        self.update_noise_floor(rms, dt);

        // Use a calibrated dB window instead of a direct RMS ratio. This keeps
        // quiet interfaces visible, avoids "every first note is max volume",
        // and still normalizes against the local noise floor.
        let floor_db = amp_to_db(self.noise_floor) + 8.0;
        let peak_db = amp_to_db(self.playing_peak).max(floor_db + 48.0);
        let range_db = (peak_db - floor_db).clamp(36.0, 60.0);
        let normalized_energy = ((raw.rms_db - floor_db) / range_db)
            .clamp(0.0, 1.0)
            .powf(0.72);

        self.update_playing_peak(rms, dt);

        self.fast_energy += (normalized_energy - self.fast_energy) * 0.34;
        self.slow_energy += (normalized_energy - self.slow_energy) * 0.045;

        let raw_onset = ((self.fast_energy - self.slow_energy * 1.10) * 2.9).max(0.0);
        let mut onset_trigger = raw_onset;
        if self.cooldown_remaining > 0 {
            let step = frames.min(self.cooldown_remaining as usize) as u32;
            self.cooldown_remaining = self.cooldown_remaining.saturating_sub(step);
            onset_trigger = 0.0;
        }

        if onset_trigger > 0.055 {
            self.cooldown_remaining = (self.sample_rate as f32 * 0.045) as u32;
            self.density = (self.density + onset_trigger * 0.42).clamp(0.0, 1.0);
        }

        let density_decay = (-(frames as f32) / (self.sample_rate as f32 * 1.05)).exp();
        self.density *= density_decay;

        // UI telemetry arrives every 100ms, while an onset can last one audio
        // callback. Hold a short meter/accent pulse so real strums are visible
        // and musically catchable without doing heavy work on the UI thread.
        let onset_decay = (-(frames as f32) / (self.sample_rate as f32 * 0.16)).exp();
        self.onset_hold = (self.onset_hold * onset_decay).max(onset_trigger.clamp(0.0, 1.0));
        let onset = self.onset_hold;

        let calibration = InputCalibration {
            noise_floor: self.noise_floor,
            noise_floor_db: amp_to_db(self.noise_floor),
            playing_peak: self.playing_peak,
            playing_peak_db: amp_to_db(self.playing_peak),
            calibrated: self.calibrated_blocks > 40,
        };

        let confidence = if normalized_energy > 0.025 && rms > self.noise_floor * 1.8 {
            1.0
        } else if normalized_energy > 0.008 {
            0.45
        } else {
            0.0
        };

        PlayerFeatures {
            raw,
            calibration,
            normalized_energy,
            energy_fast: self.fast_energy.clamp(0.0, 1.0),
            energy_slow: self.slow_energy.clamp(0.0, 1.0),
            onset: onset.clamp(0.0, 1.0),
            density: self.density.clamp(0.0, 1.0),
            confidence,
        }
    }

    fn update_noise_floor(&mut self, rms: f32, dt: f32) {
        let rms = rms.max(MIN_LEVEL);
        self.calibrated_blocks = self.calibrated_blocks.saturating_add(1);

        // Learn noise only from blocks that are plausibly quiet. This keeps
        // hum/room noise out while avoiding the classic mistake of learning
        // actual playing as the floor.
        let quiet_threshold = (self.noise_floor * 5.0).max(0.00018);
        if rms < quiet_threshold {
            let c = (1.0 - (-dt / 1.8).exp()).clamp(0.0, 0.08);
            self.noise_floor += (rms - self.noise_floor) * c;
        } else {
            // Very slow upward creep for changing rooms/interfaces.
            let c = (1.0 - (-dt / 22.0).exp()).clamp(0.0, 0.003);
            self.noise_floor += (rms.min(self.noise_floor * 1.5) - self.noise_floor) * c;
        }
        self.noise_floor = self.noise_floor.clamp(MIN_LEVEL, 0.08);
    }

    fn update_playing_peak(&mut self, rms: f32, dt: f32) {
        let rms = rms.max(MIN_LEVEL);
        if rms > self.noise_floor * 2.5 + 0.00008 {
            let attack = (1.0 - (-dt / 1.4).exp()).clamp(0.0, 0.018);
            self.playing_peak += (rms - self.playing_peak).max(0.0) * attack;
        }
        let peak_decay = (-dt / 18.0).exp();
        let min_peak = (self.noise_floor * 10.0 + 0.0012).min(0.12);
        self.playing_peak = (self.playing_peak * peak_decay)
            .max(min_peak)
            .clamp(MIN_LEVEL, 1.0);
    }
}

pub fn amp_to_db(value: f32) -> f32 {
    20.0 * value.max(MIN_LEVEL).log10()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalized_energy_rises_for_louder_signal() {
        let mut follower = Follower::new(48_000);
        let quiet = [0.0002f32; 256];
        for _ in 0..80 {
            follower.process_interleaved_f32(&quiet, 1, 0, 1.0);
        }

        let soft = [0.004f32; 256];
        let hard = [0.03f32; 256];
        let soft_features = follower.process_interleaved_f32(&soft, 1, 0, 1.0);
        let hard_features = follower.process_interleaved_f32(&hard, 1, 0, 1.0);

        assert!(hard_features.normalized_energy > soft_features.normalized_energy);
        assert!(hard_features.confidence > 0.0);
    }

    #[test]
    fn silence_stays_below_confidence_gate() {
        let mut follower = Follower::new(48_000);
        let silence = [0.0f32; 256];
        let features = follower.process_interleaved_f32(&silence, 1, 0, 1.0);
        assert_eq!(features.confidence, 0.0);
        assert!(features.normalized_energy <= 0.001);
    }
}
