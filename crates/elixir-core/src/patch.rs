//! Chapter 1 and 2 sound controls shared by every Elixir surface.

pub const PARTIAL_COUNT: usize = 6;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum HarmonicPreset {
    Sine = 0,
    Three = 1,
    Odd = 2,
    Saw = 3,
    Dark = 4,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HarmonicRecipe {
    pub amplitudes: [f32; PARTIAL_COUNT],
    /// Phase offsets in turns, where one turn is one complete cycle.
    pub phases: [f32; PARTIAL_COUNT],
}

impl HarmonicRecipe {
    pub const fn preset(preset: HarmonicPreset) -> Self {
        let amplitudes = match preset {
            HarmonicPreset::Sine => [1.0, 0.0, 0.0, 0.0, 0.0, 0.0],
            HarmonicPreset::Three => [1.0, 0.5, 0.25, 0.0, 0.0, 0.0],
            HarmonicPreset::Odd => [1.0, 0.0, 0.45, 0.0, 0.25, 0.0],
            HarmonicPreset::Saw => [1.0, 0.5, 0.333, 0.25, 0.2, 0.167],
            HarmonicPreset::Dark => [1.0, 0.25, 0.111, 0.063, 0.04, 0.028],
        };
        Self {
            amplitudes,
            phases: [0.0; PARTIAL_COUNT],
        }
    }

    pub fn sanitized(self) -> Self {
        Self {
            amplitudes: self
                .amplitudes
                .map(|value| finite_clamp(value, 0.0, 1.0, 0.0)),
            phases: self.phases.map(wrap_turn),
        }
    }

    pub(crate) fn energy(self) -> f32 {
        let sum = self
            .amplitudes
            .iter()
            .map(|amplitude| amplitude * amplitude)
            .sum::<f32>();
        let energy = libm::sqrtf(sum);
        if energy > 1.0e-6 {
            energy
        } else {
            1.0
        }
    }

    pub(crate) fn smooth_toward(&mut self, target: Self, amount: f32) {
        for index in 0..PARTIAL_COUNT {
            self.amplitudes[index] += (target.amplitudes[index] - self.amplitudes[index]) * amount;
            let delta = shortest_turn(target.phases[index] - self.phases[index]);
            self.phases[index] = wrap_turn(self.phases[index] + delta * amount);
        }
    }
}

impl Default for HarmonicRecipe {
    fn default() -> Self {
        Self::preset(HarmonicPreset::Sine)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(u8)]
pub enum CombineMode {
    #[default]
    PrimaryOnly = 0,
    Add = 1,
    Ring = 2,
}

impl CombineMode {
    pub const ALL: [Self; 3] = [Self::PrimaryOnly, Self::Add, Self::Ring];

    pub const fn from_index(index: u8) -> Option<Self> {
        match index {
            0 => Some(Self::PrimaryOnly),
            1 => Some(Self::Add),
            2 => Some(Self::Ring),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SecondaryOscillator {
    pub mode: CombineMode,
    pub semitones: f32,
    pub fine_cents: f32,
    /// Phase offset in turns.
    pub phase: f32,
    pub level: f32,
}

impl SecondaryOscillator {
    pub fn sanitized(self) -> Self {
        Self {
            mode: self.mode,
            semitones: finite_clamp(self.semitones, -24.0, 24.0, 0.0),
            fine_cents: finite_clamp(self.fine_cents, -100.0, 100.0, 0.0),
            phase: wrap_turn(self.phase),
            level: finite_clamp(self.level, 0.0, 1.0, 1.0),
        }
    }

    pub(crate) fn smooth_toward(&mut self, target: Self, amount: f32) {
        self.mode = target.mode;
        self.semitones += (target.semitones - self.semitones) * amount;
        self.fine_cents += (target.fine_cents - self.fine_cents) * amount;
        self.phase = wrap_turn(self.phase + shortest_turn(target.phase - self.phase) * amount);
        self.level += (target.level - self.level) * amount;
    }
}

impl Default for SecondaryOscillator {
    fn default() -> Self {
        Self {
            mode: CombineMode::PrimaryOnly,
            semitones: 0.0,
            fine_cents: 0.0,
            phase: 0.0,
            level: 1.0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AmpEnvelope {
    pub attack_secs: f32,
    pub decay_secs: f32,
    /// Sustain is a level, not a duration.
    pub sustain_level: f32,
    pub release_secs: f32,
    pub velocity_sensitivity: f32,
    pub expression_sensitivity: f32,
}

impl AmpEnvelope {
    pub const fn ring_down() -> Self {
        Self {
            attack_secs: 0.005,
            decay_secs: 1.2,
            sustain_level: 0.0,
            release_secs: 0.012,
            velocity_sensitivity: 1.0,
            expression_sensitivity: 1.0,
        }
    }

    pub const fn maintained() -> Self {
        Self {
            attack_secs: 0.08,
            decay_secs: 0.0,
            sustain_level: 1.0,
            release_secs: 0.12,
            velocity_sensitivity: 1.0,
            expression_sensitivity: 1.0,
        }
    }

    pub fn sanitized(self) -> Self {
        Self {
            attack_secs: finite_clamp(self.attack_secs, 0.0, 5.0, 0.005),
            decay_secs: finite_clamp(self.decay_secs, 0.0, 5.0, 0.0),
            sustain_level: finite_clamp(self.sustain_level, 0.0, 1.0, 1.0),
            release_secs: finite_clamp(self.release_secs, 0.0, 10.0, 0.005),
            velocity_sensitivity: finite_clamp(self.velocity_sensitivity, 0.0, 1.0, 1.0),
            expression_sensitivity: finite_clamp(self.expression_sensitivity, 0.0, 1.0, 1.0),
        }
    }

    pub(crate) fn smooth_toward(&mut self, target: Self, amount: f32) {
        self.attack_secs += (target.attack_secs - self.attack_secs) * amount;
        self.decay_secs += (target.decay_secs - self.decay_secs) * amount;
        self.sustain_level += (target.sustain_level - self.sustain_level) * amount;
        self.release_secs += (target.release_secs - self.release_secs) * amount;
        self.velocity_sensitivity +=
            (target.velocity_sensitivity - self.velocity_sensitivity) * amount;
        self.expression_sensitivity +=
            (target.expression_sensitivity - self.expression_sensitivity) * amount;
    }
}

impl Default for AmpEnvelope {
    fn default() -> Self {
        Self {
            attack_secs: 0.005,
            decay_secs: 0.0,
            sustain_level: 1.0,
            release_secs: 0.005,
            velocity_sensitivity: 1.0,
            expression_sensitivity: 1.0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Vibrato {
    pub rate_hz: f32,
    pub depth_cents: f32,
    pub mod_wheel_depth_cents: f32,
}

impl Vibrato {
    pub fn sanitized(self) -> Self {
        Self {
            rate_hz: finite_clamp(self.rate_hz, 1.0, 8.0, 5.0),
            depth_cents: finite_clamp(self.depth_cents, 0.0, 50.0, 0.0),
            mod_wheel_depth_cents: finite_clamp(self.mod_wheel_depth_cents, 0.0, 50.0, 0.0),
        }
    }

    pub(crate) fn smooth_toward(&mut self, target: Self, amount: f32) {
        self.rate_hz += (target.rate_hz - self.rate_hz) * amount;
        self.depth_cents += (target.depth_cents - self.depth_cents) * amount;
        self.mod_wheel_depth_cents +=
            (target.mod_wheel_depth_cents - self.mod_wheel_depth_cents) * amount;
    }
}

impl Default for Vibrato {
    fn default() -> Self {
        Self {
            rate_hz: 5.0,
            depth_cents: 0.0,
            mod_wheel_depth_cents: 0.0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RolePatch {
    pub harmonics: HarmonicRecipe,
    pub secondary: SecondaryOscillator,
    pub envelope: AmpEnvelope,
    pub vibrato: Vibrato,
}

impl RolePatch {
    pub const fn sine() -> Self {
        Self {
            harmonics: HarmonicRecipe::preset(HarmonicPreset::Sine),
            secondary: SecondaryOscillator {
                mode: CombineMode::PrimaryOnly,
                semitones: 0.0,
                fine_cents: 0.0,
                phase: 0.0,
                level: 1.0,
            },
            envelope: AmpEnvelope {
                attack_secs: 0.005,
                decay_secs: 0.0,
                sustain_level: 1.0,
                release_secs: 0.005,
                velocity_sensitivity: 1.0,
                expression_sensitivity: 1.0,
            },
            vibrato: Vibrato {
                rate_hz: 5.0,
                depth_cents: 0.0,
                mod_wheel_depth_cents: 0.0,
            },
        }
    }

    pub fn sanitized(self) -> Self {
        Self {
            harmonics: self.harmonics.sanitized(),
            secondary: self.secondary.sanitized(),
            envelope: self.envelope.sanitized(),
            vibrato: self.vibrato.sanitized(),
        }
    }

    pub(crate) fn smooth_toward(&mut self, target: Self, amount: f32) {
        self.harmonics.smooth_toward(target.harmonics, amount);
        self.secondary.smooth_toward(target.secondary, amount);
        self.envelope.smooth_toward(target.envelope, amount);
        self.vibrato.smooth_toward(target.vibrato, amount);
    }
}

impl Default for RolePatch {
    fn default() -> Self {
        Self::sine()
    }
}

fn finite_clamp(value: f32, minimum: f32, maximum: f32, fallback: f32) -> f32 {
    if value.is_finite() {
        value.clamp(minimum, maximum)
    } else {
        fallback
    }
}

fn wrap_turn(value: f32) -> f32 {
    if !value.is_finite() {
        return 0.0;
    }
    value - libm::floorf(value)
}

fn shortest_turn(value: f32) -> f32 {
    value - libm::floorf(value + 0.5)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn published_recipes_are_exact() {
        assert_eq!(
            HarmonicRecipe::preset(HarmonicPreset::Three).amplitudes,
            [1.0, 0.5, 0.25, 0.0, 0.0, 0.0]
        );
        assert_eq!(
            HarmonicRecipe::preset(HarmonicPreset::Odd).amplitudes,
            [1.0, 0.0, 0.45, 0.0, 0.25, 0.0]
        );
        assert_eq!(
            HarmonicRecipe::preset(HarmonicPreset::Saw).amplitudes,
            [1.0, 0.5, 0.333, 0.25, 0.2, 0.167]
        );
        assert_eq!(
            HarmonicRecipe::preset(HarmonicPreset::Dark).amplitudes,
            [1.0, 0.25, 0.111, 0.063, 0.04, 0.028]
        );
    }

    #[test]
    fn invalid_patch_values_collapse_to_safe_ranges() {
        let patch = RolePatch {
            harmonics: HarmonicRecipe {
                amplitudes: [f32::NAN, -1.0, 2.0, 0.0, 0.0, 0.0],
                phases: [f32::INFINITY, -0.25, 1.25, 0.0, 0.0, 0.0],
            },
            secondary: SecondaryOscillator {
                semitones: 99.0,
                fine_cents: -999.0,
                level: f32::NAN,
                ..SecondaryOscillator::default()
            },
            envelope: AmpEnvelope {
                sustain_level: 2.0,
                ..AmpEnvelope::default()
            },
            vibrato: Vibrato {
                depth_cents: f32::INFINITY,
                ..Vibrato::default()
            },
        }
        .sanitized();
        assert_eq!(patch.harmonics.amplitudes[..3], [0.0, 0.0, 1.0]);
        assert_eq!(patch.harmonics.phases[..3], [0.0, 0.75, 0.25]);
        assert_eq!(patch.secondary.semitones, 24.0);
        assert_eq!(patch.secondary.fine_cents, -100.0);
        assert_eq!(patch.secondary.level, 1.0);
        assert_eq!(patch.envelope.sustain_level, 1.0);
        assert_eq!(patch.vibrato.depth_cents, 0.0);
    }
}
