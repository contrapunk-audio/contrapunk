#![cfg(target_arch = "wasm32")]
//! WASM entry point for Contrapunk.
//!
//! This module provides the WebAssembly entry point using eframe's WebRunner.

mod app;
mod chord;
mod harmony;
mod midi;
mod piano;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

/// WebHandle wraps eframe's WebRunner for WASM deployment.
#[wasm_bindgen]
pub struct WebHandle {
    runner: eframe::WebRunner,
}

#[wasm_bindgen]
impl WebHandle {
    /// Create a new WebHandle instance.
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        console_error_panic_hook::set_once();
        Self {
            runner: eframe::WebRunner::new(),
        }
    }

    /// Start the Contrapunk application on the given canvas element.
    #[wasm_bindgen]
    pub async fn start(&self, canvas_id: &str) -> Result<(), wasm_bindgen::JsValue> {
        let document = web_sys::window()
            .expect("no window")
            .document()
            .expect("no document");
        let canvas = document
            .get_element_by_id(canvas_id)
            .expect("canvas not found")
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .expect("element is not a canvas");

        let web_options = eframe::WebOptions::default();
        self.runner
            .start(
                canvas,
                web_options,
                Box::new(|cc| Ok(Box::new(app::ContrapunkApp::new(cc)))),
            )
            .await
    }
}
