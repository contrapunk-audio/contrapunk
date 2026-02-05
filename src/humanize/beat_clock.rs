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
//! // Check if current position is off-beat (for swing)
//! if clock.is_offbeat() {
//!     // Apply swing delay
//! }
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
    pub fn is_offbeat(&self) -> bool {
        let frac = self.beat_position.fract();
        frac >= 0.4 && frac <= 0.6
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

    /// Update tempo and time signature without resetting position.
    pub fn update_tempo(&mut self, bpm: f64, beats_per_bar: u8, beat_unit: u8) {
        self.bpm = bpm;
        self.beats_per_bar = beats_per_bar;
        self.beat_unit = beat_unit;
    }
}
