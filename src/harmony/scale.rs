use wmidi::Note;

use crate::harmony::config::ScaleMode;

/// Consonant chromatic intervals (in semitones) for out-of-key notes.
/// These sound good regardless of key context.
/// Ordered by preference: 3rds, 6ths, 4ths/5ths
const CONSONANT_INTERVALS_ABOVE: [i8; 6] = [4, 3, 9, 8, 7, 5]; // M3, m3, M6, m6, P5, P4
const CONSONANT_INTERVALS_BELOW: [i8; 6] = [-3, -4, -8, -9, -5, -7]; // m3, M3, m6, M6, P4, P5

/// Borrowing sources by range level (1-5).
/// Each level includes all modes from previous levels plus new ones.
fn borrowing_sources(range: u8) -> &'static [ScaleMode] {
    match range {
        1 => &[ScaleMode::Aeolian, ScaleMode::HarmonicMinor],
        2 => &[
            ScaleMode::Aeolian,
            ScaleMode::Dorian,
            ScaleMode::HarmonicMinor,
            ScaleMode::MelodicMinor,
        ],
        3 => &[
            ScaleMode::Aeolian,
            ScaleMode::Dorian,
            ScaleMode::Mixolydian,
            ScaleMode::Phrygian,
            ScaleMode::HarmonicMinor,
            ScaleMode::MelodicMinor,
        ],
        4 => &[
            ScaleMode::Aeolian,
            ScaleMode::Dorian,
            ScaleMode::Mixolydian,
            ScaleMode::Phrygian,
            ScaleMode::Lydian,
            ScaleMode::HarmonicMinor,
            ScaleMode::MelodicMinor,
            ScaleMode::PhrygianDominant,
        ],
        _ => &[
            ScaleMode::Aeolian,
            ScaleMode::Dorian,
            ScaleMode::Mixolydian,
            ScaleMode::Phrygian,
            ScaleMode::Lydian,
            ScaleMode::Locrian,
            ScaleMode::Ionian,
            ScaleMode::HarmonicMinor,
            ScaleMode::MelodicMinor,
            ScaleMode::PhrygianDominant,
            ScaleMode::LydianDominant,
        ],
    }
}

/// A musical scale defined by a tonic, mode, and semitone offsets.
///
/// Provides diatonic operations like finding scale degrees
/// and transposing by scale degrees (not chromatic semitones).
#[derive(Clone, Debug)]
pub struct Scale {
    /// Tonic pitch class (0-11, where 0 = C)
    tonic: u8,
    /// The scale mode
    mode: ScaleMode,
    /// Semitone offsets for each scale degree (variable length: 7 or 8)
    offsets: Vec<u8>,
    /// Whether modal interchange is enabled for out-of-key notes
    interchange_enabled: bool,
    /// How many parallel modes to search (1-5)
    borrowing_range: u8,
    /// Last mode borrowed from during interchange (for UI display)
    last_borrowed_from: Option<ScaleMode>,
}

impl Scale {
    /// Creates a new scale with the given tonic and mode.
    pub fn new(tonic: u8, mode: ScaleMode) -> Self {
        Self {
            tonic: tonic % 12,
            mode,
            offsets: mode.intervals().to_vec(),
            interchange_enabled: false,
            borrowing_range: 3,
            last_borrowed_from: None,
        }
    }

    /// Returns the number of degrees in this scale (7 for most, 8 for Barry Harris).
    pub fn scale_len(&self) -> usize {
        self.offsets.len()
    }

    /// Creates a new major scale with the given tonic.
    pub fn major(tonic: u8) -> Self {
        Self::new(tonic, ScaleMode::Ionian)
    }

    /// Returns the tonic pitch class (0-11, where 0 = C).
    pub fn tonic(&self) -> u8 {
        self.tonic
    }

    /// Returns the semitone offsets for each scale degree.
    pub fn offsets(&self) -> &[u8] {
        &self.offsets
    }

    /// Returns the current scale mode.
    pub fn mode(&self) -> ScaleMode {
        self.mode
    }

    /// Returns the last mode borrowed from during modal interchange.
    pub fn last_borrowed_from(&self) -> Option<ScaleMode> {
        self.last_borrowed_from
    }

    /// Sets whether modal interchange is enabled.
    pub fn set_interchange_enabled(&mut self, enabled: bool) {
        self.interchange_enabled = enabled;
    }

    /// Sets the borrowing range (clamped 1-5).
    pub fn set_borrowing_range(&mut self, range: u8) {
        self.borrowing_range = range.clamp(1, 5);
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
    pub fn transpose_diatonic(&self, note: Note, degrees: i8) -> Option<Note> {
        let current_degree = self.degree_of(note)? as i8;
        let note_midi = u8::from(note) as i8;

        // Calculate new degree with octave handling
        let total_degrees = current_degree + degrees;
        let len = self.scale_len() as i8;
        let octave_shift = if total_degrees < 0 {
            (total_degrees - (len - 1)) / len // Floor division for negative
        } else {
            total_degrees / len
        };
        let new_degree = ((total_degrees % len) + len) % len;

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

        note // Fallback (shouldn't happen with proper scale)
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
    /// When `interchange_enabled` is true, out-of-key notes are handled
    /// via modal interchange (borrowing from parallel modes) instead of
    /// plain chromatic intervals.
    ///
    /// # Arguments
    /// * `note` - The note to harmonize
    /// * `diatonic_degrees` - Scale degrees to transpose if in-key
    /// * `prefer_above` - If true, prefer intervals above; if false, below
    ///
    /// # Returns
    /// The harmony note, or None if out of MIDI range
    pub fn harmonize_smart(
        &mut self,
        note: Note,
        diatonic_degrees: i8,
        prefer_above: bool,
    ) -> Option<Note> {
        if self.is_in_scale(note) {
            // In-key: clear any previous borrowed mode indicator
            self.last_borrowed_from = None;
            // In-key: use diatonic transposition
            self.transpose_diatonic(note, diatonic_degrees)
        } else if self.interchange_enabled {
            // Out-of-key with interchange: try borrowing from parallel modes
            self.harmonize_with_interchange(note, prefer_above)
        } else {
            // Out-of-key without interchange: use consonant chromatic interval
            self.harmonize_chromatic(note, prefer_above)
        }
    }

    /// Finds a harmony for an out-of-key note using modal interchange.
    ///
    /// Searches parallel modes (built on the same tonic) to find one that
    /// contains the note, then uses that mode's diatonic harmonization.
    /// Falls back to chromatic harmonization if no parallel mode contains the note.
    pub fn harmonize_with_interchange(&mut self, note: Note, prefer_above: bool) -> Option<Note> {
        let sources = borrowing_sources(self.borrowing_range);
        for &borrowed_mode in sources {
            // Skip current mode (already checked in harmonize_smart)
            if borrowed_mode == self.mode {
                continue;
            }
            let borrowed_scale = Scale::new(self.tonic, borrowed_mode);
            if borrowed_scale.is_in_scale(note) {
                // Found a parallel mode containing this note — use its diatonic third
                let degrees = if prefer_above { 2 } else { -2 };
                if let Some(harmony) = borrowed_scale.transpose_diatonic(note, degrees) {
                    self.last_borrowed_from = Some(borrowed_mode);
                    return Some(harmony);
                }
            }
        }
        // No parallel mode contains this note — fall back to chromatic
        self.last_borrowed_from = None;
        self.harmonize_chromatic(note, prefer_above)
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

    /// Given a scale degree and a reference MIDI note, find the MIDI value
    /// of that degree closest to the reference note.
    ///
    /// This is used by the Barry Harris voicing builder to place chord tones
    /// near the input note for close-position voicings.
    ///
    /// # Arguments
    /// * `degree` - Scale degree (0-based, will be wrapped by scale_len)
    /// * `reference_midi` - MIDI note to find the closest realization near
    ///
    /// # Returns
    /// The closest MIDI realization of the given scale degree, or None if
    /// no valid value exists in MIDI range [0, 127].
    pub fn degree_to_midi_near(&self, degree: usize, reference_midi: u8) -> Option<u8> {
        let degree = degree % self.scale_len();
        let offset = self.offsets[degree];
        let target_pc = (self.tonic + offset) % 12;
        let ref_octave = (reference_midi / 12) as i16;

        // Try same octave, one above, one below
        let candidates: [i16; 3] = [
            ref_octave * 12 + target_pc as i16,
            (ref_octave + 1) * 12 + target_pc as i16,
            (ref_octave - 1) * 12 + target_pc as i16,
        ];

        candidates
            .iter()
            .filter(|&&m| (0..=127).contains(&m))
            .min_by_key(|&&m| (m - reference_midi as i16).abs())
            .map(|&m| m as u8)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_c_major_scale_degrees() {
        let scale = Scale::major(0); // C major

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
        let scale = Scale::major(0); // C major

        // C4 + 2 degrees = E4 (major third)
        let result = scale.transpose_diatonic(Note::C4, 2);
        assert_eq!(result, Some(Note::E4));

        // E4 + 2 degrees = G4 (minor third)
        let result = scale.transpose_diatonic(Note::E4, 2);
        assert_eq!(result, Some(Note::G4));
    }

    #[test]
    fn test_diatonic_interval_down() {
        let scale = Scale::major(0); // C major

        // E4 - 2 degrees = C4
        let result = scale.transpose_diatonic(Note::E4, -2);
        assert_eq!(result, Some(Note::C4));
    }

    #[test]
    fn test_g_major_scale() {
        let scale = Scale::major(7); // G major (7 semitones from C)

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
        let scale = Scale::major(0); // C major

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
        let mut scale = Scale::major(0); // C major

        // In-key note should use diatonic transposition
        // C4 + 2 degrees = E4
        let result = scale.harmonize_smart(Note::C4, 2, true);
        assert_eq!(result, Some(Note::E4));
    }

    #[test]
    fn test_harmonize_smart_out_of_key() {
        let mut scale = Scale::major(0); // C major

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
            "Expected consonant interval, got {} semitones",
            interval
        );
    }

    #[test]
    fn test_chromatic_harmony_prefers_scale_tones() {
        let mut scale = Scale::major(0); // C major

        // F#4 is out of key
        let result = scale.harmonize_smart(Note::Gb4, -2, false);
        assert!(result.is_some());

        let harmony = result.unwrap();
        let melody_midi = u8::from(Note::Gb4);
        let harmony_midi = u8::from(harmony);
        let interval = (melody_midi as i8 - harmony_midi as i8).abs();
        assert!(
            [3, 4, 5, 7, 8, 9].contains(&interval),
            "Expected consonant interval below, got {} semitones",
            interval
        );
    }

    // New ScaleMode and Scale::new tests

    #[test]
    fn test_scale_new_with_each_mode() {
        // Verify all 9 modes produce scales with correct degrees
        for &mode in ScaleMode::all() {
            let scale = Scale::new(0, mode); // C as tonic
                                             // Tonic (C) should always be degree 0
            assert_eq!(
                scale.degree_of(Note::C4),
                Some(0),
                "Tonic should be degree 0 for {:?}",
                mode
            );
            // Should have exactly 7 scale degrees
            let mut count = 0;
            for midi in 60..72 {
                // C4 through B4
                if let Ok(note) = Note::try_from(midi) {
                    if scale.degree_of(note).is_some() {
                        count += 1;
                    }
                }
            }
            let expected = mode.intervals().len();
            assert_eq!(
                count, expected,
                "Mode {:?} should have {} degrees in an octave",
                mode, expected
            );
        }
    }

    #[test]
    fn test_c_dorian_has_eb_and_bb() {
        let scale = Scale::new(0, ScaleMode::Dorian); // C Dorian

        // C Dorian: C D Eb F G A Bb
        assert_eq!(scale.degree_of(Note::C4), Some(0));
        assert_eq!(scale.degree_of(Note::D4), Some(1));
        assert_eq!(scale.degree_of(Note::Eb4), Some(2)); // Eb is degree 2
        assert_eq!(scale.degree_of(Note::F4), Some(3));
        assert_eq!(scale.degree_of(Note::G4), Some(4));
        assert_eq!(scale.degree_of(Note::A4), Some(5));
        assert_eq!(scale.degree_of(Note::Bb4), Some(6)); // Bb is degree 6

        // E natural should NOT be in C Dorian
        assert_eq!(scale.degree_of(Note::E4), None);
        // B natural should NOT be in C Dorian
        assert_eq!(scale.degree_of(Note::B4), None);
    }

    #[test]
    fn test_modal_interchange_finds_borrowing_source() {
        // C Ionian scale, interchange enabled
        let mut scale = Scale::new(0, ScaleMode::Ionian);
        scale.set_interchange_enabled(true);
        scale.set_borrowing_range(3);

        // Eb4 is not in C Ionian, but IS in C Aeolian and C Dorian
        let result = scale.harmonize_with_interchange(Note::Eb4, true);
        assert!(
            result.is_some(),
            "Should find harmony via interchange for Eb"
        );
        assert!(
            scale.last_borrowed_from().is_some(),
            "Should record borrowed mode"
        );

        let borrowed = scale.last_borrowed_from().unwrap();
        // Eb is in both Aeolian and Dorian; Aeolian is checked first at range >= 1
        assert!(
            borrowed == ScaleMode::Aeolian || borrowed == ScaleMode::Dorian,
            "Should borrow from Aeolian or Dorian, got {:?}",
            borrowed
        );
    }

    #[test]
    fn test_harmonize_smart_with_interchange_enabled() {
        let mut scale = Scale::new(0, ScaleMode::Ionian);
        scale.set_interchange_enabled(true);

        // In-key note: should still use diatonic (interchange doesn't affect in-key)
        let result = scale.harmonize_smart(Note::C4, 2, true);
        assert_eq!(result, Some(Note::E4));

        // Out-of-key note: should use interchange path
        let result = scale.harmonize_smart(Note::Eb4, 2, true);
        assert!(result.is_some());
        // Should have set last_borrowed_from
        assert!(scale.last_borrowed_from().is_some());
    }

    #[test]
    fn test_harmonize_smart_without_interchange_unchanged() {
        let mut scale = Scale::new(0, ScaleMode::Ionian);
        // interchange_enabled is false by default

        // Out-of-key: should use chromatic path (no last_borrowed_from)
        let result = scale.harmonize_smart(Note::Eb4, 2, true);
        assert!(result.is_some());
        assert!(scale.last_borrowed_from().is_none());
    }

    #[test]
    fn test_scale_mode_getter() {
        let scale = Scale::new(0, ScaleMode::Lydian);
        assert_eq!(scale.mode(), ScaleMode::Lydian);
    }

    #[test]
    fn test_major_is_ionian() {
        let major = Scale::major(0);
        let ionian = Scale::new(0, ScaleMode::Ionian);
        // Both should produce same degrees for all notes
        for midi in 0..=127 {
            if let Ok(note) = Note::try_from(midi) {
                assert_eq!(major.degree_of(note), ionian.degree_of(note));
            }
        }
    }

    #[test]
    fn test_borrowing_range_clamp() {
        let mut scale = Scale::new(0, ScaleMode::Ionian);
        scale.set_borrowing_range(0);
        assert_eq!(scale.borrowing_range, 1); // clamped to min
        scale.set_borrowing_range(10);
        assert_eq!(scale.borrowing_range, 5); // clamped to max
    }

    #[test]
    fn test_scale_mode_all_returns_57() {
        assert_eq!(ScaleMode::all().len(), 57);
    }

    #[test]
    fn test_barry_harris_major_6th_dim_has_8_degrees() {
        let scale = Scale::new(0, ScaleMode::BHMajor6thDim);
        assert_eq!(scale.scale_len(), 8);
        // C BH Major 6th Dim: C D E F G Ab A B (8 notes)
        let mut count = 0;
        for midi in 60..72 {
            if let Ok(note) = Note::try_from(midi) {
                if scale.degree_of(note).is_some() {
                    count += 1;
                }
            }
        }
        assert_eq!(count, 8);
        assert_eq!(scale.degree_of(Note::C4), Some(0));
        assert_eq!(scale.degree_of(Note::Ab4), Some(5));
        assert_eq!(scale.degree_of(Note::A4), Some(6));
        assert_eq!(scale.degree_of(Note::B4), Some(7));
    }

    #[test]
    fn test_barry_harris_transpose_diatonic() {
        let scale = Scale::new(0, ScaleMode::BHMajor6thDim);
        // Degree 7 (B4) + 1 = degree 0 one octave up (C5)
        let result = scale.transpose_diatonic(Note::B4, 1);
        assert_eq!(result, Some(Note::C5));
        // Degree 0 (C4) - 1 = degree 7 one octave down (B3)
        let result = scale.transpose_diatonic(Note::C4, -1);
        assert_eq!(result, Some(Note::B3));
    }

    #[test]
    fn test_phrygian_dominant_intervals() {
        let scale = Scale::new(0, ScaleMode::PhrygianDominant);
        // C Phrygian Dominant: C Db E F G Ab Bb
        assert_eq!(scale.degree_of(Note::C4), Some(0));
        assert_eq!(scale.degree_of(Note::Db4), Some(1));
        assert_eq!(scale.degree_of(Note::E4), Some(2)); // E natural, not Eb
        assert_eq!(scale.degree_of(Note::Eb4), None);
        assert_eq!(scale.degree_of(Note::G4), Some(4));
        assert_eq!(scale.degree_of(Note::Ab4), Some(5));
        assert_eq!(scale.degree_of(Note::Bb4), Some(6));
    }

    #[test]
    fn test_existing_modes_unchanged() {
        let scale = Scale::new(0, ScaleMode::Ionian);
        // C Ionian: C D E F G A B
        assert_eq!(scale.degree_of(Note::C4), Some(0));
        assert_eq!(scale.degree_of(Note::D4), Some(1));
        assert_eq!(scale.degree_of(Note::E4), Some(2));
        assert_eq!(scale.degree_of(Note::F4), Some(3));
        assert_eq!(scale.degree_of(Note::G4), Some(4));
        assert_eq!(scale.degree_of(Note::A4), Some(5));
        assert_eq!(scale.degree_of(Note::B4), Some(6));
    }

    #[test]
    fn test_degree_to_midi_near_c_major() {
        let scale = Scale::major(0); // C major
                                     // Degree 0 (C) near C4 (60) should be 60
        assert_eq!(scale.degree_to_midi_near(0, 60), Some(60));
        // Degree 2 (E) near C4 (60) should be E4 (64)
        assert_eq!(scale.degree_to_midi_near(2, 60), Some(64));
        // Degree 4 (G) near C4 (60): G3=55 is 5 away, G4=67 is 7 away. So G3.
        assert_eq!(scale.degree_to_midi_near(4, 60), Some(55));
    }

    #[test]
    fn test_degree_to_midi_near_bh_scale() {
        let scale = Scale::new(0, ScaleMode::BHMajor6thDim);
        // Degree 6 (A) near C4 (60): A3=57 is 3 away, A4=69 is 9 away. So A3.
        assert_eq!(scale.degree_to_midi_near(6, 60), Some(57));
        // Degree 7 (B) near C4 (60): B3=59 is 1 away, B4=71 is 11 away. So B3.
        assert_eq!(scale.degree_to_midi_near(7, 60), Some(59));
    }
}
