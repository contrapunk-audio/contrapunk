//! DSP-based guitar input pipeline: pitch detection, string/fret identification, MIDI output.
//!
//! Pure DSP approach using McLeod pitch detection + inharmonicity-based string
//! identification. No ML -- relies on Goertzel harmonic analysis and calibrated
//! per-string inharmonicity coefficients to disambiguate which string produced
//! a given pitch.
//!
//! # Pipeline
//!
//! 1. **Onset detection** -- RMS spike with cooldown
//! 2. **Pitch detection** -- McLeod (from `pitch_detection` crate)
//! 3. **Harmonic measurement** -- Goertzel at integer multiples of fundamental
//! 4. **String identification** -- inharmonicity B-coefficient matching
//! 5. **Bend tracking** -- continuous pitch monitoring between onsets
//! 6. **MIDI event generation** -- NoteOn / NoteOff / PitchBend

use super::guitar::{midi_to_freq, midi_to_note_name, STRING_BASE_PITCH, STRING_NAMES};
use super::pitch::freq_to_midi;

// ── Configuration ──────────────────────────────────────────────────

/// User-adjustable parameters for the guitar input pipeline.
#[derive(Clone, Debug)]
pub struct GuitarInputConfig {
    /// Analysis window in samples (256-2048).
    pub buffer_size: usize,
    /// Audio sample rate (typically 48000).
    pub sample_rate: usize,
    /// RMS threshold for pluck / onset detection.
    pub onset_threshold: f32,
    /// Minimum confidence for string identification.
    pub string_confidence_min: f32,
    /// Enable pitch-bend tracking between onsets.
    pub bends_enabled: bool,
    /// Enable legato detection (hammer-on / pull-off).
    pub legato_enabled: bool,
    /// Enable low-pass pre-filter on input.
    pub filter_enabled: bool,
    /// Minimum McLeod clarity to accept a pitch detection.
    pub min_clarity: f64,
    /// Cooldown in samples after an onset before another can fire.
    pub cooldown_samples: usize,
    /// Number of harmonics to measure for inharmonicity.
    pub n_harmonics: usize,
}

impl Default for GuitarInputConfig {
    fn default() -> Self {
        Self {
            buffer_size: 1024,
            sample_rate: 48000,
            onset_threshold: 0.02,
            string_confidence_min: 0.5,
            bends_enabled: true,
            legato_enabled: false,
            filter_enabled: false,
            min_clarity: 0.40,
            cooldown_samples: 4800, // 100ms @ 48kHz
            n_harmonics: 6,
        }
    }
}

// ── Calibration types ──────────────────────────────────────────────

/// Per-string inharmonicity profile measured during calibration.
#[derive(Clone, Debug)]
pub struct StringProfile {
    /// Display name, e.g. "E2".
    pub name: &'static str,
    /// Open string MIDI note (40, 45, 50, 55, 59, 64).
    pub open_midi: u8,
    /// Measured open-string frequency in Hz.
    pub open_frequency: f32,
    /// Measured inharmonicity B coefficient.
    pub inharmonicity_b: f32,
    /// First N harmonic magnitude ratios (h2/h1, h3/h1, ...).
    pub harmonic_ratios: Vec<f32>,
}

/// Full guitar calibration data.
#[derive(Clone, Debug)]
pub struct GuitarCalibration {
    /// Measured noise floor RMS.
    pub noise_floor: f32,
    /// Measured signal peak during calibration.
    pub signal_peak: f32,
    /// Per-string profiles (6 strings, low E to high E).
    pub string_profiles: [StringProfile; 6],
}

// ── Detected note ──────────────────────────────────────────────────

/// A fully resolved note detection with string/fret identification.
#[derive(Clone, Debug)]
pub struct DetectedNote {
    /// MIDI note number (0-127).
    pub midi_note: u8,
    /// Detected frequency in Hz.
    pub frequency: f32,
    /// Identified string index (0-5), if determined.
    pub string_idx: Option<usize>,
    /// Identified fret (0-22), if determined.
    pub fret: Option<usize>,
    /// Confidence of string identification (0.0-1.0).
    pub string_confidence: f32,
    /// Velocity (0-127) derived from onset RMS.
    pub velocity: u8,
    /// Pitch bend in cents from nearest semitone.
    pub bend_cents: i16,
}

// ── MIDI events ────────────────────────────────────────────────────

/// MIDI events produced by the pipeline.
#[derive(Clone, Debug, PartialEq)]
pub enum MidiEvent {
    NoteOn {
        channel: u8,
        note: u8,
        velocity: u8,
    },
    NoteOff {
        channel: u8,
        note: u8,
    },
    PitchBend {
        channel: u8,
        /// Bend in cents from current note.
        cents: i16,
    },
}

// ── Core pipeline ──────────────────────────────────────────────────

/// The main guitar input processor.
///
/// Feed audio blocks via [`process_block`] and receive MIDI events.
/// Optionally calibrate per-string profiles for inharmonicity-based
/// string identification.
pub struct GuitarInput {
    config: GuitarInputConfig,
    calibration: Option<GuitarCalibration>,

    // State
    current_note: Option<DetectedNote>,
    prev_rms: f32,
    cooldown_remaining: usize,
    ring_buffer: Vec<f32>,
    ring_pos: usize,
}

impl GuitarInput {
    /// Create a new pipeline with the given configuration.
    pub fn new(config: GuitarInputConfig) -> Self {
        let buf_size = config.buffer_size;
        Self {
            config,
            calibration: None,
            current_note: None,
            prev_rms: 0.0,
            cooldown_remaining: 0,
            ring_buffer: vec![0.0; buf_size],
            ring_pos: 0,
        }
    }

    /// Create with default configuration.
    pub fn with_defaults() -> Self {
        Self::new(GuitarInputConfig::default())
    }

    /// Set the calibration data (measured from `calibrate_string`).
    pub fn set_calibration(&mut self, cal: GuitarCalibration) {
        self.calibration = Some(cal);
    }

    /// Get current calibration, if any.
    pub fn calibration(&self) -> Option<&GuitarCalibration> {
        self.calibration.as_ref()
    }

    /// Get current config.
    pub fn config(&self) -> &GuitarInputConfig {
        &self.config
    }

    /// Get the currently active note, if any.
    pub fn current_note(&self) -> Option<&DetectedNote> {
        self.current_note.as_ref()
    }

    // ── Main processing ────────────────────────────────────────────

    /// Process a block of mono audio samples and return any MIDI events.
    ///
    /// Call this repeatedly with successive audio buffers. The block size
    /// does not have to match `config.buffer_size` -- the internal ring
    /// buffer accumulates samples and triggers analysis when full.
    pub fn process_block(&mut self, audio: &[f32]) -> Vec<MidiEvent> {
        let mut events = Vec::new();

        for &sample in audio {
            self.ring_buffer[self.ring_pos] = sample;
            self.ring_pos = (self.ring_pos + 1) % self.config.buffer_size;

            // Only analyze when the ring buffer wraps (one full window)
            if self.ring_pos == 0 {
                let analysis_events = self.analyze_window();
                events.extend(analysis_events);
            }
        }

        events
    }

    /// Analyze the current ring buffer contents.
    fn analyze_window(&mut self) -> Vec<MidiEvent> {
        let mut events = Vec::new();
        let buf = self.linearize_ring_buffer();

        // 1. Compute RMS for onset detection
        let rms = compute_rms(&buf);

        // 2. Onset detection with cooldown
        let onset = self.detect_onset(rms);

        // 3. Pitch detection (McLeod)
        let pitch_result = detect_pitch_mcleod(
            &buf,
            self.config.sample_rate,
            self.config.min_clarity,
        );

        // Tick down cooldown
        if self.cooldown_remaining > 0 {
            self.cooldown_remaining = self.cooldown_remaining
                .saturating_sub(self.config.buffer_size);
        }

        match pitch_result {
            Some((freq, _clarity)) => {
                let (midi_note, cents) = freq_to_midi(freq);

                if onset {
                    // New note onset
                    // End previous note first
                    if let Some(prev) = &self.current_note {
                        events.push(MidiEvent::NoteOff {
                            channel: 0,
                            note: prev.midi_note,
                        });
                    }

                    // Velocity from RMS (map to 1-127)
                    let velocity = rms_to_velocity(rms);

                    // String identification
                    let (string_idx, fret, string_conf) =
                        self.identify_string_and_fret(&buf, freq, midi_note);

                    let detected = DetectedNote {
                        midi_note,
                        frequency: freq,
                        string_idx,
                        fret,
                        string_confidence: string_conf,
                        velocity,
                        bend_cents: cents as i16,
                    };

                    events.push(MidiEvent::NoteOn {
                        channel: 0,
                        note: midi_note,
                        velocity,
                    });

                    self.current_note = Some(detected);
                    self.cooldown_remaining = self.config.cooldown_samples;
                } else if self.config.bends_enabled {
                    // Sustain phase: track pitch bends
                    if let Some(ref mut note) = self.current_note {
                        if midi_note == note.midi_note {
                            let new_bend = cents as i16;
                            if (new_bend - note.bend_cents).abs() > 5 {
                                events.push(MidiEvent::PitchBend {
                                    channel: 0,
                                    cents: new_bend,
                                });
                                note.bend_cents = new_bend;
                            }
                        }
                    }
                }
            }
            None => {
                // No pitch detected -- check for note-off
                let noise_floor = self
                    .calibration
                    .as_ref()
                    .map(|c| c.noise_floor)
                    .unwrap_or(0.001);

                if rms < noise_floor * 2.0 {
                    if let Some(prev) = self.current_note.take() {
                        events.push(MidiEvent::NoteOff {
                            channel: 0,
                            note: prev.midi_note,
                        });
                    }
                }
            }
        }

        self.prev_rms = rms;
        events
    }

    /// Detect whether the current frame is an onset (pluck).
    fn detect_onset(&self, rms: f32) -> bool {
        if self.cooldown_remaining > 0 {
            return false;
        }
        // Onset = RMS jumped above threshold and is significantly higher
        // than the previous frame.
        rms > self.config.onset_threshold && rms > self.prev_rms * 2.0
    }

    /// Identify string and fret from detected pitch + harmonic analysis.
    fn identify_string_and_fret(
        &self,
        audio: &[f32],
        fundamental: f32,
        midi_note: u8,
    ) -> (Option<usize>, Option<usize>, f32) {
        if let Some(ref cal) = self.calibration {
            match identify_string(
                audio,
                fundamental,
                midi_note,
                &cal.string_profiles,
                self.config.sample_rate,
                self.config.n_harmonics,
                self.config.string_confidence_min,
            ) {
                Some((string_idx, confidence)) => {
                    let fret = midi_note
                        .checked_sub(cal.string_profiles[string_idx].open_midi)
                        .map(|f| f as usize);
                    (Some(string_idx), fret, confidence)
                }
                None => {
                    // Fall back to simple pitch-based heuristic
                    let (s, f) = simple_string_fret(midi_note);
                    (Some(s), Some(f), 0.3)
                }
            }
        } else {
            // No calibration -- use simple heuristic
            let (s, f) = simple_string_fret(midi_note);
            (Some(s), Some(f), 0.3)
        }
    }

    /// Copy the ring buffer into a linear buffer (ordered oldest to newest).
    fn linearize_ring_buffer(&self) -> Vec<f32> {
        let mut buf = Vec::with_capacity(self.config.buffer_size);
        for i in 0..self.config.buffer_size {
            buf.push(self.ring_buffer[(self.ring_pos + i) % self.config.buffer_size]);
        }
        buf
    }

    // ── Calibration ────────────────────────────────────────────────

    /// Calibrate a single string from captured open-string audio.
    ///
    /// Extracts fundamental frequency, measures harmonics via Goertzel,
    /// and computes the inharmonicity B coefficient.
    pub fn calibrate_string(
        &self,
        string_idx: usize,
        audio: &[f32],
    ) -> Option<StringProfile> {
        if string_idx >= 6 || audio.len() < self.config.buffer_size {
            return None;
        }

        // Detect pitch from the audio
        let (freq, clarity) = detect_pitch_mcleod(
            audio,
            self.config.sample_rate,
            0.3, // lower threshold for calibration
        )?;

        if clarity < 0.3 {
            return None;
        }

        // Measure harmonics
        let harmonics = measure_harmonics(
            audio,
            freq,
            self.config.n_harmonics,
            self.config.sample_rate,
        );

        // Compute inharmonicity B
        let b = compute_inharmonicity_b(audio, freq, self.config.sample_rate);

        // Compute harmonic ratios (h_n / h_1)
        let h1 = harmonics.first().copied().unwrap_or(1.0).max(1e-10);
        let ratios: Vec<f32> = harmonics.iter().skip(1).map(|&h| h / h1).collect();

        Some(StringProfile {
            name: STRING_NAMES.get(string_idx).copied().unwrap_or("?"),
            open_midi: STRING_BASE_PITCH
                .get(string_idx)
                .copied()
                .unwrap_or(40),
            open_frequency: freq,
            inharmonicity_b: b,
            harmonic_ratios: ratios,
        })
    }

    /// Measure the noise floor from a silence capture.
    pub fn measure_noise_floor(audio: &[f32]) -> f32 {
        compute_rms(audio)
    }
}

// ── Goertzel algorithm ─────────────────────────────────────────────

/// Single-frequency magnitude estimation via the Goertzel algorithm.
///
/// Computes the magnitude of the DFT at `target_freq` without a full FFT.
/// O(N) per frequency, ideal for measuring individual harmonics.
pub fn goertzel(samples: &[f32], sample_rate: usize, target_freq: f32) -> f32 {
    let n = samples.len();
    if n == 0 {
        return 0.0;
    }
    let k = (0.5 + (n as f32 * target_freq / sample_rate as f32)) as usize;
    let w = 2.0 * std::f32::consts::PI * k as f32 / n as f32;
    let coeff = 2.0 * w.cos();
    let (mut s1, mut s2) = (0.0f32, 0.0f32);
    for &sample in samples {
        let s0 = coeff * s1 - s2 + sample;
        s2 = s1;
        s1 = s0;
    }
    (s1 * s1 + s2 * s2 - coeff * s1 * s2).sqrt()
}

// ── Harmonic measurement ───────────────────────────────────────────

/// Measure magnitudes of the first N harmonics of a fundamental frequency.
///
/// For each harmonic n = 1..=n_harmonics, uses Goertzel at the target
/// frequency (with a small search window of +/-1% to handle slight
/// inharmonicity). Returns magnitudes normalized to h1.
pub fn measure_harmonics(
    audio: &[f32],
    fundamental: f32,
    n_harmonics: usize,
    sample_rate: usize,
) -> Vec<f32> {
    let mut magnitudes = Vec::with_capacity(n_harmonics);
    for n in 1..=n_harmonics {
        let target = fundamental * n as f32;
        // Search +/- 1% around the expected harmonic
        let lo = target * 0.99;
        let hi = target * 1.01;
        let steps = 5;
        let mut best_mag = 0.0f32;
        for i in 0..=steps {
            let freq = lo + (hi - lo) * i as f32 / steps as f32;
            let mag = goertzel(audio, sample_rate, freq);
            if mag > best_mag {
                best_mag = mag;
            }
        }
        magnitudes.push(best_mag);
    }
    magnitudes
}

/// Compute the inharmonicity B coefficient from audio.
///
/// B is estimated from the deviation of measured harmonic frequencies
/// from perfect integer multiples of the fundamental:
///
///   f_n = n * f_1 * sqrt(1 + B * n^2)
///
/// Rearranging: B = ((f_n / (n * f_1))^2 - 1) / n^2
///
/// We average B across harmonics 2..6 for robustness.
pub fn compute_inharmonicity_b(audio: &[f32], fundamental: f32, sample_rate: usize) -> f32 {
    let mut b_sum = 0.0f64;
    let mut count = 0;

    for n in 2..=6 {
        let expected = fundamental * n as f32;
        // Search +/-3% for the actual harmonic peak
        let lo = expected * 0.97;
        let hi = expected * 1.03;
        let steps = 20;
        let mut best_freq = expected;
        let mut best_mag = 0.0f32;
        for i in 0..=steps {
            let freq = lo + (hi - lo) * i as f32 / steps as f32;
            let mag = goertzel(audio, sample_rate, freq);
            if mag > best_mag {
                best_mag = mag;
                best_freq = freq;
            }
        }

        if best_mag > 0.0 {
            let ratio = best_freq as f64 / (n as f64 * fundamental as f64);
            let b_n = (ratio * ratio - 1.0) / (n as f64 * n as f64);
            if b_n >= 0.0 && b_n < 0.1 {
                // Sanity check: B is typically 0.0001-0.01 for guitar
                b_sum += b_n;
                count += 1;
            }
        }
    }

    if count > 0 {
        (b_sum / count as f64) as f32
    } else {
        0.0
    }
}

// ── String identification ──────────────────────────────────────────

/// Identify which string produced a note by comparing measured
/// inharmonicity against calibrated per-string B coefficients.
///
/// For each candidate string:
///   1. Check that the MIDI note is playable (>= open, <= open+22)
///   2. Compute expected B at the detected fret position
///   3. Compare with measured B from the audio
///   4. Return the best match above the confidence threshold
///
/// Returns `(string_idx, confidence)` or `None`.
pub fn identify_string(
    audio: &[f32],
    fundamental: f32,
    midi_note: u8,
    profiles: &[StringProfile; 6],
    sample_rate: usize,
    n_harmonics: usize,
    confidence_min: f32,
) -> Option<(usize, f32)> {
    // Measure actual inharmonicity from the audio
    let measured_b = compute_inharmonicity_b(audio, fundamental, sample_rate);

    // Also measure harmonic ratios for spectral shape matching
    let measured_harmonics = measure_harmonics(audio, fundamental, n_harmonics, sample_rate);
    let h1 = measured_harmonics.first().copied().unwrap_or(1.0).max(1e-10);
    let measured_ratios: Vec<f32> = measured_harmonics
        .iter()
        .skip(1)
        .map(|&h| h / h1)
        .collect();

    let mut best_string: Option<usize> = None;
    let mut best_confidence = 0.0f32;

    for (idx, profile) in profiles.iter().enumerate() {
        // Plausibility: note must be playable on this string
        if midi_note < profile.open_midi || midi_note > profile.open_midi + 22 {
            continue;
        }

        let fret = midi_note - profile.open_midi;

        // Inharmonicity increases with fret position (shorter string).
        // B(fret) ~ B(open) * (L_open / L_fret)^2
        // L_fret / L_open = 2^(-fret/12), so L_open/L_fret = 2^(fret/12)
        // B(fret) = B(open) * 2^(fret/6)
        let expected_b = profile.inharmonicity_b * 2.0f32.powf(fret as f32 / 6.0);

        // B similarity: closer is better (log scale, since B can vary by orders)
        let b_ratio = if measured_b > 0.0 && expected_b > 0.0 {
            let log_ratio = (measured_b.ln() - expected_b.ln()).abs();
            (-log_ratio * 2.0).exp() // 1.0 = perfect match, decays with difference
        } else {
            0.3 // no B data, neutral score
        };

        // Harmonic ratio similarity (cosine-like distance)
        let harmonic_sim = if !profile.harmonic_ratios.is_empty()
            && !measured_ratios.is_empty()
        {
            let n = profile.harmonic_ratios.len().min(measured_ratios.len());
            let mut dot = 0.0f32;
            let mut mag_a = 0.0f32;
            let mut mag_b = 0.0f32;
            for i in 0..n {
                dot += profile.harmonic_ratios[i] * measured_ratios[i];
                mag_a += profile.harmonic_ratios[i] * profile.harmonic_ratios[i];
                mag_b += measured_ratios[i] * measured_ratios[i];
            }
            let denom = (mag_a.sqrt() * mag_b.sqrt()).max(1e-10);
            (dot / denom).clamp(0.0, 1.0)
        } else {
            0.5 // no harmonic data, neutral
        };

        // Fret preference: prefer lower fret positions
        let fret_bonus = 1.0 - (fret as f32 * 0.02).min(0.3);

        // Combined confidence
        let confidence = b_ratio * 0.5 + harmonic_sim * 0.3 + fret_bonus * 0.2;

        if confidence > best_confidence && confidence >= confidence_min {
            best_confidence = confidence;
            best_string = Some(idx);
        }
    }

    best_string.map(|s| (s, best_confidence))
}

// ── Utility functions ──────────────────────────────────────────────

/// Compute RMS of an audio buffer.
pub fn compute_rms(samples: &[f32]) -> f32 {
    if samples.is_empty() {
        return 0.0;
    }
    let sum_sq: f64 = samples.iter().map(|&s| (s as f64) * (s as f64)).sum();
    (sum_sq / samples.len() as f64).sqrt() as f32
}

/// Convert RMS to MIDI velocity (1-127).
pub fn rms_to_velocity(rms: f32) -> u8 {
    // Map RMS range [0.01, 0.5] to velocity [30, 127].
    let normalized = ((rms - 0.01) / 0.49).clamp(0.0, 1.0);
    let vel = 30.0 + normalized * 97.0;
    (vel as u8).clamp(1, 127)
}

/// Convert frequency to MIDI note and cents (re-export for convenience).
pub fn freq_to_midi_note(freq: f32) -> (u8, i8) {
    freq_to_midi(freq)
}

/// Simple heuristic string/fret identification (no calibration needed).
/// Prefers lower fret positions.
pub fn simple_string_fret(midi_note: u8) -> (usize, usize) {
    // Find the lowest string that can play this note
    for (idx, &open) in STRING_BASE_PITCH.iter().enumerate().rev() {
        if midi_note >= open && midi_note <= open + 22 {
            return (idx, (midi_note - open) as usize);
        }
    }
    // Fallback: assign to closest string
    (0, midi_note.saturating_sub(STRING_BASE_PITCH[0]) as usize)
}

/// Pitch detection using McLeod algorithm from the `pitch_detection` crate.
///
/// Returns `(frequency_hz, clarity)` or `None` if no pitch detected.
#[cfg(not(target_arch = "wasm32"))]
pub fn detect_pitch_mcleod(
    audio: &[f32],
    sample_rate: usize,
    min_clarity: f64,
) -> Option<(f32, f32)> {
    use pitch_detection::detector::mcleod::McLeodDetector;
    use pitch_detection::detector::PitchDetector;

    let n = audio.len();
    if n < 64 {
        return None;
    }

    // McLeod needs the buffer size and padding
    let padding = n / 2;
    let mut detector = McLeodDetector::new(n, padding);

    let pitch = detector.get_pitch(audio, sample_rate, 0.5, 0.3)?;

    if (pitch.clarity as f64) < min_clarity {
        return None;
    }

    Some((pitch.frequency, pitch.clarity))
}

/// Stub for WASM (pitch detection crate not available).
#[cfg(target_arch = "wasm32")]
pub fn detect_pitch_mcleod(
    _audio: &[f32],
    _sample_rate: usize,
    _min_clarity: f64,
) -> Option<(f32, f32)> {
    None
}

/// Get string display names.
pub fn string_display_names() -> &'static [&'static str; 6] {
    &STRING_NAMES
}

/// Get MIDI note name (e.g. 60 -> "C4").
pub fn note_name(midi: u8) -> String {
    midi_to_note_name(midi)
}

/// Get the open-string frequency for a given string index.
pub fn open_string_freq(string_idx: usize) -> f32 {
    STRING_BASE_PITCH
        .get(string_idx)
        .map(|&m| midi_to_freq(m))
        .unwrap_or(0.0)
}

// ── Tests ──────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;

    /// Generate a sine wave.
    fn sine_wave(freq: f32, sample_rate: usize, num_samples: usize) -> Vec<f32> {
        (0..num_samples)
            .map(|i| (2.0 * PI * freq * i as f32 / sample_rate as f32).sin())
            .collect()
    }

    /// Generate a guitar-like signal with inharmonic overtones.
    fn guitar_signal(
        fundamental: f32,
        b_coeff: f32,
        sample_rate: usize,
        num_samples: usize,
    ) -> Vec<f32> {
        let mut signal = vec![0.0f32; num_samples];
        for n in 1..=6 {
            let harmonic_freq =
                fundamental * n as f32 * (1.0 + b_coeff * (n * n) as f32).sqrt();
            let amplitude = 1.0 / n as f32; // natural harmonic decay
            for i in 0..num_samples {
                signal[i] += amplitude
                    * (2.0 * PI * harmonic_freq * i as f32 / sample_rate as f32)
                        .sin();
            }
        }
        signal
    }

    // -- Goertzel tests ──────────────────────────────────────────────

    #[test]
    fn goertzel_detects_known_frequency() {
        let sr = 48000;
        let freq = 440.0;
        let signal = sine_wave(freq, sr, 2048);
        let mag = goertzel(&signal, sr, freq);
        assert!(mag > 10.0, "Goertzel should find strong signal at 440Hz, got {}", mag);
    }

    #[test]
    fn goertzel_low_at_wrong_frequency() {
        let sr = 48000;
        let signal = sine_wave(440.0, sr, 2048);
        let mag_wrong = goertzel(&signal, sr, 300.0);
        let mag_right = goertzel(&signal, sr, 440.0);
        assert!(
            mag_right > mag_wrong * 5.0,
            "440Hz mag ({}) should be much stronger than 300Hz ({})",
            mag_right,
            mag_wrong
        );
    }

    #[test]
    fn goertzel_silent_returns_zero() {
        let silence = vec![0.0f32; 1024];
        let mag = goertzel(&silence, 48000, 440.0);
        assert!(mag.abs() < 1e-6, "silence should give near-zero, got {}", mag);
    }

    #[test]
    fn goertzel_empty_returns_zero() {
        let empty: Vec<f32> = vec![];
        let mag = goertzel(&empty, 48000, 440.0);
        assert_eq!(mag, 0.0);
    }

    // -- freq_to_midi tests ──────────────────────────────────────────

    #[test]
    fn freq_to_midi_a4() {
        let (note, cents) = freq_to_midi(440.0);
        assert_eq!(note, 69, "A4 = MIDI 69");
        assert_eq!(cents, 0, "exactly in tune");
    }

    #[test]
    fn freq_to_midi_e2() {
        let (note, cents) = freq_to_midi(82.41);
        assert_eq!(note, 40, "E2 = MIDI 40");
        assert!(cents.abs() <= 2, "should be near zero cents, got {}", cents);
    }

    #[test]
    fn freq_to_midi_zero() {
        let (note, cents) = freq_to_midi(0.0);
        assert_eq!(note, 0);
        assert_eq!(cents, 0);
    }

    // -- simple_string_fret tests ────────────────────────────────────

    #[test]
    fn simple_string_fret_open_e2() {
        let (s, f) = simple_string_fret(40); // E2
        assert_eq!(s, 0, "string 0 = Low E");
        assert_eq!(f, 0, "fret 0 = open");
    }

    #[test]
    fn simple_string_fret_a2_on_low_e() {
        // A2 = MIDI 45 can be played on Low E fret 5 or A string open.
        // The heuristic prefers the highest-index (thinnest) string that works,
        // iterating in reverse.
        let (s, f) = simple_string_fret(45);
        assert_eq!(s, 1, "A2 should prefer A string");
        assert_eq!(f, 0, "A2 open on A string");
    }

    #[test]
    fn simple_string_fret_e4() {
        // E4 = MIDI 64 = high E open
        let (s, f) = simple_string_fret(64);
        assert_eq!(s, 5, "high E string");
        assert_eq!(f, 0, "open");
    }

    // -- identify_string tests ───────────────────────────────────────

    #[test]
    fn identify_string_with_calibration() {
        let sr = 48000;
        let e2_freq = 82.41;

        // Create calibration profiles
        let profiles = make_test_profiles();

        // Generate an E2-like signal with low-E inharmonicity
        let audio = guitar_signal(e2_freq, 0.006, sr, 4096);

        let result = identify_string(
            &audio,
            e2_freq,
            40, // MIDI E2
            &profiles,
            sr,
            6,
            0.3,
        );

        assert!(result.is_some(), "should identify a string");
        let (string_idx, confidence) = result.unwrap();
        assert_eq!(string_idx, 0, "should identify as Low E");
        assert!(confidence > 0.3, "confidence should be reasonable, got {}", confidence);
    }

    #[test]
    fn identify_string_rejects_impossible() {
        let sr = 48000;
        // MIDI 30 is below any open string -- should still return something
        // or None since it is below all open strings.
        let audio = sine_wave(midi_to_freq(30), sr, 2048);
        let profiles = make_test_profiles();
        let result = identify_string(&audio, midi_to_freq(30), 30, &profiles, sr, 6, 0.3);
        assert!(result.is_none(), "MIDI 30 is not playable on any standard-tuned string");
    }

    // -- measure_harmonics tests ─────────────────────────────────────

    #[test]
    fn measure_harmonics_finds_overtones() {
        let sr = 48000;
        let fund = 110.0; // A2
        let audio = guitar_signal(fund, 0.003, sr, 4096);
        let harmonics = measure_harmonics(&audio, fund, 4, sr);
        assert_eq!(harmonics.len(), 4);
        // h1 should be strongest
        assert!(
            harmonics[0] > harmonics[1],
            "h1 ({}) should be stronger than h2 ({})",
            harmonics[0],
            harmonics[1]
        );
    }

    // -- compute_rms tests ───────────────────────────────────────────

    #[test]
    fn rms_of_silence_is_zero() {
        let silence = vec![0.0f32; 1024];
        assert!(compute_rms(&silence) < 1e-10);
    }

    #[test]
    fn rms_of_sine_is_correct() {
        // RMS of a sine wave with amplitude A is A / sqrt(2).
        let sr = 48000;
        // Use a large buffer for accuracy
        let signal = sine_wave(100.0, sr, sr); // 1 second
        let rms = compute_rms(&signal);
        let expected = 1.0 / (2.0f32).sqrt();
        assert!(
            (rms - expected).abs() < 0.01,
            "RMS should be ~{:.4}, got {:.4}",
            expected,
            rms
        );
    }

    // -- rms_to_velocity tests ───────────────────────────────────────

    #[test]
    fn velocity_range() {
        assert_eq!(rms_to_velocity(0.0), 30); // below min maps to 30
        assert_eq!(rms_to_velocity(1.0), 127); // above max clamps to 127
        let mid = rms_to_velocity(0.25);
        assert!(mid > 30 && mid < 127, "mid velocity {} should be in range", mid);
    }

    // -- compute_inharmonicity_b tests ───────────────────────────────

    #[test]
    fn inharmonicity_of_pure_sine_is_near_zero() {
        let sr = 48000;
        let audio = sine_wave(110.0, sr, 4096);
        let b = compute_inharmonicity_b(&audio, 110.0, sr);
        assert!(
            b < 0.005,
            "Pure sine should have near-zero B, got {}",
            b
        );
    }

    #[test]
    fn inharmonicity_of_guitar_signal_is_positive() {
        let sr = 48000;
        let b_input = 0.005;
        let audio = guitar_signal(110.0, b_input, sr, 8192);
        let b_measured = compute_inharmonicity_b(&audio, 110.0, sr);
        // Should be in the right ballpark (may not be exact due to Goertzel resolution)
        assert!(
            b_measured > 0.0 && b_measured < 0.05,
            "Measured B should be small positive, got {}",
            b_measured
        );
    }

    // -- pipeline integration test ───────────────────────────────────

    #[test]
    fn pipeline_detects_note_from_sine() {
        let sr = 48000;
        let config = GuitarInputConfig {
            buffer_size: 1024,
            sample_rate: sr,
            onset_threshold: 0.01,
            string_confidence_min: 0.3,
            bends_enabled: false,
            legato_enabled: false,
            filter_enabled: false,
            min_clarity: 0.3,
            cooldown_samples: 1024,
            n_harmonics: 6,
        };
        let mut pipeline = GuitarInput::new(config);

        // Feed silence then a loud sine
        let silence = vec![0.0f32; 1024];
        let events1 = pipeline.process_block(&silence);
        assert!(events1.is_empty(), "silence should produce no events");

        // Now feed a loud A4 sine (amplitude 0.5)
        let a4: Vec<f32> = sine_wave(440.0, sr, 2048)
            .into_iter()
            .map(|s| s * 0.5)
            .collect();
        let events2 = pipeline.process_block(&a4);
        // Should get a NoteOn somewhere in here
        let has_note_on = events2.iter().any(|e| matches!(e, MidiEvent::NoteOn { .. }));
        assert!(has_note_on, "should detect NoteOn for A4, events: {:?}", events2);
    }

    // -- Helper ──────────────────────────────────────────────────────

    fn make_test_profiles() -> [StringProfile; 6] {
        [
            StringProfile {
                name: "Low E",
                open_midi: 40,
                open_frequency: 82.41,
                inharmonicity_b: 0.006,
                harmonic_ratios: vec![0.5, 0.3, 0.2, 0.1, 0.05],
            },
            StringProfile {
                name: "A",
                open_midi: 45,
                open_frequency: 110.0,
                inharmonicity_b: 0.003,
                harmonic_ratios: vec![0.55, 0.35, 0.2, 0.12, 0.06],
            },
            StringProfile {
                name: "D",
                open_midi: 50,
                open_frequency: 146.83,
                inharmonicity_b: 0.003,
                harmonic_ratios: vec![0.5, 0.3, 0.18, 0.1, 0.05],
            },
            StringProfile {
                name: "G",
                open_midi: 55,
                open_frequency: 196.0,
                inharmonicity_b: 0.006,
                harmonic_ratios: vec![0.6, 0.4, 0.25, 0.15, 0.08],
            },
            StringProfile {
                name: "B",
                open_midi: 59,
                open_frequency: 246.94,
                inharmonicity_b: 0.003,
                harmonic_ratios: vec![0.5, 0.28, 0.15, 0.08, 0.04],
            },
            StringProfile {
                name: "High E",
                open_midi: 64,
                open_frequency: 329.63,
                inharmonicity_b: 0.002,
                harmonic_ratios: vec![0.45, 0.25, 0.12, 0.06, 0.03],
            },
        ]
    }
}
