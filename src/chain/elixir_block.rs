//! Audio-chain wrapper around the Elixir DSP engine.
//!
//! Phase 21.A0 deliverable. Gated behind the `elixir-synth` feature so
//! the legacy `src/synth/` keeps shipping until A-Cut. Both feature
//! configurations must compile cleanly on every surface (CLI, Tauri,
//! WASM, plugin) — see `ELIXIR-PLAN.md` §3 for the cutover plan.
//!
//! A0 implementation: forward [`AudioBlock`] calls into
//! [`elixir_core::Engine`]. A1 wires `midi_event` through to the
//! engine's note routing so a single sine voice actually responds to
//! MIDI notes. Future phases (A2 polyphony, A3 modulation, A4 filter,
//! A5 FX bus, A6 spectral features) extend without touching this
//! wrapper's `AudioBlock` interface.

use super::block::{AudioBlock, MidiBlockEvent};
use elixir_core::Engine;

/// `AudioBlock` adapter for the Elixir engine.
pub struct ElixirSynthBlock {
    engine: Engine,
}

impl ElixirSynthBlock {
    /// Construct an Elixir synth block. The sample rate is also pushed
    /// into the engine via [`AudioBlock::set_sample_rate`] later, but we
    /// prepare here so a fresh block is immediately usable.
    pub fn new(sample_rate: u32) -> Self {
        let mut engine = Engine::new();
        engine.prepare(sample_rate, DEFAULT_MAX_BLOCK);
        Self { engine }
    }
}

/// Conservative scratch-buffer bound. Chain `process` calls deliver
/// however many frames the cpal device hands us; this only gates
/// internal scratch allocation when later phases need it.
const DEFAULT_MAX_BLOCK: usize = 2048;

impl AudioBlock for ElixirSynthBlock {
    fn name(&self) -> &str {
        "Elixir Synth"
    }

    fn type_id(&self) -> &str {
        "builtin.elixir-synth"
    }

    fn process(&mut self, buffer: &mut [f32], channels: usize) {
        self.engine.process(buffer, channels);
    }

    fn midi_event(&mut self, event: MidiBlockEvent) {
        match event {
            MidiBlockEvent::NoteOn { note, velocity } => self.engine.note_on(note, velocity),
            MidiBlockEvent::NoteOff { note } => self.engine.note_off(note),
            MidiBlockEvent::AllNotesOff => self.engine.all_notes_off(),
            MidiBlockEvent::SustainPedal { on } => self.engine.set_sustain_pedal(on),
        }
    }

    fn set_sample_rate(&mut self, sample_rate: u32) {
        self.engine.prepare(sample_rate, DEFAULT_MAX_BLOCK);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn block_identifies_itself() {
        let b = ElixirSynthBlock::new(48_000);
        assert_eq!(b.name(), "Elixir Synth");
        assert_eq!(b.type_id(), "builtin.elixir-synth");
    }

    #[test]
    fn block_renders_silence_when_idle() {
        let mut b = ElixirSynthBlock::new(48_000);
        let mut buf = [0.42f32; 64];
        b.process(&mut buf, 2);
        assert!(buf.iter().all(|s| *s == 0.0));
    }

    #[test]
    fn set_sample_rate_reconfigures_engine() {
        let mut b = ElixirSynthBlock::new(44_100);
        b.set_sample_rate(96_000);
        let mut buf = [1.0f32; 8];
        b.process(&mut buf, 2);
        assert!(buf.iter().all(|s| *s == 0.0));
    }

    #[test]
    fn note_on_produces_audio() {
        let mut b = ElixirSynthBlock::new(48_000);
        b.midi_event(MidiBlockEvent::NoteOn {
            note: 69,
            velocity: 100,
        });
        let mut buf = [0.0f32; 1024];
        b.process(&mut buf, 2);
        let peak = buf.iter().map(|s| s.abs()).fold(0.0f32, f32::max);
        assert!(peak > 0.0, "expected audio after NoteOn, got silence");
    }

    #[test]
    fn all_notes_off_silences_eventually() {
        let mut b = ElixirSynthBlock::new(48_000);
        b.midi_event(MidiBlockEvent::NoteOn {
            note: 60,
            velocity: 100,
        });
        b.midi_event(MidiBlockEvent::AllNotesOff);
        let mut buf = [0.0f32; 32_000];
        b.process(&mut buf, 2);
        // tail of the buffer should be silent after release completes
        let tail_peak = buf[buf.len() - 64..]
            .iter()
            .map(|s| s.abs())
            .fold(0.0f32, f32::max);
        assert!(tail_peak < 1e-3, "tail still ringing: peak={tail_peak}");
    }
}
