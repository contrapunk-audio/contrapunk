//! The [`AudioBlock`] trait: anything that lives inside the audio chain.

/// MIDI-style event delivered to a block's `midi_event` method.
///
/// Broader than [`crate::elixir::SynthEvent`] because plugin-host blocks
/// (CLAP, VST3) may need to forward CC / pitch bend / aftertouch too.
/// FX blocks typically ignore everything (default impl is no-op).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MidiBlockEvent {
    NoteOn {
        note: u8,
        velocity: u8,
    },
    NoteOff {
        note: u8,
    },
    AllNotesOff,
    /// CC 64 sustain pedal. `on=true` engages, `on=false` releases.
    /// Blocks that don't model sustain (legacy synth, FX) should ignore.
    SustainPedal {
        on: bool,
    },
    // Future: CC { controller: u8, value: u8 }, PitchBend { value: i16 }, etc.
}

/// An audio-processing node in the chain.
///
/// # Contract
///
/// - `process` is called once per audio buffer from the cpal callback.
///   The block reads from and writes to the same interleaved buffer.
///   Real-time-safe: no allocations, no locks, no blocking I/O.
/// - `midi_event` is called zero or more times per buffer, before
///   `process`, when the chain receives a MIDI event. Default is no-op.
/// - `reset` is called when the chain needs a clean state (e.g. after
///   a panic or sample-rate change).
/// - `set_sample_rate` is called once at construction with the device's
///   actual rate, and again if the device changes.
///
/// # Serialization
///
/// A block identifies itself via [`type_id`](Self::type_id) (stable
/// string) so the rig system can re-instantiate blocks by type. The
/// block is responsible for exposing its own save/restore API.
pub trait AudioBlock: Send {
    /// Human-readable name shown in the UI ("Synth", "Reverb", "Diva").
    fn name(&self) -> &str;

    /// Stable type ID for rig persistence. Examples:
    /// - `"builtin.synth"` for the built-in Elixir sine synth
    /// - `"builtin.reverb"` for the built-in reverb
    /// - `"clap.com.u-he.Diva"` for a CLAP plugin
    fn type_id(&self) -> &str;

    /// Process one buffer in-place. The buffer is interleaved stereo
    /// (or however many `channels` the device has). `frames` is
    /// `buffer.len() / channels`.
    fn process(&mut self, buffer: &mut [f32], channels: usize);

    /// Deliver a MIDI event to the block. Default: ignore. Synth-like
    /// blocks override to handle notes.
    fn midi_event(&mut self, _event: MidiBlockEvent) {}

    /// Reset all internal DSP state (envelopes, filter memory, etc.).
    fn reset(&mut self) {}

    /// Update the sample rate. Called at least once at construction.
    fn set_sample_rate(&mut self, _sample_rate: u32) {}

    /// Whether the block is currently enabled. When false, `process`
    /// should be a pass-through (or silence for source blocks). Chains
    /// still call `process` so the block can maintain internal state;
    /// it's the block's job to honour the flag.
    fn enabled(&self) -> bool {
        true
    }
}
