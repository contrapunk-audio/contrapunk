//! Tauri commands for engine routing control and real-time note state.
//!
//! Handles starting/stopping the MIDI router thread and emitting
//! real-time note-update events to the frontend.

use std::collections::HashSet;
use std::sync::atomic::Ordering;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use serde::Serialize;
use tauri::{AppHandle, Emitter, State};
use wmidi::{Channel, MidiMessage, Note, Velocity};

use contrapunk::chord::chord_display_with_analysis;
use contrapunk::harmony::HarmonyEngine;
use contrapunk::audio::guitar_input::GuitarInputConfig;
use contrapunk::humanize::{DelayQueue, HumanizeConfig, HumanizedNote, Humanizer};
use contrapunk::midi::input::connect_input;
use contrapunk::midi::output::OutputRouter;

use crate::guitar_bridge::GuitarBridge;
use crate::state::AppState;

/// Sentinel value for the "Guitar Audio" virtual input.
/// When `input_idx` equals this value, we spawn a GuitarBridge
/// instead of connecting to a physical MIDI input port.
const GUITAR_AUDIO_SENTINEL: usize = usize::MAX - 2;

/// Payload for the "note-update" Tauri event.
#[derive(Clone, Serialize)]
pub struct NoteUpdatePayload {
    pub input_notes: Vec<u8>,
    pub harmony_notes: Vec<u8>,
    pub borrowed_notes: Vec<u8>,
    pub chord_name: String,
    pub last_borrowed_from: String,
}

/// Returns the current note state snapshot.
#[tauri::command]
pub fn get_note_state(state: State<AppState>) -> Result<NoteUpdatePayload, String> {
    let input = state.input_notes.lock().map_err(|e| e.to_string())?;
    let harmony = state.harmony_notes.lock().map_err(|e| e.to_string())?;
    let borrowed = state.borrowed_notes.lock().map_err(|e| e.to_string())?;
    let chord = state.chord_name.lock().map_err(|e| e.to_string())?;

    Ok(NoteUpdatePayload {
        input_notes: input.iter().copied().collect(),
        harmony_notes: harmony.iter().copied().collect(),
        borrowed_notes: borrowed.iter().copied().collect(),
        chord_name: chord.clone(),
        last_borrowed_from: String::new(),
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
        )
    };

    let humanize_config = {
        let config = state.humanize_config.lock().map_err(|e| e.to_string())?;
        config.clone()
    };

    // Capture guitar config for the router thread
    let is_guitar = input_idx == GUITAR_AUDIO_SENTINEL;
    let guitar_device = {
        state.guitar_device.lock().map_err(|e| e.to_string())?.clone()
    };
    let guitar_channel = {
        *state.guitar_channel.lock().map_err(|e| e.to_string())?
    };
    let guitar_config = {
        state.guitar_config.lock().map_err(|e| e.to_string())?.clone()
            .unwrap_or_default()
    };

    // Shared state for note updates
    let input_notes = Arc::new(Mutex::new(HashSet::<u8>::new()));
    let harmony_notes = Arc::new(Mutex::new(HashSet::<u8>::new()));
    let borrowed_notes = Arc::new(Mutex::new(HashSet::<u8>::new()));
    let chord_name = Arc::new(Mutex::new(String::new()));
    let stop_signal = Arc::new(std::sync::atomic::AtomicBool::new(false));

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

    // Spawn router thread
    thread::spawn(move || {
        if let Err(e) = run_tauri_router(
            input_idx,
            &output_indices_clone,
            engine_config,
            humanize_config,
            is_guitar,
            guitar_device,
            guitar_channel,
            guitar_config,
            in_notes,
            harm_notes,
            borr_notes,
            ch_name,
            stop,
            app_handle,
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
);

fn run_tauri_router(
    input_port: usize,
    output_ports: &[usize],
    config: EngineConfig,
    humanize_config: HumanizeConfig,
    is_guitar: bool,
    guitar_device: String,
    guitar_channel: usize,
    guitar_config: GuitarInputConfig,
    input_notes: Arc<Mutex<HashSet<u8>>>,
    harmony_notes: Arc<Mutex<HashSet<u8>>>,
    borrowed_notes: Arc<Mutex<HashSet<u8>>>,
    chord_name: Arc<Mutex<String>>,
    stop_signal: Arc<std::sync::atomic::AtomicBool>,
    app_handle: AppHandle,
) -> anyhow::Result<()> {
    let (key, mode, octave_mode, vl_enabled, vl_style, scale_mode, ic_enabled, br_range, vp) =
        config;

    // Create channel for MIDI input — both physical MIDI and guitar
    // bridge send Vec<u8> through the same channel.
    let (tx, rx) = mpsc::channel::<Vec<u8>>();

    // Connect to either Guitar Audio bridge or physical MIDI input
    let _midi_conn;
    let _guitar_bridge;

    eprintln!("[tauri-router] is_guitar={} device='{}' channel={}", is_guitar, guitar_device, guitar_channel);
    if is_guitar {
        // Guitar Audio mode: spawn cpal capture -> DSP -> same tx channel
        eprintln!("[tauri-router] Creating GuitarBridge...");
        let bridge = GuitarBridge::new(&guitar_device, guitar_channel, guitar_config, tx)
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

    // Create humanizer
    let mut humanizer = Humanizer::new(humanize_config);
    let mut delay_queue = DelayQueue::new();
    let epoch = Instant::now();
    let now_ms = || epoch.elapsed().as_secs_f64() * 1000.0;

    humanizer.clock_mut().start(now_ms());

    // Event emission timer (~30fps)
    let mut last_emit = Instant::now();
    let emit_interval = Duration::from_millis(33);

    // Main routing loop
    loop {
        if stop_signal.load(Ordering::SeqCst) {
            break;
        }

        // Tick humanizer
        let current_ms = now_ms();
        humanizer.tick(current_ms);

        // Drain delay queue
        for hn in delay_queue.drain_ready(current_ms) {
            let _ = send_humanized_note(&hn, &mut output_router);
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
            let payload = {
                let in_notes = input_notes.lock().unwrap();
                let harm_notes = harmony_notes.lock().unwrap();
                let borr_notes = borrowed_notes.lock().unwrap();
                let ch_name = chord_name.lock().unwrap();
                NoteUpdatePayload {
                    input_notes: in_notes.iter().copied().collect(),
                    harmony_notes: harm_notes.iter().copied().collect(),
                    borrowed_notes: borr_notes.iter().copied().collect(),
                    chord_name: ch_name.clone(),
                    last_borrowed_from: engine
                        .last_borrowed_from()
                        .map(|m| format!("{}", m))
                        .unwrap_or_default(),
                }
            };
            let _ = app_handle.emit("note-update", payload);
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
) {
    let notes = engine.harmonize_note_on(note);
    let num_outputs = output.connection_count();

    // Update shared state
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
    // Track borrowed notes
    if engine.last_borrowed_from().is_some() {
        let mut borr = borrowed_notes.lock().unwrap();
        for &n in notes.iter().skip(1) {
            borr.insert(n as u8);
        }
    }

    // Update chord name
    {
        let all_sounding: HashSet<u8> = {
            let in_notes = input_notes.lock().unwrap();
            let harm_notes = harmony_notes.lock().unwrap();
            in_notes.union(&harm_notes).copied().collect()
        };
        if !all_sounding.is_empty() {
            let key_tonic = Some(engine.key().semitones_from_c());
            let display = chord_display_with_analysis(&all_sounding, key_tonic);
            let mut ch = chord_name.lock().unwrap();
            *ch = display;
        }
    }

    // Send notes to outputs
    let port_map = engine.last_port_map();
    for (i, &n) in notes.iter().enumerate() {
        let port = if i < port_map.len() { port_map[i] } else { i };
        if port >= num_outputs {
            continue;
        }

        if i == 0 {
            // Melody: send immediately
            let msg = MidiMessage::NoteOn(channel, n, velocity);
            let mut buf = vec![0u8; msg.bytes_size()];
            let _ = msg.copy_to_slice(&mut buf);
            let _ = output.send_to_port(port, &buf);
        } else {
            // Harmony: humanize
            let hn = humanizer.humanize_note_on(n, channel, velocity, port);
            if hn.delay_ms == 0 {
                let _ = send_humanized_note(&hn, output);
            } else {
                delay_queue.push(hn, now_ms);
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
) {
    let notes = engine.harmonize_note_off(note);
    let num_outputs = output.connection_count();

    // Update shared state
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

    // Send note-offs
    let port_map = engine.last_port_map();
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
        } else {
            let hn = humanizer.humanize_note_off(n, channel, velocity, port);
            if hn.delay_ms == 0 {
                let _ = send_humanized_note(&hn, output);
            } else {
                delay_queue.push(hn, now_ms);
            }
        }
    }
}

fn send_humanized_note(note: &HumanizedNote, output: &mut OutputRouter) -> anyhow::Result<()> {
    let msg = if note.is_note_off {
        MidiMessage::NoteOff(note.channel, note.note, note.velocity)
    } else {
        MidiMessage::NoteOn(note.channel, note.note, note.velocity)
    };
    let mut buf = vec![0u8; msg.bytes_size()];
    msg.copy_to_slice(&mut buf)?;
    output.send_to_port(note.port, &buf)?;
    Ok(())
}
