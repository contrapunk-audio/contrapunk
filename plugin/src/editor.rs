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
        let (input, harmony) = {
            let s = self.note_state.lock().ok()?;
            let mut input: Vec<u8> = s.input_notes.iter().copied().collect();
            let mut harmony: Vec<u8> = s.harmony_notes.iter().copied().collect();
            input.sort_unstable();
            harmony.sort_unstable();
            (input, harmony)
        };
        let key = format!("{:?}", self.params.key.value());
        let empty: Vec<u8> = Vec::new();
        let payload = serde_json::json!({
            "type": "noteUpdate",
            "inputNotes": input,
            "harmonyNotes": harmony,
            "borrowedNotes": empty,
            // Plugin has no Companion lanes yet — these stay empty
            // so the Piano falls back to generic harmony coloring.
            "canonNotes": empty,
            "counterpointNotes": empty,
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
    // The Svelte build output is embedded at compile time.
    // During development, these files may not exist yet — we provide a fallback.
    match path {
        "/" | "/index.html" => {
            #[cfg(feature = "embed-ui")]
            {
                let body =
                    Cow::Borrowed(include_bytes!("../../ui/plugin-build/index.html") as &[u8]);
                return (body, "text/html");
            }
            #[cfg(not(feature = "embed-ui"))]
            {
                let html = include_str!("editor_fallback.html");
                (Cow::Owned(html.as_bytes().to_vec()), "text/html")
            }
        }
        p if p.ends_with(".js") => {
            #[cfg(feature = "embed-ui")]
            {
                let js = load_plugin_build_asset(p);
                return (js, "application/javascript");
            }
            #[cfg(not(feature = "embed-ui"))]
            {
                (
                    Cow::Owned(b"// no UI build".to_vec()),
                    "application/javascript",
                )
            }
        }
        p if p.ends_with(".css") => {
            #[cfg(feature = "embed-ui")]
            {
                let css = load_plugin_build_asset(p);
                return (css, "text/css");
            }
            #[cfg(not(feature = "embed-ui"))]
            {
                (Cow::Owned(b"/* no UI build */".to_vec()), "text/css")
            }
        }
        p if p.ends_with(".wasm") => {
            #[cfg(feature = "embed-ui")]
            {
                let wasm = load_plugin_build_asset(p);
                return (wasm, "application/wasm");
            }
            #[cfg(not(feature = "embed-ui"))]
            {
                (Cow::Owned(vec![]), "application/wasm")
            }
        }
        _ => (Cow::Owned(b"Not Found".to_vec()), "text/plain"),
    }
}

/// Load a file from the plugin build directory at compile time.
/// This uses a build script to generate the asset map.
#[cfg(feature = "embed-ui")]
fn load_plugin_build_asset(path: &str) -> Cow<'static, [u8]> {
    // This will be generated by build.rs to map paths to include_bytes!
    // For now, return empty
    include!(concat!(env!("OUT_DIR"), "/ui_assets.rs"));
    match get_asset(path) {
        Some(data) => Cow::Borrowed(data),
        None => Cow::Owned(b"Not Found".to_vec()),
    }
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
