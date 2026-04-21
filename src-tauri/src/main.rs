//! Tauri v2 application entry point for Contrapunk.
//!
//! Registers all command handlers and managed state, then launches
//! the Tauri desktop application.

// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod audio_clock;
mod commands;
mod guitar_bridge;
mod state;

use std::sync::Arc;

use state::AppState;
use tauri::Manager;

fn main() {
    tauri::Builder::default()
        .manage(AppState::default())
        .setup(|app| {
            // Start the audio clock: a silent cpal output stream that ticks
            // the Transport and emits `beat-update` events.
            let transport = Arc::clone(&app.state::<AppState>().transport);
            if let Err(e) = audio_clock::start(app.handle().clone(), transport) {
                eprintln!(
                    "[main] audio_clock::start failed: {} (transport will not tick)",
                    e
                );
            }
            Ok(())
        })
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
            commands::harmony::set_routing_mode,
            commands::harmony::set_counterpoint_species,
            commands::harmony::set_voice_count,
            commands::harmony::set_counterpoint_strictness,
            commands::harmony::set_detune,
            commands::harmony::get_detune,
            // MIDI device management
            commands::midi::list_midi_inputs,
            commands::midi::list_midi_outputs,
            commands::midi::refresh_midi_devices,
            // Engine routing
            commands::engine::start_routing,
            commands::engine::stop_routing,
            commands::engine::get_note_state,
            commands::engine::inject_note_on,
            commands::engine::inject_note_off,
            // Guitar audio input
            commands::guitar::set_guitar_device,
            commands::guitar::set_guitar_config,
            commands::guitar::list_audio_devices,
            // Presets
            commands::presets::list_presets,
            commands::presets::load_preset,
            commands::presets::save_preset,
            commands::presets::delete_preset,
            // Transport / clock
            commands::transport::get_transport_state,
            commands::transport::transport_play,
            commands::transport::transport_stop,
            commands::transport::transport_reset,
            commands::transport::set_bpm,
            commands::transport::set_time_signature,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
