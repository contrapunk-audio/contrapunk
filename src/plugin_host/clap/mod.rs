//! CLAP plugin hosting scaffold.
//!
//! # Crate choice: `clack-host` (git) over `clap-sys` or a custom wrapper
//!
//! The CLAP ecosystem in Rust currently has two realistic choices:
//!
//! 1. **`clap-sys`** (crates.io v0.5.0) — raw `#[repr(C)]` bindings to
//!    the CLAP ABI. Every host responsibility is on us: factory
//!    discovery, entry-point lifecycle, audio-buffer marshaling,
//!    event-queue plumbing, extension negotiation. This is a lot of
//!    unsafe code to get right before we can hear a plugin.
//!
//! 2. **`clack-host`** (git-only, `prokopyl/clack`) — a safe, idiomatic
//!    host API built on top of `clap-sys`. Provides `PluginBundle`,
//!    `PluginInstance`, typed host handlers, and an extension system.
//!    Companion `clack-finder` helper knows the standard CLAP paths per
//!    OS. The trade-off: the crate is not yet published on crates.io
//!    (author flagged the API as "feature-complete but subject to
//!    refinement"), so we pin to an explicit git commit.
//!
//! We go with `clack-host` pinned by `rev`. Rationale:
//!
//! * Safe wrappers around the ABI dramatically shrink the unsafe
//!   surface inside Contrapunk. Getting the lifecycle wrong on a host
//!   can crash not just our process but corrupt plugin state on disk
//!   (presets, PDCs); `clack-host` shoulders most of that risk.
//! * Extension handling (audio-ports, note-ports, params, state, gui)
//!   is already mapped to typed Rust APIs in `clack-extensions`.
//! * The author is responsive and the API is the de-facto Rust host
//!   reference — the Bitwig-adjacent plugin community uses it.
//!
//! If `clack-host` ships on crates.io we switch the dep to a regular
//! semver line. Until then the pinned `rev` is intentional: it prevents
//! a surprise upstream breaking change from bricking the build.
//!
//! # Scope of this module
//!
//! * [`discovery`] — scan OS-standard CLAP directories and return a
//!   list of [`PluginDescriptor`]s the UI can render.
//! * [`host`] — a minimal [`host::ContrapunkHost`] that implements the
//!   clack traits required to load plugins. Stubbed for v1: most
//!   callbacks return defaults or ignore requests.
//! * [`block`] — [`block::ClapBlock`], an `AudioBlock` wrapping a
//!   loaded plugin. Currently a skeleton: `process` outputs silence
//!   and MIDI is queued into a fixed-size ring buffer. The follow-up
//!   iteration wires real plugin audio through the block.
//!
//! # Known v1 limitations
//!
//! * Plugins are *discovered* but not *instantiated* by default. A real
//!   CLAP activation + process loop requires threading the host
//!   callbacks through audio/main thread boundaries, and the audio
//!   callback doesn't yet have a command queue for "add block at
//!   runtime". Until that command queue exists, `ClapBlock::new_loaded`
//!   and `add_clap_plugin_to_chain` are stubs that surface a clear
//!   error.
//! * No GUI embedding. Plugins open their own OS windows for v1.
//! * No parameter automation, no preset save/load, no per-block audio
//!   routing graph beyond "serial stereo".

pub mod audio_block;
pub mod block;
pub mod controller;
pub mod discovery;
pub mod host;
pub mod registry;
pub mod window;

pub use audio_block::ClapAudioBlock;
pub use block::ClapBlock;
pub use controller::ClapPluginController;
pub use discovery::{discover_plugins, PluginDescriptor};
pub use registry::PluginId;
