//! Transport primitive: sample-accurate clock driven by an audio callback.
//!
//! # Model
//!
//! - `sample_pos` — total samples elapsed since the transport was started.
//!   Only advanced by `advance()`, which is intended to be called from
//!   the audio thread's callback.
//! - `bpm` — tempo in beats per minute, stored as fixed-point (× 1000)
//!   in an atomic so it can be updated from any thread without locks.
//! - `beat_position` — accumulated musical position in fixed-point beat units.
//!   New tempo affects future samples only; it never rewrites history.
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

/// Q28.36 fixed-point: over a year of range at 400 BPM and sub-20-picobeat
/// resolution per audio block, without floating-point races or callback locks.
const BEAT_UNITS_PER_BEAT: u64 = 1 << 36;

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
    /// Accumulated Q28.36 musical position. Tempo changes leave it untouched.
    beat_position_units: AtomicU64,
    /// Division remainder carried between blocks so accumulation does not
    /// depend on audio callback size.
    beat_remainder: AtomicU64,
    /// Incremented by reset/seek/meter changes so schedulers can distinguish
    /// transport jumps from an ordinary delayed polling window.
    discontinuity_revision: AtomicU64,
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
            sample_rate: AtomicU32::new(sample_rate.max(1)),
            bpm_milli: AtomicU32::new(120_000),
            beat_position_units: AtomicU64::new(0),
            beat_remainder: AtomicU64::new(0),
            discontinuity_revision: AtomicU64::new(0),
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

    /// Update the sample rate without moving the current musical position.
    pub fn set_sample_rate(&self, sr: u32) {
        self.sample_rate.store(sr.max(1), Ordering::Relaxed);
        self.beat_remainder.store(0, Ordering::Relaxed);
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
        self.beat_position_units.load(Ordering::Relaxed) as f64 / BEAT_UNITS_PER_BEAT as f64
    }

    /// Revision changed by reset, seek, or meter changes, but not tempo.
    pub fn discontinuity_revision(&self) -> u64 {
        self.discontinuity_revision.load(Ordering::Acquire)
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

    /// Set tempo in BPM. Clamped to [20, 400]. Takes effect immediately
    /// for future samples without moving the current total-beat position.
    pub fn set_bpm(&self, bpm: f64) {
        let clamped = bpm.clamp(20.0, 400.0);
        self.bpm_milli
            .store((clamped * 1000.0) as u32, Ordering::Relaxed);
        self.beat_remainder.store(0, Ordering::Relaxed);
    }

    pub fn set_time_signature(&self, beats_per_bar: u8, beat_unit: u8) {
        let next = (beats_per_bar.clamp(1, 32), beat_unit.clamp(1, 64));
        if self.time_signature() == next {
            return;
        }
        self.beats_per_bar.store(next.0, Ordering::Relaxed);
        self.beat_unit.store(next.1, Ordering::Relaxed);
        self.discontinuity_revision.fetch_add(1, Ordering::AcqRel);
    }

    /// Start the clock (resume from current sample_pos).
    pub fn play(&self) {
        self.running.store(true, Ordering::Relaxed);
    }

    /// Stop the clock (freeze sample_pos). `play()` resumes from here.
    pub fn stop(&self) {
        self.running.store(false, Ordering::Relaxed);
    }

    /// Reset sample_pos and musical position to 0. Does not change running state.
    pub fn reset(&self) {
        self.sample_pos.store(0, Ordering::Relaxed);
        self.beat_position_units.store(0, Ordering::Relaxed);
        self.beat_remainder.store(0, Ordering::Relaxed);
        self.last_crossed_beat.store(u64::MAX, Ordering::Relaxed);
        self.discontinuity_revision.fetch_add(1, Ordering::AcqRel);
    }

    /// Seek `sample_pos` to an absolute value. Re-anchors
    /// `last_crossed_beat` so beat-crossing detection doesn't fire
    /// a retroactive burst (going forward) or stall (going backward).
    /// Used by host-driven adapters (e.g. nih-plug plugins reading
    /// `ProcessContext::transport().pos_samples()`) to follow DAW
    /// loop / jump / locate events.
    pub fn set_sample_pos(&self, sample_pos: u64) {
        let bpm_milli = self.bpm_milli.load(Ordering::Relaxed) as u128;
        let sample_rate = self.sample_rate.load(Ordering::Relaxed).max(1) as u128;
        let denominator = sample_rate * 60_000;
        let units = (sample_pos as u128 * bpm_milli * BEAT_UNITS_PER_BEAT as u128
            + denominator / 2)
            / denominator;
        self.sample_pos.store(sample_pos, Ordering::Relaxed);
        self.beat_position_units
            .store(units.min(u64::MAX as u128) as u64, Ordering::Relaxed);
        self.beat_remainder.store(0, Ordering::Relaxed);
        self.last_crossed_beat.store(
            (units / BEAT_UNITS_PER_BEAT as u128) as u64,
            Ordering::Relaxed,
        );
        self.discontinuity_revision.fetch_add(1, Ordering::AcqRel);
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
        let bpm_milli = self.bpm_milli.load(Ordering::Relaxed) as u128;
        let sample_rate = self.sample_rate.load(Ordering::Relaxed).max(1) as u128;
        let denominator = sample_rate * 60_000;
        let numerator = frames as u128 * bpm_milli * BEAT_UNITS_PER_BEAT as u128
            + self.beat_remainder.load(Ordering::Relaxed) as u128;
        let delta_units = numerator / denominator;
        self.beat_remainder
            .store((numerator % denominator) as u64, Ordering::Relaxed);
        let delta_units = delta_units.min(u64::MAX as u128) as u64;
        let new_units = self
            .beat_position_units
            .fetch_add(delta_units, Ordering::Relaxed)
            .saturating_add(delta_units);
        let current_beat = new_units / BEAT_UNITS_PER_BEAT;

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
    fn tempo_changes_preserve_current_beat_and_change_future_rate() {
        let t = Transport::new(48_000);
        t.play();
        let _ = t.advance(78_000); // 3.25 beats at 120 BPM.
        assert!((t.total_beats() - 3.25).abs() < 1.0e-9);

        t.set_bpm(60.0);
        assert!((t.total_beats() - 3.25).abs() < 1.0e-9);
        let _ = t.advance(48_000);
        assert!((t.total_beats() - 4.25).abs() < 1.0e-9);

        t.set_bpm(180.0);
        assert!((t.total_beats() - 4.25).abs() < 1.0e-9);
        let _ = t.advance(16_000);
        assert!((t.total_beats() - 5.25).abs() < 1.0e-9);
    }

    #[test]
    fn discontinuity_revision_excludes_tempo_changes() {
        let t = Transport::new(48_000);
        assert_eq!(t.discontinuity_revision(), 0);
        t.set_bpm(90.0);
        assert_eq!(t.discontinuity_revision(), 0);
        t.set_time_signature(3, 4);
        assert_eq!(t.discontinuity_revision(), 1);
        t.set_time_signature(3, 4);
        assert_eq!(t.discontinuity_revision(), 1);
        t.set_sample_pos(24_000);
        assert_eq!(t.discontinuity_revision(), 2);
        t.reset();
        assert_eq!(t.discontinuity_revision(), 3);
    }

    #[test]
    fn tempo_change_near_boundary_does_not_swallow_crossing() {
        let t = Transport::new(48_000);
        t.play();
        let _ = t.advance(23_999);
        t.set_bpm(60.0);
        let crossing = t.advance(3).expect("beat one must still cross");
        assert_eq!(crossing.total_beat, 1);
    }

    #[test]
    fn beat_accumulation_is_independent_of_callback_size() {
        let one_block = Transport::new(48_000);
        one_block.play();
        let _ = one_block.advance(48_000);

        let tiny_blocks = Transport::new(48_000);
        tiny_blocks.play();
        for _ in 0..48_000 {
            let _ = tiny_blocks.advance(1);
        }
        assert!((one_block.total_beats() - tiny_blocks.total_beats()).abs() < 1.0e-12);
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
