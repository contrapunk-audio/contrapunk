//! Harmony engine for MIDI note transformation.
//!
//! This module provides the core harmony generation functionality,
//! including scale-aware transposition and multiple harmony modes.

mod config;
mod engine;
mod modes;
mod scale;
mod stateful;
pub mod voice_leading;

pub use config::{Key, HarmonyMode, OctaveMode};
pub use engine::HarmonyEngine;
pub use scale::Scale;
pub use stateful::{ContraryMotionState, CounterpointState};
