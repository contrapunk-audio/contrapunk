//! Harmony engine for MIDI note transformation.
//!
//! This module provides the core harmony generation functionality,
//! including scale-aware transposition and multiple harmony modes.

mod config;
mod modes;
mod scale;

pub use config::{Key, HarmonyMode};
pub use scale::Scale;
