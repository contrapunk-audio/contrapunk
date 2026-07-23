//! Audio-chain wrapper around the Elixir DSP engine.
//!
//! Phase 21.A0 deliverable. Gated behind the `elixir-synth` feature so
//! the legacy `src/synth/` keeps shipping until A-Cut. Both feature
//! configurations must compile cleanly on every surface (CLI, Tauri,
//! WASM, plugin) — see `ELIXIR-PLAN.md` §3 for the cutover plan.
//!
//! The wrapper has grown with A1-A6 without changing the [`AudioBlock`]
//! interface: it forwards MIDI and legacy [`SynthEvent`] traffic into
//! [`elixir_core::Engine`], mirrors the existing public [`SynthParams`]
//! surface, and lets the audio chain swap from legacy synth to Elixir
//! behind the `elixir-synth` feature.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use ringbuf::traits::{Consumer, Split};
use ringbuf::{HeapCons, HeapRb};

use super::block::{AudioBlock, MidiBlockEvent};
#[cfg(test)]
use crate::synth::SynthVoiceId;
use crate::synth::{SynthEvent, SynthParams, Waveform};
use elixir_core::osc::{PhaseDistortionMode, SpectralMorph};
use elixir_core::{Engine, VoiceEvent, VoiceId, VoiceRole};

/// `AudioBlock` adapter for the Elixir engine.
pub struct ElixirSynthBlock {
    engine: Engine,
    params: Arc<SynthParams>,
    events: HeapCons<SynthEvent>,
    event_fault: Arc<AtomicBool>,
}

pub const ELIXIR_EVENT_QUEUE_CAPACITY: usize = 1024;
fn voice_role(mix_group: u8) -> Option<VoiceRole> {
    VoiceRole::ALL.get(mix_group as usize).copied()
}

impl ElixirSynthBlock {
    /// Construct an Elixir synth block. The sample rate is also pushed
    /// into the engine via [`AudioBlock::set_sample_rate`] later, but we
    /// prepare here so a fresh block is immediately usable.
    pub fn new(sample_rate: u32) -> Self {
        let queue = HeapRb::new(ELIXIR_EVENT_QUEUE_CAPACITY);
        let (_tx, rx) = queue.split();
        Self::new_with_event_consumer(
            sample_rate,
            Arc::new(SynthParams::new()),
            rx,
            Arc::new(AtomicBool::new(false)),
        )
    }

    /// Construct an Elixir synth block with a preallocated SPSC event
    /// consumer. Producers live outside the audio callback.
    pub fn new_with_event_consumer(
        sample_rate: u32,
        params: Arc<SynthParams>,
        events: HeapCons<SynthEvent>,
        event_fault: Arc<AtomicBool>,
    ) -> Self {
        let mut engine = Engine::new();
        engine.prepare(sample_rate, DEFAULT_MAX_BLOCK);
        Self {
            engine,
            params,
            events,
            event_fault,
        }
    }

    fn drain_events(&mut self) {
        if self.event_fault.swap(false, Ordering::AcqRel) {
            for _ in 0..ELIXIR_EVENT_QUEUE_CAPACITY {
                if self.events.try_pop().is_none() {
                    break;
                }
            }
            self.engine.panic();
            return;
        }

        for _ in 0..ELIXIR_EVENT_QUEUE_CAPACITY {
            let Some(ev) = self.events.try_pop() else {
                break;
            };
            match ev {
                SynthEvent::NoteOn {
                    voice_id,
                    midi_anchor,
                    frequency_hz,
                    velocity,
                    mix_group,
                } => {
                    if let Some(role) = voice_role(mix_group) {
                        self.engine.handle_voice_event(VoiceEvent::NoteOn {
                            voice_id: VoiceId::new(voice_id.get()),
                            role,
                            midi_anchor,
                            frequency_hz,
                            velocity,
                        });
                    }
                }
                SynthEvent::NoteOff { voice_id } => {
                    self.engine.handle_voice_event(VoiceEvent::NoteOff {
                        voice_id: VoiceId::new(voice_id.get()),
                    });
                }
                SynthEvent::AllNotesOff => self.engine.all_notes_off(),
            }
        }
    }

    /// Set the voice-filter cutoff in Hz (A4). Pass-through to the
    /// underlying engine.
    pub fn set_filter_cutoff_hz(&mut self, hz: f32) {
        if hz.is_finite() {
            self.params.set_cutoff_hz(hz.round() as u32);
            self.engine.set_filter_cutoff_hz(hz);
        }
    }
    /// Set the voice-filter resonance, `0..1`.
    pub fn set_filter_resonance(&mut self, r: f32) {
        if r.is_finite() {
            self.params.set_resonance(r);
            self.engine.set_filter_resonance(r);
        }
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
        let mix_gains = self.params.mix_gains();
        for (role, gain) in VoiceRole::ALL.into_iter().zip(mix_gains) {
            self.engine.set_role_gain(role, gain);
        }

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
    use ringbuf::traits::Producer;
    use ringbuf::HeapProd;

    fn event_block(
        params: Arc<SynthParams>,
    ) -> (ElixirSynthBlock, HeapProd<SynthEvent>, Arc<AtomicBool>) {
        let queue = HeapRb::new(ELIXIR_EVENT_QUEUE_CAPACITY);
        let (tx, rx) = queue.split();
        let fault = Arc::new(AtomicBool::new(false));
        (
            ElixirSynthBlock::new_with_event_consumer(48_000, params, rx, Arc::clone(&fault)),
            tx,
            fault,
        )
    }

    fn note_on(id: u64, midi_anchor: u8, frequency_hz: f32, mix_group: u8) -> SynthEvent {
        SynthEvent::note_on(
            SynthVoiceId::new(id),
            midi_anchor,
            frequency_hz,
            100,
            mix_group,
        )
    }

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
        let (mut b, mut tx, _fault) = event_block(params);
        tx.try_push(note_on(1, 69, 440.0, 0)).unwrap();
        let mut buf = [0.5f32; 1024];
        b.process(&mut buf, 2);
        assert!(buf.iter().all(|s| *s == 0.0));
    }

    #[test]
    fn synth_event_receiver_produces_audio() {
        let (mut b, mut tx, _fault) = event_block(Arc::new(SynthParams::new()));
        tx.try_push(note_on(1, 69, 440.0, 0)).unwrap();
        let mut buf = [0.0f32; 1024];
        b.process(&mut buf, 2);
        let peak = buf.iter().map(|s| s.abs()).fold(0.0f32, f32::max);
        assert!(
            peak > 0.0,
            "expected audio after queued SynthEvent::NoteOn, got silence"
        );
    }

    #[test]
    fn same_anchor_same_role_voices_keep_ids_and_release_selectively() {
        let (mut b, mut tx, _fault) = event_block(Arc::new(SynthParams::new()));
        tx.try_push(note_on(1, 69, 440.0, 0)).unwrap();
        tx.try_push(note_on(2, 69, 442.0, 0)).unwrap();

        let mut sounding = [0.0; 1024];
        b.process(&mut sounding, 2);
        assert_eq!(b.engine.live_voice_count(), 2);

        tx.try_push(SynthEvent::note_off(SynthVoiceId::new(1)))
            .unwrap();
        let mut first_tail = [0.0; 32_000];
        b.process(&mut first_tail, 2);
        assert_eq!(b.engine.live_voice_count(), 1);
        assert!(first_tail[first_tail.len() - 64..]
            .iter()
            .any(|sample| sample.abs() > 1.0e-6));

        tx.try_push(SynthEvent::note_off(SynthVoiceId::new(2)))
            .unwrap();
        let mut final_tail = [0.0; 32_000];
        b.process(&mut final_tail, 2);
        assert_eq!(b.engine.live_voice_count(), 0);
        assert!(final_tail[final_tail.len() - 64..]
            .iter()
            .all(|sample| sample.abs() < 1.0e-3));
    }

    #[test]
    fn event_fault_discards_queue_and_panics_without_blocking() {
        let (mut b, mut tx, fault) = event_block(Arc::new(SynthParams::new()));
        tx.try_push(note_on(1, 69, 440.0, 0)).unwrap();
        let mut sounding = [0.0; 1024];
        b.process(&mut sounding, 2);
        assert!(sounding.iter().any(|sample| sample.abs() > 1.0e-6));

        tx.try_push(note_on(2, 72, 523.251_1, 1)).unwrap();
        fault.store(true, Ordering::Release);
        let mut tail = [0.0; 800];
        b.process(&mut tail, 2);
        assert!(tail[tail.len() - 64..].iter().all(|sample| *sample == 0.0));
    }

    #[test]
    fn steady_state_ring_and_block_processing_allocates_nothing() {
        let (mut block, mut events, fault) = event_block(Arc::new(SynthParams::new()));
        let mut audio = [0.0; 512];

        assert_no_alloc::assert_no_alloc(|| {
            events.try_push(note_on(1, 69, 440.0, 0)).unwrap();
            block.process(&mut audio, 2);
            events
                .try_push(SynthEvent::note_off(SynthVoiceId::new(1)))
                .unwrap();
            block.process(&mut audio, 2);
            fault.store(true, Ordering::Release);
            block.process(&mut audio, 2);
        });
        assert!(audio.iter().all(|sample| sample.is_finite()));
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
