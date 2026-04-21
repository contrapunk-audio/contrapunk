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
use contrapunk::chord::chord_display_with_analysis;
use contrapunk::harmony::HarmonyEngine;
use contrapunk::midi::input::connect_input;
use contrapunk::midi::output::OutputRouter;

use crate::guitar_bridge::GuitarBridge;
use crate::state::AppState;

/// Virtual input sentinels — must match the values in MidiDevices.svelte.
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

/// Inject a Note-On event into the active router pipeline.
///
/// Used by virtual inputs (e.g. Computer Keyboard) in the UI. Only has
/// effect while routing is active; returns an error otherwise.
#[tauri::command]
pub fn inject_note_on(
    note: u8,
    velocity: Option<u8>,
    state: State<AppState>,
) -> Result<Vec<u8>, String> {
    let tx_guard = state.router_tx.lock().map_err(|e| e.to_string())?;
    let tx = tx_guard.as_ref().ok_or("Routing not active")?;
    let vel = velocity.unwrap_or(100).clamp(1, 127);
    tx.send(vec![0x90, note, vel]).map_err(|e| e.to_string())?;
    Ok(vec![note])
}

/// Inject a Note-Off event into the active router pipeline.
#[tauri::command]
pub fn inject_note_off(note: u8, state: State<AppState>) -> Result<Vec<u8>, String> {
    let tx_guard = state.router_tx.lock().map_err(|e| e.to_string())?;
    let tx = tx_guard.as_ref().ok_or("Routing not active")?;
    tx.send(vec![0x80, note, 0]).map_err(|e| e.to_string())?;
    Ok(vec![note])
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

    // Create the MIDI-input channel up front so we can share the Sender with
    // inject_note_on / inject_note_off commands (virtual input from the UI).
    let (tx, rx) = mpsc::channel::<Vec<u8>>();
    {
        let mut slot = state.router_tx.lock().map_err(|e| e.to_string())?;
        *slot = Some(tx.clone());
    }

    // Clone Arcs for the router thread
    let in_notes = Arc::clone(&input_notes);
    let harm_notes = Arc::clone(&harmony_notes);
    let borr_notes = Arc::clone(&borrowed_notes);
    let ch_name = Arc::clone(&chord_name);
    let stop = Arc::clone(&stop_signal);
    let output_indices_clone = output_indices.clone();

    let detune = Arc::clone(&state.detune_cents);
    let panic_flag = Arc::clone(&state.panic_pending);
    // Reset before spawning so a flag set from a prior session doesn't fire.
    panic_flag.store(false, Ordering::SeqCst);

    // Spawn router thread
    thread::spawn(move || {
        if let Err(e) = run_tauri_router(
            input_idx,
            &output_indices_clone,
            engine_config,
            routing_mode,
            is_guitar,
            guitar_device,
            guitar_channel,
            guitar_config,
            tx,
            rx,
            in_notes,
            harm_notes,
            borr_notes,
            ch_name,
            stop,
            app_handle,
            detune,
            panic_flag,
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

    // Drop the injection sender so further inject_note_on/off calls fail fast
    if let Ok(mut tx_slot) = state.router_tx.lock() {
        *tx_slot = None;
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

#[allow(clippy::too_many_arguments)]
fn run_tauri_router(
    input_port: usize,
    output_ports: &[usize],
    config: EngineConfig,
    routing_mode: contrapunk::harmony::RoutingMode,
    is_guitar: bool,
    guitar_device: String,
    guitar_channel: usize,
    guitar_config: GuitarInputConfig,
    tx: mpsc::Sender<Vec<u8>>,
    rx: mpsc::Receiver<Vec<u8>>,
    input_notes: Arc<Mutex<HashSet<u8>>>,
    harmony_notes: Arc<Mutex<HashSet<u8>>>,
    borrowed_notes: Arc<Mutex<HashSet<u8>>>,
    chord_name: Arc<Mutex<String>>,
    stop_signal: Arc<std::sync::atomic::AtomicBool>,
    app_handle: AppHandle,
    detune_cents: Arc<AtomicI32>,
    panic_pending: Arc<std::sync::atomic::AtomicBool>,
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

    // Connect to either Guitar Audio bridge, physical MIDI input, or
    // nothing at all (Computer Keyboard virtual input — notes are pushed
    // by inject_note_on/off commands via the shared router_tx).
    let _midi_conn;
    let _guitar_bridge;
    let is_keyboard = input_port == VIRTUAL_COMPUTER_KEYBOARD;

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
    } else if is_keyboard {
        // Computer Keyboard mode: no physical connection. The tx is kept
        // alive by the AppState clone so inject_note_on/off can push.
        // Drop the local tx copy; AppState keeps the channel open.
        drop(tx);
        _midi_conn = None;
        _guitar_bridge = None;
    } else {
        // Physical MIDI mode
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

    // Event emission timer (~30fps)
    let mut last_emit = Instant::now();
    let emit_interval = Duration::from_millis(33);

    // Detune: track the previous value so we only send pitch bend on change.
    let mut prev_detune_cents: i32 = detune_cents.load(Ordering::Relaxed);

    // Main routing loop
    loop {
        if stop_signal.load(Ordering::SeqCst) {
            break;
        }

        // Panic handling: any engine-config command that could strand
        // active notes sets panic_pending. Emit MIDI All-Notes-Off (CC
        // 123) on every channel × every port to release stuck notes,
        // then clear tracked note state so the UI stops showing them.
        if panic_pending.swap(false, Ordering::SeqCst) {
            let num_ports = output_router.connection_count();
            for p in 0..num_ports {
                for ch in 0u8..16 {
                    let _ = output_router.send_to_port(p, &[0xB0 | ch, 123, 0]);
                }
            }
            if let Ok(mut n) = input_notes.lock() {
                n.clear();
            }
            if let Ok(mut n) = harmony_notes.lock() {
                n.clear();
            }
            if let Ok(mut n) = borrowed_notes.lock() {
                n.clear();
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

        // Process MIDI messages
        match rx.recv_timeout(Duration::from_millis(5)) {
            Ok(message) => {
                process_midi_message(
                    &message,
                    &mut engine,
                    &mut output_router,
                    &input_notes,
                    &harmony_notes,
                    &borrowed_notes,
                    &chord_name,
                    routing_mode,
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
    input_notes: &Arc<Mutex<HashSet<u8>>>,
    harmony_notes: &Arc<Mutex<HashSet<u8>>>,
    borrowed_notes: &Arc<Mutex<HashSet<u8>>>,
    chord_name: &Arc<Mutex<String>>,
    routing_mode: contrapunk::harmony::RoutingMode,
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
                    input_notes,
                    harmony_notes,
                    borrowed_notes,
                    chord_name,
                    routing_mode,
                );
            } else {
                handle_note_on(
                    channel,
                    note,
                    velocity,
                    engine,
                    output,
                    input_notes,
                    harmony_notes,
                    borrowed_notes,
                    chord_name,
                    routing_mode,
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
                input_notes,
                harmony_notes,
                borrowed_notes,
                chord_name,
                routing_mode,
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
    input_notes: &Arc<Mutex<HashSet<u8>>>,
    harmony_notes: &Arc<Mutex<HashSet<u8>>>,
    borrowed_notes: &Arc<Mutex<HashSet<u8>>>,
    chord_name: &Arc<Mutex<String>>,
    routing_mode: contrapunk::harmony::RoutingMode,
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
                let msg = MidiMessage::NoteOn(voice_channel, n, velocity);
                let mut buf = vec![0u8; msg.bytes_size()];
                let _ = msg.copy_to_slice(&mut buf);
                let _ = output.send_to_first(&buf);
                let _ = i;
            }
        }
        contrapunk::harmony::RoutingMode::PortBased => {
            for (i, &n) in notes.iter().enumerate() {
                let port = if i < port_map.len() { port_map[i] } else { i };
                if port >= num_outputs {
                    continue;
                }
                let msg = MidiMessage::NoteOn(channel, n, velocity);
                let mut buf = vec![0u8; msg.bytes_size()];
                let _ = msg.copy_to_slice(&mut buf);
                let _ = output.send_to_port(port, &buf);
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
    input_notes: &Arc<Mutex<HashSet<u8>>>,
    harmony_notes: &Arc<Mutex<HashSet<u8>>>,
    borrowed_notes: &Arc<Mutex<HashSet<u8>>>,
    _chord_name: &Arc<Mutex<String>>,
    routing_mode: contrapunk::harmony::RoutingMode,
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
                let msg = MidiMessage::NoteOff(voice_channel, n, velocity);
                let mut buf = vec![0u8; msg.bytes_size()];
                let _ = msg.copy_to_slice(&mut buf);
                let _ = output.send_to_first(&buf);
                let _ = i;
            }
        }
        contrapunk::harmony::RoutingMode::PortBased => {
            for (i, &n) in notes.iter().enumerate() {
                let port = if i < port_map.len() { port_map[i] } else { i };
                if port >= num_outputs {
                    continue;
                }
                let msg = MidiMessage::NoteOff(channel, n, velocity);
                let mut buf = vec![0u8; msg.bytes_size()];
                let _ = msg.copy_to_slice(&mut buf);
                let _ = output.send_to_port(port, &buf);
            }
        }
    }
}
