use std::borrow::Cow;
use std::path::PathBuf;
use std::sync::Arc;

use elixir_preset::RolePatchState;
use nih_plug::prelude::ParamSetter;
use nih_plug_webview::{
    Context, EditorHandler, WebViewConfig, WebViewEditor, WebViewSource, WebViewState,
};

use crate::ElixirParams;

const EDITOR_WIDTH: f64 = 760.0;
const EDITOR_HEIGHT: f64 = 720.0;

pub fn default_webview_state() -> Arc<WebViewState> {
    Arc::new(WebViewState::new(EDITOR_WIDTH, EDITOR_HEIGHT))
}

struct ElixirEditorHandler {
    params: Arc<ElixirParams>,
}

impl ElixirEditorHandler {
    fn params_json(&self) -> String {
        let patch = RolePatchState::from(self.params.patch.load());
        serde_json::json!({
            "type": "paramsUpdate",
            "product": "elixir",
            "synthGain": self.params.gain.value(),
            "rolePatches": [patch, RolePatchState::default(), RolePatchState::default(), RolePatchState::default()],
        })
        .to_string()
    }

    fn set_gain(&self, setter: ParamSetter<'_>, gain: f32) {
        if gain.is_finite() {
            setter.begin_set_parameter(&self.params.gain);
            setter.set_parameter(&self.params.gain, gain);
            setter.end_set_parameter(&self.params.gain);
        }
    }
}

impl EditorHandler for ElixirEditorHandler {
    fn on_frame(&mut self, _cx: &mut Context) {}

    fn on_message(&mut self, cx: &mut Context, message: String) {
        let Ok(message) = serde_json::from_str::<serde_json::Value>(&message) else {
            return;
        };
        match message.get("type").and_then(|value| value.as_str()) {
            Some("ready") => cx.send_message(self.params_json()),
            Some("setSynthGain") => {
                if let Some(gain) = message.get("value").and_then(|value| value.as_f64()) {
                    self.set_gain(cx.get_param_setter(), gain as f32);
                }
            }
            Some("setSynthRolePatch") => {
                let group = message
                    .get("group")
                    .and_then(|value| value.as_u64())
                    .unwrap_or(u64::MAX);
                if group == 0 {
                    if let Some(value) = message.get("patch") {
                        if let Ok(patch) = serde_json::from_value::<RolePatchState>(value.clone()) {
                            self.params.patch.store(patch.to_core());
                            cx.send_message(self.params_json());
                        }
                    }
                }
            }
            _ => {}
        }
    }

    fn on_params_changed(&mut self, cx: &mut Context) {
        cx.send_message(self.params_json());
    }
}

fn webview_workdir() -> PathBuf {
    #[cfg(target_os = "windows")]
    if let Some(root) = std::env::var_os("LOCALAPPDATA").map(PathBuf::from) {
        return root.join("Contrapunk").join("ElixirWebView2");
    }
    std::env::temp_dir().join("elixir-webview")
}

pub fn create_editor(params: Arc<ElixirParams>, state: &Arc<WebViewState>) -> WebViewEditor {
    let protocol = "elixir".to_string();
    let config = WebViewConfig {
        title: "Elixir".to_string(),
        source: WebViewSource::CustomProtocol {
            protocol: protocol.clone(),
            url: "elixir-plugin.html".to_string(),
        },
        workdir: webview_workdir(),
    };

    WebViewEditor::new_with_webview(
        ElixirEditorHandler { params },
        state,
        config,
        move |webview| {
            webview.with_custom_protocol(protocol.clone(), |_id, request| {
                let path = request.uri().path();
                let (body, mime) = serve_embedded_asset(path);
                nih_plug_webview::wry::http::Response::builder()
                    .header("Content-Type", mime)
                    .body(body)
                    .unwrap()
            })
        },
    )
}

fn serve_embedded_asset(path: &str) -> (Cow<'static, [u8]>, &'static str) {
    #[cfg(feature = "embed-ui")]
    {
        let path = match path {
            "" | "/" | "/elixir-plugin" | "/elixir-plugin/" => "/elixir-plugin.html",
            path => path,
        };
        return match get_plugin_build_asset(path) {
            Some(body) => (Cow::Borrowed(body), mime_for_path(path)),
            None => (Cow::Borrowed(b"Not Found"), "text/plain"),
        };
    }

    #[cfg(not(feature = "embed-ui"))]
    {
        let _ = path;
        (
            Cow::Borrowed(b"Build Elixir with the embed-ui feature."),
            "text/plain",
        )
    }
}

#[cfg(feature = "embed-ui")]
fn mime_for_path(path: &str) -> &'static str {
    match path.rsplit('.').next().unwrap_or("") {
        "html" => "text/html",
        "js" => "application/javascript",
        "css" => "text/css",
        "wasm" => "application/wasm",
        "json" => "application/json",
        "svg" => "image/svg+xml",
        "png" => "image/png",
        "woff2" => "font/woff2",
        _ => "application/octet-stream",
    }
}

#[cfg(feature = "embed-ui")]
mod embedded_ui_assets {
    include!(concat!(env!("OUT_DIR"), "/ui_assets.rs"));
}

#[cfg(feature = "embed-ui")]
fn get_plugin_build_asset(path: &str) -> Option<&'static [u8]> {
    embedded_ui_assets::get_asset(path)
}

#[cfg(all(test, feature = "embed-ui"))]
mod tests {
    use super::*;

    #[test]
    fn dedicated_editor_route_is_embedded() {
        let (body, mime) = serve_embedded_asset("/elixir-plugin.html");
        assert_eq!(mime, "text/html");
        assert!(body.starts_with(b"<!doctype html>"));
    }
}
