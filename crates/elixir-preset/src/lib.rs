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

/// Native Elixir preset document. This is the stable interchange shape
/// for standalone and plugin state; source-specific data (e.g. raw Vital
/// JSON) is preserved in [`ElixirPreset::source`].
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ElixirPreset {
    pub schema_version: u32,
    pub name: String,
    pub author: Option<String>,
    pub style: Option<String>,
    pub patch: ElixirPatch,
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
