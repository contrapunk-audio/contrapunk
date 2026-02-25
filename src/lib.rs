//! Contrapunk - Real-time MIDI harmony generation.
//!
//! Contrapunk is a real-time MIDI harmony generator that transforms incoming
//! MIDI notes into multi-voice harmonies. It supports multiple harmony algorithms,
//! 28 scale modes, and advanced voice leading options.
//!
//! # Architecture
//!
//! The core harmony engine is available on all platforms. The GUI and MIDI I/O
//! are platform-specific:
//!
//! - **Native (macOS/Windows/Linux)**: Full GUI with hardware MIDI via `midir`
//! - **WebAssembly**: Browser-based GUI with Web MIDI API
//! - **Tauri desktop**: Svelte frontend with Rust backend via Tauri IPC
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
//! - **28 Scale Modes**: Church modes, harmonic/melodic minor, exotic scales,
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
//!
//! # Library Usage
//!
//! This crate is both a library (used by `src-tauri` and WASM builds) and a
//! binary (the legacy egui native app). Core modules are always available.
//! GUI-specific modules are gated behind `#[cfg(target_arch = "wasm32")]` or
//! `#[cfg(feature = "gui")]`.

// =============================================================================
// Core modules — always available on all platforms
// =============================================================================

pub mod harmony;
pub mod humanize;
pub mod generator;
pub mod chord;
pub mod midi;
pub mod preset;

// =============================================================================
// GUI-dependent modules (require eframe/egui feature)
// =============================================================================

/// Piano keyboard widget (requires gui feature for egui types).
#[cfg(feature = "gui")]
pub mod piano;

/// MIDI device default persistence (WASM-only, depends on app module and eframe::Storage).
#[cfg(all(feature = "gui", target_arch = "wasm32"))]
pub mod midi_defaults;

// =============================================================================
// Native-only modules
// =============================================================================

/// MIDI routing module (native-only, requires midir).
#[cfg(not(target_arch = "wasm32"))]
pub mod router;

// =============================================================================
// WASM-specific modules (egui browser app)
// =============================================================================

#[cfg(target_arch = "wasm32")]
mod app;
#[cfg(target_arch = "wasm32")]
mod ui;
#[cfg(target_arch = "wasm32")]
mod theme;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;

/// Auto-start entry point called by Trunk's generated JS bootstrap.
///
/// Initializes the panic hook for better error messages and spawns the
/// eframe application in the browser canvas.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();

    wasm_bindgen_futures::spawn_local(async {
        let document = web_sys::window()
            .expect("no window")
            .document()
            .expect("no document");
        let canvas = document
            .get_element_by_id("contrapunk_canvas")
            .expect("canvas not found")
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .expect("element is not a canvas");

        let web_options = eframe::WebOptions::default();
        eframe::WebRunner::new()
            .start(
                canvas,
                web_options,
                Box::new(|cc| Ok(Box::new(app::ContrapunkApp::new(cc)))),
            )
            .await
            .expect("failed to start eframe");
    });
}
