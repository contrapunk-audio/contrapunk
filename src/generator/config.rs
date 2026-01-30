/// Generator mode determines how selected notes are played.
#[derive(Clone, Debug, PartialEq)]
pub enum GeneratorMode {
    /// Play all selected notes simultaneously (pass-through).
    HeldNotes,
    /// Play chord tones simultaneously.
    Chord(ChordType),
    /// Arpeggiate through selected notes.
    Arpeggio(ArpDirection),
    /// Walk through selected notes sequentially.
    ScaleRunner,
    /// Pick random notes from selected set.
    RandomDiatonic,
}

/// Arpeggio direction.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ArpDirection {
    Up,
    Down,
    UpDown,
}

/// Chord quality for chord mode.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ChordType {
    Major,
    Minor,
    Dim,
    Aug,
    Maj7,
    Min7,
    Dom7,
    Dim7,
    HalfDim7,
}

impl ChordType {
    /// Returns semitone intervals from root for this chord type.
    pub fn intervals(&self) -> Vec<u8> {
        match self {
            ChordType::Major => vec![0, 4, 7],
            ChordType::Minor => vec![0, 3, 7],
            ChordType::Dim => vec![0, 3, 6],
            ChordType::Aug => vec![0, 4, 8],
            ChordType::Maj7 => vec![0, 4, 7, 11],
            ChordType::Min7 => vec![0, 3, 7, 10],
            ChordType::Dom7 => vec![0, 4, 7, 10],
            ChordType::Dim7 => vec![0, 3, 6, 9],
            ChordType::HalfDim7 => vec![0, 3, 6, 10],
        }
    }
}

/// Events produced by the generator.
#[derive(Clone, Debug, PartialEq)]
pub enum GeneratorEvent {
    /// Note on with velocity.
    NoteOn(wmidi::Note, u8),
    /// Note off.
    NoteOff(wmidi::Note),
}
