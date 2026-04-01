//! Audio capture, pitch detection, and guitar-to-MIDI conversion.
//!
//! This module provides the full pipeline for converting audio input
//! into MIDI notes with real-time performance:
//!
//! - `config` - User-adjustable thresholds and detection parameters
//! - `pitch` - Core pitch detection with onset-gated note tracking
//! - `profiles` - Named presets for different recording environments
//! - `guitar` - Guitar-specific calibration, string matching, and chord grouping
//! - `onset` - Pluck/onset detection via HFC, spectral flux, and running autocorrelation
//! - `detectors` - Alternative pitch detectors: BACF, AMDF, Goertzel filter bank
//! - `single_cycle` - Ultra-low-latency single-cycle pitch detection
//! - `buffer` - Dual-buffer analysis, overlap management, and ring buffers

pub mod buffer;
pub mod config;
pub mod detectors;
pub mod guitar;
pub mod guitar_input;
pub mod inference;
pub mod onset;
pub mod pitch;
pub mod profiles;
pub mod single_cycle;
pub mod test_signals;

// Core types
pub use config::MicConfig;
pub use pitch::{freq_to_midi, is_in_voice_range, DetectedPitch, NoteEvent, NoteTracker};
pub use profiles::{builtin_profiles, MicProfile};

// Onset detection
pub use onset::{PluckDetector, RunningAutocorrelation};

// Alternative detectors
pub use detectors::{AmdfDetector, BacfDetector, GoertzelBank};

// Single-cycle detection
pub use single_cycle::{Peak, PeakTracker, SingleCycleDetector, SingleCycleResult};

// Buffer management
pub use buffer::{BufferReady, DualBufferAnalyzer, OverlapManager, RingBuffer};

// Guitar-specific
pub use guitar::{
    AudioNormalizer, CalibrationSample, ChordEvent, GuitarCalibrationProfile,
    GuitarMatchConfig, GuitarPitchEvent, GuitarPitchMatcher, NormalizeResult,
    OnsetGrouper, StringCalibration, StringMatch,
};

// Guitar input DSP pipeline
pub use guitar_input::{
    DetectedNote, GuitarCalibration, GuitarInput, GuitarInputConfig, MidiEvent, StringProfile,
};
