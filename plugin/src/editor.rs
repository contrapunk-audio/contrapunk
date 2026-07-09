//! WebView-based plugin editor using nih-plug-webview.
//!
//! Embeds the Svelte UI inside the plugin window. The JS side communicates
//! via `window.plugin.send/listen` which maps to a `PluginAdapter` in the
//! Svelte adapter layer.

use nih_plug_webview::{
    Context, EditorHandler, WebViewConfig, WebViewEditor, WebViewSource, WebViewState,
};
use std::borrow::Cow;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use contrapunk_companion::Companion;

use crate::{ContrapunkParams, PluginNoteState};

/// Width and height of the plugin editor window.
const EDITOR_WIDTH: f64 = 900.0;
const EDITOR_HEIGHT: f64 = 700.0;

/// Editor handler that bridges the Svelte UI to nih-plug parameters.
pub struct ContrapunkEditorHandler {
    params: Arc<ContrapunkParams>,
    /// Shared with the audio thread. Audio thread writes the active
    /// input/harmony note sets here; the frame loop snapshots and
    /// pushes a `noteUpdate` message so the Piano / Fretboard light
    /// up while the plugin is generating MIDI.
    note_state: Arc<Mutex<PluginNoteState>>,
    /// Shared with the audio thread. Editor IPC handlers reach into
    /// the Companion via `try_lock` to apply user config changes
    /// (HoldMode, canon voices, etc.) without blocking the per-block
    /// `tick_tagged` call in `process()`.
    companion: Arc<Mutex<Companion>>,
    /// Last `noteUpdate` payload pushed. Used to suppress redundant
    /// sends when nothing changed — keeps the JS bridge quiet.
    last_note_json: String,
}

impl ContrapunkEditorHandler {
    /// Build a JSON string with all current parameter values.
    fn params_json(&self) -> String {
        serde_json::json!({
            "type": "paramsUpdate",
            "key": format!("{:?}", self.params.key.value()),
            "mode": format!("{:?}", self.params.harmony_mode.value()),
            "voiceLeading": self.params.voice_leading.value(),
            "octaveMode": format!("{:?}", self.params.octave_mode.value()),
            "voicePosition": self.params.voice_position.value(),
            "voiceCount": self.params.voice_count.value(),
            "autoKey": self.params.auto_key.value(),
            "inputMode": format!("{:?}", self.params.input_mode.value()),
        })
        .to_string()
    }

    /// Snapshot the shared note state and build a noteUpdate JSON
    /// payload matching the Tauri / WASM shape. Returns `None` if
    /// the snapshot matches the last one sent (no change → no IPC).
    fn note_update_json(&mut self) -> Option<String> {
        let (input, harmony, canon, counterpoint) = {
            let s = self.note_state.lock().ok()?;
            let mut input: Vec<u8> = s.input_notes.iter().copied().collect();
            let mut harmony: Vec<u8> = s.harmony_notes.iter().copied().collect();
            let mut canon: Vec<u8> = s.canon_notes.iter().copied().collect();
            let mut counterpoint: Vec<u8> = s.counterpoint_notes.iter().copied().collect();
            input.sort_unstable();
            harmony.sort_unstable();
            canon.sort_unstable();
            counterpoint.sort_unstable();
            (input, harmony, canon, counterpoint)
        };
        let key = format!("{:?}", self.params.key.value());
        let empty: Vec<u8> = Vec::new();
        let payload = serde_json::json!({
            "type": "noteUpdate",
            "inputNotes": input,
            "harmonyNotes": harmony,
            "borrowedNotes": empty,
            "canonNotes": canon,
            "counterpointNotes": counterpoint,
            "chordName": "",
            "lastBorrowedFrom": "",
            "currentKey": key,
        })
        .to_string();
        if payload == self.last_note_json {
            return None;
        }
        self.last_note_json = payload.clone();
        Some(payload)
    }
}

impl EditorHandler for ContrapunkEditorHandler {
    fn on_frame(&mut self, cx: &mut Context) {
        if let Some(json) = self.note_update_json() {
            cx.send_message(json);
        }
    }

    fn on_message(&mut self, cx: &mut Context, message: String) {
        let msg: serde_json::Value = match serde_json::from_str(&message) {
            Ok(v) => v,
            Err(_) => return,
        };

        let msg_type = msg.get("type").and_then(|t| t.as_str()).unwrap_or("");

        match msg_type {
            "ready" => {
                // UI loaded — send initial parameter state
                cx.send_message(self.params_json());
            }
            "setKey" => {
                if let Some(key_str) = msg.get("value").and_then(|v| v.as_str()) {
                    if let Some(key_variant) = parse_key(key_str) {
                        let setter = cx.get_param_setter();
                        setter.set_parameter(&self.params.key, key_variant);
                    }
                }
            }
            "setMode" => {
                if let Some(mode_str) = msg.get("value").and_then(|v| v.as_str()) {
                    if let Some(mode_variant) = parse_mode(mode_str) {
                        let setter = cx.get_param_setter();
                        setter.set_parameter(&self.params.harmony_mode, mode_variant);
                    }
                }
            }
            "setVoiceCount" => {
                if let Some(count) = msg.get("value").and_then(|v| v.as_i64()) {
                    let setter = cx.get_param_setter();
                    setter.set_parameter(&self.params.voice_count, count as i32);
                }
            }
            "setVoicePosition" => {
                if let Some(pos) = msg.get("value").and_then(|v| v.as_i64()) {
                    let setter = cx.get_param_setter();
                    setter.set_parameter(&self.params.voice_position, pos as i32);
                }
            }
            "setOctaveMode" => {
                if let Some(oct_str) = msg.get("value").and_then(|v| v.as_str()) {
                    if let Some(oct_variant) = parse_octave_mode(oct_str) {
                        let setter = cx.get_param_setter();
                        setter.set_parameter(&self.params.octave_mode, oct_variant);
                    }
                }
            }
            "setAutoKey" => {
                if let Some(enabled) = msg.get("value").and_then(|v| v.as_bool()) {
                    let setter = cx.get_param_setter();
                    setter.set_parameter(&self.params.auto_key, enabled);
                }
            }
            "setVoiceLeading" => {
                if let Some(enabled) = msg.get("value").and_then(|v| v.as_bool()) {
                    let setter = cx.get_param_setter();
                    setter.set_parameter(&self.params.voice_leading, enabled);
                }
            }
            "setInputMode" => {
                if let Some(mode_str) = msg.get("value").and_then(|v| v.as_str()) {
                    if let Some(input_variant) = parse_input_mode(mode_str) {
                        let setter = cx.get_param_setter();
                        setter.set_parameter(&self.params.input_mode, input_variant);
                    }
                }
            }
            // ─── Companion IPC ────────────────────────────────────
            // All companion handlers use `try_lock` so a busy audio
            // thread doesn't block the editor message dispatcher.
            // UI commands are user-rate (≤10/sec); dropped messages
            // are recoverable (user retries or on_params_changed
            // resyncs).
            "companionSetEnabled" => {
                if let Some(enabled) = msg.get("value").and_then(|v| v.as_bool()) {
                    if let Ok(c) = self.companion.try_lock() {
                        c.enabled
                            .store(enabled, std::sync::atomic::Ordering::Release);
                    }
                }
            }
            "companionSetGlobalHoldMode" => {
                if let Some(hm_json) = msg.get("value") {
                    if let Some(mode) = contrapunk_companion::lane::hold_mode_from_json(hm_json) {
                        if let Ok(c) = self.companion.try_lock() {
                            c.set_global_hold_mode(mode);
                        }
                    }
                }
            }
            "canonConfigure" => {
                if let Some(partial) = msg.get("value") {
                    if let Ok(mut c) = self.companion.try_lock() {
                        let _ = c.configure_lane("canon", partial.clone());
                    }
                }
            }
            "counterpointConfigure" => {
                if let Some(partial) = msg.get("value") {
                    if let Ok(mut c) = self.companion.try_lock() {
                        let _ = c.configure_lane("counterpoint", partial.clone());
                    }
                }
            }
            "canonSetVoices" => {
                // Voices arrive as the same JSON shape Tauri's
                // canon_set_voices command builds. Delegate to the
                // canon lane's configure_lane wrapped with { voices }.
                if let Some(voices) = msg.get("value") {
                    if let Ok(mut c) = self.companion.try_lock() {
                        let payload = serde_json::json!({ "voices": voices });
                        let _ = c.configure_lane("canon", payload);
                    }
                }
            }
            _ => {}
        }
    }

    fn on_params_changed(&mut self, cx: &mut Context) {
        // DAW changed params (e.g. automation, preset load) — sync to UI
        cx.send_message(self.params_json());
    }
}

/// Create the WebViewEditor for the plugin.
pub fn create_editor(
    params: Arc<ContrapunkParams>,
    note_state: Arc<Mutex<PluginNoteState>>,
    companion: Arc<Mutex<Companion>>,
    state: &Arc<WebViewState>,
) -> WebViewEditor {
    let protocol = "contrapunk".to_string();
    let config = WebViewConfig {
        title: "Contrapunk".to_string(),
        source: WebViewSource::CustomProtocol {
            protocol: protocol.clone(),
            url: "index.html".to_string(),
        },
        workdir: PathBuf::from("/tmp/contrapunk-webview"),
    };

    let handler = ContrapunkEditorHandler {
        params,
        note_state,
        companion,
        last_note_json: String::new(),
    };

    WebViewEditor::new_with_webview(handler, state, config, move |w| {
        let proto = protocol.clone();
        w.with_custom_protocol(proto, |_id, req| {
            let path = req.uri().path();
            let (body, mime) = serve_embedded_asset(path);
            nih_plug_webview::wry::http::Response::builder()
                .header("Content-Type", mime)
                .body(body)
                .unwrap()
        })
    })
}

/// Serve embedded UI assets from the compiled-in Svelte build.
/// Falls back to a minimal placeholder if the build isn't available.
fn serve_embedded_asset(path: &str) -> (Cow<'static, [u8]>, &'static str) {
    #[cfg(feature = "embed-ui")]
    {
        let path = if path == "/" { "/index.html" } else { path };
        return match get_plugin_build_asset(path) {
            Some(body) => (Cow::Borrowed(body), mime_for_path(path)),
            None => (Cow::Owned(b"Not Found".to_vec()), "text/plain"),
        };
    }

    #[cfg(not(feature = "embed-ui"))]
    {
        if path == "/" || path == "/index.html" {
            let html = include_str!("editor_fallback.html");
            return (Cow::Owned(html.as_bytes().to_vec()), "text/html");
        }
        (Cow::Owned(b"Not Found".to_vec()), "text/plain")
    }
}

fn mime_for_path(path: &str) -> &'static str {
    match path.rsplit('.').next().unwrap_or("") {
        "html" => "text/html",
        "js" => "application/javascript",
        "css" => "text/css",
        "wasm" => "application/wasm",
        "json" => "application/json",
        "svg" => "image/svg+xml",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "webp" => "image/webp",
        "ico" => "image/x-icon",
        "woff2" => "font/woff2",
        _ => "application/octet-stream",
    }
}

#[cfg(feature = "embed-ui")]
mod embedded_ui_assets {
    include!(concat!(env!("OUT_DIR"), "/ui_assets.rs"));
}

/// Load a file from the plugin build directory at compile time.
/// This uses a build script to generate the asset map.
#[cfg(feature = "embed-ui")]
fn get_plugin_build_asset(path: &str) -> Option<&'static [u8]> {
    embedded_ui_assets::get_asset(path)
}

// ── Parameter parsing helpers ───────────────────────────────────────

use crate::{PluginInputMode, PluginKey, PluginMode, PluginOctaveMode};

fn parse_key(s: &str) -> Option<PluginKey> {
    match s {
        "C" => Some(PluginKey::C),
        "Db" | "C#" | "C#/Db" => Some(PluginKey::Db),
        "D" => Some(PluginKey::D),
        "Eb" | "D#" | "D#/Eb" => Some(PluginKey::Eb),
        "E" => Some(PluginKey::E),
        "F" => Some(PluginKey::F),
        "Gb" | "F#" | "F#/Gb" => Some(PluginKey::Gb),
        "G" => Some(PluginKey::G),
        "Ab" | "G#" | "G#/Ab" => Some(PluginKey::Ab),
        "A" => Some(PluginKey::A),
        "Bb" | "A#" | "A#/Bb" => Some(PluginKey::Bb),
        "B" => Some(PluginKey::B),
        _ => None,
    }
}

fn parse_mode(s: &str) -> Option<PluginMode> {
    match s {
        "PassThrough" => Some(PluginMode::PassThrough),
        "DiatonicThirds" => Some(PluginMode::DiatonicThirds),
        "DiatonicFourths" => Some(PluginMode::DiatonicFourths),
        "RandomBelow" => Some(PluginMode::RandomBelow),
        "RandomBelowNoSeconds" => Some(PluginMode::RandomBelowNoSeconds),
        "ContraryMotion" => Some(PluginMode::ContraryMotion),
        "StrictCounterpoint" => Some(PluginMode::StrictCounterpoint),
        "BarryHarris" => Some(PluginMode::BarryHarris),
        "FunctionalHarmony" => Some(PluginMode::FunctionalHarmony),
        "BachChorale" => Some(PluginMode::BachChorale),
        _ => None,
    }
}

fn parse_octave_mode(s: &str) -> Option<PluginOctaveMode> {
    match s {
        "None" => Some(PluginOctaveMode::None),
        "Spread" => Some(PluginOctaveMode::Spread),
        "BassTrebleSplit" => Some(PluginOctaveMode::BassTrebleSplit),
        "Mirror" => Some(PluginOctaveMode::Mirror),
        _ => None,
    }
}

fn parse_input_mode(s: &str) -> Option<PluginInputMode> {
    match s {
        "Midi" | "MIDI" => Some(PluginInputMode::Midi),
        "Audio" | "Audio (Guitar)" => Some(PluginInputMode::Audio),
        _ => None,
    }
}

/// Create a new WebViewState with default dimensions.
pub fn default_webview_state() -> Arc<WebViewState> {
    Arc::new(WebViewState::new(EDITOR_WIDTH, EDITOR_HEIGHT))
}
