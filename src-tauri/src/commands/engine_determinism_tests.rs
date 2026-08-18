use super::*;
use contrapunk::elixir::{self, SynthEventReceiver};
use contrapunk::harmony::{HarmonyMode, Key, RoutingMode};
use contrapunk_companion::{LoopMidiEvent, PhrasePhase};
use std::collections::BTreeSet;

const TRACE_SCHEMA_VERSION: u16 = 1;

#[derive(Clone, Debug, PartialEq, Eq)]
struct DeterminismTrace {
    schema_version: u16,
    events: Vec<ObservedEvent>,
    final_ownership: OwnershipSnapshot,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct OwnershipSnapshot {
    routed_live: usize,
    routed_loop: usize,
    sustain: usize,
    loop_sources: usize,
    synth_voices: usize,
    midi_slide_voices: usize,
    live_phrase: PhrasePhase,
    loop_phrase: PhrasePhase,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum ObservedEvent {
    NoteOn {
        voice_id: u64,
        note: u8,
        frequency_bits: u32,
        velocity: u8,
        mix_group: u8,
        slide_role: SlideRole,
        slide_voice: u8,
        slide: String,
    },
    Retune {
        voice_id: u64,
        frequency_bits: u32,
    },
    NoteOff {
        voice_id: u64,
    },
    Sustain(bool),
    PitchBend(u32),
    Expression(u32),
    ModWheel(u32),
    AllNotesOff,
}

struct PerformanceHarness {
    engine: Arc<Mutex<HarmonyEngine>>,
    loop_engine: Arc<Mutex<HarmonyEngine>>,
    transport: Arc<Transport>,
    companion: crate::companion::Companion,
    loop_companion: crate::companion::Companion,
    tick_scheduler: BeatTickScheduler,
    output: OutputRouter,
    midi_slides: MidiSlideRuntime,
    synth_tx: SynthEventSender,
    synth_rx: SynthEventReceiver,
    voice_outputs: Arc<Mutex<VoiceOutputRoutes>>,
    slide_config: Arc<Mutex<SlideConfig>>,
    input_notes: Arc<Mutex<HashSet<u8>>>,
    harmony_notes: Arc<Mutex<HashSet<u8>>>,
    borrowed_notes: Arc<Mutex<HashSet<u8>>>,
    canon_notes: Arc<Mutex<NoteCounts>>,
    counterpoint_notes: Arc<Mutex<NoteCounts>>,
    loop_input_notes: Arc<Mutex<HashSet<u8>>>,
    loop_harmony_notes: Arc<Mutex<HashSet<u8>>>,
    loop_canon_notes: Arc<Mutex<NoteCounts>>,
    loop_counterpoint_notes: Arc<Mutex<NoteCounts>>,
    chord_name: Arc<Mutex<String>>,
    output_notes: RoutedNoteCounts,
    loop_output_notes: RoutedNoteCounts,
    sustain_owners: SustainOwners,
    loop_source_frames: LoopSourceFrames,
    active_synth_owners: BTreeSet<u64>,
    trace: Vec<ObservedEvent>,
}

impl PerformanceHarness {
    fn new(rich_companion: bool) -> Self {
        let engine = Arc::new(Mutex::new(HarmonyEngine::with_voices(
            Key::C,
            if rich_companion {
                HarmonyMode::StrictCounterpoint
            } else {
                HarmonyMode::PassThrough
            },
            4,
        )));
        let loop_engine = Arc::new(Mutex::new(
            engine
                .lock()
                .unwrap_or_else(|error| error.into_inner())
                .fork_clean_runtime(),
        ));
        let transport = Transport::new(48_000);
        transport.play();
        let mut companion =
            crate::state::new_arrangement_companion(Arc::clone(&transport), Arc::clone(&engine));

        if rich_companion {
            companion
                .configure_lane(
                    "canon",
                    serde_json::json!({
                        "enabled": true,
                        "voices": [
                            {
                                "delay_beats": 0.5,
                                "transpose_degrees": 2,
                                "time_ratio": 1.0
                            },
                            {
                                "delay_beats": 1.0,
                                "transpose_degrees": -2,
                                "time_ratio": 1.0
                            }
                        ]
                    }),
                )
                .unwrap();
            companion
                .configure_lane(
                    "counterpoint",
                    serde_json::json!({
                        "enabled": true,
                        "species": "Species2",
                        "transpose_degrees": -2,
                        "prefer_above": false,
                        "phrase_aware": true
                    }),
                )
                .unwrap();
            companion
                .configure_lane(
                    "pattern_low",
                    serde_json::json!({
                        "enabled": true,
                        "cycle_beats": 4.0,
                        "tail_beats": 4.0,
                        "pitch_anchor": "phrase_start",
                        "events": [
                            {"beat": 0.0, "degree": 0, "octave": -2, "duration_beats": 0.5, "velocity": 72},
                            {"beat": 1.0, "degree": 4, "octave": -2, "duration_beats": 0.5, "velocity": 66},
                            {"beat": 2.0, "degree": 2, "octave": -2, "duration_beats": 0.5, "velocity": 69}
                        ]
                    }),
                )
                .unwrap();
            companion
                .configure_lane(
                    "pattern_counter",
                    serde_json::json!({
                        "enabled": true,
                        "cycle_beats": 4.0,
                        "tail_beats": 4.0,
                        "pitch_anchor": "latest_input",
                        "events": [
                            {"beat": 0.5, "degree": 4, "octave": 0, "duration_beats": 0.25, "velocity": 77},
                            {"beat": 1.5, "degree": 2, "octave": 0, "duration_beats": 0.25, "velocity": 74}
                        ]
                    }),
                )
                .unwrap();
        }

        let mut state = companion.save();
        state.enabled = rich_companion;
        companion.restore(state.clone()).unwrap();
        let mut loop_companion = crate::state::new_arrangement_companion(
            Arc::clone(&transport),
            Arc::clone(&loop_engine),
        );
        loop_companion.restore(state).unwrap();
        let tick_scheduler = BeatTickScheduler::new(&transport);
        let (synth_tx, synth_rx) = elixir::synth_event_channel();

        Self {
            engine,
            loop_engine,
            transport,
            companion,
            loop_companion,
            tick_scheduler,
            output: OutputRouter::new(&[]).unwrap(),
            midi_slides: MidiSlideRuntime::new(0, Arc::new(SlideTelemetry::new())),
            synth_tx,
            synth_rx,
            voice_outputs: Arc::new(Mutex::new(VoiceOutputRoutes::default())),
            slide_config: Arc::new(Mutex::new(SlideConfig::default())),
            input_notes: Arc::new(Mutex::new(HashSet::new())),
            harmony_notes: Arc::new(Mutex::new(HashSet::new())),
            borrowed_notes: Arc::new(Mutex::new(HashSet::new())),
            canon_notes: Arc::new(Mutex::new(NoteCounts::new())),
            counterpoint_notes: Arc::new(Mutex::new(NoteCounts::new())),
            loop_input_notes: Arc::new(Mutex::new(HashSet::new())),
            loop_harmony_notes: Arc::new(Mutex::new(HashSet::new())),
            loop_canon_notes: Arc::new(Mutex::new(NoteCounts::new())),
            loop_counterpoint_notes: Arc::new(Mutex::new(NoteCounts::new())),
            chord_name: Arc::new(Mutex::new(String::new())),
            output_notes: RoutedNoteCounts::new(),
            loop_output_notes: RoutedNoteCounts::new(),
            sustain_owners: SustainOwners::new(),
            loop_source_frames: LoopSourceFrames::new(),
            active_synth_owners: BTreeSet::new(),
            trace: Vec::new(),
        }
    }

    fn set_beat_phase(&self) {
        let phase = Some(self.transport.beat_position());
        self.engine
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .set_counterpoint_beat_phase(phase);
        self.loop_engine
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .set_counterpoint_beat_phase(phase);
    }

    fn dispatch_tagged(
        &mut self,
        tagged: &[(&'static str, u8, crate::companion::DispatchOp)],
        origin: InputOrigin,
    ) {
        let (harmony_notes, canon_notes, counterpoint_notes) = match origin {
            InputOrigin::Live => (
                Arc::clone(&self.harmony_notes),
                Arc::clone(&self.canon_notes),
                Arc::clone(&self.counterpoint_notes),
            ),
            InputOrigin::Loop => (
                Arc::clone(&self.loop_harmony_notes),
                Arc::clone(&self.loop_canon_notes),
                Arc::clone(&self.loop_counterpoint_notes),
            ),
        };
        dispatch_companion_ops(
            tagged,
            self.output.connection_count(),
            &self.synth_tx,
            &mut self.output,
            &mut self.midi_slides,
            &self.voice_outputs,
            &harmony_notes,
            &canon_notes,
            &counterpoint_notes,
            &mut self.output_notes,
            &mut self.loop_output_notes,
            origin,
            &self.slide_config,
        );
    }

    fn live(&mut self, bytes: [u8; 3]) {
        self.set_beat_phase();
        let is_cc = bytes[0] & 0xf0 == 0xb0;
        if is_cc && bytes[1] == 64 {
            set_sustain_ownership(
                InputOrigin::Live,
                bytes[0] & 0x0f,
                bytes[2] >= 64,
                &mut self.sustain_owners,
                &self.voice_outputs,
                self.output.connection_count(),
                &self.synth_tx,
                &mut self.output,
                &mut self.midi_slides,
            );
        }

        let mut suppress_default = false;
        if let Some(event) = midi_bytes_to_input_event(&bytes) {
            let (tagged, suppress) = self.companion.on_input_tagged(event, &self.engine);
            self.dispatch_tagged(&tagged, InputOrigin::Live);
            suppress_default = suppress;
        }

        if !is_cc && !suppress_default {
            process_midi_message(
                &bytes,
                InputOrigin::Live,
                &self.engine,
                &mut self.output,
                &mut self.midi_slides,
                &self.input_notes,
                &self.harmony_notes,
                &self.borrowed_notes,
                &self.chord_name,
                RoutingMode::default(),
                &self.synth_tx,
                &self.voice_outputs,
                &self.slide_config,
                &mut self.output_notes,
                &mut self.loop_output_notes,
            );
        }
        self.drain_synth_events();
    }

    fn loop_event(&mut self, event: LoopMidiEvent) {
        self.set_beat_phase();
        let mut suppress_default = false;
        if let Some(input) = midi_bytes_to_input_event(&loop_event_bytes(event)) {
            let (tagged, suppress) = self.loop_companion.on_input_tagged_at(
                input,
                self.transport.total_beats(),
                &self.loop_engine,
            );
            self.dispatch_tagged(&tagged, InputOrigin::Loop);
            suppress_default = suppress;
        }

        match event {
            LoopMidiEvent::Cc64 { value, channel } => set_sustain_ownership(
                InputOrigin::Loop,
                channel,
                value >= 64,
                &mut self.sustain_owners,
                &self.voice_outputs,
                self.output.connection_count(),
                &self.synth_tx,
                &mut self.output,
                &mut self.midi_slides,
            ),
            event if !suppress_default => process_loop_midi_event(
                event,
                &self.loop_engine,
                &mut self.output,
                &mut self.midi_slides,
                &self.loop_input_notes,
                &self.loop_harmony_notes,
                &self.voice_outputs,
                &self.slide_config,
                &mut self.output_notes,
                &mut self.loop_output_notes,
                &mut self.loop_source_frames,
                &self.synth_tx,
            ),
            _ => {}
        }
        self.drain_synth_events();
    }

    fn tick(&mut self) {
        self.set_beat_phase();
        for slot in self.tick_scheduler.due_slots(&self.transport) {
            let beat = BeatTickScheduler::beat(slot);
            let loop_tagged = self.loop_companion.tick_tagged_at(beat, &self.loop_engine);
            self.dispatch_tagged(&loop_tagged, InputOrigin::Loop);
            let live_tagged = self.companion.tick_tagged_at(beat, &self.engine);
            self.dispatch_tagged(&live_tagged, InputOrigin::Live);
        }
        self.drain_synth_events();
    }

    fn advance_to(&mut self, target_sample: u64, block_size: u32) {
        assert!(target_sample >= self.transport.sample_pos());
        while self.transport.sample_pos() < target_sample {
            let remaining = target_sample - self.transport.sample_pos();
            self.transport
                .advance(remaining.min(block_size as u64) as u32);
            self.tick();
        }
    }

    fn panic(&mut self) {
        self.companion.reset_runtime();
        self.loop_companion.reset_runtime();
        self.tick_scheduler.reset(&self.transport);
        cleanup_loop_outputs(
            &mut self.output_notes,
            &mut self.loop_output_notes,
            &mut self.sustain_owners,
            self.output.connection_count(),
            &self.synth_tx,
            &mut self.output,
            &mut self.midi_slides,
        );
        self.loop_source_frames.clear();
        clear_all_sustain(
            &mut self.sustain_owners,
            self.output.connection_count(),
            &self.synth_tx,
            &mut self.output,
            &mut self.midi_slides,
        );
        drain_routed_outputs(
            &mut self.output_notes,
            self.output.connection_count(),
            &self.synth_tx,
            &mut self.output,
            &mut self.midi_slides,
        );
        self.loop_output_notes.clear();
        self.midi_slides.clear(&mut self.output);
        send_all_notes_off(
            self.output.connection_count(),
            &self.synth_tx,
            &mut self.output,
        );
        let _ = drain_all_tracked_notes(
            &self.input_notes,
            &self.harmony_notes,
            &self.borrowed_notes,
            &self.canon_notes,
            &self.counterpoint_notes,
        );
        self.loop_input_notes
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .clear();
        self.loop_harmony_notes
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .clear();
        self.loop_canon_notes
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .clear();
        self.loop_counterpoint_notes
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .clear();
        self.engine
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .clear_active_notes();
        self.loop_engine
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .clear_active_notes();
        self.chord_name
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .clear();
        self.drain_synth_events();
    }

    fn drain_synth_events(&mut self) {
        while let Ok(event) = self.synth_rx.try_recv() {
            let observed = match event {
                SynthEvent::NoteOn {
                    voice_id,
                    midi_anchor,
                    frequency_hz,
                    velocity,
                    mix_group,
                    slide_slot,
                    slide,
                } => {
                    assert!(
                        self.active_synth_owners.insert(voice_id.get()),
                        "duplicate synth voice owner {}",
                        voice_id.get()
                    );
                    ObservedEvent::NoteOn {
                        voice_id: voice_id.get(),
                        note: midi_anchor,
                        frequency_bits: frequency_hz.to_bits(),
                        velocity,
                        mix_group,
                        slide_role: slide_slot.role,
                        slide_voice: slide_slot.voice,
                        slide: format!("{slide:?}"),
                    }
                }
                SynthEvent::Retune {
                    voice_id,
                    frequency_hz,
                } => {
                    assert!(self.active_synth_owners.contains(&voice_id.get()));
                    ObservedEvent::Retune {
                        voice_id: voice_id.get(),
                        frequency_bits: frequency_hz.to_bits(),
                    }
                }
                SynthEvent::NoteOff { voice_id } => {
                    assert!(
                        self.active_synth_owners.remove(&voice_id.get()),
                        "release without synth owner {}",
                        voice_id.get()
                    );
                    ObservedEvent::NoteOff {
                        voice_id: voice_id.get(),
                    }
                }
                SynthEvent::SustainPedal { on } => ObservedEvent::Sustain(on),
                SynthEvent::PitchBend { cents } => ObservedEvent::PitchBend(cents.to_bits()),
                SynthEvent::Expression { value } => ObservedEvent::Expression(value.to_bits()),
                SynthEvent::ModWheel { value } => ObservedEvent::ModWheel(value.to_bits()),
                SynthEvent::AllNotesOff => {
                    self.active_synth_owners.clear();
                    ObservedEvent::AllNotesOff
                }
            };
            self.trace.push(observed);
        }
    }

    fn ownership_snapshot(&self) -> OwnershipSnapshot {
        OwnershipSnapshot {
            routed_live: self.output_notes.len(),
            routed_loop: self.loop_output_notes.len(),
            sustain: self.sustain_owners.len(),
            loop_sources: self.loop_source_frames.len(),
            synth_voices: self.active_synth_owners.len(),
            midi_slide_voices: self.midi_slides.voices.len(),
            live_phrase: self.companion.phrase_snapshot().phase,
            loop_phrase: self.loop_companion.phrase_snapshot().phase,
        }
    }

    fn assert_clean(&self) {
        assert!(self.output_notes.is_empty());
        assert!(self.loop_output_notes.is_empty());
        assert!(self.sustain_owners.is_empty());
        assert!(self.loop_source_frames.is_empty());
        assert!(self.active_synth_owners.is_empty());
        assert!(self.midi_slides.voices.is_empty());
        assert!(self.midi_slides.sustained_channels.is_empty());
        assert!(self.input_notes.lock().unwrap().is_empty());
        assert!(self.harmony_notes.lock().unwrap().is_empty());
        assert!(self.borrowed_notes.lock().unwrap().is_empty());
        assert!(self.canon_notes.lock().unwrap().is_empty());
        assert!(self.counterpoint_notes.lock().unwrap().is_empty());
        assert!(self.loop_input_notes.lock().unwrap().is_empty());
        assert!(self.loop_harmony_notes.lock().unwrap().is_empty());
        assert!(self.loop_canon_notes.lock().unwrap().is_empty());
        assert!(self.loop_counterpoint_notes.lock().unwrap().is_empty());
        assert_eq!(self.companion.phrase_snapshot().phase, PhrasePhase::Idle);
        assert_eq!(
            self.loop_companion.phrase_snapshot().phase,
            PhrasePhase::Idle
        );
        assert!(matches!(
            self.synth_rx.try_recv(),
            Err(mpsc::TryRecvError::Empty)
        ));
    }
}

fn rich_performance(block_size: u32) -> DeterminismTrace {
    let mut harness = PerformanceHarness::new(true);
    harness.live([0x90, 60, 102]);
    harness.advance_to(18_000, block_size);
    harness.live([0x91, 64, 91]);
    harness.live([0xb0, 64, 127]);
    harness.advance_to(36_000, block_size);
    harness.live([0x80, 60, 42]);
    harness.loop_event(LoopMidiEvent::NoteOn {
        note: 67,
        velocity: 88,
        channel: 2,
    });
    harness.advance_to(66_000, block_size);
    // Leave multiple live, loop, Canon, Counterpoint, and pattern owners active.
    // Panic must release them in one canonical order on every fresh runtime.
    harness.panic();
    harness.assert_clean();
    let final_ownership = harness.ownership_snapshot();
    DeterminismTrace {
        schema_version: TRACE_SCHEMA_VERSION,
        events: harness.trace,
        final_ownership,
    }
}

#[test]
fn full_performance_trace_is_identical_across_fresh_runtimes() {
    let expected = rich_performance(257);
    for run in 1..32 {
        assert_eq!(
            rich_performance(257),
            expected,
            "fresh runtime replay diverged on run {run}"
        );
    }
}

#[test]
fn semantic_trace_does_not_depend_on_audio_block_partitioning() {
    let expected = rich_performance(64);
    for block_size in [127, 257, 512, 1_024, 4_096, 16_384] {
        assert_eq!(
            rich_performance(block_size),
            expected,
            "semantic output changed at block size {block_size}"
        );
    }
}

#[test]
fn live_and_loop_same_pitch_release_only_after_the_last_owner() {
    let mut harness = PerformanceHarness::new(false);
    harness.live([0x90, 60, 100]);
    harness.loop_event(LoopMidiEvent::NoteOn {
        note: 60,
        velocity: 90,
        channel: 0,
    });
    harness.live([0x80, 60, 17]);

    assert_eq!(
        harness
            .trace
            .iter()
            .filter(|event| matches!(event, ObservedEvent::NoteOn { .. }))
            .count(),
        1
    );
    assert!(
        harness
            .trace
            .iter()
            .all(|event| !matches!(event, ObservedEvent::NoteOff { .. })),
        "first source release must not steal the shared sounding owner"
    );

    harness.loop_event(LoopMidiEvent::NoteOff {
        note: 60,
        velocity: 29,
        channel: 0,
    });
    assert_eq!(
        harness
            .trace
            .iter()
            .filter(|event| matches!(event, ObservedEvent::NoteOff { .. }))
            .count(),
        1
    );
    assert!(harness.output_notes.is_empty());
    assert!(harness.loop_output_notes.is_empty());
    assert!(harness.active_synth_owners.is_empty());
}

#[test]
fn repeated_same_pitch_releases_each_live_source_owner() {
    let mut harness = PerformanceHarness::new(false);
    harness
        .engine
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .set_mode(HarmonyMode::ContraryMotion);

    harness.live([0x90, 60, 100]);
    harness.live([0x90, 60, 90]);
    harness.live([0x80, 60, 21]);
    harness.live([0x80, 60, 17]);

    assert!(harness.output_notes.is_empty());
    assert!(harness.active_synth_owners.is_empty());
    let attacks = harness
        .trace
        .iter()
        .filter(|event| matches!(event, ObservedEvent::NoteOn { .. }))
        .count();
    let releases = harness
        .trace
        .iter()
        .filter(|event| matches!(event, ObservedEvent::NoteOff { .. }))
        .count();
    assert_eq!(releases, attacks, "every repeated attack needs one release");
}

#[test]
fn mpe_same_pitch_releases_the_generated_frame_owned_by_each_channel() {
    let mut harness = PerformanceHarness::new(false);
    harness
        .engine
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .set_mode(HarmonyMode::ContraryMotion);

    harness.live([0x90, 60, 100]);
    harness.live([0x91, 60, 90]);
    harness.live([0x81, 60, 21]);
    harness.live([0x80, 60, 17]);

    assert!(harness.output_notes.is_empty());
    assert!(harness.active_synth_owners.is_empty());
    let attacks = harness
        .trace
        .iter()
        .filter(|event| matches!(event, ObservedEvent::NoteOn { .. }))
        .count();
    let releases = harness
        .trace
        .iter()
        .filter(|event| matches!(event, ObservedEvent::NoteOff { .. }))
        .count();
    assert_eq!(
        releases, attacks,
        "every physical MPE attack needs one release"
    );
}

#[test]
fn loop_companion_owns_phrase_and_lane_state_independently_from_live_input() {
    let mut harness = PerformanceHarness::new(true);
    assert_eq!(harness.companion.phrase_snapshot().phase, PhrasePhase::Idle);
    assert_eq!(
        harness.loop_companion.phrase_snapshot().phase,
        PhrasePhase::Idle
    );

    harness.loop_event(LoopMidiEvent::NoteOn {
        note: 60,
        velocity: 90,
        channel: 2,
    });

    assert_eq!(harness.companion.phrase_snapshot().phase, PhrasePhase::Idle);
    assert_ne!(
        harness.loop_companion.phrase_snapshot().phase,
        PhrasePhase::Idle
    );
    harness.panic();
    harness.assert_clean();
}

#[test]
fn external_midi_owners_release_exactly_without_a_connected_device() {
    let mut harness = PerformanceHarness::new(false);
    harness
        .voice_outputs
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .set(VoiceRouteId::Input, VoiceOutputTarget::MidiPort { port: 0 });

    harness.live([0x90, 60, 100]);
    harness.live([0x91, 60, 90]);
    harness.live([0x81, 60, 19]);
    harness.live([0x80, 60, 17]);

    assert!(harness.output_notes.is_empty());
    assert!(harness.midi_slides.voices.is_empty());
    assert!(harness
        .midi_slides
        .last_bends
        .values()
        .all(|bend| *bend == 8_192));
}

#[test]
fn external_midi_wire_bytes_replay_identically_at_the_router_boundary() {
    fn trace() -> Vec<contrapunk::midi::output::MidiWireEvent> {
        let mut harness = PerformanceHarness::new(false);
        harness.output = OutputRouter::recording(&[9]);
        harness
            .voice_outputs
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .set(VoiceRouteId::Input, VoiceOutputTarget::MidiPort { port: 9 });

        harness.live([0x91, 60, 87]);
        harness.live([0x81, 60, 23]);
        assert!(harness.output_notes.is_empty());
        harness.output.take_trace()
    }

    let expected = trace();
    assert_eq!(trace(), expected);
    assert!(expected.iter().any(|event| event.message == [0x91, 60, 87]));
    assert!(expected.iter().any(|event| event.message == [0x81, 60, 23]));
    assert!(expected.iter().all(|event| event.device_port == 9));
}

#[test]
fn sustain_stays_down_until_live_and_loop_owners_release_it() {
    let mut harness = PerformanceHarness::new(false);
    harness.live([0xb3, 64, 127]);
    harness.loop_event(LoopMidiEvent::Cc64 {
        value: 127,
        channel: 3,
    });
    harness.live([0xb3, 64, 0]);
    assert_eq!(
        harness.trace,
        vec![ObservedEvent::Sustain(true)],
        "loop ownership must keep the shared synth pedal down"
    );

    harness.loop_event(LoopMidiEvent::Cc64 {
        value: 0,
        channel: 3,
    });
    assert_eq!(
        harness.trace,
        vec![ObservedEvent::Sustain(true), ObservedEvent::Sustain(false)]
    );
    assert!(harness.sustain_owners.is_empty());
}

#[test]
fn all_to_synth_override_does_not_leak_sustain_to_external_ports() {
    let mut harness = PerformanceHarness::new(false);
    harness.output = OutputRouter::recording(&[9]);
    {
        let mut routes = harness
            .voice_outputs
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        routes.set(VoiceRouteId::Input, VoiceOutputTarget::MidiPort { port: 9 });
        routes.set_all_to_synth(true);
    }

    harness.live([0xb0, 64, 127]);
    harness.live([0xb0, 64, 0]);

    assert!(harness.output.take_trace().is_empty());
    assert_eq!(
        harness.trace,
        [ObservedEvent::Sustain(true), ObservedEvent::Sustain(false)]
    );
}

#[test]
fn panic_at_every_performance_prefix_clears_all_runtime_lifecycle_state() {
    #[derive(Clone, Copy)]
    enum Step {
        Live([u8; 3]),
        Loop(LoopMidiEvent),
        Advance(u64),
    }

    let steps = [
        Step::Live([0x90, 60, 100]),
        Step::Advance(6_000),
        Step::Live([0x91, 64, 96]),
        Step::Live([0xb0, 64, 127]),
        Step::Advance(18_000),
        Step::Loop(LoopMidiEvent::NoteOn {
            note: 60,
            velocity: 84,
            channel: 0,
        }),
        Step::Loop(LoopMidiEvent::Cc64 {
            value: 127,
            channel: 0,
        }),
        Step::Advance(36_000),
        Step::Live([0x80, 60, 0]),
        Step::Advance(54_000),
    ];

    for prefix in 0..=steps.len() {
        let mut harness = PerformanceHarness::new(true);
        for step in &steps[..prefix] {
            match *step {
                Step::Live(bytes) => harness.live(bytes),
                Step::Loop(event) => harness.loop_event(event),
                Step::Advance(sample) => harness.advance_to(sample, 257),
            }
        }
        harness.panic();
        let after_panic = harness.trace.len();
        harness.assert_clean();

        // Advancing a full two bars after cleanup must not resurrect queued
        // Canon, Counterpoint, pattern, loop, or sustain-owned output.
        let target = harness.transport.sample_pos() + 192_000;
        harness.advance_to(target, 1_024);
        harness.drain_synth_events();
        assert!(
            harness.trace[after_panic..].is_empty(),
            "prefix {prefix} emitted delayed output after panic: {:?}",
            &harness.trace[after_panic..]
        );
        harness.assert_clean();
    }
}
