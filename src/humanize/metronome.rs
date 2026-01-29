use wmidi::{Channel, MidiMessage, Note, Velocity};

/// Generates MIDI metronome clicks on channel 10 (percussion).
pub struct Metronome {
    pub enabled: bool,
}

impl Metronome {
    pub fn new() -> Self {
        Self { enabled: false }
    }

    /// Generate a NoteOn click for the given beat number.
    /// Beat 0 (downbeat) uses note 76 (high woodblock), others use 77 (low woodblock).
    pub fn generate_click(&self, beat_number: u8) -> Vec<u8> {
        let midi_note = if beat_number == 0 { 76 } else { 77 };
        let note = Note::try_from(midi_note).unwrap();
        let vel = Velocity::try_from(100u8).unwrap();
        let msg = MidiMessage::NoteOn(Channel::Ch10, note, vel);
        let mut buf = vec![0u8; msg.bytes_size()];
        msg.copy_to_slice(&mut buf).unwrap();
        buf
    }

    /// Generate a NoteOff click for the given beat number.
    pub fn generate_click_off(&self, beat_number: u8) -> Vec<u8> {
        let midi_note = if beat_number == 0 { 76 } else { 77 };
        let note = Note::try_from(midi_note).unwrap();
        let vel = Velocity::try_from(0u8).unwrap();
        let msg = MidiMessage::NoteOff(Channel::Ch10, note, vel);
        let mut buf = vec![0u8; msg.bytes_size()];
        msg.copy_to_slice(&mut buf).unwrap();
        buf
    }
}
