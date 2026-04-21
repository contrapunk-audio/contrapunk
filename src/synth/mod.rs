//! Built-in polyphonic synth for Tier 1 audio output.
//!
//! Routes MIDI events from the harmony engine's router thread into an
//! audio callback, renders audio via a voice pool, and mixes the result
//! into the cpal output stream alongside the metronome click.
//!
//! # Design
//!
//! - [`SynthParams`] holds all runtime-tweakable parameters as atomics
//!   so UI commands can update them from any thread without blocking
//!   the audio callback.
//! - [`Synth`] owns an 8-voice pool, an `mpsc` receiver for MIDI events,
//!   and a reference to the shared params. `render` is called once per
//!   audio callback.
//! - Voices use a sine/saw/square/triangle oscillator, a one-pole
//!   low-pass filter, and an ADSR envelope. Mono output is duplicated
//!   to all output channels at mix time.
//!
//! This is intentionally minimal — enough to hear the harmony engine
//! without configuring IAC + an external plugin. Future FX (reverb,
//! delay, EQ, compressor) hook in after `Synth::render` in the audio
//! callback.

pub mod params;
pub mod voice;

pub use params::{SynthEvent, SynthParams, Waveform};
pub use voice::Synth;
