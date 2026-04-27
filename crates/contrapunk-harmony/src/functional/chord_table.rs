//! Precomputed chord-note membership table.
//!
//! For a given key and scale mode, builds a lookup table answering
//! "which diatonic chords contain this pitch class?" in O(1).
//!
//! Only valid for 7-note scales. 8-note scales (Barry Harris) are
//! not supported by FunctionalHarmony/BachChorale modes.

use crate::config::ScaleMode;

/// Role a pitch class plays within a chord.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChordRole {
    Root,
    Third,
    Fifth,
    Seventh,
    /// Pitch class is not a member of this chord.
    None,
}

/// Quality of a diatonic chord (triad or seventh).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChordQuality {
    Major,
    Minor,
    Diminished,
    Augmented,
    /// Dominant 7th chord (major triad + minor 7th).
    Dominant7,
    /// Major 7th chord.
    Major7,
    /// Minor 7th chord.
    Minor7,
    /// Half-diminished 7th chord.
    HalfDim7,
    /// Diminished 7th chord.
    Dim7,
}

/// Harmonic function of a scale degree.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HarmonicFunction {
    Tonic,
    Subdominant,
    Dominant,
}

/// Precomputed chord membership table for a specific key and scale mode.
///
/// All lookups are O(1) array indexing.
#[derive(Clone, Debug)]
pub struct ChordTable {
    /// `chord_pcs[degree]` = `[root_pc, third_pc, fifth_pc, seventh_pc]`
    /// for the seventh chord built on each scale degree (0-6).
    pub chord_pcs: [[u8; 4]; 7],

    /// `membership[pitch_class]` = bitmask of degrees (bit 0 = I, bit 6 = vii).
    /// A set bit means this pitch class belongs to the triad on that degree.
    pub membership: [u8; 12],

    /// `role[pitch_class][degree]` = Role this PC plays in that degree's chord.
    /// Only meaningful when the corresponding membership bit is set.
    pub role: [[ChordRole; 7]; 12],

    /// `quality[degree]` = triad quality for doubling/constraint decisions.
    pub quality: [ChordQuality; 7],

    /// `function[degree]` = harmonic function (T/S/D).
    pub function: [HarmonicFunction; 7],

    /// Scale degree pitch classes (the 7 scale tones).
    pub degree_pcs: [u8; 7],

    /// The leading tone pitch class (scale degree 7, index 6).
    pub leading_tone_pc: u8,
}

impl ChordTable {
    /// Build the chord table for a given tonic and scale mode.
    ///
    /// # Panics
    ///
    /// Panics if the scale mode has != 7 intervals (e.g. Barry Harris 8-note).
    pub fn build(tonic: u8, scale_mode: ScaleMode) -> Self {
        let intervals = scale_mode.intervals();
        assert!(
            intervals.len() == 7,
            "FunctionalHarmony requires a 7-note scale, got {} notes",
            intervals.len()
        );

        // Compute pitch classes for each scale degree
        let mut degree_pcs = [0u8; 7];
        for (i, &interval) in intervals.iter().enumerate() {
            degree_pcs[i] = (tonic + interval) % 12;
        }

        // Build seventh chords on each degree using tertian stacking
        // chord = [root, 3rd above root, 5th above root, 7th above root]
        // In scale degrees: [d, d+2, d+4, d+6] (mod 7)
        let mut chord_pcs = [[0u8; 4]; 7];
        for d in 0..7 {
            chord_pcs[d] = [
                degree_pcs[d],
                degree_pcs[(d + 2) % 7],
                degree_pcs[(d + 4) % 7],
                degree_pcs[(d + 6) % 7],
            ];
        }

        // Compute chord quality from intervals
        let mut quality = [ChordQuality::Major; 7];
        for d in 0..7 {
            let root = chord_pcs[d][0];
            let third = chord_pcs[d][1];
            let fifth = chord_pcs[d][2];
            let seventh = chord_pcs[d][3];

            let third_interval = (third + 12 - root) % 12;
            let fifth_interval = (fifth + 12 - root) % 12;
            let seventh_interval = (seventh + 12 - root) % 12;

            quality[d] = match (third_interval, fifth_interval, seventh_interval) {
                (4, 7, 11) => ChordQuality::Major7,
                (4, 7, 10) => ChordQuality::Dominant7,
                (3, 7, 10) => ChordQuality::Minor7,
                (3, 6, 10) => ChordQuality::HalfDim7,
                (3, 6, 9) => ChordQuality::Dim7,
                (4, 8, _) => ChordQuality::Augmented,
                (4, 7, _) => ChordQuality::Major,
                (3, 7, _) => ChordQuality::Minor,
                (3, 6, _) => ChordQuality::Diminished,
                _ => ChordQuality::Major, // fallback for exotic scales
            };
        }

        // Build membership bitmask and role table
        let mut membership = [0u8; 12];
        let mut role = [[ChordRole::None; 7]; 12];

        for d in 0..7usize {
            let root_pc = chord_pcs[d][0] as usize;
            let third_pc = chord_pcs[d][1] as usize;
            let fifth_pc = chord_pcs[d][2] as usize;
            let seventh_pc = chord_pcs[d][3] as usize;

            // Set membership bits (using triads only for membership -- 3 tones)
            membership[root_pc] |= 1 << d;
            membership[third_pc] |= 1 << d;
            membership[fifth_pc] |= 1 << d;
            // Seventh chord tones also get membership for scoring purposes
            membership[seventh_pc] |= 1 << d;

            // Set roles
            role[root_pc][d] = ChordRole::Root;
            role[third_pc][d] = ChordRole::Third;
            role[fifth_pc][d] = ChordRole::Fifth;
            role[seventh_pc][d] = ChordRole::Seventh;
        }

        // Assign harmonic functions
        // Standard: I/iii/vi = Tonic, ii/IV = Subdominant, V/vii = Dominant
        let function = [
            HarmonicFunction::Tonic,       // I
            HarmonicFunction::Subdominant, // ii
            HarmonicFunction::Tonic,       // iii
            HarmonicFunction::Subdominant, // IV
            HarmonicFunction::Dominant,    // V
            HarmonicFunction::Tonic,       // vi
            HarmonicFunction::Dominant,    // vii
        ];

        let leading_tone_pc = degree_pcs[6];

        ChordTable {
            chord_pcs,
            membership,
            role,
            quality,
            function,
            degree_pcs,
            leading_tone_pc,
        }
    }

    /// Returns whether a pitch class belongs to the chord on the given degree.
    #[inline]
    pub fn contains(&self, pc: u8, degree: u8) -> bool {
        self.membership[pc as usize % 12] & (1 << degree) != 0
    }

    /// Returns all degrees whose chords contain the given pitch class.
    pub fn degrees_containing(&self, pc: u8) -> Vec<u8> {
        let mask = self.membership[pc as usize % 12];
        (0..7u8).filter(|&d| mask & (1 << d) != 0).collect()
    }

    /// Returns the role of a pitch class in a given degree's chord.
    #[inline]
    pub fn role_of(&self, pc: u8, degree: u8) -> ChordRole {
        self.role[pc as usize % 12][degree as usize]
    }

    /// Returns whether the given pitch class is diatonic (belongs to the scale).
    pub fn is_diatonic(&self, pc: u8) -> bool {
        let pc = pc % 12;
        self.degree_pcs.contains(&pc)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_c_major_chord_table() {
        // C major: C D E F G A B
        let table = ChordTable::build(0, ScaleMode::Ionian);

        // Degree PCs
        assert_eq!(table.degree_pcs, [0, 2, 4, 5, 7, 9, 11]);

        // I chord = C E G B (Cmaj7)
        assert_eq!(table.chord_pcs[0], [0, 4, 7, 11]);

        // ii chord = D F A C (Dmin7)
        assert_eq!(table.chord_pcs[1], [2, 5, 9, 0]);

        // V chord = G B D F (G7)
        assert_eq!(table.chord_pcs[4], [7, 11, 2, 5]);

        // vii chord = B D F A (Bm7b5)
        assert_eq!(table.chord_pcs[6], [11, 2, 5, 9]);
    }

    #[test]
    fn test_c_major_membership() {
        let table = ChordTable::build(0, ScaleMode::Ionian);

        // C (pc=0) should be in I, IV, vi (and ii as 7th)
        let c_degrees = table.degrees_containing(0);
        assert!(c_degrees.contains(&0)); // I
        assert!(c_degrees.contains(&1)); // ii (as 7th)
        assert!(c_degrees.contains(&3)); // IV
        assert!(c_degrees.contains(&5)); // vi

        // E (pc=4) should be in I, iii (and ii as something)
        let e_degrees = table.degrees_containing(4);
        assert!(e_degrees.contains(&0)); // I (third)
        assert!(e_degrees.contains(&2)); // iii (root)
        assert!(e_degrees.contains(&3)); // IV (seventh)
    }

    #[test]
    fn test_roles() {
        let table = ChordTable::build(0, ScaleMode::Ionian);

        // C is root of I
        assert_eq!(table.role_of(0, 0), ChordRole::Root);
        // E is third of I
        assert_eq!(table.role_of(4, 0), ChordRole::Third);
        // G is fifth of I
        assert_eq!(table.role_of(7, 0), ChordRole::Fifth);
        // B is seventh of I
        assert_eq!(table.role_of(11, 0), ChordRole::Seventh);
    }

    #[test]
    fn test_qualities_c_major() {
        let table = ChordTable::build(0, ScaleMode::Ionian);

        assert_eq!(table.quality[0], ChordQuality::Major7); // Imaj7
        assert_eq!(table.quality[1], ChordQuality::Minor7); // ii7
        assert_eq!(table.quality[2], ChordQuality::Minor7); // iii7
        assert_eq!(table.quality[3], ChordQuality::Major7); // IVmaj7
        assert_eq!(table.quality[4], ChordQuality::Dominant7); // V7
        assert_eq!(table.quality[5], ChordQuality::Minor7); // vi7
        assert_eq!(table.quality[6], ChordQuality::HalfDim7); // vii-7
    }

    #[test]
    fn test_a_minor_chord_table() {
        // A Aeolian: A B C D E F G
        let table = ChordTable::build(9, ScaleMode::Aeolian);

        assert_eq!(table.degree_pcs, [9, 11, 0, 2, 4, 5, 7]);

        // i chord = A C E G (Am7)
        assert_eq!(table.chord_pcs[0], [9, 0, 4, 7]);
    }

    #[test]
    fn test_harmonic_functions() {
        let table = ChordTable::build(0, ScaleMode::Ionian);

        assert_eq!(table.function[0], HarmonicFunction::Tonic); // I
        assert_eq!(table.function[1], HarmonicFunction::Subdominant); // ii
        assert_eq!(table.function[3], HarmonicFunction::Subdominant); // IV
        assert_eq!(table.function[4], HarmonicFunction::Dominant); // V
        assert_eq!(table.function[6], HarmonicFunction::Dominant); // vii
    }

    #[test]
    fn test_is_diatonic() {
        let table = ChordTable::build(0, ScaleMode::Ionian);

        assert!(table.is_diatonic(0)); // C
        assert!(table.is_diatonic(2)); // D
        assert!(table.is_diatonic(4)); // E
        assert!(!table.is_diatonic(1)); // Db -- chromatic
        assert!(!table.is_diatonic(6)); // Gb -- chromatic
    }

    #[test]
    fn test_leading_tone() {
        let table = ChordTable::build(0, ScaleMode::Ionian);
        assert_eq!(table.leading_tone_pc, 11); // B in C major

        let table = ChordTable::build(9, ScaleMode::Aeolian);
        assert_eq!(table.leading_tone_pc, 7); // G in A minor
    }

    #[test]
    fn test_g_dorian() {
        // G Dorian: G A Bb C D E F
        let table = ChordTable::build(7, ScaleMode::Dorian);
        assert_eq!(table.degree_pcs, [7, 9, 10, 0, 2, 4, 5]);

        // i chord = G Bb D F (Gm7)
        assert_eq!(table.chord_pcs[0], [7, 10, 2, 5]);
    }

    #[test]
    #[should_panic(expected = "FunctionalHarmony requires a 7-note scale")]
    fn test_barry_harris_panics() {
        // Barry Harris is 8-note -- should panic
        ChordTable::build(0, ScaleMode::BHMajor6thDim);
    }
}
