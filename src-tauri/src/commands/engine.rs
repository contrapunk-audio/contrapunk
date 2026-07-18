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
    /// Notes currently sounding from the Companion's canon lane.
    /// Subset of `harmony_notes` — included separately so the piano
    /// can color them distinctly from generic harmony output.
    pub canon_notes: Vec<u8>,
    /// Notes currently sounding from the Companion's counterpoint lane.
    /// Subset of `harmony_notes` — same role as `canon_notes`.
    pub counterpoint_notes: Vec<u8>,
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

    // Issue #14: if the user selected external MIDI ports but all
    // voices route to the internal synth, the per-voice routing
    // table (`VoiceOutputTarget::default() = Synth`) silently swallows
    // their harmonies. Surface a one-time warning so this isn't a
    // mystery support thread — both to the desktop log and to the
    // frontend so the UI can show it.
    {
        let voice_outputs_snapshot = state
            .voice_outputs
            .lock()
            .map_err(|e| e.to_string())?
            .clone();
        if let Some(warning) =
            detect_no_external_output_warning(&voice_outputs_snapshot, &output_indices)
        {
            eprintln!("[start_routing] {}", warning);
            let _ = app_handle.emit("routing-warning", warning);
        }
    }

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

    // Snapshot the calibration profile at routing-start so the bridge
    // applies it once at GuitarInput construction. Mid-session changes
    // (load_calibration_profile from the UI, or a save_calibration_profile
    // after a sweep) DO propagate live now via state.live_guitar_pipeline
    // — the bridge publishes its pipeline Arc on construction, and the
    // calibration commands try-lock + push the new profile. A source
    // toggle (MIDI → guitar) without restarting routing still requires
    // a restart because we'd need to construct a new bridge. The snapshot
    // here is the construction-time profile; runtime updates flow through
    // the live_guitar_pipeline slot.
    let live_pipeline_handle = if is_guitar {
        Some(Arc::clone(&state.live_guitar_pipeline))
    } else {
        None
    };
    let calibration_profile_snapshot = if is_guitar {
        Some(
            state
                .calibration_profile
                .lock()
                .map_err(|e| e.to_string())?
                .clone(),
        )
    } else {
        None
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
    // Clear the live guitar pipeline slot unconditionally before
    // starting the new router. If the previous routing was guitar
    // and the new one is MIDI, the slot would otherwise retain an
    // Arc to a dead pipeline — subsequent calibration commands
    // would silently mutate the zombie. Brutal-critic round 3
    // CRITICAL. The slot will be re-populated by GuitarBridge::new
    // below when the new bridge succeeds.
    if let Ok(mut slot) = state.live_guitar_pipeline.lock() {
        *slot = None;
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

    // #91 commit A: share the Companion orchestrator with the router
    // thread. Defaults to enabled=false with zero Lanes — tick()
    // short-circuits and produces no DispatchOps until Lanes
    // register and the master switch flips.
    let companion = Arc::clone(&state.companion);

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
            calibration_profile_snapshot,
            live_pipeline_handle,
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
            companion,
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

    // Drop the live pipeline handle so subsequent calibration commands
    // know there's nothing running to hot-swap into. The cpal stream
    // owned by GuitarBridge is dropped via the router-thread teardown.
    if let Ok(mut slot) = state.live_guitar_pipeline.lock() {
        *slot = None;
    }

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
    calibration_profile: Option<contrapunk::audio::guitar::GuitarCalibrationProfile>,
    live_pipeline_handle: Option<
        Arc<Mutex<Option<Arc<Mutex<contrapunk::audio::guitar_input::GuitarInput>>>>>,
    >,
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
    companion: Arc<Mutex<crate::companion::Companion>>,
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
            calibration_profile,
            live_pipeline_handle,
            tx,
            Some(signal_tx),
        )
        .map_err(|e| anyhow::anyhow!("Guitar bridge error: {}", e))?;
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

    // Per-lane attribution sets — kept distinct from `harmony_notes`
    // so the Piano UI can color canon (gold) and counterpoint (lime)
    // emissions separately. `dispatch_companion_ops` inserts/removes
    // here based on the lane tag, *in addition to* the unified
    // `harmony_notes` set the rest of the router still reads.
    let canon_notes: Arc<Mutex<HashSet<u8>>> = Arc::new(Mutex::new(HashSet::new()));
    let counterpoint_notes: Arc<Mutex<HashSet<u8>>> = Arc::new(Mutex::new(HashSet::new()));

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

        // #91 commit A: tick the Companion orchestrator. When
        // `enabled = false` (the default), tick() short-circuits and
        // returns an empty Vec — zero overhead per iteration. When
        // Lanes are registered and the master switch is on, the
        // returned ops are translated to dispatch_voice /
        // broadcast_note_off via dispatch_companion_ops.
        //
        // Held briefly across only this loop iteration so Tauri
        // command handlers (enable/disable, register Lane — future
        // commits) can acquire the lock between ticks.
        {
            let num_ports = output_router.connection_count();
            let tagged = {
                let mut c = companion.lock().unwrap_or_else(|e| e.into_inner());
                c.tick_tagged(&engine)
            };
            dispatch_companion_ops(
                &tagged,
                num_ports,
                &synth_tx,
                &mut output_router,
                &harmony_notes,
                &canon_notes,
                &counterpoint_notes,
            );
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
            {
                let mut h = harmony_notes.lock().unwrap_or_else(|e| e.into_inner());
                *h = new_harmonies;
            }
            {
                let mut b = borrowed_notes.lock().unwrap_or_else(|e| e.into_inner());
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
            let pitch_bend_msg = cents_to_pitch_bend_msg(current_detune);
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

                    // Issue #90 fast-path: CC 123 = All Notes Off (MIDI
                    // standard panic). Drain every tracked note and send
                    // NoteOff downstream so the user can recover from
                    // dropped Note-Offs / MPE channel rotation / device
                    // disconnect mid-phrase without restarting routing.
                    // The full reconcile-against-engine.active_notes
                    // story is deferred — this gives users a one-button
                    // escape today.
                    if cc_number == 123 {
                        let notes_to_release =
                            drain_all_tracked_notes(&input_notes, &harmony_notes, &borrowed_notes);
                        let num_ports = output_router.connection_count();
                        for n in notes_to_release {
                            broadcast_note_off(n, num_ports, &synth_tx, &mut output_router);
                        }
                        eprintln!("[router] CC 123 panic: cleared all tracked notes");
                        // Continue to also forward to UI below so the
                        // Performance view's CC mapping still sees it.
                    }

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
                    // #91 commit B: ask the Companion's input-pipeline
                    // Lanes to inspect this event first. If any Lane
                    // returns `suppress_default = true`, skip the
                    // existing harmony dispatch — the Lane has taken
                    // over for this event. Returned ops are dispatched
                    // through the same translator as tick() ops.
                    let mut suppress_default = false;
                    if let Some(ev) = midi_bytes_to_input_event(&message) {
                        let (tagged, sup) = {
                            let mut c = companion.lock().unwrap_or_else(|e| e.into_inner());
                            c.on_input_tagged(ev, &engine)
                        };
                        let num_ports = output_router.connection_count();
                        dispatch_companion_ops(
                            &tagged,
                            num_ports,
                            &synth_tx,
                            &mut output_router,
                            &harmony_notes,
                            &canon_notes,
                            &counterpoint_notes,
                        );
                        suppress_default = sup;
                    }
                    if !suppress_default {
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
                let canon = canon_notes.lock().unwrap_or_else(|e| e.into_inner());
                let cp = counterpoint_notes.lock().unwrap_or_else(|e| e.into_inner());
                let ch_name = chord_name.lock().unwrap_or_else(|e| e.into_inner());
                build_note_update_payload(
                    &in_notes,
                    &harm_notes,
                    &borr_notes,
                    ch_name.clone(),
                    last_borrowed_from,
                    current_key,
                    &canon,
                    &cp,
                )
            };
            let _ = app_handle.emit("note-update", payload);

            // Emit guitar signal info for UI (drain latest from channel)
            if is_guitar {
                let mut latest_signal = None;
                while let Ok(sig) = signal_rx.try_recv() {
                    latest_signal = Some(sig);
                }
                if let Some(sig) = latest_signal {
                    let _ = app_handle.emit("guitar-signal", build_guitar_signal_payload(sig));
                }
            }
        }
    }

    // Clear note state on exit. Per-lane sets (canon_notes /
    // counterpoint_notes) MUST also be cleared — otherwise stale
    // gold/lime piano colors persist across stop/start cycles
    // until a NoteOff for those exact pitches arrives via the
    // next session.
    {
        let mut notes = input_notes.lock().unwrap_or_else(|e| e.into_inner());
        notes.clear();
    }
    {
        let mut notes = harmony_notes.lock().unwrap_or_else(|e| e.into_inner());
        notes.clear();
    }
    {
        let mut notes = borrowed_notes.lock().unwrap_or_else(|e| e.into_inner());
        notes.clear();
    }
    {
        let mut notes = canon_notes.lock().unwrap_or_else(|e| e.into_inner());
        notes.clear();
    }
    {
        let mut notes = counterpoint_notes.lock().unwrap_or_else(|e| e.into_inner());
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

/// Translate raw MIDI bytes into a Companion `InputEvent` for the
/// `on_input` pipeline. Returns `None` for messages that aren't one
/// of the three supported event types (NoteOn / NoteOff / Cc) — those
/// fall through to the legacy router path untouched.
///
/// Pure: takes a byte slice, returns Option. No I/O, no locks.
fn midi_bytes_to_input_event(bytes: &[u8]) -> Option<crate::companion::InputEvent> {
    use crate::companion::InputEvent;
    if bytes.len() < 3 {
        return None;
    }
    let status = bytes[0] & 0xF0;
    let channel = bytes[0] & 0x0F;
    match status {
        // NoteOn with velocity 0 is the wmidi-equivalent NoteOff —
        // honor that convention here too so Lanes see a consistent
        // input shape across controllers.
        0x90 if bytes[2] != 0 => Some(InputEvent::NoteOn {
            note: bytes[1],
            velocity: bytes[2],
            channel,
        }),
        0x80 | 0x90 => Some(InputEvent::NoteOff {
            note: bytes[1],
            channel,
        }),
        0xB0 => Some(InputEvent::Cc {
            number: bytes[1],
            value: bytes[2],
            channel,
        }),
        _ => None,
    }
}

/// Translate `Companion::tick`'s `DispatchOp` outputs into the
/// existing `dispatch_voice` / `broadcast_note_off` calls so Lanes
/// can drive notes through the same routing fabric harmony voices
/// use. Pure dispatch — no decisions, no state mutation beyond the
/// existing per-call helpers.
///
/// `DispatchOp::AllNotesOff { ports }` ignores its `ports` list for
/// now and broadcasts to every connected output (matches the existing
/// CC 123 panic-drain behavior in the router). Per-port targeting
/// is deferred until the audio-graph milestone introduces an
/// `InstrumentId` model.
fn dispatch_companion_ops(
    tagged: &[(&'static str, crate::companion::DispatchOp)],
    num_ports: usize,
    synth_tx: &mpsc::Sender<SynthEvent>,
    output: &mut OutputRouter,
    harmony_notes: &Arc<Mutex<HashSet<u8>>>,
    canon_notes: &Arc<Mutex<HashSet<u8>>>,
    counterpoint_notes: &Arc<Mutex<HashSet<u8>>>,
) {
    use crate::companion::DispatchOp;
    for (lane, op) in tagged {
        // Per-lane set the note belongs to, if any. Other lane tags
        // (or AllNotesOff) leave both untouched.
        let lane_set: Option<&Arc<Mutex<HashSet<u8>>>> = match *lane {
            "canon" => Some(canon_notes),
            "counterpoint" => Some(counterpoint_notes),
            _ => None,
        };
        match op {
            DispatchOp::NoteOn {
                target,
                note,
                velocity,
                channel,
            } => {
                dispatch_voice(
                    *target,
                    *channel,
                    VoiceDispatch::NoteOn {
                        note: *note,
                        velocity: *velocity,
                    },
                    num_ports,
                    synth_tx,
                    output,
                );
                // Track in the unified harmony set so the Piano knows
                // a note is sounding even without lane attribution.
                {
                    let mut h = harmony_notes.lock().unwrap_or_else(|e| e.into_inner());
                    h.insert(*note);
                }
                // Plus the per-lane set so the Piano can color it.
                if let Some(set) = lane_set {
                    let mut s = set.lock().unwrap_or_else(|e| e.into_inner());
                    s.insert(*note);
                }
            }
            DispatchOp::NoteOff {
                target,
                note,
                channel,
            } => {
                dispatch_voice(
                    *target,
                    *channel,
                    VoiceDispatch::NoteOff {
                        note: *note,
                        velocity: 0,
                    },
                    num_ports,
                    synth_tx,
                    output,
                );
                {
                    let mut h = harmony_notes.lock().unwrap_or_else(|e| e.into_inner());
                    h.remove(note);
                }
                if let Some(set) = lane_set {
                    let mut s = set.lock().unwrap_or_else(|e| e.into_inner());
                    s.remove(note);
                }
            }
            DispatchOp::AllNotesOff { .. } => {
                // Per-port `ports` field deferred to audio-graph
                // milestone; for now broadcast all 16 MIDI channels'
                // CC 123 to every connected output, matching the
                // existing router CC 123 panic path.
                for ch in 0u8..16 {
                    let msg = [0xB0 | ch, 123u8, 0u8];
                    for port in 0..num_ports {
                        let _ = output.send_to_port(port, &msg);
                    }
                }
            }
        }
    }
}

/// Drain every tracked note from the three router HashSets and return
/// the union so the caller can dispatch NoteOff for each. Handles the
/// CC 123 (All Notes Off) panic path in run_tauri_router.
///
/// Acquires the three locks in a fixed order (input → harmony →
/// borrowed) to avoid deadlock with the rest of the router which
/// also reads them in similar order. Recovers from poisoned mutexes
/// rather than panicking — matches the convention in the router's
/// emit loop.
fn drain_all_tracked_notes(
    input_notes: &Arc<Mutex<HashSet<u8>>>,
    harmony_notes: &Arc<Mutex<HashSet<u8>>>,
    borrowed_notes: &Arc<Mutex<HashSet<u8>>>,
) -> HashSet<u8> {
    let union: HashSet<u8> = {
        let in_n = input_notes.lock().unwrap_or_else(|e| e.into_inner());
        let harm = harmony_notes.lock().unwrap_or_else(|e| e.into_inner());
        let borr = borrowed_notes.lock().unwrap_or_else(|e| e.into_inner());
        in_n.iter()
            .chain(harm.iter())
            .chain(borr.iter())
            .copied()
            .collect()
    };
    // Now clear each set (separate scope so the prior read locks
    // are dropped before we acquire write locks).
    {
        let mut s = input_notes.lock().unwrap_or_else(|e| e.into_inner());
        s.clear();
    }
    {
        let mut s = harmony_notes.lock().unwrap_or_else(|e| e.into_inner());
        s.clear();
    }
    {
        let mut s = borrowed_notes.lock().unwrap_or_else(|e| e.into_inner());
        s.clear();
    }
    union
}

/// Detect the issue #14 silent-no-output condition: user selected one
/// or more external MIDI ports for routing, but no voice in the
/// per-voice routing table actually points at an external port. The
/// engine's per-voice routing (`VoiceOutputTarget::default() = Synth`)
/// will silently swallow every harmony into the internal synth, which
/// looks like "MIDI out broken" to the user.
///
/// Returns `Some(message)` describing the problem when the bad state
/// is detected, or `None` when configuration is consistent.
///
/// Pure: takes slices, returns Option<String>. No I/O, no locks.
fn detect_no_external_output_warning(
    voice_outputs: &[crate::state::VoiceOutputTarget],
    output_indices: &[usize],
) -> Option<String> {
    if output_indices.is_empty() {
        // No external ports selected at all — synth-only is what the
        // user picked. Don't warn.
        return None;
    }
    let any_external = voice_outputs
        .iter()
        .any(|t| matches!(t, crate::state::VoiceOutputTarget::MidiPort { .. }));
    if any_external {
        return None;
    }
    Some(format!(
        "Routing started with {} external MIDI port(s) selected, but no voice is routed to an external port. \
         All voices currently route to the internal synth; external instruments will receive nothing. \
         Open Voice Routing in the UI to send harmonies to your external port(s).",
        output_indices.len()
    ))
}

/// Encode a detune-in-cents value as a 3-byte MIDI pitch-bend message
/// on channel 0. Pure: takes cents (any i32), returns [status, LSB, MSB].
///
/// Conventions:
/// - Pitch bend range is ±2 semitones (±200 cents) per General MIDI default.
/// - Center (no bend) is 14-bit value 8192 (0x2000) → bytes [0xE0, 0x00, 0x40].
/// - The 14-bit value is clamped to [0, 16383]; out-of-range inputs (beyond
///   ±200 cents) saturate at the endpoints rather than wrapping or panicking.
/// - Channel bits in the status byte are zero (channel 1 in 1-indexed MIDI).
///
/// Extracted from the inline detune-tick block in run_tauri_router so the
/// 14-bit packing is independently testable. The send-to-all-ports loop
/// stays at the call site since it's pure I/O.
fn cents_to_pitch_bend_msg(cents: i32) -> [u8; 3] {
    const MAX_CENTS: i32 = 200; // ±2 semitones
    let bend_f = (cents as f64 / MAX_CENTS as f64) * 8192.0 + 8192.0;
    // Clamp BEFORE the cast — Rust's f64-to-u16 saturating cast clamps to
    // [0, 65535], but we need clamping to the MIDI [0, 16383] range. A
    // pre-clamp on the i32-equivalent avoids both negative-saturation
    // surprises and the >16383 overshoot.
    let bend_clamped = bend_f.round().clamp(0.0, 16383.0) as u16;
    let lsb = (bend_clamped & 0x7F) as u8;
    let msb = ((bend_clamped >> 7) & 0x7F) as u8;
    [0xE0, lsb, msb]
}

/// Build the payload sent on the "note-update" Tauri event.
///
/// Pure function: takes references to the three note sets + the
/// strings already extracted from the engine, returns the payload.
/// The router holds the locks; this function never blocks on I/O.
///
/// Sorts each note vector for stable UI rendering — without sorting,
/// HashSet iteration order would cause the piano-roll display to
/// shuffle pips between frames.
#[allow(clippy::too_many_arguments)]
fn build_note_update_payload(
    input_notes: &HashSet<u8>,
    harmony_notes: &HashSet<u8>,
    borrowed_notes: &HashSet<u8>,
    chord_name: String,
    last_borrowed_from: String,
    current_key: String,
    canon_notes: &HashSet<u8>,
    counterpoint_notes: &HashSet<u8>,
) -> NoteUpdatePayload {
    let mut in_vec: Vec<u8> = input_notes.iter().copied().collect();
    let mut harm_vec: Vec<u8> = harmony_notes.iter().copied().collect();
    let mut borr_vec: Vec<u8> = borrowed_notes.iter().copied().collect();
    let mut canon_vec: Vec<u8> = canon_notes.iter().copied().collect();
    let mut cp_vec: Vec<u8> = counterpoint_notes.iter().copied().collect();
    in_vec.sort_unstable();
    harm_vec.sort_unstable();
    borr_vec.sort_unstable();
    canon_vec.sort_unstable();
    cp_vec.sort_unstable();
    NoteUpdatePayload {
        input_notes: in_vec,
        harmony_notes: harm_vec,
        borrowed_notes: borr_vec,
        chord_name,
        last_borrowed_from,
        current_key,
        canon_notes: canon_vec,
        counterpoint_notes: cp_vec,
    }
}

/// Build the payload sent on the "guitar-signal" Tauri event.
///
/// Pure function: converts a `GuitarSignalInfo` from the audio
/// pipeline into the UI-facing payload, applying the standard
/// frequency-to-note mapping with two thresholds (frequency > 20 Hz
/// AND clarity > 0.3) to avoid surfacing noise as fake note
/// detections in the readout strip.
fn build_guitar_signal_payload(sig: crate::guitar_bridge::GuitarSignalInfo) -> GuitarSignalPayload {
    const NOTE_NAMES: [&str; 12] = [
        "C", "C#", "D", "Eb", "E", "F", "F#", "G", "Ab", "A", "Bb", "B",
    ];
    let (note_name, midi_note) = if let Some(freq) = sig.frequency {
        if freq > 20.0 && sig.clarity > 0.3 {
            let midi = (12.0 * (freq / 440.0).log2() + 69.0).round() as i32;
            let midi_u8 = midi.clamp(0, 127) as u8;
            let name_idx = (midi_u8 % 12) as usize;
            let octave = (midi_u8 as i32 / 12) - 1;
            (format!("{}{}", NOTE_NAMES[name_idx], octave), midi_u8)
        } else {
            (String::new(), 0)
        }
    } else {
        (String::new(), 0)
    };
    GuitarSignalPayload {
        rms: sig.rms,
        frequency: sig.frequency,
        clarity: sig.clarity,
        note_state: sig.note_state,
        note_name,
        midi_note,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::guitar_bridge::GuitarSignalInfo;
    use std::collections::HashSet;

    /// Note vectors must come out sorted ascending so the UI piano-roll
    /// renders pips in a stable order across frames. Without sorting,
    /// HashSet iteration order would shuffle the display.
    #[test]
    fn test_build_note_update_payload_sorts_notes() {
        let input: HashSet<u8> = [67u8, 60, 64].iter().copied().collect();
        let harmony: HashSet<u8> = [55u8, 71, 60].iter().copied().collect();
        let borrowed: HashSet<u8> = [70u8].iter().copied().collect();
        let canon: HashSet<u8> = [71u8, 60].iter().copied().collect();
        let cp: HashSet<u8> = [55u8].iter().copied().collect();
        let payload = build_note_update_payload(
            &input,
            &harmony,
            &borrowed,
            "Cmaj7".into(),
            "Aeolian".into(),
            "C".into(),
            &canon,
            &cp,
        );
        assert_eq!(payload.input_notes, vec![60, 64, 67]);
        assert_eq!(payload.harmony_notes, vec![55, 60, 71]);
        assert_eq!(payload.borrowed_notes, vec![70]);
        assert_eq!(payload.canon_notes, vec![60, 71]);
        assert_eq!(payload.counterpoint_notes, vec![55]);
        assert_eq!(payload.chord_name, "Cmaj7");
        assert_eq!(payload.last_borrowed_from, "Aeolian");
        assert_eq!(payload.current_key, "C");
    }

    #[test]
    fn test_build_note_update_payload_empty_state() {
        let empty: HashSet<u8> = HashSet::new();
        let payload = build_note_update_payload(
            &empty,
            &empty,
            &empty,
            String::new(),
            String::new(),
            "C".into(),
            &empty,
            &empty,
        );
        assert!(payload.input_notes.is_empty());
        assert!(payload.harmony_notes.is_empty());
        assert!(payload.borrowed_notes.is_empty());
        assert!(payload.canon_notes.is_empty());
        assert!(payload.counterpoint_notes.is_empty());
        assert!(payload.chord_name.is_empty());
        assert_eq!(payload.current_key, "C");
    }

    /// A clean signal at A4 (440 Hz) above the clarity threshold must
    /// produce the canonical name + MIDI number.
    #[test]
    fn test_build_guitar_signal_payload_a4() {
        let sig = GuitarSignalInfo {
            rms: 0.1,
            frequency: Some(440.0),
            clarity: 0.8,
            note_state: 2,
        };
        let payload = build_guitar_signal_payload(sig);
        assert_eq!(payload.note_name, "A4");
        assert_eq!(payload.midi_note, 69);
        assert_eq!(payload.note_state, 2);
    }

    /// Frequency below 20 Hz must NOT produce a note — that's
    /// sub-audible noise. The clarity threshold should also gate.
    #[test]
    fn test_build_guitar_signal_payload_rejects_subaudible_and_low_clarity() {
        let subaudible = GuitarSignalInfo {
            rms: 0.05,
            frequency: Some(10.0),
            clarity: 0.9,
            note_state: 0,
        };
        let p1 = build_guitar_signal_payload(subaudible);
        assert_eq!(p1.note_name, "");
        assert_eq!(p1.midi_note, 0);

        let noisy = GuitarSignalInfo {
            rms: 0.05,
            frequency: Some(440.0),
            clarity: 0.2,
            note_state: 0,
        };
        let p2 = build_guitar_signal_payload(noisy);
        assert_eq!(p2.note_name, "");
        assert_eq!(p2.midi_note, 0);
    }

    /// CC 123 panic must drain ALL three tracked sets — input, harmony,
    /// and borrowed. The function returns the union so callers can fan
    /// NoteOff to every downstream port.
    #[test]
    fn test_drain_all_tracked_notes_collects_union_and_clears() {
        let input = Arc::new(Mutex::new(
            [60u8, 64].iter().copied().collect::<HashSet<u8>>(),
        ));
        let harmony = Arc::new(Mutex::new(
            [67u8, 71].iter().copied().collect::<HashSet<u8>>(),
        ));
        let borrowed = Arc::new(Mutex::new([70u8].iter().copied().collect::<HashSet<u8>>()));
        let drained = drain_all_tracked_notes(&input, &harmony, &borrowed);
        let expected: HashSet<u8> = [60, 64, 67, 70, 71].iter().copied().collect();
        assert_eq!(drained, expected);
        // All three sets must be empty after the drain.
        assert!(input.lock().unwrap().is_empty());
        assert!(harmony.lock().unwrap().is_empty());
        assert!(borrowed.lock().unwrap().is_empty());
    }

    /// Empty sets must produce an empty union without panicking.
    #[test]
    fn test_drain_all_tracked_notes_empty_returns_empty() {
        let input = Arc::new(Mutex::new(HashSet::<u8>::new()));
        let harmony = Arc::new(Mutex::new(HashSet::<u8>::new()));
        let borrowed = Arc::new(Mutex::new(HashSet::<u8>::new()));
        let drained = drain_all_tracked_notes(&input, &harmony, &borrowed);
        assert!(drained.is_empty());
    }

    /// Overlapping notes across sets must dedupe in the union (HashSet
    /// semantics) so the caller doesn't fire duplicate NoteOffs.
    #[test]
    fn test_drain_all_tracked_notes_dedups_overlaps() {
        let input = Arc::new(Mutex::new([60u8].iter().copied().collect::<HashSet<u8>>()));
        let harmony = Arc::new(Mutex::new(
            [60u8, 64].iter().copied().collect::<HashSet<u8>>(),
        ));
        let borrowed = Arc::new(Mutex::new(HashSet::<u8>::new()));
        let drained = drain_all_tracked_notes(&input, &harmony, &borrowed);
        let expected: HashSet<u8> = [60, 64].iter().copied().collect();
        assert_eq!(drained, expected);
    }

    /// Regression: `drain_all_tracked_notes` must recover from a
    /// poisoned mutex via the established `unwrap_or_else(|e|
    /// e.into_inner())` pattern (and equivalent), not panic. Without
    /// this, a panic in any thread that holds these locks (e.g. an
    /// inner unwrap fault during reharm) cascades into a dead router.
    /// First poison-recovery test in the project — proves the
    /// convention on one site is enough to prove it on all.
    #[test]
    fn test_drain_all_tracked_notes_survives_poisoned_lock() {
        use std::thread;
        let input = Arc::new(Mutex::new([60u8].iter().copied().collect::<HashSet<u8>>()));
        let harmony = Arc::new(Mutex::new(
            [60u8, 64].iter().copied().collect::<HashSet<u8>>(),
        ));
        let borrowed = Arc::new(Mutex::new(HashSet::<u8>::new()));

        // Poison the middle lock from a spawned thread by holding it
        // while panicking. .join() catches the panic so the test
        // process survives.
        let poisoner = {
            let h = Arc::clone(&harmony);
            thread::spawn(move || {
                let _guard = h.lock().unwrap();
                panic!("intentional poison for test");
            })
        };
        let _ = poisoner.join();
        assert!(
            harmony.is_poisoned(),
            "poisoner thread should have flagged the mutex"
        );

        // drain_all_tracked_notes must still see the wrapped data
        // (60, 64 from the harmony set) instead of panicking.
        let drained = drain_all_tracked_notes(&input, &harmony, &borrowed);
        let expected: HashSet<u8> = [60, 64].iter().copied().collect();
        assert_eq!(
            drained, expected,
            "drain must recover via .unwrap_or_else(|e| e.into_inner()) on poisoned mutex"
        );
    }

    /// MIDI NoteOn (status 0x9X) with non-zero velocity decodes to
    /// the InputEvent::NoteOn variant carrying note + velocity +
    /// channel. The channel is the low nibble of the status byte.
    #[test]
    fn test_midi_bytes_to_input_event_note_on() {
        use crate::companion::InputEvent;
        let ev = midi_bytes_to_input_event(&[0x91, 60, 100]).unwrap();
        match ev {
            InputEvent::NoteOn {
                note,
                velocity,
                channel,
            } => {
                assert_eq!(note, 60);
                assert_eq!(velocity, 100);
                assert_eq!(channel, 1);
            }
            _ => panic!("expected NoteOn, got {:?}", ev),
        }
    }

    /// MIDI convention: NoteOn with velocity 0 is a NoteOff. Verify
    /// that decodes correctly so Lanes see a normalized input shape
    /// regardless of which form the controller sent.
    #[test]
    fn test_midi_bytes_to_input_event_note_on_zero_velocity_is_off() {
        use crate::companion::InputEvent;
        let ev = midi_bytes_to_input_event(&[0x90, 60, 0]).unwrap();
        assert!(matches!(
            ev,
            InputEvent::NoteOff {
                note: 60,
                channel: 0
            }
        ));
    }

    /// Explicit NoteOff (status 0x8X) decodes regardless of velocity.
    #[test]
    fn test_midi_bytes_to_input_event_note_off() {
        use crate::companion::InputEvent;
        let ev = midi_bytes_to_input_event(&[0x82, 64, 50]).unwrap();
        assert!(matches!(
            ev,
            InputEvent::NoteOff {
                note: 64,
                channel: 2
            }
        ));
    }

    /// Control Change (status 0xBX) decodes to InputEvent::Cc.
    #[test]
    fn test_midi_bytes_to_input_event_cc() {
        use crate::companion::InputEvent;
        let ev = midi_bytes_to_input_event(&[0xB3, 7, 100]).unwrap();
        match ev {
            InputEvent::Cc {
                number,
                value,
                channel,
            } => {
                assert_eq!(number, 7);
                assert_eq!(value, 100);
                assert_eq!(channel, 3);
            }
            _ => panic!("expected Cc, got {:?}", ev),
        }
    }

    /// Other status bytes (pitch bend, aftertouch, sysex, etc.) fall
    /// through with None so the legacy router path handles them.
    #[test]
    fn test_midi_bytes_to_input_event_passthrough() {
        assert!(midi_bytes_to_input_event(&[0xE0, 0, 64]).is_none()); // pitch bend
        assert!(midi_bytes_to_input_event(&[0xD0, 100, 0]).is_none()); // channel pressure
        assert!(midi_bytes_to_input_event(&[]).is_none());
        assert!(midi_bytes_to_input_event(&[0x90]).is_none()); // too short
    }

    /// Issue #14 detection: external ports selected + all voices Synth →
    /// the warning must fire. This is the support-thread case ("MIDI out
    /// not producing messages for some users").
    #[test]
    fn test_detect_no_external_output_warning_fires_when_all_synth() {
        use crate::state::VoiceOutputTarget;
        let voices = vec![VoiceOutputTarget::Synth; 8];
        let result = detect_no_external_output_warning(&voices, &[0, 1]);
        assert!(
            result.is_some(),
            "expected a warning when no voice routes to an external port"
        );
        let msg = result.unwrap();
        assert!(
            msg.contains("external MIDI port"),
            "msg should mention external MIDI: {}",
            msg
        );
        assert!(
            msg.contains("internal synth"),
            "msg should explain where it's going: {}",
            msg
        );
    }

    /// At least one voice routed to an external MIDI port is the
    /// happy path — no warning even if other voices stay on Synth.
    #[test]
    fn test_detect_no_external_output_warning_silent_when_any_external() {
        use crate::state::VoiceOutputTarget;
        let voices = vec![
            VoiceOutputTarget::Synth,
            VoiceOutputTarget::MidiPort { port: 0 },
            VoiceOutputTarget::Synth,
            VoiceOutputTarget::Synth,
        ];
        assert!(detect_no_external_output_warning(&voices, &[0]).is_none());
    }

    /// User chose synth-only (no external ports selected) → never warn.
    /// They explicitly opted out of MIDI; the bug doesn't apply.
    #[test]
    fn test_detect_no_external_output_warning_silent_when_no_external_ports_selected() {
        use crate::state::VoiceOutputTarget;
        let voices = vec![VoiceOutputTarget::Synth; 8];
        assert!(detect_no_external_output_warning(&voices, &[]).is_none());
    }

    /// All voices set to `Off` is a legitimate user choice (mute) — the
    /// warning logic only cares about MidiPort presence, so Off-only
    /// still warns since external ports were selected but nothing
    /// reaches them. That matches user intent: "I wanted MIDI out".
    #[test]
    fn test_detect_no_external_output_warning_fires_when_all_off() {
        use crate::state::VoiceOutputTarget;
        let voices = vec![VoiceOutputTarget::Off; 8];
        assert!(detect_no_external_output_warning(&voices, &[0]).is_some());
    }

    /// Zero detune must produce the canonical center pitch-bend: status
    /// 0xE0, LSB=0x00, MSB=0x40 (14-bit value 8192).
    #[test]
    fn test_cents_to_pitch_bend_msg_zero_is_center() {
        assert_eq!(cents_to_pitch_bend_msg(0), [0xE0, 0x00, 0x40]);
    }

    /// +200 cents (max upward bend) must produce the 14-bit max 16383.
    #[test]
    fn test_cents_to_pitch_bend_msg_max_up() {
        assert_eq!(cents_to_pitch_bend_msg(200), [0xE0, 0x7F, 0x7F]);
    }

    /// -200 cents (max downward bend) must produce 14-bit 0.
    #[test]
    fn test_cents_to_pitch_bend_msg_max_down() {
        assert_eq!(cents_to_pitch_bend_msg(-200), [0xE0, 0x00, 0x00]);
    }

    /// Out-of-range inputs must clamp at the endpoints (no panic, no wrap).
    /// Especially important for negative inputs — f64-to-u16 saturating
    /// casts in older Rust versions clamped negatives to 0, which we want,
    /// but the explicit pre-clamp here makes it robust across compilers.
    #[test]
    fn test_cents_to_pitch_bend_msg_clamps_out_of_range() {
        assert_eq!(cents_to_pitch_bend_msg(10_000), [0xE0, 0x7F, 0x7F]);
        assert_eq!(cents_to_pitch_bend_msg(-10_000), [0xE0, 0x00, 0x00]);
    }

    /// +100 cents = halfway up = 14-bit 12288 (0x3000): LSB=0, MSB=0x60.
    /// Regression guard: an off-by-one or wrong-shift bug would catch here.
    #[test]
    fn test_cents_to_pitch_bend_msg_half_up() {
        assert_eq!(cents_to_pitch_bend_msg(100), [0xE0, 0x00, 0x60]);
    }

    /// No frequency at all (idle / silence) must produce an empty name.
    #[test]
    fn test_build_guitar_signal_payload_no_frequency() {
        let silent = GuitarSignalInfo {
            rms: 0.001,
            frequency: None,
            clarity: 0.0,
            note_state: 0,
        };
        let payload = build_guitar_signal_payload(silent);
        assert_eq!(payload.note_name, "");
        assert_eq!(payload.midi_note, 0);
    }
}
