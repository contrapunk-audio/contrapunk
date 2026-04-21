//! Main-thread plugin controller.
//!
//! Owns the `PluginInstance` (which is `!Send` — bound to the thread
//! that created it) plus the loaded `PluginEntry` that backs it. All
//! main-thread plugin operations (GUI create/show/destroy, parameter
//! reads) go through here.
//!
//! Constructed via [`ClapPluginController::load_and_activate`] which
//! returns both the controller (to keep on the main thread) and the
//! activated `PluginAudioProcessor` (to send to the audio thread).

use std::ffi::CString;
use std::path::{Path, PathBuf};

use clack_extensions::audio_ports::AudioPortInfoBuffer;
use clack_extensions::gui::{
    GuiApiType, GuiConfiguration, GuiError, GuiSize, PluginGui, Window as ClapWindow,
};
use clack_host::prelude::*;

use super::block::ClapBlockError;
use super::embed::EmbeddedPluginView;
use super::host::{host_info, ContrapunkHost, ContrapunkHostMainThread, ContrapunkHostShared};
use super::window::PluginWindow;

/// Where the plugin's GUI should appear.
#[derive(Clone, Copy, Debug)]
pub enum GuiTarget {
    /// Spawn a dedicated OS window (current default; good for big plugins).
    Detached,
    /// Embed the plugin's view inside Contrapunk's main Tauri window at
    /// the given rect (webview-local coordinates, y from top).
    EmbedInHost {
        ns_window_ptr: usize, // carries *mut c_void; usize for Send across threads
        x: f64,
        y: f64,
        width: f64,
        height: f64,
    },
}

/// Per-port channel counts for a plugin. Input and output.
#[derive(Clone, Debug)]
pub struct PortLayout {
    pub inputs: Vec<u32>,
    pub outputs: Vec<u32>,
}

impl PortLayout {
    /// Stereo-only fallback used when a plugin doesn't declare an
    /// `audio_ports` extension.
    pub fn stereo_fallback() -> Self {
        Self {
            inputs: vec![2],
            outputs: vec![2],
        }
    }
}

/// Main-thread-only handle to a loaded plugin. Holds the plugin
/// instance for GUI + parameter access; stores the loaded entry so
/// the plugin's library stays mapped.
pub struct ClapPluginController {
    pub path: String,
    pub name: String,
    pub has_gui: bool,
    pub gui_open: bool,
    pub sample_rate: f64,
    pub max_frames: u32,
    pub port_layout: PortLayout,
    gui_configuration: Option<GuiConfiguration<'static>>,
    /// Host-owned native window used for CLAP-embedded GUIs opened in
    /// `Detached` target mode. `None` when detached mode isn't used.
    gui_window: Option<PluginWindow>,
    /// Host-owned NSView used for GUIs embedded inside Contrapunk's
    /// main window (`EmbedInHost` target). `None` when detached.
    gui_embed_view: Option<EmbeddedPluginView>,
    /// Audio processor in Started state. Phase 2 moves this onto the
    /// audio thread via `take_processor`; until then it sits here so
    /// the plugin stays in the activated + processing state (GUI
    /// needs the instance activated).
    processor: Option<StartedPluginAudioProcessor<ContrapunkHost>>,
    /// Hold the entry so the underlying shared library stays loaded
    /// for the lifetime of the instance. Must outlive `instance` —
    /// enforced by field declaration order (drop order is top to
    /// bottom).
    instance: PluginInstance<ContrapunkHost>,
    _entry: PluginEntry,
}

impl ClapPluginController {
    /// Load a CLAP bundle, instantiate the plugin, activate it with
    /// the given audio parameters, and start processing. Returns the
    /// controller and the audio processor handle for the audio thread.
    pub fn load_and_activate(
        path: impl AsRef<Path>,
        sample_rate: f64,
        min_frames: u32,
        max_frames: u32,
    ) -> Result<Self, ClapBlockError> {
        let path_ref = path.as_ref();
        let path_buf: PathBuf = path_ref.to_path_buf();

        if !path_ref.exists() {
            return Err(ClapBlockError::NotFound(path_buf));
        }

        // SAFETY: user-selected .clap bundle; dlopen can execute
        // arbitrary init code but that's the nature of plugin hosting.
        let entry = unsafe {
            PluginEntry::load(path_ref).map_err(|e| ClapBlockError::LoadFailed {
                path: path_buf.clone(),
                source: Box::new(e),
            })?
        };

        let (plugin_id_c, plugin_name) = {
            let factory = entry
                .get_plugin_factory()
                .ok_or_else(|| ClapBlockError::NoFactory(path_buf.clone()))?;
            let descriptor = factory
                .plugin_descriptors()
                .next()
                .ok_or_else(|| ClapBlockError::NoDescriptors(path_buf.clone()))?;
            let id = descriptor
                .id()
                .ok_or_else(|| ClapBlockError::NoDescriptors(path_buf.clone()))?;
            let name = descriptor
                .name()
                .and_then(|c| c.to_str().ok())
                .map(str::to_string)
                .unwrap_or_else(|| {
                    path_ref
                        .file_stem()
                        .map(|s| s.to_string_lossy().into_owned())
                        .unwrap_or_default()
                });
            let id_c = CString::new(id.to_bytes()).map_err(|e| ClapBlockError::LoadFailed {
                path: path_buf.clone(),
                source: Box::new(e),
            })?;
            (id_c, name)
        };

        let mut instance = PluginInstance::<ContrapunkHost>::new(
            |_shared| ContrapunkHostShared::new(),
            |shared| ContrapunkHostMainThread::new(shared),
            &entry,
            &plugin_id_c,
            &host_info(),
        )
        .map_err(|e| ClapBlockError::LoadFailed {
            path: path_buf.clone(),
            source: Box::new(e),
        })?;

        // Query the plugin's audio-port layout. Crucial — if we
        // build mismatched buffers (e.g. 1 input port when the
        // plugin has sidechain on port 1) the plugin dereferences
        // garbage and segfaults inside process().
        let port_layout = {
            let audio_ports_ext = instance.access_handler(|h| h.audio_ports);
            match audio_ports_ext {
                Some(ext) => {
                    let mut buf = AudioPortInfoBuffer::new();
                    let handle = &mut instance.plugin_handle();
                    let mut inputs = Vec::new();
                    let n_in = ext.count(handle, true);
                    for i in 0..n_in {
                        if let Some(info) = ext.get(handle, i, true, &mut buf) {
                            inputs.push(info.channel_count);
                        } else {
                            inputs.push(2);
                        }
                    }
                    let mut outputs = Vec::new();
                    let n_out = ext.count(handle, false);
                    for i in 0..n_out {
                        if let Some(info) = ext.get(handle, i, false, &mut buf) {
                            outputs.push(info.channel_count);
                        } else {
                            outputs.push(2);
                        }
                    }
                    let layout = PortLayout { inputs, outputs };
                    eprintln!(
                        "[clap/ports] plugin {} layout: in={:?} out={:?}",
                        plugin_name, layout.inputs, layout.outputs
                    );
                    layout
                }
                None => {
                    eprintln!(
                        "[clap/ports] plugin {} has no audio_ports extension; using stereo fallback",
                        plugin_name
                    );
                    PortLayout::stereo_fallback()
                }
            }
        };

        // Check if the plugin exposes a GUI extension (and negotiate
        // a platform configuration). We prefer floating since our
        // Tauri shell can't host embedded NSViews cleanly.
        let (has_gui, gui_configuration) = {
            let gui_handle: Option<PluginGui> = instance.access_handler(|h| h.gui);
            match gui_handle {
                Some(gui) => {
                    let cfg = negotiate_gui_configuration(&gui, &mut instance.plugin_handle());
                    (cfg.is_some(), cfg)
                }
                None => (false, None),
            }
        };

        // Activate + start the audio processor.
        let config = PluginAudioConfiguration {
            sample_rate,
            min_frames_count: min_frames,
            max_frames_count: max_frames,
        };
        let stopped_processor =
            instance
                .activate(|_, _| (), config)
                .map_err(|e| ClapBlockError::LoadFailed {
                    path: path_buf.clone(),
                    // `activate`'s error type holds !Sync phantom data;
                    // stringify so we can fit our error bound.
                    source: format!("activate failed: {e}").into(),
                })?;
        let processor =
            stopped_processor
                .start_processing()
                .map_err(|e| ClapBlockError::LoadFailed {
                    path: path_buf.clone(),
                    source: format!("start_processing failed: {e}").into(),
                })?;

        Ok(Self {
            path: path_buf.to_string_lossy().into_owned(),
            name: plugin_name,
            has_gui,
            gui_open: false,
            sample_rate,
            max_frames,
            port_layout,
            gui_configuration,
            gui_window: None,
            gui_embed_view: None,
            processor: Some(processor),
            instance,
            _entry: entry,
        })
    }

    /// Hand off the started audio processor. After calling this the
    /// controller no longer holds it; phase 2 will wire the returned
    /// processor into `ClapBlock::process` on the audio thread.
    pub fn take_processor(&mut self) -> Option<StartedPluginAudioProcessor<ContrapunkHost>> {
        self.processor.take()
    }

    /// Preferred GUI size the plugin wants to render at, in pixels.
    /// Queryable without opening the GUI. Returns `None` if the
    /// plugin doesn't implement the GUI extension.
    pub fn preferred_gui_size(&mut self) -> Option<(u32, u32)> {
        let gui: PluginGui = self.instance.access_handler(|h| h.gui)?;
        let config = self.gui_configuration?;
        // `get_size` is only reliable after `create`, so call create
        // temporarily if the GUI isn't already open. Some plugins
        // return canned defaults before create; it's still better
        // than our hard-coded fallback.
        if self.gui_open {
            let handle = &mut self.instance.plugin_handle();
            return gui.get_size(handle).map(|s| (s.width, s.height));
        }
        let handle = &mut self.instance.plugin_handle();
        if gui.create(handle, config).is_err() {
            return None;
        }
        let size = gui.get_size(handle).map(|s| (s.width, s.height));
        gui.destroy(handle);
        size
    }

    /// Open the plugin's GUI. Floating plugins get their own OS
    /// window; embedded plugins are hosted in a dedicated NSWindow
    /// we spawn. Returns Err if the plugin doesn't support GUI at
    /// all, or the create/set_parent/show handshake fails.
    ///
    /// Idempotent: if the GUI is already open, this is a no-op.
    /// Some plugins (Aria/Garritan sforzando) `abort()` if
    /// `gui.create` is called twice without a destroy in between.
    pub fn open_gui(&mut self, target: GuiTarget) -> Result<(), GuiError> {
        // Reconcile: if the plugin signalled its window was closed
        // while we weren't looking, tear down before re-creating —
        // otherwise Aria-family plugins abort on re-create.
        let user_closed = self
            .instance
            .access_shared_handler(|shared| shared.take_gui_closed());
        if user_closed && self.gui_open {
            self.close_gui();
        }
        if self.gui_open {
            return Ok(());
        }
        let Some(config) = self.gui_configuration else {
            return Err(GuiError::CreateError);
        };
        let gui: PluginGui = self
            .instance
            .access_handler(|h| h.gui)
            .ok_or(GuiError::CreateError)?;

        let name = self.name.clone();
        let handle = &mut self.instance.plugin_handle();
        gui.create(handle, config)?;
        let title = std::ffi::CString::new(name.as_bytes())
            .unwrap_or_else(|_| std::ffi::CString::new("Plugin").unwrap());
        gui.suggest_title(handle, title.as_c_str());

        if config.is_floating {
            // CLAP-floating: plugin creates its own OS window.
            // `target` is ignored (floating plugins manage their own).
            gui.show(handle)?;
        } else {
            match target {
                GuiTarget::Detached => {
                    // Spawn a dedicated NSWindow host.
                    let size = gui.get_size(handle).unwrap_or(GuiSize {
                        width: 800,
                        height: 500,
                    });
                    let window = PluginWindow::new(&name, size.width as f64, size.height as f64);
                    let ns_view = window.ns_view_ptr();
                    // SAFETY: `ns_view` is valid while `window` is alive.
                    let clap_window = ClapWindow::from_cocoa_nsview(ns_view);
                    unsafe { gui.set_parent(handle, clap_window)? };
                    let _ = gui.show(handle);
                    self.gui_window = Some(window);
                }
                GuiTarget::EmbedInHost {
                    ns_window_ptr,
                    x,
                    y,
                    width,
                    height,
                } => {
                    let frame = clack_extensions::gui::GuiSize {
                        width: width as u32,
                        height: height as u32,
                    };
                    let _ = frame; // unused; included for any future sizing hints
                    let ns_rect = objc2_foundation::NSRect {
                        origin: objc2_foundation::NSPoint { x, y },
                        size: objc2_foundation::NSSize { width, height },
                    };
                    // SAFETY: caller (Tauri command) guarantees
                    // `ns_window_ptr` came from `WebviewWindow::ns_window`
                    // and we're on the main thread (run_on_main_thread).
                    let view = unsafe {
                        EmbeddedPluginView::new(ns_window_ptr as *mut std::ffi::c_void, ns_rect)
                    }
                    .ok_or(GuiError::CreateError)?;
                    let ns_view = view.ns_view_ptr();
                    let clap_window = ClapWindow::from_cocoa_nsview(ns_view);
                    unsafe { gui.set_parent(handle, clap_window)? };
                    let _ = gui.show(handle);
                    self.gui_embed_view = Some(view);
                }
            }
        }

        self.gui_open = true;
        Ok(())
    }

    /// Reposition the embedded plugin view. No-op if the GUI is in
    /// detached/floating mode. Called from the UI when the rack
    /// scrolls or resizes.
    pub fn set_embed_frame(&self, x: f64, y: f64, width: f64, height: f64) {
        if let Some(view) = &self.gui_embed_view {
            view.set_frame(x, y, width, height);
        }
    }

    /// Close the plugin's GUI window. Idempotent.
    pub fn close_gui(&mut self) {
        if !self.gui_open {
            return;
        }
        if let Some(gui) = self.instance.access_handler(|h| h.gui) {
            let handle = &mut self.instance.plugin_handle();
            let _ = gui.hide(handle);
            gui.destroy(handle);
        }
        // Drop host-side GUI containers AFTER the plugin destroyed
        // its subview — reverse creation order.
        self.gui_embed_view = None;
        self.gui_window = None;
        self.gui_open = false;
    }
}

impl Drop for ClapPluginController {
    fn drop(&mut self) {
        self.close_gui();
    }
}

/// Try to find a GUI configuration the host and plugin both accept.
/// We prefer floating mode because our Tauri webview shell can't
/// natively host embedded NSViews without extra work.
fn negotiate_gui_configuration(
    gui: &PluginGui,
    plugin: &mut PluginMainThreadHandle,
) -> Option<GuiConfiguration<'static>> {
    let api_type = GuiApiType::default_for_current_platform()?;
    // Prefer embedded — we host it in our own NSWindow so we have
    // explicit control over z-order, focus, and lifecycle. Fall back
    // to floating only if the plugin refuses embedded.
    let embedded = GuiConfiguration {
        api_type,
        is_floating: false,
    };
    let supports_embedded = gui.is_api_supported(plugin, embedded);
    eprintln!("[clap/gui] negotiate: api={api_type:?} embedded={supports_embedded}");
    if supports_embedded {
        return Some(embedded);
    }
    let floating = GuiConfiguration {
        api_type,
        is_floating: true,
    };
    let supports_floating = gui.is_api_supported(plugin, floating);
    eprintln!("[clap/gui] negotiate: floating={supports_floating}");
    if supports_floating {
        Some(floating)
    } else {
        None
    }
}
