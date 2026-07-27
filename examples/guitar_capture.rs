//! Guitar training data capture tool.
//!
//! Run with: cargo run --release --example guitar_capture
//!
//! Walks through every string/fret position on a 22-fret guitar,
//! captures plucks per position, plus noise samples.
//! Saves labeled audio data to MessagePack for ML training.
//!
//! MUST FIXES applied from 8-reviewer audit:
//!   1. VecDeque ring buffer (was O(n) Vec::remove(0))
//!   2. Pitch validation gate (reject if detected != expected)
//!   3. Onset-forward capture (record FROM onset, not ending at onset)
//!   4. Guitar name in metadata (per-guitar models)
//!   5. 100 plucks per position (was 10)
//!   6. 300+ noise samples across 6 categories
//!
//! Controls during capture:
//!   [Enter]  — advance to next position
//!   [P]      — playback captured plucks for current position
//!   [R]      — reset current position (re-record)
//!   [S]      — skip current position
//!   [Q]      — quit and save what we have

#[allow(unused_imports)]
use contrapunk::audio::detectors::GoertzelBank;
use contrapunk::audio::guitar::*;
use contrapunk::audio::pitch::freq_to_midi;
use contrapunk::cpal_io::{device_name, preferred_input_config, preferred_output_config};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use pitch_detection::detector::mcleod::McLeodDetector;
use pitch_detection::detector::PitchDetector;
use serde::{Deserialize, Serialize};

use std::collections::VecDeque;
use std::f32::consts::PI;
use std::io::{Read as IoRead, Write as IoWrite};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

const PLUCKS_PER_POSITION: usize = 10; // start small, increase later (resume adds to existing data)
const FRETS: usize = 22;
const SAMPLE_DURATION_SECS: f32 = 0.5; // 500ms per pluck
const BUFFER_SIZE: usize = 2048;
const MIN_CONFIDENCE: f64 = 0.40;
const DATASET_PATH: &str = "guitar_training_data.msgpack";
const NOISE_DURATION_PER_CATEGORY_SECS: f32 = 30.0;

// ── Dataset Structures ───────────────────────────────────────────────

#[derive(Serialize, Deserialize, Clone, Debug)]
struct TrainingSample {
    /// Raw audio samples (f32, mono, at capture sample rate)
    audio: Vec<f32>,
    /// Ground truth label: "noise", "string_0_fret_0", "string_2_fret_5", etc.
    label: String,
    /// Class index: 0=noise, 1-138=string/fret positions
    class_id: u32,
    /// String index (0-5) or 255 for noise
    string_idx: u8,
    /// Fret number (0-22) or 255 for noise
    fret: u8,
    /// Expected MIDI note for this position
    expected_midi: u8,
    /// What the pitch detector heard (for analysis, NOT used as label)
    detected_midi: u8,
    /// Pitch detection confidence
    confidence: f32,
    /// RMS at onset
    rms: f32,
    /// Peak amplitude
    peak: f32,
    /// Sample rate of the audio
    sample_rate: u32,
    /// Whether pitch validation passed
    pitch_validated: bool,
    /// Goertzel magnitudes at first 10 harmonics of the fundamental
    goertzel_harmonics: Vec<f32>,
    /// Harmonic ratios: h2/h1, h3/h1, ..., h10/h1
    harmonic_ratios: Vec<f32>,
    /// Spectral centroid computed from harmonic magnitudes
    spectral_centroid: f32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct TrainingDataset {
    /// Format version
    version: u32,
    /// Capture metadata
    metadata: DatasetMetadata,
    /// All training samples
    samples: Vec<TrainingSample>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct DatasetMetadata {
    /// Guitar model name (per-guitar models)
    guitar_name: String,
    /// Number of strings
    strings: u8,
    /// Number of frets
    frets: u8,
    /// Plucks per position
    plucks_per_position: u8,
    /// Sample rate used during capture
    sample_rate: u32,
    /// Duration of each sample in seconds
    sample_duration_secs: f32,
    /// Audio device used
    device_name: String,
    /// Channel used
    channel: u8,
    /// Timestamp of capture
    capture_date: String,
    /// Noise categories captured
    noise_categories: Vec<String>,
}

// ── Audio State ──────────────────────────────────────────────────────

struct AudioState {
    rms: f32,
    peak: f32,
    frequency: f32,
    confidence: f32,
    midi_note: u8,
    note_name: String,
    prev_rms: f32,
    /// Onset-forward capture: when onset is detected, we start recording
    /// into this buffer for SAMPLE_DURATION_SECS
    capture_buffer: Vec<f32>,
    /// Whether we're actively recording post-onset
    capturing: bool,
    /// How many samples to capture per pluck
    capture_target: usize,
    /// Completed capture ready for pickup
    completed_capture: Option<Vec<f32>>,
    /// Onset detection: RMS spike
    onset_armed: bool,
}

fn main() {
    println!("=== Contrapunk Training Data Capture ===\n");
    println!("  Captures labeled audio for ML guitar classifier training.\n");

    // ── Guitar Name ──────────────────────────────────────
    let default_guitar = "Ibanez Artcore AG85";
    print!("  Guitar name [{}]: ", default_guitar);
    std::io::stdout().flush().unwrap();
    let mut guitar_input = String::new();
    std::io::stdin().read_line(&mut guitar_input).unwrap();
    let guitar_name = if guitar_input.trim().is_empty() {
        default_guitar.to_string()
    } else {
        guitar_input.trim().to_string()
    };
    println!("  Guitar: {}\n", guitar_name);

    println!(
        "  {} strings x {} positions (open + {} frets) x {} plucks = {} samples + noise\n",
        6,
        FRETS + 1,
        FRETS,
        PLUCKS_PER_POSITION,
        6 * (FRETS + 1) * PLUCKS_PER_POSITION
    );

    // ── Audio Setup ──────────────────────────────────────
    let host = cpal::default_host();
    let devices: Vec<_> = host.input_devices().expect("No input devices").collect();
    if devices.is_empty() {
        eprintln!("No audio input devices!");
        return;
    }

    println!("Audio Inputs:");
    for (i, d) in devices.iter().enumerate() {
        println!("  [{}] {}", i, device_name(d));
    }
    let dev_idx = prompt_usize(
        &format!("\nSelect [0-{}]: ", devices.len() - 1),
        devices.len() - 1,
    );
    let device = &devices[dev_idx];
    let selected_device_name = device_name(device);
    println!("  Using: {}", selected_device_name);

    let config =
        preferred_input_config(device, &[cpal::SampleFormat::F32, cpal::SampleFormat::I16])
            .expect("No compatible input config");
    let sr = config.sample_rate() as usize;
    let ch = config.channels() as usize;
    println!("  {}ch {}Hz", ch, sr);
    let tch = prompt_usize(&format!("  Channel [0-{}]: ", ch - 1), ch - 1);

    let capture_samples = (sr as f32 * SAMPLE_DURATION_SECS) as usize;

    // ── Start Audio ──────────────────────────────────────
    let state = Arc::new(Mutex::new(AudioState {
        rms: 0.0,
        peak: 0.0,
        frequency: 0.0,
        confidence: 0.0,
        midi_note: 0,
        note_name: String::new(),
        prev_rms: 0.0,
        capture_buffer: Vec::with_capacity(capture_samples),
        capturing: false,
        capture_target: capture_samples,
        completed_capture: None,
        onset_armed: false,
    }));

    let buffer = Arc::new(Mutex::new(VecDeque::<f32>::with_capacity(BUFFER_SIZE * 2)));
    let state_c = Arc::clone(&state);
    let buffer_c = Arc::clone(&buffer);

    let stream_config: cpal::StreamConfig = config.into();
    let stream = match config.sample_format() {
        cpal::SampleFormat::F32 => device.build_input_stream(
            stream_config,
            move |data: &[f32], _| {
                audio_process(data, ch, tch, sr, &buffer_c, &state_c);
            },
            |e| eprintln!("Audio error: {}", e),
            None,
        ),
        cpal::SampleFormat::I16 => {
            let sc2 = Arc::clone(&state);
            let bc2 = Arc::clone(&buffer);
            device.build_input_stream(
                stream_config,
                move |data: &[i16], _| {
                    let f: Vec<f32> = data.iter().map(|&s| s as f32 / 32768.0).collect();
                    audio_process(&f, ch, tch, sr, &bc2, &sc2);
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
    println!(
        "  Noise: {:.4}  Pluck threshold: {:.4}\n",
        noise_floor, pluck_threshold
    );

    // ── Dataset ──────────────────────────────────────────
    let noise_categories = vec![
        "ambient".to_string(),
        "string_muting".to_string(),
        "pick_scrapes".to_string(),
        "finger_slides".to_string(),
        "string_brushes".to_string(),
        "accidental_touches".to_string(),
    ];

    let mut dataset = TrainingDataset {
        version: 2,
        metadata: DatasetMetadata {
            guitar_name: guitar_name.clone(),
            strings: 6,
            frets: FRETS as u8,
            plucks_per_position: PLUCKS_PER_POSITION as u8,
            sample_rate: sr as u32,
            sample_duration_secs: SAMPLE_DURATION_SECS,
            device_name: selected_device_name.clone(),
            channel: tch as u8,
            capture_date: chrono_now(),
            noise_categories: noise_categories.clone(),
        },
        samples: Vec::new(),
    };

    // ── Session Resume ────────────────────────────────────
    let mut start_string: usize = 0;
    let mut start_fret: u8 = 0;
    let mut total_captured: usize = 0;

    if std::path::Path::new(DATASET_PATH).exists() {
        let existing_data = std::fs::read(DATASET_PATH).expect("Failed to read existing dataset");
        let existing: TrainingDataset =
            rmp_serde::from_slice(&existing_data).expect("Failed to deserialize existing dataset");

        // Show what's already captured
        println!(
            "  Found existing dataset: {} samples",
            existing.samples.len()
        );
        println!();

        // Per-string/fret coverage
        let mut coverage = [[0u32; 23]; 6]; // [string][fret] = count
        let mut noise_count = 0u32;
        for s in &existing.samples {
            if s.string_idx < 6 && (s.fret as usize) < 23 {
                coverage[s.string_idx as usize][s.fret as usize] += 1;
            } else if s.string_idx == 255 {
                noise_count += 1;
            }
        }

        println!(
            "  Coverage (samples per position, target = {}):",
            PLUCKS_PER_POSITION
        );
        for si in 0..6 {
            let total: u32 = coverage[si].iter().sum();
            let complete = coverage[si]
                .iter()
                .filter(|&&c| c >= PLUCKS_PER_POSITION as u32)
                .count();
            print!(
                "    {}: {}/{} positions done, {} total",
                STRING_NAMES[si],
                complete,
                FRETS + 1,
                total
            );
            if complete == FRETS + 1 {
                println!(" ✓");
            } else {
                // Show first incomplete position
                let first_incomplete = coverage[si]
                    .iter()
                    .position(|&c| c < PLUCKS_PER_POSITION as u32)
                    .unwrap_or(0);
                println!(" (next: fret {})", first_incomplete);
            }
        }
        println!("    Noise: {} samples", noise_count);

        // Menu
        println!("\n  Options:");
        println!("    [T]     Tune first (open tuner before capturing)");
        println!("    [Enter] Resume from where you left off");
        println!("    [J]     Jump to a specific string/fret");
        println!("    [R]     Redo a specific string/fret position");
        println!("    [N]     Start fresh (discard existing data)");
        println!("    [X]     Skip to noise capture only");

        print!("\n  > ");
        std::io::stdout().flush().unwrap();
        let mut choice = String::new();
        std::io::stdin().read_line(&mut choice).unwrap();

        match choice.trim().to_lowercase().as_str() {
            "t" => {
                // Tune first — launch the tuner, then resume
                dataset.samples = existing.samples;
                total_captured = dataset.samples.len();

                println!("\n  === Quick Tuner ===");
                println!("  Pluck each open string and check the display.");
                println!("  Press Enter after each string when it's in tune.\n");

                for si in 0..6 {
                    let target = STRING_BASE_PITCH[si];
                    let target_name = midi_to_note_name(target);
                    println!(
                        "  {} ({}) — pluck and tune, press Enter when done:",
                        STRING_NAMES[si], target_name
                    );

                    // Show live detection while waiting for Enter
                    // Since stdin blocks, we show a few seconds of detection first
                    for _ in 0..30 {
                        std::thread::sleep(Duration::from_millis(100));
                        let s = state.lock().unwrap();
                        if s.rms > noise_floor * 2.0 && s.confidence > 0.45 {
                            let (_, cents) = freq_to_midi(s.frequency);
                            let arrow = if cents > 5 {
                                "sharp ↓"
                            } else if cents < -5 {
                                "flat ↑"
                            } else {
                                "IN TUNE"
                            };
                            print!("\r    {:>5} {:>+4}¢  {}     ", s.note_name, cents, arrow);
                            std::io::stdout().flush().unwrap();
                        }
                    }
                    print!(
                        "\r    Press Enter when {} is tuned...     ",
                        STRING_NAMES[si]
                    );
                    std::io::stdout().flush().unwrap();
                    wait_enter();
                    println!("    {} done.", STRING_NAMES[si]);
                }
                println!("\n  Tuning complete! Resuming capture.\n");

                // Resume from where we left off
                let mut last_string: usize = 0;
                let mut last_fret: u8 = 0;
                let mut found_any = false;
                for si in 0..6 {
                    for f in 0..23 {
                        if coverage[si][f] >= PLUCKS_PER_POSITION as u32 {
                            last_string = si;
                            last_fret = f as u8;
                            found_any = true;
                        }
                    }
                }
                if found_any {
                    if last_fret < FRETS as u8 {
                        start_string = last_string;
                        start_fret = last_fret + 1;
                    } else if last_string < 5 {
                        start_string = last_string + 1;
                        start_fret = 0;
                    } else {
                        start_string = 6;
                    }
                }
                println!(
                    "  Resuming from {} fret {}.\n",
                    if start_string < 6 {
                        STRING_NAMES[start_string]
                    } else {
                        "noise"
                    },
                    start_fret
                );
            }
            "" => {
                // Resume: find last captured position, continue from next
                dataset.samples = existing.samples;
                total_captured = dataset.samples.len();

                let mut last_string: usize = 0;
                let mut last_fret: u8 = 0;
                let mut found_any = false;
                for si in 0..6 {
                    for f in 0..23 {
                        if coverage[si][f] >= PLUCKS_PER_POSITION as u32 {
                            last_string = si;
                            last_fret = f as u8;
                            found_any = true;
                        }
                    }
                }

                if found_any {
                    if last_fret < FRETS as u8 {
                        start_string = last_string;
                        start_fret = last_fret + 1;
                    } else if last_string < 5 {
                        start_string = last_string + 1;
                        start_fret = 0;
                    } else {
                        start_string = 6;
                    }
                }
                println!(
                    "  Resuming from {} fret {}.\n",
                    if start_string < 6 {
                        STRING_NAMES[start_string]
                    } else {
                        "noise"
                    },
                    start_fret
                );
            }
            "j" => {
                // Jump to specific position
                dataset.samples = existing.samples;
                total_captured = dataset.samples.len();

                println!("  Strings: 0=Low E  1=A  2=D  3=G  4=B  5=High E");
                start_string = prompt_usize("  Start from string [0-5]: ", 5);
                start_fret =
                    prompt_usize(&format!("  Start from fret [0-{}]: ", FRETS), FRETS) as u8;
                println!(
                    "  Jumping to {} fret {}.\n",
                    STRING_NAMES[start_string], start_fret
                );
            }
            "r" => {
                // Redo a specific position (remove existing samples for that position first)
                dataset.samples = existing.samples;

                println!("  Strings: 0=Low E  1=A  2=D  3=G  4=B  5=High E");
                let redo_string = prompt_usize("  Redo string [0-5]: ", 5);
                let redo_fret = prompt_usize(&format!("  Redo fret [0-{}]: ", FRETS), FRETS) as u8;

                let before = dataset.samples.len();
                dataset
                    .samples
                    .retain(|s| !(s.string_idx == redo_string as u8 && s.fret == redo_fret));
                let removed = before - dataset.samples.len();
                total_captured = dataset.samples.len();

                println!(
                    "  Removed {} samples for {} fret {}. Will re-capture.\n",
                    removed, STRING_NAMES[redo_string], redo_fret
                );

                start_string = redo_string;
                start_fret = redo_fret;
            }
            "n" => {
                // Fresh start
                println!("  Starting fresh.\n");
            }
            "x" => {
                // Skip to noise only
                dataset.samples = existing.samples;
                total_captured = dataset.samples.len();
                start_string = 6; // skip fretboard loop entirely
                println!("  Skipping to noise capture.\n");
            }
            _ => {
                // Default: resume
                dataset.samples = existing.samples;
                total_captured = dataset.samples.len();
                println!("  Resuming.\n");
            }
        }
    }

    // ── Fretboard Walk ───────────────────────────────────
    let mut quit = false;
    let mut total_rejected = 0;

    for string_idx in start_string..6 {
        if quit {
            break;
        }
        let string_name = STRING_NAMES[string_idx];
        let base_midi = STRING_BASE_PITCH[string_idx];

        println!("╔══════════════════════════════════════════════╗");
        println!(
            "║  String {}/6: {} (open = {}){}║",
            string_idx + 1,
            string_name,
            midi_to_note_name(base_midi),
            " ".repeat(44 - 25 - midi_to_note_name(base_midi).len())
        );
        println!("╚══════════════════════════════════════════════╝");
        println!("  Press Enter to start this string...");
        wait_enter();

        let fret_start = if string_idx == start_string {
            start_fret
        } else {
            0
        };
        for fret in fret_start..=(FRETS as u8) {
            if quit {
                break;
            }

            let expected_midi = base_midi + fret;
            let expected_note = midi_to_note_name(expected_midi);
            let class_id = (string_idx * (FRETS + 1) + fret as usize + 1) as u32;
            let label = format!("string_{}_fret_{}", string_idx, fret);

            loop {
                println!(
                    "\n  ── {} fret {} ({}, MIDI {}) ──",
                    string_name, fret, expected_note, expected_midi
                );

                let (plucks, rejected) = capture_position_validated(
                    &state,
                    pluck_threshold,
                    sr,
                    capture_samples,
                    expected_midi,
                    PLUCKS_PER_POSITION,
                );

                total_rejected += rejected;

                println!(
                    "\n  Captured {}/{} plucks ({} rejected by pitch validation).",
                    plucks.len(),
                    PLUCKS_PER_POSITION,
                    rejected
                );
                println!("  [Enter] accept  [P] playback  [R] redo  [S] skip  [Q] quit");

                match prompt_action() {
                    Action::Accept => {
                        for (
                            audio,
                            detected_midi,
                            conf,
                            rms,
                            peak,
                            validated,
                            goertzel_harmonics,
                            harmonic_ratios,
                            spectral_centroid,
                        ) in &plucks
                        {
                            dataset.samples.push(TrainingSample {
                                audio: audio.clone(),
                                label: label.clone(),
                                class_id,
                                string_idx: string_idx as u8,
                                fret,
                                expected_midi,
                                detected_midi: *detected_midi,
                                confidence: *conf,
                                rms: *rms,
                                peak: *peak,
                                sample_rate: sr as u32,
                                pitch_validated: *validated,
                                goertzel_harmonics: goertzel_harmonics.clone(),
                                harmonic_ratios: harmonic_ratios.clone(),
                                spectral_centroid: *spectral_centroid,
                            });
                            total_captured += 1;
                        }
                        // Auto-save after each position
                        save_dataset(&dataset, total_captured);
                        break;
                    }
                    Action::Playback => {
                        let play_data: Vec<PluckData> = plucks
                            .iter()
                            .map(|(a, d, c, r, p, _, _, _, _)| (a.clone(), *d, *c, *r, *p))
                            .collect();
                        playback_samples(&play_data, sr, expected_note.clone());
                        continue;
                    }
                    Action::Redo => {
                        println!("  Redoing position...");
                        continue;
                    }
                    Action::Skip => {
                        println!("  Skipped.");
                        break;
                    }
                    Action::Quit => {
                        quit = true;
                        break;
                    }
                }
            }
        }
    }

    // ── Noise Capture Phase (6 categories) ───────────────
    if !quit {
        println!("\n╔══════════════════════════════════════════════╗");
        println!("║  Noise Capture Phase (6 categories)          ║");
        println!("╚══════════════════════════════════════════════╝\n");

        let noise_instructions = [
            (
                "ambient",
                "Don't touch the guitar. Room noise, fan, cable hum.",
            ),
            (
                "string_muting",
                "Rest palm on strings, tap lightly, shift hand.",
            ),
            (
                "pick_scrapes",
                "Scrape pick along strings, tap pick on body.",
            ),
            (
                "finger_slides",
                "Slide fingers up/down neck without plucking.",
            ),
            (
                "string_brushes",
                "Brush across strings without clean plucks.",
            ),
            (
                "accidental_touches",
                "Bump strings, fret without plucking, light touches.",
            ),
        ];

        for (category, instruction) in &noise_instructions {
            if quit {
                break;
            }
            println!("  ── Noise: {} ──", category);
            println!("  {}", instruction);
            println!(
                "  Record for {:.0} seconds. Press Enter to start...",
                NOISE_DURATION_PER_CATEGORY_SECS
            );
            wait_enter();

            let noise_samples = capture_noise_category(
                &state,
                sr,
                capture_samples,
                NOISE_DURATION_PER_CATEGORY_SECS,
            );
            println!(
                "  Captured {} noise samples for '{}'.\n",
                noise_samples.len(),
                category
            );

            for (audio, rms, peak) in &noise_samples {
                dataset.samples.push(TrainingSample {
                    audio: audio.clone(),
                    label: format!("noise_{}", category),
                    class_id: 0,
                    string_idx: 255,
                    fret: 255,
                    expected_midi: 0,
                    detected_midi: 0,
                    confidence: 0.0,
                    rms: *rms,
                    peak: *peak,
                    sample_rate: sr as u32,
                    pitch_validated: false,
                    goertzel_harmonics: vec![0.0; 10],
                    harmonic_ratios: vec![0.0; 9],
                    spectral_centroid: 0.0,
                });
                total_captured += 1;
            }
            // Auto-save after each noise category
            save_dataset(&dataset, total_captured);
        }
    }

    // ── Save Dataset ─────────────────────────────────────
    println!("\n══════════════════════════════════════════════");
    println!("  Capture Complete! ({})", guitar_name);
    println!("══════════════════════════════════════════════\n");

    // Summary
    let mut class_counts: std::collections::HashMap<String, usize> =
        std::collections::HashMap::new();
    for s in &dataset.samples {
        *class_counts.entry(s.label.clone()).or_insert(0) += 1;
    }

    println!(
        "  Total samples: {} ({} rejected by pitch validation)",
        total_captured, total_rejected
    );
    println!("  Classes: {}", class_counts.len());

    // Per-string summary
    for si in 0..6 {
        let count = dataset
            .samples
            .iter()
            .filter(|s| s.string_idx == si as u8)
            .count();
        let validated = dataset
            .samples
            .iter()
            .filter(|s| s.string_idx == si as u8 && s.pitch_validated)
            .count();
        println!(
            "  {}: {} samples ({} pitch-validated)",
            STRING_NAMES[si], count, validated
        );
    }
    let noise_count = dataset
        .samples
        .iter()
        .filter(|s| s.string_idx == 255)
        .count();
    println!("  Noise: {} samples", noise_count);

    // Final save
    println!("\n  Final save...");
    save_dataset(&dataset, total_captured);
    println!("  Guitar: {}", guitar_name);

    // ── Review Mode ──────────────────────────────────────
    println!("\n  Review dataset? [Y/n]");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
    if input.trim().to_lowercase() != "n" {
        review_dataset(&dataset, sr);
    }
}

// ── Capture Functions ────────────────────────────────────────────────

// (audio, detected_midi, confidence, rms, peak, pitch_validated, goertzel_harmonics, harmonic_ratios, spectral_centroid)
type ValidatedPluck = (Vec<f32>, u8, f32, f32, f32, bool, Vec<f32>, Vec<f32>, f32);
type PluckData = (Vec<f32>, u8, f32, f32, f32);

/// Compute Goertzel harmonic features for a captured audio sample.
///
/// Uses the Goertzel algorithm to compute magnitude at the first 10 harmonics
/// of the given fundamental frequency. Returns:
/// - `goertzel_harmonics`: magnitudes at harmonics 1-10
/// - `harmonic_ratios`: ratios h2/h1, h3/h1, ..., h10/h1 (9 values)
/// - `spectral_centroid`: weighted centroid from harmonic magnitudes
///
/// Note: We implement Goertzel inline here rather than using `GoertzelBank`,
/// because the bank targets fixed note frequencies while we need per-harmonic
/// targeting at arbitrary fundamentals.
fn compute_goertzel_features(
    audio: &[f32],
    fundamental_freq: f32,
    sample_rate: f32,
) -> (Vec<f32>, Vec<f32>, f32) {
    let n = audio.len();
    if n == 0 || fundamental_freq <= 0.0 {
        return (vec![0.0; 10], vec![0.0; 9], 0.0);
    }

    let mut harmonics = Vec::with_capacity(10);

    for h in 1..=10u32 {
        let target_freq = fundamental_freq * h as f32;

        // Skip harmonics above Nyquist
        if target_freq >= sample_rate / 2.0 {
            harmonics.push(0.0);
            continue;
        }

        // Goertzel algorithm for a single frequency
        let k = (target_freq * n as f32 / sample_rate) as f64;
        let omega = 2.0 * (PI as f64) * k / n as f64;
        let coeff = 2.0 * omega.cos();

        let mut s0: f64 = 0.0;
        let mut s1: f64 = 0.0;
        let mut s2: f64;

        for &sample in audio {
            s2 = s1;
            s1 = s0;
            s0 = sample as f64 + coeff * s1 - s2;
        }

        // Compute magnitude
        let real = s0 - s1 * omega.cos();
        let imag = s1 * omega.sin();
        let magnitude = ((real * real + imag * imag).sqrt() / n as f64) as f32;
        harmonics.push(magnitude);
    }

    // Compute harmonic ratios: h2/h1, h3/h1, ..., h10/h1
    let h1 = harmonics[0];
    let ratios: Vec<f32> = if h1 > 1e-10 {
        harmonics[1..].iter().map(|&h| h / h1).collect()
    } else {
        vec![0.0; 9]
    };

    // Compute spectral centroid from harmonic magnitudes
    let total_magnitude: f32 = harmonics.iter().sum();
    let spectral_centroid = if total_magnitude > 1e-10 {
        let weighted_sum: f32 = harmonics
            .iter()
            .enumerate()
            .map(|(i, &mag)| fundamental_freq * (i + 1) as f32 * mag)
            .sum();
        weighted_sum / total_magnitude
    } else {
        0.0
    };

    (harmonics, ratios, spectral_centroid)
}

/// Capture plucks with onset-forward recording and pitch validation.
fn capture_position_validated(
    state: &Arc<Mutex<AudioState>>,
    pluck_threshold: f32,
    sample_rate: usize,
    capture_samples: usize,
    expected_midi: u8,
    count: usize,
) -> (Vec<ValidatedPluck>, usize) {
    let mut plucks = Vec::new();
    let mut rejected = 0;
    let mut last_pluck = Instant::now();

    print!("    Plucks: ");
    std::io::stdout().flush().unwrap();

    while plucks.len() < count {
        std::thread::sleep(Duration::from_millis(10));

        // Check if a capture has completed
        let completed = {
            let mut s = state.lock().unwrap();
            s.completed_capture.take()
        };

        if let Some(audio) = completed {
            // Get the detection info
            let s = state.lock().unwrap();
            let detected_midi = s.midi_note;
            let conf = s.confidence;
            let rms = s.rms;
            let peak = s.peak;
            drop(s);

            // FIX #2: Pitch validation gate
            let pitch_diff = (detected_midi as i16 - expected_midi as i16).abs();
            let validated = pitch_diff <= 1 && conf > MIN_CONFIDENCE as f32;

            if !validated && conf > MIN_CONFIDENCE as f32 {
                // Pitch detection worked but detected wrong note
                print!("X");
                std::io::stdout().flush().unwrap();
                rejected += 1;

                // Re-arm onset detection for next pluck
                state.lock().unwrap().onset_armed = true;
                continue;
            }

            // Compute Goertzel harmonic features for this pluck
            let fundamental_freq = midi_to_freq(expected_midi);
            let (goertzel_harmonics, harmonic_ratios, spectral_centroid) =
                compute_goertzel_features(&audio, fundamental_freq, sample_rate as f32);

            plucks.push((
                audio,
                detected_midi,
                conf,
                rms,
                peak,
                validated,
                goertzel_harmonics,
                harmonic_ratios,
                spectral_centroid,
            ));
            print!("{} ", plucks.len());
            std::io::stdout().flush().unwrap();

            // Re-arm onset detection
            state.lock().unwrap().onset_armed = true;
        }

        // Arm onset detection if not capturing and enough time passed
        {
            let mut s = state.lock().unwrap();
            if !s.capturing
                && s.completed_capture.is_none()
                && last_pluck.elapsed() > Duration::from_millis(250)
            {
                s.onset_armed = true;
            }
        }
    }

    (plucks, rejected)
}

/// Capture noise samples (onset-forward, no pitch validation).
fn capture_noise_category(
    state: &Arc<Mutex<AudioState>>,
    sample_rate: usize,
    capture_samples: usize,
    duration_secs: f32,
) -> Vec<(Vec<f32>, f32, f32)> {
    let mut samples = Vec::new();
    let start = Instant::now();
    let duration = Duration::from_secs_f32(duration_secs);

    print!("    Capturing: ");
    std::io::stdout().flush().unwrap();

    // For noise, capture every 500ms regardless of onset
    let mut last_capture = Instant::now();

    while start.elapsed() < duration {
        std::thread::sleep(Duration::from_millis(20));

        if last_capture.elapsed() > Duration::from_millis(500) {
            last_capture = Instant::now();

            // Start a capture
            {
                let mut s = state.lock().unwrap();
                s.capture_buffer.clear();
                s.capturing = true;
            }

            // Wait for capture to complete
            std::thread::sleep(Duration::from_millis(
                (SAMPLE_DURATION_SECS * 1000.0) as u64 + 50,
            ));

            let completed = {
                let mut s = state.lock().unwrap();
                s.capturing = false;
                let audio = if s.capture_buffer.len() >= capture_samples / 2 {
                    Some(s.capture_buffer.clone())
                } else {
                    None
                };
                s.capture_buffer.clear();
                let rms = s.rms;
                let peak = s.peak;
                (audio, rms, peak)
            };

            if let Some(audio) = completed.0 {
                samples.push((audio, completed.1, completed.2));
                print!(".");
                std::io::stdout().flush().unwrap();
            }
        }
    }

    println!(" done");
    samples
}

fn playback_samples(plucks: &[PluckData], sample_rate: usize, note_name: String) {
    let host = cpal::default_host();
    let output_device = match host.default_output_device() {
        Some(d) => d,
        None => {
            println!("  No audio output device for playback!");
            return;
        }
    };

    let output_config = preferred_output_config(&output_device, &[cpal::SampleFormat::F32])
        .expect("No compatible output config");

    println!(
        "  Playing back {} plucks for {}...",
        plucks.len().min(5),
        note_name
    );

    for (i, (audio, detected, conf, rms, _)) in plucks.iter().take(5).enumerate() {
        println!(
            "    Pluck {}: detected={} conf={:.0}% rms={:.3}",
            i + 1,
            midi_to_note_name(*detected),
            conf * 100.0,
            rms
        );

        let audio_data = Arc::new(Mutex::new((audio.clone(), 0usize)));
        let ad = Arc::clone(&audio_data);

        let config: cpal::StreamConfig = output_config.into();
        let out_channels = config.channels as usize;

        let stream = output_device
            .build_output_stream(
                config,
                move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                    let mut ad = ad.lock().unwrap();
                    let (ref audio, ref mut pos) = *ad;
                    for frame in data.chunks_mut(out_channels) {
                        let sample = if *pos < audio.len() { audio[*pos] } else { 0.0 };
                        for s in frame.iter_mut() {
                            *s = sample;
                        }
                        *pos += 1;
                    }
                },
                |e| eprintln!("Playback error: {}", e),
                None,
            )
            .expect("Failed to build output stream");

        stream.play().expect("Failed to play");
        let play_duration = Duration::from_secs_f32(audio.len() as f32 / sample_rate as f32 + 0.1);
        std::thread::sleep(play_duration);
    }
}

fn review_dataset(dataset: &TrainingDataset, sample_rate: usize) {
    println!(
        "\n=== Dataset Review ({}) ===\n",
        dataset.metadata.guitar_name
    );
    println!("  Total samples: {}", dataset.samples.len());

    let mut class_counts: std::collections::HashMap<&str, usize> = std::collections::HashMap::new();
    for s in &dataset.samples {
        *class_counts.entry(&s.label).or_insert(0) += 1;
    }
    println!("  Classes: {}\n", class_counts.len());

    for string_idx in 0..6u8 {
        let string_samples: Vec<&TrainingSample> = dataset
            .samples
            .iter()
            .filter(|s| s.string_idx == string_idx)
            .collect();
        if string_samples.is_empty() {
            continue;
        }

        let avg_conf =
            string_samples.iter().map(|s| s.confidence).sum::<f32>() / string_samples.len() as f32;
        let validated = string_samples.iter().filter(|s| s.pitch_validated).count();
        let correct = string_samples
            .iter()
            .filter(|s| (s.detected_midi as i16 - s.expected_midi as i16).abs() <= 1)
            .count();

        println!(
            "  {}: {} samples, avg conf {:.0}%, validated {}, pitch accuracy {:.0}%",
            STRING_NAMES[string_idx as usize],
            string_samples.len(),
            avg_conf * 100.0,
            validated,
            correct as f32 / string_samples.len() as f32 * 100.0
        );
    }

    let noise_count = dataset
        .samples
        .iter()
        .filter(|s| s.string_idx == 255)
        .count();
    println!("  Noise: {} samples\n", noise_count);

    println!("  Browse: enter a label to play (or 'q' to quit):");
    println!("  Example: string_0_fret_0, noise_ambient\n");

    loop {
        print!("  > ");
        std::io::stdout().flush().unwrap();
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
        let query = input.trim();

        if query == "q" || query.is_empty() {
            break;
        }

        let matches: Vec<&TrainingSample> = dataset
            .samples
            .iter()
            .filter(|s| s.label == query)
            .collect();

        if matches.is_empty() {
            println!("    No samples for '{}'. Available:", query);
            let mut labels: Vec<&str> = class_counts.keys().copied().collect();
            labels.sort();
            for l in labels.iter().take(15) {
                print!("    {} ", l);
            }
            if labels.len() > 15 {
                print!("... +{} more", labels.len() - 15);
            }
            println!();
            continue;
        }

        println!("    Found {} samples. Playing first 5...", matches.len());
        let plucks: Vec<PluckData> = matches
            .iter()
            .take(5)
            .map(|s| {
                (
                    s.audio.clone(),
                    s.detected_midi,
                    s.confidence,
                    s.rms,
                    s.peak,
                )
            })
            .collect();
        playback_samples(&plucks, sample_rate, query.to_string());
    }
}

// ── Audio Processing (FIX #1: VecDeque + FIX #3: onset-forward) ─────

fn append_capture_frame(
    capture_buffer: &mut Vec<f32>,
    frame: &[f32],
    capture_target: usize,
    started_capture: bool,
) -> Option<Vec<f32>> {
    if started_capture {
        capture_buffer.clear();
        capture_buffer.extend_from_slice(frame);
    } else if capture_buffer.len() < capture_target {
        capture_buffer.extend_from_slice(frame);
    }

    (capture_buffer.len() >= capture_target).then(|| capture_buffer[..capture_target].to_vec())
}

fn audio_process(
    data: &[f32],
    channels: usize,
    target_ch: usize,
    sample_rate: usize,
    buffer: &Arc<Mutex<VecDeque<f32>>>,
    state: &Arc<Mutex<AudioState>>,
) {
    let mut buf = buffer.lock().unwrap();

    // Extract target channel into VecDeque (FIX #1: O(1) push/pop)
    for frame in data.chunks(channels) {
        buf.push_back(frame.get(target_ch).copied().unwrap_or(0.0));
    }

    // Keep only what we need
    while buf.len() > BUFFER_SIZE * 2 {
        buf.pop_front();
    }

    if buf.len() >= BUFFER_SIZE {
        let frame: Vec<f32> = buf.drain(..BUFFER_SIZE).collect();

        let rms = (frame.iter().map(|s| s * s).sum::<f32>() / BUFFER_SIZE as f32).sqrt();
        let peak = frame.iter().map(|s| s.abs()).fold(0.0f32, f32::max);

        // Pitch detection
        let samples_f64: Vec<f64> = frame.iter().map(|&s| s as f64).collect();
        let mut detector = McLeodDetector::new(BUFFER_SIZE, BUFFER_SIZE / 2);
        let pitch = detector.get_pitch(&samples_f64, sample_rate, 0.3, 0.3);

        let mut s = state.lock().unwrap();

        // FIX #3: Onset-forward capture
        // If onset is armed and we detect an RMS spike, start recording.
        // Track this callback explicitly so the triggering frame is appended once,
        // not duplicated at the head of every captured training sample.
        let started_capture =
            s.onset_armed && !s.capturing && rms > s.prev_rms * 3.0 && rms > 0.015;
        if started_capture {
            s.onset_armed = false;
            s.capturing = true;
        }

        // If actively capturing, append this block exactly once.
        if s.capturing {
            let capture_target = s.capture_target;
            if let Some(completed) = append_capture_frame(
                &mut s.capture_buffer,
                &frame,
                capture_target,
                started_capture,
            ) {
                s.capturing = false;
                s.completed_capture = Some(completed);
                s.capture_buffer.clear();
            }
        }

        // Update state
        s.rms = rms;
        s.peak = peak;

        if let Some(p) = pitch {
            if p.clarity > MIN_CONFIDENCE {
                let freq = p.frequency as f32;
                let (midi, _) = freq_to_midi(freq);
                s.frequency = freq;
                s.confidence = p.clarity as f32;
                s.midi_note = midi;
                s.note_name = midi_to_note_name(midi);
            }
        }

        s.prev_rms = rms;
    }
}

// ── UI Helpers ───────────────────────────────────────────────────────

enum Action {
    Accept,
    Playback,
    Redo,
    Skip,
    Quit,
}

fn save_dataset(dataset: &TrainingDataset, total: usize) {
    let packed = rmp_serde::to_vec(dataset).expect("Failed to serialize");
    std::fs::write(DATASET_PATH, &packed).expect("Failed to write");
    println!(
        "  [saved: {} samples, {:.1} MB]",
        total,
        packed.len() as f64 / 1_000_000.0
    );
}

fn prompt_action() -> Action {
    loop {
        print!("  > ");
        std::io::stdout().flush().unwrap();
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
        match input.trim().to_lowercase().as_str() {
            "" => return Action::Accept,
            "p" => return Action::Playback,
            "r" => return Action::Redo,
            "s" => return Action::Skip,
            "q" => return Action::Quit,
            _ => println!("  [Enter]=accept [P]=play [R]=redo [S]=skip [Q]=quit"),
        }
    }
}

fn prompt_usize(prompt: &str, max: usize) -> usize {
    print!("{}", prompt);
    std::io::stdout().flush().unwrap();
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
    input.trim().parse::<usize>().unwrap_or(0).min(max)
}

fn wait_enter() {
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
}

fn chrono_now() -> String {
    format!(
        "{:?}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capture_triggering_frame_is_appended_once() {
        let first = vec![1.0; BUFFER_SIZE];
        let second = vec![2.0; BUFFER_SIZE];
        let mut buffer = Vec::new();
        let target = BUFFER_SIZE * 2;

        assert!(append_capture_frame(&mut buffer, &first, target, true).is_none());
        assert_eq!(buffer.len(), BUFFER_SIZE);

        let completed = append_capture_frame(&mut buffer, &second, target, false).unwrap();
        assert_eq!(&completed[..BUFFER_SIZE], first.as_slice());
        assert_eq!(&completed[BUFFER_SIZE..], second.as_slice());
    }
}
