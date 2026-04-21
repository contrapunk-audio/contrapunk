//! Built-in FX blocks for the audio chain.
//!
//! Each effect implements [`crate::chain::AudioBlock`] so it drops
//! into the chain after the synth. Parameters are stored in lock-free
//! atomic structs (same pattern as `SynthParams`) so UI commands can
//! tweak them without blocking the audio thread.
//!
//! Current set:
//!   - `reverb` — Freeverb (Schroeder/Moorer-style comb + allpass)
//!
//! Planned:
//!   - `delay` — tempo-syncable stereo delay
//!   - `eq` — 3-band peaking biquad
//!   - `compressor` — peak compressor with attack/release

pub mod delay;
pub mod reverb;

pub use delay::{Delay, DelayParams};
pub use reverb::{Reverb, ReverbParams};
