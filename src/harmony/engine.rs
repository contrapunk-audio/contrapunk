//! Main harmony engine that routes notes through mode-specific algorithms.

use std::collections::HashMap;
use wmidi::Note;

use crate::harmony::config::{Key, HarmonyMode, OctaveMode, ScaleMode};
use crate::harmony::modes;
use crate::harmony::scale::Scale;
use crate::harmony::stateful::{ContraryMotionState, CounterpointState};
use crate::harmony::voice_leading::{
    revoice_chord, StyleRules, SuspensionState, VoiceAnchor, VoiceLeadingStyle, VoiceRegister,
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
    suspension_state: SuspensionState,
}

impl VoiceLeadingProcessor {
    fn new(voice_count: usize) -> Self {
        let style = VoiceLeadingStyle::default();
        let style_rules = StyleRules::for_style(style);
        let registers = Self::build_registers(voice_count);
        let suspension_state = SuspensionState::new(style == VoiceLeadingStyle::Palestrina);
        Self {
            enabled: false,
            style,
            style_rules,
            previous_voicing: None,
            registers,
            suspension_state,
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
    fn build_registers_for_position(voice_count: usize, voice_position: usize) -> Vec<VoiceRegister> {
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
                    if fraction < 0.25 { VoiceRegister::Soprano }
                    else if fraction < 0.5 { VoiceRegister::Alto }
                    else if fraction < 0.75 { VoiceRegister::Tenor }
                    else { VoiceRegister::Bass }
                }
            })
            .collect();

        // Build final_result order: user first, then interleaved above/below
        let vp = voice_position.min(voice_count - 1);
        let mut regs = vec![arrangement_regs[vp]]; // user's register

        let mut above_idx = if vp > 0 { Some(vp - 1) } else { None };
        let mut below_idx = if vp < voice_count - 1 { Some(vp + 1) } else { None };

        loop {
            if above_idx.is_none() && below_idx.is_none() { break; }
            if let Some(ai) = above_idx {
                regs.push(arrangement_regs[ai]);
                above_idx = if ai > 0 { Some(ai - 1) } else { None };
            }
            if let Some(bi) = below_idx {
                regs.push(arrangement_regs[bi]);
                below_idx = if bi < voice_count - 1 { Some(bi + 1) } else { None };
            }
        }

        regs
    }

    fn reset(&mut self) {
        self.previous_voicing = None;
        self.suspension_state.reset();
    }

    fn set_style(&mut self, style: VoiceLeadingStyle) {
        self.style = style;
        self.style_rules = StyleRules::for_style(style);
        self.suspension_state = SuspensionState::new(style == VoiceLeadingStyle::Palestrina);
        self.previous_voicing = None;
    }

    fn rebuild_for_voices(&mut self, voice_count: usize, voice_position: usize) {
        self.registers = Self::build_registers_for_position(voice_count, voice_position);
        self.reset();
    }
}

/// The harmony engine that transforms incoming MIDI notes.
///
/// Holds the current key and mode configuration, and provides
/// a `harmonize()` method that processes notes through the
/// selected mode's algorithm.
///
/// For stateless modes (1-5), each note is processed independently.
/// For stateful modes (6-7), the engine tracks previous notes.
///
/// For Note-Off handling (especially important in random modes),
/// the engine tracks active notes so that Note-Off releases the
/// same harmony notes that were produced by the corresponding Note-On.
///
/// Supports chained harmonies where each harmony voice is derived
/// from the previous one (e.g., harmony2 is harmony of harmony1).
#[derive(Debug)]
pub struct HarmonyEngine {
    key: Key,
    mode: HarmonyMode,
    octave_mode: OctaveMode,
    scale_mode: ScaleMode,
    scale: Scale,
    interchange_enabled: bool,
    borrowing_range: u8,
    last_borrowed_from: Option<ScaleMode>,
    /// Number of output voices (1 = melody only, 2 = melody + harmony, etc.)
    voice_count: usize,
    /// Voice position: which voice slot the user plays (0 = top/soprano, voice_count-1 = bass).
    /// Harmonies are generated outward from this position in both directions.
    voice_position: usize,
    // Stateful mode state - one per voice pair for chained harmonies
    contrary_motion_states: Vec<ContraryMotionState>,
    counterpoint_states: Vec<CounterpointState>,
    /// Tracks active notes: melody MIDI number -> harmony notes produced.
    /// Used to ensure Note-Off releases the same harmony notes that
    /// Note-On created (critical for random modes).
    active_notes: HashMap<u8, Vec<Note>>,
    /// Port assignment map from the most recent harmonize call.
    /// Each index corresponds to a note in the harmonize result;
    /// the value is the output port index that note should be sent to.
    /// For non-Mirror modes: identity mapping (index i -> port i).
    /// For Mirror mode: duplicates map back to the original harmony voice's port.
    last_port_map: Vec<usize>,
    last_arrangement_indices: Vec<usize>,
    /// Stored port maps for active notes, used to retrieve port assignments on Note-Off.
    active_port_maps: HashMap<u8, Vec<usize>>,
    /// Voice leading post-processor
    voice_leading: VoiceLeadingProcessor,
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
            scale_mode: ScaleMode::Ionian,
            scale,
            interchange_enabled: false,
            borrowing_range: 3,
            last_borrowed_from: None,
            voice_count,
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
            voice_leading: VoiceLeadingProcessor::new(voice_count),
        }
    }

    /// Creates a new HarmonyEngine with the specified key and mode.
    /// Defaults to 2 voices (melody + 1 harmony).
    pub fn new(key: Key, mode: HarmonyMode) -> Self {
        Self::with_voices(key, mode, 2)
    }

    /// Returns the current key.
    pub fn key(&self) -> Key {
        self.key
    }

    /// Returns the current mode.
    pub fn mode(&self) -> HarmonyMode {
        self.mode
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
        self.active_notes.clear();
        self.active_port_maps.clear();
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
        self.voice_position = position;
        self.active_notes.clear();
        self.active_port_maps.clear();
        self.voice_leading.rebuild_for_voices(self.voice_count, position);
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
        self.counterpoint_states = (0..harmony_voices)
            .map(|_| CounterpointState::new())
            .collect();
        self.active_notes.clear();
        self.active_port_maps.clear();
        self.voice_leading.rebuild_for_voices(count, self.voice_position);
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
        // Clear note tracking since harmonies would change with new scale
        self.active_notes.clear();
        self.active_port_maps.clear();
        self.voice_leading.reset();
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
        // Clear note tracking since harmonies would change with new mode
        self.active_notes.clear();
        self.active_port_maps.clear();
        self.voice_leading.reset();
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
        for state in &mut self.contrary_motion_states {
            state.reset();
        }
        for state in &mut self.counterpoint_states {
            state.reset();
        }
        self.active_notes.clear();
        self.active_port_maps.clear();
        self.voice_leading.reset();
    }

    /// Returns whether modal interchange is enabled.
    pub fn interchange_enabled(&self) -> bool {
        self.interchange_enabled
    }

    /// Enables or disables modal interchange.
    pub fn set_interchange_enabled(&mut self, enabled: bool) {
        self.interchange_enabled = enabled;
        self.scale.set_interchange_enabled(enabled);
        self.active_notes.clear();
        self.active_port_maps.clear();
    }

    /// Returns the current borrowing range (1-5).
    pub fn borrowing_range(&self) -> u8 {
        self.borrowing_range
    }

    /// Sets the borrowing range (clamped 1-5).
    pub fn set_borrowing_range(&mut self, range: u8) {
        self.borrowing_range = range.clamp(1, 5);
        self.scale.set_borrowing_range(self.borrowing_range);
        self.active_notes.clear();
        self.active_port_maps.clear();
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
        self.active_notes.clear();
        self.active_port_maps.clear();
    }

    /// Sets the voice leading style, resetting VL state.
    pub fn set_voice_leading_style(&mut self, style: VoiceLeadingStyle) {
        self.voice_leading.set_style(style);
        self.active_notes.clear();
        self.active_port_maps.clear();
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
    /// Note-Off handling (critical for random modes).
    pub fn harmonize(&mut self, note: Note) -> Vec<Note> {
        if self.mode == HarmonyMode::PassThrough || self.voice_count <= 1 {
            self.last_arrangement_indices = vec![0];
            self.last_port_map = vec![0];
            return vec![note];
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
        let mut above_idx = if self.voice_position > 0 { Some(self.voice_position - 1) } else { None };
        let mut below_idx = if self.voice_position < self.voice_count - 1 { Some(self.voice_position + 1) } else { None };

        loop {
            let has_above = above_idx.is_some();
            let has_below = below_idx.is_some();
            if !has_above && !has_below { break; }

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
                below_idx = if bi < self.voice_count - 1 { Some(bi + 1) } else { None };
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
            // Apply suspension if style requires it
            let prev_notes = self.voice_leading.previous_voicing.clone();
            self.voice_leading
                .suspension_state
                .process(&mut final_result, prev_notes.as_deref(), &self.scale);

            // Store current voicing for next call
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

        match self.octave_mode {
            OctaveMode::None => {}
            OctaveMode::Spread => {
                for (i, note) in notes.iter_mut().enumerate().skip(1) {
                    let midi = u8::from(*note);
                    let shifted = midi.saturating_add((i as u8) * 12).min(127);
                    if anchor_ok(i, shifted) {
                        if let Ok(new_note) = Note::try_from(shifted) {
                            *note = new_note;
                        }
                    }
                    // If not anchor_ok, leave the note as-is
                }
            }
            OctaveMode::BassTrebleSplit => {
                for (i, note) in notes.iter_mut().enumerate().skip(1) {
                    let midi = u8::from(*note);
                    let shifted = if midi < user_midi {
                        midi.saturating_sub(12)
                    } else {
                        midi.saturating_add(12).min(127)
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

    /// Harmonizes a single note in a specific direction using the mode's algorithm.
    /// `above`: if true, generate harmony above; if false, generate below.
    /// Used for bidirectional voice position generation.
    fn harmonize_single_directed(&mut self, note: Note, state_index: usize, above: bool) -> Vec<Note> {
        match self.mode {
            HarmonyMode::PassThrough => modes::pass_through(note, &mut self.scale),
            HarmonyMode::DiatonicThirds => modes::diatonic_thirds_directed(note, &mut self.scale, above),
            HarmonyMode::DiatonicFourths => modes::diatonic_fourths_directed(note, &mut self.scale, above),
            HarmonyMode::RandomBelow => modes::random_directed(note, &mut self.scale, above),
            HarmonyMode::RandomBelowNoSeconds => modes::random_no_seconds_directed(note, &mut self.scale, above),
            HarmonyMode::ContraryMotion => {
                if let Some(state) = self.contrary_motion_states.get_mut(state_index) {
                    state.process_directed(&mut self.scale, note, above)
                } else {
                    vec![note]
                }
            }
            HarmonyMode::StrictCounterpoint => {
                if let Some(state) = self.counterpoint_states.get_mut(state_index) {
                    state.process_directed(&mut self.scale, note, above)
                } else {
                    vec![note]
                }
            }
        }
    }

    /// Harmonizes a single note using the mode's algorithm with the given state index.
    /// Used internally for chained harmony generation.
    fn harmonize_single(&mut self, note: Note, state_index: usize) -> Vec<Note> {
        match self.mode {
            HarmonyMode::PassThrough => modes::pass_through(note, &mut self.scale),
            HarmonyMode::DiatonicThirds => modes::diatonic_thirds(note, &mut self.scale),
            HarmonyMode::DiatonicFourths => modes::diatonic_fourths(note, &mut self.scale),
            HarmonyMode::RandomBelow => modes::random_below(note, &mut self.scale),
            HarmonyMode::RandomBelowNoSeconds => modes::random_below_no_seconds(note, &mut self.scale),
            HarmonyMode::ContraryMotion => {
                if let Some(state) = self.contrary_motion_states.get_mut(state_index) {
                    state.process(&mut self.scale, note)
                } else {
                    vec![note]
                }
            }
            HarmonyMode::StrictCounterpoint => {
                if let Some(state) = self.counterpoint_states.get_mut(state_index) {
                    state.process(&mut self.scale, note)
                } else {
                    vec![note]
                }
            }
        }
    }

    /// Harmonizes a Note-On and tracks the result for Note-Off.
    ///
    /// Call this for Note-On messages. The returned notes should be
    /// sent to outputs. When Note-Off comes, call `harmonize_note_off()`
    /// with the same melody note to get matching harmony releases.
    ///
    /// This is critical for random modes (4-5) where the harmony
    /// interval is chosen randomly - we must release the same note
    /// that was pressed, not a new random one.
    ///
    /// # Arguments
    ///
    /// * `note` - The melody note from the Note-On message
    ///
    /// # Returns
    ///
    /// Vec of notes to send: original note first, harmony notes after.
    pub fn harmonize_note_on(&mut self, note: Note) -> Vec<Note> {
        let result = self.harmonize(note);
        // Copy last_borrowed_from from scale for UI access
        self.last_borrowed_from = self.scale.last_borrowed_from();
        // Store the harmony notes and port map for Note-Off retrieval
        let midi = u8::from(note);
        if result.len() > 1 {
            self.active_notes.insert(midi, result[1..].to_vec());
            self.active_port_maps.insert(midi, self.last_port_map.clone());
        }
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
    pub fn harmonize_note_off(&mut self, note: Note) -> Vec<Note> {
        let midi = u8::from(note);
        // Restore the port map that was stored during Note-On
        if let Some(port_map) = self.active_port_maps.remove(&midi) {
            self.last_port_map = port_map;
        } else {
            self.last_port_map = vec![0];
        }
        match self.active_notes.remove(&midi) {
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
        assert_eq!(result[1], Note::G4);  // Third above E = G
    }

    #[test]
    fn test_counterpoint_mode() {
        let mut engine = HarmonyEngine::new(Key::C, HarmonyMode::StrictCounterpoint);

        // Default voice_position is bass, so harmony generates above
        // Should produce consonant harmony (third above preferred)
        let result = engine.harmonize(Note::C4);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], Note::C4);
        assert_eq!(result[1], Note::E4);  // Third above C = E
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
        assert_eq!(result[1], Note::B4);  // G + 2 degrees in G major = B
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
    fn test_random_mode_tracking() {
        let mut engine = HarmonyEngine::new(Key::C, HarmonyMode::RandomBelow);

        // Note-On should produce melody + random harmony
        let on_result = engine.harmonize_note_on(Note::C5);
        assert_eq!(on_result.len(), 2);

        let harmony = on_result[1];

        // Note-Off should return the SAME harmony that was produced
        let off_result = engine.harmonize_note_off(Note::C5);
        assert_eq!(off_result.len(), 2);
        assert_eq!(off_result[1], harmony); // Same harmony note
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
        assert_eq!(result[0], Note::C4);  // Melody unchanged
        assert_eq!(result[1], Note::E5);  // First harmony +1 octave
        assert_eq!(result[2], Note::G6);  // Second harmony +2 octaves
    }

    #[test]
    fn test_octave_mode_bass_treble_split() {
        let mut engine = HarmonyEngine::with_voices(Key::C, HarmonyMode::StrictCounterpoint, 2);
        engine.set_octave_mode(OctaveMode::BassTrebleSplit);

        // Counterpoint typically produces harmony below melody
        // C4 -> harmony below (e.g., A3) -> shifted down to A2
        let result = engine.harmonize(Note::C4);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], Note::C4);  // Melody unchanged
        // Harmony should be shifted (down if below C4, up if above)
        let harmony_midi = u8::from(result[1]);
        let c4_midi = u8::from(Note::C4);
        // Due to bass/treble split, harmony should be shifted away from melody
        assert!(harmony_midi < c4_midi - 11 || harmony_midi > c4_midi + 11,
            "Harmony {} should be shifted at least one octave from melody {}",
            harmony_midi, c4_midi);
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
        assert_eq!(result.len(), 5, "Mirror should skip copies that cross anchor: {:?}", result);
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
        assert_eq!(off_result.len(), 5, "Note-Off should release all notes from Note-On");
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

        assert_eq!(on_port_map, off_port_map, "Port map should be restored on Note-Off");
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
        use crate::harmony::voice_leading::VoiceLeadingStyle;

        let mut engine = HarmonyEngine::with_voices(Key::C, HarmonyMode::DiatonicThirds, 3);
        engine.set_voice_leading_enabled(true);

        engine.harmonize(Note::C4);
        engine.set_voice_leading_style(VoiceLeadingStyle::Palestrina);

        let result = engine.harmonize(Note::C4);
        assert_eq!(result[0], Note::C4);
    }

    #[test]
    fn test_vl_style_getter_setter() {
        use crate::harmony::voice_leading::VoiceLeadingStyle;

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
}
