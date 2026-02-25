//! Tauri commands for MIDI device management.
//!
//! List available MIDI input and output devices.

use serde::Serialize;
use tauri::State;

use contrapunk::midi::ports;

use crate::state::AppState;

/// Serializable MIDI device info.
#[derive(Serialize)]
pub struct MidiDeviceInfo {
    pub index: usize,
    pub name: String,
}

/// Lists available MIDI input devices.
#[tauri::command]
pub fn list_midi_inputs(_state: State<AppState>) -> Result<Vec<MidiDeviceInfo>, String> {
    let inputs = ports::list_input_ports().map_err(|e| e.to_string())?;
    Ok(inputs
        .into_iter()
        .map(|(index, name)| MidiDeviceInfo { index, name })
        .collect())
}

/// Lists available MIDI output devices.
#[tauri::command]
pub fn list_midi_outputs(_state: State<AppState>) -> Result<Vec<MidiDeviceInfo>, String> {
    let outputs = ports::list_output_ports().map_err(|e| e.to_string())?;
    Ok(outputs
        .into_iter()
        .map(|(index, name)| MidiDeviceInfo { index, name })
        .collect())
}

/// Re-enumerates MIDI devices (forces a fresh scan).
///
/// Returns both input and output device lists.
#[tauri::command]
pub fn refresh_midi_devices(
    _state: State<AppState>,
) -> Result<(Vec<MidiDeviceInfo>, Vec<MidiDeviceInfo>), String> {
    let inputs = ports::list_input_ports().map_err(|e| e.to_string())?;
    let outputs = ports::list_output_ports().map_err(|e| e.to_string())?;
    Ok((
        inputs
            .into_iter()
            .map(|(index, name)| MidiDeviceInfo { index, name })
            .collect(),
        outputs
            .into_iter()
            .map(|(index, name)| MidiDeviceInfo { index, name })
            .collect(),
    ))
}
