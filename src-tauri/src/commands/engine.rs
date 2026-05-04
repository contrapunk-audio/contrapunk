//! Tauri commands for engine routing control and real-time note state.
//!
//! Handles starting/stopping the MIDI router thread and emitting
//! real-time note-update events to the frontend.

use std::collections::{HashMap, HashSet};
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
use contrapunk::synth::SynthEvent;
use contrapunk::transport::Transport;

use crate::guitar_bridge::GuitarBridge;
use crate::state::{AppState, VoiceOutputTarget};

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

/// Payload for the "knob-cc-raw" Tauri event — every Control Change
/// message that arrives on the active MIDI input is forwarded to the
/// frontend, which resolves the CC number to a Performance-view knob
/// index via the user's MIDI Learn mapping (`contrapunk-knob-cc-map`
/// in localStorage). The router thread no longer hardcodes a CC →
/// knob-index table; that lives entirely in the UI now so users can
/// rebind any controller without a backend rebuild.
#[derive(Clone, Serialize)]
pub struct KnobCcRawPayload {
    /// MIDI CC number (0-127).
    pub cc: u8,
    /// Normalized 0.0..1.0 value (CC 0-127 / 127).
    pub value: f32,
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

    // Empty output_indices is fine — every voice can route to the
    // internal synth via voice_outputs. Pre-#36 the engine required
    // at least one external MIDI port; that constraint is gone now
    // that per-voice routing is the source of truth.

    // Share the engine across the router thread and command handlers.
    // Without this clone, the router would run on its own private engine
    // instance and would never see param changes (set_key, set_auto_key,
    // ...) made via Tauri commands during a routing session.
    let engine = Arc::clone(&state.engine);

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
    // Clone the Arc so the audio thread can re-read the live config every
    // block — lets the debug window's edits take effect without a restart.
    let guitar_config_shared = Arc::clone(&state.guitar_config);

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

    // Clone the synth event sender so the router thread can push note
    // events into the built-in synth alongside external MIDI output.
    let synth_tx = state.synth_tx.clone();

    // Share the per-voice output routing table with the router thread.
    // Lock-read at each note dispatch to honor live UI changes.
    let voice_outputs = Arc::clone(&state.voice_outputs);

    // Share the transport clock with the router thread so it can push
    // current beat-phase to the engine each iteration. Counterpoint
    // Species 2-4 read this; without a fresh phase they fall back to
    // Species 1 behavior.
    let transport = Arc::clone(&state.transport);

    // Spawn router thread
    thread::spawn(move || {
        if let Err(e) = run_tauri_router(
            input_idx,
            &output_indices_clone,
            engine,
            routing_mode,
            is_guitar,
            guitar_device,
            guitar_channel,
            guitar_config,
            guitar_config_shared,
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
            synth_tx,
            voice_outputs,
            transport,
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

    Ok(())
}

// ============================================================================
// Router thread implementation
// ============================================================================

#[allow(clippy::too_many_arguments)]
fn run_tauri_router(
    input_port: usize,
    output_ports: &[usize],
    engine: Arc<Mutex<HarmonyEngine>>,
    routing_mode: contrapunk::harmony::RoutingMode,
    is_guitar: bool,
    guitar_device: String,
    guitar_channel: usize,
    guitar_config: GuitarInputConfig,
    guitar_config_shared: Arc<Mutex<Option<GuitarInputConfig>>>,
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
    synth_tx: mpsc::Sender<SynthEvent>,
    voice_outputs: Arc<Mutex<Vec<VoiceOutputTarget>>>,
    transport: Arc<Transport>,
) -> anyhow::Result<()> {
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
            guitar_config_shared,
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

    // voice_count is user-controlled via `set_voice_count`. With
    // per-voice routing, voices in excess of the connected MIDI ports
    // route to the built-in synth — there's no reason to clamp the
    // engine's voice_count to `num_outputs` at routing start. (Doing
    // so silently overrode the UI's voice picker — see the wave of
    // "I picked soprano in a 4-voice setup but the engine had only
    // 2 voices" reports.)

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

        // Push current beat-phase from the transport clock to the engine.
        // Counterpoint Species 2-4 read this on every NoteOn to decide
        // passing tones / suspensions / strong-vs-weak-beat behavior.
        // Without a fresh phase, the engine sees None and silently falls
        // back to Species 1 regardless of which Species the user picked.
        // When transport is stopped we explicitly push None so Species
        // 2-4 fall back to Species 1 (per the engine doc on
        // `counterpoint_beat_phase: Option<f64>`).
        {
            let phase = if transport.is_running() {
                Some(transport.beat_position())
            } else {
                None
            };
            let mut eng = engine.lock().unwrap_or_else(|e| e.into_inner());
            eng.set_counterpoint_beat_phase(phase);
        }

        // Beat-aligned chord retrigger. When pattern is enabled and the
        // transport is running, detect cell-boundary crossings and
        // retrigger the currently-sounding harmony on each cell-on
        // boundary (Live mode). Skip silently when pattern off or
        // transport stopped — fall through to today's real-time path.
        //
        // Skip entirely when `panic_pending` is set: this iteration
        // belongs to the reharmonize cycle below, which will replay
        // held inputs and dispatch a clean diff. Firing the pattern
        // tick first would attack stale `harmony_notes` and produce an
        // audible click as reharm immediately releases the displaced
        // voices.
        let panic_will_fire = panic_pending.load(Ordering::SeqCst);

        // Reharmonize on parameter change. Any engine-config setter that
        // could change the harmony output sets panic_pending and stashes
        // the previously-held input MIDI numbers in the engine's
        // `pending_reharm_inputs`. We replay each of those inputs under
        // the new parameters, compute a diff against the previously-
        // sounding harmony set, and dispatch only the difference:
        // NoteOff for harmony notes that drop out, NoteOn for newly-
        // needed ones. The user's held input note never gets
        // interrupted — knob sweeps produce a smooth musical morph.
        if panic_pending.swap(false, Ordering::SeqCst) {
            // Snapshot what's currently sounding (harmony + borrowed only —
            // input notes belong to the user, leave them alone).
            let old_harmonies: HashSet<u8> = {
                let h = harmony_notes.lock().unwrap_or_else(|e| e.into_inner());
                let b = borrowed_notes.lock().unwrap_or_else(|e| e.into_inner());
                h.iter().chain(b.iter()).copied().collect()
            };

            // Drain held inputs from the engine and replay them under the
            // new parameters. Each replay populates `active_notes` and
            // updates `last_port_map` for the per-voice routing below.
            let mut new_harmonies: HashSet<u8> = HashSet::new();
            let mut per_input: Vec<(Vec<u8>, Vec<usize>)> = Vec::new();
            {
                let mut eng = engine.lock().unwrap_or_else(|e| e.into_inner());
                let inputs = eng.take_reharm_inputs();
                for midi in inputs {
                    if let Ok(input_note) = Note::try_from(midi) {
                        let result = eng.harmonize_note_on(input_note);
                        let port_map = eng.last_port_map().to_vec();
                        // Skip index 0 (the input itself) — only harmonies
                        // contribute to the diff. Input keeps ringing.
                        let harm_midis: Vec<u8> =
                            result.iter().skip(1).map(|n| u8::from(*n)).collect();
                        for &m in &harm_midis {
                            new_harmonies.insert(m);
                        }
                        // Keep the input + harmonies + ports together so we
                        // can route attacks correctly. Index 0 of harm_midis
                        // is unused (input is at result[0]); we store the
                        // full result for routing.
                        let full_midis: Vec<u8> = result.iter().map(|n| u8::from(*n)).collect();
                        per_input.push((full_midis, port_map));
                    }
                }
            }

            let to_release: Vec<u8> = old_harmonies.difference(&new_harmonies).copied().collect();
            let to_attack: HashSet<u8> =
                new_harmonies.difference(&old_harmonies).copied().collect();

            // Send NoteOff for released notes — synth + every external
            // port via the shared broadcast helper.
            let num_ports = output_router.connection_count();
            for n in &to_release {
                broadcast_note_off(*n, num_ports, &synth_tx, &mut output_router);
            }

            // Send NoteOn for newly-attacked notes, routed per voice via
            // each replay's port map and the live voice_outputs table.
            let voice_targets: Vec<VoiceOutputTarget> = voice_outputs
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .clone();
            for (midis, port_map) in &per_input {
                // Skip index 0 — that's the user's input note, already
                // sounding from when they pressed the key. Channel 0
                // and velocity 100 are reharm-path defaults (engine
                // doesn't track per-input channel/velocity through
                // take_reharm_inputs); same as the to_release loop
                // above.
                for (i, &n) in midis.iter().enumerate().skip(1) {
                    if !to_attack.contains(&n) {
                        continue; // already sounding from before
                    }
                    let slot = port_map.get(i).copied().unwrap_or(i);
                    let target = voice_targets.get(slot).copied().unwrap_or_default();
                    dispatch_voice(
                        target,
                        0,
                        VoiceDispatch::NoteOn {
                            note: n,
                            velocity: 100,
                        },
                        num_ports,
                        &synth_tx,
                        &mut output_router,
                    );
                }
            }

            // Replace UI tracking sets with the new harmony state. Input
            // notes stay as-is — user's still holding them.
            if let Ok(mut h) = harmony_notes.lock() {
                *h = new_harmonies;
            }
            if let Ok(mut b) = borrowed_notes.lock() {
                b.clear();
            }

            // Rebuild routing-aware tracking from the replay results.
            // Each input's harmonies (skipping i=0, the input itself)
            // get re-keyed with their freshly-resolved per-voice
            // target. Pattern attacks/releases on subsequent ticks
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
                // Intercept Control Change messages (status 0xB0-0xBF) and
                // forward every CC to the frontend as "knob-cc-raw". The UI
                // owns the CC → Performance-view-knob mapping (MIDI Learn,
                // persisted in localStorage). MPK Mini's CC 70-77 baseline
                // is seeded as the default preset on first run.
                if message.len() >= 3 && (message[0] & 0xF0) == 0xB0 {
                    let cc_number = message[1];
                    let cc_value = message[2];
                    let value = (cc_value as f32) / 127.0;
                    let _ = app_handle.emit(
                        "knob-cc-raw",
                        KnobCcRawPayload {
                            cc: cc_number,
                            value,
                        },
                    );
                    // Don't fall through — CCs are not notes. Forwarding to
                    // process_midi_message would route them to send_to_first
                    // which is wrong for control data.
                } else {
                    process_midi_message(
                        &message,
                        &engine,
                        &mut output_router,
                        &input_notes,
                        &harmony_notes,
                        &borrowed_notes,
                        &chord_name,
                        routing_mode,
                        &synth_tx,
                        &voice_outputs,
                    );
                }
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
            let (last_borrowed_from, current_key) = {
                let eng = engine.lock().unwrap_or_else(|e| e.into_inner());
                (
                    eng.last_borrowed_from()
                        .map(|m| format!("{}", m))
                        .unwrap_or_default(),
                    format!("{}", eng.key()),
                )
            };
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
                    last_borrowed_from,
                    current_key,
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

    // Clear note state on exit.
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

#[allow(clippy::too_many_arguments)]
fn process_midi_message(
    bytes: &[u8],
    engine: &Arc<Mutex<HarmonyEngine>>,
    output: &mut OutputRouter,
    input_notes: &Arc<Mutex<HashSet<u8>>>,
    harmony_notes: &Arc<Mutex<HashSet<u8>>>,
    borrowed_notes: &Arc<Mutex<HashSet<u8>>>,
    chord_name: &Arc<Mutex<String>>,
    routing_mode: contrapunk::harmony::RoutingMode,
    synth_tx: &mpsc::Sender<SynthEvent>,
    voice_outputs: &Arc<Mutex<Vec<VoiceOutputTarget>>>,
) {
    let msg = match MidiMessage::try_from(bytes) {
        Ok(m) => m,
        Err(_) => {
            let _ = output.send_to_first(bytes);
            return;
        }
    };

    // Lock the shared engine for the duration of one MIDI message. Held
    // briefly enough that Tauri command handlers (set_key etc.) waiting
    // on the same Mutex pick it up between messages.
    let mut eng = engine.lock().unwrap_or_else(|e| e.into_inner());
    let eng: &mut HarmonyEngine = &mut eng;

    match msg {
        MidiMessage::NoteOn(channel, note, velocity) => {
            if velocity == Velocity::MIN {
                handle_note_off(
                    channel,
                    note,
                    velocity,
                    eng,
                    output,
                    input_notes,
                    harmony_notes,
                    borrowed_notes,
                    chord_name,
                    routing_mode,
                    synth_tx,
                    voice_outputs,
                );
            } else {
                handle_note_on(
                    channel,
                    note,
                    velocity,
                    eng,
                    output,
                    input_notes,
                    harmony_notes,
                    borrowed_notes,
                    chord_name,
                    routing_mode,
                    synth_tx,
                    voice_outputs,
                );
            }
        }
        MidiMessage::NoteOff(channel, note, velocity) => {
            handle_note_off(
                channel,
                note,
                velocity,
                eng,
                output,
                input_notes,
                harmony_notes,
                borrowed_notes,
                chord_name,
                routing_mode,
                synth_tx,
                voice_outputs,
            );
        }
        _ => {
            let _ = output.send_to_first(bytes);
        }
    }
}

#[allow(clippy::too_many_arguments)]
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
    synth_tx: &mpsc::Sender<SynthEvent>,
    voice_outputs: &Arc<Mutex<Vec<VoiceOutputTarget>>>,
) {
    let notes = engine.harmonize_note_on(note);
    // Drain any harmonies the engine flagged for explicit release —
    // populated when an auto-key change wiped `active_notes` mid-flight.
    // These would otherwise stay sounding under the old key.
    let stale_releases = engine.take_pending_releases();
    let num_outputs = output.connection_count();

    // Send Note-Offs for stale harmonies before emitting the new ones.
    if !stale_releases.is_empty() {
        for &n in &stale_releases {
            broadcast_note_off(u8::from(n), num_outputs, synth_tx, output);
        }
        let mut harm = harmony_notes.lock().unwrap_or_else(|e| e.into_inner());
        let mut borr = borrowed_notes.lock().unwrap_or_else(|e| e.into_inner());
        for &n in &stale_releases {
            harm.remove(&u8::from(n));
            borr.remove(&u8::from(n));
        }
    }

    // Snapshot voice routing once per note event — clone is cheap
    // (8-element Vec of Copy enums) and avoids lock contention during
    // dispatch.
    let voice_targets: Vec<VoiceOutputTarget> = voice_outputs
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .clone();
    // The harmony engine returns notes as `[input, closest_harmony, ...]`
    // — input always at index 0. To route each entry to the correct
    // SATB voice slot (so e.g. the user's note plays through the
    // tenor's output target when voice_position=2), we map result-Vec
    // index → arrangement slot via the engine's port map.
    let port_map: Vec<usize> = engine.last_port_map().to_vec();
    let target_for = |i: usize| -> VoiceOutputTarget {
        let slot = port_map.get(i).copied().unwrap_or(i);
        voice_targets.get(slot).copied().unwrap_or_default()
    };

    // Fan each voice via the unified dispatch helper. Synth and
    // external MIDI go in one loop; `dispatch_voice` handles the
    // target.match cases internally. Order: dispatch first so the
    // synth's audible NoteOn precedes the tracking-set updates that
    // chord-display reads from (chord display tolerates briefly
    // stale state better than humans tolerate audible latency).
    let channel_idx: u8 = channel.index();
    let velocity_byte: u8 = u8::from(velocity);
    for (i, &n) in notes.iter().enumerate() {
        dispatch_voice(
            target_for(i),
            channel_idx,
            VoiceDispatch::NoteOn {
                note: u8::from(n),
                velocity: velocity_byte,
            },
            num_outputs,
            synth_tx,
            output,
        );
    }

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

    let _ = (channel_idx, velocity_byte, target_for);
}

#[allow(clippy::too_many_arguments)]
fn handle_note_off(
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
    synth_tx: &mpsc::Sender<SynthEvent>,
    voice_outputs: &Arc<Mutex<Vec<VoiceOutputTarget>>>,
) {
    let notes = engine.harmonize_note_off(note);
    let num_outputs = output.connection_count();

    // Snapshot voice routing — same pattern as handle_note_on. Use the
    // engine's port map so the per-voice routing slot matches the SATB
    // arrangement (input note's slot follows voice_position, not 0).
    let voice_targets: Vec<VoiceOutputTarget> = voice_outputs
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .clone();
    let port_map: Vec<usize> = engine.last_port_map().to_vec();
    let target_for = |i: usize| -> VoiceOutputTarget {
        let slot = port_map.get(i).copied().unwrap_or(i);
        voice_targets.get(slot).copied().unwrap_or_default()
    };

    // Unified per-voice release. Synth NoteOff drops release velocity
    // (SynthEvent::NoteOff has no velocity field); external MIDI
    // preserves it from the input event since some hardware (Yamaha,
    // certain virtual instruments) responds to release velocity.
    let channel_idx: u8 = channel.index();
    let velocity_byte: u8 = u8::from(velocity);
    for (i, &n) in notes.iter().enumerate() {
        dispatch_voice(
            target_for(i),
            channel_idx,
            VoiceDispatch::NoteOff {
                note: u8::from(n),
                velocity: velocity_byte,
            },
            num_outputs,
            synth_tx,
            output,
        );
    }

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

    // Recompute chord-name display after the release. Mirrors the
    // identical block in handle_note_on at the all_sounding read site —
    // without this, the previous chord text persists when a note is
    // released, so legato (note-on B before note-off A) shows "A+B"
    // even after A's release leaves only B held. Empties the chord
    // string when nothing is sounding so the UI's `{#if chordName}`
    // checks fall through to the placeholder.
    {
        let all_sounding: HashSet<u8> = {
            let in_notes = input_notes.lock().unwrap_or_else(|e| e.into_inner());
            let harm_notes = harmony_notes.lock().unwrap_or_else(|e| e.into_inner());
            in_notes.union(&harm_notes).copied().collect()
        };
        let mut ch = chord_name.lock().unwrap_or_else(|e| e.into_inner());
        if all_sounding.is_empty() {
            ch.clear();
        } else {
            let key_tonic = Some(engine.key().semitones_from_c());
            *ch = chord_display_with_analysis(&all_sounding, key_tonic);
        }
    }
}

/// One per-voice dispatch event consumed by `dispatch_voice`. Carries
/// note + velocity in u8 form (0-127). Channel is passed alongside to
/// the helper since it's typically uniform across a batch of voices
/// (one input event → many voices).
#[derive(Clone, Copy, Debug)]
enum VoiceDispatch {
    /// Send a NoteOn at the given velocity.
    NoteOn { note: u8, velocity: u8 },
    /// Send a NoteOff. `velocity` is the release velocity (0 for most
    /// MIDI consumers; some Yamaha hardware uses non-zero release).
    NoteOff { note: u8, velocity: u8 },
}

/// Single fanout shared by every router-thread NoteOn/NoteOff dispatch
/// site (real-time `handle_note_on` / `handle_note_off`, beat-pattern
/// tick, drain-on-disable, orphan-release-on-retrigger, panic-replay
/// re-attack). Centralises the `target.match { Synth | MidiPort | Off }`
/// skeleton — adding a fourth destination, changing the byte encoding,
/// or instrumenting MIDI traffic now happens in one place.
///
/// Channel and velocity are u8 (0-15 / 0-127). Real-time callers
/// convert from wmidi types at the boundary; pattern / orphan callers
/// pass captured `HeldVoice` fields directly.
fn dispatch_voice(
    target: VoiceOutputTarget,
    channel: u8,
    event: VoiceDispatch,
    num_ports: usize,
    synth_tx: &mpsc::Sender<SynthEvent>,
    output: &mut OutputRouter,
) {
    // Pin the u7/u4 invariants for callers — every existing site
    // already produces values in range (channel.index() / u8::from
    // on wmidi U7 newtypes), but a future caller passing a raw byte
    // from an untrusted source would silently corrupt the MIDI byte
    // stream without these. Zero release-build cost.
    debug_assert!(channel < 16, "MIDI channel out of range: {}", channel);
    let (n, v) = match event {
        VoiceDispatch::NoteOn { note, velocity } => (note, velocity),
        VoiceDispatch::NoteOff { note, velocity } => (note, velocity),
    };
    debug_assert!(n < 128, "MIDI note out of range: {}", n);
    debug_assert!(v < 128, "MIDI velocity out of range: {}", v);

    match (target, event) {
        (VoiceOutputTarget::Synth, VoiceDispatch::NoteOn { note, velocity }) => {
            let _ = synth_tx.send(SynthEvent::NoteOn { note, velocity });
        }
        (VoiceOutputTarget::Synth, VoiceDispatch::NoteOff { note, .. }) => {
            let _ = synth_tx.send(SynthEvent::NoteOff { note });
        }
        (VoiceOutputTarget::MidiPort { port }, _) if port >= num_ports => {}
        (VoiceOutputTarget::MidiPort { port }, VoiceDispatch::NoteOn { note, velocity }) => {
            let msg = [0x90 | (channel & 0x0F), note, velocity];
            let _ = output.send_to_port(port, &msg);
        }
        (VoiceOutputTarget::MidiPort { port }, VoiceDispatch::NoteOff { note, velocity }) => {
            let msg = [0x80 | (channel & 0x0F), note, velocity];
            let _ = output.send_to_port(port, &msg);
        }
        (VoiceOutputTarget::Off, _) => {}
    }
}

/// Broadcast NoteOff to the synth and every connected external port.
///
/// Used by code paths that release a harmony note without knowing
/// which output port it originally went to: stale-releases triggered
/// by an auto-key change in `handle_note_on`, and the panic-replay
/// `to_release` diff in `run_tauri_router`. Per-port routing isn't
/// tracked on a per-note basis at the router level, so we accept
/// over-broad delivery: extra NoteOffs to ports that didn't see the
/// matching NoteOn are inaudible on synths, samplers, and DAWs. A
/// few MIDI routing/merge utilities log a warning about unbalanced
/// pairs; that's the trade-off for not tracking per-note port
/// routing.
///
/// Channel 0 is conventional for these broadcast releases since the
/// notes' original channel may differ across the held set; 0 is a
/// safe default for general MIDI consumers that don't filter on
/// channel. Caveat: voices originally attacked on non-zero channels
/// (MPE, multi-channel routing) will not be matched by this path —
/// the reharm/panic flow doesn't preserve per-note channel through
/// `take_pending_releases` / `take_reharm_inputs`. Tracked as a
/// limitation in the held_harmonies follow-up issue #90.
///
/// Implemented as a thin loop over `dispatch_voice` so the byte
/// encoding lives in exactly one place — adding a fourth dispatch
/// destination only requires updating the helper. The helper's
/// `debug_assert!(channel < 16)` / `debug_assert!(v < 128)` invariants
/// guard this path too.
fn broadcast_note_off(
    note: u8,
    num_ports: usize,
    synth_tx: &mpsc::Sender<SynthEvent>,
    output: &mut OutputRouter,
) {
    debug_assert!(note < 128, "MIDI note out of range: {}", note);
    let event = VoiceDispatch::NoteOff { note, velocity: 0 };
    // Synth fanout — channel arg ignored by the Synth target.
    dispatch_voice(
        VoiceOutputTarget::Synth,
        0,
        event,
        num_ports,
        synth_tx,
        output,
    );
    // Every external port — channel 0 by convention (see fn doc).
    for port in 0..num_ports {
        dispatch_voice(
            VoiceOutputTarget::MidiPort { port },
            0,
            event,
            num_ports,
            synth_tx,
            output,
        );
    }
}
