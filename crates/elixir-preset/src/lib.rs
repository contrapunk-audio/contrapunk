//! Elixir preset schema and importers (Phase 21.B5).
//!
//! The crate is intentionally separate from `elixir-core` so the DSP
//! engine can remain `no_std`/audio-thread focused while standalone,
//! plugin, and tooling share one preset/import implementation.

use std::fmt;
use std::io::{Read, Seek};
use std::path::Path;

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use zip::ZipArchive;

pub const CURRENT_SCHEMA_VERSION: u32 = 2;
pub const FX_SLOT_COUNT: usize = 8;
pub const MAX_MOD_ROUTES: usize = 32;
pub const LFO_COUNT: usize = 4;

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum SpectralMorphState {
    Passthrough,
    Vocode,
    FormScale,
    HarmonicScale,
    InharmonicScale,
    Smear,
    RandomAmplitudes,
    LowPass,
    HighPass,
    PhaseDisperse,
    ShepardTone,
    Skew,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum PhaseDistortionState {
    Off,
    Quantize,
    Bend,
    Squeeze,
    Sync,
    PulseWidth,
    FmOscillatorA,
    FmOscillatorB,
    FmSample,
    RmOscillatorA,
    RmOscillatorB,
    RmSample,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum UnisonStyleState {
    Centered,
    Octaves,
    Fifths,
    PowerChord,
    HarmonicSeries,
    Wide,
    Narrow,
    Organ,
    Suspended,
    Cluster,
    Alternating,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum FilterKindState {
    DigitalSvf,
    Diode,
    Dirty,
    Formant,
    Phaser,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct OscillatorState {
    pub spectral_morph: SpectralMorphState,
    pub morph_amount: f32,
    pub phase_distortion: PhaseDistortionState,
    pub phase_amount: f32,
    pub unison_style: UnisonStyleState,
    pub unison_voices: u8,
    pub unison_detune_cents: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct EnvelopeState {
    pub attack_secs: f32,
    pub decay_secs: f32,
    pub sustain: f32,
    pub release_secs: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct FilterState {
    pub kind: FilterKindState,
    pub cutoff_hz: f32,
    pub resonance: f32,
    pub drive: f32,
    pub gain: f32,
    pub morph_x: f32,
    pub morph_y: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct LfoState {
    pub rate_hz: f32,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", content = "index", rename_all = "kebab-case")]
pub enum ModSourceState {
    Constant,
    Lfo(u8),
    AmpEnv,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", content = "index", rename_all = "kebab-case")]
pub enum ModDestinationState {
    MasterGain,
    LfoRate(u8),
    FilterCutoff,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ModRouteState {
    pub source: ModSourceState,
    pub destination: ModDestinationState,
    pub amount: f32,
    pub bipolar: bool,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum FxState {
    Drive {
        drive: f32,
        mix: f32,
    },
    Delay {
        time_secs: f32,
        feedback: f32,
        mix: f32,
    },
    Reverb {
        decay: f32,
        damping: f32,
        mix: f32,
    },
    FdnReverb {
        decay_secs: f32,
        damping: f32,
        mix: f32,
    },
    Chorus {
        rate_hz: f32,
        depth_ms: f32,
        mix: f32,
    },
    Flanger {
        rate_hz: f32,
        depth_ms: f32,
        feedback: f32,
        mix: f32,
    },
    Phaser {
        rate_hz: f32,
        depth: f32,
        feedback: f32,
        mix: f32,
    },
    Compressor {
        threshold_db: f32,
        ratio: f32,
        attack_ms: f32,
        release_ms: f32,
        makeup_db: f32,
        mix: f32,
    },
}

impl FxState {
    fn kind(&self) -> &'static str {
        match self {
            Self::Drive { .. } => "drive",
            Self::Delay { .. } => "delay",
            Self::Reverb { .. } => "reverb",
            Self::FdnReverb { .. } => "fdn-reverb",
            Self::Chorus { .. } => "chorus",
            Self::Flanger { .. } => "flanger",
            Self::Phaser { .. } => "phaser",
            Self::Compressor { .. } => "compressor",
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct FxSlotState {
    pub enabled: bool,
    #[serde(flatten)]
    pub effect: FxState,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ElixirState {
    pub master_gain: f32,
    pub role_gains: [f32; 4],
    pub oscillator: OscillatorState,
    pub amp_envelope: EnvelopeState,
    pub filter: FilterState,
    pub lfos: [LfoState; LFO_COUNT],
    pub modulation_routes: Vec<ModRouteState>,
    pub fx_slots: Vec<FxSlotState>,
}

impl Default for ElixirState {
    fn default() -> Self {
        Self {
            master_gain: 10.0_f32.powf(-12.0 / 20.0),
            role_gains: [1.0; 4],
            oscillator: OscillatorState {
                spectral_morph: SpectralMorphState::Passthrough,
                morph_amount: 0.0,
                phase_distortion: PhaseDistortionState::Off,
                phase_amount: 0.0,
                unison_style: UnisonStyleState::Centered,
                unison_voices: 1,
                unison_detune_cents: 8.0,
            },
            amp_envelope: EnvelopeState {
                attack_secs: 0.005,
                decay_secs: 0.120,
                sustain: 0.70,
                release_secs: 0.250,
            },
            filter: FilterState {
                kind: FilterKindState::DigitalSvf,
                cutoff_hz: 8_000.0,
                resonance: 0.0,
                drive: 1.0,
                gain: 1.0,
                morph_x: 0.0,
                morph_y: 0.0,
            },
            lfos: core::array::from_fn(|_| LfoState { rate_hz: 5.0 }),
            modulation_routes: Vec::new(),
            fx_slots: vec![
                FxSlotState {
                    enabled: false,
                    effect: FxState::Drive {
                        drive: 2.5,
                        mix: 0.4,
                    },
                },
                FxSlotState {
                    enabled: false,
                    effect: FxState::Delay {
                        time_secs: 0.375,
                        feedback: 0.45,
                        mix: 0.30,
                    },
                },
                FxSlotState {
                    enabled: false,
                    effect: FxState::Reverb {
                        decay: 0.85,
                        damping: 0.40,
                        mix: 0.30,
                    },
                },
                FxSlotState {
                    enabled: false,
                    effect: FxState::FdnReverb {
                        decay_secs: 2.8,
                        damping: 0.35,
                        mix: 0.35,
                    },
                },
                FxSlotState {
                    enabled: false,
                    effect: FxState::Chorus {
                        rate_hz: 0.35,
                        depth_ms: 8.0,
                        mix: 0.35,
                    },
                },
                FxSlotState {
                    enabled: false,
                    effect: FxState::Flanger {
                        rate_hz: 0.18,
                        depth_ms: 2.5,
                        feedback: 0.45,
                        mix: 0.40,
                    },
                },
                FxSlotState {
                    enabled: false,
                    effect: FxState::Phaser {
                        rate_hz: 0.20,
                        depth: 0.75,
                        feedback: 0.65,
                        mix: 0.45,
                    },
                },
                FxSlotState {
                    enabled: false,
                    effect: FxState::Compressor {
                        threshold_db: -18.0,
                        ratio: 4.0,
                        attack_ms: 8.0,
                        release_ms: 120.0,
                        makeup_db: 4.0,
                        mix: 1.0,
                    },
                },
            ],
        }
    }
}

/// Native Elixir preset document. This is the stable interchange shape
/// for standalone and plugin state; source-specific data (e.g. raw Vital
/// JSON) is preserved in [`ElixirPreset::source`].
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ElixirPreset {
    pub schema_version: u32,
    pub name: String,
    pub author: Option<String>,
    pub style: Option<String>,
    #[serde(default)]
    pub patch: ElixirPatch,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub state: Option<ElixirState>,
    pub source: PresetSource,
}

/// Elixir controls captured by the B5 importer. Not every imported preset
/// maps every field; unmapped values stay in [`PresetSource::Vital`].
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct ElixirPatch {
    pub master_gain: Option<f32>,
    pub filter_cutoff: Option<f32>,
    pub filter_resonance: Option<f32>,
    pub filter_drive: Option<f32>,
    pub delay_mix: Option<f32>,
    pub delay_feedback: Option<f32>,
    pub chorus_mix: Option<f32>,
    pub reverb_mix: Option<f32>,
    pub compressor_mix: Option<f32>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum PresetSource {
    Native,
    Vital(VitalSource),
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct VitalSource {
    pub synth_version: Option<String>,
    pub comments: Option<String>,
    pub macro_names: [Option<String>; 4],
    /// Full Vital settings dictionary. This is intentionally retained so
    /// B7 wavetable/spectral parity can remap more fields later without
    /// requiring users to re-import.
    pub settings: Map<String, Value>,
}

/// Result of importing a `.vitalbank` archive.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct VitalBankImport {
    pub presets: Vec<ElixirPreset>,
    pub wavetable_paths: Vec<String>,
    pub skipped_entries: Vec<String>,
}

#[derive(Debug)]
pub enum PresetImportError {
    Json(serde_json::Error),
    Zip(zip::result::ZipError),
    Io(std::io::Error),
    InvalidVital(String),
}

impl fmt::Display for PresetImportError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Json(e) => write!(f, "JSON parse error: {e}"),
            Self::Zip(e) => write!(f, "ZIP parse error: {e}"),
            Self::Io(e) => write!(f, "I/O error: {e}"),
            Self::InvalidVital(msg) => write!(f, "invalid Vital preset: {msg}"),
        }
    }
}

impl std::error::Error for PresetImportError {}

impl From<serde_json::Error> for PresetImportError {
    fn from(value: serde_json::Error) -> Self {
        Self::Json(value)
    }
}
impl From<zip::result::ZipError> for PresetImportError {
    fn from(value: zip::result::ZipError) -> Self {
        Self::Zip(value)
    }
}
impl From<std::io::Error> for PresetImportError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
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

fn finite_range(name: &str, value: f32, min: f32, max: f32) -> Result<(), PresetStateError> {
    if value.is_finite() && (min..=max).contains(&value) {
        Ok(())
    } else {
        Err(PresetStateError::Invalid(format!(
            "{name}={value} outside {min}..={max}"
        )))
    }
}

fn finite_min(name: &str, value: f32, min: f32) -> Result<(), PresetStateError> {
    if value.is_finite() && value >= min {
        Ok(())
    } else {
        Err(PresetStateError::Invalid(format!(
            "{name}={value} below {min}"
        )))
    }
}

impl ElixirState {
    pub fn validate(&self) -> Result<(), PresetStateError> {
        finite_range("master_gain", self.master_gain, 0.0, 1.0)?;
        for (index, gain) in self.role_gains.iter().copied().enumerate() {
            finite_range(&format!("role_gains[{index}]"), gain, 0.0, 1.0)?;
        }
        finite_range("morph_amount", self.oscillator.morph_amount, 0.0, 1.0)?;
        finite_range("phase_amount", self.oscillator.phase_amount, 0.0, 1.0)?;
        if !(1..=16).contains(&self.oscillator.unison_voices) {
            return Err(PresetStateError::Invalid(
                "unison_voices outside 1..=16".into(),
            ));
        }
        finite_range(
            "unison_detune_cents",
            self.oscillator.unison_detune_cents,
            0.0,
            1200.0,
        )?;
        finite_min("amp.attack_secs", self.amp_envelope.attack_secs, 0.001)?;
        finite_min("amp.decay_secs", self.amp_envelope.decay_secs, 0.001)?;
        finite_range("amp.sustain", self.amp_envelope.sustain, 0.0, 1.0)?;
        finite_min("amp.release_secs", self.amp_envelope.release_secs, 0.001)?;
        finite_range("filter.cutoff_hz", self.filter.cutoff_hz, 20.0, 22_000.0)?;
        finite_range("filter.resonance", self.filter.resonance, 0.0, 1.0)?;
        finite_range("filter.drive", self.filter.drive, 0.1, 32.0)?;
        finite_range("filter.gain", self.filter.gain, 0.0, 4.0)?;
        finite_range("filter.morph_x", self.filter.morph_x, 0.0, 1.0)?;
        finite_range("filter.morph_y", self.filter.morph_y, 0.0, 1.0)?;
        for (index, lfo) in self.lfos.iter().enumerate() {
            finite_min(&format!("lfos[{index}].rate_hz"), lfo.rate_hz, 0.0)?;
        }
        if self.modulation_routes.len() > MAX_MOD_ROUTES {
            return Err(PresetStateError::Invalid(format!(
                "{} modulation routes exceeds {MAX_MOD_ROUTES}",
                self.modulation_routes.len()
            )));
        }
        for (index, route) in self.modulation_routes.iter().enumerate() {
            if matches!(route.source, ModSourceState::Lfo(i) if i as usize >= LFO_COUNT)
                || matches!(route.destination, ModDestinationState::LfoRate(i) if i as usize >= LFO_COUNT)
            {
                return Err(PresetStateError::Invalid(format!(
                    "modulation_routes[{index}] has invalid LFO index"
                )));
            }
            if !route.amount.is_finite() {
                return Err(PresetStateError::Invalid(format!(
                    "modulation_routes[{index}].amount is not finite"
                )));
            }
        }
        if self.fx_slots.len() != FX_SLOT_COUNT {
            return Err(PresetStateError::Invalid(format!(
                "FX slot count {} != {FX_SLOT_COUNT}",
                self.fx_slots.len()
            )));
        }
        const ORDER: [&str; FX_SLOT_COUNT] = [
            "drive",
            "delay",
            "reverb",
            "fdn-reverb",
            "chorus",
            "flanger",
            "phaser",
            "compressor",
        ];
        for (index, slot) in self.fx_slots.iter().enumerate() {
            if slot.effect.kind() != ORDER[index] {
                return Err(PresetStateError::Invalid(format!(
                    "FX slot {index} must be {}",
                    ORDER[index]
                )));
            }
            slot.validate(index)?;
        }
        Ok(())
    }
}

impl FxSlotState {
    fn validate(&self, index: usize) -> Result<(), PresetStateError> {
        let field = |name: &str| format!("fx_slots[{index}].{name}");
        match self.effect {
            FxState::Drive { drive, mix } => {
                finite_range(&field("drive"), drive, 0.0, 20.0)?;
                finite_range(&field("mix"), mix, 0.0, 1.0)
            }
            FxState::Delay {
                time_secs,
                feedback,
                mix,
            } => {
                finite_range(&field("time_secs"), time_secs, 0.001, 2.0)?;
                finite_range(&field("feedback"), feedback, 0.0, 0.99)?;
                finite_range(&field("mix"), mix, 0.0, 1.0)
            }
            FxState::Reverb {
                decay,
                damping,
                mix,
            } => {
                finite_range(&field("decay"), decay, 0.0, 0.99)?;
                finite_range(&field("damping"), damping, 0.0, 1.0)?;
                finite_range(&field("mix"), mix, 0.0, 1.0)
            }
            FxState::FdnReverb {
                decay_secs,
                damping,
                mix,
            } => {
                finite_range(&field("decay_secs"), decay_secs, 0.2, 20.0)?;
                finite_range(&field("damping"), damping, 0.0, 1.0)?;
                finite_range(&field("mix"), mix, 0.0, 1.0)
            }
            FxState::Chorus {
                rate_hz,
                depth_ms,
                mix,
            } => {
                finite_range(&field("rate_hz"), rate_hz, 0.01, 8.0)?;
                finite_range(&field("depth_ms"), depth_ms, 0.0, 40.0)?;
                finite_range(&field("mix"), mix, 0.0, 1.0)
            }
            FxState::Flanger {
                rate_hz,
                depth_ms,
                feedback,
                mix,
            } => {
                finite_range(&field("rate_hz"), rate_hz, 0.01, 10.0)?;
                finite_range(&field("depth_ms"), depth_ms, 0.0, 10.0)?;
                finite_range(&field("feedback"), feedback, -0.95, 0.95)?;
                finite_range(&field("mix"), mix, 0.0, 1.0)
            }
            FxState::Phaser {
                rate_hz,
                depth,
                feedback,
                mix,
            } => {
                finite_range(&field("rate_hz"), rate_hz, 0.01, 8.0)?;
                finite_range(&field("depth"), depth, 0.0, 1.0)?;
                finite_range(&field("feedback"), feedback, 0.0, 0.95)?;
                finite_range(&field("mix"), mix, 0.0, 1.0)
            }
            FxState::Compressor {
                threshold_db,
                ratio,
                attack_ms,
                release_ms,
                makeup_db,
                mix,
            } => {
                finite_range(&field("threshold_db"), threshold_db, -60.0, 0.0)?;
                finite_range(&field("ratio"), ratio, 1.0, 40.0)?;
                finite_range(&field("attack_ms"), attack_ms, 0.1, 500.0)?;
                finite_range(&field("release_ms"), release_ms, 1.0, 2000.0)?;
                finite_range(&field("makeup_db"), makeup_db, -24.0, 24.0)?;
                finite_range(&field("mix"), mix, 0.0, 1.0)
            }
        }
    }
}

impl ElixirPreset {
    pub fn migrate(mut self) -> Result<Self, PresetStateError> {
        match self.schema_version {
            CURRENT_SCHEMA_VERSION => self.validate()?,
            1 => {
                self.validate_v1_patch()?;
                let mut state = ElixirState::default();
                if let Some(value) = self.patch.master_gain {
                    state.master_gain = value;
                }
                if let Some(value) = self.patch.filter_resonance {
                    state.filter.resonance = value;
                }
                if let Some(value) = self.patch.filter_drive {
                    state.filter.drive = value;
                }
                migrate_v1_fx(&self.patch, &mut state.fx_slots);
                state.validate()?;
                self.schema_version = CURRENT_SCHEMA_VERSION;
                self.state = Some(state);
                self.validate()?;
            }
            version => return Err(PresetStateError::UnsupportedSchema(version)),
        }
        Ok(self)
    }

    pub fn validate(&self) -> Result<(), PresetStateError> {
        if self.schema_version != CURRENT_SCHEMA_VERSION {
            return Err(PresetStateError::UnsupportedSchema(self.schema_version));
        }
        self.state
            .as_ref()
            .ok_or_else(|| PresetStateError::Invalid("schema v2 requires state".into()))?
            .validate()
    }

    fn validate_v1_patch(&self) -> Result<(), PresetStateError> {
        for (name, value) in [
            ("master_gain", self.patch.master_gain),
            ("filter_cutoff", self.patch.filter_cutoff),
            ("filter_resonance", self.patch.filter_resonance),
            ("filter_drive", self.patch.filter_drive),
            ("delay_mix", self.patch.delay_mix),
            ("delay_feedback", self.patch.delay_feedback),
            ("chorus_mix", self.patch.chorus_mix),
            ("reverb_mix", self.patch.reverb_mix),
            ("compressor_mix", self.patch.compressor_mix),
        ] {
            if value.is_some_and(|value| !value.is_finite()) {
                return Err(PresetStateError::Invalid(format!(
                    "v1 patch {name} is not finite"
                )));
            }
        }
        Ok(())
    }
}

fn migrate_v1_fx(patch: &ElixirPatch, slots: &mut [FxSlotState]) {
    let delay = &mut slots[1];
    if let FxState::Delay { feedback, mix, .. } = &mut delay.effect {
        delay.enabled = patch.delay_mix.is_some() || patch.delay_feedback.is_some();
        if let Some(value) = patch.delay_feedback {
            *feedback = value;
        }
        if let Some(value) = patch.delay_mix {
            *mix = value;
        }
    }
    let reverb = &mut slots[2];
    if let FxState::Reverb { mix, .. } = &mut reverb.effect {
        reverb.enabled = patch.reverb_mix.is_some();
        if let Some(value) = patch.reverb_mix {
            *mix = value;
        }
    }
    let chorus = &mut slots[4];
    if let FxState::Chorus { mix, .. } = &mut chorus.effect {
        chorus.enabled = patch.chorus_mix.is_some();
        if let Some(value) = patch.chorus_mix {
            *mix = value;
        }
    }
    let compressor = &mut slots[7];
    if let FxState::Compressor { mix, .. } = &mut compressor.effect {
        compressor.enabled = patch.compressor_mix.is_some();
        if let Some(value) = patch.compressor_mix {
            *mix = value;
        }
    }
}

pub fn parse_preset_str(json: &str) -> Result<ElixirPreset, PresetStateError> {
    serde_json::from_str::<ElixirPreset>(json)
        .map_err(|error| PresetStateError::Json(error.to_string()))?
        .migrate()
}

/// Import one `.vital` JSON document. `name_hint` is used when the file
/// omits `preset_name` (several community presets do this).
pub fn import_vital_str(
    json: &str,
    name_hint: Option<&str>,
) -> Result<ElixirPreset, PresetImportError> {
    let root: Value = serde_json::from_str(json)?;
    let obj = root
        .as_object()
        .ok_or_else(|| PresetImportError::InvalidVital("root is not an object".into()))?;
    let settings = obj
        .get("settings")
        .and_then(Value::as_object)
        .cloned()
        .ok_or_else(|| PresetImportError::InvalidVital("missing settings object".into()))?;

    let name = string_field(obj, "preset_name")
        .or_else(|| name_hint.map(clean_file_stem))
        .unwrap_or_else(|| "Imported Vital Preset".to_string());
    let author = string_field(obj, "author").filter(|s| !s.is_empty());
    let style = string_field(obj, "preset_style").filter(|s| !s.is_empty());
    let source = VitalSource {
        synth_version: string_field(obj, "synth_version"),
        comments: string_field(obj, "comments").filter(|s| !s.is_empty()),
        macro_names: [
            string_field(obj, "macro1"),
            string_field(obj, "macro2"),
            string_field(obj, "macro3"),
            string_field(obj, "macro4"),
        ],
        settings: settings.clone(),
    };

    Ok(ElixirPreset {
        schema_version: 1,
        name,
        author,
        style,
        patch: map_vital_patch(&settings),
        state: None,
        source: PresetSource::Vital(source),
    })
}

/// Import one `.vital` file from disk.
pub fn import_vital_file(path: impl AsRef<Path>) -> Result<ElixirPreset, PresetImportError> {
    let path = path.as_ref();
    let json = std::fs::read_to_string(path)?;
    let hint = path.file_stem().and_then(|s| s.to_str());
    import_vital_str(&json, hint)
}

pub const EXTERNAL_PRESET_EXTENSION: &str = "vital";
pub const EXTERNAL_BANK_EXTENSION: &str = "vitalbank";

pub fn import_external_preset_file(
    path: impl AsRef<Path>,
) -> Result<ElixirPreset, PresetImportError> {
    import_vital_file(path)
}

/// Import all `.vital` presets from a `.vitalbank` ZIP archive. Wavetables
/// are listed but not converted until B7's wavetable editor/FFT work.
pub fn import_vital_bank<R: Read + Seek>(reader: R) -> Result<VitalBankImport, PresetImportError> {
    let mut archive = ZipArchive::new(reader)?;
    let mut out = VitalBankImport::default();
    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let name = file.name().to_string();
        if name.ends_with('/') {
            continue;
        }
        if name.ends_with(".vitaltable") {
            out.wavetable_paths.push(name);
            continue;
        }
        if !name.ends_with(".vital") {
            continue;
        }
        let mut json = String::new();
        file.read_to_string(&mut json)?;
        match import_vital_str(&json, Path::new(&name).file_stem().and_then(|s| s.to_str())) {
            Ok(preset) => out.presets.push(preset),
            Err(err) => out.skipped_entries.push(format!("{name}: {err}")),
        }
    }
    Ok(out)
}

pub fn import_vital_bank_file(
    path: impl AsRef<Path>,
) -> Result<VitalBankImport, PresetImportError> {
    let file = std::fs::File::open(path)?;
    import_vital_bank(file)
}

pub fn import_external_bank_file(
    path: impl AsRef<Path>,
) -> Result<VitalBankImport, PresetImportError> {
    import_vital_bank_file(path)
}

fn map_vital_patch(settings: &Map<String, Value>) -> ElixirPatch {
    ElixirPatch {
        master_gain: number(settings, "volume"),
        filter_cutoff: number(settings, "filter_fx_cutoff")
            .or_else(|| number(settings, "filter_1_cutoff")),
        filter_resonance: number(settings, "filter_fx_resonance")
            .or_else(|| number(settings, "filter_1_resonance")),
        filter_drive: number(settings, "filter_fx_drive"),
        delay_mix: number(settings, "delay_dry_wet"),
        delay_feedback: number(settings, "delay_feedback"),
        chorus_mix: number(settings, "chorus_dry_wet"),
        reverb_mix: number(settings, "reverb_dry_wet"),
        compressor_mix: number(settings, "compressor_mix"),
    }
}

fn string_field(obj: &Map<String, Value>, key: &str) -> Option<String> {
    obj.get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .map(str::to_string)
}

fn number(obj: &Map<String, Value>, key: &str) -> Option<f32> {
    obj.get(key)
        .and_then(Value::as_f64)
        .map(|v| v as f32)
        .filter(|v| v.is_finite())
}

fn clean_file_stem(s: &str) -> String {
    s.strip_suffix(" Preset")
        .or_else(|| s.strip_suffix(" Presets"))
        .unwrap_or(s)
        .trim()
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Cursor, Write};

    fn native_preset(state: ElixirState) -> ElixirPreset {
        ElixirPreset {
            schema_version: CURRENT_SCHEMA_VERSION,
            name: "Test".into(),
            author: None,
            style: None,
            patch: ElixirPatch::default(),
            state: Some(state),
            source: PresetSource::Native,
        }
    }

    #[test]
    fn schema_v2_round_trips_complete_typed_state() {
        let mut state = ElixirState {
            master_gain: 0.4,
            role_gains: [0.1, 0.2, 0.3, 0.4],
            ..Default::default()
        };
        state.oscillator.spectral_morph = SpectralMorphState::Smear;
        state.oscillator.phase_distortion = PhaseDistortionState::Sync;
        state.oscillator.unison_style = UnisonStyleState::Fifths;
        state.filter.kind = FilterKindState::Dirty;
        state.lfos[2].rate_hz = 7.5;
        state.modulation_routes.push(ModRouteState {
            source: ModSourceState::Lfo(2),
            destination: ModDestinationState::FilterCutoff,
            amount: 1200.0,
            bipolar: true,
        });
        state.fx_slots[0].enabled = true;
        state.fx_slots[7].enabled = true;
        let preset = native_preset(state);

        let json = serde_json::to_string(&preset).unwrap();
        let decoded = parse_preset_str(&json).unwrap();
        assert_eq!(decoded, preset);
    }

    #[test]
    fn validation_rejects_schema_bounds_non_finite_indices_and_slot_order() {
        let mut preset = native_preset(ElixirState::default());
        preset.schema_version = 99;
        assert!(matches!(
            preset.validate(),
            Err(PresetStateError::UnsupportedSchema(99))
        ));

        let state = ElixirState {
            master_gain: f32::NAN,
            ..Default::default()
        };
        assert!(state.validate().is_err());

        let mut state = ElixirState::default();
        state.modulation_routes.push(ModRouteState {
            source: ModSourceState::Lfo(LFO_COUNT as u8),
            destination: ModDestinationState::MasterGain,
            amount: 1.0,
            bipolar: true,
        });
        assert!(state.validate().is_err());

        let mut state = ElixirState::default();
        state.fx_slots.pop();
        assert!(state.validate().is_err());

        let mut state = ElixirState::default();
        state.fx_slots.swap(0, 1);
        assert!(state.validate().is_err());

        let json = serde_json::to_string(&native_preset(ElixirState::default()))
            .unwrap()
            .replace("passthrough", "unknown-morph");
        assert!(matches!(
            parse_preset_str(&json),
            Err(PresetStateError::Json(_))
        ));
    }

    #[test]
    fn schema_v1_migrates_defaults_effect_enables_and_preserves_raw_vital() {
        let json = r#"{
            "preset_name":"Legacy",
            "settings":{
                "volume":0.5,
                "filter_fx_cutoff":125.0,
                "filter_fx_resonance":0.37,
                "filter_fx_drive":3.0,
                "delay_dry_wet":0.33,
                "delay_feedback":0.5,
                "chorus_dry_wet":0.07,
                "compressor_mix":0.8
            }
        }"#;
        let legacy = import_vital_str(json, None).unwrap();
        let raw_source = legacy.source.clone();
        let migrated = legacy.migrate().unwrap();
        let state = migrated.state.as_ref().unwrap();

        assert_eq!(migrated.schema_version, CURRENT_SCHEMA_VERSION);
        assert_eq!(migrated.patch.filter_cutoff, Some(125.0));
        assert_eq!(migrated.source, raw_source);
        assert_eq!(state.master_gain, 0.5);
        assert_eq!(state.filter.cutoff_hz, 8_000.0);
        assert_eq!(state.filter.resonance, 0.37);
        assert_eq!(state.filter.drive, 3.0);
        assert!(state.fx_slots[1].enabled);
        assert!(!state.fx_slots[2].enabled);
        assert!(state.fx_slots[4].enabled);
        assert!(state.fx_slots[7].enabled);
        assert!(matches!(
            state.fx_slots[1].effect,
            FxState::Delay {
                feedback: 0.5,
                mix: 0.33,
                ..
            }
        ));
    }

    #[test]
    fn imports_vital_json_with_metadata_and_patch_subset() {
        let json = r#"{
            "author":"Flamedragonz",
            "comments":"",
            "macro1":"Tone",
            "macro2":"",
            "macro3":"",
            "macro4":"",
            "preset_name":"Cyberpunk 2077",
            "preset_style":"Keys",
            "synth_version":"1.5.5",
            "settings":{
                "filter_fx_cutoff":125.0,
                "filter_fx_resonance":0.37,
                "delay_dry_wet":0.33,
                "delay_feedback":0.5,
                "chorus_dry_wet":0.07,
                "compressor_mix":1.0
            }
        }"#;
        let preset = import_vital_str(json, None).unwrap();
        assert_eq!(preset.name, "Cyberpunk 2077");
        assert_eq!(preset.author.as_deref(), Some("Flamedragonz"));
        assert_eq!(preset.patch.filter_cutoff, Some(125.0));
        assert_eq!(preset.patch.delay_mix, Some(0.33));
        match preset.source {
            PresetSource::Vital(source) => {
                assert_eq!(source.synth_version.as_deref(), Some("1.5.5"));
                assert_eq!(source.settings.len(), 6);
            }
            PresetSource::Native => panic!("expected Vital source"),
        }
    }

    #[test]
    fn imports_vital_json_without_preset_name_from_hint() {
        let json = r#"{"author":"HIKEMAH","settings":{"reverb_dry_wet":0.25}}"#;
        let preset = import_vital_str(json, Some("Dear April Pad Preset")).unwrap();
        assert_eq!(preset.name, "Dear April Pad");
        assert_eq!(preset.patch.reverb_mix, Some(0.25));
    }

    #[test]
    fn imports_vital_bank_zip_and_tracks_wavetables() {
        let mut cursor = Cursor::new(Vec::<u8>::new());
        {
            let mut zip = zip::ZipWriter::new(&mut cursor);
            let options = zip::write::SimpleFileOptions::default();
            zip.start_file("Pack/Presets/Test.vital", options).unwrap();
            zip.write_all(br#"{"preset_name":"Test","settings":{"chorus_dry_wet":0.5}}"#)
                .unwrap();
            zip.start_file("Factory/Wavetables/Basic.vitaltable", options)
                .unwrap();
            zip.write_all(b"table").unwrap();
            zip.finish().unwrap();
        }
        cursor.set_position(0);
        let bank = import_vital_bank(cursor).unwrap();
        assert_eq!(bank.presets.len(), 1);
        assert_eq!(bank.presets[0].name, "Test");
        assert_eq!(
            bank.wavetable_paths,
            vec!["Factory/Wavetables/Basic.vitaltable"]
        );
        assert!(bank.skipped_entries.is_empty());
    }
}
