use super::StylePreset;

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
