//! Chord selection scoring engine.
//!
//! Implements the 6-term composite scoring function to disambiguate which
//! chord best fits the current played note, given harmonic context.
//!
//! Key design: the scorer never IMPOSES a progression -- it REACTS to each
//! played note by selecting the best-fit chord from candidates.

use super::chord_table::{ChordRole, ChordTable, HarmonicFunction};
use super::context::{CadencePhase, HarmonicContext};
use super::markov;

use crate::config::ScaleMode;

/// Scoring weights for the 6-term composite function.
/// Sum should be 1.0.
#[derive(Clone, Debug)]
pub struct ScoringWeights {
    pub markov: f32,
    pub frequency: f32,
    pub voice_lead: f32,
    pub beat: f32,
    pub role: f32,
    pub cadence: f32,
}

impl Default for ScoringWeights {
    fn default() -> Self {
        Self {
            markov: 0.30,
            frequency: 0.10,
            voice_lead: 0.25,
            beat: 0.10,
            role: 0.15,
            cadence: 0.10,
        }
    }
}

/// Hold inertia: bonus for staying on the current chord.
const HOLD_INERTIA: f32 = 0.15;

/// Score a candidate chord degree for the given note and context.
pub fn score_chord(
    degree: u8,
    note_pc: u8,
    ctx: &HarmonicContext,
    scale_mode: ScaleMode,
    weights: &ScoringWeights,
) -> f32 {
    let m = markov_score(degree, ctx, scale_mode);
    let f = frequency_score(degree);
    let v = voice_leading_cost(degree, ctx);
    let b = beat_weight(ctx.beat_position);
    let r = role_bonus(note_pc, degree, &ctx.chord_table);
    let c = cadence_bonus(degree, ctx);

    let score = weights.markov * m
        + weights.frequency * f
        + weights.voice_lead * v
        + weights.beat * b
        + weights.role * r
        + weights.cadence * c;

    // Add hold-chord inertia if this is the current chord
    score + hold_bonus(degree, ctx, note_pc)
}

/// Markov transition probability P(degree | prev, current).
fn markov_score(degree: u8, ctx: &HarmonicContext, scale_mode: ScaleMode) -> f32 {
    let table = markov::table_for_scale(scale_mode);
    match (ctx.prev_degree, ctx.current_degree) {
        (Some(prev), Some(curr)) => {
            let cadential = ctx.cadence_phase == CadencePhase::DominantArrived
                || ctx.cadence_phase == CadencePhase::Approaching;
            markov::lookup_table(table, prev, curr, ctx.beat_position, cadential, degree)
        }
        (None, Some(curr)) => {
            // Only one previous chord -- use stationary as prev
            markov::lookup_table(table, 0, curr, ctx.beat_position, false, degree)
        }
        _ => {
            // No previous chords -- use stationary distribution
            markov::STATIONARY[degree.min(6) as usize]
        }
    }
}

/// Corpus frequency prior for chord degree.
fn frequency_score(degree: u8) -> f32 {
    markov::FREQUENCY[degree.min(6) as usize]
}

/// Voice leading cost: negative total semitone movement from previous voicing.
/// Normalized to [-1, 0] range. Smooth voice leading scores closer to 0.
fn voice_leading_cost(degree: u8, ctx: &HarmonicContext) -> f32 {
    if let Some(ref prev) = ctx.prev_voicing {
        let chord_pcs = &ctx.chord_table.chord_pcs[degree as usize];
        let mut total_movement: i32 = 0;

        for (i, &prev_note) in prev.iter().enumerate() {
            let pc = chord_pcs[i % 4];
            // Find nearest placement of this PC to the previous note
            let prev_pc = prev_note % 12;
            let dist = min_pc_distance(prev_pc, pc);
            total_movement += dist as i32;
        }

        // Normalize: max movement = 4 voices * 6 semitones = 24
        let normalized = -(total_movement as f32) / 24.0;
        normalized.max(-1.0)
    } else {
        0.0 // No previous voicing -- no cost
    }
}

/// Beat weight: stronger beats encourage chord changes.
fn beat_weight(beat_position: u8) -> f32 {
    match beat_position {
        0 => 1.0, // Downbeat (beat 1)
        2 => 0.5, // Secondary strong (beat 3)
        _ => 0.2, // Weak beats
    }
}

/// Role bonus: bonus when the played note occupies a strong position in the chord.
fn role_bonus(note_pc: u8, degree: u8, table: &ChordTable) -> f32 {
    match table.role_of(note_pc, degree) {
        ChordRole::Root => 0.3,
        ChordRole::Fifth => 0.1,
        ChordRole::Seventh => 0.15,
        ChordRole::Third => 0.0,
        ChordRole::None => -0.1, // Note not in chord -- penalize
    }
}

/// Cadence bonus: boost resolution targets when cadence is in progress.
fn cadence_bonus(degree: u8, ctx: &HarmonicContext) -> f32 {
    match ctx.cadence_phase {
        CadencePhase::DominantArrived => {
            let func = ctx.chord_table.function[degree as usize];
            match func {
                HarmonicFunction::Tonic if degree == 0 => 0.30, // I resolution
                HarmonicFunction::Tonic => 0.10,                // vi (deceptive)
                _ => -0.10,                                     // Don't abandon cadence
            }
        }
        CadencePhase::Approaching => {
            let func = ctx.chord_table.function[degree as usize];
            if func == HarmonicFunction::Dominant {
                0.20 // Approaching V
            } else {
                0.0
            }
        }
        CadencePhase::None => 0.0,
    }
}

/// Hold-chord inertia: bonus for staying on the current chord when the note fits.
fn hold_bonus(degree: u8, ctx: &HarmonicContext, note_pc: u8) -> f32 {
    if let Some(curr) = ctx.current_degree {
        if degree == curr && ctx.chord_table.contains(note_pc, degree) {
            return HOLD_INERTIA;
        }
    }
    0.0
}

/// Select the best chord degree for a given note.
///
/// Scores all candidate degrees (those whose chords contain the note)
/// and returns the winner. For chromatic notes, scores all 7 degrees
/// with a penalty for non-membership.
pub fn select_chord(
    note_pc: u8,
    ctx: &HarmonicContext,
    scale_mode: ScaleMode,
    weights: &ScoringWeights,
) -> u8 {
    let candidates: Vec<u8> = if ctx.chord_table.is_diatonic(note_pc) {
        ctx.chord_table.degrees_containing(note_pc)
    } else {
        // Chromatic note: score all degrees (they'll get role penalty)
        (0..7).collect()
    };

    if candidates.is_empty() {
        // Fallback: use stationary distribution favorite (I)
        return 0;
    }

    let mut scored: Vec<(u8, f32)> = candidates
        .iter()
        .map(|&d| (d, score_chord(d, note_pc, ctx, scale_mode, weights)))
        .collect();

    // iii penalty: never select iii unless it scores 0.05 above all others
    // (iii is functionally ambiguous)
    let non_iii_max = scored
        .iter()
        .filter(|(d, _)| *d != 2)
        .map(|(_, s)| *s)
        .fold(f32::NEG_INFINITY, f32::max);
    scored.retain(|(d, s)| *d != 2 || *s >= non_iii_max + 0.05);

    if scored.is_empty() {
        return 0;
    }

    select_winner(&scored, ctx)
}

/// Deterministic tiebreaker for chord selection.
fn select_winner(candidates: &[(u8, f32)], ctx: &HarmonicContext) -> u8 {
    let max_score = candidates
        .iter()
        .map(|(_, s)| *s)
        .fold(f32::NEG_INFINITY, f32::max);

    let tied: Vec<u8> = candidates
        .iter()
        .filter(|(_, s)| (s - max_score).abs() < 0.01)
        .map(|(d, _)| *d)
        .collect();

    if tied.len() == 1 {
        return tied[0];
    }

    // Tiebreak 1: prefer chord whose root is closest to previous bass
    if let Some(ref prev) = ctx.prev_voicing {
        let bass_pc = prev[3] % 12;
        let winner = tied
            .iter()
            .min_by_key(|&&d| {
                let root_pc = ctx.chord_table.chord_pcs[d as usize][0];
                min_pc_distance(bass_pc, root_pc)
            })
            .copied();
        if let Some(w) = winner {
            return w;
        }
    }

    // Tiebreak 2: prefer lower Roman numeral
    *tied.iter().min().unwrap_or(&0)
}

/// Minimum distance between two pitch classes on the circle of 12.
#[inline]
fn min_pc_distance(a: u8, b: u8) -> u8 {
    let d = if a > b { a - b } else { b - a };
    d.min(12 - d)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::ScaleMode;

    fn make_ctx() -> HarmonicContext {
        HarmonicContext::new(0, ScaleMode::Ionian)
    }

    #[test]
    fn test_select_chord_c_in_c_major() {
        let ctx = make_ctx();
        let weights = ScoringWeights::default();

        // C (pc=0) in C major: candidates are I, ii(7th), IV, vi
        // I should win on first note (stationary distribution favors I)
        let degree = select_chord(0, &ctx, ScaleMode::Ionian, &weights);
        assert!(
            degree == 0 || degree == 3 || degree == 5,
            "C should select I, IV, or vi, got {}",
            degree
        );
    }

    #[test]
    fn test_select_chord_b_in_c_major() {
        let ctx = make_ctx();
        let weights = ScoringWeights::default();

        // B (pc=11) is in V(third), vii(root), I(seventh), iii(fifth)
        // V should score well from stationary
        let degree = select_chord(11, &ctx, ScaleMode::Ionian, &weights);
        assert!(
            degree == 4 || degree == 6 || degree == 0,
            "B should select V, vii, or I, got {}",
            degree
        );
    }

    #[test]
    fn test_hold_inertia() {
        let mut ctx = make_ctx();
        ctx.change_chord(0); // Currently on I
        ctx.advance_note();

        let weights = ScoringWeights::default();

        // E (pc=4) is in I(third), ii(... no), iii(root)
        // Since we're currently on I and E fits I, hold inertia should boost I
        let i_score = score_chord(0, 4, &ctx, ScaleMode::Ionian, &weights);
        let iii_score = score_chord(2, 4, &ctx, ScaleMode::Ionian, &weights);

        assert!(
            i_score > iii_score,
            "I ({}) should beat iii ({}) due to hold inertia",
            i_score,
            iii_score
        );
    }

    #[test]
    fn test_cadence_boosts_resolution() {
        let mut ctx = make_ctx();

        // Set up V->I cadence context
        ctx.change_chord(3); // IV (approaching)
        ctx.beat_position = 0;
        ctx.change_chord(4); // V (dominant arrived)

        let weights = ScoringWeights::default();

        // Score I vs other targets
        let i_score = score_chord(0, 0, &ctx, ScaleMode::Ionian, &weights);
        let ii_score = score_chord(1, 0, &ctx, ScaleMode::Ionian, &weights);

        assert!(
            i_score > ii_score,
            "I ({}) should beat ii ({}) during cadential resolution",
            i_score,
            ii_score
        );
    }

    #[test]
    fn test_role_bonus_root_beats_third() {
        let table = super::super::chord_table::ChordTable::build(0, ScaleMode::Ionian);

        // C is root of I, third of vi
        let root_bonus = role_bonus(0, 0, &table);
        let third_bonus = role_bonus(0, 5, &table); // C is... actually root of vi? no
                                                    // C (pc=0) role in vi (degree 5): vi = A C E G -> C is third
                                                    // Actually chord_pcs[5] = [9, 0, 4, 7], so pc=0 is third
        assert!(
            root_bonus > third_bonus,
            "Root bonus ({}) should exceed third bonus ({})",
            root_bonus,
            third_bonus
        );
    }

    #[test]
    fn test_min_pc_distance() {
        assert_eq!(min_pc_distance(0, 0), 0);
        assert_eq!(min_pc_distance(0, 6), 6);
        assert_eq!(min_pc_distance(0, 7), 5);
        assert_eq!(min_pc_distance(11, 0), 1);
        assert_eq!(min_pc_distance(0, 11), 1);
    }

    #[test]
    fn test_chromatic_note_still_selects() {
        let ctx = make_ctx();
        let weights = ScoringWeights::default();

        // Db (pc=1) is chromatic in C major -- should still return a valid degree
        let degree = select_chord(1, &ctx, ScaleMode::Ionian, &weights);
        assert!(degree < 7, "Should return valid degree, got {}", degree);
    }
}
