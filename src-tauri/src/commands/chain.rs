//! Tauri commands for listing and mutating the live audio chain.
//!
//! Block-specific param commands (e.g. reverb mix) live in `fx.rs` and
//! operate directly on the params struct. This module is for the
//! topology of the chain: which blocks exist, in what order, and
//! adding/removing blocks at runtime.

use tauri::State;

use contrapunk::chain::BlockDescriptor;

use crate::state::AppState;

/// Snapshot the current chain. Returns an empty vec if the audio
/// clock failed to start.
#[tauri::command]
pub fn list_chain_blocks(state: State<AppState>) -> Vec<BlockDescriptor> {
    let guard = match state.chain_commander.lock() {
        Ok(g) => g,
        Err(_) => return Vec::new(),
    };
    match guard.as_ref() {
        Some(c) => c.snapshot(),
        None => Vec::new(),
    }
}

/// Remove the block at the given index. Built-in blocks (synth, delay,
/// reverb) can be removed, but expect silence or missing FX until they
/// are re-added. No-op if index is out of range.
#[tauri::command]
pub fn remove_chain_block(index: usize, state: State<AppState>) -> Result<(), String> {
    let guard = state
        .chain_commander
        .lock()
        .map_err(|e| format!("chain commander lock: {e}"))?;
    let commander = guard
        .as_ref()
        .ok_or_else(|| "audio chain not initialized".to_string())?;
    commander.remove_at(index)
}

/// Drop every block from the chain. Useful for rig reload.
#[tauri::command]
pub fn clear_chain(state: State<AppState>) -> Result<(), String> {
    let guard = state
        .chain_commander
        .lock()
        .map_err(|e| format!("chain commander lock: {e}"))?;
    let commander = guard
        .as_ref()
        .ok_or_else(|| "audio chain not initialized".to_string())?;
    commander.clear()
}
