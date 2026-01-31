//! Chord detection module for Contrapunk.
//!
//! Analyzes MIDI note combinations to identify common chord types.
//! Supports extended chords (9th, 11th, 13th), altered dominants,
//! slash chords, add chords, 6th chords, and roman numeral analysis.

use std::collections::HashSet;

/// Note names for display (C, C#, D, ..., B).
const NOTE_NAMES: [&str; 12] = ["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"];

/// Flat-key note names for context-aware enharmonic spelling.
const NOTE_NAMES_FLAT: [&str; 12] = ["C", "Db", "D", "Eb", "E", "F", "Gb", "G", "Ab", "A", "Bb", "B"];

/// Common chord types defined by intervals from root (in semitones).
struct ChordPattern {
    name: &'static str,
    intervals: &'static [u8],
}

/// Known chord patterns ordered by specificity (longer patterns first).
const CHORD_PATTERNS: &[ChordPattern] = &[
    // 13th chords (6 notes)
    ChordPattern { name: "maj13", intervals: &[0, 2, 4, 7, 9, 11] },
    ChordPattern { name: "13", intervals: &[0, 2, 4, 7, 9, 10] },
    ChordPattern { name: "min13", intervals: &[0, 2, 3, 7, 9, 10] },

    // 11th chords (5 notes)
    ChordPattern { name: "maj9#11", intervals: &[0, 2, 4, 6, 7, 11] },
    ChordPattern { name: "11", intervals: &[0, 2, 4, 5, 7, 10] },
    ChordPattern { name: "min11", intervals: &[0, 2, 3, 5, 7, 10] },

    // 6/9 chord (5 notes)
    ChordPattern { name: "6/9", intervals: &[0, 2, 4, 7, 9] },

    // 9th chords (5 notes)
    ChordPattern { name: "maj9", intervals: &[0, 2, 4, 7, 11] },
    ChordPattern { name: "9", intervals: &[0, 2, 4, 7, 10] },
    ChordPattern { name: "min9", intervals: &[0, 2, 3, 7, 10] },
    ChordPattern { name: "7b9", intervals: &[0, 1, 4, 7, 10] },
    ChordPattern { name: "7#9", intervals: &[0, 3, 4, 7, 10] },

    // Altered dominants (4-5 notes)
    ChordPattern { name: "7#11", intervals: &[0, 4, 6, 7, 10] },
    ChordPattern { name: "7b13", intervals: &[0, 4, 7, 8, 10] },
    ChordPattern { name: "7alt", intervals: &[0, 1, 4, 8, 10] },

    // 6th chords (4 notes)
    ChordPattern { name: "maj6", intervals: &[0, 4, 7, 9] },
    ChordPattern { name: "min6", intervals: &[0, 3, 7, 9] },

    // Add chords (4 notes)
    ChordPattern { name: "add9", intervals: &[0, 2, 4, 7] },
    ChordPattern { name: "madd9", intervals: &[0, 2, 3, 7] },
    ChordPattern { name: "add11", intervals: &[0, 4, 5, 7] },

    // Seventh chords (4 notes)
    ChordPattern { name: "maj7", intervals: &[0, 4, 7, 11] },
    ChordPattern { name: "7", intervals: &[0, 4, 7, 10] },
    ChordPattern { name: "min7", intervals: &[0, 3, 7, 10] },
    ChordPattern { name: "dim7", intervals: &[0, 3, 6, 9] },
    ChordPattern { name: "m7b5", intervals: &[0, 3, 6, 10] },
    ChordPattern { name: "minmaj7", intervals: &[0, 3, 7, 11] },
    ChordPattern { name: "aug7", intervals: &[0, 4, 8, 10] },
    ChordPattern { name: "augmaj7", intervals: &[0, 4, 8, 11] },
    ChordPattern { name: "7sus4", intervals: &[0, 5, 7, 10] },
    ChordPattern { name: "7sus2", intervals: &[0, 2, 7, 10] },

    // Triads (3 notes)
    ChordPattern { name: "maj", intervals: &[0, 4, 7] },
    ChordPattern { name: "min", intervals: &[0, 3, 7] },
    ChordPattern { name: "dim", intervals: &[0, 3, 6] },
    ChordPattern { name: "aug", intervals: &[0, 4, 8] },
    ChordPattern { name: "sus4", intervals: &[0, 5, 7] },
    ChordPattern { name: "sus2", intervals: &[0, 2, 7] },

    // Power chord (2 notes)
    ChordPattern { name: "5", intervals: &[0, 7] },
];

/// Detects the chord from a set of MIDI note numbers.
///
/// Analyzes the pitch classes (ignoring octaves) and attempts to match
/// against known chord patterns. Detects slash chords when the lowest
/// note differs from the chord root.
///
/// # Arguments
/// * `notes` - Set of MIDI note numbers (0-127)
///
/// # Returns
/// The chord name (e.g., "Cmaj", "Am7", "Cmaj/E") or None if no chord detected.
pub fn detect_chord(notes: &HashSet<u8>) -> Option<String> {
    if notes.len() < 2 {
        return None;
    }

    // Find the lowest MIDI note for slash chord detection
    let bass = *notes.iter().min().unwrap();
    let bass_pc = bass % 12;

    // Convert to pitch classes (0-11) and remove duplicates
    let pitch_classes: HashSet<u8> = notes.iter().map(|n| n % 12).collect();
    let mut pcs: Vec<u8> = pitch_classes.into_iter().collect();
    pcs.sort();

    if pcs.len() < 2 {
        return None;
    }

    // Try all patterns for all roots, collecting matches.
    // Prefer: (1) more intervals, (2) root == bass note, (3) earlier pattern order.
    let mut best: Option<(usize, bool, usize, u8, &str)> = None; // (interval_count desc, root_is_bass, pattern_idx, root, name)

    for &root in &pcs {
        let intervals: HashSet<u8> = pcs.iter()
            .map(|&pc| (pc + 12 - root) % 12)
            .collect();

        for (pidx, pattern) in CHORD_PATTERNS.iter().enumerate() {
            let pattern_set: HashSet<u8> = pattern.intervals.iter().copied().collect();
            if intervals == pattern_set {
                let root_is_bass = root == bass_pc;
                let ilen = pattern.intervals.len();
                // Compare: prefer more intervals, then root==bass, then earlier pattern
                let dominated = if let Some((best_ilen, best_rib, best_pidx, _, _)) = best {
                    if ilen > best_ilen { false }
                    else if ilen < best_ilen { true }
                    else if root_is_bass && !best_rib { false }
                    else if !root_is_bass && best_rib { true }
                    else { pidx >= best_pidx }
                } else { false };
                if !dominated || best.is_none() {
                    best = Some((ilen, root_is_bass, pidx, root, pattern.name));
                }
            }
        }
    }

    best.map(|(_, _, _, root, name)| {
        let root_name = NOTE_NAMES[root as usize];
        if bass_pc != root {
            let bass_name = NOTE_NAMES[bass_pc as usize];
            format!("{}{}/{}", root_name, name, bass_name)
        } else {
            format!("{}{}", root_name, name)
        }
    })
}

/// Returns a display string for the chord, or special indicators.
///
/// - If no notes: returns em-dash
/// - If chord detected: returns chord name (e.g., "Cmaj")
/// - If no chord pattern matches: returns individual note names
pub fn chord_display(notes: &HashSet<u8>) -> String {
    if notes.is_empty() {
        return "\u{2014}".to_string();
    }

    match detect_chord(notes) {
        Some(chord) => chord,
        None => {
            let mut pcs: Vec<u8> = notes.iter().map(|n| n % 12).collect();
            pcs.sort();
            pcs.dedup();
            let names: Vec<&str> = pcs.iter().map(|&pc| NOTE_NAMES[pc as usize]).collect();
            names.join(" + ")
        }
    }
}

/// Returns the roman numeral for a scale degree relative to a key tonic.
///
/// # Arguments
/// * `root_pc` - Pitch class of the chord root (0-11)
/// * `key_tonic` - Pitch class of the key tonic (0-11)
pub fn roman_numeral(root_pc: u8, key_tonic: u8) -> String {
    let degree = (root_pc + 12 - key_tonic) % 12;
    match degree {
        0 => "I",
        1 => "bII",
        2 => "II",
        3 => "bIII",
        4 => "III",
        5 => "IV",
        6 => "#IV/bV",
        7 => "V",
        8 => "bVI",
        9 => "VI",
        10 => "bVII",
        11 => "VII",
        _ => "?",
    }.to_string()
}

/// Returns chord name with roman numeral analysis in a given key.
///
/// Format: "Fmaj7 (IVmaj7 in C)"
pub fn chord_display_with_analysis(notes: &HashSet<u8>, key_tonic: Option<u8>) -> String {
    if notes.is_empty() {
        return "\u{2014}".to_string();
    }

    let chord_str = match detect_chord(notes) {
        Some(chord) => chord,
        None => {
            let mut pcs: Vec<u8> = notes.iter().map(|n| n % 12).collect();
            pcs.sort();
            pcs.dedup();
            let names: Vec<&str> = pcs.iter().map(|&pc| NOTE_NAMES[pc as usize]).collect();
            return names.join(" + ");
        }
    };

    if let Some(tonic) = key_tonic {
        // Extract root from chord name (first 1-2 chars)
        let root_pc = parse_root_from_chord(&chord_str);
        if let Some(rpc) = root_pc {
            let rn = roman_numeral(rpc, tonic);
            // Extract quality (everything after root name)
            let quality = extract_quality(&chord_str);
            let key_name = NOTE_NAMES[tonic as usize];
            return format!("{} ({}{} in {})", chord_str, rn, quality, key_name);
        }
    }

    chord_str
}

/// Parse the root pitch class from a chord name string.
fn parse_root_from_chord(chord: &str) -> Option<u8> {
    if chord.len() >= 2 && (chord.as_bytes()[1] == b'#' || chord.as_bytes()[1] == b'b') {
        let root_str = &chord[..2];
        NOTE_NAMES.iter().position(|&n| n == root_str)
            .or_else(|| NOTE_NAMES_FLAT.iter().position(|&n| n == root_str))
            .map(|i| i as u8)
    } else {
        let root_str = &chord[..1];
        NOTE_NAMES.iter().position(|&n| n == root_str).map(|i| i as u8)
    }
}

/// Extract chord quality (everything after root name).
fn extract_quality(chord: &str) -> &str {
    if chord.len() >= 2 && (chord.as_bytes()[1] == b'#' || chord.as_bytes()[1] == b'b') {
        &chord[2..]
    } else {
        &chord[1..]
    }
}

/// Returns whether a key tonic corresponds to a flat key.
pub fn is_flat_key(tonic: u8) -> bool {
    // F, Bb, Eb, Ab, Db, Gb
    matches!(tonic, 5 | 10 | 3 | 8 | 1 | 6)
}

/// Returns context-aware note name based on key.
///
/// Flat keys use flats (Bb, Eb, etc.), sharp keys use sharps (C#, F#, etc.).
pub fn note_name_in_context(pc: u8, key_tonic: u8) -> &'static str {
    if is_flat_key(key_tonic) {
        NOTE_NAMES_FLAT[pc as usize]
    } else {
        NOTE_NAMES[pc as usize]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // === Existing tests (backward compatibility) ===

    #[test]
    fn test_c_major() {
        let notes: HashSet<u8> = [60, 64, 67].into_iter().collect();
        assert_eq!(detect_chord(&notes), Some("Cmaj".to_string()));
    }

    #[test]
    fn test_a_minor() {
        let notes: HashSet<u8> = [57, 60, 64].into_iter().collect();
        assert_eq!(detect_chord(&notes), Some("Amin".to_string()));
    }

    #[test]
    fn test_c_major_7() {
        let notes: HashSet<u8> = [60, 64, 67, 71].into_iter().collect();
        assert_eq!(detect_chord(&notes), Some("Cmaj7".to_string()));
    }

    #[test]
    fn test_d_dominant_7() {
        let notes: HashSet<u8> = [62, 66, 69, 72].into_iter().collect();
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
        let notes: HashSet<u8> = [60, 61].into_iter().collect();
        let display = chord_display(&notes);
        assert!(display.contains("C") && display.contains("C#"));
    }

    // === Extended chord tests ===

    #[test]
    fn test_cmaj9() {
        let notes: HashSet<u8> = [60, 64, 67, 71, 74].into_iter().collect();
        assert_eq!(detect_chord(&notes), Some("Cmaj9".to_string()));
    }

    #[test]
    fn test_dominant_9() {
        let notes: HashSet<u8> = [60, 64, 67, 70, 74].into_iter().collect();
        assert_eq!(detect_chord(&notes), Some("C9".to_string()));
    }

    #[test]
    fn test_min9() {
        let notes: HashSet<u8> = [60, 63, 67, 70, 74].into_iter().collect();
        assert_eq!(detect_chord(&notes), Some("Cmin9".to_string()));
    }

    #[test]
    fn test_7b9() {
        let notes: HashSet<u8> = [67, 71, 74, 77, 68].into_iter().collect();
        assert_eq!(detect_chord(&notes), Some("G7b9".to_string()));
    }

    #[test]
    fn test_7sharp9() {
        let notes: HashSet<u8> = [60, 63, 64, 67, 70].into_iter().collect();
        assert_eq!(detect_chord(&notes), Some("C7#9".to_string()));
    }

    #[test]
    fn test_13th() {
        let notes: HashSet<u8> = [60, 64, 67, 70, 74, 69].into_iter().collect();
        assert_eq!(detect_chord(&notes), Some("C13".to_string()));
    }

    #[test]
    fn test_11th() {
        let notes: HashSet<u8> = [60, 64, 65, 67, 70, 74].into_iter().collect();
        assert_eq!(detect_chord(&notes), Some("C11".to_string()));
    }

    // === Slash chord tests ===

    #[test]
    fn test_slash_chord_c_over_e() {
        // C major with E as lowest note
        let notes: HashSet<u8> = [52, 60, 67].into_iter().collect(); // E3, C4, G4
        assert_eq!(detect_chord(&notes), Some("Cmaj/E".to_string()));
    }

    #[test]
    fn test_slash_chord_g7_over_b() {
        // G7 with B as lowest note
        let notes: HashSet<u8> = [47, 67, 71, 74, 77].into_iter().collect(); // B2, G4, B4, D5, F5
        assert_eq!(detect_chord(&notes), Some("G7/B".to_string()));
    }

    // === 6th chord tests ===

    #[test]
    fn test_cmaj6() {
        let notes: HashSet<u8> = [60, 64, 67, 69].into_iter().collect();
        assert_eq!(detect_chord(&notes), Some("Cmaj6".to_string()));
    }

    #[test]
    fn test_amin6() {
        let notes: HashSet<u8> = [57, 60, 64, 66].into_iter().collect();
        assert_eq!(detect_chord(&notes), Some("Amin6".to_string()));
    }

    // === Add chord tests ===

    #[test]
    fn test_cadd9() {
        let notes: HashSet<u8> = [60, 62, 64, 67].into_iter().collect();
        assert_eq!(detect_chord(&notes), Some("Cadd9".to_string()));
    }

    #[test]
    fn test_madd9() {
        let notes: HashSet<u8> = [60, 62, 63, 67].into_iter().collect();
        assert_eq!(detect_chord(&notes), Some("Cmadd9".to_string()));
    }

    // === Roman numeral tests ===

    #[test]
    fn test_roman_numeral_iv() {
        assert_eq!(roman_numeral(5, 0), "IV");
    }

    #[test]
    fn test_roman_numeral_v() {
        assert_eq!(roman_numeral(7, 0), "V");
    }

    #[test]
    fn test_roman_numeral_bvii() {
        assert_eq!(roman_numeral(10, 0), "bVII");
    }

    // === Context-aware enharmonics ===

    #[test]
    fn test_flat_key_enharmonics() {
        assert_eq!(note_name_in_context(1, 5), "Db"); // Db in key of F
        assert_eq!(note_name_in_context(10, 5), "Bb"); // Bb in key of F
    }

    #[test]
    fn test_sharp_key_enharmonics() {
        assert_eq!(note_name_in_context(1, 0), "C#"); // C# in key of C
        assert_eq!(note_name_in_context(6, 7), "F#"); // F# in key of G
    }

    // === Chord display with analysis ===

    #[test]
    fn test_chord_display_with_analysis() {
        let notes: HashSet<u8> = [65, 69, 72, 76].into_iter().collect(); // F4, A4, C5, E5 = Fmaj7
        let result = chord_display_with_analysis(&notes, Some(0));
        assert_eq!(result, "Fmaj7 (IVmaj7 in C)");
    }

    #[test]
    fn test_chord_display_with_analysis_no_key() {
        let notes: HashSet<u8> = [60, 64, 67].into_iter().collect();
        let result = chord_display_with_analysis(&notes, None);
        assert_eq!(result, "Cmaj");
    }

    // === Additional 7th chord tests ===

    #[test]
    fn test_minmaj7() {
        let notes: HashSet<u8> = [60, 63, 67, 71].into_iter().collect();
        assert_eq!(detect_chord(&notes), Some("Cminmaj7".to_string()));
    }

    #[test]
    fn test_7sus4() {
        let notes: HashSet<u8> = [60, 65, 67, 70].into_iter().collect();
        assert_eq!(detect_chord(&notes), Some("C7sus4".to_string()));
    }

    // === Altered dominant tests ===

    #[test]
    fn test_7sharp11() {
        let notes: HashSet<u8> = [60, 64, 66, 67, 70].into_iter().collect();
        assert_eq!(detect_chord(&notes), Some("C7#11".to_string()));
    }

    #[test]
    fn test_power_chord() {
        let notes: HashSet<u8> = [60, 67].into_iter().collect();
        assert_eq!(detect_chord(&notes), Some("C5".to_string()));
    }
}
