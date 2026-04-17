//! Musical beat clock for tempo and time tracking.
//!
//! The [`BeatClock`] tracks musical time based on BPM and elapsed time,
//! providing beat position for groove calculations and metronome synchronization.
//!
//! # Usage
//!
//! ```ignore
//! use contrapunk::humanize::BeatClock;
//!
//! let mut clock = BeatClock::new(120.0, 4, 4); // 120 BPM, 4/4 time
//! clock.start(current_time_ms);
//!
//! // On each frame:
//! let beat_pos = clock.tick(current_time_ms);
//!
//! // Check for beat boundaries (useful for metronome)
//! if let Some(beat_num) = clock.beat_crossed() {
//!     // Beat boundary crossed! Play click on beat_num
//! }
//!
//! // Subdivision-level crossing detection (for metronome subdivisions)
//! if let Some(crossing) = clock.subdivision_crossed() {
//!     // A subdivision boundary was crossed
//! }
//!
//! // Get the current subdivision phase (for swing)
//! let phase = clock.subdivision_phase();
//! // phase.beat = which beat we're on (0-indexed)
//! // phase.eighth = whether we're on the 8th-note offbeat
//! // phase.sixteenth = which 16th-note slot (0-3)
//! ```
//!
//! # Beat Position
//!
//! The beat position is a floating-point value representing the current
//! position within the bar:
//!
//! - `0.0` = Start of bar (beat 1)
//! - `1.0` = Beat 2
//! - `2.5` = Halfway between beat 3 and beat 4
//! - `3.99` = Just before bar wrap-around
//!
//! # Cross-Platform
//!
//! Uses `f64` milliseconds for time input, which works on both native
//! (using `std::time::Instant`) and WASM (using `performance.now()`).

/// Which subdivision slot the current beat position falls in.
///
/// For a beat at position 2.75:
/// - `beat = 2` (third beat, 0-indexed)
/// - `eighth = true` (we're past the midpoint of the beat)
/// - `sixteenth = 3` (we're in the last 16th-note slot)
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SubdivisionPhase {
    /// Which beat we're on (0-indexed within the bar).
    pub beat: u8,
    /// Whether we're on the 8th-note offbeat (past 0.5 of the beat).
    pub eighth: bool,
    /// Which 16th-note slot within the beat (0-3).
    pub sixteenth: u8,
}

/// A subdivision boundary crossing detected between ticks.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SubdivisionCrossing {
    /// Beat number (0-indexed within bar).
    pub beat: u8,
    /// Which 16th-note slot was entered (0-3).
    /// 0 = beat downbeat, 1 = first 16th, 2 = 8th offbeat, 3 = third 16th.
    pub sixteenth: u8,
}

/// Tracks beat position from absolute elapsed time.
///
/// Uses f64 milliseconds for time so it works on both native and WASM.
/// The clock calculates beat position based on BPM and elapsed time since start.
///
/// # Fields
///
/// - `bpm`: Tempo in beats per minute
/// - `beats_per_bar`: Time signature numerator (e.g., 4 in 4/4)
/// - `beat_unit`: Time signature denominator (e.g., 4 in 4/4)
/// - `running`: Whether the clock is currently advancing
///
/// # Beat Calculation
///
/// ```text
/// elapsed_seconds = (now_ms - start_time_ms) / 1000
/// total_beats = elapsed_seconds * bpm / 60
/// beat_position = total_beats % beats_per_bar
/// ```
#[derive(Clone, Debug)]
pub struct BeatClock {
    /// Tempo in beats per minute.
    pub bpm: f64,
    /// Time signature numerator (beats per bar).
    pub beats_per_bar: u8,
    /// Time signature denominator (beat unit).
    pub beat_unit: u8,
    /// Whether the clock is running.
    pub running: bool,
    /// Timestamp when clock was started (milliseconds).
    start_time_ms: f64,
    /// Current beat position within the bar (0.0 to beats_per_bar).
    beat_position: f64,
    /// Previous beat position (for detecting beat crossings).
    prev_beat_position: f64,
}

impl BeatClock {
    pub fn new(bpm: f64, beats_per_bar: u8, beat_unit: u8) -> Self {
        Self {
            bpm,
            beats_per_bar,
            beat_unit,
            running: false,
            start_time_ms: 0.0,
            beat_position: 0.0,
            prev_beat_position: 0.0,
        }
    }

    /// Start the clock. `now_ms` is the current time in milliseconds.
    pub fn start(&mut self, now_ms: f64) {
        self.start_time_ms = now_ms;
        self.running = true;
        self.beat_position = 0.0;
        self.prev_beat_position = 0.0;
    }

    /// Stop the clock.
    pub fn stop(&mut self) {
        self.running = false;
    }

    /// Tick the clock and return the current beat position within the bar.
    pub fn tick(&mut self, now_ms: f64) -> f64 {
        if !self.running {
            return self.beat_position;
        }
        self.prev_beat_position = self.beat_position;
        let elapsed_secs = (now_ms - self.start_time_ms) / 1000.0;
        self.beat_position = (elapsed_secs * self.bpm / 60.0) % self.beats_per_bar as f64;
        self.beat_position
    }

    /// Returns the current beat position within the bar.
    pub fn beat_position(&self) -> f64 {
        self.beat_position
    }

    /// Returns true if the current position is on an offbeat (fractional part between 0.4 and 0.6).
    ///
    /// Deprecated: use `subdivision_phase()` for finer-grained swing control.
    pub fn is_offbeat(&self) -> bool {
        let frac = self.beat_position.fract();
        frac >= 0.4 && frac <= 0.6
    }

    /// Returns the current subdivision phase within the beat.
    ///
    /// Decomposes the fractional beat position into beat number, 8th-note
    /// offbeat flag, and 16th-note slot (0-3).
    pub fn subdivision_phase(&self) -> SubdivisionPhase {
        let beat = self.beat_position.floor() as u8;
        let frac = self.beat_position.fract();
        // 16th-note slot: 0.00-0.25 = slot 0, 0.25-0.50 = slot 1,
        //                 0.50-0.75 = slot 2, 0.75-1.00 = slot 3
        let sixteenth = (frac * 4.0).floor() as u8;
        let eighth = frac >= 0.5;
        SubdivisionPhase {
            beat,
            eighth,
            sixteenth: sixteenth.min(3),
        }
    }

    /// If a beat boundary was crossed since last tick, return the beat number (0-indexed).
    pub fn beat_crossed(&self) -> Option<u8> {
        let prev_floor = self.prev_beat_position.floor() as i64;
        let curr_floor = self.beat_position.floor() as i64;
        if prev_floor != curr_floor {
            Some((curr_floor % self.beats_per_bar as i64) as u8)
        } else {
            None
        }
    }

    /// If a subdivision boundary (16th-note level) was crossed since last
    /// tick, return the crossing info.
    ///
    /// Returns crossings for all four 16th-note slots within each beat:
    /// - slot 0 = beat downbeat
    /// - slot 1 = first 16th (e- subdivision)
    /// - slot 2 = 8th-note offbeat
    /// - slot 3 = third 16th (a- subdivision)
    ///
    /// When the beat wraps around (bar boundary), the crossing is reported
    /// for beat 0, slot 0.
    pub fn subdivision_crossed(&self) -> Option<SubdivisionCrossing> {
        // Convert to 16th-note resolution: each beat has 4 slots.
        let total_16ths_per_bar = self.beats_per_bar as f64 * 4.0;

        let prev_slot = (self.prev_beat_position * 4.0).floor() as i64;
        let curr_slot = (self.beat_position * 4.0).floor() as i64;

        if prev_slot != curr_slot {
            // Normalize to bar range
            let slot = ((curr_slot % total_16ths_per_bar as i64) + total_16ths_per_bar as i64)
                % total_16ths_per_bar as i64;
            let beat = (slot / 4) as u8;
            let sixteenth = (slot % 4) as u8;
            Some(SubdivisionCrossing { beat, sixteenth })
        } else {
            None
        }
    }

    /// Update tempo and time signature without resetting position.
    pub fn update_tempo(&mut self, bpm: f64, beats_per_bar: u8, beat_unit: u8) {
        self.bpm = bpm;
        self.beats_per_bar = beats_per_bar;
        self.beat_unit = beat_unit;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn subdivision_phase_at_beat_start() {
        let mut clock = BeatClock::new(120.0, 4, 4);
        clock.start(0.0);
        clock.tick(0.0);
        let phase = clock.subdivision_phase();
        assert_eq!(phase.beat, 0);
        assert!(!phase.eighth);
        assert_eq!(phase.sixteenth, 0);
    }

    #[test]
    fn subdivision_phase_at_offbeat() {
        let mut clock = BeatClock::new(120.0, 4, 4);
        clock.start(0.0);
        // At 120 BPM, one beat = 500ms. 0.5 beat = 250ms.
        clock.tick(250.0);
        let phase = clock.subdivision_phase();
        assert_eq!(phase.beat, 0);
        assert!(phase.eighth);
        assert_eq!(phase.sixteenth, 2);
    }

    #[test]
    fn subdivision_phase_16th_slots() {
        let mut clock = BeatClock::new(120.0, 4, 4);
        clock.start(0.0);
        // At 120 BPM, one beat = 500ms. One 16th = 125ms.
        // Slot 0: 0-125ms
        clock.tick(60.0);
        assert_eq!(clock.subdivision_phase().sixteenth, 0);
        // Slot 1: 125-250ms
        clock.tick(130.0);
        assert_eq!(clock.subdivision_phase().sixteenth, 1);
        // Slot 2: 250-375ms
        clock.tick(260.0);
        assert_eq!(clock.subdivision_phase().sixteenth, 2);
        // Slot 3: 375-500ms
        clock.tick(400.0);
        assert_eq!(clock.subdivision_phase().sixteenth, 3);
    }

    #[test]
    fn subdivision_crossed_detects_16th() {
        let mut clock = BeatClock::new(120.0, 4, 4);
        clock.start(0.0);
        // First tick at 0 — no previous crossing
        clock.tick(0.0);
        // Tick just past the first 16th boundary (125ms at 120BPM)
        let _ = clock.tick(130.0);
        let crossing = clock.subdivision_crossed();
        assert!(crossing.is_some());
        let c = crossing.unwrap();
        assert_eq!(c.beat, 0);
        assert_eq!(c.sixteenth, 1);
    }

    #[test]
    fn subdivision_crossed_none_within_slot() {
        let mut clock = BeatClock::new(120.0, 4, 4);
        clock.start(0.0);
        clock.tick(10.0);
        let _ = clock.tick(20.0);
        // Both within slot 0 — no crossing
        assert!(clock.subdivision_crossed().is_none());
    }

    #[test]
    fn beat_crossed_still_works() {
        let mut clock = BeatClock::new(120.0, 4, 4);
        clock.start(0.0);
        clock.tick(0.0);
        // Move past beat 1 (500ms at 120BPM)
        clock.tick(510.0);
        assert_eq!(clock.beat_crossed(), Some(1));
    }
}
