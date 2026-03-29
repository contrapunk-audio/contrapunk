//! Guitar auto-calibration — just play naturally.
//!
//! Run with: cargo run --release --example guitar_calibrate
//!
//! Play your guitar normally for 30-60 seconds. The tool listens,
//! auto-identifies which string you're playing, tracks soft/hard
//! dynamics, and builds a calibration profile from your natural playing.
//! A live tuner runs the whole time so you can tune as you go.

use contrapunk::audio::guitar::*;
use contrapunk::audio::pitch::freq_to_midi;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use pitch_detection::detector::mcleod::McLeodDetector;
use pitch_detection::detector::PitchDetector;

use std::io::Write;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

const BUFFER_SIZE: usize = 2048;
const MIN_CONFIDENCE: f64 = 0.50;
const MIN_SAMPLES_PER_STRING: usize = 4; // need at least 4 plucks per string

struct AudioState {
    frequency: f32,
    confidence: f32,
    rms: f32,
    midi_note: u8,
    cents: i8,
    note_name: String,
    peak: f32,
    bright_rms: f32,
    bright_peak: f32,
    prev_rms: f32,
    slope: f32,
}

/// A captured pluck with all its characteristics.
#[derive(Clone)]
struct Pluck {
    midi_note: u8,
    string_idx: usize,
    fret: u8,
    freq: f32,
    conf: f32,
    rms: f32,
    peak: f32,
    bright_rms: f32,
    bright_peak: f32,
    slope: f32,
}

/// Tracks all captured plucks per string.
struct CalibrationCollector {
    plucks: Vec<Pluck>,
    string_matcher: GuitarPitchMatcher,
    noise_floor: f32,
}

impl CalibrationCollector {
    fn new(noise_floor: f32) -> Self {
        Self {
            plucks: Vec::new(),
            string_matcher: GuitarPitchMatcher::with_defaults(),
            noise_floor,
        }
    }

    fn add_pluck(&mut self, p: Pluck) {
        self.plucks.push(p);
    }

    /// Count plucks per string.
    fn string_counts(&self) -> [usize; 6] {
        let mut counts = [0usize; 6];
        for p in &self.plucks {
            if p.string_idx < 6 {
                counts[p.string_idx] += 1;
            }
        }
        counts
    }

    /// Check if we have enough data for all strings.
    fn is_complete(&self) -> bool {
        self.string_counts().iter().all(|&c| c >= MIN_SAMPLES_PER_STRING)
    }

    /// How many strings still need more data.
    fn strings_needed(&self) -> Vec<&str> {
        let counts = self.string_counts();
        (0..6)
            .filter(|&i| counts[i] < MIN_SAMPLES_PER_STRING)
            .map(|i| STRING_NAMES[i])
            .collect()
    }

    /// Build a calibration profile from collected data.
    fn build_profile(&self) -> GuitarCalibrationProfile {
        let mut profile = GuitarCalibrationProfile::default();

        for string_idx in 0..6 {
            let string_plucks: Vec<&Pluck> = self.plucks.iter()
                .filter(|p| p.string_idx == string_idx)
                .collect();

            if string_plucks.is_empty() {
                continue;
            }

            // Sort by RMS to split into soft (bottom half) and strong (top half)
            let mut sorted = string_plucks.clone();
            sorted.sort_by(|a, b| a.rms.partial_cmp(&b.rms).unwrap());

            let split = sorted.len() / 2;
            let soft = &sorted[..split.max(1)];
            let strong = &sorted[split..];

            profile.strings[string_idx].soft_samples = soft.iter().map(|p| to_cal(p)).collect();
            profile.strings[string_idx].strong_samples = strong.iter().map(|p| to_cal(p)).collect();
        }

        profile
    }

    /// Status line showing per-string collection progress.
    fn status_bar(&self) -> String {
        let counts = self.string_counts();
        let parts: Vec<String> = (0..6).map(|i| {
            let count = counts[i];
            let needed = MIN_SAMPLES_PER_STRING;
            let name = match i {
                0 => "E2", 1 => "A ", 2 => "D ", 3 => "G ", 4 => "B ", 5 => "E4",
                _ => "? ",
            };
            if count >= needed {
                format!("{}:OK", name)
            } else {
                format!("{}:{}/{}", name, count, needed)
            }
        }).collect();
        parts.join("  ")
    }
}

fn to_cal(p: &Pluck) -> CalibrationSample {
    CalibrationSample {
        note: midi_to_note_name(p.midi_note),
        freq: p.freq,
        conf: p.conf,
        peak: p.peak,
        rms: p.rms,
        bright_peak: p.bright_peak,
        bright_rms: p.bright_rms,
        main_delta: 0.0,
        main_ratio: if p.rms > 0.0 { p.peak / p.rms } else { 0.0 },
        main_slope: p.slope,
        bright_delta: 0.0,
        bright_ratio: if p.bright_rms > 0.0 { p.bright_peak / p.bright_rms } else { 0.0 },
        bright_slope: 0.0,
    }
}

fn main() {
    println!("=== Contrapunk Auto-Calibrate ===\n");
    println!("Just play your guitar normally. The tool will:");
    println!("  - Show a live tuner so you can tune as you go");
    println!("  - Auto-detect which string you're playing");
    println!("  - Capture soft and hard pluck characteristics");
    println!("  - Build a calibration profile from your natural playing\n");
    println!("Play each string at least {} times (mix of soft and hard).", MIN_SAMPLES_PER_STRING);
    println!("The status bar shows progress per string.\n");

    // ── Audio Setup ──────────────────────────────────────
    let host = cpal::default_host();
    let devices: Vec<_> = host.input_devices().expect("No input devices").collect();
    if devices.is_empty() { eprintln!("No audio input devices!"); return; }

    println!("Audio Input Devices:");
    for (i, d) in devices.iter().enumerate() {
        println!("  [{}] {}", i, d.name().unwrap_or_default());
    }
    let audio_idx = prompt_select(&format!("\nSelect [0-{}]: ", devices.len()-1), devices.len()-1);
    let audio_device = &devices[audio_idx];
    println!("  Using: {}", audio_device.name().unwrap_or_default());

    let config = audio_device.default_input_config().expect("No input config");
    let sample_rate = config.sample_rate().0 as usize;
    let channels = config.channels() as usize;

    println!("  {}ch {}Hz", channels, sample_rate);
    let target_channel = prompt_select(
        &format!("  Channel [0-{}] (default 0): ", channels-1), channels-1);

    // ── Start Audio ──────────────────────────────────────
    let state = Arc::new(Mutex::new(AudioState {
        frequency: 0.0, confidence: 0.0, rms: 0.0,
        midi_note: 0, cents: 0, note_name: String::new(),
        peak: 0.0, bright_rms: 0.0, bright_peak: 0.0,
        prev_rms: 0.0, slope: 0.0,
    }));
    let buffer = Arc::new(Mutex::new(Vec::<f32>::with_capacity(BUFFER_SIZE)));

    let state_c = Arc::clone(&state);
    let buffer_c = Arc::clone(&buffer);
    let (sr, ch, tch) = (sample_rate, channels, target_channel);

    let stream_config: cpal::StreamConfig = config.clone().into();
    let stream = match config.sample_format() {
        cpal::SampleFormat::F32 => audio_device.build_input_stream(
            &stream_config,
            move |data: &[f32], _| { audio_process(data, ch, tch, sr, &buffer_c, &state_c); },
            |e| eprintln!("Audio error: {}", e), None,
        ),
        cpal::SampleFormat::I16 => {
            let sc2 = Arc::clone(&state);
            let bc2 = Arc::clone(&buffer);
            audio_device.build_input_stream(
                &stream_config,
                move |data: &[i16], _| {
                    let f: Vec<f32> = data.iter().map(|&s| s as f32 / 32768.0).collect();
                    audio_process(&f, ch, tch, sr, &bc2, &sc2);
                },
                |e| eprintln!("Audio error: {}", e), None,
            )
        }
        f => { eprintln!("Unsupported: {:?}", f); return; }
    }.expect("Failed to build stream");

    stream.play().expect("Failed to start stream");

    // ── Noise Floor ──────────────────────────────────────
    println!("\n  Measuring noise floor...");
    std::thread::sleep(Duration::from_millis(1200));
    let noise_floor = { state.lock().unwrap().rms };
    println!("  Noise floor: {:.4}\n", noise_floor);

    // ── Main Loop: tuner + auto-capture ──────────────────
    let mut collector = CalibrationCollector::new(noise_floor);
    let pluck_threshold = (noise_floor * 6.0).max(0.015);
    let mut was_quiet = true;
    let mut last_pluck = Instant::now();
    let start = Instant::now();

    println!("  {:>6}  {:>5}  {:>7}  {:>6}  {:>4}  {:>5}  {}",
        "Note", "Cents", "Freq", "String", "Fret", "Conf", "Meter");
    println!("  {}", "-".repeat(55));

    loop {
        std::thread::sleep(Duration::from_millis(30));

        let s = state.lock().unwrap();
        let rms = s.rms;
        let conf = s.confidence;

        // ── Pluck Detection ──────────────────────────────
        if rms < pluck_threshold * 0.5 {
            was_quiet = true;
        }

        if was_quiet
            && rms > pluck_threshold
            && conf > MIN_CONFIDENCE as f32
            && s.midi_note >= 40
            && s.midi_note <= 88
            && last_pluck.elapsed() > Duration::from_millis(250)
        {
            was_quiet = false;
            last_pluck = Instant::now();

            // Identify string
            if let Some(sm) = collector.string_matcher.identify_string(s.midi_note, conf) {
                collector.add_pluck(Pluck {
                    midi_note: s.midi_note,
                    string_idx: sm.string_idx,
                    fret: sm.fret,
                    freq: s.frequency,
                    conf,
                    rms,
                    peak: s.peak,
                    bright_rms: s.bright_rms,
                    bright_peak: s.bright_peak,
                    slope: s.slope,
                });
            }
        }

        // ── Tuner Display ────────────────────────────────
        if rms > noise_floor * 2.0 && conf > MIN_CONFIDENCE as f32 {
            let cents_str = if s.cents >= 0 { format!("+{}¢", s.cents) } else { format!("{}¢", s.cents) };
            let sm = collector.string_matcher.identify_string(s.midi_note, conf);
            let (s_str, f_str) = match sm {
                Some(ref m) => (STRING_NAMES[m.string_idx].to_string(), format!("{}", m.fret)),
                None => ("?".into(), "?".into()),
            };
            let meter = build_meter(s.cents);

            print!("\r  {:>6}  {:>5}  {:>6.1}  {:>6}  {:>4}  {:>4.0}%  {}",
                s.note_name, cents_str, s.frequency, s_str, f_str, conf * 100.0, meter);
        } else {
            print!("\r  {:>6}  {:>5}  {:>7}  {:>6}  {:>4}  {:>5}  {:21}",
                "---", "", "", "", "", "", "");
        }
        std::io::stdout().flush().unwrap();

        // ── Progress Bar (every 2 seconds) ───────────────
        let elapsed = start.elapsed().as_secs();
        if elapsed % 2 == 0 && elapsed > 0 {
            // Show on line below
            print!("\n  {}\r\x1b[1A", collector.status_bar());
            std::io::stdout().flush().unwrap();
        }

        // ── Check Completion ─────────────────────────────
        if collector.is_complete() {
            break;
        }

        // Auto-finish after 90 seconds with whatever we have
        if elapsed > 90 {
            let needed = collector.strings_needed();
            if !needed.is_empty() {
                println!("\n\n  Timeout — missing data for: {}", needed.join(", "));
                println!("  Saving partial profile...");
            }
            break;
        }
    }

    // ── Build & Save Profile ─────────────────────────────
    let total_plucks = collector.plucks.len();
    let profile = collector.build_profile();

    println!("\n\n══════════════════════════════════════════════");
    println!("  Calibration Complete! ({} plucks captured)", total_plucks);
    println!("══════════════════════════════════════════════\n");

    println!("  {:>6}  {:>5}  {:>8}  {:>8}  {:>8}  {:>6}",
        "String", "Count", "SoftRMS", "StrongRMS", "OnsetTh", "Conf");
    println!("  {}", "-".repeat(50));
    let counts = collector.string_counts();
    for (i, cal) in profile.strings.iter().enumerate() {
        let strong_rms = if cal.strong_samples.is_empty() { 0.0 }
            else { cal.strong_samples.iter().map(|s| s.rms).sum::<f32>() / cal.strong_samples.len() as f32 };
        println!("  {:>6}  {:>5}  {:>8.4}  {:>8.4}  {:>8.4}  {:>5.1}%",
            STRING_NAMES[i], counts[i],
            cal.soft_rms_threshold(), strong_rms,
            cal.onset_slope_threshold(), cal.avg_confidence() * 100.0);
    }

    let filename = "guitar_calibration_profile.json";
    let json = profile.to_json().expect("Failed to serialize");
    std::fs::write(filename, &json).expect("Failed to write");
    println!("\n  Saved: {} ({} bytes)", filename, json.len());
    println!("\n  Use with:");
    println!("    cargo run --release --example guitar_harmony -- --profile {}", filename);
}

// ── Audio Processing ─────────────────────────────────────────────────

fn audio_process(
    data: &[f32], channels: usize, target_channel: usize, sample_rate: usize,
    buffer: &Arc<Mutex<Vec<f32>>>, state: &Arc<Mutex<AudioState>>,
) {
    let mut buf = buffer.lock().unwrap();
    for frame in data.chunks(channels) {
        buf.push(frame.get(target_channel).copied().unwrap_or(0.0));
    }

    if buf.len() >= BUFFER_SIZE {
        let frame = buf[..BUFFER_SIZE].to_vec();
        let overflow: Vec<f32> = buf[BUFFER_SIZE..].to_vec();
        buf.clear();
        buf.extend_from_slice(&overflow);

        let rms = (frame.iter().map(|s| s * s).sum::<f32>() / BUFFER_SIZE as f32).sqrt();
        let peak = frame.iter().map(|s| s.abs()).fold(0.0f32, f32::max);
        let half = BUFFER_SIZE / 2;
        let bright_rms = (frame[half..].iter().map(|s| s * s).sum::<f32>() / half as f32).sqrt();
        let bright_peak = frame[half..].iter().map(|s| s.abs()).fold(0.0f32, f32::max);

        let samples_f64: Vec<f64> = frame.iter().map(|&s| s as f64).collect();
        let mut detector = McLeodDetector::new(BUFFER_SIZE, BUFFER_SIZE / 2);
        let pitch = detector.get_pitch(&samples_f64, sample_rate, 0.3, 0.3);

        let mut s = state.lock().unwrap();
        let slope = rms - s.prev_rms;
        s.rms = rms; s.peak = peak; s.bright_rms = bright_rms;
        s.bright_peak = bright_peak; s.slope = slope; s.prev_rms = rms;

        if let Some(p) = pitch {
            if p.clarity > MIN_CONFIDENCE {
                let freq = p.frequency as f32;
                let (midi, cents) = freq_to_midi(freq);
                s.frequency = freq; s.confidence = p.clarity as f32;
                s.midi_note = midi; s.cents = cents;
                s.note_name = midi_to_note_name(midi);
            }
        }
    }
}

// ── Utilities ────────────────────────────────────────────────────────

fn prompt_select(prompt: &str, max: usize) -> usize {
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
        .round().clamp(0.0, (width - 1) as f32) as usize;
    let ch = if cents.abs() <= 5 { '*' } else if cents.abs() <= 15 { '=' } else { '#' };
    bar[pos] = ch;
    format!("[{}]", bar.iter().collect::<String>())
}
