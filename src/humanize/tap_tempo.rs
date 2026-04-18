//! Tap-tempo BPM detection.
//!
//! The user taps a button rhythmically and the BPM is calculated from
//! the average interval between taps.
//!
//! # Algorithm
//!
//! 1. Record each tap timestamp (milliseconds).
//! 2. If the gap since the last tap exceeds `timeout_ms`, reset.
//! 3. Keep the last `max_taps` timestamps.
//! 4. Once there are at least 2 taps, compute BPM from the average interval:
//!    `BPM = 60000 / average_interval_ms`.
//!
//! # Example
//!
//! ```
//! use contrapunk::humanize::TapTempo;
//!
//! let mut tap = TapTempo::new();
//! assert!(tap.tap(0.0).is_none());      // first tap, no interval yet
//! assert_eq!(tap.tap(500.0), Some(120.0)); // 500ms interval = 120 BPM
//! ```

/// Computes BPM from rhythmic taps.
pub struct TapTempo {
    /// Recorded tap timestamps in milliseconds.
    taps: Vec<f64>,
    /// Maximum number of taps to keep for averaging.
    max_taps: usize,
    /// If the gap between two consecutive taps exceeds this, reset.
    timeout_ms: f64,
}

impl TapTempo {
    /// Create a new TapTempo with default settings (4 taps, 2000ms timeout).
    pub fn new() -> Self {
        Self {
            taps: Vec::new(),
            max_taps: 4,
            timeout_ms: 2000.0,
        }
    }

    /// Record a tap at the given timestamp (milliseconds).
    ///
    /// Returns the computed BPM if there are at least 2 taps.
    /// Resets the tap buffer if the gap since the last tap exceeds the timeout.
    pub fn tap(&mut self, now_ms: f64) -> Option<f64> {
        // Reset if the gap since the last tap exceeds the timeout.
        if let Some(&last) = self.taps.last() {
            if now_ms - last > self.timeout_ms {
                self.taps.clear();
            }
        }

        self.taps.push(now_ms);

        // Keep only the last max_taps timestamps.
        if self.taps.len() > self.max_taps {
            let excess = self.taps.len() - self.max_taps;
            self.taps.drain(..excess);
        }

        // Need at least 2 taps to compute an interval.
        if self.taps.len() < 2 {
            return None;
        }

        // Average interval across all consecutive pairs.
        let total_span = self.taps.last().unwrap() - self.taps.first().unwrap();
        let num_intervals = (self.taps.len() - 1) as f64;
        let avg_interval = total_span / num_intervals;

        if avg_interval <= 0.0 {
            return None;
        }

        Some(60_000.0 / avg_interval)
    }

    /// Clear all recorded taps.
    pub fn reset(&mut self) {
        self.taps.clear();
    }
}

impl Default for TapTempo {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn four_taps_at_500ms_gives_120_bpm() {
        let mut tap = TapTempo::new();
        assert!(tap.tap(0.0).is_none());
        assert_eq!(tap.tap(500.0), Some(120.0));
        assert_eq!(tap.tap(1000.0), Some(120.0));
        assert_eq!(tap.tap(1500.0), Some(120.0));
    }

    #[test]
    fn timeout_resets_after_2s_gap() {
        let mut tap = TapTempo::new();
        tap.tap(0.0);
        tap.tap(500.0);
        // Gap of 3000ms > timeout (2000ms) -> reset
        assert!(tap.tap(3500.0).is_none());
        // Now the next tap should give a result from the new sequence
        assert_eq!(tap.tap(4000.0), Some(120.0));
    }

    #[test]
    fn fewer_than_2_taps_returns_none() {
        let mut tap = TapTempo::new();
        assert!(tap.tap(0.0).is_none());
    }

    #[test]
    fn reset_clears_taps() {
        let mut tap = TapTempo::new();
        tap.tap(0.0);
        tap.tap(500.0);
        tap.reset();
        assert!(tap.tap(1000.0).is_none());
    }

    #[test]
    fn max_taps_sliding_window() {
        let mut tap = TapTempo::new();
        // max_taps = 4, so after 5 taps, oldest is dropped
        tap.tap(0.0);
        tap.tap(500.0);
        tap.tap(1000.0);
        tap.tap(1500.0);
        // 5th tap: oldest (0.0) is dropped, window is [500, 1000, 1500, 2000]
        let bpm = tap.tap(2000.0).unwrap();
        assert_eq!(bpm, 120.0);
    }

    #[test]
    fn varying_intervals() {
        let mut tap = TapTempo::new();
        tap.tap(0.0);
        // 400ms + 600ms = 1000ms total, 2 intervals, avg = 500ms = 120 BPM
        tap.tap(400.0);
        let bpm = tap.tap(1000.0).unwrap();
        assert_eq!(bpm, 120.0);
    }
}
