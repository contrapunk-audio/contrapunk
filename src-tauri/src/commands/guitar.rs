//! Tauri commands for guitar audio input device and DSP configuration.
//!
//! Provides commands to enumerate audio input devices, select the
//! guitar input device/channel, and configure the DSP pipeline
//! parameters before starting routing.

use cpal::traits::{DeviceTrait, HostTrait};
use serde::Serialize;
use tauri::{Manager, State};

use contrapunk::audio::guitar::GuitarCalibrationProfile;
use contrapunk::audio::guitar_input::GuitarInputConfig;

use crate::state::AppState;

/// Filename used for the persisted per-string calibration profile inside
/// the OS-specific app-data directory. Matches the filename produced by
/// `examples/guitar_calibrate.rs` so users can drop a CLI-generated
/// profile into the directory by hand and pick it up on next launch.
const CALIBRATION_FILENAME: &str = "guitar_calibration_profile.json";

/// Resolve the absolute path of the calibration profile file inside the
/// app's per-user data directory. Creates the directory tree if missing.
fn calibration_profile_path(app: &tauri::AppHandle) -> Result<std::path::PathBuf, String> {
    let dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to resolve app_data_dir: {}", e))?;
    if !dir.exists() {
        std::fs::create_dir_all(&dir).map_err(|e| format!("mkdir app_data_dir: {}", e))?;
    }
    Ok(dir.join(CALIBRATION_FILENAME))
}

/// Status returned to the UI: whether a non-default profile is loaded,
/// the absolute path the load/save uses, and per-string sample counts.
#[derive(Debug, Clone, Serialize)]
pub struct CalibrationStatus {
    pub exists_on_disk: bool,
    pub path: String,
    pub version: u32,
    pub sample_counts: Vec<usize>,
}

/// Returns the current guitar DSP pipeline configuration, or the engine
/// defaults if the user has not configured one yet. Used by the debug
/// window to populate its controls.
#[tauri::command]
pub fn get_full_guitar_config(state: State<AppState>) -> Result<GuitarInputConfig, String> {
    let guard = state.guitar_config.lock().map_err(|e| e.to_string())?;
    Ok(guard.clone().unwrap_or_default())
}

/// Replace the entire guitar DSP pipeline configuration.
///
/// Unlike `set_guitar_config` (which is destructive in a different way
/// — it rebuilds from a fixed subset of fields and resets everything
/// else), this command takes the full struct and stores it as-is. Used
/// by the debug window to write back partial edits without losing
/// fields the legacy command doesn't expose.
///
/// **Note:** the routing thread reads `state.guitar_config` once at
/// routing start. Changes made via this command do *not* take effect
/// mid-session — the user must stop and restart routing for the new
/// config to be picked up.
#[tauri::command]
pub fn set_full_guitar_config(
    config: GuitarInputConfig,
    state: State<AppState>,
) -> Result<(), String> {
    *state.guitar_config.lock().map_err(|e| e.to_string())? = Some(config);
    Ok(())
}

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
        hop_size: 256,
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

/// Inner load helper — does the disk read + AppState mutation. Shared
/// between the Tauri command wrapper and the app-startup hook in
/// `main::setup`. Returns the loaded profile (or default when no file
/// exists). Tolerant on a missing file; surfaces real read/parse errors.
///
/// This is the single source of truth for "apply calibration to the
/// engine" — startup, UI reload, and post-save all funnel here so the
/// badge can never disagree with what GuitarBridge sees.
pub fn apply_calibration_profile_from_disk(
    app: &tauri::AppHandle,
    state: &AppState,
) -> Result<GuitarCalibrationProfile, String> {
    let path = calibration_profile_path(app)?;
    let profile = if path.exists() {
        let raw = std::fs::read_to_string(&path)
            .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;
        GuitarCalibrationProfile::from_json(&raw)
            .map_err(|e| format!("Failed to parse calibration profile: {}", e))?
    } else {
        GuitarCalibrationProfile::default()
    };
    let summary: Vec<usize> = profile
        .strings
        .iter()
        .map(|s| s.soft_samples.len() + s.strong_samples.len())
        .collect();
    let total: usize = summary.iter().sum();
    eprintln!(
        "[calibration] applied profile v{}: {} samples total, per-string={:?}, path={}",
        profile.version,
        total,
        summary,
        path.display()
    );
    *state
        .calibration_profile
        .lock()
        .map_err(|e| e.to_string())? = profile.clone();
    Ok(profile)
}

/// Load the per-string calibration profile from `app_data_dir()` into
/// AppState. Returns a `CalibrationStatus` directly so the UI doesn't
/// need a follow-up `get_calibration_status` roundtrip (brutal-critic
/// #13). Idempotent — safe to call at startup AND from the UI's
/// "reload" affordance.
#[tauri::command]
pub fn load_calibration_profile(
    app: tauri::AppHandle,
    state: State<AppState>,
) -> Result<CalibrationStatus, String> {
    let path = calibration_profile_path(&app)?;
    let profile = apply_calibration_profile_from_disk(&app, &state)?;
    Ok(CalibrationStatus {
        exists_on_disk: path.exists(),
        path: path.to_string_lossy().into_owned(),
        version: profile.version,
        sample_counts: profile
            .strings
            .iter()
            .map(|s| s.soft_samples.len() + s.strong_samples.len())
            .collect(),
    })
}

/// Delete the calibration profile from disk and reset AppState to the
/// default profile. Brutal-critic #17 — gives the user a "Reset to
/// Default" affordance for the case where a bad calibration sweep
/// poisons the file and the user wants to start over without
/// rummaging in `~/Library/Application Support`.
#[tauri::command]
pub fn delete_calibration_profile(
    app: tauri::AppHandle,
    state: State<AppState>,
) -> Result<CalibrationStatus, String> {
    let path = calibration_profile_path(&app)?;
    // Tolerate "not found" — already-deleted is success.
    if let Err(e) = std::fs::remove_file(&path) {
        if e.kind() != std::io::ErrorKind::NotFound {
            return Err(format!("Failed to delete {}: {}", path.display(), e));
        }
    }
    let default_profile = GuitarCalibrationProfile::default();
    *state
        .calibration_profile
        .lock()
        .map_err(|e| e.to_string())? = default_profile.clone();
    eprintln!("[calibration] profile deleted, AppState reset to defaults");
    Ok(CalibrationStatus {
        exists_on_disk: false,
        path: path.to_string_lossy().into_owned(),
        version: default_profile.version,
        sample_counts: vec![0; 6],
    })
}

/// Save a JSON-serialized calibration profile to `app_data_dir()` AND
/// update the AppState slot. Replaces the previous file atomically via
/// temp-file + `rename` so a crash mid-write can't leave a half-written
/// JSON that the next `load_calibration_profile` would fail to parse.
#[tauri::command]
pub fn save_calibration_profile(
    profile_json: String,
    app: tauri::AppHandle,
    state: State<AppState>,
) -> Result<CalibrationStatus, String> {
    use std::io::Write;
    // Round-trip parse to validate the payload before persisting.
    let profile = GuitarCalibrationProfile::from_json(&profile_json)
        .map_err(|e| format!("Invalid calibration profile JSON: {}", e))?;
    let path = calibration_profile_path(&app)?;
    let serialized = profile
        .to_json()
        .map_err(|e| format!("Failed to serialize profile: {}", e))?;
    let tmp_path = path.with_extension("json.tmp");
    {
        let mut f = std::fs::File::create(&tmp_path)
            .map_err(|e| format!("Failed to create {}: {}", tmp_path.display(), e))?;
        f.write_all(serialized.as_bytes())
            .map_err(|e| format!("Failed to write {}: {}", tmp_path.display(), e))?;
        f.sync_all()
            .map_err(|e| format!("Failed to sync {}: {}", tmp_path.display(), e))?;
    }
    std::fs::rename(&tmp_path, &path).map_err(|e| {
        // Best-effort cleanup of the temp file on rename failure.
        let _ = std::fs::remove_file(&tmp_path);
        format!(
            "Failed to rename {} -> {}: {}",
            tmp_path.display(),
            path.display(),
            e
        )
    })?;
    *state
        .calibration_profile
        .lock()
        .map_err(|e| e.to_string())? = profile.clone();
    Ok(CalibrationStatus {
        exists_on_disk: true,
        path: path.to_string_lossy().into_owned(),
        version: profile.version,
        sample_counts: profile
            .strings
            .iter()
            .map(|s| s.soft_samples.len() + s.strong_samples.len())
            .collect(),
    })
}

/// Inspect the current calibration profile and report whether a file
/// exists at the canonical path. Used by the UI Input subtab to render
/// a "calibrated / default" badge next to the Calibrate button.
///
/// IMPORTANT: This command re-applies the profile from disk before
/// reporting. Brutal-critic #18 showed that the previous "stat only"
/// implementation let AppState lie when the disk had a newer profile
/// than what was loaded into memory. Now status truth and engine
/// truth can never disagree.
#[tauri::command]
pub fn get_calibration_status(
    app: tauri::AppHandle,
    state: State<AppState>,
) -> Result<CalibrationStatus, String> {
    let path = calibration_profile_path(&app)?;
    // Re-apply on status check so disk + AppState + engine are always
    // in sync. Tolerant: a missing file produces a default profile,
    // not an error. Read/parse errors DO bubble — they indicate a
    // genuinely broken profile the user needs to know about.
    let _profile = apply_calibration_profile_from_disk(&app, &state)?;
    let profile = state
        .calibration_profile
        .lock()
        .map_err(|e| e.to_string())?;
    Ok(CalibrationStatus {
        exists_on_disk: path.exists(),
        path: path.to_string_lossy().into_owned(),
        version: profile.version,
        sample_counts: profile
            .strings
            .iter()
            .map(|s| s.soft_samples.len() + s.strong_samples.len())
            .collect(),
    })
}
