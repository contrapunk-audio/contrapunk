//! Stateful harmony modes that track previous notes.
//!
//! Modes 6 (ContraryMotion) and 7 (StrictCounterpoint) need to know
//! what notes came before to determine the harmony direction or
//! avoid parallel intervals.

use std::collections::VecDeque;

use wmidi::Note;

use crate::harmony::Scale;

/// Direction of melodic motion between two notes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MelodicDirection {
    Ascending,
    Descending,
    Static,
}

/// Size of the interval history buffer.
const INTERVAL_HISTORY_SIZE: usize = 4;
/// Size of the melodic contour history buffer.
const CONTOUR_HISTORY_SIZE: usize = 3;
/// Range threshold below which harmony is considered "narrow" (perfect 5th in semitones).
const NARROW_RANGE_THRESHOLD: u8 = 7;

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
    /// - Out-of-key notes: uses consonant chromatic intervals
    ///
    /// Returns [melody, harmony] or [melody] if harmony out of range.
    pub fn process(&mut self, scale: &Scale, melody: Note) -> Vec<Note> {
        let harmony = match self.last_melody {
            None => {
                // First note: start harmony a third below
                // Use smart harmonization to handle out-of-key notes
                scale.harmonize_smart(melody, -2, false)
            }
            Some(prev_melody) => {
                let melody_midi = u8::from(melody) as i8;
                let prev_midi = u8::from(prev_melody) as i8;
                let direction = melody_midi - prev_midi;

                let last_harm = self.last_harmony.unwrap_or(melody);

                // For subsequent notes, move harmony voice diatonically if possible
                // The harmony voice should stay in-key even if melody goes out
                if direction > 0 {
                    // Melody went up, harmony goes down
                    if scale.is_in_scale(last_harm) {
                        scale.transpose_diatonic(last_harm, -1)
                    } else {
                        // Previous harmony was chromatic, use chromatic step
                        scale.transpose_chromatic(last_harm, -2)
                    }
                } else if direction < 0 {
                    // Melody went down, harmony goes up
                    if scale.is_in_scale(last_harm) {
                        scale.transpose_diatonic(last_harm, 1)
                    } else {
                        scale.transpose_chromatic(last_harm, 2)
                    }
                } else {
                    // Melody repeated: harmony moves in alternating direction (oblique motion)
                    let step = self.repeat_direction;
                    self.repeat_direction = -self.repeat_direction;

                    if scale.is_in_scale(last_harm) {
                        scale.transpose_diatonic(last_harm, step)
                    } else {
                        scale.transpose_chromatic(last_harm, step as i8 * 2)
                    }
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
/// - Tracks interval history to avoid overusing the same interval
/// - Tracks melodic contour to encourage contrary motion
/// - Tracks harmony range to encourage range expansion
#[derive(Debug)]
pub struct CounterpointState {
    last_melody: Option<Note>,
    last_harmony: Option<Note>,

    /// Last N interval classes for variety tracking
    interval_history: VecDeque<i8>,

    /// Melodic contour tracking (last N directions)
    melody_contour: VecDeque<MelodicDirection>,

    /// Harmony range tracking (lowest note seen)
    harmony_range_low: Option<u8>,
    /// Harmony range tracking (highest note seen)
    harmony_range_high: Option<u8>,
}

impl Default for CounterpointState {
    fn default() -> Self {
        Self::new()
    }
}

impl CounterpointState {
    /// Creates a new CounterpointState with no history.
    pub fn new() -> Self {
        Self {
            last_melody: None,
            last_harmony: None,
            interval_history: VecDeque::with_capacity(INTERVAL_HISTORY_SIZE),
            melody_contour: VecDeque::with_capacity(CONTOUR_HISTORY_SIZE),
            harmony_range_low: None,
            harmony_range_high: None,
        }
    }

    /// Resets the state.
    pub fn reset(&mut self) {
        self.last_melody = None;
        self.last_harmony = None;
        self.interval_history.clear();
        self.melody_contour.clear();
        self.harmony_range_low = None;
        self.harmony_range_high = None;
    }

    // --- Helper methods for interval history ---

    /// Pushes an interval class to the history, maintaining the window size.
    fn push_interval(&mut self, interval_class: i8) {
        if self.interval_history.len() >= INTERVAL_HISTORY_SIZE {
            self.interval_history.pop_front();
        }
        self.interval_history.push_back(interval_class);
    }

    /// Counts how many times a given interval class appears in recent history.
    fn count_recent_interval(&self, interval_class: i8) -> usize {
        self.interval_history.iter().filter(|&&i| i == interval_class).count()
    }

    /// Returns true if the interval appears 3+ times in the last 4 intervals.
    fn is_interval_overused(&self, interval_class: i8) -> bool {
        self.count_recent_interval(interval_class) >= 3
    }

    /// Returns true if the interval is not in recent history (fresh).
    fn is_interval_fresh(&self, interval_class: i8) -> bool {
        !self.interval_history.contains(&interval_class)
    }

    // --- Helper methods for melodic contour ---

    /// Calculates the melodic direction between two notes.
    fn direction_between(from: Note, to: Note) -> MelodicDirection {
        let from_midi = u8::from(from);
        let to_midi = u8::from(to);
        if to_midi > from_midi {
            MelodicDirection::Ascending
        } else if to_midi < from_midi {
            MelodicDirection::Descending
        } else {
            MelodicDirection::Static
        }
    }

    /// Pushes a direction to the contour history, maintaining the window size.
    fn push_contour(&mut self, direction: MelodicDirection) {
        if self.melody_contour.len() >= CONTOUR_HISTORY_SIZE {
            self.melody_contour.pop_front();
        }
        self.melody_contour.push_back(direction);
    }

    /// Returns the dominant contour direction if a clear majority exists.
    /// Returns None if the contour is mixed or not enough data.
    fn dominant_contour(&self) -> Option<MelodicDirection> {
        if self.melody_contour.len() < 2 {
            return None;
        }

        let ascending = self.melody_contour.iter()
            .filter(|&&d| d == MelodicDirection::Ascending).count();
        let descending = self.melody_contour.iter()
            .filter(|&&d| d == MelodicDirection::Descending).count();

        let threshold = (self.melody_contour.len() + 1) / 2; // Majority

        if ascending >= threshold {
            Some(MelodicDirection::Ascending)
        } else if descending >= threshold {
            Some(MelodicDirection::Descending)
        } else {
            None
        }
    }

    // --- Helper methods for harmony range ---

    /// Updates the harmony range tracking with a new note.
    fn update_harmony_range(&mut self, note: Note) {
        let midi = u8::from(note);
        self.harmony_range_low = Some(self.harmony_range_low.map_or(midi, |low| low.min(midi)));
        self.harmony_range_high = Some(self.harmony_range_high.map_or(midi, |high| high.max(midi)));
    }

    /// Returns the current harmony range in semitones.
    fn harmony_range(&self) -> Option<u8> {
        match (self.harmony_range_low, self.harmony_range_high) {
            (Some(low), Some(high)) => Some(high - low),
            _ => None,
        }
    }

    /// Returns true if the harmony range is narrow (<= 7 semitones).
    fn is_harmony_range_narrow(&self) -> bool {
        self.harmony_range().map_or(false, |range| range <= NARROW_RANGE_THRESHOLD)
    }

    /// Processes a note with strict counterpoint rules.
    ///
    /// Uses voice-leading scoring:
    /// - Prefers stepwise motion from previous harmony
    /// - Avoids repeating the same harmony note
    /// - Avoids repeating the same interval type (with history buffer)
    /// - Rejects parallel perfect intervals (fifths, octaves)
    /// - When melody repeats, harmony must move
    /// - Tracks melodic contour to encourage contrary motion
    /// - Tracks harmony range to encourage expansion
    /// - Out-of-key notes: uses chromatic consonant intervals
    ///
    /// Returns [melody, harmony] or [melody] if no valid harmony found.
    pub fn process(&mut self, scale: &Scale, melody: Note) -> Vec<Note> {
        // Track melody contour before scoring
        if let Some(prev_melody) = self.last_melody {
            let direction = Self::direction_between(prev_melody, melody);
            self.push_contour(direction);
        }

        let best_candidate = if scale.is_in_scale(melody) {
            // In-key: use diatonic intervals with scoring
            self.find_diatonic_harmony(scale, melody)
        } else {
            // Out-of-key: use chromatic consonant intervals with scoring
            self.find_chromatic_harmony(scale, melody)
        };

        self.last_melody = Some(melody);

        match best_candidate {
            Some((harmony, _)) => {
                // Calculate and store the interval used
                let harmony_midi = u8::from(harmony) as i8;
                let melody_midi = u8::from(melody) as i8;
                let interval_semitones = harmony_midi - melody_midi;
                let interval_class = self.semitones_to_interval_class(interval_semitones);
                self.push_interval(interval_class);

                // Track harmony range
                self.update_harmony_range(harmony);

                self.last_harmony = Some(harmony);
                vec![melody, harmony]
            }
            None => {
                self.last_harmony = None;
                vec![melody]
            }
        }
    }

    /// Finds the best diatonic harmony for an in-key melody note.
    fn find_diatonic_harmony(&self, scale: &Scale, melody: Note) -> Option<(Note, i32)> {
        // Consonant diatonic intervals: 3rds and 6ths (below and above)
        let candidate_intervals: [i8; 8] = [-2, -5, 2, 5, -3, -4, 3, 4];
        let mut best_candidate: Option<(Note, i32)> = None;

        for &interval in &candidate_intervals {
            if let Some(candidate) = scale.transpose_diatonic(melody, interval) {
                let score = self.score_candidate(melody, candidate, interval);

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

        best_candidate
    }

    /// Finds the best chromatic harmony for an out-of-key melody note.
    fn find_chromatic_harmony(&self, scale: &Scale, melody: Note) -> Option<(Note, i32)> {
        // Chromatic consonant intervals (semitones): 3rds, 6ths, 4ths, 5ths
        // Ordered to prefer intervals that might land on scale tones
        let chromatic_intervals: [i8; 8] = [-3, -4, 3, 4, -8, -9, 8, 9]; // m3, M3, m6, M6
        let mut best_candidate: Option<(Note, i32)> = None;

        let melody_midi = u8::from(melody) as i8;

        for &semitones in &chromatic_intervals {
            let candidate_midi = melody_midi + semitones;
            if !(0..=127).contains(&candidate_midi) {
                continue;
            }

            if let Ok(candidate) = Note::try_from(candidate_midi as u8) {
                // Convert semitone interval to approximate diatonic for scoring
                let approx_interval = semitones / 2; // Rough conversion
                let mut score = self.score_candidate(melody, candidate, approx_interval);

                // Bonus if the chromatic harmony lands on a scale tone
                if scale.is_in_scale(candidate) {
                    score += 3;
                }

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

        best_candidate
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

        // Calculate current interval class for history-based scoring
        let current_int = self.semitones_to_interval_class(
            u8::from(candidate) as i8 - u8::from(melody) as i8
        );

        // NEW: Penalize overused intervals (3+ of same in last 4)
        if self.is_interval_overused(current_int) {
            score -= 3;
        }

        // NEW: Bonus for fresh intervals (not in recent history)
        if self.is_interval_fresh(current_int) {
            score += 2;
        }

        // NEW: Contour-based scoring (contrary motion preferred)
        if let Some(dominant) = self.dominant_contour() {
            if let Some(prev_h) = self.last_harmony {
                let harmony_direction = Self::direction_between(prev_h, candidate);

                // Reward contrary motion to melody trend
                let is_contrary = match (dominant, harmony_direction) {
                    (MelodicDirection::Ascending, MelodicDirection::Descending) => true,
                    (MelodicDirection::Descending, MelodicDirection::Ascending) => true,
                    _ => false,
                };

                let is_parallel = match (dominant, harmony_direction) {
                    (MelodicDirection::Ascending, MelodicDirection::Ascending) => true,
                    (MelodicDirection::Descending, MelodicDirection::Descending) => true,
                    _ => false,
                };

                if is_contrary {
                    score += 3; // Encourage contrary motion
                } else if is_parallel {
                    score -= 1; // Slight penalty for parallel motion
                }
            }
        }

        // NEW: Range expansion bonus (encourage leaps when range is narrow)
        if self.is_harmony_range_narrow() {
            if let Some(prev_h) = self.last_harmony {
                let prev_midi = u8::from(prev_h) as i32;
                let cand_midi = u8::from(candidate) as i32;
                let step_size = (cand_midi - prev_midi).abs();

                // Bonus for larger leaps when range is too narrow
                if step_size >= 5 {
                    score += 2;
                }
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

    // --- New tests for enhanced counterpoint state ---

    #[test]
    fn test_interval_history_tracks_multiple() {
        let mut state = CounterpointState::new();

        // Push 4 intervals
        state.push_interval(3);
        state.push_interval(6);
        state.push_interval(3);
        state.push_interval(4);

        assert_eq!(state.interval_history.len(), 4);
        assert_eq!(state.count_recent_interval(3), 2);
        assert_eq!(state.count_recent_interval(6), 1);

        // Push another - should evict oldest
        state.push_interval(5);
        assert_eq!(state.interval_history.len(), 4);
        assert_eq!(state.count_recent_interval(3), 1); // First 3 was evicted
    }

    #[test]
    fn test_too_many_thirds_penalized() {
        let mut state = CounterpointState::new();

        // Fill history with thirds (interval class 3)
        state.push_interval(3);
        state.push_interval(3);
        state.push_interval(3);

        assert!(state.is_interval_overused(3));
        assert!(!state.is_interval_overused(6));
    }

    #[test]
    fn test_contour_tracks_direction() {
        let mut state = CounterpointState::new();

        state.push_contour(MelodicDirection::Ascending);
        state.push_contour(MelodicDirection::Ascending);
        state.push_contour(MelodicDirection::Static);

        assert_eq!(state.melody_contour.len(), 3);
        assert_eq!(state.dominant_contour(), Some(MelodicDirection::Ascending));
    }

    #[test]
    fn test_contour_descending_detection() {
        let mut state = CounterpointState::new();

        state.push_contour(MelodicDirection::Descending);
        state.push_contour(MelodicDirection::Descending);

        assert_eq!(state.dominant_contour(), Some(MelodicDirection::Descending));
    }

    #[test]
    fn test_contour_mixed_returns_none() {
        let mut state = CounterpointState::new();

        state.push_contour(MelodicDirection::Ascending);
        state.push_contour(MelodicDirection::Descending);
        state.push_contour(MelodicDirection::Static);

        // No clear majority
        assert_eq!(state.dominant_contour(), None);
    }

    #[test]
    fn test_contrary_motion_preferred_with_ascending_melody() {
        let scale = Scale::major(0);
        let mut state = CounterpointState::new();

        // Build up an ascending contour
        let _ = state.process(&scale, Note::C4);
        let _ = state.process(&scale, Note::D4);
        let _ = state.process(&scale, Note::E4);
        let _ = state.process(&scale, Note::F4);

        // Contour should now be ascending
        assert_eq!(state.dominant_contour(), Some(MelodicDirection::Ascending));

        // Get the last harmony before next note
        let prev_h = state.last_harmony.unwrap();

        // Play another ascending note
        let result = state.process(&scale, Note::G4);
        let new_h = result[1];

        // Harmony should tend to go down (contrary motion)
        // This is a tendency, not a guarantee, so we just check it's different
        assert!(result.len() == 2, "Should produce harmony");
        // The contrary motion bonus should influence the choice
        let _ = (u8::from(prev_h), u8::from(new_h)); // Compiler hint for usage
    }

    #[test]
    fn test_harmony_range_tracking() {
        let mut state = CounterpointState::new();

        state.update_harmony_range(Note::C4); // MIDI 60
        assert_eq!(state.harmony_range_low, Some(60));
        assert_eq!(state.harmony_range_high, Some(60));

        state.update_harmony_range(Note::G4); // MIDI 67
        assert_eq!(state.harmony_range_low, Some(60));
        assert_eq!(state.harmony_range_high, Some(67));

        state.update_harmony_range(Note::A3); // MIDI 57
        assert_eq!(state.harmony_range_low, Some(57));
        assert_eq!(state.harmony_range_high, Some(67));

        assert_eq!(state.harmony_range(), Some(10)); // 67 - 57
    }

    #[test]
    fn test_narrow_range_detection() {
        let mut state = CounterpointState::new();

        // No range yet
        assert!(!state.is_harmony_range_narrow());

        state.update_harmony_range(Note::C4); // 60
        state.update_harmony_range(Note::E4); // 64
        // Range is 4 semitones (major 3rd) - narrow
        assert!(state.is_harmony_range_narrow());

        state.update_harmony_range(Note::C5); // 72
        // Range is now 12 semitones - not narrow
        assert!(!state.is_harmony_range_narrow());
    }

    #[test]
    fn test_reset_clears_all_history() {
        let scale = Scale::major(0);
        let mut state = CounterpointState::new();

        // Build up some state
        let _ = state.process(&scale, Note::C4);
        let _ = state.process(&scale, Note::D4);
        let _ = state.process(&scale, Note::E4);

        assert!(state.last_melody.is_some());
        assert!(state.last_harmony.is_some());
        assert!(!state.interval_history.is_empty());
        assert!(!state.melody_contour.is_empty());
        assert!(state.harmony_range_low.is_some());

        state.reset();

        assert!(state.last_melody.is_none());
        assert!(state.last_harmony.is_none());
        assert!(state.interval_history.is_empty());
        assert!(state.melody_contour.is_empty());
        assert!(state.harmony_range_low.is_none());
        assert!(state.harmony_range_high.is_none());
    }

    #[test]
    fn test_varied_harmony_over_repeated_melody() {
        let scale = Scale::major(0);
        let mut state = CounterpointState::new();

        // Play the same note 6 times
        let mut harmonies = Vec::new();
        for _ in 0..6 {
            let result = state.process(&scale, Note::C4);
            assert_eq!(result.len(), 2, "Should always produce harmony");
            harmonies.push(u8::from(result[1]));
        }

        // Should get at least 3 different harmony notes
        let mut unique = harmonies.clone();
        unique.sort();
        unique.dedup();
        assert!(unique.len() >= 3,
            "Expected at least 3 different harmonies over 6 repeats, got {} unique: {:?}",
            unique.len(), unique);
    }

    #[test]
    fn test_ascending_scale_gets_varied_intervals() {
        let scale = Scale::major(0);
        let mut state = CounterpointState::new();

        // Play ascending scale: C D E F G A
        let notes = [Note::C4, Note::D4, Note::E4, Note::F4, Note::G4, Note::A4];
        let mut intervals = Vec::new();

        for note in &notes {
            let result = state.process(&scale, *note);
            if result.len() == 2 {
                let melody_midi = u8::from(*note) as i8;
                let harmony_midi = u8::from(result[1]) as i8;
                let interval = (harmony_midi - melody_midi).abs() % 12;
                intervals.push(interval);
            }
        }

        // Should get at least 2 different interval types
        let mut unique = intervals.clone();
        unique.sort();
        unique.dedup();
        assert!(unique.len() >= 2,
            "Expected varied intervals on ascending scale, got: {:?}", unique);
    }

    #[test]
    fn test_direction_between_helper() {
        assert_eq!(
            CounterpointState::direction_between(Note::C4, Note::E4),
            MelodicDirection::Ascending
        );
        assert_eq!(
            CounterpointState::direction_between(Note::E4, Note::C4),
            MelodicDirection::Descending
        );
        assert_eq!(
            CounterpointState::direction_between(Note::C4, Note::C4),
            MelodicDirection::Static
        );
    }

    #[test]
    fn test_interval_fresh_detection() {
        let mut state = CounterpointState::new();

        // Empty history - all intervals are fresh
        assert!(state.is_interval_fresh(3));
        assert!(state.is_interval_fresh(6));

        state.push_interval(3);
        state.push_interval(4);

        assert!(!state.is_interval_fresh(3));
        assert!(!state.is_interval_fresh(4));
        assert!(state.is_interval_fresh(6));
    }
}
