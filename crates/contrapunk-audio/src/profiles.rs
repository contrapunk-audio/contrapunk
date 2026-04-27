//! Mic profile save/load with built-in presets.

use serde::{Deserialize, Serialize};

use super::config::MicConfig;

/// A saved microphone configuration profile.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MicProfile {
    /// Profile name (e.g., "Quiet Studio", "Noisy Room")
    pub name: String,
    /// Microphone configuration
    pub config: MicConfig,
    /// Whether this is a built-in profile (cannot be deleted)
    pub is_builtin: bool,
}

impl MicProfile {
    /// Create a new custom profile.
    pub fn new(name: impl Into<String>, config: MicConfig) -> Self {
        Self {
            name: name.into(),
            config,
            is_builtin: false,
        }
    }

    /// Create a built-in profile.
    pub fn builtin(name: impl Into<String>, config: MicConfig) -> Self {
        Self {
            name: name.into(),
            config,
            is_builtin: true,
        }
    }
}

/// Returns the default built-in mic profiles.
pub fn builtin_profiles() -> Vec<MicProfile> {
    vec![
        MicProfile::builtin("Balanced", MicConfig::default()),
        MicProfile::builtin(
            "Quiet Studio",
            MicConfig {
                volume_threshold: 0.005,
                confidence_threshold: 0.8,
                buffer_size: 2048,
                ..MicConfig::default()
            },
        ),
        MicProfile::builtin(
            "Noisy Room",
            MicConfig {
                volume_threshold: 0.05,
                confidence_threshold: 0.6,
                buffer_size: 2048,
                ..MicConfig::default()
            },
        ),
        MicProfile::builtin(
            "Fast Playing",
            MicConfig {
                buffer_size: 1024,
                min_note_duration_ms: 30,
                hysteresis_cents: 30,
                ..MicConfig::default()
            },
        ),
        MicProfile::builtin(
            "Ballad",
            MicConfig {
                buffer_size: 4096,
                min_note_duration_ms: 80,
                hysteresis_cents: 50,
                confidence_threshold: 0.85,
                ..MicConfig::default()
            },
        ),
    ]
}
