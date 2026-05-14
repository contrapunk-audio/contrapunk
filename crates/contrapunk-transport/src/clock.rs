//! Transport primitive: sample-accurate clock driven by an audio callback.
//!
//! # Model
//!
//! - `sample_pos` — total samples elapsed since the transport was started.
//!   Only advanced by `advance()`, which is intended to be called from
//!   the audio thread's callback.
//! - `bpm` — tempo in beats per minute, stored as fixed-point (× 1000)
//!   in an atomic so it can be updated from any thread without locks.
//! - `running` — whether the clock is currently advancing. When false,
//!   `advance()` is a no-op even if called; sample_pos stays put.
//! - `time_signature` — `(beats_per_bar, beat_unit)`, atomic.
//!
//! # Threading
//!
//! All state is in atomics. Any thread can read or write; audio callback
//! never blocks. Readers see relaxed-ordering values, which is fine for
//! display — the audio thread's own reads (for `advance`) use the same.

use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, AtomicU8, Ordering};
use std::sync::Arc;

/// A single beat-boundary crossing detected by `advance()`.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct BeatCrossing {
    /// Absolute sample position at which this beat starts (first sample of
    /// the beat, may be slightly inside the current block — approximated
    /// to the start of the block containing the crossing).
    pub sample_pos: u64,
    /// Monotonically-increasing total beat index since transport start.
    pub total_beat: u64,
    /// Beat index within the current bar, 0-based.
    pub beat_in_bar: u8,
    /// Bar index, 0-based since transport start.
    pub bar: u64,
}

/// Sample-accurate transport clock.
///
/// See module docs for the threading model. Clone the `Arc<Transport>` to
/// share between the audio thread, UI thread, and router thread.
pub struct Transport {
    /// Samples elapsed since the clock started advancing.
    sample_pos: AtomicU64,
    /// Audio sample rate (Hz). Atomic so the audio-clock startup path
    /// can correct it once the cpal stream's actual rate is known.
    /// Do NOT change after transport has been playing — sample_pos
    /// was measured in the old rate.
    sample_rate: AtomicU32,
    /// Current tempo in milli-BPM (BPM × 1000) so we can use AtomicU32.
    bpm_milli: AtomicU32,
    /// Whether the clock is advancing.
    running: AtomicBool,
    /// Time signature numerator.
    beats_per_bar: AtomicU8,
    /// Time signature denominator.
    beat_unit: AtomicU8,
    /// Last observed `total_beat` the advance path reported via a
    /// crossing. Stored as AtomicU64 so `advance()` can detect crossings
    /// monotonically without locks.
    last_crossed_beat: AtomicU64,
}

impl Transport {
    /// Create a stopped transport at sample position 0, tempo 120 BPM, 4/4.
    pub fn new(sample_rate: u32) -> Arc<Self> {
        Arc::new(Self {
            sample_pos: AtomicU64::new(0),
            sample_rate: AtomicU32::new(sample_rate),
            bpm_milli: AtomicU32::new(120_000),
            running: AtomicBool::new(false),
            beats_per_bar: AtomicU8::new(4),
            beat_unit: AtomicU8::new(4),
            last_crossed_beat: AtomicU64::new(u64::MAX),
        })
    }

    // ─── Readers ──────────────────────────────────────────────────

    pub fn sample_rate(&self) -> u32 {
        self.sample_rate.load(Ordering::Relaxed)
    }

    /// Update the sample rate. Should only be called at startup, before
    /// the transport has advanced — otherwise past sample_pos values
    /// become meaningless.
    pub fn set_sample_rate(&self, sr: u32) {
        self.sample_rate.store(sr, Ordering::Relaxed);
    }

    pub fn sample_pos(&self) -> u64 {
        self.sample_pos.load(Ordering::Relaxed)
    }

    pub fn bpm(&self) -> f64 {
        self.bpm_milli.load(Ordering::Relaxed) as f64 / 1000.0
    }

    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::Relaxed)
    }

    pub fn time_signature(&self) -> (u8, u8) {
        (
            self.beats_per_bar.load(Ordering::Relaxed),
            self.beat_unit.load(Ordering::Relaxed),
        )
    }

    /// Current fractional beat position within the bar (0.0 .. beats_per_bar).
    pub fn beat_position(&self) -> f64 {
        let total = self.total_beats();
        let per_bar = self.beats_per_bar.load(Ordering::Relaxed) as f64;
        if per_bar == 0.0 {
            return 0.0;
        }
        total - (total / per_bar).floor() * per_bar
    }

    /// Monotonically-increasing total beats elapsed since transport start.
    pub fn total_beats(&self) -> f64 {
        let samples = self.sample_pos() as f64;
        let seconds = samples / self.sample_rate() as f64;
        seconds * self.bpm() / 60.0
    }

    /// Current bar (0-based) since transport start.
    pub fn bar(&self) -> u64 {
        let per_bar = self.beats_per_bar.load(Ordering::Relaxed) as u64;
        if per_bar == 0 {
            return 0;
        }
        (self.total_beats() as u64) / per_bar
    }

    // ─── Mutators (from command thread) ───────────────────────────

    /// Set tempo in BPM. Clamped to [20, 400]. Takes effect immediately.
    ///
    /// Also re-anchors `last_crossed_beat` to the beat-count implied
    /// by the current `sample_pos` under the NEW bpm. Without this
    /// re-anchor, lowering BPM mid-play made the metronome silently
    /// stop firing: the recomputed `current_beat` (under the new
    /// smaller bpm) was less than the old `last_crossed_beat`, so
    /// every subsequent `advance()` saw `current_beat <= last` and
    /// returned `None`. Raising BPM did the inverse — fired a
    /// retroactive burst of crossings to "catch up." Both are
    /// surprising behaviours; re-anchoring sidesteps both by
    /// declaring "the next crossing fires after the next beat in
    /// the NEW tempo, starting from where we are right now."
    pub fn set_bpm(&self, bpm: f64) {
        let clamped = bpm.clamp(20.0, 400.0);
        self.bpm_milli
            .store((clamped * 1000.0) as u32, Ordering::Relaxed);
        // Re-anchor the monotonic beat counter at the new tempo.
        let sample_pos = self.sample_pos.load(Ordering::Relaxed);
        let new_total_beats = (sample_pos as f64 / self.sample_rate() as f64) * (clamped / 60.0);
        let new_current_beat = new_total_beats.floor() as u64;
        self.last_crossed_beat
            .store(new_current_beat, Ordering::Relaxed);
    }

    pub fn set_time_signature(&self, beats_per_bar: u8, beat_unit: u8) {
        self.beats_per_bar
            .store(beats_per_bar.clamp(1, 32), Ordering::Relaxed);
        self.beat_unit
            .store(beat_unit.clamp(1, 64), Ordering::Relaxed);
    }

    /// Start the clock (resume from current sample_pos).
    pub fn play(&self) {
        self.running.store(true, Ordering::Relaxed);
    }

    /// Stop the clock (freeze sample_pos). `play()` resumes from here.
    pub fn stop(&self) {
        self.running.store(false, Ordering::Relaxed);
    }

    /// Reset sample_pos to 0. Does not change running state.
    pub fn reset(&self) {
        self.sample_pos.store(0, Ordering::Relaxed);
        self.last_crossed_beat.store(u64::MAX, Ordering::Relaxed);
    }

    /// Seek `sample_pos` to an absolute value. Re-anchors
    /// `last_crossed_beat` so beat-crossing detection doesn't fire
    /// a retroactive burst (going forward) or stall (going backward).
    /// Used by host-driven adapters (e.g. nih-plug plugins reading
    /// `ProcessContext::transport().pos_samples()`) to follow DAW
    /// loop / jump / locate events.
    pub fn set_sample_pos(&self, sample_pos: u64) {
        self.sample_pos.store(sample_pos, Ordering::Relaxed);
        let bpm = self.bpm();
        let total_beats = (sample_pos as f64 / self.sample_rate() as f64) * (bpm / 60.0);
        let current_beat = total_beats.floor() as u64;
        self.last_crossed_beat
            .store(current_beat, Ordering::Relaxed);
    }

    // ─── Audio-thread API ─────────────────────────────────────────

    /// Advance the clock by `frames` samples. Called from the audio
    /// callback once per block. Returns a `BeatCrossing` if a beat
    /// boundary was entered during this advance, otherwise `None`.
    ///
    /// If the transport is stopped, this is a no-op.
    ///
    /// If more than one beat boundary is crossed in a single call
    /// (possible at very high tempo or very large buffer), only the
    /// latest crossing is reported.
    pub fn advance(&self, frames: u32) -> Option<BeatCrossing> {
        if !self.running.load(Ordering::Relaxed) {
            return None;
        }

        let new_pos = self.sample_pos.fetch_add(frames as u64, Ordering::Relaxed) + frames as u64;
        // Compute the current monotonic beat count from the new position.
        let bpm = self.bpm();
        let total_beats = (new_pos as f64 / self.sample_rate() as f64) * (bpm / 60.0);
        let current_beat = total_beats.floor() as u64;

        let last = self.last_crossed_beat.load(Ordering::Relaxed);
        if last == u64::MAX || current_beat > last {
            self.last_crossed_beat
                .store(current_beat, Ordering::Relaxed);

            let per_bar = self.beats_per_bar.load(Ordering::Relaxed) as u64;
            let (bar, beat_in_bar) = if per_bar == 0 {
                (0u64, 0u8)
            } else {
                (current_beat / per_bar, (current_beat % per_bar) as u8)
            };

            return Some(BeatCrossing {
                sample_pos: new_pos,
                total_beat: current_beat,
                beat_in_bar,
                bar,
            });
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_transport_is_stopped_at_120_bpm() {
        let t = Transport::new(48_000);
        assert!(!t.is_running());
        assert_eq!(t.sample_pos(), 0);
        assert!((t.bpm() - 120.0).abs() < 1e-9);
        assert_eq!(t.time_signature(), (4, 4));
    }

    #[test]
    fn advance_is_noop_when_stopped() {
        let t = Transport::new(48_000);
        assert_eq!(t.advance(512), None);
        assert_eq!(t.sample_pos(), 0);
    }

    #[test]
    fn advance_increments_sample_pos_when_running() {
        let t = Transport::new(48_000);
        t.play();
        let _ = t.advance(512);
        assert_eq!(t.sample_pos(), 512);
        let _ = t.advance(512);
        assert_eq!(t.sample_pos(), 1024);
    }

    #[test]
    fn advance_reports_beat_crossings_at_120_bpm() {
        // 120 BPM → 1 beat per 500ms → 24000 samples at 48kHz.
        let t = Transport::new(48_000);
        t.play();

        // Advance past the first beat boundary.
        let c = t.advance(24_001).expect("should cross beat 0");
        assert_eq!(c.total_beat, 1, "first crossing is beat 1 (from 0)");
        assert_eq!(c.beat_in_bar, 1);
        assert_eq!(c.bar, 0);

        // Advance into beat 2 (but not past it).
        assert!(t.advance(1000).is_none());

        // Advance past beat 2.
        let c = t.advance(24_000).expect("should cross beat 2");
        assert_eq!(c.total_beat, 2);
        assert_eq!(c.beat_in_bar, 2);
    }

    #[test]
    fn beat_position_wraps_within_bar() {
        let t = Transport::new(48_000);
        t.play();
        // 4 beats per bar, so beat_position wraps at 4.0.
        t.advance(48_000 * 5); // 5 seconds @ 120 BPM = 10 beats = 2.5 bars
        let pos = t.beat_position();
        assert!((pos - 2.0).abs() < 0.01, "expected ~2.0, got {}", pos);
    }

    #[test]
    fn set_bpm_clamps() {
        let t = Transport::new(48_000);
        t.set_bpm(5.0);
        assert_eq!(t.bpm(), 20.0);
        t.set_bpm(1000.0);
        assert_eq!(t.bpm(), 400.0);
        t.set_bpm(140.0);
        assert!((t.bpm() - 140.0).abs() < 1e-9);
    }

    #[test]
    fn reset_zeroes_sample_pos() {
        let t = Transport::new(48_000);
        t.play();
        t.advance(48_000);
        assert!(t.sample_pos() > 0);
        t.reset();
        assert_eq!(t.sample_pos(), 0);
    }
}
