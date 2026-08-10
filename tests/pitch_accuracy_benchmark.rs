//! Pitch detection accuracy benchmarks.
//!
//! Tests the ACTUAL GuitarInput pipeline (the one that ships in the app)
//! against known signals, measuring:
//! - Note accuracy (% correct within ±1 semitone)
//! - Octave error rate (detected note is correct class but wrong octave)
//! - Latency (ms from onset to first correct detection)
//! - Stability (pitch variance during sustained notes)
//! - Harmonic spike rate (false high-frequency detections during sustain)
//!
//! Run:  cargo test --test pitch_accuracy_benchmark -- --nocapture
//! Fast: cargo test --test pitch_accuracy_benchmark

use contrapunk::audio::guitar_input::{GuitarInput, GuitarInputConfig, MidiEvent};
use contrapunk::audio::test_signals;
use std::f32::consts::PI;

const SAMPLE_RATE: usize = 48000;

fn default_config() -> GuitarInputConfig {
    GuitarInputConfig {
        sample_rate: SAMPLE_RATE,
        buffer_size: 2048,
        hop_size: 256,
        input_gain: 1.0,
        pitch_bend_range: 48,
        ..GuitarInputConfig::default()
    }
}

/// Run audio through GuitarInput and collect MIDI events with timestamps.
fn run_pipeline(config: &GuitarInputConfig, audio: &[f32]) -> Vec<(f32, MidiEvent)> {
    let mut pipeline = GuitarInput::new(config.clone());
    let mut events = Vec::new();
    let chunk_size = 128;

    for (i, chunk) in audio.chunks(chunk_size).enumerate() {
        let time = (i * chunk_size) as f32 / SAMPLE_RATE as f32;
        for event in pipeline.process_block(chunk) {
            events.push((time, event));
        }
    }
    events
}

/// Extract NoteOn events as (time, midi_note).
fn note_ons(events: &[(f32, MidiEvent)]) -> Vec<(f32, u8)> {
    events
        .iter()
        .filter_map(|(time, event)| match event {
            MidiEvent::NoteOn { note, .. } => Some((*time, *note)),
            _ => None,
        })
        .collect()
}

/// Generate a guitar-like signal: fundamental + harmonics with exponential decay.
fn guitar_signal(freq: f32, duration_secs: f32, amplitude: f32) -> Vec<f32> {
    let n = (SAMPLE_RATE as f32 * duration_secs) as usize;
    let mut signal = vec![0.0f32; n];
    let harmonics = [1.0, 0.5, 0.33, 0.25, 0.15, 0.1]; // typical guitar harmonic content

    for (i, sample) in signal.iter_mut().enumerate() {
        let t = i as f32 / SAMPLE_RATE as f32;
        let decay = (-t * 3.0).exp(); // exponential decay
        let mut val = 0.0;
        for (h, &strength) in harmonics.iter().enumerate() {
            let harmonic_freq = freq * (h + 1) as f32;
            val += strength * (2.0 * PI * harmonic_freq * t).sin();
        }
        *sample = val * decay * amplitude;
    }
    signal
}

/// Generate silence then a pluck onset.
fn onset_signal(freq: f32, silence_secs: f32, note_secs: f32, amplitude: f32) -> Vec<f32> {
    let silence = vec![0.0f32; (SAMPLE_RATE as f32 * silence_secs) as usize];
    let note = guitar_signal(freq, note_secs, amplitude);
    [silence, note].concat()
}

/// Generate a sequence of plucked notes with gaps.
fn note_sequence(
    notes: &[(f32, f32)],
    gap_secs: f32,
    amplitude: f32,
) -> (Vec<f32>, Vec<(f32, u8)>) {
    let mut signal = Vec::new();
    let mut expected = Vec::new();
    let gap = vec![0.0f32; (SAMPLE_RATE as f32 * gap_secs) as usize];

    for &(freq, duration) in notes {
        let onset_time = signal.len() as f32 / SAMPLE_RATE as f32;
        let midi = freq_to_midi_note(freq);
        expected.push((onset_time, midi));
        signal.extend(guitar_signal(freq, duration, amplitude));
        signal.extend(&gap);
    }
    (signal, expected)
}

fn freq_to_midi_note(freq: f32) -> u8 {
    (69.0 + 12.0 * (freq / 440.0).log2()).round() as u8
}

fn midi_name(midi: u8) -> String {
    const NAMES: [&str; 12] = [
        "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B",
    ];
    format!("{}{}", NAMES[(midi % 12) as usize], (midi / 12) as i8 - 1)
}

// ═══════════════════════════════════════════════════════════════════════
// 1. Open String Accuracy — the most fundamental test
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn bench_open_string_accuracy() {
    let config = default_config();
    let open_strings: [(f32, &str, u8); 6] = [
        (82.41, "E2", 40),
        (110.0, "A2", 45),
        (146.83, "D3", 50),
        (196.0, "G3", 55),
        (246.94, "B3", 59),
        (329.63, "E4", 64),
    ];

    println!("\n=== GuitarInput Pipeline: Open String Accuracy ===");

    let mut total = 0;
    let mut correct = 0;

    for &(freq, name, expected_midi) in &open_strings {
        let audio = onset_signal(freq, 0.1, 0.5, 0.6);
        let events = run_pipeline(&config, &audio);
        let ons = note_ons(&events);

        total += 1;
        if let Some(&(_, detected)) = ons.first() {
            let diff = (detected as i16 - expected_midi as i16).abs();
            if diff <= 1 {
                correct += 1;
                println!(
                    "  {} ({:.1}Hz): detected {} — OK",
                    name,
                    freq,
                    midi_name(detected)
                );
            } else {
                println!(
                    "  {} ({:.1}Hz): expected {} got {} (diff {}) — FAIL",
                    name,
                    freq,
                    midi_name(expected_midi),
                    midi_name(detected),
                    diff
                );
            }
        } else {
            println!("  {} ({:.1}Hz): NO DETECTION — FAIL", name, freq);
        }
    }

    let accuracy = correct as f32 / total as f32;
    println!("  Result: {}/{} ({:.0}%)", correct, total, accuracy * 100.0);
    assert!(
        accuracy >= 0.83,
        "Open string accuracy should be >= 83%, got {:.0}%",
        accuracy * 100.0
    );
}

// ═══════════════════════════════════════════════════════════════════════
// 2. Latency Benchmark — time from pluck to first NoteOn
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn bench_latency() {
    let config = default_config();
    let test_notes: [(f32, &str, f32); 3] = [
        (329.63, "E4 (high)", 30.0), // target: <30ms
        (196.0, "G3 (mid)", 40.0),   // target: <40ms
        (82.41, "E2 (low)", 60.0),   // target: <60ms
    ];

    println!("\n=== GuitarInput Pipeline: Latency Benchmark ===");

    for &(freq, name, target_ms) in &test_notes {
        let silence = 0.1; // 100ms silence before pluck
        let audio = onset_signal(freq, silence, 0.5, 0.6);
        let events = run_pipeline(&config, &audio);
        let ons = note_ons(&events);

        if let Some(&(time, note)) = ons.first() {
            let latency_ms = (time - silence) * 1000.0;
            println!(
                "  {}: {} at {:.1}ms (target <{:.0}ms)",
                name,
                midi_name(note),
                latency_ms,
                target_ms
            );
            // Soft assertion — log but don't fail on latency (it's a benchmark, not a gate)
            if latency_ms > target_ms {
                println!(
                    "    WARNING: exceeds target by {:.1}ms",
                    latency_ms - target_ms
                );
            }
        } else {
            println!("  {}: NO DETECTION", name);
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════
// 3. Octave Error Rate — detect correct pitch class but wrong octave
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn bench_octave_error_rate() {
    let config = default_config();

    // Test each open string for 1 second — check how often it jumps octave
    let open_strings: [(f32, &str, u8); 6] = [
        (82.41, "E2", 40),
        (110.0, "A2", 45),
        (146.83, "D3", 50),
        (196.0, "G3", 55),
        (246.94, "B3", 59),
        (329.63, "E4", 64),
    ];

    println!("\n=== GuitarInput Pipeline: Octave Error Rate ===");

    let mut total_events = 0;
    let mut octave_errors = 0;

    for &(freq, name, expected_midi) in &open_strings {
        let audio = onset_signal(freq, 0.05, 1.0, 0.6);
        let events = run_pipeline(&config, &audio);
        let ons = note_ons(&events);

        for &(_, detected) in &ons {
            total_events += 1;
            let expected_class = expected_midi % 12;
            let detected_class = detected % 12;

            if expected_class == detected_class && detected != expected_midi {
                octave_errors += 1;
                println!(
                    "  {} octave error: expected {} got {}",
                    name,
                    midi_name(expected_midi),
                    midi_name(detected)
                );
            }
        }
    }

    let error_rate = if total_events > 0 {
        octave_errors as f32 / total_events as f32
    } else {
        0.0
    };
    println!(
        "  Total events: {}, octave errors: {}, rate: {:.1}%",
        total_events,
        octave_errors,
        error_rate * 100.0
    );
    assert!(
        error_rate < 0.15,
        "Octave error rate should be < 15%, got {:.1}%",
        error_rate * 100.0
    );
}

// ═══════════════════════════════════════════════════════════════════════
// 4. Harmonic Spike Rate — false high-freq detections during sustain
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn bench_harmonic_spike_rate() {
    let config = default_config();

    println!("\n=== GuitarInput Pipeline: Harmonic Spike Rate ===");

    // Play low E for 2 seconds — check for any detections above E4
    let audio = onset_signal(82.41, 0.05, 2.0, 0.6);
    let events = run_pipeline(&config, &audio);
    let ons = note_ons(&events);

    let spikes: Vec<_> = ons.iter().filter(|&&(_, midi)| midi > 70).collect(); // Above B4 = likely harmonic
    let total = ons.len();

    println!(
        "  Low E sustained 2s: {} NoteOns, {} harmonic spikes",
        total,
        spikes.len()
    );
    for &&(time, midi) in &spikes {
        println!("    spike at {:.2}s: {} ({})", time, midi_name(midi), midi);
    }

    let spike_rate = if total > 0 {
        spikes.len() as f32 / total as f32
    } else {
        0.0
    };
    assert!(
        spike_rate < 0.2,
        "Harmonic spike rate should be < 20%, got {:.1}%",
        spike_rate * 100.0
    );
}

// ═══════════════════════════════════════════════════════════════════════
// 5. Note Sequence — multiple notes in succession
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn bench_note_sequence_accuracy() {
    let config = default_config();

    let notes = vec![
        (82.41, 0.4),  // E2
        (110.0, 0.4),  // A2
        (146.83, 0.4), // D3
        (196.0, 0.4),  // G3
        (246.94, 0.4), // B3
        (329.63, 0.4), // E4
    ];

    let (audio, expected) = note_sequence(&notes, 0.1, 0.6);
    let events = run_pipeline(&config, &audio);
    let ons = note_ons(&events);

    println!("\n=== GuitarInput Pipeline: Note Sequence ===");

    let mut matched = 0;
    for &(exp_time, exp_midi) in &expected {
        // Find a detection within 200ms of expected onset
        let found = ons
            .iter()
            .find(|&&(t, m)| (t - exp_time).abs() < 0.2 && (m as i16 - exp_midi as i16).abs() <= 1);
        if let Some(&(t, m)) = found {
            matched += 1;
            println!(
                "  {} at {:.2}s: detected {} at {:.2}s — OK",
                midi_name(exp_midi),
                exp_time,
                midi_name(m),
                t
            );
        } else {
            let nearest = ons.iter().min_by(|a, b| {
                (a.0 - exp_time)
                    .abs()
                    .partial_cmp(&(b.0 - exp_time).abs())
                    .unwrap()
            });
            println!(
                "  {} at {:.2}s: MISS (nearest: {:?})",
                midi_name(exp_midi),
                exp_time,
                nearest.map(|(t, m)| format!("{} at {:.2}s", midi_name(*m), t))
            );
        }
    }

    let accuracy = matched as f32 / expected.len() as f32;
    println!(
        "  Result: {}/{} ({:.0}%)",
        matched,
        expected.len(),
        accuracy * 100.0
    );
    assert!(
        accuracy >= 0.66,
        "Sequence accuracy should be >= 66%, got {:.0}%",
        accuracy * 100.0
    );
}

// ═══════════════════════════════════════════════════════════════════════
// 6. Noise Rejection — pipeline should NOT fire NoteOns on noise
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn bench_noise_rejection() {
    let config = default_config();

    println!("\n=== GuitarInput Pipeline: Noise Rejection ===");

    // 2 seconds of low-level noise
    let n = SAMPLE_RATE * 2;
    let mut seed = 0x6d2b_79f5_u32;
    let noise: Vec<f32> = (0..n)
        .map(|_| {
            seed ^= seed << 13;
            seed ^= seed >> 17;
            seed ^= seed << 5;
            (seed as f32 / u32::MAX as f32 - 0.5) * 0.02
        })
        .collect();

    let events = run_pipeline(&config, &noise);
    let ons = note_ons(&events);

    println!("  2s noise: {} false NoteOns", ons.len());
    for &(time, midi) in &ons {
        println!("    false trigger at {:.2}s: {}", time, midi_name(midi));
    }
    assert!(
        ons.len() <= 2,
        "Should have <= 2 false triggers on noise, got {}",
        ons.len()
    );
}
