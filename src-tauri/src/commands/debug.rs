//! Tauri commands for the debug-pipeline window.
//!
//! Opens a second webview window that lives alongside the main app.
//! Both windows share the same backend process and the same managed
//! state, so config writes from the debug window are visible to the
//! main app immediately. Window label is `debug-pipeline`; the window
//! must also be listed in `capabilities/default.json` for command
//! invocations from inside it to be accepted.

use tauri::{AppHandle, Manager, Runtime, WebviewUrl, WebviewWindowBuilder};

const WINDOW_LABEL: &str = "debug-pipeline";

/// Open (or focus, if already open) the guitar-pipeline debug window.
#[tauri::command]
pub fn open_debug_pipeline_window<R: Runtime>(app: AppHandle<R>) -> Result<(), String> {
    if let Some(win) = app.get_webview_window(WINDOW_LABEL) {
        let _ = win.set_focus();
        return Ok(());
    }

    WebviewWindowBuilder::new(
        &app,
        WINDOW_LABEL,
        WebviewUrl::App("/debug/pipeline".into()),
    )
    .title("Contrapunk — Guitar Pipeline Debug")
    .inner_size(900.0, 700.0)
    .min_inner_size(600.0, 400.0)
    .build()
    .map_err(|e| format!("failed to open debug window: {e}"))?;

    Ok(())
}
