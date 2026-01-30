use rand::seq::SliceRandom;

use super::config::*;

/// Beat-driven note generator that produces MIDI events based on mode and selected notes.
pub struct NoteGenerator {
    mode: GeneratorMode,
    selected_notes: Vec<wmidi::Note>,
    active_notes: Vec<wmidi::Note>,
    sequence_index: usize,
    ascending: bool,
    enabled: bool,
    note_duration_beats: f64,
    last_beat_position: f64,
    velocity: u8,
}

impl NoteGenerator {
    pub fn new() -> Self {
        Self {
            mode: GeneratorMode::HeldNotes,
            selected_notes: Vec::new(),
            active_notes: Vec::new(),
            sequence_index: 0,
            ascending: true,
            enabled: false,
            note_duration_beats: 0.5,
            last_beat_position: -1.0,
            velocity: 100,
        }
    }

    /// Process a beat tick and return any generated events.
    pub fn tick(&mut self, beat_pos: f64, _bpm: f64) -> Vec<GeneratorEvent> {
        if !self.enabled || self.selected_notes.is_empty() {
            return Vec::new();
        }

        let current_step = (beat_pos / self.note_duration_beats).floor() as i64;
        let last_step = (self.last_beat_position / self.note_duration_beats).floor() as i64;

        // Detect new step or beat wrap-around (new bar).
        let is_new_step = current_step != last_step || beat_pos < self.last_beat_position;

        self.last_beat_position = beat_pos;

        if !is_new_step {
            return Vec::new();
        }

        let mut events = Vec::new();

        // Release all active notes first.
        for note in self.active_notes.drain(..) {
            events.push(GeneratorEvent::NoteOff(note));
        }

        // Generate new notes based on mode.
        let new_notes = self.generate_notes();
        for &note in &new_notes {
            events.push(GeneratorEvent::NoteOn(note, self.velocity));
        }
        self.active_notes = new_notes;

        events
    }

    fn generate_notes(&mut self) -> Vec<wmidi::Note> {
        match &self.mode {
            GeneratorMode::HeldNotes | GeneratorMode::Chord(_) => {
                // Play all selected notes simultaneously.
                self.selected_notes.clone()
            }
            GeneratorMode::Arpeggio(dir) => {
                if self.selected_notes.is_empty() {
                    return Vec::new();
                }
                let len = self.selected_notes.len();
                let note = self.selected_notes[self.sequence_index % len];

                match dir {
                    ArpDirection::Up => {
                        self.sequence_index = (self.sequence_index + 1) % len;
                    }
                    ArpDirection::Down => {
                        if self.sequence_index == 0 {
                            self.sequence_index = len.saturating_sub(1);
                        } else {
                            self.sequence_index -= 1;
                        }
                    }
                    ArpDirection::UpDown => {
                        if len <= 1 {
                            self.sequence_index = 0;
                        } else if self.ascending {
                            if self.sequence_index >= len - 1 {
                                self.ascending = false;
                                self.sequence_index = self.sequence_index.saturating_sub(1);
                            } else {
                                self.sequence_index += 1;
                            }
                        } else {
                            if self.sequence_index == 0 {
                                self.ascending = true;
                                self.sequence_index = 1.min(len - 1);
                            } else {
                                self.sequence_index -= 1;
                            }
                        }
                    }
                }

                vec![note]
            }
            GeneratorMode::ScaleRunner => {
                if self.selected_notes.is_empty() {
                    return Vec::new();
                }
                let len = self.selected_notes.len();
                let note = self.selected_notes[self.sequence_index % len];
                self.sequence_index = (self.sequence_index + 1) % len;
                vec![note]
            }
            GeneratorMode::RandomDiatonic => {
                let mut rng = rand::thread_rng();
                self.selected_notes
                    .choose(&mut rng)
                    .map(|&n| vec![n])
                    .unwrap_or_default()
            }
        }
    }

    /// Send NoteOff for all active notes and reset state.
    pub fn all_notes_off(&mut self) -> Vec<GeneratorEvent> {
        self.sequence_index = 0;
        self.ascending = true;
        self.active_notes
            .drain(..)
            .map(GeneratorEvent::NoteOff)
            .collect()
    }

    /// Change mode, releasing all active notes first.
    pub fn set_mode(&mut self, mode: GeneratorMode) -> Vec<GeneratorEvent> {
        let events = self.all_notes_off();
        self.mode = mode;
        events
    }

    /// Change selected notes, releasing all active notes first.
    pub fn set_selected_notes(&mut self, notes: Vec<wmidi::Note>) -> Vec<GeneratorEvent> {
        let events = self.all_notes_off();
        self.selected_notes = notes;
        events
    }

    // --- Getters ---

    pub fn enabled(&self) -> bool {
        self.enabled
    }

    pub fn mode(&self) -> &GeneratorMode {
        &self.mode
    }

    pub fn selected_notes(&self) -> &[wmidi::Note] {
        &self.selected_notes
    }

    pub fn velocity(&self) -> u8 {
        self.velocity
    }

    pub fn note_duration_beats(&self) -> f64 {
        self.note_duration_beats
    }

    // --- Setters ---

    pub fn set_enabled(&mut self, enabled: bool) -> Vec<GeneratorEvent> {
        if !enabled && self.enabled {
            self.enabled = false;
            return self.all_notes_off();
        }
        self.enabled = enabled;
        Vec::new()
    }

    pub fn set_velocity(&mut self, velocity: u8) {
        self.velocity = velocity;
    }

    pub fn set_note_duration_beats(&mut self, beats: f64) {
        self.note_duration_beats = beats;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn note(n: u8) -> wmidi::Note {
        wmidi::Note::from_u8_lossy(n)
    }

    #[test]
    fn test_new_defaults() {
        let gen = NoteGenerator::new();
        assert!(!gen.enabled());
        assert_eq!(gen.velocity(), 100);
        assert_eq!(*gen.mode(), GeneratorMode::HeldNotes);
    }

    #[test]
    fn test_disabled_produces_nothing() {
        let mut gen = NoteGenerator::new();
        gen.selected_notes = vec![note(60)];
        assert!(gen.tick(0.0, 120.0).is_empty());
    }

    #[test]
    fn test_held_notes_produces_all() {
        let mut gen = NoteGenerator::new();
        gen.enabled = true;
        gen.selected_notes = vec![note(60), note(64), note(67)];
        let events = gen.tick(0.0, 120.0);
        assert_eq!(events.len(), 3); // 3 NoteOn
        assert!(matches!(events[0], GeneratorEvent::NoteOn(_, 100)));
    }

    #[test]
    fn test_arpeggio_up() {
        let mut gen = NoteGenerator::new();
        gen.enabled = true;
        gen.mode = GeneratorMode::Arpeggio(ArpDirection::Up);
        gen.selected_notes = vec![note(60), note(64), note(67)];

        let e1 = gen.tick(0.0, 120.0);
        assert_eq!(e1.len(), 1);
        assert!(matches!(e1[0], GeneratorEvent::NoteOn(n, _) if n == note(60)));

        let e2 = gen.tick(0.5, 120.0);
        // NoteOff for previous + NoteOn for next
        assert_eq!(e2.len(), 2);
        assert!(matches!(e2[1], GeneratorEvent::NoteOn(n, _) if n == note(64)));
    }

    #[test]
    fn test_set_mode_clears_active() {
        let mut gen = NoteGenerator::new();
        gen.enabled = true;
        gen.selected_notes = vec![note(60)];
        gen.tick(0.0, 120.0); // activate a note
        assert_eq!(gen.active_notes.len(), 1);

        let events = gen.set_mode(GeneratorMode::ScaleRunner);
        assert!(events.iter().any(|e| matches!(e, GeneratorEvent::NoteOff(_))));
        assert!(gen.active_notes.is_empty());
    }

    #[test]
    fn test_set_enabled_false_sends_note_off() {
        let mut gen = NoteGenerator::new();
        gen.enabled = true;
        gen.selected_notes = vec![note(60)];
        gen.tick(0.0, 120.0);

        let events = gen.set_enabled(false);
        assert!(events.iter().any(|e| matches!(e, GeneratorEvent::NoteOff(_))));
    }

    #[test]
    fn test_chord_intervals() {
        assert_eq!(ChordType::Major.intervals(), vec![0, 4, 7]);
        assert_eq!(ChordType::Minor.intervals(), vec![0, 3, 7]);
        assert_eq!(ChordType::Dom7.intervals(), vec![0, 4, 7, 10]);
    }
}
