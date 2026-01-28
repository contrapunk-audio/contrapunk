//! Chord detection module for Contrapunk.
//!
//! Analyzes MIDI note combinations to identify common chord types.

use std::collections::HashSet;

/// Note names for display (C, C#, D, ..., B).
const NOTE_NAMES: [&str; 12] = ["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"];

/// Common chord types defined by intervals from root (in semitones).
/// The intervals are relative to the root note, not absolute pitches.
struct ChordPattern {
    name: &'static str,
    intervals: &'static [u8], // Semitones from root
}

/// Known chord patterns ordered by specificity (longer patterns first).
/// This ensures we match the most specific chord type when multiple patterns could match.
const CHORD_PATTERNS: &[ChordPattern] = &[
    // Seventh chords (4 notes - check first)
    ChordPattern { name: "maj7", intervals: &[0, 4, 7, 11] },     // Major 7th
    ChordPattern { name: "7", intervals: &[0, 4, 7, 10] },        // Dominant 7th
    ChordPattern { name: "min7", intervals: &[0, 3, 7, 10] },     // Minor 7th
    ChordPattern { name: "dim7", intervals: &[0, 3, 6, 9] },      // Diminished 7th
    ChordPattern { name: "m7b5", intervals: &[0, 3, 6, 10] },     // Half-diminished

    // Triads (3 notes)
    ChordPattern { name: "maj", intervals: &[0, 4, 7] },          // Major
    ChordPattern { name: "min", intervals: &[0, 3, 7] },          // Minor
    ChordPattern { name: "dim", intervals: &[0, 3, 6] },          // Diminished
    ChordPattern { name: "aug", intervals: &[0, 4, 8] },          // Augmented
    ChordPattern { name: "sus4", intervals: &[0, 5, 7] },         // Suspended 4th
    ChordPattern { name: "sus2", intervals: &[0, 2, 7] },         // Suspended 2nd

    // Power chord (2 notes)
    ChordPattern { name: "5", intervals: &[0, 7] },               // Power chord
];

/// Detects the chord from a set of MIDI note numbers.
///
/// Analyzes the pitch classes (ignoring octaves) and attempts to match
/// against known chord patterns.
///
/// # Arguments
/// * `notes` - Set of MIDI note numbers (0-127)
///
/// # Returns
/// The chord name (e.g., "Cmaj", "Am7") or None if no chord detected.
pub fn detect_chord(notes: &HashSet<u8>) -> Option<String> {
    if notes.len() < 2 {
        return None;
    }

    // Convert to pitch classes (0-11) and remove duplicates
    let pitch_classes: HashSet<u8> = notes.iter().map(|n| n % 12).collect();
    let mut pcs: Vec<u8> = pitch_classes.into_iter().collect();
    pcs.sort();

    if pcs.len() < 2 {
        return None;
    }

    // Try each pitch class as potential root
    for &root in &pcs {
        // Calculate intervals from this root
        let intervals: HashSet<u8> = pcs.iter()
            .map(|&pc| (pc + 12 - root) % 12)
            .collect();

        // Check against known patterns (ordered by specificity)
        for pattern in CHORD_PATTERNS {
            let pattern_set: HashSet<u8> = pattern.intervals.iter().copied().collect();
            if intervals == pattern_set {
                let root_name = NOTE_NAMES[root as usize];
                return Some(format!("{}{}", root_name, pattern.name));
            }
        }
    }

    None
}

/// Returns a display string for the chord, or special indicators.
///
/// - If no notes: returns em-dash
/// - If chord detected: returns chord name (e.g., "Cmaj")
/// - If no chord pattern matches: returns individual note names
///
/// # Arguments
/// * `notes` - Set of MIDI note numbers (0-127)
pub fn chord_display(notes: &HashSet<u8>) -> String {
    if notes.is_empty() {
        return "\u{2014}".to_string(); // Em-dash
    }

    match detect_chord(notes) {
        Some(chord) => chord,
        None => {
            // Show individual notes if no chord detected
            let mut pcs: Vec<u8> = notes.iter().map(|n| n % 12).collect();
            pcs.sort();
            pcs.dedup();
            let names: Vec<&str> = pcs.iter().map(|&pc| NOTE_NAMES[pc as usize]).collect();
            names.join(" + ")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_c_major() {
        let notes: HashSet<u8> = [60, 64, 67].into_iter().collect(); // C4, E4, G4
        assert_eq!(detect_chord(&notes), Some("Cmaj".to_string()));
    }

    #[test]
    fn test_a_minor() {
        let notes: HashSet<u8> = [57, 60, 64].into_iter().collect(); // A3, C4, E4
        assert_eq!(detect_chord(&notes), Some("Amin".to_string()));
    }

    #[test]
    fn test_c_major_7() {
        let notes: HashSet<u8> = [60, 64, 67, 71].into_iter().collect(); // C4, E4, G4, B4
        assert_eq!(detect_chord(&notes), Some("Cmaj7".to_string()));
    }

    #[test]
    fn test_d_dominant_7() {
        let notes: HashSet<u8> = [62, 66, 69, 72].into_iter().collect(); // D4, F#4, A4, C5
        assert_eq!(detect_chord(&notes), Some("D7".to_string()));
    }

    #[test]
    fn test_single_note() {
        let notes: HashSet<u8> = [60].into_iter().collect();
        assert_eq!(detect_chord(&notes), None);
    }

    #[test]
    fn test_chord_display_empty() {
        let notes: HashSet<u8> = HashSet::new();
        assert_eq!(chord_display(&notes), "\u{2014}");
    }

    #[test]
    fn test_chord_display_unknown() {
        let notes: HashSet<u8> = [60, 61].into_iter().collect(); // C, C# - no pattern
        let display = chord_display(&notes);
        assert!(display.contains("C") && display.contains("C#"));
    }
}
