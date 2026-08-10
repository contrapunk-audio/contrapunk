//! Per-voice output destination. Hoisted from src-tauri/state.rs so
//! the Companion + Lane code can live in a shared crate consumed by
//! both the Tauri backend and the WASM build. The original Tauri
//! `crate::state::VoiceOutputTarget` is now a re-export from here.

use serde::{Deserialize, Serialize};

/// Per-part output destination. Surfaces assign stable musical roles to these
/// targets rather than deriving identity from a generated pitch.
///
/// Three explicit destinations only — no implicit "defer to global
/// routing_mode" fallback. Default is `Synth` so users get audio out
/// of the box without needing to add a MIDI port first.
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum VoiceOutputTarget {
    /// Send to the internal synth only. Skip external MIDI for this voice.
    #[default]
    Synth,
    /// Send to a surface-defined MIDI port ID only. Tauri uses the system MIDI
    /// device index, which stays independent of connection-pool ordering.
    MidiPort { port: usize },
    /// Skip both synth and external MIDI. Voice is silent.
    Off,
}
