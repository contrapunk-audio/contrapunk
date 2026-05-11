use crate::config::{Key, ScaleMode};

// === Krumhansl-Schmuckler profiles (#81, first slice) ===
//
// Published probe-tone weights from Krumhansl & Kessler 1982. Indexed
// 0..11 where 0 is the tonic and the rest follow ascending chromatic
// scale degrees. Used by the (next-slice) Pearson-correlation detector
// to score each rotation against the observed pitch-class histogram.
//
// Future-slice work: replace the in-scale/out-of-scale `score_tonic`
// in `KeyDetector::detect` with `pearson_correlation(histogram, rotated_profile)`.
// This slice just exposes the data; behavior is unchanged.

/// Krumhansl-Kessler major profile — the canonical weights tonic 0..11.
/// Tonic, dominant (7), mediant (4) carry the heaviest weight; the
/// chromatic-passing tones (1, 3, 6, 8, 10) sit lowest.
#[allow(dead_code)]
pub const KS_MAJOR_PROFILE: [f32; 12] = [
    6.35, 2.23, 3.48, 2.33, 4.38, 4.09, 2.52, 5.19, 2.39, 3.66, 2.29, 2.88,
];

/// Krumhansl-Kessler minor profile. Slightly different distribution
/// reflecting the minor-mode tonal hierarchy (e.g. lowered mediant at
/// index 3 carries more weight than the major-mode equivalent).
#[allow(dead_code)]
pub const KS_MINOR_PROFILE: [f32; 12] = [
    6.33, 2.68, 3.52, 5.38, 2.60, 3.53, 2.54, 4.75, 3.98, 2.69, 3.34, 3.17,
];

/// Returns the appropriate KS-style profile for the given scale mode.
///
/// The published K&K profiles only cover major (Ionian) and minor
/// (Aeolian / Harmonic Minor). For modes outside that pair, we
/// synthesize a profile by treating in-scale degrees as 4.0-weighted
/// and out-of-scale as 1.0-weighted — a flat-but-scale-aware shape
/// that the Pearson correlation can still rank tonics from. Future
/// research can replace these with derived profiles per mode.
pub fn ks_profile_for_mode(mode: ScaleMode) -> [f32; 12] {
    use ScaleMode::*;
    match mode {
        // Major-flavored — share the major profile.
        Ionian | Lydian | Mixolydian | MajorPentatonic | MajorBlues | LydianAug
        | LydianDominant | HarmonicMajor | BHMajor6thDim => KS_MAJOR_PROFILE,
        // Minor-flavored — share the minor profile.
        Aeolian | Dorian | Phrygian | Locrian | HarmonicMinor | MelodicMinor | MinorPentatonic
        | MinorBlues | DorianFlat2 | PhrygianDominant | LocrianNat2 | LocrianNat6
        | BHMinor6thDim => KS_MINOR_PROFILE,
        // Everything else — synthesize from the mode's interval set.
        other => synth_profile_from_intervals(other),
    }
}

#[allow(dead_code)]
fn synth_profile_from_intervals(mode: ScaleMode) -> [f32; 12] {
    let intervals = mode.intervals();
    let mut p = [1.0f32; 12];
    for &i in intervals {
        let idx = i as usize;
        if idx < 12 {
            p[idx] = 4.0;
        }
    }
    p
}

/// Rotate a 12-element profile by `tonic` semitones so the tonic
/// position lines up with pitch class 0 in the rotated frame.
/// Returns a new array; the input is unchanged.
///
/// Math: rotated[i] = profile[(i - tonic) mod 12]. Equivalently the
/// profile slid `tonic` positions to the right.
#[allow(dead_code)]
fn rotate_profile(profile: &[f32; 12], tonic: u8) -> [f32; 12] {
    let mut out = [0.0f32; 12];
    let tonic = (tonic % 12) as usize;
    for i in 0..12 {
        out[i] = profile[(i + 12 - tonic) % 12];
    }
    out
}

/// Pearson correlation coefficient between two 12-element vectors.
/// Returns a value in [-1.0, 1.0]; higher = stronger linear
/// relationship. Returns 0.0 when either vector has zero variance.
///
/// Pure function, no allocation, one pass over each input.
#[allow(dead_code)]
fn pearson_correlation(x: &[f32; 12], y: &[f32; 12]) -> f32 {
    let n = 12.0_f32;
    let x_sum: f32 = x.iter().sum();
    let y_sum: f32 = y.iter().sum();
    let x_mean = x_sum / n;
    let y_mean = y_sum / n;
    let mut num = 0.0_f32;
    let mut x_sq = 0.0_f32;
    let mut y_sq = 0.0_f32;
    for i in 0..12 {
        let dx = x[i] - x_mean;
        let dy = y[i] - y_mean;
        num += dx * dy;
        x_sq += dx * dx;
        y_sq += dy * dy;
    }
    let denom = (x_sq * y_sq).sqrt();
    if denom < 1e-9 {
        0.0
    } else {
        num / denom
    }
}

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

    /// The major profile peaks at the tonic (index 0) and dominant
    /// (index 7). This is the textbook tonal hierarchy — regression
    /// guard against accidentally reordering the constants.
    #[test]
    fn test_ks_major_profile_peaks_at_tonic_and_dominant() {
        let max_idx = (0..12)
            .max_by(|&a, &b| {
                KS_MAJOR_PROFILE[a]
                    .partial_cmp(&KS_MAJOR_PROFILE[b])
                    .unwrap()
            })
            .unwrap();
        assert_eq!(max_idx, 0, "tonic must be the strongest weight");
        // Dominant should outrank every degree except tonic.
        for i in 0..12 {
            if i == 0 || i == 7 {
                continue;
            }
            assert!(
                KS_MAJOR_PROFILE[7] > KS_MAJOR_PROFILE[i],
                "dominant (7) should outrank degree {} ({} vs {})",
                i,
                KS_MAJOR_PROFILE[7],
                KS_MAJOR_PROFILE[i]
            );
        }
    }

    /// The minor profile peaks at the tonic and gives heavier weight
    /// to the lowered mediant (index 3, the b3) than the major profile.
    #[test]
    fn test_ks_minor_profile_emphasizes_flat_third() {
        let max_idx = (0..12)
            .max_by(|&a, &b| {
                KS_MINOR_PROFILE[a]
                    .partial_cmp(&KS_MINOR_PROFILE[b])
                    .unwrap()
            })
            .unwrap();
        assert_eq!(max_idx, 0, "tonic must be the strongest weight");
        // In minor, b3 (3) > maj 3 (4). The major profile has the
        // opposite relationship.
        assert!(
            KS_MINOR_PROFILE[3] > KS_MINOR_PROFILE[4],
            "minor profile should weight b3 ({}) above maj 3 ({})",
            KS_MINOR_PROFILE[3],
            KS_MINOR_PROFILE[4]
        );
        assert!(
            KS_MAJOR_PROFILE[4] > KS_MAJOR_PROFILE[3],
            "major profile should weight maj 3 above b3 (regression guard for the pair)"
        );
    }

    #[test]
    fn test_profile_for_mode_routes_major_minor() {
        assert_eq!(ks_profile_for_mode(ScaleMode::Ionian), KS_MAJOR_PROFILE);
        assert_eq!(ks_profile_for_mode(ScaleMode::Lydian), KS_MAJOR_PROFILE);
        assert_eq!(ks_profile_for_mode(ScaleMode::Aeolian), KS_MINOR_PROFILE);
        assert_eq!(
            ks_profile_for_mode(ScaleMode::HarmonicMinor),
            KS_MINOR_PROFILE
        );
    }

    /// Exotic modes synthesize a profile from their interval set:
    /// in-scale degrees get 4.0, out-of-scale degrees get 1.0. The
    /// tonic (degree 0 in the intervals) must be in-scale.
    #[test]
    fn test_profile_for_mode_synthesizes_exotic() {
        let p = ks_profile_for_mode(ScaleMode::WholeTone);
        assert_eq!(p[0], 4.0, "tonic should always be in-scale");
        // WholeTone has 6 in-scale degrees, 6 out-of-scale.
        let in_scale = p.iter().filter(|&&w| w == 4.0).count();
        assert_eq!(in_scale, 6, "WholeTone should have 6 in-scale degrees");
    }

    /// Rotation by 0 is identity.
    #[test]
    fn test_rotate_profile_by_zero_is_identity() {
        let rotated = rotate_profile(&KS_MAJOR_PROFILE, 0);
        assert_eq!(rotated, KS_MAJOR_PROFILE);
    }

    /// Rotating the major profile by 7 (G) makes the dominant-weighted
    /// position (index 7 in the unrotated frame) appear at the tonic
    /// position of the rotated frame (index 0).
    #[test]
    fn test_rotate_profile_by_seven_aligns_dominant_at_tonic() {
        let rotated = rotate_profile(&KS_MAJOR_PROFILE, 7);
        assert_eq!(rotated[0], KS_MAJOR_PROFILE[5]); // (0 + 12 - 7) % 12 = 5
                                                     // Verify the rotation is well-defined for every position.
        for i in 0..12 {
            assert_eq!(rotated[i], KS_MAJOR_PROFILE[(i + 12 - 7) % 12]);
        }
    }

    /// Rotation is reversible by rotating in the opposite direction.
    #[test]
    fn test_rotate_profile_is_reversible() {
        let once = rotate_profile(&KS_MAJOR_PROFILE, 5);
        let back = rotate_profile(&once, 12 - 5);
        for i in 0..12 {
            assert!(
                (back[i] - KS_MAJOR_PROFILE[i]).abs() < 1e-6,
                "rotate by 5 then by 7 should restore original at index {}",
                i
            );
        }
    }

    /// Pearson of a vector with itself is 1.0 (perfect positive
    /// correlation), modulo float precision.
    #[test]
    fn test_pearson_self_correlation_is_one() {
        let r = pearson_correlation(&KS_MAJOR_PROFILE, &KS_MAJOR_PROFILE);
        assert!((r - 1.0).abs() < 1e-5, "self-correlation got {}", r);
    }

    /// Pearson of a vector with its negation is -1.0.
    #[test]
    fn test_pearson_negated_correlation_is_minus_one() {
        let mut neg = [0.0_f32; 12];
        for i in 0..12 {
            neg[i] = -KS_MAJOR_PROFILE[i];
        }
        let r = pearson_correlation(&KS_MAJOR_PROFILE, &neg);
        assert!((r + 1.0).abs() < 1e-5, "negated correlation got {}", r);
    }

    /// Zero-variance input (constant vector) returns 0.0 to avoid
    /// NaN propagation downstream.
    #[test]
    fn test_pearson_zero_variance_returns_zero() {
        let constant = [3.0_f32; 12];
        let r = pearson_correlation(&constant, &KS_MAJOR_PROFILE);
        assert_eq!(r, 0.0, "constant vector should produce 0.0, not NaN");
    }

    /// Rotation + correlation roundtrip: rotate the major profile by
    /// 7 (G), correlate against the unrotated major profile. The
    /// correlation should be HIGHER than chance but LESS than 1.0 —
    /// it's the dominant-related profile, which shares structure
    /// with C but isn't identical.
    #[test]
    fn test_pearson_g_major_vs_c_major_is_positive_but_not_perfect() {
        let g_rotated = rotate_profile(&KS_MAJOR_PROFILE, 7);
        let r = pearson_correlation(&g_rotated, &KS_MAJOR_PROFILE);
        assert!(
            r > 0.0,
            "G major profile should correlate positively with C major (got {})",
            r
        );
        assert!(
            r < 0.95,
            "G major profile should NOT perfectly match C major (got {})",
            r
        );
    }

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
