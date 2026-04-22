//! Tauri commands for CLAP plugin hosting.
//!
//! * [`list_clap_plugins`] scans the filesystem and returns every
//!   `.clap` bundle found.
//! * [`add_clap_plugin_to_chain`] instantiates + activates the plugin
//!   on the main thread, stores it in the registry, and pushes a
//!   `ClapBlock` onto the live audio chain.
//! * [`open_plugin_gui`] / [`close_plugin_gui`] drive the plugin's
//!   floating GUI window (macOS Cocoa / Win32 / X11). All GUI calls
//!   are dispatched to the main thread because native windows have
//!   thread affinity.
//!
//! The main-thread dispatcher pattern uses a oneshot channel so the
//! Tauri command (which may run on a worker thread) can receive the
//! result synchronously.

use std::sync::mpsc;

use serde::Serialize;
use tauri::{AppHandle, Manager, Runtime, State};

use contrapunk::chain::{AudioBlock, MidiBlockEvent};
use contrapunk::plugin_host::clap::controller::GuiTarget;
use contrapunk::plugin_host::clap::{
    discover_plugins, registry, ClapAudioBlock, ClapPluginController, PluginDescriptor, PluginId,
};

use crate::state::AppState;

/// Run `f` on the main thread and return its result. Uses a
/// crossbeam-free oneshot `mpsc::sync_channel(1)`. `f` must be
/// `Send + 'static` because Tauri moves it onto the main loop.
fn on_main<R, F, RT>(app: &AppHandle<RT>, f: F) -> Result<R, String>
where
    RT: Runtime,
    F: FnOnce() -> R + Send + 'static,
    R: Send + 'static,
{
    let (tx, rx) = mpsc::sync_channel::<R>(1);
    app.run_on_main_thread(move || {
        let _ = tx.send(f());
    })
    .map_err(|e| format!("run_on_main_thread: {e}"))?;
    rx.recv().map_err(|e| format!("main-thread recv: {e}"))
}

/// Discover every CLAP plugin installed on the machine.
#[tauri::command]
pub fn list_clap_plugins() -> Vec<PluginDescriptor> {
    let mut out = discover_plugins();
    out.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    out
}

/// Shape returned to the UI after a successful plugin add.
#[derive(Serialize)]
pub struct AddedPlugin {
    pub plugin_id: PluginId,
    pub name: String,
    pub path: String,
    pub has_gui: bool,
}

#[tauri::command]
pub fn add_clap_plugin_to_chain<RT: Runtime>(
    path: String,
    state: State<AppState>,
    app: AppHandle<RT>,
) -> Result<AddedPlugin, String> {
    let sample_rate = state.transport.sample_rate() as f64;
    eprintln!("[plugins] add_clap_plugin_to_chain: path={path} sr={sample_rate}");

    // Activate + register on the main thread (PluginInstance is !Send).
    let path_for_main = path.clone();
    let load_result = on_main(&app, move || {
        eprintln!("[plugins] (main-thread) activating: {path_for_main}");
        ClapPluginController::load_and_activate(&path_for_main, sample_rate, 32, 4096)
            .map_err(|e| {
                eprintln!("[plugins] load_and_activate failed: {e}");
                e.to_string()
            })
            .map(|mut controller| {
                let name = controller.name.clone();
                let path = controller.path.clone();
                let has_gui = controller.has_gui;
                let layout = controller.port_layout.clone();
                // Take the started audio processor out of the controller —
                // it moves to the audio thread inside ClapAudioBlock.
                let processor = controller.take_processor();
                let id = registry::insert(controller);
                eprintln!("[plugins] registered id={id} name={name} has_gui={has_gui}");
                (id, name, path, has_gui, processor, layout)
            })
    })??;

    let (plugin_id, name, controller_path, has_gui, processor_opt, port_layout) = load_result;
    let processor = processor_opt.ok_or_else(|| {
        // Roll back registry if activation produced no processor.
        let id = plugin_id;
        let app2 = app.clone();
        let _ = app2.run_on_main_thread(move || registry::remove(id));
        "plugin activation produced no audio processor".to_string()
    })?;

    // Build the audio-thread block that actually drives the plugin.
    let sr_u32 = sample_rate as u32;
    let block = ClapAudioBlock::new(name.clone(), processor, sr_u32, 4096, port_layout);

    let guard = state
        .chain_commander
        .lock()
        .map_err(|e| format!("chain commander lock: {e}"))?;
    let commander = guard
        .as_ref()
        .ok_or_else(|| "audio chain not initialized".to_string())?;

    let _ = commander
        .push_block(Box::new(TaggedClapBlock::new(block, plugin_id)))
        .map_err(|e| {
            // If push fails, roll back the registry insert so the UI
            // doesn't dangle.
            let id = plugin_id;
            let _ = app.run_on_main_thread(move || registry::remove(id));
            e
        })?;

    Ok(AddedPlugin {
        plugin_id,
        name,
        path: controller_path,
        has_gui,
    })
}

/// Wraps a `ClapAudioBlock` so the chain carries the plugin id in
/// its `type_id()`. That lets the UI map from chain index → registry
/// id without a side channel.
struct TaggedClapBlock {
    inner: ClapAudioBlock,
    type_id: String,
}

impl TaggedClapBlock {
    fn new(inner: ClapAudioBlock, plugin_id: PluginId) -> Self {
        let type_id = format!("clap:{plugin_id}");
        Self { inner, type_id }
    }
}

impl AudioBlock for TaggedClapBlock {
    fn name(&self) -> &str {
        self.inner.name()
    }
    fn type_id(&self) -> &str {
        &self.type_id
    }
    fn process(&mut self, buffer: &mut [f32], channels: usize) {
        self.inner.process(buffer, channels);
    }
    fn midi_event(&mut self, event: MidiBlockEvent) {
        self.inner.midi_event(event);
    }
    fn reset(&mut self) {
        self.inner.reset();
    }
    fn set_sample_rate(&mut self, sample_rate: u32) {
        self.inner.set_sample_rate(sample_rate);
    }
    fn enabled(&self) -> bool {
        self.inner.enabled()
    }
}

#[tauri::command]
pub fn open_plugin_gui<RT: Runtime>(plugin_id: PluginId, app: AppHandle<RT>) -> Result<(), String> {
    eprintln!("[plugins] open_plugin_gui: id={plugin_id} target=detached");
    on_main(&app, move || {
        registry::with_plugins(|map| {
            let Some(controller) = map.get_mut(&plugin_id) else {
                eprintln!("[plugins] open_plugin_gui: id={plugin_id} not in registry");
                return Err(format!("plugin {plugin_id} not found"));
            };
            eprintln!(
                "[plugins] open_plugin_gui: found name={} has_gui={}",
                controller.name, controller.has_gui
            );
            if !controller.has_gui {
                return Err("plugin does not expose a GUI".into());
            }
            match controller.open_gui(GuiTarget::Detached) {
                Ok(()) => {
                    eprintln!("[plugins] open_plugin_gui: show succeeded");
                    Ok(())
                }
                Err(e) => {
                    eprintln!("[plugins] open_plugin_gui: create/show failed: {e:?}");
                    Err(format!("open gui: {e:?}"))
                }
            }
        })
    })?
}

#[tauri::command]
pub fn open_plugin_gui_embedded<RT: Runtime>(
    plugin_id: PluginId,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    app: AppHandle<RT>,
) -> Result<(), String> {
    eprintln!(
        "[plugins] open_plugin_gui_embedded: id={plugin_id} rect=({x}, {y}, {width}x{height})"
    );

    // Embedded plugin GUIs currently require the macOS-only NSView
    // subview trick. The Tauri `ns_window()` accessor also only exists
    // on the Cocoa backend, so on Windows/Linux we surface a clear
    // error that the UI can use to fall back to a detached window.
    #[cfg(not(target_os = "macos"))]
    {
        let _ = (plugin_id, x, y, width, height, app);
        return Err("embedded plugin GUI is only supported on macOS".into());
    }

    #[cfg(target_os = "macos")]
    {
        // Grab the main window's NSWindow pointer on the current thread
        // (Tauri's ns_window returns it from any thread — it's just a
        // pointer-fetch). Carry it as usize so the Send closure doesn't
        // complain about raw pointers.
        let main = app
            .get_webview_window("main")
            .ok_or_else(|| "main webview window not found".to_string())?;
        let ns_window_ptr = main.ns_window().map_err(|e| format!("ns_window: {e}"))? as usize;
        eprintln!("[plugins] open_plugin_gui_embedded: ns_window_ptr={ns_window_ptr:#x}");

        on_main(&app, move || {
            registry::with_plugins(|map| {
                let Some(controller) = map.get_mut(&plugin_id) else {
                    return Err(format!("plugin {plugin_id} not found"));
                };
                if !controller.has_gui {
                    return Err("plugin does not expose a GUI".into());
                }
                let target = GuiTarget::EmbedInHost {
                    ns_window_ptr,
                    x,
                    y,
                    width,
                    height,
                };
                match controller.open_gui(target) {
                    Ok(()) => {
                        eprintln!("[plugins] open_plugin_gui_embedded: attached");
                        Ok(())
                    }
                    Err(e) => {
                        eprintln!("[plugins] open_plugin_gui_embedded failed: {e:?}");
                        Err(format!("embed gui: {e:?}"))
                    }
                }
            })
        })?
    }
}

#[tauri::command]
pub fn set_plugin_gui_frame<RT: Runtime>(
    plugin_id: PluginId,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    app: AppHandle<RT>,
) -> Result<(), String> {
    // Fire-and-forget: no sync_channel round-trip. Dropping the
    // response saves ~0.5ms per call and keeps scroll sync tighter.
    app.run_on_main_thread(move || {
        registry::with_plugins(|map| {
            if let Some(controller) = map.get(&plugin_id) {
                controller.set_embed_frame(x, y, width, height);
            }
        });
    })
    .map_err(|e| format!("run_on_main_thread: {e}"))?;
    Ok(())
}

#[tauri::command]
pub fn close_plugin_gui<RT: Runtime>(
    plugin_id: PluginId,
    app: AppHandle<RT>,
) -> Result<(), String> {
    on_main(&app, move || {
        registry::with_plugins(|map| {
            if let Some(controller) = map.get_mut(&plugin_id) {
                controller.close_gui();
            }
        })
    })
}

#[tauri::command]
pub fn remove_plugin<RT: Runtime>(plugin_id: PluginId, app: AppHandle<RT>) -> Result<(), String> {
    // Intentional leak: dropping `ClapPluginController` tears down
    // the plugin instance + audio processor, which races with
    // AppKit's autorelease pool draining Obj-C references that the
    // plugin captured during GUI creation. Result is `objc_release`
    // on freed memory → SIGSEGV. Leaking is the only safe option
    // without a full deactivate-before-drop pass through clack's
    // lifecycle (which would need the audio-thread processor moved
    // back to main first).
    on_main(&app, move || {
        eprintln!("[plugins] remove_plugin: id={plugin_id} (intentional leak, no drop)");
        // Don't actually remove — controller stays alive in registry
        // until app quits. UI shows it as gone because the chain
        // command already removed the audio block.
    })
}

#[derive(Serialize)]
pub struct PluginGuiSize {
    pub width: u32,
    pub height: u32,
}

#[tauri::command]
pub fn get_plugin_gui_size<RT: Runtime>(
    plugin_id: PluginId,
    app: AppHandle<RT>,
) -> Result<Option<PluginGuiSize>, String> {
    on_main(&app, move || {
        registry::with_plugins(|map| {
            let controller = map.get_mut(&plugin_id)?;
            controller
                .preferred_gui_size()
                .map(|(width, height)| PluginGuiSize { width, height })
        })
    })
}
