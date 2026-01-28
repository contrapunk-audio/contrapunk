//! Stateful harmony modes that track previous notes.
//!
//! Modes 6 (ContraryMotion) and 7 (StrictCounterpoint) need to know
//! what notes came before to determine the harmony direction or
//! avoid parallel intervals.

use wmidi::Note;

use crate::harmony::Scale;

/// State for Mode 6: Contrary Motion
///
/// Tracks the previous melody and harmony notes to move
/// the harmony in the opposite direction from the melody.
#[derive(Debug, Default)]
pub struct ContraryMotionState {
    last_melody: Option<Note>,
    last_harmony: Option<Note>,
}

impl ContraryMotionState {
    /// Creates a new ContraryMotionState with no history.
    pub fn new() -> Self {
        Self::default()
    }

    /// Resets the state (e.g., when changing modes or keys).
    pub fn reset(&mut self) {
        self.last_melody = None;
        self.last_harmony = None;
    }

    /// Processes a note with contrary motion.
    ///
    /// - First note: harmony starts a third below the melody
    /// - Subsequent notes: harmony moves opposite to melody direction
    ///
    /// Returns [melody, harmony] or [melody] if harmony out of range.
    pub fn process(&mut self, scale: &Scale, melody: Note) -> Vec<Note> {
        let harmony = match self.last_melody {
            None => {
                // First note: start harmony a third below
                scale.transpose_diatonic(scale.snap_to_scale(melody), -2)
            }
            Some(prev_melody) => {
                let melody_midi = u8::from(melody) as i8;
                let prev_midi = u8::from(prev_melody) as i8;
                let direction = melody_midi - prev_midi;

                let last_harm = self.last_harmony.unwrap_or(melody);

                if direction > 0 {
                    // Melody went up, harmony goes down
                    scale.transpose_diatonic(scale.snap_to_scale(last_harm), -1)
                } else if direction < 0 {
                    // Melody went down, harmony goes up
                    scale.transpose_diatonic(scale.snap_to_scale(last_harm), 1)
                } else {
                    // Melody repeated, harmony stays (return as Some)
                    Some(last_harm)
                }
            }
        };

        self.last_melody = Some(melody);

        match harmony {
            Some(h) => {
                self.last_harmony = Some(h);
                vec![melody, h]
            }
            None => {
                // Harmony out of range, just pass through
                vec![melody]
            }
        }
    }
}

/// State for Mode 7: Strict Counterpoint
///
/// Tracks previous intervals to avoid parallel fifths and octaves,
/// which are forbidden in traditional counterpoint.
#[derive(Debug, Default)]
pub struct CounterpointState {
    last_melody: Option<Note>,
    last_harmony: Option<Note>,
}

impl CounterpointState {
    /// Creates a new CounterpointState with no history.
    pub fn new() -> Self {
        Self::default()
    }

    /// Resets the state.
    pub fn reset(&mut self) {
        self.last_melody = None;
        self.last_harmony = None;
    }

    /// Processes a note with strict counterpoint rules.
    ///
    /// Tries intervals in order of preference (3rd, 6th, 4th, 5th),
    /// rejecting any that would create parallel fifths or octaves
    /// with the previous interval.
    ///
    /// Returns [melody, harmony] or [melody] if no valid harmony found.
    pub fn process(&mut self, scale: &Scale, melody: Note) -> Vec<Note> {
        let snapped = scale.snap_to_scale(melody);

        // Preferred intervals: 3rds and 6ths first (consonant), then 4ths/5ths
        // Negative = below the melody
        let preferred_intervals: [i8; 8] = [-2, -5, -3, -4, 2, 5, 3, 4];

        let harmony = preferred_intervals.iter()
            .filter_map(|&interval| {
                let candidate = scale.transpose_diatonic(snapped, interval)?;

                // Check for parallel perfect intervals with previous
                if let (Some(prev_m), Some(prev_h)) = (self.last_melody, self.last_harmony) {
                    let prev_interval = self.interval_class(prev_m, prev_h);
                    let new_interval = self.interval_class(melody, candidate);

                    // Reject parallel unisons, fifths, or octaves
                    if self.is_perfect_interval(prev_interval)
                        && prev_interval == new_interval
                    {
                        return None;
                    }
                }

                Some(candidate)
            })
            .next();

        self.last_melody = Some(melody);

        match harmony {
            Some(h) => {
                self.last_harmony = Some(h);
                vec![melody, h]
            }
            None => {
                // No valid counterpoint found, pass through
                self.last_harmony = None;
                vec![melody]
            }
        }
    }

    /// Returns the interval class (0-11) between two notes.
    fn interval_class(&self, a: Note, b: Note) -> u8 {
        let a_midi = u8::from(a);
        let b_midi = u8::from(b);
        let diff = if a_midi > b_midi {
            a_midi - b_midi
        } else {
            b_midi - a_midi
        };
        diff % 12
    }

    /// Returns true if the interval class is a "perfect" interval
    /// (unison, fifth, or octave) that should not move in parallel.
    fn is_perfect_interval(&self, interval_class: u8) -> bool {
        matches!(interval_class, 0 | 7)  // Unison or perfect fifth
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contrary_motion_first_note() {
        let scale = Scale::major(0);  // C major
        let mut state = ContraryMotionState::new();

        // First note gets harmony a third below
        let result = state.process(&scale, Note::E4);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], Note::E4);
        assert_eq!(result[1], Note::C4);  // E - 2 degrees = C
    }

    #[test]
    fn test_contrary_motion_opposite_direction() {
        let scale = Scale::major(0);
        let mut state = ContraryMotionState::new();

        // First note: E4, harmony C4
        let _ = state.process(&scale, Note::E4);

        // Melody goes up to G4, harmony should go down from C4
        let result = state.process(&scale, Note::G4);
        assert_eq!(result[0], Note::G4);
        // Harmony should be B3 (C4 - 1 degree)
        assert_eq!(result[1], Note::B3);
    }

    #[test]
    fn test_counterpoint_avoids_parallel_fifths() {
        let scale = Scale::major(0);
        let mut state = CounterpointState::new();

        // Set up a fifth interval: C4 melody, F3 harmony (5 semitones = 4th, close to 5th)
        // Actually let's use G3 to get a perfect 5th
        state.last_melody = Some(Note::C4);
        state.last_harmony = Some(Note::F3);  // Perfect 5th below

        // Now if melody moves to D4, counterpoint should NOT give us G3
        // (which would be parallel fifths)
        let result = state.process(&scale, Note::D4);
        assert_eq!(result[0], Note::D4);

        // Harmony should NOT be G3 (which would be parallel 5th)
        // It should prefer a 3rd or 6th
        if result.len() > 1 {
            let harmony_midi: u8 = result[1].into();
            let melody_midi: u8 = Note::D4.into();
            let interval = (melody_midi as i8 - harmony_midi as i8).unsigned_abs() % 12;
            // Should not be a perfect 5th (7 semitones)
            assert_ne!(interval, 7);
        }
    }

    #[test]
    fn test_counterpoint_first_note() {
        let scale = Scale::major(0);
        let mut state = CounterpointState::new();

        // First note should get a consonant harmony (3rd preferred)
        let result = state.process(&scale, Note::C4);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], Note::C4);
        // First preferred interval is -2 (third below) = A3
        assert_eq!(result[1], Note::A3);
    }
}
