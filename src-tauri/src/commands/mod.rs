//! Tauri command modules.
//!
//! Each sub-module defines `#[tauri::command]` functions registered
//! in the Tauri builder.

pub mod chain;
pub mod engine;
pub mod fx;
pub mod guitar;
pub mod harmony;
pub mod midi;
pub mod plugins;
pub mod presets;
pub mod routing;
pub mod synth;
pub mod transport;
