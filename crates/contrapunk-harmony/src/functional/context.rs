//! HarmonicContext state machine for functional harmony modes.
//!
//! Tracks the current harmonic state (current/previous chord, cadence phase)
//! and determines when chord changes should occur.
//!
//! WASM-safe: uses note count for harmonic rhythm, not `std::time::Instant`.

use super::chord_table::ChordTable;
use crate::config::ScaleMode;

/// Cadence detection state machine.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CadencePhase {
    /// No cadence in progress.
    None,
    /// Subdominant/pre-dominant function detected -- approaching dominant.
    Approaching,
    /// Dominant (V) arrived -- expecting tonic resolution.
    DominantArrived,
}

/// Harmonic context shared by FunctionalHarmony and BachChorale modes.
///
/// This is the core state object that tracks which chord we are on,
/// what came before, and whether a cadence is approaching.
#[derive(Clone, Debug)]
pub struct HarmonicContext {
    /// Current chord degree (0-6 = I-vii), None if no chord yet.
    pub current_degree: Option<u8>,
    /// Previous chord degree.
    pub prev_degree: Option<u8>,
    /// Beat position within a 4/4 bar (0-3). Defaults to 0.
    /// Can be wired to Performance Mode beat tracking in the future.
    pub beat_position: u8,
    /// Cadence detection state.
    pub cadence_phase: CadencePhase,
    /// Previous voicing as MIDI note values [soprano, alto, tenor, bass].
    /// None if this is the first chord.
    pub prev_voicing: Option<[u8; 4]>,
    /// Precomputed chord membership table.
    pub chord_table: ChordTable,
    /// Number of consecutive chromatic (non-diatonic) notes.
    pub consecutive_chromatic: u8,
    /// Notes processed since last chord change (for harmonic rhythm).
    pub notes_since_last_change: u16,
    /// Total notes processed (for beat position approximation).
    pub total_notes: u64,
}

/// Minimum notes between chord changes to avoid frantic harmony.
const MIN_CHORD_CHANGE_INTERVAL: u16 = 2;

impl HarmonicContext {
    /// Create a new context for the given tonic and scale mode.
    ///
    /// # Panics
    ///
    /// Panics if scale_mode is not a 7-note scale.
    pub fn new(tonic: u8, scale_mode: ScaleMode) -> Self {
        let chord_table = ChordTable::build(tonic, scale_mode);
        Self {
            current_degree: None,
            prev_degree: None,
            beat_position: 0,
            cadence_phase: CadencePhase::None,
            prev_voicing: None,
            chord_table,
            consecutive_chromatic: 0,
            notes_since_last_change: 0,
            total_notes: 0,
        }
    }

    /// Full reset (called on mode change).
    pub fn reset_full(&mut self) {
        self.current_degree = None;
        self.prev_degree = None;
        self.beat_position = 0;
        self.cadence_phase = CadencePhase::None;
        self.prev_voicing = None;
        self.consecutive_chromatic = 0;
        self.notes_since_last_change = 0;
        self.total_notes = 0;
    }

    /// Key/scale change reset. Rebuilds chord table, clears degree/cadence,
    /// but keeps prev_voicing for smooth voice leading across key changes.
    pub fn reset_key(&mut self, tonic: u8, scale_mode: ScaleMode) {
        self.chord_table = ChordTable::build(tonic, scale_mode);
        self.current_degree = None;
        self.prev_degree = None;
        self.cadence_phase = CadencePhase::None;
        self.consecutive_chromatic = 0;
        self.notes_since_last_change = 0;
    }

    /// Returns whether a chord change is permitted right now.
    ///
    /// Uses note count as the harmonic rhythm governor.
    /// First note always triggers a chord; subsequent changes require
    /// at least MIN_CHORD_CHANGE_INTERVAL notes.
    pub fn should_change_chord(&self) -> bool {
        if self.current_degree.is_none() {
            return true;
        }
        self.notes_since_last_change >= MIN_CHORD_CHANGE_INTERVAL
    }

    /// Record that a chord change occurred to the given degree.
    pub fn change_chord(&mut self, new_degree: u8) {
        self.prev_degree = self.current_degree;
        self.current_degree = Some(new_degree);
        self.notes_since_last_change = 0;

        // Update cadence state machine
        self.update_cadence(new_degree);
    }

    /// Advance the note counter and beat position.
    pub fn advance_note(&mut self) {
        self.notes_since_last_change = self.notes_since_last_change.saturating_add(1);
        self.total_notes = self.total_notes.wrapping_add(1);
        // Approximate beat position from note count (4 notes per bar)
        self.beat_position = (self.total_notes % 4) as u8;
    }

    /// Update cadence detection based on the new chord degree.
    fn update_cadence(&mut self, new_degree: u8) {
        use super::chord_table::HarmonicFunction;

        let func = self.chord_table.function[new_degree as usize];
        let prev_func = self
            .prev_degree
            .map(|d| self.chord_table.function[d as usize]);

        self.cadence_phase = match self.cadence_phase {
            CadencePhase::None => {
                if func == HarmonicFunction::Subdominant
                    && (self.beat_position == 0 || self.beat_position == 2)
                {
                    CadencePhase::Approaching
                } else if func == HarmonicFunction::Dominant {
                    CadencePhase::DominantArrived
                } else {
                    CadencePhase::None
                }
            }
            CadencePhase::Approaching => {
                if func == HarmonicFunction::Dominant {
                    CadencePhase::DominantArrived
                } else if func == HarmonicFunction::Tonic {
                    // Plagal cadence (IV -> I)
                    CadencePhase::None
                } else {
                    CadencePhase::None // Abandoned
                }
            }
            CadencePhase::DominantArrived => {
                if func == HarmonicFunction::Tonic {
                    // Authentic cadence resolved -- reset
                    CadencePhase::None
                } else {
                    // Abandoned cadence
                    CadencePhase::None
                }
            }
        };

        // Also detect half cadence: arrival on V from subdominant
        if func == HarmonicFunction::Dominant {
            if let Some(pf) = prev_func {
                if pf == HarmonicFunction::Subdominant {
                    self.cadence_phase = CadencePhase::DominantArrived;
                }
            }
        }
    }

    /// Returns whether the given scale mode is compatible with functional harmony.
    /// Only 7-note scales are supported.
    pub fn is_compatible_scale(scale_mode: ScaleMode) -> bool {
        scale_mode.intervals().len() == 7
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_context() {
        let ctx = HarmonicContext::new(0, ScaleMode::Ionian);
        assert_eq!(ctx.current_degree, None);
        assert_eq!(ctx.prev_degree, None);
        assert_eq!(ctx.cadence_phase, CadencePhase::None);
        assert!(ctx.should_change_chord()); // First note always can change
    }

    #[test]
    fn test_chord_change_interval() {
        let mut ctx = HarmonicContext::new(0, ScaleMode::Ionian);

        // First chord -- always allowed
        assert!(ctx.should_change_chord());
        ctx.change_chord(0); // I
        ctx.advance_note();

        // After 1 note -- not yet
        assert!(!ctx.should_change_chord());
        ctx.advance_note();

        // After 2 notes -- allowed
        assert!(ctx.should_change_chord());
    }

    #[test]
    fn test_cadence_detection_authentic() {
        let mut ctx = HarmonicContext::new(0, ScaleMode::Ionian);
        ctx.beat_position = 0; // Strong beat

        // IV (subdominant) on strong beat -> Approaching
        ctx.change_chord(3); // IV
        assert_eq!(ctx.cadence_phase, CadencePhase::Approaching);

        // V (dominant) -> DominantArrived
        ctx.change_chord(4); // V
        assert_eq!(ctx.cadence_phase, CadencePhase::DominantArrived);

        // I (tonic) -> resolved (None)
        ctx.change_chord(0); // I
        assert_eq!(ctx.cadence_phase, CadencePhase::None);
    }

    #[test]
    fn test_reset_full() {
        let mut ctx = HarmonicContext::new(0, ScaleMode::Ionian);
        ctx.change_chord(4);
        ctx.advance_note();
        ctx.advance_note();
        ctx.advance_note();

        ctx.reset_full();
        assert_eq!(ctx.current_degree, None);
        assert_eq!(ctx.total_notes, 0);
        assert_eq!(ctx.cadence_phase, CadencePhase::None);
    }

    #[test]
    fn test_reset_key() {
        let mut ctx = HarmonicContext::new(0, ScaleMode::Ionian);
        ctx.change_chord(0);
        ctx.prev_voicing = Some([72, 64, 60, 48]);

        ctx.reset_key(7, ScaleMode::Ionian); // G major
        assert_eq!(ctx.current_degree, None);
        // prev_voicing is preserved for smooth voice leading
        assert!(ctx.prev_voicing.is_some());
        // But chord table is rebuilt
        assert_eq!(ctx.chord_table.degree_pcs[0], 7); // G
    }

    #[test]
    fn test_compatible_scale() {
        assert!(HarmonicContext::is_compatible_scale(ScaleMode::Ionian));
        assert!(HarmonicContext::is_compatible_scale(ScaleMode::Aeolian));
        assert!(HarmonicContext::is_compatible_scale(ScaleMode::Dorian));
        assert!(!HarmonicContext::is_compatible_scale(
            ScaleMode::BHMajor6thDim
        ));
        assert!(!HarmonicContext::is_compatible_scale(
            ScaleMode::BHMinor6thDim
        ));
    }

    #[test]
    fn test_beat_position_cycles() {
        let mut ctx = HarmonicContext::new(0, ScaleMode::Ionian);
        for i in 0..8 {
            ctx.advance_note();
            assert_eq!(ctx.beat_position, ((i + 1) % 4) as u8);
        }
    }
}
