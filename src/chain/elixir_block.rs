//! Audio-chain adapter for the Elixir harmonic foundations engine.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use ringbuf::traits::{Consumer, Split};
use ringbuf::{HeapCons, HeapRb};

use super::block::{AudioBlock, MidiBlockEvent};
#[cfg(test)]
use crate::elixir::SynthVoiceId;
use crate::elixir::{SynthEvent, SynthParams};
use crate::slide::{SlideRuntime, SlideSettings, SlideSlot, SlideTelemetry};
use elixir_core::{Engine, VoiceEvent, VoiceId, VoiceRole, MAX_POLYPHONY};

pub const ELIXIR_EVENT_QUEUE_CAPACITY: usize = 1024;
const DEFAULT_MAX_BLOCK: usize = 2048;
const MIDI_VOICE_PREFIX: u64 = 1 << 63;
const SLIDE_UPDATE_FRAMES: usize = 8;

#[derive(Clone, Copy)]
struct MidiOwner {
    note: u8,
    role: VoiceRole,
    id: VoiceId,
    frequency_hz: f32,
    slide_slot: SlideSlot,
    age: u64,
}

/// `AudioBlock` adapter for Elixir's bounded, allocation-free core.
pub struct ElixirSynthBlock {
    engine: Engine,
    params: Arc<SynthParams>,
    events: HeapCons<SynthEvent>,
    event_fault: Arc<AtomicBool>,
    midi_owners: [Option<MidiOwner>; MAX_POLYPHONY],
    next_midi_id: u64,
    compare_standard: bool,
    sample_rate: u32,
    slide: SlideRuntime,
    slide_telemetry: Arc<SlideTelemetry>,
}

fn voice_role(mix_group: u8) -> Option<VoiceRole> {
    VoiceRole::ALL.get(mix_group as usize).copied()
}

fn slide_role(role: VoiceRole) -> crate::slide::SlideRole {
    match role {
        VoiceRole::Input => crate::slide::SlideRole::Input,
        VoiceRole::Harmony => crate::slide::SlideRole::Harmony,
        VoiceRole::Canon => crate::slide::SlideRole::Canon,
        VoiceRole::Counterpoint => crate::slide::SlideRole::Counterpoint,
    }
}

impl ElixirSynthBlock {
    pub fn new(sample_rate: u32) -> Self {
        Self::new_with_params(sample_rate, Arc::new(SynthParams::new()))
    }

    pub fn new_with_params(sample_rate: u32, params: Arc<SynthParams>) -> Self {
        Self::new_with_params_and_telemetry(sample_rate, params, Arc::new(SlideTelemetry::new()))
    }

    pub fn new_with_params_and_telemetry(
        sample_rate: u32,
        params: Arc<SynthParams>,
        slide_telemetry: Arc<SlideTelemetry>,
    ) -> Self {
        let queue = HeapRb::new(ELIXIR_EVENT_QUEUE_CAPACITY);
        let (_tx, rx) = queue.split();
        Self::new_with_event_consumer_and_telemetry(
            sample_rate,
            params,
            rx,
            Arc::new(AtomicBool::new(false)),
            slide_telemetry,
        )
    }

    pub fn new_with_event_consumer(
        sample_rate: u32,
        params: Arc<SynthParams>,
        events: HeapCons<SynthEvent>,
        event_fault: Arc<AtomicBool>,
    ) -> Self {
        Self::new_with_event_consumer_and_telemetry(
            sample_rate,
            params,
            events,
            event_fault,
            Arc::new(SlideTelemetry::new()),
        )
    }

    pub fn new_with_event_consumer_and_telemetry(
        sample_rate: u32,
        params: Arc<SynthParams>,
        events: HeapCons<SynthEvent>,
        event_fault: Arc<AtomicBool>,
        slide_telemetry: Arc<SlideTelemetry>,
    ) -> Self {
        let mut engine = Engine::new();
        engine.prepare(sample_rate, DEFAULT_MAX_BLOCK);
        Self {
            engine,
            params,
            events,
            event_fault,
            midi_owners: [None; MAX_POLYPHONY],
            next_midi_id: 0,
            compare_standard: false,
            sample_rate,
            slide: SlideRuntime::new(),
            slide_telemetry,
        }
    }

    fn drain_events(&mut self) {
        if self.event_fault.swap(false, Ordering::AcqRel) {
            for _ in 0..ELIXIR_EVENT_QUEUE_CAPACITY {
                if self.events.try_pop().is_none() {
                    break;
                }
            }
            self.clear_ownership();
            self.engine.panic();
            return;
        }

        for _ in 0..ELIXIR_EVENT_QUEUE_CAPACITY {
            let Some(event) = self.events.try_pop() else {
                break;
            };
            match event {
                SynthEvent::NoteOn {
                    voice_id,
                    midi_anchor,
                    frequency_hz,
                    velocity,
                    mix_group,
                    slide_slot,
                    slide,
                } => {
                    if let Some(role) = voice_role(mix_group) {
                        let frequency_hz = self.slide.note_on(
                            voice_id.get(),
                            slide_slot,
                            frequency_hz,
                            slide,
                            self.sample_rate as f32,
                        );
                        self.engine.handle_voice_event(VoiceEvent::NoteOn {
                            voice_id: VoiceId::new(voice_id.get()),
                            role,
                            midi_anchor,
                            frequency_hz,
                            velocity,
                        });
                    }
                }
                SynthEvent::Retune {
                    voice_id,
                    frequency_hz,
                } => {
                    self.slide.retune_now(voice_id.get(), frequency_hz);
                    self.engine.handle_voice_event(VoiceEvent::Retune {
                        voice_id: VoiceId::new(voice_id.get()),
                        frequency_hz,
                    });
                }
                SynthEvent::NoteOff { voice_id } => {
                    self.slide.note_off(voice_id.get());
                    self.engine.handle_voice_event(VoiceEvent::NoteOff {
                        voice_id: VoiceId::new(voice_id.get()),
                    });
                }
                SynthEvent::SustainPedal { on } => self.engine.set_sustain_pedal(on),
                SynthEvent::PitchBend { cents } => self
                    .engine
                    .handle_voice_event(VoiceEvent::PitchBend { cents }),
                SynthEvent::Expression { value } => self
                    .engine
                    .handle_voice_event(VoiceEvent::Expression { value }),
                SynthEvent::ModWheel { value } => self
                    .engine
                    .handle_voice_event(VoiceEvent::ModWheel { value }),
                SynthEvent::AllNotesOff => {
                    self.clear_ownership();
                    self.slide.clear();
                    self.engine.all_notes_off();
                }
            }
        }
    }

    pub fn note_on_for_role(&mut self, note: u8, velocity: u8, role: VoiceRole) {
        self.note_on_frequency_for_role(
            note,
            elixir_core::util::midi_to_freq(note),
            velocity,
            role,
        );
    }

    pub fn note_on_frequency_for_role(
        &mut self,
        note: u8,
        frequency_hz: f32,
        velocity: u8,
        role: VoiceRole,
    ) {
        self.note_on_frequency_for_role_with_slide(
            note,
            frequency_hz,
            velocity,
            role,
            SlideSlot::new(slide_role(role), 0),
            SlideSettings::default(),
        );
    }

    #[allow(clippy::too_many_arguments)]
    pub fn note_on_frequency_for_role_with_slide(
        &mut self,
        note: u8,
        frequency_hz: f32,
        velocity: u8,
        role: VoiceRole,
        slide_slot: SlideSlot,
        slide: SlideSettings,
    ) {
        if note >= 128 || velocity == 0 || !frequency_hz.is_finite() || frequency_hz <= 0.0 {
            return;
        }
        let age = self.next_midi_id;
        let id = VoiceId::new(MIDI_VOICE_PREFIX | age);
        self.next_midi_id = self.next_midi_id.wrapping_add(1) & !MIDI_VOICE_PREFIX;
        let index = self
            .midi_owners
            .iter()
            .position(Option::is_none)
            .or_else(|| {
                self.midi_owners
                    .iter()
                    .enumerate()
                    .min_by_key(|(_, owner)| owner.unwrap().age)
                    .map(|(index, _)| index)
            })
            .unwrap_or(0);
        if let Some(stolen) = self.midi_owners[index] {
            self.slide.note_off(stolen.id.get());
            self.engine.handle_voice_event(VoiceEvent::NoteOff {
                voice_id: stolen.id,
            });
        }
        self.midi_owners[index] = Some(MidiOwner {
            note,
            role,
            id,
            frequency_hz,
            slide_slot,
            age,
        });
        let sounding_frequency = if self.compare_standard {
            elixir_core::util::midi_to_freq(note)
        } else {
            frequency_hz
        };
        let sounding_frequency = self.slide.note_on(
            id.get(),
            slide_slot,
            sounding_frequency,
            if self.compare_standard {
                SlideSettings::default()
            } else {
                slide
            },
            self.sample_rate as f32,
        );
        self.engine.handle_voice_event(VoiceEvent::NoteOn {
            voice_id: id,
            role,
            midi_anchor: note,
            frequency_hz: sounding_frequency,
            velocity,
        });
    }

    pub fn set_compare_standard(&mut self, enabled: bool) {
        if self.compare_standard == enabled {
            return;
        }
        self.compare_standard = enabled;
        for owner in self.midi_owners.iter().flatten() {
            let frequency_hz = if enabled {
                elixir_core::util::midi_to_freq(owner.note)
            } else {
                owner.frequency_hz
            };
            self.slide.retune_now(owner.id.get(), frequency_hz);
            self.engine.handle_voice_event(VoiceEvent::Retune {
                voice_id: owner.id,
                frequency_hz,
            });
        }
    }

    pub fn note_off_for_role(&mut self, note: u8, role: VoiceRole) {
        self.note_off_for_role_and_slot(note, role, None);
    }

    pub fn note_off_for_slot(&mut self, note: u8, role: VoiceRole, slide_slot: SlideSlot) {
        self.note_off_for_role_and_slot(note, role, Some(slide_slot));
    }

    fn note_off_for_role_and_slot(
        &mut self,
        note: u8,
        role: VoiceRole,
        slide_slot: Option<SlideSlot>,
    ) {
        let owner = self
            .midi_owners
            .iter()
            .enumerate()
            .filter_map(|(index, owner)| owner.map(|owner| (index, owner)))
            .filter(|(_, owner)| {
                owner.note == note
                    && owner.role == role
                    && slide_slot.is_none_or(|slot| owner.slide_slot == slot)
            })
            .min_by_key(|(_, owner)| owner.age);
        if let Some((index, owner)) = owner {
            self.midi_owners[index] = None;
            self.slide.note_off(owner.id.get());
            self.engine
                .handle_voice_event(VoiceEvent::NoteOff { voice_id: owner.id });
        }
    }

    fn clear_ownership(&mut self) {
        self.midi_owners.fill(None);
        self.slide.clear();
    }

    pub fn set_pitch_bend_cents(&mut self, cents: f32) {
        self.engine.set_pitch_bend_cents(cents);
    }

    pub fn set_expression(&mut self, value: f32) {
        self.engine.set_expression(value);
    }

    pub fn set_mod_wheel(&mut self, value: f32) {
        self.engine.set_mod_wheel(value);
    }

    fn apply_params(&mut self) {
        self.engine.set_master_gain(self.params.master_gain());
        let gains = self.params.mix_gains();
        let patches = self.params.role_patches();
        for (index, role) in VoiceRole::ALL.into_iter().enumerate() {
            self.engine.set_role_gain(role, gains[index]);
            self.engine.set_role_patch(role, patches[index]);
        }
    }
}

impl AudioBlock for ElixirSynthBlock {
    fn name(&self) -> &str {
        "Elixir"
    }

    fn type_id(&self) -> &str {
        "builtin.synth"
    }

    fn process(&mut self, buffer: &mut [f32], channels: usize) {
        self.drain_events();
        self.apply_params();
        if self.params.enabled() {
            if channels == 0 || !self.slide.is_moving() {
                self.engine.process(buffer, channels);
            } else {
                for chunk in buffer.chunks_mut(channels * SLIDE_UPDATE_FRAMES) {
                    let engine = &mut self.engine;
                    self.slide.for_each_moving(|voice_id, frequency_hz| {
                        engine.handle_voice_event(VoiceEvent::Retune {
                            voice_id: VoiceId::new(voice_id),
                            frequency_hz,
                        });
                    });
                    self.slide.finish_completed();
                    self.engine.process(chunk, channels);
                    self.slide.advance(chunk.len() / channels);
                }
            }
        } else {
            self.clear_ownership();
            self.engine.panic();
            self.engine.process(buffer, channels);
            buffer.fill(0.0);
        }
        self.slide_telemetry.publish(&self.slide);
    }

    fn midi_event(&mut self, event: MidiBlockEvent) {
        match event {
            MidiBlockEvent::NoteOn { note, velocity } => {
                self.note_on_for_role(note, velocity, VoiceRole::Input)
            }
            MidiBlockEvent::NoteOff { note } => self.note_off_for_role(note, VoiceRole::Input),
            MidiBlockEvent::AllNotesOff => {
                self.clear_ownership();
                self.engine.all_notes_off();
            }
            MidiBlockEvent::SustainPedal { on } => self.engine.set_sustain_pedal(on),
            MidiBlockEvent::PitchBend { cents } => self.set_pitch_bend_cents(cents),
            MidiBlockEvent::Expression { value } => self.set_expression(value),
            MidiBlockEvent::ModWheel { value } => self.set_mod_wheel(value),
        }
    }

    fn reset(&mut self) {
        self.clear_ownership();
        self.engine.panic();
    }

    fn set_sample_rate(&mut self, sample_rate: u32) {
        self.clear_ownership();
        self.sample_rate = sample_rate;
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

    fn note_on(id: u64, frequency_hz: f32, mix_group: u8) -> SynthEvent {
        SynthEvent::note_on(SynthVoiceId::new(id), 69, frequency_hz, 100, mix_group)
    }

    fn sliding_note_on(id: u64, frequency_hz: f32) -> SynthEvent {
        SynthEvent::note_on_with_slide(
            SynthVoiceId::new(id),
            69,
            frequency_hz,
            100,
            0,
            SlideSlot::new(crate::slide::SlideRole::Input, 0),
            SlideSettings {
                travel: crate::slide::SlideTravel::Time {
                    milliseconds: 100.0,
                },
                trigger: crate::slide::SlideTrigger::Always,
                curve: crate::slide::SlideCurve::Exponential,
            },
        )
    }

    #[test]
    fn queued_ids_release_overlapping_anchors_independently() {
        let (mut block, mut tx, _) = event_block(Arc::new(SynthParams::new()));
        tx.try_push(note_on(1, 440.0, 0)).unwrap();
        tx.try_push(note_on(2, 442.0, 0)).unwrap();
        let mut audio = [0.0; 512];
        block.process(&mut audio, 2);
        assert_eq!(block.engine.live_voice_count(), 2);

        tx.try_push(SynthEvent::note_off(SynthVoiceId::new(1)))
            .unwrap();
        block.process(&mut audio, 2);
        assert_eq!(block.engine.live_voice_count(), 1);
    }

    #[test]
    fn queued_slide_is_bounded_allocation_free_and_identity_exact() {
        let (mut block, mut tx, _) = event_block(Arc::new(SynthParams::new()));
        let mut audio = [0.0; 512];
        tx.try_push(sliding_note_on(1, 440.0)).unwrap();
        block.process(&mut audio, 2);
        tx.try_push(SynthEvent::note_off(SynthVoiceId::new(1)))
            .unwrap();
        tx.try_push(sliding_note_on(2, 880.0)).unwrap();
        assert_no_alloc::assert_no_alloc(|| block.process(&mut audio, 2));
        assert!(block.slide.is_moving());
        assert_eq!(block.engine.live_voice_count(), 1);
        tx.try_push(SynthEvent::note_off(SynthVoiceId::new(2)))
            .unwrap();
        block.process(&mut audio, 2);
        assert_eq!(block.engine.live_voice_count(), 0);
    }

    #[test]
    fn direct_midi_repeats_release_fifo_without_orphans() {
        let mut block = ElixirSynthBlock::new(48_000);
        block.midi_event(MidiBlockEvent::NoteOn {
            note: 69,
            velocity: 100,
        });
        block.midi_event(MidiBlockEvent::NoteOn {
            note: 69,
            velocity: 90,
        });
        assert_eq!(block.engine.live_voice_count(), 2);
        block.midi_event(MidiBlockEvent::NoteOff { note: 69 });
        assert_eq!(block.engine.live_voice_count(), 1);
        block.midi_event(MidiBlockEvent::NoteOff { note: 69 });
        assert_eq!(block.engine.live_voice_count(), 0);
    }

    #[test]
    fn exact_frequency_keeps_midi_anchor_release_ownership() {
        let mut exact = ElixirSynthBlock::new(48_000);
        exact.note_on_frequency_for_role(69, 432.0, 100, VoiceRole::Input);
        let mut exact_audio = [0.0; 512];
        exact.process(&mut exact_audio, 2);

        let mut standard = ElixirSynthBlock::new(48_000);
        standard.note_on_for_role(69, 100, VoiceRole::Input);
        let mut standard_audio = [0.0; 512];
        standard.process(&mut standard_audio, 2);
        assert_ne!(exact_audio, standard_audio);

        exact.note_off_for_role(69, VoiceRole::Input);
        assert_eq!(exact.engine.live_voice_count(), 0);
    }

    #[test]
    fn same_pitch_roles_keep_independent_gain_and_release_ownership() {
        let params = Arc::new(SynthParams::new());
        params.set_mix_gain(0, 0.0);
        let mut block = ElixirSynthBlock::new_with_params(48_000, params);
        block.note_on_for_role(69, 100, VoiceRole::Input);
        block.note_on_for_role(69, 100, VoiceRole::Harmony);
        let mut audio = [0.0; 512];
        block.process(&mut audio, 2);
        assert!(audio.iter().any(|sample| sample.abs() > 1.0e-6));

        block.note_off_for_role(69, VoiceRole::Harmony);
        block.process(&mut audio, 2);
        assert!(audio[audio.len() - 16..]
            .iter()
            .all(|sample| sample.abs() < 1.0e-6));
        assert_eq!(block.engine.live_voice_count(), 1);
    }

    #[test]
    fn role_patch_and_expression_reach_the_audio_engine() {
        let params = Arc::new(SynthParams::new());
        let mut patch = crate::elixir::RolePatch::sine();
        patch.harmonics.amplitudes = [1.0, 0.0, 0.45, 0.0, 0.25, 0.0];
        params.set_role_patch(1, patch);
        let mut block = ElixirSynthBlock::new_with_params(48_000, params);
        block.note_on_for_role(69, 127, VoiceRole::Harmony);
        let mut sounding = [0.0; 512];
        block.process(&mut sounding, 1);
        assert_eq!(block.engine.role_patch(VoiceRole::Harmony), patch);
        assert!(sounding.iter().any(|sample| sample.abs() > 1.0e-6));

        block.midi_event(MidiBlockEvent::Expression { value: 0.0 });
        let mut silent = [1.0; 64];
        block.process(&mut silent, 1);
        assert!(silent.iter().all(|sample| sample.abs() < 1.0e-6));
        block.midi_event(MidiBlockEvent::PitchBend { cents: 50.0 });
        block.midi_event(MidiBlockEvent::ModWheel { value: 0.75 });
        assert_eq!(block.engine.pitch_bend_cents(), 50.0);
        assert_eq!(block.engine.mod_wheel(), 0.75);
    }

    #[test]
    fn same_pitch_generated_slots_release_exact_owner() {
        let mut block = ElixirSynthBlock::new(48_000);
        let first = SlideSlot::new(crate::slide::SlideRole::Canon, 0);
        let second = SlideSlot::new(crate::slide::SlideRole::Canon, 1);
        block.note_on_frequency_for_role_with_slide(
            69,
            440.0,
            100,
            VoiceRole::Canon,
            first,
            SlideSettings::default(),
        );
        block.note_on_frequency_for_role_with_slide(
            69,
            440.0,
            100,
            VoiceRole::Canon,
            second,
            SlideSettings::default(),
        );
        block.note_off_for_slot(69, VoiceRole::Canon, second);
        assert!(block
            .midi_owners
            .iter()
            .flatten()
            .any(|owner| owner.slide_slot == first));
        assert!(!block
            .midi_owners
            .iter()
            .flatten()
            .any(|owner| owner.slide_slot == second));
    }

    #[test]
    fn direct_midi_overflow_steals_without_losing_release_ownership() {
        let mut block = ElixirSynthBlock::new(48_000);
        for note in 48..48 + MAX_POLYPHONY as u8 + 1 {
            block.midi_event(MidiBlockEvent::NoteOn {
                note,
                velocity: 100,
            });
        }
        assert_eq!(block.engine.live_voice_count(), MAX_POLYPHONY);
        for note in 48..48 + MAX_POLYPHONY as u8 + 1 {
            block.midi_event(MidiBlockEvent::NoteOff { note });
        }
        assert_eq!(block.engine.live_voice_count(), 0);
    }

    #[test]
    fn queue_fault_discards_pending_events_and_panics() {
        let (mut block, mut tx, fault) = event_block(Arc::new(SynthParams::new()));
        tx.try_push(note_on(1, 440.0, 0)).unwrap();
        let mut audio = [0.0; 512];
        block.process(&mut audio, 2);
        tx.try_push(note_on(2, 523.251_1, 1)).unwrap();
        fault.store(true, Ordering::Release);
        block.process(&mut audio, 2);
        assert_eq!(block.engine.live_voice_count(), 0);
        assert!(audio.iter().all(|sample| sample.is_finite()));
    }

    #[test]
    fn processing_and_lifecycle_are_allocation_free() {
        let (mut block, mut tx, fault) = event_block(Arc::new(SynthParams::new()));
        let mut audio = [0.0; 512];
        assert_no_alloc::assert_no_alloc(|| {
            tx.try_push(note_on(1, 440.0, 0)).unwrap();
            block.process(&mut audio, 2);
            tx.try_push(SynthEvent::note_off(SynthVoiceId::new(1)))
                .unwrap();
            block.process(&mut audio, 2);
            fault.store(true, Ordering::Release);
            block.process(&mut audio, 2);
        });
        assert!(audio.iter().all(|sample| sample.is_finite()));
    }

    #[test]
    fn disable_reset_and_sample_rate_change_drop_ownership() {
        let params = Arc::new(SynthParams::new());
        let (mut block, mut tx, _) = event_block(Arc::clone(&params));
        tx.try_push(note_on(1, 440.0, 0)).unwrap();
        let mut audio = [0.0; 512];
        block.process(&mut audio, 2);
        params.set_enabled(false);
        block.process(&mut audio, 2);
        assert_eq!(block.engine.live_voice_count(), 0);
        assert!(audio.iter().all(|sample| *sample == 0.0));
        params.set_enabled(true);
        block.process(&mut audio, 2);
        assert!(audio.iter().all(|sample| *sample == 0.0));
        block.reset();
        block.set_sample_rate(44_100);
        assert_eq!(block.engine.sample_rate(), 44_100);
    }
}
