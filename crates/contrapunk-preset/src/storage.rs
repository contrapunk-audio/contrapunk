use super::StylePreset;
use crate::ContrapunkPresetPayload;

use std::fs;
use std::io::{self, Error, ErrorKind};
use std::path::Path;

/// Exports a single preset as pretty-printed JSON.
pub fn export_preset_json(preset: &StylePreset) -> String {
    serde_json::to_string_pretty(preset).unwrap_or_default()
}

/// Imports a preset from JSON, marking it as non-builtin.
pub fn import_preset_json(json: &str) -> Option<StylePreset> {
    let mut preset: StylePreset = serde_json::from_str(json).ok()?;
    preset.is_builtin = false;
    Some(preset)
}

/// Saves the full preset payload safely.
/// Uses a temp file to prevent data loss on write failure.
pub fn save_preset_to_file(payload: &ContrapunkPresetPayload, file_path: &Path) -> io::Result<()> {
    // Force .cpk extension
    let final_path = file_path.with_extension("cpk");

    // Serialize and propagate errors
    let json_data =
        serde_json::to_string_pretty(payload).map_err(|e| Error::new(ErrorKind::InvalidData, e))?;

    // Write to a temporary file first
    let temp_path = final_path.with_extension("cpk.tmp");
    fs::write(&temp_path, json_data)?;

    // Atomic rename to replace the original file
    fs::rename(temp_path, final_path)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ContrapunkPresetPayload, StylePreset};
    use contrapunk_harmony::{HarmonyMode, Key, OctaveMode, ScaleMode, VoiceLeadingStyle};
    use serde_json::json;
    use std::env;
    use std::fs;

    fn get_dummy_payload() -> ContrapunkPresetPayload {
        ContrapunkPresetPayload {
            version: 1,
            style: StylePreset {
                name: "Test Preset".into(),
                persona: "Test".into(),
                genre: "Test".into(),
                harmony_mode: HarmonyMode::PassThrough, // Fixed based on config.rs
                key: Key::C,                            // Valid Key
                voice_leading_enabled: false,
                voice_leading_style: Default::default(), // Auto-fetch the default style
                octave_mode: OctaveMode::None,           // Fixed based on config.rs
                scale_mode: ScaleMode::Ionian,           // Valid ScaleMode
                interchange_enabled: false,
                borrowing_range: 3,
                is_builtin: false,
            },
            voice_count: 4,
            mixer: json!({}),
            tuning: json!({}),
            routing: json!({}),
            slide_enabled: false,
            companion: json!({}),
        }
    }

    #[test]
    fn test_cpk_extension_enforcement() {
        let payload = get_dummy_payload();

        // Create a path with a WRONG extension
        let mut temp_path = env::temp_dir();
        temp_path.push("wrong_name.txt");

        // Save it using our function
        save_preset_to_file(&payload, &temp_path).expect("Failed to save preset");

        // Verify it forced the .cpk extension
        let expected_path = temp_path.with_extension("cpk");
        assert!(expected_path.exists(), "The .cpk file was not created!");

        // Cleanup
        let _ = fs::remove_file(expected_path);
    }

    #[test]
    fn test_json_round_trip() {
        let payload = get_dummy_payload();

        // Serialize
        let json_str = serde_json::to_string(&payload).expect("Failed to serialize");

        // Deserialize back
        let decoded: ContrapunkPresetPayload =
            serde_json::from_str(&json_str).expect("Failed to deserialize");

        // Verify data integrity
        assert_eq!(payload.version, decoded.version);
        assert_eq!(payload.style.name, decoded.style.name);
        assert_eq!(payload.voice_count, decoded.voice_count);
    }
}
