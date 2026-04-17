//! Tauri commands for engine routing control and real-time note state.
//!
//! Handles starting/stopping the MIDI router thread and emitting
//! real-time note-update events to the frontend.

use std::collections::HashSet;
use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use serde::Serialize;
use tauri::{AppHandle, Emitter, State};
use wmidi::{Channel, MidiMessage, Note, Velocity};

use contrapunk::audio::guitar_input::GuitarInputConfig;
use contrapunk::audio_out::{MidiEvent, MidiProducer};
use contrapunk::chord::chord_display_with_analysis;
use contrapunk::generator::{GeneratorEvent, NoteGenerator};
use contrapunk::harmony::HarmonyEngine;
use contrapunk::humanize::{DelayQueue, HumanizeConfig, HumanizedNote, Humanizer};
use contrapunk::midi::input::connect_input;
use contrapunk::midi::output::OutputRouter;

use crate::guitar_bridge::GuitarBridge;
use crate::state::AppState;

/// Sentinel value for the "Guitar Audio" virtual input.
/// When `input_idx` equals this value, we spawn a GuitarBridge
/// instead of connecting to a physical MIDI input port.
/// Virtual input sentinels — must match the values in MidiDevices.svelte.
/// Using small memorable values instead of MAX_SAFE_INTEGER to avoid
/// cross-language integer size mismatches.
const VIRTUAL_NOTE_GENERATOR: usize = 999_999;
const VIRTUAL_COMPUTER_KEYBOARD: usize = 999_998;
const GUITAR_AUDIO_SENTINEL: usize = 999_997;

/// Payload for the "note-update" Tauri event.
#[derive(Clone, Serialize)]
pub struct NoteUpdatePayload {
    pub input_notes: Vec<u8>,
    pub harmony_notes: Vec<u8>,
    pub borrowed_notes: Vec<u8>,
    pub chord_name: String,
    pub last_borrowed_from: String,
    pub current_key: String,
}

/// Payload for the "beat-update" Tauri event (replaces JS setInterval approximation).
#[derive(Clone, Serialize)]
pub struct BeatUpdatePayload {
    pub beat_position: f64,
    pub beat_number: u8,
    pub bpm: f64,
    pub running: bool,
}

/// Payload for the "guitar-signal" Tauri event (UI signal feedback).
#[derive(Clone, Serialize)]
pub struct GuitarSignalPayload {
    pub rms: f32,
    pub frequency: Option<f32>,
    pub clarity: f32,
    pub note_state: u8,
    pub note_name: String,
    pub midi_note: u8,
}

/// Returns the current note state snapshot.
#[tauri::command]
pub fn get_note_state(state: State<AppState>) -> Result<NoteUpdatePayload, String> {
    let input = state.input_notes.lock().map_err(|e| e.to_string())?;
    let harmony = state.harmony_notes.lock().map_err(|e| e.to_string())?;
    let borrowed = state.borrowed_notes.lock().map_err(|e| e.to_string())?;
    let chord = state.chord_name.lock().map_err(|e| e.to_string())?;

    let engine = state.engine.lock().map_err(|e| e.to_string())?;
    Ok(NoteUpdatePayload {
        input_notes: input.iter().copied().collect(),
        harmony_notes: harmony.iter().copied().collect(),
        borrowed_notes: borrowed.iter().copied().collect(),
        chord_name: chord.clone(),
        last_borrowed_from: String::new(),
        current_key: format!("{}", engine.key()),
    })
}

/// Starts MIDI routing from the specified input to the specified outputs.
///
/// Spawns a background router thread that processes MIDI messages through
/// the harmony engine and emits "note-update" events at ~30fps.
#[tauri::command]
pub fn start_routing(
    input_idx: usize,
    output_indices: Vec<usize>,
    app_handle: AppHandle,
    state: State<AppState>,
) -> Result<(), String> {
    // Check if already running
    if state.is_running.load(Ordering::SeqCst) {
        return Err("Routing is already active".to_string());
    }

    if output_indices.is_empty() {
        return Err("At least one output port required".to_string());
    }

    // Capture engine config for the router thread
    let engine_config = {
        let engine = state.engine.lock().map_err(|e| e.to_string())?;
        (
            engine.key(),
            engine.mode(),
            engine.octave_mode(),
            engine.voice_leading_enabled(),
            engine.voice_leading_style(),
            engine.scale_mode(),
            engine.interchange_enabled(),
            engine.borrowing_range(),
            engine.voice_position(),
            engine.counterpoint_species(),
            engine.counterpoint_strictness(),
        )
    };

    // Share the humanize config with the router thread so live UI
    // changes (metronome toggle, swing, BPM) take effect immediately.
    let humanize_config = Arc::clone(&state.humanize_config);

    // Capture generator config for the router thread.
    // The generator lives inside the router loop (like the harmony engine)
    // so live UI changes require stop+restart of routing.
    let generator_config = {
        let gen = state.generator.lock().map_err(|e| e.to_string())?;
        (
            gen.enabled(),
            gen.mode().clone(),
            gen.selected_notes().to_vec(),
            gen.velocity(),
            gen.note_duration_beats(),
        )
    };

    let routing_mode = { *state.routing_mode.lock().map_err(|e| e.to_string())? };

    // Capture guitar config for the router thread
    let is_guitar = input_idx == GUITAR_AUDIO_SENTINEL;
    let guitar_device = {
        state
            .guitar_device
            .lock()
            .map_err(|e| e.to_string())?
            .clone()
    };
    let guitar_channel = { *state.guitar_channel.lock().map_err(|e| e.to_string())? };
    let guitar_config = {
        state
            .guitar_config
            .lock()
            .map_err(|e| e.to_string())?
            .clone()
            .unwrap_or_default()
    };

    // Shared state for note updates
    let input_notes = Arc::new(Mutex::new(HashSet::<u8>::new()));
    let harmony_notes = Arc::new(Mutex::new(HashSet::<u8>::new()));
    let borrowed_notes = Arc::new(Mutex::new(HashSet::<u8>::new()));
    let chord_name = Arc::new(Mutex::new(String::new()));
    let stop_signal = Arc::new(std::sync::atomic::AtomicBool::new(false));

    // Stop any previous router thread before starting a new one
    if let Ok(mut prev_stop) = state.stop_signal.lock() {
        if let Some(prev) = prev_stop.take() {
            prev.store(true, Ordering::SeqCst);
        }
    }

    // Store the new stop signal so stop_routing can use it
    if let Ok(mut sig) = state.stop_signal.lock() {
        *sig = Some(Arc::clone(&stop_signal));
    }

    // Store references in AppState for stop_routing and get_note_state
    {
        let mut app_input = state.input_notes.lock().map_err(|e| e.to_string())?;
        app_input.clear();
        let mut app_harmony = state.harmony_notes.lock().map_err(|e| e.to_string())?;
        app_harmony.clear();
        let mut app_borrowed = state.borrowed_notes.lock().map_err(|e| e.to_string())?;
        app_borrowed.clear();
        let mut app_chord = state.chord_name.lock().map_err(|e| e.to_string())?;
        app_chord.clear();
    }

    state.is_running.store(true, Ordering::SeqCst);

    // Clone Arcs for the router thread
    let in_notes = Arc::clone(&input_notes);
    let harm_notes = Arc::clone(&harmony_notes);
    let borr_notes = Arc::clone(&borrowed_notes);
    let ch_name = Arc::clone(&chord_name);
    let stop = Arc::clone(&stop_signal);
    let output_indices_clone = output_indices.clone();

    // Share the audio-out producer slot with the router thread so
    // toggling audio output doesn't require stop+restart routing.
    let audio_out_slot = Arc::clone(&state.audio_out_producer);

    let detune = Arc::clone(&state.detune_cents);

    // Spawn router thread
    thread::spawn(move || {
        if let Err(e) = run_tauri_router(
            input_idx,
            &output_indices_clone,
            engine_config,
            humanize_config,
            routing_mode,
            is_guitar,
            guitar_device,
            guitar_channel,
            guitar_config,
            generator_config,
            in_notes,
            harm_notes,
            borr_notes,
            ch_name,
            stop,
            app_handle,
            audio_out_slot,
            detune,
        ) {
            eprintln!("[tauri-router] Error: {}", e);
        }
    });

    Ok(())
}

/// Stops the currently active MIDI routing.
#[tauri::command]
pub fn stop_routing(state: State<AppState>) -> Result<(), String> {
    if !state.is_running.load(Ordering::SeqCst) {
        return Err("Routing is not active".to_string());
    }

    // Signal the router thread to stop
    if let Ok(mut sig) = state.stop_signal.lock() {
        if let Some(stop) = sig.take() {
            stop.store(true, Ordering::SeqCst);
        }
    }

    state.is_running.store(false, Ordering::SeqCst);

    // Clear note state
    if let Ok(mut notes) = state.input_notes.lock() {
        notes.clear();
    }
    if let Ok(mut notes) = state.harmony_notes.lock() {
        notes.clear();
    }
    if let Ok(mut notes) = state.borrowed_notes.lock() {
        notes.clear();
    }
    if let Ok(mut name) = state.chord_name.lock() {
        name.clear();
    }

    Ok(())
}

// ============================================================================
// Router thread implementation
// ============================================================================

type EngineConfig = (
    contrapunk::harmony::Key,
    contrapunk::harmony::HarmonyMode,
    contrapunk::harmony::OctaveMode,
    bool, // voice_leading_enabled
    contrapunk::harmony::VoiceLeadingStyle,
    contrapunk::harmony::ScaleMode,
    bool,  // interchange_enabled
    u8,    // borrowing_range
    usize, // voice_position
    contrapunk::harmony::CounterpointSpecies,
    contrapunk::harmony::CounterpointStrictness,
);

type GeneratorConfig = (
    bool,                                 // enabled
    contrapunk::generator::GeneratorMode, // mode
    Vec<wmidi::Note>,                     // selected_notes
    u8,                                   // velocity
    f64,                                  // note_duration_beats
);

fn run_tauri_router(
    input_port: usize,
    output_ports: &[usize],
    config: EngineConfig,
    humanize_config: Arc<Mutex<HumanizeConfig>>,
    routing_mode: contrapunk::harmony::RoutingMode,
    is_guitar: bool,
    guitar_device: String,
    guitar_channel: usize,
    guitar_config: GuitarInputConfig,
    generator_config: GeneratorConfig,
    input_notes: Arc<Mutex<HashSet<u8>>>,
    harmony_notes: Arc<Mutex<HashSet<u8>>>,
    borrowed_notes: Arc<Mutex<HashSet<u8>>>,
    chord_name: Arc<Mutex<String>>,
    stop_signal: Arc<std::sync::atomic::AtomicBool>,
    app_handle: AppHandle,
    audio_out_slot: Arc<Mutex<Option<MidiProducer>>>,
    detune_cents: Arc<AtomicI32>,
) -> anyhow::Result<()> {
    let (
        key,
        mode,
        octave_mode,
        vl_enabled,
        vl_style,
        scale_mode,
        ic_enabled,
        br_range,
        vp,
        cp_species,
        cp_strictness,
    ) = config;

    // Create channel for MIDI input — both physical MIDI and guitar
    // bridge send Vec<u8> through the same channel.
    let (tx, rx) = mpsc::channel::<Vec<u8>>();

    // Connect to either Guitar Audio bridge or physical MIDI input
    let _midi_conn;
    let _guitar_bridge;

    // Signal channel for guitar UI feedback (only used in guitar mode)
    let (signal_tx, signal_rx) = mpsc::channel::<crate::guitar_bridge::GuitarSignalInfo>();

    if is_guitar {
        // Guitar Audio mode: spawn cpal capture -> DSP -> same tx channel
        let bridge = GuitarBridge::new(
            &guitar_device,
            guitar_channel,
            guitar_config,
            tx,
            Some(signal_tx),
        )
        .map_err(|e| anyhow::anyhow!("Guitar bridge error: {}", e))?;
        bridge
            .start()
            .map_err(|e| anyhow::anyhow!("Guitar bridge start error: {}", e))?;
        _guitar_bridge = Some(bridge);
        _midi_conn = None;
    } else {
        // Physical MIDI mode: existing behavior
        _midi_conn = Some(connect_input(input_port, tx)?);
        _guitar_bridge = None;
    };

    // Create output router
    let mut output_router = OutputRouter::new(output_ports)?;
    let num_outputs = output_router.connection_count();

    // Create harmony engine
    let mut engine = HarmonyEngine::new(key, mode);
    engine.set_voice_count(num_outputs);
    engine.set_octave_mode(octave_mode);
    engine.set_voice_leading_enabled(vl_enabled);
    engine.set_voice_leading_style(vl_style);
    engine.set_scale_mode(scale_mode);
    engine.set_interchange_enabled(ic_enabled);
    engine.set_borrowing_range(br_range);
    engine.set_voice_position(vp);
    engine.set_counterpoint_species(cp_species);
    engine.set_counterpoint_strictness(cp_strictness);

    // Create humanizer + metronome from the shared config.
    let initial_hconfig = humanize_config
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .clone();
    let mut humanizer = Humanizer::new(initial_hconfig);
    let mut delay_queue = DelayQueue::new();
    let mut metronome = contrapunk::humanize::Metronome::new();
    metronome.enabled = humanizer.config().metronome_enabled;
    let epoch = Instant::now();
    let now_ms = || epoch.elapsed().as_secs_f64() * 1000.0;

    humanizer.clock_mut().start(now_ms());

    // Create note generator from captured config
    let (gen_enabled, gen_mode, gen_notes, gen_velocity, gen_duration) = generator_config;
    let mut generator = NoteGenerator::new();
    generator.set_mode(gen_mode);
    generator.set_enabled(gen_enabled);
    generator.set_selected_notes(gen_notes);
    generator.set_velocity(gen_velocity);
    generator.set_note_duration_beats(gen_duration);

    // Event emission timer (~30fps)
    let mut last_emit = Instant::now();
    let emit_interval = Duration::from_millis(33);

    // Detune: track the previous value so we only send pitch bend on change.
    let mut prev_detune_cents: i32 = detune_cents.load(Ordering::Relaxed);

    // Take initial audio-out producer if available. The router owns it
    // for the duration; hot-swap happens below if audio-out is toggled.
    let mut audio_out: Option<MidiProducer> = {
        audio_out_slot
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .take()
    };

    // Pending metronome NoteOff: (midi_note, fire_at_ms). Delayed so the
    // PolySynth envelope has time to rise before release — pushing NoteOn
    // and NoteOff in the same frame produces silence.
    let mut pending_metro_off: Option<(u8, f64)> = None;

    // Main routing loop
    loop {
        if stop_signal.load(Ordering::SeqCst) {
            break;
        }

        // Fire pending metronome NoteOff if its time has come.
        if let Some((note, fire_at)) = pending_metro_off {
            if now_ms() >= fire_at {
                if let Some(ref mut producer) = audio_out {
                    let _ = producer.push(MidiEvent::NoteOff { voice: 255, note });
                }
                pending_metro_off = None;
            }
        }

        // Hot-swap audio-out producer: if the slot has a new producer
        // (audio-out was started/restarted), take it. If our producer
        // is dead (audio-out was stopped and consumer dropped), the
        // pushes silently fail until a new producer appears.
        if let Ok(mut slot) = audio_out_slot.try_lock() {
            if slot.is_some() {
                // New producer available — swap in
                audio_out = slot.take();
            }
        }

        // Poll for humanize config changes from the UI (metronome toggle,
        // BPM, swing, etc.). Non-blocking: skip if the lock is held.
        if let Ok(shared) = humanize_config.try_lock() {
            humanizer.update_config(shared.clone());
        }

        // Tick humanizer
        let current_ms = now_ms();
        humanizer.tick(current_ms);

        // Push the current beat-phase position into the harmony engine.
        engine.set_counterpoint_beat_phase(Some(humanizer.clock().beat_position()));

        // Emit beat-update event on every beat crossing so the UI gets
        // Rust-driven timing instead of a drifting JS setInterval.
        if let Some(beat_num) = humanizer.clock().beat_crossed() {
            let _ = app_handle.emit(
                "beat-update",
                BeatUpdatePayload {
                    beat_position: humanizer.clock().beat_position(),
                    beat_number: beat_num,
                    bpm: humanizer.config().bpm,
                    running: humanizer.clock().running,
                },
            );
        }

        // Generate metronome clicks on subdivision crossings and send
        // to the configured output port (or port 0 if unset).
        // Also push to the audio synth so it clicks through speakers
        // without requiring an external GM drum kit on channel 10.
        if let Some(crossing) = humanizer.clock().subdivision_crossed() {
            metronome.enabled = humanizer.config().metronome_enabled;
            if let Some(click_bytes) =
                metronome.generate_click_for_crossing(&crossing, humanizer.config())
            {
                let metro_port = humanizer.config().metronome_output_port.unwrap_or(0);
                let _ = output_router.send_to_port(metro_port, &click_bytes);

                // Audio-out: short sine click through PolySynth.
                // Accent (beat 0) = C7 (96) loud, others = G6 (91) softer.
                if let Some(ref mut producer) = audio_out {
                    // Release any previous click first
                    if let Some((prev_note, _)) = pending_metro_off.take() {
                        let _ = producer.push(MidiEvent::NoteOff {
                            voice: 255,
                            note: prev_note,
                        });
                    }
                    let (click_note, click_vel) = if crossing.sixteenth == 0 && crossing.beat == 0 {
                        (96u8, 120u8) // accent: C7
                    } else if crossing.sixteenth == 0 {
                        (91u8, 90u8) // normal beat: G6
                    } else {
                        (98u8, 60u8) // subdivision: D7, quiet
                    };
                    let _ = producer.push(MidiEvent::NoteOn {
                        voice: 255,
                        note: click_note,
                        velocity: click_vel,
                    });
                    // Schedule NoteOff 50ms later so the envelope has time to rise
                    pending_metro_off = Some((click_note, now_ms() + 50.0));
                }
            }
        }

        // Apply detune as MIDI pitch bend when the value changes.
        let current_detune = detune_cents.load(Ordering::Relaxed);
        if current_detune != prev_detune_cents {
            prev_detune_cents = current_detune;
            // Convert cents to 14-bit pitch bend (center = 8192, ±2 semitones = ±200 cents)
            let max_cents = 200i32; // standard ±2 semitone range
            let bend_14bit = ((current_detune as f64 / max_cents as f64) * 8192.0 + 8192.0) as u16;
            let bend_clamped = bend_14bit.clamp(0, 16383);
            let lsb = (bend_clamped & 0x7F) as u8;
            let msb = ((bend_clamped >> 7) & 0x7F) as u8;
            let pitch_bend_msg = [0xE0, lsb, msb]; // channel 0
            let num_ports = output_router.connection_count();
            for p in 0..num_ports {
                let _ = output_router.send_to_port(p, &pitch_bend_msg);
            }
        }

        // Drain delay queue — fanout to audio synth for delayed harmony notes.
        for hn in delay_queue.drain_ready(current_ms) {
            let _ =
                send_humanized_note(&hn, &mut output_router, audio_out.as_mut(), hn.voice_index);
        }

        // Tick the note generator and route any resulting events through
        // the harmony engine and output router.
        let beat_pos = humanizer.clock().beat_position();
        let bpm = humanizer.config().bpm;
        let gen_events = generator.tick(beat_pos, bpm);
        for event in gen_events {
            match event {
                GeneratorEvent::NoteOn(note, velocity) => {
                    let vel = Velocity::try_from(velocity.clamp(1, 127)).unwrap();
                    let notes = engine.harmonize_note_on(note);
                    let num_outputs = output_router.connection_count();

                    // Update shared input note state
                    {
                        let mut in_notes = input_notes.lock().unwrap();
                        in_notes.insert(note as u8);
                    }
                    {
                        let mut harm_notes = harmony_notes.lock().unwrap();
                        for &n in notes.iter().skip(1) {
                            harm_notes.insert(n as u8);
                        }
                    }
                    if engine.last_borrowed_from().is_some() {
                        let mut borr = borrowed_notes.lock().unwrap();
                        for &n in notes.iter().skip(1) {
                            borr.insert(n as u8);
                        }
                    }

                    // Route through outputs (channel-based, matching routing_mode)
                    let port_map = engine.last_port_map();
                    match routing_mode {
                        contrapunk::harmony::RoutingMode::ChannelBased => {
                            for (i, &n) in notes.iter().enumerate() {
                                let voice_channel = match i + 1 {
                                    1 => Channel::Ch2,
                                    2 => Channel::Ch3,
                                    3 => Channel::Ch4,
                                    4 => Channel::Ch5,
                                    5 => Channel::Ch6,
                                    6 => Channel::Ch7,
                                    _ => Channel::Ch8,
                                };
                                let msg = MidiMessage::NoteOn(voice_channel, n, vel);
                                let mut buf = vec![0u8; msg.bytes_size()];
                                let _ = msg.copy_to_slice(&mut buf);
                                let _ = output_router.send_to_first(&buf);
                                if let Some(ref mut producer) = audio_out {
                                    let _ = producer.push(MidiEvent::NoteOn {
                                        voice: i as u8,
                                        note: u8::from(n),
                                        velocity,
                                    });
                                }
                            }
                        }
                        contrapunk::harmony::RoutingMode::PortBased => {
                            for (i, &n) in notes.iter().enumerate() {
                                let port = if i < port_map.len() { port_map[i] } else { i };
                                if port >= num_outputs {
                                    continue;
                                }
                                let msg = MidiMessage::NoteOn(Channel::Ch1, n, vel);
                                let mut buf = vec![0u8; msg.bytes_size()];
                                let _ = msg.copy_to_slice(&mut buf);
                                let _ = output_router.send_to_port(port, &buf);
                                if let Some(ref mut producer) = audio_out {
                                    let _ = producer.push(MidiEvent::NoteOn {
                                        voice: i as u8,
                                        note: u8::from(n),
                                        velocity,
                                    });
                                }
                            }
                        }
                    }
                }
                GeneratorEvent::NoteOff(note) => {
                    let notes = engine.harmonize_note_off(note);
                    let num_outputs = output_router.connection_count();

                    // Update shared note state
                    {
                        let mut in_notes = input_notes.lock().unwrap();
                        in_notes.remove(&(note as u8));
                    }
                    {
                        let mut harm_notes = harmony_notes.lock().unwrap();
                        let mut borr = borrowed_notes.lock().unwrap();
                        for &n in notes.iter().skip(1) {
                            harm_notes.remove(&(n as u8));
                            borr.remove(&(n as u8));
                        }
                    }

                    let port_map = engine.last_port_map();
                    let vel = Velocity::MIN;
                    match routing_mode {
                        contrapunk::harmony::RoutingMode::ChannelBased => {
                            for (i, &n) in notes.iter().enumerate() {
                                let voice_channel = match i + 1 {
                                    1 => Channel::Ch2,
                                    2 => Channel::Ch3,
                                    3 => Channel::Ch4,
                                    4 => Channel::Ch5,
                                    5 => Channel::Ch6,
                                    6 => Channel::Ch7,
                                    _ => Channel::Ch8,
                                };
                                let msg = MidiMessage::NoteOff(voice_channel, n, vel);
                                let mut buf = vec![0u8; msg.bytes_size()];
                                let _ = msg.copy_to_slice(&mut buf);
                                let _ = output_router.send_to_first(&buf);
                                if let Some(ref mut producer) = audio_out {
                                    let _ = producer.push(MidiEvent::NoteOff {
                                        voice: i as u8,
                                        note: u8::from(n),
                                    });
                                }
                            }
                        }
                        contrapunk::harmony::RoutingMode::PortBased => {
                            for (i, &n) in notes.iter().enumerate() {
                                let port = if i < port_map.len() { port_map[i] } else { i };
                                if port >= num_outputs {
                                    continue;
                                }
                                let msg = MidiMessage::NoteOff(Channel::Ch1, n, vel);
                                let mut buf = vec![0u8; msg.bytes_size()];
                                let _ = msg.copy_to_slice(&mut buf);
                                let _ = output_router.send_to_port(port, &buf);
                                if let Some(ref mut producer) = audio_out {
                                    let _ = producer.push(MidiEvent::NoteOff {
                                        voice: i as u8,
                                        note: u8::from(n),
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }

        // Process MIDI messages
        match rx.recv_timeout(Duration::from_millis(5)) {
            Ok(message) => {
                let current_ms = now_ms();
                process_midi_message(
                    &message,
                    &mut engine,
                    &mut output_router,
                    &mut humanizer,
                    &mut delay_queue,
                    current_ms,
                    &input_notes,
                    &harmony_notes,
                    &borrowed_notes,
                    &chord_name,
                    routing_mode,
                    audio_out.as_mut(),
                );
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {}
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                eprintln!("[tauri-router] Input channel disconnected");
                break;
            }
        }

        // Emit note-update event at ~30fps
        if last_emit.elapsed() >= emit_interval {
            last_emit = Instant::now();
            // Recover from poisoned mutexes instead of panicking the router
            // thread. A poisoned lock means a thread panicked while holding
            // it — the data is likely still usable, and silently crashing
            // the emit loop leaves the UI stuck with no user-visible error.
            let payload = {
                let in_notes = input_notes.lock().unwrap_or_else(|e| e.into_inner());
                let harm_notes = harmony_notes.lock().unwrap_or_else(|e| e.into_inner());
                let borr_notes = borrowed_notes.lock().unwrap_or_else(|e| e.into_inner());
                let ch_name = chord_name.lock().unwrap_or_else(|e| e.into_inner());
                let mut in_vec: Vec<u8> = in_notes.iter().copied().collect();
                let mut harm_vec: Vec<u8> = harm_notes.iter().copied().collect();
                let mut borr_vec: Vec<u8> = borr_notes.iter().copied().collect();
                in_vec.sort_unstable();
                harm_vec.sort_unstable();
                borr_vec.sort_unstable();
                NoteUpdatePayload {
                    input_notes: in_vec,
                    harmony_notes: harm_vec,
                    borrowed_notes: borr_vec,
                    chord_name: ch_name.clone(),
                    last_borrowed_from: engine
                        .last_borrowed_from()
                        .map(|m| format!("{}", m))
                        .unwrap_or_default(),
                    current_key: format!("{}", engine.key()),
                }
            };
            let _ = app_handle.emit("note-update", payload);

            // Emit guitar signal info for UI (drain latest from channel)
            if is_guitar {
                let mut latest_signal = None;
                while let Ok(sig) = signal_rx.try_recv() {
                    latest_signal = Some(sig);
                }
                if let Some(sig) = latest_signal {
                    let note_names = [
                        "C", "C#", "D", "Eb", "E", "F", "F#", "G", "Ab", "A", "Bb", "B",
                    ];
                    let (note_name, midi_note) = if let Some(freq) = sig.frequency {
                        if freq > 20.0 && sig.clarity > 0.3 {
                            let midi = (12.0 * (freq / 440.0).log2() + 69.0).round() as i32;
                            let midi_u8 = midi.clamp(0, 127) as u8;
                            let name_idx = (midi_u8 % 12) as usize;
                            let octave = (midi_u8 as i32 / 12) - 1;
                            (format!("{}{}", note_names[name_idx], octave), midi_u8)
                        } else {
                            (String::new(), 0)
                        }
                    } else {
                        (String::new(), 0)
                    };
                    let _ = app_handle.emit(
                        "guitar-signal",
                        GuitarSignalPayload {
                            rms: sig.rms,
                            frequency: sig.frequency,
                            clarity: sig.clarity,
                            note_state: sig.note_state,
                            note_name,
                            midi_note,
                        },
                    );
                }
            }
        }
    }

    // Clear note state on exit
    if let Ok(mut notes) = input_notes.lock() {
        notes.clear();
    }
    if let Ok(mut notes) = harmony_notes.lock() {
        notes.clear();
    }
    if let Ok(mut notes) = borrowed_notes.lock() {
        notes.clear();
    }

    Ok(())
}

fn process_midi_message(
    bytes: &[u8],
    engine: &mut HarmonyEngine,
    output: &mut OutputRouter,
    humanizer: &mut Humanizer,
    delay_queue: &mut DelayQueue,
    now_ms: f64,
    input_notes: &Arc<Mutex<HashSet<u8>>>,
    harmony_notes: &Arc<Mutex<HashSet<u8>>>,
    borrowed_notes: &Arc<Mutex<HashSet<u8>>>,
    chord_name: &Arc<Mutex<String>>,
    routing_mode: contrapunk::harmony::RoutingMode,
    mut audio_out: Option<&mut MidiProducer>,
) {
    let msg = match MidiMessage::try_from(bytes) {
        Ok(m) => m,
        Err(_) => {
            let _ = output.send_to_first(bytes);
            return;
        }
    };

    match msg {
        MidiMessage::NoteOn(channel, note, velocity) => {
            if velocity == Velocity::MIN {
                handle_note_off(
                    channel,
                    note,
                    velocity,
                    engine,
                    output,
                    humanizer,
                    delay_queue,
                    now_ms,
                    input_notes,
                    harmony_notes,
                    borrowed_notes,
                    chord_name,
                    routing_mode,
                    audio_out.as_deref_mut(),
                );
            } else {
                handle_note_on(
                    channel,
                    note,
                    velocity,
                    engine,
                    output,
                    humanizer,
                    delay_queue,
                    now_ms,
                    input_notes,
                    harmony_notes,
                    borrowed_notes,
                    chord_name,
                    routing_mode,
                    audio_out.as_deref_mut(),
                );
            }
        }
        MidiMessage::NoteOff(channel, note, velocity) => {
            handle_note_off(
                channel,
                note,
                velocity,
                engine,
                output,
                humanizer,
                delay_queue,
                now_ms,
                input_notes,
                harmony_notes,
                borrowed_notes,
                chord_name,
                routing_mode,
                audio_out.as_deref_mut(),
            );
        }
        _ => {
            let _ = output.send_to_first(bytes);
        }
    }
}

fn handle_note_on(
    channel: Channel,
    note: Note,
    velocity: Velocity,
    engine: &mut HarmonyEngine,
    output: &mut OutputRouter,
    humanizer: &mut Humanizer,
    delay_queue: &mut DelayQueue,
    now_ms: f64,
    input_notes: &Arc<Mutex<HashSet<u8>>>,
    harmony_notes: &Arc<Mutex<HashSet<u8>>>,
    borrowed_notes: &Arc<Mutex<HashSet<u8>>>,
    chord_name: &Arc<Mutex<String>>,
    routing_mode: contrapunk::harmony::RoutingMode,
    mut audio_out: Option<&mut MidiProducer>,
) {
    let notes = engine.harmonize_note_on(note);
    let num_outputs = output.connection_count();

    // Update shared state — recover from poisoned mutexes rather than panic.
    {
        let mut in_notes = input_notes.lock().unwrap_or_else(|e| e.into_inner());
        in_notes.insert(note as u8);
    }
    {
        let mut harm_notes = harmony_notes.lock().unwrap_or_else(|e| e.into_inner());
        for &n in notes.iter().skip(1) {
            harm_notes.insert(n as u8);
        }
    }
    if engine.last_borrowed_from().is_some() {
        let mut borr = borrowed_notes.lock().unwrap_or_else(|e| e.into_inner());
        for &n in notes.iter().skip(1) {
            borr.insert(n as u8);
        }
    }

    {
        let all_sounding: HashSet<u8> = {
            let in_notes = input_notes.lock().unwrap_or_else(|e| e.into_inner());
            let harm_notes = harmony_notes.lock().unwrap_or_else(|e| e.into_inner());
            in_notes.union(&harm_notes).copied().collect()
        };
        if !all_sounding.is_empty() {
            let key_tonic = Some(engine.key().semitones_from_c());
            let display = chord_display_with_analysis(&all_sounding, key_tonic);
            let mut ch = chord_name.lock().unwrap_or_else(|e| e.into_inner());
            *ch = display;
        }
    }

    // Send notes to outputs based on routing mode
    let port_map = engine.last_port_map();
    match routing_mode {
        contrapunk::harmony::RoutingMode::ChannelBased => {
            // MPE: all voices on port 0, each on its own MIDI channel.
            // Ch 1 (index 0) = master, Ch 2+ = voices (melody first, then harmonies)
            for (i, &n) in notes.iter().enumerate() {
                // Voice i → MIDI channel i+1 (channel 2 = melody, 3 = harm1, etc.)
                // Channel enum is 0-indexed: Ch2 = Channel::Ch2
                let voice_channel = match i + 1 {
                    1 => Channel::Ch2,
                    2 => Channel::Ch3,
                    3 => Channel::Ch4,
                    4 => Channel::Ch5,
                    5 => Channel::Ch6,
                    6 => Channel::Ch7,
                    _ => Channel::Ch8,
                };

                if i == 0 {
                    let msg = MidiMessage::NoteOn(voice_channel, n, velocity);
                    let mut buf = vec![0u8; msg.bytes_size()];
                    let _ = msg.copy_to_slice(&mut buf);
                    let _ = output.send_to_first(&buf);
                    // Fanout to audio synth queue (fire-and-forget).
                    if let Some(ref mut producer) = audio_out {
                        let _ = producer.push(MidiEvent::NoteOn {
                            voice: i as u8,
                            note: u8::from(n),
                            velocity: u8::from(velocity),
                        });
                    }
                } else {
                    let hn = humanizer.humanize_note_on(n, voice_channel, velocity, 0, i as u8);
                    if hn.delay_ms == 0 {
                        let _ = send_humanized_note(
                            &hn,
                            output,
                            audio_out.as_deref_mut(),
                            hn.voice_index,
                        );
                    } else {
                        delay_queue.push(hn, now_ms);
                    }
                }
            }
        }
        contrapunk::harmony::RoutingMode::PortBased => {
            for (i, &n) in notes.iter().enumerate() {
                let port = if i < port_map.len() { port_map[i] } else { i };
                if port >= num_outputs {
                    continue;
                }

                if i == 0 {
                    let msg = MidiMessage::NoteOn(channel, n, velocity);
                    let mut buf = vec![0u8; msg.bytes_size()];
                    let _ = msg.copy_to_slice(&mut buf);
                    let _ = output.send_to_port(port, &buf);
                    if let Some(ref mut producer) = audio_out {
                        let _ = producer.push(MidiEvent::NoteOn {
                            voice: i as u8,
                            note: u8::from(n),
                            velocity: u8::from(velocity),
                        });
                    }
                } else {
                    let hn = humanizer.humanize_note_on(n, channel, velocity, port, i as u8);
                    if hn.delay_ms == 0 {
                        let _ = send_humanized_note(
                            &hn,
                            output,
                            audio_out.as_deref_mut(),
                            hn.voice_index,
                        );
                    } else {
                        delay_queue.push(hn, now_ms);
                    }
                }
            }
        }
    }
}

fn handle_note_off(
    channel: Channel,
    note: Note,
    velocity: Velocity,
    engine: &mut HarmonyEngine,
    output: &mut OutputRouter,
    humanizer: &mut Humanizer,
    delay_queue: &mut DelayQueue,
    now_ms: f64,
    input_notes: &Arc<Mutex<HashSet<u8>>>,
    harmony_notes: &Arc<Mutex<HashSet<u8>>>,
    borrowed_notes: &Arc<Mutex<HashSet<u8>>>,
    _chord_name: &Arc<Mutex<String>>,
    routing_mode: contrapunk::harmony::RoutingMode,
    mut audio_out: Option<&mut MidiProducer>,
) {
    let notes = engine.harmonize_note_off(note);
    let num_outputs = output.connection_count();

    {
        let mut in_notes = input_notes.lock().unwrap_or_else(|e| e.into_inner());
        in_notes.remove(&(note as u8));
    }
    {
        let mut harm_notes = harmony_notes.lock().unwrap_or_else(|e| e.into_inner());
        let mut borr = borrowed_notes.lock().unwrap_or_else(|e| e.into_inner());
        for &n in notes.iter().skip(1) {
            harm_notes.remove(&(n as u8));
            borr.remove(&(n as u8));
        }
    }

    // Send note-offs based on routing mode
    let port_map = engine.last_port_map();
    match routing_mode {
        contrapunk::harmony::RoutingMode::ChannelBased => {
            for (i, &n) in notes.iter().enumerate() {
                let voice_channel = match i + 1 {
                    1 => Channel::Ch2,
                    2 => Channel::Ch3,
                    3 => Channel::Ch4,
                    4 => Channel::Ch5,
                    5 => Channel::Ch6,
                    6 => Channel::Ch7,
                    _ => Channel::Ch8,
                };

                if i == 0 {
                    let msg = MidiMessage::NoteOff(voice_channel, n, velocity);
                    let mut buf = vec![0u8; msg.bytes_size()];
                    let _ = msg.copy_to_slice(&mut buf);
                    let _ = output.send_to_first(&buf);
                    // Fanout to audio synth queue (fire-and-forget).
                    if let Some(ref mut producer) = audio_out {
                        let _ = producer.push(MidiEvent::NoteOff {
                            voice: i as u8,
                            note: u8::from(n),
                        });
                    }
                } else {
                    let hn = humanizer.humanize_note_off(n, voice_channel, velocity, 0, i as u8);
                    if hn.delay_ms == 0 {
                        let _ = send_humanized_note(
                            &hn,
                            output,
                            audio_out.as_deref_mut(),
                            hn.voice_index,
                        );
                    } else {
                        delay_queue.push(hn, now_ms);
                    }
                }
            }
        }
        contrapunk::harmony::RoutingMode::PortBased => {
            for (i, &n) in notes.iter().enumerate() {
                let port = if i < port_map.len() { port_map[i] } else { i };
                if port >= num_outputs {
                    continue;
                }

                if i == 0 {
                    let msg = MidiMessage::NoteOff(channel, n, velocity);
                    let mut buf = vec![0u8; msg.bytes_size()];
                    let _ = msg.copy_to_slice(&mut buf);
                    let _ = output.send_to_port(port, &buf);
                    if let Some(ref mut producer) = audio_out {
                        let _ = producer.push(MidiEvent::NoteOff {
                            voice: i as u8,
                            note: u8::from(n),
                        });
                    }
                } else {
                    let hn = humanizer.humanize_note_off(n, channel, velocity, port, i as u8);
                    if hn.delay_ms == 0 {
                        let _ = send_humanized_note(
                            &hn,
                            output,
                            audio_out.as_deref_mut(),
                            hn.voice_index,
                        );
                    } else {
                        delay_queue.push(hn, now_ms);
                    }
                }
            }
        }
    }
}

fn send_humanized_note(
    note: &HumanizedNote,
    output: &mut OutputRouter,
    audio_out: Option<&mut MidiProducer>,
    voice_index: u8,
) -> anyhow::Result<()> {
    let msg = if note.is_note_off {
        MidiMessage::NoteOff(note.channel, note.note, note.velocity)
    } else {
        MidiMessage::NoteOn(note.channel, note.note, note.velocity)
    };
    let mut buf = vec![0u8; msg.bytes_size()];
    msg.copy_to_slice(&mut buf)?;
    output.send_to_port(note.port, &buf)?;

    // Parallel fanout to audio synth queue (fire-and-forget; drop on full).
    if let Some(producer) = audio_out {
        let event = if note.is_note_off {
            MidiEvent::NoteOff {
                voice: voice_index,
                note: u8::from(note.note),
            }
        } else {
            MidiEvent::NoteOn {
                voice: voice_index,
                note: u8::from(note.note),
                velocity: u8::from(note.velocity),
            }
        };
        let _ = producer.push(event);
    }

    Ok(())
}
