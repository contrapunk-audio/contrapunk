//! Tauri commands for the Companion orchestrator (#91 commit C-minimal).
//!
//! The Companion lives on `AppState` (see state.rs) and is ticked by
//! the router thread every loop iteration. These commands are the UI's
//! handle on the master enable switch — turn the Companion on/off and
//! inspect the current state.
//!
//! Snapshot / restore of `CompanionState` (Lane registry + per-Lane
//! save data) is intentionally NOT in this module yet — that's the
//! follow-up commit. The enable/disable pair is the minimum surface
//! needed for a UI toggle to drive the running Companion.

use std::sync::atomic::Ordering;

use tauri::State;

use crate::state::AppState;

/// Master enable flag for the Companion. When `false` (the default),
/// `Companion::tick()` short-circuits and `Companion::on_input()`
/// returns `{ ops: [], suppress_default: false }` — bit-identical to
/// pre-companion behavior in the router. When `true`, registered
/// Lanes run.
///
/// Stored as an `AtomicBool` so this command takes only a brief lock
/// on the Companion mutex (just to acquire the inner Arc), then
/// mutates atomically without holding the mutex while the router is
/// in the middle of a tick.
#[tauri::command]
pub fn companion_set_enabled(enabled: bool, state: State<AppState>) -> Result<(), String> {
    let companion = state.companion.lock().map_err(|e| e.to_string())?;
    companion.enabled.store(enabled, Ordering::Release);
    Ok(())
}

/// Inspect the Companion master flag. Used by the UI to render the
/// toggle's state after a refresh / on app startup.
#[tauri::command]
pub fn companion_is_enabled(state: State<AppState>) -> Result<bool, String> {
    let companion = state.companion.lock().map_err(|e| e.to_string())?;
    Ok(companion.enabled.load(Ordering::Acquire))
}

// === CanonLane (#3) commands =================================================
// The canon voice is the first concrete Lane shipped on the Companion.
// These commands let the UI configure delay / transpose / enabled
// without needing a separate adapter or downcast. Behind the scenes
// they all funnel through `Companion::configure_lane("canon", ...)`
// which applies a partial JSON blob to the lane's `deserialize_state`.

/// Enable or disable the canon voice. Requires `companion_set_enabled(true)`
/// for the lane to actually fire — this is the per-lane gate inside
/// the Companion. Defaults to disabled.
#[tauri::command]
pub fn canon_set_enabled(enabled: bool, state: State<AppState>) -> Result<(), String> {
    let mut companion = state.companion.lock().map_err(|e| e.to_string())?;
    companion.configure_lane("canon", serde_json::json!({ "enabled": enabled }))
}

/// Set the canon delay in beats (0.0 .. 8.0). Out-of-range values are
/// clamped engine-side. (#3)
#[tauri::command]
pub fn canon_set_delay(beats: f32, state: State<AppState>) -> Result<(), String> {
    let mut companion = state.companion.lock().map_err(|e| e.to_string())?;
    companion.configure_lane("canon", serde_json::json!({ "delay_beats": beats }))
}

/// Set the canon diatonic transpose in scale degrees (-7 .. +7).
/// Reserved for v2 — v1 ignores the value (emits unison). (#3)
#[tauri::command]
pub fn canon_set_transpose(degrees: i8, state: State<AppState>) -> Result<(), String> {
    let mut companion = state.companion.lock().map_err(|e| e.to_string())?;
    companion.configure_lane("canon", serde_json::json!({ "transpose_degrees": degrees }))
}

/// Read the canon lane's current config as a JSON object. Contains:
///   - `enabled` (bool)
///   - `voices` (array of `{delay_beats, transpose_degrees}`)
///   - `delay_beats` / `transpose_degrees` (legacy single-voice keys,
///     mirror of `voices[0]` for backward compat)
/// Returns `null` if the canon lane is somehow not registered. (#3)
#[tauri::command]
pub fn canon_state(state: State<AppState>) -> Result<serde_json::Value, String> {
    let companion = state.companion.lock().map_err(|e| e.to_string())?;
    Ok(companion
        .lane_state("canon")
        .unwrap_or(serde_json::Value::Null))
}

/// One canon voice config from the UI. Mirrors `CanonVoice` in
/// `src-tauri/src/companion/canon_lane.rs`. Tauri's command codegen
/// deserializes this from the JS side automatically.
///
/// `time_ratio` defaults to 1.0 if the JS side hasn't been updated
/// to carry it yet — old UIs keep working as strict-imitation canons.
#[derive(serde::Deserialize)]
pub struct CanonVoiceArg {
    pub delay_beats: f32,
    pub transpose_degrees: i8,
    #[serde(default = "default_time_ratio")]
    pub time_ratio: f32,
    /// Optional per-voice harmony mode override. None = the voice
    /// inherits the engine's global mode for its harmony stack.
    /// Some(name) = the canon lane temporarily swaps to that mode
    /// while harmonizing this voice's subject pitch. Canonical
    /// names match `parse_harmony_mode` in commands/harmony.rs.
    #[serde(default)]
    pub harmony_mode: Option<String>,
    /// Cascade reference. None = the voice harmonizes against the
    /// player's input note. Some(idx) = the voice harmonizes against
    /// canon voice `idx`'s subject pitch instead. The lane validates
    /// `idx < self_index` server-side; out-of-range falls back to
    /// the player.
    #[serde(default)]
    pub reference_voice: Option<usize>,
}

fn default_time_ratio() -> f32 {
    1.0
}

/// Replace the entire canon voices array. Clamped to 8 voices max
/// engine-side. Each voice has its own delay and transpose so
/// callers can configure multi-entry canons (e.g. a 3-voice canon
/// at +0/+2/+4 beats). (#3 multi-voice)
#[tauri::command]
pub fn canon_set_voices(voices: Vec<CanonVoiceArg>, state: State<AppState>) -> Result<(), String> {
    let voices_json: Vec<serde_json::Value> = voices
        .into_iter()
        .map(|v| {
            serde_json::json!({
                "delay_beats": v.delay_beats,
                "transpose_degrees": v.transpose_degrees,
                "time_ratio": v.time_ratio,
                "harmony_mode": v.harmony_mode,
                "reference_voice": v.reference_voice,
            })
        })
        .collect();
    let mut companion = state.companion.lock().map_err(|e| e.to_string())?;
    companion.configure_lane("canon", serde_json::json!({ "voices": voices_json }))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// AtomicBool ordering: a Release store must be visible to a
    /// subsequent Acquire load. This is the contract the router
    /// thread depends on — the AppState default constructs companion
    /// with `enabled = false`, so toggling true via this command and
    /// then reading back must observe `true`.
    ///
    /// Driven without standing up Tauri's State<> wrapper (which
    /// requires a tauri::Manager) — the underlying AtomicBool is
    /// the contract being tested.
    #[test]
    fn test_companion_enabled_atomic_round_trip() {
        use std::sync::atomic::AtomicBool;
        let flag = AtomicBool::new(false);
        assert!(!flag.load(Ordering::Acquire));
        flag.store(true, Ordering::Release);
        assert!(flag.load(Ordering::Acquire));
        flag.store(false, Ordering::Release);
        assert!(!flag.load(Ordering::Acquire));
    }
}
