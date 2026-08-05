//! Tauri commands for preset management.
//!
//! List, load, save, and delete presets.

use std::collections::HashSet;
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::sync::atomic::Ordering;

use serde::Serialize;
use tauri::{AppHandle, Manager, State};

use contrapunk::preset::storage::{load_preset_from_file, save_preset_to_file};
use contrapunk::preset::StylePreset;

use crate::state::AppState;

const MAX_PRESET_NAME_BYTES: usize = 80;

/// Serializable preset info for the frontend.
#[derive(Serialize)]
pub struct PresetInfo {
    pub index: usize,
    pub name: String,
    pub persona: String,
    pub genre: String,
    pub is_builtin: bool,
}

fn normalize_preset_name(name: &str) -> Result<String, String> {
    let name = name.trim();
    if name.is_empty() {
        return Err("Preset name cannot be empty".into());
    }
    if name.len() > MAX_PRESET_NAME_BYTES {
        return Err(format!(
            "Preset name cannot exceed {MAX_PRESET_NAME_BYTES} bytes"
        ));
    }
    if name.chars().any(char::is_control) {
        return Err("Preset name cannot contain control characters".into());
    }
    Ok(name.to_owned())
}

/// Encodes every path-significant byte while keeping ordinary names readable.
fn preset_file_name(name: &str) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut encoded = String::with_capacity(name.len() * 3 + 11);
    encoded.push_str("preset-");
    for &byte in name.as_bytes() {
        match byte {
            b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' => encoded.push(byte as char),
            _ => {
                encoded.push('~');
                encoded.push(HEX[(byte >> 4) as usize] as char);
                encoded.push(HEX[(byte & 0x0f) as usize] as char);
            }
        }
    }
    encoded.push_str(".cpk");
    encoded
}

fn presets_dir(app: &AppHandle) -> Result<PathBuf, String> {
    app.path()
        .app_data_dir()
        .map(|path| path.join("presets"))
        .map_err(|error| format!("Cannot find app data directory: {error}"))
}

fn is_cpk_file(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("cpk"))
}

fn preset_files_in(directory: &Path) -> Result<Vec<PathBuf>, String> {
    let entries = match fs::read_dir(directory) {
        Ok(entries) => entries,
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(error.to_string()),
    };

    let mut files = Vec::new();
    for entry in entries {
        let path = entry.map_err(|error| error.to_string())?.path();
        if is_cpk_file(&path) {
            files.push(path);
        }
    }
    files.sort();
    Ok(files)
}

fn load_custom_presets_from_dir(directory: &Path) -> Result<Vec<StylePreset>, String> {
    let mut loaded = Vec::new();
    for path in preset_files_in(directory)? {
        match load_preset_from_file(&path) {
            Ok(mut preset) => match normalize_preset_name(&preset.name) {
                Ok(name) => {
                    preset.name = name;
                    loaded.push(preset);
                }
                Err(error) => eprintln!("[presets] skipping {}: {error}", path.display()),
            },
            Err(error) => eprintln!("[presets] skipping {}: {error}", path.display()),
        }
    }
    Ok(loaded)
}

fn refresh_custom_presets(app: &AppHandle, state: &AppState) -> Result<(), String> {
    let mut loaded = load_custom_presets_from_dir(&presets_dir(app)?)?;
    let mut manager = state.preset_manager.lock().map_err(|e| e.to_string())?;
    let mut names: HashSet<String> = manager
        .all_presets()
        .into_iter()
        .filter(|preset| preset.is_builtin)
        .map(|preset| preset.name.clone())
        .collect();
    loaded.retain(|preset| {
        if names.insert(preset.name.clone()) {
            true
        } else {
            eprintln!("[presets] skipping duplicate preset: {}", preset.name);
            false
        }
    });
    manager.set_custom_presets(loaded);
    Ok(())
}

fn preset_file_for_name_in(directory: &Path, name: &str) -> Result<PathBuf, String> {
    let expected = directory.join(preset_file_name(name));
    let matches_name = |path: &Path| {
        load_preset_from_file(path)
            .ok()
            .and_then(|preset| normalize_preset_name(&preset.name).ok())
            .is_some_and(|stored_name| stored_name == name)
    };
    if expected.is_file() && matches_name(&expected) {
        return Ok(expected);
    }

    for path in preset_files_in(directory)? {
        if matches_name(&path) {
            return Ok(path);
        }
    }
    Err(format!("Preset file not found: {name}"))
}

fn preset_file_for_name(app: &AppHandle, name: &str) -> Result<PathBuf, String> {
    preset_file_for_name_in(&presets_dir(app)?, name)
}

/// Lists all available presets (builtins + persisted custom styles).
#[tauri::command]
pub fn list_presets(app: AppHandle, state: State<AppState>) -> Result<Vec<PresetInfo>, String> {
    refresh_custom_presets(&app, &state)?;
    let manager = state.preset_manager.lock().map_err(|e| e.to_string())?;
    let presets = manager.all_presets();
    Ok(presets
        .iter()
        .enumerate()
        .map(|(i, p)| PresetInfo {
            index: i,
            name: p.name.clone(),
            persona: p.persona.clone(),
            genre: p.genre.clone(),
            is_builtin: p.is_builtin,
        })
        .collect())
}

/// Loads a preset by name and applies it to the engine.
#[tauri::command]
pub fn load_preset(name: String, app: AppHandle, state: State<AppState>) -> Result<(), String> {
    let name = normalize_preset_name(&name)?;
    refresh_custom_presets(&app, &state)?;
    apply_preset_inner(&state, &name)
}

/// Inner implementation of `load_preset` that operates on a plain
/// `&AppState` reference. Extracted from the Tauri command wrapper so
/// the regression test can exercise the full reharm path without
/// standing up a `tauri::State`.
pub(crate) fn apply_preset_inner(state: &AppState, name: &str) -> Result<(), String> {
    let mut manager = state.preset_manager.lock().map_err(|e| e.to_string())?;

    // Find preset by name — two-phase to avoid borrow conflict
    let (idx, preset) = {
        let all = manager.all_presets();
        let found = all.iter().enumerate().find(|(_, p)| p.name == name);
        match found {
            Some((idx, p)) => (idx, (*p).clone()),
            None => return Err(format!("Preset not found: {}", name)),
        }
    };
    manager.set_active(idx);

    // Apply preset to engine
    {
        let mut engine = state.engine.lock().map_err(|e| e.to_string())?;
        engine.set_key(preset.key);
        engine.set_mode(preset.harmony_mode);
        engine.set_octave_mode(preset.octave_mode);
        engine.set_voice_leading_enabled(preset.voice_leading_enabled);
        engine.set_voice_leading_style(preset.voice_leading_style);
        engine.set_scale_mode(preset.scale_mode);
        engine.set_interchange_enabled(preset.interchange_enabled);
        engine.set_borrowing_range(preset.borrowing_range);
    }

    // Mirror `raise_panic` from commands/harmony.rs — each of the 8
    // setters above stashed held inputs into `pending_reharm_inputs`
    // via `clear_active_for_reharm`. Without raising the panic flag
    // the router's reharm-diff replay never fires, so external synths
    // hold the prior preset's harmony forever (stuck-MIDI-notes on
    // preset switch). 5-line miss that broke a major user flow.
    state.panic_pending.store(true, Ordering::SeqCst);

    Ok(())
}

/// Saves the current harmony style as a custom `.cpk` preset.
#[tauri::command]
pub fn save_preset(name: String, app: AppHandle, state: State<AppState>) -> Result<(), String> {
    refresh_custom_presets(&app, &state)?;
    let name = normalize_preset_name(&name)?;
    {
        let manager = state.preset_manager.lock().map_err(|e| e.to_string())?;
        if manager
            .all_presets()
            .into_iter()
            .any(|preset| preset.name == name)
        {
            return Err(format!("Preset already exists: {name}"));
        }
    }

    let preset = {
        let engine = state.engine.lock().map_err(|e| e.to_string())?;
        StylePreset {
            name: name.clone(),
            persona: "Custom".to_string(),
            genre: "Custom".to_string(),
            harmony_mode: engine.mode(),
            key: engine.key(),
            voice_leading_enabled: engine.voice_leading_enabled(),
            voice_leading_style: engine.voice_leading_style(),
            octave_mode: engine.octave_mode(),
            scale_mode: engine.scale_mode(),
            interchange_enabled: engine.interchange_enabled(),
            borrowing_range: engine.borrowing_range(),
            is_builtin: false,
        }
    };

    let directory = presets_dir(&app)?;
    fs::create_dir_all(&directory).map_err(|error| error.to_string())?;
    save_preset_to_file(&preset, &directory.join(preset_file_name(&name)))
        .map_err(|error| error.to_string())?;

    let mut manager = state.preset_manager.lock().map_err(|e| e.to_string())?;
    manager.add_custom(preset);
    Ok(())
}

/// Deletes a persisted custom preset by name.
#[tauri::command]
pub fn delete_preset(name: String, app: AppHandle, state: State<AppState>) -> Result<(), String> {
    refresh_custom_presets(&app, &state)?;
    let name = normalize_preset_name(&name)?;
    {
        let manager = state.preset_manager.lock().map_err(|e| e.to_string())?;
        if !manager
            .custom_presets()
            .iter()
            .any(|preset| preset.name == name)
        {
            return Err(format!("Custom preset not found: {name}"));
        }
    }

    fs::remove_file(preset_file_for_name(&app, &name)?).map_err(|error| error.to_string())?;

    let mut manager = state.preset_manager.lock().map_err(|e| e.to_string())?;
    let idx = manager
        .custom_presets()
        .iter()
        .position(|preset| preset.name == name)
        .ok_or_else(|| format!("Custom preset not found: {name}"))?;
    manager.remove_custom(idx);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    static TEST_DIR_ID: AtomicU64 = AtomicU64::new(0);

    struct TestDir(PathBuf);

    impl TestDir {
        fn new() -> Self {
            let path = std::env::temp_dir().join(format!(
                "contrapunk-tauri-presets-{}-{}",
                std::process::id(),
                TEST_DIR_ID.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir(&path).unwrap();
            Self(path)
        }
    }

    impl Drop for TestDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn custom_preset(name: &str) -> StylePreset {
        let state = AppState::default();
        let manager = state.preset_manager.lock().unwrap();
        let mut preset = manager.all_presets()[0].clone();
        preset.name = name.into();
        preset.is_builtin = false;
        preset
    }

    /// Regression for commit `b065eb5`: `load_preset` must raise
    /// `panic_pending` so the router-loop drain sends NoteOffs for
    /// stale harmonies after the engine setters mutate. Without this,
    /// external synths hold the prior preset's harmony forever
    /// (stuck-MIDI-notes on preset switch). The fix is one line; the
    /// hole was 5 months old.
    ///
    /// Test mirrors the structural contract: after `apply_preset_inner`
    /// returns, `state.panic_pending` MUST be `true`. The router-loop
    /// drain itself (engine.rs:463-558) is tested separately via the
    /// `drain_all_tracked_notes` cases.
    #[test]
    fn load_preset_raises_panic_pending() {
        let state = AppState::default();
        state.panic_pending.store(false, Ordering::SeqCst);

        // Find any builtin preset to load — the specific contents
        // don't matter, only the side-effect.
        let preset_name = {
            let manager = state.preset_manager.lock().unwrap();
            let all = manager.all_presets();
            assert!(!all.is_empty(), "default AppState should ship builtins");
            all[0].name.clone()
        };

        apply_preset_inner(&state, &preset_name).expect("load_preset failed");
        assert!(
            state.panic_pending.load(Ordering::SeqCst),
            "load_preset must raise panic_pending — without it, stale \
             harmonies stay stuck on external synths after preset switch"
        );
    }

    /// Negative regression: if someone deletes the
    /// `state.panic_pending.store(true, ...)` line in `apply_preset_inner`,
    /// the test above fires. This second case nails down the
    /// no-op-on-error contract: loading an unknown preset must NOT
    /// raise panic (no engine mutation happened, no NoteOff needed).
    #[test]
    fn load_preset_unknown_name_does_not_raise_panic() {
        let state = AppState::default();
        state.panic_pending.store(false, Ordering::SeqCst);

        let result = apply_preset_inner(&state, "nonexistent-preset-zzz");
        assert!(result.is_err(), "unknown preset must return Err");
        assert!(
            !state.panic_pending.load(Ordering::SeqCst),
            "failed load must not raise panic — nothing changed"
        );
    }

    #[test]
    fn preset_names_cannot_escape_or_collide() {
        assert_eq!(
            preset_file_name("../My Preset/🎵"),
            "preset-~2e~2e~2f~4dy~20~50reset~2f~f0~9f~8e~b5.cpk"
        );
        assert_ne!(preset_file_name("My Preset"), preset_file_name("my_preset"));
        assert_ne!(preset_file_name("A"), preset_file_name("a"));
        assert!(normalize_preset_name("  ").is_err());
        assert!(normalize_preset_name("bad\nname").is_err());
        assert!(normalize_preset_name(&"x".repeat(MAX_PRESET_NAME_BYTES + 1)).is_err());
    }

    #[test]
    fn persisted_presets_survive_reload_and_delete_from_disk() {
        let dir = TestDir::new();
        let preset = custom_preset("Saved Style");
        let expected_path = dir.0.join(preset_file_name(&preset.name));
        save_preset_to_file(&custom_preset("Different Style"), &expected_path).unwrap();
        let imported_path = dir.0.join("shared-file.cpk");
        save_preset_to_file(&preset, &imported_path).unwrap();

        let loaded = load_custom_presets_from_dir(&dir.0).unwrap();
        assert_eq!(loaded.len(), 2);
        assert!(loaded.iter().any(|loaded| loaded.name == preset.name));

        let stored_path = preset_file_for_name_in(&dir.0, &preset.name).unwrap();
        assert_eq!(stored_path, imported_path);
        fs::remove_file(stored_path).unwrap();
        let remaining = load_custom_presets_from_dir(&dir.0).unwrap();
        assert_eq!(remaining.len(), 1);
        assert_eq!(remaining[0].name, "Different Style");
    }
}
