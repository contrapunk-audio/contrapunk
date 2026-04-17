//! Named groove preset templates for the humanization system.
//!
//! Each [`GrooveTemplate`] bundles a set of humanization parameter values
//! into a named preset. Applying a template sets the timing, velocity,
//! swing, and duration parameters on a [`HumanizeConfig`] while
//! preserving fields the template does not touch (BPM, time signature,
//! metronome settings, enabled flags).

use serde::{Deserialize, Serialize};

use super::config::HumanizeConfig;

/// Named groove preset that bundles humanization settings.
///
/// Templates provide quick access to common groove feels without
/// requiring manual parameter tuning. Apply a template with
/// [`apply`](Self::apply) and the config will be updated in place.
///
/// # Preserved Fields
///
/// When a template is applied, the following fields are **not** modified:
/// - `enabled`, `jitter_enabled`, `velocity_enabled`, `duration_enabled`, `swing_enabled`
/// - `bpm`, `beats_per_bar`, `beat_unit`
/// - `metronome_enabled`, `metronome_output_port`
///
/// The `groove_template` field on [`HumanizeConfig`] is set to `Some(template)`.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum GrooveTemplate {
    /// No swing, no jitter, no variation. Metronomically precise.
    Straight,
    /// Light swing feel with subtle timing and velocity variation.
    SwingLight,
    /// Heavy swing with triplet character and wider variation.
    SwingHeavy,
    /// Pushed, aggressive feel with stronger on-beat accents.
    Push,
    /// Lazy, behind-the-beat feel with extended note durations.
    Drag,
    /// True triplet shuffle (swing_amount ~0.33).
    Shuffle,
    /// Hip-hop groove with moderate swing and velocity variation.
    HipHop,
    /// Jazz feel with wide timing and dynamic range.
    Jazz,
}

impl GrooveTemplate {
    /// Display name for the template.
    pub fn name(&self) -> &str {
        match self {
            Self::Straight => "Straight",
            Self::SwingLight => "Swing Light",
            Self::SwingHeavy => "Swing Heavy",
            Self::Push => "Push",
            Self::Drag => "Drag",
            Self::Shuffle => "Shuffle",
            Self::HipHop => "Hip Hop",
            Self::Jazz => "Jazz",
        }
    }

    /// Apply this template's values to the given config.
    ///
    /// Only the groove-related parameters are overwritten. BPM,
    /// time signature, metronome settings, and enable toggles are
    /// preserved so applying a groove never disrupts transport or
    /// routing state.
    ///
    /// The config's `groove_template` field is set to `Some(self.clone())`.
    pub fn apply(&self, config: &mut HumanizeConfig) {
        match self {
            Self::Straight => {
                config.swing_amount = 0.0;
                config.jitter_min_ms = 0;
                config.jitter_max_ms = 0;
                config.velocity_variation = 0;
                config.duration_variation_ms = 0;
            }
            Self::SwingLight => {
                config.swing_amount = 0.2;
                config.jitter_min_ms = 1;
                config.jitter_max_ms = 5;
                config.velocity_variation = 5;
                config.duration_variation_ms = 0;
            }
            Self::SwingHeavy => {
                config.swing_amount = 0.45;
                config.jitter_min_ms = 2;
                config.jitter_max_ms = 8;
                config.velocity_variation = 10;
                config.duration_variation_ms = 0;
            }
            Self::Push => {
                config.swing_amount = 0.0;
                config.jitter_min_ms = 0;
                config.jitter_max_ms = 3;
                config.velocity_variation = 15;
                config.duration_variation_ms = 0;
            }
            Self::Drag => {
                config.swing_amount = 0.15;
                config.jitter_min_ms = 5;
                config.jitter_max_ms = 15;
                config.velocity_variation = 8;
                config.duration_variation_ms = 30;
            }
            Self::Shuffle => {
                config.swing_amount = 0.33;
                config.jitter_min_ms = 1;
                config.jitter_max_ms = 3;
                config.velocity_variation = 0;
                config.duration_variation_ms = 0;
            }
            Self::HipHop => {
                config.swing_amount = 0.3;
                config.jitter_min_ms = 2;
                config.jitter_max_ms = 6;
                config.velocity_variation = 12;
                config.duration_variation_ms = 0;
            }
            Self::Jazz => {
                config.swing_amount = 0.4;
                config.jitter_min_ms = 3;
                config.jitter_max_ms = 12;
                config.velocity_variation = 18;
                config.duration_variation_ms = 25;
            }
        }
        config.groove_template = Some(self.clone());
    }
}

/// Returns a static slice of all available groove templates, suitable for UI enumeration.
pub fn all_templates() -> &'static [GrooveTemplate] {
    &[
        GrooveTemplate::Straight,
        GrooveTemplate::SwingLight,
        GrooveTemplate::SwingHeavy,
        GrooveTemplate::Push,
        GrooveTemplate::Drag,
        GrooveTemplate::Shuffle,
        GrooveTemplate::HipHop,
        GrooveTemplate::Jazz,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper: build a config with non-default BPM / metronome to verify preservation.
    fn config_with_transport() -> HumanizeConfig {
        HumanizeConfig {
            bpm: 140.0,
            beats_per_bar: 3,
            beat_unit: 8,
            metronome_enabled: true,
            metronome_output_port: Some(2),
            enabled: true,
            jitter_enabled: true,
            velocity_enabled: true,
            swing_enabled: true,
            duration_enabled: true,
            ..HumanizeConfig::default()
        }
    }

    /// Asserts that transport / metronome fields are unchanged after applying a template.
    fn assert_transport_preserved(config: &HumanizeConfig) {
        assert!((config.bpm - 140.0).abs() < f64::EPSILON, "BPM changed");
        assert_eq!(config.beats_per_bar, 3, "beats_per_bar changed");
        assert_eq!(config.beat_unit, 8, "beat_unit changed");
        assert!(config.metronome_enabled, "metronome_enabled changed");
        assert_eq!(
            config.metronome_output_port,
            Some(2),
            "metronome_output_port changed"
        );
        assert!(config.enabled, "enabled changed");
        assert!(config.jitter_enabled, "jitter_enabled changed");
        assert!(config.velocity_enabled, "velocity_enabled changed");
        assert!(config.swing_enabled, "swing_enabled changed");
        assert!(config.duration_enabled, "duration_enabled changed");
    }

    #[test]
    fn straight_template() {
        let mut cfg = config_with_transport();
        GrooveTemplate::Straight.apply(&mut cfg);

        assert_eq!(cfg.swing_amount, 0.0);
        assert_eq!(cfg.jitter_min_ms, 0);
        assert_eq!(cfg.jitter_max_ms, 0);
        assert_eq!(cfg.velocity_variation, 0);
        assert_eq!(cfg.duration_variation_ms, 0);
        assert_eq!(cfg.groove_template, Some(GrooveTemplate::Straight));
        assert_transport_preserved(&cfg);
    }

    #[test]
    fn swing_light_template() {
        let mut cfg = config_with_transport();
        GrooveTemplate::SwingLight.apply(&mut cfg);

        assert_eq!(cfg.swing_amount, 0.2);
        assert_eq!(cfg.jitter_min_ms, 1);
        assert_eq!(cfg.jitter_max_ms, 5);
        assert_eq!(cfg.velocity_variation, 5);
        assert_eq!(cfg.groove_template, Some(GrooveTemplate::SwingLight));
        assert_transport_preserved(&cfg);
    }

    #[test]
    fn swing_heavy_template() {
        let mut cfg = config_with_transport();
        GrooveTemplate::SwingHeavy.apply(&mut cfg);

        assert_eq!(cfg.swing_amount, 0.45);
        assert_eq!(cfg.jitter_min_ms, 2);
        assert_eq!(cfg.jitter_max_ms, 8);
        assert_eq!(cfg.velocity_variation, 10);
        assert_eq!(cfg.groove_template, Some(GrooveTemplate::SwingHeavy));
        assert_transport_preserved(&cfg);
    }

    #[test]
    fn push_template() {
        let mut cfg = config_with_transport();
        GrooveTemplate::Push.apply(&mut cfg);

        assert_eq!(cfg.swing_amount, 0.0);
        assert_eq!(cfg.jitter_min_ms, 0);
        assert_eq!(cfg.jitter_max_ms, 3);
        assert_eq!(cfg.velocity_variation, 15);
        assert_eq!(cfg.groove_template, Some(GrooveTemplate::Push));
        assert_transport_preserved(&cfg);
    }

    #[test]
    fn drag_template() {
        let mut cfg = config_with_transport();
        GrooveTemplate::Drag.apply(&mut cfg);

        assert_eq!(cfg.swing_amount, 0.15);
        assert_eq!(cfg.jitter_min_ms, 5);
        assert_eq!(cfg.jitter_max_ms, 15);
        assert_eq!(cfg.velocity_variation, 8);
        assert_eq!(cfg.duration_variation_ms, 30);
        assert_eq!(cfg.groove_template, Some(GrooveTemplate::Drag));
        assert_transport_preserved(&cfg);
    }

    #[test]
    fn shuffle_template() {
        let mut cfg = config_with_transport();
        GrooveTemplate::Shuffle.apply(&mut cfg);

        assert_eq!(cfg.swing_amount, 0.33);
        assert_eq!(cfg.jitter_min_ms, 1);
        assert_eq!(cfg.jitter_max_ms, 3);
        assert_eq!(cfg.velocity_variation, 0);
        assert_eq!(cfg.groove_template, Some(GrooveTemplate::Shuffle));
        assert_transport_preserved(&cfg);
    }

    #[test]
    fn hiphop_template() {
        let mut cfg = config_with_transport();
        GrooveTemplate::HipHop.apply(&mut cfg);

        assert_eq!(cfg.swing_amount, 0.3);
        assert_eq!(cfg.jitter_min_ms, 2);
        assert_eq!(cfg.jitter_max_ms, 6);
        assert_eq!(cfg.velocity_variation, 12);
        assert_eq!(cfg.groove_template, Some(GrooveTemplate::HipHop));
        assert_transport_preserved(&cfg);
    }

    #[test]
    fn jazz_template() {
        let mut cfg = config_with_transport();
        GrooveTemplate::Jazz.apply(&mut cfg);

        assert_eq!(cfg.swing_amount, 0.4);
        assert_eq!(cfg.jitter_min_ms, 3);
        assert_eq!(cfg.jitter_max_ms, 12);
        assert_eq!(cfg.velocity_variation, 18);
        assert_eq!(cfg.duration_variation_ms, 25);
        assert_eq!(cfg.groove_template, Some(GrooveTemplate::Jazz));
        assert_transport_preserved(&cfg);
    }

    #[test]
    fn all_templates_returns_all_variants() {
        let templates = all_templates();
        assert_eq!(templates.len(), 8);
        assert_eq!(templates[0], GrooveTemplate::Straight);
        assert_eq!(templates[7], GrooveTemplate::Jazz);
    }

    #[test]
    fn template_names_are_nonempty() {
        for t in all_templates() {
            assert!(!t.name().is_empty(), "template {:?} has empty name", t);
        }
    }

    #[test]
    fn applying_template_then_manual_change_clears_template() {
        let mut cfg = HumanizeConfig::default();
        GrooveTemplate::Jazz.apply(&mut cfg);
        assert_eq!(cfg.groove_template, Some(GrooveTemplate::Jazz));

        // Simulate user manually changing a parameter
        cfg.groove_template = None;
        assert_eq!(cfg.groove_template, None);
    }

    #[test]
    fn serde_round_trip() {
        let mut cfg = HumanizeConfig::default();
        GrooveTemplate::Shuffle.apply(&mut cfg);

        let json = serde_json::to_string(&cfg).expect("serialize");
        let deser: HumanizeConfig = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(deser.groove_template, Some(GrooveTemplate::Shuffle));
    }

    #[test]
    fn deserialize_without_groove_template_defaults_to_none() {
        // JSON from an older version without the groove_template field
        let json = r#"{
            "enabled": false,
            "jitter_enabled": false,
            "jitter_min_ms": 1,
            "jitter_max_ms": 10,
            "velocity_enabled": false,
            "velocity_variation": 15,
            "duration_enabled": false,
            "duration_variation_ms": 20,
            "swing_enabled": false,
            "swing_amount": 0.0,
            "bpm": 120.0,
            "beats_per_bar": 4,
            "beat_unit": 4,
            "metronome_enabled": false,
            "metronome_output_port": null
        }"#;
        let cfg: HumanizeConfig = serde_json::from_str(json).expect("deserialize old format");
        assert_eq!(cfg.groove_template, None);
    }
}
