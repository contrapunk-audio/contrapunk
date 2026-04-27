//! Sample-accurate transport / clock.
//!
//! The [`Transport`] primitive owns the canonical notion of musical time
//! for the running session. It is driven by an audio-output callback
//! (opened for this purpose even if no audio plays through it) which
//! increments a sample counter every block. All readers pull state via
//! lock-free atomics — the audio thread never blocks or takes locks.
//!
//! Derived values (beat position, total beats) are computed on demand
//! from `sample_pos`, `sample_rate`, and `bpm`. BPM changes apply from
//! the next callback forward; past sample positions are not rescaled.
//!
//! This module is the foundation for anything time-dependent in the
//! app: metronome clicks, humanization, scheduled delays, loop playback.
//! Those features are *consumers* of the clock, never its source.

pub mod clock;

pub use clock::{BeatCrossing, Transport};
