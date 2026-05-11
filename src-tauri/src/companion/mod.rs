//! Companion — automated bandmate that plays alongside the user.
//!
//! The companion is a state machine that runs `Lane` impls in three
//! phases each router-loop iteration: Sense lanes write `WorldState`,
//! Mutate lanes write `HarmonyEngine`, Decide lanes emit `DispatchOp`s.
//! Six of the nine jam-pipeline weeks (looper, arp, drone, pad, beat
//! machine, motif transposer) fit cleanly as Lane impls.
//!
//! Phase 1.4 ships the abstraction: `WorldState`, the `Lane` trait,
//! and the `Companion` orchestrator with no concrete Lanes. The
//! looper slot types in `loops` are scaffolding for Phase 2's
//! `LooperLane`, the first concrete Lane.
//!
//! See `.planning/jam-features-2026/01-companion-architecture.md` for
//! the full architecture, phase ordering rationale, and Lane catalog.

use crate::state::VoiceOutputTarget;

pub mod canon_lane;
pub mod lane;
pub mod loops;
pub mod orchestrator;
pub mod world;

pub use canon_lane::CanonLane;

#[allow(unused_imports)]
pub use lane::{
    EngineMutation, InputEvent, InputFilter, Lane, LaneOutput, LanePhase, LaneState, WorldWrite,
};
#[allow(unused_imports)]
pub use loops::{LoopBuffer, LoopEvent, LoopEventKind, LoopSlot, LoopSource, LoopState};
#[allow(unused_imports)]
pub use orchestrator::{Companion, CompanionInputResult, CompanionState};
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
