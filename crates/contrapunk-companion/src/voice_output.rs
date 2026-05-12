//! Per-voice output destination. Hoisted from src-tauri/state.rs so
//! the Companion + Lane code can live in a shared crate consumed by
//! both the Tauri backend and the WASM build. The original Tauri
//! `crate::state::VoiceOutputTarget` is now a re-export from here.

use serde::{Deserialize, Serialize};

/// Per-voice output destination. Each engine-emitted voice (by index
/// 0..voice_count-1) can be routed independently.
///
/// Three explicit destinations only — no implicit "defer to global
/// routing_mode" fallback. Default is `Synth` so users get audio out
/// of the box without needing to add a MIDI port first.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum VoiceOutputTarget {
    /// Send to the internal synth only. Skip external MIDI for this voice.
    #[default]
    Synth,
    /// Send to a specific external MIDI port only. Skip the internal synth.
    MidiPort { port: usize },
    /// Skip both synth and external MIDI. Voice is silent.
    Off,
}
