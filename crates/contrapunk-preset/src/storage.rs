use super::StylePreset;

//Imports for File system (fs) and path handling
use std::fs;
use std::path::Path;
use std::io;

/// Exports a single preset as pretty-printed JSON.
pub fn export_preset_json(preset: &StylePreset) -> String 
{
    serde_json::to_string_pretty(preset).unwrap_or_default()
}

/// Imports a preset from JSON, marking it as non-builtin.
pub fn import_preset_json(json: &str) -> Option<StylePreset> 
{
    let mut preset: StylePreset = serde_json::from_str(json).ok()?;
    preset.is_builtin = false;
    Some(preset)
}

/// To save preset with new custom extention for CONTRAPUNK .cpk
/// This function uses original `export_preset_json`for data serialize
/// And ensures that saved file extension is strictly ".cpk"

pub fn save_preset_to_file(preset: &StylePreset, file_path: &Path) -> io::Result<()> 
{
    // Serialize preset using existing function

    let json_data = export_preset_json(preset);
    
    // Add .cpk extension wherever user want
    
    let final_path = file_path.with_extension("cpk");
    
    // Write Serialized data onto the system file

    fs::write(final_path, json_data)
}