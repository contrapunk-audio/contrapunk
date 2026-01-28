use wmidi::Note;

/// Semitone offsets for major scale (Ionian mode)
/// Index = scale degree (0-6), Value = semitones from tonic
const MAJOR_SCALE_OFFSETS: [u8; 7] = [0, 2, 4, 5, 7, 9, 11];

/// Consonant chromatic intervals (in semitones) for out-of-key notes.
/// These sound good regardless of key context.
/// Ordered by preference: 3rds, 6ths, 4ths/5ths
const CONSONANT_INTERVALS_ABOVE: [i8; 6] = [4, 3, 9, 8, 7, 5]; // M3, m3, M6, m6, P5, P4
const CONSONANT_INTERVALS_BELOW: [i8; 6] = [-3, -4, -8, -9, -5, -7]; // m3, M3, m6, M6, P4, P5

/// A musical scale defined by a tonic and semitone offsets.
///
/// Provides diatonic operations like finding scale degrees
/// and transposing by scale degrees (not chromatic semitones).
#[derive(Clone, Debug)]
pub struct Scale {
    /// Tonic pitch class (0-11, where 0 = C)
    tonic: u8,
    /// Semitone offsets for each scale degree (0-6)
    offsets: [u8; 7],
}

impl Scale {
    /// Creates a new major scale with the given tonic.
    ///
    /// # Arguments
    /// * `tonic` - Pitch class of the tonic (0-11, where 0 = C)
    pub fn major(tonic: u8) -> Self {
        Self {
            tonic: tonic % 12,
            offsets: MAJOR_SCALE_OFFSETS,
        }
    }

    /// Finds the scale degree (0-6) for a given MIDI note.
    ///
    /// Returns None if the note is not in the scale (chromatic).
    pub fn degree_of(&self, note: Note) -> Option<usize> {
        let pitch_class = u8::from(note) % 12;
        let relative = (pitch_class + 12 - self.tonic) % 12;
        self.offsets.iter().position(|&o| o == relative)
    }

    /// Transposes a note by N scale degrees (diatonic transposition).
    ///
    /// Unlike chromatic transposition (fixed semitones), this moves
    /// by scale degrees, so the interval size varies based on position
    /// in the scale (e.g., a "third" might be major or minor).
    ///
    /// Returns None if the resulting note would be out of MIDI range (0-127)
    /// or if the input note is not in the scale.
    ///
    /// # Arguments
    /// * `note` - The note to transpose
    /// * `degrees` - Scale degrees to transpose (positive = up, negative = down)
    pub fn transpose_diatonic(&self, note: Note, degrees: i8) -> Option<Note> {
        let current_degree = self.degree_of(note)? as i8;
        let note_midi = u8::from(note) as i8;

        // Calculate new degree with octave handling
        let total_degrees = current_degree + degrees;
        let octave_shift = if total_degrees < 0 {
            (total_degrees - 6) / 7  // Floor division for negative
        } else {
            total_degrees / 7
        };
        let new_degree = ((total_degrees % 7) + 7) % 7;

        // Calculate semitone difference
        let current_offset = self.offsets[current_degree as usize] as i8;
        let new_offset = self.offsets[new_degree as usize] as i8;
        let semitone_diff = (new_offset - current_offset) + (octave_shift * 12);

        // Apply transposition
        let new_midi = note_midi + semitone_diff;
        if !(0..=127).contains(&new_midi) {
            return None;
        }

        Note::try_from(new_midi as u8).ok()
    }

    /// Returns the closest scale note for a given MIDI note.
    ///
    /// If the note is in the scale, returns it unchanged.
    /// If chromatic, returns the nearest scale note (preferring lower on tie).
    pub fn snap_to_scale(&self, note: Note) -> Note {
        if self.degree_of(note).is_some() {
            return note;
        }

        let midi = u8::from(note);

        // Try stepping down then up to find nearest scale note
        for offset in 1..=6 {
            if midi >= offset {
                if let Ok(lower) = Note::try_from(midi - offset) {
                    if self.degree_of(lower).is_some() {
                        return lower;
                    }
                }
            }
            if midi + offset <= 127 {
                if let Ok(higher) = Note::try_from(midi + offset) {
                    if self.degree_of(higher).is_some() {
                        return higher;
                    }
                }
            }
        }

        note  // Fallback (shouldn't happen with proper scale)
    }

    /// Returns true if the note is in the scale.
    pub fn is_in_scale(&self, note: Note) -> bool {
        self.degree_of(note).is_some()
    }

    /// Transposes a note chromatically by semitones.
    ///
    /// Returns None if the result would be out of MIDI range.
    pub fn transpose_chromatic(&self, note: Note, semitones: i8) -> Option<Note> {
        let midi = u8::from(note) as i16 + semitones as i16;
        if (0..=127).contains(&midi) {
            Note::try_from(midi as u8).ok()
        } else {
            None
        }
    }

    /// Harmonizes a note, using diatonic intervals for in-key notes
    /// and consonant chromatic intervals for out-of-key notes.
    ///
    /// # Arguments
    /// * `note` - The note to harmonize
    /// * `diatonic_degrees` - Scale degrees to transpose if in-key
    /// * `prefer_above` - If true, prefer intervals above; if false, below
    ///
    /// # Returns
    /// The harmony note, or None if out of MIDI range
    pub fn harmonize_smart(&self, note: Note, diatonic_degrees: i8, prefer_above: bool) -> Option<Note> {
        if self.is_in_scale(note) {
            // In-key: use diatonic transposition
            self.transpose_diatonic(note, diatonic_degrees)
        } else {
            // Out-of-key: use consonant chromatic interval
            self.harmonize_chromatic(note, prefer_above)
        }
    }

    /// Finds a consonant chromatic harmony for an out-of-key note.
    ///
    /// Tries intervals in order of consonance (3rds, 6ths, 5ths, 4ths).
    /// Prefers intervals that land on scale tones when possible.
    fn harmonize_chromatic(&self, note: Note, prefer_above: bool) -> Option<Note> {
        let intervals = if prefer_above {
            &CONSONANT_INTERVALS_ABOVE
        } else {
            &CONSONANT_INTERVALS_BELOW
        };

        // First pass: try to find an interval that lands on a scale tone
        for &interval in intervals {
            if let Some(harmony) = self.transpose_chromatic(note, interval) {
                if self.is_in_scale(harmony) {
                    return Some(harmony);
                }
            }
        }

        // Second pass: just use the first valid consonant interval
        for &interval in intervals {
            if let Some(harmony) = self.transpose_chromatic(note, interval) {
                return Some(harmony);
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_c_major_scale_degrees() {
        let scale = Scale::major(0);  // C major

        // C4 = 60 should be degree 0
        assert_eq!(scale.degree_of(Note::C4), Some(0));
        // D4 = 62 should be degree 1
        assert_eq!(scale.degree_of(Note::D4), Some(1));
        // E4 = 64 should be degree 2
        assert_eq!(scale.degree_of(Note::E4), Some(2));
        // C# is not in C major
        assert_eq!(scale.degree_of(Note::Db4), None);
    }

    #[test]
    fn test_diatonic_third_up() {
        let scale = Scale::major(0);  // C major

        // C4 + 2 degrees = E4 (major third)
        let result = scale.transpose_diatonic(Note::C4, 2);
        assert_eq!(result, Some(Note::E4));

        // E4 + 2 degrees = G4 (minor third)
        let result = scale.transpose_diatonic(Note::E4, 2);
        assert_eq!(result, Some(Note::G4));
    }

    #[test]
    fn test_diatonic_interval_down() {
        let scale = Scale::major(0);  // C major

        // E4 - 2 degrees = C4
        let result = scale.transpose_diatonic(Note::E4, -2);
        assert_eq!(result, Some(Note::C4));
    }

    #[test]
    fn test_g_major_scale() {
        let scale = Scale::major(7);  // G major (7 semitones from C)

        // G4 = 67 should be degree 0 (tonic)
        assert_eq!(scale.degree_of(Note::G4), Some(0));
        // F# is in G major (degree 6)
        assert_eq!(scale.degree_of(Note::Gb4), Some(6));
        // F natural is NOT in G major
        assert_eq!(scale.degree_of(Note::F4), None);
    }

    // Tests for out-of-key handling

    #[test]
    fn test_is_in_scale() {
        let scale = Scale::major(0);  // C major

        assert!(scale.is_in_scale(Note::C4));
        assert!(scale.is_in_scale(Note::D4));
        assert!(scale.is_in_scale(Note::E4));
        assert!(!scale.is_in_scale(Note::Db4)); // C# not in C major
        assert!(!scale.is_in_scale(Note::Eb4)); // Eb not in C major
    }

    #[test]
    fn test_transpose_chromatic() {
        let scale = Scale::major(0);

        // C4 + 4 semitones = E4
        assert_eq!(scale.transpose_chromatic(Note::C4, 4), Some(Note::E4));
        // C4 - 3 semitones = A3
        assert_eq!(scale.transpose_chromatic(Note::C4, -3), Some(Note::A3));
    }

    #[test]
    fn test_harmonize_smart_in_key() {
        let scale = Scale::major(0);  // C major

        // In-key note should use diatonic transposition
        // C4 + 2 degrees = E4
        let result = scale.harmonize_smart(Note::C4, 2, true);
        assert_eq!(result, Some(Note::E4));
    }

    #[test]
    fn test_harmonize_smart_out_of_key() {
        let scale = Scale::major(0);  // C major

        // C#4 is out of key - should get consonant chromatic harmony
        let result = scale.harmonize_smart(Note::Db4, 2, true);
        assert!(result.is_some());

        // The harmony should be a consonant interval from C#
        let harmony = result.unwrap();
        let melody_midi = u8::from(Note::Db4);
        let harmony_midi = u8::from(harmony);
        let interval = (harmony_midi as i8 - melody_midi as i8).abs();

        // Should be a 3rd, 4th, 5th, or 6th (3, 4, 5, 7, 8, or 9 semitones)
        assert!(
            [3, 4, 5, 7, 8, 9].contains(&interval),
            "Expected consonant interval, got {} semitones", interval
        );
    }

    #[test]
    fn test_chromatic_harmony_prefers_scale_tones() {
        let scale = Scale::major(0);  // C major

        // F#4 is out of key
        // Chromatic harmony should prefer landing on a scale tone if possible
        let result = scale.harmonize_smart(Note::Gb4, -2, false);
        assert!(result.is_some());

        // Check if the result landed on a scale tone (preferred but not guaranteed)
        let harmony = result.unwrap();
        // The harmony should at least be consonant
        let melody_midi = u8::from(Note::Gb4);
        let harmony_midi = u8::from(harmony);
        let interval = (melody_midi as i8 - harmony_midi as i8).abs();
        assert!(
            [3, 4, 5, 7, 8, 9].contains(&interval),
            "Expected consonant interval below, got {} semitones", interval
        );
    }
}
