//! Harmony engine for MIDI note transformation.
//!
//! This module provides the core harmony generation functionality for Contrapunk,
//! transforming incoming MIDI notes into multi-voice harmonies using configurable
//! algorithms and scales.
//!
//! # Overview
//!
//! The harmony system consists of:
//!
//! - **[`Scale`]** - Defines a musical scale (tonic + mode) with diatonic operations
//! - **[`HarmonyMode`]** - One of 8 algorithms for generating harmony notes
//! - **[`HarmonyEngine`]** - Orchestrates note processing through the selected mode
//! - **[`VoiceLeadingStyle`]** - Post-processing for smooth voice transitions
//!
//! # Architecture
//!
//! ```text
//! MIDI Note In --> HarmonyEngine --> Mode Algorithm --> Voice Leading --> MIDI Notes Out
//!                       |                  |                  |
//!                   [Scale]          [Stateful/           [Revoicing]
//!                   [Key]             Stateless]
//! ```
//!
//! # Harmony Modes
//!
//! The engine supports 8 harmony modes, each implementing a different algorithm:
//!
//! | Mode | Name | Description |
//! |------|------|-------------|
//! | 1 | PassThrough | No harmony, notes pass through unchanged |
//! | 2 | DiatonicThirds | Parallel thirds (+2 scale degrees per voice) |
//! | 3 | DiatonicFourths | Parallel fourths (+3 scale degrees per voice) |
//! | 4 | RandomBelow | Random diatonic interval below (2nd-7th) |
//! | 5 | RandomBelowNoSeconds | Random below excluding dissonant 2nds |
//! | 6 | ContraryMotion | Harmony moves opposite to melody direction |
//! | 7 | StrictCounterpoint | Note-against-note voice leading with partial Species 1 rules |
//!
//! ## Stateless vs Stateful Modes
//!
//! - **Stateless (1-5):** Each note is processed independently
//! - **Stateful (6-7):** Track previous notes to determine harmony direction
//!
//! # Scale Families
//!
//! The system supports 57 scale modes organized into 10 families:
//!
//! | Family | Count | Notes |
//! |--------|-------|-------|
//! | Diatonic | 7 | Ionian through Locrian |
//! | Harmonic Minor | 7 | HarmonicMinor and rotations |
//! | Melodic Minor | 7 | MelodicMinor and rotations |
//! | Harmonic Major | 7 | HarmonicMajor and rotations |
//! | Double Harmonic | 7 | DoubleHarmonic and rotations |
//! | Pentatonic | 8 | Major/Minor pentatonic, Hirajoshi, InSen, Iwato, Yo, Kumoi, Pelog |
//! | Blues & Bebop | 3 | MinorBlues, MajorBlues, BebopDominant |
//! | Symmetric | 4 | WholeTone, Diminished (2 forms), Augmented |
//! | World | 5 | Enigmatic, Neapolitan, Persian, Hungarian Major |
//! | Barry Harris | 2 | BHMajor6thDim, BHMinor6thDim (8-note scales) |
//!
//! # Chord/Note Evaluation Algorithm
//!
//! The harmony engine uses a multi-stage evaluation process to determine which
//! notes to generate:
//!
//! ## 1. Scale Membership Check
//!
//! When a note arrives, the engine first checks if it belongs to the current scale
//! using `Scale::is_in_scale()`. This determines the harmonization path:
//!
//! - **In-scale notes:** Use diatonic transposition (move by scale degrees)
//! - **Out-of-scale notes:** Use modal interchange or chromatic consonant intervals
//!
//! ## 2. Diatonic Transposition
//!
//! For in-scale notes, `Scale::transpose_diatonic()` moves by scale degrees:
//!
//! ```text
//! C4 + 2 degrees in C Major = E4 (major 3rd)
//! E4 + 2 degrees in C Major = G4 (minor 3rd)
//! ```
//!
//! The interval size in semitones varies based on position in the scale.
//!
//! ## 3. Modal Interchange (Optional)
//!
//! When enabled, out-of-scale notes search parallel modes for borrowing:
//!
//! ```text
//! Eb4 in C Ionian -> Not in scale
//!                 -> Search C Aeolian (has Eb) -> Borrow harmonization
//!                 -> Track borrowed mode for UI display
//! ```
//!
//! Borrowing range (1-5) controls how many parallel modes to search.
//!
//! ## 4. Stateful Mode Evaluation
//!
//! Modes 6 and 7 use sliding window history via `VecDeque` for context-aware
//! harmony generation:
//!
//! ### Contrary Motion (Mode 6)
//!
//! Tracks `last_melody` and `last_harmony` to determine direction:
//!
//! ```text
//! Melody: C4 -> E4 (ascending)
//! Harmony: A3 -> G3 (descending - contrary motion)
//! ```
//!
//! When melody repeats, harmony alternates direction (oblique motion).
//!
//! ### Strict Counterpoint (Mode 7)
//!
//! Uses multiple sliding windows for sophisticated evaluation:
//!
//! - **`interval_history: VecDeque<i8>`** (size 4) - Tracks last 4 interval types
//!   to penalize overused intervals and reward variety
//! - **`melody_contour: VecDeque<MelodicDirection>`** (size 3) - Tracks melodic
//!   direction to encourage contrary motion
//! - **`harmony_range_low/high`** - Tracks harmony voice range to encourage
//!   expansion when too narrow
//!
//! The scoring system evaluates each candidate harmony:
//!
//! | Factor | Score Impact |
//! |--------|--------------|
//! | Parallel perfect 5ths/octaves | -100 (reject) |
//! | Static harmony when melody repeats | -100 (reject) |
//! | Different note from previous | +3 |
//! | Stepwise motion (1-2 semitones) | +4 |
//! | Small leap (3-4 semitones) | +2 |
//! | Interval overused (3+ in last 4) | -3 |
//! | Fresh interval (not in history) | +2 |
//! | Contrary motion to melody trend | +3 |
//! | Parallel motion to melody trend | -1 |
//! | Large leap when range narrow | +2 |
//! | 3rds and 6ths (vs 4ths/5ths) | +1 |
//!
//! ## 5. Voice Position and Chaining
//!
//! The engine supports multi-voice output with configurable voice position:
//!
//! ```text
//! 4-voice with position=2 (tenor):
//!
//! Soprano [0]  <-- harmony chain above
//! Alto    [1]  <-- harmony chain above
//! Tenor   [2]  <-- USER INPUT (voice_position)
//! Bass    [3]  <-- harmony chain below
//! ```
//!
//! Each voice is chained from the previous: harm2 = harmony_of(harm1).
//!
//! ## 6. Octave Mode Post-Processing
//!
//! After harmony generation, octave transformations are applied:
//!
//! - **None:** No modification
//! - **Spread:** Each voice +1 octave from previous
//! - **BassTrebleSplit:** Below melody -1 oct, above +1 oct
//! - **Mirror:** Duplicate each harmony at +1 and -1 octave (tripling)
//!
//! # Example Usage
//!
//! ```ignore
//! use contrapunk::harmony::{HarmonyEngine, Key, HarmonyMode, ScaleMode};
//! use wmidi::Note;
//!
//! // Create engine in C major with diatonic thirds
//! let mut engine = HarmonyEngine::new(Key::C, HarmonyMode::DiatonicThirds);
//!
//! // Harmonize a note
//! let result = engine.harmonize(Note::C4);
//! // result = [C4, E4] (melody + third above)
//!
//! // For MIDI routing, use note_on/note_off tracking:
//! let notes_on = engine.harmonize_note_on(Note::C4);   // [C4, E4]
//! let notes_off = engine.harmonize_note_off(Note::C4); // [C4, E4] (same notes)
//! ```
//!
//! # Re-exports
//!
//! This module re-exports the main types needed for harmony generation:
//!
//! - [`Key`] - Musical keys (C through B)
//! - [`HarmonyMode`] - Harmony algorithms
//! - [`OctaveMode`] - Octave transformation modes
//! - [`ScaleFamily`] - Scale family groupings
//! - [`ScaleMode`] - All 57 scale modes
//! - [`HarmonyEngine`] - Main engine struct
//! - [`Scale`] - Scale operations
//! - [`ContraryMotionState`], [`CounterpointState`] - Stateful mode state machines
//! - [`VoiceLeadingStyle`] - Voice leading post-processor styles

pub mod barry_harris;
mod config;
mod engine;
pub mod functional;
mod key_detect;
mod modes;
mod scale;
mod stateful;
pub mod voice_leading;

pub use barry_harris::{BhScaleGuard, Parity};
pub use config::{BeatPhase, HarmonyMode, Key, OctaveMode, RoutingMode, ScaleFamily, ScaleMode};
pub use engine::HarmonyEngine;
pub use key_detect::KeyDetector;
pub use scale::Scale;
pub use stateful::{
    ContraryMotionState, CounterpointSpecies, CounterpointState, CounterpointStrictness,
};
pub use voice_leading::VoiceLeadingStyle;
