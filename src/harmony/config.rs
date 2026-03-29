//! Harmony configuration types.
//!
//! This module defines the enums used to configure the harmony engine:
//! musical keys, harmony modes, octave modes, and scale modes.

use serde::{Deserialize, Serialize};

/// Musical keys (C through B, 12 options).
///
/// Represents the tonic of a scale. Used with [`ScaleMode`] to define
/// the complete scale for harmony generation.
///
/// Each key corresponds to a pitch class (0-11) where C=0, Db=1, ..., B=11.
///
/// # Example
///
/// ```
/// use contrapunk::harmony::Key;
///
/// let key = Key::C;
/// assert_eq!(key.semitones_from_c(), 0);
///
/// let key = Key::G;
/// assert_eq!(key.semitones_from_c(), 7);
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Key {
    C,
    Db,
    D,
    Eb,
    E,
    F,
    Gb,
    G,
    Ab,
    A,
    Bb,
    B,
}

impl Key {
    /// Returns semitones from C (0-11).
    ///
    /// This value is used as the tonic pitch class when constructing scales.
    pub fn semitones_from_c(&self) -> u8 {
        match self {
            Key::C => 0,
            Key::Db => 1,
            Key::D => 2,
            Key::Eb => 3,
            Key::E => 4,
            Key::F => 5,
            Key::Gb => 6,
            Key::G => 7,
            Key::Ab => 8,
            Key::A => 9,
            Key::Bb => 10,
            Key::B => 11,
        }
    }

    /// Returns all keys in order for menu display.
    ///
    /// # Example
    ///
    /// ```
    /// use contrapunk::harmony::Key;
    ///
    /// let keys = Key::all();
    /// assert_eq!(keys.len(), 12);
    /// assert_eq!(keys[0], Key::C);
    /// ```
    pub fn all() -> &'static [Key] {
        &[
            Key::C,
            Key::Db,
            Key::D,
            Key::Eb,
            Key::E,
            Key::F,
            Key::Gb,
            Key::G,
            Key::Ab,
            Key::A,
            Key::Bb,
            Key::B,
        ]
    }
}

impl std::fmt::Display for Key {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

/// Octave placement modes for harmony voices.
///
/// Controls how harmony notes are shifted in octave after generation.
/// Applied as post-processing after the harmony algorithm runs.
///
/// # Example
///
/// ```
/// use contrapunk::harmony::OctaveMode;
///
/// let mode = OctaveMode::default();
/// assert_eq!(mode, OctaveMode::None);
///
/// for mode in OctaveMode::all() {
///     println!("{}: {}", mode, mode.description());
/// }
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OctaveMode {
    /// No octave modification - harmonies stay at their generated pitch.
    #[default]
    None,
    /// Spread voices across octaves - each successive voice is +1 octave higher.
    Spread,
    /// Bass/Treble split - harmonies below melody go down an octave, above go up.
    BassTrebleSplit,
    /// Mirror - duplicate each harmony note in +1 and -1 octaves (tripling harmony notes).
    Mirror,
}

impl OctaveMode {
    /// Returns all octave modes for menu display.
    pub fn all() -> &'static [OctaveMode] {
        &[
            OctaveMode::None,
            OctaveMode::Spread,
            OctaveMode::BassTrebleSplit,
            OctaveMode::Mirror,
        ]
    }

    /// Returns description for menu display.
    pub fn description(&self) -> &'static str {
        match self {
            OctaveMode::None => "None (default pitch)",
            OctaveMode::Spread => "Spread (+1 octave per voice)",
            OctaveMode::BassTrebleSplit => "Bass/Treble split",
            OctaveMode::Mirror => "Mirror (±1 octave)",
        }
    }
}

impl std::fmt::Display for OctaveMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.description())
    }
}

/// Scale family grouping for UI display.
///
/// Organizes the 28 scale modes into 5 logical families for easier navigation.
/// Each family contains related modes derived from the same parent scale.
///
/// # Families
///
/// - **Diatonic**: The 7 modes of the major scale (Ionian, Dorian, etc.)
/// - **HarmonicMinor**: The 7 modes of the harmonic minor scale
/// - **MelodicMinor**: The 7 modes of the melodic minor (ascending) scale
/// - **Exotic**: Non-Western scales (Double Harmonic, Hungarian Minor, etc.)
/// - **BarryHarris**: 8-note bebop scales with added diminished passing tones
///
/// # Example
///
/// ```
/// use contrapunk::harmony::{ScaleFamily, ScaleMode};
///
/// let dorian = ScaleMode::Dorian;
/// assert_eq!(dorian.family(), ScaleFamily::Diatonic);
///
/// let phrygian_dom = ScaleMode::PhrygianDominant;
/// assert_eq!(phrygian_dom.family(), ScaleFamily::HarmonicMinor);
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScaleFamily {
    /// Diatonic modes (Ionian through Locrian) - the 7 modes of the major scale.
    #[serde(alias = "church")]
    Diatonic,
    /// Harmonic minor modes - characterized by the augmented 2nd interval.
    HarmonicMinor,
    /// Melodic minor modes - also known as jazz minor modes.
    MelodicMinor,
    /// Exotic scales - non-Western and synthetic scales.
    Exotic,
    /// Barry Harris 6th diminished scales - 8-note bebop scales.
    BarryHarris,
}

impl ScaleFamily {
    /// Returns all scale families.
    pub fn all() -> &'static [ScaleFamily] {
        &[
            ScaleFamily::Diatonic,
            ScaleFamily::HarmonicMinor,
            ScaleFamily::MelodicMinor,
            ScaleFamily::Exotic,
            ScaleFamily::BarryHarris,
        ]
    }
}

impl std::fmt::Display for ScaleFamily {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ScaleFamily::Diatonic => write!(f, "Diatonic Modes"),
            ScaleFamily::HarmonicMinor => write!(f, "Harmonic Minor"),
            ScaleFamily::MelodicMinor => write!(f, "Melodic Minor"),
            ScaleFamily::Exotic => write!(f, "Exotic"),
            ScaleFamily::BarryHarris => write!(f, "Barry Harris"),
        }
    }
}

/// Scale modes (28 variants grouped by family).
///
/// Defines the interval pattern for a scale. Combined with a [`Key`] (tonic),
/// this determines which notes belong to the scale and how diatonic
/// transposition works.
///
/// Most scales have 7 notes per octave. Barry Harris scales have 8 notes,
/// adding a diminished passing tone between the 5th and 6th degrees.
///
/// # Families
///
/// - **Church Modes (7)**: Ionian, Dorian, Phrygian, Lydian, Mixolydian, Aeolian, Locrian
/// - **Harmonic Minor Modes (7)**: HarmonicMinor, LocrianNat6, IonianAug, DorianSharp4,
///   PhrygianDominant, LydianSharp2, SuperLocrianDim
/// - **Melodic Minor Modes (7)**: MelodicMinor, DorianFlat2, LydianAug, LydianDominant,
///   MixolydianFlat6, LocrianNat2, SuperLocrian
/// - **Exotic (5)**: DoubleHarmonic, HungarianMinor, Enigmatic, NeapolitanMinor, NeapolitanMajor
/// - **Barry Harris (2)**: BHMajor6thDim, BHMinor6thDim
///
/// # Example
///
/// ```
/// use contrapunk::harmony::ScaleMode;
///
/// // Get intervals for C Dorian: C D Eb F G A Bb
/// let dorian = ScaleMode::Dorian;
/// let intervals = dorian.intervals();
/// assert_eq!(intervals, vec![0, 2, 3, 5, 7, 9, 10]);
///
/// // Barry Harris scales have 8 notes
/// let bh = ScaleMode::BHMajor6thDim;
/// assert_eq!(bh.intervals().len(), 8);
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScaleMode {
    // Church modes (7)
    /// Ionian mode (major scale): W-W-H-W-W-W-H
    Ionian,
    /// Dorian mode: W-H-W-W-W-H-W (minor with raised 6th)
    Dorian,
    /// Phrygian mode: H-W-W-W-H-W-W (minor with lowered 2nd)
    Phrygian,
    /// Lydian mode: W-W-W-H-W-W-H (major with raised 4th)
    Lydian,
    /// Mixolydian mode: W-W-H-W-W-H-W (major with lowered 7th)
    Mixolydian,
    /// Aeolian mode (natural minor): W-H-W-W-H-W-W
    Aeolian,
    /// Locrian mode: H-W-W-H-W-W-W (diminished tonic)
    Locrian,
    // Harmonic minor modes (7)
    /// Harmonic minor: natural minor with raised 7th
    HarmonicMinor,
    /// Locrian natural 6: Locrian with natural 6th
    LocrianNat6,
    /// Ionian augmented: major with raised 5th
    IonianAug,
    /// Dorian #4: Dorian with raised 4th
    DorianSharp4,
    /// Phrygian dominant: Phrygian with raised 3rd (Spanish scale)
    PhrygianDominant,
    /// Lydian #2: Lydian with raised 2nd
    LydianSharp2,
    /// Super Locrian diminished: altered scale variant
    SuperLocrianDim,
    // Melodic minor modes (7)
    /// Melodic minor (ascending): minor with raised 6th and 7th
    MelodicMinor,
    /// Dorian b2: Dorian with lowered 2nd
    DorianFlat2,
    /// Lydian augmented: Lydian with raised 5th
    LydianAug,
    /// Lydian dominant: Lydian with lowered 7th
    LydianDominant,
    /// Mixolydian b6: Mixolydian with lowered 6th
    MixolydianFlat6,
    /// Locrian natural 2: Locrian with natural 2nd
    LocrianNat2,
    /// Super Locrian (altered scale): all tensions altered
    SuperLocrian,
    // Exotic (5)
    /// Double harmonic: two augmented 2nd intervals
    DoubleHarmonic,
    /// Hungarian minor: harmonic minor with raised 4th
    HungarianMinor,
    /// Enigmatic: synthetic scale with unusual intervals
    Enigmatic,
    /// Neapolitan minor: harmonic minor with lowered 2nd
    NeapolitanMinor,
    /// Neapolitan major: major with lowered 2nd
    NeapolitanMajor,
    // Barry Harris (2)
    /// Barry Harris major 6th diminished: 8-note bebop major scale
    BHMajor6thDim,
    /// Barry Harris minor 6th diminished: 8-note bebop minor scale
    BHMinor6thDim,
}

impl ScaleMode {
    /// Returns semitone offsets from tonic for each scale degree.
    ///
    /// Most scales return 7 elements; Barry Harris scales return 8.
    /// Values are semitones from the tonic (0-11 range for one octave).
    ///
    /// # Example
    ///
    /// ```
    /// use contrapunk::harmony::ScaleMode;
    ///
    /// // C Major intervals: C(0) D(2) E(4) F(5) G(7) A(9) B(11)
    /// assert_eq!(ScaleMode::Ionian.intervals(), vec![0, 2, 4, 5, 7, 9, 11]);
    ///
    /// // C Dorian intervals: C(0) D(2) Eb(3) F(5) G(7) A(9) Bb(10)
    /// assert_eq!(ScaleMode::Dorian.intervals(), vec![0, 2, 3, 5, 7, 9, 10]);
    /// ```
    pub fn intervals(&self) -> Vec<u8> {
        match self {
            // Church modes
            ScaleMode::Ionian => vec![0, 2, 4, 5, 7, 9, 11],
            ScaleMode::Dorian => vec![0, 2, 3, 5, 7, 9, 10],
            ScaleMode::Phrygian => vec![0, 1, 3, 5, 7, 8, 10],
            ScaleMode::Lydian => vec![0, 2, 4, 6, 7, 9, 11],
            ScaleMode::Mixolydian => vec![0, 2, 4, 5, 7, 9, 10],
            ScaleMode::Aeolian => vec![0, 2, 3, 5, 7, 8, 10],
            ScaleMode::Locrian => vec![0, 1, 3, 5, 6, 8, 10],
            // Harmonic minor modes
            ScaleMode::HarmonicMinor => vec![0, 2, 3, 5, 7, 8, 11],
            ScaleMode::LocrianNat6 => vec![0, 1, 3, 5, 6, 9, 10],
            ScaleMode::IonianAug => vec![0, 2, 4, 5, 8, 9, 11],
            ScaleMode::DorianSharp4 => vec![0, 2, 3, 6, 7, 9, 10],
            ScaleMode::PhrygianDominant => vec![0, 1, 4, 5, 7, 8, 10],
            ScaleMode::LydianSharp2 => vec![0, 3, 4, 6, 7, 9, 11],
            ScaleMode::SuperLocrianDim => vec![0, 1, 3, 4, 6, 8, 9],
            // Melodic minor modes
            ScaleMode::MelodicMinor => vec![0, 2, 3, 5, 7, 9, 11],
            ScaleMode::DorianFlat2 => vec![0, 1, 3, 5, 7, 9, 10],
            ScaleMode::LydianAug => vec![0, 2, 4, 6, 8, 9, 11],
            ScaleMode::LydianDominant => vec![0, 2, 4, 6, 7, 9, 10],
            ScaleMode::MixolydianFlat6 => vec![0, 2, 4, 5, 7, 8, 10],
            ScaleMode::LocrianNat2 => vec![0, 2, 3, 5, 6, 8, 10],
            ScaleMode::SuperLocrian => vec![0, 1, 3, 4, 6, 8, 10],
            // Exotic
            ScaleMode::DoubleHarmonic => vec![0, 1, 4, 5, 7, 8, 11],
            ScaleMode::HungarianMinor => vec![0, 2, 3, 6, 7, 8, 11],
            ScaleMode::Enigmatic => vec![0, 1, 4, 6, 8, 10, 11],
            ScaleMode::NeapolitanMinor => vec![0, 1, 3, 5, 7, 8, 11],
            ScaleMode::NeapolitanMajor => vec![0, 1, 3, 5, 7, 9, 11],
            // Barry Harris 6th Diminished (8-note)
            ScaleMode::BHMajor6thDim => vec![0, 2, 4, 5, 7, 8, 9, 11],
            ScaleMode::BHMinor6thDim => vec![0, 2, 3, 5, 7, 8, 9, 11],
        }
    }

    /// Returns the scale family for this mode.
    ///
    /// Used for grouping modes in the UI and determining related scales.
    pub fn family(&self) -> ScaleFamily {
        match self {
            ScaleMode::Ionian
            | ScaleMode::Dorian
            | ScaleMode::Phrygian
            | ScaleMode::Lydian
            | ScaleMode::Mixolydian
            | ScaleMode::Aeolian
            | ScaleMode::Locrian => ScaleFamily::Diatonic,

            ScaleMode::HarmonicMinor
            | ScaleMode::LocrianNat6
            | ScaleMode::IonianAug
            | ScaleMode::DorianSharp4
            | ScaleMode::PhrygianDominant
            | ScaleMode::LydianSharp2
            | ScaleMode::SuperLocrianDim => ScaleFamily::HarmonicMinor,

            ScaleMode::MelodicMinor
            | ScaleMode::DorianFlat2
            | ScaleMode::LydianAug
            | ScaleMode::LydianDominant
            | ScaleMode::MixolydianFlat6
            | ScaleMode::LocrianNat2
            | ScaleMode::SuperLocrian => ScaleFamily::MelodicMinor,

            ScaleMode::DoubleHarmonic
            | ScaleMode::HungarianMinor
            | ScaleMode::Enigmatic
            | ScaleMode::NeapolitanMinor
            | ScaleMode::NeapolitanMajor => ScaleFamily::Exotic,

            ScaleMode::BHMajor6thDim | ScaleMode::BHMinor6thDim => ScaleFamily::BarryHarris,
        }
    }

    /// Returns all scale modes for menu display.
    ///
    /// Modes are ordered by family: Church, Harmonic Minor, Melodic Minor, Exotic, Barry Harris.
    pub fn all() -> &'static [ScaleMode] {
        &[
            // Church
            ScaleMode::Ionian,
            ScaleMode::Dorian,
            ScaleMode::Phrygian,
            ScaleMode::Lydian,
            ScaleMode::Mixolydian,
            ScaleMode::Aeolian,
            ScaleMode::Locrian,
            // Harmonic Minor
            ScaleMode::HarmonicMinor,
            ScaleMode::LocrianNat6,
            ScaleMode::IonianAug,
            ScaleMode::DorianSharp4,
            ScaleMode::PhrygianDominant,
            ScaleMode::LydianSharp2,
            ScaleMode::SuperLocrianDim,
            // Melodic Minor
            ScaleMode::MelodicMinor,
            ScaleMode::DorianFlat2,
            ScaleMode::LydianAug,
            ScaleMode::LydianDominant,
            ScaleMode::MixolydianFlat6,
            ScaleMode::LocrianNat2,
            ScaleMode::SuperLocrian,
            // Exotic
            ScaleMode::DoubleHarmonic,
            ScaleMode::HungarianMinor,
            ScaleMode::Enigmatic,
            ScaleMode::NeapolitanMinor,
            ScaleMode::NeapolitanMajor,
            // Barry Harris
            ScaleMode::BHMajor6thDim,
            ScaleMode::BHMinor6thDim,
        ]
    }

    /// Returns all scale modes in a given family.
    ///
    /// # Example
    ///
    /// ```
    /// use contrapunk::harmony::{ScaleFamily, ScaleMode};
    ///
    /// let church_modes = ScaleMode::all_in_family(ScaleFamily::Diatonic);
    /// assert_eq!(church_modes.len(), 7);
    /// assert!(church_modes.contains(&ScaleMode::Dorian));
    /// ```
    pub fn all_in_family(family: ScaleFamily) -> Vec<ScaleMode> {
        ScaleMode::all()
            .iter()
            .filter(|m| m.family() == family)
            .copied()
            .collect()
    }
}

impl Default for ScaleMode {
    fn default() -> Self {
        ScaleMode::Ionian
    }
}

impl std::fmt::Display for ScaleMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ScaleMode::Ionian => write!(f, "Ionian (Major)"),
            ScaleMode::Dorian => write!(f, "Dorian"),
            ScaleMode::Phrygian => write!(f, "Phrygian"),
            ScaleMode::Lydian => write!(f, "Lydian"),
            ScaleMode::Mixolydian => write!(f, "Mixolydian"),
            ScaleMode::Aeolian => write!(f, "Aeolian (Minor)"),
            ScaleMode::Locrian => write!(f, "Locrian"),
            ScaleMode::HarmonicMinor => write!(f, "Harmonic Minor"),
            ScaleMode::LocrianNat6 => write!(f, "Locrian Nat 6"),
            ScaleMode::IonianAug => write!(f, "Ionian Augmented"),
            ScaleMode::DorianSharp4 => write!(f, "Dorian #4"),
            ScaleMode::PhrygianDominant => write!(f, "Phrygian Dominant"),
            ScaleMode::LydianSharp2 => write!(f, "Lydian #2"),
            ScaleMode::SuperLocrianDim => write!(f, "Super Locrian Dim"),
            ScaleMode::MelodicMinor => write!(f, "Melodic Minor"),
            ScaleMode::DorianFlat2 => write!(f, "Dorian b2"),
            ScaleMode::LydianAug => write!(f, "Lydian Augmented"),
            ScaleMode::LydianDominant => write!(f, "Lydian Dominant"),
            ScaleMode::MixolydianFlat6 => write!(f, "Mixolydian b6"),
            ScaleMode::LocrianNat2 => write!(f, "Locrian Nat 2"),
            ScaleMode::SuperLocrian => write!(f, "Super Locrian"),
            ScaleMode::DoubleHarmonic => write!(f, "Double Harmonic"),
            ScaleMode::HungarianMinor => write!(f, "Hungarian Minor"),
            ScaleMode::Enigmatic => write!(f, "Enigmatic"),
            ScaleMode::NeapolitanMinor => write!(f, "Neapolitan Minor"),
            ScaleMode::NeapolitanMajor => write!(f, "Neapolitan Major"),
            ScaleMode::BHMajor6thDim => write!(f, "BH Major 6th Dim"),
            ScaleMode::BHMinor6thDim => write!(f, "BH Minor 6th Dim"),
        }
    }
}

/// Harmony modes (algorithms for generating harmony notes).
///
/// Each mode implements a different algorithm for transforming input notes
/// into harmonized output. Modes 1-5 and 8 are stateless (each note processed
/// independently). Modes 6-7 are stateful (track previous notes).
///
/// # Mode Overview
///
/// | # | Mode | Algorithm |
/// |---|------|-----------|
/// | 1 | PassThrough | No harmony, notes pass unchanged |
/// | 2 | DiatonicThirds | Add third above (2 scale degrees) |
/// | 3 | DiatonicFourths | Add fourth above (3 scale degrees) |
/// | 4 | RandomBelow | Random interval below (2nd-7th) |
/// | 5 | RandomBelowNoSeconds | Random below, excluding 2nds |
/// | 6 | ContraryMotion | Harmony moves opposite to melody |
/// | 7 | StrictCounterpoint | Voice-leading rules with scoring |
/// | 8 | BarryHarris | 6th dim movement (chord/passing parity) |
///
/// # Example
///
/// ```
/// use contrapunk::harmony::HarmonyMode;
///
/// let mode = HarmonyMode::default();
/// assert_eq!(mode, HarmonyMode::PassThrough);
/// assert_eq!(mode.number(), 1);
///
/// for mode in HarmonyMode::all() {
///     println!("Mode {}: {}", mode.number(), mode.description());
/// }
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HarmonyMode {
    /// Mode 1: Pass-through (no harmony). Notes pass unchanged.
    #[default]
    PassThrough,
    /// Mode 2: Diatonic thirds above. Adds a third (2 scale degrees) above.
    DiatonicThirds,
    /// Mode 3: Diatonic fourths above. Adds a fourth (3 scale degrees) above.
    DiatonicFourths,
    /// Mode 4: Random diatonic interval below (2nd-7th).
    RandomBelow,
    /// Mode 5: Random below excluding 2nds (which can sound dissonant).
    RandomBelowNoSeconds,
    /// Mode 6: Contrary motion. Harmony moves opposite to melody direction.
    /// Stateful: tracks previous melody and harmony notes.
    ContraryMotion,
    /// Mode 7: Strict counterpoint. Traditional voice-leading with scoring.
    /// Stateful: uses sliding window history for interval variety and contour.
    StrictCounterpoint,
    /// Mode 8: Barry Harris 6th diminished movement.
    /// Moves by 2 scale degrees, preserving chord-tone/passing-tone parity
    /// in 8-note Barry Harris scales.
    BarryHarris,
}

impl HarmonyMode {
    /// Returns mode number (1-8) for display.
    ///
    /// # Example
    ///
    /// ```
    /// use contrapunk::harmony::HarmonyMode;
    ///
    /// assert_eq!(HarmonyMode::PassThrough.number(), 1);
    /// assert_eq!(HarmonyMode::BarryHarris.number(), 8);
    /// ```
    pub fn number(&self) -> u8 {
        match self {
            HarmonyMode::PassThrough => 1,
            HarmonyMode::DiatonicThirds => 2,
            HarmonyMode::DiatonicFourths => 3,
            HarmonyMode::RandomBelow => 4,
            HarmonyMode::RandomBelowNoSeconds => 5,
            HarmonyMode::ContraryMotion => 6,
            HarmonyMode::StrictCounterpoint => 7,
            HarmonyMode::BarryHarris => 8,
        }
    }

    /// Returns all modes in order for menu display.
    pub fn all() -> &'static [HarmonyMode] {
        &[
            HarmonyMode::PassThrough,
            HarmonyMode::DiatonicThirds,
            HarmonyMode::DiatonicFourths,
            HarmonyMode::RandomBelow,
            HarmonyMode::RandomBelowNoSeconds,
            HarmonyMode::ContraryMotion,
            HarmonyMode::StrictCounterpoint,
            HarmonyMode::BarryHarris,
        ]
    }

    /// Returns description for menu display.
    pub fn description(&self) -> &'static str {
        match self {
            HarmonyMode::PassThrough => "Pass-through (no harmony)",
            HarmonyMode::DiatonicThirds => "Diatonic thirds above",
            HarmonyMode::DiatonicFourths => "Diatonic fourths above",
            HarmonyMode::RandomBelow => "Random diatonic below",
            HarmonyMode::RandomBelowNoSeconds => "Random below (no 2nds)",
            HarmonyMode::ContraryMotion => "Contrary motion",
            HarmonyMode::StrictCounterpoint => "Strict counterpoint",
            HarmonyMode::BarryHarris => "Barry Harris (6th dim movement)",
        }
    }
}
