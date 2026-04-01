//! Tauri commands for guitar audio input device and DSP configuration.
//!
//! Provides commands to enumerate audio input devices, select the
//! guitar input device/channel, and configure the DSP pipeline
//! parameters before starting routing.

use cpal::traits::{DeviceTrait, HostTrait};
use tauri::State;

use contrapunk::audio::guitar_input::GuitarInputConfig;

use crate::state::AppState;

/// Set the guitar audio input device and channel.
///
/// Called from the UI when the user selects an audio device and
/// channel for guitar input. These values are read by the routing
/// thread when "Guitar Audio" is selected as the input source.
#[tauri::command]
pub fn set_guitar_device(
    device_name: String,
    channel: usize,
    state: State<AppState>,
) -> Result<(), String> {
    *state.guitar_device.lock().map_err(|e| e.to_string())? = device_name;
    *state.guitar_channel.lock().map_err(|e| e.to_string())? = channel;
    Ok(())
}

/// Set the guitar DSP pipeline configuration.
///
/// Configures latency, gain, string detection confidence, and
/// expressive technique toggles (bends, legato, slides, vibrato).
/// The config is stored in AppState and read when the guitar bridge
/// is created at routing start.
#[tauri::command]
pub fn set_guitar_config(
    latency_ms: f32,
    gain: f32,
    string_confidence: f32,
    bends: bool,
    legato: bool,
    slides: bool,
    vibrato: bool,
    state: State<AppState>,
) -> Result<(), String> {
    let sample_rate = 48000; // default; updated by bridge on actual start
    let config = GuitarInputConfig {
        buffer_size: GuitarInputConfig::buffer_size_for_latency(latency_ms, sample_rate),
        sample_rate,
        onset_threshold: 0.015,
        string_confidence_min: string_confidence,
        bends_enabled: bends,
        legato_enabled: legato,
        slides_enabled: slides,
        vibrato_detection: vibrato,
        vibrato_passthrough: true,
        filter_enabled: false,
        min_clarity: 0.40,
        cooldown_samples: sample_rate / 10,
        n_harmonics: 6,
        input_gain: gain,
        flux_threshold: 0.5,
        per_string_channels: true,
        pitch_bend_range: 2,
        pressure_enabled: true,
        pressure_hold: 0.3,
        brightness_enabled: true,
    };
    *state.guitar_config.lock().map_err(|e| e.to_string())? = Some(config);
    Ok(())
}

/// List available audio input devices.
///
/// Enumerates all cpal input devices by name, for the UI to display
/// in a device selection dropdown.
#[tauri::command]
pub fn list_audio_devices() -> Result<Vec<String>, String> {
    let host = cpal::default_host();
    let devices = host
        .input_devices()
        .map_err(|e| format!("Failed to enumerate audio devices: {}", e))?;
    Ok(devices.map(|d| d.name().unwrap_or_default()).collect())
}
