//! Main harmony engine that routes notes through mode-specific algorithms.

use std::collections::HashMap;
use wmidi::Note;

use crate::harmony::config::{Key, HarmonyMode};
use crate::harmony::modes;
use crate::harmony::scale::Scale;
use crate::harmony::stateful::{ContraryMotionState, CounterpointState};

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
    scale: Scale,
    /// Number of output voices (1 = melody only, 2 = melody + harmony, etc.)
    voice_count: usize,
    // Stateful mode state - one per voice pair for chained harmonies
    contrary_motion_states: Vec<ContraryMotionState>,
    counterpoint_states: Vec<CounterpointState>,
    /// Tracks active notes: melody MIDI number -> harmony notes produced.
    /// Used to ensure Note-Off releases the same harmony notes that
    /// Note-On created (critical for random modes).
    active_notes: HashMap<u8, Vec<Note>>,
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
        let scale = Scale::major(key.semitones_from_c());
        let voice_count = voice_count.max(1); // At least 1 voice
        let harmony_voices = if voice_count > 1 { voice_count - 1 } else { 0 };

        Self {
            key,
            mode,
            scale,
            voice_count,
            contrary_motion_states: (0..harmony_voices)
                .map(|_| ContraryMotionState::new())
                .collect(),
            counterpoint_states: (0..harmony_voices)
                .map(|_| CounterpointState::new())
                .collect(),
            active_notes: HashMap::new(),
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

    /// Returns the current voice count.
    pub fn voice_count(&self) -> usize {
        self.voice_count
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
        let harmony_voices = if count > 1 { count - 1 } else { 0 };

        // Rebuild state vectors
        self.contrary_motion_states = (0..harmony_voices)
            .map(|_| ContraryMotionState::new())
            .collect();
        self.counterpoint_states = (0..harmony_voices)
            .map(|_| CounterpointState::new())
            .collect();
        self.active_notes.clear();
    }

    /// Sets the musical key, rebuilding the scale.
    /// Resets stateful mode state and active notes since scale changes.
    ///
    /// This can be called during playback without stopping.
    pub fn set_key(&mut self, key: Key) {
        self.key = key;
        self.scale = Scale::major(key.semitones_from_c());
        // Reset stateful modes since scale changed
        for state in &mut self.contrary_motion_states {
            state.reset();
        }
        for state in &mut self.counterpoint_states {
            state.reset();
        }
        // Clear note tracking since harmonies would change with new scale
        self.active_notes.clear();
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
    }

    /// Harmonizes a single note based on the current mode.
    ///
    /// Returns a Vec containing:
    /// - For Mode 1: Just the input note
    /// - For Modes 2-7: Input note + chained harmony notes
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
            return vec![note];
        }

        let mut result = vec![note];
        let mut current = note;

        // Generate chained harmonies: each derived from the previous
        for i in 0..(self.voice_count - 1) {
            let harmony_result = self.harmonize_single(current, i);

            // Extract the harmony note (second element if present)
            if harmony_result.len() > 1 {
                current = harmony_result[1];
                result.push(current);
            } else {
                // No harmony produced, stop chain
                break;
            }
        }

        result
    }

    /// Harmonizes a single note using the mode's algorithm with the given state index.
    /// Used internally for chained harmony generation.
    fn harmonize_single(&mut self, note: Note, state_index: usize) -> Vec<Note> {
        match self.mode {
            HarmonyMode::PassThrough => modes::pass_through(note, &self.scale),
            HarmonyMode::DiatonicThirds => modes::diatonic_thirds(note, &self.scale),
            HarmonyMode::DiatonicFourths => modes::diatonic_fourths(note, &self.scale),
            HarmonyMode::RandomBelow => modes::random_below(note, &self.scale),
            HarmonyMode::RandomBelowNoSeconds => modes::random_below_no_seconds(note, &self.scale),
            HarmonyMode::ContraryMotion => {
                if let Some(state) = self.contrary_motion_states.get_mut(state_index) {
                    state.process(&self.scale, note)
                } else {
                    vec![note]
                }
            }
            HarmonyMode::StrictCounterpoint => {
                if let Some(state) = self.counterpoint_states.get_mut(state_index) {
                    state.process(&self.scale, note)
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
        // Store the harmony notes (all notes after the first) for Note-Off retrieval
        if result.len() > 1 {
            self.active_notes.insert(u8::from(note), result[1..].to_vec());
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

        // First note should produce harmony (third below)
        let result = engine.harmonize(Note::E4);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], Note::E4);
        assert_eq!(result[1], Note::C4);  // Third below E = C
    }

    #[test]
    fn test_counterpoint_mode() {
        let mut engine = HarmonyEngine::new(Key::C, HarmonyMode::StrictCounterpoint);

        // Should produce consonant harmony (third preferred)
        let result = engine.harmonize(Note::C4);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], Note::C4);
        assert_eq!(result[1], Note::A3);  // Third below C = A
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
        let result = engine.harmonize(Note::G4);
        assert_eq!(result.len(), 2);
        // First note in contrary motion gets third below
        assert_eq!(result[1], Note::E4);  // G - 2 degrees in G major = E
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
}
