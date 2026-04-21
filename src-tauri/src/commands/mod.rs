//! Tauri command modules.
//!
//! Each sub-module defines `#[tauri::command]` functions registered
//! in the Tauri builder.

pub mod engine;
pub mod guitar;
pub mod harmony;
pub mod midi;
pub mod presets;
pub mod synth;
pub mod transport;
