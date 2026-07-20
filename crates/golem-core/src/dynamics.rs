//! Musical interpretation layer for Golem.
//!
//! `AdaptiveDynamics` turns calibrated player features into drummer intent.
//! It deliberately reacts on musical timescales instead of mapping input RMS
//! directly to drum volume.

use crate::follow::FollowInput;

/// High-level drummer intent consumed by the groove engine.
#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct DrummerIntent {
    /// Overall band energy the drummer should match.
    pub energy: f32,
    /// How hard hits should generally be played.
    pub velocity: f32,
    /// How busy subdivisions/ghosts/fills should become.
    pub density: f32,
    /// Short-lived attack to accent the next musical event.
    pub accent: f32,
    /// Phrase-scale pressure that resolves into fills.
    pub fill_tension: f32,
    /// Sustained lift over multiple beats/bars.
    pub section_lift: f32,
    /// How much the drummer should leave space.
    pub restraint: f32,
}

impl Default for DrummerIntent {
    fn default() -> Self {
        Self {
            energy: 0.0,
            velocity: 0.45,
            density: 0.0,
            accent: 0.0,
            fill_tension: 0.0,
            section_lift: 0.0,
            restraint: 1.0,
        }
    }
}

/// Stateful musical dynamics follower.
pub struct AdaptiveDynamics {
    energy: f32,
    density: f32,
    accent: f32,
    fill_tension: f32,
    section_lift: f32,
    restraint: f32,
}

impl Default for AdaptiveDynamics {
    fn default() -> Self {
        Self::new()
    }
}

impl AdaptiveDynamics {
    pub fn new() -> Self {
        Self {
            energy: 0.0,
            density: 0.0,
            accent: 0.0,
            fill_tension: 0.0,
            section_lift: 0.0,
            restraint: 1.0,
        }
    }

    /// Update intent once per audio block. No allocation, locks, or I/O.
    pub fn update(&mut self, input: FollowInput, frames: usize, sample_rate: u32) -> DrummerIntent {
        let dt = frames as f32 / sample_rate.max(1) as f32;
        let confidence = input.confidence.clamp(0.0, 1.0);

        let player_energy =
            (input.normalized_energy * 0.70 + input.energy_fast * 0.20 + input.energy_slow * 0.10)
                .clamp(0.0, 1.0)
                * confidence;
        let player_density = input.strum_density.clamp(0.0, 1.0) * confidence;
        let player_accent = input.onset_strength.clamp(0.0, 1.0) * confidence;

        self.energy = approach(self.energy, player_energy, coeff(dt, 0.55));
        self.density = approach(self.density, player_density, coeff(dt, 0.90));

        // Hold accents long enough for the next 8th/16th to catch them.
        self.accent = (self.accent * (-dt / 0.28).exp()).max(player_accent);

        let lift_target = ((self.energy - 0.48) / 0.42).clamp(0.0, 1.0);
        self.section_lift = approach(self.section_lift, lift_target, coeff(dt, 4.0));

        let restraint_target =
            (1.0 - (self.energy * 1.15 + self.density * 0.35).clamp(0.0, 1.0)).clamp(0.0, 1.0);
        self.restraint = approach(self.restraint, restraint_target, coeff(dt, 1.8));

        // Fills should build slowly from sustained energy/density and release
        // later at phrase boundaries in the groove engine.
        let tension_drive =
            (self.energy * 0.45 + self.density * 0.55 + self.section_lift * 0.35).clamp(0.0, 1.0);
        self.fill_tension += tension_drive * dt * 0.22;
        self.fill_tension -= self.restraint * dt * 0.06;
        self.fill_tension = self.fill_tension.clamp(0.0, 1.0);

        let velocity = (0.42 + self.energy * 0.42 + self.section_lift * 0.18 + self.accent * 0.10
            - self.restraint * 0.12)
            .clamp(0.20, 1.0);

        DrummerIntent {
            energy: self.energy,
            velocity,
            density: self.density,
            accent: self.accent,
            fill_tension: self.fill_tension,
            section_lift: self.section_lift,
            restraint: self.restraint,
        }
    }

    /// Called by the groove engine when a phrase-end fill resolves.
    pub fn consume_fill(&mut self, amount: f32) {
        self.fill_tension = (self.fill_tension - amount).clamp(0.0, 1.0);
    }
}

fn coeff(dt: f32, tau_seconds: f32) -> f32 {
    if tau_seconds <= 0.0 {
        1.0
    } else {
        (1.0 - (-dt / tau_seconds).exp()).clamp(0.0, 1.0)
    }
}

fn approach(current: f32, target: f32, coeff: f32) -> f32 {
    current + (target - current) * coeff.clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loud_playing_builds_energy_and_reduces_restraint() {
        let mut dynamics = AdaptiveDynamics::new();
        let input = FollowInput {
            normalized_energy: 0.8,
            energy_fast: 0.8,
            energy_slow: 0.7,
            onset_strength: 0.1,
            strum_density: 0.5,
            confidence: 1.0,
            ..FollowInput::default()
        };

        let mut intent = DrummerIntent::default();
        for _ in 0..900 {
            intent = dynamics.update(input, 128, 48_000);
        }

        assert!(intent.energy > 0.25);
        assert!(intent.restraint < 0.9);
        assert!(intent.velocity > 0.45);
    }

    #[test]
    fn onset_creates_short_lived_accent() {
        let mut dynamics = AdaptiveDynamics::new();
        let accent = dynamics.update(
            FollowInput {
                normalized_energy: 0.4,
                energy_fast: 0.5,
                energy_slow: 0.3,
                onset_strength: 0.9,
                confidence: 1.0,
                ..FollowInput::default()
            },
            128,
            48_000,
        );
        assert!(accent.accent > 0.8);

        let later = dynamics.update(FollowInput::default(), 48_000, 48_000);
        assert!(later.accent < accent.accent);
    }
}
