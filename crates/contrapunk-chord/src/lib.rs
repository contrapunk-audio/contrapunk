//! Chord detection module for Contrapunk.
//!
//! Analyzes MIDI note combinations to identify common chord types.
//! Supports extended chords (9th, 11th, 13th), altered dominants,
//! slash chords, add chords, 6th chords, and roman numeral analysis.

use std::collections::HashSet;

/// Note names for display (C, C#, D, ..., B).
const NOTE_NAMES: [&str; 12] = [
    "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B",
];

/// Flat-key note names for context-aware enharmonic spelling.
const NOTE_NAMES_FLAT: [&str; 12] = [
    "C", "Db", "D", "Eb", "E", "F", "Gb", "G", "Ab", "A", "Bb", "B",
];

/// Common chord types defined by intervals from root (in semitones).
struct ChordPattern {
    name: &'static str,
    intervals: &'static [u8],
}

/// Known chord patterns ordered by specificity (longer patterns first).
const CHORD_PATTERNS: &[ChordPattern] = &[
    // 13th chords (6 notes)
    ChordPattern {
        name: "maj13",
        intervals: &[0, 2, 4, 7, 9, 11],
    },
    ChordPattern {
        name: "13",
        intervals: &[0, 2, 4, 7, 9, 10],
    },
    ChordPattern {
        name: "min13",
        intervals: &[0, 2, 3, 7, 9, 10],
    },
    // 11th chords (5 notes)
    ChordPattern {
        name: "maj9#11",
        intervals: &[0, 2, 4, 6, 7, 11],
    },
    ChordPattern {
        name: "11",
        intervals: &[0, 2, 4, 5, 7, 10],
    },
    ChordPattern {
        name: "min11",
        intervals: &[0, 2, 3, 5, 7, 10],
    },
    // 6/9 chord (5 notes)
    ChordPattern {
        name: "6/9",
        intervals: &[0, 2, 4, 7, 9],
    },
    // 9th chords (5 notes)
    ChordPattern {
        name: "maj9",
        intervals: &[0, 2, 4, 7, 11],
    },
    ChordPattern {
        name: "9",
        intervals: &[0, 2, 4, 7, 10],
    },
    ChordPattern {
        name: "min9",
        intervals: &[0, 2, 3, 7, 10],
    },
    ChordPattern {
        name: "7b9",
        intervals: &[0, 1, 4, 7, 10],
    },
    ChordPattern {
        name: "7#9",
        intervals: &[0, 3, 4, 7, 10],
    },
    // Altered dominants (4-5 notes)
    ChordPattern {
        name: "7#11",
        intervals: &[0, 4, 6, 7, 10],
    },
    ChordPattern {
        name: "7b13",
        intervals: &[0, 4, 7, 8, 10],
    },
    ChordPattern {
        name: "7alt",
        intervals: &[0, 1, 4, 8, 10],
    },
    // 6th chords (4 notes)
    ChordPattern {
        name: "maj6",
        intervals: &[0, 4, 7, 9],
    },
    ChordPattern {
        name: "min6",
        intervals: &[0, 3, 7, 9],
    },
    // Add chords (4 notes)
    ChordPattern {
        name: "add9",
        intervals: &[0, 2, 4, 7],
    },
    ChordPattern {
        name: "madd9",
        intervals: &[0, 2, 3, 7],
    },
    ChordPattern {
        name: "add11",
        intervals: &[0, 4, 5, 7],
    },
    // Seventh chords (4 notes)
    ChordPattern {
        name: "maj7",
        intervals: &[0, 4, 7, 11],
    },
    ChordPattern {
        name: "7",
        intervals: &[0, 4, 7, 10],
    },
    ChordPattern {
        name: "min7",
        intervals: &[0, 3, 7, 10],
    },
    ChordPattern {
        name: "dim7",
        intervals: &[0, 3, 6, 9],
    },
    ChordPattern {
        name: "m7b5",
        intervals: &[0, 3, 6, 10],
    },
    ChordPattern {
        name: "minmaj7",
        intervals: &[0, 3, 7, 11],
    },
    ChordPattern {
        name: "aug7",
        intervals: &[0, 4, 8, 10],
    },
    ChordPattern {
        name: "augmaj7",
        intervals: &[0, 4, 8, 11],
    },
    ChordPattern {
        name: "7sus4",
        intervals: &[0, 5, 7, 10],
    },
    ChordPattern {
        name: "7sus2",
        intervals: &[0, 2, 7, 10],
    },
    // Triads (3 notes)
    ChordPattern {
        name: "maj",
        intervals: &[0, 4, 7],
    },
    ChordPattern {
        name: "min",
        intervals: &[0, 3, 7],
    },
    ChordPattern {
        name: "dim",
        intervals: &[0, 3, 6],
    },
    ChordPattern {
        name: "aug",
        intervals: &[0, 4, 8],
    },
    ChordPattern {
        name: "sus4",
        intervals: &[0, 5, 7],
    },
    ChordPattern {
        name: "sus2",
        intervals: &[0, 2, 7],
    },
    // Power chord (2 notes)
    ChordPattern {
        name: "5",
        intervals: &[0, 7],
    },
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
        let intervals: HashSet<u8> = pcs.iter().map(|&pc| (pc + 12 - root) % 12).collect();

        for (pidx, pattern) in CHORD_PATTERNS.iter().enumerate() {
            let pattern_set: HashSet<u8> = pattern.intervals.iter().copied().collect();
            if intervals == pattern_set {
                let root_is_bass = root == bass_pc;
                let ilen = pattern.intervals.len();
                // Compare: prefer more intervals, then root==bass, then earlier pattern
                let dominated = if let Some((best_ilen, best_rib, best_pidx, _, _)) = best {
                    if ilen > best_ilen {
                        false
                    } else if ilen < best_ilen {
                        true
                    } else if root_is_bass && !best_rib {
                        false
                    } else if !root_is_bass && best_rib {
                        true
                    } else {
                        pidx >= best_pidx
                    }
                } else {
                    false
                };
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

/// Chord-tone label for a given interval in semitones from the root.
/// Used by both the partial-chord "(no X)" omission suffix and the
/// intervals-fallback display. Uses Unicode music-notation accidentals
/// (♭, ♯) so output matches the rest of the UI's notation surfaces.
fn interval_label(semis: u8) -> &'static str {
    match semis % 12 {
        0 => "R",
        1 => "♭9",
        2 => "9",
        3 => "♭3",
        4 => "3",
        5 => "11",
        6 => "♭5",
        7 => "5",
        8 => "♭13",
        9 => "13",
        10 => "♭7",
        11 => "7",
        _ => "?",
    }
}

/// Try to identify the input as a recognized chord pattern with up to
/// 2 missing chord tones, returning the chord name with `(no X)` or
/// `(no X, Y)` suffix per common jazz lead-sheet convention.
///
/// Used as a fallback when `detect_chord` returns None — captures the
/// musical intent of partial voicings (shell voicings of jazz 7ths,
/// power chords with the 5th omitted, etc.) instead of just listing
/// pitch classes. Examples:
///   - {C, E, B}      → "Cmaj7(no 5)"
///   - {C, G, B♭}     → "C7(no 3)"
///   - {C, E♭, B♭, D} → "Cmin9(no 5)"
///
/// Tie-break: prefer fewer missing tones, then earlier pattern in the
/// CHORD_PATTERNS table (which is roughly ordered by specificity, more
/// intervals first).
fn detect_partial_chord(notes: &HashSet<u8>) -> Option<String> {
    if notes.len() < 2 {
        return None;
    }

    let bass = *notes.iter().min().unwrap();
    let bass_pc = bass % 12;
    let pcs: HashSet<u8> = notes.iter().map(|n| n % 12).collect();
    if pcs.len() < 2 {
        return None;
    }

    // (missing_count, pattern_idx, root, name, missing_intervals)
    let mut best: Option<(usize, usize, u8, &str, Vec<u8>)> = None;

    for &root in &pcs {
        let input_intervals: HashSet<u8> = pcs.iter().map(|&pc| (pc + 12 - root) % 12).collect();

        // Partial-match only makes musical sense when the candidate root
        // is actually present in the input — otherwise we'd be naming a
        // chord by a pitch the user didn't play.
        if !input_intervals.contains(&0) {
            continue;
        }

        for (pidx, pattern) in CHORD_PATTERNS.iter().enumerate() {
            let pattern_set: HashSet<u8> = pattern.intervals.iter().copied().collect();
            if !input_intervals.is_subset(&pattern_set) {
                continue;
            }
            let mut missing: Vec<u8> = pattern
                .intervals
                .iter()
                .filter(|i| !input_intervals.contains(i))
                .copied()
                .collect();
            missing.sort();
            let missing_count = missing.len();
            // Cap omissions at 2 — naming a 6-tone chord with 4 missing
            // tones isn't useful to the reader. Such cases fall through
            // to the intervals-notation fallback in chord_display.
            if missing_count == 0 || missing_count > 2 {
                continue;
            }

            let better = match &best {
                None => true,
                Some((bm, bp, _, _, _)) => {
                    missing_count < *bm || (missing_count == *bm && pidx < *bp)
                }
            };
            if better {
                best = Some((missing_count, pidx, root, pattern.name, missing));
            }
        }
    }

    best.map(|(_, _, root, name, missing)| {
        let root_name = NOTE_NAMES[root as usize];
        let parts: Vec<&str> = missing.iter().map(|&i| interval_label(i)).collect();
        let suffix = format!("(no {})", parts.join(", "));
        if bass_pc != root {
            let bass_name = NOTE_NAMES[bass_pc as usize];
            format!("{}{}{}/{}", root_name, name, suffix, bass_name)
        } else {
            format!("{}{}{}", root_name, name, suffix)
        }
    })
}

/// Final fallback: bass note + space-separated intervals-from-bass.
/// e.g. {C, D♭, E, G♭} → "C ♭9 3 ♭5". Visually distinct from a real
/// chord name (no maj/min/sus suffix, intervals listed explicitly with
/// spaces) so the reader knows it's an unidentified note set.
fn notes_as_intervals(notes: &HashSet<u8>) -> String {
    if notes.is_empty() {
        return String::new();
    }
    let bass = *notes.iter().min().unwrap();
    let bass_pc = bass % 12;
    let pcs: HashSet<u8> = notes.iter().map(|n| n % 12).collect();
    let mut intervals: Vec<u8> = pcs
        .iter()
        .map(|&pc| (pc + 12 - bass_pc) % 12)
        .filter(|&i| i != 0)
        .collect();
    intervals.sort();
    let root_name = NOTE_NAMES[bass_pc as usize];
    if intervals.is_empty() {
        return root_name.to_string();
    }
    let parts: Vec<&str> = intervals.iter().map(|&i| interval_label(i)).collect();
    format!("{} {}", root_name, parts.join(" "))
}

/// Returns a display string for the chord, or special indicators.
///
/// Three-tier chain so the chord readout always carries information:
///   1. `detect_chord` — exact pattern match, e.g. "Cmaj7"
///   2. `detect_partial_chord` — closest superset with up to 2 missing
///      chord tones, e.g. "Cmaj7(no 5)"
///   3. `notes_as_intervals` — bass + intervals notation when no chord
///      pattern is even close, e.g. "C ♭9 3 ♭5"
///
/// Empty notes return em-dash.
pub fn chord_display(notes: &HashSet<u8>) -> String {
    if notes.is_empty() {
        return "\u{2014}".to_string();
    }
    detect_chord(notes)
        .or_else(|| detect_partial_chord(notes))
        .unwrap_or_else(|| notes_as_intervals(notes))
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
    }
    .to_string()
}

/// Returns chord name with roman numeral analysis in a given key.
///
/// Format: "Fmaj7 (IVmaj7 in C)"
pub fn chord_display_with_analysis(notes: &HashSet<u8>, key_tonic: Option<u8>) -> String {
    if notes.is_empty() {
        return "\u{2014}".to_string();
    }

    // Mirror chord_display's three-tier chain. Roman numeral analysis
    // only attaches to results that came from detect_chord OR
    // detect_partial_chord (both produce parseable Root+quality strings);
    // the intervals fallback returns a space-separated form that doesn't
    // play well with the analysis suffix, so we hand it back as-is.
    let (chord_str, has_chord_name) = if let Some(s) = detect_chord(notes) {
        (s, true)
    } else if let Some(s) = detect_partial_chord(notes) {
        (s, true)
    } else {
        (notes_as_intervals(notes), false)
    };

    if has_chord_name {
        if let Some(tonic) = key_tonic {
            let root_pc = parse_root_from_chord(&chord_str);
            if let Some(rpc) = root_pc {
                let rn = roman_numeral(rpc, tonic);
                let quality = extract_quality(&chord_str);
                let key_name = NOTE_NAMES[tonic as usize];
                return format!("{} ({}{} in {})", chord_str, rn, quality, key_name);
            }
        }
    }

    chord_str
}

/// Parse the root pitch class from a chord name string.
fn parse_root_from_chord(chord: &str) -> Option<u8> {
    if chord.len() >= 2 && (chord.as_bytes()[1] == b'#' || chord.as_bytes()[1] == b'b') {
        let root_str = &chord[..2];
        NOTE_NAMES
            .iter()
            .position(|&n| n == root_str)
            .or_else(|| NOTE_NAMES_FLAT.iter().position(|&n| n == root_str))
            .map(|i| i as u8)
    } else {
        let root_str = &chord[..1];
        NOTE_NAMES
            .iter()
            .position(|&n| n == root_str)
            .map(|i| i as u8)
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
    fn test_chord_display_chromatic_cluster_falls_to_intervals() {
        // Three adjacent semitones {C, C♯, D} — no chord pattern in the
        // table contains an interval set with three consecutive
        // semitones for ANY root choice (chord patterns always include
        // a 3rd or larger gap somewhere), so even the partial-chord
        // pass can't superset-match within the 2-omission cap. Drops
        // through to the intervals-notation fallback.
        let notes: HashSet<u8> = [60, 61, 62].into_iter().collect();
        assert_eq!(chord_display(&notes), "C ♭9 9");
    }

    #[test]
    fn test_chord_display_partial_slash_voicing() {
        // {C, C♯} — partial-chord pass DOES find a valid slash
        // interpretation: C♯maj7 (intervals 0, 4, 7, 11 from C♯) with
        // the 3rd and 5th omitted, voiced over a C bass. Esoteric but
        // musically real (cluster jazz voicing). Locks the slash-form
        // partial match so future changes don't accidentally break the
        // bass-different-from-root path. Note: NOTE_NAMES uses ASCII
        // '#' on the Rust side (engine wire format); the UI's
        // formatMusicalString converts '#' → '♯' at display time.
        let notes: HashSet<u8> = [60, 61].into_iter().collect();
        assert_eq!(chord_display(&notes), "C#maj7(no 3, 5)/C");
    }

    #[test]
    fn test_chord_display_partial_maj7_no_5() {
        // {C, E, B} — Cmaj7 with the 5th omitted. Common shell voicing.
        // detect_chord finds nothing exact; detect_partial_chord names it.
        let notes: HashSet<u8> = [60, 64, 71].into_iter().collect();
        assert_eq!(chord_display(&notes), "Cmaj7(no 5)");
    }

    #[test]
    fn test_chord_display_partial_7_no_3() {
        // {C, G, B♭} — C7 with the 3rd omitted. The pattern table also
        // contains "5" for {C, G}, but the 4-tone superset match wins
        // because it covers more of the input intent (the B♭ is the
        // defining flat-7 of a dominant chord).
        let notes: HashSet<u8> = [60, 67, 70].into_iter().collect();
        assert_eq!(chord_display(&notes), "C7(no 3)");
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
