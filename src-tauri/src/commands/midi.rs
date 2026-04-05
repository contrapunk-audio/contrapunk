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

/// Response for refresh_midi_devices containing both input and output lists.
#[derive(Serialize)]
pub struct MidiDeviceRefreshResponse {
    pub inputs: Vec<MidiDeviceInfo>,
    pub outputs: Vec<MidiDeviceInfo>,
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
/// Returns both input and output device lists as a struct (not a tuple,
/// which can cause serialization issues with Tauri v2 IPC).
#[tauri::command]
pub fn refresh_midi_devices(_state: State<AppState>) -> Result<MidiDeviceRefreshResponse, String> {
    let inputs = ports::list_input_ports().map_err(|e| e.to_string())?;
    let outputs = ports::list_output_ports().map_err(|e| e.to_string())?;
    Ok(MidiDeviceRefreshResponse {
        inputs: inputs
            .into_iter()
            .map(|(index, name)| MidiDeviceInfo { index, name })
            .collect(),
        outputs: outputs
            .into_iter()
            .map(|(index, name)| MidiDeviceInfo { index, name })
            .collect(),
    })
}
