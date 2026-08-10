//! Companion — automated bandmate that plays alongside the user.
//!
//! The companion is a state machine that runs `Lane` impls in three
//! phases on each deterministic scheduler beat: Sense lanes write `WorldState`,
//! Mutate lanes write `HarmonyEngine`, Decide lanes emit `DispatchOp`s.
//! Six of the nine jam-pipeline weeks (looper, arp, drone, pad, beat
//! machine, motif transposer) fit cleanly as Lane impls.
//!
//! `WorldState`, the `Lane` trait, and the `Companion` orchestrator
//! host timed musical behavior. The input-only `LooperLane` in `loops`
//! owns a pure volatile capture/playback state machine; adapters route
//! its replay through an isolated arrangement runtime.
//!
//! See `.planning/jam-features-2026/01-companion-architecture.md` for
//! the full architecture, phase ordering rationale, and Lane catalog.

pub mod canon_lane;
pub mod counterpoint_lane;
pub mod lane;
pub mod loops;
pub mod orchestrator;
pub mod pattern_lane;
pub mod phrase;
pub mod voice_output;
pub mod world;

use crate::voice_output::VoiceOutputTarget;
pub use voice_output::VoiceOutputTarget as VoiceOutput;

pub use canon_lane::CanonLane;
#[allow(unused_imports)]
pub use counterpoint_lane::CounterpointLane;
pub use pattern_lane::PatternLane;
pub use phrase::{
    PhrasePhase, PhraseSnapshot, DEFAULT_PHRASE_GAP_BEATS, MAX_PHRASE_GAP_BEATS,
    MIN_PHRASE_GAP_BEATS,
};

#[allow(unused_imports)]
pub use lane::{
    EngineMutation, HoldMode, InputEvent, InputFilter, Lane, LaneOutput, LanePhase, LaneState,
    WorldWrite,
};
#[allow(unused_imports)]
pub use loops::{
    InputOrigin, LoopBuffer, LoopEvent, LoopMidiEvent, LoopPressOutcome, LoopState, LoopStatus,
    LoopStatusState, LooperLane, OriginMidiEvent, MAX_LOOP_BEATS, MAX_LOOP_EVENTS,
    MICROBEATS_PER_BEAT,
};
#[allow(unused_imports)]
pub use orchestrator::{
    BeatTickScheduler, Companion, CompanionInputResult, CompanionState, MAX_TICK_SLOTS_PER_CALL,
    TICK_QUANTUM_BEATS,
};
#[allow(unused_imports)]
pub use world::{ChordQuality, DetectedChord, HeldInput, HeldVoice, WorldState};

/// One unit of work emitted by `Companion::tick()` for the router
/// thread to execute. The router converts these into `dispatch_voice`
/// / `broadcast_note_off` calls — no decisions, just dispatch.
///
/// Phase 1 keeps the existing `VoiceOutputTarget`. The audio-graph
/// phase migrates this to `InstrumentId` per the arch doc § Lane
/// abstraction note.
#[allow(dead_code)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DispatchOp {
    NoteOn {
        target: VoiceOutputTarget,
        note: u8,
        velocity: u8,
        channel: u8,
    },
    NoteOff {
        target: VoiceOutputTarget,
        note: u8,
        channel: u8,
    },
    /// CC 123 broadcast on every external port. Used on companion
    /// stop, loop clear, and panic-replay overspray.
    AllNotesOff { ports: Vec<u8> },
}
