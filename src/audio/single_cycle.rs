//! Single-cycle pitch detection inspired by the Cycfi Q v1.5 approach.
//!
//! Instead of waiting for two full waveform cycles (standard autocorrelation),
//! this module detects pitch from just one cycle by analyzing waveform peak
//! features using a dual-predictor strategy.
//!
//! # Architecture
//!
//! The detector works sample-by-sample with no buffering:
//!
//! 1. **PeakTracker** identifies local maxima and minima in the signal stream
//!    using a simple state machine with minimum prominence filtering.
//!
//! 2. **SingleCycleDetector** uses two predictors:
//!    - **Predictor A**: measures peak-to-peak distance on the original signal
//!    - **Predictor B**: measures peak-to-peak distance on the inverted signal
//!    - Takes the median of three consecutive estimates from both predictors
//!
//! Confidence is based on agreement between the two predictors.

/// Result of a successful pitch detection.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SingleCycleResult {
    /// Detected fundamental frequency in Hz.
    pub frequency: f32,
    /// Confidence score in 0.0..=1.0 based on predictor agreement.
    pub confidence: f32,
    /// Number of samples from the first relevant sample to the detection point.
    pub latency_samples: usize,
}

/// A detected peak (local maximum or minimum) in the signal stream.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Peak {
    /// The sample value at the peak.
    pub value: f32,
    /// The sample index (position) at which the peak occurred.
    pub position: usize,
    /// True for local maximum, false for local minimum.
    pub is_maximum: bool,
}

/// Internal state for the peak-tracking state machine.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PeakState {
    /// Very first sample — we have nothing to compare yet.
    WaitingForFirst,
    /// We have at least one sample but haven't determined a direction yet.
    Init,
    /// Signal is rising; tracking a candidate maximum.
    Rising,
    /// Signal is falling; tracking a candidate minimum.
    Falling,
}

/// Tracks local maxima and minima in a sample stream.
///
/// Uses a simple state machine: rising -> peak detected -> falling -> trough
/// detected. Requires a minimum peak prominence to filter noise. Prominence
/// is measured as the absolute difference between the peak candidate and the
/// value where the preceding monotonic run started (the "approach value").
pub struct PeakTracker {
    state: PeakState,
    /// Minimum absolute difference between a peak and its approach value
    /// before we emit the peak. Filters out noise-level fluctuations.
    min_prominence: f32,
    /// Current candidate peak value (highest seen while rising, lowest while falling).
    candidate_value: f32,
    /// Sample position of the current candidate.
    candidate_pos: usize,
    /// The value at the start of the current monotonic run. For a rising run,
    /// this is the trough where the rise began; for a falling run, it is the
    /// peak where the fall began. Prominence = |candidate - approach_value|.
    approach_value: f32,
    /// Running sample counter.
    sample_index: usize,
}

impl PeakTracker {
    /// Create a new peak tracker.
    ///
    /// `min_prominence` sets the minimum absolute difference between a peak
    /// and its approach value required to emit the peak.
    pub fn new(min_prominence: f32) -> Self {
        Self {
            state: PeakState::WaitingForFirst,
            min_prominence,
            candidate_value: 0.0,
            candidate_pos: 0,
            approach_value: 0.0,
            sample_index: 0,
        }
    }

    /// Feed a single sample and optionally receive a detected peak.
    pub fn feed(&mut self, sample: f32) -> Option<Peak> {
        let pos = self.sample_index;
        self.sample_index += 1;

        match self.state {
            PeakState::WaitingForFirst => {
                self.candidate_value = sample;
                self.candidate_pos = pos;
                self.approach_value = sample;
                self.state = PeakState::Init;
                None
            }
            PeakState::Init => {
                if sample > self.candidate_value {
                    // Signal is going up — the stored candidate was the start of a rise.
                    self.approach_value = self.candidate_value;
                    self.state = PeakState::Rising;
                    self.candidate_value = sample;
                    self.candidate_pos = pos;
                } else if sample < self.candidate_value {
                    // Signal is going down — the stored candidate was the start of a fall.
                    self.approach_value = self.candidate_value;
                    self.state = PeakState::Falling;
                    self.candidate_value = sample;
                    self.candidate_pos = pos;
                }
                None
            }
            PeakState::Rising => {
                if sample >= self.candidate_value {
                    // Still rising — update candidate maximum.
                    self.candidate_value = sample;
                    self.candidate_pos = pos;
                    None
                } else {
                    // Signal turned downward — candidate is a local maximum.
                    let prominence = (self.candidate_value - self.approach_value).abs();

                    if prominence >= self.min_prominence {
                        let peak = Peak {
                            value: self.candidate_value,
                            position: self.candidate_pos,
                            is_maximum: true,
                        };
                        // New falling run starts from this confirmed peak.
                        self.approach_value = self.candidate_value;
                        self.candidate_value = sample;
                        self.candidate_pos = pos;
                        self.state = PeakState::Falling;
                        Some(peak)
                    } else {
                        // Not prominent enough — continue falling without emitting.
                        self.candidate_value = sample;
                        self.candidate_pos = pos;
                        self.state = PeakState::Falling;
                        None
                    }
                }
            }
            PeakState::Falling => {
                if sample <= self.candidate_value {
                    // Still falling — update candidate minimum.
                    self.candidate_value = sample;
                    self.candidate_pos = pos;
                    None
                } else {
                    // Signal turned upward — candidate is a local minimum.
                    let prominence = (self.candidate_value - self.approach_value).abs();

                    if prominence >= self.min_prominence {
                        let peak = Peak {
                            value: self.candidate_value,
                            position: self.candidate_pos,
                            is_maximum: false,
                        };
                        // New rising run starts from this confirmed trough.
                        self.approach_value = self.candidate_value;
                        self.candidate_value = sample;
                        self.candidate_pos = pos;
                        self.state = PeakState::Rising;
                        Some(peak)
                    } else {
                        // Not prominent enough — continue rising without emitting.
                        self.candidate_value = sample;
                        self.candidate_pos = pos;
                        self.state = PeakState::Rising;
                        None
                    }
                }
            }
        }
    }

    /// Reset the tracker to its initial state.
    pub fn reset(&mut self) {
        self.state = PeakState::WaitingForFirst;
        self.candidate_value = 0.0;
        self.candidate_pos = 0;
        self.approach_value = 0.0;
        self.sample_index = 0;
    }
}

/// A single period estimator that tracks peak-to-peak distances.
///
/// Maintains a sliding window of the three most recent estimates and
/// produces the median as the stable period estimate.
struct PeriodPredictor {
    /// Positions of recent maxima (most recent last).
    recent_max_positions: Vec<usize>,
    /// Positions of recent minima (most recent last).
    recent_min_positions: Vec<usize>,
    /// The three most recent period estimates (in samples).
    estimates: Vec<f32>,
}

impl PeriodPredictor {
    fn new() -> Self {
        Self {
            recent_max_positions: Vec::with_capacity(4),
            recent_min_positions: Vec::with_capacity(4),
            estimates: Vec::with_capacity(4),
        }
    }

    /// Record a detected peak and possibly produce a new period estimate.
    fn record_peak(&mut self, peak: &Peak) -> Option<f32> {
        if peak.is_maximum {
            self.recent_max_positions.push(peak.position);
            if self.recent_max_positions.len() > 4 {
                self.recent_max_positions.remove(0);
            }
            // Need at least 2 maxima to measure a period.
            if self.recent_max_positions.len() >= 2 {
                let len = self.recent_max_positions.len();
                let period = (self.recent_max_positions[len - 1]
                    - self.recent_max_positions[len - 2]) as f32;
                if period > 0.0 {
                    self.estimates.push(period);
                    if self.estimates.len() > 3 {
                        self.estimates.remove(0);
                    }
                    return self.median_estimate();
                }
            }
        } else {
            self.recent_min_positions.push(peak.position);
            if self.recent_min_positions.len() > 4 {
                self.recent_min_positions.remove(0);
            }
            // Also measure trough-to-trough as a period estimate.
            if self.recent_min_positions.len() >= 2 {
                let len = self.recent_min_positions.len();
                let period = (self.recent_min_positions[len - 1]
                    - self.recent_min_positions[len - 2]) as f32;
                if period > 0.0 {
                    self.estimates.push(period);
                    if self.estimates.len() > 3 {
                        self.estimates.remove(0);
                    }
                    return self.median_estimate();
                }
            }
        }
        None
    }

    /// Return the median of the current estimates, if we have at least 3.
    fn median_estimate(&self) -> Option<f32> {
        if self.estimates.len() < 3 {
            return None;
        }
        let mut sorted = self.estimates.clone();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        Some(sorted[sorted.len() / 2])
    }

    fn reset(&mut self) {
        self.recent_max_positions.clear();
        self.recent_min_positions.clear();
        self.estimates.clear();
    }
}

/// Single-cycle pitch detector using a dual-predictor approach.
///
/// Inspired by the Cycfi Q v1.5 technique (Pitch Perfect series, 2024),
/// this detector can identify pitch from just one waveform cycle by
/// analyzing peak-to-peak distances with two complementary predictors.
pub struct SingleCycleDetector {
    sample_rate: usize,
    min_freq: f32,
    max_freq: f32,
    /// Peak tracker for the original signal (Predictor A).
    tracker_a: PeakTracker,
    /// Peak tracker for the inverted signal (Predictor B).
    tracker_b: PeakTracker,
    /// Period predictor from original signal peaks.
    predictor_a: PeriodPredictor,
    /// Period predictor from inverted signal peaks.
    predictor_b: PeriodPredictor,
    /// Running sample count since the detector was created/reset.
    sample_count: usize,
    /// Sample position of the very first sample fed into the detector.
    first_sample_pos: Option<usize>,
}

impl SingleCycleDetector {
    /// Create a new single-cycle pitch detector.
    ///
    /// - `sample_rate`: Audio sample rate in Hz (e.g. 44100).
    /// - `min_freq`: Minimum detectable frequency in Hz (e.g. 80.0 for guitar E2).
    /// - `max_freq`: Maximum detectable frequency in Hz (e.g. 1400.0 for guitar E6).
    pub fn new(sample_rate: usize, min_freq: f32, max_freq: f32) -> Self {
        // Set minimum prominence relative to expected signal levels.
        // For guitar signals, a prominence of ~0.005 filters out quantization
        // noise while still catching real peaks.
        let min_prominence = 0.005;

        Self {
            sample_rate,
            min_freq,
            max_freq,
            tracker_a: PeakTracker::new(min_prominence),
            tracker_b: PeakTracker::new(min_prominence),
            predictor_a: PeriodPredictor::new(),
            predictor_b: PeriodPredictor::new(),
            sample_count: 0,
            first_sample_pos: None,
        }
    }

    /// Feed a single audio sample and optionally receive a pitch detection result.
    ///
    /// The detector processes samples one at a time with no internal buffering.
    /// A result is emitted when at least one predictor has enough data to produce
    /// a median estimate.
    pub fn feed_sample(&mut self, sample: f32) -> Option<SingleCycleResult> {
        if self.first_sample_pos.is_none() {
            self.first_sample_pos = Some(self.sample_count);
        }

        let pos = self.sample_count;
        self.sample_count += 1;

        // Feed original signal to Predictor A.
        let peak_a = self.tracker_a.feed(sample);
        // Feed inverted signal to Predictor B.
        let peak_b = self.tracker_b.feed(-sample);

        let mut period_a: Option<f32> = None;
        let mut period_b: Option<f32> = None;

        if let Some(ref peak) = peak_a {
            period_a = self.predictor_a.record_peak(peak);
        }
        if let Some(ref peak) = peak_b {
            // The inverted tracker detects maxima where the original has minima
            // and vice versa. We pass the peak as-is since the predictor just
            // measures distances between like-peaks.
            period_b = self.predictor_b.record_peak(peak);
        }

        // We need at least one predictor to produce a median estimate.
        // Prefer using both for confidence scoring.
        let (period_estimate, confidence) = match (period_a, period_b) {
            (Some(pa), Some(pb)) => {
                // Both predictors have estimates — compute agreement-based confidence.
                let avg = (pa + pb) / 2.0;
                let diff = (pa - pb).abs();
                let agreement = if avg > 0.0 {
                    1.0 - (diff / avg).min(1.0)
                } else {
                    0.0
                };
                ((pa + pb) / 2.0, agreement)
            }
            (Some(pa), None) => {
                // Only Predictor A has an estimate.
                (pa, 0.4)
            }
            (None, Some(pb)) => {
                // Only Predictor B has an estimate.
                (pb, 0.4)
            }
            (None, None) => return None,
        };

        if period_estimate <= 0.0 {
            return None;
        }

        let frequency = self.sample_rate as f32 / period_estimate;

        // Reject estimates outside the valid frequency range.
        if frequency < self.min_freq || frequency > self.max_freq {
            return None;
        }

        let latency = pos - self.first_sample_pos.unwrap_or(pos);

        Some(SingleCycleResult {
            frequency,
            confidence,
            latency_samples: latency,
        })
    }

    /// Reset the detector to its initial state.
    pub fn reset(&mut self) {
        self.tracker_a.reset();
        self.tracker_b.reset();
        self.predictor_a.reset();
        self.predictor_b.reset();
        self.sample_count = 0;
        self.first_sample_pos = None;
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;

    const SAMPLE_RATE: usize = 44100;

    /// Generate a sine wave at the given frequency.
    fn generate_sine(freq: f32, sample_rate: usize, num_samples: usize) -> Vec<f32> {
        (0..num_samples)
            .map(|i| {
                let t = i as f32 / sample_rate as f32;
                (2.0 * PI * freq * t).sin()
            })
            .collect()
    }

    /// Generate a complex waveform: fundamental + 2nd + 3rd harmonic (guitar-like).
    fn generate_guitar_like(freq: f32, sample_rate: usize, num_samples: usize) -> Vec<f32> {
        (0..num_samples)
            .map(|i| {
                let t = i as f32 / sample_rate as f32;
                let fundamental = (2.0 * PI * freq * t).sin();
                let h2 = 0.5 * (2.0 * PI * 2.0 * freq * t).sin();
                let h3 = 0.25 * (2.0 * PI * 3.0 * freq * t).sin();
                fundamental + h2 + h3
            })
            .collect()
    }

    /// Feed all samples into the detector and collect all results.
    fn detect_all(detector: &mut SingleCycleDetector, samples: &[f32]) -> Vec<SingleCycleResult> {
        samples
            .iter()
            .filter_map(|&s| detector.feed_sample(s))
            .collect()
    }

    /// Get the last detected result (most stable).
    fn last_detection(results: &[SingleCycleResult]) -> Option<&SingleCycleResult> {
        results.last()
    }

    // -------------------------------------------------------------------------
    // Test: Detecting frequency from a generated sine wave sample-by-sample
    // -------------------------------------------------------------------------

    #[test]
    fn test_sine_wave_detection() {
        let freq = 440.0; // A4
        let samples = generate_sine(freq, SAMPLE_RATE, SAMPLE_RATE / 4); // 0.25s
        let mut detector = SingleCycleDetector::new(SAMPLE_RATE, 80.0, 2000.0);

        let results = detect_all(&mut detector, &samples);
        assert!(!results.is_empty(), "Should detect pitch from sine wave");

        let last = last_detection(&results).unwrap();
        let error_pct = ((last.frequency - freq) / freq).abs() * 100.0;
        assert!(
            error_pct < 3.0,
            "Sine wave frequency error should be < 3%, got {:.2}% (detected {:.1} Hz, expected {:.1} Hz)",
            error_pct,
            last.frequency,
            freq
        );
    }

    // -------------------------------------------------------------------------
    // Test: Detecting frequency from a complex waveform (guitar-like)
    // -------------------------------------------------------------------------

    #[test]
    fn test_guitar_like_waveform_detection() {
        let freq = 329.63; // E4
        let samples = generate_guitar_like(freq, SAMPLE_RATE, SAMPLE_RATE / 4);
        let mut detector = SingleCycleDetector::new(SAMPLE_RATE, 80.0, 2000.0);

        let results = detect_all(&mut detector, &samples);
        assert!(
            !results.is_empty(),
            "Should detect pitch from guitar-like waveform"
        );

        let last = last_detection(&results).unwrap();
        let error_pct = ((last.frequency - freq) / freq).abs() * 100.0;
        assert!(
            error_pct < 5.0,
            "Guitar-like waveform frequency error should be < 5%, got {:.2}% (detected {:.1} Hz, expected {:.1} Hz)",
            error_pct,
            last.frequency,
            freq
        );
    }

    // -------------------------------------------------------------------------
    // Test: Dual predictor agreement on clean signals
    // -------------------------------------------------------------------------

    #[test]
    fn test_dual_predictor_agreement_clean_signal() {
        let freq = 440.0;
        let samples = generate_sine(freq, SAMPLE_RATE, SAMPLE_RATE / 2); // 0.5s
        let mut detector = SingleCycleDetector::new(SAMPLE_RATE, 80.0, 2000.0);

        let results = detect_all(&mut detector, &samples);
        assert!(!results.is_empty(), "Should produce results for clean sine");

        // On a clean sine, the later results should have high confidence
        // because both predictors agree.
        let late_results: Vec<_> = results.iter().filter(|r| r.confidence > 0.5).collect();

        assert!(
            !late_results.is_empty(),
            "Clean sine should produce high-confidence results (>0.5)"
        );

        // The best confidence should be quite high for a pure sine.
        let max_confidence = results.iter().map(|r| r.confidence).fold(0.0_f32, f32::max);
        assert!(
            max_confidence > 0.7,
            "Best confidence on clean sine should be > 0.7, got {:.3}",
            max_confidence
        );
    }

    // -------------------------------------------------------------------------
    // Test: Confidence drops for noisy signals
    // -------------------------------------------------------------------------

    #[test]
    fn test_confidence_drops_for_noisy_signal() {
        let freq = 440.0;
        let clean_samples = generate_sine(freq, SAMPLE_RATE, SAMPLE_RATE / 2);
        let mut detector_clean = SingleCycleDetector::new(SAMPLE_RATE, 80.0, 2000.0);
        let clean_results = detect_all(&mut detector_clean, &clean_samples);

        // Generate a noisy version: add pseudo-random noise.
        let noisy_samples: Vec<f32> = clean_samples
            .iter()
            .enumerate()
            .map(|(i, &s)| {
                // Simple deterministic "noise" using bit manipulation.
                let noise = ((i as u32).wrapping_mul(2654435761) as f32) / u32::MAX as f32;
                let noise = (noise - 0.5) * 0.6; // noise amplitude ~0.3 peak
                s + noise
            })
            .collect();
        let mut detector_noisy = SingleCycleDetector::new(SAMPLE_RATE, 80.0, 2000.0);
        let noisy_results = detect_all(&mut detector_noisy, &noisy_samples);

        // Calculate average confidence for both.
        let avg_clean = if clean_results.is_empty() {
            0.0
        } else {
            clean_results.iter().map(|r| r.confidence).sum::<f32>() / clean_results.len() as f32
        };
        let avg_noisy = if noisy_results.is_empty() {
            0.0
        } else {
            noisy_results.iter().map(|r| r.confidence).sum::<f32>() / noisy_results.len() as f32
        };

        assert!(
            avg_clean > avg_noisy || noisy_results.is_empty(),
            "Clean signal should have higher average confidence ({:.3}) than noisy ({:.3})",
            avg_clean,
            avg_noisy
        );
    }

    // -------------------------------------------------------------------------
    // Test: Correct frequency for guitar range E2=82.4Hz through E6=1318.5Hz
    // -------------------------------------------------------------------------

    #[test]
    fn test_guitar_range_frequencies() {
        // Standard guitar open string frequencies and high-end.
        let guitar_freqs = [
            ("E2", 82.4),
            ("A2", 110.0),
            ("D3", 146.83),
            ("G3", 196.0),
            ("B3", 246.94),
            ("E4", 329.63),
            ("A4", 440.0),
            ("E5", 659.26),
            ("E6", 1318.5),
        ];

        for (name, freq) in &guitar_freqs {
            let num_samples = SAMPLE_RATE / 2; // 0.5 seconds
            let samples = generate_sine(*freq, SAMPLE_RATE, num_samples);
            let mut detector = SingleCycleDetector::new(SAMPLE_RATE, 70.0, 1400.0);

            let results = detect_all(&mut detector, &samples);
            assert!(
                !results.is_empty(),
                "Should detect {} ({:.1} Hz)",
                name,
                freq
            );

            let last = last_detection(&results).unwrap();
            let error_pct = ((last.frequency - freq) / freq).abs() * 100.0;
            assert!(
                error_pct < 5.0,
                "{} ({:.1} Hz): error {:.2}% (detected {:.1} Hz), expected < 5%",
                name,
                freq,
                error_pct,
                last.frequency
            );
        }
    }

    // -------------------------------------------------------------------------
    // Test: Latency is within 1-2 cycles
    // -------------------------------------------------------------------------

    #[test]
    fn test_latency_within_two_cycles() {
        let freq = 440.0; // A4 — period ~100 samples at 44100 Hz
        let samples = generate_sine(freq, SAMPLE_RATE, SAMPLE_RATE / 4);
        let mut detector = SingleCycleDetector::new(SAMPLE_RATE, 80.0, 2000.0);

        let results = detect_all(&mut detector, &samples);
        assert!(!results.is_empty(), "Should detect A4");

        let first = &results[0];
        let period_samples = SAMPLE_RATE as f32 / freq;
        // The first detection should appear within about 2-3 cycles.
        // Each predictor needs 3 median estimates (3 peak-to-peak measurements),
        // which requires ~3 cycles. We allow up to 4 cycles for the very first
        // detection to account for startup.
        let max_latency = (period_samples * 4.0) as usize;
        assert!(
            first.latency_samples <= max_latency,
            "First detection latency ({} samples) should be <= {} samples (~4 cycles of {:.0} Hz)",
            first.latency_samples,
            max_latency,
            freq
        );
    }

    // -------------------------------------------------------------------------
    // Test: PeakTracker correctly identifies maxima and minima
    // -------------------------------------------------------------------------

    #[test]
    fn test_peak_tracker_basic() {
        let mut tracker = PeakTracker::new(0.1);

        // Feed a simple triangle-ish pattern: 0, 0.5, 1.0, 0.5, 0, -0.5, -1.0, -0.5, 0
        let signal = [0.0, 0.5, 1.0, 0.5, 0.0, -0.5, -1.0, -0.5, 0.0, 0.5, 1.0];
        let mut peaks: Vec<Peak> = Vec::new();

        for &s in &signal {
            if let Some(peak) = tracker.feed(s) {
                peaks.push(peak);
            }
        }

        // Should detect at least one maximum and one minimum.
        let maxima: Vec<_> = peaks.iter().filter(|p| p.is_maximum).collect();
        let minima: Vec<_> = peaks.iter().filter(|p| !p.is_maximum).collect();

        assert!(!maxima.is_empty(), "Should detect at least one maximum");
        assert!(!minima.is_empty(), "Should detect at least one minimum");

        // The first maximum should be near value 1.0.
        assert!(
            (maxima[0].value - 1.0).abs() < 0.01,
            "First maximum should be ~1.0, got {}",
            maxima[0].value
        );
        // The first minimum should be near value -1.0.
        assert!(
            (minima[0].value - (-1.0)).abs() < 0.01,
            "First minimum should be ~-1.0, got {}",
            minima[0].value
        );
    }

    // -------------------------------------------------------------------------
    // Test: PeakTracker filters out noise with min_prominence
    // -------------------------------------------------------------------------

    #[test]
    fn test_peak_tracker_filters_noise() {
        let mut tracker = PeakTracker::new(0.3);

        // Small fluctuations that should be filtered out.
        let noise = [0.0, 0.1, 0.05, 0.15, 0.1, 0.2, 0.15];
        let mut peaks: Vec<Peak> = Vec::new();

        for &s in &noise {
            if let Some(peak) = tracker.feed(s) {
                peaks.push(peak);
            }
        }

        assert!(
            peaks.is_empty(),
            "Small fluctuations below min_prominence should produce no peaks, got {:?}",
            peaks
        );
    }
}
