//! Host implementation for loaded CLAP plugins.
//!
//! Declares [`HostLog`], [`HostGui`], [`HostAudioPorts`], and
//! [`HostNotePorts`] so the plugin can query them. Each impl is
//! minimal but sufficient to satisfy the spec.
//!
//! # Threading
//!
//! clack splits host-facing callbacks by thread affinity:
//!
//! * [`SharedHandler`] — any thread (`Send + Sync`).
//! * [`MainThreadHandler`] — main thread only.
//! * [`AudioProcessorHandler`] — audio thread only.

use clack_extensions::audio_ports::{
    AudioPortRescanFlags, HostAudioPorts, HostAudioPortsImpl, PluginAudioPorts,
};
use clack_extensions::gui::{GuiSize, HostGui, HostGuiImpl, PluginGui};
use clack_extensions::log::{HostLog, HostLogImpl, LogSeverity};
use clack_extensions::note_ports::{
    HostNotePorts, HostNotePortsImpl, NoteDialects, NotePortRescanFlags,
};
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

/// Zero-sized marker that implements [`HostHandlers`]. Every plugin we
/// load is parameterised on this type.
pub struct ContrapunkHost;

impl HostHandlers for ContrapunkHost {
    type Shared<'a> = ContrapunkHostShared;
    type MainThread<'a> = ContrapunkHostMainThread<'a>;
    type AudioProcessor<'a> = ();

    fn declare_extensions(builder: &mut HostExtensions<Self>, _shared: &Self::Shared<'_>) {
        builder
            .register::<HostLog>()
            .register::<HostGui>()
            .register::<HostAudioPorts>()
            .register::<HostNotePorts>();
    }
}

/// Shared host state, accessible from any thread. Tracks whether the
/// plugin has signalled that its floating window was closed.
pub struct ContrapunkHostShared {
    gui_closed: std::sync::Mutex<bool>,
}

impl ContrapunkHostShared {
    pub fn new() -> Self {
        Self {
            gui_closed: std::sync::Mutex::new(false),
        }
    }

    /// Read and clear the "gui closed" flag. Polled by the main
    /// thread so the controller knows when to destroy GUI resources.
    pub fn take_gui_closed(&self) -> bool {
        let mut flag = self.gui_closed.lock().expect("gui_closed mutex");
        let was = *flag;
        *flag = false;
        was
    }
}

impl Default for ContrapunkHostShared {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> SharedHandler<'a> for ContrapunkHostShared {
    fn initializing(&self, _instance: InitializingPluginHandle<'a>) {}
    fn request_restart(&self) {}
    fn request_process(&self) {}
    fn request_callback(&self) {}
}

/// Main-thread host state. Caches the plugin's GUI and audio-ports
/// extension handles so we can call them without re-querying each time.
pub struct ContrapunkHostMainThread<'a> {
    _shared: &'a ContrapunkHostShared,
    pub gui: Option<PluginGui>,
    pub audio_ports: Option<PluginAudioPorts>,
}

impl<'a> ContrapunkHostMainThread<'a> {
    pub fn new(shared: &'a ContrapunkHostShared) -> Self {
        Self {
            _shared: shared,
            gui: None,
            audio_ports: None,
        }
    }
}

impl<'a> MainThreadHandler<'a> for ContrapunkHostMainThread<'a> {
    fn initialized(&mut self, instance: InitializedPluginHandle<'a>) {
        self.gui = instance.get_extension();
        self.audio_ports = instance.get_extension();
    }
}

// ------------ Extension implementations ------------

impl HostLogImpl for ContrapunkHostShared {
    fn log(&self, severity: LogSeverity, message: &str) {
        if severity <= LogSeverity::Debug {
            return;
        }
        eprintln!("[clap/{severity}] {message}");
    }
}

impl HostGuiImpl for ContrapunkHostShared {
    fn resize_hints_changed(&self) {}

    fn request_resize(&self, _new_size: GuiSize) -> Result<(), HostError> {
        // Floating windows own their own sizing.
        Ok(())
    }

    fn request_show(&self) -> Result<(), HostError> {
        Ok(())
    }

    fn request_hide(&self) -> Result<(), HostError> {
        Ok(())
    }

    fn closed(&self, _was_destroyed: bool) {
        if let Ok(mut flag) = self.gui_closed.lock() {
            *flag = true;
        }
    }
}

impl HostAudioPortsImpl for ContrapunkHostMainThread<'_> {
    fn is_rescan_flag_supported(&self, _flag: AudioPortRescanFlags) -> bool {
        false
    }
    fn rescan(&mut self, _flag: AudioPortRescanFlags) {}
}

impl HostNotePortsImpl for ContrapunkHostMainThread<'_> {
    fn supported_dialects(&self) -> NoteDialects {
        NoteDialects::CLAP
    }
    fn rescan(&mut self, _flags: NotePortRescanFlags) {}
}
