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

use std::sync::{mpsc, Arc};

use super::block::{AudioBlock, MidiBlockEvent};
use crate::synth::{SynthEvent, SynthParams, Waveform};
use elixir_core::osc::{PhaseDistortionMode, SpectralMorph};
use elixir_core::Engine;

/// `AudioBlock` adapter for the Elixir engine.
pub struct ElixirSynthBlock {
    engine: Engine,
    params: Arc<SynthParams>,
    events: mpsc::Receiver<SynthEvent>,
}

impl ElixirSynthBlock {
    /// Construct an Elixir synth block. The sample rate is also pushed
    /// into the engine via [`AudioBlock::set_sample_rate`] later, but we
    /// prepare here so a fresh block is immediately usable.
    pub fn new(sample_rate: u32) -> Self {
        let (_tx, rx) = mpsc::channel();
        Self::new_with_params_and_events(sample_rate, Arc::new(SynthParams::new()), rx)
    }

    /// Construct an Elixir synth block wired to the existing router →
    /// audio-thread event channel. This is the A-Cut bridge: router code
    /// can keep sending [`SynthEvent`] while the chain swaps the first
    /// block from legacy [`crate::synth::Synth`] to Elixir.
    pub fn new_with_events(sample_rate: u32, events: mpsc::Receiver<SynthEvent>) -> Self {
        Self::new_with_params_and_events(sample_rate, Arc::new(SynthParams::new()), events)
    }

    /// Construct an Elixir synth block sharing the same public synth
    /// params used by the legacy built-in synth. This keeps existing UI
    /// commands meaningful under the feature flag while A6-specific
    /// controls are introduced separately.
    pub fn new_with_params_and_events(
        sample_rate: u32,
        params: Arc<SynthParams>,
        events: mpsc::Receiver<SynthEvent>,
    ) -> Self {
        let mut engine = Engine::new();
        engine.prepare(sample_rate, DEFAULT_MAX_BLOCK);
        Self {
            engine,
            params,
            events,
        }
    }

    fn drain_events(&mut self) {
        while let Ok(ev) = self.events.try_recv() {
            match ev {
                SynthEvent::NoteOn { note, velocity } => self.engine.note_on(note, velocity),
                SynthEvent::NoteOff { note } => self.engine.note_off(note),
                SynthEvent::AllNotesOff => self.engine.all_notes_off(),
            }
        }
    }

    /// Set the voice-filter cutoff in Hz (A4). Pass-through to the
    /// underlying engine.
    pub fn set_filter_cutoff_hz(&mut self, hz: f32) {
        self.engine.set_filter_cutoff_hz(hz);
    }
    /// Set the voice-filter resonance, `0..1`.
    pub fn set_filter_resonance(&mut self, r: f32) {
        self.engine.set_filter_resonance(r);
    }

    fn apply_params(&mut self, buffer: &mut [f32]) -> bool {
        if !self.params.enabled() {
            self.engine.all_notes_off();
            for s in buffer.iter_mut() {
                *s = 0.0;
            }
            return false;
        }

        self.engine.set_amp_attack_secs(self.params.attack_secs());
        self.engine.set_amp_decay_secs(self.params.decay_secs());
        self.engine.set_amp_sustain(self.params.sustain_level());
        self.engine.set_amp_release_secs(self.params.release_secs());
        self.engine.set_filter_cutoff_hz(self.params.cutoff_hz());
        self.engine.set_filter_resonance(self.params.resonance());
        self.engine.set_master_gain(self.params.master_gain());

        // Compatibility mapping for the existing four-shape UI. These
        // are musical approximations until the Elixir-specific UI owns
        // the full oscillator surface end-to-end in Contrapunk.
        match self.params.waveform() {
            Waveform::Sine => {
                self.engine.set_spectral_morph(SpectralMorph::Passthrough);
                self.engine.set_morph_amount(0.0);
                self.engine.set_phase_distortion(PhaseDistortionMode::Off);
                self.engine.set_phase_amount(0.0);
            }
            Waveform::Saw => {
                self.engine.set_spectral_morph(SpectralMorph::HarmonicScale);
                self.engine.set_morph_amount(1.0);
                self.engine.set_phase_distortion(PhaseDistortionMode::Off);
                self.engine.set_phase_amount(0.0);
            }
            Waveform::Square => {
                self.engine.set_spectral_morph(SpectralMorph::HighPass);
                self.engine.set_morph_amount(0.75);
                self.engine
                    .set_phase_distortion(PhaseDistortionMode::PulseWidth);
                self.engine.set_phase_amount(0.5);
            }
            Waveform::Triangle => {
                self.engine.set_spectral_morph(SpectralMorph::LowPass);
                self.engine.set_morph_amount(0.9);
                self.engine.set_phase_distortion(PhaseDistortionMode::Bend);
                self.engine.set_phase_amount(0.35);
            }
        }
        true
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
        self.drain_events();
        if self.apply_params(buffer) {
            self.engine.process(buffer, channels);
        }
    }

    fn midi_event(&mut self, event: MidiBlockEvent) {
        match event {
            MidiBlockEvent::NoteOn { note, velocity } => self.engine.note_on(note, velocity),
            MidiBlockEvent::NoteOff { note } => self.engine.note_off(note),
            MidiBlockEvent::AllNotesOff => self.engine.all_notes_off(),
            MidiBlockEvent::SustainPedal { on } => self.engine.set_sustain_pedal(on),
        }
    }

    fn reset(&mut self) {
        self.engine.all_notes_off();
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
    fn shared_synth_params_can_mute_elixir() {
        let params = Arc::new(SynthParams::new());
        params.set_enabled(false);
        let (tx, rx) = mpsc::channel();
        let mut b = ElixirSynthBlock::new_with_params_and_events(48_000, params, rx);
        tx.send(SynthEvent::NoteOn {
            note: 69,
            velocity: 100,
        })
        .unwrap();
        let mut buf = [0.5f32; 1024];
        b.process(&mut buf, 2);
        assert!(buf.iter().all(|s| *s == 0.0));
    }

    #[test]
    fn synth_event_receiver_produces_audio() {
        let (tx, rx) = mpsc::channel();
        let mut b = ElixirSynthBlock::new_with_events(48_000, rx);
        tx.send(SynthEvent::NoteOn {
            note: 69,
            velocity: 100,
        })
        .unwrap();
        let mut buf = [0.0f32; 1024];
        b.process(&mut buf, 2);
        let peak = buf.iter().map(|s| s.abs()).fold(0.0f32, f32::max);
        assert!(
            peak > 0.0,
            "expected audio after queued SynthEvent::NoteOn, got silence"
        );
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
