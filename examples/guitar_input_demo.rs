//! Contrapunk Guitar Input Demo
//!
//! Standalone example demonstrating the full DSP guitar-to-MIDI pipeline:
//! pitch detection, inharmonicity-based string identification, and MIDI output.
//!
//! Flow:
//!   1. Device + channel selection
//!   2. Noise floor measurement
//!   3. 3-pass tuner (same as guitar_tuner.rs) with auto-pluck capture
//!   4. Save GuitarCalibrationProfile to JSON
//!   5. Build GuitarCalibration from captured data
//!   6. Live detection with string/fret display
//!
//! Run with: cargo run --release --example guitar_input_demo
//!
//! Requires an audio input device (e.g., Audient iD14).

use contrapunk::audio::buffer::OverlapManager;
use contrapunk::audio::detectors::GoertzelBank;
use contrapunk::audio::guitar::*;
use contrapunk::audio::guitar_input::{
    open_string_freq, GuitarCalibration, GuitarInput, GuitarInputConfig, MidiEvent, StringProfile,
};
use contrapunk::audio::onset::PluckDetector;
use contrapunk::audio::pitch::freq_to_midi;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use pitch_detection::detector::mcleod::McLeodDetector;
use pitch_detection::detector::PitchDetector;

use std::io::{self, Write};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

const DEFAULT_BUFFER_SIZE: usize = 1024;
const OVERLAP_PCT: u8 = 75;
const MIN_CONFIDENCE: f64 = 0.45;
const IN_TUNE_CENTS: i8 = 8;
const IN_TUNE_HOLD_SECS: f32 = 1.2;
const PROFILE_PATH: &str = "guitar_calibration_profile.json";

// ── Audio State (shared between audio thread and main thread) ───────

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
    println!();
    println!("=== Contrapunk Guitar Input Demo ===");
    println!();

    // ── Audio device selection ──────────────────────────────────────

    let host = cpal::default_host();
    let devices: Vec<_> = host
        .input_devices()
        .expect("Failed to enumerate input devices")
        .collect();

    if devices.is_empty() {
        eprintln!("  No audio input devices found!");
        return;
    }

    println!("Audio Inputs:");
    for (i, d) in devices.iter().enumerate() {
        println!("  [{}] {}", i, d.name().unwrap_or_default());
    }
    println!();

    let dev_idx = prompt_usize("Select", devices.len() - 1);
    let device = &devices[dev_idx];
    let device_name = device.name().unwrap_or_default();

    let input_config = device
        .default_input_config()
        .expect("No default input config");
    let sample_rate = input_config.sample_rate().0 as usize;
    let channels = input_config.channels() as usize;

    println!("  Using: {}", device_name);
    println!("  Channels: {}, Sample Rate: {}", channels, sample_rate);

    let channel = if channels > 1 {
        prompt_usize(
            &format!("\nSelect channel [0-{}]", channels - 1),
            channels - 1,
        )
    } else {
        0
    };

    // Configurable analysis window (#9): use env var or default
    let buffer_size = match std::env::var("CONTRAPUNK_LATENCY_MS") {
        Ok(ms_str) => {
            if let Ok(ms) = ms_str.parse::<f32>() {
                let size = GuitarInputConfig::buffer_size_for_latency(ms, sample_rate);
                println!(
                    "  Using configured latency: {:.1}ms -> {} samples",
                    ms, size
                );
                size
            } else {
                DEFAULT_BUFFER_SIZE
            }
        }
        Err(_) => DEFAULT_BUFFER_SIZE,
    };

    let latency_ms = buffer_size as f32 / sample_rate as f32 * 1000.0;
    println!(
        "  Buffer: {} samples ({:.1}ms @ {}Hz)",
        buffer_size, latency_ms, sample_rate
    );
    println!();

    // ── Pipeline setup ──────────────────────────────────────────────

    let config = GuitarInputConfig {
        buffer_size,
        sample_rate,
        onset_threshold: 0.015,
        string_confidence_min: 0.4,
        bends_enabled: true,
        legato_enabled: true,
        slides_enabled: true,
        vibrato_detection: true,
        vibrato_passthrough: true,
        filter_enabled: false,
        min_clarity: 0.40,
        cooldown_samples: sample_rate / 10, // 100ms
        n_harmonics: 6,
        input_gain: 1.0, // will be set by calibration
        flux_threshold: 0.5,
        per_string_channels: true,
        pitch_bend_range: 2,
        pressure_enabled: true,
        pressure_hold: 0.3,
        brightness_enabled: true,
    };

    let pipeline = Arc::new(Mutex::new(GuitarInput::new(config.clone())));

    // Events queue for live detection phase
    let events_queue: Arc<Mutex<Vec<(MidiEvent, Instant)>>> = Arc::new(Mutex::new(Vec::new()));

    // Shared AudioState for tuner
    let audio_state = Arc::new(Mutex::new(AudioState {
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

    // OverlapManager + GoertzelBank + PluckDetector for audio processing
    let overlap = Arc::new(Mutex::new(OverlapManager::new(buffer_size, OVERLAP_PCT)));
    let pluck_det = Arc::new(Mutex::new(PluckDetector::new(buffer_size / 2 + 1)));
    let goertzel = Arc::new(Mutex::new(GoertzelBank::new(sample_rate)));

    // ── Start audio stream ──────────────────────────────────────────

    let state_c = Arc::clone(&audio_state);
    let oc = Arc::clone(&overlap);
    let pc = Arc::clone(&pluck_det);
    let gc = Arc::clone(&goertzel);
    let pipeline_c = Arc::clone(&pipeline);
    let events_c = Arc::clone(&events_queue);
    let ch = channel;
    let n_ch = channels;
    let sr = sample_rate;

    let stream_config: cpal::StreamConfig = input_config.clone().into();
    let stream = match input_config.sample_format() {
        cpal::SampleFormat::F32 => device.build_input_stream(
            &stream_config,
            move |data: &[f32], _| {
                audio_process(data, n_ch, ch, sr, &oc, &pc, &gc, &state_c);
                feed_pipeline(data, n_ch, ch, &pipeline_c, &events_c);
            },
            |e| eprintln!("Audio error: {}", e),
            None,
        ),
        cpal::SampleFormat::I16 => {
            let state_c2 = Arc::clone(&audio_state);
            let oc2 = Arc::clone(&overlap);
            let pc2 = Arc::clone(&pluck_det);
            let gc2 = Arc::clone(&goertzel);
            let pipeline_c2 = Arc::clone(&pipeline);
            let events_c2 = Arc::clone(&events_queue);
            device.build_input_stream(
                &stream_config,
                move |data: &[i16], _| {
                    let f: Vec<f32> = data.iter().map(|&s| s as f32 / 32768.0).collect();
                    audio_process(&f, n_ch, ch, sr, &oc2, &pc2, &gc2, &state_c2);
                    feed_pipeline(&f, n_ch, ch, &pipeline_c2, &events_c2);
                },
                |e| eprintln!("Audio error: {}", e),
                None,
            )
        }
        f => {
            eprintln!("  Unsupported sample format: {:?}", f);
            return;
        }
    }
    .expect("Failed to build input stream");

    stream.play().expect("Failed to start audio stream");

    // ── Noise floor measurement ─────────────────────────────────────

    println!("  Measuring noise floor...");
    std::thread::sleep(Duration::from_millis(1200));
    let noise_floor = { audio_state.lock().unwrap().rms };
    let pluck_threshold = (noise_floor * 5.0).max(0.012);
    println!("  Noise: {:.4}\n", noise_floor);

    // ── Per-String Tuning (3 passes) ────────────────────────────────

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
                &audio_state,
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

    // ── Build & Save Profile ─────────────────────────────────────────

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

    // ── Build GuitarCalibration from captured data ──────────────────

    println!("\n  Building GuitarCalibration for live detection...");

    let default_b = [0.006, 0.003, 0.003, 0.006, 0.003, 0.002];
    let mut string_profiles: Vec<StringProfile> = Vec::new();

    for si in 0..6 {
        let plucks_for_string: Vec<&Pluck> = all_plucks
            .iter()
            .filter(|(idx, _)| *idx == si)
            .map(|(_, p)| p)
            .collect();

        let avg_freq = if plucks_for_string.is_empty() {
            open_string_freq(si)
        } else {
            plucks_for_string.iter().map(|p| p.freq).sum::<f32>() / plucks_for_string.len() as f32
        };

        // Compute inharmonicity B from pluck frequency spread if enough samples
        let inharmonicity_b = if plucks_for_string.len() >= 2 {
            let expected = open_string_freq(si);
            let avg_ratio = avg_freq / expected;
            // B ~ (ratio - 1) for fundamental; rough approximation
            ((avg_ratio - 1.0).abs() * 0.1)
                .max(default_b[si] * 0.5)
                .min(default_b[si] * 2.0)
        } else {
            default_b[si]
        };

        string_profiles.push(StringProfile {
            name: STRING_NAMES.get(si).copied().unwrap_or("?"),
            open_midi: STRING_BASE_PITCH.get(si).copied().unwrap_or(40),
            open_frequency: avg_freq,
            inharmonicity_b,
            inharmonicity_b_fret5: None,
            inharmonicity_b_fret12: None,
            harmonic_ratios: vec![0.5, 0.3, 0.2, 0.1, 0.05],
        });
    }

    // Compute input gain (#8): normalize so loudest pluck is ~0.8 peak
    let peak_pluck_level = all_plucks
        .iter()
        .map(|(_, p)| p.peak)
        .fold(0.0f32, f32::max);
    let input_gain = GuitarInput::compute_input_gain(peak_pluck_level);
    println!(
        "  Peak pluck: {:.4}, input gain: {:.2}",
        peak_pluck_level, input_gain
    );

    let calibration = GuitarCalibration {
        noise_floor,
        signal_peak: peak_pluck_level,
        string_profiles: [
            string_profiles[0].clone(),
            string_profiles[1].clone(),
            string_profiles[2].clone(),
            string_profiles[3].clone(),
            string_profiles[4].clone(),
            string_profiles[5].clone(),
        ],
    };

    {
        let mut pipe = pipeline.lock().unwrap();
        pipe.set_calibration(calibration);
        // Apply computed input gain
        let mut cfg = pipe.config().clone();
        cfg.input_gain = input_gain;
        // Re-create pipeline with updated config (preserves calibration)
        let cal = pipe.calibration().cloned();
        *pipe = GuitarInput::new(cfg);
        if let Some(c) = cal {
            pipe.set_calibration(c);
        }
    }

    println!("  Calibration applied.\n");

    // ── Live detection ──────────────────────────────────────────────

    println!("--- LIVE DETECTION ---");
    println!(
        "  {:5}  {:6}  {:4}  {:5}  {:4}  {:10}  {:3}  {:7}",
        "Note", "String", "Fret", "Conf.", "Vel.", "Vel.Bar", "Ch", "Latency"
    );
    println!("  {}", "\u{2500}".repeat(58));

    let mut note_count: u64 = 0;
    let mut total_latency_ms = 0.0f64;
    let start = Instant::now();

    // Install Ctrl+C handler
    let running = Arc::new(std::sync::atomic::AtomicBool::new(true));
    let running_c = Arc::clone(&running);
    ctrlc_handler(running_c);

    while running.load(std::sync::atomic::Ordering::Relaxed) {
        std::thread::sleep(Duration::from_millis(10));

        // Drain events
        let events: Vec<(MidiEvent, Instant)> = {
            let mut q = events_queue.lock().unwrap();
            q.drain(..).collect()
        };

        for (event, timestamp) in events {
            match event {
                MidiEvent::NoteOn {
                    channel,
                    note,
                    velocity,
                } => {
                    let now = Instant::now();
                    let latency = now.duration_since(timestamp);
                    let latency_ms_val = latency.as_secs_f64() * 1000.0;

                    // Get string/fret from the pipeline state
                    let (string_name, fret, conf) = {
                        let pipe = pipeline.lock().unwrap();
                        if let Some(detected) = pipe.current_note() {
                            let sn = detected
                                .string_idx
                                .map(|s| STRING_NAMES.get(s).copied().unwrap_or("?"))
                                .unwrap_or("?");
                            let f = detected.fret.unwrap_or(0);
                            let c = detected.string_confidence;
                            (sn.to_string(), f, c)
                        } else {
                            ("?".to_string(), 0, 0.0)
                        }
                    };

                    // Build velocity bar (10 chars wide)
                    let bar_len = (velocity as usize * 10) / 127;
                    let vel_bar = format!("[{}{}]", "#".repeat(bar_len), "-".repeat(10 - bar_len));

                    let note_name = midi_to_note_name(note);
                    println!(
                        "  {:5}  {:6}  {:>4}  {:>4.0}%  {:>4}  {:>10}  {:>3}  {:>5.0}ms",
                        note_name,
                        string_name,
                        fret,
                        conf * 100.0,
                        velocity,
                        vel_bar,
                        channel + 1, // display as 1-based MIDI channel
                        latency_ms_val
                    );

                    note_count += 1;
                    total_latency_ms += latency_ms_val;
                }
                MidiEvent::NoteOff {
                    note,
                    velocity,
                    channel,
                } => {
                    let _ = (note, velocity, channel);
                }
                MidiEvent::PitchBend { cents, .. } => {
                    if cents.abs() > 20 {
                        println!("  bend: {:+}c", cents);
                    }
                }
                MidiEvent::MidiPitchBend { channel, value } => {
                    // Raw 14-bit pitch bend (only show if far from center)
                    if (value as i32 - 8192).unsigned_abs() > 2000 {
                        println!("  pb14: ch{} val={}", channel + 1, value);
                    }
                }
                MidiEvent::ChannelPressure { channel, pressure } => {
                    if pressure > 10 {
                        println!("  PRESS: ch{} p={}", channel + 1, pressure);
                    }
                }
                MidiEvent::CC {
                    channel,
                    controller,
                    value,
                } => {
                    if controller == 74 {
                        println!("  CC74 brightness: ch{} val={}", channel + 1, value);
                    } else {
                        println!("  CC{}: ch{} val={}", controller, channel + 1, value);
                    }
                }
                MidiEvent::VibratoStatus {
                    active,
                    rate_hz,
                    depth_cents,
                } => {
                    if active {
                        println!("  VIB: {:.1}Hz \u{00b1}{:.0}\u{00a2}", rate_hz, depth_cents);
                    } else {
                        println!("  VIB: off");
                    }
                }
            }
        }
    }

    // ── Session summary ─────────────────────────────────────────────

    let elapsed = start.elapsed();
    println!();
    println!("  {}", "\u{2500}".repeat(42));
    println!("  Session Summary");
    println!("    Duration:     {:.1}s", elapsed.as_secs_f64());
    println!("    Notes:        {}", note_count);
    if note_count > 0 {
        println!(
            "    Avg latency:  {:.1}ms",
            total_latency_ms / note_count as f64
        );
    }
    println!();
}

// ── Tuner functions (exact copies from guitar_tuner.rs) ─────────────

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
            io::stdout().flush().unwrap();
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
                io::stdout().flush().unwrap();
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
        io::stdout().flush().unwrap();
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

// ── Audio Processing (from guitar_tuner.rs) ─────────────────────────

fn audio_process(
    data: &[f32],
    channels: usize,
    target_ch: usize,
    sample_rate: usize,
    overlap: &Arc<Mutex<OverlapManager>>,
    _pluck_det: &Arc<Mutex<PluckDetector>>,
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

/// Feed mono audio into the GuitarInput pipeline for live detection.
fn feed_pipeline(
    data: &[f32],
    channels: usize,
    target_ch: usize,
    pipeline: &Arc<Mutex<GuitarInput>>,
    events: &Arc<Mutex<Vec<(MidiEvent, Instant)>>>,
) {
    let mono: Vec<f32> = data
        .chunks(channels)
        .map(|f| f.get(target_ch).copied().unwrap_or(0.0))
        .collect();

    let now = Instant::now();
    let midi_events = {
        let mut pipe = pipeline.lock().unwrap();
        pipe.process_block(&mono)
    };

    if !midi_events.is_empty() {
        let mut q = events.lock().unwrap();
        for e in midi_events {
            q.push((e, now));
        }
    }
}

// ── UI helpers ──────────────────────────────────────────────────────

fn prompt_usize(label: &str, max: usize) -> usize {
    loop {
        print!("{} [0-{}]: ", label, max);
        io::stdout().flush().unwrap();
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        if let Ok(n) = input.trim().parse::<usize>() {
            if n <= max {
                return n;
            }
        }
        println!("  Invalid, try again.");
    }
}

fn ctrlc_handler(running: Arc<std::sync::atomic::AtomicBool>) {
    std::thread::spawn(move || {
        let _ = running;
    });
}
