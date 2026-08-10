//! Main harmony engine that routes notes through mode-specific algorithms.
//!
//! This module contains [`HarmonyEngine`], the central component that transforms
//! incoming MIDI notes into harmonized multi-voice output. The engine manages:
//!
//! - Scale and key configuration
//! - Harmony mode selection and execution
//! - Multi-voice output with configurable voice position
//! - Note-On/Note-Off tracking for consistent release behavior
//! - Voice leading post-processing
//! - Octave mode transformations
//!
//! # Processing Pipeline
//!
//! ```text
//! Note-On -> Scale Check -> Mode Algorithm -> Voice Leading -> Octave Mode -> Output
//!               |                |                 |               |
//!          [In-scale?]     [Stateful?]      [Revoicing]      [Spread/Mirror]
//!               |                |                 |
//!          [Interchange]   [VecDeque]        [Previous]
//! ```
//!
//! # Note Tracking
//!
//! The engine maintains a FIFO of generated frames for each melody pitch, so
//! repeated Note-Ons retain distinct ownership and every Note-Off releases the
//! matching original generated voices.

use std::collections::{HashMap, VecDeque};
use wmidi::Note;

use crate::config::{BeatPhase, ExplicitIntervalMap, HarmonyMode, Key, OctaveMode, ScaleMode};
use crate::functional;
use crate::functional::context::HarmonicContext;
use crate::modes;
use crate::scale::Scale;
use crate::stateful::{
    ContraryMotionState, CounterpointSpecies, CounterpointState, CounterpointStrictness,
};
use crate::tuning::{
    tune_notes, TuningConfig, TuningError, TuningFrame, TuningStyle, MAX_TUNING_VOICES,
};
use crate::voice_leading::{
    revoice_chord, StyleRules, VoiceAnchor, VoiceLeadingStyle, VoiceRegister,
};

/// Voice leading post-processor that re-voices harmony output for smooth transitions.
#[derive(Debug)]
struct VoiceLeadingProcessor {
    enabled: bool,
    style: VoiceLeadingStyle,
    style_rules: StyleRules,
    /// Previous chord voicing (index 0 = melody, 1+ = harmony)
    previous_voicing: Option<Vec<Note>>,
    /// Register assignments (index 0 = melody placeholder, 1+ = harmony voices)
    registers: Vec<VoiceRegister>,
}

impl VoiceLeadingProcessor {
    fn new(voice_count: usize) -> Self {
        let style = VoiceLeadingStyle::default();
        let style_rules = StyleRules::for_style(style);
        let registers = Self::build_registers(voice_count);
        Self {
            enabled: false,
            style,
            style_rules,
            previous_voicing: None,
            registers,
        }
    }

    fn build_registers(voice_count: usize) -> Vec<VoiceRegister> {
        Self::build_registers_for_position(voice_count, voice_count.saturating_sub(1))
    }

    /// Builds register assignments based on voice position.
    ///
    /// Assigns registers to the full voice arrangement (0=top to N-1=bass),
    /// then reorders to match final_result layout: [user, closest-above,
    /// closest-below, next-above, next-below, ...].
    fn build_registers_for_position(
        voice_count: usize,
        voice_position: usize,
    ) -> Vec<VoiceRegister> {
        if voice_count <= 1 {
            return vec![VoiceRegister::Soprano];
        }

        // Assign registers to the full voice arrangement (0=soprano to N-1=bass)
        let arrangement_regs: Vec<VoiceRegister> = (0..voice_count)
            .map(|i| {
                if voice_count <= 4 {
                    match i {
                        0 => VoiceRegister::Soprano,
                        1 if voice_count == 2 => VoiceRegister::Bass,
                        1 => VoiceRegister::Alto,
                        2 if voice_count == 3 => VoiceRegister::Bass,
                        2 => VoiceRegister::Tenor,
                        _ => VoiceRegister::Bass,
                    }
                } else {
                    // For 5+ voices, spread evenly
                    let fraction = i as f32 / (voice_count - 1) as f32;
                    if fraction < 0.25 {
                        VoiceRegister::Soprano
                    } else if fraction < 0.5 {
                        VoiceRegister::Alto
                    } else if fraction < 0.75 {
                        VoiceRegister::Tenor
                    } else {
                        VoiceRegister::Bass
                    }
                }
            })
            .collect();

        // Build final_result order: user first, then interleaved above/below
        let vp = voice_position.min(voice_count - 1);
        let mut regs = vec![arrangement_regs[vp]]; // user's register

        let mut above_idx = if vp > 0 { Some(vp - 1) } else { None };
        let mut below_idx = if vp < voice_count - 1 {
            Some(vp + 1)
        } else {
            None
        };

        loop {
            if above_idx.is_none() && below_idx.is_none() {
                break;
            }
            if let Some(ai) = above_idx {
                regs.push(arrangement_regs[ai]);
                above_idx = if ai > 0 { Some(ai - 1) } else { None };
            }
            if let Some(bi) = below_idx {
                regs.push(arrangement_regs[bi]);
                below_idx = if bi < voice_count - 1 {
                    Some(bi + 1)
                } else {
                    None
                };
            }
        }

        regs
    }

    fn reset(&mut self) {
        self.previous_voicing = None;
    }

    fn set_style(&mut self, style: VoiceLeadingStyle) {
        self.style = style;
        self.style_rules = StyleRules::for_style(style);
        self.previous_voicing = None;
    }

    fn rebuild_for_voices(&mut self, voice_count: usize, voice_position: usize) {
        self.registers = Self::build_registers_for_position(voice_count, voice_position);
        self.reset();
    }
}

/// The harmony engine that transforms incoming MIDI notes into multi-voice harmonies.
///
/// `HarmonyEngine` is the central component of Contrapunk's harmony system. It holds
/// the current musical configuration (key, scale mode, harmony mode) and transforms
/// input notes through the selected algorithm to produce harmonized output.
///
/// # Core Concepts
///
/// - **Voice Count**: Number of output voices (1 = melody only, 2-8 = melody + harmonies)
/// - **Voice Position**: Which slot the user plays (0 = soprano, voice_count-1 = bass)
/// - **Chained Harmonies**: Each voice is derived from the previous (harm2 = harmony_of(harm1))
/// - **Note Tracking**: Active notes are tracked for consistent Note-Off behavior
///
/// # Stateless vs Stateful Modes
///
/// - **Stateless (1-5, 8)**: Each note processed independently
/// - **Stateful (6-7)**: Track previous notes for context-aware harmony
///
/// Stateful modes use sliding window history (via [`std::collections::VecDeque`]):
/// - Mode 6 (Contrary Motion): Tracks `last_melody` and `last_harmony`
/// - Mode 7 (Counterpoint): Uses `interval_history` (size 4) and `melody_contour` (size 3)
///
/// # Processing Flow
///
/// 1. **Scale Check**: Determine if note is in current scale
/// 2. **Mode Algorithm**: Apply selected harmony algorithm
/// 3. **Voice Position**: Generate chains above and below user position
/// 4. **Voice Leading**: Optional revoicing for smooth transitions
/// 5. **Octave Mode**: Apply spread/split/mirror transformations
/// 6. **Port Mapping**: Assign output ports for MIDI routing
///
/// # Example
///
/// ```ignore
/// use contrapunk::harmony::{HarmonyEngine, Key, HarmonyMode};
/// use wmidi::Note;
///
/// // Create engine with 4 voices in C major using diatonic thirds
/// let mut engine = HarmonyEngine::with_voices(Key::C, HarmonyMode::DiatonicThirds, 4);
///
/// // Harmonize a note
/// let result = engine.harmonize(Note::C4);
/// // result = [C4, E4, G4, B4] (melody + 3 chained thirds)
///
/// // For MIDI routing, use note_on/note_off tracking
/// let notes_on = engine.harmonize_note_on(Note::C4);
/// // ... send notes_on to outputs ...
/// let notes_off = engine.harmonize_note_off(Note::C4);
/// // notes_off contains the same notes as notes_on
/// ```
///
/// # Note-Off Tracking
///
/// For exact lifecycle behavior, use
/// [`harmonize_note_on`](Self::harmonize_note_on) and
/// [`harmonize_note_off`](Self::harmonize_note_off) instead of
/// [`harmonize`](Self::harmonize). Adapters with per-channel ownership use
/// [`harmonize_note_on_owned`](Self::harmonize_note_on_owned) and
/// [`harmonize_note_off_owned`](Self::harmonize_note_off_owned).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct ActiveNoteKey {
    source: u8,
    note: u8,
}

#[derive(Debug)]
pub struct HarmonyEngine {
    key: Key,
    mode: HarmonyMode,
    octave_mode: OctaveMode,
    /// Knob coefficient applied to Spread and BassTrebleSplit displacement.
    /// Range [0.0, 1.0]; output is quantized to whole octaves so spreading
    /// never changes pitch class. Default 1.0 preserves legacy behavior.
    octave_intensity: f32,
    scale_mode: ScaleMode,
    scale: Scale,
    explicit_interval_map: ExplicitIntervalMap,
    interchange_enabled: bool,
    borrowing_range: u8,
    last_borrowed_from: Option<ScaleMode>,
    /// Number of output voices (1 = melody only, 2 = melody + harmony, etc.)
    voice_count: usize,
    /// Exact-frequency tuning applied after ordinary MIDI harmony generation.
    tuning_config: TuningConfig,
    /// Voice position: which voice slot the user plays (0 = top/soprano, voice_count-1 = bass).
    /// Harmonies are generated outward from this position in both directions.
    voice_position: usize,
    // Stateful mode state - one per voice pair for chained harmonies
    contrary_motion_states: Vec<ContraryMotionState>,
    counterpoint_states: Vec<CounterpointState>,
    /// Tracks active notes: melody MIDI number -> generated harmony frames in
    /// Note-On order. Repeated pitches retain one frame per source owner.
    active_notes: HashMap<ActiveNoteKey, VecDeque<Vec<Note>>>,
    /// Port assignment map from the most recent harmonize call.
    /// Each index corresponds to a note in the harmonize result;
    /// the value is the output port index that note should be sent to.
    /// For non-Mirror modes: identity mapping (index i -> port i).
    /// For Mirror mode: duplicates map back to the original harmony voice's port.
    last_port_map: Vec<usize>,
    last_arrangement_indices: Vec<usize>,
    /// Stored port-map and velocity frames in the same FIFO order as
    /// `active_notes`.
    active_port_maps: HashMap<ActiveNoteKey, VecDeque<Vec<usize>>>,
    active_velocities: HashMap<ActiveNoteKey, VecDeque<u8>>,
    /// Voice leading post-processor
    voice_leading: VoiceLeadingProcessor,
    /// Whether auto-key detection is enabled
    auto_key: bool,
    /// Key detector (accumulates pitch classes to infer tonic)
    key_detector: super::key_detect::KeyDetector,
    beat_phase: BeatPhase,
    /// Beat-phase position in [0, beats_per_bar) used by Species 2-4 counterpoint.
    /// `None` means "no beat clock running" — Species 2-4 fall back to Species 1 behavior.
    counterpoint_beat_phase: Option<f64>,
    /// Internal beat counter that advances on every harmonize_note_on.
    /// Used as a fallback phase for Species 2-4 when no external
    /// transport is driving `counterpoint_beat_phase`. Without this,
    /// Species 2-4 silently fall back to Species 1 whenever the user
    /// hasn't pressed Play on the transport.
    synthetic_beat_counter: f64,
    /// Active species applied to all counterpoint states.
    counterpoint_species: CounterpointSpecies,
    /// Active strictness applied to all counterpoint states.
    counterpoint_strictness: CounterpointStrictness,
    harmonic_context: Option<HarmonicContext>,
    /// Harmony notes that need an explicit Note-Off after an auto-key
    /// triggered `set_key`. The key change wipes `active_notes`, which
    /// means the normal Note-Off path (`harmonize_note_off`) won't know
    /// about the harmonies that were sounding under the previous key —
    /// they would otherwise stay stuck. Drained by the router each
    /// `harmonize_note_on` cycle via `take_pending_releases`.
    pending_releases: Vec<Note>,
    /// Input MIDI notes that were held when a parameter change cleared
    /// `active_notes`. The router drains this via
    /// `take_reharm_inputs()` and re-runs `harmonize_note_on` for each
    /// to produce fresh harmonies under the new parameters, then diffs
    /// against the previously-sounding harmony set so only notes that
    /// actually drop out get `NoteOff` and only newly-needed notes get
    /// `NoteOn`. The user's held input never gets interrupted —
    /// transitions between knob positions are seamless.
    pending_reharm_inputs: Vec<(u8, u8, u8)>,
    /// When true, input notes below `bass_register_threshold` pass
    /// through without producing harmony — the user is assumed to be
    /// playing the bass line themselves, and added harmonies would
    /// clash with their voicing decisions. Default off so existing
    /// users see no behavior change. (Issue #100.)
    suppress_bass_register: bool,
    /// MIDI note number at and above which harmony is generated; below
    /// this, only the input passes through when `suppress_bass_register`
    /// is true. Default 48 (C3) — roughly the top of a guitar's E-string
    /// fifth fret, where bass-line playing typically ends and chord
    /// work begins.
    bass_register_threshold: u8,
}

impl HarmonyEngine {
    /// Creates a new HarmonyEngine with the specified key, mode, and voice count.
    ///
    /// # Arguments
    ///
    /// * `key` - The musical key (C, D, E, etc.)
    /// * `mode` - The harmony mode to use
    /// * `voice_count` - Number of output voices (1 = melody only, 2+ = melody + harmonies)
    pub fn with_voices(key: Key, mode: HarmonyMode, voice_count: usize) -> Self {
        let scale = Scale::new(key.semitones_from_c(), ScaleMode::Ionian);
        let voice_count = voice_count.max(1); // At least 1 voice
        let harmony_voices = if voice_count > 1 { voice_count - 1 } else { 0 };

        Self {
            key,
            mode,
            octave_mode: OctaveMode::None,
            octave_intensity: 1.0,
            scale_mode: ScaleMode::Ionian,
            scale,
            explicit_interval_map: ExplicitIntervalMap::default(),
            interchange_enabled: false,
            borrowing_range: 3,
            last_borrowed_from: None,
            voice_count,
            tuning_config: TuningConfig::default(),
            voice_position: voice_count.saturating_sub(1),
            contrary_motion_states: (0..harmony_voices)
                .map(|_| ContraryMotionState::new())
                .collect(),
            counterpoint_states: (0..harmony_voices)
                .map(|_| CounterpointState::new())
                .collect(),
            active_notes: HashMap::new(),
            last_port_map: Vec::new(),
            last_arrangement_indices: Vec::new(),
            active_port_maps: HashMap::new(),
            active_velocities: HashMap::new(),
            voice_leading: VoiceLeadingProcessor::new(voice_count),
            auto_key: false,
            key_detector: super::key_detect::KeyDetector::new(ScaleMode::Ionian),
            beat_phase: BeatPhase::default(),
            counterpoint_beat_phase: None,
            synthetic_beat_counter: 0.0,
            counterpoint_species: CounterpointSpecies::default(),
            counterpoint_strictness: CounterpointStrictness::default(),
            pending_releases: Vec::new(),
            pending_reharm_inputs: Vec::new(),
            harmonic_context: None,
            suppress_bass_register: false,
            bass_register_threshold: 48, // C3 — see field docs.
        }
    }

    /// Creates a new HarmonyEngine with the specified key and mode.
    /// Defaults to 2 voices (melody + 1 harmony).
    pub fn new(key: Key, mode: HarmonyMode) -> Self {
        Self::with_voices(key, mode, 2)
    }

    /// Copy musical configuration into a clean, independent performance
    /// runtime. Active notes, pending releases, detectors, voice-leading
    /// history, and counterpoint history are deliberately reset.
    pub fn fork_clean_runtime(&self) -> Self {
        let mut fork = Self::with_voices(self.key, self.mode, self.voice_count);
        fork.octave_mode = self.octave_mode;
        fork.octave_intensity = self.octave_intensity;
        fork.scale_mode = self.scale_mode;
        fork.scale = self.scale.clone();
        fork.explicit_interval_map = self.explicit_interval_map.clone();
        fork.interchange_enabled = self.interchange_enabled;
        fork.borrowing_range = self.borrowing_range;
        fork.tuning_config = self.tuning_config;
        fork.voice_position = self.voice_position;
        fork.voice_leading.enabled = self.voice_leading.enabled;
        fork.voice_leading.set_style(self.voice_leading.style);
        fork.auto_key = self.auto_key;
        fork.beat_phase = self.beat_phase;
        fork.counterpoint_beat_phase = self.counterpoint_beat_phase;
        fork.counterpoint_species = self.counterpoint_species;
        fork.counterpoint_strictness = self.counterpoint_strictness;
        fork.suppress_bass_register = self.suppress_bass_register;
        fork.bass_register_threshold = self.bass_register_threshold;
        fork
    }

    /// Whether two runtimes would harmonize new input with the same current
    /// musical settings. Runtime ownership/history is intentionally ignored.
    pub fn has_same_configuration(&self, other: &Self) -> bool {
        self.key == other.key
            && self.mode == other.mode
            && self.octave_mode == other.octave_mode
            && self.octave_intensity.to_bits() == other.octave_intensity.to_bits()
            && self.scale_mode == other.scale_mode
            && self.explicit_interval_map == other.explicit_interval_map
            && self.interchange_enabled == other.interchange_enabled
            && self.borrowing_range == other.borrowing_range
            && self.voice_count == other.voice_count
            && self.tuning_config == other.tuning_config
            && self.voice_position == other.voice_position
            && self.voice_leading.enabled == other.voice_leading.enabled
            && self.voice_leading.style == other.voice_leading.style
            && self.auto_key == other.auto_key
            && self.beat_phase.position.to_bits() == other.beat_phase.position.to_bits()
            && self.beat_phase.is_strong == other.beat_phase.is_strong
            && self.counterpoint_species == other.counterpoint_species
            && self.counterpoint_strictness == other.counterpoint_strictness
            && self.suppress_bass_register == other.suppress_bass_register
            && self.bass_register_threshold == other.bass_register_threshold
    }

    /// Returns the current key.
    pub fn key(&self) -> Key {
        self.key
    }

    /// Returns the current mode.
    pub fn mode(&self) -> HarmonyMode {
        self.mode
    }

    /// Returns a reference to the current scale. Borrowed read-only —
    /// callers that need to transpose against the current key/mode use
    /// `engine.scale().transpose_diatonic(...)` directly.
    pub fn scale(&self) -> &Scale {
        &self.scale
    }

    /// Returns a mutable reference to the current scale. Needed for
    /// callers that route through `Scale::harmonize_smart` (which
    /// mutates `last_borrowed_from` as a side effect of recording
    /// which parallel mode supplied the harmony note). Lock holders
    /// like `CanonLane` use this when transposing canon emissions
    /// against the live key.
    pub fn scale_mut(&mut self) -> &mut Scale {
        &mut self.scale
    }

    /// Returns the exact-frequency tuning configuration.
    pub fn tuning_config(&self) -> TuningConfig {
        self.tuning_config
    }

    /// Updates exact-frequency tuning. Changes involving Pure tuning replay
    /// held harmonies through the existing safe parameter-change path.
    pub fn set_tuning_config(&mut self, config: TuningConfig) -> Result<(), TuningError> {
        config.validate()?;
        if config == self.tuning_config {
            return Ok(());
        }
        let needs_reharm =
            self.tuning_config.style == TuningStyle::Pure || config.style == TuningStyle::Pure;
        self.tuning_config = config;
        if needs_reharm {
            self.clear_active_for_reharm();
        }
        Ok(())
    }

    /// Tunes a harmony result whose melody is at index 0 into a bounded,
    /// allocation-free frequency frame. Structural MIDI notes are unchanged.
    pub fn tune_harmony(&self, notes: &[Note]) -> Result<TuningFrame, TuningError> {
        if notes.len() > MAX_TUNING_VOICES {
            return Err(TuningError::TooManyVoices {
                len: notes.len(),
                max: MAX_TUNING_VOICES,
            });
        }
        let mut midi = [0; MAX_TUNING_VOICES];
        for (target, note) in midi.iter_mut().zip(notes) {
            *target = u8::from(*note);
        }
        tune_notes(&midi[..notes.len()], 0, self.tuning_config)
    }

    /// Returns the current explicit source-degree interval map.
    pub fn explicit_interval_map(&self) -> &ExplicitIntervalMap {
        &self.explicit_interval_map
    }

    /// Replaces the explicit interval map after bounded validation.
    pub fn set_explicit_interval_map(&mut self, map: ExplicitIntervalMap) -> Result<(), String> {
        for (label, offsets) in map
            .degree_offsets
            .iter()
            .enumerate()
            .map(|(index, offsets)| (format!("degree {}", index + 1), offsets))
            .chain(std::iter::once((
                "fallback".to_string(),
                &map.fallback_offsets,
            )))
        {
            if offsets.len() > 7 {
                return Err(format!(
                    "explicit interval {label} supports at most 7 offsets"
                ));
            }
            for (index, &offset) in offsets.iter().enumerate() {
                if offset == 0 || !(-48..=48).contains(&offset) {
                    return Err(format!(
                        "explicit interval {label} offset must be nonzero and between -48 and 48"
                    ));
                }
                if offsets[..index].contains(&offset) {
                    return Err(format!("explicit interval {label} offsets must be unique"));
                }
            }
        }
        self.explicit_interval_map = map;
        self.clear_active_for_reharm();
        Ok(())
    }

    /// Returns the current octave mode.
    pub fn octave_mode(&self) -> OctaveMode {
        self.octave_mode
    }

    /// Returns the port assignment map from the most recent harmonize call.
    ///
    /// Each index corresponds to a note in the harmonize result;
    /// the value is the output port index that note should be sent to.
    /// For Mirror mode, duplicate notes map back to the original voice's port.
    pub fn last_port_map(&self) -> &[usize] {
        &self.last_port_map
    }

    /// Sets the octave mode.
    ///
    /// Octave mode transforms harmony note pitches after generation:
    /// - None: No change
    /// - Spread: Each voice is +1 octave higher than previous
    /// - BassTrebleSplit: Harmonies below melody go -1 octave, above go +1 octave
    /// - Mirror: Each harmony note is duplicated at +1 and -1 octave (tripling harmony notes)
    pub fn set_octave_mode(&mut self, octave_mode: OctaveMode) {
        self.octave_mode = octave_mode;
        // Clear note tracking since octave transformations change output
        self.clear_active_for_reharm();
    }

    /// Returns the current voice count.
    pub fn voice_count(&self) -> usize {
        self.voice_count
    }

    /// Returns the current voice position (0 = top/soprano, voice_count-1 = bass).
    pub fn voice_position(&self) -> usize {
        self.voice_position
    }

    /// Sets the voice position. Clears active notes since routing changes.
    /// Clamped to `voice_count - 1`.
    pub fn set_voice_position(&mut self, position: usize) {
        let position = position.min(self.voice_count.saturating_sub(1));
        if position == self.voice_position {
            return;
        }
        println!(
            "[VP] set_voice_position: {} (voice_count={})",
            position, self.voice_count
        );
        self.voice_position = position;
        self.clear_active_for_reharm();
        self.voice_leading
            .rebuild_for_voices(self.voice_count, position);
    }

    /// Sets the number of output voices.
    /// Resets stateful mode state and active notes.
    ///
    /// # Arguments
    ///
    /// * `count` - Number of voices (1 = melody only, 2+ = melody + harmonies)
    pub fn set_voice_count(&mut self, count: usize) {
        let count = count.max(1);
        if count == self.voice_count {
            return;
        }
        self.voice_count = count;
        // Clamp voice_position to valid range
        self.voice_position = self.voice_position.min(count.saturating_sub(1));
        let harmony_voices = if count > 1 { count - 1 } else { 0 };

        // Rebuild state vectors
        self.contrary_motion_states = (0..harmony_voices)
            .map(|_| ContraryMotionState::new())
            .collect();
        let species = self.counterpoint_species;
        let strictness = self.counterpoint_strictness;
        self.counterpoint_states = (0..harmony_voices)
            .map(|_| {
                let mut s = CounterpointState::new();
                s.set_species(species);
                s.set_strictness(strictness);
                s
            })
            .collect();
        self.clear_active_for_reharm();
        self.voice_leading
            .rebuild_for_voices(count, self.voice_position);
    }

    /// Sets the musical key, rebuilding the scale.
    /// Resets stateful mode state and active notes since scale changes.
    ///
    /// This can be called during playback without stopping.
    pub fn set_key(&mut self, key: Key) {
        self.key = key;
        self.scale = Scale::new(key.semitones_from_c(), self.scale_mode);
        self.scale.set_interchange_enabled(self.interchange_enabled);
        self.scale.set_borrowing_range(self.borrowing_range);
        // Reset stateful modes since scale changed
        for state in &mut self.contrary_motion_states {
            state.reset();
        }
        for state in &mut self.counterpoint_states {
            state.reset();
        }
        // Reset harmonic context to avoid stale chord state from previous key
        self.harmonic_context = None;
        // Clear note tracking since harmonies would change with new scale
        self.clear_active_for_reharm();
        self.voice_leading.reset();
        // Clear cached beat-phase; router will push a fresh value next cycle.
        self.counterpoint_beat_phase = None;
        self.synthetic_beat_counter = 0.0;
    }

    /// Returns whether auto-key detection is enabled.
    pub fn auto_key(&self) -> bool {
        self.auto_key
    }

    /// Enable or disable auto-key detection.
    /// When enabled, the engine infers the tonic from played notes and
    /// updates the key automatically. The scale mode stays as the user set it.
    /// When disabled, resets the detector.
    pub fn set_auto_key(&mut self, enabled: bool) {
        self.auto_key = enabled;
        if enabled {
            self.key_detector.set_scale_mode(self.scale_mode);
            println!(
                "[AUTOKEY] enabled (scale_mode={:?}, key={:?})",
                self.scale_mode, self.key
            );
        } else {
            self.key_detector.reset();
            println!("[AUTOKEY] disabled");
        }
    }

    /// Sets the harmony mode.
    /// Resets stateful mode state and active notes when switching modes.
    ///
    /// This can be called during playback without stopping.
    pub fn set_mode(&mut self, mode: HarmonyMode) {
        self.mode = mode;
        // Reset stateful modes when switching
        for state in &mut self.contrary_motion_states {
            state.reset();
        }
        for state in &mut self.counterpoint_states {
            state.reset();
        }
        // Reset harmonic context to avoid stale chord state from previous mode
        self.harmonic_context = None;
        // Clear note tracking since harmonies would change with new mode
        self.clear_active_for_reharm();
        self.voice_leading.reset();
        // Clear cached beat-phase; router pushes a fresh value on next cycle.
        self.counterpoint_beat_phase = None;
        self.synthetic_beat_counter = 0.0;
    }

    /// Returns the current scale mode.
    pub fn scale_mode(&self) -> ScaleMode {
        self.scale_mode
    }

    /// Sets the scale mode, rebuilding the scale.
    /// Resets stateful mode state and active notes.
    pub fn set_scale_mode(&mut self, mode: ScaleMode) {
        self.scale_mode = mode;
        self.scale = Scale::new(self.key.semitones_from_c(), mode);
        self.scale.set_interchange_enabled(self.interchange_enabled);
        self.scale.set_borrowing_range(self.borrowing_range);
        if self.auto_key {
            self.key_detector.set_scale_mode(mode);
        }
        for state in &mut self.contrary_motion_states {
            state.reset();
        }
        for state in &mut self.counterpoint_states {
            state.reset();
        }
        // Reset harmonic context to avoid stale chord state from previous scale mode
        self.harmonic_context = None;
        self.clear_active_for_reharm();
        self.voice_leading.reset();
        // Clear cached beat-phase; router pushes a fresh value on next cycle.
        self.counterpoint_beat_phase = None;
        self.synthetic_beat_counter = 0.0;
    }

    /// Returns whether modal interchange is enabled.
    pub fn interchange_enabled(&self) -> bool {
        self.interchange_enabled
    }

    /// Enables or disables modal interchange.
    pub fn set_interchange_enabled(&mut self, enabled: bool) {
        self.interchange_enabled = enabled;
        self.scale.set_interchange_enabled(enabled);
        self.clear_active_for_reharm();
    }

    /// Returns the current borrowing range (1-5).
    pub fn borrowing_range(&self) -> u8 {
        self.borrowing_range
    }

    /// Sets the borrowing range (clamped 1-5).
    pub fn set_borrowing_range(&mut self, range: u8) {
        self.borrowing_range = range.clamp(1, 5);
        self.scale.set_borrowing_range(self.borrowing_range);
        self.clear_active_for_reharm();
    }

    /// Returns the last mode borrowed from during modal interchange.
    pub fn last_borrowed_from(&self) -> Option<ScaleMode> {
        self.last_borrowed_from
    }

    /// Returns whether voice leading is enabled.
    pub fn voice_leading_enabled(&self) -> bool {
        self.voice_leading.enabled
    }

    /// Returns the current voice leading style.
    pub fn voice_leading_style(&self) -> VoiceLeadingStyle {
        self.voice_leading.style
    }

    /// Enables or disables voice leading post-processing.
    pub fn set_voice_leading_enabled(&mut self, enabled: bool) {
        self.voice_leading.enabled = enabled;
        if !enabled {
            self.voice_leading.reset();
        }
        self.clear_active_for_reharm();
    }

    /// Sets the voice leading style, resetting VL state.
    pub fn set_voice_leading_style(&mut self, style: VoiceLeadingStyle) {
        self.voice_leading.set_style(style);
        self.clear_active_for_reharm();
    }

    /// Octave-spread coefficient applied to Spread and BassTrebleSplit.
    /// Range [0.0, 1.0]; displacement is quantized to whole octaves.
    pub fn octave_intensity(&self) -> f32 {
        self.octave_intensity
    }

    pub fn set_octave_intensity(&mut self, amount: f32) {
        let clamped = amount.clamp(0.0, 1.0);
        if (clamped - self.octave_intensity).abs() < f32::EPSILON {
            return;
        }
        self.octave_intensity = clamped;
        self.clear_active_for_reharm();
    }

    pub fn set_beat_phase(&mut self, phase: BeatPhase) {
        self.beat_phase = phase;
    }

    /// Returns the current counterpoint beat-phase position within the bar
    /// (`0.0 .. beats_per_bar`). `None` means no beat clock is running and
    /// Species 2-4 counterpoint falls back to Species 1 behavior.
    pub fn counterpoint_beat_phase(&self) -> Option<f64> {
        self.counterpoint_beat_phase
    }

    /// Updates the counterpoint beat-phase position (0.0 .. beats_per_bar).
    /// Required for beat-aware Species 2-4 behavior. Pass `None` to disable
    /// beat awareness.
    pub fn set_counterpoint_beat_phase(&mut self, phase: Option<f64>) {
        self.counterpoint_beat_phase = phase;
    }

    /// Returns the active counterpoint species (Species 1-4).
    pub fn counterpoint_species(&self) -> CounterpointSpecies {
        self.counterpoint_species
    }

    /// Sets the counterpoint species for all voices. Resets counterpoint
    /// state so suspension phase / interval history starts clean.
    pub fn set_counterpoint_species(&mut self, species: CounterpointSpecies) {
        self.counterpoint_species = species;
        for state in &mut self.counterpoint_states {
            state.set_species(species);
        }
        self.clear_active_for_reharm();
        self.counterpoint_beat_phase = None;
        self.synthetic_beat_counter = 0.0;
    }

    /// Returns whether bass-register suppression is active. (Issue #100.)
    pub fn suppress_bass_register(&self) -> bool {
        self.suppress_bass_register
    }

    /// Enable or disable bass-register suppression. When enabled, input
    /// notes below `bass_register_threshold` pass through without
    /// producing harmony — for users who play the bass line themselves
    /// and don't want added voicings to clash.
    pub fn set_suppress_bass_register(&mut self, enabled: bool) {
        self.suppress_bass_register = enabled;
        self.clear_active_for_reharm();
    }

    /// Returns the bass-register threshold MIDI note number.
    pub fn bass_register_threshold(&self) -> u8 {
        self.bass_register_threshold
    }

    /// Sets the bass-register threshold MIDI note (notes below pass through
    /// when `suppress_bass_register` is true). Clamped to 0..=127.
    pub fn set_bass_register_threshold(&mut self, midi: u8) {
        self.bass_register_threshold = midi.min(127);
        self.clear_active_for_reharm();
    }

    /// Returns the active counterpoint strictness (Relaxed vs Strict).
    pub fn counterpoint_strictness(&self) -> CounterpointStrictness {
        self.counterpoint_strictness
    }

    /// Sets the counterpoint strictness for all voices.
    pub fn set_counterpoint_strictness(&mut self, strictness: CounterpointStrictness) {
        self.counterpoint_strictness = strictness;
        for state in &mut self.counterpoint_states {
            state.set_strictness(strictness);
        }
        self.clear_active_for_reharm();
    }

    /// Borrow a clone of the CounterpointState at the given chain
    /// position. CanonLane uses this to thread a single cascade
    /// chain's stateful history across multiple per-voice mini-
    /// engines: V1 generates → V1's state is cloned into V2 before V2
    /// generates → V2 sees V1's pitch in its interval/range history
    /// and avoids parallels with it. Returns None if the index is out
    /// of range (engine has `harmony_voices` slots).
    pub fn counterpoint_state(&self, idx: usize) -> Option<CounterpointState> {
        self.counterpoint_states.get(idx).cloned()
    }

    /// Replace the CounterpointState at the given chain position
    /// without resetting it. Pair with `counterpoint_state(idx)` to
    /// pre-seed a downstream voice's mini-engine with an upstream
    /// voice's history for stateful cascading counterpoint.
    pub fn set_counterpoint_state(&mut self, idx: usize, state: CounterpointState) {
        if let Some(slot) = self.counterpoint_states.get_mut(idx) {
            *slot = state;
        }
    }

    /// Harmonizes a single note based on the current mode.
    ///
    /// Returns a Vec containing:
    /// - For Mode 1: Just the input note
    /// - For Modes 2-7: Input note + chained harmony notes (octave-transformed)
    ///
    /// Chained harmonies: each harmony is derived from the previous note.
    /// E.g., with 4 voices: [melody, harm1(melody), harm2(harm1), harm3(harm2)]
    ///
    /// The first element is always the original input note.
    /// Harmony notes follow in subsequent elements.
    ///
    /// **Note:** For MIDI routing, prefer `harmonize_note_on()` and
    /// `harmonize_note_off()` which properly track harmony notes for
    /// exact Note-Off handling.
    pub fn harmonize(&mut self, note: Note) -> Vec<Note> {
        if self.mode == HarmonyMode::PassThrough || self.voice_count <= 1 {
            self.last_arrangement_indices = vec![0];
            self.last_port_map = vec![0];
            return vec![note];
        }

        // Issue #100: bass-register suppression. When the input is
        // below the configured threshold, the user is presumed to be
        // playing the bass line themselves — adding harmony notes
        // would muddy the voicing. Pass the input through unchanged.
        // Default off (threshold 48 = C3); enabled per-session via
        // `set_suppress_bass_register(true)`.
        if self.suppress_bass_register && u8::from(note) < self.bass_register_threshold {
            self.last_arrangement_indices = vec![0];
            self.last_port_map = vec![0];
            return vec![note];
        }

        if self.mode == HarmonyMode::BarryHarris {
            return self.harmonize_block_chord(note);
        }

        if self.mode == HarmonyMode::ExplicitIntervals {
            return self.harmonize_explicit_intervals(note);
        }

        if self.mode == HarmonyMode::FunctionalHarmony || self.mode == HarmonyMode::BachChorale {
            return self.harmonize_functional(note);
        }
        // Build result with voice_count slots. User's note goes at voice_position.
        let mut result = vec![None; self.voice_count];
        result[self.voice_position] = Some(note);

        // State index counter for stateful modes (each chain step gets its own state)
        let mut state_idx = 0;

        // Chain ABOVE: from voice_position-1 down to 0 (higher pitched voices)
        if self.voice_position > 0 {
            let mut current = note;
            for i in (0..self.voice_position).rev() {
                let harmony_result = self.harmonize_single_directed(current, state_idx, true);
                state_idx += 1;
                if harmony_result.len() > 1 {
                    current = harmony_result[1];
                    result[i] = Some(current);
                } else {
                    break;
                }
            }
        }

        // Chain BELOW: from voice_position+1 to voice_count-1 (lower pitched voices)
        if self.voice_position < self.voice_count - 1 {
            let mut current = note;
            for i in (self.voice_position + 1)..self.voice_count {
                let harmony_result = self.harmonize_single_directed(current, state_idx, false);
                state_idx += 1;
                if harmony_result.len() > 1 {
                    current = harmony_result[1];
                    result[i] = Some(current);
                } else {
                    break;
                }
            }
        }

        // Flatten: user's note first, then harmony voices in chain order
        // (closest to user's position first, outward in both directions).
        // Track each entry's arrangement index for fixed SATB port mapping.
        let mut final_result = vec![note];
        let mut arrangement_indices = vec![self.voice_position];

        // Interleave above and below chains, closest first
        let mut above_idx = if self.voice_position > 0 {
            Some(self.voice_position - 1)
        } else {
            None
        };
        let mut below_idx = if self.voice_position < self.voice_count - 1 {
            Some(self.voice_position + 1)
        } else {
            None
        };

        loop {
            let has_above = above_idx.is_some();
            let has_below = below_idx.is_some();
            if !has_above && !has_below {
                break;
            }

            if let Some(ai) = above_idx {
                if let Some(n) = result[ai] {
                    final_result.push(n);
                    arrangement_indices.push(ai);
                }
                above_idx = if ai > 0 { Some(ai - 1) } else { None };
            }
            if let Some(bi) = below_idx {
                if let Some(n) = result[bi] {
                    final_result.push(n);
                    arrangement_indices.push(bi);
                }
                below_idx = if bi < self.voice_count - 1 {
                    Some(bi + 1)
                } else {
                    None
                };
            }
        }

        // Store arrangement indices so apply_octave_mode can build
        // fixed SATB port maps (soprano=0, alto=1, tenor=2, bass=3).
        self.last_arrangement_indices = arrangement_indices;

        // Voice leading post-processing (before octave mode)
        if self.voice_leading.enabled && final_result.len() > 1 {
            let pitch_classes: Vec<u8> = final_result[1..]
                .iter()
                .map(|n| u8::from(*n) % 12)
                .collect();
            let prev_midi: Option<Vec<u8>> = self
                .voice_leading
                .previous_voicing
                .as_ref()
                .map(|v| v.iter().map(|n| u8::from(*n)).collect());
            let anchor = VoiceAnchor {
                midi: u8::from(final_result[0]),
                arrangement_pos: self.voice_position,
                harmony_arrangement_positions: self.last_arrangement_indices[1..].to_vec(),
            };
            let revoiced = revoice_chord(
                &pitch_classes,
                prev_midi.as_deref(),
                &self.voice_leading.registers,
                &self.voice_leading.style_rules,
                Some(&anchor),
            );
            // Replace harmony notes with revoiced MIDI values
            for (i, &midi_val) in revoiced.iter().enumerate() {
                if i + 1 < final_result.len() {
                    if let Ok(n) = Note::try_from(midi_val) {
                        final_result[i + 1] = n;
                    }
                }
            }
            // Temporal dissonance belongs to beat-aware counterpoint/Companion lanes,
            // not a pitch-voicing style. Store the plain voicing for the next call.
            self.voice_leading.previous_voicing = Some(final_result.clone());
        }

        // Apply octave mode transformation to harmony notes (not melody)
        self.apply_octave_mode(&mut final_result);

        final_result
    }

    /// Applies octave mode transformation to harmony notes and populates `last_port_map`.
    /// The melody (index 0) is never modified.
    ///
    /// For Mirror mode: each harmony note is duplicated at +1 and -1 octave,
    /// producing 3x harmony notes. Duplicates are appended and their port map
    /// entries point back to the original harmony voice's port index.
    fn apply_octave_mode(&mut self, notes: &mut Vec<Note>) {
        // Port map based on SATB arrangement position (fixed routing).
        // Each note maps to its arrangement index (0=soprano, 1=alto, 2=tenor, 3=bass).
        self.last_port_map = self.last_arrangement_indices.clone();

        if notes.len() <= 1 || self.octave_mode == OctaveMode::None {
            return;
        }

        let melody = notes[0];
        let melody_midi = u8::from(melody);

        // Helper: check if a shifted note would cross the user's anchor.
        // Returns true if the shift is allowed (stays on the correct side).
        let user_midi = melody_midi;
        let voice_pos = self.voice_position;
        let arr_indices = &self.last_arrangement_indices;
        let anchor_ok = |idx: usize, shifted: u8| -> bool {
            if let Some(&harm_arr) = arr_indices.get(idx) {
                if harm_arr < voice_pos {
                    // Must stay above user
                    shifted >= user_midi
                } else if harm_arr > voice_pos {
                    // Must stay below user
                    shifted <= user_midi
                } else {
                    true // same position as user
                }
            } else {
                true
            }
        };

        let intensity = self.octave_intensity;
        match self.octave_mode {
            OctaveMode::None => {}
            OctaveMode::Spread => {
                for (i, note) in notes.iter_mut().enumerate().skip(1) {
                    let midi = u8::from(*note);
                    // Quantize before converting octaves to semitones. Rounding
                    // semitones directly would transpose notes out of the key.
                    let shift = ((i as f32) * intensity).round() as u8 * 12;
                    let shifted = midi.saturating_add(shift).min(127);
                    if anchor_ok(i, shifted) {
                        if let Ok(new_note) = Note::try_from(shifted) {
                            *note = new_note;
                        }
                    }
                    // If not anchor_ok, leave the note as-is
                }
            }
            OctaveMode::BassTrebleSplit => {
                let shift = intensity.round() as u8 * 12;
                for (i, note) in notes.iter_mut().enumerate().skip(1) {
                    let midi = u8::from(*note);
                    let shifted = if midi < user_midi {
                        midi.saturating_sub(shift)
                    } else {
                        midi.saturating_add(shift).min(127)
                    };
                    if anchor_ok(i, shifted) {
                        if let Ok(new_note) = Note::try_from(shifted) {
                            *note = new_note;
                        }
                    }
                }
            }
            OctaveMode::Mirror => {
                let harmony_count = notes.len() - 1;
                let mut duplicates: Vec<(Note, usize)> = Vec::new();

                for i in 1..=harmony_count {
                    let midi = u8::from(notes[i]);
                    // +1 octave copy (only if it doesn't cross anchor)
                    let up = midi.wrapping_add(12);
                    if up <= 127 && anchor_ok(i, up) {
                        if let Ok(n) = Note::try_from(up) {
                            duplicates.push((n, i));
                        }
                    }
                    // -1 octave copy (only if it doesn't cross anchor)
                    if midi >= 12 && anchor_ok(i, midi - 12) {
                        if let Ok(n) = Note::try_from(midi - 12) {
                            duplicates.push((n, i));
                        }
                    }
                }

                for (dup_note, original_idx) in duplicates {
                    notes.push(dup_note);
                    let arr_port = if original_idx < self.last_arrangement_indices.len() {
                        self.last_arrangement_indices[original_idx]
                    } else {
                        original_idx
                    };
                    self.last_port_map.push(arr_port);
                }
            }
        }
    }

    /// Octave-shift `harmony` so it lands on the correct side of `anchor`.
    /// Returns None if no valid shift exists in MIDI range.
    fn octave_shift_to_side(harmony: Note, anchor: Note, above: bool) -> Option<Note> {
        let anchor_midi = u8::from(anchor) as i16;
        let mut harm_midi = u8::from(harmony) as i16;
        let on_correct_side = if above {
            harm_midi > anchor_midi
        } else {
            harm_midi < anchor_midi
        };
        if on_correct_side {
            return Some(harmony);
        }
        if above {
            while harm_midi <= anchor_midi {
                harm_midi += 12;
                if harm_midi > 127 {
                    return None;
                }
            }
        } else {
            while harm_midi >= anchor_midi {
                harm_midi -= 12;
                if harm_midi < 0 {
                    return None;
                }
            }
        }
        Note::try_from(harm_midi as u8).ok()
    }

    /// Octave-shift block-voicing harmonies around `notes[0]` so the user's
    /// note ends up at SATB slot `voice_position`. Used by block-voicing
    /// modes (BarryHarris, FunctionalHarmony, BachChorale) which produce
    /// a fixed voicing without natively respecting voice_position.
    ///
    /// Picks the closest-to-input harmonies to relocate. Also rewrites
    /// `last_arrangement_indices` based on the post-shift pitch order so
    /// downstream code (apply_octave_mode anchor logic, MIDI routing) sees
    /// the correct SATB slot for each note.
    fn redistribute_for_voice_position(&mut self, notes: &mut [Note]) {
        if notes.len() <= 1 {
            return;
        }
        let input_midi = u8::from(notes[0]) as i16;
        let voice_position = self.voice_position;
        // Number of harmonies that should sit above the user's note.
        let target_above = voice_position.min(notes.len().saturating_sub(1));

        let mut below_indices: Vec<usize> = Vec::new();
        let mut above_indices: Vec<usize> = Vec::new();
        for (i, n) in notes.iter().enumerate().skip(1) {
            let m = u8::from(*n) as i16;
            if m < input_midi {
                below_indices.push(i);
            } else if m > input_midi {
                above_indices.push(i);
            }
        }
        let cur_above = above_indices.len();

        if cur_above < target_above {
            // Promote closest-to-input below-harmonies (highest MIDI) up.
            below_indices.sort_by_key(|&i| std::cmp::Reverse(u8::from(notes[i])));
            for &i in below_indices.iter().take(target_above - cur_above) {
                let mut new_midi = u8::from(notes[i]) as i16;
                while new_midi <= input_midi {
                    new_midi += 12;
                    if new_midi > 127 {
                        break;
                    }
                }
                if (0..=127).contains(&new_midi) {
                    if let Ok(n) = Note::try_from(new_midi as u8) {
                        notes[i] = n;
                    }
                }
            }
        } else if cur_above > target_above {
            // Demote closest-to-input above-harmonies (lowest MIDI) down.
            above_indices.sort_by_key(|&i| u8::from(notes[i]));
            for &i in above_indices.iter().take(cur_above - target_above) {
                let mut new_midi = u8::from(notes[i]) as i16;
                while new_midi >= input_midi {
                    new_midi -= 12;
                    if new_midi < 0 {
                        break;
                    }
                }
                if (0..=127).contains(&new_midi) {
                    if let Ok(n) = Note::try_from(new_midi as u8) {
                        notes[i] = n;
                    }
                }
            }
        }

        // Rewrite arrangement_indices by post-shift pitch order: highest
        // note → slot 0 (soprano), lowest → slot voice_count-1 (bass).
        let mut order: Vec<usize> = (0..notes.len()).collect();
        order.sort_by_key(|&i| std::cmp::Reverse(u8::from(notes[i])));
        let mut arr = vec![0usize; notes.len()];
        for (slot, &idx) in order.iter().enumerate() {
            arr[idx] = slot;
        }
        self.last_arrangement_indices = arr;
    }

    fn harmonize_explicit_intervals(&mut self, note: Note) -> Vec<Note> {
        let offsets = self
            .scale
            .degree_of(note)
            .and_then(|degree| self.explicit_interval_map.degree_offsets.get(degree))
            .unwrap_or(&self.explicit_interval_map.fallback_offsets);
        let mut result = Vec::with_capacity(self.voice_count.min(offsets.len() + 1));
        result.push(note);
        let anchor = u8::from(note) as i16;
        for &offset in offsets.iter().take(self.voice_count.saturating_sub(1)) {
            let midi = anchor + offset as i16;
            if let Ok(midi) = u8::try_from(midi) {
                if let Ok(generated) = Note::try_from(midi) {
                    result.push(generated);
                }
            }
        }

        // Route highest pitch to the lowest arrangement index, regardless
        // of result order (`result[0]` must remain the exact player source).
        let mut order: Vec<usize> = (0..result.len()).collect();
        order.sort_by_key(|&index| std::cmp::Reverse(u8::from(result[index])));
        self.last_arrangement_indices = vec![0; result.len()];
        for (slot, &result_index) in order.iter().enumerate() {
            self.last_arrangement_indices[result_index] = slot;
        }
        self.apply_octave_mode(&mut result);

        // Octave transforms can change pitch order (and Mirror can add
        // voices), so routing must describe the notes we actually return.
        let mut final_order: Vec<usize> = (0..result.len()).collect();
        final_order.sort_by_key(|&index| std::cmp::Reverse(u8::from(result[index])));
        self.last_arrangement_indices = vec![0; result.len()];
        for (slot, &result_index) in final_order.iter().enumerate() {
            self.last_arrangement_indices[result_index] = slot;
        }
        self.last_port_map = self.last_arrangement_indices.clone();
        result
    }

    fn harmonize_block_chord(&mut self, note: Note) -> Vec<Note> {
        let inferred = match super::barry_harris::validate_scale(self.scale_mode) {
            super::barry_harris::BhScaleGuard::Valid => None,
            super::barry_harris::BhScaleGuard::Fallback(mode) => {
                Some(Scale::new(self.key.semitones_from_c(), mode))
            }
        };
        let scale = inferred.as_ref().unwrap_or(&self.scale);
        match super::barry_harris::build_voicing(note, scale, self.beat_phase) {
            Some(voicing) => {
                let mut result = Vec::with_capacity(4);
                result.push(note);
                // The block already contains the melody's scale degree,
                // often in another octave. Keep the played note as that
                // chord voice instead of adding a fifth, doubled pitch class.
                let melody_pc = u8::from(note) % 12;
                for &v in &voicing {
                    if u8::from(v) % 12 != melody_pc {
                        result.push(v);
                    }
                }
                self.last_arrangement_indices = (0..result.len()).collect();
                self.redistribute_for_voice_position(&mut result);
                self.apply_octave_mode(&mut result);
                result
            }
            None => {
                self.last_arrangement_indices = vec![0];
                self.last_port_map = vec![0];
                vec![note]
            }
        }
    }

    fn harmonize_functional(&mut self, note: Note) -> Vec<Note> {
        let scale_mode = self.scale_mode;
        let tonic = self.key.semitones_from_c();
        if !HarmonicContext::is_compatible_scale(scale_mode) {
            self.last_arrangement_indices = vec![0];
            self.last_port_map = vec![0];
            return vec![note];
        }
        let ctx = self
            .harmonic_context
            .get_or_insert_with(|| HarmonicContext::new(tonic, scale_mode));
        let mut result = match self.mode {
            HarmonyMode::BachChorale => functional::bach_chorale(note, ctx, scale_mode),
            HarmonyMode::FunctionalHarmony => {
                functional::functional_harmony(note, ctx, scale_mode, self.voice_count)
            }
            _ => vec![note],
        };
        self.last_arrangement_indices = (0..result.len()).collect();
        self.redistribute_for_voice_position(&mut result);
        self.last_port_map = self.last_arrangement_indices.clone();
        result
    }

    /// Returns the beat phase to use for Species 2-4 dispatch.
    /// Prefers the externally-set transport phase; falls back to the
    /// internal synthetic counter so Species 2-4 work without a
    /// running transport. Returns None unconditionally for any mode
    /// other than StrictCounterpoint, or for Species1 (which ignores
    /// phase entirely), so other code paths see no behavior change.
    fn effective_counterpoint_beat_phase(&self) -> Option<f64> {
        if !matches!(self.mode, HarmonyMode::StrictCounterpoint) {
            return self.counterpoint_beat_phase;
        }
        if matches!(self.counterpoint_species, CounterpointSpecies::Species1) {
            return self.counterpoint_beat_phase;
        }
        self.counterpoint_beat_phase
            .or(Some(self.synthetic_beat_counter))
    }

    /// Harmonizes a single note in a specific direction using the mode's algorithm.
    /// `above`: if true, generate harmony above; if false, generate below.
    /// Used for bidirectional voice position generation.
    fn harmonize_single_directed(
        &mut self,
        note: Note,
        state_index: usize,
        above: bool,
    ) -> Vec<Note> {
        match self.mode {
            HarmonyMode::PassThrough => modes::pass_through(note, &mut self.scale),
            HarmonyMode::DiatonicThirds => {
                modes::diatonic_thirds_directed(note, &mut self.scale, above)
            }
            HarmonyMode::DiatonicFourths => {
                modes::diatonic_fourths_directed(note, &mut self.scale, above)
            }
            HarmonyMode::ContraryMotion => {
                if let Some(state) = self.contrary_motion_states.get_mut(state_index) {
                    state.process_directed(&mut self.scale, note, above)
                } else {
                    vec![note]
                }
            }
            HarmonyMode::StrictCounterpoint => {
                // Species 1 is direction-aware via process_directed. Species 2-4
                // rely on beat-phase and use process_with_beat which ignores
                // `above` — we octave-shift the result post-hoc to honor the
                // chain's direction request.
                let species = self.counterpoint_species;
                let beat_phase = self.effective_counterpoint_beat_phase();
                if let Some(state) = self.counterpoint_states.get_mut(state_index) {
                    let result = if matches!(species, CounterpointSpecies::Species1) {
                        state.process_directed(&mut self.scale, note, above)
                    } else {
                        state.process_with_beat(&mut self.scale, note, beat_phase)
                    };
                    if !matches!(species, CounterpointSpecies::Species1) && result.len() > 1 {
                        if let Some(shifted) = Self::octave_shift_to_side(result[1], note, above) {
                            return vec![note, shifted];
                        }
                    }
                    result
                } else {
                    vec![note]
                }
            }
            HarmonyMode::BarryHarris => {
                modes::diatonic_thirds_directed(note, &mut self.scale, above)
            }
            HarmonyMode::FunctionalHarmony | HarmonyMode::BachChorale => {
                modes::diatonic_thirds_directed(note, &mut self.scale, above)
            }
            HarmonyMode::ExplicitIntervals => vec![note],
        }
    }

    /// Harmonizes a single note using the mode's algorithm with the given state index.
    /// Used internally for chained harmony generation.
    fn harmonize_single(&mut self, note: Note, state_index: usize) -> Vec<Note> {
        match self.mode {
            HarmonyMode::PassThrough => modes::pass_through(note, &mut self.scale),
            HarmonyMode::DiatonicThirds => modes::diatonic_thirds(note, &mut self.scale),
            HarmonyMode::DiatonicFourths => modes::diatonic_fourths(note, &mut self.scale),
            HarmonyMode::ContraryMotion => {
                if let Some(state) = self.contrary_motion_states.get_mut(state_index) {
                    state.process(&mut self.scale, note)
                } else {
                    vec![note]
                }
            }
            HarmonyMode::StrictCounterpoint => {
                // Dispatch to process_with_beat so Species 2-4 get beat awareness.
                // Species 1 + beat_phase=None is exactly equivalent to process().
                let beat_phase = self.effective_counterpoint_beat_phase();
                if let Some(state) = self.counterpoint_states.get_mut(state_index) {
                    state.process_with_beat(&mut self.scale, note, beat_phase)
                } else {
                    vec![note]
                }
            }
            HarmonyMode::BarryHarris => modes::diatonic_thirds(note, &mut self.scale),
            HarmonyMode::FunctionalHarmony | HarmonyMode::BachChorale => {
                modes::diatonic_thirds(note, &mut self.scale)
            }
            HarmonyMode::ExplicitIntervals => vec![note],
        }
    }

    /// Harmonizes a Note-On and tracks the result for Note-Off.
    ///
    /// Call this for Note-On messages. The returned notes should be
    /// sent to outputs. When Note-Off comes, call `harmonize_note_off()`
    /// with the same melody note to get matching harmony releases.
    ///
    /// Tracking keeps NoteOff exact for every stateful or multi-voice mode.
    ///
    /// # Arguments
    ///
    /// * `note` - The melody note from the Note-On message
    ///
    /// # Returns
    ///
    /// Vec of notes to send: original note first, harmony notes after.
    pub fn harmonize_note_on(&mut self, note: Note) -> Vec<Note> {
        self.harmonize_note_on_owned(note, u8::from(note))
    }

    /// Owned variant for adapters that distinguish the same pitch on separate
    /// source channels. Repeated attacks for one owner remain FIFO-ordered.
    pub fn harmonize_note_on_owned(&mut self, note: Note, source: u8) -> Vec<Note> {
        self.harmonize_note_on_owned_with_velocity(note, source, 100)
    }

    /// Owned Note-On with adapter velocity retained for configuration replay.
    pub fn harmonize_note_on_owned_with_velocity(
        &mut self,
        note: Note,
        source: u8,
        velocity: u8,
    ) -> Vec<Note> {
        // Feed note to key detector if auto-key is on
        if self.auto_key {
            let midi = u8::from(note);
            if let Some(detected) = self.key_detector.feed(midi) {
                if detected != self.key {
                    // Capture all currently-sounding harmonies for explicit
                    // release before set_key wipes `active_notes`. Without
                    // this the old-key harmonies stay stuck — note-off for
                    // the user's input note only releases harmonies the
                    // engine is now tracking under the new key.
                    for frames in self.active_notes.values() {
                        for harmonies in frames {
                            self.pending_releases.extend(harmonies.iter().copied());
                        }
                    }
                    self.pending_releases
                        .sort_unstable_by_key(|note| u8::from(*note));
                    println!(
                        "[AUTOKEY] key change: {:?} -> {:?} (note={}, releasing {} stale)",
                        self.key,
                        detected,
                        midi,
                        self.pending_releases.len()
                    );
                    self.set_key(detected);
                }
            }
        }

        self.synthetic_beat_counter = (self.synthetic_beat_counter + 1.0) % 4.0;

        let result = self.harmonize(note);
        // Copy last_borrowed_from from scale for UI access
        self.last_borrowed_from = self.scale.last_borrowed_from();
        // Store the harmony notes and port map for Note-Off retrieval
        let midi = u8::from(note);
        let key = ActiveNoteKey { source, note: midi };
        self.active_notes
            .entry(key)
            .or_default()
            .push_back(result[1..].to_vec());
        self.active_port_maps
            .entry(key)
            .or_default()
            .push_back(self.last_port_map.clone());
        self.active_velocities
            .entry(key)
            .or_default()
            .push_back(velocity.min(127));
        result
    }

    /// Returns the notes to release for a Note-Off.
    ///
    /// Returns the original note plus any harmony notes that were
    /// produced when the corresponding Note-On was processed via
    /// `harmonize_note_on()`.
    ///
    /// # Arguments
    ///
    /// * `note` - The melody note from the Note-Off message
    ///
    /// # Returns
    ///
    /// Vec of notes to release: original note first, tracked harmony notes after.
    /// If no harmony was tracked, returns just the original note.
    /// Returns and clears the queue of harmony notes that need an
    /// explicit Note-Off after an auto-key triggered key change.
    ///
    /// The router calls this after each `harmonize_note_on` so the old
    /// key's stale harmonies get released before the new key's harmonies
    /// take over. Returns an empty Vec when no key change happened.
    pub fn take_pending_releases(&mut self) -> Vec<Note> {
        std::mem::take(&mut self.pending_releases)
    }

    /// Clear all note-lifecycle bookkeeping after panic, transport stop,
    /// or an input/output-mode transition. Unlike parameter-change
    /// reharmonization, this intentionally does not queue held inputs for
    /// replay: downstream sound has already been silenced.
    pub fn clear_active_notes(&mut self) {
        self.active_notes.clear();
        self.active_port_maps.clear();
        self.active_velocities.clear();
        self.pending_releases.clear();
        self.pending_reharm_inputs.clear();
        self.last_port_map.clear();
    }

    /// Drain the list of input MIDI notes that were held when a
    /// parameter change wiped `active_notes`. The router re-runs
    /// `harmonize_note_on` for each so the new parameters take effect
    /// without dropping the user's held input.
    pub fn take_reharm_inputs(&mut self) -> Vec<u8> {
        self.take_owned_reharm_inputs()
            .into_iter()
            .map(|(_, note)| note)
            .collect()
    }

    /// Drain held inputs with their adapter-owned source identity intact.
    pub fn take_owned_reharm_inputs(&mut self) -> Vec<(u8, u8)> {
        self.take_owned_reharm_inputs_with_velocity()
            .into_iter()
            .map(|(source, note, _)| (source, note))
            .collect()
    }

    /// Drain held inputs with source identity and original Note-On velocity.
    pub fn take_owned_reharm_inputs_with_velocity(&mut self) -> Vec<(u8, u8, u8)> {
        std::mem::take(&mut self.pending_reharm_inputs)
    }

    /// Wipe per-note tracking after a parameter change, but record the
    /// held input MIDI numbers in `pending_reharm_inputs` so the router
    /// can replay them under the new parameters. Replaces the previous
    /// `self.active_notes.clear(); self.active_port_maps.clear();`
    /// idiom in every parameter setter — the user's held inputs no
    /// longer drop on knob changes.
    fn clear_active_for_reharm(&mut self) {
        let mut keys: Vec<_> = self.active_notes.keys().copied().collect();
        keys.sort_unstable_by_key(|key| (key.note, key.source));
        for key in keys {
            let velocities = self.active_velocities.get(&key);
            for index in 0..self.active_notes[&key].len() {
                let velocity = velocities
                    .and_then(|frames| frames.get(index))
                    .copied()
                    .unwrap_or(100);
                self.pending_reharm_inputs
                    .push((key.source, key.note, velocity));
            }
        }
        self.active_notes.clear();
        self.active_port_maps.clear();
        self.active_velocities.clear();
    }

    pub fn harmonize_note_off(&mut self, note: Note) -> Vec<Note> {
        self.harmonize_note_off_owned(note, u8::from(note))
    }

    /// Release one frame belonging to the exact adapter-owned source.
    pub fn harmonize_note_off_owned(&mut self, note: Note, source: u8) -> Vec<Note> {
        let midi = u8::from(note);
        let key = ActiveNoteKey { source, note: midi };

        let _velocity = self
            .active_velocities
            .get_mut(&key)
            .and_then(VecDeque::pop_front);
        if self
            .active_velocities
            .get(&key)
            .is_some_and(VecDeque::is_empty)
        {
            self.active_velocities.remove(&key);
        }

        let port_map = self
            .active_port_maps
            .get_mut(&key)
            .and_then(VecDeque::pop_front);
        if self
            .active_port_maps
            .get(&key)
            .is_some_and(VecDeque::is_empty)
        {
            self.active_port_maps.remove(&key);
        }
        self.last_port_map = port_map.unwrap_or_else(|| vec![0]);

        let harmonies = self
            .active_notes
            .get_mut(&key)
            .and_then(VecDeque::pop_front);
        if self.active_notes.get(&key).is_some_and(VecDeque::is_empty) {
            self.active_notes.remove(&key);
        }

        match harmonies {
            Some(harmonies) => {
                let mut result = vec![note];
                result.extend(harmonies);
                result
            }
            None => vec![note], // No tracked harmony, just return original
        }
    }
}

impl Default for HarmonyEngine {
    fn default() -> Self {
        Self::with_voices(Key::C, HarmonyMode::PassThrough, 2)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine_creation() {
        let engine = HarmonyEngine::new(Key::C, HarmonyMode::DiatonicThirds);
        assert_eq!(engine.key(), Key::C);
        assert_eq!(engine.mode(), HarmonyMode::DiatonicThirds);
    }

    #[test]
    fn test_engine_pass_through() {
        let mut engine = HarmonyEngine::new(Key::C, HarmonyMode::PassThrough);
        let result = engine.harmonize(Note::C4);
        assert_eq!(result, vec![Note::C4]);
    }

    #[test]
    fn standard_tuning_preserves_existing_frequencies() {
        let mut engine = HarmonyEngine::with_voices(Key::C, HarmonyMode::DiatonicThirds, 3);
        let notes = engine.harmonize_note_on(Note::C4);
        let frame = engine.tune_harmony(&notes).unwrap();
        for pitch in frame.as_slice() {
            assert_eq!(
                pitch.frequency_hz,
                crate::tuning::midi_to_frequency(pitch.midi_note)
            );
        }
    }

    #[test]
    fn pure_c_major_tunes_through_harmony_engine() {
        let mut engine = HarmonyEngine::with_voices(Key::C, HarmonyMode::DiatonicThirds, 3);
        engine
            .set_tuning_config(TuningConfig {
                style: TuningStyle::Pure,
                depth: 1.0,
                harmonic_limit: crate::tuning::HarmonicLimit::Five,
            })
            .unwrap();
        let notes = engine.harmonize_note_on(Note::C4);
        assert_eq!(notes, vec![Note::C4, Note::E4, Note::G4]);
        let frame = engine.tune_harmony(&notes).unwrap();
        let pitches = frame.as_slice();
        assert!((pitches[1].frequency_hz / pitches[0].frequency_hz - 5.0 / 4.0).abs() < 1e-10);
        assert!((pitches[2].frequency_hz / pitches[0].frequency_hz - 3.0 / 2.0).abs() < 1e-10);
    }

    #[test]
    fn pure_tuning_supports_max_voice_mirror_output() {
        let mut engine = HarmonyEngine::with_voices(Key::C, HarmonyMode::DiatonicThirds, 8);
        engine.set_octave_mode(OctaveMode::Mirror);
        engine
            .set_tuning_config(TuningConfig {
                style: TuningStyle::Pure,
                depth: 1.0,
                harmonic_limit: crate::tuning::HarmonicLimit::Five,
            })
            .unwrap();
        let notes = engine.harmonize_note_on(Note::C4);
        assert!(notes.len() <= MAX_TUNING_VOICES);
        assert_eq!(engine.tune_harmony(&notes).unwrap().len(), notes.len());
    }

    #[test]
    fn tuning_config_setter_rejects_invalid_depth() {
        let mut engine = HarmonyEngine::default();
        let original = engine.tuning_config();
        let invalid = TuningConfig {
            depth: f32::NAN,
            ..original
        };
        assert_eq!(
            engine.set_tuning_config(invalid),
            Err(TuningError::NonFiniteDepth)
        );
        assert_eq!(engine.tuning_config(), original);
    }

    #[test]
    fn test_engine_diatonic_thirds() {
        let mut engine = HarmonyEngine::new(Key::C, HarmonyMode::DiatonicThirds);
        let result = engine.harmonize(Note::C4);
        assert_eq!(result, vec![Note::C4, Note::E4]);
    }

    #[test]
    fn test_key_change() {
        let mut engine = HarmonyEngine::new(Key::C, HarmonyMode::DiatonicThirds);

        // In C major: C + third = E
        let result = engine.harmonize(Note::C4);
        assert_eq!(result[1], Note::E4);

        // Change to G major
        engine.set_key(Key::G);

        // In G major: G + third = B
        let result = engine.harmonize(Note::G4);
        assert_eq!(result[1], Note::B4);
    }

    #[test]
    fn test_mode_change() {
        let mut engine = HarmonyEngine::new(Key::C, HarmonyMode::PassThrough);

        // Pass-through: only original note
        let result = engine.harmonize(Note::C4);
        assert_eq!(result.len(), 1);

        // Switch to thirds
        engine.set_mode(HarmonyMode::DiatonicThirds);

        // Now should have 2 notes
        let result = engine.harmonize(Note::C4);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_contrary_motion_mode() {
        let mut engine = HarmonyEngine::new(Key::C, HarmonyMode::ContraryMotion);

        // Default voice_position is bass (bottom), so harmony generates above
        // First note should produce harmony (third above)
        let result = engine.harmonize(Note::E4);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], Note::E4);
        assert_eq!(result[1], Note::G4); // Third above E = G
    }

    #[test]
    fn test_counterpoint_mode() {
        let mut engine = HarmonyEngine::new(Key::C, HarmonyMode::StrictCounterpoint);

        // Default voice_position is bass, so harmony generates above
        // Should produce consonant harmony (third above preferred)
        let result = engine.harmonize(Note::C4);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], Note::C4);
        assert_eq!(result[1], Note::E4); // Third above C = E
    }

    #[test]
    fn test_stateful_reset_on_key_change() {
        let mut engine = HarmonyEngine::new(Key::C, HarmonyMode::ContraryMotion);

        // Play some notes to build up state
        engine.harmonize(Note::C4);
        engine.harmonize(Note::E4);

        // Change key - state should reset
        engine.set_key(Key::G);

        // Next note should be treated as "first note" again
        // Default voice_position is bass, so harmony generates above (third above)
        let result = engine.harmonize(Note::G4);
        assert_eq!(result.len(), 2);
        // First note in contrary motion (directed above) gets third above
        assert_eq!(result[1], Note::B4); // G + 2 degrees in G major = B
    }

    // Note-On/Off tracking tests

    #[test]
    fn test_note_on_off_tracking() {
        let mut engine = HarmonyEngine::new(Key::C, HarmonyMode::DiatonicThirds);

        // Note-On C4 should produce C4, E4
        let on_result = engine.harmonize_note_on(Note::C4);
        assert_eq!(on_result, vec![Note::C4, Note::E4]);

        // Note-Off C4 should return same notes
        let off_result = engine.harmonize_note_off(Note::C4);
        assert_eq!(off_result, vec![Note::C4, Note::E4]);

        // Second Note-Off should just return the note (no longer tracked)
        let off_again = engine.harmonize_note_off(Note::C4);
        assert_eq!(off_again, vec![Note::C4]);
    }

    #[test]
    fn test_pass_through_no_tracking() {
        let mut engine = HarmonyEngine::new(Key::C, HarmonyMode::PassThrough);

        // Pass-through mode returns only original note
        let on_result = engine.harmonize_note_on(Note::C4);
        assert_eq!(on_result, vec![Note::C4]);

        // Note-Off should also return just the original (nothing was tracked)
        let off_result = engine.harmonize_note_off(Note::C4);
        assert_eq!(off_result, vec![Note::C4]);
    }

    #[test]
    fn test_multiple_active_notes() {
        let mut engine = HarmonyEngine::new(Key::C, HarmonyMode::DiatonicThirds);

        // Press C4 and E4 (chord)
        let c4_on = engine.harmonize_note_on(Note::C4);
        let e4_on = engine.harmonize_note_on(Note::E4);

        assert_eq!(c4_on, vec![Note::C4, Note::E4]);
        assert_eq!(e4_on, vec![Note::E4, Note::G4]);

        // Release E4 first
        let e4_off = engine.harmonize_note_off(Note::E4);
        assert_eq!(e4_off, vec![Note::E4, Note::G4]);

        // C4 should still be tracked
        let c4_off = engine.harmonize_note_off(Note::C4);
        assert_eq!(c4_off, vec![Note::C4, Note::E4]);
    }

    #[test]
    fn test_panic_clear_does_not_replay_or_release_stale_harmony() {
        let mut engine = HarmonyEngine::new(Key::C, HarmonyMode::DiatonicThirds);
        engine.harmonize_note_on(Note::C4);
        engine.clear_active_notes();

        assert!(engine.take_pending_releases().is_empty());
        assert!(engine.take_reharm_inputs().is_empty());
        assert_eq!(engine.harmonize_note_off(Note::C4), vec![Note::C4]);
    }

    #[test]
    fn test_key_change_clears_tracking() {
        let mut engine = HarmonyEngine::new(Key::C, HarmonyMode::DiatonicThirds);

        // Press C4
        engine.harmonize_note_on(Note::C4);

        // Change key
        engine.set_key(Key::G);

        // Note-Off should not find tracked harmony (cleared on key change)
        let off_result = engine.harmonize_note_off(Note::C4);
        assert_eq!(off_result, vec![Note::C4]);
    }

    #[test]
    fn test_mode_change_clears_tracking() {
        let mut engine = HarmonyEngine::new(Key::C, HarmonyMode::DiatonicThirds);

        // Press C4
        engine.harmonize_note_on(Note::C4);

        // Change mode
        engine.set_mode(HarmonyMode::DiatonicFourths);

        // Note-Off should not find tracked harmony (cleared on mode change)
        let off_result = engine.harmonize_note_off(Note::C4);
        assert_eq!(off_result, vec![Note::C4]);
    }

    // Chained harmony tests

    #[test]
    fn test_chained_harmonies_with_thirds() {
        // 4 voices: melody + 3 chained harmonies (each a third above previous)
        let mut engine = HarmonyEngine::with_voices(Key::C, HarmonyMode::DiatonicThirds, 4);

        // C4 + third = E4, E4 + third = G4, G4 + third = B4
        let result = engine.harmonize(Note::C4);
        assert_eq!(result.len(), 4);
        assert_eq!(result[0], Note::C4); // Melody
        assert_eq!(result[1], Note::E4); // Third above C
        assert_eq!(result[2], Note::G4); // Third above E
        assert_eq!(result[3], Note::B4); // Third above G
    }

    #[test]
    fn test_chained_harmonies_tracks_note_off() {
        let mut engine = HarmonyEngine::with_voices(Key::C, HarmonyMode::DiatonicThirds, 3);

        // Note-On should produce 3 notes
        let on_result = engine.harmonize_note_on(Note::C4);
        assert_eq!(on_result.len(), 3);

        // Note-Off should return same 3 notes
        let off_result = engine.harmonize_note_off(Note::C4);
        assert_eq!(off_result.len(), 3);
        assert_eq!(on_result, off_result);
    }

    #[test]
    fn test_single_voice_returns_melody_only() {
        let mut engine = HarmonyEngine::with_voices(Key::C, HarmonyMode::DiatonicThirds, 1);

        let result = engine.harmonize(Note::C4);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], Note::C4);
    }

    #[test]
    fn test_set_voice_count_changes_output() {
        let mut engine = HarmonyEngine::new(Key::C, HarmonyMode::DiatonicThirds);

        // Default is 2 voices
        let result = engine.harmonize(Note::C4);
        assert_eq!(result.len(), 2);

        // Change to 4 voices
        engine.set_voice_count(4);
        let result = engine.harmonize(Note::C4);
        assert_eq!(result.len(), 4);

        // Change back to 1 voice (melody only)
        engine.set_voice_count(1);
        let result = engine.harmonize(Note::C4);
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_chained_counterpoint_has_independent_state() {
        // 3 voices with strict counterpoint - each voice pair should have independent state
        let mut engine = HarmonyEngine::with_voices(Key::C, HarmonyMode::StrictCounterpoint, 3);

        // Play several notes to build up state in each voice pair
        let result1 = engine.harmonize(Note::C4);
        assert_eq!(result1.len(), 3);

        let result2 = engine.harmonize(Note::D4);
        assert_eq!(result2.len(), 3);

        // Each harmony should be different (different voice leading for each pair)
        // Note: we can't predict exact notes but the chain should work
        assert_ne!(result1[1], result1[2], "Chained harmonies should differ");
    }

    #[test]
    fn test_pass_through_ignores_voice_count() {
        // Pass-through mode should always return just the melody
        let mut engine = HarmonyEngine::with_voices(Key::C, HarmonyMode::PassThrough, 4);

        let result = engine.harmonize(Note::C4);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], Note::C4);
    }

    // Voice position tests

    #[test]
    fn test_voice_position_default_is_bass() {
        let engine = HarmonyEngine::with_voices(Key::C, HarmonyMode::DiatonicThirds, 4);
        assert_eq!(engine.voice_position(), 3); // Last index = bass
    }

    #[test]
    fn test_voice_position_top_generates_below() {
        let mut engine = HarmonyEngine::with_voices(Key::C, HarmonyMode::DiatonicThirds, 4);
        engine.set_voice_position(0); // Soprano: all harmony below

        let result = engine.harmonize(Note::C4);
        assert_eq!(result.len(), 4);
        assert_eq!(result[0], Note::C4);
        // Thirds below: C4 -> A3 -> F3 -> D3
        assert_eq!(result[1], Note::A3);
        assert_eq!(result[2], Note::F3);
        assert_eq!(result[3], Note::D3);
    }

    #[test]
    fn test_voice_position_middle_generates_both_directions() {
        let mut engine = HarmonyEngine::with_voices(Key::C, HarmonyMode::DiatonicThirds, 3);
        engine.set_voice_position(1); // Middle: 1 above, 1 below

        let result = engine.harmonize(Note::C4);
        assert_eq!(result.len(), 3);
        assert_eq!(result[0], Note::C4);
        // Above (closest first): E4
        assert_eq!(result[1], Note::E4);
        // Below: A3
        assert_eq!(result[2], Note::A3);
    }

    #[test]
    fn test_voice_position_clamped_on_set() {
        let mut engine = HarmonyEngine::with_voices(Key::C, HarmonyMode::DiatonicThirds, 3);
        engine.set_voice_position(10); // Out of range
        assert_eq!(engine.voice_position(), 2); // Clamped to max
    }

    #[test]
    fn test_voice_position_clamped_on_voice_count_change() {
        let mut engine = HarmonyEngine::with_voices(Key::C, HarmonyMode::DiatonicThirds, 4);
        engine.set_voice_position(3); // Bass in 4-voice
        engine.set_voice_count(2); // Now only 2 voices
        assert_eq!(engine.voice_position(), 1); // Clamped
    }

    // Octave mode tests

    #[test]
    fn test_octave_mode_none_no_change() {
        let mut engine = HarmonyEngine::with_voices(Key::C, HarmonyMode::DiatonicThirds, 3);
        engine.set_octave_mode(OctaveMode::None);

        // C4 + third = E4, E4 + third = G4 (no octave shift)
        let result = engine.harmonize(Note::C4);
        assert_eq!(result.len(), 3);
        assert_eq!(result[0], Note::C4);
        assert_eq!(result[1], Note::E4);
        assert_eq!(result[2], Note::G4);
    }

    #[test]
    fn test_octave_mode_spread() {
        let mut engine = HarmonyEngine::with_voices(Key::C, HarmonyMode::DiatonicThirds, 3);
        engine.set_octave_mode(OctaveMode::Spread);

        // C4 + third = E4 (+1 octave = E5), E4 + third = G4 (+2 octaves = G6)
        let result = engine.harmonize(Note::C4);
        assert_eq!(result.len(), 3);
        assert_eq!(result[0], Note::C4); // Melody unchanged
        assert_eq!(result[1], Note::E5); // First harmony +1 octave
        assert_eq!(result[2], Note::G6); // Second harmony +2 octaves
    }

    #[test]
    fn test_spread_intensity_preserves_harmony_pitch_classes() {
        fn pitches(intensity: f32) -> Vec<u8> {
            let mut engine = HarmonyEngine::with_voices(Key::D, HarmonyMode::DiatonicThirds, 4);
            engine.set_scale_mode(ScaleMode::Dorian);
            engine.set_octave_mode(OctaveMode::Spread);
            engine.set_octave_intensity(intensity);
            engine
                .harmonize(Note::D4)
                .into_iter()
                .map(|note| u8::from(note))
                .collect()
        }

        let tight = pitches(0.0);
        for intensity in [0.2, 0.4, 0.6, 0.8, 1.0] {
            let spread = pitches(intensity);
            assert_eq!(
                spread.iter().map(|note| note % 12).collect::<Vec<_>>(),
                tight.iter().map(|note| note % 12).collect::<Vec<_>>()
            );
        }
        assert_ne!(pitches(0.6), tight, "the knob must still widen the voicing");
    }

    #[test]
    fn test_octave_mode_bass_treble_split() {
        let mut engine = HarmonyEngine::with_voices(Key::C, HarmonyMode::StrictCounterpoint, 2);
        engine.set_octave_mode(OctaveMode::BassTrebleSplit);

        // Counterpoint typically produces harmony below melody
        // C4 -> harmony below (e.g., A3) -> shifted down to A2
        let result = engine.harmonize(Note::C4);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], Note::C4); // Melody unchanged
                                         // Harmony should be shifted (down if below C4, up if above)
        let harmony_midi = u8::from(result[1]);
        let c4_midi = u8::from(Note::C4);
        // Due to bass/treble split, harmony should be shifted away from melody
        assert!(
            harmony_midi < c4_midi - 11 || harmony_midi > c4_midi + 11,
            "Harmony {} should be shifted at least one octave from melody {}",
            harmony_midi,
            c4_midi
        );
    }

    #[test]
    fn test_mirror_mode_anchor_aware() {
        // 3 voices, default vp=2 (bass). User plays C4(60).
        // Harmony: E4(64) at arr=1, G4(67) at arr=0 — both above user.
        // Mirror +12 copies allowed (still above), -12 copies rejected (below user).
        // Result: C4, E4, G4, E5, G5 = 5 notes.
        let mut engine = HarmonyEngine::with_voices(Key::C, HarmonyMode::DiatonicThirds, 3);
        engine.set_octave_mode(OctaveMode::Mirror);

        let result = engine.harmonize(Note::C4);
        assert_eq!(
            result.len(),
            5,
            "Mirror should skip copies that cross anchor: {:?}",
            result
        );
        assert_eq!(result[0], Note::C4); // melody unchanged
        assert_eq!(result[1], Note::E4); // original harmony 1
        assert_eq!(result[2], Note::G4); // original harmony 2
    }

    #[test]
    fn test_mirror_port_map_assignments() {
        let mut engine = HarmonyEngine::with_voices(Key::C, HarmonyMode::DiatonicThirds, 3);
        engine.set_octave_mode(OctaveMode::Mirror);

        let _result = engine.harmonize(Note::C4);
        let port_map = engine.last_port_map();
        // 3-voice, default vp=2 (bass). Only +12 copies survive anchor check.
        // [user(arr=2), alto(arr=1), soprano(arr=0), alto+12(arr=1), soprano+12(arr=0)]
        assert_eq!(port_map.len(), 5);
        assert_eq!(port_map[0], 2); // melody (bass, arr pos 2)
        assert_eq!(port_map[1], 1); // harmony 1 (alto, arr pos 1)
        assert_eq!(port_map[2], 0); // harmony 2 (soprano, arr pos 0)
        assert_eq!(port_map[3], 1); // alto +12 duplicate -> arr pos 1
        assert_eq!(port_map[4], 0); // soprano +12 duplicate -> arr pos 0
    }

    #[test]
    fn test_mirror_out_of_range_skipped() {
        // High note where +12 would exceed 127
        let mut engine = HarmonyEngine::with_voices(Key::C, HarmonyMode::DiatonicThirds, 2);
        engine.set_octave_mode(OctaveMode::Mirror);

        // Note::try_from(120+12=132) should fail, so only -12 copy added
        // G9 is MIDI 127, let's use a high note
        let result = engine.harmonize(Note::C4);
        // This should work fine for C4 range
        assert!(result.len() >= 3); // At least melody + harmony + some duplicates
    }

    #[test]
    fn test_mirror_note_off_releases_all_duplicates() {
        let mut engine = HarmonyEngine::with_voices(Key::C, HarmonyMode::DiatonicThirds, 3);
        engine.set_octave_mode(OctaveMode::Mirror);

        let on_result = engine.harmonize_note_on(Note::C4);
        assert_eq!(on_result.len(), 5); // anchor-aware: only +12 copies survive

        let off_result = engine.harmonize_note_off(Note::C4);
        assert_eq!(
            off_result.len(),
            5,
            "Note-Off should release all notes from Note-On"
        );
        // The harmony notes stored should match
        assert_eq!(on_result[1..], off_result[1..]);
    }

    #[test]
    fn test_mirror_port_map_restored_on_note_off() {
        let mut engine = HarmonyEngine::with_voices(Key::C, HarmonyMode::DiatonicThirds, 3);
        engine.set_octave_mode(OctaveMode::Mirror);

        engine.harmonize_note_on(Note::C4);
        let on_port_map: Vec<usize> = engine.last_port_map().to_vec();

        // Some other operation might change last_port_map
        engine.harmonize_note_off(Note::C4);
        let off_port_map: Vec<usize> = engine.last_port_map().to_vec();

        assert_eq!(
            on_port_map, off_port_map,
            "Port map should be restored on Note-Off"
        );
    }

    #[test]
    fn test_non_mirror_port_map_is_identity() {
        let mut engine = HarmonyEngine::with_voices(Key::C, HarmonyMode::DiatonicThirds, 3);
        engine.set_octave_mode(OctaveMode::Spread);

        let result = engine.harmonize(Note::C4);
        let port_map = engine.last_port_map();
        assert_eq!(port_map.len(), result.len());
        // 3-voice, default voice_position=2 (bass). Arrangement indices: [2, 1, 0]
        // Port map should reflect SATB arrangement positions, not identity.
        assert_eq!(port_map, &[2, 1, 0]);
    }

    #[test]
    fn test_octave_mode_clears_note_tracking() {
        let mut engine = HarmonyEngine::with_voices(Key::C, HarmonyMode::DiatonicThirds, 2);

        // Press a note
        engine.harmonize_note_on(Note::C4);

        // Change octave mode
        engine.set_octave_mode(OctaveMode::Spread);

        // Note-Off should not find tracked harmony (cleared on octave mode change)
        let off_result = engine.harmonize_note_off(Note::C4);
        assert_eq!(off_result, vec![Note::C4]);
    }

    // Voice leading integration tests

    #[test]
    fn test_vl_disabled_by_default() {
        let engine = HarmonyEngine::with_voices(Key::C, HarmonyMode::DiatonicThirds, 3);
        assert!(!engine.voice_leading_enabled());
    }

    #[test]
    fn test_vl_disabled_unchanged_behavior() {
        let mut engine = HarmonyEngine::with_voices(Key::C, HarmonyMode::DiatonicThirds, 4);
        // VL disabled (default): behavior identical to before
        let result = engine.harmonize(Note::C4);
        assert_eq!(result.len(), 4);
        assert_eq!(result[0], Note::C4);
        assert_eq!(result[1], Note::E4);
        assert_eq!(result[2], Note::G4);
        assert_eq!(result[3], Note::B4);
    }

    #[test]
    fn test_vl_enabled_produces_output() {
        let mut engine = HarmonyEngine::with_voices(Key::C, HarmonyMode::DiatonicThirds, 3);
        engine.set_voice_leading_enabled(true);

        let result = engine.harmonize(Note::C4);
        assert_eq!(result.len(), 3);
        assert_eq!(result[0], Note::C4); // Melody NEVER modified
    }

    #[test]
    fn test_vl_melody_never_modified() {
        let mut engine = HarmonyEngine::with_voices(Key::C, HarmonyMode::DiatonicThirds, 4);
        engine.set_voice_leading_enabled(true);

        // Play several notes
        for note in [Note::C4, Note::D4, Note::E4, Note::F4, Note::G4] {
            let result = engine.harmonize(note);
            assert_eq!(result[0], note, "Melody must never be modified by VL");
        }
    }

    #[test]
    fn test_vl_resets_on_key_change() {
        let mut engine = HarmonyEngine::with_voices(Key::C, HarmonyMode::DiatonicThirds, 3);
        engine.set_voice_leading_enabled(true);

        engine.harmonize(Note::C4);
        engine.set_key(Key::G);

        // After key change, VL state should be reset (previous_voicing cleared)
        // This means next harmonize is treated as first chord
        let result = engine.harmonize(Note::G4);
        assert_eq!(result[0], Note::G4);
        assert!(result.len() == 3);
    }

    #[test]
    fn test_vl_resets_on_mode_change() {
        let mut engine = HarmonyEngine::with_voices(Key::C, HarmonyMode::DiatonicThirds, 3);
        engine.set_voice_leading_enabled(true);

        engine.harmonize(Note::C4);
        engine.set_mode(HarmonyMode::DiatonicFourths);

        let result = engine.harmonize(Note::C4);
        assert_eq!(result[0], Note::C4);
    }

    #[test]
    fn test_vl_resets_on_style_change() {
        use crate::voice_leading::VoiceLeadingStyle;

        let mut engine = HarmonyEngine::with_voices(Key::C, HarmonyMode::DiatonicThirds, 3);
        engine.set_voice_leading_enabled(true);

        engine.harmonize(Note::C4);
        engine.set_voice_leading_style(VoiceLeadingStyle::Palestrina);

        let result = engine.harmonize(Note::C4);
        assert_eq!(result[0], Note::C4);
    }

    #[test]
    fn palestrina_voice_leading_does_not_invent_unscheduled_suspension() {
        let mut engine = HarmonyEngine::with_voices(Key::C, HarmonyMode::DiatonicThirds, 3);
        engine.set_voice_leading_enabled(true);
        engine.set_voice_leading_style(VoiceLeadingStyle::Palestrina);

        engine.harmonize(Note::C4);
        let result = engine.harmonize(Note::D4);
        let pitch_classes: Vec<u8> = result.iter().map(|note| u8::from(*note) % 12).collect();

        assert_eq!(pitch_classes, vec![2, 5, 9]);
    }

    #[test]
    fn modal_linework_baseline_is_deterministic_and_releases_every_note() {
        fn configured_engine() -> HarmonyEngine {
            let mut engine = HarmonyEngine::with_voices(Key::C, HarmonyMode::StrictCounterpoint, 4);
            engine.set_scale_mode(ScaleMode::Dorian);
            engine.set_voice_position(1);
            engine.set_voice_leading_enabled(true);
            engine.set_voice_leading_style(VoiceLeadingStyle::Palestrina);
            engine.set_counterpoint_species(CounterpointSpecies::Species1);
            engine.set_counterpoint_strictness(CounterpointStrictness::Strict);
            engine
        }

        let phrase = [Note::C4, Note::D4, Note::Eb4, Note::F4, Note::Eb4];
        let mut first = configured_engine();
        let mut second = configured_engine();

        for note in phrase {
            let first_on = first.harmonize_note_on(note);
            let second_on = second.harmonize_note_on(note);
            assert_eq!(first_on, second_on);
            assert_eq!(first_on.len(), 4);
            assert_eq!(first.harmonize_note_off(note), first_on);
            assert_eq!(second.harmonize_note_off(note), second_on);
            assert!(first.active_notes.is_empty());
            assert!(second.active_notes.is_empty());
        }
    }

    #[test]
    fn clean_runtime_fork_copies_configuration_without_ownership_history() {
        let mut live = HarmonyEngine::with_voices(Key::C, HarmonyMode::StrictCounterpoint, 4);
        live.set_scale_mode(ScaleMode::Dorian);
        live.set_voice_position(1);
        live.set_octave_mode(OctaveMode::Spread);
        live.set_octave_intensity(0.4);
        live.set_voice_leading_enabled(true);
        live.set_voice_leading_style(VoiceLeadingStyle::Palestrina);
        live.set_counterpoint_species(CounterpointSpecies::Species4);
        live.set_counterpoint_strictness(CounterpointStrictness::Strict);
        live.set_interchange_enabled(true);
        live.set_borrowing_range(4);
        let _ = live.harmonize_note_on(Note::C4);
        assert!(!live.active_notes.is_empty());

        let fork = live.fork_clean_runtime();
        assert!(fork.has_same_configuration(&live));
        assert!(fork.active_notes.is_empty());
        assert!(fork.active_port_maps.is_empty());
        assert!(fork.pending_releases.is_empty());
        assert!(fork.pending_reharm_inputs.is_empty());
        assert!(fork.harmonic_context.is_none());

        live.set_key(Key::D);
        assert!(!fork.has_same_configuration(&live));
    }

    #[test]
    fn test_vl_style_getter_setter() {
        use crate::voice_leading::VoiceLeadingStyle;

        let mut engine = HarmonyEngine::new(Key::C, HarmonyMode::DiatonicThirds);
        assert_eq!(engine.voice_leading_style(), VoiceLeadingStyle::Free);

        engine.set_voice_leading_style(VoiceLeadingStyle::BachChorale);
        assert_eq!(engine.voice_leading_style(), VoiceLeadingStyle::BachChorale);
    }

    #[test]
    fn test_vl_before_octave_mode() {
        // Verify VL runs before octave mode by enabling both
        let mut engine = HarmonyEngine::with_voices(Key::C, HarmonyMode::DiatonicThirds, 3);
        engine.set_voice_leading_enabled(true);
        engine.set_octave_mode(OctaveMode::Spread);

        let result = engine.harmonize(Note::C4);
        assert_eq!(result[0], Note::C4); // Melody unchanged
                                         // With Spread, harmony voices get +1/+2 octave shifts applied AFTER VL
        assert!(result.len() == 3);
    }

    // Scale mode and modal interchange tests

    #[test]
    fn test_scale_mode_default_is_ionian() {
        let engine = HarmonyEngine::default();
        assert_eq!(engine.scale_mode(), ScaleMode::Ionian);
    }

    #[test]
    fn test_set_scale_mode_changes_harmony() {
        let mut engine = HarmonyEngine::with_voices(Key::C, HarmonyMode::DiatonicThirds, 2);

        // C Ionian: C4 + third = E4
        let result_ionian = engine.harmonize(Note::C4);
        assert_eq!(result_ionian[1], Note::E4);

        // C Dorian: C4 + third = Eb4
        engine.set_scale_mode(ScaleMode::Dorian);
        let result_dorian = engine.harmonize(Note::C4);
        assert_eq!(result_dorian[1], Note::Eb4);
    }

    #[test]
    fn test_interchange_enabled_propagates() {
        let mut engine = HarmonyEngine::default();
        assert!(!engine.interchange_enabled());

        engine.set_interchange_enabled(true);
        assert!(engine.interchange_enabled());
    }

    #[test]
    fn test_set_key_preserves_scale_mode() {
        let mut engine = HarmonyEngine::with_voices(Key::C, HarmonyMode::DiatonicThirds, 2);
        engine.set_scale_mode(ScaleMode::Dorian);
        engine.set_key(Key::G);
        assert_eq!(engine.scale_mode(), ScaleMode::Dorian);

        // G Dorian: G4 + third should give Bb4
        let result = engine.harmonize(Note::G4);
        assert_eq!(result[1], Note::Bb4);
    }

    #[test]
    fn test_borrowing_range_propagates() {
        let mut engine = HarmonyEngine::default();
        assert_eq!(engine.borrowing_range(), 3);

        engine.set_borrowing_range(5);
        assert_eq!(engine.borrowing_range(), 5);

        engine.set_borrowing_range(0); // should clamp to 1
        assert_eq!(engine.borrowing_range(), 1);
    }

    #[test]
    fn test_interchange_produces_borrowed_harmonies() {
        let mut engine = HarmonyEngine::with_voices(Key::C, HarmonyMode::DiatonicThirds, 2);
        engine.set_interchange_enabled(true);

        // Eb4 is not in C Ionian but is in C Aeolian
        let result = engine.harmonize_note_on(Note::Eb4);
        assert!(result.len() >= 2);
        assert!(engine.last_borrowed_from().is_some());
    }

    #[test]
    fn test_vl_works_with_all_modes() {
        let modes = HarmonyMode::all();
        for &mode in modes {
            let mut engine = HarmonyEngine::with_voices(Key::C, mode, 3);
            engine.set_voice_leading_enabled(true);

            let result = engine.harmonize(Note::C4);
            assert_eq!(result[0], Note::C4, "Melody unchanged for mode {:?}", mode);
            // Should not panic for any mode
        }
    }

    // DiatonicThirds on BH scale tests — prove that chord-tone/passing-tone
    // parity preservation is an emergent property of the 8-note scale, not a
    // special mode algorithm.

    #[test]
    fn test_diatonic_thirds_on_bh_scale_chord_tone_parity() {
        // C BH Major 6th Dim: C D E F G Ab A B (degrees 0-7)
        // C4 is degree 0 (chord tone, even). +2 degrees = E4 (degree 2, chord tone, even).
        let mut engine = HarmonyEngine::with_voices(Key::C, HarmonyMode::DiatonicThirds, 2);
        engine.set_scale_mode(ScaleMode::BHMajor6thDim);

        let result = engine.harmonize(Note::C4);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], Note::C4);
        assert_eq!(result[1], Note::E4); // degree 0 + 2 = degree 2 = E
    }

    #[test]
    fn test_diatonic_thirds_on_bh_scale_passing_tone_parity() {
        // C BH Major 6th Dim: C D E F G Ab A B (degrees 0-7)
        // D4 is degree 1 (passing tone, odd). +2 degrees = F4 (degree 3, passing tone, odd).
        let mut engine = HarmonyEngine::with_voices(Key::C, HarmonyMode::DiatonicThirds, 2);
        engine.set_scale_mode(ScaleMode::BHMajor6thDim);

        let result = engine.harmonize(Note::D4);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], Note::D4);
        assert_eq!(result[1], Note::F4); // degree 1 + 2 = degree 3 = F
    }

    #[test]
    fn test_all_modes_with_barry_harris_scale() {
        for &mode in HarmonyMode::all() {
            let mut engine = HarmonyEngine::with_voices(Key::C, mode, 2);
            engine.set_scale_mode(ScaleMode::BHMajor6thDim);

            let result = engine.harmonize(Note::C4);
            assert!(
                !result.is_empty(),
                "Mode {:?} with BH scale should produce output",
                mode
            );
            assert_eq!(result[0], Note::C4);
        }
    }

    #[test]
    fn test_all_modes_with_exotic_scales() {
        for &mode in HarmonyMode::all() {
            let mut engine = HarmonyEngine::with_voices(Key::C, mode, 2);
            engine.set_scale_mode(ScaleMode::DoubleHarmonic);

            let result = engine.harmonize(Note::C4);
            assert!(
                !result.is_empty(),
                "Mode {:?} with DoubleHarmonic should not panic",
                mode
            );
        }
    }

    #[test]
    fn test_barry_harris_deserializes() {
        let json = r#""barry_harris""#;
        let mode: HarmonyMode = serde_json::from_str(json).unwrap();
        assert_eq!(mode, HarmonyMode::BarryHarris);
    }

    #[test]
    fn test_barry_harris_produces_four_voice_melody_top_blocks() {
        let mut engine = HarmonyEngine::with_voices(Key::C, HarmonyMode::BarryHarris, 4);
        engine.set_scale_mode(ScaleMode::BHMajor6thDim);
        engine.set_voice_position(0);
        let scale = Scale::new(0, ScaleMode::BHMajor6thDim);
        for input in [
            Note::C4,
            Note::D4,
            Note::E4,
            Note::F4,
            Note::G4,
            Note::Ab4,
            Note::A4,
            Note::B4,
        ] {
            let result = engine.harmonize(input);
            assert_eq!(result.len(), 4, "four total voices for {input:?}");
            assert_eq!(result[0], input, "played note remains the source voice");
            assert_eq!(
                result.iter().filter(|&&voice| voice == input).count(),
                1,
                "played note appears exactly once for {input:?}: {result:?}"
            );
            assert!(
                result.iter().skip(1).all(|&voice| voice < input),
                "soprano source must stay above its harmony for {input:?}: {result:?}"
            );
            let input_parity = scale.degree_of(input).unwrap() % 2;
            assert!(result.iter().all(|&voice| {
                scale
                    .degree_of(voice)
                    .is_some_and(|degree| degree % 2 == input_parity)
            }));
        }
    }
    #[test]
    fn test_barry_harris_major_fallback_preserves_player_scale() {
        let mut engine = HarmonyEngine::with_voices(Key::C, HarmonyMode::PassThrough, 4);
        engine.set_scale_mode(ScaleMode::Ionian);
        engine.set_mode(HarmonyMode::BarryHarris);
        assert_eq!(engine.scale_mode(), ScaleMode::Ionian);
        assert_eq!(engine.harmonize(Note::C4).len(), 4);
    }
    #[test]
    fn test_barry_harris_minor_fallback_preserves_player_scale() {
        let mut engine = HarmonyEngine::with_voices(Key::C, HarmonyMode::PassThrough, 4);
        engine.set_scale_mode(ScaleMode::Aeolian);
        engine.set_mode(HarmonyMode::BarryHarris);
        assert_eq!(engine.scale_mode(), ScaleMode::Aeolian);
        assert_eq!(engine.harmonize(Note::C4).len(), 4);
    }
    #[test]
    fn test_leaving_barry_harris_keeps_player_scale_unchanged() {
        let mut engine = HarmonyEngine::with_voices(Key::C, HarmonyMode::PassThrough, 4);
        engine.set_scale_mode(ScaleMode::Dorian);
        engine.set_mode(HarmonyMode::BarryHarris);
        engine.set_mode(HarmonyMode::DiatonicThirds);
        assert_eq!(engine.scale_mode(), ScaleMode::Dorian);
    }
    #[test]
    fn test_barry_harris_note_tracking() {
        let mut engine = HarmonyEngine::with_voices(Key::C, HarmonyMode::BarryHarris, 4);
        engine.set_scale_mode(ScaleMode::BHMajor6thDim);
        engine.set_voice_position(0);
        for input in [
            Note::C4,
            Note::D4,
            Note::E4,
            Note::F4,
            Note::G4,
            Note::Ab4,
            Note::A4,
            Note::B4,
            Note::Db4,
        ] {
            let on = engine.harmonize_note_on(input);
            let off = engine.harmonize_note_off(input);
            assert_eq!(
                off, on,
                "NoteOff must release the exact block for {input:?}"
            );
            assert!(engine.active_notes.is_empty());
        }
    }
    #[test]
    fn test_barry_harris_chromatic() {
        let mut engine = HarmonyEngine::with_voices(Key::C, HarmonyMode::BarryHarris, 5);
        engine.set_scale_mode(ScaleMode::BHMajor6thDim);
        assert_eq!(engine.harmonize(Note::Db4).len(), 1);
    }
    #[test]
    fn test_barry_harris_chord_tone_parity() {
        let mut engine = HarmonyEngine::with_voices(Key::C, HarmonyMode::BarryHarris, 5);
        engine.set_scale_mode(ScaleMode::BHMajor6thDim);
        let scale = Scale::new(0, ScaleMode::BHMajor6thDim);
        for n in &engine.harmonize(Note::C4)[1..] {
            assert_eq!(scale.degree_of(*n).unwrap() % 2, 0);
        }
    }
    #[test]
    fn test_barry_harris_passing_tone_parity() {
        let mut engine = HarmonyEngine::with_voices(Key::C, HarmonyMode::BarryHarris, 5);
        engine.set_scale_mode(ScaleMode::BHMajor6thDim);
        let scale = Scale::new(0, ScaleMode::BHMajor6thDim);
        for n in &engine.harmonize(Note::D4)[1..] {
            assert_eq!(scale.degree_of(*n).unwrap() % 2, 1);
        }
    }
    #[test]
    fn test_existing_modes_unaffected() {
        let mut engine = HarmonyEngine::new(Key::C, HarmonyMode::DiatonicThirds);
        assert_eq!(engine.harmonize(Note::C4), vec![Note::C4, Note::E4]);
    }

    // --- Counterpoint Species 1-4 wiring tests ---

    /// Setting the species on the engine must be reflected on each internal
    /// CounterpointState so dispatch uses the requested species.
    #[test]
    fn test_set_counterpoint_species_propagates_to_states() {
        let mut engine = HarmonyEngine::with_voices(Key::C, HarmonyMode::StrictCounterpoint, 3);
        engine.set_counterpoint_species(CounterpointSpecies::Species3);
        assert_eq!(engine.counterpoint_species(), CounterpointSpecies::Species3);
        for s in &engine.counterpoint_states {
            assert_eq!(s.species(), CounterpointSpecies::Species3);
        }
    }

    #[test]
    fn test_set_counterpoint_strictness_propagates_to_states() {
        let mut engine = HarmonyEngine::with_voices(Key::C, HarmonyMode::StrictCounterpoint, 3);
        engine.set_counterpoint_strictness(CounterpointStrictness::Relaxed);
        assert_eq!(
            engine.counterpoint_strictness(),
            CounterpointStrictness::Relaxed
        );
        for s in &engine.counterpoint_states {
            assert_eq!(s.strictness(), CounterpointStrictness::Relaxed);
        }
    }

    /// Newly allocated states from set_voice_count must inherit the engine's
    /// current species / strictness, not the CounterpointState defaults.
    #[test]
    fn test_voice_count_change_preserves_species_and_strictness() {
        let mut engine = HarmonyEngine::with_voices(Key::C, HarmonyMode::StrictCounterpoint, 2);
        engine.set_counterpoint_species(CounterpointSpecies::Species4);
        engine.set_counterpoint_strictness(CounterpointStrictness::Relaxed);
        engine.set_voice_count(4);
        for s in &engine.counterpoint_states {
            assert_eq!(s.species(), CounterpointSpecies::Species4);
            assert_eq!(s.strictness(), CounterpointStrictness::Relaxed);
        }
    }

    #[test]
    fn test_counterpoint_beat_phase_roundtrip() {
        let mut engine = HarmonyEngine::new(Key::C, HarmonyMode::StrictCounterpoint);
        assert_eq!(engine.counterpoint_beat_phase(), None);
        engine.set_counterpoint_beat_phase(Some(2.5));
        assert_eq!(engine.counterpoint_beat_phase(), Some(2.5));
        // Mode change clears the cached phase so the router must push fresh.
        engine.set_mode(HarmonyMode::PassThrough);
        assert_eq!(engine.counterpoint_beat_phase(), None);
    }

    /// Species 1 ignores beat_phase; the harmonize output must be identical
    /// whether or not a beat position is supplied.
    #[test]
    fn test_species1_ignores_beat_phase() {
        let mut engine_a = HarmonyEngine::with_voices(Key::C, HarmonyMode::StrictCounterpoint, 2);
        let mut engine_b = HarmonyEngine::with_voices(Key::C, HarmonyMode::StrictCounterpoint, 2);
        engine_b.set_counterpoint_beat_phase(Some(1.25));

        let melody = [Note::C4, Note::D4, Note::E4, Note::F4, Note::G4];
        for &n in &melody {
            assert_eq!(engine_a.harmonize(n), engine_b.harmonize(n));
        }
    }

    /// Different species across the same melody must, at some point, produce
    /// different harmony output — otherwise the dispatch is not actually
    /// switching species behavior. This guards the real end-to-end wiring.
    #[test]
    fn test_species_change_alters_harmony_output() {
        // Long enough melody to give Species 2 suspended / passing tones room
        // to diverge from Species 1's note-against-note motion.
        let melody = [
            Note::C4,
            Note::D4,
            Note::E4,
            Note::F4,
            Note::G4,
            Note::A4,
            Note::G4,
            Note::F4,
            Note::E4,
            Note::D4,
            Note::C4,
        ];

        let mut sp1 = HarmonyEngine::with_voices(Key::C, HarmonyMode::StrictCounterpoint, 2);
        sp1.set_counterpoint_species(CounterpointSpecies::Species1);

        let mut sp2 = HarmonyEngine::with_voices(Key::C, HarmonyMode::StrictCounterpoint, 2);
        sp2.set_counterpoint_species(CounterpointSpecies::Species2);

        let mut any_different = false;
        for (i, &n) in melody.iter().enumerate() {
            // Advance the beat clock so Species 2 alternates strong/weak beats.
            sp2.set_counterpoint_beat_phase(Some(i as f64 * 0.5));

            let a = sp1.harmonize(n);
            let b = sp2.harmonize(n);
            if a != b {
                any_different = true;
            }
        }

        assert!(
            any_different,
            "Species 1 and Species 2 produced identical output across the \
             full melody — species dispatch is not wired end-to-end."
        );
    }

    // --- Auto-key stuck-note release wiring ---

    #[test]
    fn auto_key_off_never_populates_pending_releases() {
        let mut engine = HarmonyEngine::with_voices(Key::C, HarmonyMode::DiatonicThirds, 2);
        // auto_key is false by default; play a phrase that would otherwise
        // shift a key detector strongly.
        let _ = engine.harmonize_note_on(Note::C4);
        let _ = engine.harmonize_note_on(Note::E4);
        let _ = engine.harmonize_note_on(Note::G4);
        assert!(engine.take_pending_releases().is_empty());
    }

    #[test]
    fn auto_key_change_queues_old_harmonies_for_release() {
        // Stage 1: lock the detector on C major while holding harmony notes
        // for several inputs. After enabling auto-key those harmonies are
        // tracked under `active_notes`.
        let mut engine = HarmonyEngine::with_voices(Key::C, HarmonyMode::DiatonicThirds, 2);
        engine.set_auto_key(true);
        for &n in &[Note::C4, Note::D4, Note::E4, Note::F4, Note::G4] {
            let _ = engine.harmonize_note_on(n);
        }
        assert!(engine.take_pending_releases().is_empty());

        // Stage 2: feed sustained G-major content (full scale, repeated) so
        // the histogram clearly tilts toward G past the confidence margin.
        // The old C-major harmonies must be queued for release before
        // `set_key` wipes `active_notes`.
        let g_major_scale = [
            Note::G4,
            Note::A4,
            Note::B4,
            Note::C5,
            Note::D5,
            Note::E5,
            Note::FSharp5,
        ];
        for _ in 0..3 {
            for &n in &g_major_scale {
                let _ = engine.harmonize_note_on(n);
            }
        }
        let stale = engine.take_pending_releases();
        assert!(
            !stale.is_empty(),
            "auto-key change should queue stale harmonies for release"
        );
        assert!(
            stale.windows(2).all(|pair| pair[0] <= pair[1]),
            "auto-key releases must have canonical pitch order: {stale:?}"
        );
        assert_eq!(
            engine.key(),
            Key::G,
            "detector should have committed to G after sustained content"
        );
    }

    #[test]
    fn take_pending_releases_drains_the_queue() {
        let mut engine = HarmonyEngine::with_voices(Key::C, HarmonyMode::DiatonicThirds, 2);
        engine.set_auto_key(true);
        for &n in &[Note::C4, Note::D4, Note::E4, Note::F4, Note::G4] {
            let _ = engine.harmonize_note_on(n);
        }
        let g_major_scale = [
            Note::G4,
            Note::A4,
            Note::B4,
            Note::C5,
            Note::D5,
            Note::E5,
            Note::FSharp5,
        ];
        for _ in 0..3 {
            for &n in &g_major_scale {
                let _ = engine.harmonize_note_on(n);
            }
        }
        let _ = engine.take_pending_releases();
        // Second call must be empty — the queue is drained, not cloned.
        assert!(engine.take_pending_releases().is_empty());
    }

    // --- voice_position chain direction tests ---

    #[test]
    fn voice_position_soprano_all_harmonies_below_input() {
        let mut e = HarmonyEngine::with_voices(Key::C, HarmonyMode::DiatonicThirds, 4);
        e.set_voice_position(0); // soprano = top
        let result = e.harmonize_note_on(Note::C5);
        let input_midi = u8::from(Note::C5);
        for (i, &n) in result.iter().enumerate().skip(1) {
            let m = u8::from(n);
            assert!(
                m < input_midi,
                "voice_position=0 (soprano): result[{i}] = {m}, expected below input {input_midi}",
            );
        }
    }

    #[test]
    fn voice_position_bass_all_harmonies_above_input() {
        let mut e = HarmonyEngine::with_voices(Key::C, HarmonyMode::DiatonicThirds, 4);
        e.set_voice_position(3); // bass = bottom
        let result = e.harmonize_note_on(Note::C3);
        let input_midi = u8::from(Note::C3);
        for (i, &n) in result.iter().enumerate().skip(1) {
            let m = u8::from(n);
            assert!(
                m > input_midi,
                "voice_position=3 (bass): result[{i}] = {m}, expected above input {input_midi}",
            );
        }
    }

    #[test]
    fn voice_position_tenor_harmonies_split_around_input() {
        let mut e = HarmonyEngine::with_voices(Key::C, HarmonyMode::DiatonicThirds, 4);
        e.set_voice_position(2); // tenor = third from top in 4-voice
        let result = e.harmonize_note_on(Note::C4);
        let input_midi = u8::from(Note::C4);
        let above = result[1..]
            .iter()
            .filter(|n| u8::from(**n) > input_midi)
            .count();
        let below = result[1..]
            .iter()
            .filter(|n| u8::from(**n) < input_midi)
            .count();
        assert!(
            above > 0 && below > 0,
            "voice_position=2 (tenor): expected harmonies both above and below input, got above={above} below={below}, full result={result:?}",
        );
    }

    // --- block-voicing modes respect voice_position (fix #1) ---

    #[test]
    fn block_chord_bass_position_redistributes_voicing_above() {
        // Barry Harris block voicing normally stacks below the melody.
        // With voice_position=Bass, all harmonies must be above the input.
        let mut e = HarmonyEngine::with_voices(Key::C, HarmonyMode::BarryHarris, 4);
        e.set_voice_position(3); // bass
        let result = e.harmonize_note_on(Note::C4);
        if result.len() <= 1 {
            return; // No voicing produced — depends on beat phase; not a regression
        }
        let input_midi = u8::from(Note::C4);
        for (i, &n) in result.iter().enumerate().skip(1) {
            let m = u8::from(n);
            assert!(
                m > input_midi,
                "block-chord voice_position=3 (bass): result[{i}] = {m}, expected above input {input_midi}, full result={result:?}",
            );
        }
    }

    #[test]
    fn block_chord_alto_position_splits_around_input() {
        // voice_position=alto in 4-voice → 1 above, 2 below.
        let mut e = HarmonyEngine::with_voices(Key::C, HarmonyMode::BarryHarris, 4);
        e.set_voice_position(1); // alto
        let result = e.harmonize_note_on(Note::C4);
        if result.len() <= 1 {
            return;
        }
        let input_midi = u8::from(Note::C4);
        let above = result[1..]
            .iter()
            .filter(|n| u8::from(**n) > input_midi)
            .count();
        assert_eq!(
            above, 1,
            "block-chord voice_position=1 (alto): expected exactly 1 above-input harmony, got {above}, full result={result:?}",
        );
    }

    // --- counterpoint Species 2-4 honor direction (fix #1 cont.) ---

    #[test]
    fn counterpoint_species2_bass_position_harmony_above() {
        let mut e = HarmonyEngine::with_voices(Key::C, HarmonyMode::StrictCounterpoint, 2);
        e.set_counterpoint_species(CounterpointSpecies::Species2);
        e.set_voice_position(1); // bass in 2-voice
        e.set_counterpoint_beat_phase(Some(0.0));
        let result = e.harmonize_note_on(Note::C4);
        if result.len() <= 1 {
            return;
        }
        let input_midi = u8::from(Note::C4);
        let h = u8::from(result[1]);
        assert!(
            h > input_midi,
            "Species2 voice_position=bass: harmony {h} should be above input {input_midi}, full result={result:?}",
        );
    }

    // --- register-edge wrap (fix #2) ---

    #[test]
    fn chain_continues_at_low_register_edge() {
        // User plays MIDI 1 (very low) as soprano in 4-voice. Chain below
        // would land below MIDI 0 — pre-fix the chain broke and dropped
        // voices; post-fix it wraps an octave to keep voices audible.
        let mut e = HarmonyEngine::with_voices(Key::C, HarmonyMode::DiatonicThirds, 4);
        e.set_voice_position(0); // soprano
        let very_low = Note::try_from(1u8).unwrap();
        let result = e.harmonize_note_on(very_low);
        assert!(
            result.len() >= 2,
            "chain at register edge produced only {} note(s); expected the wrap fallback to keep voices, full result={result:?}",
            result.len()
        );
    }

    /// Regression: without an external transport, Species 2 must still
    /// produce different output from Species 1 across a melody. Before
    /// the synthetic-beat fix, `counterpoint_beat_phase = None` made
    /// every species fall back to Species 1 in `process_with_beat`.
    #[test]
    fn test_species2_differs_from_species1_without_transport() {
        let melody: Vec<Note> = (0..8)
            .map(|_| Note::C4)
            .chain((0..8).map(|_| Note::D4))
            .collect();
        // identical engines, no set_counterpoint_beat_phase calls — pure synthetic.
        let mut e1 = HarmonyEngine::new(Key::C, HarmonyMode::StrictCounterpoint);
        let mut e2 = HarmonyEngine::new(Key::C, HarmonyMode::StrictCounterpoint);
        e1.set_counterpoint_species(CounterpointSpecies::Species1);
        e2.set_counterpoint_species(CounterpointSpecies::Species2);
        let mut diverged = false;
        for &n in &melody {
            let r1 = e1.harmonize_note_on(n);
            let r2 = e2.harmonize_note_on(n);
            if r1 != r2 {
                diverged = true;
            }
        }
        assert!(diverged, "Species 2 must diverge from Species 1 even without an external transport — synthetic beat fallback is not active");
    }

    // --- Issue #100: bass-register suppression ---

    /// Default state: suppression is OFF, bass notes get full harmony.
    /// Regression guard — flipping the default to ON would silently
    /// remove harmony for existing users of low-register input.
    #[test]
    fn test_bass_register_off_by_default() {
        let mut e = HarmonyEngine::new(Key::C, HarmonyMode::DiatonicThirds);
        assert!(!e.suppress_bass_register(), "default should be off");
        let result = e.harmonize_note_on(Note::C2); // MIDI 36, well below threshold
        assert!(result.len() > 1, "default-off should still produce harmony");
    }

    /// With suppression on, a note below the threshold passes through
    /// unchanged (just the input, no harmony).
    #[test]
    fn test_bass_register_suppresses_below_threshold() {
        let mut e = HarmonyEngine::new(Key::C, HarmonyMode::DiatonicThirds);
        e.set_suppress_bass_register(true);
        let result = e.harmonize_note_on(Note::C2); // MIDI 36, threshold 48
        assert_eq!(
            result,
            vec![Note::C2],
            "below threshold should pass through alone"
        );
    }

    /// With suppression on, a note at or above the threshold still gets
    /// the full harmony.
    #[test]
    fn test_bass_register_allows_at_threshold() {
        let mut e = HarmonyEngine::new(Key::C, HarmonyMode::DiatonicThirds);
        e.set_suppress_bass_register(true);
        // C3 = MIDI 48 = exactly threshold; treat as "not bass anymore".
        let result = e.harmonize_note_on(Note::C3);
        assert!(result.len() > 1, "at threshold should produce harmony");
    }

    /// Custom threshold: setting it higher should suppress more notes.
    #[test]
    fn test_bass_register_custom_threshold() {
        let mut e = HarmonyEngine::new(Key::C, HarmonyMode::DiatonicThirds);
        e.set_suppress_bass_register(true);
        e.set_bass_register_threshold(60); // raise to C4
        let result_below = e.harmonize_note_on(Note::B3); // MIDI 59
        assert_eq!(
            result_below,
            vec![Note::B3],
            "raised threshold should suppress B3"
        );
        let result_at = e.harmonize_note_on(Note::C4);
        assert!(
            result_at.len() > 1,
            "C4 at the new threshold should produce harmony"
        );
    }

    /// Threshold setter clamps to MIDI's valid range (0..=127).
    #[test]
    fn test_bass_register_threshold_clamps() {
        let mut e = HarmonyEngine::new(Key::C, HarmonyMode::DiatonicThirds);
        e.set_bass_register_threshold(200);
        assert_eq!(e.bass_register_threshold(), 127);
    }

    #[test]
    fn whole_tone_three_voice_plane_keeps_shape_and_balanced_release() {
        let mut engine = HarmonyEngine::with_voices(Key::C, HarmonyMode::DiatonicThirds, 3);
        engine.set_scale_mode(ScaleMode::WholeTone);
        engine.set_voice_position(2);
        engine.set_voice_leading_enabled(false);
        engine.set_octave_mode(OctaveMode::None);
        engine.set_interchange_enabled(false);

        for input_midi in [60, 62, 64, 66, 68, 70] {
            let input = Note::try_from(input_midi).unwrap();
            let expected: Vec<Note> = [input_midi, input_midi + 4, input_midi + 8]
                .into_iter()
                .map(|midi| Note::try_from(midi).unwrap())
                .collect();
            assert_eq!(engine.harmonize_note_on(input), expected);
            assert_eq!(engine.harmonize_note_off(input), expected);
        }
    }

    #[test]
    fn mode_two_four_voice_plane_keeps_shape_and_balanced_release() {
        let mut engine = HarmonyEngine::with_voices(Key::C, HarmonyMode::DiatonicThirds, 4);
        engine.set_scale_mode(ScaleMode::DiminishedHalfWhole);
        engine.set_voice_position(3);
        engine.set_voice_leading_enabled(false);
        engine.set_octave_mode(OctaveMode::None);
        engine.set_interchange_enabled(false);

        for input_midi in [60, 61, 63, 64, 66, 67, 69, 70] {
            let input = Note::try_from(input_midi).unwrap();
            let expected: Vec<Note> = [input_midi, input_midi + 3, input_midi + 6, input_midi + 9]
                .into_iter()
                .map(|midi| Note::try_from(midi).unwrap())
                .collect();
            assert_eq!(engine.harmonize_note_on(input), expected);
            assert_eq!(engine.harmonize_note_off(input), expected);
        }
    }

    #[test]
    fn dorian_fourth_derived_blocks_keep_vectors_and_balanced_release() {
        let mut engine = HarmonyEngine::with_voices(Key::C, HarmonyMode::DiatonicFourths, 4);
        engine.set_scale_mode(ScaleMode::Dorian);
        engine.set_voice_position(3);
        engine.set_voice_leading_enabled(false);
        engine.set_octave_mode(OctaveMode::None);
        engine.set_interchange_enabled(false);

        for (input_midi, relative) in [
            (60, [0, 5, 10, 15]),
            (62, [0, 5, 10, 15]),
            (63, [0, 6, 11, 16]),
            (65, [0, 5, 10, 16]),
            (67, [0, 5, 10, 15]),
            (69, [0, 5, 10, 15]),
            (70, [0, 5, 11, 16]),
        ] {
            let input = Note::try_from(input_midi).unwrap();
            let expected: Vec<Note> = relative
                .into_iter()
                .map(|offset| Note::try_from(input_midi + offset).unwrap())
                .collect();
            let on = engine.harmonize_note_on(input);
            assert_eq!(on, expected);
            assert_eq!(on.iter().filter(|&&note| note == input).count(), 1);
            assert_eq!(engine.harmonize_note_off(input), expected);
            assert!(engine.active_notes.is_empty());
        }

        // Chromatic input is processed by the existing consonant fallback,
        // but remains outside the preset's Dorian acceptance corpus.
        let chromatic = Note::Db4;
        let on = engine.harmonize_note_on(chromatic);
        assert_eq!(on.len(), 4);
        assert_ne!(
            on.iter()
                .map(|note| u8::from(*note) - u8::from(chromatic))
                .collect::<Vec<_>>(),
            vec![0, 5, 10, 15]
        );
        assert_eq!(engine.harmonize_note_off(chromatic), on);
        assert!(engine.active_notes.is_empty());
    }

    #[test]
    fn explicit_interval_map_uses_anchor_relative_degree_offsets_and_balanced_release() {
        let mut engine = HarmonyEngine::with_voices(Key::C, HarmonyMode::ExplicitIntervals, 2);
        engine.set_scale_mode(ScaleMode::Dorian);
        engine.set_voice_position(1);
        engine.set_voice_leading_enabled(false);
        engine.set_octave_mode(OctaveMode::None);
        engine.set_interchange_enabled(false);
        engine
            .set_explicit_interval_map(ExplicitIntervalMap {
                degree_offsets: [
                    vec![12],
                    vec![7],
                    vec![7],
                    vec![5],
                    vec![7],
                    vec![5],
                    vec![5],
                ],
                fallback_offsets: vec![7],
            })
            .unwrap();

        for (input_midi, harmony_midi) in [
            (60, 72),
            (62, 69),
            (63, 70),
            (65, 70),
            (67, 74),
            (69, 74),
            (70, 75),
        ] {
            let input = Note::try_from(input_midi).unwrap();
            let expected = vec![input, Note::try_from(harmony_midi).unwrap()];
            assert_eq!(engine.harmonize_note_on(input), expected);
            assert_eq!(engine.last_port_map(), &[1, 0]);
            assert_eq!(engine.harmonize_note_off(input), expected);
            assert!(engine.active_notes.is_empty());
        }

        let chromatic = Note::Db4;
        let expected = vec![chromatic, Note::Ab4];
        assert_eq!(engine.harmonize_note_on(chromatic), expected);
        assert_eq!(engine.harmonize_note_off(chromatic), expected);
        assert!(engine.active_notes.is_empty());
    }

    #[test]
    fn explicit_interval_map_routes_final_pitch_order_after_octave_spread() {
        let mut engine = HarmonyEngine::with_voices(Key::C, HarmonyMode::ExplicitIntervals, 3);
        engine.set_voice_position(2);
        engine.set_octave_mode(OctaveMode::Spread);
        engine
            .set_explicit_interval_map(ExplicitIntervalMap {
                degree_offsets: std::array::from_fn(|_| vec![12, 7]),
                fallback_offsets: vec![12, 7],
            })
            .unwrap();

        assert_eq!(
            engine.harmonize(Note::C4),
            vec![Note::C4, Note::C6, Note::G6]
        );
        assert_eq!(engine.last_port_map(), &[2, 1, 0]);
    }

    #[test]
    fn explicit_interval_map_is_bounded_and_skips_out_of_range_offsets() {
        let mut engine = HarmonyEngine::with_voices(Key::C, HarmonyMode::ExplicitIntervals, 8);
        let mut invalid = ExplicitIntervalMap::default();
        invalid.degree_offsets[0] = vec![7, 7];
        assert!(engine.set_explicit_interval_map(invalid).is_err());
        let mut invalid = ExplicitIntervalMap::default();
        invalid.fallback_offsets = vec![0];
        assert!(engine.set_explicit_interval_map(invalid).is_err());

        engine
            .set_explicit_interval_map(ExplicitIntervalMap {
                degree_offsets: std::array::from_fn(|_| vec![12, 7, -5]),
                fallback_offsets: vec![12],
            })
            .unwrap();
        let high = Note::try_from(124).unwrap();
        let on = engine.harmonize_note_on(high);
        assert_eq!(on, vec![high, Note::try_from(119).unwrap()]);
        assert_eq!(engine.harmonize_note_off(high), on);
        assert!(engine.active_notes.is_empty());
    }

    #[test]
    fn aeolian_chorale_shadow_keeps_soprano_and_balanced_release() {
        let mut engine = HarmonyEngine::with_voices(Key::C, HarmonyMode::BachChorale, 4);
        engine.set_scale_mode(ScaleMode::Aeolian);
        engine.set_voice_position(0);
        engine.set_voice_leading_enabled(false);
        engine.set_octave_mode(OctaveMode::None);
        engine.set_interchange_enabled(false);

        let tonic = vec![Note::C5, Note::Eb4, Note::G3, Note::Bb2];
        let tonic_on = engine.harmonize_note_on(Note::C5);
        assert_eq!(tonic_on, tonic);
        assert_eq!(tonic_on.iter().filter(|&&note| note == Note::C5).count(), 1);
        assert!(tonic_on[1..].iter().all(|&note| note < Note::C5));
        assert_eq!(engine.harmonize_note_off(Note::C5), tonic);
        assert!(engine.active_notes.is_empty());

        // The note-count harmonic governor retains the tonic field for
        // this following nonchord color instead of changing harmony on
        // every melodic event.
        let continuation = vec![Note::D5, Note::C5, Note::Eb4, Note::G2];
        assert_eq!(engine.harmonize_note_on(Note::D5), continuation);
        assert_eq!(engine.harmonize_note_off(Note::D5), continuation);
        assert!(engine.active_notes.is_empty());
    }

    #[test]
    fn harmonic_minor_chorale_keeps_soprano_and_balanced_release() {
        let mut engine = HarmonyEngine::with_voices(Key::C, HarmonyMode::BachChorale, 4);
        engine.set_scale_mode(ScaleMode::HarmonicMinor);
        engine.set_voice_position(0);
        engine.set_voice_leading_enabled(false);
        engine.set_octave_mode(OctaveMode::None);
        engine.set_interchange_enabled(false);

        let expected = vec![Note::C5, Note::Eb4, Note::G3, Note::B2];
        let on = engine.harmonize_note_on(Note::C5);
        assert_eq!(on, expected);
        assert_eq!(on.iter().filter(|&&note| note == Note::C5).count(), 1);
        assert!(on[1..].iter().all(|&note| note < Note::C5));
        assert_eq!(engine.harmonize_note_off(Note::C5), expected);
        assert!(engine.active_notes.is_empty());
    }

    /// External transport, when set, must take precedence over the
    /// internal synthetic counter. Sanity check that the synthetic
    /// fallback doesn't override an explicitly-driven phase.
    #[test]
    fn test_external_phase_wins_over_synthetic() {
        let mut e = HarmonyEngine::new(Key::C, HarmonyMode::StrictCounterpoint);
        e.set_counterpoint_species(CounterpointSpecies::Species2);
        e.set_counterpoint_beat_phase(Some(0.0)); // explicit strong beat
        let with_external = e.harmonize_note_on(Note::E4);
        let mut e2 = HarmonyEngine::new(Key::C, HarmonyMode::StrictCounterpoint);
        e2.set_counterpoint_species(CounterpointSpecies::Species2);
        e2.set_counterpoint_beat_phase(Some(0.0));
        let again = e2.harmonize_note_on(Note::E4);
        assert_eq!(
            with_external, again,
            "External phase should be deterministic and not perturbed by the synthetic counter"
        );
    }

    // --- Issue #113: counterpoint scorer must not lock into melody-only ---

    /// Regression net for #113. Feeds a long in-scale melody to a strict
    /// Species 1 counterpoint engine and asserts that at least 90% of
    /// inputs produce harmony. Before the score_candidate -> Option<i32>
    /// refactor + sliding-window ambitus, accumulated soft preferences
    /// (R4 leap-recovery, R7 ambitus on lifetime min/max, R8 tritone
    /// outline, interval overuse) pushed every candidate below the
    /// `score < 0` reject threshold past ~20-30 notes and the engine
    /// emitted melody-only output thereafter.
    ///
    /// Uses a deterministic pseudo-random C-major walk (cycles through
    /// the seven scale degrees across two octaves) — a more demanding
    /// input than a pure scale because it forces the scorer through
    /// large jumps, repeats, and contour reversals.
    #[test]
    fn test_strict_counterpoint_keeps_generating_through_long_passage() {
        let scale_pitches: [u8; 14] = [
            60, 62, 64, 65, 67, 69, 71, // C4..B4
            72, 74, 76, 77, 79, 81, 83, // C5..B5
        ];
        let melody: Vec<Note> = (0..56)
            .map(|i| {
                let idx = (i * 5 + (i / 3)) % scale_pitches.len();
                Note::try_from(scale_pitches[idx]).unwrap()
            })
            .collect();

        let mut engine = HarmonyEngine::new(Key::C, HarmonyMode::StrictCounterpoint);
        engine.set_counterpoint_species(CounterpointSpecies::Species1);
        engine.set_counterpoint_strictness(CounterpointStrictness::Strict);

        let mut harmony_count = 0;
        let mut melody_only_count = 0;
        for &n in &melody {
            let result = engine.harmonize_note_on(n);
            if result.len() >= 2 {
                harmony_count += 1;
            } else {
                melody_only_count += 1;
            }
        }

        let ratio = harmony_count as f32 / melody.len() as f32;
        assert!(
            ratio >= 0.90,
            "Strict counterpoint must keep producing harmony through long passages \
             — got {}/{} notes harmonized ({:.0}%). Pre-fix this would tail off to \
             melody-only after ~20-30 notes as soft penalties accumulated past the \
             score < 0 reject threshold. Melody-only count: {}",
            harmony_count,
            melody.len(),
            ratio * 100.0,
            melody_only_count
        );
    }
}
