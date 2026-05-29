//! Lightweight guitar-follow feature extraction.
//!
//! v0.1 intentionally uses simple realtime-safe features: RMS, onset
//! strength, and smoothed strum density. This is enough for Golem to
//! react to guitar energy without attempting pitch/chord transcription.

/// Features consumed by the drummer brain.
#[derive(Clone, Copy, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct FollowInput {
    pub guitar_rms: f32,
    pub onset_strength: f32,
    pub strum_density: f32,
    pub confidence: f32,
}

/// Realtime-safe guitar feature follower.
pub struct Follower {
    sample_rate: u32,
    fast_energy: f32,
    slow_energy: f32,
    density: f32,
    cooldown_remaining: u32,
}

impl Follower {
    pub fn new(sample_rate: u32) -> Self {
        Self {
            sample_rate: sample_rate.max(1),
            fast_energy: 0.0,
            slow_energy: 0.0,
            density: 0.0,
            cooldown_remaining: 0,
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
        if channels == 0 || input.is_empty() {
            return FollowInput::default();
        }

        let mut sum = 0.0f32;
        let mut frames = 0usize;
        for frame in input.chunks(channels) {
            let x = frame.get(channel).copied().unwrap_or(0.0) * input_gain;
            sum += x * x;
            frames += 1;
        }

        if frames == 0 {
            return FollowInput::default();
        }

        let rms = (sum / frames as f32).sqrt().clamp(0.0, 1.0);
        let energy = rms;

        // Fast/slow envelope follower. Difference between the two is a
        // cheap onset proxy that works well for strums.
        self.fast_energy += (energy - self.fast_energy) * 0.35;
        self.slow_energy += (energy - self.slow_energy) * 0.035;

        let mut onset = ((self.fast_energy - self.slow_energy * 1.35) * 9.0).max(0.0);
        if self.cooldown_remaining > 0 {
            let step = frames.min(self.cooldown_remaining as usize) as u32;
            self.cooldown_remaining = self.cooldown_remaining.saturating_sub(step);
            onset = 0.0;
        }

        if onset > 0.08 {
            self.cooldown_remaining = (self.sample_rate as f32 * 0.045) as u32;
            self.density = (self.density + onset * 0.45).clamp(0.0, 1.0);
        }

        // Density decays over roughly one second, adjusted by block size.
        let decay = (-(frames as f32) / (self.sample_rate as f32 * 0.85)).exp();
        self.density *= decay;

        FollowInput {
            guitar_rms: rms,
            onset_strength: onset.clamp(0.0, 1.0),
            strum_density: self.density.clamp(0.0, 1.0),
            confidence: if rms > 0.004 { 1.0 } else { 0.0 },
        }
    }
}
