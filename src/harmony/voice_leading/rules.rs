//! Voice leading rule checkers.
//!
//! All functions operate on slices of MIDI note values (`u8`).
//! Index 0 is the melody voice; harmony voices start at index 1.
//! Rule checks skip index 0 unless stated otherwise.

use super::voicer::VoiceRegister;

/// Returns `(a - b)` absolute difference mod 12.
pub fn interval_class(a: u8, b: u8) -> u8 {
    let diff = if a >= b { a - b } else { b - a };
    diff % 12
}

/// Returns voice-index pairs that move in parallel fifths.
/// Both voices must have moved, and both the previous and current intervals
/// must be 7 semitones (mod 12) — a perfect fifth.
/// Only checks harmony voices (indices >= 1).
pub fn check_parallel_fifths(prev: &[u8], curr: &[u8]) -> Vec<(usize, usize)> {
    check_parallel_interval(prev, curr, 7)
}

/// Returns voice-index pairs that move in parallel octaves/unisons.
/// Interval = 0 mod 12, both voices moved.
pub fn check_parallel_octaves(prev: &[u8], curr: &[u8]) -> Vec<(usize, usize)> {
    check_parallel_interval(prev, curr, 0)
}

fn check_parallel_interval(prev: &[u8], curr: &[u8], target_ic: u8) -> Vec<(usize, usize)> {
    let len = prev.len().min(curr.len());
    let mut pairs = Vec::new();
    // Only check harmony voices (start from 1)
    for i in 1..len {
        for j in (i + 1)..len {
            let prev_interval = interval_class(prev[i], prev[j]);
            let curr_interval = interval_class(curr[i], curr[j]);
            let i_moved = prev[i] != curr[i];
            let j_moved = prev[j] != curr[j];
            if prev_interval == target_ic && curr_interval == target_ic && i_moved && j_moved {
                pairs.push((i, j));
            }
        }
    }
    pairs
}

/// Returns pairs where a higher-register voice has a lower MIDI note
/// than a lower-register voice (voice crossing).
/// `registers` maps each index to its expected register.
pub fn check_voice_crossing(voicing: &[u8], registers: &[VoiceRegister]) -> Vec<(usize, usize)> {
    let len = voicing.len().min(registers.len());
    let mut pairs = Vec::new();
    for i in 1..len {
        for j in (i + 1)..len {
            let ri = registers[i].order();
            let rj = registers[j].order();
            // Higher register (lower order number) should have higher MIDI note
            if ri < rj && voicing[i] < voicing[j] {
                pairs.push((i, j));
            } else if rj < ri && voicing[j] < voicing[i] {
                pairs.push((i, j));
            }
        }
    }
    pairs
}

/// Returns adjacent voice pairs (i, i+1) where spacing exceeds 12 semitones,
/// EXCEPT for the last two voices (bass-tenor gap is allowed to exceed an octave).
pub fn check_spacing(voicing: &[u8]) -> Vec<(usize, usize)> {
    let mut pairs = Vec::new();
    if voicing.len() < 3 {
        return pairs;
    }
    // Check adjacent harmony voice pairs, skip melody (index 0)
    // The last pair of harmony voices (lowest two) is exempt
    let harmony_start = 1;
    let harmony_end = voicing.len();
    for i in harmony_start..(harmony_end - 1) {
        // Skip the last adjacent pair (bass-tenor)
        if i == harmony_end - 2 {
            continue;
        }
        let diff = if voicing[i] >= voicing[i + 1] {
            voicing[i] - voicing[i + 1]
        } else {
            voicing[i + 1] - voicing[i]
        };
        if diff > 12 {
            pairs.push((i, i + 1));
        }
    }
    pairs
}

/// Returns true if ALL harmony voices (indices 1+) move in the same direction
/// (all up or all down), indicating poor motion independence.
/// Returns false if any voice is stationary or moves in a different direction.
pub fn check_motion_independence(prev: &[u8], curr: &[u8]) -> bool {
    let len = prev.len().min(curr.len());
    if len <= 2 {
        // With 0 or 1 harmony voices, parallel motion is not meaningful
        return false;
    }
    let mut all_up = true;
    let mut all_down = true;
    for i in 1..len {
        if curr[i] > prev[i] {
            all_down = false;
        } else if curr[i] < prev[i] {
            all_up = false;
        } else {
            // Stationary — not all same direction
            return false;
        }
    }
    all_up || all_down
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_interval_class() {
        assert_eq!(interval_class(60, 67), 7); // C4 to G4 = P5
        assert_eq!(interval_class(60, 72), 0); // C4 to C5 = octave
        assert_eq!(interval_class(64, 60), 4); // E4 to C4 = M3
        assert_eq!(interval_class(60, 60), 0); // unison
    }

    #[test]
    fn test_parallel_fifths_detected() {
        // Voices: [melody, soprano, alto]
        // soprano C4->D4 (60->62), alto G3->A3 (55->57)
        // Both intervals are 5 semitones... wait, C-G = 7, D-A = 7. Yes, parallel fifths.
        let prev = vec![72, 60, 53]; // melody, C4, F3
        let curr = vec![74, 62, 55]; // melody, D4, G3
                                     // interval_class(60, 53) = 7, interval_class(62, 55) = 7 — parallel fifths
        let result = check_parallel_fifths(&prev, &curr);
        assert_eq!(result, vec![(1, 2)]);
    }

    #[test]
    fn test_parallel_fifths_not_detected_when_one_stationary() {
        let prev = vec![72, 60, 53];
        let curr = vec![74, 60, 53]; // harmony voices didn't move
        let result = check_parallel_fifths(&prev, &curr);
        assert!(result.is_empty());
    }

    #[test]
    fn test_parallel_octaves_detected() {
        // soprano C4->D4 (60->62), alto C3->D3 (48->50)
        let prev = vec![72, 60, 48];
        let curr = vec![74, 62, 50];
        let result = check_parallel_octaves(&prev, &curr);
        assert_eq!(result, vec![(1, 2)]);
    }

    #[test]
    fn test_voice_crossing_detected() {
        // Soprano should be higher than alto
        let voicing = vec![72, 55, 65]; // melody=72, soprano=55, alto=65 (crossing!)
        let registers = vec![
            VoiceRegister::Soprano, // melody placeholder
            VoiceRegister::Soprano,
            VoiceRegister::Alto,
        ];
        let result = check_voice_crossing(&voicing, &registers);
        assert_eq!(result, vec![(1, 2)]);
    }

    #[test]
    fn test_no_voice_crossing() {
        let voicing = vec![72, 70, 60];
        let registers = vec![
            VoiceRegister::Soprano,
            VoiceRegister::Soprano,
            VoiceRegister::Alto,
        ];
        let result = check_voice_crossing(&voicing, &registers);
        assert!(result.is_empty());
    }

    #[test]
    fn test_spacing_violation() {
        // 3 harmony voices: soprano=76, alto=50, tenor=48
        // soprano-alto gap = 26 > 12 => violation
        // alto-tenor is last pair => exempt
        let voicing = vec![80, 76, 50, 48];
        let result = check_spacing(&voicing);
        assert_eq!(result, vec![(1, 2)]);
    }

    #[test]
    fn test_spacing_no_violation() {
        let voicing = vec![80, 72, 65, 50];
        let result = check_spacing(&voicing);
        assert!(result.is_empty());
    }

    #[test]
    fn test_motion_independence_all_up() {
        let prev = vec![72, 60, 55, 48];
        let curr = vec![74, 62, 57, 50]; // all harmony up
        assert!(check_motion_independence(&prev, &curr));
    }

    #[test]
    fn test_motion_independence_mixed() {
        let prev = vec![72, 60, 55, 48];
        let curr = vec![74, 62, 53, 50]; // mixed: up, down, up
        assert!(!check_motion_independence(&prev, &curr));
    }

    #[test]
    fn test_motion_independence_with_stationary() {
        let prev = vec![72, 60, 55, 48];
        let curr = vec![74, 62, 55, 50]; // alto stationary
        assert!(!check_motion_independence(&prev, &curr));
    }
}
