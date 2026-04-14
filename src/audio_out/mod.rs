//! Native audio output for Contrapunk.
//!
//! Drives a cpal output stream with a polyphonic sine synth, fed by the
//! harmony engine via a lock-free SPSC ringbuffer. The audio thread is
//! real-time safe — no allocations, no locks.
//!
//! The MIDI-out path (IAC, external synths) continues to run in parallel;
//! audio output is additive, not a replacement.
//!
//! Sub-project 1 of plugin hosting. VST3 plugin loading is sub-project 2.

pub mod config;
pub mod engine;
pub mod midi_queue;
pub mod sine_synth;

// TODO: uncomment as types land in Tasks 2-6
// pub use config::AudioConfig;
// pub use engine::AudioOutEngine;
// pub use midi_queue::{MidiConsumer, MidiEvent, MidiProducer, midi_queue};
// pub use sine_synth::{PolySynth, SineVoice};
