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
use crate::state::{AppState, HeldVoice, VoiceOutputTarget};

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

    // Beat-aligned chord trigger: master enable + pattern config. When
    // enabled, the router retriggers harmony on each pattern cell-on
    // boundary (Live mode). Pushed by the frontend pattern store via
    // `set_pattern_enabled` / `set_pattern_config`.
    let pattern_enabled = Arc::clone(&state.pattern_enabled);
    let pattern_config = Arc::clone(&state.pattern_config);

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
            pattern_enabled,
            pattern_config,
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
    pattern_enabled: Arc<std::sync::atomic::AtomicBool>,
    pattern_config: Arc<Mutex<crate::state::PatternConfig>>,
) -> anyhow::Result<()> {
    let mut last_pattern_cell: Option<usize> = None;
    let mut last_pattern_cell_on: bool = false;
    // Routing-aware tracker for currently-sounding harmony voices.
    // Indexed by input MIDI note (the user's held key); each entry is
    // the list of harmony voices the engine produced for that input,
    // each carrying the routing target it was attacked through.
    //
    // Owned by the router thread; not exposed via AppState because no
    // other thread needs to read it. Pattern-tick attacks/releases use
    // this to honor per-voice routing without re-running the harmony
    // engine; panic-replay rebuilds it after engine-config changes.
    let held_harmonies: Arc<Mutex<HashMap<u8, Vec<HeldVoice>>>> =
        Arc::new(Mutex::new(HashMap::new()));
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
        if !panic_will_fire && pattern_enabled.load(Ordering::SeqCst) && transport.is_running() {
            let total_beats = transport.total_beats();
            let (current_cell, cell_is_on, input_mode) = {
                let cfg = pattern_config.lock().unwrap_or_else(|e| e.into_inner());
                if cfg.cells.is_empty() {
                    (0usize, false, cfg.input_mode)
                } else {
                    let idx = cfg.cell_index_at(total_beats);
                    let on = cfg.cells.get(idx).copied().unwrap_or(false);
                    (idx, on, cfg.input_mode)
                }
            };
            // First iteration after pattern-enable (or transport-start
            // while enabled): seed `last_pattern_cell` to the current
            // cell without firing a transition. This phase-aligns to
            // the music — the next genuine cell-boundary crossing
            // produces the first attack. Without this, enabling the
            // pattern mid-cell fires a misaligned NoteOn the very
            // next router-loop iteration.
            //
            // Trade-off: on cold-start (pattern already enabled when
            // transport begins), cell 0 is silent for the first cell
            // duration. Acceptable — minor lead-in delay vs. spurious
            // mid-cell attacks on every panel toggle.
            if last_pattern_cell.is_none() {
                last_pattern_cell = Some(current_cell);
                last_pattern_cell_on = cell_is_on;
            } else if Some(current_cell) != last_pattern_cell {
                let prev_was_on = last_pattern_cell_on;
                last_pattern_cell = Some(current_cell);
                last_pattern_cell_on = cell_is_on;

                // Snapshot the held voices once for both off- and on-
                // dispatch. Flatten across all currently-held inputs,
                // then dedupe by (note, target, channel) — when
                // polyphonic input has overlapping harmonies (e.g.
                // C and E both producing G in Mirror mode), the same
                // (note, target, channel) tuple appears once per
                // input. Synth voice allocators that don't dedupe
                // would otherwise allocate two voices and only one
                // would release on cell-off.
                let mut voices: Vec<HeldVoice> = held_harmonies
                    .lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .values()
                    .flatten()
                    .copied()
                    .collect();
                {
                    let mut seen: HashSet<(u8, VoiceOutputTarget, u8)> = HashSet::new();
                    voices.retain(|v| seen.insert((v.note, v.target, v.channel)));
                }
                let num_ports_now = output_router.connection_count();

                // Mode semantics:
                //   Live      — every cell-on boundary attacks (and
                //               retriggers consecutive ons). Cell-off
                //               boundaries silence. Staccato / step-seq.
                //   Quantized — currently hidden from the UI (see
                //               InputMode in pattern.svelte.ts). Code
                //               path remains so a stale persisted
                //               value still works; treated as Live.
                //   Gated     — sustained legato: NoteOn only on rising
                //               edge (off→on), NoteOff only on falling
                //               edge (on→off). No retrigger on
                //               consecutive ons.
                use crate::state::PatternInputMode;
                let (do_off, do_on) = match input_mode {
                    PatternInputMode::Live | PatternInputMode::Quantized => {
                        (prev_was_on, cell_is_on)
                    }
                    PatternInputMode::Gated => {
                        // off→on: only NoteOn. on→off: only NoteOff.
                        // on→on, off→off: nothing.
                        let rising = !prev_was_on && cell_is_on;
                        let falling = prev_was_on && !cell_is_on;
                        (falling, rising)
                    }
                };
                if do_off {
                    for v in &voices {
                        match v.target {
                            VoiceOutputTarget::Synth => {
                                let _ = synth_tx.send(SynthEvent::NoteOff { note: v.note });
                            }
                            VoiceOutputTarget::MidiPort { port } if port < num_ports_now => {
                                let msg = [0x80 | (v.channel & 0x0F), v.note, 0];
                                let _ = output_router.send_to_port(port, &msg);
                            }
                            _ => {}
                        }
                    }
                }
                if do_on {
                    for v in &voices {
                        match v.target {
                            VoiceOutputTarget::Synth => {
                                let _ = synth_tx.send(SynthEvent::NoteOn {
                                    note: v.note,
                                    velocity: v.velocity,
                                });
                            }
                            VoiceOutputTarget::MidiPort { port } if port < num_ports_now => {
                                let msg = [0x90 | (v.channel & 0x0F), v.note, v.velocity];
                                let _ = output_router.send_to_port(port, &msg);
                            }
                            _ => {}
                        }
                    }
                }
            }
        } else if !panic_will_fire {
            // Pattern disabled or transport stopped. If we left harmony
            // sounding (last cell was on), drain it per-voice so each
            // voice's actual destination (synth or specific MIDI port)
            // gets the matching NoteOff. Without this, external MIDI
            // gear keeps any pattern-attacked notes hanging.
            //
            // Don't drain when `panic_will_fire`: the panic block below
            // will rebuild harmony state from scratch by replaying held
            // inputs; an extra drain here would NoteOff notes the panic
            // path is about to legitimately re-attack.
            //
            // Disable-pattern + setter in same router iteration is safe
            // by construction: the panic block's `to_release` set is
            // computed as `old_harmonies - new_harmonies` where
            // `old_harmonies` reads `harmony_notes`, which always
            // contains every pattern-attacked note (handle_note_on
            // populates it on the original input event, before pattern
            // ever fires). So pattern-attacked notes that the new
            // engine config no longer wants get released by the diff;
            // notes the new config still wants stay sounding via the
            // attacks the panic block dispatches per-voice. This
            // invariant should be pinned by a regression test once the
            // router-loop logic is factored into a testable pure
            // function — currently the loop spawns threads + side-
            // effects, which makes unit testing impractical (TODO).
            if last_pattern_cell_on {
                let mut voices: Vec<HeldVoice> = held_harmonies
                    .lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .values()
                    .flatten()
                    .copied()
                    .collect();
                // Same dedup as the pattern-tick block above —
                // overlapping harmonies from polyphonic inputs would
                // otherwise produce two NoteOffs to the same target.
                {
                    let mut seen: HashSet<(u8, VoiceOutputTarget, u8)> = HashSet::new();
                    voices.retain(|v| seen.insert((v.note, v.target, v.channel)));
                }
                let num_ports_now = output_router.connection_count();
                for v in &voices {
                    match v.target {
                        VoiceOutputTarget::Synth => {
                            let _ = synth_tx.send(SynthEvent::NoteOff { note: v.note });
                        }
                        VoiceOutputTarget::MidiPort { port } if port < num_ports_now => {
                            let msg = [0x80 | (v.channel & 0x0F), v.note, 0];
                            let _ = output_router.send_to_port(port, &msg);
                        }
                        _ => {}
                    }
                }
            }
            last_pattern_cell = None;
            last_pattern_cell_on = false;
        }

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
            // port (overspray on external is harmless; we don't track
            // per-note routing for releases).
            let num_ports = output_router.connection_count();
            for n in &to_release {
                let _ = synth_tx.send(SynthEvent::NoteOff { note: *n });
                let msg = [0x80, *n, 0]; // NoteOff channel 0
                for p in 0..num_ports {
                    let _ = output_router.send_to_port(p, &msg);
                }
            }

            // Send NoteOn for newly-attacked notes, routed per voice via
            // each replay's port map and the live voice_outputs table.
            let voice_targets: Vec<VoiceOutputTarget> = voice_outputs
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .clone();
            for (midis, port_map) in &per_input {
                // Skip index 0 — that's the user's input note, already
                // sounding from when they pressed the key.
                for (i, &n) in midis.iter().enumerate().skip(1) {
                    if !to_attack.contains(&n) {
                        continue; // already sounding from before
                    }
                    let slot = port_map.get(i).copied().unwrap_or(i);
                    let target = voice_targets.get(slot).copied().unwrap_or_default();
                    match target {
                        VoiceOutputTarget::Synth => {
                            let _ = synth_tx.send(SynthEvent::NoteOn {
                                note: n,
                                velocity: 100,
                            });
                        }
                        VoiceOutputTarget::MidiPort { port } => {
                            if port < num_ports {
                                let msg = [0x90, n, 100]; // NoteOn channel 0
                                let _ = output_router.send_to_port(port, &msg);
                            }
                        }
                        VoiceOutputTarget::Off => {}
                    }
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
            // dispatch from this map.
            //
            // Channel + velocity: rebuilt as 0 / 100 because
            // `take_reharm_inputs()` returns just MIDI numbers and
            // the engine doesn't track per-input channel/velocity
            // through the reharm path (existing limitation, predates
            // these fixes). Pattern dispatch after a panic event
            // will use these defaults for these voices until the
            // user releases and re-presses the input. The panic
            // block's own to_release dispatch above also uses
            // channel 0 for the same reason.
            if let Ok(mut hh) = held_harmonies.lock() {
                hh.clear();
                for (midis, port_map) in &per_input {
                    if midis.is_empty() {
                        continue;
                    }
                    let input_note = midis[0];
                    let mut voices: Vec<HeldVoice> = Vec::with_capacity(midis.len() - 1);
                    for (i, &n) in midis.iter().enumerate().skip(1) {
                        let slot = port_map.get(i).copied().unwrap_or(i);
                        let target = voice_targets.get(slot).copied().unwrap_or_default();
                        voices.push(HeldVoice {
                            note: n,
                            target,
                            channel: 0,
                            velocity: 100,
                        });
                    }
                    hh.insert(input_note, voices);
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

        // Process MIDI messages
        match rx.recv_timeout(Duration::from_millis(5)) {
            Ok(message) => {
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
                    &held_harmonies,
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

    // Clear note state on exit. held_harmonies is also cleared so
    // that any voices left in the tracker (e.g. from a Note-Off the
    // device never sent due to USB glitch / cable yank / MPE channel
    // rotation) don't survive a routing restart and produce ghost-
    // voice retriggers under a freshly-spawned router thread.
    if let Ok(mut notes) = input_notes.lock() {
        notes.clear();
    }
    if let Ok(mut notes) = harmony_notes.lock() {
        notes.clear();
    }
    if let Ok(mut notes) = borrowed_notes.lock() {
        notes.clear();
    }
    if let Ok(mut hh) = held_harmonies.lock() {
        hh.clear();
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
    held_harmonies: &Arc<Mutex<HashMap<u8, Vec<HeldVoice>>>>,
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
                    held_harmonies,
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
                    held_harmonies,
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
                held_harmonies,
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
    held_harmonies: &Arc<Mutex<HashMap<u8, Vec<HeldVoice>>>>,
) {
    let notes = engine.harmonize_note_on(note);
    // Drain any harmonies the engine flagged for explicit release —
    // populated when an auto-key change wiped `active_notes` mid-flight.
    // These would otherwise stay sounding under the old key.
    let stale_releases = engine.take_pending_releases();
    let num_outputs = output.connection_count();

    // Send Note-Offs for stale harmonies before emitting the new ones.
    // Sending to every external port is over-broad (each note went out
    // on a single port), but extra Note-Offs to ports that didn't see
    // the matching Note-On are harmless and we don't track per-note
    // routing on the router side.
    if !stale_releases.is_empty() {
        for &n in &stale_releases {
            let _ = synth_tx.send(SynthEvent::NoteOff { note: u8::from(n) });
            let msg = MidiMessage::NoteOff(channel, n, velocity);
            let mut buf = vec![0u8; msg.bytes_size()];
            let _ = msg.copy_to_slice(&mut buf);
            for port in 0..num_outputs {
                let _ = output.send_to_port(port, &buf);
            }
        }
        let mut harm = harmony_notes.lock().unwrap_or_else(|e| e.into_inner());
        let mut borr = borrowed_notes.lock().unwrap_or_else(|e| e.into_inner());
        for &n in &stale_releases {
            harm.remove(&u8::from(n));
            borr.remove(&u8::from(n));
        }
        // Drop matching entries from the routing-aware tracker so
        // subsequent pattern attacks don't retrigger notes the engine
        // already released. If an input's harmony list empties out,
        // remove the input entry too.
        let stale_set: HashSet<u8> = stale_releases.iter().map(|n| u8::from(*n)).collect();
        let mut hh = held_harmonies.lock().unwrap_or_else(|e| e.into_inner());
        hh.retain(|_, voices| {
            voices.retain(|v| !stale_set.contains(&v.note));
            !voices.is_empty()
        });
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

    // Fan each voice into the built-in synth, gated by voice_outputs.
    for (i, &n) in notes.iter().enumerate() {
        if matches!(target_for(i), VoiceOutputTarget::Synth) {
            let _ = synth_tx.send(SynthEvent::NoteOn {
                note: u8::from(n),
                velocity: u8::from(velocity),
            });
        }
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

    // Send notes to external MIDI outputs, per-voice. Synth and Off
    // skip MIDI entirely; only MidiPort dispatches.
    for (i, &n) in notes.iter().enumerate() {
        if let VoiceOutputTarget::MidiPort { port } = target_for(i) {
            if port >= num_outputs {
                continue;
            }
            let msg = MidiMessage::NoteOn(channel, n, velocity);
            let mut buf = vec![0u8; msg.bytes_size()];
            let _ = msg.copy_to_slice(&mut buf);
            let _ = output.send_to_port(port, &buf);
        }
    }

    // Track this input's voices for routing-aware pattern dispatch.
    // Skip i=0 (the input note itself); only the harmonies are
    // pattern-controllable. Channel + velocity from the input event
    // are captured per voice so pattern-driven NoteOn/NoteOff
    // dispatch on the same channel and at the original velocity —
    // a soft input doesn't get reattacked at full volume on every
    // cell tick, and MPE / multi-channel routing is preserved.
    let channel_idx: u8 = channel.index();
    let velocity_byte: u8 = u8::from(velocity);
    let mut voices_for_input: Vec<HeldVoice> = Vec::with_capacity(notes.len().saturating_sub(1));
    for (i, &n) in notes.iter().enumerate().skip(1) {
        voices_for_input.push(HeldVoice {
            note: u8::from(n),
            target: target_for(i),
            channel: channel_idx,
            velocity: velocity_byte,
        });
    }
    // Release any orphaned voices from a prior entry for this input.
    // Common case: user retriggers the same key while still holding;
    // engine returns the same harmony notes with the same routing →
    // new_notes ⊇ old_notes → no orphan releases. Edge case: user
    // changed `voice_outputs` between presses, or the engine's mode
    // changed mid-session and harmonies differ. Without this, the
    // old voices stay sounding on their captured target with no path
    // to release them.
    //
    // Lock pattern: single acquire for read + install, then drop
    // before dispatching MIDI. Holding the Mutex across
    // `output.send_to_port` would couple lock duration to USB
    // backpressure; held_harmonies is router-thread-local in
    // practice so contention is impossible, but the explicit
    // drop-before-dispatch keeps that guarantee local.
    let new_notes: HashSet<u8> = voices_for_input.iter().map(|v| v.note).collect();
    let orphaned: Vec<HeldVoice> = {
        let mut hh = held_harmonies.lock().unwrap_or_else(|e| e.into_inner());
        let prev = hh
            .get(&u8::from(note))
            .map(|p| {
                p.iter()
                    .filter(|v| !new_notes.contains(&v.note))
                    .copied()
                    .collect()
            })
            .unwrap_or_default();
        hh.insert(u8::from(note), voices_for_input);
        prev
    };
    for v in &orphaned {
        match v.target {
            VoiceOutputTarget::Synth => {
                let _ = synth_tx.send(SynthEvent::NoteOff { note: v.note });
            }
            VoiceOutputTarget::MidiPort { port } if port < num_outputs => {
                // Use the typed wmidi API like the realtime path. The
                // captured `v.channel` belongs to the original input
                // event, which may differ from this NoteOn's channel
                // (e.g., user retriggers the same MIDI number on a
                // different channel — the orphan release has to land
                // on the OLD channel where the voices attacked).
                let v_chan =
                    wmidi::Channel::from_index(v.channel & 0x0F).unwrap_or(wmidi::Channel::Ch1);
                let v_note = wmidi::Note::from_u8_lossy(v.note);
                let v_vel = wmidi::Velocity::from_u8_lossy(0);
                let msg = MidiMessage::NoteOff(v_chan, v_note, v_vel);
                let mut buf = vec![0u8; msg.bytes_size()];
                let _ = msg.copy_to_slice(&mut buf);
                let _ = output.send_to_port(port, &buf);
            }
            _ => {}
        }
    }
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
    _chord_name: &Arc<Mutex<String>>,
    routing_mode: contrapunk::harmony::RoutingMode,
    synth_tx: &mpsc::Sender<SynthEvent>,
    voice_outputs: &Arc<Mutex<Vec<VoiceOutputTarget>>>,
    held_harmonies: &Arc<Mutex<HashMap<u8, Vec<HeldVoice>>>>,
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

    // Fan releases into the built-in synth only for voices routed there.
    for (i, &n) in notes.iter().enumerate() {
        if matches!(target_for(i), VoiceOutputTarget::Synth) {
            let _ = synth_tx.send(SynthEvent::NoteOff { note: u8::from(n) });
        }
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

    // Drop this input's tracked harmony voices. The engine already
    // recomputed which notes should be released; our routing-aware
    // tracker just follows.
    held_harmonies
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .remove(&u8::from(note));

    // External MIDI note-offs, per-voice. Synth + Off skip MIDI.
    for (i, &n) in notes.iter().enumerate() {
        if let VoiceOutputTarget::MidiPort { port } = target_for(i) {
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
