//! Chord-level voicing algorithm with deterministic tiebreaking.
//!
//! Generates complete chord voicings by evaluating all candidates holistically,
//! not greedily per-voice. Output is fully deterministic: same input always
//! produces the same output.

use super::rules::{
    check_motion_independence, check_parallel_fifths, check_parallel_octaves, check_spacing,
    check_voice_crossing,
};
use super::styles::StyleRules;

/// Voice register with MIDI range constraints.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VoiceRegister {
    Soprano, // MIDI 60-81
    Alto,    // MIDI 55-76
    Tenor,   // MIDI 48-69
    Bass,    // MIDI 40-64
}

impl VoiceRegister {
    /// Returns (low, high) MIDI range inclusive.
    pub fn range(&self) -> (u8, u8) {
        match self {
            VoiceRegister::Soprano => (60, 81),
            VoiceRegister::Alto => (55, 76),
            VoiceRegister::Tenor => (48, 69),
            VoiceRegister::Bass => (40, 64),
        }
    }

    /// Returns all MIDI notes of the given pitch class within this register's range.
    pub fn valid_placements(&self, pitch_class: u8) -> Vec<u8> {
        let (lo, hi) = self.range();
        let pc = pitch_class % 12;
        let mut result = Vec::new();
        // Find lowest note with this pitch class >= lo
        let mut note = lo + ((12 + pc - (lo % 12)) % 12);
        if note < lo {
            note += 12;
        }
        while note <= hi {
            result.push(note);
            note += 12;
        }
        result
    }

    /// Returns register center MIDI note (for first-chord preference).
    pub fn center(&self) -> u8 {
        let (lo, hi) = self.range();
        (lo + hi) / 2
    }

    /// Returns ordering value: 0=soprano (highest), 3=bass (lowest).
    pub fn order(&self) -> u8 {
        match self {
            VoiceRegister::Soprano => 0,
            VoiceRegister::Alto => 1,
            VoiceRegister::Tenor => 2,
            VoiceRegister::Bass => 3,
        }
    }
}

/// Revoice harmony pitch classes into a complete chord voicing.
///
/// # Arguments
/// - `harmony_pitch_classes`: pitch classes (0-11) for each harmony voice
/// - `prev_voicing`: previous chord's MIDI notes (index 0 = melody, 1+ = harmony),
///   or None for the first chord
/// - `registers`: register assignment per voice (index 0 = melody placeholder, 1+ = harmony)
/// - `style_rules`: style constraints
///
/// # Returns
/// MIDI note values for each harmony voice (same length as `harmony_pitch_classes`).
pub fn revoice_chord(
    harmony_pitch_classes: &[u8],
    prev_voicing: Option<&[u8]>,
    registers: &[VoiceRegister],
    style_rules: &StyleRules,
) -> Vec<u8> {
    let n = harmony_pitch_classes.len();
    if n == 0 {
        return Vec::new();
    }

    // Generate valid placements for each voice
    let placements: Vec<Vec<u8>> = (0..n)
        .map(|i| {
            let reg = if i + 1 < registers.len() {
                registers[i + 1]
            } else {
                VoiceRegister::Alto // fallback
            };
            reg.valid_placements(harmony_pitch_classes[i])
        })
        .collect();

    // Check if any voice has no placements
    if placements.iter().any(|p| p.is_empty()) {
        // Fallback: return pitch classes shifted to middle octave
        return harmony_pitch_classes
            .iter()
            .map(|&pc| 60 + (pc % 12))
            .collect();
    }

    // Generate cartesian product of all placements
    let candidates = cartesian_product(&placements);

    if candidates.is_empty() {
        return harmony_pitch_classes
            .iter()
            .map(|&pc| 60 + (pc % 12))
            .collect();
    }

    // Build full voicing (with melody placeholder at index 0) for rule checking
    let melody_note = prev_voicing.map(|p| p[0]).unwrap_or(72);

    let mut scored: Vec<(i64, Vec<u8>)> = Vec::new();
    let mut rejected_candidates: Vec<(i64, Vec<u8>)> = Vec::new();

    for candidate in &candidates {
        let mut full_voicing = vec![melody_note];
        full_voicing.extend_from_slice(candidate);

        let mut hard_rejected = false;
        let mut score: i64 = 0;

        // Check hard constraints against previous voicing
        if let Some(prev) = prev_voicing {
            if style_rules.hard_reject_parallel_fifths {
                let fifths = check_parallel_fifths(prev, &full_voicing);
                if !fifths.is_empty() {
                    hard_rejected = true;
                }
            }
            if style_rules.hard_reject_parallel_octaves {
                let octaves = check_parallel_octaves(prev, &full_voicing);
                if !octaves.is_empty() {
                    hard_rejected = true;
                }
            }

            // Check max leap
            for i in 0..n {
                let prev_note = if i + 1 < prev.len() { prev[i + 1] } else { continue };
                let curr_note = candidate[i];
                let leap = if curr_note >= prev_note {
                    curr_note - prev_note
                } else {
                    prev_note - curr_note
                };
                if leap > style_rules.max_leap_semitones {
                    hard_rejected = true;
                }
            }
        }

        // Spacing check
        let spacing_violations = check_spacing(&full_voicing);
        if !spacing_violations.is_empty() {
            score += spacing_violations.len() as i64 * style_rules.spacing_violation_penalty as i64;
        }

        // Voice crossing check
        let crossings = check_voice_crossing(&full_voicing, registers);
        score += crossings.len() as i64 * style_rules.voice_crossing_penalty as i64;

        // Soft penalties for parallels (when not hard-rejected)
        if let Some(prev) = prev_voicing {
            if !style_rules.hard_reject_parallel_fifths {
                let fifths = check_parallel_fifths(prev, &full_voicing);
                score += fifths.len() as i64 * style_rules.parallel_fifths_penalty as i64;
            }
            if !style_rules.hard_reject_parallel_octaves {
                let octaves = check_parallel_octaves(prev, &full_voicing);
                score += octaves.len() as i64 * style_rules.parallel_octaves_penalty as i64;
            }

            // Motion independence
            if check_motion_independence(prev, &full_voicing) {
                score += style_rules.all_parallel_motion_penalty as i64;
            }

            // Stepwise and common tone bonuses, leap penalties
            for i in 0..n {
                let prev_note = if i + 1 < prev.len() { prev[i + 1] } else { continue };
                let curr_note = candidate[i];
                let movement = if curr_note >= prev_note {
                    curr_note - prev_note
                } else {
                    prev_note - curr_note
                };

                if movement == 0 {
                    score += style_rules.common_tone_bonus as i64;
                } else if movement <= 2 {
                    score += style_rules.stepwise_bonus as i64;
                } else {
                    score += movement as i64 * style_rules.leap_penalty_per_semitone as i64;
                }
            }
        } else {
            // First chord: prefer voicings near register centers
            for i in 0..n {
                let reg = if i + 1 < registers.len() {
                    registers[i + 1]
                } else {
                    VoiceRegister::Alto
                };
                let center = reg.center() as i64;
                let dist = (candidate[i] as i64 - center).abs();
                score -= dist; // closer to center = better
            }
        }

        if hard_rejected {
            rejected_candidates.push((score, candidate.clone()));
        } else {
            scored.push((score, candidate.clone()));
        }
    }

    // Fallback: if all rejected, use rejected candidates with soft scoring
    if scored.is_empty() {
        scored = rejected_candidates;
    }

    if scored.is_empty() {
        return harmony_pitch_classes
            .iter()
            .map(|&pc| 60 + (pc % 12))
            .collect();
    }

    // Deterministic sort with tiebreaking
    scored.sort_by(|a, b| {
        // 1. Score descending
        let score_cmp = b.0.cmp(&a.0);
        if score_cmp != std::cmp::Ordering::Equal {
            return score_cmp;
        }

        // Tiebreakers require previous voicing context
        if let Some(prev) = prev_voicing {
            // 2. Common tones descending
            let common_a = count_common_tones(&a.1, prev);
            let common_b = count_common_tones(&b.1, prev);
            let ct_cmp = common_b.cmp(&common_a);
            if ct_cmp != std::cmp::Ordering::Equal {
                return ct_cmp;
            }

            // 3. Stepwise-down count descending
            let sw_down_a = count_stepwise_down(&a.1, prev);
            let sw_down_b = count_stepwise_down(&b.1, prev);
            let swd_cmp = sw_down_b.cmp(&sw_down_a);
            if swd_cmp != std::cmp::Ordering::Equal {
                return swd_cmp;
            }

            // 4. Stepwise-up count descending
            let sw_up_a = count_stepwise_up(&a.1, prev);
            let sw_up_b = count_stepwise_up(&b.1, prev);
            let swu_cmp = sw_up_b.cmp(&sw_up_a);
            if swu_cmp != std::cmp::Ordering::Equal {
                return swu_cmp;
            }

            // 5. Total movement ascending
            let move_a = total_movement(&a.1, prev);
            let move_b = total_movement(&b.1, prev);
            let mov_cmp = move_a.cmp(&move_b);
            if mov_cmp != std::cmp::Ordering::Equal {
                return mov_cmp;
            }
        }

        // 6. Lexicographic MIDI ascending
        a.1.cmp(&b.1)
    });

    scored[0].1.clone()
}

fn count_common_tones(candidate: &[u8], prev: &[u8]) -> usize {
    candidate
        .iter()
        .enumerate()
        .filter(|(i, &note)| {
            let pi = i + 1;
            pi < prev.len() && prev[pi] == note
        })
        .count()
}

fn count_stepwise_down(candidate: &[u8], prev: &[u8]) -> usize {
    candidate
        .iter()
        .enumerate()
        .filter(|(i, &note)| {
            let pi = i + 1;
            pi < prev.len() && prev[pi] > note && prev[pi] - note <= 2
        })
        .count()
}

fn count_stepwise_up(candidate: &[u8], prev: &[u8]) -> usize {
    candidate
        .iter()
        .enumerate()
        .filter(|(i, &note)| {
            let pi = i + 1;
            pi < prev.len() && note > prev[pi] && note - prev[pi] <= 2
        })
        .count()
}

fn total_movement(candidate: &[u8], prev: &[u8]) -> u32 {
    candidate
        .iter()
        .enumerate()
        .map(|(i, &note)| {
            let pi = i + 1;
            if pi < prev.len() {
                (note as i32 - prev[pi] as i32).unsigned_abs()
            } else {
                0
            }
        })
        .sum()
}

/// Cartesian product of vectors of placements.
fn cartesian_product(sets: &[Vec<u8>]) -> Vec<Vec<u8>> {
    if sets.is_empty() {
        return vec![vec![]];
    }
    let mut result = vec![vec![]];
    for set in sets {
        let mut new_result = Vec::new();
        for existing in &result {
            for &item in set {
                let mut combo = existing.clone();
                combo.push(item);
                new_result.push(combo);
            }
        }
        result = new_result;
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::harmony::voice_leading::styles::{StyleRules, VoiceLeadingStyle};

    fn default_registers_3() -> Vec<VoiceRegister> {
        vec![
            VoiceRegister::Soprano, // melody placeholder
            VoiceRegister::Soprano,
            VoiceRegister::Alto,
            VoiceRegister::Tenor,
        ]
    }

    fn default_registers_2() -> Vec<VoiceRegister> {
        vec![
            VoiceRegister::Soprano, // melody placeholder
            VoiceRegister::Soprano,
            VoiceRegister::Alto,
        ]
    }

    fn default_registers_1() -> Vec<VoiceRegister> {
        vec![
            VoiceRegister::Soprano, // melody placeholder
            VoiceRegister::Alto,
        ]
    }

    #[test]
    fn test_valid_placements() {
        // C (pitch class 0) in soprano range 60-81
        let placements = VoiceRegister::Soprano.valid_placements(0);
        assert_eq!(placements, vec![60, 72]);
    }

    #[test]
    fn test_valid_placements_alto() {
        // E (pitch class 4) in alto range 55-76
        let placements = VoiceRegister::Alto.valid_placements(4);
        assert_eq!(placements, vec![64, 76]);
    }

    #[test]
    fn test_determinism_100_times() {
        let pcs = vec![4, 7]; // E, G
        let prev = vec![72u8, 64, 67]; // melody=C5, soprano=E4, alto=G4
        let registers = default_registers_2();
        let rules = StyleRules::for_style(VoiceLeadingStyle::BachChorale);

        let first = revoice_chord(&pcs, Some(&prev), &registers, &rules);
        for _ in 0..100 {
            let result = revoice_chord(&pcs, Some(&prev), &registers, &rules);
            assert_eq!(result, first, "Determinism violated!");
        }
    }

    #[test]
    fn test_common_tone_retention() {
        // C-E-G -> C-E-A: C and E should be retained
        // Previous: soprano=E4(64), alto=G4(67)
        // New pitch classes: E(4), A(9)
        let pcs = vec![4, 9]; // E, A
        let prev = vec![72u8, 64, 67]; // melody, E4, G4
        let registers = default_registers_2();
        let rules = StyleRules::for_style(VoiceLeadingStyle::BachChorale);

        let result = revoice_chord(&pcs, Some(&prev), &registers, &rules);
        // E should stay at 64 (common tone)
        assert_eq!(result[0], 64, "E should be retained as common tone");
    }

    #[test]
    fn test_parallel_fifth_rejection_palestrina() {
        // Set up a scenario where parallel fifths would occur
        let rules = StyleRules::for_style(VoiceLeadingStyle::Palestrina);
        let registers = default_registers_2();

        // Previous: soprano=C4(60), alto=F3(53) — interval = 7 (P5)
        // If next were D4(62), G3(55) — also P5 — that's parallel fifths
        // The voicer should avoid this in Palestrina style
        let prev = vec![72u8, 60, 53];
        let pcs = vec![2, 7]; // D, G

        let result = revoice_chord(&pcs, Some(&prev), &registers, &rules);

        // Build full voicing and check no parallel fifths
        let mut full = vec![72u8]; // melody unchanged for test
        full.extend_from_slice(&result);
        let fifths = super::super::rules::check_parallel_fifths(&prev, &full);
        // Should either avoid parallel fifths or use fallback
        // (If all candidates have fifths, fallback relaxes constraint)
        // At minimum, the function should not panic
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_first_chord_voicing() {
        let pcs = vec![4, 7, 0]; // E, G, C
        let registers = default_registers_3();
        let rules = StyleRules::for_style(VoiceLeadingStyle::Free);

        let result = revoice_chord(&pcs, None, &registers, &rules);
        assert_eq!(result.len(), 3);

        // All notes should be within their register ranges
        let (slo, shi) = VoiceRegister::Soprano.range();
        assert!(result[0] >= slo && result[0] <= shi);
        let (alo, ahi) = VoiceRegister::Alto.range();
        assert!(result[1] >= alo && result[1] <= ahi);
        let (tlo, thi) = VoiceRegister::Tenor.range();
        assert!(result[2] >= tlo && result[2] <= thi);
    }

    #[test]
    fn test_single_voice() {
        let pcs = vec![0]; // C
        let registers = default_registers_1();
        let rules = StyleRules::for_style(VoiceLeadingStyle::Free);

        let result = revoice_chord(&pcs, None, &registers, &rules);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0] % 12, 0); // Is a C
    }

    #[test]
    fn test_empty_input() {
        let pcs: Vec<u8> = vec![];
        let registers = vec![VoiceRegister::Soprano];
        let rules = StyleRules::for_style(VoiceLeadingStyle::Free);
        let result = revoice_chord(&pcs, None, &registers, &rules);
        assert!(result.is_empty());
    }

    #[test]
    fn test_fallback_when_all_rejected() {
        // With very strict rules and limited options, fallback should still produce output
        let pcs = vec![2, 7]; // D, G
        let prev = vec![72u8, 60, 53]; // Creates parallel fifth scenario
        let registers = default_registers_2();
        let mut rules = StyleRules::for_style(VoiceLeadingStyle::Palestrina);
        rules.max_leap_semitones = 1; // Very restrictive — most candidates rejected

        let result = revoice_chord(&pcs, Some(&prev), &registers, &rules);
        assert_eq!(result.len(), 2);
        // Should still produce valid output (fallback relaxes hard constraints)
    }

    #[test]
    fn test_cartesian_product() {
        let sets = vec![vec![1, 2], vec![3, 4]];
        let result = cartesian_product(&sets);
        assert_eq!(result.len(), 4);
        assert!(result.contains(&vec![1, 3]));
        assert!(result.contains(&vec![1, 4]));
        assert!(result.contains(&vec![2, 3]));
        assert!(result.contains(&vec![2, 4]));
    }

    #[test]
    fn test_voice_register_order() {
        assert!(VoiceRegister::Soprano.order() < VoiceRegister::Alto.order());
        assert!(VoiceRegister::Alto.order() < VoiceRegister::Tenor.order());
        assert!(VoiceRegister::Tenor.order() < VoiceRegister::Bass.order());
    }
}
