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
        .manage(AppState::new())
        .setup(|app| {
            // Start the audio clock: a cpal output stream that ticks the
            // Transport, emits `beat-update` events, synthesizes metronome
            // clicks, and renders the built-in synth.
            let state = app.state::<AppState>();
            let transport = Arc::clone(&state.transport);
            let metronome_enabled = Arc::clone(&state.metronome_enabled);
            let synth_params = Arc::clone(&state.synth_params);
            let synth_rx = state
                .synth_rx
                .lock()
                .ok()
                .and_then(|mut guard| guard.take());
            let reverb_params = Arc::clone(&state.reverb_params);
            let delay_params = Arc::clone(&state.delay_params);
            match audio_clock::start(
                app.handle().clone(),
                transport,
                metronome_enabled,
                synth_params,
                synth_rx,
                reverb_params,
                delay_params,
            ) {
                Ok(commander) => {
                    if let Ok(mut slot) = state.chain_commander.lock() {
                        *slot = Some(commander);
                    }
                }
                Err(e) => {
                    eprintln!(
                        "[main] audio_clock::start failed: {} (transport will not tick)",
                        e
                    );
                }
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
            commands::harmony::set_octave_intensity,
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
            // Per-voice output routing
            commands::routing::set_voice_output,
            commands::routing::get_voice_outputs,
            // Guitar audio input
            commands::guitar::set_guitar_device,
            commands::guitar::set_guitar_config,
            commands::guitar::get_full_guitar_config,
            commands::guitar::set_full_guitar_config,
            commands::guitar::list_audio_devices,
            // Debug window
            commands::debug::open_debug_pipeline_window,
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
            commands::transport::set_metronome_enabled,
            // Built-in synth
            commands::synth::get_synth_state,
            commands::synth::set_synth_enabled,
            commands::synth::set_synth_waveform,
            commands::synth::set_synth_attack_ms,
            commands::synth::set_synth_decay_ms,
            commands::synth::set_synth_sustain,
            commands::synth::set_synth_release_ms,
            commands::synth::set_synth_cutoff_hz,
            commands::synth::set_synth_resonance,
            commands::synth::set_synth_master_gain,
            // Built-in FX (reverb + delay)
            commands::fx::get_reverb_state,
            commands::fx::set_reverb_enabled,
            commands::fx::set_reverb_mix,
            commands::fx::set_reverb_room_size,
            commands::fx::set_reverb_damping,
            commands::fx::get_delay_state,
            commands::fx::set_delay_enabled,
            commands::fx::set_delay_mix,
            commands::fx::set_delay_time_ms,
            commands::fx::set_delay_feedback,
            // Audio chain topology
            commands::chain::list_chain_blocks,
            commands::chain::remove_chain_block,
            commands::chain::clear_chain,
            // CLAP plugin hosting
            commands::plugins::list_clap_plugins,
            commands::plugins::add_clap_plugin_to_chain,
            commands::plugins::open_plugin_gui,
            commands::plugins::open_plugin_gui_embedded,
            commands::plugins::set_plugin_gui_frame,
            commands::plugins::close_plugin_gui,
            commands::plugins::remove_plugin,
            commands::plugins::get_plugin_gui_size,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
