//! Barry Harris 6th Diminished harmony mode.
//!
//! Implements proper BH voicing using the 8-note scale's chord-tone/passing-tone
//! parity system. Produces 4-voice drop-2 voicings where even-degree notes
//! (0,2,4,6) generate 6th chord voicings and odd-degree notes (1,3,5,7) generate
//! diminished 7th voicings.

use wmidi::Note;

use crate::config::{BeatPhase, ScaleMode};
use crate::scale::Scale;

/// Chord-tone vs passing-tone classification.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Parity {
    /// Even-indexed scale degree (0, 2, 4, 6) -- part of the 6th chord.
    ChordTone,
    /// Odd-indexed scale degree (1, 3, 5, 7) -- part of the diminished 7th chord.
    PassingTone,
}

/// Returns the parity of a note in the BH scale.
/// Even degree (0,2,4,6) = chord tone, odd degree (1,3,5,7) = passing tone.
/// Returns None if the note is not in the scale.
pub fn note_parity(note: Note, scale: &Scale) -> Option<Parity> {
    let degree = scale.degree_of(note)?;
    if degree % 2 == 0 {
        Some(Parity::ChordTone)
    } else {
        Some(Parity::PassingTone)
    }
}

/// Result of validating whether the current scale is appropriate for BH mode.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BhScaleGuard {
    /// Scale is a BH scale, proceed normally.
    Valid,
    /// Scale is not BH; contains the suggested BH scale to auto-switch to.
    Fallback(ScaleMode),
}

/// Validates that the current scale is a BH 8-note scale.
/// If not, suggests an appropriate BH scale based on the current mode's tonality.
pub fn validate_scale(scale_mode: ScaleMode) -> BhScaleGuard {
    match scale_mode {
        ScaleMode::BHMajor6thDim | ScaleMode::BHMinor6thDim => BhScaleGuard::Valid,
        _ => BhScaleGuard::Fallback(infer_bh_scale(scale_mode)),
    }
}

/// Infers the best BH scale from the current scale mode's tonality.
fn infer_bh_scale(current: ScaleMode) -> ScaleMode {
    use crate::config::ScaleFamily;
    match current.family() {
        ScaleFamily::Diatonic => match current {
            ScaleMode::Aeolian | ScaleMode::Dorian | ScaleMode::Phrygian | ScaleMode::Locrian => {
                ScaleMode::BHMinor6thDim
            }
            _ => ScaleMode::BHMajor6thDim,
        },
        ScaleFamily::HarmonicMinor | ScaleFamily::MelodicMinor => ScaleMode::BHMinor6thDim,
        _ => ScaleMode::BHMajor6thDim,
    }
}

/// Builds a 4-voice drop-2 voicing for the given note in a BH scale.
///
/// # Algorithm
/// 1. Find the note's degree in the 8-note scale
/// 2. Build close-position chord: [d, d+2, d+4, d+6] (mod 8) -- preserves parity
/// 3. Find MIDI realizations near the input note
/// 4. Sort ascending (close position)
/// 5. Apply drop-2: move second-from-top down an octave
///
/// # Returns
/// 4 MIDI note values as the drop-2 voicing, or None if the note is not in scale
/// or the voicing would fall outside MIDI range.
pub fn build_voicing(note: Note, scale: &Scale, _beat_phase: BeatPhase) -> Option<[Note; 4]> {
    let input_midi = u8::from(note);
    let input_degree = scale.degree_of(note)?;

    // Must be an 8-note scale
    if scale.scale_len() != 8 {
        return None;
    }

    let voicing = build_drop2_voicing(input_midi, input_degree, scale)?;

    // Convert to Note values
    let n0 = Note::try_from(voicing[0]).ok()?;
    let n1 = Note::try_from(voicing[1]).ok()?;
    let n2 = Note::try_from(voicing[2]).ok()?;
    let n3 = Note::try_from(voicing[3]).ok()?;
    Some([n0, n1, n2, n3])
}

/// Core drop-2 voicing math. Returns 4 MIDI values.
fn build_drop2_voicing(input_midi: u8, input_degree: usize, scale: &Scale) -> Option<[u8; 4]> {
    // Step 1: Get 4 chord degrees (parity-preserving, all same parity)
    let degrees = [
        input_degree,
        (input_degree + 2) % 8,
        (input_degree + 4) % 8,
        (input_degree + 6) % 8,
    ];

    // Step 2: Find MIDI values for each degree, closest to input
    let mut voices = [0u8; 4];
    for (i, &deg) in degrees.iter().enumerate() {
        voices[i] = scale.degree_to_midi_near(deg, input_midi)?;
    }

    // Step 3: Sort ascending (close position)
    voices.sort();

    // Step 4: Drop-2 transformation
    // Second from top (index 2) drops an octave
    let dropped = voices[2].checked_sub(12)?;

    Some([dropped, voices[0], voices[1], voices[3]])
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bh_major_c() -> Scale {
        Scale::new(0, ScaleMode::BHMajor6thDim)
    }

    fn bh_minor_c() -> Scale {
        Scale::new(0, ScaleMode::BHMinor6thDim)
    }

    #[test]
    fn test_parity_chord_tones() {
        let scale = bh_major_c();
        // Even degrees: C(0), E(2), G(4), A(6) are chord tones
        assert_eq!(note_parity(Note::C4, &scale), Some(Parity::ChordTone));
        assert_eq!(note_parity(Note::E4, &scale), Some(Parity::ChordTone));
        assert_eq!(note_parity(Note::G4, &scale), Some(Parity::ChordTone));
        assert_eq!(note_parity(Note::A4, &scale), Some(Parity::ChordTone));
    }

    #[test]
    fn test_parity_passing_tones() {
        let scale = bh_major_c();
        // Odd degrees: D(1), F(3), Ab(5), B(7) are passing tones
        assert_eq!(note_parity(Note::D4, &scale), Some(Parity::PassingTone));
        assert_eq!(note_parity(Note::F4, &scale), Some(Parity::PassingTone));
        assert_eq!(note_parity(Note::Ab4, &scale), Some(Parity::PassingTone));
        assert_eq!(note_parity(Note::B4, &scale), Some(Parity::PassingTone));
    }

    #[test]
    fn test_parity_chromatic_note() {
        let scale = bh_major_c();
        // Db is not in the BH major scale
        assert_eq!(note_parity(Note::Db4, &scale), None);
    }

    #[test]
    fn test_scale_guard_valid() {
        assert_eq!(
            validate_scale(ScaleMode::BHMajor6thDim),
            BhScaleGuard::Valid
        );
        assert_eq!(
            validate_scale(ScaleMode::BHMinor6thDim),
            BhScaleGuard::Valid
        );
    }

    #[test]
    fn test_scale_guard_fallback_major() {
        assert_eq!(
            validate_scale(ScaleMode::Ionian),
            BhScaleGuard::Fallback(ScaleMode::BHMajor6thDim)
        );
        assert_eq!(
            validate_scale(ScaleMode::Lydian),
            BhScaleGuard::Fallback(ScaleMode::BHMajor6thDim)
        );
    }

    #[test]
    fn test_scale_guard_fallback_minor() {
        assert_eq!(
            validate_scale(ScaleMode::Aeolian),
            BhScaleGuard::Fallback(ScaleMode::BHMinor6thDim)
        );
        assert_eq!(
            validate_scale(ScaleMode::HarmonicMinor),
            BhScaleGuard::Fallback(ScaleMode::BHMinor6thDim)
        );
    }

    #[test]
    fn test_drop2_c6_root_position() {
        let scale = bh_major_c();
        let beat = BeatPhase::default();

        // C4 = degree 0 (chord tone)
        let result = build_voicing(Note::C4, &scale, beat);
        assert!(result.is_some(), "Should produce voicing for C4");
        let voicing = result.unwrap();
        // All 4 notes should be valid
        assert_eq!(voicing.len(), 4);
    }

    #[test]
    fn test_drop2_produces_4_notes() {
        let scale = bh_major_c();
        let beat = BeatPhase::default();

        // Test with every scale degree
        let test_notes = [
            Note::C4,
            Note::D4,
            Note::E4,
            Note::F4,
            Note::G4,
            Note::Ab4,
            Note::A4,
            Note::B4,
        ];

        for note in &test_notes {
            let result = build_voicing(*note, &scale, beat);
            assert!(result.is_some(), "Should produce voicing for {:?}", note);
        }
    }

    #[test]
    fn test_parity_preserved_in_voicing() {
        let scale = bh_major_c();
        let beat = BeatPhase::default();

        // Chord tone input (C4, degree 0) -> all voices should be chord tones
        let voicing = build_voicing(Note::C4, &scale, beat).unwrap();
        for voice in &voicing {
            let parity = note_parity(*voice, &scale);
            assert_eq!(
                parity,
                Some(Parity::ChordTone),
                "Voice {:?} should be chord tone",
                voice
            );
        }

        // Passing tone input (D4, degree 1) -> all voices should be passing tones
        let voicing = build_voicing(Note::D4, &scale, beat).unwrap();
        for voice in &voicing {
            let parity = note_parity(*voice, &scale);
            assert_eq!(
                parity,
                Some(Parity::PassingTone),
                "Voice {:?} should be passing tone",
                voice
            );
        }
    }

    #[test]
    fn test_non_bh_scale_returns_none() {
        let scale = Scale::major(0); // 7-note scale
        let beat = BeatPhase::default();
        assert_eq!(build_voicing(Note::C4, &scale, beat), None);
    }

    #[test]
    fn test_chromatic_note_returns_none() {
        let scale = bh_major_c();
        let beat = BeatPhase::default();
        // Db4 is not in C BH major scale
        assert_eq!(build_voicing(Note::Db4, &scale, beat), None);
    }

    #[test]
    fn test_bh_minor_voicing() {
        let scale = bh_minor_c();
        let beat = BeatPhase::default();
        // C4 in BH minor: degrees [0,2,4,6] = [C, Eb, G, A] = Cm6
        let result = build_voicing(Note::C4, &scale, beat);
        assert!(result.is_some());
    }

    #[test]
    fn test_bh_minor_parity_preserved() {
        let scale = bh_minor_c();
        let beat = BeatPhase::default();

        // Chord tone input (C4, degree 0)
        let voicing = build_voicing(Note::C4, &scale, beat).unwrap();
        for voice in &voicing {
            let parity = note_parity(*voice, &scale);
            assert_eq!(
                parity,
                Some(Parity::ChordTone),
                "Voice {:?} should be chord tone in minor BH",
                voice
            );
        }
    }
}
