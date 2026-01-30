use super::StylePreset;

/// Saves custom presets to eframe storage as JSON.
#[cfg(feature = "gui")]
pub fn save_custom_presets(storage: &mut dyn eframe::Storage, presets: &[StylePreset]) {
    if let Ok(json) = serde_json::to_string(presets) {
        storage.set_string("custom_presets", json);
    }
}

/// Loads custom presets from eframe storage.
#[cfg(feature = "gui")]
pub fn load_custom_presets(storage: &dyn eframe::Storage) -> Vec<StylePreset> {
    storage
        .get_string("custom_presets")
        .and_then(|json| serde_json::from_str(&json).ok())
        .unwrap_or_default()
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
