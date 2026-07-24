//! Event-rate exact-frequency tuning after structural harmony generation.
//!
//! The harmony engine still chooses ordinary MIDI notes. This module assigns
//! bounded per-voice frequency offsets for Contrapunk-owned renderers; it does
//! not change external MIDI note numbers or run on an audio callback.

use serde::{Deserialize, Serialize};

pub const MAX_TUNING_VOICES: usize = 8;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TuningStyle {
    #[default]
    Standard,
    Pure,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HarmonicLimit {
    #[default]
    Five,
    Seven,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct TuningConfig {
    pub style: TuningStyle,
    /// Blend between 12-TET and the selected target. Must be within 0..=1.
    pub depth: f32,
    pub harmonic_limit: HarmonicLimit,
}

impl Default for TuningConfig {
    fn default() -> Self {
        Self {
            style: TuningStyle::Standard,
            depth: 0.6,
            harmonic_limit: HarmonicLimit::Five,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TuningError {
    TooManyVoices { len: usize, max: usize },
    MelodyIndexOutOfRange { melody_index: usize, len: usize },
    NonFiniteDepth,
    DepthOutOfRange,
    SingularSystem,
    InvalidFrequency,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Ratio {
    pub numerator: u16,
    pub denominator: u16,
}

impl Ratio {
    pub const fn new(numerator: u16, denominator: u16) -> Self {
        Self {
            numerator,
            denominator,
        }
    }

    pub fn as_f64(self) -> f64 {
        f64::from(self.numerator) / f64::from(self.denominator)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TunedPitch {
    pub midi_note: u8,
    pub cents_offset: f64,
    pub frequency_hz: f64,
    /// Preferred pitch-class ratio relative to the melody. The frequency is
    /// authoritative because dense sonorities may require a compromise.
    pub ratio: Option<Ratio>,
}

impl TunedPitch {
    const EMPTY: Self = Self {
        midi_note: 0,
        cents_offset: 0.0,
        frequency_hz: 0.0,
        ratio: None,
    };
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TuningFrame {
    pitches: [TunedPitch; MAX_TUNING_VOICES],
    len: usize,
}

impl TuningFrame {
    pub const fn empty() -> Self {
        Self {
            pitches: [TunedPitch::EMPTY; MAX_TUNING_VOICES],
            len: 0,
        }
    }

    pub fn as_slice(&self) -> &[TunedPitch] {
        &self.pitches[..self.len]
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
}

/// Tune one bounded sonority. `melody_index` is held at its 12-TET frequency;
/// generated voices settle around it. The resulting frequencies remain fixed
/// until the caller releases those exact voices.
pub fn tune_notes(
    notes: &[u8],
    melody_index: usize,
    config: TuningConfig,
) -> Result<TuningFrame, TuningError> {
    let len = notes.len();
    if len > MAX_TUNING_VOICES {
        return Err(TuningError::TooManyVoices {
            len,
            max: MAX_TUNING_VOICES,
        });
    }
    if len == 0 {
        return Ok(TuningFrame::empty());
    }
    if melody_index >= len {
        return Err(TuningError::MelodyIndexOutOfRange { melody_index, len });
    }
    if !config.depth.is_finite() {
        return Err(TuningError::NonFiniteDepth);
    }
    if !(0.0..=1.0).contains(&config.depth) {
        return Err(TuningError::DepthOutOfRange);
    }

    let mut frame = TuningFrame::empty();
    frame.len = len;
    for (pitch, &note) in frame.pitches.iter_mut().zip(notes).take(len) {
        *pitch = TunedPitch {
            midi_note: note,
            cents_offset: 0.0,
            frequency_hz: midi_to_frequency(note),
            ratio: None,
        };
    }

    let depth = f64::from(config.depth);
    if config.style == TuningStyle::Standard || depth == 0.0 {
        return Ok(frame);
    }

    let mut matrix = [[0.0; MAX_TUNING_VOICES]; MAX_TUNING_VOICES];
    let mut rhs = [0.0; MAX_TUNING_VOICES];

    for first in 0..len {
        for second in (first + 1)..len {
            // Orient every equation from the lower MIDI pitch to the higher
            // pitch so reversing the input slice cannot change its target.
            let (low, high) = if notes[first] <= notes[second] {
                (first, second)
            } else {
                (second, first)
            };
            let interval = i16::from(notes[high]) - i16::from(notes[low]);
            let class = interval.rem_euclid(12) as u8;
            let target = interval_offset_cents(interval, config.harmonic_limit);
            let weight = interval_weight(class);
            matrix[low][low] += weight;
            matrix[high][high] += weight;
            matrix[low][high] -= weight;
            matrix[high][low] -= weight;
            rhs[low] -= weight * target;
            rhs[high] += weight * target;
        }
    }

    // One strong reference removes the global transposition ambiguity and
    // preserves the player's melody exactly.
    matrix[melody_index][melody_index] += 1024.0;
    let mut offsets = solve(matrix, rhs, len).ok_or(TuningError::SingularSystem)?;
    offsets[melody_index] = 0.0;

    let melody_frequency = midi_to_frequency(notes[melody_index]);
    for index in 0..len {
        let offset = offsets[index].clamp(-50.0, 50.0) * depth;
        let interval = i16::from(notes[index]) - i16::from(notes[melody_index]);
        // A common f64 anchor keeps full-depth ratios exact. Standard and
        // zero-depth paths returned above still use the renderer's existing
        // f32-derived 12-TET helper bit-for-bit.
        let equal_tempered = melody_frequency * 2.0_f64.powf(f64::from(interval) / 12.0);
        let frequency = equal_tempered * 2.0_f64.powf(offset / 1200.0);
        if !offset.is_finite() || !frequency.is_finite() || frequency <= 0.0 {
            return Err(TuningError::InvalidFrequency);
        }
        frame.pitches[index].cents_offset = offset;
        frame.pitches[index].frequency_hz = frequency;
        if index != melody_index {
            let interval = i16::from(notes[index]) - i16::from(notes[melody_index]);
            frame.pitches[index].ratio = Some(ratio_for_class(
                interval.rem_euclid(12) as u8,
                config.harmonic_limit,
            ));
        }
    }

    Ok(frame)
}

pub fn midi_to_frequency(note: u8) -> f64 {
    f64::from(contrapunk_dsp::pitch::midi_to_freq(note))
}

fn interval_offset_cents(interval: i16, limit: HarmonicLimit) -> f64 {
    let octaves = interval.div_euclid(12);
    let class = interval.rem_euclid(12) as u8;
    let pure_cents =
        1200.0 * (ratio_for_class(class, limit).as_f64() * 2.0_f64.powi(i32::from(octaves))).log2();
    pure_cents - f64::from(interval) * 100.0
}

fn ratio_for_class(class: u8, limit: HarmonicLimit) -> Ratio {
    match class % 12 {
        0 => Ratio::new(1, 1),
        1 => Ratio::new(16, 15),
        2 => Ratio::new(9, 8),
        3 => Ratio::new(6, 5),
        4 => Ratio::new(5, 4),
        5 => Ratio::new(4, 3),
        6 => match limit {
            HarmonicLimit::Five => Ratio::new(45, 32),
            HarmonicLimit::Seven => Ratio::new(7, 5),
        },
        7 => Ratio::new(3, 2),
        8 => Ratio::new(8, 5),
        9 => Ratio::new(5, 3),
        10 => match limit {
            HarmonicLimit::Five => Ratio::new(9, 5),
            HarmonicLimit::Seven => Ratio::new(7, 4),
        },
        11 => Ratio::new(15, 8),
        _ => unreachable!(),
    }
}

fn interval_weight(class: u8) -> f64 {
    match class % 12 {
        0 | 5 | 7 => 8.0,
        3 | 4 | 8 | 9 => 5.0,
        6 | 10 | 11 => 3.0,
        1 | 2 => 2.0,
        _ => 1.0,
    }
}

fn solve(
    mut matrix: [[f64; MAX_TUNING_VOICES]; MAX_TUNING_VOICES],
    mut rhs: [f64; MAX_TUNING_VOICES],
    len: usize,
) -> Option<[f64; MAX_TUNING_VOICES]> {
    for column in 0..len {
        let pivot = (column..len)
            .max_by(|&a, &b| matrix[a][column].abs().total_cmp(&matrix[b][column].abs()))?;
        if matrix[pivot][column].abs() < 1.0e-12 {
            return None;
        }
        if pivot != column {
            matrix.swap(pivot, column);
            rhs.swap(pivot, column);
        }

        let pivot_row = matrix[column];
        for row in (column + 1)..len {
            let factor = matrix[row][column] / pivot_row[column];
            matrix[row][column] = 0.0;
            for (value, pivot_value) in matrix[row][column + 1..len]
                .iter_mut()
                .zip(&pivot_row[column + 1..len])
            {
                *value -= factor * pivot_value;
            }
            rhs[row] -= factor * rhs[column];
        }
    }

    let mut result = [0.0; MAX_TUNING_VOICES];
    for row in (0..len).rev() {
        let remainder = matrix[row][row + 1..len]
            .iter()
            .zip(&result[row + 1..len])
            .map(|(coefficient, value)| coefficient * value)
            .sum::<f64>();
        result[row] = (rhs[row] - remainder) / matrix[row][row];
        if !result[row].is_finite() {
            return None;
        }
    }
    Some(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pure(depth: f32, harmonic_limit: HarmonicLimit) -> TuningConfig {
        TuningConfig {
            style: TuningStyle::Pure,
            depth,
            harmonic_limit,
        }
    }

    fn close(actual: f64, expected: f64, tolerance: f64) {
        assert!(
            (actual - expected).abs() <= tolerance,
            "{actual} != {expected} (tolerance {tolerance})"
        );
    }

    #[test]
    fn standard_is_exactly_the_existing_frequency_formula() {
        let frame = tune_notes(&[0, 60, 69, 127], 0, TuningConfig::default()).unwrap();
        for pitch in frame.as_slice() {
            assert_eq!(pitch.cents_offset, 0.0);
            assert_eq!(pitch.frequency_hz, midi_to_frequency(pitch.midi_note));
            assert_eq!(pitch.ratio, None);
        }
    }

    #[test]
    fn full_depth_c_major_is_exact_five_limit() {
        let frame = tune_notes(&[60, 64, 67], 0, pure(1.0, HarmonicLimit::Five)).unwrap();
        let pitches = frame.as_slice();
        close(
            pitches[1].frequency_hz / pitches[0].frequency_hz,
            5.0 / 4.0,
            1.0e-10,
        );
        close(
            pitches[2].frequency_hz / pitches[0].frequency_hz,
            3.0 / 2.0,
            1.0e-10,
        );
        close(
            pitches[2].frequency_hz / pitches[1].frequency_hz,
            6.0 / 5.0,
            1.0e-10,
        );
        close(pitches[1].cents_offset, -13.686_286, 1.0e-5);
        close(pitches[2].cents_offset, 1.955_001, 1.0e-5);
    }

    #[test]
    fn melody_anchor_remains_standard() {
        let frame = tune_notes(&[60, 64, 67], 1, pure(1.0, HarmonicLimit::Five)).unwrap();
        let melody = frame.as_slice()[1];
        assert_eq!(melody.cents_offset, 0.0);
        assert_eq!(melody.frequency_hz, midi_to_frequency(64));
    }

    #[test]
    fn depth_blends_in_cents_and_rejects_out_of_range_values() {
        let full = tune_notes(&[60, 64, 67], 0, pure(1.0, HarmonicLimit::Five)).unwrap();
        let partial = tune_notes(&[60, 64, 67], 0, pure(0.6, HarmonicLimit::Five)).unwrap();
        close(
            partial.as_slice()[1].cents_offset,
            full.as_slice()[1].cents_offset * 0.6,
            1.0e-6,
        );
        assert_eq!(
            tune_notes(&[60, 64], 0, pure(-1.0, HarmonicLimit::Five)),
            Err(TuningError::DepthOutOfRange)
        );
        assert_eq!(
            tune_notes(&[60, 64], 0, pure(2.0, HarmonicLimit::Five)),
            Err(TuningError::DepthOutOfRange)
        );
    }

    #[test]
    fn seven_limit_uses_a_harmonic_seventh() {
        let five = tune_notes(&[60, 70], 0, pure(1.0, HarmonicLimit::Five)).unwrap();
        let seven = tune_notes(&[60, 70], 0, pure(1.0, HarmonicLimit::Seven)).unwrap();
        close(
            five.as_slice()[1].frequency_hz / five.as_slice()[0].frequency_hz,
            9.0 / 5.0,
            1.0e-10,
        );
        close(
            seven.as_slice()[1].frequency_hz / seven.as_slice()[0].frequency_hz,
            7.0 / 4.0,
            1.0e-10,
        );
    }

    #[test]
    fn reversing_and_permuting_voices_preserves_tuning_by_note() {
        let dyad = tune_notes(&[60, 62], 0, pure(1.0, HarmonicLimit::Five)).unwrap();
        let reversed_dyad = tune_notes(&[62, 60], 1, pure(1.0, HarmonicLimit::Five)).unwrap();
        for note in [60, 62] {
            let original = dyad
                .as_slice()
                .iter()
                .find(|pitch| pitch.midi_note == note)
                .unwrap();
            let reversed = reversed_dyad
                .as_slice()
                .iter()
                .find(|pitch| pitch.midi_note == note)
                .unwrap();
            close(original.cents_offset, reversed.cents_offset, 1.0e-10);
            close(original.frequency_hz, reversed.frequency_hz, 1.0e-10);
        }

        let triad = tune_notes(&[60, 64, 67], 0, pure(1.0, HarmonicLimit::Five)).unwrap();
        let permuted = tune_notes(&[67, 60, 64], 1, pure(1.0, HarmonicLimit::Five)).unwrap();
        for note in [60, 64, 67] {
            let original = triad
                .as_slice()
                .iter()
                .find(|pitch| pitch.midi_note == note)
                .unwrap();
            let reordered = permuted
                .as_slice()
                .iter()
                .find(|pitch| pitch.midi_note == note)
                .unwrap();
            close(original.cents_offset, reordered.cents_offset, 1.0e-10);
            close(original.frequency_hz, reordered.frequency_hz, 1.0e-10);
        }
    }

    #[test]
    fn dense_contradictory_sonorities_are_bounded_and_finite() {
        let frame = tune_notes(
            &[48, 49, 52, 55, 58, 61, 64, 67],
            3,
            pure(1.0, HarmonicLimit::Seven),
        )
        .unwrap();
        assert_eq!(frame.len(), MAX_TUNING_VOICES);
        assert!(frame.as_slice().iter().all(|pitch| {
            pitch.frequency_hz.is_finite()
                && pitch.frequency_hz > 0.0
                && pitch.cents_offset.is_finite()
                && pitch.cents_offset.abs() <= 50.0
        }));
    }

    #[test]
    fn invalid_depth_is_rejected() {
        assert_eq!(
            tune_notes(&[60, 64], 0, pure(f32::NAN, HarmonicLimit::Five)),
            Err(TuningError::NonFiniteDepth)
        );
    }

    #[test]
    fn invalid_voice_bounds_are_rejected() {
        assert_eq!(
            tune_notes(
                &[48, 49, 50, 51, 52, 53, 54, 55, 56],
                0,
                pure(1.0, HarmonicLimit::Five),
            ),
            Err(TuningError::TooManyVoices {
                len: 9,
                max: MAX_TUNING_VOICES,
            })
        );
        assert_eq!(
            tune_notes(&[60, 64], 2, pure(1.0, HarmonicLimit::Five)),
            Err(TuningError::MelodyIndexOutOfRange {
                melody_index: 2,
                len: 2,
            })
        );
    }
}
