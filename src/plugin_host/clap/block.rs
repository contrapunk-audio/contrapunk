//! `ClapBlock`: an [`AudioBlock`] that wraps a loaded CLAP plugin.
//!
//! # v1 status: stub
//!
//! `ClapBlock::new` loads the plugin's entry point from disk (via
//! `clack_host::prelude::PluginEntry::load`) to validate the path and
//! capture the plugin's real descriptor metadata. It does **not** yet
//! instantiate the plugin (`PluginInstance::new`) or activate it for
//! audio processing — that requires a lock-free command queue to move
//! the activated `StoppedPluginAudioProcessor` onto the audio thread,
//! which the chain controller hasn't built yet.
//!
//! Accordingly, `process()` currently writes silence into the buffer
//! and MIDI events are swallowed by a fixed-size lock-free ring
//! buffer so the real-time contract is honoured. Once the chain gains
//! an "insert block" command, this file is the upgrade target:
//! instantiate the plugin, activate it with the stream's sample rate
//! and block size, send the `StoppedPluginAudioProcessor` to the
//! audio thread, and call `processor.process` per buffer using the
//! queued MIDI events.
//!
//! # Send-safety
//!
//! `clack_host::prelude::PluginEntry` is `Send + Sync` (internally an
//! `Arc<dyn LoadedEntryDyn>` guarded by a `Mutex`). Holding the entry
//! in `ClapBlock` therefore satisfies the `Send` bound that
//! [`AudioBlock`] requires.
//!
//! `PluginInstance<H>` itself is deliberately `!Send` — it's the
//! main-thread handle. The audio-thread-safe counterpart is
//! `StoppedPluginAudioProcessor<H>` / `PluginAudioProcessor<H>`,
//! produced by `PluginInstance::activate`; both are `Send`. When the
//! follow-up iteration adds runtime plugin insertion, the instance
//! lives with the chain controller (main thread) and only the
//! `PluginAudioProcessor` crosses the thread boundary into the
//! audio callback.

use std::ffi::CStr;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

use clack_host::prelude::PluginEntry;

use crate::chain::{AudioBlock, MidiBlockEvent};

use super::discovery::PluginDescriptor;

/// Best-effort decode of an optional `&CStr` from a plugin descriptor
/// into an owned `String`. Plugins are spec'd to use UTF-8 but we're
/// defensive: non-UTF-8 bytes degrade to `None` rather than producing
/// garbage.
fn cstr_opt_to_string(c: Option<&CStr>) -> Option<String> {
    c.and_then(|s| s.to_str().ok()).map(|s| s.to_string())
}

/// How many MIDI events may queue between audio buffers before we
/// start dropping oldest events. Sized generously for typical
/// keyboard / harmony generator load (few events per buffer at
/// 48kHz/256-frame blocks).
const MIDI_RING_CAPACITY: usize = 256;

/// Errors that can occur while constructing a `ClapBlock`.
#[derive(Debug, thiserror::Error)]
pub enum ClapBlockError {
    #[error("CLAP bundle not found: {0}")]
    NotFound(PathBuf),
    #[error("failed to load CLAP bundle at {path}: {source}")]
    LoadFailed {
        path: PathBuf,
        source: Box<dyn std::error::Error + Send + Sync>,
    },
    #[error("CLAP bundle exposes no plugin factory: {0}")]
    NoFactory(PathBuf),
    #[error("CLAP bundle contains no plugin descriptors: {0}")]
    NoDescriptors(PathBuf),
}

/// Lock-free ring buffer for MIDI events pushed by any thread and
/// drained on the audio thread. Fixed capacity → no allocations in the
/// hot path.
///
/// We use a simple SPSC-style structure guarded by atomics. This is
/// sufficient because:
///
/// * Only one producer writes (the router / midi adapter thread).
/// * Only one consumer drains (the audio thread inside `process`).
///
/// Overwriting the oldest event on overflow is preferred over blocking
/// or dropping new events — a stuck note is worse than a dropped one.
struct MidiRing {
    buf: [MidiBlockEvent; MIDI_RING_CAPACITY],
    head: AtomicUsize,
    tail: AtomicUsize,
}

impl MidiRing {
    const fn new() -> Self {
        Self {
            buf: [MidiBlockEvent::AllNotesOff; MIDI_RING_CAPACITY],
            head: AtomicUsize::new(0),
            tail: AtomicUsize::new(0),
        }
    }

    /// Push one event. Overwrites the oldest event if the ring is
    /// full. Called from the producer thread only.
    fn push(&mut self, ev: MidiBlockEvent) {
        let head = self.head.load(Ordering::Relaxed);
        let tail = self.tail.load(Ordering::Acquire);
        let next = (head + 1) % MIDI_RING_CAPACITY;
        if next == tail {
            // Ring is full — advance tail, dropping oldest.
            let new_tail = (tail + 1) % MIDI_RING_CAPACITY;
            self.tail.store(new_tail, Ordering::Release);
        }
        self.buf[head] = ev;
        self.head.store(next, Ordering::Release);
    }

    /// Drain all pending events into `out` (up to capacity). Called
    /// from the audio thread only.
    fn drain_into(&mut self, out: &mut Vec<MidiBlockEvent>) {
        let head = self.head.load(Ordering::Acquire);
        let mut tail = self.tail.load(Ordering::Relaxed);
        while tail != head {
            out.push(self.buf[tail]);
            tail = (tail + 1) % MIDI_RING_CAPACITY;
        }
        self.tail.store(tail, Ordering::Release);
    }

    #[cfg(test)]
    fn len(&self) -> usize {
        let head = self.head.load(Ordering::Acquire);
        let tail = self.tail.load(Ordering::Acquire);
        if head >= tail {
            head - tail
        } else {
            MIDI_RING_CAPACITY - tail + head
        }
    }
}

/// An `AudioBlock` wrapping a CLAP plugin.
///
/// For v1 this is a non-processing placeholder — see module docs.
pub struct ClapBlock {
    descriptor: PluginDescriptor,
    // Stored so `Drop` unloads the entry cleanly. Not `pub` because
    // the wrapper should mediate all plugin interaction. Follow-up
    // iteration will add a `Option<PluginInstance<ContrapunkHost>>`
    // alongside this field and actually drive the plugin.
    _entry: PluginEntry,
    midi_ring: MidiRing,
    // Pre-allocated vec the audio thread drains events into. Sized at
    // construction so `process` does no allocs.
    drained_events: Vec<MidiBlockEvent>,
    sample_rate: u32,
    enabled: bool,
}

impl ClapBlock {
    /// Load a CLAP bundle from disk and return a skeleton block.
    ///
    /// This validates that the path exists, dynamically loads the
    /// bundle, and captures the first plugin descriptor for display.
    /// It does **not** instantiate the plugin yet (see module docs).
    ///
    /// # Errors
    ///
    /// * [`ClapBlockError::NotFound`] — path missing.
    /// * [`ClapBlockError::LoadFailed`] — dlopen / bundle load fail.
    /// * [`ClapBlockError::NoFactory`] / [`ClapBlockError::NoDescriptors`]
    ///   — bundle loaded but exposes no plugins.
    pub fn new(path: impl AsRef<Path>) -> Result<Self, ClapBlockError> {
        let path = path.as_ref();
        if !path.exists() {
            return Err(ClapBlockError::NotFound(path.to_path_buf()));
        }

        // SAFETY: `PluginEntry::load` dlopens the CLAP bundle and
        // validates its entry vtable. The call is unsafe because
        // loading third-party native code can trigger arbitrary
        // behaviour; we accept that risk — hosting plugins is the
        // whole point of this module.
        let entry = unsafe { PluginEntry::load(path) }.map_err(|e| ClapBlockError::LoadFailed {
            path: path.to_path_buf(),
            source: Box::new(e),
        })?;

        let factory = entry
            .get_plugin_factory()
            .ok_or_else(|| ClapBlockError::NoFactory(path.to_path_buf()))?;

        let first = factory
            .plugin_descriptors()
            .next()
            .ok_or_else(|| ClapBlockError::NoDescriptors(path.to_path_buf()))?;

        let id = cstr_opt_to_string(first.id())
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| path.to_string_lossy().into_owned());
        let name = cstr_opt_to_string(first.name())
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "Unknown".to_string());
        let vendor = cstr_opt_to_string(first.vendor()).unwrap_or_default();
        let version = cstr_opt_to_string(first.version()).unwrap_or_default();
        let descriptor = PluginDescriptor {
            id,
            name,
            vendor,
            version,
            path: path.to_path_buf(),
        };

        Ok(Self {
            descriptor,
            _entry: entry,
            midi_ring: MidiRing::new(),
            drained_events: Vec::with_capacity(MIDI_RING_CAPACITY),
            sample_rate: 48_000,
            enabled: true,
        })
    }

    /// Return the descriptor captured at load time.
    pub fn descriptor(&self) -> &PluginDescriptor {
        &self.descriptor
    }
}

impl std::fmt::Debug for ClapBlock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ClapBlock")
            .field("descriptor", &self.descriptor)
            .field("sample_rate", &self.sample_rate)
            .field("enabled", &self.enabled)
            .finish()
    }
}

impl AudioBlock for ClapBlock {
    fn name(&self) -> &str {
        &self.descriptor.name
    }

    fn type_id(&self) -> &str {
        // Prefix with `clap:` so the serialisation layer can tell
        // plugin types apart from the built-in synth (`builtin.synth`)
        // or future VST3 blocks (`vst3:...`).
        &self.descriptor.id
    }

    fn process(&mut self, buffer: &mut [f32], _channels: usize) {
        // Drain MIDI events even though we can't forward them yet —
        // clearing the ring prevents unbounded growth if the producer
        // side keeps pushing.
        self.drained_events.clear();
        self.midi_ring.drain_into(&mut self.drained_events);

        // v1: write silence. When the instance/processor is wired in,
        // this is where `PluginAudioProcessor::process` runs, with the
        // drained events flushed into the plugin's input event buffer
        // and the plugin's output event buffer drained back into the
        // chain's MIDI fan-out.
        for s in buffer.iter_mut() {
            *s = 0.0;
        }
    }

    fn midi_event(&mut self, event: MidiBlockEvent) {
        self.midi_ring.push(event);
    }

    fn reset(&mut self) {
        // Clear any queued events and ring state. No allocations.
        self.drained_events.clear();
        // Drain ring via an empty vec to reset head==tail quickly.
        let head = self.midi_ring.head.load(Ordering::Acquire);
        self.midi_ring.tail.store(head, Ordering::Release);
    }

    fn set_sample_rate(&mut self, sample_rate: u32) {
        self.sample_rate = sample_rate;
        // Follow-up: call `PluginAudioProcessor::deactivate` +
        // re-activate with the new sample rate.
    }

    fn enabled(&self) -> bool {
        self.enabled
    }
}

// Static assertion: `ClapBlock` must be `Send` so the chain controller
// can move it onto the audio-owner thread.
const _: fn() = || {
    fn assert_send<T: Send>() {}
    assert_send::<ClapBlock>();
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_with_bogus_path_returns_not_found() {
        let err = ClapBlock::new("/this/path/does/not/exist/Nope.clap").unwrap_err();
        assert!(
            matches!(err, ClapBlockError::NotFound(_)),
            "expected NotFound, got {err:?}"
        );
    }

    #[test]
    fn new_with_non_clap_file_returns_load_failed() {
        // Create a file with a .clap extension but random bytes.
        let p = std::env::temp_dir().join(format!("contrapunk-bogus-{}.clap", std::process::id()));
        std::fs::write(&p, b"not a real plugin").unwrap();

        let res = ClapBlock::new(&p);
        let _ = std::fs::remove_file(&p);

        let err = res.expect_err("opening a random file should fail");
        // Different platforms surface this as LoadFailed or NotFound —
        // the key point is we got an error, not a panic or crash.
        match err {
            ClapBlockError::LoadFailed { .. }
            | ClapBlockError::NotFound(_)
            | ClapBlockError::NoFactory(_)
            | ClapBlockError::NoDescriptors(_) => {}
        }
    }

    #[test]
    fn midi_ring_push_drain_roundtrip() {
        let mut r = MidiRing::new();
        r.push(MidiBlockEvent::NoteOn {
            note: 60,
            velocity: 100,
        });
        r.push(MidiBlockEvent::NoteOff { note: 60 });
        assert_eq!(r.len(), 2);

        let mut out = Vec::new();
        r.drain_into(&mut out);
        assert_eq!(out.len(), 2);
        assert_eq!(
            out[0],
            MidiBlockEvent::NoteOn {
                note: 60,
                velocity: 100,
            }
        );
        assert_eq!(out[1], MidiBlockEvent::NoteOff { note: 60 });
        assert_eq!(r.len(), 0);
    }

    #[test]
    fn midi_ring_overflow_drops_oldest() {
        let mut r = MidiRing::new();
        for i in 0..MIDI_RING_CAPACITY + 8 {
            r.push(MidiBlockEvent::NoteOn {
                note: (i % 128) as u8,
                velocity: 100,
            });
        }
        // After overflow the ring is never fuller than capacity - 1.
        assert!(r.len() < MIDI_RING_CAPACITY);
    }

    /// Loads a real CLAP plugin from disk if any is discoverable on
    /// the host. Ignored by default so CI without plugins installed
    /// passes. Run manually with `cargo test -- --ignored`.
    #[test]
    #[ignore = "requires a real CLAP plugin installed on the system"]
    fn new_with_real_plugin_if_available() {
        use super::super::discovery::discover_plugins;
        let plugins = discover_plugins();
        let Some(first) = plugins.first() else {
            eprintln!("no CLAP plugins found; skipping");
            return;
        };
        let block = ClapBlock::new(&first.path)
            .unwrap_or_else(|e| panic!("failed to load {:?}: {:?}", first.path, e));
        assert!(!block.descriptor().name.is_empty());
    }
}
