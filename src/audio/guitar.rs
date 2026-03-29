//! Guitar-specific calibration and pitch-to-MIDI conversion.
//!
//! Ported from TheStringTheory's GuitarBridgeServer calibration system.
//! Provides per-string calibration profiles, onset detection thresholds,
//! and guitar-aware pitch matching that accounts for string harmonics
//! and pluck dynamics.

use serde::{Deserialize, Serialize};

// ── Standard Guitar Tuning ──────────────────────────────────────────

/// Standard tuning MIDI base pitches per string (low to high).
/// E2=40, A2=45, D3=50, G3=55, B3=59, E4=64
pub const STRING_BASE_PITCH: [u8; 6] = [40, 45, 50, 55, 59, 64];

/// String names for display.
pub const STRING_NAMES: [&str; 6] = ["Low E", "A", "D", "G", "B", "High E"];

/// Maximum fret count for standard guitar.
pub const MAX_FRETS: u8 = 24;

// ── Calibration Sample ──────────────────────────────────────────────

/// A single calibration sample captured during the calibration process.
/// Mirrors TheStringTheory's per-pluck sample data.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CalibrationSample {
    /// Detected note name (e.g., "E2") or empty if undetected
    pub note: String,
    /// Detected frequency in Hz
    pub freq: f32,
    /// Pitch detection confidence (0.0-1.0)
    pub conf: f32,
    /// Peak amplitude of the main (full-band) signal
    pub peak: f32,
    /// RMS amplitude of the main signal
    pub rms: f32,
    /// Peak amplitude of the high-frequency (brightness) band
    pub bright_peak: f32,
    /// RMS amplitude of the brightness band
    pub bright_rms: f32,
    /// Delta of main peak vs running average (onset indicator)
    pub main_delta: f32,
    /// Ratio of main peak vs running average
    pub main_ratio: f32,
    /// Slope of main signal envelope (positive = attack)
    pub main_slope: f32,
    /// Delta of brightness peak vs running average
    pub bright_delta: f32,
    /// Ratio of brightness peak vs running average
    pub bright_ratio: f32,
    /// Slope of brightness envelope
    pub bright_slope: f32,
}

impl Default for CalibrationSample {
    fn default() -> Self {
        Self {
            note: String::new(),
            freq: 0.0,
            conf: 0.0,
            peak: 0.0,
            rms: 0.0,
            bright_peak: 0.0,
            bright_rms: 0.0,
            main_delta: 0.0,
            main_ratio: 0.0,
            main_slope: 0.0,
            bright_delta: 0.0,
            bright_ratio: 0.0,
            bright_slope: 0.0,
        }
    }
}

// ── Per-String Calibration ──────────────────────────────────────────

/// Calibration data for a single guitar string.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StringCalibration {
    /// Display name (e.g., "Low E")
    pub string_name: String,
    /// Expected note name (e.g., "E2")
    pub expected_note: String,
    /// Expected open-string frequency in Hz
    pub expected_frequency: f32,
    /// Samples from soft plucks (used for quiet threshold)
    pub soft_samples: Vec<CalibrationSample>,
    /// Samples from strong plucks (used for loud threshold)
    pub strong_samples: Vec<CalibrationSample>,
}

impl StringCalibration {
    /// Compute the average confidence across all valid samples.
    pub fn avg_confidence(&self) -> f32 {
        let samples: Vec<&CalibrationSample> = self
            .soft_samples
            .iter()
            .chain(self.strong_samples.iter())
            .filter(|s| s.conf > 0.0)
            .collect();
        if samples.is_empty() {
            return 0.0;
        }
        samples.iter().map(|s| s.conf).sum::<f32>() / samples.len() as f32
    }

    /// Compute the soft pluck RMS threshold (average of soft sample RMS values).
    pub fn soft_rms_threshold(&self) -> f32 {
        let valid: Vec<f32> = self
            .soft_samples
            .iter()
            .filter(|s| s.rms > 0.0)
            .map(|s| s.rms)
            .collect();
        if valid.is_empty() {
            return 0.01; // sensible default
        }
        valid.iter().sum::<f32>() / valid.len() as f32 * 0.5 // 50% of average as gate
    }

    /// Compute onset detection threshold from strong pluck slope data.
    pub fn onset_slope_threshold(&self) -> f32 {
        let slopes: Vec<f32> = self
            .strong_samples
            .iter()
            .filter(|s| s.main_slope > 0.0)
            .map(|s| s.main_slope)
            .collect();
        if slopes.is_empty() {
            return 0.02; // sensible default
        }
        slopes.iter().sum::<f32>() / slopes.len() as f32 * 0.3 // 30% of average strong attack
    }

    /// Compute brightness ratio threshold for pluck detection.
    pub fn brightness_threshold(&self) -> f32 {
        let ratios: Vec<f32> = self
            .strong_samples
            .iter()
            .filter(|s| s.bright_ratio > 0.0)
            .map(|s| s.bright_ratio)
            .collect();
        if ratios.is_empty() {
            return 0.5;
        }
        ratios.iter().sum::<f32>() / ratios.len() as f32 * 0.5
    }
}

// ── Full Guitar Calibration Profile ─────────────────────────────────

/// Complete guitar calibration profile with per-string data.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GuitarCalibrationProfile {
    /// Profile format version
    pub version: u32,
    /// Per-string calibration data (6 strings, low to high)
    pub strings: Vec<StringCalibration>,
}

impl GuitarCalibrationProfile {
    /// Load a profile from JSON.
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    /// Serialize to JSON.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Get calibration for a specific string index (0=Low E, 5=High E).
    pub fn string(&self, idx: usize) -> Option<&StringCalibration> {
        self.strings.get(idx)
    }
}

impl Default for GuitarCalibrationProfile {
    /// Creates a default profile with standard tuning and no calibration samples.
    fn default() -> Self {
        Self {
            version: 1,
            strings: STRING_NAMES
                .iter()
                .enumerate()
                .map(|(i, name)| {
                    let (note_name, freq) = standard_tuning_note(i);
                    StringCalibration {
                        string_name: name.to_string(),
                        expected_note: note_name,
                        expected_frequency: freq,
                        soft_samples: Vec::new(),
                        strong_samples: Vec::new(),
                    }
                })
                .collect(),
        }
    }
}

/// Returns (note_name, frequency) for standard tuning string index.
fn standard_tuning_note(string_idx: usize) -> (String, f32) {
    let midi = STRING_BASE_PITCH.get(string_idx).copied().unwrap_or(40);
    let note_name = midi_to_note_name(midi);
    let freq = midi_to_freq(midi);
    (note_name, freq)
}

// ── Guitar Pitch Matcher ────────────────────────────────────────────

/// Configurable parameters for guitar pitch matching.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GuitarMatchConfig {
    /// Time window (seconds) to search before expected note time
    pub event_match_early: f32,
    /// Time window (seconds) to search after expected note time
    pub event_match_late: f32,
    /// Window to merge duplicate onset events (seconds)
    pub duplicate_merge_window: f32,
    /// Extra early tolerance for high strings (B, high E)
    pub high_string_extra_early: f32,
    /// Extra late tolerance for high strings
    pub high_string_extra_late: f32,
    /// Minimum confidence to accept a detection
    pub min_confidence: f32,
    /// Minimum RMS to consider as a real pluck (not noise)
    pub min_rms: f32,
}

impl Default for GuitarMatchConfig {
    fn default() -> Self {
        Self {
            event_match_early: 0.15,
            event_match_late: 0.15,
            duplicate_merge_window: 0.085,
            high_string_extra_early: 0.02,
            high_string_extra_late: 0.08,
            min_confidence: 0.45,
            min_rms: 0.005,
        }
    }
}

/// A detected pitch event from audio input.
#[derive(Clone, Debug)]
pub struct GuitarPitchEvent {
    /// Time of detection (seconds from start)
    pub time: f32,
    /// Detected MIDI pitches (may contain harmonics)
    pub pitches: Vec<u8>,
    /// Confidence of primary pitch
    pub confidence: f32,
    /// RMS level at detection time
    pub rms: f32,
    /// Brightness ratio at detection time
    pub brightness: f32,
}

/// Identifies which string a pitch most likely came from.
#[derive(Clone, Debug)]
pub struct StringMatch {
    /// String index (0-5)
    pub string_idx: usize,
    /// Detected fret number
    pub fret: u8,
    /// MIDI note number
    pub midi_note: u8,
    /// Confidence of the match
    pub confidence: f32,
}

/// Guitar-aware pitch matcher that uses calibration data.
pub struct GuitarPitchMatcher {
    config: GuitarMatchConfig,
    profile: GuitarCalibrationProfile,
    /// Recent pitch events for time-windowed matching
    recent_events: Vec<GuitarPitchEvent>,
    /// Currently sustained pitches
    active_pitches: Vec<u8>,
    /// Maximum event history (seconds)
    history_window: f32,
}

impl GuitarPitchMatcher {
    pub fn new(config: GuitarMatchConfig, profile: GuitarCalibrationProfile) -> Self {
        Self {
            config,
            profile,
            recent_events: Vec::new(),
            active_pitches: Vec::new(),
            history_window: 3.0,
        }
    }

    /// Create with default config and profile.
    pub fn with_defaults() -> Self {
        Self::new(GuitarMatchConfig::default(), GuitarCalibrationProfile::default())
    }

    /// Update the calibration profile.
    pub fn set_profile(&mut self, profile: GuitarCalibrationProfile) {
        self.profile = profile;
    }

    /// Update match config.
    pub fn set_config(&mut self, config: GuitarMatchConfig) {
        self.config = config;
    }

    /// Feed a new pitch detection result. Returns a StringMatch if
    /// the pitch can be identified as a guitar note.
    pub fn feed(
        &mut self,
        time: f32,
        frequency: f32,
        confidence: f32,
        rms: f32,
        brightness: f32,
    ) -> Option<StringMatch> {
        // Gate: reject low confidence or quiet signals
        if confidence < self.config.min_confidence || rms < self.config.min_rms {
            return None;
        }

        let (midi_note, _cents) = super::pitch::freq_to_midi(frequency);

        // Store event
        let event = GuitarPitchEvent {
            time,
            pitches: vec![midi_note],
            confidence,
            rms,
            brightness,
        };

        // Deduplicate: merge if within duplicate window of last event
        if let Some(last) = self.recent_events.last_mut() {
            if (time - last.time).abs() < self.config.duplicate_merge_window {
                if !last.pitches.contains(&midi_note) {
                    last.pitches.push(midi_note);
                }
                // Update confidence if this detection is better
                if confidence > last.confidence {
                    last.confidence = confidence;
                    last.rms = rms;
                    last.brightness = brightness;
                }
                return self.identify_string(midi_note, confidence);
            }
        }

        self.recent_events.push(event);
        self.prune_history(time);

        self.identify_string(midi_note, confidence)
    }

    /// Identify which string/fret a MIDI note most likely belongs to.
    /// Uses pitch class matching and calibration thresholds.
    pub fn identify_string(&self, midi_note: u8, confidence: f32) -> Option<StringMatch> {
        let mut best: Option<StringMatch> = None;
        let mut best_score = f32::MIN;

        for (string_idx, base_pitch) in STRING_BASE_PITCH.iter().enumerate() {
            // Check if this note could be on this string (open to max fret)
            if midi_note < *base_pitch || midi_note > base_pitch + MAX_FRETS {
                continue;
            }

            let fret = midi_note - base_pitch;
            let mut score = confidence;

            // Prefer lower fret positions (more natural)
            score -= fret as f32 * 0.01;

            // Prefer middle strings slightly (less ambiguous)
            if string_idx >= 1 && string_idx <= 4 {
                score += 0.02;
            }

            // Use calibration data if available
            if let Some(cal) = self.profile.string(string_idx) {
                let avg_conf = cal.avg_confidence();
                if avg_conf > 0.0 {
                    // Boost score if calibration shows this string is reliable
                    score += avg_conf * 0.1;
                }
            }

            if score > best_score {
                best_score = score;
                best = Some(StringMatch {
                    string_idx,
                    fret,
                    midi_note,
                    confidence,
                });
            }
        }

        best
    }

    /// Try to match a detected pitch against an expected note at a given time.
    /// Used for tab/chart matching (like TheStringTheory's gameplay).
    pub fn try_match_note(
        &self,
        expected_string: usize,
        expected_fret: u8,
        expected_time: f32,
    ) -> Option<f32> {
        if expected_string >= STRING_BASE_PITCH.len() {
            return None;
        }

        let target_pitch = STRING_BASE_PITCH[expected_string] + expected_fret;
        let target_pitch_class = target_pitch % 12;

        // Adjust window for high strings
        let extra_early = if expected_string >= 4 {
            self.config.high_string_extra_early
        } else {
            0.0
        };
        let extra_late = if expected_string >= 4 {
            self.config.high_string_extra_late
        } else {
            0.0
        };

        let window_start = expected_time - self.config.event_match_early - extra_early;
        let window_end = expected_time + self.config.event_match_late + extra_late;

        let mut best_distance = f32::MAX;
        let mut best_time = None;

        for event in self.recent_events.iter().rev() {
            if event.time < window_start {
                break;
            }
            if event.time > window_end {
                continue;
            }

            // Pitch class match (octave-agnostic)
            if event.pitches.iter().any(|p| p % 12 == target_pitch_class) {
                let distance = (event.time - expected_time).abs();
                if distance < best_distance {
                    best_distance = distance;
                    best_time = Some(event.time);
                }
            }
        }

        best_time
    }

    /// Get currently detected active pitches.
    pub fn active_pitches(&self) -> &[u8] {
        &self.active_pitches
    }

    /// Note on: add to active pitches.
    pub fn note_on(&mut self, midi_note: u8) {
        if !self.active_pitches.contains(&midi_note) {
            self.active_pitches.push(midi_note);
        }
    }

    /// Note off: remove from active pitches.
    pub fn note_off(&mut self, midi_note: u8) {
        self.active_pitches.retain(|&n| n != midi_note);
    }

    fn prune_history(&mut self, current_time: f32) {
        let cutoff = current_time - self.history_window;
        self.recent_events.retain(|e| e.time >= cutoff);
    }
}

// ── Utility Functions ───────────────────────────────────────────────

/// Convert MIDI note number to frequency (Hz).
/// Formula: freq = 440 * 2^((midi - 69) / 12)
pub fn midi_to_freq(midi: u8) -> f32 {
    440.0 * 2.0_f32.powf((midi as f32 - 69.0) / 12.0)
}

/// Convert MIDI note number to note name (e.g., 40 -> "E2").
pub fn midi_to_note_name(midi: u8) -> String {
    const NAMES: [&str; 12] = [
        "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B",
    ];
    let octave = (midi / 12) as i8 - 1;
    let note = (midi % 12) as usize;
    format!("{}{}", NAMES[note], octave)
}

/// Parse a note name to MIDI number (e.g., "E2" -> 40).
pub fn note_name_to_midi(name: &str) -> Option<u8> {
    let name = name.trim();
    if name.is_empty() {
        return None;
    }

    // Find where the octave number starts
    let (note_part, octave_part) = if name.len() >= 2 {
        let split = if name.as_bytes()[1] == b'#' || name.as_bytes()[1] == b'b' {
            2
        } else {
            1
        };
        (&name[..split], &name[split..])
    } else {
        return None;
    };

    let pitch_class = match note_part {
        "C" => 0,
        "C#" | "Db" => 1,
        "D" => 2,
        "D#" | "Eb" => 3,
        "E" => 4,
        "F" => 5,
        "F#" | "Gb" => 6,
        "G" => 7,
        "G#" | "Ab" => 8,
        "A" => 9,
        "A#" | "Bb" => 10,
        "B" => 11,
        _ => return None,
    };

    let octave: i8 = octave_part.parse().ok()?;
    let midi = (octave + 1) as i16 * 12 + pitch_class as i16;

    if midi >= 0 && midi <= 127 {
        Some(midi as u8)
    } else {
        None
    }
}

// ── Audio Normalizer (calibration-based noise rejection + normalization) ──

/// Result of audio normalization — tells the pipeline whether to proceed.
#[derive(Clone, Debug)]
pub enum NormalizeResult {
    /// Signal is valid. Contains normalized RMS and brightness ratio.
    Valid {
        /// RMS normalized to 0.0-1.0 range based on calibrated soft/strong levels
        normalized_rms: f32,
        /// Brightness ratio: bright_rms / total_rms
        brightness_ratio: f32,
        /// Whether this looks like a pluck (high brightness + sufficient confidence)
        is_pluck_like: bool,
    },
    /// Signal rejected — below noise gate or fails brightness check.
    Rejected,
}

/// Calibration-driven audio normalizer and noise gate.
///
/// Uses per-string calibration data to:
/// 1. Gate noise below the calibrated noise floor
/// 2. Reject non-pitched sounds (brushes/scrapes) via brightness filtering
/// 3. Normalize RMS across strings so detection thresholds are consistent
pub struct AudioNormalizer {
    /// Per-string calibrated thresholds
    string_noise_gates: [f32; 6],
    /// Per-string RMS normalization factors (maps soft..strong to 0..1)
    string_rms_soft: [f32; 6],
    string_rms_strong: [f32; 6],
    /// Per-string expected brightness ratios (from calibration)
    string_brightness_mean: [f32; 6],
    string_brightness_stddev: [f32; 6],
    /// Global noise floor
    noise_floor: f32,
    /// Minimum brightness confidence for a valid pluck
    /// High brightness + high pitch confidence = real pluck
    /// High brightness + low pitch confidence = brush/scrape noise
    min_pluck_confidence: f32,
    /// Brightness rejection: if brightness is above this multiple of
    /// the calibrated mean AND confidence is below min_pluck_confidence,
    /// reject as noise
    brightness_noise_multiplier: f32,
}

impl AudioNormalizer {
    /// Create from a calibration profile.
    pub fn from_profile(profile: &GuitarCalibrationProfile) -> Self {
        let mut noise_gates = [0.01f32; 6];
        let mut rms_soft = [0.01f32; 6];
        let mut rms_strong = [0.1f32; 6];
        let mut bright_mean = [0.3f32; 6];
        let mut bright_stddev = [0.15f32; 6];
        let mut global_noise = 0.005f32;

        for (i, cal) in profile.strings.iter().enumerate().take(6) {
            let soft_rms = cal.soft_rms_threshold();
            noise_gates[i] = soft_rms * 0.4; // gate at 40% of softest pluck

            // Soft RMS average
            let soft_vals: Vec<f32> = cal.soft_samples.iter()
                .filter(|s| s.rms > 0.0).map(|s| s.rms).collect();
            rms_soft[i] = if soft_vals.is_empty() { 0.01 }
                else { soft_vals.iter().sum::<f32>() / soft_vals.len() as f32 };

            // Strong RMS average
            let strong_vals: Vec<f32> = cal.strong_samples.iter()
                .filter(|s| s.rms > 0.0).map(|s| s.rms).collect();
            rms_strong[i] = if strong_vals.is_empty() { 0.1 }
                else { strong_vals.iter().sum::<f32>() / strong_vals.len() as f32 };

            // Brightness ratio: bright_rms / total_rms from all samples
            let bright_ratios: Vec<f32> = cal.soft_samples.iter()
                .chain(cal.strong_samples.iter())
                .filter(|s| s.rms > 0.0 && s.bright_rms > 0.0)
                .map(|s| s.bright_rms / s.rms)
                .collect();

            if !bright_ratios.is_empty() {
                let mean = bright_ratios.iter().sum::<f32>() / bright_ratios.len() as f32;
                bright_mean[i] = mean;
                let variance = bright_ratios.iter()
                    .map(|&r| (r - mean) * (r - mean))
                    .sum::<f32>() / bright_ratios.len() as f32;
                bright_stddev[i] = variance.sqrt().max(0.05);
            }

            global_noise = global_noise.min(noise_gates[i]);
        }

        Self {
            string_noise_gates: noise_gates,
            string_rms_soft: rms_soft,
            string_rms_strong: rms_strong,
            string_brightness_mean: bright_mean,
            string_brightness_stddev: bright_stddev,
            noise_floor: global_noise,
            min_pluck_confidence: 0.5,
            brightness_noise_multiplier: 2.5,
        }
    }

    /// Create with generic defaults (no calibration).
    pub fn default_uncalibrated() -> Self {
        Self {
            string_noise_gates: [0.01; 6],
            string_rms_soft: [0.02; 6],
            string_rms_strong: [0.15; 6],
            string_brightness_mean: [0.3; 6],
            string_brightness_stddev: [0.15; 6],
            noise_floor: 0.008,
            min_pluck_confidence: 0.5,
            brightness_noise_multiplier: 2.5,
        }
    }

    /// Process a frame of audio and decide whether it's a valid signal.
    ///
    /// * `rms` — total frame RMS
    /// * `bright_rms` — RMS of upper-half frequencies (brightness proxy)
    /// * `confidence` — pitch detection confidence (0.0-1.0)
    /// * `string_hint` — if we have a guess which string this is (0-5), use for per-string thresholds
    pub fn normalize(
        &self,
        rms: f32,
        bright_rms: f32,
        confidence: f32,
        string_hint: Option<usize>,
    ) -> NormalizeResult {
        // 1. Noise gate
        let gate = string_hint
            .map(|si| self.string_noise_gates[si.min(5)])
            .unwrap_or(self.noise_floor);

        if rms < gate {
            return NormalizeResult::Rejected;
        }

        // 2. Brightness ratio
        let brightness_ratio = if rms > 0.0 { bright_rms / rms } else { 0.0 };

        // 3. Brightness noise check
        // Brushes/scrapes: very high brightness, low pitch confidence
        let expected_brightness = string_hint
            .map(|si| self.string_brightness_mean[si.min(5)])
            .unwrap_or(0.3);
        let brightness_threshold = expected_brightness * self.brightness_noise_multiplier;

        let is_noise = brightness_ratio > brightness_threshold
            && confidence < self.min_pluck_confidence;

        if is_noise {
            return NormalizeResult::Rejected;
        }

        // 4. Pluck-likeness: pluck has elevated brightness AND good confidence
        let is_pluck_like = brightness_ratio > expected_brightness * 0.8
            && confidence >= self.min_pluck_confidence;

        // 5. Normalize RMS to 0.0-1.0 based on calibrated range
        let (soft, strong) = string_hint
            .map(|si| (self.string_rms_soft[si.min(5)], self.string_rms_strong[si.min(5)]))
            .unwrap_or((0.02, 0.15));

        let range = (strong - soft).max(0.01);
        let normalized_rms = ((rms - soft) / range).clamp(0.0, 1.5);

        NormalizeResult::Valid {
            normalized_rms,
            brightness_ratio,
            is_pluck_like,
        }
    }

    /// Get the noise gate threshold for a specific string.
    pub fn noise_gate(&self, string_idx: usize) -> f32 {
        self.string_noise_gates[string_idx.min(5)]
    }

    /// Get the global noise floor.
    pub fn global_noise_floor(&self) -> f32 {
        self.noise_floor
    }
}

// ── Onset Grouper (chord detection from rapid note sequence) ────────

/// Events emitted by the onset grouper.
#[derive(Clone, Debug, PartialEq)]
pub enum ChordEvent {
    /// A chord (or single note) is ready — all notes arrived within the group window.
    ChordOn(Vec<u8>),
    /// Previous chord released (silence detected).
    ChordOff(Vec<u8>),
}

/// Groups rapid-fire monophonic pitch detections into chords.
///
/// When you strum a guitar, each string is plucked in quick succession
/// (~5-15ms apart). The onset grouper collects notes arriving within
/// a configurable time window and emits them as a single chord event.
pub struct OnsetGrouper {
    /// Time window to group notes into a chord (seconds).
    pub group_window: f32,
    /// Silence duration before releasing chord (seconds).
    pub release_time: f32,
    /// Notes accumulated in the current group.
    pending_notes: Vec<u8>,
    /// Time of the first note in the current group.
    group_start: f32,
    /// Time of the last detection (for silence tracking).
    last_detection: f32,
    /// Currently active chord (for release tracking).
    active_chord: Vec<u8>,
    /// Whether we've emitted the pending group as a chord.
    group_emitted: bool,
}

impl OnsetGrouper {
    /// Create a new onset grouper.
    ///
    /// * `group_window` — max time (seconds) between first and last note in a chord.
    ///   Guitar strums typically span 20-60ms. Default: 0.06 (60ms).
    /// * `release_time` — silence duration before releasing the chord.
    ///   Default: 0.15 (150ms).
    pub fn new(group_window: f32, release_time: f32) -> Self {
        Self {
            group_window,
            release_time,
            pending_notes: Vec::new(),
            group_start: 0.0,
            last_detection: 0.0,
            active_chord: Vec::new(),
            group_emitted: false,
        }
    }

    /// Create with sensible defaults for guitar.
    pub fn default_guitar() -> Self {
        Self::new(0.06, 0.15)
    }

    /// Feed a detected note. Call with `None` on silence frames.
    /// Returns any chord events to process.
    pub fn feed(&mut self, time: f32, note: Option<u8>) -> Vec<ChordEvent> {
        let mut events = Vec::new();

        match note {
            Some(n) => {
                self.last_detection = time;

                if self.pending_notes.is_empty() {
                    // Start a new group
                    self.group_start = time;
                    self.pending_notes.push(n);
                    self.group_emitted = false;
                } else if time - self.group_start <= self.group_window {
                    // Within group window — add if not duplicate pitch class
                    if !self.pending_notes.contains(&n) {
                        self.pending_notes.push(n);
                    }
                } else {
                    // Outside group window — emit current group, start new one
                    if !self.group_emitted {
                        // Release previous chord first
                        if !self.active_chord.is_empty() {
                            events.push(ChordEvent::ChordOff(self.active_chord.clone()));
                        }
                        let mut chord = self.pending_notes.clone();
                        chord.sort();
                        self.active_chord = chord.clone();
                        events.push(ChordEvent::ChordOn(chord));
                        self.group_emitted = true;
                    }

                    // Check if this is a new note or same chord re-detected
                    if !self.active_chord.contains(&n) {
                        // Genuinely new note — release old, start new group
                        if !self.active_chord.is_empty() {
                            events.push(ChordEvent::ChordOff(self.active_chord.clone()));
                            self.active_chord.clear();
                        }
                        self.pending_notes.clear();
                        self.pending_notes.push(n);
                        self.group_start = time;
                        self.group_emitted = false;
                    }
                    // If it's in active_chord, just a sustain re-detection — ignore
                }
            }
            None => {
                // Silence frame
                if !self.pending_notes.is_empty() && !self.group_emitted {
                    // Group window expired by silence — emit what we have
                    if time - self.group_start > self.group_window {
                        if !self.active_chord.is_empty() {
                            events.push(ChordEvent::ChordOff(self.active_chord.clone()));
                        }
                        let mut chord = self.pending_notes.clone();
                        chord.sort();
                        self.active_chord = chord.clone();
                        events.push(ChordEvent::ChordOn(chord));
                        self.group_emitted = true;
                        self.pending_notes.clear();
                    }
                }

                // Check for release
                if !self.active_chord.is_empty()
                    && time - self.last_detection > self.release_time
                {
                    events.push(ChordEvent::ChordOff(self.active_chord.clone()));
                    self.active_chord.clear();
                    self.pending_notes.clear();
                    self.group_emitted = false;
                }
            }
        }

        events
    }

    /// Get the currently active chord.
    pub fn active_chord(&self) -> &[u8] {
        &self.active_chord
    }

    /// Force release everything.
    pub fn release_all(&mut self) -> Vec<ChordEvent> {
        let mut events = Vec::new();
        if !self.active_chord.is_empty() {
            events.push(ChordEvent::ChordOff(self.active_chord.clone()));
            self.active_chord.clear();
        }
        self.pending_notes.clear();
        self.group_emitted = false;
        events
    }
}

// ── Tests ───────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_midi_to_freq() {
        // A4 = 440 Hz
        assert!((midi_to_freq(69) - 440.0).abs() < 0.01);
        // Middle C (C4) = 261.63 Hz
        assert!((midi_to_freq(60) - 261.63).abs() < 0.1);
        // E2 (Low E string) = 82.41 Hz
        assert!((midi_to_freq(40) - 82.41).abs() < 0.1);
    }

    #[test]
    fn test_midi_to_note_name() {
        assert_eq!(midi_to_note_name(40), "E2");
        assert_eq!(midi_to_note_name(45), "A2");
        assert_eq!(midi_to_note_name(50), "D3");
        assert_eq!(midi_to_note_name(55), "G3");
        assert_eq!(midi_to_note_name(59), "B3");
        assert_eq!(midi_to_note_name(64), "E4");
        assert_eq!(midi_to_note_name(60), "C4");
        assert_eq!(midi_to_note_name(69), "A4");
    }

    #[test]
    fn test_note_name_to_midi() {
        assert_eq!(note_name_to_midi("E2"), Some(40));
        assert_eq!(note_name_to_midi("A2"), Some(45));
        assert_eq!(note_name_to_midi("D3"), Some(50));
        assert_eq!(note_name_to_midi("G3"), Some(55));
        assert_eq!(note_name_to_midi("B3"), Some(59));
        assert_eq!(note_name_to_midi("E4"), Some(64));
        assert_eq!(note_name_to_midi("C#4"), Some(61));
        assert_eq!(note_name_to_midi(""), None);
    }

    #[test]
    fn test_roundtrip_note_name() {
        for midi in 21..=108 {
            let name = midi_to_note_name(midi);
            let back = note_name_to_midi(&name);
            assert_eq!(back, Some(midi), "Roundtrip failed for MIDI {midi} -> {name}");
        }
    }

    #[test]
    fn test_string_base_pitches() {
        assert_eq!(midi_to_note_name(STRING_BASE_PITCH[0]), "E2");
        assert_eq!(midi_to_note_name(STRING_BASE_PITCH[1]), "A2");
        assert_eq!(midi_to_note_name(STRING_BASE_PITCH[2]), "D3");
        assert_eq!(midi_to_note_name(STRING_BASE_PITCH[3]), "G3");
        assert_eq!(midi_to_note_name(STRING_BASE_PITCH[4]), "B3");
        assert_eq!(midi_to_note_name(STRING_BASE_PITCH[5]), "E4");
    }

    #[test]
    fn test_fret_to_midi() {
        // 5th fret on low E = A2 = 45
        assert_eq!(STRING_BASE_PITCH[0] + 5, 45);
        // 7th fret on A = E3 = 52
        assert_eq!(STRING_BASE_PITCH[1] + 7, 52);
        // Open D string = D3 = 50
        assert_eq!(STRING_BASE_PITCH[2] + 0, 50);
    }

    #[test]
    fn test_default_profile() {
        let profile = GuitarCalibrationProfile::default();
        assert_eq!(profile.strings.len(), 6);
        assert_eq!(profile.strings[0].string_name, "Low E");
        assert_eq!(profile.strings[0].expected_note, "E2");
        assert!((profile.strings[0].expected_frequency - 82.41).abs() < 0.1);
    }

    #[test]
    fn test_identify_string_open_notes() {
        let matcher = GuitarPitchMatcher::with_defaults();

        // Open Low E (40) should identify as string 0, fret 0
        let m = matcher.identify_string(40, 0.9).unwrap();
        assert_eq!(m.string_idx, 0);
        assert_eq!(m.fret, 0);

        // Open A (45) — could be string 0 fret 5 or string 1 fret 0
        // Should prefer string 1 fret 0 (lower fret)
        let m = matcher.identify_string(45, 0.9).unwrap();
        assert_eq!(m.string_idx, 1);
        assert_eq!(m.fret, 0);
    }

    #[test]
    fn test_identify_string_fretted_notes() {
        let matcher = GuitarPitchMatcher::with_defaults();

        // MIDI 64 (E4) = string 5 fret 0, or string 4 fret 5, etc.
        // Should prefer string 5 fret 0 (open string)
        let m = matcher.identify_string(64, 0.9).unwrap();
        assert_eq!(m.string_idx, 5);
        assert_eq!(m.fret, 0);
    }

    #[test]
    fn test_feed_gates_low_confidence() {
        let mut matcher = GuitarPitchMatcher::with_defaults();
        // Below min_confidence threshold
        assert!(matcher.feed(0.0, 82.41, 0.1, 0.05, 0.5).is_none());
    }

    #[test]
    fn test_feed_gates_low_rms() {
        let mut matcher = GuitarPitchMatcher::with_defaults();
        // Below min_rms threshold
        assert!(matcher.feed(0.0, 82.41, 0.9, 0.001, 0.5).is_none());
    }

    #[test]
    fn test_feed_accepts_valid_detection() {
        let mut matcher = GuitarPitchMatcher::with_defaults();
        let result = matcher.feed(0.0, 82.41, 0.9, 0.05, 0.5);
        assert!(result.is_some());
        let m = result.unwrap();
        assert_eq!(m.midi_note, 40); // E2
        assert_eq!(m.string_idx, 0);
        assert_eq!(m.fret, 0);
    }

    #[test]
    fn test_feed_deduplicates() {
        let mut matcher = GuitarPitchMatcher::with_defaults();
        // Two detections within duplicate window
        let r1 = matcher.feed(0.0, 82.41, 0.9, 0.05, 0.5);
        let r2 = matcher.feed(0.05, 82.41, 0.95, 0.06, 0.5); // within 85ms
        assert!(r1.is_some());
        assert!(r2.is_some());
        // Should have merged into one event
        assert_eq!(matcher.recent_events.len(), 1);
    }

    #[test]
    fn test_try_match_note() {
        let mut matcher = GuitarPitchMatcher::with_defaults();
        // Feed a pitch event at time 1.0
        matcher.feed(1.0, 82.41, 0.9, 0.05, 0.5);

        // Try matching expected note: string 0 (Low E), fret 0, at time 1.05
        let result = matcher.try_match_note(0, 0, 1.05);
        assert!(result.is_some());
        assert!((result.unwrap() - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_try_match_note_miss() {
        let mut matcher = GuitarPitchMatcher::with_defaults();
        // Feed a pitch event at time 1.0
        matcher.feed(1.0, 82.41, 0.9, 0.05, 0.5);

        // Try matching a completely different note
        let result = matcher.try_match_note(3, 5, 1.0); // G3 + 5 frets = C4
        assert!(result.is_none());
    }

    #[test]
    fn test_calibration_sample_thresholds() {
        let cal = StringCalibration {
            string_name: "Low E".to_string(),
            expected_note: "E2".to_string(),
            expected_frequency: 82.41,
            soft_samples: vec![
                CalibrationSample {
                    rms: 0.02,
                    conf: 0.7,
                    main_slope: 0.01,
                    ..Default::default()
                },
                CalibrationSample {
                    rms: 0.04,
                    conf: 0.8,
                    main_slope: 0.02,
                    ..Default::default()
                },
            ],
            strong_samples: vec![
                CalibrationSample {
                    rms: 0.10,
                    conf: 0.9,
                    main_slope: 0.08,
                    bright_ratio: 1.2,
                    ..Default::default()
                },
                CalibrationSample {
                    rms: 0.13,
                    conf: 0.85,
                    main_slope: 0.10,
                    bright_ratio: 1.0,
                    ..Default::default()
                },
            ],
        };

        // Soft RMS threshold = 50% of avg(0.02, 0.04) = 50% of 0.03 = 0.015
        assert!((cal.soft_rms_threshold() - 0.015).abs() < 0.001);

        // Avg confidence across all valid = avg(0.7, 0.8, 0.9, 0.85) = 0.8125
        assert!((cal.avg_confidence() - 0.8125).abs() < 0.001);

        // Onset slope threshold = 30% of avg(0.08, 0.10) = 30% of 0.09 = 0.027
        assert!((cal.onset_slope_threshold() - 0.027).abs() < 0.001);
    }

    #[test]
    fn test_profile_json_roundtrip() {
        let profile = GuitarCalibrationProfile::default();
        let json = profile.to_json().unwrap();
        let loaded = GuitarCalibrationProfile::from_json(&json).unwrap();
        assert_eq!(loaded.strings.len(), 6);
        assert_eq!(loaded.strings[0].expected_note, "E2");
    }

    #[test]
    fn test_note_on_off_tracking() {
        let mut matcher = GuitarPitchMatcher::with_defaults();
        matcher.note_on(40);
        matcher.note_on(45);
        assert_eq!(matcher.active_pitches(), &[40, 45]);

        matcher.note_off(40);
        assert_eq!(matcher.active_pitches(), &[45]);

        matcher.note_off(45);
        assert!(matcher.active_pitches().is_empty());
    }

    // ── OnsetGrouper Tests ──────────────────────────────────

    #[test]
    fn test_single_note_becomes_chord_of_one() {
        let mut g = OnsetGrouper::default_guitar();
        // Feed a note at t=0.0
        let ev = g.feed(0.0, Some(40));
        assert!(ev.is_empty()); // still within group window

        // Silence after group window expires
        let ev = g.feed(0.1, None);
        assert_eq!(ev.len(), 1);
        assert_eq!(ev[0], ChordEvent::ChordOn(vec![40]));
    }

    #[test]
    fn test_strum_groups_into_chord() {
        let mut g = OnsetGrouper::new(0.06, 0.15);
        // Simulate a quick strum: 6 notes within 50ms
        assert!(g.feed(0.000, Some(40)).is_empty()); // E2
        assert!(g.feed(0.010, Some(45)).is_empty()); // A2
        assert!(g.feed(0.020, Some(50)).is_empty()); // D3
        assert!(g.feed(0.030, Some(55)).is_empty()); // G3
        assert!(g.feed(0.040, Some(59)).is_empty()); // B3
        assert!(g.feed(0.050, Some(64)).is_empty()); // E4

        // Silence after group window
        let ev = g.feed(0.12, None);
        assert_eq!(ev.len(), 1);
        assert_eq!(ev[0], ChordEvent::ChordOn(vec![40, 45, 50, 55, 59, 64]));
    }

    #[test]
    fn test_chord_releases_on_silence() {
        let mut g = OnsetGrouper::new(0.06, 0.15);
        g.feed(0.0, Some(40));
        g.feed(0.01, Some(45));

        // Emit chord
        let ev = g.feed(0.1, None);
        assert!(ev.iter().any(|e| matches!(e, ChordEvent::ChordOn(_))));

        // Silence long enough for release
        let ev = g.feed(0.3, None);
        assert_eq!(ev.len(), 1);
        assert_eq!(ev[0], ChordEvent::ChordOff(vec![40, 45]));
    }

    #[test]
    fn test_new_note_after_chord_triggers_new_group() {
        let mut g = OnsetGrouper::new(0.06, 0.15);
        // First chord
        g.feed(0.0, Some(40));
        g.feed(0.02, Some(45));
        let ev = g.feed(0.1, None); // emit first chord
        assert!(ev.iter().any(|e| matches!(e, ChordEvent::ChordOn(_))));

        // Release first chord via silence
        let ev = g.feed(0.3, None);
        assert!(ev.iter().any(|e| matches!(e, ChordEvent::ChordOff(_))));

        // New note starts a new group
        let ev = g.feed(0.5, Some(60));
        assert!(ev.is_empty()); // collecting in group window

        // Let new group emit
        let ev = g.feed(0.6, None);
        assert!(ev.iter().any(|e| *e == ChordEvent::ChordOn(vec![60])));
    }

    #[test]
    fn test_duplicate_notes_ignored_in_group() {
        let mut g = OnsetGrouper::new(0.06, 0.15);
        g.feed(0.0, Some(40));
        g.feed(0.02, Some(40)); // duplicate
        g.feed(0.04, Some(45));

        let ev = g.feed(0.1, None);
        assert_eq!(ev.len(), 1);
        assert_eq!(ev[0], ChordEvent::ChordOn(vec![40, 45])); // no duplicate 40
    }

    #[test]
    fn test_release_all() {
        let mut g = OnsetGrouper::new(0.06, 0.15);
        g.feed(0.0, Some(40));
        g.feed(0.1, None); // emit chord

        let ev = g.release_all();
        assert_eq!(ev.len(), 1);
        assert_eq!(ev[0], ChordEvent::ChordOff(vec![40]));
        assert!(g.active_chord().is_empty());
    }
}
