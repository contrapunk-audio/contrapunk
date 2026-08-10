//! Managed Tauri state wrapping HarmonyEngine and related components.
//!
//! AppState is registered with Tauri's managed state system and accessed
//! via `State<AppState>` in command handlers.

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicI32};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};

use contrapunk::audio::guitar::GuitarCalibrationProfile;
use contrapunk::audio::guitar_input::{GuitarInput, GuitarInputConfig};
use contrapunk::chain::ChainCommander;
use contrapunk::elixir::{synth_event_channel, SynthEventReceiver, SynthEventSender, SynthParams};
use contrapunk::fx::{DelayParams, ReverbParams};
use contrapunk::harmony::{HarmonyEngine, HarmonyMode, Key, RoutingMode};
use contrapunk::preset::PresetManager;
use contrapunk::slide::{SlideConfig, SlideTelemetry};
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

/// Stable identity of one routable musical part. Live input and loop replay
/// share these identities so changing a destination affects both paths.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum VoiceRouteId {
    Input,
    Harmony { slot: u8 },
    Canon { voice: u8 },
    Counterpoint { voice: u8 },
    PatternLow,
    PatternCounter,
}

impl VoiceRouteId {
    pub fn parse(value: &str) -> Result<Self, String> {
        match value {
            "input" => Ok(Self::Input),
            "pattern_low" => Ok(Self::PatternLow),
            "pattern_counter" => Ok(Self::PatternCounter),
            _ => {
                let (kind, index) = value
                    .split_once(':')
                    .ok_or_else(|| format!("unknown voice route: {value}"))?;
                let index = index
                    .parse::<u8>()
                    .map_err(|_| format!("invalid voice route index: {value}"))?;
                if index as usize >= MAX_VOICES {
                    return Err(format!(
                        "voice route index {index} out of range (max {})",
                        MAX_VOICES - 1
                    ));
                }
                match kind {
                    "harmony" => Ok(Self::Harmony { slot: index }),
                    "canon" => Ok(Self::Canon { voice: index }),
                    "counterpoint" => Ok(Self::Counterpoint { voice: index }),
                    _ => Err(format!("unknown voice route: {value}")),
                }
            }
        }
    }

    pub fn key(self) -> String {
        match self {
            Self::Input => "input".into(),
            Self::Harmony { slot } => format!("harmony:{slot}"),
            Self::Canon { voice } => format!("canon:{voice}"),
            Self::Counterpoint { voice } => format!("counterpoint:{voice}"),
            Self::PatternLow => "pattern_low".into(),
            Self::PatternCounter => "pattern_counter".into(),
        }
    }
}

/// Central destination table. Missing entries intentionally mean Synth, so a
/// fresh install produces sound without pre-populating every possible route.
#[derive(Clone, Debug, Default)]
pub struct VoiceOutputRoutes {
    targets: HashMap<VoiceRouteId, VoiceOutputTarget>,
    all_to_synth: bool,
}

impl VoiceOutputRoutes {
    pub fn get(&self, route: VoiceRouteId) -> VoiceOutputTarget {
        if self.all_to_synth {
            VoiceOutputTarget::Synth
        } else {
            self.configured_target(route)
        }
    }

    fn configured_target(&self, route: VoiceRouteId) -> VoiceOutputTarget {
        self.targets.get(&route).copied().unwrap_or_default()
    }

    pub fn set(&mut self, route: VoiceRouteId, target: VoiceOutputTarget) -> bool {
        if self.configured_target(route) == target {
            return false;
        }
        if target == VoiceOutputTarget::Synth {
            self.targets.remove(&route);
        } else {
            self.targets.insert(route, target);
        }
        true
    }

    pub fn set_all_to_synth(&mut self, enabled: bool) -> bool {
        if self.all_to_synth == enabled {
            return false;
        }
        self.all_to_synth = enabled;
        true
    }

    pub fn assignments(&self) -> impl Iterator<Item = (VoiceRouteId, VoiceOutputTarget)> + '_ {
        self.targets.iter().map(|(&route, &target)| (route, target))
    }

    pub fn has_external_target(&self) -> bool {
        !self.all_to_synth
            && self
                .targets
                .values()
                .any(|target| matches!(target, VoiceOutputTarget::MidiPort { .. }))
    }
}

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

    /// Raised after a routing destination changes. The router drains sounding
    /// notes before using the new table, so a held note cannot be stranded on
    /// its old synth or MIDI port.
    pub route_change_pending: Arc<AtomicBool>,

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
    pub synth_tx: SynthEventSender,

    /// Receiver, stored in an Option so the audio_clock setup hook can
    /// `.take()` it and move it into the stream callback.
    pub synth_rx: Mutex<Option<SynthEventReceiver>>,

    /// Renderer-neutral Slide defaults and per-generated-voice overrides.
    /// Read only on note/control events, never from the audio callback.
    pub slide_config: Arc<Mutex<SlideConfig>>,
    pub slide_telemetry: Arc<SlideTelemetry>,
    pub midi_slide_telemetry: Arc<SlideTelemetry>,

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

    /// Destinations keyed by stable musical part rather than a single shared
    /// harmony index. Main harmony, Canon, Counterpoint, and pattern voices
    /// all resolve through this table; loop replay reuses the same routes.
    pub voice_outputs: Arc<Mutex<VoiceOutputRoutes>>,

    /// Live Companion orchestrator and its concrete arrangement lanes.
    /// The looper router constructs a second instance with independent
    /// WorldState/Phrase Context and copies configuration only.
    #[allow(dead_code)]
    pub companion: Arc<Mutex<crate::companion::Companion>>,

    /// One volatile MIDI loop. Router owns replay/cleanup; commands only
    /// mutate the pure state machine. Deliberately absent from rig/presets.
    pub looper: Arc<Mutex<contrapunk_companion::LooperLane>>,
}

pub(crate) fn new_arrangement_companion(
    transport: Arc<Transport>,
    engine: Arc<Mutex<HarmonyEngine>>,
) -> crate::companion::Companion {
    let world = crate::companion::WorldState::new(transport, engine);
    let mut companion = crate::companion::Companion::new(world);
    companion
        .lanes
        .push(Box::new(crate::companion::CanonLane::new()));
    companion
        .lanes
        .push(Box::new(crate::companion::CounterpointLane::new()));
    companion
        .lanes
        .push(Box::new(crate::companion::PatternLane::new(
            "Low Support",
            "pattern_low",
        )));
    companion
        .lanes
        .push(Box::new(crate::companion::PatternLane::new(
            "Counterline Pattern",
            "pattern_counter",
        )));
    companion
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
        let companion = Arc::new(Mutex::new(new_arrangement_companion(
            Arc::clone(&transport),
            Arc::clone(&engine),
        )));
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
            route_change_pending: Arc::new(AtomicBool::new(false)),
            detune_cents: Arc::new(AtomicI32::new(0)),
            // Placeholder sample rate; audio_clock::start() corrects it
            // to the actual cpal device rate at app launch.
            transport,
            metronome_enabled: Arc::new(AtomicBool::new(false)),
            synth_params: Arc::new(SynthParams::default()),
            synth_tx: {
                let (tx, _) = synth_event_channel();
                tx
            },
            synth_rx: Mutex::new(None),
            slide_config: Arc::new(Mutex::new(SlideConfig::default())),
            slide_telemetry: Arc::new(SlideTelemetry::new()),
            midi_slide_telemetry: Arc::new(SlideTelemetry::new()),
            reverb_params: Arc::new(ReverbParams::default()),
            delay_params: Arc::new(DelayParams::default()),
            chain_commander: Mutex::new(None),
            voice_outputs: Arc::new(Mutex::new(VoiceOutputRoutes::default())),
            companion,
            looper: Arc::new(Mutex::new(contrapunk_companion::LooperLane::new())),
        }
    }
}

impl AppState {
    /// Build AppState with a freshly-connected synth MIDI channel. Use
    /// this instead of `default()` so the audio-clock setup hook has a
    /// matching rx to `.take()`.
    pub fn new() -> Self {
        let mut base = Self::default();
        let (tx, rx) = synth_event_channel();
        base.synth_tx = tx;
        base.synth_rx = Mutex::new(Some(rx));
        base
    }
}
