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
//! # WASM Entry Point
//!
//! This crate's library target is primarily for WebAssembly. The native application
//! uses a binary target (`main.rs`). For WASM builds, this module provides the
//! entry point via `wasm_bindgen`.

#![cfg(target_arch = "wasm32")]

mod app;
mod chord;
mod midi_defaults;
mod generator;
mod harmony;
mod humanize;
mod midi;
mod piano;
mod preset;
mod ui;
mod theme;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

/// Auto-start entry point called by Trunk's generated JS bootstrap.
///
/// Initializes the panic hook for better error messages and spawns the
/// eframe application in the browser canvas.
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
