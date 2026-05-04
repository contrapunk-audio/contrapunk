//! Companion — automated bandmate that plays alongside the user.
//!
//! Owns the pattern programmer (rhythmic gate on held inputs), loop slots
//! (input/output recordings, multiple parallel), and auto-key state. The
//! router thread pulls dispatch ops from `Companion::tick()` each iteration
//! and executes them via `dispatch_voice` — it does not embed companion-
//! specific decision logic of its own.
//!
//! Build phase: this module currently contains only type scaffolding.
//! Behavior is still in `state.rs` (PatternConfig) and `commands/engine.rs`
//! (router loop). Subsequent increments move the algorithms in.
//!
//! See `.planning/jam-features-2026/01-companion-architecture.md` for the
//! full architecture and phase plan.

use crate::state::VoiceOutputTarget;

pub mod loops;

// Re-exports — used as the public companion surface. Allow dead_code
// during the multi-increment migration; subsequent phases consume these.
#[allow(unused_imports)]
pub use loops::{LoopBuffer, LoopEvent, LoopEventKind, LoopSlot, LoopSource, LoopState};

/// One unit of work emitted by `Companion::tick()` for the router thread
/// to execute. The router converts these into `dispatch_voice` /
/// `broadcast_note_off` calls — no decisions, just dispatch.
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
    /// CC 123 broadcast on every external port. Used on companion stop,
    /// loop clear, and panic-replay overspray.
    AllNotesOff { ports: Vec<u8> },
}
