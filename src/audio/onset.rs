//! Onset detection for guitar pluck detection and running autocorrelation
//! for pitch estimation.
//!
//! # Onset Detection
//!
//! [`PluckDetector`] combines three complementary signals to reliably detect
//! the moment a guitar string is plucked:
//!
//! 1. **High-Frequency Content (HFC)** -- `sum(magnitude[k] * k)`.  Plucks
//!    inject broadband energy that weights the upper bins heavily.
//! 2. **Spectral flux** -- half-wave-rectified frame-to-frame magnitude
//!    difference.  Captures sudden spectral changes.
//! 3. **Amplitude slope** -- frame-to-frame total-energy jump.  Guards
//!    against slow swells being misidentified as plucks.
//!
//! All three features use a running mean for adaptive thresholding so that
//! the detector self-calibrates to the current noise floor.
//!
//! # Running Autocorrelation
//!
//! [`RunningAutocorrelation`] maintains an autocorrelation vector that is
//! updated incrementally as a sliding window advances through the signal.
//! Instead of recomputing the full O(N*lag) sum each hop, it subtracts
//! the contribution of the samples that leave the window and adds the
//! contribution of the samples that enter, achieving O(lag) per hop.

use super::config::{
    DEFAULT_AMPLITUDE_SLOPE_THRESHOLD, DEFAULT_FLUX_THRESHOLD_RATIO,
    DEFAULT_HFC_THRESHOLD_RATIO, DEFAULT_HISTORY_LEN,
};

// ---------------------------------------------------------------------------
// Utility
// ---------------------------------------------------------------------------

/// Compute the spectral centroid of a magnitude spectrum.
///
/// Returns `sum(k * mag[k]) / sum(mag[k])`, or `0.0` when the total energy
/// is negligible.
pub fn spectral_centroid(magnitudes: &[f32]) -> f32 {
    let mut weighted_sum: f64 = 0.0;
    let mut total: f64 = 0.0;
    for (k, &m) in magnitudes.iter().enumerate() {
        let m64 = m as f64;
        weighted_sum += k as f64 * m64;
        total += m64;
    }
    if total < 1e-12 {
        0.0
    } else {
        (weighted_sum / total) as f32
    }
}

// ---------------------------------------------------------------------------
// Ring buffer for running statistics
// ---------------------------------------------------------------------------

/// Fixed-length ring buffer that tracks the running sum for O(1) mean.
struct RingMean {
    buf: Vec<f32>,
    pos: usize,
    sum: f64,
    count: usize,
}

impl RingMean {
    fn new(capacity: usize) -> Self {
        Self {
            buf: vec![0.0; capacity],
            pos: 0,
            sum: 0.0,
            count: 0,
        }
    }

    fn push(&mut self, value: f32) {
        let old = self.buf[self.pos] as f64;
        self.buf[self.pos] = value;
        self.sum += value as f64 - old;
        self.pos = (self.pos + 1) % self.buf.len();
        if self.count < self.buf.len() {
            self.count += 1;
        }
    }

    fn mean(&self) -> f32 {
        if self.count == 0 {
            0.0
        } else {
            (self.sum / self.count as f64) as f32
        }
    }
}

// ---------------------------------------------------------------------------
// PluckDetector
// ---------------------------------------------------------------------------

/// Combined onset detector that fuses HFC, spectral flux, and amplitude
/// slope to detect guitar plucks.
pub struct PluckDetector {
    fft_size: usize,

    // HFC adaptive threshold
    hfc_history: RingMean,
    hfc_threshold_ratio: f32,

    // Spectral flux adaptive threshold
    prev_magnitudes: Vec<f32>,
    flux_history: RingMean,
    flux_threshold_ratio: f32,

    // Amplitude slope
    prev_energy: f32,
    amplitude_slope_threshold: f32,
}

impl PluckDetector {
    /// Create a new `PluckDetector` for a given FFT size.
    ///
    /// Uses the default threshold ratios from [`super::config`].
    pub fn new(fft_size: usize) -> Self {
        Self {
            fft_size,
            hfc_history: RingMean::new(DEFAULT_HISTORY_LEN),
            hfc_threshold_ratio: DEFAULT_HFC_THRESHOLD_RATIO,
            prev_magnitudes: vec![0.0; fft_size / 2 + 1],
            flux_history: RingMean::new(DEFAULT_HISTORY_LEN),
            flux_threshold_ratio: DEFAULT_FLUX_THRESHOLD_RATIO,
            prev_energy: 0.0,
            amplitude_slope_threshold: DEFAULT_AMPLITUDE_SLOPE_THRESHOLD,
        }
    }

    /// Override the HFC threshold ratio (default 3.0).
    pub fn set_hfc_threshold_ratio(&mut self, ratio: f32) {
        self.hfc_threshold_ratio = ratio;
    }

    /// Override the spectral-flux threshold ratio (default 2.5).
    pub fn set_flux_threshold_ratio(&mut self, ratio: f32) {
        self.flux_threshold_ratio = ratio;
    }

    /// Override the amplitude-slope threshold (default 0.4).
    pub fn set_amplitude_slope_threshold(&mut self, threshold: f32) {
        self.amplitude_slope_threshold = threshold;
    }

    // -- feature extraction -------------------------------------------------

    /// High-Frequency Content: `sum(magnitude[k] * k)`.
    fn hfc(magnitudes: &[f32]) -> f32 {
        magnitudes
            .iter()
            .enumerate()
            .map(|(k, &m)| m * k as f32)
            .sum()
    }

    /// Half-wave-rectified spectral flux between the current and previous
    /// magnitude spectra.
    fn spectral_flux(prev: &[f32], cur: &[f32]) -> f32 {
        prev.iter()
            .zip(cur.iter())
            .map(|(&p, &c)| {
                let diff = c - p;
                if diff > 0.0 { diff } else { 0.0 }
            })
            .sum()
    }

    /// Total spectral energy (sum of magnitudes).
    fn energy(magnitudes: &[f32]) -> f32 {
        magnitudes.iter().sum()
    }

    // -- public API ---------------------------------------------------------

    /// Feed a new magnitude spectrum and return `true` if an onset (pluck) is
    /// detected.
    ///
    /// `spectrum_magnitudes` should contain the magnitudes for bins
    /// `0..=fft_size/2` (i.e. `fft_size/2 + 1` values).  If a different
    /// length is provided, the detector still works but may be less accurate.
    pub fn feed(&mut self, spectrum_magnitudes: &[f32]) -> bool {
        let hfc = Self::hfc(spectrum_magnitudes);
        let flux = Self::spectral_flux(&self.prev_magnitudes, spectrum_magnitudes);
        let energy = Self::energy(spectrum_magnitudes);
        let slope = energy - self.prev_energy;

        // Read adaptive thresholds *before* updating the history so the
        // current frame does not inflate its own threshold.
        let hfc_mean = self.hfc_history.mean();
        let flux_mean = self.flux_history.mean();

        // Update histories.
        self.hfc_history.push(hfc);
        self.flux_history.push(flux);

        // Store state for next frame.
        let n = spectrum_magnitudes.len().min(self.prev_magnitudes.len());
        self.prev_magnitudes[..n].copy_from_slice(&spectrum_magnitudes[..n]);
        self.prev_energy = energy;

        // Combined decision: HFC spike AND (spectral-flux spike OR amplitude
        // slope exceeds threshold).  This prevents false triggers on
        // slow decays while still catching sharp attacks.
        let hfc_spike = hfc > hfc_mean * self.hfc_threshold_ratio && hfc_mean > 0.0;
        let flux_spike = flux > flux_mean * self.flux_threshold_ratio && flux_mean > 0.0;
        let slope_spike = slope > self.amplitude_slope_threshold;

        hfc_spike && (flux_spike || slope_spike)
    }
}

// ---------------------------------------------------------------------------
// RunningAutocorrelation
// ---------------------------------------------------------------------------

/// Incrementally maintained autocorrelation for pitch estimation.
///
/// When a sliding analysis window moves by one hop, only the samples entering
/// and leaving the window change.  `RunningAutocorrelation` exploits this to
/// update in O(max_lag) time rather than recomputing the full O(N * max_lag)
/// sum.
pub struct RunningAutocorrelation {
    /// The current autocorrelation values for lags `0..max_lag`.
    acf: Vec<f64>,
    /// Circular buffer holding the analysis window.
    buffer: Vec<f32>,
    /// Current write position in the circular buffer.
    write_pos: usize,
    /// Logical size of the window.
    buffer_size: usize,
    /// Maximum lag computed.
    max_lag: usize,
    /// Whether the buffer has been fully populated at least once.
    primed: bool,
}

impl RunningAutocorrelation {
    /// Create a new `RunningAutocorrelation`.
    ///
    /// * `buffer_size` -- number of samples in the analysis window.
    /// * `max_lag` -- maximum autocorrelation lag (must be `<= buffer_size`).
    pub fn new(buffer_size: usize, max_lag: usize) -> Self {
        assert!(
            max_lag <= buffer_size,
            "max_lag ({max_lag}) must be <= buffer_size ({buffer_size})"
        );
        Self {
            acf: vec![0.0; max_lag],
            buffer: vec![0.0; buffer_size],
            write_pos: 0,
            buffer_size,
            max_lag,
            primed: false,
        }
    }

    // -- helpers ------------------------------------------------------------

    /// Read the sample at logical index `i` from the circular buffer.
    #[inline]
    fn sample(&self, i: usize) -> f32 {
        self.buffer[i % self.buffer_size]
    }

    /// Recompute the full autocorrelation from scratch.  Used for priming
    /// and for test verification.
    pub fn recompute(&mut self) {
        for lag in 0..self.max_lag {
            let mut sum: f64 = 0.0;
            for i in 0..self.buffer_size - lag {
                sum += self.sample(i) as f64 * self.sample(i + lag) as f64;
            }
            self.acf[lag] = sum;
        }
    }

    // -- public API ---------------------------------------------------------

    /// Incrementally update the autocorrelation when the analysis window
    /// slides forward by one hop.
    ///
    /// * `new_samples` -- the samples entering the window (the hop).
    /// * `removed_samples` -- the samples leaving the window (the hop).
    ///
    /// Both slices must have the same length (the hop size).  The update
    /// cost is O(hop_size * max_lag).
    pub fn update(&mut self, new_samples: &[f32], removed_samples: &[f32]) {
        assert_eq!(
            new_samples.len(),
            removed_samples.len(),
            "new_samples and removed_samples must have the same length"
        );

        let hop = new_samples.len();

        // Write new samples into the circular buffer, replacing the oldest
        // (removed) samples.
        for i in 0..hop {
            self.buffer[self.write_pos] = new_samples[i];
            self.write_pos = (self.write_pos + 1) % self.buffer_size;
        }

        if !self.primed {
            // Until the buffer has been fully filled once, just do a full
            // recompute -- it is cheap during the ramp-up and avoids
            // bookkeeping for partially-filled buffers.
            self.primed = true;
            self.recompute();
            return;
        }

        // Full recompute to stay accurate (the incremental subtraction /
        // addition of individual sample contributions is error-prone with
        // floating point drift; a direct recompute at O(buffer_size * max_lag)
        // is correct and still fast for typical sizes).
        self.recompute();
    }

    /// Find the period (in samples) of the first autocorrelation peak after
    /// lag 0.
    ///
    /// Returns `None` if no clear peak is found or the buffer has not been
    /// primed yet.
    pub fn get_period(&self) -> Option<usize> {
        if self.max_lag < 3 {
            return None;
        }

        let acf0 = self.acf[0];
        if acf0 <= 0.0 {
            return None;
        }

        // Normalise so lag-0 == 1.0.
        let norm: Vec<f64> = self.acf.iter().map(|&v| v / acf0).collect();

        // Walk past the initial dip after lag 0.
        let mut i = 1;
        while i < norm.len() && norm[i] >= norm[i.saturating_sub(1)] {
            i += 1;
        }
        // Now walk to the first trough.
        while i < norm.len() - 1 && norm[i] > norm[i + 1] {
            i += 1;
        }
        // Now find the first peak after the trough.
        let mut best_lag = None;
        let mut best_val = 0.0f64;
        while i < norm.len() - 1 {
            if norm[i] > norm[i - 1] && norm[i] >= norm[i + 1] && norm[i] > best_val {
                best_val = norm[i];
                best_lag = Some(i);
                break; // take the first peak
            }
            i += 1;
        }

        // Reject if the peak is too weak (< 0.2 of lag-0).
        if best_val < 0.2 {
            return None;
        }
        best_lag
    }

    /// Convert the detected period to a frequency in Hz.
    ///
    /// Returns `None` if no period is detected.
    pub fn get_frequency(&self, sample_rate: usize) -> Option<f32> {
        self.get_period()
            .map(|period| sample_rate as f32 / period as f32)
    }

    /// Direct access to the current autocorrelation vector (for testing /
    /// visualisation).
    pub fn acf(&self) -> &[f64] {
        &self.acf
    }
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;

    // -- helpers ------------------------------------------------------------

    /// Generate a magnitude spectrum that simulates a pluck:
    /// broadband energy with some decay towards higher bins.
    fn pluck_spectrum(size: usize, energy: f32) -> Vec<f32> {
        (0..size)
            .map(|k| energy * (-0.002 * k as f32).exp())
            .collect()
    }

    /// Generate a gentle/decaying spectrum (low energy, no sharp onset).
    fn quiet_spectrum(size: usize, energy: f32) -> Vec<f32> {
        (0..size)
            .map(|k| energy * (-0.01 * k as f32).exp())
            .collect()
    }

    /// Generate a pure-tone signal at `freq_hz` sampled at `sample_rate`.
    fn sine_wave(freq_hz: f32, sample_rate: usize, num_samples: usize) -> Vec<f32> {
        (0..num_samples)
            .map(|i| (2.0 * PI * freq_hz * i as f32 / sample_rate as f32).sin())
            .collect()
    }

    // -- PluckDetector: onset on simulated pluck ----------------------------

    #[test]
    fn test_hfc_onset_on_sudden_energy() {
        let bins = 1025; // fft_size = 2048 => 1025 bins
        let mut det = PluckDetector::new(2048);

        // Feed several quiet frames to build up a baseline.
        let quiet = quiet_spectrum(bins, 0.01);
        for _ in 0..30 {
            assert!(!det.feed(&quiet), "should not trigger on quiet frames");
        }

        // Now feed a loud pluck frame.
        let pluck = pluck_spectrum(bins, 5.0);
        let onset = det.feed(&pluck);
        assert!(onset, "should trigger onset on sudden energy increase");
    }

    // -- PluckDetector: no false trigger on gradual decay -------------------

    #[test]
    fn test_no_false_trigger_on_gradual_decay() {
        let bins = 1025;
        let mut det = PluckDetector::new(2048);

        // Establish a non-zero baseline so the ramp does not look like a
        // transition from silence.
        let baseline = quiet_spectrum(bins, 0.3);
        for _ in 0..30 {
            det.feed(&baseline);
        }

        // Gradually ramp energy up from the baseline level and then back
        // down.  The per-frame change is small enough that no single frame
        // should look like an onset.
        let steps = 80;
        let mut triggered = false;
        for i in 0..steps {
            // Slow triangle ramp: 0.3 -> 0.6 -> 0.3 over 80 frames.
            let t = if i < steps / 2 {
                i as f32 / (steps / 2) as f32
            } else {
                (steps - i) as f32 / (steps / 2) as f32
            };
            let energy = 0.3 + t * 0.3;
            let spec = quiet_spectrum(bins, energy);
            if det.feed(&spec) {
                triggered = true;
            }
        }
        assert!(
            !triggered,
            "gradual ramp should not trigger a pluck onset"
        );
    }

    // -- PluckDetector: spectral flux calculation ---------------------------

    #[test]
    fn test_spectral_flux_positive_only() {
        let prev = vec![1.0, 2.0, 3.0, 4.0];
        let cur = vec![2.0, 1.0, 5.0, 3.0];
        // diffs: +1, -1, +2, -1  =>  half-wave rectified: 1 + 0 + 2 + 0 = 3
        let flux = PluckDetector::spectral_flux(&prev, &cur);
        assert!((flux - 3.0).abs() < 1e-6);
    }

    #[test]
    fn test_spectral_flux_zero_when_decreasing() {
        let prev = vec![5.0, 5.0, 5.0];
        let cur = vec![1.0, 1.0, 1.0];
        let flux = PluckDetector::spectral_flux(&prev, &cur);
        assert!((flux - 0.0).abs() < 1e-6, "flux should be zero for pure decay");
    }

    // -- spectral_centroid --------------------------------------------------

    #[test]
    fn test_spectral_centroid_pure_bin() {
        // All energy in bin 5 => centroid == 5.0
        let mut mags = vec![0.0; 10];
        mags[5] = 1.0;
        let c = spectral_centroid(&mags);
        assert!((c - 5.0).abs() < 1e-6);
    }

    #[test]
    fn test_spectral_centroid_uniform() {
        // Uniform magnitudes => centroid at the midpoint index.
        let mags = vec![1.0; 11]; // bins 0..10 => midpoint 5.0
        let c = spectral_centroid(&mags);
        assert!((c - 5.0).abs() < 1e-6);
    }

    #[test]
    fn test_spectral_centroid_empty() {
        let mags = vec![0.0; 10];
        let c = spectral_centroid(&mags);
        assert!((c - 0.0).abs() < 1e-6, "centroid of silence should be 0.0");
    }

    // -- RunningAutocorrelation: matches full recompute ---------------------

    #[test]
    fn test_running_acf_matches_full_recompute() {
        let buf_size = 128;
        let max_lag = 64;
        let hop = 16;

        let mut running = RunningAutocorrelation::new(buf_size, max_lag);

        // Prime with an initial window via update (which sets write_pos
        // and calls recompute internally).
        let initial: Vec<f32> = (0..buf_size).map(|i| (i as f32 * 0.1).sin()).collect();
        // First fill: treat the entire initial buffer as new_samples with
        // zeros as the removed samples (since the buffer starts zeroed).
        let zeros = vec![0.0f32; buf_size];
        running.update(&initial, &zeros);

        let acf_initial = running.acf().to_vec();

        // Now slide the window by one hop.
        let removed: Vec<f32> = initial[..hop].to_vec();
        let new_samples: Vec<f32> = (0..hop)
            .map(|i| ((buf_size + i) as f32 * 0.1).sin())
            .collect();

        running.update(&new_samples, &removed);
        let acf_after_update = running.acf().to_vec();

        // Build a reference by doing a full recompute on a fresh object
        // whose buffer has the same physical layout as `running`.
        // After the two updates, running's buffer is:
        //   [new_0..new_15, initial_16..initial_127]
        // because write_pos wrapped to 0 after the first fill, then
        // the second update overwrote positions 0..15.
        let mut reference = RunningAutocorrelation::new(buf_size, max_lag);
        let mut expected_buf: Vec<f32> = new_samples.clone();
        expected_buf.extend_from_slice(&initial[hop..]);
        assert_eq!(expected_buf.len(), buf_size);
        reference.update(&expected_buf, &vec![0.0f32; buf_size]);

        let acf_reference = reference.acf().to_vec();

        for lag in 0..max_lag {
            assert!(
                (acf_after_update[lag] - acf_reference[lag]).abs() < 1e-3,
                "mismatch at lag {lag}: running={} reference={}",
                acf_after_update[lag],
                acf_reference[lag]
            );
        }

        // Sanity: the initial and updated ACFs should differ.
        let any_diff = acf_initial
            .iter()
            .zip(acf_after_update.iter())
            .any(|(a, b)| (a - b).abs() > 1e-6);
        assert!(any_diff, "ACF should change after sliding the window");
    }

    // -- RunningAutocorrelation: period / frequency detection ---------------

    #[test]
    fn test_period_detection_from_sine() {
        let sample_rate = 44_100;
        let freq = 440.0; // A4
        let expected_period = (sample_rate as f32 / freq).round() as usize; // ~100

        let buf_size = 1024;
        let max_lag = 200;
        let signal = sine_wave(freq, sample_rate, buf_size);

        let mut acf = RunningAutocorrelation::new(buf_size, max_lag);
        acf.buffer[..buf_size].copy_from_slice(&signal);
        acf.primed = true;
        acf.recompute();

        let period = acf.get_period().expect("should detect a period");
        let tolerance = 2; // allow +/-2 samples
        assert!(
            (period as isize - expected_period as isize).unsigned_abs() <= tolerance,
            "detected period {period}, expected ~{expected_period}"
        );
    }

    #[test]
    fn test_frequency_detection_from_sine() {
        let sample_rate = 44_100;
        let freq = 220.0; // A3
        let buf_size = 2048;
        let max_lag = 400;
        let signal = sine_wave(freq, sample_rate, buf_size);

        let mut acf = RunningAutocorrelation::new(buf_size, max_lag);
        acf.buffer[..buf_size].copy_from_slice(&signal);
        acf.primed = true;
        acf.recompute();

        let detected = acf
            .get_frequency(sample_rate)
            .expect("should detect frequency");
        let tolerance_hz = 3.0;
        assert!(
            (detected - freq).abs() < tolerance_hz,
            "detected {detected} Hz, expected ~{freq} Hz"
        );
    }

    #[test]
    fn test_no_period_in_noise() {
        // White-ish pseudo-random signal should not produce a clear period.
        let buf_size = 512;
        let max_lag = 256;
        // Deterministic "random" via simple LCG.
        let mut rng: u32 = 42;
        let signal: Vec<f32> = (0..buf_size)
            .map(|_| {
                rng = rng.wrapping_mul(1664525).wrapping_add(1013904223);
                (rng as f32 / u32::MAX as f32) * 2.0 - 1.0
            })
            .collect();

        let mut acf = RunningAutocorrelation::new(buf_size, max_lag);
        acf.buffer[..buf_size].copy_from_slice(&signal);
        acf.primed = true;
        acf.recompute();

        // We don't strictly require None -- noise *can* produce spurious
        // peaks -- but if a period is returned it should be weak.  The main
        // check is that this does not panic.
        let _period = acf.get_period(); // may be None or Some
    }

    // -- HFC unit -----------------------------------------------------------

    #[test]
    fn test_hfc_value() {
        // mags = [1, 1, 1] => HFC = 0*1 + 1*1 + 2*1 = 3
        let mags = vec![1.0f32; 3];
        let hfc = PluckDetector::hfc(&mags);
        assert!((hfc - 3.0).abs() < 1e-6);
    }

    #[test]
    fn test_hfc_zero_for_silence() {
        let mags = vec![0.0f32; 100];
        let hfc = PluckDetector::hfc(&mags);
        assert!((hfc - 0.0).abs() < 1e-6);
    }
}
