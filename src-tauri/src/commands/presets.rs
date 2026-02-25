//! Tauri commands for preset management.
//!
//! List, load, save, and delete presets.

use serde::Serialize;
use tauri::State;

use contrapunk::preset::StylePreset;

use crate::state::AppState;

/// Serializable preset info for the frontend.
#[derive(Serialize)]
pub struct PresetInfo {
    pub index: usize,
    pub name: String,
    pub persona: String,
    pub genre: String,
    pub is_builtin: bool,
}

/// Lists all available presets (builtins + custom).
#[tauri::command]
pub fn list_presets(state: State<AppState>) -> Result<Vec<PresetInfo>, String> {
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
pub fn load_preset(name: String, state: State<AppState>) -> Result<(), String> {
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
    let mut engine = state.engine.lock().map_err(|e| e.to_string())?;
    engine.set_key(preset.key);
    engine.set_mode(preset.harmony_mode);
    engine.set_octave_mode(preset.octave_mode);
    engine.set_voice_leading_enabled(preset.voice_leading_enabled);
    engine.set_voice_leading_style(preset.voice_leading_style);
    engine.set_scale_mode(preset.scale_mode);
    engine.set_interchange_enabled(preset.interchange_enabled);
    engine.set_borrowing_range(preset.borrowing_range);

    // Apply humanize config
    let mut hconfig = state.humanize_config.lock().map_err(|e| e.to_string())?;
    *hconfig = preset.humanize_config;

    Ok(())
}

/// Saves the current engine config as a custom preset.
#[tauri::command]
pub fn save_preset(name: String, state: State<AppState>) -> Result<(), String> {
    let engine = state.engine.lock().map_err(|e| e.to_string())?;
    let hconfig = state.humanize_config.lock().map_err(|e| e.to_string())?;

    let preset = StylePreset {
        name: name.clone(),
        persona: "Custom".to_string(),
        genre: "Custom".to_string(),
        harmony_mode: engine.mode(),
        key: engine.key(),
        voice_leading_enabled: engine.voice_leading_enabled(),
        voice_leading_style: engine.voice_leading_style(),
        octave_mode: engine.octave_mode(),
        humanize_config: hconfig.clone(),
        scale_mode: engine.scale_mode(),
        interchange_enabled: engine.interchange_enabled(),
        borrowing_range: engine.borrowing_range(),
        is_builtin: false,
    };

    let mut manager = state.preset_manager.lock().map_err(|e| e.to_string())?;
    manager.add_custom(preset);
    Ok(())
}

/// Deletes a custom preset by name.
#[tauri::command]
pub fn delete_preset(name: String, state: State<AppState>) -> Result<(), String> {
    let mut manager = state.preset_manager.lock().map_err(|e| e.to_string())?;

    // Find index in custom presets
    let custom = manager.custom_presets();
    let idx = custom
        .iter()
        .position(|p| p.name == name)
        .ok_or_else(|| format!("Custom preset not found: {}", name))?;

    manager.remove_custom(idx);
    Ok(())
}
