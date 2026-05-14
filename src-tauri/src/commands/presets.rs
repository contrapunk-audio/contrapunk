//! Tauri commands for preset management.
//!
//! List, load, save, and delete presets.

use std::sync::atomic::Ordering;

use serde::Serialize;
use tauri::State;

use contrapunk::preset::StylePreset;

use crate::state::AppState;

/// Serializable preset info for the frontend.
#[derive(Serialize)]
pub struct PresetInfo {
    pub index: usize,
    pub name: String,
    pub persona: String,
    pub genre: String,
    pub is_builtin: bool,
}

/// Lists all available presets (builtins + custom).
#[tauri::command]
pub fn list_presets(state: State<AppState>) -> Result<Vec<PresetInfo>, String> {
    let manager = state.preset_manager.lock().map_err(|e| e.to_string())?;
    let presets = manager.all_presets();
    Ok(presets
        .iter()
        .enumerate()
        .map(|(i, p)| PresetInfo {
            index: i,
            name: p.name.clone(),
            persona: p.persona.clone(),
            genre: p.genre.clone(),
            is_builtin: p.is_builtin,
        })
        .collect())
}

/// Loads a preset by name and applies it to the engine.
#[tauri::command]
pub fn load_preset(name: String, state: State<AppState>) -> Result<(), String> {
    apply_preset_inner(&state, &name)
}

/// Inner implementation of `load_preset` that operates on a plain
/// `&AppState` reference. Extracted from the Tauri command wrapper so
/// the regression test can exercise the full reharm path without
/// standing up a `tauri::State`.
pub(crate) fn apply_preset_inner(state: &AppState, name: &str) -> Result<(), String> {
    let mut manager = state.preset_manager.lock().map_err(|e| e.to_string())?;

    // Find preset by name — two-phase to avoid borrow conflict
    let (idx, preset) = {
        let all = manager.all_presets();
        let found = all.iter().enumerate().find(|(_, p)| p.name == name);
        match found {
            Some((idx, p)) => (idx, (*p).clone()),
            None => return Err(format!("Preset not found: {}", name)),
        }
    };
    manager.set_active(idx);

    // Apply preset to engine
    {
        let mut engine = state.engine.lock().map_err(|e| e.to_string())?;
        engine.set_key(preset.key);
        engine.set_mode(preset.harmony_mode);
        engine.set_octave_mode(preset.octave_mode);
        engine.set_voice_leading_enabled(preset.voice_leading_enabled);
        engine.set_voice_leading_style(preset.voice_leading_style);
        engine.set_scale_mode(preset.scale_mode);
        engine.set_interchange_enabled(preset.interchange_enabled);
        engine.set_borrowing_range(preset.borrowing_range);
    }

    // Mirror `raise_panic` from commands/harmony.rs — each of the 8
    // setters above stashed held inputs into `pending_reharm_inputs`
    // via `clear_active_for_reharm`. Without raising the panic flag
    // the router's reharm-diff replay never fires, so external synths
    // hold the prior preset's harmony forever (stuck-MIDI-notes on
    // preset switch). 5-line miss that broke a major user flow.
    state.panic_pending.store(true, Ordering::SeqCst);

    Ok(())
}

/// Saves the current engine config as a custom preset.
#[tauri::command]
pub fn save_preset(name: String, state: State<AppState>) -> Result<(), String> {
    let engine = state.engine.lock().map_err(|e| e.to_string())?;

    let preset = StylePreset {
        name: name.clone(),
        persona: "Custom".to_string(),
        genre: "Custom".to_string(),
        harmony_mode: engine.mode(),
        key: engine.key(),
        voice_leading_enabled: engine.voice_leading_enabled(),
        voice_leading_style: engine.voice_leading_style(),
        octave_mode: engine.octave_mode(),
        scale_mode: engine.scale_mode(),
        interchange_enabled: engine.interchange_enabled(),
        borrowing_range: engine.borrowing_range(),
        is_builtin: false,
    };

    let mut manager = state.preset_manager.lock().map_err(|e| e.to_string())?;
    manager.add_custom(preset);
    Ok(())
}

/// Deletes a custom preset by name.
#[tauri::command]
pub fn delete_preset(name: String, state: State<AppState>) -> Result<(), String> {
    let mut manager = state.preset_manager.lock().map_err(|e| e.to_string())?;

    // Find index in custom presets
    let custom = manager.custom_presets();
    let idx = custom
        .iter()
        .position(|p| p.name == name)
        .ok_or_else(|| format!("Custom preset not found: {}", name))?;

    manager.remove_custom(idx);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::Ordering;

    /// Regression for commit `b065eb5`: `load_preset` must raise
    /// `panic_pending` so the router-loop drain sends NoteOffs for
    /// stale harmonies after the engine setters mutate. Without this,
    /// external synths hold the prior preset's harmony forever
    /// (stuck-MIDI-notes on preset switch). The fix is one line; the
    /// hole was 5 months old.
    ///
    /// Test mirrors the structural contract: after `apply_preset_inner`
    /// returns, `state.panic_pending` MUST be `true`. The router-loop
    /// drain itself (engine.rs:463-558) is tested separately via the
    /// `drain_all_tracked_notes` cases.
    #[test]
    fn load_preset_raises_panic_pending() {
        let state = AppState::default();
        state.panic_pending.store(false, Ordering::SeqCst);

        // Find any builtin preset to load — the specific contents
        // don't matter, only the side-effect.
        let preset_name = {
            let manager = state.preset_manager.lock().unwrap();
            let all = manager.all_presets();
            assert!(!all.is_empty(), "default AppState should ship builtins");
            all[0].name.clone()
        };

        apply_preset_inner(&state, &preset_name).expect("load_preset failed");
        assert!(
            state.panic_pending.load(Ordering::SeqCst),
            "load_preset must raise panic_pending — without it, stale \
             harmonies stay stuck on external synths after preset switch"
        );
    }

    /// Negative regression: if someone deletes the
    /// `state.panic_pending.store(true, ...)` line in `apply_preset_inner`,
    /// the test above fires. This second case nails down the
    /// no-op-on-error contract: loading an unknown preset must NOT
    /// raise panic (no engine mutation happened, no NoteOff needed).
    #[test]
    fn load_preset_unknown_name_does_not_raise_panic() {
        let state = AppState::default();
        state.panic_pending.store(false, Ordering::SeqCst);

        let result = apply_preset_inner(&state, "nonexistent-preset-zzz");
        assert!(result.is_err(), "unknown preset must return Err");
        assert!(
            !state.panic_pending.load(Ordering::SeqCst),
            "failed load must not raise panic — nothing changed"
        );
    }
}
