//! Live guitar → onset-locked detection → harmony engine → MIDI output.
//!
//! Run with: cargo run --release --example guitar_harmony
//!
//! Model: notes ONLY fire on pluck onset. During sustain, pitch is locked.
//! A short correction window (~15ms) after onset allows pitch refinement.
//! Pipeline: Audio → OverlapManager → onset detection → pitch detection →
//! onset-locked tracker → HarmonyEngine → MIDI out.

use contrapunk::audio::buffer::OverlapManager;
use contrapunk::audio::detectors::GoertzelBank;
use contrapunk::audio::guitar::*;
use contrapunk::audio::onset::PluckDetector;
use contrapunk::audio::pitch::freq_to_midi;
use contrapunk::harmony::{HarmonyEngine, HarmonyMode, Key};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use midir::{MidiOutput, MidiOutputConnection};
use pitch_detection::detector::mcleod::McLeodDetector;
use pitch_detection::detector::PitchDetector;
use wmidi::Note;

use std::io::Write;
use std::sync::{Arc, Mutex};

const BUFFER_SIZE: usize = 1024;
const OVERLAP_PCT: u8 = 75;
const MIN_CONFIDENCE: f64 = 0.55;
const MIN_RMS: f32 = 0.015;
const VELOCITY: u8 = 100;
const SILENCE_FRAMES: u32 = 10; // ~53ms at 5.3ms/frame before note-off
const CORRECTION_FRAMES: u32 = 3; // ~16ms window to correct pitch after onset

// ── Onset-Locked Note State ──────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq)]
enum TrackState {
    /// Waiting for a pluck. No note active.
    Idle,
    /// Onset detected, collecting pitch candidates for confirmation.
    Confirming { frames: u32 },
    /// Note is locked and playing. No pitch changes allowed until next onset.
    Locked,
}

/// Events emitted by the onset-locked tracker.
#[derive(Debug, Clone)]
enum PipelineEvent {
    NoteOn(u8),
    NoteOff(u8),
    /// Correction: release old, play new (within correction window).
    Correct {
        off: u8,
        on: u8,
    },
}

/// Onset-locked tracker: only fires on pluck, locks during sustain.
struct OnsetLockedTracker {
    state: TrackState,
    /// Active MIDI note (None if idle)
    current_note: Option<u8>,
    /// Pitch candidates during confirmation window: (midi_note, confidence)
    candidates: Vec<(u8, f32)>,
    /// Frames of silence
    silence_count: u32,
}

impl OnsetLockedTracker {
    fn new() -> Self {
        Self {
            state: TrackState::Idle,
            current_note: None,
            candidates: Vec::with_capacity(8),
            silence_count: 0,
        }
    }

    /// Call when an onset (pluck) is detected.
    fn on_onset(&mut self) -> Vec<PipelineEvent> {
        let mut events = Vec::new();

        // If a note is currently playing, release it
        if let Some(old) = self.current_note.take() {
            events.push(PipelineEvent::NoteOff(old));
        }

        self.state = TrackState::Confirming { frames: 0 };
        self.candidates.clear();
        self.silence_count = 0;
        events
    }

    /// Feed a pitch detection (or None for silence).
    fn feed(&mut self, detected: Option<(u8, f32)>) -> Vec<PipelineEvent> {
        let mut events = Vec::new();

        match detected {
            Some((midi_note, confidence)) => {
                self.silence_count = 0;

                match self.state {
                    TrackState::Idle => {
                        // Pitch without onset — ignore (noise/brush)
                    }
                    TrackState::Confirming { frames } => {
                        self.candidates.push((midi_note, confidence));
                        let new_frames = frames + 1;

                        if new_frames >= CORRECTION_FRAMES {
                            // Confirmation window closed — pick the best candidate
                            let best = self.best_candidate();
                            self.current_note = Some(best);
                            self.state = TrackState::Locked;
                            events.push(PipelineEvent::NoteOn(best));
                        } else {
                            self.state = TrackState::Confirming { frames: new_frames };

                            // Fire immediately on first candidate for low latency,
                            // but allow correction on subsequent frames
                            if new_frames == 1 {
                                let best = self.best_candidate();
                                self.current_note = Some(best);
                                events.push(PipelineEvent::NoteOn(best));
                            } else {
                                // Check if best candidate changed — correct if so
                                let best = self.best_candidate();
                                if let Some(current) = self.current_note {
                                    if best != current {
                                        events.push(PipelineEvent::Correct {
                                            off: current,
                                            on: best,
                                        });
                                        self.current_note = Some(best);
                                    }
                                }
                            }
                        }
                    }
                    TrackState::Locked => {
                        // Sustain — pitch is locked, ignore new detections
                    }
                }
            }
            None => {
                // Silence
                match self.state {
                    TrackState::Confirming { .. } => {
                        // Onset detected but silence before confirmation — false trigger
                        // Release any premature note
                        if let Some(old) = self.current_note.take() {
                            events.push(PipelineEvent::NoteOff(old));
                        }
                        self.state = TrackState::Idle;
                        self.candidates.clear();
                    }
                    TrackState::Locked | TrackState::Idle => {
                        if self.current_note.is_some() {
                            self.silence_count += 1;
                            if self.silence_count >= SILENCE_FRAMES {
                                if let Some(old) = self.current_note.take() {
                                    events.push(PipelineEvent::NoteOff(old));
                                }
                                self.state = TrackState::Idle;
                            }
                        }
                    }
                }
            }
        }

        events
    }

    /// Pick the best candidate: most frequent note, tie-broken by highest confidence.
    fn best_candidate(&self) -> u8 {
        if self.candidates.is_empty() {
            return 0;
        }

        // Count occurrences of each note
        let mut counts: Vec<(u8, u32, f32)> = Vec::new(); // (note, count, max_confidence)
        for &(note, conf) in &self.candidates {
            if let Some(entry) = counts.iter_mut().find(|e| e.0 == note) {
                entry.1 += 1;
                if conf > entry.2 {
                    entry.2 = conf;
                }
            } else {
                counts.push((note, 1, conf));
            }
        }

        // Sort by count desc, then confidence desc
        counts.sort_by(|a, b| {
            b.1.cmp(&a.1)
                .then(b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal))
        });
        counts[0].0
    }
}

// ── Audio Pipeline ───────────────────────────────────────────────────

struct AudioPipeline {
    overlap: OverlapManager,
    pluck_detector: PluckDetector,
    goertzel: GoertzelBank,
    tracker: OnsetLockedTracker,
    string_matcher: GuitarPitchMatcher,
    normalizer: AudioNormalizer,
    target_channel: usize,
    channels: usize,
    sample_rate: usize,
    prev_rms: f32,
    /// Last detected string for per-string normalization
    last_string_hint: Option<usize>,
}

struct DisplayInfo {
    frequency: f32,
    confidence: f32,
    rms: f32,
    midi_note: u8,
    cents: i8,
    note_name: String,
    string_match: Option<(usize, u8)>,
}

impl AudioPipeline {
    fn new(
        channels: usize,
        target_channel: usize,
        sample_rate: usize,
        profile: Option<GuitarCalibrationProfile>,
    ) -> Self {
        let (normalizer, matcher) = match profile {
            Some(ref p) => {
                let n = AudioNormalizer::from_profile(p);
                let mut m = GuitarPitchMatcher::with_defaults();
                m.set_profile(p.clone());
                (n, m)
            }
            None => (
                AudioNormalizer::default_uncalibrated(),
                GuitarPitchMatcher::with_defaults(),
            ),
        };

        Self {
            overlap: OverlapManager::new(BUFFER_SIZE, OVERLAP_PCT),
            pluck_detector: PluckDetector::new(BUFFER_SIZE / 2 + 1),
            goertzel: GoertzelBank::new(sample_rate),
            tracker: OnsetLockedTracker::new(),
            string_matcher: matcher,
            normalizer,
            target_channel,
            channels,
            sample_rate,
            prev_rms: 0.0,
            last_string_hint: None,
        }
    }

    fn process(&mut self, data: &[f32]) -> (Vec<PipelineEvent>, Option<DisplayInfo>) {
        let mut events = Vec::new();
        let mut display = None;

        let mono: Vec<f32> = data
            .chunks(self.channels)
            .map(|frame| frame.get(self.target_channel).copied().unwrap_or(0.0))
            .collect();

        let frames = self.overlap.feed(&mono);

        for frame in frames {
            let rms = (frame.iter().map(|s| s * s).sum::<f32>() / frame.len() as f32).sqrt();
            let half = frame.len() / 2;
            let bright_rms =
                (frame[half..].iter().map(|s| s * s).sum::<f32>() / half as f32).sqrt();

            // ── Normalizer: noise gate + brightness filter ───
            // Do a preliminary pitch detection to get confidence for the normalizer
            let samples_f64: Vec<f64> = frame.iter().map(|&s| s as f64).collect();
            let mut mcleod = McLeodDetector::new(frame.len(), frame.len() / 2);
            let mcleod_result = mcleod.get_pitch(&samples_f64, self.sample_rate, 0.3, 0.3);

            let pre_confidence = mcleod_result
                .as_ref()
                .map(|p| p.clarity as f32)
                .unwrap_or(0.0);

            let norm =
                self.normalizer
                    .normalize(rms, bright_rms, pre_confidence, self.last_string_hint);

            match norm {
                NormalizeResult::Rejected => {
                    events.extend(self.tracker.feed(None));
                    self.prev_rms = rms * 0.2 + self.prev_rms * 0.8;
                    continue;
                }
                NormalizeResult::Valid { is_pluck_like, .. } => {
                    // ── Onset detection ──────────────────────
                    let spectrum = simple_magnitude_spectrum(&frame);
                    let hfc_onset = self.pluck_detector.feed(&spectrum);

                    let rms_onset = self.prev_rms > 0.002
                        && rms > self.prev_rms * 4.0
                        && rms > self.normalizer.global_noise_floor() * 3.0;
                    self.prev_rms = rms * 0.2 + self.prev_rms * 0.8;

                    // Only trigger onset if it looks pluck-like OR HFC fires
                    if (hfc_onset && is_pluck_like) || rms_onset {
                        events.extend(self.tracker.on_onset());
                    }

                    // ── Pitch detection ──────────────────────
                    let goertzel_note = self.goertzel.analyze(&frame).map(|(midi, _)| midi);

                    let detected = match mcleod_result {
                        Some(p) if p.clarity > MIN_CONFIDENCE => {
                            let freq = p.frequency as f32;
                            let (midi_note, cents) = freq_to_midi(freq);
                            let confidence = p.clarity as f32;

                            // Cross-validate with Goertzel
                            let goertzel_agrees = goertzel_note.map_or(true, |g| {
                                (midi_note as i16 - g as i16).unsigned_abs() <= 2
                            });

                            if midi_note >= 40 && midi_note <= 88 && goertzel_agrees {
                                let string_match = self
                                    .string_matcher
                                    .identify_string(midi_note, confidence)
                                    .map(|sm| {
                                        self.last_string_hint = Some(sm.string_idx);
                                        (sm.string_idx, sm.fret)
                                    });

                                display = Some(DisplayInfo {
                                    frequency: freq,
                                    confidence,
                                    rms,
                                    midi_note,
                                    cents,
                                    note_name: midi_to_note_name(midi_note),
                                    string_match,
                                });

                                Some((midi_note, confidence))
                            } else {
                                None
                            }
                        }
                        _ => None,
                    };

                    // Feed tracker
                    events.extend(self.tracker.feed(detected));
                }
            }
        }

        (events, display)
    }
}

fn simple_magnitude_spectrum(samples: &[f32]) -> Vec<f32> {
    let n = samples.len();
    let num_bins = n / 2 + 1;
    let mut magnitudes = vec![0.0f32; num_bins];
    let bin_size = if num_bins > 0 { n / num_bins } else { 1 };
    for (i, mag) in magnitudes.iter_mut().enumerate() {
        let start = i * bin_size;
        let end = (start + bin_size).min(n);
        if start < end {
            let sum_sq: f32 = samples[start..end].iter().map(|s| s * s).sum();
            *mag = (sum_sq / (end - start) as f32).sqrt();
        }
    }
    magnitudes
}

// ── Utilities ────────────────────────────────────────────────────────

fn list_midi_outputs() -> Vec<(usize, String)> {
    let midi_out = MidiOutput::new("contrapunk-list").unwrap();
    let ports = midi_out.ports();
    ports
        .iter()
        .enumerate()
        .map(|(i, p)| (i, midi_out.port_name(p).unwrap_or("Unknown".into())))
        .collect()
}

fn connect_midi_output(port_idx: usize) -> MidiOutputConnection {
    let midi_out = MidiOutput::new("contrapunk-guitar").unwrap();
    let ports = midi_out.ports();
    let port = &ports[port_idx];
    let name = midi_out.port_name(port).unwrap_or("Unknown".into());
    println!("  Connected to MIDI output: {}", name);
    midi_out
        .connect(port, "contrapunk-guitar-out")
        .expect("Failed to connect MIDI output")
}

fn prompt_selection(prompt: &str, max: usize) -> usize {
    print!("{}", prompt);
    std::io::stdout().flush().unwrap();
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
    input.trim().parse::<usize>().unwrap_or(0).min(max)
}

fn build_meter(cents: i8) -> String {
    let width = 21;
    let center = width / 2;
    let mut bar = vec!['-'; width];
    bar[center] = '|';
    let pos = ((cents as f32 / 50.0) * center as f32 + center as f32)
        .round()
        .clamp(0.0, (width - 1) as f32) as usize;
    let indicator = if cents.abs() <= 5 {
        '*'
    } else if cents.abs() <= 15 {
        '='
    } else {
        '#'
    };
    bar[pos] = indicator;
    format!("[{}]", bar.iter().collect::<String>())
}

// ── Main ─────────────────────────────────────────────────────────────

fn main() {
    println!("=== Contrapunk Guitar Harmony v3 ===");
    println!("  Onset-locked: notes only fire on pluck, locked during sustain\n");

    // ── Load calibration profile ───────────────────────
    // Check --profile flag first, then auto-load from default path
    let args: Vec<String> = std::env::args().collect();
    let profile_path = args
        .iter()
        .position(|a| a == "--profile")
        .and_then(|idx| args.get(idx + 1).cloned())
        .unwrap_or_else(|| "guitar_calibration_profile.json".into());

    let profile = match std::fs::read_to_string(&profile_path) {
        Ok(json) => match GuitarCalibrationProfile::from_json(&json) {
            Ok(p) => {
                println!(
                    "  Loaded profile: {} ({} plucks)\n",
                    profile_path,
                    p.strings
                        .iter()
                        .map(|s| s.soft_samples.len() + s.strong_samples.len())
                        .sum::<usize>()
                );
                Some(p)
            }
            Err(e) => {
                eprintln!("  Bad profile: {} — using defaults\n", e);
                None
            }
        },
        Err(_) => {
            println!("  No profile found. Run guitar_tuner first to auto-calibrate.\n");
            None
        }
    };

    // ── Audio Input ──────────────────────────────────────
    let host = cpal::default_host();
    let devices: Vec<_> = host.input_devices().expect("No input devices").collect();
    if devices.is_empty() {
        eprintln!("No audio input devices!");
        return;
    }

    println!("Audio Input Devices:");
    for (i, d) in devices.iter().enumerate() {
        println!("  [{}] {}", i, d.name().unwrap_or_default());
    }
    let audio_idx = prompt_selection(
        &format!("\nSelect [0-{}]: ", devices.len() - 1),
        devices.len() - 1,
    );
    let audio_device = &devices[audio_idx];
    println!("  Using: {}\n", audio_device.name().unwrap_or_default());

    // ── MIDI Output ──────────────────────────────────────
    let midi_outputs = list_midi_outputs();
    if midi_outputs.is_empty() {
        eprintln!("No MIDI outputs!");
        return;
    }

    println!("MIDI Output Devices:");
    for (i, name) in &midi_outputs {
        println!("  [{}] {}", i, name);
    }
    let midi_idx = prompt_selection(
        &format!("\nSelect [0-{}]: ", midi_outputs.len() - 1),
        midi_outputs.len() - 1,
    );
    let midi_conn = Arc::new(Mutex::new(connect_midi_output(midi_idx)));

    // ── Harmony Setup ────────────────────────────────────
    println!("\nHarmony Modes:");
    let modes = [
        ("0", "Pass Through", HarmonyMode::PassThrough),
        ("1", "Parallel Thirds", HarmonyMode::DiatonicThirds),
        ("2", "Parallel Fourths", HarmonyMode::DiatonicFourths),
        ("3", "Contrary Motion", HarmonyMode::ContraryMotion),
        (
            "4",
            "Counterpoint (Species 1)",
            HarmonyMode::StrictCounterpoint,
        ),
    ];
    for (i, label, _) in &modes {
        println!("  [{}] {}", i, label);
    }
    let mode_idx = prompt_selection("\nSelect mode [0-4]: ", 4);
    let harmony_mode = modes[mode_idx].2.clone();

    let keys = [
        ("0", "C", Key::C),
        ("1", "Db", Key::Db),
        ("2", "D", Key::D),
        ("3", "Eb", Key::Eb),
        ("4", "E", Key::E),
        ("5", "F", Key::F),
        ("6", "Gb", Key::Gb),
        ("7", "G", Key::G),
        ("8", "Ab", Key::Ab),
        ("9", "A", Key::A),
        ("10", "Bb", Key::Bb),
        ("11", "B", Key::B),
    ];
    println!("\nKeys:");
    for (i, label, _) in &keys {
        print!(" [{}]{}", i, label);
    }
    println!();
    let key_idx = prompt_selection("Select key [0-11]: ", 11);
    let mut engine = HarmonyEngine::new(keys[key_idx].2.clone(), harmony_mode.clone());

    // ── Audio Stream ─────────────────────────────────────
    let config = audio_device
        .default_input_config()
        .expect("No input config");
    let sample_rate = config.sample_rate().0 as usize;
    let channels = config.channels() as usize;

    println!(
        "\n  Engine: key={} mode={:?}",
        keys[key_idx].1, harmony_mode
    );
    println!(
        "  Audio: {}ch {}Hz | Buf:{} | Overlap:{}% | Correction:{}frames",
        channels, sample_rate, BUFFER_SIZE, OVERLAP_PCT, CORRECTION_FRAMES
    );

    println!("  Device has {} channels.", channels);
    let target_channel = prompt_selection(
        &format!("  Select channel [0-{}]: ", channels - 1),
        channels - 1,
    );
    println!("  Using channel {}\n", target_channel);

    let state = Arc::new(Mutex::new((
        Vec::<PipelineEvent>::new(),
        String::new(),
        0i8,
        None::<(usize, u8)>,
        0.0f32,
        0.0f32,
    )));

    let pipeline = Arc::new(Mutex::new(AudioPipeline::new(
        channels,
        target_channel,
        sample_rate,
        profile,
    )));

    let state_c = Arc::clone(&state);
    let pipeline_c = Arc::clone(&pipeline);

    let stream_config: cpal::StreamConfig = config.clone().into();
    let stream = match config.sample_format() {
        cpal::SampleFormat::F32 => audio_device.build_input_stream(
            &stream_config,
            move |data: &[f32], _: &cpal::InputCallbackInfo| {
                let mut pipe = pipeline_c.lock().unwrap();
                let (events, display) = pipe.process(data);
                let mut s = state_c.lock().unwrap();
                s.0.extend(events);
                if let Some(d) = display {
                    s.1 = d.note_name;
                    s.2 = d.cents;
                    s.3 = d.string_match;
                    s.4 = d.confidence;
                    s.5 = d.rms;
                }
            },
            |e| eprintln!("Audio error: {}", e),
            None,
        ),
        cpal::SampleFormat::I16 => {
            let state_c2 = Arc::clone(&state);
            let pipeline_c2 = Arc::clone(&pipeline);
            audio_device.build_input_stream(
                &stream_config,
                move |data: &[i16], _: &cpal::InputCallbackInfo| {
                    let floats: Vec<f32> = data.iter().map(|&s| s as f32 / 32768.0).collect();
                    let mut pipe = pipeline_c2.lock().unwrap();
                    let (events, display) = pipe.process(&floats);
                    let mut s = state_c2.lock().unwrap();
                    s.0.extend(events);
                    if let Some(d) = display {
                        s.1 = d.note_name;
                        s.2 = d.cents;
                        s.3 = d.string_match;
                        s.4 = d.confidence;
                        s.5 = d.rms;
                    }
                },
                |e| eprintln!("Audio error: {}", e),
                None,
            )
        }
        f => {
            eprintln!("Unsupported: {:?}", f);
            return;
        }
    }
    .expect("Failed to build stream");

    stream.play().expect("Failed to start stream");

    println!("Listening... Play your guitar! (Ctrl+C to quit)\n");
    println!(
        "  {:>6} {:>5} {:>6} {:>4}  {:>20}  {}",
        "Note", "Cents", "String", "Fret", "Harmony", "Meter"
    );
    println!("  {}", "-".repeat(68));

    let mut active_harmony: Vec<u8> = Vec::new();

    loop {
        std::thread::sleep(std::time::Duration::from_millis(3));

        let (events, name, cents, string_match, conf, rms) = {
            let mut s = state.lock().unwrap();
            let evs: Vec<PipelineEvent> = s.0.drain(..).collect();
            (evs, s.1.clone(), s.2, s.3, s.4, s.5)
        };

        for event in &events {
            match event {
                PipelineEvent::NoteOff(off) => {
                    let wmidi_note = Note::from_u8_lossy(*off);
                    let release = engine.harmonize_note_off(wmidi_note);
                    let mut conn = midi_conn.lock().unwrap();
                    for n in &release {
                        let _ = conn.send(&[0x80, u8::from(*n), 0]);
                    }
                    active_harmony.clear();
                }
                PipelineEvent::NoteOn(on) => {
                    // Release previous
                    if !active_harmony.is_empty() {
                        let mut conn = midi_conn.lock().unwrap();
                        for &m in &active_harmony {
                            let _ = conn.send(&[0x80, m, 0]);
                        }
                    }
                    let wmidi_note = Note::from_u8_lossy(*on);
                    let harmony = engine.harmonize_note_on(wmidi_note);
                    let mut sorted: Vec<u8> = harmony.iter().map(|n| u8::from(*n)).collect();
                    sorted.sort();
                    let mut conn = midi_conn.lock().unwrap();
                    for &m in &sorted {
                        let _ = conn.send(&[0x90, m, VELOCITY]);
                    }
                    active_harmony = sorted;
                }
                PipelineEvent::Correct { off, on } => {
                    // Quick correction: release wrong, play right
                    {
                        let wmidi_off = Note::from_u8_lossy(*off);
                        let release = engine.harmonize_note_off(wmidi_off);
                        let mut conn = midi_conn.lock().unwrap();
                        for n in &release {
                            let _ = conn.send(&[0x80, u8::from(*n), 0]);
                        }
                    }
                    let wmidi_on = Note::from_u8_lossy(*on);
                    let harmony = engine.harmonize_note_on(wmidi_on);
                    let mut sorted: Vec<u8> = harmony.iter().map(|n| u8::from(*n)).collect();
                    sorted.sort();
                    let mut conn = midi_conn.lock().unwrap();
                    for &m in &sorted {
                        let _ = conn.send(&[0x90, m, VELOCITY]);
                    }
                    active_harmony = sorted;
                }
            }
        }

        // Display
        let has_signal = rms >= MIN_RMS && conf > MIN_CONFIDENCE as f32;
        if !events.is_empty() || has_signal {
            let cents_str = if cents >= 0 {
                format!("+{}¢", cents)
            } else {
                format!("{}¢", cents)
            };
            let (s_str, f_str) = match string_match {
                Some((idx, fret)) => (STRING_NAMES[idx].to_string(), format!("{}", fret)),
                None => ("?".into(), "?".into()),
            };
            let h: Vec<String> = active_harmony
                .iter()
                .map(|&m| midi_to_note_name(m))
                .collect();
            let h_str = if h.is_empty() {
                "---".into()
            } else {
                h.join(" ")
            };
            print!(
                "\r  {:>6} {:>5} {:>6} {:>4}  {:>20}  {}",
                name,
                cents_str,
                s_str,
                f_str,
                h_str,
                build_meter(cents)
            );
            std::io::stdout().flush().unwrap();
        } else if active_harmony.is_empty() {
            print!(
                "\r  {:>6} {:>5} {:>6} {:>4}  {:>20}  {:21}",
                "---", "", "", "", "", ""
            );
            std::io::stdout().flush().unwrap();
        }
    }
}
