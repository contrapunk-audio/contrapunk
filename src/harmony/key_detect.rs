use crate::harmony::config::{Key, ScaleMode};

/// Minimum notes before the detector will produce a result.
const MIN_NOTES: usize = 4;

/// Decay factor applied to the histogram each note (exponential weighting
/// toward recent notes). 0.85 means each old note loses 15% weight per new note.
const DECAY: f32 = 0.85;

/// The detected key must score this much higher than the runner-up
/// (as a fraction of the winner's score) to be accepted.
const CONFIDENCE_MARGIN: f32 = 0.15;

/// Auto-detects the musical key (tonic) from a stream of MIDI notes.
///
/// Uses a decay-weighted pitch class histogram scored against the user's
/// selected scale mode. Only the tonic is detected — the mode stays as
/// the user selected it.
#[derive(Debug)]
pub struct KeyDetector {
    /// Weighted pitch class histogram (12 bins, one per semitone).
    histogram: [f32; 12],
    /// Total notes fed since last reset.
    note_count: usize,
    /// The scale mode to score against.
    scale_mode: ScaleMode,
    /// Last detected key (None until confidence is reached).
    detected: Option<Key>,
}

impl KeyDetector {
    pub fn new(scale_mode: ScaleMode) -> Self {
        Self {
            histogram: [0.0; 12],
            note_count: 0,
            scale_mode,
            detected: None,
        }
    }

    /// Feed a MIDI note number (0-127). Returns the detected key if
    /// confidence is sufficient, or the previous detection otherwise.
    pub fn feed(&mut self, midi_note: u8) -> Option<Key> {
        let pc = (midi_note % 12) as usize;

        // Decay old data, then add the new note
        for bin in self.histogram.iter_mut() {
            *bin *= DECAY;
        }
        self.histogram[pc] += 1.0;
        self.note_count += 1;

        if self.note_count < MIN_NOTES {
            return self.detected;
        }

        self.detect()
    }

    /// Update the scale mode used for scoring. Resets detection state.
    pub fn set_scale_mode(&mut self, mode: ScaleMode) {
        self.scale_mode = mode;
        self.detected = None;
    }

    /// Reset the detector (e.g. when the user manually sets a key).
    pub fn reset(&mut self) {
        self.histogram = [0.0; 12];
        self.note_count = 0;
        self.detected = None;
    }

    /// Score each possible tonic and pick the best.
    fn detect(&mut self) -> Option<Key> {
        let intervals = self.scale_mode.intervals();
        let mut best_score = f32::NEG_INFINITY;
        let mut second_score = f32::NEG_INFINITY;
        let mut best_tonic: u8 = 0;

        for tonic in 0u8..12 {
            let score = self.score_tonic(tonic, intervals);
            if score > best_score {
                second_score = best_score;
                best_score = score;
                best_tonic = tonic;
            } else if score > second_score {
                second_score = score;
            }
        }

        // Require a margin between winner and runner-up
        if best_score > 0.0
            && (second_score <= 0.0
                || (best_score - second_score) / best_score >= CONFIDENCE_MARGIN)
        {
            let key = key_from_semitones(best_tonic);
            self.detected = Some(key);
        }

        self.detected
    }

    /// Score a candidate tonic: sum histogram weight for in-scale pitch classes,
    /// subtract a penalty for out-of-scale pitch classes that are present.
    fn score_tonic(&self, tonic: u8, intervals: &[u8]) -> f32 {
        let mut in_scale = 0.0f32;
        let mut out_scale = 0.0f32;

        for (pc, &weight) in self.histogram.iter().enumerate() {
            if weight < 0.01 {
                continue;
            }
            let relative = ((pc as u8 + 12) - tonic) % 12;
            if intervals.contains(&relative) {
                in_scale += weight;
            } else {
                out_scale += weight;
            }
        }

        in_scale - out_scale * 0.5
    }
}

fn key_from_semitones(s: u8) -> Key {
    match s % 12 {
        0 => Key::C,
        1 => Key::Db,
        2 => Key::D,
        3 => Key::Eb,
        4 => Key::E,
        5 => Key::F,
        6 => Key::Gb,
        7 => Key::G,
        8 => Key::Ab,
        9 => Key::A,
        10 => Key::Bb,
        11 => Key::B,
        _ => unreachable!(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_c_major() {
        let mut d = KeyDetector::new(ScaleMode::Ionian);
        // Play a C major scale: C D E F G A B
        let notes = [60, 62, 64, 65, 67, 69, 71]; // C4-B4
        for &n in &notes {
            d.feed(n);
        }
        assert_eq!(d.detected, Some(Key::C));
    }

    #[test]
    fn detects_g_major() {
        let mut d = KeyDetector::new(ScaleMode::Ionian);
        // G major: G A B C D E F#
        let notes = [67, 69, 71, 60, 62, 64, 66];
        for &n in &notes {
            d.feed(n);
        }
        assert_eq!(d.detected, Some(Key::G));
    }

    #[test]
    fn detects_a_minor() {
        let mut d = KeyDetector::new(ScaleMode::Aeolian);
        // A minor: A B C D E F G
        let notes = [57, 59, 60, 62, 64, 65, 67];
        for &n in &notes {
            d.feed(n);
        }
        assert_eq!(d.detected, Some(Key::A));
    }

    #[test]
    fn needs_minimum_notes() {
        let mut d = KeyDetector::new(ScaleMode::Ionian);
        d.feed(60); // C
        d.feed(64); // E
        d.feed(67); // G
        assert_eq!(d.detected, None); // Only 3 notes, min is 4
    }

    #[test]
    fn reset_clears_state() {
        let mut d = KeyDetector::new(ScaleMode::Ionian);
        let notes = [60, 62, 64, 65, 67, 69, 71];
        for &n in &notes {
            d.feed(n);
        }
        assert!(d.detected.is_some());
        d.reset();
        assert_eq!(d.detected, None);
        assert_eq!(d.note_count, 0);
    }
}
