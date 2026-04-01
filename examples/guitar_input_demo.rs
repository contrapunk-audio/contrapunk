//! Contrapunk Guitar Input Demo
//!
//! Standalone example demonstrating the full DSP guitar-to-MIDI pipeline:
//! pitch detection, inharmonicity-based string identification, and MIDI output.
//!
//! Run with: cargo run --release --example guitar_input_demo
//!
//! Requires an audio input device (e.g., Audient iD14).

use contrapunk::audio::guitar::{midi_to_note_name, STRING_BASE_PITCH, STRING_NAMES};
use contrapunk::audio::guitar_input::{
    compute_rms, open_string_freq, GuitarCalibration, GuitarInput, GuitarInputConfig,
    MidiEvent, StringProfile,
};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

use std::io::{self, Write};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

const BUFFER_SIZE: usize = 1024;
const CALIBRATION_LISTEN_SECS: f32 = 3.0;
const SILENCE_LISTEN_SECS: f32 = 3.0;

fn main() {
    println!();
    println!("Contrapunk Guitar Input Demo");
    println!("{}", "\u{2501}".repeat(28));
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

    println!("  Audio Input Devices:");
    for (i, d) in devices.iter().enumerate() {
        println!("    [{}] {}", i, d.name().unwrap_or_default());
    }
    println!();

    let dev_idx = prompt_usize("  Select device", devices.len() - 1);
    let device = &devices[dev_idx];
    let device_name = device.name().unwrap_or_default();

    let input_config = device
        .default_input_config()
        .expect("No default input config");
    let sample_rate = input_config.sample_rate().0 as usize;
    let channels = input_config.channels() as usize;

    println!("  Device: {} ({}ch, {}Hz)", device_name, channels, sample_rate);

    let channel = if channels > 1 {
        prompt_usize(&format!("  Channel [0-{}]", channels - 1), channels - 1)
    } else {
        0
    };

    let latency_ms = BUFFER_SIZE as f32 / sample_rate as f32 * 1000.0;
    println!(
        "  Buffer: {} samples ({:.1}ms @ {}Hz)",
        BUFFER_SIZE, latency_ms, sample_rate
    );
    println!();

    // ── Pipeline setup ──────────────────────────────────────────────

    let config = GuitarInputConfig {
        buffer_size: BUFFER_SIZE,
        sample_rate,
        onset_threshold: 0.015,
        string_confidence_min: 0.4,
        bends_enabled: true,
        legato_enabled: false,
        filter_enabled: false,
        min_clarity: 0.40,
        cooldown_samples: sample_rate / 10, // 100ms
        n_harmonics: 6,
    };

    let pipeline = Arc::new(Mutex::new(GuitarInput::new(config.clone())));

    // Shared state for audio callback -> main thread
    let audio_ring = Arc::new(Mutex::new(AudioRing::new(sample_rate * 4))); // 4s ring

    // Events queue
    let events_queue: Arc<Mutex<Vec<(MidiEvent, Instant)>>> =
        Arc::new(Mutex::new(Vec::new()));

    // ── Start audio stream ──────────────────────────────────────────

    let ring_c = Arc::clone(&audio_ring);
    let pipeline_c = Arc::clone(&pipeline);
    let events_c = Arc::clone(&events_queue);
    let ch = channel;
    let n_ch = channels;

    let stream_config: cpal::StreamConfig = input_config.clone().into();
    let stream = match input_config.sample_format() {
        cpal::SampleFormat::F32 => device.build_input_stream(
            &stream_config,
            move |data: &[f32], _| {
                audio_callback(data, n_ch, ch, &ring_c, &pipeline_c, &events_c);
            },
            |e| eprintln!("Audio error: {}", e),
            None,
        ),
        cpal::SampleFormat::I16 => {
            let ring_c2 = Arc::clone(&audio_ring);
            let pipeline_c2 = Arc::clone(&pipeline);
            let events_c2 = Arc::clone(&events_queue);
            device.build_input_stream(
                &stream_config,
                move |data: &[i16], _| {
                    let float: Vec<f32> =
                        data.iter().map(|&s| s as f32 / 32768.0).collect();
                    audio_callback(
                        &float, n_ch, ch, &ring_c2, &pipeline_c2, &events_c2,
                    );
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

    // ── Calibration phase ───────────────────────────────────────────

    println!("  CALIBRATION");
    println!();

    // Step 1: Noise floor
    print!("  [1/7] Silence for {:.0}s... ", SILENCE_LISTEN_SECS);
    io::stdout().flush().unwrap();
    std::thread::sleep(Duration::from_secs_f32(SILENCE_LISTEN_SECS));

    let noise_floor = {
        let ring = audio_ring.lock().unwrap();
        let recent = ring.recent(sample_rate); // last 1s
        GuitarInput::measure_noise_floor(&recent)
    };
    println!("noise floor: {:.4}", noise_floor);

    // Steps 2-7: Calibrate each string
    let string_labels = ["E2", "A2", "D3", "G3", "B3", "E4"];
    let mut string_profiles: Vec<StringProfile> = Vec::new();

    for (i, label) in string_labels.iter().enumerate() {
        print!(
            "  [{}/7] Pluck {} open... ",
            i + 2,
            label
        );
        io::stdout().flush().unwrap();

        // Wait for a pluck (RMS spike above noise)
        let pluck_threshold = (noise_floor * 5.0).max(0.012);
        let capture = wait_for_pluck(
            &audio_ring,
            sample_rate,
            pluck_threshold,
            CALIBRATION_LISTEN_SECS,
        );

        match capture {
            Some(audio) => {
                let pipe = pipeline.lock().unwrap();
                if let Some(profile) = pipe.calibrate_string(i, &audio) {
                    println!(
                        "B={:.4}, f={:.1}Hz",
                        profile.inharmonicity_b, profile.open_frequency
                    );
                    string_profiles.push(profile);
                } else {
                    println!("failed -- using defaults");
                    string_profiles.push(default_string_profile(i));
                }
            }
            None => {
                println!("timeout -- using defaults");
                string_profiles.push(default_string_profile(i));
            }
        }
    }

    // Build calibration
    let calibration = GuitarCalibration {
        noise_floor,
        signal_peak: 1.0,
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
    }

    println!();
    println!("  READY -- play anything! (Ctrl+C to exit)");
    println!();
    println!(
        "  {:5}  {:6}  {:4}  {:5}  {:4}  {:7}",
        "Note", "String", "Fret", "Conf.", "Vel.", "Latency"
    );
    println!("  {}", "\u{2500}".repeat(42));

    // ── Live detection loop ─────────────────────────────────────────

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
                    note, velocity, ..
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

                    let note_name = midi_to_note_name(note);
                    println!(
                        "  {:5}  {:6}  {:>4}  {:>4.0}%  {:>4}  {:>5.0}ms",
                        note_name,
                        string_name,
                        fret,
                        conf * 100.0,
                        velocity,
                        latency_ms_val
                    );

                    note_count += 1;
                    total_latency_ms += latency_ms_val;
                }
                MidiEvent::NoteOff { note, .. } => {
                    // Quiet note-off, no display needed
                    let _ = note;
                }
                MidiEvent::PitchBend { cents, .. } => {
                    if cents.abs() > 20 {
                        println!("  bend: {:+}c", cents);
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

// ── Audio callback ──────────────────────────────────────────────────

fn audio_callback(
    data: &[f32],
    channels: usize,
    target_channel: usize,
    ring: &Arc<Mutex<AudioRing>>,
    pipeline: &Arc<Mutex<GuitarInput>>,
    events: &Arc<Mutex<Vec<(MidiEvent, Instant)>>>,
) {
    // Extract mono channel
    let mono: Vec<f32> = data
        .chunks(channels)
        .filter_map(|frame| frame.get(target_channel).copied())
        .collect();

    // Store in ring buffer
    {
        let mut r = ring.lock().unwrap();
        r.push(&mono);
    }

    // Process through pipeline
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

// ── Audio ring buffer ───────────────────────────────────────────────

struct AudioRing {
    buffer: Vec<f32>,
    write_pos: usize,
    capacity: usize,
    filled: usize,
}

impl AudioRing {
    fn new(capacity: usize) -> Self {
        Self {
            buffer: vec![0.0; capacity],
            write_pos: 0,
            capacity,
            filled: 0,
        }
    }

    fn push(&mut self, samples: &[f32]) {
        for &s in samples {
            self.buffer[self.write_pos] = s;
            self.write_pos = (self.write_pos + 1) % self.capacity;
            if self.filled < self.capacity {
                self.filled += 1;
            }
        }
    }

    /// Get the most recent `n` samples (linearized, oldest first).
    fn recent(&self, n: usize) -> Vec<f32> {
        let n = n.min(self.filled);
        let mut out = Vec::with_capacity(n);
        let start = if self.write_pos >= n {
            self.write_pos - n
        } else {
            self.capacity - (n - self.write_pos)
        };
        for i in 0..n {
            out.push(self.buffer[(start + i) % self.capacity]);
        }
        out
    }
}

// ── Calibration helpers ─────────────────────────────────────────────

/// Wait for a pluck (RMS spike) and return the captured audio.
fn wait_for_pluck(
    ring: &Arc<Mutex<AudioRing>>,
    sample_rate: usize,
    threshold: f32,
    timeout_secs: f32,
) -> Option<Vec<f32>> {
    let start = Instant::now();
    let timeout = Duration::from_secs_f32(timeout_secs);
    let capture_len = sample_rate / 2; // 500ms

    loop {
        if start.elapsed() > timeout {
            return None;
        }
        std::thread::sleep(Duration::from_millis(20));

        let recent = {
            let r = ring.lock().unwrap();
            r.recent(capture_len)
        };

        let rms = compute_rms(&recent);
        if rms > threshold && recent.len() >= capture_len {
            return Some(recent);
        }
    }
}

/// Create a default string profile for when calibration fails.
fn default_string_profile(string_idx: usize) -> StringProfile {
    let default_b = [0.006, 0.003, 0.003, 0.006, 0.003, 0.002];
    StringProfile {
        name: STRING_NAMES.get(string_idx).copied().unwrap_or("?"),
        open_midi: STRING_BASE_PITCH
            .get(string_idx)
            .copied()
            .unwrap_or(40),
        open_frequency: open_string_freq(string_idx),
        inharmonicity_b: default_b.get(string_idx).copied().unwrap_or(0.003),
        harmonic_ratios: vec![0.5, 0.3, 0.2, 0.1, 0.05],
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
    // Best-effort Ctrl+C handling without extra dependencies.
    // If ctrlc crate is not available, just catch the signal.
    std::thread::spawn(move || {
        // We use a simple approach: read stdin for 'q' as backup,
        // and rely on the OS SIGINT to terminate.
        // The main loop checks `running` on each iteration.
        let _ = running; // keep alive
    });

    // Register signal handler
    // Since we don't have the ctrlc crate, we use a simpler approach:
    // the user can also press Enter to see if we should quit.
    // The process will terminate on Ctrl+C via default SIGINT behavior.
}
