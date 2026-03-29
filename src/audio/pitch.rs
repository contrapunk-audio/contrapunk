//! Pitch detection types, frequency conversion, and note tracking.
//!
//! Pitch tracking with octave-error correction, pitch-class voting, and onset gating.
//!
//! `NoteTracker` sits between a raw pitch detector (e.g. pyin / autocorrelation)
//! and the harmony engine.  It filters out common guitar-pitch-detection artefacts:
//!
//! - **Octave errors** caused by harmonics (T1)
//! - **Transient glitches** smoothed by 3-frame median voting (T3)
//! - **Ghost notes** suppressed by onset gating (T5)

/// Result of pitch detection.
#[derive(Clone, Debug)]
pub struct DetectedPitch {
    /// Detected frequency in Hz
    pub frequency: f32,
    /// Nearest MIDI note number (0-127)
    pub midi_note: u8,
    /// Deviation from nearest note in cents (-50 to +50)
    pub cents_deviation: i8,
    /// Confidence/clarity score (0.0-1.0)
    pub clarity: f32,
}

/// Converts frequency (Hz) to MIDI note with cents deviation.
/// Formula: midi = 69 + 12 * log2(freq / 440)
pub fn freq_to_midi(freq: f32) -> (u8, i8) {
    if freq <= 0.0 {
        return (0, 0);
    }
    let midi_float = 69.0 + 12.0 * (freq / 440.0).log2();
    let midi_note = midi_float.round() as i32;
    let cents = ((midi_float - midi_note as f32) * 100.0) as i8;
    (midi_note.clamp(0, 127) as u8, cents.clamp(-50, 50))
}

/// Checks if a MIDI note is within the configured voice range.
pub fn is_in_voice_range(midi_note: u8, min: u8, max: u8) -> bool {
    midi_note >= min && midi_note <= max
}

/// Configuration for the note tracker.
#[derive(Debug, Clone)]
pub struct NoteTrackerConfig {
    /// Minimum confidence / clarity threshold for accepting an octave jump.
    /// When a detected note is exactly +/-12 semitones from the current note
    /// and confidence is below this value, the jump is rejected.
    pub octave_jump_confidence_threshold: f32,

    /// When `true`, new NoteOn events only fire after `signal_onset()` is called.
    /// Sustain and NoteOff processing are unaffected.
    pub onset_required: bool,
}

impl Default for NoteTrackerConfig {
    fn default() -> Self {
        Self {
            octave_jump_confidence_threshold: 0.6,
            onset_required: true,
        }
    }
}

/// Events emitted by the tracker.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NoteEvent {
    /// A new note has been confirmed (MIDI note number).
    NoteOn(u8),
    /// The current note has ended.
    NoteOff(u8),
}

/// A pitch tracker that filters raw pitch detections before emitting note events.
///
/// # Filtering stages
///
/// 1. **Octave-error rejection (T1)** -- if the new note is exactly +/-12
///    semitones from the current note and `confidence < 0.6`, the detection
///    is silently dropped.
///
/// 2. **3-frame median voting (T3)** -- a 3-slot circular buffer records
///    recent MIDI note values.  A NoteOn is only emitted when at least 2 of
///    the last 3 slots agree on the same note.
///
/// 3. **Onset gating (T5)** -- unless `onset_required` is `false`, a NoteOn
///    is suppressed until `signal_onset()` has been called.  The gate
///    automatically re-arms after a note is emitted.
#[derive(Debug)]
pub struct NoteTracker {
    config: NoteTrackerConfig,

    /// The currently active (emitted) note, if any.
    current_note: Option<u8>,

    // -- T3: 3-frame voting buffer --
    vote_buf: [Option<u8>; 3],
    vote_idx: usize,

    // -- T5: onset gate --
    onset_armed: bool,
}

impl NoteTracker {
    /// Create a new tracker with the given configuration.
    pub fn new(config: NoteTrackerConfig) -> Self {
        Self {
            config,
            current_note: None,
            vote_buf: [None; 3],
            vote_idx: 0,
            onset_armed: false,
        }
    }

    /// Create a tracker with default configuration.
    pub fn with_defaults() -> Self {
        Self::new(NoteTrackerConfig::default())
    }

    // -- T5 API ---------------------------------------------------------------

    /// Signal that an onset has been detected by an external detector.
    ///
    /// This arms the gate so the next confirmed pitch detection can emit a
    /// NoteOn.
    pub fn signal_onset(&mut self) {
        self.onset_armed = true;
    }

    /// Returns whether the onset gate is currently armed.
    pub fn is_onset_armed(&self) -> bool {
        self.onset_armed
    }

    // -- Primary API ----------------------------------------------------------

    /// Feed a new pitch detection into the tracker.
    ///
    /// * `midi_note` -- detected MIDI note number (0-127).
    /// * `confidence` -- detection confidence / clarity in `[0.0, 1.0]`.
    ///
    /// Returns an optional `NoteEvent` if a note change should be propagated.
    pub fn update(&mut self, midi_note: u8, confidence: f32) -> Option<NoteEvent> {
        // -- T1: Octave error rejection ---------------------------------------
        if let Some(current) = self.current_note {
            let interval = (midi_note as i16 - current as i16).abs();
            if interval == 12 {
                if self.config.onset_required {
                    // During sustain (no onset armed), ALWAYS reject octave jumps.
                    // The fundamental doesn't jump an octave without a new pluck.
                    // Only allow octave change if onset is armed (new attack detected).
                    if !self.onset_armed
                        || confidence < self.config.octave_jump_confidence_threshold
                    {
                        return None;
                    }
                } else {
                    // Without onset gating, fall back to confidence-only check.
                    if confidence < self.config.octave_jump_confidence_threshold {
                        return None;
                    }
                }
            }
        }

        // -- T3: 3-frame voting -----------------------------------------------
        self.vote_buf[self.vote_idx] = Some(midi_note);
        self.vote_idx = (self.vote_idx + 1) % 3;

        let consensus = self.majority_vote();
        let agreed_note = match consensus {
            Some(n) => n,
            None => return None, // no 2/3 agreement yet
        };

        // If the agreed note is the same as the current note, nothing to do.
        if self.current_note == Some(agreed_note) {
            return None;
        }

        // -- T5: Onset gating -------------------------------------------------
        if self.config.onset_required && !self.onset_armed {
            return None;
        }

        // Emit note change.
        // If there was a previous note we could emit NoteOff first, but the
        // caller can handle that via the returned NoteOn (previous note is
        // implicitly ended).  We keep it simple here.
        self.current_note = Some(agreed_note);
        self.onset_armed = false; // re-arm required for next note
        Some(NoteEvent::NoteOn(agreed_note))
    }

    /// Notify the tracker that the current note has ended (e.g. silence
    /// detected).  Returns a `NoteOff` event if a note was active.
    ///
    /// This works regardless of onset gating -- sustain / NoteOff is never
    /// suppressed.
    pub fn note_off(&mut self) -> Option<NoteEvent> {
        if let Some(note) = self.current_note.take() {
            // Clear the voting buffer so stale data doesn't bleed into the
            // next note.
            self.vote_buf = [None; 3];
            Some(NoteEvent::NoteOff(note))
        } else {
            None
        }
    }

    /// Returns the currently active note, if any.
    pub fn current_note(&self) -> Option<u8> {
        self.current_note
    }

    // -- internals ------------------------------------------------------------

    /// Check if at least 2 of the 3 vote slots agree on the same note.
    fn majority_vote(&self) -> Option<u8> {
        let a = self.vote_buf[0];
        let b = self.vote_buf[1];
        let c = self.vote_buf[2];

        // Check all pairs for agreement.
        if a.is_some() && a == b {
            return a;
        }
        if a.is_some() && a == c {
            return a;
        }
        if b.is_some() && b == c {
            return b;
        }
        None
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper: create a tracker with onset gating disabled for simpler tests.
    fn tracker_no_onset() -> NoteTracker {
        NoteTracker::new(NoteTrackerConfig {
            onset_required: false,
            ..Default::default()
        })
    }

    /// Helper: create a tracker with onset gating enabled (default).
    fn tracker_with_onset() -> NoteTracker {
        NoteTracker::new(NoteTrackerConfig::default())
    }

    // =========================================================================
    // T1: Octave error rejection
    // =========================================================================

    #[test]
    fn octave_error_rejected_when_confidence_low() {
        let mut t = tracker_no_onset();

        // Establish a note at MIDI 60 (middle C) -- need 2/3 agreement.
        assert_eq!(t.update(60, 0.9), None);
        assert_eq!(t.update(60, 0.9), Some(NoteEvent::NoteOn(60)));

        // Now attempt +12 jump with LOW confidence -- should be rejected.
        assert_eq!(t.update(72, 0.3), None);
        assert_eq!(t.update(72, 0.3), None);

        // Current note should still be 60.
        assert_eq!(t.current_note(), Some(60));
    }

    #[test]
    fn octave_error_rejected_minus_12_low_confidence() {
        let mut t = tracker_no_onset();
        assert_eq!(t.update(72, 0.9), None);
        assert_eq!(t.update(72, 0.9), Some(NoteEvent::NoteOn(72)));

        // -12 jump with low confidence.
        assert_eq!(t.update(60, 0.4), None);
        assert_eq!(t.update(60, 0.4), None);
        assert_eq!(t.current_note(), Some(72));
    }

    #[test]
    fn octave_change_accepted_when_confidence_high() {
        let mut t = tracker_no_onset();
        assert_eq!(t.update(60, 0.9), None);
        assert_eq!(t.update(60, 0.9), Some(NoteEvent::NoteOn(60)));

        // +12 jump with HIGH confidence should be allowed.
        // First 72 -> buffer [60, 60, 72]: majority is still 60 (same as current).
        assert_eq!(t.update(72, 0.8), None);
        // Second 72 -> buffer [72, 60, 72]: a==c -> majority is 72 -> NoteOn.
        assert_eq!(t.update(72, 0.8), Some(NoteEvent::NoteOn(72)));

        assert_eq!(t.current_note(), Some(72));
    }

    #[test]
    fn non_octave_interval_not_affected_by_low_confidence() {
        let mut t = tracker_no_onset();
        assert_eq!(t.update(60, 0.9), None);
        assert_eq!(t.update(60, 0.9), Some(NoteEvent::NoteOn(60)));

        // +7 semitones (perfect fifth) with LOW confidence -- should NOT be
        // rejected by the octave filter.
        assert_eq!(t.update(67, 0.3), None); // vote_buf [60, 60, 67] -> a==b=60 same as current -> None
        assert_eq!(t.update(67, 0.3), Some(NoteEvent::NoteOn(67))); // [67, 60, 67] -> a==c=67 -> new note
    }

    // =========================================================================
    // T3: Pitch-class voting (3-frame median)
    // =========================================================================

    #[test]
    fn voting_requires_two_of_three_agreement() {
        let mut t = tracker_no_onset();

        // Three completely different notes -- no consensus.
        assert_eq!(t.update(60, 0.9), None);
        assert_eq!(t.update(64, 0.9), None);
        assert_eq!(t.update(67, 0.9), None);

        // Overwrites slot 0 with 60 -> [60, 64, 67]: still no pair.
        assert_eq!(t.update(60, 0.9), None);
        // Overwrites slot 1 with 60 -> [60, 60, 67]: a==b -> NoteOn(60).
        assert_eq!(t.update(60, 0.9), Some(NoteEvent::NoteOn(60)));
    }

    #[test]
    fn voting_single_glitch_does_not_trigger_note() {
        let mut t = tracker_no_onset();

        // Stable note.
        assert_eq!(t.update(60, 0.9), None);
        assert_eq!(t.update(60, 0.9), Some(NoteEvent::NoteOn(60)));

        // One glitch to 65 -- majority still 60 (same as current).
        assert_eq!(t.update(65, 0.9), None);

        // Back to 60 -- majority still 60, no change.
        assert_eq!(t.update(60, 0.9), None);

        // Still on 60 -- the glitch was absorbed.
        assert_eq!(t.current_note(), Some(60));
    }

    // =========================================================================
    // T5: Onset gating
    // =========================================================================

    #[test]
    fn onset_gating_prevents_notes_without_onset() {
        let mut t = tracker_with_onset();

        // Feed pitches without signaling onset.
        assert_eq!(t.update(60, 0.9), None);
        assert_eq!(t.update(60, 0.9), None); // would get 2/3 agreement, but gate closed
        assert_eq!(t.update(60, 0.9), None);

        // No note should have been emitted.
        assert_eq!(t.current_note(), None);
    }

    #[test]
    fn onset_gating_allows_notes_after_signal() {
        let mut t = tracker_with_onset();

        // Signal onset, then provide pitch detections.
        t.signal_onset();
        assert!(t.is_onset_armed());

        assert_eq!(t.update(60, 0.9), None); // 1st frame, no consensus yet
        assert_eq!(t.update(60, 0.9), Some(NoteEvent::NoteOn(60))); // 2/3 agree + onset armed

        // Gate should now be disarmed.
        assert!(!t.is_onset_armed());
        assert_eq!(t.current_note(), Some(60));
    }

    #[test]
    fn onset_gate_rearms_after_note() {
        let mut t = tracker_with_onset();

        // First note.
        t.signal_onset();
        assert_eq!(t.update(60, 0.9), None);
        assert_eq!(t.update(60, 0.9), Some(NoteEvent::NoteOn(60)));

        // Without a new onset signal, a different note should not fire.
        // Clear buffer via note_off so 60 doesn't linger.
        t.note_off();
        assert_eq!(t.update(64, 0.9), None);
        assert_eq!(t.update(64, 0.9), None); // 2/3 agree, but gate closed
        assert_eq!(t.current_note(), None);

        // Signal another onset.
        t.signal_onset();
        assert_eq!(t.update(64, 0.9), Some(NoteEvent::NoteOn(64)));
        assert_eq!(t.current_note(), Some(64));
    }

    #[test]
    fn note_off_works_without_onset() {
        let mut t = tracker_with_onset();

        // Get a note going.
        t.signal_onset();
        t.update(60, 0.9);
        t.update(60, 0.9);
        assert_eq!(t.current_note(), Some(60));

        // NoteOff should work even though onset is not armed.
        assert!(!t.is_onset_armed());
        assert_eq!(t.note_off(), Some(NoteEvent::NoteOff(60)));
        assert_eq!(t.current_note(), None);
    }

    #[test]
    fn onset_gating_disabled_via_config() {
        let mut t = NoteTracker::new(NoteTrackerConfig {
            onset_required: false,
            ..Default::default()
        });

        // Should work without any onset signal.
        assert_eq!(t.update(60, 0.9), None);
        assert_eq!(t.update(60, 0.9), Some(NoteEvent::NoteOn(60)));
    }

    // =========================================================================
    // Integration: all three features together
    // =========================================================================

    #[test]
    fn combined_octave_reject_and_onset_gate() {
        let mut t = tracker_with_onset();

        // Establish note 60.
        t.signal_onset();
        t.update(60, 0.9);
        t.update(60, 0.9);
        assert_eq!(t.current_note(), Some(60));

        // Attempt octave jump with low confidence + onset armed.
        t.signal_onset();
        assert_eq!(t.update(72, 0.3), None); // rejected by T1
        assert_eq!(t.update(72, 0.3), None); // still rejected
        assert_eq!(t.current_note(), Some(60));
    }

    #[test]
    fn note_off_clears_voting_buffer() {
        let mut t = tracker_no_onset();

        t.update(60, 0.9);
        t.update(60, 0.9);
        assert_eq!(t.current_note(), Some(60));

        t.note_off();

        // After note_off, the buffer is cleared. A single detection should not
        // emit (needs 2/3).
        assert_eq!(t.update(64, 0.9), None);
        assert_eq!(t.current_note(), None);

        // Second detection -> 2/3 agree.
        assert_eq!(t.update(64, 0.9), Some(NoteEvent::NoteOn(64)));
    }
}
