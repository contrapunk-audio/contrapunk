//! Harmony mode implementations.
//!
//! Each mode function takes an input note and scale, returning a Vec<Note>
//! containing the original note plus any harmony notes.

use rand::Rng;
use wmidi::Note;

use crate::harmony::Scale;

/// Mode 1: Pass-through (no harmony)
///
/// Returns only the input note unchanged.
pub fn pass_through(note: Note, _scale: &mut Scale) -> Vec<Note> {
    vec![note]
}

/// Mode 2: Diatonic thirds above
///
/// Returns the input note plus a diatonic third (2 scale degrees) above.
/// For out-of-key notes, uses a consonant chromatic third instead.
/// If harmony would be out of range, returns only the original note.
pub fn diatonic_thirds(note: Note, scale: &mut Scale) -> Vec<Note> {
    match scale.harmonize_smart(note, 2, true) {
        Some(harmony) => vec![note, harmony],
        None => vec![note],  // Out of range, just pass through
    }
}

/// Mode 3: Diatonic fourths above
///
/// Returns the input note plus a diatonic fourth (3 scale degrees) above.
/// For out-of-key notes, uses a consonant chromatic interval instead.
/// If harmony would be out of range, returns only the original note.
pub fn diatonic_fourths(note: Note, scale: &mut Scale) -> Vec<Note> {
    match scale.harmonize_smart(note, 3, true) {
        Some(harmony) => vec![note, harmony],
        None => vec![note],
    }
}

/// Mode 4: Random diatonic interval below
///
/// Returns the input note plus a random diatonic interval below
/// (2nd through 7th below, i.e., -1 to -6 scale degrees).
/// For out-of-key notes, uses a consonant chromatic interval instead.
///
/// Note: For deterministic Note-Off handling, this mode requires
/// tracking active notes. The random selection happens on Note-On
/// and the same interval is used for Note-Off.
pub fn random_below(note: Note, scale: &mut Scale) -> Vec<Note> {
    let mut rng = rand::thread_rng();

    // Intervals: -1 (2nd below) to -6 (7th below)
    let intervals = [-1, -2, -3, -4, -5, -6];
    let interval = intervals[rng.gen_range(0..intervals.len())];

    match scale.harmonize_smart(note, interval, false) {
        Some(harmony) => vec![note, harmony],
        None => vec![note],
    }
}

/// Mode 5: Random diatonic below, excluding seconds
///
/// Like Mode 4, but excludes 2nds (which can sound dissonant).
/// Returns input plus a random interval from 3rd to 7th below.
/// For out-of-key notes, uses a consonant chromatic interval instead.
pub fn random_below_no_seconds(note: Note, scale: &mut Scale) -> Vec<Note> {
    let mut rng = rand::thread_rng();

    // Intervals: -2 (3rd below) to -6 (7th below), skipping -1 (2nd)
    let intervals = [-2, -3, -4, -5, -6];
    let interval = intervals[rng.gen_range(0..intervals.len())];

    match scale.harmonize_smart(note, interval, false) {
        Some(harmony) => vec![note, harmony],
        None => vec![note],
    }
}

// --- Directed variants for voice position ---
// These take an `above: bool` parameter to control harmony direction.
// When `above` is true, harmony is generated above the input note.
// When `above` is false, harmony is generated below.

/// Directed diatonic thirds: above or below the input note.
pub fn diatonic_thirds_directed(note: Note, scale: &mut Scale, above: bool) -> Vec<Note> {
    let interval = if above { 2 } else { -2 };
    match scale.harmonize_smart(note, interval, above) {
        Some(harmony) => vec![note, harmony],
        None => vec![note],
    }
}

/// Directed diatonic fourths: above or below the input note.
pub fn diatonic_fourths_directed(note: Note, scale: &mut Scale, above: bool) -> Vec<Note> {
    let interval = if above { 3 } else { -3 };
    match scale.harmonize_smart(note, interval, above) {
        Some(harmony) => vec![note, harmony],
        None => vec![note],
    }
}

/// Directed random diatonic interval.
/// Above: random 2nd-7th above. Below: random 2nd-7th below.
pub fn random_directed(note: Note, scale: &mut Scale, above: bool) -> Vec<Note> {
    let mut rng = rand::thread_rng();
    let intervals: [i8; 6] = if above {
        [1, 2, 3, 4, 5, 6]
    } else {
        [-1, -2, -3, -4, -5, -6]
    };
    let interval = intervals[rng.gen_range(0..intervals.len())];

    match scale.harmonize_smart(note, interval, above) {
        Some(harmony) => vec![note, harmony],
        None => vec![note],
    }
}

/// Directed random diatonic interval, excluding seconds.
/// Above: random 3rd-7th above. Below: random 3rd-7th below.
pub fn random_no_seconds_directed(note: Note, scale: &mut Scale, above: bool) -> Vec<Note> {
    let mut rng = rand::thread_rng();
    let intervals: [i8; 5] = if above {
        [2, 3, 4, 5, 6]
    } else {
        [-2, -3, -4, -5, -6]
    };
    let interval = intervals[rng.gen_range(0..intervals.len())];

    match scale.harmonize_smart(note, interval, above) {
        Some(harmony) => vec![note, harmony],
        None => vec![note],
    }
}

/// Mode 8: Barry Harris 6th diminished movement
///
/// Moves by 2 scale degrees in the current scale. In an 8-note Barry Harris
/// scale, this preserves chord-tone/passing-tone parity (even degrees map to
/// even, odd to odd). Works with any scale but is musically intended for
/// BH 6th Dim scales.
pub fn barry_harris(note: Note, scale: &mut Scale) -> Vec<Note> {
    match scale.harmonize_smart(note, 2, true) {
        Some(harmony) => vec![note, harmony],
        None => vec![note],
    }
}

/// Directed Barry Harris movement: above or below the input note.
pub fn barry_harris_directed(note: Note, scale: &mut Scale, above: bool) -> Vec<Note> {
    let interval = if above { 2 } else { -2 };
    match scale.harmonize_smart(note, interval, above) {
        Some(harmony) => vec![note, harmony],
        None => vec![note],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pass_through() {
        let mut scale = Scale::major(0);
        let result = pass_through(Note::C4, &mut scale);
        assert_eq!(result, vec![Note::C4]);
    }

    #[test]
    fn test_diatonic_thirds() {
        let mut scale = Scale::major(0);  // C major

        // C4 + third = E4
        let result = diatonic_thirds(Note::C4, &mut scale);
        assert_eq!(result, vec![Note::C4, Note::E4]);

        // E4 + third = G4 (minor third, but still 2 scale degrees)
        let result = diatonic_thirds(Note::E4, &mut scale);
        assert_eq!(result, vec![Note::E4, Note::G4]);
    }

    #[test]
    fn test_diatonic_fourths() {
        let mut scale = Scale::major(0);

        // C4 + fourth = F4
        let result = diatonic_fourths(Note::C4, &mut scale);
        assert_eq!(result, vec![Note::C4, Note::F4]);
    }

    #[test]
    fn test_random_below_produces_harmony() {
        let mut scale = Scale::major(0);

        // Should produce 2 notes (original + harmony below)
        let result = random_below(Note::C5, &mut scale);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], Note::C5);

        // Harmony should be lower than input
        let harmony_midi: u8 = result[1].into();
        let input_midi: u8 = Note::C5.into();
        assert!(harmony_midi < input_midi);
    }
}
