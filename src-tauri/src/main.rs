//! Tauri v2 application entry point for Contrapunk.
//!
//! Registers all command handlers and managed state, then launches
//! the Tauri desktop application.

// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod guitar_bridge;
mod state;

use state::AppState;

fn main() {
    tauri::Builder::default()
        .manage(AppState::default())
        .invoke_handler(tauri::generate_handler![
            // Harmony engine control
            commands::harmony::get_engine_state,
            commands::harmony::set_key,
            commands::harmony::set_mode,
            commands::harmony::set_scale_mode,
            commands::harmony::set_octave_mode,
            commands::harmony::set_voice_leading,
            commands::harmony::set_interchange,
            commands::harmony::set_voice_position,
            commands::harmony::set_auto_key,
            // MIDI device management
            commands::midi::list_midi_inputs,
            commands::midi::list_midi_outputs,
            commands::midi::refresh_midi_devices,
            // Engine routing
            commands::engine::start_routing,
            commands::engine::stop_routing,
            commands::engine::get_note_state,
            // Guitar audio input
            commands::guitar::set_guitar_device,
            commands::guitar::set_guitar_config,
            commands::guitar::list_audio_devices,
            // Presets
            commands::presets::list_presets,
            commands::presets::load_preset,
            commands::presets::save_preset,
            commands::presets::delete_preset,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
