#![cfg(target_arch = "wasm32")]
//! WASM entry point for Contrapunk.
//!
//! This module provides the WebAssembly entry point using eframe's WebRunner.

mod app;
mod chord;
mod harmony;
mod humanize;
mod midi;
mod piano;
mod preset;
mod tabs;
mod theme;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

/// Auto-start entry point called by Trunk's generated JS bootstrap.
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
