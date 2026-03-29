//! Integration tests for the full audio → MIDI pipeline.
//!
//! Tests the complete chain: signal → onset detection → pitch detection →
//! note tracking → accuracy/latency metrics.
//!
//! Run with: cargo test --test audio_pipeline
//! Verbose:  cargo test --test audio_pipeline -- --nocapture

use contrapunk::audio::buffer::OverlapManager;
use contrapunk::audio::detectors::{AmdfDetector, BacfDetector, GoertzelBank};
use contrapunk::audio::guitar::*;
use contrapunk::audio::onset::PluckDetector;
use contrapunk::audio::pitch::{freq_to_midi, NoteEvent, NoteTracker, NoteTrackerConfig};
use contrapunk::audio::single_cycle::SingleCycleDetector;
use contrapunk::audio::test_signals;

use pitch_detection::detector::mcleod::McLeodDetector;
use pitch_detection::detector::PitchDetector;

const SAMPLE_RATE: usize = 44100;

// ═══════════════════════════════════════════════════════════════════════
// 1. Accuracy Tests — detect correct notes from reference signals
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn accuracy_open_strings_mcleod() {
    let (signal, expected) = test_signals::open_strings_sequence(SAMPLE_RATE);
    let results = run_mcleod_detection(&signal, SAMPLE_RATE);

    println!("\n=== Open Strings Accuracy (McLeod) ===");
    let accuracy = check_accuracy(&results, &expected, 1);
    println!("  Accuracy: {:.1}% ({}/{})", accuracy * 100.0,
        (accuracy * expected.len() as f32) as usize, expected.len());
    assert!(accuracy >= 0.83, "Open string accuracy should be >= 83%, got {:.1}%", accuracy * 100.0);
}

#[test]
fn accuracy_open_strings_goertzel() {
    let (signal, expected) = test_signals::open_strings_sequence(SAMPLE_RATE);
    let results = run_goertzel_detection(&signal, SAMPLE_RATE);

    println!("\n=== Open Strings Accuracy (Goertzel) ===");
    let accuracy = check_accuracy(&results, &expected, 2);
    println!("  Accuracy: {:.1}% ({}/{})", accuracy * 100.0,
        (accuracy * expected.len() as f32) as usize, expected.len());
    assert!(accuracy >= 0.66, "Goertzel open string accuracy should be >= 66%, got {:.1}%", accuracy * 100.0);
}

#[test]
fn accuracy_chromatic_range() {
    let (signal, expected) = test_signals::chromatic_guitar_range(SAMPLE_RATE);
    let results = run_mcleod_detection(&signal, SAMPLE_RATE);

    println!("\n=== Chromatic Range Accuracy (McLeod, E2-E4) ===");
    let mut correct = 0;
    let mut total = 0;
    for &(time, expected_midi) in &expected {
        total += 1;
        if let Some(detected) = find_detection_near(&results, time, 0.15) {
            let diff = (detected as i16 - expected_midi as i16).abs();
            if diff <= 1 {
                correct += 1;
            } else {
                println!("  MISS at {:.2}s: expected {} got {} (diff {})",
                    time, midi_name(expected_midi), midi_name(detected), diff);
            }
        } else {
            println!("  MISS at {:.2}s: expected {} — no detection", time, midi_name(expected_midi));
        }
    }
    let accuracy = correct as f32 / total as f32;
    println!("  Accuracy: {:.1}% ({}/{})", accuracy * 100.0, correct, total);
    assert!(accuracy >= 0.80, "Chromatic accuracy should be >= 80%, got {:.1}%", accuracy * 100.0);
}

// ═══════════════════════════════════════════════════════════════════════
// 2. Latency Tests — how quickly does the pipeline detect a note?
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn latency_high_string() {
    // High E (329.6 Hz) — should be detected faster than low strings
    let signal = test_signals::pluck_onset(329.63, SAMPLE_RATE, 0.1, 0.5, 0.8);
    let onset_time = 0.1; // silence ends at 0.1s

    let first_detection = run_mcleod_first_detection(&signal, SAMPLE_RATE);

    println!("\n=== Latency: High E ===");
    match first_detection {
        Some((time, note)) => {
            let latency_ms = (time - onset_time) * 1000.0;
            println!("  Detected {} at {:.3}s (latency: {:.1}ms)", midi_name(note), time, latency_ms);
            assert!(latency_ms < 50.0, "High string latency should be < 50ms, got {:.1}ms", latency_ms);
        }
        None => panic!("Failed to detect High E at all"),
    }
}

#[test]
fn latency_low_string() {
    // Low E (82.4 Hz) — needs more cycles, higher latency expected
    let signal = test_signals::pluck_onset(82.41, SAMPLE_RATE, 0.1, 0.5, 0.8);
    let onset_time = 0.1;

    let first_detection = run_mcleod_first_detection(&signal, SAMPLE_RATE);

    println!("\n=== Latency: Low E ===");
    match first_detection {
        Some((time, note)) => {
            let latency_ms = (time - onset_time) * 1000.0;
            println!("  Detected {} at {:.3}s (latency: {:.1}ms)", midi_name(note), time, latency_ms);
            assert!(latency_ms < 80.0, "Low string latency should be < 80ms, got {:.1}ms", latency_ms);
        }
        None => panic!("Failed to detect Low E at all"),
    }
}

// ═══════════════════════════════════════════════════════════════════════
// 3. Noise Rejection Tests — should NOT detect notes from noise
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn noise_rejection_white_noise() {
    let signal = test_signals::noise(SAMPLE_RATE, 1.0, 0.1);
    let results = run_mcleod_detection(&signal, SAMPLE_RATE);

    println!("\n=== Noise Rejection: White Noise ===");
    let guitar_range_detections: Vec<_> = results.iter()
        .filter(|&&(_, midi)| midi >= 40 && midi <= 88)
        .collect();
    let false_trigger_rate = guitar_range_detections.len() as f32
        / (results.len().max(1) as f32);
    println!("  Total detections: {}, in guitar range: {}, false trigger rate: {:.1}%",
        results.len(), guitar_range_detections.len(), false_trigger_rate * 100.0);
    // Some false detections are OK but should be low
    assert!(guitar_range_detections.len() <= 5,
        "Too many false triggers from noise: {}", guitar_range_detections.len());
}

#[test]
fn noise_rejection_brush() {
    let signal = test_signals::brush(SAMPLE_RATE, 0.5, 0.3);
    let normalizer = AudioNormalizer::default_uncalibrated();

    // Run normalizer on brush signal
    let frame_size = 1024;
    let mut rejected = 0;
    let mut total = 0;

    for chunk in signal.chunks(frame_size) {
        if chunk.len() < frame_size { break; }
        let rms = (chunk.iter().map(|s| s * s).sum::<f32>() / frame_size as f32).sqrt();
        let half = frame_size / 2;
        let bright_rms = (chunk[half..].iter().map(|s| s * s).sum::<f32>() / half as f32).sqrt();
        total += 1;

        // Low confidence for noise
        match normalizer.normalize(rms, bright_rms, 0.2, None) {
            NormalizeResult::Rejected => rejected += 1,
            _ => {}
        }
    }

    println!("\n=== Noise Rejection: Brush ===");
    let rejection_rate = rejected as f32 / total.max(1) as f32;
    println!("  Frames: {}, rejected: {}, rate: {:.1}%", total, rejected, rejection_rate * 100.0);
    assert!(rejection_rate >= 0.7, "Brush rejection should be >= 70%, got {:.1}%", rejection_rate * 100.0);
}

// ═══════════════════════════════════════════════════════════════════════
// 4. Onset Detection Tests
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn onset_detects_pluck_in_silence() {
    let signal = test_signals::pluck_onset(220.0, SAMPLE_RATE, 0.2, 0.5, 0.6);
    let onset_time = 0.2;

    let mut pluck_det = PluckDetector::new(513); // 1024/2 + 1
    let frame_size = 1024;
    let mut first_onset: Option<f32> = None;

    for (i, chunk) in signal.chunks(frame_size).enumerate() {
        if chunk.len() < frame_size { break; }
        let spectrum = simple_mag_spectrum(chunk);
        if pluck_det.feed(&spectrum) && first_onset.is_none() {
            first_onset = Some(i as f32 * frame_size as f32 / SAMPLE_RATE as f32);
        }
    }

    println!("\n=== Onset Detection: Pluck in Silence ===");
    match first_onset {
        Some(t) => {
            let latency_ms = (t - onset_time) * 1000.0;
            println!("  Onset at {:.3}s (latency: {:.1}ms from actual onset at {:.1}s)", t, latency_ms, onset_time);
            assert!(latency_ms < 60.0 && latency_ms > -30.0,
                "Onset latency should be reasonable, got {:.1}ms", latency_ms);
        }
        None => panic!("Pluck onset not detected"),
    }
}

#[test]
fn onset_no_false_trigger_on_silence() {
    let signal = vec![0.0f32; SAMPLE_RATE]; // 1 second of silence

    let mut pluck_det = PluckDetector::new(513);
    let frame_size = 1024;
    let mut onset_count = 0;

    for chunk in signal.chunks(frame_size) {
        if chunk.len() < frame_size { break; }
        let spectrum = simple_mag_spectrum(chunk);
        if pluck_det.feed(&spectrum) {
            onset_count += 1;
        }
    }

    println!("\n=== Onset: No False Trigger on Silence ===");
    println!("  Onsets detected: {}", onset_count);
    assert_eq!(onset_count, 0, "Should detect 0 onsets in silence");
}

// ═══════════════════════════════════════════════════════════════════════
// 5. Detector Comparison — all detectors on same signal
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn detector_comparison_440hz() {
    let signal = test_signals::sine(440.0, SAMPLE_RATE, 0.2, 0.8);

    // McLeod
    let mcleod_freq = {
        let samples: Vec<f64> = signal.iter().map(|&s| s as f64).collect();
        let mut det = McLeodDetector::new(1024, 512);
        det.get_pitch(&samples[..1024], SAMPLE_RATE, 0.3, 0.3)
            .map(|p| p.frequency as f32)
    };

    // BACF
    let bacf_freq = {
        let mut det = BacfDetector::new();
        det.detect(&signal[..2048], SAMPLE_RATE).map(|(f, _)| f)
    };

    // AMDF
    let amdf_freq = {
        let mut det = AmdfDetector::new();
        det.detect(&signal[..2048], SAMPLE_RATE).map(|(f, _)| f)
    };

    // Goertzel
    let goertzel_note = {
        let mut bank = GoertzelBank::new(SAMPLE_RATE);
        bank.analyze(&signal[..1024]).map(|(midi, _)| midi)
    };

    // SingleCycle
    let sc_freq = {
        let mut det = SingleCycleDetector::new(SAMPLE_RATE, 60.0, 2000.0);
        let mut last_freq = None;
        for &s in &signal[..2000] {
            if let Some(r) = det.feed_sample(s) {
                last_freq = Some(r.frequency);
            }
        }
        last_freq
    };

    println!("\n=== Detector Comparison: 440 Hz ===");
    println!("  McLeod:      {:>8}", mcleod_freq.map(|f| format!("{:.1} Hz", f)).unwrap_or("FAIL".into()));
    println!("  BACF:        {:>8}", bacf_freq.map(|f| format!("{:.1} Hz", f)).unwrap_or("FAIL".into()));
    println!("  AMDF:        {:>8}", amdf_freq.map(|f| format!("{:.1} Hz", f)).unwrap_or("FAIL".into()));
    println!("  Goertzel:    {:>8}", goertzel_note.map(|n| format!("MIDI {} ({})", n, midi_name(n))).unwrap_or("FAIL".into()));
    println!("  SingleCycle: {:>8}", sc_freq.map(|f| format!("{:.1} Hz", f)).unwrap_or("FAIL".into()));

    // All should be within 5Hz of 440
    if let Some(f) = mcleod_freq { assert!((f - 440.0).abs() < 5.0, "McLeod: {}", f); }
    if let Some(f) = bacf_freq { assert!((f - 440.0).abs() < 5.0, "BACF: {}", f); }
    if let Some(f) = amdf_freq { assert!((f - 440.0).abs() < 5.0, "AMDF: {}", f); }
    if let Some(n) = goertzel_note { assert_eq!(n, 69, "Goertzel should detect A4=MIDI 69"); }
}

#[test]
fn detector_comparison_all_open_strings() {
    let open_strings: [(u8, f32, &str); 6] = [
        (40, 82.41, "E2"), (45, 110.0, "A2"), (50, 146.83, "D3"),
        (55, 196.0, "G3"), (59, 246.94, "B3"), (64, 329.63, "E4"),
    ];

    println!("\n=== Detector Comparison: All Open Strings ===");
    println!("  {:>4}  {:>8}  {:>8}  {:>8}  {:>8}  {:>8}",
        "Note", "McLeod", "BACF", "AMDF", "Goertzel", "SngCyc");

    let mut all_pass = true;

    for &(midi, freq, name) in &open_strings {
        let signal = test_signals::guitar_pluck(freq, SAMPLE_RATE, 0.3, 0.8);

        let mcleod = {
            let s: Vec<f64> = signal[..1024].iter().map(|&v| v as f64).collect();
            let mut d = McLeodDetector::new(1024, 512);
            d.get_pitch(&s, SAMPLE_RATE, 0.3, 0.3).map(|p| freq_to_midi(p.frequency as f32).0)
        };

        let bacf = {
            let mut d = BacfDetector::new();
            d.detect(&signal[..2048], SAMPLE_RATE).map(|(f, _)| freq_to_midi(f).0)
        };

        let amdf = {
            let mut d = AmdfDetector::new();
            d.detect(&signal[..2048], SAMPLE_RATE).map(|(f, _)| freq_to_midi(f).0)
        };

        let goertzel = {
            let mut b = GoertzelBank::new(SAMPLE_RATE);
            b.analyze(&signal[..1024]).map(|(m, _)| m)
        };

        let sc = {
            let mut d = SingleCycleDetector::new(SAMPLE_RATE, 60.0, 2000.0);
            let mut last = None;
            for &s in &signal[..4000.min(signal.len())] {
                if let Some(r) = d.feed_sample(s) { last = Some(freq_to_midi(r.frequency).0); }
            }
            last
        };

        let mut fmt = |r: Option<u8>| -> String {
            match r {
                Some(n) if (n as i16 - midi as i16).abs() <= 1 => format!("{}  ", midi_name(n)),
                Some(n) => { all_pass = false; format!("{}!!", midi_name(n)) }
                None => { all_pass = false; "FAIL".into() }
            }
        };

        println!("  {:>4}  {:>8}  {:>8}  {:>8}  {:>8}  {:>8}",
            name, fmt(mcleod), fmt(bacf), fmt(amdf), fmt(goertzel), fmt(sc));
    }

    assert!(all_pass, "Some detectors failed on open strings");
}

// ═══════════════════════════════════════════════════════════════════════
// 6. NoteTracker Integration Tests
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn note_tracker_rejects_octave_glitch() {
    let mut tracker = NoteTracker::new(NoteTrackerConfig {
        onset_required: false,
        octave_jump_confidence_threshold: 0.6,
    });

    // Establish E2
    tracker.update(40, 0.9);
    tracker.update(40, 0.9);
    assert_eq!(tracker.current_note(), Some(40));

    // Glitch to E3 (octave) with low confidence — should be rejected
    assert!(tracker.update(52, 0.3).is_none());
    assert!(tracker.update(52, 0.3).is_none());
    assert_eq!(tracker.current_note(), Some(40), "Octave glitch should be rejected");
}

#[test]
fn note_tracker_voting_absorbs_single_glitch() {
    let mut tracker = NoteTracker::new(NoteTrackerConfig {
        onset_required: false,
        octave_jump_confidence_threshold: 0.6,
    });

    tracker.update(60, 0.9);
    tracker.update(60, 0.9); // note established
    assert_eq!(tracker.current_note(), Some(60));

    // Single frame glitch
    tracker.update(65, 0.9);
    // Back to normal
    tracker.update(60, 0.9);

    assert_eq!(tracker.current_note(), Some(60), "Single glitch should be absorbed by voting");
}

// ═══════════════════════════════════════════════════════════════════════
// 7. Full Pipeline Test — onset → detect → track → output
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn full_pipeline_note_sequence() {
    let notes = vec![(40, 0.4), (45, 0.4), (50, 0.4), (55, 0.4)]; // E2 A2 D3 G3
    let (signal, expected) = test_signals::note_sequence(&notes, SAMPLE_RATE, 0.2, 0.6);

    let mut overlap = OverlapManager::new(1024, 75);
    let mut pluck_det = PluckDetector::new(513);
    let mut tracker = NoteTracker::new(NoteTrackerConfig {
        onset_required: true,
        octave_jump_confidence_threshold: 0.6,
    });

    let mut detected_notes: Vec<(f32, u8)> = Vec::new();
    let mut silence_frames = 0u32;

    for (i, sample) in signal.iter().enumerate() {
        let frames = overlap.feed(&[*sample]);
        for frame in frames {
            let time = i as f32 / SAMPLE_RATE as f32;
            let rms = (frame.iter().map(|s| s * s).sum::<f32>() / frame.len() as f32).sqrt();

            if rms < 0.01 {
                silence_frames += 1;
                if silence_frames >= 8 {
                    if let Some(NoteEvent::NoteOff(_)) = tracker.note_off() {}
                }
                continue;
            }
            silence_frames = 0;

            // Onset
            let spectrum = simple_mag_spectrum(&frame);
            if pluck_det.feed(&spectrum) {
                tracker.signal_onset();
            }

            // Pitch
            let samples_f64: Vec<f64> = frame.iter().map(|&s| s as f64).collect();
            let mut det = McLeodDetector::new(frame.len(), frame.len() / 2);
            if let Some(p) = det.get_pitch(&samples_f64, SAMPLE_RATE, 0.3, 0.3) {
                if p.clarity > 0.5 {
                    let (midi, _) = freq_to_midi(p.frequency as f32);
                    if let Some(NoteEvent::NoteOn(note)) = tracker.update(midi, p.clarity as f32) {
                        detected_notes.push((time, note));
                    }
                }
            }
        }
    }

    println!("\n=== Full Pipeline: Note Sequence ===");
    println!("  Expected: {:?}", expected.iter().map(|(t, n)| format!("{:.1}s:{}", t, midi_name(*n))).collect::<Vec<_>>());
    println!("  Detected: {:?}", detected_notes.iter().map(|(t, n)| format!("{:.1}s:{}", t, midi_name(*n))).collect::<Vec<_>>());

    // Should detect at least 3 of 4 notes
    let mut matched = 0;
    for &(_, exp_midi) in &expected {
        if detected_notes.iter().any(|&(_, det_midi)| (det_midi as i16 - exp_midi as i16).abs() <= 1) {
            matched += 1;
        }
    }
    println!("  Matched: {}/{}", matched, expected.len());
    assert!(matched >= 3, "Should match at least 3/4 notes, got {}", matched);
}

// ═══════════════════════════════════════════════════════════════════════
// Helpers
// ═══════════════════════════════════════════════════════════════════════

fn run_mcleod_detection(signal: &[f32], sample_rate: usize) -> Vec<(f32, u8)> {
    let frame_size = 1024;
    let hop = 256;
    let mut results = Vec::new();

    let mut offset = 0;
    while offset + frame_size <= signal.len() {
        let chunk = &signal[offset..offset + frame_size];
        let rms = (chunk.iter().map(|s| s * s).sum::<f32>() / frame_size as f32).sqrt();

        if rms > 0.01 {
            let samples: Vec<f64> = chunk.iter().map(|&s| s as f64).collect();
            let mut det = McLeodDetector::new(frame_size, frame_size / 2);
            if let Some(p) = det.get_pitch(&samples, sample_rate, 0.3, 0.3) {
                if p.clarity > 0.5 {
                    let (midi, _) = freq_to_midi(p.frequency as f32);
                    let time = offset as f32 / sample_rate as f32;
                    results.push((time, midi));
                }
            }
        }
        offset += hop;
    }
    results
}

fn run_goertzel_detection(signal: &[f32], sample_rate: usize) -> Vec<(f32, u8)> {
    let frame_size = 1024;
    let hop = 256;
    let mut bank = GoertzelBank::new(sample_rate);
    let mut results = Vec::new();

    let mut offset = 0;
    while offset + frame_size <= signal.len() {
        let chunk = &signal[offset..offset + frame_size];
        let rms = (chunk.iter().map(|s| s * s).sum::<f32>() / frame_size as f32).sqrt();

        if rms > 0.01 {
            if let Some((midi, _)) = bank.analyze(chunk) {
                let time = offset as f32 / sample_rate as f32;
                results.push((time, midi));
            }
        }
        offset += hop;
    }
    results
}

fn run_mcleod_first_detection(signal: &[f32], sample_rate: usize) -> Option<(f32, u8)> {
    let frame_size = 1024;
    let hop = 256;

    let mut offset = 0;
    while offset + frame_size <= signal.len() {
        let chunk = &signal[offset..offset + frame_size];
        let rms = (chunk.iter().map(|s| s * s).sum::<f32>() / frame_size as f32).sqrt();

        if rms > 0.01 {
            let samples: Vec<f64> = chunk.iter().map(|&s| s as f64).collect();
            let mut det = McLeodDetector::new(frame_size, frame_size / 2);
            if let Some(p) = det.get_pitch(&samples, sample_rate, 0.3, 0.3) {
                if p.clarity > 0.5 {
                    let (midi, _) = freq_to_midi(p.frequency as f32);
                    if midi >= 40 && midi <= 88 {
                        return Some((offset as f32 / sample_rate as f32, midi));
                    }
                }
            }
        }
        offset += hop;
    }
    None
}

fn find_detection_near(results: &[(f32, u8)], time: f32, window: f32) -> Option<u8> {
    results.iter()
        .filter(|&&(t, _)| (t - time).abs() < window)
        .max_by(|a, b| a.0.partial_cmp(&b.0).unwrap())
        .map(|&(_, midi)| midi)
}

fn check_accuracy(results: &[(f32, u8)], expected: &[(f32, u8)], tolerance: u8) -> f32 {
    let mut correct = 0;
    for &(time, exp_midi) in expected {
        if let Some(det_midi) = find_detection_near(results, time, 0.2) {
            if (det_midi as i16 - exp_midi as i16).unsigned_abs() <= tolerance as u16 {
                correct += 1;
            }
        }
    }
    correct as f32 / expected.len().max(1) as f32
}

fn midi_name(midi: u8) -> String {
    const NAMES: [&str; 12] = ["C","C#","D","D#","E","F","F#","G","G#","A","A#","B"];
    format!("{}{}", NAMES[(midi % 12) as usize], (midi / 12) as i8 - 1)
}

fn simple_mag_spectrum(samples: &[f32]) -> Vec<f32> {
    let n = samples.len();
    let num_bins = n / 2 + 1;
    let mut mags = vec![0.0f32; num_bins];
    let bin_size = (n / num_bins).max(1);
    for (i, mag) in mags.iter_mut().enumerate() {
        let start = i * bin_size;
        let end = (start + bin_size).min(n);
        if start < end {
            *mag = (samples[start..end].iter().map(|s| s * s).sum::<f32>() / (end - start) as f32).sqrt();
        }
    }
    mags
}
