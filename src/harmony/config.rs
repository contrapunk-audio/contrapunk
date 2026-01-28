/// Musical keys (C through B, 12 options)
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum OctaveMode {
    /// No octave modification - harmonies stay at their generated pitch
    #[default]
    None,
    /// Spread voices across octaves - each successive voice is +1 octave higher
    Spread,
    /// Bass/Treble split - harmonies below melody go down an octave, above go up
    BassTrebleSplit,
    /// Mirror - duplicate harmonies in multiple octaves (shifts all harmonies ±1 octave)
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

/// Harmony modes (1-7)
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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
