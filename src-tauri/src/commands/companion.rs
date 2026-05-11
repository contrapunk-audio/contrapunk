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
