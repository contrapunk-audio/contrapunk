//! Persisted Elixir synthesizer state and deterministic factory patches.

use std::fmt;

use elixir_core::{
    AmpEnvelope, CombineMode, Engine, HarmonicPreset, HarmonicRecipe, RolePatch,
    SecondaryOscillator, Vibrato, VoiceRole, PARTIAL_COUNT,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const CURRENT_SCHEMA_VERSION: u32 = 4;
const DEFAULT_MASTER_GAIN: f32 = 0.25;
const DEFAULT_ROLE_GAINS: [f32; 4] = [1.0; 4];

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CombineModeState {
    #[default]
    PrimaryOnly,
    Add,
    Ring,
}

impl From<CombineMode> for CombineModeState {
    fn from(value: CombineMode) -> Self {
        match value {
            CombineMode::PrimaryOnly => Self::PrimaryOnly,
            CombineMode::Add => Self::Add,
            CombineMode::Ring => Self::Ring,
        }
    }
}

impl From<CombineModeState> for CombineMode {
    fn from(value: CombineModeState) -> Self {
        match value {
            CombineModeState::PrimaryOnly => Self::PrimaryOnly,
            CombineModeState::Add => Self::Add,
            CombineModeState::Ring => Self::Ring,
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct HarmonicState {
    pub amplitudes: [f32; PARTIAL_COUNT],
    pub phases: [f32; PARTIAL_COUNT],
}

impl Default for HarmonicState {
    fn default() -> Self {
        let recipe = HarmonicRecipe::default();
        Self {
            amplitudes: recipe.amplitudes,
            phases: recipe.phases,
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct SecondaryState {
    pub mode: CombineModeState,
    pub semitones: f32,
    pub fine_cents: f32,
    pub phase: f32,
    pub level: f32,
}

impl Default for SecondaryState {
    fn default() -> Self {
        let oscillator = SecondaryOscillator::default();
        Self {
            mode: oscillator.mode.into(),
            semitones: oscillator.semitones,
            fine_cents: oscillator.fine_cents,
            phase: oscillator.phase,
            level: oscillator.level,
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct EnvelopeState {
    pub attack_secs: f32,
    pub decay_secs: f32,
    pub sustain_level: f32,
    pub release_secs: f32,
    pub velocity_sensitivity: f32,
    pub expression_sensitivity: f32,
}

impl Default for EnvelopeState {
    fn default() -> Self {
        AmpEnvelope::default().into()
    }
}

impl From<AmpEnvelope> for EnvelopeState {
    fn from(value: AmpEnvelope) -> Self {
        Self {
            attack_secs: value.attack_secs,
            decay_secs: value.decay_secs,
            sustain_level: value.sustain_level,
            release_secs: value.release_secs,
            velocity_sensitivity: value.velocity_sensitivity,
            expression_sensitivity: value.expression_sensitivity,
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct VibratoState {
    pub rate_hz: f32,
    pub depth_cents: f32,
    pub mod_wheel_depth_cents: f32,
}

impl Default for VibratoState {
    fn default() -> Self {
        let vibrato = Vibrato::default();
        Self {
            rate_hz: vibrato.rate_hz,
            depth_cents: vibrato.depth_cents,
            mod_wheel_depth_cents: vibrato.mod_wheel_depth_cents,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct RolePatchState {
    pub harmonics: HarmonicState,
    pub secondary: SecondaryState,
    pub envelope: EnvelopeState,
    pub vibrato: VibratoState,
}

impl RolePatchState {
    pub fn to_core(self) -> RolePatch {
        RolePatch {
            harmonics: HarmonicRecipe {
                amplitudes: self.harmonics.amplitudes,
                phases: self.harmonics.phases,
            },
            secondary: SecondaryOscillator {
                mode: self.secondary.mode.into(),
                semitones: self.secondary.semitones,
                fine_cents: self.secondary.fine_cents,
                phase: self.secondary.phase,
                level: self.secondary.level,
            },
            envelope: AmpEnvelope {
                attack_secs: self.envelope.attack_secs,
                decay_secs: self.envelope.decay_secs,
                sustain_level: self.envelope.sustain_level,
                release_secs: self.envelope.release_secs,
                velocity_sensitivity: self.envelope.velocity_sensitivity,
                expression_sensitivity: self.envelope.expression_sensitivity,
            },
            vibrato: Vibrato {
                rate_hz: self.vibrato.rate_hz,
                depth_cents: self.vibrato.depth_cents,
                mod_wheel_depth_cents: self.vibrato.mod_wheel_depth_cents,
            },
        }
    }

    fn validate(self, name: &str) -> Result<(), PresetStateError> {
        let patch = self.to_core();
        if patch == patch.sanitized() {
            Ok(())
        } else {
            Err(PresetStateError::Invalid(format!(
                "{name} contains an out-of-range sound control"
            )))
        }
    }
}

impl From<RolePatch> for RolePatchState {
    fn from(value: RolePatch) -> Self {
        Self {
            harmonics: HarmonicState {
                amplitudes: value.harmonics.amplitudes,
                phases: value.harmonics.phases,
            },
            secondary: SecondaryState {
                mode: value.secondary.mode.into(),
                semitones: value.secondary.semitones,
                fine_cents: value.secondary.fine_cents,
                phase: value.secondary.phase,
                level: value.secondary.level,
            },
            envelope: value.envelope.into(),
            vibrato: VibratoState {
                rate_hz: value.vibrato.rate_hz,
                depth_cents: value.vibrato.depth_cents,
                mod_wheel_depth_cents: value.vibrato.mod_wheel_depth_cents,
            },
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct ElixirState {
    pub master_gain: f32,
    pub role_gains: [f32; 4],
    pub role_patches: [RolePatchState; 4],
}

impl Default for ElixirState {
    fn default() -> Self {
        Self {
            master_gain: DEFAULT_MASTER_GAIN,
            role_gains: DEFAULT_ROLE_GAINS,
            role_patches: [RolePatchState::default(); 4],
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
        for (index, patch) in self.role_patches.iter().copied().enumerate() {
            patch.validate(&format!("role_patches[{index}]"))?;
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
        for (index, role) in VoiceRole::ALL.into_iter().enumerate() {
            engine.set_role_gain(role, self.role_gains[index]);
            engine.set_role_patch(role, self.role_patches[index].to_core());
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
            role_patches: engine.role_patches().map(RolePatchState::from),
        };
        state.validate()?;
        Ok(state)
    }
}

impl ElixirPreset {
    pub fn migrate(mut self) -> Result<Self, PresetStateError> {
        match self.schema_version {
            CURRENT_SCHEMA_VERSION => self.state.validate()?,
            1..=3 => {
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
        role_patches: [RolePatchState::default(); 4],
    }
}

/// Parse current state strictly, or deterministically collapse v1-v3 state to
/// safe sine defaults plus valid master and role gains.
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
    } else if matches!(version, 1..=3) {
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
            state: sanitized_state(master_gain, role_gains),
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

/// Stable factory states used for auditions, tests, and video capture.
pub fn factory_presets() -> [ElixirPreset; 7] {
    let preset = |name: &str, patch: RolePatch| ElixirPreset {
        name: name.into(),
        author: Some("Contrapunk Audio".into()),
        state: ElixirState {
            role_patches: [RolePatchState::from(patch); 4],
            ..ElixirState::default()
        },
        ..ElixirPreset::default()
    };

    let mut colour_roles = ElixirState::default();
    for (index, recipe) in [
        HarmonicPreset::Sine,
        HarmonicPreset::Dark,
        HarmonicPreset::Odd,
        HarmonicPreset::Three,
    ]
    .into_iter()
    .enumerate()
    {
        colour_roles.role_patches[index] = RolePatchState::from(RolePatch {
            harmonics: HarmonicRecipe::preset(recipe),
            ..RolePatch::sine()
        });
    }

    [
        preset(
            "Harmonic family",
            RolePatch {
                harmonics: HarmonicRecipe::preset(HarmonicPreset::Three),
                ..RolePatch::sine()
            },
        ),
        ElixirPreset {
            name: "Ensemble colours".into(),
            author: Some("Contrapunk Audio".into()),
            state: colour_roles,
            ..ElixirPreset::default()
        },
        preset(
            "Phase reinforcement",
            RolePatch {
                secondary: SecondaryOscillator {
                    mode: CombineMode::Add,
                    phase: 0.0,
                    ..SecondaryOscillator::default()
                },
                ..RolePatch::sine()
            },
        ),
        preset(
            "Phase cancellation",
            RolePatch {
                secondary: SecondaryOscillator {
                    mode: CombineMode::Add,
                    phase: 0.5,
                    ..SecondaryOscillator::default()
                },
                ..RolePatch::sine()
            },
        ),
        preset(
            "Ring difference",
            RolePatch {
                secondary: SecondaryOscillator {
                    mode: CombineMode::Ring,
                    semitones: -12.0,
                    ..SecondaryOscillator::default()
                },
                ..RolePatch::sine()
            },
        ),
        preset(
            "Passive ring-down",
            RolePatch {
                envelope: AmpEnvelope::ring_down(),
                ..RolePatch::sine()
            },
        ),
        preset(
            "Maintained vibrato",
            RolePatch {
                envelope: AmpEnvelope::maintained(),
                vibrato: Vibrato {
                    rate_hz: 5.0,
                    depth_cents: 18.0,
                    mod_wheel_depth_cents: 0.0,
                },
                ..RolePatch::sine()
            },
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn current_state_round_trips_and_applies() {
        let patch = RolePatch {
            harmonics: HarmonicRecipe::preset(HarmonicPreset::Odd),
            secondary: SecondaryOscillator {
                mode: CombineMode::Ring,
                semitones: -12.0,
                ..SecondaryOscillator::default()
            },
            envelope: AmpEnvelope::ring_down(),
            vibrato: Vibrato {
                depth_cents: 18.0,
                ..Vibrato::default()
            },
        };
        let preset = ElixirPreset {
            state: ElixirState {
                master_gain: 0.4,
                role_gains: [1.0, 0.8, 0.6, 0.4],
                role_patches: [RolePatchState::from(patch); 4],
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
    fn schema_three_collapses_to_exact_legacy_defaults() {
        let old = r#"{
            "schema_version": 3,
            "name": "Fixed sine",
            "state": {"master_gain":0.33,"role_gains":[0.9,0.8,0.7,0.6]}
        }"#;
        let migrated = parse_preset_str(old).unwrap();
        assert_eq!(migrated.schema_version, CURRENT_SCHEMA_VERSION);
        assert_eq!(migrated.state.master_gain, 0.33);
        assert_eq!(migrated.state.role_gains, [0.9, 0.8, 0.7, 0.6]);
        assert_eq!(migrated.state.role_patches, [RolePatchState::default(); 4]);
    }

    #[test]
    fn older_full_state_keeps_only_safe_gains() {
        let old = r#"{
            "schema_version": 2,
            "name": "Old Advanced Patch",
            "author": "User",
            "state": {
                "master_gain": 9.0,
                "role_gains": [0.9, null, -1.0, 0.4],
                "oscillator": {"spectral_morph":"skew","unison_voices":16}
            }
        }"#;
        let migrated = parse_preset_str(old).unwrap();
        assert_eq!(migrated.state.master_gain, DEFAULT_MASTER_GAIN);
        assert_eq!(migrated.state.role_gains, [0.9, 1.0, 1.0, 0.4]);
        migrated.validate().unwrap();
    }

    #[test]
    fn current_schema_rejects_out_of_range_patch_state() {
        let mut preset = ElixirPreset::default();
        preset.state.role_patches[0].harmonics.amplitudes[0] = 2.0;
        let json = serde_json::to_string(&preset).unwrap();
        assert!(matches!(
            parse_preset_str(&json),
            Err(PresetStateError::Invalid(_))
        ));
    }

    #[test]
    fn factory_presets_are_named_valid_and_deterministic() {
        let first = factory_presets();
        let second = factory_presets();
        assert_eq!(first, second);
        assert_eq!(first.len(), 7);
        for preset in first {
            assert!(!preset.name.contains("Chapter"));
            preset.validate().unwrap();
        }
    }

    #[test]
    fn future_schema_is_rejected() {
        assert_eq!(
            parse_preset_str(r#"{"schema_version":99}"#).unwrap_err(),
            PresetStateError::UnsupportedSchema(99)
        );
    }
}
