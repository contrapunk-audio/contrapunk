//! Managed Tauri state wrapping HarmonyEngine and related components.
//!
//! AppState is registered with Tauri's managed state system and accessed
//! via `State<AppState>` in command handlers.

use std::sync::atomic::{AtomicBool, AtomicI32};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};

use contrapunk::audio::guitar::GuitarCalibrationProfile;
use contrapunk::audio::guitar_input::{GuitarInput, GuitarInputConfig};
use contrapunk::chain::ChainCommander;
use contrapunk::fx::{DelayParams, ReverbParams};
use contrapunk::harmony::{HarmonyEngine, HarmonyMode, Key, RoutingMode};
use contrapunk::preset::PresetManager;
use contrapunk::synth::{SynthEvent, SynthParams};
use contrapunk::transport::Transport;

/// Maximum number of voices the app exposes. Mirrors the 8 voice slots
/// in the output panel UI; `voice_outputs` is sized to this. The engine
/// itself accepts any voice_count up to this value.
pub const MAX_VOICES: usize = 8;

/// Per-voice output destination. Source of truth lives in the
/// shared `contrapunk-companion` crate so the WASM build can use
/// the same type. Re-exported here so the rest of src-tauri keeps
/// importing `crate::state::VoiceOutputTarget` unchanged.
pub use contrapunk_companion::voice_output::VoiceOutputTarget;

/// Application state managed by Tauri.
///
/// Wraps the core harmony engine, humanization config, preset manager,
/// and real-time note state in thread-safe containers for access from
/// both the main thread (command handlers) and the router thread.
pub struct AppState {
    /// Core harmony engine (key, mode, scale, voice leading, etc.).
    ///
    /// Wrapped in `Arc<Mutex<...>>` so the router thread (spawned by
    /// `start_routing`) and the Tauri command handlers operate on the
    /// same instance — without this sharing, mid-session parameter
    /// changes (set_key, set_auto_key, etc.) would not reach the live
    /// MIDI processing path.
    pub engine: Arc<Mutex<HarmonyEngine>>,

    /// Preset manager for built-in and custom presets
    pub preset_manager: Mutex<PresetManager>,

    /// Whether MIDI routing is currently active
    pub is_running: AtomicBool,

    /// Guitar input DSP configuration (None = use defaults).
    ///
    /// Wrapped in `Arc<Mutex<...>>` so the `GuitarBridge` worker
    /// can hold a clone and re-read the config between DSP blocks — without
    /// this, edits made via the debug window would only take effect on
    /// the next routing restart. Same pattern as `engine` (#80).
    pub guitar_config: Arc<Mutex<Option<GuitarInputConfig>>>,

    /// Guitar audio device name (empty = default input device)
    pub guitar_device: Mutex<String>,

    /// Guitar audio channel index (0-based, e.g. 0 = left, 1 = right)
    pub guitar_channel: Mutex<usize>,

    /// Per-string calibration profile loaded from `app_data_dir()`. The
    /// router thread reads this when constructing `GuitarBridge`; the
    /// pipeline applies it via `GuitarInput::set_calibration_profile`.
    /// Default is `GuitarCalibrationProfile::default()` (no samples).
    pub calibration_profile: Arc<Mutex<GuitarCalibrationProfile>>,

    /// Live handle to the running guitar pipeline, populated by
    /// `GuitarBridge::new` and cleared on stop_routing. Wrapped as
    /// `Arc<Mutex<Option<Arc<Mutex<GuitarInput>>>>>` because:
    ///   - the outer Arc<Mutex<Option<...>>> is the AppState slot the
    ///     bridge writes to and the calibration commands read from
    ///   - the inner Arc<Mutex<GuitarInput>> is worker-owned; the cpal data
    ///     callback never touches it. Cloning it lets command handlers lock and
    ///     hot-swap the calibration profile mid-session.
    /// Brutal-critic round 2 CRITICAL: previously, hot-reload via
    /// `load_calibration_profile` only updated AppState — the live
    /// audio pipeline kept its old normalizer until routing restart.
    /// The status badge claimed "calibrated" while the engine was not.
    /// The callback itself only writes to the bounded sample ring buffer.
    pub live_guitar_pipeline: Arc<Mutex<Option<Arc<Mutex<GuitarInput>>>>>,

    /// MIDI routing mode (channel-based MPE or port-based)
    pub routing_mode: Mutex<RoutingMode>,

    /// Stop signal for the router thread — set to true to stop the current routing
    pub stop_signal: Mutex<Option<Arc<AtomicBool>>>,

    /// Sender half of the router thread's MIDI input channel. Populated when
    /// routing starts so that `inject_note_on` / `inject_note_off` (virtual
    /// input from the UI) can push directly into the same pipeline physical
    /// MIDI + guitar audio already feed.
    pub router_tx: Mutex<Option<mpsc::Sender<Vec<u8>>>>,

    /// Flag raised by command handlers after any engine mutation that could
    /// leave notes stuck (key/mode/scale/voices change). The router thread
    /// clears it each loop by emitting MIDI All-Notes-Off on every channel
    /// across every port and clearing tracked note state.
    pub panic_pending: Arc<AtomicBool>,

    /// Global detune in cents. Read by the router thread each frame (lock-free).
    /// Updated by the `set_detune` command.
    pub detune_cents: Arc<AtomicI32>,

    /// Sample-accurate transport clock. Driven by the audio-output
    /// callback (see `audio_clock`). Shared across command threads and
    /// the audio thread via atomics.
    pub transport: Arc<Transport>,

    /// Metronome audible-click toggle. Read by the audio callback on
    /// every beat crossing; no click synthesis when false. Off by default
    /// so enabling the transport doesn't produce surprise audio.
    pub metronome_enabled: Arc<AtomicBool>,

    /// Built-in synth parameters (waveform, ADSR, filter, master gain).
    /// Read by the audio callback each buffer; mutated by Tauri command
    /// handlers in response to UI changes.
    pub synth_params: Arc<SynthParams>,

    /// Sender half of the synth's MIDI-event channel. The router thread
    /// pushes events here whenever harmony produces a note-on/off; the
    /// audio callback drains the receiver end on every buffer.
    pub synth_tx: mpsc::Sender<SynthEvent>,

    /// Receiver, stored in an Option so the audio_clock setup hook can
    /// `.take()` it and move it into the stream callback.
    pub synth_rx: Mutex<Option<mpsc::Receiver<SynthEvent>>>,

    /// Built-in reverb parameters. Read by the audio callback each
    /// buffer; mutated by Tauri command handlers in response to UI
    /// changes.
    pub reverb_params: Arc<ReverbParams>,

    /// Built-in delay parameters. Same atomic-mutation pattern as
    /// `reverb_params`.
    pub delay_params: Arc<DelayParams>,

    /// Main-thread handle for mutating the live audio chain at
    /// runtime (add/remove blocks). Populated by the audio-clock
    /// setup hook after the Chain is constructed.
    pub chain_commander: Mutex<Option<Arc<ChainCommander>>>,

    /// Per-voice output routing table, indexed 0..MAX_VOICES.
    /// Router thread reads this on every note to decide whether each
    /// voice goes to the internal synth, to a specific MIDI port, or
    /// nowhere. Default all `Synth` so users get audio out of the box
    /// without configuring a MIDI port first.
    pub voice_outputs: Arc<Mutex<Vec<VoiceOutputTarget>>>,

    /// Companion orchestrator (#91 wiring, commit A-minus). Owns the
    /// Sense/Mutate/Decide Lane registry. The router thread will
    /// clone this Arc and tick it once per loop iteration (next
    /// commit); future Tauri commands take brief locks to
    /// enable/disable / register Lanes.
    /// Constructed `enabled = false`, no Lanes registered — fully
    /// no-op until the router-thread wiring (commit A) and the
    /// input pipeline + UI controls (commits B/C) land.
    /// `#[allow(dead_code)]` until router wiring consumes it.
    #[allow(dead_code)]
    pub companion: Arc<Mutex<crate::companion::Companion>>,
}

impl Default for AppState {
    fn default() -> Self {
        // Construct engine + transport first so we can wire WorldState
        // and Companion against the same shared handles. Companion
        // construction MUST come after both — Lane logic reads from
        // WorldState which holds an Arc<Mutex<HarmonyEngine>> snapshot.
        let engine = Arc::new(Mutex::new(HarmonyEngine::new(
            Key::C,
            HarmonyMode::PassThrough,
        )));
        let transport = Transport::new(48_000);
        let world = crate::companion::WorldState::new(Arc::clone(&transport), Arc::clone(&engine));
        let companion = Arc::new(Mutex::new(crate::companion::Companion::new(world)));
        // Register concrete Lanes here. CanonLane is the first one
        // (issue #3) and ships disabled by default — the user opts in
        // via Tauri command. Future Lanes (LooperLane, BeatMachineLane,
        // etc.) get pushed alongside as they ship.
        {
            let mut c = companion.lock().expect("companion mutex poisoned at init");
            c.lanes.push(Box::new(crate::companion::CanonLane::new()));
            c.lanes
                .push(Box::new(crate::companion::CounterpointLane::new()));
            c.lanes.push(Box::new(crate::companion::PatternLane::new(
                "Low Support",
                "pattern_low",
            )));
            c.lanes.push(Box::new(crate::companion::PatternLane::new(
                "Counterline Pattern",
                "pattern_counter",
            )));
        }
        Self {
            engine,
            preset_manager: Mutex::new(PresetManager::new()),
            is_running: AtomicBool::new(false),
            guitar_config: Arc::new(Mutex::new(None)),
            guitar_device: Mutex::new(String::new()),
            guitar_channel: Mutex::new(0),
            calibration_profile: Arc::new(Mutex::new(GuitarCalibrationProfile::default())),
            live_guitar_pipeline: Arc::new(Mutex::new(None)),
            routing_mode: Mutex::new(RoutingMode::default()),
            stop_signal: Mutex::new(None),
            router_tx: Mutex::new(None),
            panic_pending: Arc::new(AtomicBool::new(false)),
            detune_cents: Arc::new(AtomicI32::new(0)),
            // Placeholder sample rate; audio_clock::start() corrects it
            // to the actual cpal device rate at app launch.
            transport,
            metronome_enabled: Arc::new(AtomicBool::new(false)),
            synth_params: Arc::new(SynthParams::default()),
            synth_tx: {
                let (tx, _) = mpsc::channel();
                tx
            },
            synth_rx: Mutex::new(None),
            reverb_params: Arc::new(ReverbParams::default()),
            delay_params: Arc::new(DelayParams::default()),
            chain_commander: Mutex::new(None),
            voice_outputs: Arc::new(Mutex::new(vec![VoiceOutputTarget::default(); MAX_VOICES])),
            companion,
        }
    }
}

impl AppState {
    /// Build AppState with a freshly-connected synth MIDI channel. Use
    /// this instead of `default()` so the audio-clock setup hook has a
    /// matching rx to `.take()`.
    pub fn new() -> Self {
        let mut base = Self::default();
        let (tx, rx) = mpsc::channel::<SynthEvent>();
        base.synth_tx = tx;
        base.synth_rx = Mutex::new(Some(rx));
        base
    }
}
