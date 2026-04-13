//! Harmony configuration types.
//!
//! This module defines the enums used to configure the harmony engine:
//! musical keys, harmony modes, octave modes, scale modes, and routing modes.

use serde::{Deserialize, Serialize};

/// MIDI routing mode for harmony voice output.
///
/// Controls how harmony voices are distributed to MIDI destinations.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RoutingMode {
    /// Channel-based (MPE): all voices on one output, separated by MIDI channel.
    /// Ch 1 = master, Ch 2 = melody, Ch 3-7 = harmony voices.
    /// Default — works in VST/AU plugins and requires only one MIDI connection.
    #[default]
    ChannelBased,
    /// Port-based: each voice goes to a separate MIDI output port.
    /// Requires multiple IAC buses or MIDI outputs. Legacy mode for
    /// multi-bus DAW routing.
    PortBased,
}

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
/// Organizes the 57 scale modes into 10 logical families for easier navigation.
/// Each family contains related modes derived from the same parent scale
/// or sharing a common construction principle.
///
/// # Families
///
/// - **Diatonic**: The 7 modes of the major scale (Ionian, Dorian, etc.)
/// - **HarmonicMinor**: The 7 modes of the harmonic minor scale
/// - **MelodicMinor**: The 7 modes of the melodic minor (ascending) scale
/// - **HarmonicMajor**: The 7 modes of the harmonic major scale
/// - **DoubleHarmonic**: The 7 modes of the double harmonic scale
/// - **Pentatonic**: 5-note scales (major/minor pentatonic, Japanese, Balinese)
/// - **Blues**: Blues scales and bebop dominant (6-8 notes)
/// - **Symmetric**: Scales with transpositional symmetry (whole tone, diminished, augmented)
/// - **World**: Scales from diverse traditions (Persian, Hungarian Major, Enigmatic, Neapolitan)
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
    /// Harmonic major modes - major scale with lowered 6th and its rotations.
    HarmonicMajor,
    /// Double harmonic modes - two augmented 2nd intervals and rotations.
    DoubleHarmonic,
    /// Pentatonic scales - 5-note scales from various traditions.
    Pentatonic,
    /// Blues and bebop scales - 6 or 8 note scales with chromatic additions.
    Blues,
    /// Symmetric scales - scales with transpositional symmetry.
    Symmetric,
    /// World scales - scales from diverse musical traditions.
    #[serde(alias = "exotic")]
    World,
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
            ScaleFamily::HarmonicMajor,
            ScaleFamily::DoubleHarmonic,
            ScaleFamily::Pentatonic,
            ScaleFamily::Blues,
            ScaleFamily::Symmetric,
            ScaleFamily::World,
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
            ScaleFamily::HarmonicMajor => write!(f, "Harmonic Major"),
            ScaleFamily::DoubleHarmonic => write!(f, "Double Harmonic"),
            ScaleFamily::Pentatonic => write!(f, "Pentatonic"),
            ScaleFamily::Blues => write!(f, "Blues & Bebop"),
            ScaleFamily::Symmetric => write!(f, "Symmetric"),
            ScaleFamily::World => write!(f, "World Scales"),
            ScaleFamily::BarryHarris => write!(f, "Barry Harris"),
        }
    }
}

/// Scale modes (57 variants grouped by family).
///
/// Defines the interval pattern for a scale. Combined with a [`Key`] (tonic),
/// this determines which notes belong to the scale and how diatonic
/// transposition works.
///
/// Scale cardinalities: pentatonic (5), hexatonic/blues (6), heptatonic (7),
/// octatonic/bebop (8). Barry Harris and bebop scales have 8 notes.
///
/// # Families (10)
///
/// - **Diatonic (7)**: Ionian, Dorian, Phrygian, Lydian, Mixolydian, Aeolian, Locrian
/// - **Harmonic Minor (7)**: HarmonicMinor, LocrianNat6, IonianAug, DorianSharp4,
///   PhrygianDominant, LydianSharp2, SuperLocrianDim
/// - **Melodic Minor (7)**: MelodicMinor, DorianFlat2, LydianAug, LydianDominant,
///   MixolydianFlat6, LocrianNat2, SuperLocrian
/// - **Harmonic Major (7)**: HarmonicMajor through LocrianDoubleFlat7
/// - **Double Harmonic (7)**: DoubleHarmonic, LydianSharp2Sharp6, Ultraphrygian,
///   HungarianMinor, Oriental, IonianSharp2Sharp5, LocrianDoubleFlat3DoubleFlat7
/// - **Pentatonic (8)**: MajorPentatonic, MinorPentatonic, Hirajoshi, InSen, Iwato,
///   Yo, Kumoi, Pelog
/// - **Blues & Bebop (3)**: MinorBlues, MajorBlues, BebopDominant
/// - **Symmetric (4)**: WholeTone, DiminishedWholeHalf, DiminishedHalfWhole, AugmentedHex
/// - **World (5)**: Enigmatic, NeapolitanMinor, NeapolitanMajor, Persian, HungarianMajor
/// - **Barry Harris (2)**: BHMajor6thDim, BHMinor6thDim
///
/// Scales sourced from langoy.net/skala (mathematical interval data).
///
/// # Example
///
/// ```
/// use contrapunk::harmony::ScaleMode;
///
/// // Get intervals for C Dorian: C D Eb F G A Bb
/// let dorian = ScaleMode::Dorian;
/// let intervals = dorian.intervals();
/// assert_eq!(intervals, &[0, 2, 3, 5, 7, 9, 10]);
///
/// // Barry Harris scales have 8 notes
/// let bh = ScaleMode::BHMajor6thDim;
/// assert_eq!(bh.intervals().len(), 8);
///
/// // Pentatonic scales have 5 notes
/// let pent = ScaleMode::MinorPentatonic;
/// assert_eq!(pent.intervals().len(), 5);
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScaleMode {
    // Diatonic modes (7)
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

    // Harmonic major modes (7) — major scale with lowered 6th
    /// Harmonic major: major scale with b6 (7 notes)
    HarmonicMajor,
    /// Dorian b5: mode 2 of harmonic major (7 notes)
    DorianFlat5,
    /// Phrygian b4: mode 3 of harmonic major (7 notes)
    PhrygianFlat4,
    /// Lydian b3 (Lydian diminished): mode 4 of harmonic major (7 notes)
    LydianFlat3,
    /// Mixolydian b2: mode 5 of harmonic major (7 notes)
    MixolydianFlat2,
    /// Lydian augmented #2: mode 6 of harmonic major (7 notes)
    LydianAugSharp2,
    /// Locrian bb7: mode 7 of harmonic major (7 notes)
    LocrianDoubleFlat7,

    // Double harmonic modes (7) — two augmented 2nd intervals
    /// Double harmonic: mode 1, two augmented 2nd intervals (7 notes)
    DoubleHarmonic,
    /// Lydian #2 #6: mode 2 of double harmonic (7 notes)
    LydianSharp2Sharp6,
    /// Ultraphrygian: mode 3 of double harmonic (7 notes)
    Ultraphrygian,
    /// Hungarian minor: mode 4 of double harmonic, harmonic minor with #4 (7 notes)
    HungarianMinor,
    /// Oriental: mode 5 of double harmonic (7 notes)
    Oriental,
    /// Ionian #2 #5: mode 6 of double harmonic (7 notes)
    IonianSharp2Sharp5,
    /// Locrian bb3 bb7: mode 7 of double harmonic (7 notes)
    LocrianDoubleFlat3DoubleFlat7,

    // Pentatonic (8) — 5-note scales
    /// Major pentatonic: the most common 5-note scale (5 notes)
    MajorPentatonic,
    /// Minor pentatonic: the most used guitar scale (5 notes)
    MinorPentatonic,
    /// Hirajoshi: Japanese pentatonic (5 notes)
    Hirajoshi,
    /// In Sen: Japanese pentatonic, mode 4 of Hirajoshi (5 notes)
    InSen,
    /// Iwato: Japanese pentatonic, mode 2 of Hirajoshi (5 notes)
    Iwato,
    /// Yo: mode 4 of major pentatonic (5 notes)
    Yo,
    /// Kumoi: Japanese pentatonic (5 notes)
    Kumoi,
    /// Pelog: Balinese/Javanese pentatonic (5 notes)
    Pelog,

    // Blues & Bebop (3)
    /// Minor blues: minor pentatonic + b5 (6 notes)
    MinorBlues,
    /// Major blues: major pentatonic + #2 (6 notes)
    MajorBlues,
    /// Bebop dominant: dominant 7th + natural 7th passing tone (8 notes)
    BebopDominant,

    // Symmetric (4) — scales with transpositional symmetry
    /// Whole tone: 6 equally-spaced notes (6 notes, symmetry order 6)
    WholeTone,
    /// Diminished whole-half: alternating W-H (8 notes, symmetry order 4)
    DiminishedWholeHalf,
    /// Diminished half-whole: alternating H-W (8 notes, symmetry order 4)
    DiminishedHalfWhole,
    /// Augmented hexatonic: alternating m3-H (6 notes, symmetry order 3)
    AugmentedHex,

    // World scales (5) — diverse musical traditions
    /// Enigmatic: synthetic scale with unusual intervals (7 notes)
    Enigmatic,
    /// Neapolitan minor: harmonic minor with lowered 2nd (7 notes)
    NeapolitanMinor,
    /// Neapolitan major: major with lowered 2nd (7 notes)
    NeapolitanMajor,
    /// Persian: similar to double harmonic but with b5 (7 notes)
    Persian,
    /// Hungarian major: Lydian dominant with #2 (7 notes)
    HungarianMajor,

    // Barry Harris (2) — 8-note bebop scales
    /// Barry Harris major 6th diminished: 8-note bebop major scale
    BHMajor6thDim,
    /// Barry Harris minor 6th diminished: 8-note bebop minor scale
    BHMinor6thDim,
}

impl ScaleMode {
    /// Returns semitone offsets from tonic for each scale degree.
    ///
    /// Returns a static slice -- no heap allocation. Scale cardinalities:
    /// pentatonic (5), hexatonic (6), heptatonic (7), octatonic (8).
    ///
    /// # Example
    ///
    /// ```
    /// use contrapunk::harmony::ScaleMode;
    ///
    /// // C Major intervals: C(0) D(2) E(4) F(5) G(7) A(9) B(11)
    /// assert_eq!(ScaleMode::Ionian.intervals(), &[0, 2, 4, 5, 7, 9, 11]);
    ///
    /// // C Dorian intervals: C(0) D(2) Eb(3) F(5) G(7) A(9) Bb(10)
    /// assert_eq!(ScaleMode::Dorian.intervals(), &[0, 2, 3, 5, 7, 9, 10]);
    /// ```
    pub fn intervals(&self) -> &'static [u8] {
        match self {
            // Diatonic modes (7 notes)
            ScaleMode::Ionian => &[0, 2, 4, 5, 7, 9, 11],
            ScaleMode::Dorian => &[0, 2, 3, 5, 7, 9, 10],
            ScaleMode::Phrygian => &[0, 1, 3, 5, 7, 8, 10],
            ScaleMode::Lydian => &[0, 2, 4, 6, 7, 9, 11],
            ScaleMode::Mixolydian => &[0, 2, 4, 5, 7, 9, 10],
            ScaleMode::Aeolian => &[0, 2, 3, 5, 7, 8, 10],
            ScaleMode::Locrian => &[0, 1, 3, 5, 6, 8, 10],
            // Harmonic minor modes (7 notes)
            ScaleMode::HarmonicMinor => &[0, 2, 3, 5, 7, 8, 11],
            ScaleMode::LocrianNat6 => &[0, 1, 3, 5, 6, 9, 10],
            ScaleMode::IonianAug => &[0, 2, 4, 5, 8, 9, 11],
            ScaleMode::DorianSharp4 => &[0, 2, 3, 6, 7, 9, 10],
            ScaleMode::PhrygianDominant => &[0, 1, 4, 5, 7, 8, 10],
            ScaleMode::LydianSharp2 => &[0, 3, 4, 6, 7, 9, 11],
            ScaleMode::SuperLocrianDim => &[0, 1, 3, 4, 6, 8, 9],
            // Melodic minor modes (7 notes)
            ScaleMode::MelodicMinor => &[0, 2, 3, 5, 7, 9, 11],
            ScaleMode::DorianFlat2 => &[0, 1, 3, 5, 7, 9, 10],
            ScaleMode::LydianAug => &[0, 2, 4, 6, 8, 9, 11],
            ScaleMode::LydianDominant => &[0, 2, 4, 6, 7, 9, 10],
            ScaleMode::MixolydianFlat6 => &[0, 2, 4, 5, 7, 8, 10],
            ScaleMode::LocrianNat2 => &[0, 2, 3, 5, 6, 8, 10],
            ScaleMode::SuperLocrian => &[0, 1, 3, 4, 6, 8, 10],
            // Harmonic major modes (7 notes)
            ScaleMode::HarmonicMajor => &[0, 2, 4, 5, 7, 8, 11],
            ScaleMode::DorianFlat5 => &[0, 2, 3, 5, 6, 9, 10],
            ScaleMode::PhrygianFlat4 => &[0, 1, 3, 4, 7, 9, 10],
            ScaleMode::LydianFlat3 => &[0, 2, 3, 6, 7, 9, 11],
            ScaleMode::MixolydianFlat2 => &[0, 1, 4, 5, 7, 9, 10],
            ScaleMode::LydianAugSharp2 => &[0, 3, 4, 6, 8, 9, 11],
            ScaleMode::LocrianDoubleFlat7 => &[0, 1, 3, 5, 6, 8, 9],
            // Double harmonic modes (7 notes)
            ScaleMode::DoubleHarmonic => &[0, 1, 4, 5, 7, 8, 11],
            ScaleMode::LydianSharp2Sharp6 => &[0, 3, 4, 6, 7, 10, 11],
            ScaleMode::Ultraphrygian => &[0, 1, 3, 4, 7, 8, 9],
            ScaleMode::HungarianMinor => &[0, 2, 3, 6, 7, 8, 11],
            ScaleMode::Oriental => &[0, 1, 4, 5, 6, 9, 10],
            ScaleMode::IonianSharp2Sharp5 => &[0, 3, 4, 5, 8, 9, 11],
            ScaleMode::LocrianDoubleFlat3DoubleFlat7 => &[0, 1, 2, 5, 6, 8, 9],
            // Pentatonic (5 notes)
            ScaleMode::MajorPentatonic => &[0, 2, 4, 7, 9],
            ScaleMode::MinorPentatonic => &[0, 3, 5, 7, 10],
            ScaleMode::Hirajoshi => &[0, 2, 3, 7, 8],
            ScaleMode::InSen => &[0, 1, 5, 7, 8],
            ScaleMode::Iwato => &[0, 1, 5, 6, 10],
            ScaleMode::Yo => &[0, 2, 5, 7, 9],
            ScaleMode::Kumoi => &[0, 2, 3, 7, 9],
            ScaleMode::Pelog => &[0, 1, 3, 7, 8],
            // Blues & Bebop
            ScaleMode::MinorBlues => &[0, 3, 5, 6, 7, 10],
            ScaleMode::MajorBlues => &[0, 2, 3, 4, 7, 9],
            ScaleMode::BebopDominant => &[0, 2, 4, 5, 7, 9, 10, 11],
            // Symmetric
            ScaleMode::WholeTone => &[0, 2, 4, 6, 8, 10],
            ScaleMode::DiminishedWholeHalf => &[0, 2, 3, 5, 6, 8, 9, 11],
            ScaleMode::DiminishedHalfWhole => &[0, 1, 3, 4, 6, 7, 9, 10],
            ScaleMode::AugmentedHex => &[0, 3, 4, 7, 8, 11],
            // World scales (7 notes)
            ScaleMode::Enigmatic => &[0, 1, 4, 6, 8, 10, 11],
            ScaleMode::NeapolitanMinor => &[0, 1, 3, 5, 7, 8, 11],
            ScaleMode::NeapolitanMajor => &[0, 1, 3, 5, 7, 9, 11],
            ScaleMode::Persian => &[0, 1, 4, 5, 6, 8, 11],
            ScaleMode::HungarianMajor => &[0, 3, 4, 6, 7, 9, 10],
            // Barry Harris 6th Diminished (8 notes)
            ScaleMode::BHMajor6thDim => &[0, 2, 4, 5, 7, 8, 9, 11],
            ScaleMode::BHMinor6thDim => &[0, 2, 3, 5, 7, 8, 9, 11],
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

            ScaleMode::HarmonicMajor
            | ScaleMode::DorianFlat5
            | ScaleMode::PhrygianFlat4
            | ScaleMode::LydianFlat3
            | ScaleMode::MixolydianFlat2
            | ScaleMode::LydianAugSharp2
            | ScaleMode::LocrianDoubleFlat7 => ScaleFamily::HarmonicMajor,

            ScaleMode::DoubleHarmonic
            | ScaleMode::LydianSharp2Sharp6
            | ScaleMode::Ultraphrygian
            | ScaleMode::HungarianMinor
            | ScaleMode::Oriental
            | ScaleMode::IonianSharp2Sharp5
            | ScaleMode::LocrianDoubleFlat3DoubleFlat7 => ScaleFamily::DoubleHarmonic,

            ScaleMode::MajorPentatonic
            | ScaleMode::MinorPentatonic
            | ScaleMode::Hirajoshi
            | ScaleMode::InSen
            | ScaleMode::Iwato
            | ScaleMode::Yo
            | ScaleMode::Kumoi
            | ScaleMode::Pelog => ScaleFamily::Pentatonic,

            ScaleMode::MinorBlues | ScaleMode::MajorBlues | ScaleMode::BebopDominant => {
                ScaleFamily::Blues
            }

            ScaleMode::WholeTone
            | ScaleMode::DiminishedWholeHalf
            | ScaleMode::DiminishedHalfWhole
            | ScaleMode::AugmentedHex => ScaleFamily::Symmetric,

            ScaleMode::Enigmatic
            | ScaleMode::NeapolitanMinor
            | ScaleMode::NeapolitanMajor
            | ScaleMode::Persian
            | ScaleMode::HungarianMajor => ScaleFamily::World,

            ScaleMode::BHMajor6thDim | ScaleMode::BHMinor6thDim => ScaleFamily::BarryHarris,
        }
    }

    /// Returns all scale modes for menu display.
    ///
    /// Modes are ordered by family: Diatonic, Harmonic Minor, Melodic Minor,
    /// Harmonic Major, Double Harmonic, Pentatonic, Blues, Symmetric, World,
    /// Barry Harris. Total: 57 scales.
    pub fn all() -> &'static [ScaleMode] {
        &[
            // Diatonic (7)
            ScaleMode::Ionian,
            ScaleMode::Dorian,
            ScaleMode::Phrygian,
            ScaleMode::Lydian,
            ScaleMode::Mixolydian,
            ScaleMode::Aeolian,
            ScaleMode::Locrian,
            // Harmonic Minor (7)
            ScaleMode::HarmonicMinor,
            ScaleMode::LocrianNat6,
            ScaleMode::IonianAug,
            ScaleMode::DorianSharp4,
            ScaleMode::PhrygianDominant,
            ScaleMode::LydianSharp2,
            ScaleMode::SuperLocrianDim,
            // Melodic Minor (7)
            ScaleMode::MelodicMinor,
            ScaleMode::DorianFlat2,
            ScaleMode::LydianAug,
            ScaleMode::LydianDominant,
            ScaleMode::MixolydianFlat6,
            ScaleMode::LocrianNat2,
            ScaleMode::SuperLocrian,
            // Harmonic Major (7)
            ScaleMode::HarmonicMajor,
            ScaleMode::DorianFlat5,
            ScaleMode::PhrygianFlat4,
            ScaleMode::LydianFlat3,
            ScaleMode::MixolydianFlat2,
            ScaleMode::LydianAugSharp2,
            ScaleMode::LocrianDoubleFlat7,
            // Double Harmonic (7)
            ScaleMode::DoubleHarmonic,
            ScaleMode::LydianSharp2Sharp6,
            ScaleMode::Ultraphrygian,
            ScaleMode::HungarianMinor,
            ScaleMode::Oriental,
            ScaleMode::IonianSharp2Sharp5,
            ScaleMode::LocrianDoubleFlat3DoubleFlat7,
            // Pentatonic (8)
            ScaleMode::MajorPentatonic,
            ScaleMode::MinorPentatonic,
            ScaleMode::Hirajoshi,
            ScaleMode::InSen,
            ScaleMode::Iwato,
            ScaleMode::Yo,
            ScaleMode::Kumoi,
            ScaleMode::Pelog,
            // Blues & Bebop (3)
            ScaleMode::MinorBlues,
            ScaleMode::MajorBlues,
            ScaleMode::BebopDominant,
            // Symmetric (4)
            ScaleMode::WholeTone,
            ScaleMode::DiminishedWholeHalf,
            ScaleMode::DiminishedHalfWhole,
            ScaleMode::AugmentedHex,
            // World (5)
            ScaleMode::Enigmatic,
            ScaleMode::NeapolitanMinor,
            ScaleMode::NeapolitanMajor,
            ScaleMode::Persian,
            ScaleMode::HungarianMajor,
            // Barry Harris (2)
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
            // Diatonic
            ScaleMode::Ionian => write!(f, "Ionian (Major)"),
            ScaleMode::Dorian => write!(f, "Dorian"),
            ScaleMode::Phrygian => write!(f, "Phrygian"),
            ScaleMode::Lydian => write!(f, "Lydian"),
            ScaleMode::Mixolydian => write!(f, "Mixolydian"),
            ScaleMode::Aeolian => write!(f, "Aeolian (Minor)"),
            ScaleMode::Locrian => write!(f, "Locrian"),
            // Harmonic Minor
            ScaleMode::HarmonicMinor => write!(f, "Harmonic Minor"),
            ScaleMode::LocrianNat6 => write!(f, "Locrian Nat 6"),
            ScaleMode::IonianAug => write!(f, "Ionian Augmented"),
            ScaleMode::DorianSharp4 => write!(f, "Dorian #4"),
            ScaleMode::PhrygianDominant => write!(f, "Phrygian Dominant"),
            ScaleMode::LydianSharp2 => write!(f, "Lydian #2"),
            ScaleMode::SuperLocrianDim => write!(f, "Super Locrian Dim"),
            // Melodic Minor
            ScaleMode::MelodicMinor => write!(f, "Melodic Minor"),
            ScaleMode::DorianFlat2 => write!(f, "Dorian b2"),
            ScaleMode::LydianAug => write!(f, "Lydian Augmented"),
            ScaleMode::LydianDominant => write!(f, "Lydian Dominant"),
            ScaleMode::MixolydianFlat6 => write!(f, "Mixolydian b6"),
            ScaleMode::LocrianNat2 => write!(f, "Locrian Nat 2"),
            ScaleMode::SuperLocrian => write!(f, "Super Locrian"),
            // Harmonic Major
            ScaleMode::HarmonicMajor => write!(f, "Harmonic Major"),
            ScaleMode::DorianFlat5 => write!(f, "Dorian b5"),
            ScaleMode::PhrygianFlat4 => write!(f, "Phrygian b4"),
            ScaleMode::LydianFlat3 => write!(f, "Lydian b3"),
            ScaleMode::MixolydianFlat2 => write!(f, "Mixolydian b2"),
            ScaleMode::LydianAugSharp2 => write!(f, "Lydian Aug #2"),
            ScaleMode::LocrianDoubleFlat7 => write!(f, "Locrian bb7"),
            // Double Harmonic
            ScaleMode::DoubleHarmonic => write!(f, "Double Harmonic"),
            ScaleMode::LydianSharp2Sharp6 => write!(f, "Lydian #2 #6"),
            ScaleMode::Ultraphrygian => write!(f, "Ultraphrygian"),
            ScaleMode::HungarianMinor => write!(f, "Hungarian Minor"),
            ScaleMode::Oriental => write!(f, "Oriental"),
            ScaleMode::IonianSharp2Sharp5 => write!(f, "Ionian #2 #5"),
            ScaleMode::LocrianDoubleFlat3DoubleFlat7 => write!(f, "Locrian bb3 bb7"),
            // Pentatonic
            ScaleMode::MajorPentatonic => write!(f, "Major Pentatonic"),
            ScaleMode::MinorPentatonic => write!(f, "Minor Pentatonic"),
            ScaleMode::Hirajoshi => write!(f, "Hirajoshi"),
            ScaleMode::InSen => write!(f, "In Sen"),
            ScaleMode::Iwato => write!(f, "Iwato"),
            ScaleMode::Yo => write!(f, "Yo"),
            ScaleMode::Kumoi => write!(f, "Kumoi"),
            ScaleMode::Pelog => write!(f, "Pelog"),
            // Blues & Bebop
            ScaleMode::MinorBlues => write!(f, "Minor Blues"),
            ScaleMode::MajorBlues => write!(f, "Major Blues"),
            ScaleMode::BebopDominant => write!(f, "Bebop Dominant"),
            // Symmetric
            ScaleMode::WholeTone => write!(f, "Whole Tone"),
            ScaleMode::DiminishedWholeHalf => write!(f, "Diminished (WH)"),
            ScaleMode::DiminishedHalfWhole => write!(f, "Diminished (HW)"),
            ScaleMode::AugmentedHex => write!(f, "Augmented"),
            // World
            ScaleMode::Enigmatic => write!(f, "Enigmatic"),
            ScaleMode::NeapolitanMinor => write!(f, "Neapolitan Minor"),
            ScaleMode::NeapolitanMajor => write!(f, "Neapolitan Major"),
            ScaleMode::Persian => write!(f, "Persian"),
            ScaleMode::HungarianMajor => write!(f, "Hungarian Major"),
            // Barry Harris
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
/// | 7 | StrictCounterpoint | Note-against-note voice leading with partial Species 1 rules |
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
    /// Mode 7: Counterpoint (Species 1, basic). Note-against-note voice leading
    /// with scoring. Partial Species 1 rules: no parallel 5ths/octaves, prefers
    /// contrary/stepwise motion.
    /// Stateful: uses sliding window history for interval variety and contour.
    StrictCounterpoint,
    #[serde(alias = "barry_harris")]
    BarryHarris,
    /// Mode 9: Functional Harmony.
    FunctionalHarmony,
    /// Mode 10: Bach Chorale.
    BachChorale,
}

impl HarmonyMode {
    /// Returns mode number (1-7) for display.
    ///
    /// # Example
    ///
    /// ```
    /// use contrapunk::harmony::HarmonyMode;
    ///
    /// assert_eq!(HarmonyMode::PassThrough.number(), 1);
    /// assert_eq!(HarmonyMode::StrictCounterpoint.number(), 7);
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
            HarmonyMode::FunctionalHarmony => 9,
            HarmonyMode::BachChorale => 10,
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
            HarmonyMode::FunctionalHarmony,
            HarmonyMode::BachChorale,
        ]
    }

    /// Returns short description for menu display.
    pub fn description(&self) -> &'static str {
        match self {
            HarmonyMode::PassThrough => "Pass-through (no harmony)",
            HarmonyMode::DiatonicThirds => "Parallel Thirds",
            HarmonyMode::DiatonicFourths => "Parallel Fourths",
            HarmonyMode::RandomBelow => "Random diatonic below",
            HarmonyMode::RandomBelowNoSeconds => "Random Below (consonant)",
            HarmonyMode::ContraryMotion => "Contrary motion",
            HarmonyMode::StrictCounterpoint => "Counterpoint (Species 1, basic)",
            HarmonyMode::BarryHarris => "Barry Harris (drop-2 voicings)",
            HarmonyMode::FunctionalHarmony => "Functional Harmony",
            HarmonyMode::BachChorale => "Bach Chorale",
        }
    }

    /// Returns a longer tooltip explaining what the mode's algorithm actually does.
    pub fn tooltip(&self) -> &'static str {
        match self {
            HarmonyMode::PassThrough => "Notes pass unchanged to output",
            HarmonyMode::DiatonicThirds => {
                "Adds +2 scale degrees per voice. Multiple voices stack into 7th chords."
            }
            HarmonyMode::DiatonicFourths => {
                "Adds +3 scale degrees per voice. Multiple voices stack into extended chords."
            }
            HarmonyMode::RandomBelow => "Random diatonic interval (2nd-7th) below the melody",
            HarmonyMode::RandomBelowNoSeconds => {
                "Random diatonic interval below, excluding dissonant 2nds"
            }
            HarmonyMode::ContraryMotion => {
                "Harmony moves opposite to melody direction. Stateful \u{2014} tracks previous notes."
            }
            HarmonyMode::StrictCounterpoint => {
                "Note-against-note voice leading with scoring. No parallel 5ths/octaves, \
                 prefers contrary/stepwise motion. Partial Species 1 rules."
            }
            HarmonyMode::BarryHarris => {
                "4-voice drop-2 voicings using 8-note BH scale. Chord tones produce \
                 6th chord voicings, passing tones produce dim7 voicings. Requires \
                 BH Major or Minor 6th Dim scale."
            }
            HarmonyMode::FunctionalHarmony => { "Reactive chord selection." }
            HarmonyMode::BachChorale => { "4-voice SATB." }
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct BeatPhase {
    pub position: f32,
    pub is_strong: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    /// All 57 scales have intervals that start at 0, are ascending,
    /// stay within [0,11], contain no duplicates, and have 5-8 notes.
    #[test]
    fn scale_invariants() {
        for &mode in ScaleMode::all() {
            let iv = mode.intervals();
            let name = format!("{:?}", mode);

            // Length in [5, 8]
            assert!(
                (5..=8).contains(&iv.len()),
                "{}: expected 5-8 notes, got {}",
                name,
                iv.len()
            );

            // Starts at 0
            assert_eq!(iv[0], 0, "{}: first interval must be 0", name);

            // Ascending and within [0, 11]
            for i in 1..iv.len() {
                assert!(
                    iv[i] > iv[i - 1],
                    "{}: intervals not ascending at index {} ({} <= {})",
                    name,
                    i,
                    iv[i],
                    iv[i - 1]
                );
                assert!(
                    iv[i] <= 11,
                    "{}: interval {} out of range at index {}",
                    name,
                    iv[i],
                    i
                );
            }

            // No duplicates
            let unique: HashSet<u8> = iv.iter().copied().collect();
            assert_eq!(
                unique.len(),
                iv.len(),
                "{}: duplicate intervals found",
                name
            );
        }
    }

    /// No two ScaleMode variants share identical interval sets.
    #[test]
    fn no_duplicate_intervals() {
        let all = ScaleMode::all();
        for i in 0..all.len() {
            for j in (i + 1)..all.len() {
                assert_ne!(
                    all[i].intervals(),
                    all[j].intervals(),
                    "{:?} and {:?} have identical intervals",
                    all[i],
                    all[j]
                );
            }
        }
    }

    /// family() does not panic for any variant, and every family has >= 1 scale.
    #[test]
    fn every_scale_has_family_and_every_family_has_scales() {
        for &mode in ScaleMode::all() {
            let _ = mode.family();
        }
        for &family in ScaleFamily::all() {
            let scales = ScaleMode::all_in_family(family);
            assert!(!scales.is_empty(), "Family {:?} has no scales", family);
        }
    }

    /// Every scale in a family reports that family.
    #[test]
    fn family_round_trip() {
        for &family in ScaleFamily::all() {
            for &mode in &ScaleMode::all_in_family(family) {
                assert_eq!(
                    mode.family(),
                    family,
                    "{:?} is in all_in_family({:?}) but reports family {:?}",
                    mode,
                    family,
                    mode.family()
                );
            }
        }
    }

    /// Verify known reference values for select scales.
    #[test]
    fn known_value_spot_checks() {
        assert_eq!(ScaleMode::Ionian.intervals(), &[0, 2, 4, 5, 7, 9, 11]);
        assert_eq!(ScaleMode::Dorian.intervals(), &[0, 2, 3, 5, 7, 9, 10]);
        assert_eq!(ScaleMode::MinorPentatonic.intervals(), &[0, 3, 5, 7, 10]);
        assert_eq!(ScaleMode::MajorPentatonic.intervals(), &[0, 2, 4, 7, 9]);
        assert_eq!(ScaleMode::WholeTone.intervals(), &[0, 2, 4, 6, 8, 10]);
        assert_eq!(
            ScaleMode::DiminishedHalfWhole.intervals(),
            &[0, 1, 3, 4, 6, 7, 9, 10]
        );
        assert_eq!(ScaleMode::MinorBlues.intervals(), &[0, 3, 5, 6, 7, 10]);
        assert_eq!(
            ScaleMode::HarmonicMajor.intervals(),
            &[0, 2, 4, 5, 7, 8, 11]
        );
        assert_eq!(
            ScaleMode::DoubleHarmonic.intervals(),
            &[0, 1, 4, 5, 7, 8, 11]
        );
        assert_eq!(ScaleMode::Persian.intervals(), &[0, 1, 4, 5, 6, 8, 11]);
        assert_eq!(
            ScaleMode::BebopDominant.intervals(),
            &[0, 2, 4, 5, 7, 9, 10, 11]
        );
    }

    /// Verify modal relationships: MinorPentatonic is mode 5 of MajorPentatonic,
    /// DiminishedHalfWhole is mode 2 of DiminishedWholeHalf.
    #[test]
    fn modal_relations() {
        fn compute_mode(parent: &[u8], mode_idx: usize) -> Vec<u8> {
            let root_pc = parent[mode_idx];
            let mut result: Vec<u8> = parent.iter().map(|&p| (p + 12 - root_pc) % 12).collect();
            result.sort();
            result
        }

        let major = ScaleMode::MajorPentatonic.intervals();
        assert_eq!(
            compute_mode(major, 4),
            ScaleMode::MinorPentatonic.intervals().to_vec()
        );
        assert_eq!(compute_mode(major, 3), ScaleMode::Yo.intervals().to_vec());

        let wh = ScaleMode::DiminishedWholeHalf.intervals();
        assert_eq!(
            compute_mode(wh, 1),
            ScaleMode::DiminishedHalfWhole.intervals().to_vec()
        );

        let dh = ScaleMode::DoubleHarmonic.intervals();
        assert_eq!(
            compute_mode(dh, 3),
            ScaleMode::HungarianMinor.intervals().to_vec()
        );
    }

    /// all() contains exactly 57 variants.
    #[test]
    fn all_variants_count() {
        assert_eq!(ScaleMode::all().len(), 57);
    }

    /// ScaleFamily::all() contains exactly 10 families.
    #[test]
    fn all_families_count() {
        assert_eq!(ScaleFamily::all().len(), 10);
    }

    /// The total number of scales across all families equals 57.
    #[test]
    fn family_totals_match() {
        let total: usize = ScaleFamily::all()
            .iter()
            .map(|&f| ScaleMode::all_in_family(f).len())
            .sum();
        assert_eq!(total, 57);
    }

    /// Display impl does not panic for any variant.
    #[test]
    fn display_all_variants() {
        for &mode in ScaleMode::all() {
            let s = format!("{}", mode);
            assert!(!s.is_empty(), "{:?} has empty display", mode);
        }
        for &family in ScaleFamily::all() {
            let s = format!("{}", family);
            assert!(!s.is_empty(), "{:?} has empty display", family);
        }
    }
}
