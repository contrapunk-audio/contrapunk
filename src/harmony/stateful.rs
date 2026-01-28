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
    /// Direction to move when melody repeats (alternates)
    repeat_direction: i8,
}

impl ContraryMotionState {
    /// Creates a new ContraryMotionState with no history.
    pub fn new() -> Self {
        Self {
            last_melody: None,
            last_harmony: None,
            repeat_direction: -1, // Start by moving down
        }
    }

    /// Resets the state (e.g., when changing modes or keys).
    pub fn reset(&mut self) {
        self.last_melody = None;
        self.last_harmony = None;
        self.repeat_direction = -1;
    }

    /// Processes a note with contrary motion.
    ///
    /// - First note: harmony starts a third below the melody
    /// - Subsequent notes: harmony moves opposite to melody direction
    /// - When melody repeats: harmony alternates direction (oblique motion)
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
                    // Melody repeated: harmony moves in alternating direction (oblique motion)
                    let result = scale.transpose_diatonic(scale.snap_to_scale(last_harm), self.repeat_direction);
                    // Alternate direction for next repeat
                    self.repeat_direction = -self.repeat_direction;
                    result
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
/// Implements proper voice-leading rules:
/// - Prefers stepwise motion in the harmony voice
/// - Avoids repeating the same harmony note
/// - When melody repeats, harmony MUST move
/// - Avoids parallel fifths and octaves
/// - Varies interval types for musical interest
#[derive(Debug, Default)]
pub struct CounterpointState {
    last_melody: Option<Note>,
    last_harmony: Option<Note>,
    /// Tracks the last interval type used (in scale degrees) for variety
    last_interval: Option<i8>,
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
        self.last_interval = None;
    }

    /// Processes a note with strict counterpoint rules.
    ///
    /// Uses voice-leading scoring:
    /// - Prefers stepwise motion from previous harmony
    /// - Avoids repeating the same harmony note
    /// - Avoids repeating the same interval type
    /// - Rejects parallel perfect intervals (fifths, octaves)
    /// - When melody repeats, harmony must move
    ///
    /// Returns [melody, harmony] or [melody] if no valid harmony found.
    pub fn process(&mut self, scale: &Scale, melody: Note) -> Vec<Note> {
        let snapped = scale.snap_to_scale(melody);

        // Consonant intervals: 3rds and 6ths (below and above)
        let candidate_intervals: [i8; 8] = [-2, -5, 2, 5, -3, -4, 3, 4];

        // Score each candidate and pick the best
        let mut best_candidate: Option<(Note, i32)> = None;

        for &interval in &candidate_intervals {
            if let Some(candidate) = scale.transpose_diatonic(snapped, interval) {
                let score = self.score_candidate(melody, candidate, interval);

                // Skip if score is negative (hard constraint violated)
                if score < 0 {
                    continue;
                }

                match best_candidate {
                    None => best_candidate = Some((candidate, score)),
                    Some((_, best_score)) if score > best_score => {
                        best_candidate = Some((candidate, score));
                    }
                    _ => {}
                }
            }
        }

        self.last_melody = Some(melody);

        match best_candidate {
            Some((harmony, _)) => {
                // Calculate and store the interval used
                let harmony_midi = u8::from(harmony) as i8;
                let melody_midi = u8::from(snapped) as i8;
                let interval_semitones = harmony_midi - melody_midi;
                self.last_interval = Some(self.semitones_to_interval_class(interval_semitones));

                self.last_harmony = Some(harmony);
                vec![melody, harmony]
            }
            None => {
                self.last_harmony = None;
                self.last_interval = None;
                vec![melody]
            }
        }
    }

    /// Scores a harmony candidate based on voice-leading principles.
    /// Returns negative score if hard constraint violated.
    fn score_candidate(&self, melody: Note, candidate: Note, interval: i8) -> i32 {
        let mut score: i32 = 0;

        // Hard constraint: avoid parallel perfect intervals
        if let (Some(prev_m), Some(prev_h)) = (self.last_melody, self.last_harmony) {
            let prev_interval = self.interval_class(prev_m, prev_h);
            let new_interval = self.interval_class(melody, candidate);

            if self.is_perfect_interval(prev_interval) && prev_interval == new_interval {
                return -100; // Hard reject
            }
        }

        // Hard constraint: when melody repeats, harmony MUST move
        if let (Some(prev_m), Some(prev_h)) = (self.last_melody, self.last_harmony) {
            let melody_repeated = u8::from(melody) == u8::from(prev_m);
            let harmony_same = u8::from(candidate) == u8::from(prev_h);

            if melody_repeated && harmony_same {
                return -100; // Hard reject - no static voice when melody repeats
            }
        }

        // Soft preference: avoid repeating the same harmony note
        if let Some(prev_h) = self.last_harmony {
            if u8::from(candidate) != u8::from(prev_h) {
                score += 3; // Bonus for different note
            }
        }

        // Soft preference: stepwise motion in harmony voice
        if let Some(prev_h) = self.last_harmony {
            let prev_midi = u8::from(prev_h) as i32;
            let cand_midi = u8::from(candidate) as i32;
            let step_size = (cand_midi - prev_midi).abs();

            match step_size {
                1 | 2 => score += 4,  // Stepwise (semitone or whole tone)
                3 | 4 => score += 2,  // Small leap (minor/major 3rd)
                _ => score += 0,       // Larger leaps get no bonus
            }
        }

        // Soft preference: vary the interval type
        if let Some(last_int) = self.last_interval {
            let current_int = self.semitones_to_interval_class(
                u8::from(candidate) as i8 - u8::from(melody) as i8
            );
            if current_int != last_int {
                score += 2; // Bonus for different interval type
            }
        }

        // Slight preference for 3rds and 6ths over 4ths and 5ths
        let abs_interval = interval.abs();
        if abs_interval == 2 || abs_interval == 5 {
            score += 1; // 3rds and 6ths
        }

        score
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

    /// Converts semitone difference to interval class (3rd, 6th, etc.)
    fn semitones_to_interval_class(&self, semitones: i8) -> i8 {
        // Normalize to positive interval
        let normalized = semitones.abs() % 12;
        match normalized {
            0 => 0,       // Unison
            1 | 2 => 2,   // 2nd
            3 | 4 => 3,   // 3rd
            5 => 4,       // 4th
            7 => 5,       // 5th
            8 | 9 => 6,   // 6th
            10 | 11 => 7, // 7th
            6 => 4,       // Tritone (treat as 4th)
            _ => 0,
        }
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
    fn test_contrary_motion_melody_repeats() {
        let scale = Scale::major(0);
        let mut state = ContraryMotionState::new();

        // First note: C4, harmony A3
        let result1 = state.process(&scale, Note::C4);
        let harmony1 = result1[1];

        // Melody repeats: harmony should MOVE, not stay
        let result2 = state.process(&scale, Note::C4);
        let harmony2 = result2[1];
        assert_ne!(u8::from(harmony1), u8::from(harmony2),
            "Harmony should move when melody repeats");

        // Third repeat: harmony should move again (opposite direction)
        let result3 = state.process(&scale, Note::C4);
        let harmony3 = result3[1];
        assert_ne!(u8::from(harmony2), u8::from(harmony3),
            "Harmony should continue moving on repeated melody");
    }

    #[test]
    fn test_counterpoint_avoids_parallel_fifths() {
        let scale = Scale::major(0);
        let mut state = CounterpointState::new();

        state.last_melody = Some(Note::C4);
        state.last_harmony = Some(Note::F3);  // Perfect 5th below

        let result = state.process(&scale, Note::D4);
        assert_eq!(result[0], Note::D4);

        if result.len() > 1 {
            let harmony_midi: u8 = result[1].into();
            let melody_midi: u8 = Note::D4.into();
            let interval = (melody_midi as i8 - harmony_midi as i8).unsigned_abs() % 12;
            assert_ne!(interval, 7, "Should not produce parallel fifth");
        }
    }

    #[test]
    fn test_counterpoint_first_note() {
        let scale = Scale::major(0);
        let mut state = CounterpointState::new();

        let result = state.process(&scale, Note::C4);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], Note::C4);
        // Should get a consonant harmony (3rd or 6th preferred)
    }

    #[test]
    fn test_counterpoint_melody_repeats_harmony_moves() {
        let scale = Scale::major(0);
        let mut state = CounterpointState::new();

        // First note
        let result1 = state.process(&scale, Note::C4);
        assert_eq!(result1.len(), 2);
        let harmony1 = result1[1];

        // Same melody note: harmony MUST change
        let result2 = state.process(&scale, Note::C4);
        assert_eq!(result2.len(), 2);
        let harmony2 = result2[1];

        assert_ne!(u8::from(harmony1), u8::from(harmony2),
            "Harmony must move when melody repeats");
    }

    #[test]
    fn test_counterpoint_prefers_stepwise_motion() {
        let scale = Scale::major(0);
        let mut state = CounterpointState::new();

        // Play several notes and check harmony moves smoothly
        let result1 = state.process(&scale, Note::C4);
        let h1 = u8::from(result1[1]) as i32;

        let result2 = state.process(&scale, Note::D4);
        let h2 = u8::from(result2[1]) as i32;

        // Harmony should move by a small interval (stepwise preferred)
        let step = (h2 - h1).abs();
        assert!(step <= 4, "Harmony should prefer stepwise motion, got step of {}", step);
    }

    #[test]
    fn test_counterpoint_varies_intervals() {
        let scale = Scale::major(0);
        let mut state = CounterpointState::new();

        // Play a repeated note several times
        let mut harmonies = Vec::new();
        for _ in 0..4 {
            let result = state.process(&scale, Note::C4);
            harmonies.push(u8::from(result[1]));
        }

        // Check that we got at least 2 different harmony notes
        harmonies.sort();
        harmonies.dedup();
        assert!(harmonies.len() >= 2,
            "Counterpoint should vary harmonies on repeated notes");
    }
}
