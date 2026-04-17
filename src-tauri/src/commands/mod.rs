//! Tauri command modules.
//!
//! Each sub-module defines `#[tauri::command]` functions registered
//! in the Tauri builder.

pub mod audio_out;
pub mod engine;
pub mod generator;
pub mod guitar;
pub mod harmony;
pub mod midi;
pub mod presets;
