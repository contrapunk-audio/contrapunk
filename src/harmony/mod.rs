//! Harmony engine for MIDI note transformation.
//!
//! This module provides the core harmony generation functionality,
//! including scale-aware transposition and multiple harmony modes.

mod config;
mod scale;

// Re-exports will be used by harmony engine in subsequent plans
#[allow(unused_imports)]
pub use config::{Key, HarmonyMode};
#[allow(unused_imports)]
pub use scale::Scale;
