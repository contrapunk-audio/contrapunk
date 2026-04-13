//! Next-Note Suggestion Overlay — Scoring Engine
//!
//! Computes ranked suggestions for the next note to play based on an
//! 11-term normalized scoring function. This is a **visual overlay only** --
//! suggestions are never played audibly. The overlay reads harmony engine
//! state but does not write to it.
//!
//! Each scoring term outputs a value in [0, 1]. The final score is a
//! weighted average: `Score(n) = sum(w_i * f_i(n)) / sum(w_i)` over
//! active terms. Terms without applicable context are excluded from
//! both numerator and denominator.
//!
//! Default weights are calibrated from Bach chorale soprano statistics:
//! 79% stepwise motion, 89% strong-beat chord tones.

use serde::{Deserialize, Serialize};

// ── Snapshot ────────────────────────────────────────────────────────────────

/// Lightweight, Copy-able snapshot of harmony engine state for suggestion scoring.
///
/// Written by the engine after each note event; read by the UI thread
/// to compute suggestions without locking.
#[derive(Clone, Copy, Debug, Default)]
pub struct SuggestionSnapshot {
    /// Ring buffer of recent MIDI note numbers (most recent last).
    pub recent_notes: [u8; 8],
    /// How many valid entries in `recent_notes` (0..=8).
    pub recent_count: u8,
    /// Root pitch class (0-11) of the current chord, or 255 if unknown.
    pub current_chord_root: u8,
    /// Chord quality: 0=maj, 1=min, 2=dim, 3=aug, 4=dom7, 255=unknown.
    pub current_chord_quality: u8,
    /// Pitch classes in the current chord as a bitmask (bit i = pc i present).
    pub chord_pc_mask: u16,
    /// Root pitch class of the anticipated next chord, or 255 if unknown.
    pub next_chord_root: u8,
    /// Confidence in the next chord prediction [0.0, 1.0].
    pub next_chord_confidence: f32,
    /// Pitch classes in the next chord as a bitmask.
    pub next_chord_pc_mask: u16,
    /// Scale pitch classes as a bitmask (bit i = pc i is in scale).
    pub scale_mask: u16,
    /// Pitch class of the tonic (0-11).
    pub key_root: u8,
    /// Milliseconds since the last Note-On event.
    pub last_note_on_time_ms: u32,
}

// Compile-time trait assertions for SuggestionSnapshot.
const _: fn() = || {
    fn assert<T: Copy + Send>() {}
    assert::<SuggestionSnapshot>();
};

impl SuggestionSnapshot {
    /// Get the most recently played MIDI note, or None if empty.
    #[inline]
    pub fn last_note(&self) -> Option<u8> {
        if self.recent_count == 0 {
            None
        } else {
            Some(self.recent_notes[(self.recent_count - 1) as usize])
        }
    }

    /// Get the note before last, or None if fewer than 2 notes.
    #[inline]
    pub fn prev_note(&self) -> Option<u8> {
        if self.recent_count < 2 {
            None
        } else {
            Some(self.recent_notes[(self.recent_count - 2) as usize])
        }
    }

    /// Check if a pitch class is in the current scale.
    #[inline]
    pub fn is_in_scale(&self, pc: u8) -> bool {
        self.scale_mask & (1 << (pc % 12)) != 0
    }

    /// Check if a pitch class is in the current chord.
    #[inline]
    pub fn is_chord_tone(&self, pc: u8) -> bool {
        self.chord_pc_mask & (1 << (pc % 12)) != 0
    }

    /// Compute mean MIDI value of recent notes.
    pub fn recent_mean(&self) -> f32 {
        if self.recent_count == 0 {
            return 60.0; // sensible default
        }
        let sum: u32 = self.recent_notes[..self.recent_count as usize]
            .iter()
            .map(|&n| n as u32)
            .sum();
        sum as f32 / self.recent_count as f32
    }

    /// Count occurrences of a MIDI note in the recent window.
    pub fn count_in_recent(&self, note: u8) -> u8 {
        self.recent_notes[..self.recent_count as usize]
            .iter()
            .filter(|&&n| n == note)
            .count() as u8
    }
}

// ── Configuration ───────────────────────────────────────────────────────────

/// Weights and parameters for the 11-term suggestion scorer.
///
/// Default values are calibrated from Bach chorale soprano statistics.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SuggestionConfig {
    // Harmonic terms
    pub w_chord_tone: f32,
    pub w_scale_tone: f32,
    pub w_dissonance: f32,
    // Melodic terms
    pub w_proximity: f32,
    pub w_contour: f32,
    pub w_leap_recovery: f32,
    pub w_repetition: f32,
    // Contextual terms
    pub w_next_chord_prep: f32,
    pub w_leading_tone: f32,
    pub w_narmour: f32,
    pub w_tessitura: f32,
    // Parameters
    pub proximity_tau: f32,
    pub tessitura_sigma: f32,
    pub variety_window: u8,
    /// MIDI range low (inclusive).
    pub range_low: u8,
    /// MIDI range high (inclusive).
    pub range_high: u8,
}

impl Default for SuggestionConfig {
    fn default() -> Self {
        Self {
            w_chord_tone: 10.0,
            w_scale_tone: 4.0,
            w_dissonance: 6.0,
            w_proximity: 8.0,
            w_contour: 3.0,
            w_leap_recovery: 6.0,
            w_repetition: 5.0,
            w_next_chord_prep: 4.0,
            w_leading_tone: 7.0,
            w_narmour: 3.0,
            w_tessitura: 2.0,
            proximity_tau: 3.0,
            tessitura_sigma: 5.0,
            variety_window: 8,
            range_low: 40,  // E2 (guitar low)
            range_high: 88, // E6 (guitar high)
        }
    }
}

impl SuggestionConfig {
    /// Set a weight by its term name. Returns true if the name matched.
    pub fn set_weight(&mut self, term: &str, value: f32) -> bool {
        let clamped = value.clamp(0.0, 10.0);
        match term {
            "chord_tone" => self.w_chord_tone = clamped,
            "scale_tone" => self.w_scale_tone = clamped,
            "dissonance" => self.w_dissonance = clamped,
            "proximity" => self.w_proximity = clamped,
            "contour" => self.w_contour = clamped,
            "leap_recovery" => self.w_leap_recovery = clamped,
            "repetition" => self.w_repetition = clamped,
            "next_chord_prep" => self.w_next_chord_prep = clamped,
            "leading_tone" => self.w_leading_tone = clamped,
            "narmour" => self.w_narmour = clamped,
            "tessitura" => self.w_tessitura = clamped,
            _ => return false,
        }
        true
    }

    /// Get weight by term name.
    pub fn get_weight(&self, term: &str) -> Option<f32> {
        match term {
            "chord_tone" => Some(self.w_chord_tone),
            "scale_tone" => Some(self.w_scale_tone),
            "dissonance" => Some(self.w_dissonance),
            "proximity" => Some(self.w_proximity),
            "contour" => Some(self.w_contour),
            "leap_recovery" => Some(self.w_leap_recovery),
            "repetition" => Some(self.w_repetition),
            "next_chord_prep" => Some(self.w_next_chord_prep),
            "leading_tone" => Some(self.w_leading_tone),
            "narmour" => Some(self.w_narmour),
            "tessitura" => Some(self.w_tessitura),
            _ => None,
        }
    }

    /// Return all term names in order.
    pub fn term_names() -> &'static [&'static str] {
        &[
            "chord_tone",
            "scale_tone",
            "dissonance",
            "proximity",
            "contour",
            "leap_recovery",
            "repetition",
            "next_chord_prep",
            "leading_tone",
            "narmour",
            "tessitura",
        ]
    }
}

// ── Individual scoring terms (all return [0,1]) ─────────────────────────────

/// f_chord: Chord-tone affinity (Lerdahl pitch space mapping).
#[inline]
pub fn f_chord_tone(candidate: u8, snap: &SuggestionSnapshot) -> Option<f32> {
    if snap.current_chord_root == 255 {
        return None; // no chord context
    }
    let pc = candidate % 12;
    if snap.is_chord_tone(pc) {
        // Root gets highest, then other chord tones
        if pc == snap.current_chord_root % 12 {
            Some(1.0)
        } else {
            Some(0.8)
        }
    } else if snap.is_in_scale(pc) {
        Some(0.2)
    } else {
        Some(0.0)
    }
}

/// f_scale: Scale-tone affinity.
#[inline]
pub fn f_scale_tone(candidate: u8, snap: &SuggestionSnapshot) -> f32 {
    let pc = candidate % 12;
    if snap.is_in_scale(pc) {
        // Tonic gets highest
        if pc == snap.key_root {
            1.0
        } else {
            0.7
        }
    } else {
        // Chromatic neighbor of scale tone
        let above = (pc + 1) % 12;
        let below = (pc + 11) % 12;
        if snap.is_in_scale(above) || snap.is_in_scale(below) {
            0.3
        } else {
            0.0
        }
    }
}

/// f_proximity: Exponential pitch proximity.
#[inline]
pub fn f_proximity(candidate: u8, snap: &SuggestionSnapshot, tau: f32) -> Option<f32> {
    let last = snap.last_note()?;
    let dist = (candidate as f32 - last as f32).abs();
    Some((-dist / tau).exp())
}

/// f_contour: Contour continuation/reversal.
#[inline]
pub fn f_contour(candidate: u8, snap: &SuggestionSnapshot) -> Option<f32> {
    let last = snap.last_note()?;
    let prev = snap.prev_note()?;

    let dir_prev = (last as i16 - prev as i16).signum();
    let dir_cand = (candidate as i16 - last as i16).signum();
    let cand_interval = (candidate as i16 - last as i16).unsigned_abs();

    if dir_prev == 0 {
        // Static: any step is acceptable
        return if cand_interval <= 2 {
            Some(0.75)
        } else {
            Some(0.4)
        };
    }

    if dir_cand == dir_prev {
        // Same direction
        if cand_interval <= 2 {
            Some(1.0) // step continues contour
        } else if cand_interval <= 7 {
            Some(0.5) // leap continues contour
        } else {
            Some(0.0) // large leap in same direction
        }
    } else if dir_cand == 0 {
        Some(0.4) // unison
    } else {
        // Reversal
        if cand_interval <= 2 {
            Some(0.75) // step reverses
        } else {
            Some(0.25) // leap reverses
        }
    }
}

/// f_leap_recovery: Post-leap correction (Narmour RD + Huron).
#[inline]
pub fn f_leap_recovery(candidate: u8, snap: &SuggestionSnapshot) -> Option<f32> {
    let last = snap.last_note()?;
    let prev = snap.prev_note()?;

    let i1 = last as i16 - prev as i16; // previous interval (signed)
    let i2 = candidate as i16 - last as i16; // candidate interval (signed)

    if i1.unsigned_abs() <= 2 {
        // Was a step, term is N/A -- return neutral
        return Some(0.5);
    }

    let reverses = (i1 > 0 && i2 < 0) || (i1 < 0 && i2 > 0);
    let abs_i2 = i2.unsigned_abs();

    if i1.unsigned_abs() >= 5 {
        if reverses {
            if abs_i2 <= 2 {
                Some(1.0) // ideal: reverse by step
            } else if abs_i2 <= 4 {
                Some(0.75)
            } else {
                Some(0.5)
            }
        } else {
            Some(0.1) // continued leap -- undesirable
        }
    } else {
        // i1 in [3, 4]
        Some(0.5) // neutral
    }
}

/// f_repetition: Sliding window variety penalty.
#[inline]
pub fn f_repetition(candidate: u8, snap: &SuggestionSnapshot, window: u8) -> f32 {
    let w = window.max(1) as f32;
    let count = snap.count_in_recent(candidate) as f32;
    (1.0 - count / w).max(0.0)
}

/// f_next_chord_prep: Next-chord preparation scoring.
#[inline]
pub fn f_next_chord_prep(candidate: u8, snap: &SuggestionSnapshot) -> Option<f32> {
    if snap.next_chord_root == 255 || snap.next_chord_confidence < 0.01 {
        return None; // no next chord info
    }
    let c = snap.next_chord_confidence;
    let pc = candidate % 12;

    // Is candidate a chord tone of the next chord?
    let is_next_ct = snap.next_chord_pc_mask & (1 << pc) != 0;
    // Is candidate also a chord tone of the current chord?
    let is_curr_ct = snap.chord_pc_mask & (1 << pc) != 0;

    if is_next_ct && is_curr_ct {
        Some(c * 1.0) // pivot tone
    } else if pc == (snap.next_chord_root + 11) % 12 {
        Some(c * 0.83) // leading tone to next root
    } else if is_next_ct {
        Some(c * 0.5)
    } else {
        Some(0.0)
    }
}

/// f_leading_tone: Tendency-tone resolution.
#[inline]
pub fn f_leading_tone(candidate: u8, snap: &SuggestionSnapshot) -> Option<f32> {
    let last = snap.last_note()?;
    let last_pc = last % 12;
    let cand_pc = candidate % 12;

    // Scale degree 7 -> 1
    let degree_7 = (snap.key_root + 11) % 12;
    if last_pc == degree_7 && cand_pc == snap.key_root {
        return Some(1.0);
    }

    // Scale degree 4 -> 3
    let degree_4 = (snap.key_root + 5) % 12;
    let degree_3 = (snap.key_root + 4) % 12;
    if last_pc == degree_4 && cand_pc == degree_3 {
        return Some(0.86);
    }

    // If last note IS a tendency tone but candidate is NOT the resolution
    if last_pc == degree_7 || last_pc == degree_4 {
        return Some(0.0);
    }

    // Last note is not a tendency tone -- term is N/A
    None
}

/// f_narmour: Schellenberg 2-factor implication-realization.
#[inline]
pub fn f_narmour(candidate: u8, snap: &SuggestionSnapshot) -> Option<f32> {
    let last = snap.last_note()?;
    let prev = snap.prev_note()?;

    let i1 = last as i16 - prev as i16;
    let i2 = candidate as i16 - last as i16;

    // Factor 1: Proximity (smaller intervals more expected)
    let f_prox = (1.0 - (i2.unsigned_abs() as f32) / 12.0).max(0.0);

    // Factor 2: Registral direction
    let abs_i1 = i1.unsigned_abs();
    let same_dir = (i1 > 0 && i2 > 0) || (i1 < 0 && i2 < 0);
    let reverses = (i1 > 0 && i2 < 0) || (i1 < 0 && i2 > 0);

    let f_dir = if abs_i1 >= 7 {
        if reverses {
            1.0
        } else if same_dir {
            0.25
        } else {
            0.5
        }
    } else if abs_i1 <= 5 {
        if same_dir {
            0.75
        } else {
            0.5 // reversal or unison after small interval
        }
    } else {
        0.5 // ambiguous zone
    };

    Some(0.625 * f_prox + 0.375 * f_dir)
}

/// f_dissonance: Dissonance avoidance.
#[inline]
pub fn f_dissonance(candidate: u8, snap: &SuggestionSnapshot) -> Option<f32> {
    if snap.current_chord_root == 255 {
        return None;
    }
    let pc = candidate % 12;
    let root = snap.current_chord_root % 12;
    let ic = ((pc as i16 - root as i16).rem_euclid(12)) as u8;

    Some(match ic {
        0 | 7 => 1.0,         // perfect consonance
        3 | 4 | 8 | 9 => 0.8, // imperfect consonance
        2 | 5 | 10 => 0.5,    // mild dissonance
        1 | 6 | 11 => 0.125,  // sharp dissonance
        _ => 0.0,
    })
}

/// f_tessitura: Tessitura centrality (Temperley Gaussian).
#[inline]
pub fn f_tessitura(candidate: u8, snap: &SuggestionSnapshot, sigma: f32) -> f32 {
    let mean = snap.recent_mean();
    let d = candidate as f32 - mean;
    (-d * d / (2.0 * sigma * sigma)).exp()
}

// ── Main scoring function ───────────────────────────────────────────────────

/// Score a single candidate note. Returns a value in [0.0, 1.0].
///
/// Terms without applicable context are excluded from both numerator
/// and denominator of the weighted average.
pub fn score_candidate(
    candidate: u8,
    snapshot: &SuggestionSnapshot,
    config: &SuggestionConfig,
) -> f32 {
    let mut weighted_sum = 0.0f32;
    let mut weight_sum = 0.0f32;

    // Helper: add a term if it's active (Some)
    macro_rules! add_term {
        ($weight:expr, $value:expr) => {
            if let Some(v) = $value {
                weighted_sum += $weight * v;
                weight_sum += $weight;
            }
        };
    }

    // Always-active terms (use Some wrapper for uniformity)
    add_term!(config.w_scale_tone, Some(f_scale_tone(candidate, snapshot)));
    add_term!(
        config.w_proximity,
        f_proximity(candidate, snapshot, config.proximity_tau)
    );
    add_term!(
        config.w_repetition,
        Some(f_repetition(candidate, snapshot, config.variety_window))
    );
    add_term!(
        config.w_tessitura,
        Some(f_tessitura(candidate, snapshot, config.tessitura_sigma))
    );

    // Context-dependent terms
    add_term!(config.w_chord_tone, f_chord_tone(candidate, snapshot));
    add_term!(config.w_contour, f_contour(candidate, snapshot));
    add_term!(config.w_leap_recovery, f_leap_recovery(candidate, snapshot));
    add_term!(
        config.w_next_chord_prep,
        f_next_chord_prep(candidate, snapshot)
    );
    add_term!(config.w_leading_tone, f_leading_tone(candidate, snapshot));
    add_term!(config.w_narmour, f_narmour(candidate, snapshot));
    add_term!(config.w_dissonance, f_dissonance(candidate, snapshot));

    if weight_sum < 0.001 {
        return 0.0;
    }

    weighted_sum / weight_sum
}

/// Score all candidates in the configured MIDI range.
/// Returns sorted Vec of (midi_note, score), descending by score.
pub fn rank_candidates(snapshot: &SuggestionSnapshot, config: &SuggestionConfig) -> Vec<(u8, f32)> {
    let mut candidates: Vec<(u8, f32)> = (config.range_low..=config.range_high)
        .map(|n| (n, score_candidate(n, snapshot, config)))
        .collect();

    // Sort descending by score (stable sort preserves MIDI order on ties)
    candidates.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    candidates
}

// ── Guitar Position Scoring ─────────────────────────────────────────────────

/// Standard tuning open string MIDI values [E2, A2, D3, G3, B3, E4].
pub const STANDARD_TUNING: [u8; 6] = [40, 45, 50, 55, 59, 64];

/// A fretboard position with ergonomic score.
#[derive(Clone, Copy, Debug)]
pub struct FretPosition {
    /// String index (0 = low E, 5 = high E).
    pub string: u8,
    /// Fret number (0-24).
    pub fret: u8,
    /// Ergonomic position score in [0, 1].
    pub score: f32,
}

/// Parameters for position scoring.
#[derive(Clone, Copy, Debug)]
pub struct PositionConfig {
    /// Fret distance cost coefficient.
    pub alpha: f32,
    /// String cross cost coefficient.
    pub beta: f32,
    /// Stretch penalty coefficient.
    pub gamma: f32,
    /// Upper fret bias coefficient.
    pub delta: f32,
    /// Exponential decay temperature.
    pub tau_pos: f32,
}

impl Default for PositionConfig {
    fn default() -> Self {
        Self {
            alpha: 1.0,
            beta: 0.5,
            gamma: 3.0,
            delta: 0.3,
            tau_pos: 4.0,
        }
    }
}

/// Enumerate all valid (string, fret) positions for a MIDI note.
pub fn midi_to_positions(note: u8) -> Vec<(u8, u8)> {
    let mut positions = Vec::new();
    for (s, &open) in STANDARD_TUNING.iter().enumerate() {
        let fret = note as i16 - open as i16;
        if (0..=24).contains(&fret) {
            positions.push((s as u8, fret as u8));
        }
    }
    positions
}

/// Score all valid positions for a MIDI note given current hand state.
///
/// Returns positions sorted by score descending.
pub fn score_positions(
    midi_note: u8,
    hand_position: f32,
    last_string: u8,
    config: &PositionConfig,
) -> Vec<FretPosition> {
    let positions = midi_to_positions(midi_note);
    let mut scored: Vec<FretPosition> = positions
        .into_iter()
        .map(|(s, f)| {
            let fret_dist = (f as f32 - hand_position).abs();
            let string_dist = (s as f32 - last_string as f32).abs();
            let stretch = (fret_dist - 4.0).max(0.0);
            let upper_bias = ((f as f32 - 12.0).max(0.0)) / 12.0;

            let cost = config.alpha * fret_dist
                + config.beta * string_dist
                + config.gamma * stretch
                + config.delta * upper_bias;

            let score = (-cost / config.tau_pos).exp();

            FretPosition {
                string: s,
                fret: f,
                score,
            }
        })
        .collect();

    scored.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    scored
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_snapshot_c_major() -> SuggestionSnapshot {
        // C major scale: C D E F G A B
        let scale_mask =
            (1 << 0) | (1 << 2) | (1 << 4) | (1 << 5) | (1 << 7) | (1 << 9) | (1 << 11);
        // C major chord: C E G
        let chord_mask = (1 << 0) | (1 << 4) | (1 << 7);

        SuggestionSnapshot {
            recent_notes: [60, 62, 64, 65, 67, 69, 71, 72],
            recent_count: 8,
            current_chord_root: 0,    // C
            current_chord_quality: 0, // major
            chord_pc_mask: chord_mask,
            next_chord_root: 255,
            next_chord_confidence: 0.0,
            next_chord_pc_mask: 0,
            scale_mask,
            key_root: 0,
            last_note_on_time_ms: 0,
        }
    }

    #[test]
    fn snapshot_is_copy_and_send() {
        let snap = SuggestionSnapshot::default();
        let _copy = snap; // Copy
        let _move = snap; // still valid because Copy
    }

    #[test]
    fn snapshot_last_note() {
        let snap = make_snapshot_c_major();
        assert_eq!(snap.last_note(), Some(72)); // C5
        assert_eq!(snap.prev_note(), Some(71)); // B4
    }

    #[test]
    fn snapshot_recent_mean() {
        let snap = make_snapshot_c_major();
        let mean = snap.recent_mean();
        // Mean of 60..72 step = (60+62+64+65+67+69+71+72)/8 = 66.25
        assert!((mean - 66.25).abs() < 0.01);
    }

    // ── Individual term tests ──

    #[test]
    fn test_f_proximity() {
        let mut snap = SuggestionSnapshot::default();
        snap.recent_notes[0] = 60;
        snap.recent_count = 1;

        // Distance 0 -> 1.0
        assert!((f_proximity(60, &snap, 3.0).unwrap() - 1.0).abs() < 0.001);
        // Distance 1 -> exp(-1/3) ~= 0.716
        assert!((f_proximity(61, &snap, 3.0).unwrap() - 0.716).abs() < 0.01);
        // Distance 3 -> exp(-1) ~= 0.368
        assert!((f_proximity(63, &snap, 3.0).unwrap() - 0.368).abs() < 0.01);
    }

    #[test]
    fn test_f_scale_tone() {
        let snap = make_snapshot_c_major();
        // C (in scale, tonic)
        assert!((f_scale_tone(60, &snap) - 1.0).abs() < 0.001);
        // D (in scale, not tonic)
        assert!((f_scale_tone(62, &snap) - 0.7).abs() < 0.001);
        // C# (chromatic neighbor)
        assert!((f_scale_tone(61, &snap) - 0.3).abs() < 0.001);
    }

    #[test]
    fn test_f_chord_tone() {
        let snap = make_snapshot_c_major();
        // C (chord root) -> 1.0
        assert!((f_chord_tone(60, &snap).unwrap() - 1.0).abs() < 0.001);
        // E (chord tone) -> 0.8
        assert!((f_chord_tone(64, &snap).unwrap() - 0.8).abs() < 0.001);
        // D (scale but not chord) -> 0.2
        assert!((f_chord_tone(62, &snap).unwrap() - 0.2).abs() < 0.001);
        // C# (chromatic) -> 0.0
        assert!((f_chord_tone(61, &snap).unwrap() - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_f_contour_continuation() {
        let mut snap = SuggestionSnapshot::default();
        snap.recent_notes[0] = 60;
        snap.recent_notes[1] = 62;
        snap.recent_count = 2;

        // Ascending context (60->62), candidate 64 continues ascending by step
        assert!((f_contour(64, &snap).unwrap() - 1.0).abs() < 0.001);
        // Candidate 60 reverses by step
        assert!((f_contour(60, &snap).unwrap() - 0.75).abs() < 0.001);
    }

    #[test]
    fn test_f_leap_recovery() {
        let mut snap = SuggestionSnapshot::default();
        snap.recent_notes[0] = 60;
        snap.recent_notes[1] = 67; // ascending leap of 7
        snap.recent_count = 2;

        // Descending step (reversal) = 1.0
        assert!((f_leap_recovery(65, &snap).unwrap() - 1.0).abs() < 0.001);
        // Continuing upward = 0.1
        assert!((f_leap_recovery(72, &snap).unwrap() - 0.1).abs() < 0.001);
    }

    #[test]
    fn test_f_repetition() {
        let mut snap = SuggestionSnapshot::default();
        snap.recent_notes = [60, 60, 60, 60, 62, 64, 65, 67];
        snap.recent_count = 8;

        // 60 appeared 4 times: f = 1 - 4/8 = 0.5
        assert!((f_repetition(60, &snap, 8) - 0.5).abs() < 0.001);
        // 69 never appeared: f = 1.0
        assert!((f_repetition(69, &snap, 8) - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_f_leading_tone_resolution() {
        let mut snap = make_snapshot_c_major();
        // Last note is B (degree 7 in C major) = MIDI 71
        snap.recent_notes[7] = 71;
        snap.recent_count = 8;

        // C (tonic) resolves leading tone
        assert!((f_leading_tone(60, &snap).unwrap() - 1.0).abs() < 0.001);
        // D does not resolve
        assert!((f_leading_tone(62, &snap).unwrap() - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_f_dissonance() {
        let snap = make_snapshot_c_major();
        // C against C chord root = perfect consonance -> 1.0
        assert!((f_dissonance(60, &snap).unwrap() - 1.0).abs() < 0.001);
        // G against C = P5 = perfect consonance -> 1.0
        assert!((f_dissonance(67, &snap).unwrap() - 1.0).abs() < 0.001);
        // E against C = M3 = imperfect consonance -> 0.8
        assert!((f_dissonance(64, &snap).unwrap() - 0.8).abs() < 0.001);
        // F# against C = tritone -> 0.125
        assert!((f_dissonance(66, &snap).unwrap() - 0.125).abs() < 0.001);
    }

    #[test]
    fn test_f_tessitura() {
        let snap = make_snapshot_c_major();
        let mean = snap.recent_mean(); // ~66.25
                                       // At the mean -> 1.0
        let at_mean = f_tessitura(66, &snap, 5.0);
        assert!(at_mean > 0.99);
        // Far away -> small
        let far = f_tessitura(40, &snap, 5.0);
        assert!(far < 0.01);
    }

    #[test]
    fn test_f_narmour() {
        let mut snap = SuggestionSnapshot::default();
        snap.recent_notes[0] = 60;
        snap.recent_notes[1] = 67; // large ascending leap
        snap.recent_count = 2;

        // Reversal (descending step) should score high
        let rev = f_narmour(65, &snap).unwrap();
        // Continued large leap should score low
        let cont = f_narmour(74, &snap).unwrap();
        assert!(rev > cont, "Reversal {rev} should beat continuation {cont}");
    }

    // ── Integration tests ──

    #[test]
    fn test_rank_candidates_returns_sorted() {
        let snap = make_snapshot_c_major();
        let config = SuggestionConfig::default();
        let ranked = rank_candidates(&snap, &config);

        assert!(!ranked.is_empty());
        // Check descending order
        for w in ranked.windows(2) {
            assert!(
                w[0].1 >= w[1].1,
                "Not sorted: {} ({}) should be >= {} ({})",
                w[0].0,
                w[0].1,
                w[1].0,
                w[1].1
            );
        }
    }

    #[test]
    fn test_leading_tone_resolution_ranks_high() {
        // B4 as last note in C major -> C5 should rank in top 3
        let scale_mask =
            (1 << 0) | (1 << 2) | (1 << 4) | (1 << 5) | (1 << 7) | (1 << 9) | (1 << 11);
        let chord_mask = (1 << 0) | (1 << 4) | (1 << 7);

        let snap = SuggestionSnapshot {
            recent_notes: [60, 62, 64, 65, 67, 69, 67, 71],
            recent_count: 8,
            current_chord_root: 0,
            current_chord_quality: 0,
            chord_pc_mask: chord_mask,
            next_chord_root: 255,
            next_chord_confidence: 0.0,
            next_chord_pc_mask: 0,
            scale_mask,
            key_root: 0,
            last_note_on_time_ms: 0,
        };

        let config = SuggestionConfig::default();
        let ranked = rank_candidates(&snap, &config);
        let top3: Vec<u8> = ranked.iter().take(3).map(|(n, _)| *n).collect();

        // C5 (72) should be in top 3 due to leading tone resolution
        assert!(
            top3.contains(&72),
            "C5 (72) should be in top 3 after B4. Top 3: {:?}",
            top3
        );
    }

    #[test]
    fn test_determinism() {
        let snap = make_snapshot_c_major();
        let config = SuggestionConfig::default();

        let r1 = rank_candidates(&snap, &config);
        let r2 = rank_candidates(&snap, &config);

        assert_eq!(r1.len(), r2.len());
        for (a, b) in r1.iter().zip(r2.iter()) {
            assert_eq!(a.0, b.0);
            assert!((a.1 - b.1).abs() < 1e-6);
        }
    }

    #[test]
    fn test_score_within_bounds() {
        let snap = make_snapshot_c_major();
        let config = SuggestionConfig::default();

        for midi in config.range_low..=config.range_high {
            let score = score_candidate(midi, &snap, &config);
            assert!(
                (0.0..=1.0).contains(&score),
                "Score for MIDI {} = {} out of [0,1]",
                midi,
                score
            );
        }
    }

    #[test]
    fn test_empty_snapshot_no_panic() {
        let snap = SuggestionSnapshot::default();
        let config = SuggestionConfig::default();
        let ranked = rank_candidates(&snap, &config);
        assert!(!ranked.is_empty());
        // All scores should be valid
        for (_, score) in &ranked {
            assert!(score.is_finite());
        }
    }

    // ── Guitar position tests ──

    #[test]
    fn test_midi_to_positions() {
        // C4 (MIDI 60) can be played at:
        // String 2 (B3=59), fret 1
        // String 3 (G3=55), fret 5
        // String 4 (D3=50), fret 10
        // String 5 (A2=45), fret 15
        // String 6 (E2=40), fret 20
        let positions = midi_to_positions(60);
        assert!(positions.len() >= 4);
        assert!(positions.contains(&(4, 1))); // B string index 4, fret 1
        assert!(positions.contains(&(3, 5))); // G string index 3, fret 5
    }

    #[test]
    fn test_position_scoring_prefers_near_hand() {
        // Hand at fret 5, last string 2 (G string)
        // MIDI 60 at string 3 fret 5 should score highest
        let positions = score_positions(60, 5.0, 3, &PositionConfig::default());
        assert!(!positions.is_empty());
        // Best position should be string 3 fret 5
        assert_eq!(positions[0].string, 3);
        assert_eq!(positions[0].fret, 5);
    }

    #[test]
    fn test_position_scoring_penalizes_far_frets() {
        let positions = score_positions(60, 5.0, 2, &PositionConfig::default());
        // String 5 fret 20 should score much lower than string 3 fret 5
        let near = positions.iter().find(|p| p.fret == 5).unwrap().score;
        let far = positions.iter().find(|p| p.fret == 20).unwrap().score;
        assert!(
            near > far * 3.0,
            "Near ({near}) should be much higher than far ({far})"
        );
    }

    #[test]
    fn test_config_set_weight() {
        let mut config = SuggestionConfig::default();
        assert!(config.set_weight("proximity", 5.0));
        assert_eq!(config.w_proximity, 5.0);
        assert!(!config.set_weight("nonexistent", 5.0));

        // Clamping
        config.set_weight("proximity", 15.0);
        assert_eq!(config.w_proximity, 10.0);
        config.set_weight("proximity", -5.0);
        assert_eq!(config.w_proximity, 0.0);
    }
}
