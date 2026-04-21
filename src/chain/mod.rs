//! Audio-processing chain: a linear pipeline of `AudioBlock`s that
//! runs inside the cpal callback each buffer.
//!
//! # Design
//!
//! - [`AudioBlock`] — trait implemented by anything that processes
//!   audio in-place (synths, effects, plugin hosts). Also receives
//!   MIDI events so synth-like blocks can play notes.
//! - [`Chain`] — owns a `Vec<Box<dyn AudioBlock>>` and processes them
//!   in order. First block typically takes MIDI in and produces audio
//!   (e.g. the built-in synth). Later blocks filter/shape the audio
//!   (reverb, delay, EQ, plugin FX).
//!
//! # Threading
//!
//! The chain lives inside the audio-owner thread. Parameter changes
//! come in through each block's own lock-free param store (e.g.
//! `SynthParams`). Block add/remove would go through an SPSC command
//! queue (future work — v1 hard-codes the chain layout at startup).
//!
//! # Extension points
//!
//! - New effect block: implement `AudioBlock`, register in the chain.
//! - Plugin host block (CLAP / VST3): implement `AudioBlock` with the
//!   plugin's `process` as the body. Exposes plugin params via its own
//!   shared-atomic store.
//! - Rig serialization: each block returns a `type_id()` + JSON of its
//!   params. Rig loader instantiates blocks by `type_id` and applies
//!   the params.

pub mod block;
pub mod chain;

pub use block::{AudioBlock, MidiBlockEvent};
pub use chain::Chain;
