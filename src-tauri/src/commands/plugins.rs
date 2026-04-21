//! Tauri commands for CLAP plugin hosting.
//!
//! * [`list_clap_plugins`] scans the filesystem and returns every
//!   `.clap` bundle found. Invoked by the "Add plugin" picker in the
//!   chain panel.
//! * [`add_clap_plugin_to_chain`] constructs a [`ClapBlock`] and
//!   pushes it onto the live audio chain via the main-thread
//!   [`ChainCommander`]. The command queue delivers it to the audio
//!   thread, which appends it to the chain on the next buffer.
//!
//! See `src/plugin_host/clap/` for the scaffolding these commands
//! delegate to. `ClapBlock` is still a stub that emits silence — the
//! plumbing for actual audio flow through the plugin is the next
//! iteration inside that module. Wiring it here now means the UI
//! side is ready when block.rs is upgraded.

use tauri::State;

use contrapunk::plugin_host::clap::{discover_plugins, ClapBlock, PluginDescriptor};

use crate::state::AppState;

/// Discover every CLAP plugin installed on the machine.
///
/// Walks the OS-standard CLAP directories + `$CLAP_PATH`. Returns the
/// list sorted by name so the UI can render it directly.
#[tauri::command]
pub fn list_clap_plugins() -> Vec<PluginDescriptor> {
    let mut out = discover_plugins();
    out.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    out
}

/// Push a CLAP plugin onto the live audio chain.
///
/// Returns `Ok(())` once the command has been enqueued. The audio
/// thread consumes it on the next buffer.
#[tauri::command]
pub fn add_clap_plugin_to_chain(path: String, state: State<AppState>) -> Result<(), String> {
    let block = ClapBlock::new(&path).map_err(|e| e.to_string())?;

    let guard = state
        .chain_commander
        .lock()
        .map_err(|e| format!("chain commander lock: {e}"))?;
    let commander = guard
        .as_ref()
        .ok_or_else(|| "audio chain not initialized".to_string())?;
    commander.push_block(Box::new(block))?;
    Ok(())
}
