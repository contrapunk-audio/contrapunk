//! Minimal persisted Elixir state.
//!
//! Schema v3 stores only controls that still affect the fixed sine engine.
//! Older spectral/filter/envelope/modulation/FX documents migrate to these
//! defaults while preserving valid master and role gains.

use std::fmt;

use elixir_core::{Engine, VoiceRole};
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const CURRENT_SCHEMA_VERSION: u32 = 3;
const DEFAULT_MASTER_GAIN: f32 = 0.25;
const DEFAULT_ROLE_GAINS: [f32; 4] = [1.0; 4];

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct ElixirState {
    pub master_gain: f32,
    pub role_gains: [f32; 4],
}

impl Default for ElixirState {
    fn default() -> Self {
        Self {
            master_gain: DEFAULT_MASTER_GAIN,
            role_gains: DEFAULT_ROLE_GAINS,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct ElixirPreset {
    pub schema_version: u32,
    pub name: String,
    pub author: Option<String>,
    pub state: ElixirState,
}

impl Default for ElixirPreset {
    fn default() -> Self {
        Self {
            schema_version: CURRENT_SCHEMA_VERSION,
            name: "Sine".into(),
            author: None,
            state: ElixirState::default(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PresetStateError {
    UnsupportedSchema(u32),
    Invalid(String),
    Json(String),
}

impl fmt::Display for PresetStateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedSchema(version) => write!(f, "unsupported preset schema {version}"),
            Self::Invalid(message) => write!(f, "invalid preset state: {message}"),
            Self::Json(message) => write!(f, "preset JSON error: {message}"),
        }
    }
}

impl std::error::Error for PresetStateError {}

impl ElixirState {
    pub fn validate(&self) -> Result<(), PresetStateError> {
        validate_gain("master_gain", self.master_gain)?;
        for (index, gain) in self.role_gains.iter().copied().enumerate() {
            validate_gain(&format!("role_gains[{index}]"), gain)?;
        }
        Ok(())
    }

    pub fn apply_to_engine(&self, engine: &mut Engine) -> Result<(), PresetStateError> {
        self.validate()?;
        if engine.sample_rate() == 0 {
            return Err(PresetStateError::Invalid(
                "engine must be prepared before state application".into(),
            ));
        }
        engine.panic();
        engine.set_master_gain(self.master_gain);
        for (role, gain) in VoiceRole::ALL.into_iter().zip(self.role_gains) {
            engine.set_role_gain(role, gain);
        }
        Ok(())
    }

    pub fn snapshot_engine(engine: &Engine) -> Result<Self, PresetStateError> {
        if engine.sample_rate() == 0 {
            return Err(PresetStateError::Invalid(
                "engine must be prepared before snapshot".into(),
            ));
        }
        let state = Self {
            master_gain: engine.master_gain(),
            role_gains: engine.role_gains(),
        };
        state.validate()?;
        Ok(state)
    }
}

impl ElixirPreset {
    pub fn migrate(mut self) -> Result<Self, PresetStateError> {
        match self.schema_version {
            CURRENT_SCHEMA_VERSION => self.state.validate()?,
            1 | 2 => {
                self.schema_version = CURRENT_SCHEMA_VERSION;
                self.state = sanitized_state(self.state.master_gain, self.state.role_gains);
            }
            version => return Err(PresetStateError::UnsupportedSchema(version)),
        }
        Ok(self)
    }

    pub fn validate(&self) -> Result<(), PresetStateError> {
        if self.schema_version != CURRENT_SCHEMA_VERSION {
            return Err(PresetStateError::UnsupportedSchema(self.schema_version));
        }
        self.state.validate()
    }
}

fn validate_gain(name: &str, gain: f32) -> Result<(), PresetStateError> {
    if gain.is_finite() && (0.0..=1.0).contains(&gain) {
        Ok(())
    } else {
        Err(PresetStateError::Invalid(format!(
            "{name}={gain} outside 0..=1"
        )))
    }
}

fn valid_gain(value: Option<&Value>) -> Option<f32> {
    let gain = value?.as_f64()? as f32;
    (gain.is_finite() && (0.0..=1.0).contains(&gain)).then_some(gain)
}

fn sanitized_state(master_gain: f32, role_gains: [f32; 4]) -> ElixirState {
    ElixirState {
        master_gain: if master_gain.is_finite() && (0.0..=1.0).contains(&master_gain) {
            master_gain
        } else {
            DEFAULT_MASTER_GAIN
        },
        role_gains: core::array::from_fn(|index| {
            let gain = role_gains[index];
            if gain.is_finite() && (0.0..=1.0).contains(&gain) {
                gain
            } else {
                DEFAULT_ROLE_GAINS[index]
            }
        }),
    }
}

/// Parse current state strictly, or deterministically collapse v1/v2 full
/// synth state to safe sine defaults plus valid master/role gains.
pub fn parse_preset_str(json: &str) -> Result<ElixirPreset, PresetStateError> {
    let value: Value =
        serde_json::from_str(json).map_err(|error| PresetStateError::Json(error.to_string()))?;
    let version = value
        .get("schema_version")
        .and_then(Value::as_u64)
        .unwrap_or(1) as u32;

    if version == CURRENT_SCHEMA_VERSION {
        let preset: ElixirPreset = serde_json::from_value(value)
            .map_err(|error| PresetStateError::Json(error.to_string()))?;
        preset.migrate()
    } else if matches!(version, 1 | 2) {
        let state = value.get("state").unwrap_or(&Value::Null);
        let patch = value.get("patch").unwrap_or(&Value::Null);
        let master_gain = valid_gain(state.get("master_gain"))
            .or_else(|| valid_gain(patch.get("master_gain")))
            .unwrap_or(DEFAULT_MASTER_GAIN);
        let mut role_gains = DEFAULT_ROLE_GAINS;
        if let Some(old_roles) = state.get("role_gains").and_then(Value::as_array) {
            for (index, gain) in role_gains.iter_mut().enumerate() {
                if let Some(valid) = valid_gain(old_roles.get(index)) {
                    *gain = valid;
                }
            }
        }
        Ok(ElixirPreset {
            schema_version: CURRENT_SCHEMA_VERSION,
            name: value
                .get("name")
                .and_then(Value::as_str)
                .unwrap_or("Sine")
                .to_owned(),
            author: value
                .get("author")
                .and_then(Value::as_str)
                .map(str::to_owned),
            state: ElixirState {
                master_gain,
                role_gains,
            },
        })
    } else {
        Err(PresetStateError::UnsupportedSchema(version))
    }
}

pub fn contrapunk_default_state() -> ElixirState {
    ElixirState::default()
}

pub fn contrapunk_default_preset() -> ElixirPreset {
    ElixirPreset {
        name: "Contrapunk Sine".into(),
        author: Some("Contrapunk Audio".into()),
        ..ElixirPreset::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn current_state_round_trips_and_applies() {
        let preset = ElixirPreset {
            state: ElixirState {
                master_gain: 0.4,
                role_gains: [1.0, 0.8, 0.6, 0.4],
            },
            ..ElixirPreset::default()
        };
        let json = serde_json::to_string(&preset).unwrap();
        let parsed = parse_preset_str(&json).unwrap();
        assert_eq!(parsed, preset);

        let mut engine = Engine::new();
        engine.prepare(48_000, 256);
        parsed.state.apply_to_engine(&mut engine).unwrap();
        assert_eq!(ElixirState::snapshot_engine(&engine).unwrap(), preset.state);
    }

    #[test]
    fn old_full_state_collapses_to_sine_controls_without_poisoning() {
        let old = r#"{
            "schema_version": 2,
            "name": "Old Advanced Patch",
            "author": "User",
            "state": {
                "master_gain": 9.0,
                "legacy_compatibility": true,
                "role_gains": [0.9, null, -1.0, 0.4],
                "oscillator": {"spectral_morph":"skew","unison_voices":16},
                "amp_envelope": {"attack_secs":4.0},
                "filter": {"kind":"phaser"},
                "modulation_routes": [{"amount":999}],
                "fx_slots": [{"type":"delay"}]
            }
        }"#;
        let migrated = parse_preset_str(old).unwrap();
        assert_eq!(migrated.schema_version, CURRENT_SCHEMA_VERSION);
        assert_eq!(migrated.name, "Old Advanced Patch");
        assert_eq!(migrated.state.master_gain, DEFAULT_MASTER_GAIN);
        assert_eq!(migrated.state.role_gains, [0.9, 1.0, 1.0, 0.4]);
        migrated.validate().unwrap();
    }

    #[test]
    fn v1_patch_preserves_only_valid_master_gain() {
        let old = r#"{
            "schema_version": 1,
            "name": "Vital import",
            "patch": {"master_gain":0.33,"filter_cutoff":400,"reverb_mix":0.8},
            "source": {"kind":"vital","settings":{"anything":true}}
        }"#;
        let migrated = parse_preset_str(old).unwrap();
        assert_eq!(migrated.state.master_gain, 0.33);
        assert_eq!(migrated.state.role_gains, DEFAULT_ROLE_GAINS);
    }

    #[test]
    fn current_schema_rejects_non_finite_or_out_of_range_state() {
        let invalid =
            r#"{"schema_version":3,"name":"bad","state":{"master_gain":2,"role_gains":[1,1,1,1]}}"#;
        assert!(matches!(
            parse_preset_str(invalid),
            Err(PresetStateError::Invalid(_))
        ));
    }

    #[test]
    fn future_schema_is_rejected() {
        assert_eq!(
            parse_preset_str(r#"{"schema_version":99}"#).unwrap_err(),
            PresetStateError::UnsupportedSchema(99)
        );
    }
}
