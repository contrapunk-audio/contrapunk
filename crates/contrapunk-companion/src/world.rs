//! WorldState — the observable Lanes read and write.
//!
//! The companion's three-phase orchestration (Sense → Mutate → Decide)
//! revolves around a single shared `WorldState`. Sense lanes write into
//! it (auto-key, chord-detect), Mutate lanes read it to decide engine
//! mutations, Decide lanes read it to emit dispatch ops. Each tick sees
//! a coherent snapshot.
//!
//! Phase 1.4 ships the minimum surface — `transport`, `engine_snapshot`,
//! `held_inputs`, `sounding_voices`, `current_chord`. `recent_input_window`
//! is intentionally deferred until the AutoKeyLane rewrite needs it
//! (before Wk 6) per the arch doc § "WorldState — the observable".

#![allow(dead_code)]

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use contrapunk_harmony::HarmonyEngine;
use contrapunk_transport::Transport;

use crate::phrase::{PhraseContext, PhraseSnapshot};
use crate::voice_output::VoiceOutputTarget;

/// One currently-held input note. Populated on `Companion::on_input`
/// note-on, removed on note-off. The router consults `held_inputs` for
/// auto-key analysis and any Lane that needs the live input snapshot.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HeldInput {
    pub note: u8,
    pub velocity: u8,
    pub channel: u8,
    pub pressed_at: Instant,
}

/// One currently-sounding harmony voice. The router populates
/// `sounding_voices` after default-harmonize so Sense lanes
/// (chord-detect) and Decide lanes (motif transposer, pad, drone) can
/// observe what's actually playing.
///
/// `target` records where the voice was dispatched so a future
/// `panic` / `AllNotesOff` knows which surface to clear. Phase 1 keeps
/// the existing `VoiceOutputTarget`; the audio-graph Phase migrates
/// this to `InstrumentId` per arch doc § Lane abstraction note.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HeldVoice {
    pub note: u8,
    pub channel: u8,
    pub target: VoiceOutputTarget,
}

/// Common chord qualities used by `DetectedChord`. Ordered roughly by
/// frequency in tonal music; the `ChordDetectLane` (Sense, future
/// phase) populates this. The existing `contrapunk::chord` crate
/// returns a display string — quality classification is layered on top
/// when needed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChordQuality {
    Major,
    Minor,
    Diminished,
    Augmented,
    Dominant7,
    Major7,
    Minor7,
    HalfDiminished7,
    Diminished7,
    Sus2,
    Sus4,
    Other,
}

/// Snapshot of the current detected chord. Powers Wk 6 motif
/// transpose, Wk 7 arp, Wk 9 pad. `display` is always populated when
/// a chord is detected; `root` and `quality` are best-effort.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DetectedChord {
    pub root: Option<u8>,
    pub quality: Option<ChordQuality>,
    pub display: String,
}

/// The observable shared across the three Lane phases.
///
/// All interior fields are wrapped in `Arc<Mutex<…>>` rather than
/// `Arc<…>` alone because router-thread Sense writes (and main-thread
/// UI display reads) need to mutate / observe in tandem. Per the arch
/// doc § "Cross-cutting audio concerns / Thread responsibilities":
/// the audio thread NEVER touches `WorldState`. Only router and main.
///
/// Phase 1 deliberately keeps `engine_snapshot` as `Arc<Mutex<…>>`
/// rather than `ArcSwap`. Profiling will tell us when the read-mostly
/// pattern justifies the swap.
pub struct WorldState {
    /// Sample-accurate musical time. The `Transport` uses lock-free
    /// atomics internally, so handing out the `Arc` directly is safe.
    pub transport: Arc<Transport>,

    /// Live `HarmonyEngine` — read by Sense lanes, written by Mutate
    /// lanes through the orchestrator. Same Arc as `AppState.engine`
    /// so all three threads see the same instance.
    pub engine_snapshot: Arc<Mutex<HarmonyEngine>>,

    /// Currently-held inputs, keyed by MIDI note number. The
    /// orchestrator's `on_input` handler updates this before
    /// dispatching to Lanes so every Lane sees the post-event world.
    pub held_inputs: Arc<Mutex<HashMap<u8, HeldInput>>>,

    /// Currently-sounding harmony voices, keyed by *input* note. One
    /// input note can produce multiple voices (the harmony fan-out);
    /// each voice records its dispatch target so Sense lanes know
    /// which output surface to inspect.
    pub sounding_voices: Arc<Mutex<HashMap<u8, Vec<HeldVoice>>>>,

    /// Best-effort chord detection from `sounding_voices`. Default
    /// (no chord) is `DetectedChord::default()` — empty `display`.
    pub current_chord: Arc<Mutex<DetectedChord>>,

    /// Companion's global default HoldMode for pending-emission
    /// cancellation. Lanes read this on NoteOff to resolve the
    /// effective mode when their own (and the voice's) override is
    /// `None`. The `Companion` writes to it via
    /// `set_global_hold_mode`. Shared via `Arc<Mutex>` rather than
    /// `Arc<AtomicX>` because `HoldMode` is an enum with payload,
    /// not a primitive.
    pub global_hold_mode: Arc<Mutex<crate::lane::HoldMode>>,

    /// Canonical phrase/input lifecycle. Companion ingress updates this
    /// exactly once before lane fan-out; Decide lanes read snapshots.
    phrase: Arc<Mutex<PhraseContext>>,
}

impl WorldState {
    /// Construct a new `WorldState` sharing the given transport and
    /// engine. The other fields start empty.
    pub fn new(transport: Arc<Transport>, engine: Arc<Mutex<HarmonyEngine>>) -> Arc<Self> {
        Arc::new(Self {
            transport,
            engine_snapshot: engine,
            held_inputs: Arc::new(Mutex::new(HashMap::new())),
            sounding_voices: Arc::new(Mutex::new(HashMap::new())),
            current_chord: Arc::new(Mutex::new(DetectedChord::default())),
            global_hold_mode: Arc::new(Mutex::new(crate::lane::HoldMode::default())),
            phrase: Arc::new(Mutex::new(PhraseContext::new())),
        })
    }

    pub fn observe_phrase_input(&self, event: crate::lane::InputEvent) {
        let now = self.transport.total_beats();
        self.phrase
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .observe(event, now);
    }

    pub fn advance_phrase(&self) {
        let now = self.transport.total_beats();
        self.phrase
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .advance(now);
    }

    pub fn reset_phrase_runtime(&self) {
        self.phrase
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .reset_runtime();
    }

    pub fn set_phrase_gap_beats(&self, beats: f64) -> Result<(), String> {
        self.phrase
            .lock()
            .map_err(|error| error.to_string())?
            .set_gap_beats(beats)
    }

    pub fn phrase_snapshot(&self) -> PhraseSnapshot {
        self.phrase
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .snapshot()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use contrapunk_harmony::{HarmonyMode, Key};

    fn fixture() -> Arc<WorldState> {
        let transport = Transport::new(48_000);
        let engine = Arc::new(Mutex::new(HarmonyEngine::new(
            Key::C,
            HarmonyMode::PassThrough,
        )));
        WorldState::new(transport, engine)
    }

    #[test]
    fn fresh_world_is_empty() {
        let w = fixture();
        assert!(w.held_inputs.lock().unwrap().is_empty());
        assert!(w.sounding_voices.lock().unwrap().is_empty());
        assert_eq!(*w.current_chord.lock().unwrap(), DetectedChord::default());
    }

    #[test]
    fn engine_handle_is_shared() {
        let w = fixture();
        // Mutate through one handle; observe through `engine_snapshot`.
        w.engine_snapshot.lock().unwrap().set_key(Key::G);
        assert_eq!(w.engine_snapshot.lock().unwrap().key(), Key::G);
    }

    #[test]
    fn held_input_round_trip() {
        let w = fixture();
        let entry = HeldInput {
            note: 60,
            velocity: 100,
            channel: 0,
            pressed_at: Instant::now(),
        };
        w.held_inputs.lock().unwrap().insert(60, entry);
        assert_eq!(w.held_inputs.lock().unwrap().get(&60), Some(&entry));
    }
}
