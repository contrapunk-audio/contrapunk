//! Tauri commands for guitar audio input device and DSP configuration.
//!
//! Provides commands to enumerate audio input devices, select the
//! guitar input device/channel, and configure the DSP pipeline
//! parameters before starting routing.

use std::sync::mpsc;
use std::time::{Duration, Instant};

use cpal::traits::{DeviceTrait, HostTrait};
use cpal::{SampleFormat, StreamConfig};
use serde::Serialize;
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

// ============================================================================
// Guitar calibration
// ============================================================================

/// Standard tuning open-string names and expected frequencies (Hz).
const OPEN_STRINGS: [(&str, f64); 6] = [
    ("E2", 82.41),
    ("A2", 110.00),
    ("D3", 146.83),
    ("G3", 196.00),
    ("B3", 246.94),
    ("E4", 329.63),
];

/// Per-string calibration data captured during the calibration flow.
#[derive(Clone, Debug, Serialize)]
pub struct StringCalibration {
    pub name: String,
    pub expected_freq: f64,
    pub measured_freq: f64,
    pub rms: f32,
    pub cents_offset: f64,
}

/// Full calibration profile returned to the UI and saved to disk.
#[derive(Clone, Debug, Serialize)]
pub struct GuitarCalibrationProfile {
    pub noise_floor_rms: f32,
    pub strings: Vec<StringCalibration>,
    pub timestamp: String,
}

/// Start a simplified guitar calibration flow.
///
/// 1. Measures noise floor (3 seconds of silence)
/// 2. For each of 6 strings, waits for a pluck then captures 0.5s
/// 3. Measures frequency via zero-crossing estimation
/// 4. Saves the calibration profile to disk
///
/// This is an async Tauri command because it runs for ~10+ seconds.
#[tauri::command]
pub async fn start_guitar_calibration(
    state: State<'_, AppState>,
) -> Result<GuitarCalibrationProfile, String> {
    // Read device/channel from state
    let device_name = state
        .guitar_device
        .lock()
        .map_err(|e| e.to_string())?
        .clone();
    let channel = *state.guitar_channel.lock().map_err(|e| e.to_string())?;

    // Run the blocking calibration on a separate thread
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let result = run_calibration(&device_name, channel);
        let _ = tx.send(result);
    });

    rx.recv()
        .map_err(|e| format!("Calibration thread failed: {}", e))?
}

/// Blocking calibration implementation (runs on a worker thread).
fn run_calibration(device_name: &str, channel: usize) -> Result<GuitarCalibrationProfile, String> {
    let host = cpal::default_host();

    // Find the requested device (or default)
    let device = if device_name.is_empty() {
        host.default_input_device()
            .ok_or_else(|| "No default audio input device".to_string())?
    } else {
        host.input_devices()
            .map_err(|e| format!("Cannot enumerate audio devices: {}", e))?
            .find(|d| d.name().map(|n| n == device_name).unwrap_or(false))
            .ok_or_else(|| format!("Audio device '{}' not found", device_name))?
    };

    let supported = device
        .default_input_config()
        .map_err(|e| format!("No supported input config: {}", e))?;

    let sample_rate = supported.sample_rate().0 as f64;
    let channels = supported.channels() as usize;
    let sample_format = supported.sample_format();

    let config: StreamConfig = supported.into();

    // ── Phase 1: Measure noise floor (3 seconds) ──────────────────
    let noise_rms = capture_rms_segment(
        &device,
        &config,
        sample_format,
        channels,
        channel,
        Duration::from_secs(3),
    )?;

    // ── Phase 2: Capture each open string ─────────────────────────
    let pluck_threshold = (noise_rms * 5.0).max(0.005);
    let mut string_cals = Vec::new();

    for (name, expected_freq) in &OPEN_STRINGS {
        let samples = capture_pluck(
            &device,
            &config,
            sample_format,
            channels,
            channel,
            pluck_threshold,
            Duration::from_millis(500),
            Duration::from_secs(15), // max wait per string
        )?;

        if samples.is_empty() {
            // Timed out waiting for pluck — record a miss
            string_cals.push(StringCalibration {
                name: name.to_string(),
                expected_freq: *expected_freq,
                measured_freq: 0.0,
                rms: 0.0,
                cents_offset: 0.0,
            });
            continue;
        }

        let rms = compute_rms(&samples);
        let measured_freq = estimate_frequency(&samples, sample_rate);
        let cents_offset = if measured_freq > 0.0 {
            1200.0 * (measured_freq / expected_freq).log2()
        } else {
            0.0
        };

        string_cals.push(StringCalibration {
            name: name.to_string(),
            expected_freq: *expected_freq,
            measured_freq,
            rms,
            cents_offset,
        });
    }

    let profile = GuitarCalibrationProfile {
        noise_floor_rms: noise_rms,
        strings: string_cals,
        timestamp: chrono_timestamp(),
    };

    // Save to disk
    if let Ok(json) = serde_json::to_string_pretty(&profile) {
        let _ = std::fs::write("guitar_calibration_profile.json", json);
    }

    Ok(profile)
}

/// Capture audio for a fixed duration, return RMS of the target channel.
fn capture_rms_segment(
    device: &cpal::Device,
    config: &StreamConfig,
    sample_format: SampleFormat,
    channels: usize,
    target_channel: usize,
    duration: Duration,
) -> Result<f32, String> {
    let samples = capture_audio_segment(
        device,
        config,
        sample_format,
        channels,
        target_channel,
        duration,
    )?;
    Ok(compute_rms(&samples))
}

/// Capture raw audio samples from the target channel for a fixed duration.
fn capture_audio_segment(
    device: &cpal::Device,
    config: &StreamConfig,
    sample_format: SampleFormat,
    channels: usize,
    target_channel: usize,
    duration: Duration,
) -> Result<Vec<f32>, String> {
    let (tx, rx) = mpsc::channel::<Vec<f32>>();
    let ch = target_channel.min(channels.saturating_sub(1));

    let stream = match sample_format {
        SampleFormat::F32 => {
            let tx = tx.clone();
            device.build_input_stream(
                config,
                move |data: &[f32], _: &cpal::InputCallbackInfo| {
                    let mono: Vec<f32> = data
                        .chunks(channels)
                        .filter_map(|frame| frame.get(ch).copied())
                        .collect();
                    let _ = tx.send(mono);
                },
                |err| eprintln!("[calibration] Stream error: {}", err),
                None,
            )
        }
        SampleFormat::I16 => {
            let tx = tx.clone();
            device.build_input_stream(
                config,
                move |data: &[i16], _: &cpal::InputCallbackInfo| {
                    let mono: Vec<f32> = data
                        .chunks(channels)
                        .filter_map(|frame| frame.get(ch).map(|&s| s as f32 / 32768.0))
                        .collect();
                    let _ = tx.send(mono);
                },
                |err| eprintln!("[calibration] Stream error: {}", err),
                None,
            )
        }
        _ => return Err(format!("Unsupported sample format: {:?}", sample_format)),
    }
    .map_err(|e| format!("Failed to build input stream: {}", e))?;

    use cpal::traits::StreamTrait;
    stream
        .play()
        .map_err(|e| format!("Failed to start stream: {}", e))?;

    let mut all_samples = Vec::new();
    let start = Instant::now();
    while start.elapsed() < duration {
        match rx.recv_timeout(Duration::from_millis(100)) {
            Ok(chunk) => all_samples.extend(chunk),
            Err(mpsc::RecvTimeoutError::Timeout) => {}
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
        }
    }

    drop(stream);
    Ok(all_samples)
}

/// Wait for a pluck (RMS exceeds threshold), then capture `capture_duration` of audio.
fn capture_pluck(
    device: &cpal::Device,
    config: &StreamConfig,
    sample_format: SampleFormat,
    channels: usize,
    target_channel: usize,
    threshold: f32,
    capture_duration: Duration,
    max_wait: Duration,
) -> Result<Vec<f32>, String> {
    let (tx, rx) = mpsc::channel::<Vec<f32>>();
    let ch = target_channel.min(channels.saturating_sub(1));

    let stream = match sample_format {
        SampleFormat::F32 => {
            let tx = tx.clone();
            device.build_input_stream(
                config,
                move |data: &[f32], _: &cpal::InputCallbackInfo| {
                    let mono: Vec<f32> = data
                        .chunks(channels)
                        .filter_map(|frame| frame.get(ch).copied())
                        .collect();
                    let _ = tx.send(mono);
                },
                |err| eprintln!("[calibration] Stream error: {}", err),
                None,
            )
        }
        SampleFormat::I16 => {
            let tx = tx.clone();
            device.build_input_stream(
                config,
                move |data: &[i16], _: &cpal::InputCallbackInfo| {
                    let mono: Vec<f32> = data
                        .chunks(channels)
                        .filter_map(|frame| frame.get(ch).map(|&s| s as f32 / 32768.0))
                        .collect();
                    let _ = tx.send(mono);
                },
                |err| eprintln!("[calibration] Stream error: {}", err),
                None,
            )
        }
        _ => return Err(format!("Unsupported sample format: {:?}", sample_format)),
    }
    .map_err(|e| format!("Failed to build input stream: {}", e))?;

    use cpal::traits::StreamTrait;
    stream
        .play()
        .map_err(|e| format!("Failed to start stream: {}", e))?;

    let start = Instant::now();
    let mut detected = false;
    let mut capture_start: Option<Instant> = None;
    let mut all_samples = Vec::new();

    loop {
        if start.elapsed() > max_wait {
            break; // Timed out
        }

        match rx.recv_timeout(Duration::from_millis(50)) {
            Ok(chunk) => {
                if !detected {
                    let chunk_rms = compute_rms(&chunk);
                    if chunk_rms > threshold {
                        detected = true;
                        capture_start = Some(Instant::now());
                        all_samples.extend(chunk);
                    }
                } else {
                    all_samples.extend(chunk);
                    if let Some(cs) = capture_start {
                        if cs.elapsed() >= capture_duration {
                            break;
                        }
                    }
                }
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {}
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
        }
    }

    drop(stream);
    Ok(all_samples)
}

/// Compute RMS of a sample buffer.
fn compute_rms(samples: &[f32]) -> f32 {
    if samples.is_empty() {
        return 0.0;
    }
    let sum_sq: f64 = samples.iter().map(|&s| (s as f64) * (s as f64)).sum();
    (sum_sq / samples.len() as f64).sqrt() as f32
}

/// Simple zero-crossing frequency estimation.
fn estimate_frequency(samples: &[f32], sample_rate: f64) -> f64 {
    if samples.len() < 4 {
        return 0.0;
    }

    // Count zero crossings (positive-going)
    let mut crossings = Vec::new();
    for i in 1..samples.len() {
        if samples[i - 1] <= 0.0 && samples[i] > 0.0 {
            // Linear interpolation for sub-sample accuracy
            let frac = -samples[i - 1] as f64 / (samples[i] as f64 - samples[i - 1] as f64);
            crossings.push(i as f64 - 1.0 + frac);
        }
    }

    if crossings.len() < 2 {
        return 0.0;
    }

    // Average period between successive zero crossings
    let total_span = crossings.last().unwrap() - crossings.first().unwrap();
    let num_periods = (crossings.len() - 1) as f64;
    let avg_period_samples = total_span / num_periods;

    if avg_period_samples <= 0.0 {
        return 0.0;
    }

    sample_rate / avg_period_samples
}

/// Simple ISO-8601 timestamp without pulling in chrono.
fn chrono_timestamp() -> String {
    use std::time::SystemTime;
    match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
        Ok(d) => format!("{}", d.as_secs()),
        Err(_) => "0".to_string(),
    }
}
