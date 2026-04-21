//! Minimal CLAP host implementation.
//!
//! Only enough surface to satisfy `clack-host`'s trait bounds so
//! `PluginInstance::new` compiles. No extensions are declared yet — we
//! will layer on `HostLog`, `HostGui`, `HostParams`, `HostTimer` when
//! the follow-up iteration actually drives plugins through audio.
//!
//! # Threading
//!
//! clack splits host-facing callbacks by thread affinity:
//!
//! * [`SharedHandler`] — any thread (`Send + Sync`).
//! * [`MainThreadHandler`] — main thread only.
//! * [`AudioProcessorHandler`] — audio thread only.
//!
//! Until we wire a plugin into `ClapBlock::process`, the audio-thread
//! handler is `()`. The main-thread handler holds the initialised
//! plugin handle so the chain controller can call into the plugin
//! between audio buffers (parameter reads, extension queries).

use clack_host::prelude::*;

/// Human-facing host identity returned to every plugin on load.
pub fn host_info() -> HostInfo {
    HostInfo::new(
        "Contrapunk",
        "Contrapunk",
        "https://github.com/waveywaves/contrapunk",
        env!("CARGO_PKG_VERSION"),
    )
    .expect("HostInfo strings are static and contain no NULs")
}

/// The zero-sized host type that glues the three thread-scoped
/// handlers together. Implementing [`HostHandlers`] is what lets us
/// call `PluginInstance::<ContrapunkHost>::new(...)`.
pub struct ContrapunkHost;

impl HostHandlers for ContrapunkHost {
    type Shared<'a> = ContrapunkHostShared;
    type MainThread<'a> = ContrapunkHostMainThread<'a>;
    type AudioProcessor<'a> = ();

    fn declare_extensions(_builder: &mut HostExtensions<Self>, _shared: &Self::Shared<'_>) {
        // Intentionally empty for v1. Log / GUI / Params / Timer
        // extensions will be registered here once the real block is
        // wired to the audio callback.
    }
}

/// Shared host state accessible from any thread. Lives for the life
/// of the plugin instance. Kept empty for v1 — no inter-thread
/// plumbing needed while the block is stubbed.
pub struct ContrapunkHostShared;

impl ContrapunkHostShared {
    pub fn new() -> Self {
        Self
    }
}

impl Default for ContrapunkHostShared {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> SharedHandler<'a> for ContrapunkHostShared {
    fn initializing(&self, _instance: InitializingPluginHandle<'a>) {
        // Hook to grab extension pointers. No-op for v1.
    }

    fn request_restart(&self) {
        // Plugin asking to be restarted. We ignore for now; a real
        // impl would signal the main thread to recreate the plugin.
    }

    fn request_process(&self) {
        // Plugin asking us to begin processing. Our stream is always
        // active while the chain exists, so no action needed.
    }

    fn request_callback(&self) {
        // Plugin asking for `on_main_thread` to be called. Will be
        // routed through a channel when we add Timer / GUI support.
    }
}

/// Main-thread-only host state. Holds the initialised plugin handle
/// so the chain controller can talk to the plugin between buffers.
pub struct ContrapunkHostMainThread<'a> {
    _shared: &'a ContrapunkHostShared,
    plugin: Option<InitializedPluginHandle<'a>>,
}

impl<'a> ContrapunkHostMainThread<'a> {
    pub fn new(shared: &'a ContrapunkHostShared) -> Self {
        Self {
            _shared: shared,
            plugin: None,
        }
    }

    /// Access the initialised plugin handle, if the plugin has
    /// finished the `initialize` phase. Used by follow-up work to read
    /// the plugin's extensions.
    pub fn plugin(&self) -> Option<&InitializedPluginHandle<'a>> {
        self.plugin.as_ref()
    }
}

impl<'a> MainThreadHandler<'a> for ContrapunkHostMainThread<'a> {
    fn initialized(&mut self, instance: InitializedPluginHandle<'a>) {
        self.plugin = Some(instance);
    }
}
