//! Contrapunk - Real-time MIDI harmony generation.
//!
//! Contrapunk is a real-time MIDI harmony generator that transforms incoming
//! MIDI notes into multi-voice harmonies. It supports multiple harmony algorithms,
//! 28 scale modes, and advanced voice leading options.
//!
//! # Architecture
//!
//! The core harmony engine is available on all platforms. Platform-specific
//! frontends consume it as a library:
//!
//! - **Tauri desktop**: Svelte frontend with Rust backend via Tauri IPC (`src-tauri/`)
//! - **Browser WASM**: SvelteKit app with contrapunk-wasm bridge (`wasm/`, `ui/`)
//! - **Server/Client**: Network-based MIDI harmony processing (`--server`, `--client`)
//!
//! # Harmony Module
//!
//! The `harmony` module contains the core harmony generation system:
//!
//! - `HarmonyEngine` - Main engine for note transformation
//! - `Key` - Musical keys (C through B)
//! - `HarmonyMode` - 8 harmony algorithms
//! - `ScaleMode` - 28 scale modes across 5 families
//! - `Scale` - Scale operations and diatonic transposition
//!
//! # Features
//!
//! - **8 Harmony Modes**: Pass-through, thirds, fourths, random, contrary motion,
//!   counterpoint, Barry Harris
//! - **28 Scale Modes**: Diatonic modes, harmonic/melodic minor, exotic scales,
//!   Barry Harris 8-note scales
//! - **Modal Interchange**: Borrow notes from parallel modes for chromatic color
//! - **Voice Leading**: Optional post-processing for smooth voice transitions
//! - **Octave Modes**: Spread, split, mirror transformations
//!
//! # Example (conceptual)
//!
//! ```ignore
//! use contrapunk::harmony::{HarmonyEngine, Key, HarmonyMode};
//!
//! let mut engine = HarmonyEngine::new(Key::C, HarmonyMode::DiatonicThirds);
//! // engine.harmonize(Note::C4) would return [C4, E4]
//! ```

// =============================================================================
// Core modules -- always available on all platforms
// =============================================================================

pub mod audio;
pub mod chord;
pub mod generator;
pub mod harmony;
pub mod humanize;
pub mod midi;
pub mod preset;

// =============================================================================
// Native-only modules (require midir, networking, etc.)
// =============================================================================

/// MIDI routing module (native-only, requires midir).
#[cfg(not(target_arch = "wasm32"))]
pub mod router;

/// Server module for network-based MIDI harmony processing (native-only).
#[cfg(not(target_arch = "wasm32"))]
pub mod server;
