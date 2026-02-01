use serde::{Serialize, Deserialize};

/// Musical keys (C through B, 12 options)
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Key {
    C, Db, D, Eb, E, F, Gb, G, Ab, A, Bb, B,
}

impl Key {
    /// Returns semitones from C (0-11)
    pub fn semitones_from_c(&self) -> u8 {
        match self {
            Key::C  => 0,  Key::Db => 1,  Key::D  => 2,
            Key::Eb => 3,  Key::E  => 4,  Key::F  => 5,
            Key::Gb => 6,  Key::G  => 7,  Key::Ab => 8,
            Key::A  => 9,  Key::Bb => 10, Key::B  => 11,
        }
    }

    /// Returns all keys in order for menu display
    pub fn all() -> &'static [Key] {
        &[Key::C, Key::Db, Key::D, Key::Eb, Key::E, Key::F,
          Key::Gb, Key::G, Key::Ab, Key::A, Key::Bb, Key::B]
    }
}

impl std::fmt::Display for Key {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

/// Octave placement modes for harmony voices
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OctaveMode {
    /// No octave modification - harmonies stay at their generated pitch
    #[default]
    None,
    /// Spread voices across octaves - each successive voice is +1 octave higher
    Spread,
    /// Bass/Treble split - harmonies below melody go down an octave, above go up
    BassTrebleSplit,
    /// Mirror - duplicate each harmony note in +1 and -1 octaves (tripling harmony notes)
    Mirror,
}

impl OctaveMode {
    /// Returns all octave modes for menu display
    pub fn all() -> &'static [OctaveMode] {
        &[
            OctaveMode::None,
            OctaveMode::Spread,
            OctaveMode::BassTrebleSplit,
            OctaveMode::Mirror,
        ]
    }

    /// Returns description for menu display
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

/// Scale family grouping for UI display
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScaleFamily {
    Church,
    HarmonicMinor,
    MelodicMinor,
    Exotic,
    BarryHarris,
}

impl ScaleFamily {
    /// Returns all scale families
    pub fn all() -> &'static [ScaleFamily] {
        &[
            ScaleFamily::Church,
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
            ScaleFamily::Church => write!(f, "Church Modes"),
            ScaleFamily::HarmonicMinor => write!(f, "Harmonic Minor"),
            ScaleFamily::MelodicMinor => write!(f, "Melodic Minor"),
            ScaleFamily::Exotic => write!(f, "Exotic"),
            ScaleFamily::BarryHarris => write!(f, "Barry Harris"),
        }
    }
}

/// Scale modes (~28 variants grouped by family)
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScaleMode {
    // Church modes (7)
    Ionian,
    Dorian,
    Phrygian,
    Lydian,
    Mixolydian,
    Aeolian,
    Locrian,
    // Harmonic minor modes (7)
    HarmonicMinor,
    LocrianNat6,
    IonianAug,
    DorianSharp4,
    PhrygianDominant,
    LydianSharp2,
    SuperLocrianDim,
    // Melodic minor modes (7)
    MelodicMinor,
    DorianFlat2,
    LydianAug,
    LydianDominant,
    MixolydianFlat6,
    LocrianNat2,
    SuperLocrian,
    // Exotic (5)
    DoubleHarmonic,
    HungarianMinor,
    Enigmatic,
    NeapolitanMinor,
    NeapolitanMajor,
    // Barry Harris (2)
    BHMajor6thDim,
    BHMinor6thDim,
}

impl ScaleMode {
    /// Returns semitone offsets from tonic for each scale degree.
    /// Most scales return 7 elements; Barry Harris scales return 8.
    pub fn intervals(&self) -> Vec<u8> {
        match self {
            // Church modes
            ScaleMode::Ionian       => vec![0, 2, 4, 5, 7, 9, 11],
            ScaleMode::Dorian       => vec![0, 2, 3, 5, 7, 9, 10],
            ScaleMode::Phrygian     => vec![0, 1, 3, 5, 7, 8, 10],
            ScaleMode::Lydian       => vec![0, 2, 4, 6, 7, 9, 11],
            ScaleMode::Mixolydian   => vec![0, 2, 4, 5, 7, 9, 10],
            ScaleMode::Aeolian      => vec![0, 2, 3, 5, 7, 8, 10],
            ScaleMode::Locrian      => vec![0, 1, 3, 5, 6, 8, 10],
            // Harmonic minor modes
            ScaleMode::HarmonicMinor    => vec![0, 2, 3, 5, 7, 8, 11],
            ScaleMode::LocrianNat6      => vec![0, 1, 3, 5, 6, 9, 10],
            ScaleMode::IonianAug        => vec![0, 2, 4, 5, 8, 9, 11],
            ScaleMode::DorianSharp4     => vec![0, 2, 3, 6, 7, 9, 10],
            ScaleMode::PhrygianDominant => vec![0, 1, 4, 5, 7, 8, 10],
            ScaleMode::LydianSharp2     => vec![0, 3, 4, 6, 7, 9, 11],
            ScaleMode::SuperLocrianDim  => vec![0, 1, 3, 4, 6, 8, 9],
            // Melodic minor modes
            ScaleMode::MelodicMinor     => vec![0, 2, 3, 5, 7, 9, 11],
            ScaleMode::DorianFlat2      => vec![0, 1, 3, 5, 7, 9, 10],
            ScaleMode::LydianAug        => vec![0, 2, 4, 6, 8, 9, 11],
            ScaleMode::LydianDominant   => vec![0, 2, 4, 6, 7, 9, 10],
            ScaleMode::MixolydianFlat6  => vec![0, 2, 4, 5, 7, 8, 10],
            ScaleMode::LocrianNat2      => vec![0, 2, 3, 5, 6, 8, 10],
            ScaleMode::SuperLocrian     => vec![0, 1, 3, 4, 6, 8, 10],
            // Exotic
            ScaleMode::DoubleHarmonic   => vec![0, 1, 4, 5, 7, 8, 11],
            ScaleMode::HungarianMinor   => vec![0, 2, 3, 6, 7, 8, 11],
            ScaleMode::Enigmatic        => vec![0, 1, 4, 6, 8, 10, 11],
            ScaleMode::NeapolitanMinor  => vec![0, 1, 3, 5, 7, 8, 11],
            ScaleMode::NeapolitanMajor  => vec![0, 1, 3, 5, 7, 9, 11],
            // Barry Harris 6th Diminished (8-note)
            ScaleMode::BHMajor6thDim    => vec![0, 2, 4, 5, 7, 8, 9, 11],
            ScaleMode::BHMinor6thDim    => vec![0, 2, 3, 5, 7, 8, 9, 11],
        }
    }

    /// Returns the scale family for this mode.
    pub fn family(&self) -> ScaleFamily {
        match self {
            ScaleMode::Ionian | ScaleMode::Dorian | ScaleMode::Phrygian |
            ScaleMode::Lydian | ScaleMode::Mixolydian | ScaleMode::Aeolian |
            ScaleMode::Locrian => ScaleFamily::Church,

            ScaleMode::HarmonicMinor | ScaleMode::LocrianNat6 | ScaleMode::IonianAug |
            ScaleMode::DorianSharp4 | ScaleMode::PhrygianDominant |
            ScaleMode::LydianSharp2 | ScaleMode::SuperLocrianDim => ScaleFamily::HarmonicMinor,

            ScaleMode::MelodicMinor | ScaleMode::DorianFlat2 | ScaleMode::LydianAug |
            ScaleMode::LydianDominant | ScaleMode::MixolydianFlat6 |
            ScaleMode::LocrianNat2 | ScaleMode::SuperLocrian => ScaleFamily::MelodicMinor,

            ScaleMode::DoubleHarmonic | ScaleMode::HungarianMinor |
            ScaleMode::Enigmatic | ScaleMode::NeapolitanMinor |
            ScaleMode::NeapolitanMajor => ScaleFamily::Exotic,

            ScaleMode::BHMajor6thDim | ScaleMode::BHMinor6thDim => ScaleFamily::BarryHarris,
        }
    }

    /// Returns all scale modes for menu display.
    pub fn all() -> &'static [ScaleMode] {
        &[
            // Church
            ScaleMode::Ionian, ScaleMode::Dorian, ScaleMode::Phrygian,
            ScaleMode::Lydian, ScaleMode::Mixolydian, ScaleMode::Aeolian,
            ScaleMode::Locrian,
            // Harmonic Minor
            ScaleMode::HarmonicMinor, ScaleMode::LocrianNat6, ScaleMode::IonianAug,
            ScaleMode::DorianSharp4, ScaleMode::PhrygianDominant,
            ScaleMode::LydianSharp2, ScaleMode::SuperLocrianDim,
            // Melodic Minor
            ScaleMode::MelodicMinor, ScaleMode::DorianFlat2, ScaleMode::LydianAug,
            ScaleMode::LydianDominant, ScaleMode::MixolydianFlat6,
            ScaleMode::LocrianNat2, ScaleMode::SuperLocrian,
            // Exotic
            ScaleMode::DoubleHarmonic, ScaleMode::HungarianMinor,
            ScaleMode::Enigmatic, ScaleMode::NeapolitanMinor,
            ScaleMode::NeapolitanMajor,
            // Barry Harris
            ScaleMode::BHMajor6thDim, ScaleMode::BHMinor6thDim,
        ]
    }

    /// Returns all scale modes in a given family.
    pub fn all_in_family(family: ScaleFamily) -> Vec<ScaleMode> {
        ScaleMode::all().iter().filter(|m| m.family() == family).copied().collect()
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

/// Harmony modes (1-7)
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HarmonyMode {
    PassThrough,       // Mode 1: Forward as-is
    DiatonicThirds,    // Mode 2: Add diatonic thirds above
    DiatonicFourths,   // Mode 3: Add diatonic fourths above
    RandomBelow,       // Mode 4: Random diatonic interval below
    RandomBelowNoSeconds, // Mode 5: Random below excluding seconds
    ContraryMotion,    // Mode 6: Harmony moves opposite to melody
    StrictCounterpoint, // Mode 7: Traditional voice leading rules
}

impl HarmonyMode {
    /// Returns mode number (1-7) for display
    pub fn number(&self) -> u8 {
        match self {
            HarmonyMode::PassThrough => 1,
            HarmonyMode::DiatonicThirds => 2,
            HarmonyMode::DiatonicFourths => 3,
            HarmonyMode::RandomBelow => 4,
            HarmonyMode::RandomBelowNoSeconds => 5,
            HarmonyMode::ContraryMotion => 6,
            HarmonyMode::StrictCounterpoint => 7,
        }
    }

    /// Returns all modes in order for menu display
    pub fn all() -> &'static [HarmonyMode] {
        &[
            HarmonyMode::PassThrough,
            HarmonyMode::DiatonicThirds,
            HarmonyMode::DiatonicFourths,
            HarmonyMode::RandomBelow,
            HarmonyMode::RandomBelowNoSeconds,
            HarmonyMode::ContraryMotion,
            HarmonyMode::StrictCounterpoint,
        ]
    }

    /// Returns description for menu display
    pub fn description(&self) -> &'static str {
        match self {
            HarmonyMode::PassThrough => "Pass-through (no harmony)",
            HarmonyMode::DiatonicThirds => "Diatonic thirds above",
            HarmonyMode::DiatonicFourths => "Diatonic fourths above",
            HarmonyMode::RandomBelow => "Random diatonic below",
            HarmonyMode::RandomBelowNoSeconds => "Random below (no 2nds)",
            HarmonyMode::ContraryMotion => "Contrary motion",
            HarmonyMode::StrictCounterpoint => "Strict counterpoint",
        }
    }
}
