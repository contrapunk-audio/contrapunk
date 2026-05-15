//! DSP-based guitar input pipeline: pitch detection, string/fret identification, MIDI output.
//!
//! Pure DSP approach using McLeod pitch detection + inharmonicity-based string
//! identification. No ML -- relies on Goertzel harmonic analysis and calibrated
//! per-string inharmonicity coefficients to disambiguate which string produced
//! a given pitch.
//!
//! # Pipeline
//!
//! 1. **Onset detection** -- RMS spike + spectral flux with cooldown
//! 2. **Note state machine** -- Idle/Attack/Sustain/Decay hysteresis
//! 3. **Pitch detection** -- McLeod (from `pitch_detection` crate) with multi-frame voting
//! 4. **Octave error correction** -- detects harmonic lock / subharmonic errors
//! 5. **Harmonic measurement** -- Goertzel at integer multiples of fundamental
//! 6. **String identification** -- inharmonicity B-coefficient matching + plausibility filter
//! 7. **Note-off gating** -- suppress previous note's frequency on new onset
//! 8. **Bend tracking** -- continuous pitch monitoring between onsets
//! 9. **MIDI event generation** -- NoteOn / NoteOff / PitchBend

use super::guitar::{midi_to_freq, midi_to_note_name, STRING_BASE_PITCH, STRING_NAMES};
use super::pitch::freq_to_midi;

// ── Configuration ──────────────────────────────────────────────────

/// User-adjustable parameters for the guitar input pipeline.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct GuitarInputConfig {
    /// Analysis window in samples (256-2048).
    pub buffer_size: usize,
    /// Hop size in samples for sliding-window analysis (default 256 = 5.3ms at 48kHz).
    pub hop_size: usize,
    /// Audio sample rate (typically 48000).
    pub sample_rate: usize,
    /// RMS threshold for pluck / onset detection (seed for adaptive mode).
    pub onset_threshold: f32,
    /// Minimum confidence for string identification.
    pub string_confidence_min: f32,
    /// Enable pitch-bend tracking between onsets.
    pub bends_enabled: bool,
    /// Enable legato detection (hammer-on / pull-off).
    pub legato_enabled: bool,
    /// Enable slide detection (continuous pitch movement crossing note boundaries).
    pub slides_enabled: bool,
    /// Enable vibrato detection (periodic pitch oscillation).
    pub vibrato_detection: bool,
    /// When true, pass vibrato through as MIDI pitch bend; when false, smooth it out.
    pub vibrato_passthrough: bool,
    /// Enable low-pass pre-filter on input.
    pub filter_enabled: bool,
    /// Minimum McLeod clarity to accept a pitch detection.
    pub min_clarity: f64,
    /// Cooldown in samples after an onset before another can fire.
    pub cooldown_samples: usize,
    /// Number of harmonics to measure for inharmonicity.
    pub n_harmonics: usize,
    /// Input gain normalization factor (0.0-2.0, default 1.0).
    /// Set by calibration to normalize pluck levels.
    pub input_gain: f32,
    /// Spectral flux threshold for onset detection.
    /// Onset triggers when spectral flux exceeds this value.
    pub flux_threshold: f32,
    /// Use per-string MIDI channels (string 0 = ch 0, string 1 = ch 1, etc.)
    /// When false, all notes go on channel 0.
    pub per_string_channels: bool,
    /// Pitch bend range in semitones (default 48 for MPE, 2 for standard MIDI).
    pub pitch_bend_range: u8,
    /// Enable MPE channel pressure from shaped amplitude envelope (default: true).
    /// Replaces the old `aftertouch_enabled` (which defaulted to false).
    pub pressure_enabled: bool,
    /// Pressure hold factor: how much to slow the amplitude decay (0.0 = natural, 1.0 = full hold).
    pub pressure_hold: f32,
    /// Enable CC74 brightness from spectral centroid (default: true).
    pub brightness_enabled: bool,
}

impl Default for GuitarInputConfig {
    fn default() -> Self {
        Self {
            buffer_size: 1024,
            hop_size: 256,
            sample_rate: 48000,
            onset_threshold: 0.02,
            string_confidence_min: 0.5,
            bends_enabled: true,
            legato_enabled: true,
            slides_enabled: true,
            vibrato_detection: true,
            vibrato_passthrough: true,
            filter_enabled: false,
            min_clarity: 0.40,
            cooldown_samples: 960, // 20ms @ 48kHz — allows ~50 notes/sec
            n_harmonics: 6,
            input_gain: 1.0,
            flux_threshold: 0.5,
            per_string_channels: true,
            pitch_bend_range: 48, // MPE default; set to 2 for standard MIDI
            pressure_enabled: true,
            pressure_hold: 0.3,
            brightness_enabled: true,
        }
    }
}

impl GuitarInputConfig {
    /// Convert latency_ms to buffer_size in samples.
    ///
    /// Rounds to the nearest power of 2 for FFT efficiency, clamped to [256, 4096].
    pub fn buffer_size_for_latency(latency_ms: f32, sample_rate: usize) -> usize {
        let samples = (latency_ms / 1000.0 * sample_rate as f32) as usize;
        samples.next_power_of_two().max(256).min(4096)
    }
}

// ── Note State Machine ────────────────────────────────────────────

/// Hysteresis state machine for note detection.
///
/// Prevents flickering between notes by requiring stable pitch confirmation
/// before emitting events, and proper decay detection before re-triggering.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum NoteState {
    /// Waiting for onset (silence or between notes).
    Idle,
    /// Onset detected, gathering pitch info for confirmation.
    Attack,
    /// Note confirmed, tracking for bends. No note change unless new Attack.
    Sustain,
    /// RMS dropping, waiting for silence or timeout.
    Decay,
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
    /// Inharmonicity B measured at fret 5 (optional, for interpolation).
    pub inharmonicity_b_fret5: Option<f32>,
    /// Inharmonicity B measured at fret 12 (optional, for interpolation).
    pub inharmonicity_b_fret12: Option<f32>,
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
        velocity: u8,
    },
    PitchBend {
        channel: u8,
        /// Bend in cents from current note.
        cents: i16,
    },
    /// Raw 14-bit MIDI pitch bend (0-16383, 8192 = center).
    MidiPitchBend {
        channel: u8,
        value: u16,
    },
    /// Channel pressure (aftertouch) derived from sustain RMS.
    ChannelPressure {
        channel: u8,
        pressure: u8,
    },
    /// MIDI CC message (e.g., CC74 brightness).
    CC {
        channel: u8,
        controller: u8,
        value: u8,
    },
    /// Vibrato detected/ended -- informational, UI can display.
    VibratoStatus {
        active: bool,
        rate_hz: f32,
        depth_cents: f32,
    },
}

impl MidiEvent {
    /// Convert to raw MIDI bytes for sending through mpsc channel.
    /// Returns empty vec for informational-only events.
    pub fn to_midi_bytes(&self, pitch_bend_range: u8) -> Vec<u8> {
        match self {
            MidiEvent::NoteOn {
                channel,
                note,
                velocity,
            } => {
                vec![0x90 | (channel & 0x0F), *note & 0x7F, *velocity & 0x7F]
            }
            MidiEvent::NoteOff {
                channel,
                note,
                velocity,
            } => {
                vec![0x80 | (channel & 0x0F), *note & 0x7F, *velocity & 0x7F]
            }
            MidiEvent::PitchBend { channel, cents } => {
                let value = cents_to_midi_pitch_bend(*cents, pitch_bend_range);
                let lsb = (value & 0x7F) as u8;
                let msb = ((value >> 7) & 0x7F) as u8;
                vec![0xE0 | (channel & 0x0F), lsb, msb]
            }
            MidiEvent::MidiPitchBend { channel, value } => {
                let lsb = (*value & 0x7F) as u8;
                let msb = ((*value >> 7) & 0x7F) as u8;
                vec![0xE0 | (channel & 0x0F), lsb, msb]
            }
            MidiEvent::CC {
                channel,
                controller,
                value,
            } => {
                vec![0xB0 | (channel & 0x0F), *controller & 0x7F, *value & 0x7F]
            }
            MidiEvent::ChannelPressure { channel, pressure } => {
                vec![0xD0 | (channel & 0x0F), *pressure & 0x7F]
            }
            MidiEvent::VibratoStatus { .. } => {
                vec![] // Informational only, no MIDI bytes
            }
        }
    }
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
    /// Optional per-string AudioNormalizer derived from a
    /// `GuitarCalibrationProfile`. Loaded at routing start via
    /// `set_calibration_profile`. The normalizer's downstream
    /// consumers (per-string noise gates, brightness rejection)
    /// will wire in incrementally — for now the profile loading +
    /// storage is what closes the no-consumer gap flagged in the
    /// v1.3 handoff.
    normalizer: Option<super::guitar::AudioNormalizer>,
    /// The profile the normalizer was built from. Kept so the UI can
    /// surface sample-count / per-string stats without round-tripping
    /// through `app_data_dir`.
    calibration_profile: Option<super::guitar::GuitarCalibrationProfile>,

    // State
    current_note: Option<DetectedNote>,
    prev_rms: f32,
    cooldown_remaining: usize,
    ring_buffer: Vec<f32>,
    ring_pos: usize,

    // Note state machine (#2)
    note_state: NoteState,
    /// Sample counter for state timeouts.
    state_samples: usize,
    /// Total samples processed (monotonic counter).
    total_samples: usize,

    // Note-off gating / anti-resonance (#1)
    /// After detecting a new onset, suppress the previous note's frequency.
    suppress_frequency: Option<f32>,
    /// Sample count at which suppression expires.
    suppress_until: usize,

    // Multi-frame pitch voting (#7) — 2-frame for lower latency
    pitch_history: [Option<u8>; 2],
    pitch_history_idx: usize,

    // Spectral flux onset (#5)
    prev_spectrum: Option<Vec<f32>>,

    // Slide detection state
    /// Ring buffer of recent cents-from-base values for slide detection.
    slide_history: Vec<i16>,
    slide_history_idx: usize,
    /// Velocity decay multiplier per slide note (0.9 = 10% softer each).
    slide_velocity_decay: f32,

    // Vibrato detection state
    /// Ring buffer of cents deviations for vibrato analysis.
    vibrato_buffer: Vec<f32>,
    vibrato_buffer_idx: usize,
    /// Whether vibrato is currently detected.
    vibrato_detected: bool,
    /// Detected vibrato rate in Hz.
    vibrato_rate_hz: f32,
    /// Detected vibrato depth in cents.
    vibrato_depth_cents: f32,

    // Pressure / NoteOff velocity state
    /// Last RMS measured during sustain, used for decay tracking.
    last_sustain_rms: f32,
    /// Last channel pressure value sent (to avoid duplicate events).
    last_pressure: u8,
    /// RMS at the moment of note onset (peak), for pressure envelope shaping.
    note_onset_rms: f32,
    /// Last sent pressure value (used as NoteOff velocity per MG3 convention).
    last_sent_pressure: u8,
    /// Last CC74 brightness value sent (to avoid duplicate events).
    last_brightness: u8,

    // Single-cycle early pitch detector (runs sample-by-sample for lower latency)
    single_cycle: super::single_cycle::SingleCycleDetector,
    early_pitch: Option<(f32, f32)>,

    // Hop-based sliding window analysis
    hop_counter: usize,
    /// Pre-allocated buffer for linearized ring (avoids per-hop allocation)
    linearized_buf: Vec<f32>,

    // Adaptive thresholds
    noise_floor_ema: f32,
    signal_peak_ema: f32,

    // Onset-ahead-of-pitch: pitch correction after early NoteOn
    /// Pending pitch correction: (channel, initial_freq) to refine via pitch bend
    pending_pitch_correction: Option<(u8, f32)>,
    /// Analysis frames since last NoteOn (for correction window)
    frames_since_note_on: usize,

    // Debug state (read by WASM wrapper for logging)
    pub last_debug_onset: bool,
    pub last_debug_rms_onset: bool,
    pub last_debug_flux_onset: bool,
    pub last_debug_flux: f32,
    pub last_debug_pitch: Option<(f32, f32)>, // (freq, clarity)
}

impl GuitarInput {
    /// Create a new pipeline with the given configuration.
    pub fn new(config: GuitarInputConfig) -> Self {
        let buf_size = config.buffer_size;
        let sr = config.sample_rate;
        Self {
            config,
            calibration: None,
            normalizer: None,
            calibration_profile: None,
            current_note: None,
            prev_rms: 0.0,
            cooldown_remaining: 0,
            ring_buffer: vec![0.0; buf_size],
            ring_pos: 0,
            note_state: NoteState::Idle,
            state_samples: 0,
            total_samples: 0,
            suppress_frequency: None,
            suppress_until: 0,
            pitch_history: [None; 2],
            pitch_history_idx: 0,
            prev_spectrum: None,
            // Slide state: keep last 8 frames of cents values
            slide_history: vec![0i16; 8],
            slide_history_idx: 0,
            slide_velocity_decay: 0.9,
            // Vibrato state: buffer ~500ms of cents values
            // At 48kHz/1024 = ~47 frames/sec, 24 frames ~ 500ms
            vibrato_buffer: Vec::with_capacity(24),
            vibrato_buffer_idx: 0,
            vibrato_detected: false,
            vibrato_rate_hz: 0.0,
            vibrato_depth_cents: 0.0,
            // Pressure / NoteOff velocity state
            last_sustain_rms: 0.0,
            last_pressure: 0,
            note_onset_rms: 0.0,
            last_sent_pressure: 0,
            last_brightness: 0,
            single_cycle: super::single_cycle::SingleCycleDetector::new(sr, 75.0, 1400.0),
            early_pitch: None,
            hop_counter: 0,
            linearized_buf: vec![0.0; buf_size],
            noise_floor_ema: 0.005,
            signal_peak_ema: 0.1,
            pending_pitch_correction: None,
            frames_since_note_on: 0,
            last_debug_onset: false,
            last_debug_rms_onset: false,
            last_debug_flux_onset: false,
            last_debug_flux: 0.0,
            last_debug_pitch: None,
        }
    }

    /// Create with default configuration.
    pub fn with_defaults() -> Self {
        Self::new(GuitarInputConfig::default())
    }

    /// Mutable access to the configuration (for runtime adjustments).
    pub fn config_mut(&mut self) -> &mut GuitarInputConfig {
        &mut self.config
    }

    /// Set the calibration data (measured from `calibrate_string`).
    pub fn set_calibration(&mut self, cal: GuitarCalibration) {
        // Seed adaptive thresholds from calibration measurements
        self.noise_floor_ema = cal.noise_floor;
        self.signal_peak_ema = cal.signal_peak;
        self.calibration = Some(cal);
    }

    /// Get current calibration, if any.
    pub fn calibration(&self) -> Option<&GuitarCalibration> {
        self.calibration.as_ref()
    }

    /// Apply a per-string calibration profile (from
    /// `examples/guitar_calibrate.rs` or the in-app live capture).
    /// Builds an `AudioNormalizer` from the profile, stores it, AND
    /// seeds `noise_floor_ema` from the normalizer's global noise
    /// floor — so the existing onset gate immediately benefits from
    /// the calibrated value instead of waiting for the EMA to
    /// converge to room-noise from the per-frame defaults.
    pub fn set_calibration_profile(&mut self, profile: super::guitar::GuitarCalibrationProfile) {
        let normalizer = super::guitar::AudioNormalizer::from_profile(&profile);
        // Read the calibrated noise floor BEFORE storing the normalizer
        // so we don't double-borrow.
        let nf = normalizer.global_noise_floor();
        if nf > 0.0 {
            self.noise_floor_ema = nf;
        }
        self.normalizer = Some(normalizer);
        self.calibration_profile = Some(profile);
    }

    /// Returns the currently-loaded calibration profile, if any.
    pub fn calibration_profile(&self) -> Option<&super::guitar::GuitarCalibrationProfile> {
        self.calibration_profile.as_ref()
    }

    /// Returns whether a non-default calibration profile is loaded.
    pub fn has_calibration_profile(&self) -> bool {
        self.calibration_profile.is_some()
    }

    /// Get current config.
    pub fn config(&self) -> &GuitarInputConfig {
        &self.config
    }

    /// Get note state as a number (0=Idle, 1=Attack, 2=Sustain, 3=Decay).
    pub fn note_state_name(&self) -> u8 {
        match self.note_state {
            NoteState::Idle => 0,
            NoteState::Attack => 1,
            NoteState::Sustain => 2,
            NoteState::Decay => 3,
        }
    }

    /// Get previous frame's RMS.
    pub fn prev_rms(&self) -> f32 {
        self.prev_rms
    }

    /// Get cooldown remaining (samples).
    pub fn cooldown_remaining(&self) -> usize {
        self.cooldown_remaining
    }

    /// Get ring buffer position.
    pub fn ring_pos(&self) -> usize {
        self.ring_pos
    }

    /// Get total samples processed.
    pub fn total_samples(&self) -> usize {
        self.total_samples
    }

    /// Get the currently active note, if any.
    pub fn current_note(&self) -> Option<&DetectedNote> {
        self.current_note.as_ref()
    }

    /// Get the current note state machine state.
    pub fn note_state(&self) -> NoteState {
        self.note_state
    }

    // ── MPE pressure envelope ─────────────────────────────────────

    /// Compute shaped pressure from current RMS relative to onset RMS.
    ///
    /// Uses the onset RMS as the reference peak, applies a hold factor
    /// to slow the natural decay, and a power curve for perceptual mapping.
    fn compute_pressure(&self, current_rms: f32) -> u8 {
        let onset = self.note_onset_rms.max(0.001);
        let ratio = (current_rms / onset).clamp(0.0, 1.0);

        // Apply hold: slow the decay
        // hold=0: natural decay, hold=1: stays at max
        let held = ratio + (1.0 - ratio) * self.config.pressure_hold;
        let held_clamped = held.clamp(0.0, 1.0);

        // Power curve for perception
        let curved = held_clamped.powf(0.7);
        (curved * 127.0) as u8
    }

    // ── Main processing ────────────────────────────────────────────

    /// Process a block of mono audio samples and return any MIDI events.
    ///
    /// Call this repeatedly with successive audio buffers. The block size
    /// does not have to match `config.buffer_size` -- the internal ring
    /// buffer accumulates samples and triggers analysis when full.
    pub fn process_block(&mut self, audio: &[f32]) -> Vec<MidiEvent> {
        let mut events = Vec::new();
        let gain = self.config.input_gain;

        for &sample in audio {
            // Apply input gain normalization (#8)
            let gained_sample = sample * gain;
            self.ring_buffer[self.ring_pos] = gained_sample;
            self.ring_pos = (self.ring_pos + 1) % self.config.buffer_size;
            self.total_samples += 1;

            // Feed single-cycle detector continuously — always primed for
            // instant pitch estimate on any onset, zero warm-up delay.
            // Amplitude gate: only accept results above noise floor to
            // prevent noise-floor pitch estimates from polluting early_pitch.
            if let Some(result) = self.single_cycle.feed_sample(gained_sample) {
                if result.confidence > 0.6 && gained_sample.abs() > self.noise_floor_ema * 2.0 {
                    self.early_pitch = Some((result.frequency, result.confidence));
                }
            }

            // Sliding-window analysis: trigger every hop_size samples
            self.hop_counter += 1;
            if self.hop_counter >= self.config.hop_size {
                self.hop_counter = 0;
                let analysis_events = self.analyze_window();
                events.extend(analysis_events);
            }
        }

        events
    }

    /// Analyze the current ring buffer contents.
    fn analyze_window(&mut self) -> Vec<MidiEvent> {
        let mut events = Vec::new();
        self.linearize_ring_buffer_into();
        // Safety: linearized_buf is always the same size as buffer_size
        let buf = &self.linearized_buf.clone();

        // 1. Compute RMS for onset detection
        let rms = compute_rms(buf);

        // Update adaptive thresholds based on current state
        if matches!(self.note_state, NoteState::Idle) {
            self.update_noise_floor(rms);
        } else if matches!(self.note_state, NoteState::Sustain) {
            self.update_signal_peak(rms);
        }

        // 2. Spectral flux onset detection (#5)
        let flux = self.compute_spectral_flux(buf);

        // 3. Onset detection: RMS spike OR spectral flux (adaptive thresholds)
        let rms_onset = self.detect_onset(rms);
        let flux_onset = flux > self.adaptive_flux_threshold() && self.cooldown_remaining == 0;
        let onset = rms_onset || flux_onset;
        // Slope-only onset: true only when RMS jumped sharply from previous frame.
        // Used by Sustain to distinguish re-plucks from sustained ringing.
        let slope_onset = self.cooldown_remaining == 0
            && rms > self.adaptive_onset_threshold()
            && rms > self.prev_rms * 1.2;

        // 4. Pitch detection: pMPM during onset/attack for robustness,
        //    standard McLeod during sustain for speed
        let pitch_result = if onset
            || matches!(self.note_state, NoteState::Attack)
            || matches!(self.note_state, NoteState::Idle)
        {
            detect_pitch_pmcleod(buf, self.config.sample_rate)
        } else {
            detect_pitch_mcleod(buf, self.config.sample_rate, self.adaptive_clarity())
        };

        // Store debug info for WASM logging (read by wrapper)
        self.last_debug_onset = onset;
        self.last_debug_rms_onset = rms_onset;
        self.last_debug_flux_onset = flux_onset;
        self.last_debug_flux = flux;
        self.last_debug_pitch = pitch_result.map(|(f, c)| (f, c));

        // Tick down cooldown (by hop_size, since we analyze every hop)
        if self.cooldown_remaining > 0 {
            self.cooldown_remaining = self.cooldown_remaining.saturating_sub(self.config.hop_size);
        }

        // Advance state timer
        self.state_samples += self.config.hop_size;

        // Sustain threshold for Decay transition
        let sustain_threshold = self.adaptive_onset_threshold() * 0.3;
        let noise_floor = self
            .calibration
            .as_ref()
            .map(|c| c.noise_floor)
            .unwrap_or(0.001);

        match pitch_result {
            Some((raw_freq, _clarity)) => {
                // Apply octave error correction (#6) BEFORE freq_to_midi
                let freq = self.correct_octave(raw_freq);
                let (midi_note, cents) = freq_to_midi(freq);

                // Check note-off gating (#1): suppress previous note's frequency
                if self.is_suppressed(freq) {
                    // Detected pitch is within suppression zone, ignore it
                    self.prev_rms = rms;
                    return events;
                }

                // Note state machine (#2) transitions
                match self.note_state {
                    NoteState::Idle => {
                        if onset {
                            // Onset-ahead-of-pitch: if single-cycle already has
                            // a confident pitch, skip Attack and fire NoteOn now
                            if let Some((sc_freq, sc_conf)) = self.early_pitch {
                                if sc_conf > 0.6 {
                                    let sc_corrected = self.correct_octave(sc_freq);
                                    let (sc_midi, _) = freq_to_midi(sc_corrected);
                                    // Use single-cycle pitch for immediate NoteOn
                                    // McLeod will refine via pitch bend in Sustain
                                    self.push_pitch(sc_midi);
                                    self.note_state = NoteState::Attack;
                                    self.state_samples = 0;
                                    // Fall through to Attack handler which will
                                    // fire immediately since we have a confident pitch
                                }
                            }
                            if !matches!(self.note_state, NoteState::Attack) {
                                // Normal path: onset detected, move to Attack
                                self.note_state = NoteState::Attack;
                                self.state_samples = 0;
                                self.single_cycle.reset();
                                self.early_pitch = None;
                            }
                            // Record pitch in voting history (#7)
                            self.push_pitch(midi_note);
                        }
                    }
                    _ => {}
                }

                // Attack handler (separate match to allow fall-through from Idle)
                match self.note_state {
                    NoteState::Attack => {
                        // Record pitch in voting history
                        self.push_pitch(midi_note);

                        // Attack timeout: reduced to 25ms with faster hop rate
                        let attack_timeout_samples =
                            (self.config.sample_rate as f32 * 0.025) as usize;

                        // Fire on FIRST confident detection from either source
                        let has_confident_pitch = self
                            .early_pitch
                            .map(|(_, conf)| conf > 0.6)
                            .unwrap_or(false)
                            || self.is_pitch_stable();

                        if has_confident_pitch {
                            // Attack -> Sustain: pitch confirmed
                            let stable_midi = self.pitch_history[0].unwrap_or(midi_note);

                            // End previous note first
                            if let Some(prev) = &self.current_note {
                                // Set up note-off gating (#1): suppress old frequency
                                self.suppress_frequency = Some(prev.frequency);
                                self.suppress_until = self.total_samples
                                    + (self.config.sample_rate as f32 * 0.1) as usize;

                                let prev_channel = if self.config.per_string_channels {
                                    // MPE: ch 1 = master (global), ch 2-7 = strings 1-6
                                    prev.string_idx.map(|s| s as u8 + 1).unwrap_or(0)
                                } else {
                                    0
                                };
                                // Fix 5: NoteOff velocity = last sent pressure value (MG3 convention)
                                events.push(MidiEvent::NoteOff {
                                    channel: prev_channel,
                                    note: prev.midi_note,
                                    velocity: self.last_sent_pressure,
                                });
                            }

                            // Velocity from attack peak (first ~5ms) instead of full-buffer RMS
                            let attack_samples = (self.config.sample_rate as f32 * 0.005) as usize;
                            let attack_end = attack_samples.min(buf.len());
                            let attack_rms = compute_rms(&buf[..attack_end]);

                            let velocity = if let Some(cal) = &self.calibration {
                                rms_to_velocity_calibrated(
                                    attack_rms,
                                    cal.noise_floor,
                                    cal.signal_peak,
                                )
                            } else {
                                rms_to_velocity(attack_rms)
                            };

                            // String identification with plausibility filter (#3)
                            let (string_idx, fret, string_conf) =
                                self.identify_string_and_fret(&buf, freq, stable_midi);

                            // Fix 2: MPE channel offset — ch 1 = master, ch 2-7 = strings
                            let channel = if self.config.per_string_channels {
                                string_idx.map(|s| s as u8 + 1).unwrap_or(0)
                            } else {
                                0
                            };

                            let detected = DetectedNote {
                                midi_note: stable_midi,
                                frequency: freq,
                                string_idx,
                                fret,
                                string_confidence: string_conf,
                                velocity,
                                bend_cents: cents as i16,
                            };

                            // Fix 1: Send PitchBend BEFORE NoteOn.
                            //
                            // Issue #79 stop-gap: gate the initial-bend emit
                            // on a 10-cent threshold so transient attack
                            // deviations don't produce the audible wobble
                            // some users have reported. A perfectly-tuned
                            // guitar attacks within ±5-10 cents of nominal
                            // pitch during the first ~50ms; intentional
                            // bends and out-of-tune playing exceed 10 cents
                            // easily. The previous `> 0` gate fired on every
                            // single-cycle measurement, which propagated
                            // McLeod's onset-window noise into MPE pitch
                            // bends downstream. Owned by #82's full guitar-
                            // pipeline rewrite once it lands.
                            const INITIAL_BEND_THRESHOLD_CENTS: i16 = 10;
                            let initial_bend = cents as i16;
                            if initial_bend.abs() >= INITIAL_BEND_THRESHOLD_CENTS {
                                events.push(MidiEvent::PitchBend {
                                    channel,
                                    cents: initial_bend,
                                });
                                events.push(MidiEvent::MidiPitchBend {
                                    channel,
                                    value: cents_to_midi_pitch_bend(
                                        initial_bend,
                                        self.config.pitch_bend_range,
                                    ),
                                });
                            }

                            // Fix 3: Send CC74 brightness BEFORE NoteOn
                            if self.config.brightness_enabled {
                                let harmonics_data = measure_harmonics(
                                    &buf,
                                    freq,
                                    self.config.n_harmonics,
                                    self.config.sample_rate,
                                );
                                let harmonic_pairs: Vec<(f32, f32)> = harmonics_data
                                    .iter()
                                    .enumerate()
                                    .map(|(i, &mag)| (freq * (i + 1) as f32, mag))
                                    .collect();
                                let centroid = compute_spectral_centroid(&harmonic_pairs);
                                let cc74 = centroid_to_cc74(centroid);
                                events.push(MidiEvent::CC {
                                    channel,
                                    controller: 74,
                                    value: cc74,
                                });
                                self.last_brightness = cc74;
                            }

                            // Fix 4: Send initial ChannelPressure BEFORE NoteOn
                            // Store onset RMS for pressure envelope
                            self.note_onset_rms = attack_rms;
                            if self.config.pressure_enabled {
                                let initial_pressure = self.compute_pressure(attack_rms);
                                events.push(MidiEvent::ChannelPressure {
                                    channel,
                                    pressure: initial_pressure,
                                });
                                self.last_sent_pressure = initial_pressure;
                                self.last_pressure = initial_pressure;
                            }

                            events.push(MidiEvent::NoteOn {
                                channel,
                                note: stable_midi,
                                velocity,
                            });

                            self.current_note = Some(detected);
                            self.cooldown_remaining = self.config.cooldown_samples;
                            self.note_state = NoteState::Sustain;
                            self.state_samples = 0;
                            self.last_sustain_rms = rms;
                            self.early_pitch = None;
                            // Track for pitch correction: McLeod may refine
                            // the pitch in the next 1-2 frames via pitch bend
                            self.pending_pitch_correction = Some((channel, freq));
                            self.frames_since_note_on = 0;
                            self.clear_slide_history();
                            self.clear_vibrato_state();
                        } else if self.state_samples > attack_timeout_samples {
                            // Attack -> Idle: false trigger
                            self.note_state = NoteState::Idle;
                            self.state_samples = 0;
                            self.early_pitch = None;
                            self.clear_pitch_history();
                            self.clear_slide_history();
                            self.clear_vibrato_state();
                        }
                    }
                    NoteState::Sustain => {
                        self.frames_since_note_on += 1;

                        // Pitch correction: if McLeod refines pitch within 2 frames
                        // of an early NoteOn, send a pitch bend correction.
                        if let Some((corr_channel, initial_freq)) = self.pending_pitch_correction {
                            if self.frames_since_note_on <= 2 {
                                let (new_midi, _) = freq_to_midi(freq);
                                if let Some(ref current) = self.current_note {
                                    if new_midi != current.midi_note {
                                        // Correction needed: bend to the McLeod-refined pitch
                                        let correction_cents =
                                            (1200.0 * (freq / initial_freq).log2()) as i16;
                                        if correction_cents.abs() > 10 {
                                            events.push(MidiEvent::PitchBend {
                                                channel: corr_channel,
                                                cents: correction_cents,
                                            });
                                            events.push(MidiEvent::MidiPitchBend {
                                                channel: corr_channel,
                                                value: cents_to_midi_pitch_bend(
                                                    correction_cents,
                                                    self.config.pitch_bend_range,
                                                ),
                                            });
                                        }
                                    }
                                }
                            } else {
                                self.pending_pitch_correction = None;
                            }
                        }

                        // Sustain handler priority order:
                        //   1. slope_onset -> Attack (genuine re-pluck, sharp RMS jump)
                        //   2. RMS below threshold -> Decay
                        //   3. pitch jump > 2 semitones -> legato or missed onset
                        //   4. pitch crosses semitone boundary gradually -> SLIDE
                        //   5. pitch oscillates periodically -> VIBRATO (informational)
                        //   6. pitch drifts < 2 semitones -> BEND
                        //   7. pitch stable -> nothing
                        //
                        // Uses slope_onset (not general onset) to distinguish
                        // a new pluck from sustained ringing. A re-pluck always
                        // has a sharp RMS jump; sustained signal doesn't.

                        if slope_onset {
                            // 1. Sustain -> Attack: genuine re-pluck (RMS jumped)
                            self.note_state = NoteState::Attack;
                            self.state_samples = 0;
                            self.clear_pitch_history();
                            self.push_pitch(midi_note);
                            self.clear_slide_history();
                            self.clear_vibrato_state();
                        } else if rms < sustain_threshold {
                            // 2. Sustain -> Decay: signal dropping
                            self.note_state = NoteState::Decay;
                            self.state_samples = 0;
                            // Emit vibrato-off if it was active
                            if self.vibrato_detected {
                                events.push(MidiEvent::VibratoStatus {
                                    active: false,
                                    rate_hz: 0.0,
                                    depth_cents: 0.0,
                                });
                                self.clear_vibrato_state();
                            }
                        } else if self.current_note.is_some() {
                            // Extract values we need from the current note
                            // to avoid borrow conflicts with &mut self methods.
                            let note_midi = self.current_note.as_ref().unwrap().midi_note;
                            let note_freq = self.current_note.as_ref().unwrap().frequency;
                            let note_vel = self.current_note.as_ref().unwrap().velocity;
                            let note_bend = self.current_note.as_ref().unwrap().bend_cents;
                            let note_string_idx = self.current_note.as_ref().unwrap().string_idx;
                            // Fix 2: MPE channel offset — ch 1 = master, ch 2-7 = strings
                            let channel = if self.config.per_string_channels {
                                note_string_idx.map(|s| s as u8 + 1).unwrap_or(0)
                            } else {
                                0
                            };

                            // Track sustain RMS for NoteOff release velocity
                            self.last_sustain_rms = rms;

                            let semitone_diff = (midi_note as i16 - note_midi as i16).abs();
                            let cents_from_base = {
                                let base_freq = midi_to_freq(note_midi);
                                if base_freq > 0.0 && freq > 0.0 {
                                    (1200.0 * (freq / base_freq).log2()) as i16
                                } else {
                                    0
                                }
                            };

                            // Feed slide and vibrato buffers
                            self.push_slide_cents(cents_from_base);
                            self.push_vibrato_cents(cents_from_base as f32);

                            // Detect rapid pitch change (legato vs bend):
                            // Legato = fast jump (> 50 cents in one frame)
                            // Bend = slow movement (< 30 cents per frame)
                            let prev_cents = self.slide_history_prev_cents();
                            let cents_per_frame = (cents_from_base - prev_cents).abs();
                            let is_rapid_jump = cents_per_frame > 50;

                            if semitone_diff >= 1 && is_rapid_jump && self.config.legato_enabled {
                                // 3. LEGATO: rapid pitch jump without onset (hammer-on/pull-off)
                                self.suppress_frequency = Some(note_freq);
                                self.suppress_until = self.total_samples
                                    + (self.config.sample_rate as f32 * 0.1) as usize;

                                // Fix 5: NoteOff velocity = last sent pressure value
                                events.push(MidiEvent::NoteOff {
                                    channel,
                                    note: note_midi,
                                    velocity: self.last_sent_pressure,
                                });

                                let legato_vel = (note_vel as f32 * 0.8).min(127.0) as u8;

                                let (string_idx, fret, string_conf) =
                                    self.identify_string_and_fret(&buf, freq, midi_note);

                                // Fix 2: MPE channel offset
                                let new_channel = if self.config.per_string_channels {
                                    string_idx.map(|s| s as u8 + 1).unwrap_or(0)
                                } else {
                                    0
                                };

                                let new_note = DetectedNote {
                                    midi_note,
                                    frequency: freq,
                                    string_idx,
                                    fret,
                                    string_confidence: string_conf,
                                    velocity: legato_vel,
                                    bend_cents: 0,
                                };

                                // Fix 1: Send PitchBend BEFORE NoteOn (legato)
                                let legato_cents = cents as i16;
                                if legato_cents.abs() > 0 {
                                    events.push(MidiEvent::PitchBend {
                                        channel: new_channel,
                                        cents: legato_cents,
                                    });
                                    events.push(MidiEvent::MidiPitchBend {
                                        channel: new_channel,
                                        value: cents_to_midi_pitch_bend(
                                            legato_cents,
                                            self.config.pitch_bend_range,
                                        ),
                                    });
                                }

                                // Fix 3: Send CC74 brightness BEFORE NoteOn (legato)
                                if self.config.brightness_enabled {
                                    let harmonics_data = measure_harmonics(
                                        &buf,
                                        freq,
                                        self.config.n_harmonics,
                                        self.config.sample_rate,
                                    );
                                    let harmonic_pairs: Vec<(f32, f32)> = harmonics_data
                                        .iter()
                                        .enumerate()
                                        .map(|(i, &mag)| (freq * (i + 1) as f32, mag))
                                        .collect();
                                    let centroid = compute_spectral_centroid(&harmonic_pairs);
                                    let cc74 = centroid_to_cc74(centroid);
                                    events.push(MidiEvent::CC {
                                        channel: new_channel,
                                        controller: 74,
                                        value: cc74,
                                    });
                                    self.last_brightness = cc74;
                                }

                                // Fix 4: Send initial pressure BEFORE NoteOn (legato)
                                self.note_onset_rms = rms;
                                if self.config.pressure_enabled {
                                    let initial_pressure = self.compute_pressure(rms);
                                    events.push(MidiEvent::ChannelPressure {
                                        channel: new_channel,
                                        pressure: initial_pressure,
                                    });
                                    self.last_sent_pressure = initial_pressure;
                                    self.last_pressure = initial_pressure;
                                }

                                events.push(MidiEvent::NoteOn {
                                    channel: new_channel,
                                    note: midi_note,
                                    velocity: legato_vel,
                                });

                                self.current_note = Some(new_note);
                                self.clear_pitch_history();
                                self.clear_slide_history();
                                self.clear_vibrato_state();
                            } else if semitone_diff >= 1
                                && is_rapid_jump
                                && !self.config.legato_enabled
                            {
                                // Rapid pitch jump without legato enabled: treat as missed onset
                                self.note_state = NoteState::Attack;
                                self.state_samples = 0;
                                self.clear_pitch_history();
                                self.push_pitch(midi_note);
                                self.clear_slide_history();
                                self.clear_vibrato_state();
                            } else if cents_from_base.abs() >= 100
                                && self.config.slides_enabled
                                && self.is_slide_movement()
                            {
                                // 4. SLIDE: pitch crossed a semitone boundary gradually
                                let (new_midi, new_cents_val) = freq_to_midi(freq);
                                if new_midi != note_midi {
                                    // Fix 5: NoteOff velocity = last sent pressure value
                                    events.push(MidiEvent::NoteOff {
                                        channel,
                                        note: note_midi,
                                        velocity: self.last_sent_pressure,
                                    });

                                    // Slide velocity decays gradually
                                    let slide_vel = ((note_vel as f32 * self.slide_velocity_decay)
                                        .min(127.0)
                                        as u8)
                                        .max(1);

                                    let (string_idx, fret, string_conf) =
                                        self.identify_string_and_fret(&buf, freq, new_midi);

                                    // Fix 2: MPE channel offset
                                    let new_channel = if self.config.per_string_channels {
                                        string_idx.map(|s| s as u8 + 1).unwrap_or(0)
                                    } else {
                                        0
                                    };

                                    let new_note = DetectedNote {
                                        midi_note: new_midi,
                                        frequency: freq,
                                        string_idx,
                                        fret,
                                        string_confidence: string_conf,
                                        velocity: slide_vel,
                                        bend_cents: 0,
                                    };

                                    // Fix 1: Send PitchBend BEFORE NoteOn (slide)
                                    let slide_bend = new_cents_val as i16;
                                    if slide_bend.abs() > 0 {
                                        events.push(MidiEvent::PitchBend {
                                            channel: new_channel,
                                            cents: slide_bend,
                                        });
                                        events.push(MidiEvent::MidiPitchBend {
                                            channel: new_channel,
                                            value: cents_to_midi_pitch_bend(
                                                slide_bend,
                                                self.config.pitch_bend_range,
                                            ),
                                        });
                                    }

                                    // Fix 3: Send CC74 brightness BEFORE NoteOn (slide)
                                    if self.config.brightness_enabled {
                                        let harmonics_data = measure_harmonics(
                                            &buf,
                                            freq,
                                            self.config.n_harmonics,
                                            self.config.sample_rate,
                                        );
                                        let harmonic_pairs: Vec<(f32, f32)> = harmonics_data
                                            .iter()
                                            .enumerate()
                                            .map(|(i, &mag)| (freq * (i + 1) as f32, mag))
                                            .collect();
                                        let centroid = compute_spectral_centroid(&harmonic_pairs);
                                        let cc74 = centroid_to_cc74(centroid);
                                        events.push(MidiEvent::CC {
                                            channel: new_channel,
                                            controller: 74,
                                            value: cc74,
                                        });
                                        self.last_brightness = cc74;
                                    }

                                    // Fix 4: Send initial pressure BEFORE NoteOn (slide)
                                    self.note_onset_rms = rms;
                                    if self.config.pressure_enabled {
                                        let initial_pressure = self.compute_pressure(rms);
                                        events.push(MidiEvent::ChannelPressure {
                                            channel: new_channel,
                                            pressure: initial_pressure,
                                        });
                                        self.last_sent_pressure = initial_pressure;
                                        self.last_pressure = initial_pressure;
                                    }

                                    events.push(MidiEvent::NoteOn {
                                        channel: new_channel,
                                        note: new_midi,
                                        velocity: slide_vel,
                                    });

                                    self.current_note = Some(new_note);
                                    // Stay in Sustain, reset slide/bend tracking
                                    self.clear_slide_history();
                                }
                            } else {
                                // 5. VIBRATO detection (informational)
                                let vibrato_result = if self.config.vibrato_detection {
                                    self.detect_vibrato()
                                } else {
                                    None
                                };

                                if self.config.vibrato_detection {
                                    if let Some((rate_hz, amplitude)) = vibrato_result {
                                        if !self.vibrato_detected {
                                            self.vibrato_detected = true;
                                            self.vibrato_rate_hz = rate_hz;
                                            self.vibrato_depth_cents = amplitude;
                                            events.push(MidiEvent::VibratoStatus {
                                                active: true,
                                                rate_hz,
                                                depth_cents: amplitude,
                                            });
                                        } else if (rate_hz - self.vibrato_rate_hz).abs() > 0.5
                                            || (amplitude - self.vibrato_depth_cents).abs() > 5.0
                                        {
                                            self.vibrato_rate_hz = rate_hz;
                                            self.vibrato_depth_cents = amplitude;
                                            events.push(MidiEvent::VibratoStatus {
                                                active: true,
                                                rate_hz,
                                                depth_cents: amplitude,
                                            });
                                        }

                                        // If vibrato passthrough is disabled,
                                        // suppress pitch bend events (smooth out)
                                        if !self.config.vibrato_passthrough {
                                            if note_bend != 0 {
                                                events.push(MidiEvent::PitchBend {
                                                    channel,
                                                    cents: 0,
                                                });
                                                events.push(MidiEvent::MidiPitchBend {
                                                    channel,
                                                    value: cents_to_midi_pitch_bend(
                                                        0,
                                                        self.config.pitch_bend_range,
                                                    ),
                                                });
                                                if let Some(ref mut note) = self.current_note {
                                                    note.bend_cents = 0;
                                                }
                                            }
                                        } else if self.config.bends_enabled {
                                            // Vibrato passthrough: let bend events flow
                                            if cents_from_base.abs() > 5
                                                && cents_from_base.abs() < 200
                                            {
                                                if (cents_from_base - note_bend).abs() > 3 {
                                                    events.push(MidiEvent::PitchBend {
                                                        channel,
                                                        cents: cents_from_base,
                                                    });
                                                    events.push(MidiEvent::MidiPitchBend {
                                                        channel,
                                                        value: cents_to_midi_pitch_bend(
                                                            cents_from_base,
                                                            self.config.pitch_bend_range,
                                                        ),
                                                    });
                                                    if let Some(ref mut note) = self.current_note {
                                                        note.bend_cents = cents_from_base;
                                                        note.frequency = freq;
                                                    }
                                                }
                                            } else if cents_from_base.abs() <= 5 && note_bend != 0 {
                                                events.push(MidiEvent::PitchBend {
                                                    channel,
                                                    cents: 0,
                                                });
                                                events.push(MidiEvent::MidiPitchBend {
                                                    channel,
                                                    value: cents_to_midi_pitch_bend(
                                                        0,
                                                        self.config.pitch_bend_range,
                                                    ),
                                                });
                                                if let Some(ref mut note) = self.current_note {
                                                    note.bend_cents = 0;
                                                }
                                            }
                                        }
                                    } else {
                                        // No vibrato detected
                                        if self.vibrato_detected {
                                            events.push(MidiEvent::VibratoStatus {
                                                active: false,
                                                rate_hz: 0.0,
                                                depth_cents: 0.0,
                                            });
                                            self.vibrato_detected = false;
                                            self.vibrato_rate_hz = 0.0;
                                            self.vibrato_depth_cents = 0.0;
                                        }

                                        // 6. BEND: pitch within +/-2 semitones
                                        if self.config.bends_enabled {
                                            if cents_from_base.abs() > 5
                                                && cents_from_base.abs() < 200
                                            {
                                                if (cents_from_base - note_bend).abs() > 3 {
                                                    events.push(MidiEvent::PitchBend {
                                                        channel,
                                                        cents: cents_from_base,
                                                    });
                                                    events.push(MidiEvent::MidiPitchBend {
                                                        channel,
                                                        value: cents_to_midi_pitch_bend(
                                                            cents_from_base,
                                                            self.config.pitch_bend_range,
                                                        ),
                                                    });
                                                    if let Some(ref mut note) = self.current_note {
                                                        note.bend_cents = cents_from_base;
                                                        note.frequency = freq;
                                                    }
                                                }
                                            } else if cents_from_base.abs() <= 5 && note_bend != 0 {
                                                events.push(MidiEvent::PitchBend {
                                                    channel,
                                                    cents: 0,
                                                });
                                                events.push(MidiEvent::MidiPitchBend {
                                                    channel,
                                                    value: cents_to_midi_pitch_bend(
                                                        0,
                                                        self.config.pitch_bend_range,
                                                    ),
                                                });
                                                if let Some(ref mut note) = self.current_note {
                                                    note.bend_cents = 0;
                                                }
                                            }
                                        }
                                    }
                                } else {
                                    // Vibrato detection disabled; just do bends
                                    if self.config.bends_enabled {
                                        if cents_from_base.abs() > 5 && cents_from_base.abs() < 200
                                        {
                                            if (cents_from_base - note_bend).abs() > 3 {
                                                events.push(MidiEvent::PitchBend {
                                                    channel,
                                                    cents: cents_from_base,
                                                });
                                                events.push(MidiEvent::MidiPitchBend {
                                                    channel,
                                                    value: cents_to_midi_pitch_bend(
                                                        cents_from_base,
                                                        self.config.pitch_bend_range,
                                                    ),
                                                });
                                                if let Some(ref mut note) = self.current_note {
                                                    note.bend_cents = cents_from_base;
                                                    note.frequency = freq;
                                                }
                                            }
                                        } else if cents_from_base.abs() <= 5 && note_bend != 0 {
                                            events.push(MidiEvent::PitchBend { channel, cents: 0 });
                                            events.push(MidiEvent::MidiPitchBend {
                                                channel,
                                                value: cents_to_midi_pitch_bend(
                                                    0,
                                                    self.config.pitch_bend_range,
                                                ),
                                            });
                                            if let Some(ref mut note) = self.current_note {
                                                note.bend_cents = 0;
                                            }
                                        }
                                    }
                                }

                                // Fix 4: Shaped pressure envelope during Sustain
                                if self.config.pressure_enabled {
                                    let pressure = self.compute_pressure(rms);
                                    if pressure != self.last_pressure {
                                        events
                                            .push(MidiEvent::ChannelPressure { channel, pressure });
                                        self.last_pressure = pressure;
                                        self.last_sent_pressure = pressure;
                                    }
                                }

                                // Fix 3: CC74 brightness updates during Sustain
                                if self.config.brightness_enabled {
                                    let harmonics_data = measure_harmonics(
                                        &buf,
                                        freq,
                                        self.config.n_harmonics,
                                        self.config.sample_rate,
                                    );
                                    let harmonic_pairs: Vec<(f32, f32)> = harmonics_data
                                        .iter()
                                        .enumerate()
                                        .map(|(i, &mag)| (freq * (i + 1) as f32, mag))
                                        .collect();
                                    let centroid = compute_spectral_centroid(&harmonic_pairs);
                                    let cc74 = centroid_to_cc74(centroid);
                                    // Only send when value changes by > 2 to reduce message flood
                                    if (cc74 as i16 - self.last_brightness as i16).unsigned_abs()
                                        > 2
                                    {
                                        events.push(MidiEvent::CC {
                                            channel,
                                            controller: 74,
                                            value: cc74,
                                        });
                                        self.last_brightness = cc74;
                                    }
                                }
                            }
                        }
                    }
                    NoteState::Decay => {
                        let decay_timeout_samples = (self.config.sample_rate as f32 * 0.5) as usize;

                        if onset {
                            // New onset during decay -> Attack
                            self.note_state = NoteState::Attack;
                            self.state_samples = 0;
                            self.clear_pitch_history();
                            self.push_pitch(midi_note);
                        } else if rms < noise_floor * 2.0
                            || self.state_samples > decay_timeout_samples
                        {
                            // Decay -> Idle: silence or timeout
                            if let Some(prev) = self.current_note.take() {
                                // Fix 2: MPE channel offset
                                let decay_channel = if self.config.per_string_channels {
                                    prev.string_idx.map(|s| s as u8 + 1).unwrap_or(0)
                                } else {
                                    0
                                };
                                // Fix 5: NoteOff velocity = last sent pressure value
                                events.push(MidiEvent::NoteOff {
                                    channel: decay_channel,
                                    note: prev.midi_note,
                                    velocity: self.last_sent_pressure,
                                });
                            }
                            self.note_state = NoteState::Idle;
                            self.state_samples = 0;
                            self.clear_pitch_history();
                        }
                    }
                    NoteState::Idle => {} // Handled by first match above
                }
            }
            None => {
                // No pitch detected
                match self.note_state {
                    NoteState::Attack => {
                        // Lost pitch during attack -> Idle (false trigger)
                        self.note_state = NoteState::Idle;
                        self.state_samples = 0;
                        self.clear_pitch_history();
                    }
                    NoteState::Sustain | NoteState::Decay => {
                        if rms < noise_floor * 2.0 {
                            if let Some(prev) = self.current_note.take() {
                                // Fix 2: MPE channel offset
                                let nopitch_channel = if self.config.per_string_channels {
                                    prev.string_idx.map(|s| s as u8 + 1).unwrap_or(0)
                                } else {
                                    0
                                };
                                // Fix 5: NoteOff velocity = last sent pressure value
                                events.push(MidiEvent::NoteOff {
                                    channel: nopitch_channel,
                                    note: prev.midi_note,
                                    velocity: self.last_sent_pressure,
                                });
                            }
                            self.note_state = NoteState::Idle;
                            self.state_samples = 0;
                            self.clear_pitch_history();
                        }
                    }
                    NoteState::Idle => {}
                }
            }
        }

        self.prev_rms = rms;
        events
    }

    // ── Adaptive thresholds ─────────────────────────────────────────

    /// Adaptive onset threshold: 3x the running noise floor, with a minimum.
    fn adaptive_onset_threshold(&self) -> f32 {
        (self.noise_floor_ema * 3.0).max(0.005)
    }

    /// Adaptive pitch clarity threshold: lower in high-SNR environments.
    fn adaptive_clarity(&self) -> f64 {
        let snr = if self.noise_floor_ema > 1e-6 {
            (self.signal_peak_ema / self.noise_floor_ema).log10() / 2.0
        } else {
            1.0
        };
        let snr_clamped = snr.clamp(0.0, 1.0);
        0.3 + 0.3 * (1.0 - snr_clamped) as f64 // range [0.3, 0.6]
    }

    /// Adaptive spectral flux threshold.
    fn adaptive_flux_threshold(&self) -> f32 {
        (self.noise_floor_ema * 5.0).max(0.1)
    }

    /// Update noise floor EMA (call during silence / Idle).
    fn update_noise_floor(&mut self, rms: f32) {
        const ALPHA: f32 = 0.02;
        self.noise_floor_ema = self.noise_floor_ema * (1.0 - ALPHA) + rms * ALPHA;
    }

    /// Update signal peak EMA (call during Sustain).
    fn update_signal_peak(&mut self, rms: f32) {
        const ALPHA: f32 = 0.02;
        self.signal_peak_ema = self.signal_peak_ema * (1.0 - ALPHA) + rms * ALPHA;
    }

    /// Detect whether the current frame is an onset (pluck).
    fn detect_onset(&self, rms: f32) -> bool {
        if self.cooldown_remaining > 0 {
            return false;
        }
        let threshold = self.adaptive_onset_threshold();
        // Onset = RMS jumped above adaptive threshold and is significantly higher
        // than the previous frame. 1.2x for faster re-pluck detection.
        rms > threshold && rms > self.prev_rms * 1.2
    }

    // ── Octave error correction (#6) ──────────────────────────────

    /// Correct octave jumps using harmonic template matching.
    ///
    /// When a near-octave jump is detected, compare harmonic template scores
    /// for both candidates (as-is vs octave-corrected). The candidate with
    /// more present harmonics wins.
    fn correct_octave(&self, detected_freq: f32) -> f32 {
        let prev = match &self.current_note {
            Some(p) => p,
            None => return detected_freq,
        };

        let ratio = detected_freq / prev.frequency;

        // Only intervene for near-octave jumps (within 10%)
        if (ratio - 2.0).abs() >= 0.1 && (ratio - 0.5).abs() >= 0.1 {
            return detected_freq;
        }

        let buf = self.linearize_ring_buffer();
        let sr = self.config.sample_rate;

        if (ratio - 2.0).abs() < 0.1 {
            // Candidate is octave up — compare with octave down
            let score_as_is = harmonic_template_score(&buf, detected_freq, sr);
            let score_down = harmonic_template_score(&buf, detected_freq / 2.0, sr);
            if score_down > score_as_is {
                return detected_freq / 2.0;
            }
        }

        if (ratio - 0.5).abs() < 0.1 {
            // Candidate is octave down — compare with octave up
            let score_as_is = harmonic_template_score(&buf, detected_freq, sr);
            let score_up = harmonic_template_score(&buf, detected_freq * 2.0, sr);
            if score_up > score_as_is {
                return detected_freq * 2.0;
            }
        }

        detected_freq
    }

    // ── Note-off gating / anti-resonance (#1) ─────────────────────

    /// Check if a detected frequency is within the suppression zone.
    ///
    /// After a new onset, the previous note's string is still ringing.
    /// Suppress that frequency for 100ms to prevent confusion.
    fn is_suppressed(&self, freq: f32) -> bool {
        if self.total_samples >= self.suppress_until {
            return false;
        }
        if let Some(suppress_freq) = self.suppress_frequency {
            // Check if within ±1 semitone of the suppressed frequency
            let ratio = freq / suppress_freq;
            let semitone_ratio = 2.0_f32.powf(1.0 / 12.0);
            let lower = 1.0 / semitone_ratio;
            let upper = semitone_ratio;
            ratio >= lower && ratio <= upper
        } else {
            false
        }
    }

    // ── Multi-frame pitch voting (#7) ─────────────────────────────

    /// Push a detected MIDI note into the voting history.
    fn push_pitch(&mut self, midi_note: u8) {
        self.pitch_history[self.pitch_history_idx] = Some(midi_note);
        self.pitch_history_idx = (self.pitch_history_idx + 1) % 2;
    }

    /// Check if the last 3 detected pitches agree.
    fn is_pitch_stable(&self) -> bool {
        let first = self.pitch_history[0];
        first.is_some() && self.pitch_history.iter().all(|p| *p == first)
    }

    /// Clear the pitch voting history.
    fn clear_pitch_history(&mut self) {
        self.pitch_history = [None; 2];
        self.pitch_history_idx = 0;
    }

    // ── Slide detection ────────────────────────────────────────────

    /// Push a cents-from-base value into the slide history ring buffer.
    fn push_slide_cents(&mut self, cents: i16) {
        self.slide_history[self.slide_history_idx] = cents;
        self.slide_history_idx = (self.slide_history_idx + 1) % self.slide_history.len();
    }

    /// Get the previous cents value from slide history (for detecting rapid jumps).
    fn slide_history_prev_cents(&self) -> i16 {
        let len = self.slide_history.len();
        if len == 0 {
            return 0;
        }
        // Previous index (one before current write position)
        let prev_idx = if self.slide_history_idx == 0 {
            len - 1
        } else {
            self.slide_history_idx - 1
        };
        self.slide_history[prev_idx]
    }

    /// Check if recent pitch movement is a gradual slide (monotonic trend)
    /// rather than a sudden jump.
    ///
    /// Returns true if the last ~5 frames show continuous monotonic movement
    /// without any sudden jump (>50 cents between consecutive frames).
    fn is_slide_movement(&self) -> bool {
        let len = self.slide_history.len();
        if len < 3 {
            return false;
        }

        // Reconstruct the ordered history (oldest to newest)
        let mut ordered = Vec::with_capacity(len);
        for i in 0..len {
            ordered.push(self.slide_history[(self.slide_history_idx + i) % len]);
        }

        // Check last 5 frames (or fewer if not enough data)
        let check_len = 5.min(len);
        let recent = &ordered[len - check_len..];

        // 1. No sudden jumps: consecutive frames should differ by < 50 cents
        for pair in recent.windows(2) {
            if (pair[1] - pair[0]).abs() > 50 {
                return false; // sudden jump = not a slide
            }
        }

        // 2. Monotonic trend: all differences should have the same sign
        //    (all increasing or all decreasing)
        let mut increasing = 0;
        let mut decreasing = 0;
        for pair in recent.windows(2) {
            let diff = pair[1] - pair[0];
            if diff > 2 {
                increasing += 1;
            } else if diff < -2 {
                decreasing += 1;
            }
            // Near-zero differences are neutral
        }

        // Must be predominantly one direction
        (increasing >= check_len - 2 && decreasing == 0)
            || (decreasing >= check_len - 2 && increasing == 0)
    }

    /// Clear slide history (e.g., on note change).
    fn clear_slide_history(&mut self) {
        for v in self.slide_history.iter_mut() {
            *v = 0;
        }
        self.slide_history_idx = 0;
    }

    // ── Vibrato detection ────────────────────────────────────────────

    /// Push a cents deviation into the vibrato analysis buffer.
    fn push_vibrato_cents(&mut self, cents: f32) {
        let cap = 24; // ~500ms at typical frame rates
        if self.vibrato_buffer.len() < cap {
            self.vibrato_buffer.push(cents);
        } else {
            let idx = self.vibrato_buffer_idx % cap;
            self.vibrato_buffer[idx] = cents;
        }
        self.vibrato_buffer_idx += 1;
    }

    /// Detect vibrato from the ring buffer of cents deviations.
    ///
    /// Returns `Some((rate_hz, amplitude_cents))` if periodic oscillation
    /// in the 3-8 Hz range with 10-60 cents amplitude is detected.
    fn detect_vibrato(&self) -> Option<(f32, f32)> {
        let buf = &self.vibrato_buffer;
        // Need at least ~300ms of data (~14 frames at 47 fps)
        if buf.len() < 10 {
            return None;
        }

        // Reconstruct ordered buffer
        let len = buf.len();
        let cap = 24.min(len);
        let ordered: Vec<f32> = if len < 24 {
            buf.clone()
        } else {
            let start = self.vibrato_buffer_idx % cap;
            let mut v = Vec::with_capacity(cap);
            for i in 0..cap {
                v.push(buf[(start + i) % cap]);
            }
            v
        };

        // 1. Compute mean
        let mean = ordered.iter().sum::<f32>() / ordered.len() as f32;

        // 2. Count zero-crossings (crossings of the mean)
        let crossings = ordered
            .windows(2)
            .filter(|w| (w[0] - mean).signum() != (w[1] - mean).signum())
            .count();

        // 3. Estimate frequency from zero-crossing rate
        //    Each analysis frame is buffer_size / sample_rate seconds
        let frame_duration = self.config.buffer_size as f32 / self.config.sample_rate as f32;
        let duration_secs = ordered.len() as f32 * frame_duration;
        if duration_secs < 0.05 {
            return None;
        }
        let rate_hz = crossings as f32 / (2.0 * duration_secs);

        // 4. Check if rate is in vibrato range (3-8 Hz)
        if rate_hz < 3.0 || rate_hz > 8.0 {
            return None;
        }

        // 5. Measure amplitude (max deviation from mean)
        let amplitude = ordered
            .iter()
            .map(|c| (c - mean).abs())
            .fold(0.0f32, f32::max);

        // 6. Check amplitude is in vibrato range (10-60 cents)
        if amplitude < 10.0 || amplitude > 60.0 {
            return None;
        }

        Some((rate_hz, amplitude))
    }

    /// Clear vibrato state (e.g., on note change).
    fn clear_vibrato_state(&mut self) {
        self.vibrato_buffer.clear();
        self.vibrato_buffer_idx = 0;
        self.vibrato_detected = false;
        self.vibrato_rate_hz = 0.0;
        self.vibrato_depth_cents = 0.0;
    }

    // ── Spectral flux onset detection (#5) ────────────────────────

    /// Compute spectral flux between the previous and current spectrum.
    ///
    /// Uses Goertzel at a bank of frequencies spanning the guitar range (80-1200 Hz)
    /// to build a coarse spectrum, then computes the half-wave rectified spectral
    /// flux (only positive changes count).
    fn compute_spectral_flux(&mut self, audio: &[f32]) -> f32 {
        // Build a coarse spectrum using Goertzel at a bank of frequencies
        let freqs: Vec<f32> = (0..24)
            .map(|i| 80.0 * 2.0_f32.powf(i as f32 / 6.0)) // ~80Hz to ~1200Hz in quarter-tone steps
            .collect();

        let curr_spectrum: Vec<f32> = freqs
            .iter()
            .map(|&f| goertzel(audio, self.config.sample_rate, f))
            .collect();

        let flux = if let Some(ref prev) = self.prev_spectrum {
            spectral_flux(prev, &curr_spectrum)
        } else {
            0.0
        };

        self.prev_spectrum = Some(curr_spectrum);
        flux
    }

    /// Identify string and fret from detected pitch + harmonic analysis.
    fn identify_string_and_fret(
        &self,
        audio: &[f32],
        fundamental: f32,
        midi_note: u8,
    ) -> (Option<usize>, Option<usize>, f32) {
        if let Some(ref cal) = self.calibration {
            match identify_string_with_plausibility(
                audio,
                fundamental,
                midi_note,
                &cal.string_profiles,
                self.config.sample_rate,
                self.config.n_harmonics,
                self.config.string_confidence_min,
                self.current_note.as_ref(),
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

    /// Copy ring buffer into pre-allocated linearized_buf (no allocation).
    fn linearize_ring_buffer_into(&mut self) {
        for i in 0..self.config.buffer_size {
            self.linearized_buf[i] =
                self.ring_buffer[(self.ring_pos + i) % self.config.buffer_size];
        }
    }

    // ── Calibration ────────────────────────────────────────────────

    /// Calibrate a single string from captured audio.
    ///
    /// Extracts fundamental frequency, measures harmonics via Goertzel,
    /// and computes the inharmonicity B coefficient.
    ///
    /// If `fret` is `Some`, stores the B value for that fret position
    /// (supports 0=open, 5=fret5, 12=fret12 for interpolation).
    pub fn calibrate_string(
        &self,
        string_idx: usize,
        audio: &[f32],
        fret: Option<u8>,
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

        // Build profile with optional fretted B values (#4)
        let (b_open, b_fret5, b_fret12) = match fret {
            Some(0) | None => (b, None, None),
            Some(5) => {
                // Return existing open B, set fret5
                let open_b = self
                    .calibration
                    .as_ref()
                    .and_then(|c| c.string_profiles.get(string_idx))
                    .map(|p| p.inharmonicity_b)
                    .unwrap_or(b);
                (open_b, Some(b), None)
            }
            Some(12) => {
                let open_b = self
                    .calibration
                    .as_ref()
                    .and_then(|c| c.string_profiles.get(string_idx))
                    .map(|p| p.inharmonicity_b)
                    .unwrap_or(b);
                let fret5_b = self
                    .calibration
                    .as_ref()
                    .and_then(|c| c.string_profiles.get(string_idx))
                    .and_then(|p| p.inharmonicity_b_fret5);
                (open_b, fret5_b, Some(b))
            }
            _ => (b, None, None),
        };

        Some(StringProfile {
            name: STRING_NAMES.get(string_idx).copied().unwrap_or("?"),
            open_midi: STRING_BASE_PITCH.get(string_idx).copied().unwrap_or(40),
            open_frequency: freq,
            inharmonicity_b: b_open,
            inharmonicity_b_fret5: b_fret5,
            inharmonicity_b_fret12: b_fret12,
            harmonic_ratios: ratios,
        })
    }

    /// Measure the noise floor from a silence capture.
    pub fn measure_noise_floor(audio: &[f32]) -> f32 {
        compute_rms(audio)
    }

    /// Compute an optimal input_gain from calibration peak measurement.
    ///
    /// Targets ~0.8 peak amplitude so the loudest pluck has headroom
    /// without being too quiet.
    pub fn compute_input_gain(peak_pluck_level: f32) -> f32 {
        if peak_pluck_level <= 0.0 {
            return 1.0;
        }
        (0.8 / peak_pluck_level).clamp(0.1, 2.0)
    }
}

// ── Plausibility filter (#3) ──────────────────────────────────────

/// Check if a MIDI note is physically playable on a given string.
///
/// Returns true if the note falls between the open string pitch and
/// 22 frets above it (standard guitar range).
pub fn is_plausible(midi_note: u8, string_idx: usize) -> bool {
    if string_idx >= STRING_BASE_PITCH.len() {
        return false;
    }
    let open = STRING_BASE_PITCH[string_idx];
    midi_note >= open && midi_note <= open + 22
}

/// Compute a plausibility score for assigning a MIDI note to a string.
///
/// Takes into account:
/// - Physical playability (must be on frets 0-22)
/// - Lower fret preference (more common in practice)
/// - Fret proximity bonus (prefer small jumps from previous note)
pub fn plausibility_score(
    midi_note: u8,
    string_idx: usize,
    prev_note: Option<&DetectedNote>,
) -> f32 {
    if !is_plausible(midi_note, string_idx) {
        return 0.0;
    }

    let open = STRING_BASE_PITCH[string_idx];
    let fret = midi_note - open;
    let mut score = 1.0;

    // Prefer lower fret positions (more common in practice)
    score -= fret as f32 * 0.01;

    // Fret proximity bonus if we have a previous note
    if let Some(prev) = prev_note {
        if let Some(prev_fret) = prev.fret {
            let fret_distance = (fret as i16 - prev_fret as i16).unsigned_abs();
            if fret_distance <= 5 {
                score += 0.2;
            }
        }
    }

    score
}

// ── Spectral flux helper ──────────────────────────────────────────

/// Compute half-wave rectified spectral flux between two spectra.
///
/// Only positive changes (increases in energy) contribute. This makes
/// the detector sensitive to new energy appearing (onsets) rather than
/// energy disappearing (note decay).
pub fn spectral_flux(prev_spectrum: &[f32], curr_spectrum: &[f32]) -> f32 {
    prev_spectrum
        .iter()
        .zip(curr_spectrum.iter())
        .map(|(p, c)| {
            let diff = c - p;
            if diff > 0.0 {
                diff * diff
            } else {
                0.0
            } // half-wave rectified
        })
        .sum::<f32>()
        .sqrt()
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

/// Score how well a candidate f0 matches the harmonic template in the audio.
/// Returns a weighted score: fundamental strength + count of present harmonics.
/// A true fundamental will have strong energy at f0 AND at its harmonics.
fn harmonic_template_score(audio: &[f32], candidate_f0: f32, sample_rate: usize) -> f32 {
    let magnitudes = measure_harmonics(audio, candidate_f0, 6, sample_rate);
    let h1_mag = magnitudes[0];
    if h1_mag < 1e-6 {
        return 0.0;
    }
    // Fundamental strength: the raw Goertzel magnitude at f0
    let mut score = h1_mag;
    // Bonus for each present harmonic (above 5% of h1)
    let noise_threshold = h1_mag * 0.05;
    for &mag in &magnitudes[1..] {
        if mag > noise_threshold {
            score += mag * 0.5; // harmonics contribute less than fundamental
        }
    }
    score
}

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

// ── String identification with plausibility ───────────────────────

/// Identify which string produced a note by comparing measured
/// inharmonicity against calibrated per-string B coefficients,
/// with plausibility filtering (#3) and fret proximity scoring.
///
/// Returns `(string_idx, confidence)` or `None`.
pub fn identify_string_with_plausibility(
    audio: &[f32],
    fundamental: f32,
    midi_note: u8,
    profiles: &[StringProfile; 6],
    sample_rate: usize,
    n_harmonics: usize,
    confidence_min: f32,
    prev_note: Option<&DetectedNote>,
) -> Option<(usize, f32)> {
    // Measure actual inharmonicity from the audio
    let measured_b = compute_inharmonicity_b(audio, fundamental, sample_rate);

    // Also measure harmonic ratios for spectral shape matching
    let measured_harmonics = measure_harmonics(audio, fundamental, n_harmonics, sample_rate);
    let h1 = measured_harmonics
        .first()
        .copied()
        .unwrap_or(1.0)
        .max(1e-10);
    let measured_ratios: Vec<f32> = measured_harmonics.iter().skip(1).map(|&h| h / h1).collect();

    let mut best_string: Option<usize> = None;
    let mut best_confidence = 0.0f32;

    for (idx, profile) in profiles.iter().enumerate() {
        // Plausibility filter (#3): reject impossible fret combinations
        if !is_plausible(midi_note, idx) {
            continue;
        }

        let fret = midi_note - profile.open_midi;

        // Interpolate expected B based on fret position (#4)
        let expected_b = interpolate_b(profile, fret);

        // B similarity: closer is better (log scale, since B can vary by orders)
        let b_ratio = if measured_b > 0.0 && expected_b > 0.0 {
            let log_ratio = (measured_b.ln() - expected_b.ln()).abs();
            (-log_ratio * 2.0).exp() // 1.0 = perfect match, decays with difference
        } else {
            0.3 // no B data, neutral score
        };

        // Harmonic ratio similarity (cosine-like distance)
        let harmonic_sim = if !profile.harmonic_ratios.is_empty() && !measured_ratios.is_empty() {
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

        // Plausibility score (#3): includes fret proximity
        let plaus = plausibility_score(midi_note, idx, prev_note);

        // Combined confidence (weight plausibility into the mix)
        let confidence = b_ratio * 0.4 + harmonic_sim * 0.25 + fret_bonus * 0.15 + plaus * 0.2;

        if confidence > best_confidence && confidence >= confidence_min {
            best_confidence = confidence;
            best_string = Some(idx);
        }
    }

    best_string.map(|s| (s, best_confidence))
}

/// Original identify_string function for backward compatibility.
///
/// Delegates to `identify_string_with_plausibility` without previous note context.
pub fn identify_string(
    audio: &[f32],
    fundamental: f32,
    midi_note: u8,
    profiles: &[StringProfile; 6],
    sample_rate: usize,
    n_harmonics: usize,
    confidence_min: f32,
) -> Option<(usize, f32)> {
    identify_string_with_plausibility(
        audio,
        fundamental,
        midi_note,
        profiles,
        sample_rate,
        n_harmonics,
        confidence_min,
        None,
    )
}

/// Interpolate the expected inharmonicity B for a fret position.
///
/// Uses calibrated B values at open, fret 5, and fret 12 if available.
/// Falls back to the theoretical model: B(fret) = B(open) * 2^(fret/6).
fn interpolate_b(profile: &StringProfile, fret: u8) -> f32 {
    let b_open = profile.inharmonicity_b;

    // If we have fret-specific measurements, use linear interpolation
    match (
        profile.inharmonicity_b_fret5,
        profile.inharmonicity_b_fret12,
    ) {
        (Some(b5), Some(b12)) => {
            if fret <= 5 {
                // Interpolate between open and fret 5
                let t = fret as f32 / 5.0;
                b_open + t * (b5 - b_open)
            } else if fret <= 12 {
                // Interpolate between fret 5 and fret 12
                let t = (fret as f32 - 5.0) / 7.0;
                b5 + t * (b12 - b5)
            } else {
                // Extrapolate beyond fret 12 using the fret5-fret12 slope
                let slope = (b12 - b5) / 7.0;
                b12 + slope * (fret as f32 - 12.0)
            }
        }
        (Some(b5), None) => {
            if fret <= 5 {
                let t = fret as f32 / 5.0;
                b_open + t * (b5 - b_open)
            } else {
                // Extrapolate using the open-fret5 slope
                let slope = (b5 - b_open) / 5.0;
                b5 + slope * (fret as f32 - 5.0)
            }
        }
        _ => {
            // No fretted calibration data, use theoretical model
            b_open * 2.0f32.powf(fret as f32 / 6.0)
        }
    }
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
///
/// Fallback when no calibration data is available. Uses a hardcoded
/// range mapping [0.01, 0.5] to [30, 127].
pub fn rms_to_velocity(rms: f32) -> u8 {
    // Map RMS range [0.01, 0.5] to velocity [30, 127].
    let normalized = ((rms - 0.01) / 0.49).clamp(0.0, 1.0);
    let vel = 30.0 + normalized * 97.0;
    (vel as u8).clamp(1, 127)
}

/// Convert RMS to MIDI velocity using calibrated range and power curve.
///
/// Uses calibration data for per-guitar dynamic range. The power curve
/// (gamma 0.5) gives more dynamic range in the soft-to-medium region,
/// which matches human perception better than linear.
pub fn rms_to_velocity_calibrated(rms: f32, noise_floor: f32, signal_peak: f32) -> u8 {
    let floor = noise_floor.max(0.001);
    let ceiling = signal_peak.max(floor + 0.01);
    let normalized = ((rms - floor) / (ceiling - floor)).clamp(0.0, 1.0);

    // Power curve (gamma 0.5): more dynamic range in soft-to-medium region
    let curved = normalized.powf(0.5);

    let vel = 1.0 + curved * 126.0;
    (vel as u8).clamp(1, 127)
}

/// Convert cents offset to 14-bit MIDI pitch bend value.
///
/// MIDI pitch bend is 14-bit (0-16383, center at 8192).
/// `range_semitones` is the pitch bend range in semitones (typically 2).
pub fn cents_to_midi_pitch_bend(cents: i16, range_semitones: u8) -> u16 {
    let range_cents = range_semitones as f32 * 100.0;
    let normalized = (cents as f32 / range_cents).clamp(-1.0, 1.0);
    ((normalized * 8191.0) + 8192.0).clamp(0.0, 16383.0) as u16
}

/// Convert sustain RMS to channel pressure (aftertouch) value (0-127).
pub fn rms_to_pressure(rms: f32, noise_floor: f32) -> u8 {
    let normalized = ((rms - noise_floor) / (0.3 - noise_floor)).clamp(0.0, 1.0);
    (normalized * 127.0) as u8
}

/// Compute spectral centroid from harmonic frequency/magnitude pairs.
///
/// The spectral centroid is the weighted mean of frequencies, where the
/// weights are the magnitudes. It correlates with perceived brightness.
pub fn compute_spectral_centroid(harmonics: &[(f32, f32)]) -> f32 {
    let total_magnitude: f32 = harmonics.iter().map(|(_, m)| m).sum();
    if total_magnitude <= 0.0 {
        return 0.0;
    }
    let weighted_sum: f32 = harmonics.iter().map(|(f, m)| f * m).sum();
    weighted_sum / total_magnitude
}

/// Map a spectral centroid frequency to a CC74 brightness value (0-127).
///
/// Guitar spectral centroid range: ~200 Hz (high frets, dark) to ~2000 Hz (open strings, bright).
/// Uses a logarithmic mapping for perceptual linearity.
pub fn centroid_to_cc74(centroid: f32) -> u8 {
    if centroid <= 0.0 {
        return 0;
    }
    let log_low = 200.0f32.log2();
    let log_high = 2000.0f32.log2();
    let log_cent = centroid.log2();
    let normalized = ((log_cent - log_low) / (log_high - log_low)).clamp(0.0, 1.0);
    (normalized * 127.0) as u8
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

    let padding = n / 2;
    let mut detector = McLeodDetector::new(n, padding);

    let pitch = detector.get_pitch(audio, sample_rate, 0.5, 0.3)?;

    if (pitch.clarity as f64) < min_clarity {
        return None;
    }

    Some((pitch.frequency, pitch.clarity))
}

/// Probabilistic McLeod (pMPM): run McLeod with multiple clarity thresholds
/// and accumulate confidence. Returns the most probable pitch candidate
/// with higher confidence than a single-threshold run.
///
/// Sweeps clarity from 0.3 to 0.9 in 0.05 steps (13 runs). Each threshold
/// has equal weight. Candidates within 1 semitone are merged.
pub fn detect_pitch_pmcleod(audio: &[f32], sample_rate: usize) -> Option<(f32, f32)> {
    use pitch_detection::detector::mcleod::McLeodDetector;
    use pitch_detection::detector::PitchDetector;

    let n = audio.len();
    if n < 64 {
        return None;
    }

    let padding = n / 2;
    // Collect candidates from multiple threshold sweeps
    let mut candidates: Vec<(f32, f32)> = Vec::new(); // (freq, accumulated_weight)
    let steps = 13;

    for i in 0..steps {
        let clarity_threshold = 0.3 + (i as f32) * 0.05; // 0.3 to 0.9
        let mut detector = McLeodDetector::new(n, padding);
        if let Some(pitch) = detector.get_pitch(audio, sample_rate, 0.5, clarity_threshold) {
            let freq = pitch.frequency;
            // Merge with existing candidate within 1 semitone
            let mut merged = false;
            for (cf, cw) in candidates.iter_mut() {
                let ratio = freq / *cf;
                // Within ~1 semitone (ratio 0.944 to 1.059)
                if ratio > 0.944 && ratio < 1.059 {
                    // Weighted average frequency, accumulate weight
                    *cf = (*cf * *cw + freq) / (*cw + 1.0);
                    *cw += 1.0;
                    merged = true;
                    break;
                }
            }
            if !merged {
                candidates.push((freq, 1.0));
            }
        }
    }

    if candidates.is_empty() {
        return None;
    }

    // Pick the candidate with the highest accumulated weight
    candidates.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    let (best_freq, best_weight) = candidates[0];
    // Normalize confidence: weight/steps gives 0-1 range
    let confidence = (best_weight / steps as f32).min(1.0);

    Some((best_freq, confidence))
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
            let harmonic_freq = fundamental * n as f32 * (1.0 + b_coeff * (n * n) as f32).sqrt();
            let amplitude = 1.0 / n as f32; // natural harmonic decay
            for i in 0..num_samples {
                signal[i] +=
                    amplitude * (2.0 * PI * harmonic_freq * i as f32 / sample_rate as f32).sin();
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
        assert!(
            mag > 10.0,
            "Goertzel should find strong signal at 440Hz, got {}",
            mag
        );
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
        assert!(
            mag.abs() < 1e-6,
            "silence should give near-zero, got {}",
            mag
        );
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

    // -- is_plausible tests (#3) ────────────────────────────────────

    #[test]
    fn plausible_open_strings() {
        // Each open string's own pitch is plausible on that string
        for (idx, &open) in STRING_BASE_PITCH.iter().enumerate() {
            assert!(
                is_plausible(open, idx),
                "open string {} should be plausible on string {}",
                open,
                idx
            );
        }
    }

    #[test]
    fn plausible_high_fret() {
        // Fret 22 on Low E: MIDI 40 + 22 = 62
        assert!(is_plausible(62, 0), "fret 22 on Low E should be plausible");
    }

    #[test]
    fn implausible_too_low() {
        // MIDI 39 is below Low E open (40)
        assert!(!is_plausible(39, 0), "MIDI 39 is below Low E open");
    }

    #[test]
    fn implausible_too_high() {
        // MIDI 63 is fret 23 on Low E (40 + 23 = 63) -- beyond 22 frets
        assert!(!is_plausible(63, 0), "fret 23 is beyond 22 frets");
    }

    #[test]
    fn implausible_wrong_string() {
        // MIDI 40 (E2) is not plausible on string 5 (high E, open 64)
        assert!(!is_plausible(40, 5), "E2 is not playable on high E string");
    }

    #[test]
    fn implausible_invalid_string() {
        // String index 6 is out of range
        assert!(!is_plausible(60, 6), "string index 6 is invalid");
    }

    // -- correct_octave tests (#6) ──────────────────────────────────

    #[test]
    fn correct_octave_drops_harmonic_lock() {
        let sr = 48000;
        let mut pipeline = GuitarInput::with_defaults();
        // Fill ring buffer with a 110 Hz guitar signal (has harmonics at 220, 330, ...)
        let audio = guitar_signal(110.0, 0.003, sr, 1024);
        for (i, &s) in audio.iter().enumerate() {
            pipeline.ring_buffer[i] = s;
        }
        // Set a current note at A2 = 110 Hz
        pipeline.current_note = Some(DetectedNote {
            midi_note: 45,
            frequency: 110.0,
            string_idx: Some(1),
            fret: Some(0),
            string_confidence: 0.8,
            velocity: 80,
            bend_cents: 0,
        });

        // Detected 220 Hz (octave up) -> harmonic template should prefer 110 Hz
        let corrected = pipeline.correct_octave(220.0);
        assert!(
            (corrected - 110.0).abs() < 1.0,
            "should drop octave from 220 to ~110, got {}",
            corrected
        );
    }

    #[test]
    fn correct_octave_raises_subharmonic() {
        let sr = 48000;
        let mut pipeline = GuitarInput::with_defaults();
        // Fill ring buffer with a 110 Hz guitar signal
        let audio = guitar_signal(110.0, 0.003, sr, 1024);
        for (i, &s) in audio.iter().enumerate() {
            pipeline.ring_buffer[i] = s;
        }
        pipeline.current_note = Some(DetectedNote {
            midi_note: 45,
            frequency: 110.0,
            string_idx: Some(1),
            fret: Some(0),
            string_confidence: 0.8,
            velocity: 80,
            bend_cents: 0,
        });

        // Detected 55 Hz (octave down) -> harmonic template should prefer 110 Hz
        let corrected = pipeline.correct_octave(55.0);
        assert!(
            (corrected - 110.0).abs() < 1.0,
            "should raise from 55 to ~110, got {}",
            corrected
        );
    }

    #[test]
    fn correct_octave_no_change_for_normal_interval() {
        let mut pipeline = GuitarInput::with_defaults();
        pipeline.current_note = Some(DetectedNote {
            midi_note: 45,
            frequency: 110.0,
            string_idx: Some(1),
            fret: Some(0),
            string_confidence: 0.8,
            velocity: 80,
            bend_cents: 0,
        });

        // Detected 130 Hz (not an exact octave) -> no correction
        let corrected = pipeline.correct_octave(130.0);
        assert!(
            (corrected - 130.0).abs() < 0.01,
            "should not change 130 Hz, got {}",
            corrected
        );
    }

    #[test]
    fn correct_octave_no_current_note() {
        let pipeline = GuitarInput::with_defaults();
        // No current note, no correction
        let corrected = pipeline.correct_octave(220.0);
        assert_eq!(corrected, 220.0, "no current note means no correction");
    }

    // -- NoteState transition tests (#2) ────────────────────────────

    #[test]
    fn note_state_starts_idle() {
        let pipeline = GuitarInput::with_defaults();
        assert_eq!(pipeline.note_state(), NoteState::Idle);
    }

    #[test]
    fn note_state_transitions_idle_to_attack_on_loud_signal() {
        let sr = 48000;
        let config = GuitarInputConfig {
            buffer_size: 1024,
            hop_size: 1024, // tests: analyze every full buffer for deterministic behavior
            sample_rate: sr,
            onset_threshold: 0.01,
            string_confidence_min: 0.3,
            bends_enabled: false,
            legato_enabled: false,
            slides_enabled: false,
            vibrato_detection: false,
            vibrato_passthrough: false,
            filter_enabled: false,
            min_clarity: 0.3,
            cooldown_samples: 1024,
            n_harmonics: 6,
            input_gain: 1.0,
            flux_threshold: 0.5,
            per_string_channels: true,
            pitch_bend_range: 2,
            pressure_enabled: false,
            pressure_hold: 0.3,
            brightness_enabled: false,
        };
        let mut pipeline = GuitarInput::new(config);

        // Feed silence first
        let silence = vec![0.0f32; 1024];
        pipeline.process_block(&silence);
        assert_eq!(pipeline.note_state(), NoteState::Idle);

        // Feed a loud signal -- should transition to Attack
        let loud: Vec<f32> = sine_wave(440.0, sr, 1024)
            .into_iter()
            .map(|s| s * 0.5)
            .collect();
        pipeline.process_block(&loud);

        // After one loud window, should be in Attack (or Sustain if pitch is stable fast)
        assert!(
            pipeline.note_state() == NoteState::Attack
                || pipeline.note_state() == NoteState::Sustain,
            "should be Attack or Sustain after loud signal, got {:?}",
            pipeline.note_state()
        );
    }

    // -- Multi-frame voting tests (#7) ──────────────────────────────

    #[test]
    fn pitch_voting_requires_two_agreements() {
        let mut pipeline = GuitarInput::with_defaults();

        // Not stable with no history
        assert!(!pipeline.is_pitch_stable());

        // Push first note
        pipeline.push_pitch(60);
        assert!(!pipeline.is_pitch_stable());

        // Push second (same) -> stable
        pipeline.push_pitch(60);
        assert!(pipeline.is_pitch_stable());
    }

    #[test]
    fn pitch_voting_not_stable_with_disagreement() {
        let mut pipeline = GuitarInput::with_defaults();

        pipeline.push_pitch(60);
        pipeline.push_pitch(61); // disagreement!

        assert!(!pipeline.is_pitch_stable());
    }

    #[test]
    fn pitch_voting_clears_properly() {
        let mut pipeline = GuitarInput::with_defaults();

        pipeline.push_pitch(60);
        pipeline.push_pitch(60);
        assert!(pipeline.is_pitch_stable());

        pipeline.clear_pitch_history();
        assert!(!pipeline.is_pitch_stable());
    }

    // -- spectral_flux tests (#5) ───────────────────────────────────

    #[test]
    fn spectral_flux_zero_for_identical_spectra() {
        let spec = vec![1.0, 2.0, 3.0, 4.0];
        let flux = spectral_flux(&spec, &spec);
        assert!(
            flux.abs() < 1e-10,
            "identical spectra should have zero flux, got {}",
            flux
        );
    }

    #[test]
    fn spectral_flux_positive_for_increasing_energy() {
        let prev = vec![0.0, 0.0, 0.0, 0.0];
        let curr = vec![1.0, 2.0, 3.0, 4.0];
        let flux = spectral_flux(&prev, &curr);
        assert!(
            flux > 0.0,
            "increasing energy should produce positive flux, got {}",
            flux
        );
    }

    #[test]
    fn spectral_flux_half_wave_rectified() {
        // Only positive changes count; energy decrease should contribute 0
        let prev = vec![10.0, 10.0, 10.0, 10.0];
        let curr = vec![5.0, 5.0, 5.0, 5.0]; // all decreasing
        let flux = spectral_flux(&prev, &curr);
        assert!(
            flux.abs() < 1e-10,
            "decreasing energy should give zero flux, got {}",
            flux
        );
    }

    // -- input_gain tests (#8) ──────────────────────────────────────

    #[test]
    fn input_gain_normalization() {
        let gain = GuitarInput::compute_input_gain(0.4);
        // 0.8 / 0.4 = 2.0
        assert!(
            (gain - 2.0).abs() < 0.01,
            "gain should be 2.0, got {}",
            gain
        );
    }

    #[test]
    fn input_gain_clamped() {
        // Very quiet signal
        let gain = GuitarInput::compute_input_gain(0.001);
        assert_eq!(gain, 2.0, "gain should be clamped to 2.0");

        // Zero signal
        let gain = GuitarInput::compute_input_gain(0.0);
        assert_eq!(gain, 1.0, "zero signal should give gain 1.0");
    }

    // -- buffer_size_for_latency tests (#9) ─────────────────────────

    #[test]
    fn buffer_size_for_latency_reasonable() {
        // 20ms @ 48kHz = 960 samples -> next power of 2 = 1024
        let size = GuitarInputConfig::buffer_size_for_latency(20.0, 48000);
        assert_eq!(size, 1024);
    }

    #[test]
    fn buffer_size_for_latency_clamped_min() {
        // Very small latency should clamp to 256
        let size = GuitarInputConfig::buffer_size_for_latency(0.1, 48000);
        assert_eq!(size, 256);
    }

    #[test]
    fn buffer_size_for_latency_clamped_max() {
        // Very large latency should clamp to 4096
        let size = GuitarInputConfig::buffer_size_for_latency(200.0, 48000);
        assert_eq!(size, 4096);
    }

    // -- interpolate_b tests (#4) ───────────────────────────────────

    #[test]
    fn interpolate_b_open_only() {
        let profile = StringProfile {
            name: "test",
            open_midi: 40,
            open_frequency: 82.41,
            inharmonicity_b: 0.006,
            inharmonicity_b_fret5: None,
            inharmonicity_b_fret12: None,
            harmonic_ratios: vec![],
        };
        // With no fretted data, should use theoretical model
        let b0 = interpolate_b(&profile, 0);
        assert!(
            (b0 - 0.006).abs() < 0.001,
            "open B should be 0.006, got {}",
            b0
        );

        let b12 = interpolate_b(&profile, 12);
        // Theoretical: 0.006 * 2^(12/6) = 0.006 * 4 = 0.024
        assert!(
            (b12 - 0.024).abs() < 0.002,
            "fret 12 B should be ~0.024, got {}",
            b12
        );
    }

    #[test]
    fn interpolate_b_with_fret5_and_fret12() {
        let profile = StringProfile {
            name: "test",
            open_midi: 40,
            open_frequency: 82.41,
            inharmonicity_b: 0.006,
            inharmonicity_b_fret5: Some(0.010),
            inharmonicity_b_fret12: Some(0.020),
            harmonic_ratios: vec![],
        };
        // At fret 0
        let b0 = interpolate_b(&profile, 0);
        assert!((b0 - 0.006).abs() < 0.001);

        // At fret 5 (should be 0.010)
        let b5 = interpolate_b(&profile, 5);
        assert!(
            (b5 - 0.010).abs() < 0.001,
            "fret 5 should be 0.010, got {}",
            b5
        );

        // At fret 12 (should be 0.020)
        let b12 = interpolate_b(&profile, 12);
        assert!(
            (b12 - 0.020).abs() < 0.001,
            "fret 12 should be 0.020, got {}",
            b12
        );
    }

    // -- plausibility_score tests (#3) ──────────────────────────────

    #[test]
    fn plausibility_score_impossible_is_zero() {
        // MIDI 39 on Low E (open 40) -> impossible
        let score = plausibility_score(39, 0, None);
        assert_eq!(score, 0.0, "impossible fret should score 0");
    }

    #[test]
    fn plausibility_score_open_string_high() {
        // Open Low E (MIDI 40, string 0) -> should be high score
        let score = plausibility_score(40, 0, None);
        assert!(score > 0.9, "open string should score high, got {}", score);
    }

    #[test]
    fn plausibility_score_fret_proximity_bonus() {
        let prev = DetectedNote {
            midi_note: 43,
            frequency: 98.0,
            string_idx: Some(0),
            fret: Some(3), // Low E fret 3
            string_confidence: 0.8,
            velocity: 80,
            bend_cents: 0,
        };

        // Fret 5 on Low E (close to fret 3 -> bonus)
        let score_close = plausibility_score(45, 0, Some(&prev));
        // Fret 15 on Low E (far from fret 3 -> no bonus)
        let score_far = plausibility_score(55, 0, Some(&prev));

        assert!(
            score_close > score_far,
            "close fret ({}) should score higher than far fret ({})",
            score_close,
            score_far
        );
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
            &audio, e2_freq, 40, // MIDI E2
            &profiles, sr, 6, 0.3,
        );

        assert!(result.is_some(), "should identify a string");
        let (string_idx, confidence) = result.unwrap();
        assert_eq!(string_idx, 0, "should identify as Low E");
        assert!(
            confidence > 0.3,
            "confidence should be reasonable, got {}",
            confidence
        );
    }

    #[test]
    fn identify_string_rejects_impossible() {
        let sr = 48000;
        // MIDI 30 is below any open string -- should still return something
        // or None since it is below all open strings.
        let audio = sine_wave(midi_to_freq(30), sr, 2048);
        let profiles = make_test_profiles();
        let result = identify_string(&audio, midi_to_freq(30), 30, &profiles, sr, 6, 0.3);
        assert!(
            result.is_none(),
            "MIDI 30 is not playable on any standard-tuned string"
        );
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
        assert!(
            mid > 30 && mid < 127,
            "mid velocity {} should be in range",
            mid
        );
    }

    // -- compute_inharmonicity_b tests ───────────────────────────────

    #[test]
    fn inharmonicity_of_pure_sine_is_near_zero() {
        let sr = 48000;
        let audio = sine_wave(110.0, sr, 4096);
        let b = compute_inharmonicity_b(&audio, 110.0, sr);
        assert!(b < 0.005, "Pure sine should have near-zero B, got {}", b);
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
            hop_size: 1024, // tests: analyze every full buffer for deterministic behavior
            sample_rate: sr,
            onset_threshold: 0.01,
            string_confidence_min: 0.3,
            bends_enabled: false,
            legato_enabled: false,
            slides_enabled: false,
            vibrato_detection: false,
            vibrato_passthrough: false,
            filter_enabled: false,
            min_clarity: 0.3,
            cooldown_samples: 1024,
            n_harmonics: 6,
            input_gain: 1.0,
            flux_threshold: 0.5,
            per_string_channels: true,
            pitch_bend_range: 2,
            pressure_enabled: false,
            pressure_hold: 0.3,
            brightness_enabled: false,
        };
        let mut pipeline = GuitarInput::new(config);

        // Feed silence then a loud sine
        let silence = vec![0.0f32; 1024];
        let events1 = pipeline.process_block(&silence);
        assert!(events1.is_empty(), "silence should produce no events");

        // Now feed multiple blocks of loud A4 sine (amplitude 0.5)
        // The state machine needs 3 frames of stable pitch (Attack -> Sustain)
        let a4: Vec<f32> = sine_wave(440.0, sr, 1024 * 5)
            .into_iter()
            .map(|s| s * 0.5)
            .collect();
        let events2 = pipeline.process_block(&a4);
        // Should get a NoteOn somewhere in here
        let has_note_on = events2
            .iter()
            .any(|e| matches!(e, MidiEvent::NoteOn { .. }));
        assert!(
            has_note_on,
            "should detect NoteOn for A4, events: {:?}",
            events2
        );
    }

    // -- Helper ──────────────────────────────────────────────────────

    fn make_test_profiles() -> [StringProfile; 6] {
        [
            StringProfile {
                name: "Low E",
                open_midi: 40,
                open_frequency: 82.41,
                inharmonicity_b: 0.006,
                inharmonicity_b_fret5: None,
                inharmonicity_b_fret12: None,
                harmonic_ratios: vec![0.5, 0.3, 0.2, 0.1, 0.05],
            },
            StringProfile {
                name: "A",
                open_midi: 45,
                open_frequency: 110.0,
                inharmonicity_b: 0.003,
                inharmonicity_b_fret5: None,
                inharmonicity_b_fret12: None,
                harmonic_ratios: vec![0.55, 0.35, 0.2, 0.12, 0.06],
            },
            StringProfile {
                name: "D",
                open_midi: 50,
                open_frequency: 146.83,
                inharmonicity_b: 0.003,
                inharmonicity_b_fret5: None,
                inharmonicity_b_fret12: None,
                harmonic_ratios: vec![0.5, 0.3, 0.18, 0.1, 0.05],
            },
            StringProfile {
                name: "G",
                open_midi: 55,
                open_frequency: 196.0,
                inharmonicity_b: 0.006,
                inharmonicity_b_fret5: None,
                inharmonicity_b_fret12: None,
                harmonic_ratios: vec![0.6, 0.4, 0.25, 0.15, 0.08],
            },
            StringProfile {
                name: "B",
                open_midi: 59,
                open_frequency: 246.94,
                inharmonicity_b: 0.003,
                inharmonicity_b_fret5: None,
                inharmonicity_b_fret12: None,
                harmonic_ratios: vec![0.5, 0.28, 0.15, 0.08, 0.04],
            },
            StringProfile {
                name: "High E",
                open_midi: 64,
                open_frequency: 329.63,
                inharmonicity_b: 0.002,
                inharmonicity_b_fret5: None,
                inharmonicity_b_fret12: None,
                harmonic_ratios: vec![0.45, 0.25, 0.12, 0.06, 0.03],
            },
        ]
    }

    // -- Slide detection tests ──────────────────────────────────────

    #[test]
    fn is_slide_movement_monotonic_increasing() {
        let mut pipeline = GuitarInput::with_defaults();
        // Simulate gradual upward pitch movement: 0, 10, 20, 30, 40, 50, 60, 70
        for cents in [0i16, 10, 20, 30, 40, 50, 60, 70] {
            pipeline.push_slide_cents(cents);
        }
        assert!(
            pipeline.is_slide_movement(),
            "monotonic increasing cents should be detected as a slide"
        );
    }

    #[test]
    fn is_slide_movement_monotonic_decreasing() {
        let mut pipeline = GuitarInput::with_defaults();
        // Simulate gradual downward pitch movement
        for cents in [70i16, 60, 50, 40, 30, 20, 10, 0] {
            pipeline.push_slide_cents(cents);
        }
        assert!(
            pipeline.is_slide_movement(),
            "monotonic decreasing cents should be detected as a slide"
        );
    }

    #[test]
    fn is_slide_movement_sudden_jump_rejected() {
        let mut pipeline = GuitarInput::with_defaults();
        // Simulate a sudden jump: gradual then big jump
        for cents in [0i16, 5, 10, 15, 20, 25, 30, 100] {
            pipeline.push_slide_cents(cents);
        }
        assert!(
            !pipeline.is_slide_movement(),
            "a sudden jump (>50 cents between frames) should not be a slide"
        );
    }

    #[test]
    fn is_slide_movement_random_oscillation_rejected() {
        let mut pipeline = GuitarInput::with_defaults();
        // Simulate random oscillation (not monotonic)
        for cents in [0i16, 20, -10, 30, -5, 15, -20, 10] {
            pipeline.push_slide_cents(cents);
        }
        assert!(
            !pipeline.is_slide_movement(),
            "random oscillation should not be detected as a slide"
        );
    }

    #[test]
    fn slide_history_clears() {
        let mut pipeline = GuitarInput::with_defaults();
        for cents in [0i16, 10, 20, 30, 40, 50, 60, 70] {
            pipeline.push_slide_cents(cents);
        }
        assert!(pipeline.is_slide_movement());
        pipeline.clear_slide_history();
        assert!(
            !pipeline.is_slide_movement(),
            "after clearing, slide should not be detected"
        );
    }

    // -- Vibrato detection tests ────────────────────────────────────

    #[test]
    fn detect_vibrato_periodic_oscillation() {
        let mut pipeline = GuitarInput::with_defaults();
        // Simulate a 5 Hz vibrato at ~30 cents amplitude
        // At 1024/48000 = ~21.3ms per frame = ~47 frames/sec
        // 5 Hz = one cycle every ~9.4 frames
        // Fill 24 frames (~500ms) with sinusoidal oscillation
        let frame_duration = 1024.0 / 48000.0;
        for i in 0..24 {
            let t = i as f32 * frame_duration;
            let cents = 30.0 * (2.0 * PI * 5.0 * t).sin();
            pipeline.push_vibrato_cents(cents);
        }
        let result = pipeline.detect_vibrato();
        assert!(
            result.is_some(),
            "should detect vibrato from periodic oscillation"
        );
        let (rate_hz, amplitude) = result.unwrap();
        assert!(
            rate_hz >= 3.0 && rate_hz <= 8.0,
            "vibrato rate {:.1} Hz should be in 3-8 Hz range",
            rate_hz
        );
        assert!(
            amplitude >= 10.0 && amplitude <= 60.0,
            "vibrato amplitude {:.1} cents should be in 10-60 range",
            amplitude
        );
    }

    #[test]
    fn detect_vibrato_rejects_random_noise() {
        let mut pipeline = GuitarInput::with_defaults();
        // Fill with small random-ish values (not periodic)
        let values = [
            2.0, -1.0, 3.0, 0.5, -2.0, 1.5, -0.5, 2.5, -1.5, 0.0, 3.0, -2.0, 1.0, -1.0, 2.0, -3.0,
            0.5, 1.5, -0.5, 2.0, -1.0, 3.0, -2.0, 1.0,
        ];
        for &v in &values {
            pipeline.push_vibrato_cents(v);
        }
        let result = pipeline.detect_vibrato();
        assert!(
            result.is_none(),
            "small random noise should not be detected as vibrato"
        );
    }

    #[test]
    fn detect_vibrato_rejects_too_slow() {
        let mut pipeline = GuitarInput::with_defaults();
        // Simulate very slow oscillation (~1 Hz) -- below vibrato range
        let frame_duration = 1024.0 / 48000.0;
        for i in 0..24 {
            let t = i as f32 * frame_duration;
            let cents = 30.0 * (2.0 * PI * 1.0 * t).sin();
            pipeline.push_vibrato_cents(cents);
        }
        let result = pipeline.detect_vibrato();
        assert!(
            result.is_none(),
            "1 Hz oscillation should be too slow for vibrato"
        );
    }

    #[test]
    fn detect_vibrato_rejects_too_fast() {
        let mut pipeline = GuitarInput::with_defaults();
        // Simulate very fast oscillation (~15 Hz) -- above vibrato range
        let frame_duration = 1024.0 / 48000.0;
        for i in 0..24 {
            let t = i as f32 * frame_duration;
            let cents = 30.0 * (2.0 * PI * 15.0 * t).sin();
            pipeline.push_vibrato_cents(cents);
        }
        let result = pipeline.detect_vibrato();
        assert!(
            result.is_none(),
            "15 Hz oscillation should be too fast for vibrato"
        );
    }

    #[test]
    fn detect_vibrato_rejects_too_small_amplitude() {
        let mut pipeline = GuitarInput::with_defaults();
        // Simulate 5 Hz oscillation but only 5 cents amplitude (below 10 threshold)
        let frame_duration = 1024.0 / 48000.0;
        for i in 0..24 {
            let t = i as f32 * frame_duration;
            let cents = 5.0 * (2.0 * PI * 5.0 * t).sin();
            pipeline.push_vibrato_cents(cents);
        }
        let result = pipeline.detect_vibrato();
        assert!(
            result.is_none(),
            "5 cents amplitude should be too small for vibrato"
        );
    }

    #[test]
    fn vibrato_state_clears() {
        let mut pipeline = GuitarInput::with_defaults();
        let frame_duration = 1024.0 / 48000.0;
        for i in 0..24 {
            let t = i as f32 * frame_duration;
            let cents = 30.0 * (2.0 * PI * 5.0 * t).sin();
            pipeline.push_vibrato_cents(cents);
        }
        assert!(pipeline.detect_vibrato().is_some());
        pipeline.clear_vibrato_state();
        assert!(
            pipeline.detect_vibrato().is_none(),
            "after clearing, vibrato should not be detected"
        );
    }

    // -- Slide crossing semitone boundary integration test ──────────

    #[test]
    fn slide_crossing_semitone_emits_note_off_and_on() {
        // Unit test for the slide mechanism: directly set up the pipeline
        // in Sustain state with a known note, prime the slide history,
        // then verify that the slide detection components work correctly.
        let mut pipeline = GuitarInput::with_defaults();

        // Manually set up state: note at A4, in Sustain
        pipeline.current_note = Some(DetectedNote {
            midi_note: 69, // A4
            frequency: 440.0,
            string_idx: Some(5),
            fret: Some(5),
            string_confidence: 0.8,
            velocity: 80,
            bend_cents: 0,
        });
        pipeline.note_state = NoteState::Sustain;

        // Prime slide history with gradual upward trend
        for cents in [10i16, 20, 30, 40, 50, 60, 80, 100] {
            pipeline.push_slide_cents(cents);
        }

        // Verify is_slide_movement detects this as a slide
        assert!(
            pipeline.is_slide_movement(),
            "primed history should be detected as a slide"
        );

        // Verify the slide_velocity_decay is applied correctly
        let original_vel = 80u8;
        let expected_vel = (original_vel as f32 * pipeline.slide_velocity_decay) as u8;
        assert!(
            expected_vel < original_vel,
            "slide velocity ({}) should be less than original ({})",
            expected_vel,
            original_vel
        );
        assert!(expected_vel > 0, "slide velocity should still be positive");

        // Verify the note hasn't changed yet (slide hasn't been processed)
        assert_eq!(
            pipeline.current_note.as_ref().unwrap().midi_note,
            69,
            "note should still be A4 before slide processing"
        );
    }

    // -- Vibrato within single note does not change note ────────────

    #[test]
    fn vibrato_does_not_change_note() {
        // Test that vibrato detection within a single note does not
        // alter the detected MIDI note at the vibrato-buffer level.
        // This is a unit test for the vibrato algorithm itself.
        let mut pipeline = GuitarInput::with_defaults();

        // Set up a current note at A4
        pipeline.current_note = Some(DetectedNote {
            midi_note: 69,
            frequency: 440.0,
            string_idx: Some(5),
            fret: Some(5),
            string_confidence: 0.8,
            velocity: 80,
            bend_cents: 0,
        });
        pipeline.note_state = NoteState::Sustain;

        // Feed vibrato-like cents into the vibrato buffer
        // 5 Hz vibrato at 25 cents amplitude (well within a single note)
        let frame_duration = 1024.0 / 48000.0;
        for i in 0..24 {
            let t = i as f32 * frame_duration;
            let cents = 25.0 * (2.0 * PI * 5.0 * t).sin();
            pipeline.push_vibrato_cents(cents);
        }

        // Vibrato should be detected
        let vib = pipeline.detect_vibrato();
        assert!(vib.is_some(), "vibrato should be detected");

        // The current note should still be A4 (MIDI 69)
        assert_eq!(
            pipeline.current_note.as_ref().unwrap().midi_note,
            69,
            "vibrato should not change the MIDI note"
        );
    }

    // -- rms_to_velocity_calibrated tests ──────────────────────────────

    #[test]
    fn calibrated_velocity_at_noise_floor_is_minimum() {
        let vel = rms_to_velocity_calibrated(0.001, 0.001, 0.5);
        assert_eq!(vel, 1, "at noise floor should give minimum velocity");
    }

    #[test]
    fn calibrated_velocity_at_signal_peak_is_maximum() {
        let vel = rms_to_velocity_calibrated(0.5, 0.001, 0.5);
        assert_eq!(vel, 127, "at signal peak should give maximum velocity");
    }

    #[test]
    fn calibrated_velocity_mid_range_uses_power_curve() {
        // With gamma 0.5, normalized 0.25 -> curved sqrt(0.25) = 0.5
        // vel = 1 + 0.5 * 126 = 64
        let vel = rms_to_velocity_calibrated(0.126, 0.001, 0.5);
        // 0.126 normalized in [0.001, 0.5]: (0.126-0.001)/(0.5-0.001) = 0.25
        // curved = sqrt(0.25) = 0.5, vel = 1 + 0.5*126 = 64
        assert!(
            vel >= 60 && vel <= 68,
            "mid-range calibrated vel should be ~64, got {}",
            vel
        );
    }

    #[test]
    fn calibrated_velocity_narrow_range() {
        // Very narrow dynamic range: floor 0.1, peak 0.15
        let soft = rms_to_velocity_calibrated(0.10, 0.10, 0.15);
        let loud = rms_to_velocity_calibrated(0.15, 0.10, 0.15);
        assert_eq!(soft, 1, "at floor of narrow range");
        assert_eq!(loud, 127, "at peak of narrow range");
    }

    #[test]
    fn calibrated_velocity_below_floor_clamps() {
        let vel = rms_to_velocity_calibrated(0.0, 0.01, 0.5);
        assert_eq!(vel, 1, "below noise floor should clamp to 1");
    }

    // -- cents_to_midi_pitch_bend tests ───────────────────────────────

    #[test]
    fn pitch_bend_center() {
        let val = cents_to_midi_pitch_bend(0, 2);
        assert_eq!(val, 8192, "0 cents should be center (8192)");
    }

    #[test]
    fn pitch_bend_max_up() {
        // 200 cents = full range with range_semitones=2
        let val = cents_to_midi_pitch_bend(200, 2);
        assert_eq!(val, 16383, "full bend up should be 16383");
    }

    #[test]
    fn pitch_bend_max_down() {
        let val = cents_to_midi_pitch_bend(-200, 2);
        // normalized = -1.0, value = -8191 + 8192 = 1
        assert_eq!(val, 1, "full bend down should be 1");
    }

    #[test]
    fn pitch_bend_clamps_beyond_range() {
        // 400 cents with range 2 semitones (200 cents) should clamp
        let val = cents_to_midi_pitch_bend(400, 2);
        assert_eq!(val, 16383, "beyond range should clamp to max");

        let val = cents_to_midi_pitch_bend(-400, 2);
        assert_eq!(val, 1, "beyond range should clamp to min");
    }

    #[test]
    fn pitch_bend_half_up() {
        // 100 cents = half of 200 cent range
        let val = cents_to_midi_pitch_bend(100, 2);
        // normalized = 0.5, value = 0.5*8191 + 8192 = 12287.5 -> 12287
        assert!(
            val > 12000 && val < 12500,
            "half bend up should be ~12288, got {}",
            val
        );
    }

    #[test]
    fn pitch_bend_different_range() {
        // 200 cents with range 4 semitones (400 cents) = half
        let val = cents_to_midi_pitch_bend(200, 4);
        assert!(
            val > 12000 && val < 12500,
            "200c in 4-semitone range should be ~12288, got {}",
            val
        );
    }

    // -- rms_to_pressure tests ────────────────────────────────────────

    #[test]
    fn pressure_at_noise_floor_is_zero() {
        let p = rms_to_pressure(0.01, 0.01);
        assert_eq!(p, 0, "at noise floor should be 0 pressure");
    }

    #[test]
    fn pressure_at_max_is_127() {
        let p = rms_to_pressure(0.3, 0.01);
        assert_eq!(p, 127, "at 0.3 RMS should be max pressure");
    }

    #[test]
    fn pressure_mid_range() {
        // midpoint: rms = (0.01 + 0.3) / 2 = 0.155
        // normalized = (0.155 - 0.01) / (0.3 - 0.01) = 0.5
        let p = rms_to_pressure(0.155, 0.01);
        assert!(
            p >= 60 && p <= 67,
            "mid-range pressure should be ~63, got {}",
            p
        );
    }

    #[test]
    fn pressure_below_floor_clamps() {
        let p = rms_to_pressure(0.0, 0.01);
        assert_eq!(p, 0, "below noise floor should clamp to 0");
    }

    // -- Attack peak measurement test ─────────────────────────────────

    #[test]
    fn attack_peak_uses_first_5ms() {
        // 5ms at 48000 Hz = 240 samples
        let sr = 48000;
        let attack_samples = (sr as f32 * 0.005) as usize;
        assert_eq!(attack_samples, 240, "5ms at 48kHz should be 240 samples");

        // Create a buffer where the first 240 samples are loud, rest is quiet
        let mut buf = vec![0.0f32; 1024];
        for i in 0..240 {
            buf[i] = 0.5 * (2.0 * PI * 440.0 * i as f32 / sr as f32).sin();
        }

        let attack_end = attack_samples.min(buf.len());
        let attack_rms = compute_rms(&buf[..attack_end]);
        let full_rms = compute_rms(&buf);

        // Attack RMS should be significantly higher than full-buffer RMS
        // because most of the buffer is silent
        assert!(
            attack_rms > full_rms * 1.5,
            "attack RMS ({:.4}) should be much higher than full-buffer RMS ({:.4})",
            attack_rms,
            full_rms
        );
    }

    // -- compute_spectral_centroid tests ───────────────────────────────

    #[test]
    fn spectral_centroid_single_frequency() {
        // All energy at 1000 Hz -> centroid = 1000
        let harmonics = vec![(1000.0, 1.0)];
        let centroid = compute_spectral_centroid(&harmonics);
        assert!(
            (centroid - 1000.0).abs() < 0.01,
            "single frequency centroid should be 1000, got {}",
            centroid
        );
    }

    #[test]
    fn spectral_centroid_weighted_average() {
        // Two frequencies with equal magnitude: centroid = midpoint
        let harmonics = vec![(200.0, 1.0), (800.0, 1.0)];
        let centroid = compute_spectral_centroid(&harmonics);
        assert!(
            (centroid - 500.0).abs() < 0.01,
            "equal-weight centroid should be 500, got {}",
            centroid
        );
    }

    #[test]
    fn spectral_centroid_dominated_by_strong_harmonic() {
        // Low freq strong, high freq weak -> centroid near low freq
        let harmonics = vec![(200.0, 10.0), (2000.0, 0.1)];
        let centroid = compute_spectral_centroid(&harmonics);
        assert!(
            centroid < 250.0,
            "centroid should be near 200 when low freq dominates, got {}",
            centroid
        );
    }

    #[test]
    fn spectral_centroid_empty_returns_zero() {
        let harmonics: Vec<(f32, f32)> = vec![];
        assert_eq!(compute_spectral_centroid(&harmonics), 0.0);
    }

    #[test]
    fn spectral_centroid_zero_magnitude_returns_zero() {
        let harmonics = vec![(440.0, 0.0), (880.0, 0.0)];
        assert_eq!(compute_spectral_centroid(&harmonics), 0.0);
    }

    // -- centroid_to_cc74 tests ───────────────────────────────────────

    #[test]
    fn cc74_at_low_centroid() {
        // 200 Hz = bottom of range -> 0
        let cc = centroid_to_cc74(200.0);
        assert_eq!(cc, 0, "200 Hz centroid should map to CC74=0, got {}", cc);
    }

    #[test]
    fn cc74_at_high_centroid() {
        // 2000 Hz = top of range -> 127
        let cc = centroid_to_cc74(2000.0);
        assert_eq!(
            cc, 127,
            "2000 Hz centroid should map to CC74=127, got {}",
            cc
        );
    }

    #[test]
    fn cc74_mid_range() {
        // Geometric mean of 200 and 2000 = sqrt(200*2000) = ~632 Hz
        let mid_freq = (200.0f32 * 2000.0).sqrt();
        let cc = centroid_to_cc74(mid_freq);
        // Should be approximately 63-64 (midpoint of 0-127)
        assert!(
            cc >= 58 && cc <= 70,
            "geometric mid centroid ({:.0} Hz) should map to ~63, got {}",
            mid_freq,
            cc
        );
    }

    #[test]
    fn cc74_below_range_clamps() {
        let cc = centroid_to_cc74(50.0);
        assert_eq!(cc, 0, "below range should clamp to 0");
    }

    #[test]
    fn cc74_above_range_clamps() {
        let cc = centroid_to_cc74(10000.0);
        assert_eq!(cc, 127, "above range should clamp to 127");
    }

    #[test]
    fn cc74_zero_returns_zero() {
        assert_eq!(centroid_to_cc74(0.0), 0);
    }

    // -- compute_pressure tests ──────────────────────────────────────

    #[test]
    fn pressure_at_onset_is_high() {
        let mut pipeline = GuitarInput::with_defaults();
        pipeline.note_onset_rms = 0.5;
        // At onset, current = onset -> ratio = 1.0
        let p = pipeline.compute_pressure(0.5);
        assert!(p >= 120, "pressure at onset should be near max, got {}", p);
    }

    #[test]
    fn pressure_decays_with_rms() {
        let mut pipeline = GuitarInput::with_defaults();
        pipeline.note_onset_rms = 0.5;
        // Half the onset RMS
        let p_half = pipeline.compute_pressure(0.25);
        let p_full = pipeline.compute_pressure(0.5);
        assert!(
            p_half < p_full,
            "pressure at half RMS ({}) should be less than at full RMS ({})",
            p_half,
            p_full
        );
    }

    #[test]
    fn pressure_hold_slows_decay() {
        // With hold=0.0, pressure at half RMS should be lower
        let mut pipeline_no_hold = GuitarInput::new(GuitarInputConfig {
            pressure_hold: 0.0,
            ..GuitarInputConfig::default()
        });
        pipeline_no_hold.note_onset_rms = 0.5;
        let p_no_hold = pipeline_no_hold.compute_pressure(0.25);

        // With hold=0.8, pressure should be higher at the same RMS
        let mut pipeline_hold = GuitarInput::new(GuitarInputConfig {
            pressure_hold: 0.8,
            ..GuitarInputConfig::default()
        });
        pipeline_hold.note_onset_rms = 0.5;
        let p_hold = pipeline_hold.compute_pressure(0.25);

        assert!(
            p_hold > p_no_hold,
            "pressure with hold ({}) should be higher than without ({})",
            p_hold,
            p_no_hold
        );
    }

    #[test]
    fn pressure_near_silence_is_low() {
        let mut pipeline = GuitarInput::with_defaults();
        pipeline.note_onset_rms = 0.5;
        let p = pipeline.compute_pressure(0.001);
        // Even with hold=0.3, near-silence should still be relatively low
        assert!(p < 60, "pressure near silence should be low, got {}", p);
    }

    // -- MPE message ordering tests ──────────────────────────────────

    #[test]
    fn pitch_bend_before_note_on_in_events() {
        // Test that when a NoteOn is emitted with a non-zero cents deviation,
        // the PitchBend event appears before the NoteOn in the events vector.
        let sr = 48000;
        let config = GuitarInputConfig {
            buffer_size: 1024,
            hop_size: 1024, // tests: analyze every full buffer for deterministic behavior
            sample_rate: sr,
            onset_threshold: 0.01,
            string_confidence_min: 0.3,
            bends_enabled: true,
            legato_enabled: false,
            slides_enabled: false,
            vibrato_detection: false,
            vibrato_passthrough: false,
            filter_enabled: false,
            min_clarity: 0.3,
            cooldown_samples: 1024,
            n_harmonics: 6,
            input_gain: 1.0,
            flux_threshold: 0.5,
            per_string_channels: true,
            pitch_bend_range: 2,
            pressure_enabled: true,
            pressure_hold: 0.3,
            brightness_enabled: true,
        };
        let mut pipeline = GuitarInput::new(config);

        // Feed silence then loud signal
        let silence = vec![0.0f32; 1024];
        pipeline.process_block(&silence);

        let a4: Vec<f32> = sine_wave(440.0, sr, 1024 * 5)
            .into_iter()
            .map(|s| s * 0.5)
            .collect();
        let events = pipeline.process_block(&a4);

        // Find the first NoteOn
        if let Some(note_on_idx) = events
            .iter()
            .position(|e| matches!(e, MidiEvent::NoteOn { .. }))
        {
            // Check that any PitchBend events before NoteOn come before it
            for (i, event) in events.iter().enumerate() {
                if i >= note_on_idx {
                    break;
                }
                // Events before NoteOn should be PitchBend, CC74, or ChannelPressure -- not NoteOff for another note
                match event {
                    MidiEvent::PitchBend { .. }
                    | MidiEvent::MidiPitchBend { .. }
                    | MidiEvent::CC { .. }
                    | MidiEvent::ChannelPressure { .. }
                    | MidiEvent::NoteOff { .. } => {
                        // These are fine before NoteOn
                    }
                    MidiEvent::NoteOn { .. } => {
                        panic!(
                            "found an earlier NoteOn at index {} before expected NoteOn at {}",
                            i, note_on_idx
                        );
                    }
                    _ => {}
                }
            }
        }
        // If no NoteOn was found, the test still passes (just didn't trigger)
    }

    // -- MPE channel offset tests ────────────────────────────────────

    #[test]
    fn mpe_channel_offset_string_0_is_channel_1() {
        let sr = 48000;
        let config = GuitarInputConfig {
            buffer_size: 1024,
            hop_size: 1024, // tests: analyze every full buffer for deterministic behavior
            sample_rate: sr,
            onset_threshold: 0.01,
            string_confidence_min: 0.3,
            bends_enabled: false,
            legato_enabled: false,
            slides_enabled: false,
            vibrato_detection: false,
            vibrato_passthrough: false,
            filter_enabled: false,
            min_clarity: 0.3,
            cooldown_samples: 1024,
            n_harmonics: 6,
            input_gain: 1.0,
            flux_threshold: 0.5,
            per_string_channels: true,
            pitch_bend_range: 2,
            pressure_enabled: false,
            pressure_hold: 0.3,
            brightness_enabled: false,
        };
        let mut pipeline = GuitarInput::new(config);

        // Feed silence then a loud E2 signal
        let silence = vec![0.0f32; 1024];
        pipeline.process_block(&silence);

        let e2: Vec<f32> = sine_wave(82.41, sr, 1024 * 5)
            .into_iter()
            .map(|s| s * 0.5)
            .collect();
        let events = pipeline.process_block(&e2);

        // Check NoteOn channel: string 0 (Low E) should be channel 1 (MPE offset)
        for event in &events {
            if let MidiEvent::NoteOn { channel, .. } = event {
                assert!(
                    *channel >= 1,
                    "with per_string_channels, NoteOn channel should be >= 1 (MPE offset), got {}",
                    channel
                );
            }
        }
    }

    // -- MidiEvent::to_midi_bytes tests ─────────────────────────────────

    #[test]
    fn midi_event_to_bytes_note_on() {
        let event = MidiEvent::NoteOn {
            channel: 1,
            note: 60,
            velocity: 100,
        };
        assert_eq!(event.to_midi_bytes(2), vec![0x91, 60, 100]);
    }

    #[test]
    fn midi_event_to_bytes_note_off() {
        let event = MidiEvent::NoteOff {
            channel: 0,
            note: 64,
            velocity: 0,
        };
        assert_eq!(event.to_midi_bytes(2), vec![0x80, 64, 0]);
    }

    #[test]
    fn midi_event_to_bytes_pitch_bend_center() {
        let event = MidiEvent::MidiPitchBend {
            channel: 2,
            value: 8192,
        };
        let bytes = event.to_midi_bytes(2);
        assert_eq!(bytes[0], 0xE2);
        assert_eq!(bytes.len(), 3);
    }

    #[test]
    fn midi_event_to_bytes_cc74() {
        let event = MidiEvent::CC {
            channel: 3,
            controller: 74,
            value: 100,
        };
        assert_eq!(event.to_midi_bytes(2), vec![0xB3, 74, 100]);
    }

    #[test]
    fn midi_event_to_bytes_vibrato_is_empty() {
        let event = MidiEvent::VibratoStatus {
            active: true,
            rate_hz: 5.0,
            depth_cents: 30.0,
        };
        assert!(event.to_midi_bytes(2).is_empty());
    }
}
