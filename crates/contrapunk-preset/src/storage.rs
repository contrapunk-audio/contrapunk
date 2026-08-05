use super::StylePreset;

use serde::{Deserialize, Serialize};
use std::fs::{self, OpenOptions};
use std::io::{self, Error, ErrorKind, Write};
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};

const PRESET_FILE_VERSION: u32 = 1;
static TEMP_FILE_ID: AtomicU64 = AtomicU64::new(0);

#[derive(Serialize, Deserialize)]
struct PresetFile {
    version: u32,
    style: StylePreset,
}

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

/// Saves a versioned style preset to a new `.cpk` file without replacing an
/// existing preset. The completed temporary file is linked into place only
/// after its contents have been flushed to disk.
pub fn save_preset_to_file(preset: &StylePreset, file_path: &Path) -> io::Result<()> {
    let final_path = file_path.with_extension("cpk");
    if final_path.exists() {
        return Err(Error::new(
            ErrorKind::AlreadyExists,
            format!("preset already exists: {}", final_path.display()),
        ));
    }

    let data = serde_json::to_vec_pretty(&PresetFile {
        version: PRESET_FILE_VERSION,
        style: preset.clone(),
    })
    .map_err(|error| Error::new(ErrorKind::InvalidData, error))?;
    let temp_path = final_path.with_extension(format!(
        "cpk.{}.{}.tmp",
        std::process::id(),
        TEMP_FILE_ID.fetch_add(1, Ordering::Relaxed)
    ));

    let result = (|| {
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temp_path)?;
        file.write_all(&data)?;
        file.sync_all()?;
        drop(file);

        // hard_link is an atomic no-clobber install on the supported desktop
        // filesystems; unlike rename, it cannot replace an existing Windows file.
        fs::hard_link(&temp_path, &final_path)
    })();
    let _ = fs::remove_file(temp_path);
    result
}

/// Loads one versioned `.cpk` style preset.
pub fn load_preset_from_file(file_path: &Path) -> io::Result<StylePreset> {
    let data = fs::read(file_path)?;
    let stored: PresetFile =
        serde_json::from_slice(&data).map_err(|error| Error::new(ErrorKind::InvalidData, error))?;
    if stored.version != PRESET_FILE_VERSION {
        return Err(Error::new(
            ErrorKind::InvalidData,
            format!("unsupported preset version: {}", stored.version),
        ));
    }

    let mut preset = stored.style;
    preset.is_builtin = false;
    Ok(preset)
}

#[cfg(test)]
mod tests {
    use super::*;
    use contrapunk_harmony::{HarmonyMode, Key, OctaveMode, ScaleMode, VoiceLeadingStyle};
    use std::path::{Path, PathBuf};

    static TEST_DIR_ID: AtomicU64 = AtomicU64::new(0);

    struct TestDir(PathBuf);

    impl TestDir {
        fn new(label: &str) -> Self {
            let path = std::env::temp_dir().join(format!(
                "contrapunk-preset-{label}-{}-{}",
                std::process::id(),
                TEST_DIR_ID.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir(&path).unwrap();
            Self(path)
        }

        fn path(&self) -> &Path {
            &self.0
        }
    }

    impl Drop for TestDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn preset(name: &str) -> StylePreset {
        StylePreset {
            name: name.into(),
            persona: "Test".into(),
            genre: "Test".into(),
            harmony_mode: HarmonyMode::PassThrough,
            key: Key::C,
            voice_leading_enabled: false,
            voice_leading_style: VoiceLeadingStyle::default(),
            octave_mode: OctaveMode::None,
            scale_mode: ScaleMode::Ionian,
            interchange_enabled: false,
            borrowing_range: 3,
            is_builtin: false,
        }
    }

    #[test]
    fn save_enforces_extension_and_round_trips() {
        let dir = TestDir::new("round-trip");
        let requested_path = dir.path().join("custom.txt");
        let expected_path = requested_path.with_extension("cpk");
        let original = preset("Round Trip");

        save_preset_to_file(&original, &requested_path).unwrap();

        assert!(expected_path.exists());
        assert!(!requested_path.exists());
        let loaded = load_preset_from_file(&expected_path).unwrap();
        assert_eq!(export_preset_json(&original), export_preset_json(&loaded));
    }

    #[test]
    fn save_preserves_an_existing_preset() {
        let dir = TestDir::new("no-clobber");
        let path = dir.path().join("custom.cpk");
        save_preset_to_file(&preset("Original"), &path).unwrap();

        let error = save_preset_to_file(&preset("Replacement"), &path).unwrap_err();

        assert_eq!(error.kind(), ErrorKind::AlreadyExists);
        assert_eq!(load_preset_from_file(&path).unwrap().name, "Original");
        assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 1);
    }

    #[test]
    fn load_rejects_unknown_versions() {
        let dir = TestDir::new("version");
        let path = dir.path().join("future.cpk");
        let data = serde_json::json!({ "version": 2, "style": preset("Future") });
        fs::write(&path, serde_json::to_vec(&data).unwrap()).unwrap();

        let error = load_preset_from_file(&path).unwrap_err();

        assert_eq!(error.kind(), ErrorKind::InvalidData);
        assert!(error.to_string().contains("unsupported preset version"));
    }
}
