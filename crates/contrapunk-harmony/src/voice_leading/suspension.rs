//! Suspension state machine for voice leading dissonance handling.
//!
//! Suspensions work on a one-chord-delay principle: when a chord changes,
//! one voice may be held from the previous chord (creating a dissonance),
//! then on the NEXT chord change it resolves stepwise down.

use wmidi::Note;

use crate::scale::Scale;

/// Phase of the suspension cycle.
#[derive(Debug, Clone, PartialEq)]
enum SuspensionPhase {
    /// No active suspension
    None,
    /// A voice is being held over from the previous chord (dissonant)
    Suspended,
    /// Next chord change should resolve this voice stepwise down
    NeedResolution,
}

/// Suspension state machine that manages preparation -> suspension -> resolution.
///
/// Only active for Palestrina style (enabled flag). For other styles, `process` is a no-op.
#[derive(Debug, Clone)]
pub struct SuspensionState {
    /// Which voice index in the full voicing (0=melody, 1+=harmony) is suspended
    suspended_voice: Option<usize>,
    /// The MIDI note being held from the previous chord
    suspended_note: Option<u8>,
    /// Current phase of the suspension cycle
    phase: SuspensionPhase,
    /// Whether suspensions are enabled (only for Palestrina style)
    enabled: bool,
}

impl SuspensionState {
    /// Creates a new SuspensionState.
    ///
    /// # Arguments
    /// * `enabled` - true for Palestrina style, false for all others
    pub fn new(enabled: bool) -> Self {
        Self {
            suspended_voice: None,
            suspended_note: None,
            phase: SuspensionPhase::None,
            enabled,
        }
    }

    /// Processes suspension logic on a voicing.
    ///
    /// Modifies `voicing` in place. Index 0 is melody (never modified).
    /// Only harmony voices (index 1+) are candidates for suspension.
    ///
    /// # Arguments
    /// * `voicing` - Current chord voicing (index 0 = melody, 1+ = harmony). Modified in place.
    /// * `prev_voicing` - Previous chord's voicing, or None for first chord.
    /// * `scale` - Current scale for stepwise resolution.
    pub fn process(
        &mut self,
        voicing: &mut Vec<Note>,
        prev_voicing: Option<&[Note]>,
        scale: &Scale,
    ) {
        if !self.enabled || voicing.len() <= 1 {
            return;
        }

        match self.phase {
            SuspensionPhase::NeedResolution => {
                // Resolve: move suspended voice one scale step down
                if let (Some(voice_idx), Some(sus_midi)) =
                    (self.suspended_voice, self.suspended_note)
                {
                    if voice_idx < voicing.len() {
                        if let Ok(sus_note) = Note::try_from(sus_midi) {
                            if let Some(resolved) = scale.transpose_diatonic(sus_note, -1) {
                                voicing[voice_idx] = resolved;
                            }
                        }
                    }
                }
                self.phase = SuspensionPhase::None;
                self.suspended_voice = None;
                self.suspended_note = None;
            }
            SuspensionPhase::Suspended => {
                // Transition to NeedResolution for the next call
                self.phase = SuspensionPhase::NeedResolution;
            }
            SuspensionPhase::None => {
                // Try to create a new suspension from prev_voicing
                if let Some(prev) = prev_voicing {
                    // Pick highest-indexed harmony voice that can form a suspension:
                    // hold a note from prev that differs from the new voicing at that index
                    for i in (1..voicing.len().min(prev.len())).rev() {
                        let prev_midi = u8::from(prev[i]);
                        let curr_midi = u8::from(voicing[i]);
                        if prev_midi != curr_midi {
                            // Hold the previous note (creating dissonance)
                            voicing[i] = prev[i];
                            self.suspended_voice = Some(i);
                            self.suspended_note = Some(prev_midi);
                            self.phase = SuspensionPhase::Suspended;
                            break;
                        }
                    }
                }
            }
        }
    }

    /// Resets all suspension state.
    pub fn reset(&mut self) {
        self.suspended_voice = None;
        self.suspended_note = None;
        self.phase = SuspensionPhase::None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn note(midi: u8) -> Note {
        Note::try_from(midi).unwrap()
    }

    #[test]
    fn test_suspension_hold_and_resolve() {
        let scale = Scale::major(0); // C major
        let mut state = SuspensionState::new(true);

        // Chord 1: C4(melody), E4, G4
        let prev = vec![note(60), note(64), note(67)];

        // Chord 2: C4(melody), F4, A4 — voice 2 changed from G4 to A4
        let mut voicing2 = vec![note(60), note(65), note(69)];
        state.process(&mut voicing2, Some(&prev), &scale);

        // Highest changed voice (index 2) should be held at G4 (67)
        assert_eq!(u8::from(voicing2[2]), 67, "Voice 2 should be held at G4");
        assert_eq!(state.phase, SuspensionPhase::Suspended);

        // Chord 3: C4(melody), D4, F4
        let mut voicing3 = vec![note(60), note(62), note(65)];
        state.process(&mut voicing3, Some(&voicing2), &scale);

        // Phase transitions to NeedResolution
        assert_eq!(state.phase, SuspensionPhase::NeedResolution);

        // Chord 4: resolve — suspended voice (index 2) resolves stepwise down from G4
        let mut voicing4 = vec![note(60), note(64), note(69)];
        state.process(&mut voicing4, Some(&voicing3), &scale);

        // G4 (67) resolves stepwise down to F4 (65) in C major
        assert_eq!(
            u8::from(voicing4[2]),
            65,
            "Suspended G4 should resolve to F4"
        );
        assert_eq!(state.phase, SuspensionPhase::None);
    }

    #[test]
    fn test_non_palestrina_skips_suspension() {
        let scale = Scale::major(0);
        let mut state = SuspensionState::new(false); // Not Palestrina

        let prev = vec![note(60), note(64), note(67)];
        let mut voicing = vec![note(60), note(65), note(69)];
        let original = voicing.clone();

        state.process(&mut voicing, Some(&prev), &scale);

        // Voicing should be unchanged
        assert_eq!(voicing, original);
        assert_eq!(state.phase, SuspensionPhase::None);
    }

    #[test]
    fn test_reset_clears_state() {
        let scale = Scale::major(0);
        let mut state = SuspensionState::new(true);

        // Start a suspension
        let prev = vec![note(60), note(64), note(67)];
        let mut voicing = vec![note(60), note(65), note(69)];
        state.process(&mut voicing, Some(&prev), &scale);
        assert_eq!(state.phase, SuspensionPhase::Suspended);

        // Reset
        state.reset();
        assert_eq!(state.phase, SuspensionPhase::None);
        assert!(state.suspended_voice.is_none());
        assert!(state.suspended_note.is_none());
    }

    #[test]
    fn test_no_suspension_on_first_chord() {
        let scale = Scale::major(0);
        let mut state = SuspensionState::new(true);

        let mut voicing = vec![note(60), note(64), note(67)];
        let original = voicing.clone();

        state.process(&mut voicing, None, &scale);

        // No previous voicing -> no suspension possible
        assert_eq!(voicing, original);
        assert_eq!(state.phase, SuspensionPhase::None);
    }

    #[test]
    fn test_no_suspension_when_voices_unchanged() {
        let scale = Scale::major(0);
        let mut state = SuspensionState::new(true);

        let prev = vec![note(60), note(64), note(67)];
        let mut voicing = vec![note(60), note(64), note(67)]; // Same as prev

        state.process(&mut voicing, Some(&prev), &scale);

        // No voice changed, so no suspension
        assert_eq!(state.phase, SuspensionPhase::None);
    }

    #[test]
    fn test_melody_never_suspended() {
        let scale = Scale::major(0);
        let mut state = SuspensionState::new(true);

        // Only melody changed, harmony same
        let prev = vec![note(60), note(64)];
        let mut voicing = vec![note(62), note(64)]; // Melody changed, harmony same

        state.process(&mut voicing, Some(&prev), &scale);

        // No harmony voice changed, so no suspension
        assert_eq!(state.phase, SuspensionPhase::None);
        // Melody must remain unchanged
        assert_eq!(u8::from(voicing[0]), 62);
    }
}
