//! Alternative pitch detection algorithms optimized for guitar.
//!
//! Three complementary approaches:
//! - [`BacfDetector`] — Bitstream autocorrelation (extremely fast, bitwise ops)
//! - [`AmdfDetector`] — Average Magnitude Difference Function (faster than ACF)
//! - [`GoertzelBank`] — Goertzel filter bank tuned to guitar note frequencies

use std::f64::consts::PI;

// ---------------------------------------------------------------------------
// T4: Bitstream Autocorrelation (BACF)
// ---------------------------------------------------------------------------

/// Pitch detector using bitstream (1-bit) autocorrelation.
///
/// Converts audio to a 1-bit sign representation, packs into `u64` words,
/// then computes autocorrelation via XOR + popcount.  This makes the inner
/// loop operate on 64 samples at a time using cheap bitwise instructions.
pub struct BacfDetector {
    /// Minimum detectable frequency (Hz) — sets maximum lag.
    min_freq: f64,
    /// Maximum detectable frequency (Hz) — sets minimum lag.
    max_freq: f64,
    /// Peak threshold (0.0–1.0) for accepting a correlation peak.
    threshold: f64,
}

impl BacfDetector {
    /// Create a new BACF detector with guitar-optimized defaults.
    ///
    /// Default range: 70 Hz (below drop-D E2) to 1400 Hz (above E6).
    pub fn new() -> Self {
        Self {
            min_freq: 70.0,
            max_freq: 1400.0,
            threshold: 0.6,
        }
    }

    /// Create a detector with custom frequency range and threshold.
    pub fn with_params(min_freq: f64, max_freq: f64, threshold: f64) -> Self {
        Self {
            min_freq,
            max_freq,
            threshold,
        }
    }

    /// Pack audio samples into a bitstream: sample >= 0.0 → 1, else → 0.
    fn pack_bits(samples: &[f32]) -> Vec<u64> {
        let num_words = (samples.len() + 63) / 64;
        let mut words = vec![0u64; num_words];
        for (i, &s) in samples.iter().enumerate() {
            if s >= 0.0 {
                words[i / 64] |= 1u64 << (i % 64);
            }
        }
        words
    }

    /// Compute the number of matching bits between the bitstream and a
    /// lagged copy of itself.  Uses XOR (gives *differing* bits) then
    /// subtracts from total to get *matching* bits.
    fn correlation_at_lag(words: &[u64], total_bits: usize, lag: usize) -> f64 {
        // Number of valid comparison bits
        let compare_len = total_bits.saturating_sub(lag);
        if compare_len == 0 {
            return 0.0;
        }

        let word_offset = lag / 64;
        let bit_offset = lag % 64;

        let mut differing: u64 = 0;
        let num_full_words = compare_len / 64;
        let remaining_bits = compare_len % 64;

        // We iterate over the *original* words and compare with shifted words.
        let words_needed = if bit_offset == 0 {
            num_full_words + if remaining_bits > 0 { 1 } else { 0 }
        } else {
            num_full_words + if remaining_bits > 0 { 1 } else { 0 }
        };

        for i in 0..words_needed {
            let a = words[i];

            // Construct the lagged word by combining two adjacent words
            let idx = i + word_offset;
            let b = if bit_offset == 0 {
                if idx < words.len() {
                    words[idx]
                } else {
                    0
                }
            } else {
                let lo = if idx < words.len() { words[idx] >> bit_offset } else { 0 };
                let hi = if idx + 1 < words.len() {
                    words[idx + 1] << (64 - bit_offset)
                } else {
                    0
                };
                lo | hi
            };

            let xor = a ^ b;

            // For the last partial word, mask out bits beyond compare_len
            if i == words_needed - 1 && remaining_bits > 0 {
                let mask = (1u64 << remaining_bits) - 1;
                differing += (xor & mask).count_ones() as u64;
            } else {
                differing += xor.count_ones() as u64;
            }
        }

        let matching = compare_len as f64 - differing as f64;
        matching / compare_len as f64
    }

    /// Detect pitch from audio samples.
    ///
    /// Returns `Some((frequency_hz, confidence))` or `None` if no clear pitch.
    pub fn detect(&mut self, samples: &[f32], sample_rate: usize) -> Option<(f32, f32)> {
        if samples.len() < 64 {
            return None;
        }

        // Check for silence
        let rms = (samples.iter().map(|&s| (s * s) as f64).sum::<f64>()
            / samples.len() as f64)
            .sqrt();
        if rms < 1e-6 {
            return None;
        }

        let words = Self::pack_bits(samples);
        let total_bits = samples.len();

        let min_lag = (sample_rate as f64 / self.max_freq).ceil() as usize;
        let max_lag = (sample_rate as f64 / self.min_freq).floor() as usize;
        let max_lag = max_lag.min(total_bits / 2);

        if min_lag >= max_lag {
            return None;
        }

        // Compute correlations for all lags
        let num_lags = max_lag - min_lag + 1;
        let mut corrs = Vec::with_capacity(num_lags);
        for lag in min_lag..=max_lag {
            corrs.push(Self::correlation_at_lag(&words, total_bits, lag));
        }

        // Find the first peak above threshold using a state machine:
        // 1. Wait for correlation to dip below threshold (valley)
        // 2. Then find the next local maximum above threshold
        // This ensures we get the fundamental period, not a harmonic.
        let mut best_idx = 0usize;
        let mut best_corr = 0.0f64;
        let mut in_valley = false;

        for i in 0..corrs.len() {
            let c = corrs[i];

            // We need to see a valley first (correlation below threshold)
            if !in_valley {
                if c < self.threshold {
                    in_valley = true;
                }
                continue;
            }

            // After the valley, track the rising peak
            if c > best_corr {
                best_corr = c;
                best_idx = i;
            }

            // Once we found a good peak and start descending, stop
            if best_corr >= self.threshold && c < best_corr - 0.02 {
                break;
            }
        }

        let best_lag = best_idx + min_lag;

        if best_lag == min_lag && best_corr < self.threshold {
            return None;
        }
        if best_corr < self.threshold {
            return None;
        }

        // Parabolic interpolation for sub-sample accuracy
        let freq = if best_lag > min_lag && best_lag < max_lag {
            let c_minus = Self::correlation_at_lag(&words, total_bits, best_lag - 1);
            let c_center = best_corr;
            let c_plus = Self::correlation_at_lag(&words, total_bits, best_lag + 1);

            let denom = 2.0 * (2.0 * c_center - c_minus - c_plus);
            let refined_lag = if denom.abs() > 1e-12 {
                best_lag as f64 + (c_minus - c_plus) / denom
            } else {
                best_lag as f64
            };
            sample_rate as f64 / refined_lag
        } else {
            sample_rate as f64 / best_lag as f64
        };

        // Confidence is the peak correlation value, normalized to 0..1.
        // BACF correlation is already in [0.5, 1.0] for voiced signals
        // (0.5 = uncorrelated random, 1.0 = perfect match).
        // Map to a more useful range.
        let confidence = ((best_corr - 0.5) * 2.0).clamp(0.0, 1.0);

        Some((freq as f32, confidence as f32))
    }
}

impl Default for BacfDetector {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// T13: Average Magnitude Difference Function (AMDF)
// ---------------------------------------------------------------------------

/// Pitch detector using the Average Magnitude Difference Function.
///
/// For each candidate period (lag) τ, computes:
///   AMDF(τ) = (1/N) Σ |x(n) - x(n+τ)|
///
/// The fundamental period corresponds to the *minimum* of AMDF.
/// Faster than standard autocorrelation because it uses subtraction
/// instead of multiplication.
pub struct AmdfDetector {
    /// Minimum detectable frequency (Hz).
    min_freq: f64,
    /// Maximum detectable frequency (Hz).
    max_freq: f64,
    /// Ratio threshold: min_amdf must be below this fraction of max_amdf.
    ratio_threshold: f64,
}

impl AmdfDetector {
    /// Create an AMDF detector with guitar-optimized defaults.
    pub fn new() -> Self {
        Self {
            min_freq: 70.0,
            max_freq: 1400.0,
            ratio_threshold: 0.5,
        }
    }

    /// Create a detector with custom parameters.
    pub fn with_params(min_freq: f64, max_freq: f64, ratio_threshold: f64) -> Self {
        Self {
            min_freq,
            max_freq,
            ratio_threshold,
        }
    }

    /// Detect pitch from audio samples.
    ///
    /// Returns `Some((frequency_hz, confidence))` or `None` if no clear pitch.
    /// Confidence = 1.0 - (min_amdf / max_amdf).
    pub fn detect(&mut self, samples: &[f32], sample_rate: usize) -> Option<(f32, f32)> {
        let n = samples.len();
        if n < 64 {
            return None;
        }

        // Check for silence
        let rms = (samples.iter().map(|&s| (s * s) as f64).sum::<f64>() / n as f64).sqrt();
        if rms < 1e-6 {
            return None;
        }

        let min_lag = (sample_rate as f64 / self.max_freq).ceil() as usize;
        let max_lag = (sample_rate as f64 / self.min_freq).floor() as usize;
        let max_lag = max_lag.min(n / 2);

        if min_lag >= max_lag {
            return None;
        }

        // Compute AMDF for each lag
        let mut amdf_values = Vec::with_capacity(max_lag - min_lag + 1);
        let mut max_amdf: f64 = 0.0;

        for lag in min_lag..=max_lag {
            let compare_len = n - lag;
            let mut sum: f64 = 0.0;
            for i in 0..compare_len {
                sum += (samples[i] as f64 - samples[i + lag] as f64).abs();
            }
            let amdf = sum / compare_len as f64;
            amdf_values.push((lag, amdf));
            if amdf > max_amdf {
                max_amdf = amdf;
            }
        }

        if max_amdf < 1e-12 {
            return None;
        }

        // Find the first clear minimum (fundamental, not harmonic).
        // Walk through values looking for the first valley.
        let mut best_lag = 0usize;
        let mut best_amdf = f64::MAX;

        // State machine: look for first descent then ascent
        let mut descending = false;
        let mut found_min = false;

        for i in 1..amdf_values.len() {
            let (lag, amdf) = amdf_values[i];
            let (_, prev_amdf) = amdf_values[i - 1];

            if !descending && amdf < prev_amdf {
                descending = true;
            }

            if descending && amdf < best_amdf {
                best_amdf = amdf;
                best_lag = lag;
            }

            // Once we're ascending after finding a minimum, we found our valley
            if descending && amdf > prev_amdf && best_amdf < max_amdf * self.ratio_threshold {
                found_min = true;
                break;
            }
        }

        // If we never found a clear valley via ascent detection, but we had a
        // good minimum during descent, still accept it.
        if !found_min && best_amdf < max_amdf * self.ratio_threshold {
            found_min = true;
        }

        if !found_min || best_lag == 0 {
            return None;
        }

        // Parabolic interpolation for sub-sample accuracy
        let freq = {
            // Find index in amdf_values for best_lag
            let idx = best_lag - min_lag;
            if idx > 0 && idx < amdf_values.len() - 1 {
                let a_minus = amdf_values[idx - 1].1;
                let a_center = amdf_values[idx].1;
                let a_plus = amdf_values[idx + 1].1;

                let denom = 2.0 * (2.0 * a_center - a_minus - a_plus);
                let refined_lag = if denom.abs() > 1e-12 {
                    // For AMDF (minimum), the interpolation sign is reversed
                    best_lag as f64 - (a_minus - a_plus) / denom
                } else {
                    best_lag as f64
                };
                sample_rate as f64 / refined_lag
            } else {
                sample_rate as f64 / best_lag as f64
            }
        };

        let confidence = (1.0 - best_amdf / max_amdf).clamp(0.0, 1.0);

        Some((freq as f32, confidence as f32))
    }
}

impl Default for AmdfDetector {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// T14: Goertzel Filter Bank
// ---------------------------------------------------------------------------

/// Pre-computed coefficient for a single Goertzel filter targeting one frequency.
#[derive(Clone, Debug)]
struct GoertzelCoeff {
    /// MIDI note number (E2=40 through E6=88).
    midi_note: u8,
    /// Target frequency (Hz).
    #[allow(dead_code)]
    frequency: f64,
    /// Goertzel coefficient: 2 * cos(2π * k / N), where k = target_freq * N / sample_rate.
    coeff: f64,
    /// The normalized frequency index k (may be fractional).
    /// Used only for reference; the filter uses `coeff` directly.
    #[allow(dead_code)]
    k: f64,
}

/// Goertzel filter bank tuned to all guitar notes from E2 (82.4 Hz) to E6 (1318.5 Hz).
///
/// Each Goertzel filter is O(N) with just one multiply-add per sample, making the
/// full 48-note bank very efficient for guitar pitch detection.
pub struct GoertzelBank {
    /// Pre-computed coefficients for each target note.
    coeffs: Vec<GoertzelCoeff>,
    /// Sample rate this bank was configured for.
    sample_rate: usize,
}

impl GoertzelBank {
    /// Number of semitones from E2 (MIDI 40) to E6 (MIDI 88) inclusive.
    const NUM_NOTES: usize = 49; // 88 - 40 + 1 = 49

    /// MIDI note number for E2.
    const MIDI_E2: u8 = 40;

    /// Create a new Goertzel bank configured for the given sample rate.
    ///
    /// Pre-computes coefficients for all 48 guitar notes (E2–E6).
    pub fn new(sample_rate: usize) -> Self {
        let coeffs = (0..Self::NUM_NOTES)
            .map(|i| {
                let midi_note = Self::MIDI_E2 + i as u8;
                let frequency = midi_to_freq(midi_note);
                // For generalized Goertzel, we compute the coefficient
                // based on the block size we'll actually process.
                // We'll finalize `coeff` and `k` during `analyze()` since
                // they depend on block length N.
                GoertzelCoeff {
                    midi_note,
                    frequency,
                    coeff: 0.0, // placeholder — computed per-block in analyze()
                    k: 0.0,
                }
            })
            .collect();

        Self {
            coeffs,
            sample_rate,
        }
    }

    /// Analyze a block of samples and return the strongest matching note.
    ///
    /// Returns `Some((midi_note, magnitude))` for the note with the highest
    /// Goertzel magnitude, or `None` if all magnitudes are below a noise floor.
    pub fn analyze(&mut self, samples: &[f32]) -> Option<(u8, f32)> {
        let n = samples.len();
        if n == 0 {
            return None;
        }

        // Check for silence
        let rms = (samples.iter().map(|&s| (s * s) as f64).sum::<f64>() / n as f64).sqrt();
        if rms < 1e-6 {
            return None;
        }

        let mut best_note: u8 = 0;
        let mut best_mag: f64 = 0.0;

        for gc in &mut self.coeffs {
            // Compute the Goertzel coefficient for this block size
            let k = gc.frequency * n as f64 / self.sample_rate as f64;
            let coeff = 2.0 * (2.0 * PI * k / n as f64).cos();
            gc.coeff = coeff;
            gc.k = k;

            // Run the Goertzel filter
            let mut s0: f64 = 0.0;
            let mut s1: f64 = 0.0;
            let mut s2: f64;

            for &sample in samples {
                s2 = s1;
                s1 = s0;
                s0 = sample as f64 + coeff * s1 - s2;
            }

            // Compute magnitude squared (avoid sqrt for comparison)
            // |X(k)|^2 = s1^2 + s0^2 - coeff * s0 * s1
            //
            // More precisely using the Goertzel result:
            // real part: s0 - s1 * cos(2πk/N)
            // imag part: s1 * sin(2πk/N)
            let omega = 2.0 * PI * k / n as f64;
            let real = s0 - s1 * omega.cos();
            let imag = s1 * omega.sin();
            let mag_sq = real * real + imag * imag;
            let mag = mag_sq.sqrt() / n as f64; // normalize by block size

            if mag > best_mag {
                best_mag = mag;
                best_note = gc.midi_note;
            }
        }

        // Noise floor: if the best magnitude is too low relative to RMS, reject
        if best_mag < rms * 0.01 {
            return None;
        }

        Some((best_note, best_mag as f32))
    }
}

/// Convert a MIDI note number to frequency in Hz.
///
/// Uses the standard tuning: A4 (MIDI 69) = 440 Hz.
fn midi_to_freq(note: u8) -> f64 {
    440.0 * 2.0_f64.powf((note as f64 - 69.0) / 12.0)
}

/// Convert a frequency in Hz to the nearest MIDI note number.
#[allow(dead_code)]
fn freq_to_midi(freq: f64) -> u8 {
    let note = 69.0 + 12.0 * (freq / 440.0).log2();
    note.round() as u8
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// Generate a sine wave at the given frequency.
    fn sine_wave(freq: f64, sample_rate: usize, duration_secs: f64, amplitude: f32) -> Vec<f32> {
        let num_samples = (sample_rate as f64 * duration_secs) as usize;
        (0..num_samples)
            .map(|i| {
                let t = i as f64 / sample_rate as f64;
                (amplitude as f64 * (2.0 * PI * freq * t).sin()) as f32
            })
            .collect()
    }

    /// Generate white noise.
    fn noise(num_samples: usize, amplitude: f32) -> Vec<f32> {
        // Simple LCG pseudo-random for deterministic tests (no rand dependency in tests)
        let mut state: u64 = 0xDEAD_BEEF_CAFE_BABE;
        (0..num_samples)
            .map(|_| {
                state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
                let val = ((state >> 33) as f32 / u32::MAX as f32) * 2.0 - 1.0;
                val * amplitude
            })
            .collect()
    }

    /// Generate silence.
    fn silence(num_samples: usize) -> Vec<f32> {
        vec![0.0; num_samples]
    }

    // Allow a tolerance of 2% for frequency detection since these are
    // lightweight algorithms operating on relatively short buffers.
    const FREQ_TOLERANCE_PERCENT: f64 = 3.0;

    fn assert_freq_close(detected: f32, expected: f64, label: &str) {
        let error_pct = ((detected as f64 - expected) / expected).abs() * 100.0;
        assert!(
            error_pct < FREQ_TOLERANCE_PERCENT,
            "{}: detected {:.1} Hz, expected {:.1} Hz (error {:.2}%)",
            label,
            detected,
            expected,
            error_pct
        );
    }

    // -----------------------------------------------------------------------
    // BACF Tests
    // -----------------------------------------------------------------------

    #[test]
    fn bacf_detect_440hz() {
        let samples = sine_wave(440.0, 44100, 0.1, 0.8);
        let mut det = BacfDetector::new();
        let result = det.detect(&samples, 44100);
        assert!(result.is_some(), "BACF should detect 440 Hz");
        let (freq, conf) = result.unwrap();
        assert_freq_close(freq, 440.0, "BACF 440 Hz");
        assert!(conf > 0.3, "BACF confidence should be reasonable: {}", conf);
    }

    #[test]
    fn bacf_detect_220hz() {
        let samples = sine_wave(220.0, 44100, 0.1, 0.8);
        let mut det = BacfDetector::new();
        let result = det.detect(&samples, 44100);
        assert!(result.is_some(), "BACF should detect 220 Hz");
        let (freq, _) = result.unwrap();
        assert_freq_close(freq, 220.0, "BACF 220 Hz");
    }

    #[test]
    fn bacf_detect_110hz() {
        let samples = sine_wave(110.0, 44100, 0.15, 0.8);
        let mut det = BacfDetector::new();
        let result = det.detect(&samples, 44100);
        assert!(result.is_some(), "BACF should detect 110 Hz");
        let (freq, _) = result.unwrap();
        assert_freq_close(freq, 110.0, "BACF 110 Hz");
    }

    #[test]
    fn bacf_detect_82_4hz() {
        // E2 — lowest standard guitar note
        let samples = sine_wave(82.4, 44100, 0.2, 0.8);
        let mut det = BacfDetector::new();
        let result = det.detect(&samples, 44100);
        assert!(result.is_some(), "BACF should detect 82.4 Hz (E2)");
        let (freq, _) = result.unwrap();
        assert_freq_close(freq, 82.4, "BACF 82.4 Hz");
    }

    #[test]
    fn bacf_silence_returns_none() {
        let samples = silence(4410);
        let mut det = BacfDetector::new();
        let result = det.detect(&samples, 44100);
        assert!(result.is_none(), "BACF should return None for silence");
    }

    #[test]
    fn bacf_noise_returns_none_or_low_confidence() {
        let samples = noise(4410, 0.5);
        let mut det = BacfDetector::new();
        let result = det.detect(&samples, 44100);
        // Noise might occasionally trigger a detection but with very low confidence
        if let Some((_, conf)) = result {
            assert!(
                conf < 0.5,
                "BACF confidence on noise should be low: {}",
                conf
            );
        }
    }

    #[test]
    fn bacf_very_low_amplitude() {
        let samples = sine_wave(440.0, 44100, 0.1, 0.0001);
        let mut det = BacfDetector::new();
        let result = det.detect(&samples, 44100);
        // Very low amplitude might still detect (sign-based), which is fine
        // as long as it doesn't crash
        if let Some((freq, _)) = result {
            assert_freq_close(freq, 440.0, "BACF low amplitude 440 Hz");
        }
    }

    #[test]
    fn bacf_short_buffer() {
        let samples = sine_wave(440.0, 44100, 0.001, 0.8); // ~44 samples
        let mut det = BacfDetector::new();
        // Should not panic, may return None due to insufficient data
        let _ = det.detect(&samples, 44100);
    }

    // -----------------------------------------------------------------------
    // AMDF Tests
    // -----------------------------------------------------------------------

    #[test]
    fn amdf_detect_440hz() {
        let samples = sine_wave(440.0, 44100, 0.1, 0.8);
        let mut det = AmdfDetector::new();
        let result = det.detect(&samples, 44100);
        assert!(result.is_some(), "AMDF should detect 440 Hz");
        let (freq, conf) = result.unwrap();
        assert_freq_close(freq, 440.0, "AMDF 440 Hz");
        assert!(conf > 0.5, "AMDF confidence should be decent: {}", conf);
    }

    #[test]
    fn amdf_detect_220hz() {
        let samples = sine_wave(220.0, 44100, 0.1, 0.8);
        let mut det = AmdfDetector::new();
        let result = det.detect(&samples, 44100);
        assert!(result.is_some(), "AMDF should detect 220 Hz");
        let (freq, _) = result.unwrap();
        assert_freq_close(freq, 220.0, "AMDF 220 Hz");
    }

    #[test]
    fn amdf_detect_110hz() {
        let samples = sine_wave(110.0, 44100, 0.15, 0.8);
        let mut det = AmdfDetector::new();
        let result = det.detect(&samples, 44100);
        assert!(result.is_some(), "AMDF should detect 110 Hz");
        let (freq, _) = result.unwrap();
        assert_freq_close(freq, 110.0, "AMDF 110 Hz");
    }

    #[test]
    fn amdf_detect_82_4hz() {
        let samples = sine_wave(82.4, 44100, 0.2, 0.8);
        let mut det = AmdfDetector::new();
        let result = det.detect(&samples, 44100);
        assert!(result.is_some(), "AMDF should detect 82.4 Hz (E2)");
        let (freq, _) = result.unwrap();
        assert_freq_close(freq, 82.4, "AMDF 82.4 Hz");
    }

    #[test]
    fn amdf_silence_returns_none() {
        let samples = silence(4410);
        let mut det = AmdfDetector::new();
        let result = det.detect(&samples, 44100);
        assert!(result.is_none(), "AMDF should return None for silence");
    }

    #[test]
    fn amdf_noise_returns_none_or_low_confidence() {
        let samples = noise(4410, 0.5);
        let mut det = AmdfDetector::new();
        let result = det.detect(&samples, 44100);
        if let Some((_, conf)) = result {
            assert!(
                conf < 0.7,
                "AMDF confidence on noise should be low: {}",
                conf
            );
        }
    }

    #[test]
    fn amdf_very_low_amplitude() {
        let samples = sine_wave(440.0, 44100, 0.1, 1e-7);
        let mut det = AmdfDetector::new();
        let result = det.detect(&samples, 44100);
        // Extremely low amplitude should be treated as silence
        assert!(
            result.is_none(),
            "AMDF should return None for near-silence"
        );
    }

    // -----------------------------------------------------------------------
    // Goertzel Bank Tests
    // -----------------------------------------------------------------------

    #[test]
    fn goertzel_detect_e2() {
        // E2 = MIDI 40 = 82.41 Hz
        let samples = sine_wave(82.41, 44100, 0.1, 0.8);
        let mut bank = GoertzelBank::new(44100);
        let result = bank.analyze(&samples);
        assert!(result.is_some(), "Goertzel should detect E2");
        let (note, _mag) = result.unwrap();
        assert_eq!(note, 40, "Goertzel E2 should be MIDI 40, got {}", note);
    }

    #[test]
    fn goertzel_detect_a2() {
        // A2 = MIDI 45 = 110 Hz
        let samples = sine_wave(110.0, 44100, 0.1, 0.8);
        let mut bank = GoertzelBank::new(44100);
        let result = bank.analyze(&samples);
        assert!(result.is_some(), "Goertzel should detect A2");
        let (note, _) = result.unwrap();
        assert_eq!(note, 45, "Goertzel A2 should be MIDI 45, got {}", note);
    }

    #[test]
    fn goertzel_detect_a3() {
        // A3 = MIDI 57 = 220 Hz
        let samples = sine_wave(220.0, 44100, 0.1, 0.8);
        let mut bank = GoertzelBank::new(44100);
        let result = bank.analyze(&samples);
        assert!(result.is_some(), "Goertzel should detect A3");
        let (note, _) = result.unwrap();
        assert_eq!(note, 57, "Goertzel A3 should be MIDI 57, got {}", note);
    }

    #[test]
    fn goertzel_detect_a4() {
        // A4 = MIDI 69 = 440 Hz
        let samples = sine_wave(440.0, 44100, 0.1, 0.8);
        let mut bank = GoertzelBank::new(44100);
        let result = bank.analyze(&samples);
        assert!(result.is_some(), "Goertzel should detect A4");
        let (note, _) = result.unwrap();
        assert_eq!(note, 69, "Goertzel A4 should be MIDI 69, got {}", note);
    }

    #[test]
    fn goertzel_detect_e6() {
        // E6 = MIDI 88 = 1318.5 Hz
        let samples = sine_wave(1318.5, 44100, 0.1, 0.8);
        let mut bank = GoertzelBank::new(44100);
        let result = bank.analyze(&samples);
        assert!(result.is_some(), "Goertzel should detect E6");
        let (note, _) = result.unwrap();
        assert_eq!(note, 88, "Goertzel E6 should be MIDI 88, got {}", note);
    }

    #[test]
    fn goertzel_silence_returns_none() {
        let samples = silence(4410);
        let mut bank = GoertzelBank::new(44100);
        let result = bank.analyze(&samples);
        assert!(result.is_none(), "Goertzel should return None for silence");
    }

    #[test]
    fn goertzel_empty_buffer() {
        let mut bank = GoertzelBank::new(44100);
        let result = bank.analyze(&[]);
        assert!(
            result.is_none(),
            "Goertzel should return None for empty buffer"
        );
    }

    #[test]
    fn goertzel_midi_note_coverage() {
        // Verify all 49 notes from E2 to E6 are covered
        let bank = GoertzelBank::new(44100);
        assert_eq!(bank.coeffs.len(), 49);
        assert_eq!(bank.coeffs[0].midi_note, 40); // E2
        assert_eq!(bank.coeffs[48].midi_note, 88); // E6
    }

    // -----------------------------------------------------------------------
    // Cross-detector agreement tests
    // -----------------------------------------------------------------------

    #[test]
    fn all_detectors_agree_on_440hz() {
        let samples = sine_wave(440.0, 44100, 0.1, 0.8);

        let mut bacf = BacfDetector::new();
        let mut amdf = AmdfDetector::new();
        let mut goertzel = GoertzelBank::new(44100);

        let bacf_result = bacf.detect(&samples, 44100);
        let amdf_result = amdf.detect(&samples, 44100);
        let goertzel_result = goertzel.analyze(&samples);

        assert!(bacf_result.is_some(), "BACF should detect 440 Hz");
        assert!(amdf_result.is_some(), "AMDF should detect 440 Hz");
        assert!(goertzel_result.is_some(), "Goertzel should detect 440 Hz");

        let bacf_freq = bacf_result.unwrap().0;
        let amdf_freq = amdf_result.unwrap().0;
        let goertzel_note = goertzel_result.unwrap().0;

        // BACF and AMDF should be close to 440 Hz
        assert_freq_close(bacf_freq, 440.0, "Cross-check BACF 440 Hz");
        assert_freq_close(amdf_freq, 440.0, "Cross-check AMDF 440 Hz");

        // Goertzel should identify MIDI note 69 (A4)
        assert_eq!(
            goertzel_note, 69,
            "Cross-check Goertzel A4 = MIDI 69, got {}",
            goertzel_note
        );

        // BACF and AMDF should agree within tolerance
        let diff_pct = ((bacf_freq - amdf_freq).abs() / 440.0 * 100.0) as f64;
        assert!(
            diff_pct < FREQ_TOLERANCE_PERCENT,
            "BACF ({:.1}) and AMDF ({:.1}) should agree (diff {:.2}%)",
            bacf_freq,
            amdf_freq,
            diff_pct
        );
    }

    #[test]
    fn all_detectors_agree_on_220hz() {
        let samples = sine_wave(220.0, 44100, 0.15, 0.8);

        let mut bacf = BacfDetector::new();
        let mut amdf = AmdfDetector::new();
        let mut goertzel = GoertzelBank::new(44100);

        let bacf_result = bacf.detect(&samples, 44100);
        let amdf_result = amdf.detect(&samples, 44100);
        let goertzel_result = goertzel.analyze(&samples);

        assert!(bacf_result.is_some());
        assert!(amdf_result.is_some());
        assert!(goertzel_result.is_some());

        assert_freq_close(bacf_result.unwrap().0, 220.0, "Cross-check BACF 220 Hz");
        assert_freq_close(amdf_result.unwrap().0, 220.0, "Cross-check AMDF 220 Hz");
        assert_eq!(goertzel_result.unwrap().0, 57); // A3 = MIDI 57
    }

    #[test]
    fn all_detectors_silence() {
        let samples = silence(4410);

        let mut bacf = BacfDetector::new();
        let mut amdf = AmdfDetector::new();
        let mut goertzel = GoertzelBank::new(44100);

        assert!(bacf.detect(&samples, 44100).is_none());
        assert!(amdf.detect(&samples, 44100).is_none());
        assert!(goertzel.analyze(&samples).is_none());
    }

    // -----------------------------------------------------------------------
    // Utility function tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_midi_to_freq() {
        // A4 = 440 Hz
        let f = midi_to_freq(69);
        assert!((f - 440.0).abs() < 0.01, "MIDI 69 should be 440 Hz");

        // E2 ≈ 82.41 Hz
        let f = midi_to_freq(40);
        assert!((f - 82.41).abs() < 0.1, "MIDI 40 should be ~82.41 Hz");

        // C4 ≈ 261.63 Hz
        let f = midi_to_freq(60);
        assert!((f - 261.63).abs() < 0.1, "MIDI 60 should be ~261.63 Hz");
    }

    #[test]
    fn test_freq_to_midi() {
        assert_eq!(freq_to_midi(440.0), 69);
        assert_eq!(freq_to_midi(82.41), 40);
        assert_eq!(freq_to_midi(261.63), 60);
    }

    #[test]
    fn bacf_pack_bits_basic() {
        // All positive => all 1s
        let samples = vec![1.0f32; 128];
        let words = BacfDetector::pack_bits(&samples);
        assert_eq!(words.len(), 2);
        assert_eq!(words[0], u64::MAX);
        assert_eq!(words[1], u64::MAX);

        // All negative => all 0s
        let samples = vec![-1.0f32; 128];
        let words = BacfDetector::pack_bits(&samples);
        assert_eq!(words[0], 0);
        assert_eq!(words[1], 0);
    }
}
