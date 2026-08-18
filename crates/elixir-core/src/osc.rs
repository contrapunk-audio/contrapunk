//! Phase-continuous harmonic oscillator and constrained Chapter 2 interaction.

use crate::{CombineMode, RolePatch, PARTIAL_COUNT};

const TAU: f32 = core::f32::consts::TAU;

/// One phase-coherent harmonic source plus one secondary sine source.
pub(crate) struct Oscillator {
    primary_phase: f32,
    secondary_phase: f32,
    vibrato_phase: f32,
    mode_mix: [f32; 3],
}

impl Oscillator {
    pub const fn new() -> Self {
        Self {
            primary_phase: 0.0,
            secondary_phase: 0.0,
            vibrato_phase: 0.0,
            mode_mix: [1.0, 0.0, 0.0],
        }
    }

    pub fn start(&mut self, patch: RolePatch) {
        self.primary_phase = 0.0;
        self.secondary_phase = 0.0;
        self.vibrato_phase = 0.0;
        self.mode_mix = mode_weights(patch.secondary.mode);
    }

    #[inline]
    pub fn tick(
        &mut self,
        base_frequency_hz: f32,
        patch: RolePatch,
        pitch_bend_cents: f32,
        mod_wheel: f32,
        sample_rate: f32,
        smoothing: f32,
    ) -> f32 {
        let vibrato_cents =
            patch.vibrato.depth_cents + patch.vibrato.mod_wheel_depth_cents * mod_wheel;
        let vibrato = vibrato_cents * libm::sinf(TAU * self.vibrato_phase);
        let frequency_hz =
            base_frequency_hz * libm::powf(2.0, (pitch_bend_cents + vibrato) / 1200.0);
        let secondary_ratio = libm::powf(
            2.0,
            (patch.secondary.semitones + patch.secondary.fine_cents / 100.0) / 12.0,
        );
        let secondary_frequency_hz = frequency_hz * secondary_ratio;
        let nyquist = sample_rate * 0.5;
        let energy = patch.harmonics.energy();
        let secondary_angle = TAU * (self.secondary_phase + patch.secondary.phase);

        let mut primary = 0.0;
        let mut ring = 0.0;
        for index in 0..PARTIAL_COUNT {
            let harmonic = (index + 1) as f32;
            let amplitude = patch.harmonics.amplitudes[index];
            if amplitude <= 0.0 {
                continue;
            }
            let harmonic_frequency = harmonic * frequency_hz;
            let primary_angle =
                TAU * (harmonic * self.primary_phase + patch.harmonics.phases[index]);
            if harmonic_frequency < nyquist {
                primary += amplitude * libm::sinf(primary_angle);
            }

            let difference_frequency = (harmonic_frequency - secondary_frequency_hz).abs();
            let sum_frequency = harmonic_frequency + secondary_frequency_hz;
            if difference_frequency < nyquist {
                ring += 0.5
                    * amplitude
                    * patch.secondary.level
                    * libm::cosf(primary_angle - secondary_angle);
            }
            if sum_frequency < nyquist {
                ring -= 0.5
                    * amplitude
                    * patch.secondary.level
                    * libm::cosf(primary_angle + secondary_angle);
            }
        }
        primary /= energy;
        ring /= energy;

        let secondary = if secondary_frequency_hz < nyquist {
            patch.secondary.level * libm::sinf(secondary_angle)
        } else {
            0.0
        };
        let add = primary + secondary;

        let target_mix = mode_weights(patch.secondary.mode);
        for (current, target) in self.mode_mix.iter_mut().zip(target_mix) {
            *current += (target - *current) * smoothing;
        }
        let sample = self.mode_mix[0] * primary + self.mode_mix[1] * add + self.mode_mix[2] * ring;

        self.primary_phase = advance(self.primary_phase, frequency_hz / sample_rate);
        self.secondary_phase = advance(self.secondary_phase, secondary_frequency_hz / sample_rate);
        self.vibrato_phase = advance(self.vibrato_phase, patch.vibrato.rate_hz / sample_rate);
        sample
    }

    #[cfg(test)]
    pub fn primary_phase(&self) -> f32 {
        self.primary_phase
    }
}

const fn mode_weights(mode: CombineMode) -> [f32; 3] {
    match mode {
        CombineMode::PrimaryOnly => [1.0, 0.0, 0.0],
        CombineMode::Add => [0.0, 1.0, 0.0],
        CombineMode::Ring => [0.0, 0.0, 1.0],
    }
}

fn advance(phase: f32, increment: f32) -> f32 {
    let next = phase + increment;
    next - libm::floorf(next)
}
