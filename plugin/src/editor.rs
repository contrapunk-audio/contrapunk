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
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};

use contrapunk::slide::SlideConfig;
use contrapunk_companion::Companion;

use crate::{
    ContrapunkParams, PluginGuitarSignal, PluginHarmonicLimit, PluginNoteState, PluginTuningStyle,
};

/// Width and height of the plugin editor window.
const EDITOR_WIDTH: f64 = 1200.0;
const EDITOR_HEIGHT: f64 = 800.0;

/// Editor handler that bridges the Svelte UI to nih-plug parameters.
pub struct ContrapunkEditorHandler {
    params: Arc<ContrapunkParams>,
    /// Shared with the audio thread. Audio thread writes the active
    /// input/harmony note sets here; the frame loop snapshots and
    /// pushes a `noteUpdate` message so the Piano / Fretboard light
    /// up while the plugin is generating MIDI.
    note_state: Arc<Mutex<PluginNoteState>>,
    /// Shared with the audio thread. Editor IPC handlers take the
    /// Companion lock to guarantee user-rate config changes are applied;
    /// the audio thread uses `try_lock` and may skip one processing tick.
    companion: Arc<Mutex<Companion>>,
    /// Last `noteUpdate` payload pushed. Used to suppress redundant
    /// sends when nothing changed — keeps the JS bridge quiet.
    last_note_json: String,
    /// Set by UI panic button; drained by plugin process() on audio thread.
    panic_requested: Arc<AtomicBool>,
    /// Momentary, non-persisted Compare state shared with the audio thread.
    compare_standard: Arc<AtomicBool>,
    slide_config: Arc<Mutex<SlideConfig>>,
    /// The dedicated Logic Audio FX always consumes its audio bus.
    guitar_component: bool,
    guitar_signal: Arc<Mutex<PluginGuitarSignal>>,
    guitar_was_live: bool,
}

impl ContrapunkEditorHandler {
    /// Build a JSON string with all current parameter values.
    fn params_json(&self) -> String {
        serde_json::json!({
            "type": "paramsUpdate",
            "key": format!("{:?}", self.params.key.value()),
            "mode": format!("{:?}", self.params.harmony_mode.value()),
            "voiceLeading": self.params.voice_leading.value(),
            "voiceLeadingStyle": format!("{:?}", self.params.voice_leading_style.value()),
            "tuningStyle": format!("{:?}", self.params.tuning_style.value()),
            "tuningDepth": self.params.tuning_depth.value(),
            "harmonicLimit": format!("{:?}", self.params.harmonic_limit.value()),
            "tuningCompare": self.compare_standard.load(Ordering::Acquire),
            "slideConfig": *self.slide_config.lock().unwrap_or_else(|error| error.into_inner()),
            "octaveMode": format!("{:?}", self.params.octave_mode.value()),
            "octaveIntensity": self.params.octave_intensity.value(),
            "voicePosition": self.params.voice_position.value(),
            "voiceCount": self.params.voice_count.value(),
            "autoKey": self.params.auto_key.value(),
            "inputMode": format!(
                "{:?}",
                crate::effective_input_mode(
                    self.params.input_mode.value(),
                    self.guitar_component,
                )
            ),
            "synthEnabled": self.params.synth_enabled.value(),
            "synthGain": self.params.synth_gain.value(),
            "mixGains": [
                self.params.synth_input_gain.value(),
                self.params.synth_harmony_gain.value(),
                self.params.synth_canon_gain.value(),
                self.params.synth_counterpoint_gain.value(),
            ],
            "midiOutputMode": format!("{:?}", self.params.midi_output_mode.value()),
        })
        .to_string()
    }

    fn guitar_signal_json(&mut self) -> Option<String> {
        if !self.guitar_component {
            return None;
        }
        let signal = *self
            .guitar_signal
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let live = signal.frequency.is_some();
        if !live && !self.guitar_was_live {
            return None;
        }
        self.guitar_was_live = live;
        Some(
            serde_json::json!({
                "type": "guitarSignal",
                "rms": signal.rms,
                "frequency": signal.frequency,
                "clarity": signal.clarity,
                "note_state": signal.note_state,
                "note_name": "",
                "midi_note": signal.midi_note,
            })
            .to_string(),
        )
    }

    /// Snapshot the shared note state and build a noteUpdate JSON
    /// payload matching the Tauri / WASM shape. Returns `None` if
    /// the snapshot matches the last one sent (no change → no IPC).
    fn note_update_json(&mut self) -> Option<String> {
        let (input, harmony, canon, counterpoint) = {
            let s = self.note_state.lock().ok()?;
            let input: Vec<u8> = s.input_notes.active_notes().collect();
            let harmony: Vec<u8> = s.harmony_notes.active_notes().collect();
            let canon: Vec<u8> = s.canon_notes.active_notes().collect();
            let counterpoint: Vec<u8> = s.counterpoint_notes.active_notes().collect();
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

impl Drop for ContrapunkEditorHandler {
    fn drop(&mut self) {
        self.compare_standard.store(false, Ordering::Release);
    }
}

impl EditorHandler for ContrapunkEditorHandler {
    fn on_frame(&mut self, cx: &mut Context) {
        if let Some(json) = self.note_update_json() {
            cx.send_message(json);
        }
        if let Some(json) = self.guitar_signal_json() {
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
            "setOctaveIntensity" => {
                if let Some(amount) = msg.get("value").and_then(|v| v.as_f64()) {
                    let setter = cx.get_param_setter();
                    setter.set_parameter(&self.params.octave_intensity, amount as f32);
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
            "setVoiceLeadingStyle" => {
                if let Some(style) = msg.get("value").and_then(|v| v.as_str()) {
                    if let Some(style) = parse_voice_leading_style(style) {
                        let setter = cx.get_param_setter();
                        setter.set_parameter(&self.params.voice_leading_style, style);
                    }
                }
            }
            "setTuningStyle" => {
                if let Some(style) = msg.get("value").and_then(|value| value.as_str()) {
                    if let Some(style) = parse_tuning_style(style) {
                        cx.get_param_setter()
                            .set_parameter(&self.params.tuning_style, style);
                    }
                }
            }
            "setTuningDepth" => {
                if let Some(depth) = msg.get("value").and_then(|value| value.as_f64()) {
                    cx.get_param_setter()
                        .set_parameter(&self.params.tuning_depth, depth as f32);
                }
            }
            "setHarmonicLimit" => {
                if let Some(limit) = msg.get("value").and_then(|value| value.as_str()) {
                    if let Some(limit) = parse_harmonic_limit(limit) {
                        cx.get_param_setter()
                            .set_parameter(&self.params.harmonic_limit, limit);
                    }
                }
            }
            "setTuningCompare" => {
                if let Some(enabled) = msg.get("value").and_then(|value| value.as_bool()) {
                    self.compare_standard.store(enabled, Ordering::Release);
                }
            }
            "setSlideConfig" => {
                if let Some(value) = msg.get("value") {
                    if let Ok(config) = serde_json::from_value::<SlideConfig>(value.clone()) {
                        if config.validate() {
                            *self
                                .slide_config
                                .lock()
                                .unwrap_or_else(|error| error.into_inner()) = config;
                        }
                    }
                }
            }
            "setInputMode" => {
                if let Some(mode_str) = msg.get("value").and_then(|v| v.as_str()) {
                    if let Some(input_variant) = parse_input_mode(mode_str) {
                        let setter = cx.get_param_setter();
                        setter.set_parameter(
                            &self.params.input_mode,
                            crate::effective_input_mode(input_variant, self.guitar_component),
                        );
                    }
                }
            }
            "setMidiOutputMode" => {
                if let Some(mode_str) = msg.get("value").and_then(|v| v.as_str()) {
                    if let Some(output_variant) = parse_midi_output_mode(mode_str) {
                        let setter = cx.get_param_setter();
                        setter.set_parameter(&self.params.midi_output_mode, output_variant);
                    }
                }
            }
            "setSynthEnabled" => {
                if let Some(enabled) = msg.get("value").and_then(|v| v.as_bool()) {
                    let setter = cx.get_param_setter();
                    setter.set_parameter(&self.params.synth_enabled, enabled);
                }
            }
            "setSynthGain" => {
                if let Some(gain) = msg.get("value").and_then(|v| v.as_f64()) {
                    let setter = cx.get_param_setter();
                    setter.begin_set_parameter(&self.params.synth_gain);
                    setter.set_parameter(&self.params.synth_gain, gain as f32);
                    setter.end_set_parameter(&self.params.synth_gain);
                }
            }
            "setSynthMixGain" => {
                let group = msg
                    .get("group")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(u64::MAX);
                let gain = msg.get("value").and_then(|v| v.as_f64());
                let params = [
                    &self.params.synth_input_gain,
                    &self.params.synth_harmony_gain,
                    &self.params.synth_canon_gain,
                    &self.params.synth_counterpoint_gain,
                ];
                if let (Some(param), Some(gain)) = (params.get(group as usize), gain) {
                    let setter = cx.get_param_setter();
                    setter.begin_set_parameter(*param);
                    setter.set_parameter(*param, gain as f32);
                    setter.end_set_parameter(*param);
                }
            }
            "panic" => {
                self.panic_requested.store(true, Ordering::Release);
            }
            // ─── Companion IPC ────────────────────────────────────
            // Companion commands are user-rate and must not be dropped.
            // The editor may wait briefly; the audio thread never waits.
            "companionSetEnabled" => {
                if let Some(enabled) = msg.get("value").and_then(|v| v.as_bool()) {
                    let c = self.companion.lock().unwrap_or_else(|e| e.into_inner());
                    c.enabled
                        .store(enabled, std::sync::atomic::Ordering::Release);
                }
            }
            "companionSetGlobalHoldMode" => {
                if let Some(hm_json) = msg.get("value") {
                    if let Some(mode) = contrapunk_companion::lane::hold_mode_from_json(hm_json) {
                        let c = self.companion.lock().unwrap_or_else(|e| e.into_inner());
                        c.set_global_hold_mode(mode);
                    }
                }
            }
            "canonConfigure" => {
                if let Some(partial) = msg.get("value") {
                    let mut c = self.companion.lock().unwrap_or_else(|e| e.into_inner());
                    let _ = c.configure_lane("canon", partial.clone());
                }
            }
            "counterpointConfigure" => {
                if let Some(partial) = msg.get("value") {
                    let mut c = self.companion.lock().unwrap_or_else(|e| e.into_inner());
                    let _ = c.configure_lane("counterpoint", partial.clone());
                }
            }
            "canonSetVoices" => {
                // Voices arrive as the same JSON shape Tauri's
                // canon_set_voices command builds. Delegate to the
                // canon lane's configure_lane wrapped with { voices }.
                if let Some(voices) = msg.get("value") {
                    let mut c = self.companion.lock().unwrap_or_else(|e| e.into_inner());
                    let payload = serde_json::json!({ "voices": voices });
                    let _ = c.configure_lane("canon", payload);
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

fn webview_workdir() -> PathBuf {
    // FL Studio overrides TEMP/TMP with a directory under Program Files, which
    // WebView2 cannot write without elevation. Use the per-user data root.
    #[cfg(target_os = "windows")]
    if let Some(root) = std::env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .or_else(|| {
            std::env::var_os("USERPROFILE")
                .map(PathBuf::from)
                .map(|home| home.join("AppData").join("Local"))
        })
    {
        return root.join("Contrapunk").join("WebView2");
    }

    std::env::temp_dir().join("contrapunk-webview")
}

/// Create the WebViewEditor for the plugin.
pub fn create_editor(
    params: Arc<ContrapunkParams>,
    note_state: Arc<Mutex<PluginNoteState>>,
    guitar_signal: Arc<Mutex<PluginGuitarSignal>>,
    companion: Arc<Mutex<Companion>>,
    panic_requested: Arc<AtomicBool>,
    compare_standard: Arc<AtomicBool>,
    slide_config: Arc<Mutex<SlideConfig>>,
    guitar_component: bool,
    state: &Arc<WebViewState>,
) -> WebViewEditor {
    let protocol = "contrapunk".to_string();
    let config = WebViewConfig {
        title: "Contrapunk".to_string(),
        source: WebViewSource::CustomProtocol {
            protocol: protocol.clone(),
            // nih-plug-webview turns this into `contrapunk://localhost/{url}`.
            // Use the origin root so SvelteKit hydrates the `/` route instead
            // of treating `/index.html` as an app route and showing a 404.
            url: String::new(),
        },
        workdir: webview_workdir(),
    };

    let handler = ContrapunkEditorHandler {
        params,
        note_state,
        companion,
        last_note_json: String::new(),
        panic_requested,
        compare_standard,
        slide_config,
        guitar_component,
        guitar_signal,
        guitar_was_live: false,
    };

    WebViewEditor::new_with_webview(handler, state, config, move |w| {
        let proto = protocol.clone();
        w.with_custom_protocol(proto, |_id, req| {
            let uri = req.uri().to_string();
            let path = req.uri().path();
            let (body, mime) = serve_embedded_asset(path, &uri);
            nih_plug_webview::wry::http::Response::builder()
                .header("Content-Type", mime)
                .body(body)
                .unwrap()
        })
    })
}

/// Serve embedded UI assets from the compiled-in Svelte build.
/// Falls back to a minimal placeholder if the build isn't available.
fn serve_embedded_asset(path: &str, uri: &str) -> (Cow<'static, [u8]>, &'static str) {
    #[cfg(feature = "embed-ui")]
    {
        let path = match path {
            "" | "/" | "/index.html" => "/index.html",
            path => path,
        };
        return match get_plugin_build_asset(path) {
            Some(body) => (Cow::Borrowed(body), mime_for_path(path)),
            None => (
                Cow::Owned(format!("Not Found: path={path:?} uri={uri:?}").into_bytes()),
                "text/plain",
            ),
        };
    }

    #[cfg(not(feature = "embed-ui"))]
    {
        if path == "" || path == "/" || path == "/index.html" {
            let html = include_str!("editor_fallback.html");
            return (Cow::Owned(html.as_bytes().to_vec()), "text/html");
        }
        (
            Cow::Owned(format!("Not Found: path={path:?} uri={uri:?}").into_bytes()),
            "text/plain",
        )
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

use crate::{
    PluginInputMode, PluginKey, PluginMidiOutputMode, PluginMode, PluginOctaveMode,
    PluginVoiceLeadingStyle,
};

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

fn parse_voice_leading_style(s: &str) -> Option<PluginVoiceLeadingStyle> {
    match s {
        "Free" => Some(PluginVoiceLeadingStyle::Free),
        "Jazz" => Some(PluginVoiceLeadingStyle::Jazz),
        "Palestrina" => Some(PluginVoiceLeadingStyle::Palestrina),
        "BachChorale" => Some(PluginVoiceLeadingStyle::BachChorale),
        _ => None,
    }
}

fn parse_tuning_style(s: &str) -> Option<PluginTuningStyle> {
    match s {
        "Standard" | "standard" => Some(PluginTuningStyle::Standard),
        "Pure" | "pure" => Some(PluginTuningStyle::Pure),
        _ => None,
    }
}

fn parse_harmonic_limit(s: &str) -> Option<PluginHarmonicLimit> {
    match s {
        "Five" | "five" | "5" => Some(PluginHarmonicLimit::Five),
        "Seven" | "seven" | "7" => Some(PluginHarmonicLimit::Seven),
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

fn parse_midi_output_mode(s: &str) -> Option<PluginMidiOutputMode> {
    match s {
        "Full" | "Full Contrapunk" | "full" => Some(PluginMidiOutputMode::Full),
        "PassThrough" | "Pass Through" | "pass_through" => Some(PluginMidiOutputMode::PassThrough),
        _ => None,
    }
}

/// Create a new WebViewState with default dimensions.
pub fn default_webview_state() -> Arc<WebViewState> {
    Arc::new(WebViewState::new(EDITOR_WIDTH, EDITOR_HEIGHT))
}
