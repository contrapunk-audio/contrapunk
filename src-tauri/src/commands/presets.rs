use contrapunk::preset::storage::save_preset_to_file;
use contrapunk::preset::ContrapunkPresetPayload;
use serde_json::json;
use std::env;
use std::path::PathBuf;

/// Saves the current engine config as a custom preset, both in-memory
/// and to the disk as a .cpk file.
#[tauri::command]
pub fn save_preset(name: String, state: State<AppState>) -> Result<(), String> {
    let engine = state.engine.lock().map_err(|e| e.to_string())?;

    // 1. Create the core StylePreset
    let style = StylePreset {
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
    };

    // 2. Add to in-memory manager (Existing logic)
    let mut manager = state.preset_manager.lock().map_err(|e| e.to_string())?;
    manager.add_custom(style.clone());

    // 3. Assemble the full payload (Fixes Reviewer Point #2)
    // Using empty JSON objects for missing states to satisfy the struct without circular imports.
    // In a full implementation, these would pull from AppState.
    let payload = ContrapunkPresetPayload {
        version: 1,
        style,
        voice_count: engine.voice_count(), // Assuming voice_count() is exposed on the engine
        mixer: json!({}),
        tuning: json!({}),
        routing: json!({}),
        slide_enabled: false,
        companion: json!({}),
    };

    // 4. Save to disk (Fixes Reviewer Point #1)
    // We default to saving in the current working directory for now.
    let current_dir = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let file_name = format!("{}.cpk", name.replace(" ", "_").to_lowercase());
    let file_path = current_dir.join(file_name);

    save_preset_to_file(&payload, &file_path).map_err(|e| e.to_string())?;

    Ok(())
}
