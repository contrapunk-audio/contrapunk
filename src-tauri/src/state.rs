//! Managed Tauri state wrapping HarmonyEngine and related components.
//!
//! AppState is registered with Tauri's managed state system and accessed
//! via `State<AppState>` in command handlers.

use std::collections::HashSet;
use std::sync::atomic::{AtomicBool, AtomicI32};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};

use contrapunk::audio::guitar_input::GuitarInputConfig;
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

/// Beat-aligned chord trigger pattern config. Mirrors the frontend
/// pattern store. Pushed via `set_pattern_config` when the user edits.
/// Read by the router thread per loop iteration to decide whether to
/// fire harmony NoteOn on cell boundaries.
#[derive(Clone, Debug)]
pub struct PatternConfig {
    pub cells: Vec<bool>,
    pub subdivision: u8,
    pub length: u8,
    pub beats_per_bar: u8,
    pub input_mode: PatternInputMode,
}

impl Default for PatternConfig {
    fn default() -> Self {
        // 4 subdivision × 4 beats × 1 bar = 16 cells, all on.
        Self {
            cells: vec![true; 16],
            subdivision: 4,
            length: 1,
            beats_per_bar: 4,
            input_mode: PatternInputMode::Live,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum PatternInputMode {
    /// Input plays freely; harmony fires only on pattern-active beats.
    #[default]
    Live,
    /// Input + harmony snap to next pattern beat.
    Quantized,
    /// Input continuous; harmony NoteOff on pattern-off cells.
    Gated,
}

impl PatternConfig {
    pub fn cell_count(&self) -> usize {
        self.subdivision.max(1) as usize
            * self.beats_per_bar.max(1) as usize
            * self.length.max(1) as usize
    }

    pub fn cell_index_at(&self, total_beats: f64) -> usize {
        let beats_per_loop = (self.beats_per_bar.max(1) as f64) * (self.length.max(1) as f64);
        if beats_per_loop <= 0.0 {
            return 0;
        }
        let position_in_loop = ((total_beats % beats_per_loop) + beats_per_loop) % beats_per_loop;
        let idx = (position_in_loop * self.subdivision.max(1) as f64).floor() as usize;
        let total = self.cell_count();
        if total == 0 {
            return 0;
        }
        idx % total
    }
}

#[cfg(test)]
mod pattern_config_tests {
    use super::*;

    fn cfg(subdivision: u8, beats_per_bar: u8, length: u8) -> PatternConfig {
        let cells_count = (subdivision.max(1) as usize)
            * (beats_per_bar.max(1) as usize)
            * (length.max(1) as usize);
        PatternConfig {
            cells: vec![true; cells_count],
            subdivision,
            length,
            beats_per_bar,
            input_mode: PatternInputMode::Live,
        }
    }

    /// Reference table pinning `cell_index_at` outputs for known
    /// `(subdivision, beats_per_bar, length, total_beats) -> idx`
    /// tuples. The same table is mirrored in
    /// `ui/src/lib/stores/pattern.svelte.ts`'s `cellIndexAt` doc
    /// comment and dev-mode self-check — keep both in lockstep.
    /// If you change this table, change both.
    #[test]
    fn cell_index_at_matches_reference_table() {
        // (subdivision, beats_per_bar, length, total_beats, expected_idx)
        let cases: &[(u8, u8, u8, f64, usize)] = &[
            // 4×4×1 = 16 cells (16th-note bar in 4/4)
            (4, 4, 1, 0.0, 0),
            (4, 4, 1, 0.5, 2),
            (4, 4, 1, 1.0, 4),
            (4, 4, 1, 2.0, 8),
            (4, 4, 1, 3.75, 15),
            (4, 4, 1, 4.0, 0), // wraps to start
            (4, 4, 1, 4.25, 1),
            (4, 4, 1, -0.25, 15), // negative wraps backward
            // 1×4×1 = 4 cells (quarter-note bar)
            (1, 4, 1, 0.0, 0),
            (1, 4, 1, 1.0, 1),
            (1, 4, 1, 3.5, 3),
            (1, 4, 1, 4.0, 0),
            // 8×4×2 = 64 cells (32nd-note 2-bar loop)
            (8, 4, 2, 0.0, 0),
            (8, 4, 2, 7.5, 60),
            (8, 4, 2, 8.0, 0),
            // 2×3×1 = 6 cells (eighth-note 3/4 bar)
            (2, 3, 1, 0.0, 0),
            (2, 3, 1, 1.5, 3),
            (2, 3, 1, 3.0, 0),
        ];
        for &(s, bpb, l, tb, expected) in cases {
            let got = cfg(s, bpb, l).cell_index_at(tb);
            assert_eq!(
                got, expected,
                "subdivision={}, bpb={}, length={}, total_beats={}",
                s, bpb, l, tb
            );
        }
    }

    /// Reference table for `cell_count`. Same lockstep contract as
    /// `cell_index_at_matches_reference_table` above.
    #[test]
    fn cell_count_matches_reference_table() {
        let cases: &[(u8, u8, u8, usize)] = &[
            (4, 4, 1, 16),
            (1, 4, 1, 4),
            (8, 4, 2, 64),
            (2, 3, 1, 6),
            (4, 4, 4, 64),
        ];
        for &(s, bpb, l, expected) in cases {
            let got = cfg(s, bpb, l).cell_count();
            assert_eq!(got, expected, "s={}, bpb={}, l={}", s, bpb, l);
        }
    }

    #[test]
    fn cell_index_at_handles_pathological_inputs() {
        // Zero subdivision/length/bpb get clamped to 1 by max(1).
        // Pattern with 1 cell is degenerate but stable.
        let mut c = cfg(1, 1, 1);
        c.cells = vec![true];
        assert_eq!(c.cell_index_at(0.0), 0);
        assert_eq!(c.cell_index_at(100.0), 0);
        assert_eq!(c.cell_index_at(-100.0), 0);
    }
}

/// Per-voice output destination. Each engine-emitted voice (by index
/// 0..voice_count-1) can be routed independently.
///
/// Three explicit destinations only — no implicit "defer to global
/// routing_mode" fallback. Default is `Synth` so users get audio out
/// of the box without needing to add a MIDI port first.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum VoiceOutputTarget {
    /// Send to the internal synth only. Skip external MIDI for this voice.
    #[default]
    Synth,
    /// Send to a specific external MIDI port only. Skip the internal synth.
    MidiPort { port: usize },
    /// Skip both synth and external MIDI. Voice is silent.
    Off,
}

/// One harmony voice currently sounding, with the routing target it
/// was attacked through.
///
/// Tracked alongside `harmony_notes` (a flat HashSet of MIDI numbers
/// used for UI display + chord detection) so the beat-pattern can
/// dispatch NoteOn/NoteOff per-voice without re-running the harmony
/// engine. The captured `target` reflects the routing as of attack
/// time — later changes to `voice_outputs` apply to subsequent
/// attacks but do not retroactively reroute already-sounding voices.
#[derive(Clone, Copy, Debug)]
pub struct HeldVoice {
    pub note: u8,
    pub target: VoiceOutputTarget,
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

    /// MIDI note numbers currently held on input (melody)
    pub input_notes: Mutex<HashSet<u8>>,

    /// MIDI note numbers generated by harmony engine
    pub harmony_notes: Mutex<HashSet<u8>>,

    /// MIDI note numbers borrowed from another mode via interchange
    pub borrowed_notes: Mutex<HashSet<u8>>,

    /// Currently detected chord name
    pub chord_name: Mutex<String>,

    /// Guitar input DSP configuration (None = use defaults).
    ///
    /// Wrapped in `Arc<Mutex<...>>` so the `GuitarBridge` audio thread
    /// can hold a clone and re-read the config every block — without
    /// this, edits made via the debug window would only take effect on
    /// the next routing restart. Same pattern as `engine` (#80).
    pub guitar_config: Arc<Mutex<Option<GuitarInputConfig>>>,

    /// Guitar audio device name (empty = default input device)
    pub guitar_device: Mutex<String>,

    /// Guitar audio channel index (0-based, e.g. 0 = left, 1 = right)
    pub guitar_channel: Mutex<usize>,

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
    /// nowhere. Default all `UseDefault` preserves the legacy fan-out
    /// behavior for existing users until the UI sets explicit values.
    pub voice_outputs: Arc<Mutex<Vec<VoiceOutputTarget>>>,

    /// Master enable for the beat-aligned pattern feature. When false
    /// (default), the pattern panel + cells are inert and harmony
    /// dispatch follows today's real-time path. Toggled via
    /// `set_pattern_enabled`.
    pub pattern_enabled: Arc<AtomicBool>,

    /// Beat-aligned chord trigger pattern config. Pushed by the frontend
    /// `pattern` store via `set_pattern_config` whenever the user edits.
    /// Read by the router thread per loop iteration.
    pub pattern_config: Arc<Mutex<PatternConfig>>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            engine: Arc::new(Mutex::new(HarmonyEngine::new(
                Key::C,
                HarmonyMode::PassThrough,
            ))),
            preset_manager: Mutex::new(PresetManager::new()),
            is_running: AtomicBool::new(false),
            input_notes: Mutex::new(HashSet::new()),
            harmony_notes: Mutex::new(HashSet::new()),
            borrowed_notes: Mutex::new(HashSet::new()),
            chord_name: Mutex::new(String::new()),
            guitar_config: Arc::new(Mutex::new(None)),
            guitar_device: Mutex::new(String::new()),
            guitar_channel: Mutex::new(0),
            routing_mode: Mutex::new(RoutingMode::default()),
            stop_signal: Mutex::new(None),
            router_tx: Mutex::new(None),
            panic_pending: Arc::new(AtomicBool::new(false)),
            detune_cents: Arc::new(AtomicI32::new(0)),
            // Placeholder sample rate; audio_clock::start() corrects it
            // to the actual cpal device rate at app launch.
            transport: Transport::new(48_000),
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
            pattern_enabled: Arc::new(AtomicBool::new(false)),
            pattern_config: Arc::new(Mutex::new(PatternConfig::default())),
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
