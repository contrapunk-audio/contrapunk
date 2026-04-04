//! Guitar tuner with per-string guided tuning and auto-calibration.
//!
//! Run with: cargo run --release --example guitar_tuner
//!
//! Walks you through tuning each string (low E → high E). Shows a live
//! tuner for each string. While you tune, it captures pluck data
//! automatically. Once in tune, moves to the next string. On exit,
//! saves a calibration profile that guitar_harmony loads automatically.

use contrapunk::audio::buffer::OverlapManager;
use contrapunk::audio::detectors::GoertzelBank;
use contrapunk::audio::guitar::*;
use contrapunk::audio::onset::PluckDetector;
use contrapunk::audio::pitch::freq_to_midi;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use pitch_detection::detector::mcleod::McLeodDetector;
use pitch_detection::detector::PitchDetector;

use std::io::Write;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

const BUFFER_SIZE: usize = 1024;
const OVERLAP_PCT: u8 = 75;
const MIN_CONFIDENCE: f64 = 0.45;
const IN_TUNE_CENTS: i8 = 8;
const IN_TUNE_HOLD_SECS: f32 = 1.2;
const MIN_PLUCKS_PER_STRING: usize = 4;
const PROFILE_PATH: &str = "guitar_calibration_profile.json";

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
    slope: f32,
    prev_rms: f32,
    goertzel_note: Option<u8>,
}

#[derive(Clone)]
struct Pluck {
    midi_note: u8,
    freq: f32,
    conf: f32,
    rms: f32,
    peak: f32,
    bright_rms: f32,
    bright_peak: f32,
    slope: f32,
}

fn main() {
    println!("=== Contrapunk Guitar Tuner ===\n");

    // ── Audio Setup ──────────────────────────────────────
    let host = cpal::default_host();
    let devices: Vec<_> = host.input_devices().expect("No input devices").collect();
    if devices.is_empty() {
        eprintln!("No audio input devices!");
        return;
    }

    println!("Audio Inputs:");
    for (i, d) in devices.iter().enumerate() {
        println!("  [{}] {}", i, d.name().unwrap_or_default());
    }
    let dev_idx = prompt(
        &format!("\nSelect [0-{}]: ", devices.len() - 1),
        devices.len() - 1,
    );
    let device = &devices[dev_idx];
    println!("  Using: {}", device.name().unwrap_or_default());

    let config = device.default_input_config().expect("No input config");
    let sr = config.sample_rate().0 as usize;
    let ch = config.channels() as usize;
    println!("  {}ch {}Hz", ch, sr);
    let tch = prompt(&format!("  Channel [0-{}]: ", ch - 1), ch - 1);

    // ── Start Audio ──────────────────────────────────────
    let state = Arc::new(Mutex::new(AudioState {
        frequency: 0.0,
        confidence: 0.0,
        rms: 0.0,
        midi_note: 0,
        cents: 0,
        note_name: String::new(),
        peak: 0.0,
        bright_rms: 0.0,
        bright_peak: 0.0,
        slope: 0.0,
        prev_rms: 0.0,
        goertzel_note: None,
    }));

    let overlap = Arc::new(Mutex::new(OverlapManager::new(BUFFER_SIZE, OVERLAP_PCT)));
    let pluck_det = Arc::new(Mutex::new(PluckDetector::new(BUFFER_SIZE / 2 + 1)));
    let goertzel = Arc::new(Mutex::new(GoertzelBank::new(sr)));

    let state_c = Arc::clone(&state);
    let oc = Arc::clone(&overlap);
    let pc = Arc::clone(&pluck_det);
    let gc = Arc::clone(&goertzel);

    let stream_config: cpal::StreamConfig = config.clone().into();
    let stream = match config.sample_format() {
        cpal::SampleFormat::F32 => device.build_input_stream(
            &stream_config,
            move |data: &[f32], _| {
                audio_process(data, ch, tch, sr, &oc, &pc, &gc, &state_c);
            },
            |e| eprintln!("Audio error: {}", e),
            None,
        ),
        cpal::SampleFormat::I16 => {
            let sc2 = Arc::clone(&state);
            let oc2 = Arc::clone(&overlap);
            let pc2 = Arc::clone(&pluck_det);
            let gc2 = Arc::clone(&goertzel);
            device.build_input_stream(
                &stream_config,
                move |data: &[i16], _| {
                    let f: Vec<f32> = data.iter().map(|&s| s as f32 / 32768.0).collect();
                    audio_process(&f, ch, tch, sr, &oc2, &pc2, &gc2, &sc2);
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

    // ── Noise Floor ──────────────────────────────────────
    println!("\n  Measuring noise floor...");
    std::thread::sleep(Duration::from_millis(1200));
    let noise_floor = { state.lock().unwrap().rms };
    let pluck_threshold = (noise_floor * 5.0).max(0.012);
    println!("  Noise: {:.4}\n", noise_floor);

    // ── Per-String Tuning (3 passes) ────────────────────
    let mut all_plucks: Vec<(usize, Pluck)> = Vec::new();

    let passes: Vec<(&str, Vec<usize>)> = vec![
        ("Pass 1/3: Low E → High E", (0..6).collect()),
        ("Pass 2/3: High E → Low E", (0..6).rev().collect()),
        ("Pass 3/3: Low E → High E (final check)", (0..6).collect()),
    ];

    for (pass_idx, (pass_name, string_order)) in passes.iter().enumerate() {
        println!("\n╔══════════════════════════════════════════════╗");
        println!("║  {}{}║", pass_name, " ".repeat(44 - pass_name.len()));
        println!("╚══════════════════════════════════════════════╝\n");

        for (step, &string_idx) in string_order.iter().enumerate() {
            let total_strings = string_order.len();
            let pluck_count = tune_string(
                string_idx,
                step + 1,
                total_strings,
                pass_idx,
                &state,
                noise_floor,
                pluck_threshold,
                &mut all_plucks,
            );

            println!(
                "\n  {} {}! ({} plucks)\n",
                STRING_NAMES[string_idx],
                if pass_idx == 2 { "confirmed" } else { "tuned" },
                pluck_count
            );
        }
    }

    // ── Build & Save Profile ─────────────────────────────
    println!("\n══════════════════════════════════════════════");
    println!("  All strings tuned! Building calibration profile...");
    println!("══════════════════════════════════════════════\n");

    let mut profile = GuitarCalibrationProfile::default();

    for string_idx in 0..6 {
        let mut plucks: Vec<&Pluck> = all_plucks
            .iter()
            .filter(|(si, _)| *si == string_idx)
            .map(|(_, p)| p)
            .collect();

        if plucks.is_empty() {
            continue;
        }

        plucks.sort_by(|a, b| a.rms.partial_cmp(&b.rms).unwrap());
        let split = plucks.len() / 2;
        let soft = &plucks[..split.max(1)];
        let strong = &plucks[split..];

        profile.strings[string_idx].soft_samples = soft.iter().map(|p| to_cal(p)).collect();
        profile.strings[string_idx].strong_samples = strong.iter().map(|p| to_cal(p)).collect();
    }

    // Summary
    println!(
        "  {:>6}  {:>5}  {:>8}  {:>8}  {:>6}",
        "String", "Count", "SoftRMS", "StrongRMS", "Conf"
    );
    println!("  {}", "-".repeat(42));

    for (i, cal) in profile.strings.iter().enumerate() {
        let count = all_plucks.iter().filter(|(si, _)| *si == i).count();
        let strong_rms = if cal.strong_samples.is_empty() {
            0.0
        } else {
            cal.strong_samples.iter().map(|s| s.rms).sum::<f32>() / cal.strong_samples.len() as f32
        };
        println!(
            "  {:>6}  {:>5}  {:>8.4}  {:>8.4}  {:>5.1}%",
            STRING_NAMES[i],
            count,
            cal.soft_rms_threshold(),
            strong_rms,
            cal.avg_confidence() * 100.0
        );
    }

    let json = profile.to_json().expect("Failed to serialize");
    std::fs::write(PROFILE_PATH, &json).expect("Failed to write");
    println!(
        "\n  Saved: {} ({} plucks, {} bytes)",
        PROFILE_PATH,
        all_plucks.len(),
        json.len()
    );
    println!("\n  Now run:");
    println!("    cargo run --release --example guitar_harmony");
}

/// Tune a single string. Returns number of plucks captured.
fn tune_string(
    string_idx: usize,
    step: usize,
    total: usize,
    pass_idx: usize,
    state: &Arc<Mutex<AudioState>>,
    noise_floor: f32,
    pluck_threshold: f32,
    all_plucks: &mut Vec<(usize, Pluck)>,
) -> usize {
    let string_name = STRING_NAMES[string_idx];
    let target_midi = STRING_BASE_PITCH[string_idx];
    let target_note = midi_to_note_name(target_midi);
    let target_freq = midi_to_freq(target_midi);

    let pass_label = if pass_idx == 2 { "verify" } else { "tune" };

    println!(
        "  ── {}/{}: {} {} ({}, {:.1} Hz) ──",
        step, total, pass_label, string_name, target_note, target_freq
    );

    let mut in_tune_since: Option<Instant> = None;
    let mut string_plucks: Vec<Pluck> = Vec::new();
    let mut was_quiet = true;
    let mut last_pluck_time = Instant::now();

    // Final pass has shorter hold time since strings should be stable
    let hold_secs = if pass_idx == 2 {
        IN_TUNE_HOLD_SECS * 0.6
    } else {
        IN_TUNE_HOLD_SECS
    };

    loop {
        std::thread::sleep(Duration::from_millis(40));

        let s = state.lock().unwrap();

        // Auto-capture plucks
        if s.rms < pluck_threshold * 0.4 {
            was_quiet = true;
        }

        if was_quiet
            && s.rms > pluck_threshold
            && s.confidence > MIN_CONFIDENCE as f32
            && last_pluck_time.elapsed() > Duration::from_millis(200)
        {
            was_quiet = false;
            last_pluck_time = Instant::now();
            let dist = (s.midi_note as i16 - target_midi as i16).abs();
            if dist <= 12 {
                string_plucks.push(Pluck {
                    midi_note: s.midi_note,
                    freq: s.frequency,
                    conf: s.confidence,
                    rms: s.rms,
                    peak: s.peak,
                    bright_rms: s.bright_rms,
                    bright_peak: s.bright_peak,
                    slope: s.slope,
                });
            }
        }

        // Tuner display
        if s.rms < noise_floor * 2.0 || s.confidence < MIN_CONFIDENCE as f32 {
            in_tune_since = None;
            print!(
                "\r    {:>5} {:>5} {:>6} {:>4}%  {:21}  {:20}",
                "---", "", "", "", "", "pluck..."
            );
            std::io::stdout().flush().unwrap();
            continue;
        }

        let dist = (s.midi_note as i16 - target_midi as i16).abs();
        let is_target = dist == 0 || dist == 12;
        let cents = if dist == 0 { s.cents } else { 50i8 };
        let meter = build_meter(cents);

        let status = if !is_target {
            in_tune_since = None;
            format!("{} — play {}", s.note_name, target_note)
        } else if cents.abs() <= IN_TUNE_CENTS {
            if in_tune_since.is_none() {
                in_tune_since = Some(Instant::now());
            }
            let held_ms = in_tune_since.unwrap().elapsed().as_millis() as f32;
            let hold_ms = hold_secs * 1000.0;
            let progress = (held_ms / hold_ms).min(1.0);
            let filled = (progress * 10.0) as usize;
            let bar = "█".repeat(filled) + &"░".repeat(10 - filled);

            if held_ms >= hold_ms {
                // Done — store plucks and return
                let count = string_plucks.len();
                for p in string_plucks {
                    all_plucks.push((string_idx, p));
                }
                let cents_str = if s.cents >= 0 {
                    format!("+{}¢", s.cents)
                } else {
                    format!("{}¢", s.cents)
                };
                print!(
                    "\r    {:>5} {:>5} {:>6.1} {:>4.0}%  {}  IN TUNE [{}]   ",
                    s.note_name,
                    cents_str,
                    s.frequency,
                    s.confidence * 100.0,
                    meter,
                    bar
                );
                std::io::stdout().flush().unwrap();
                return count;
            }
            format!("hold [{}] x{}", bar, string_plucks.len())
        } else {
            in_tune_since = None;
            if cents > 0 {
                "sharp ↓".into()
            } else {
                "flat ↑".into()
            }
        };

        let cents_str = if is_target {
            if s.cents >= 0 {
                format!("+{}¢", s.cents)
            } else {
                format!("{}¢", s.cents)
            }
        } else {
            "".into()
        };

        print!(
            "\r    {:>5} {:>5} {:>6.1} {:>4.0}%  {}  {}   ",
            s.note_name,
            cents_str,
            s.frequency,
            s.confidence * 100.0,
            meter,
            status
        );
        std::io::stdout().flush().unwrap();
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
        bright_ratio: if p.bright_rms > 0.0 {
            p.bright_peak / p.bright_rms
        } else {
            0.0
        },
        bright_slope: 0.0,
    }
}

// ── Audio Processing ─────────────────────────────────────────────────

fn audio_process(
    data: &[f32],
    channels: usize,
    target_ch: usize,
    sample_rate: usize,
    overlap: &Arc<Mutex<OverlapManager>>,
    pluck_det: &Arc<Mutex<PluckDetector>>,
    goertzel: &Arc<Mutex<GoertzelBank>>,
    state: &Arc<Mutex<AudioState>>,
) {
    let mono: Vec<f32> = data
        .chunks(channels)
        .map(|f| f.get(target_ch).copied().unwrap_or(0.0))
        .collect();

    let frames = { overlap.lock().unwrap().feed(&mono) };

    for frame in frames {
        let rms = (frame.iter().map(|s| s * s).sum::<f32>() / frame.len() as f32).sqrt();
        let peak = frame.iter().map(|s| s.abs()).fold(0.0f32, f32::max);
        let half = frame.len() / 2;
        let bright_rms = (frame[half..].iter().map(|s| s * s).sum::<f32>() / half as f32).sqrt();
        let bright_peak = frame[half..].iter().map(|s| s.abs()).fold(0.0f32, f32::max);

        let g_note = { goertzel.lock().unwrap().analyze(&frame).map(|(m, _)| m) };

        let samples_f64: Vec<f64> = frame.iter().map(|&s| s as f64).collect();
        let mut det = McLeodDetector::new(frame.len(), frame.len() / 2);
        let pitch = det.get_pitch(&samples_f64, sample_rate, 0.3, 0.3);

        let mut s = state.lock().unwrap();
        let slope = rms - s.prev_rms;
        s.rms = rms;
        s.peak = peak;
        s.bright_rms = bright_rms;
        s.bright_peak = bright_peak;
        s.slope = slope;
        s.prev_rms = rms;
        s.goertzel_note = g_note;

        if let Some(p) = pitch {
            if p.clarity > MIN_CONFIDENCE {
                let freq = p.frequency as f32;
                let (midi, cents) = freq_to_midi(freq);
                s.frequency = freq;
                s.confidence = p.clarity as f32;
                s.midi_note = midi;
                s.cents = cents;
                s.note_name = midi_to_note_name(midi);
            }
        }
    }
}

// ── Utilities ────────────────────────────────────────────────────────

fn prompt(p: &str, max: usize) -> usize {
    print!("{}", p);
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
    let ch = if cents.abs() <= 5 {
        '*'
    } else if cents.abs() <= 15 {
        '='
    } else {
        '#'
    };
    bar[pos] = ch;
    format!("[{}]", bar.iter().collect::<String>())
}
