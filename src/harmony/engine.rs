//! Main harmony engine that routes notes through mode-specific algorithms.

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
#[derive(Debug)]
pub struct HarmonyEngine {
    key: Key,
    mode: HarmonyMode,
    scale: Scale,
    // Stateful mode state
    contrary_motion: ContraryMotionState,
    counterpoint: CounterpointState,
}

impl HarmonyEngine {
    /// Creates a new HarmonyEngine with the specified key and mode.
    pub fn new(key: Key, mode: HarmonyMode) -> Self {
        let scale = Scale::major(key.semitones_from_c());
        Self {
            key,
            mode,
            scale,
            contrary_motion: ContraryMotionState::new(),
            counterpoint: CounterpointState::new(),
        }
    }

    /// Returns the current key.
    pub fn key(&self) -> Key {
        self.key
    }

    /// Returns the current mode.
    pub fn mode(&self) -> HarmonyMode {
        self.mode
    }

    /// Sets the musical key, rebuilding the scale.
    /// Resets stateful mode state since scale changes.
    ///
    /// This can be called during playback without stopping.
    pub fn set_key(&mut self, key: Key) {
        self.key = key;
        self.scale = Scale::major(key.semitones_from_c());
        // Reset stateful modes since scale changed
        self.contrary_motion.reset();
        self.counterpoint.reset();
    }

    /// Sets the harmony mode.
    /// Resets stateful mode state when switching modes.
    ///
    /// This can be called during playback without stopping.
    pub fn set_mode(&mut self, mode: HarmonyMode) {
        self.mode = mode;
        // Reset stateful modes when switching
        self.contrary_motion.reset();
        self.counterpoint.reset();
    }

    /// Harmonizes a single note based on the current mode.
    ///
    /// Returns a Vec containing:
    /// - For Mode 1: Just the input note
    /// - For Modes 2-5: Input note + one harmony note
    /// - For Modes 6-7: Input note + one harmony note (stateful)
    ///
    /// The first element is always the original input note.
    /// Harmony notes follow in subsequent elements.
    pub fn harmonize(&mut self, note: Note) -> Vec<Note> {
        match self.mode {
            HarmonyMode::PassThrough => modes::pass_through(note, &self.scale),
            HarmonyMode::DiatonicThirds => modes::diatonic_thirds(note, &self.scale),
            HarmonyMode::DiatonicFourths => modes::diatonic_fourths(note, &self.scale),
            HarmonyMode::RandomBelow => modes::random_below(note, &self.scale),
            HarmonyMode::RandomBelowNoSeconds => modes::random_below_no_seconds(note, &self.scale),
            HarmonyMode::ContraryMotion => self.contrary_motion.process(&self.scale, note),
            HarmonyMode::StrictCounterpoint => self.counterpoint.process(&self.scale, note),
        }
    }
}

impl Default for HarmonyEngine {
    fn default() -> Self {
        Self::new(Key::C, HarmonyMode::PassThrough)
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
}
