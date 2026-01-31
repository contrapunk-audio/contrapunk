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

/// Scale modes (7 church modes + harmonic minor + melodic minor)
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScaleMode {
    Ionian,
    Dorian,
    Phrygian,
    Lydian,
    Mixolydian,
    Aeolian,
    Locrian,
    HarmonicMinor,
    MelodicMinor,
}

impl ScaleMode {
    /// Returns semitone offsets from tonic for each scale degree (0-6).
    pub fn intervals(&self) -> [u8; 7] {
        match self {
            ScaleMode::Ionian       => [0, 2, 4, 5, 7, 9, 11],
            ScaleMode::Dorian       => [0, 2, 3, 5, 7, 9, 10],
            ScaleMode::Phrygian     => [0, 1, 3, 5, 7, 8, 10],
            ScaleMode::Lydian       => [0, 2, 4, 6, 7, 9, 11],
            ScaleMode::Mixolydian   => [0, 2, 4, 5, 7, 9, 10],
            ScaleMode::Aeolian      => [0, 2, 3, 5, 7, 8, 10],
            ScaleMode::Locrian      => [0, 1, 3, 5, 6, 8, 10],
            ScaleMode::HarmonicMinor => [0, 2, 3, 5, 7, 8, 11],
            ScaleMode::MelodicMinor => [0, 2, 3, 5, 7, 9, 11],
        }
    }

    /// Returns all scale modes for menu display.
    pub fn all() -> &'static [ScaleMode] {
        &[
            ScaleMode::Ionian,
            ScaleMode::Dorian,
            ScaleMode::Phrygian,
            ScaleMode::Lydian,
            ScaleMode::Mixolydian,
            ScaleMode::Aeolian,
            ScaleMode::Locrian,
            ScaleMode::HarmonicMinor,
            ScaleMode::MelodicMinor,
        ]
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
            ScaleMode::MelodicMinor => write!(f, "Melodic Minor"),
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
