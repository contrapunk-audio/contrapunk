//! Pipeline stage traits for the guitar→MIDI rewrite.
//!
//! See [issue #82](https://github.com/contrapunk-audio/contrapunk/issues/82)
//! for the design context. The legacy monolithic
//! [`crate::audio::guitar_input::GuitarInput`] composes onset detection,
//! pitch detection, octave correction, the note state machine, and event
//! emission into one struct with intermingled state. The rewrite splits
//! these into independent stages so each can be implemented, tested, and
//! A/B-toggled against its legacy equivalent in isolation.
//!
//! Each trait represents one stage in the linear data flow:
//!
//! ```text
//! audio buffer → [OnsetDetector] → onset bool
//!              → [PitchDetector] → PitchEstimate
//!              → [OctaveCorrector] → corrected PitchEstimate
//!              → [StateMachine] → StateTransition
//!              → [EventEmitter] → Vec<MidiEvent>
//! ```
//!
//! All stages are stateful and block-by-block: `process()` is called once
//! per audio block in capture order. `reset()` clears any accumulated
//! state — used on pipeline restart, MIDI panic, or user-driven config
//! change that invalidates history.
//!
//! These traits are intentionally narrow. The legacy code is *not*
//! decomposed in Phase 0; it continues to run as today. New
//! implementations land one stage at a time in subsequent phases, each
//! gated by a runtime config flag so it can be A/B tested live against
//! the legacy code.

use crate::guitar_input::{GuitarInputConfig, MidiEvent};

/// One audio block — a slice of f32 samples at the configured sample rate.
pub type AudioBlock<'a> = &'a [f32];

/// Result of pitch detection on a single block.
///
/// `frequency = None` means no clear pitch was detected (clarity below
/// threshold, signal too quiet, etc.). Implementations should not fabricate
/// a frequency under those conditions — downstream stages rely on `None` to
/// gate state transitions.
#[derive(Debug, Clone, Copy, Default)]
pub struct PitchEstimate {
    /// Estimated fundamental frequency in Hz, or `None` if no clear pitch.
    pub frequency: Option<f32>,
    /// Detector confidence in `[0.0, 1.0]`. 1.0 = perfectly periodic signal.
    pub clarity: f32,
    /// Cents from the nearest semitone, signed. Zero when `frequency` is `None`.
    pub cents: f32,
}

/// Onset detection — fires when a new note attack is heard in the block.
///
/// Implementations may use HFC, spectral flux, RMS rise, autocorrelation
/// drops, or any combination. The contract is just `bool per block`;
/// internal cooldown / refractory logic stays inside the implementation.
pub trait OnsetDetector: Send {
    /// Returns `true` if an onset is detected at this block.
    fn process(&mut self, block: AudioBlock<'_>) -> bool;
    /// Clear any accumulated state (running spectra, cooldown counters, ...).
    fn reset(&mut self);
}

/// Pitch detection — extracts the strongest fundamental and its confidence.
///
/// This is the *raw* detector output. Octave correction is a separate stage
/// downstream so detector implementations can be A/B'd without conflating
/// detector quality with octave-leak fixes.
pub trait PitchDetector: Send {
    fn process(&mut self, block: AudioBlock<'_>) -> PitchEstimate;
    fn reset(&mut self);
}

/// Octave correction — maps a raw pitch estimate to its most likely
/// fundamental octave, fixing 2nd/3rd-harmonic locks.
///
/// Receives the audio block alongside the raw estimate so harmonic-template
/// methods (Goertzel sums at f, 2f, 3f) can decide whether the detector
/// locked on a higher harmonic.
///
/// **Contract** (per #82): octave correction is *unconditional* — applied
/// on every onset, including the very first. No leak path for fresh-onset
/// 2nd/3rd-harmonic locks.
pub trait OctaveCorrector: Send {
    fn correct(&mut self, raw: PitchEstimate, block: AudioBlock<'_>) -> PitchEstimate;
    fn reset(&mut self);
}

/// Note states the state machine can be in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NoteState {
    Idle,
    Attack { since_block: u64 },
    Sustain { since_block: u64 },
    Release { since_block: u64 },
}

/// What the state machine wants the emitter to do this block.
///
/// `None` is the common case — the user is sustaining a note and nothing
/// changed. The transition variants fire only at the boundaries.
#[derive(Debug, Clone)]
pub enum StateTransition {
    None,
    StartNote { midi: u8, velocity: u8, cents: f32 },
    UpdatePitch { midi: u8, cents: f32 },
    EndNote { midi: u8 },
}

/// State machine — fuses onset + pitch + correction signals into note-level
/// decisions (start, update, end).
///
/// **Contract** (per #82): velocity floor is enforced at this stage. Below
/// the floor, no `StartNote` is emitted regardless of onset signal.
pub trait StateMachine: Send {
    fn process(
        &mut self,
        onset: bool,
        pitch: PitchEstimate,
        block: AudioBlock<'_>,
    ) -> StateTransition;
    fn current_state(&self) -> NoteState;
    fn reset(&mut self);
}

/// Event emitter — turns state transitions into `MidiEvent`s, applying gating
/// (initial-bend flag, continuous-bend flag, etc.) and channel routing.
///
/// **Contract** (per #82):
/// - Initial pitch bend is emitted only when `config` enables it.
/// - Continuous pitch bend is emitted only when its own flag is enabled.
/// - The two are separately gated.
/// - String-ID falls back to a single channel when calibration is absent.
pub trait EventEmitter: Send {
    fn emit(&mut self, transition: StateTransition, config: &GuitarInputConfig) -> Vec<MidiEvent>;
    fn reset(&mut self);
}
