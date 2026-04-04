//! Guitar calibration module demo.
//!
//! Run with: cargo run --example guitar_demo

use contrapunk::audio::guitar::*;

fn main() {
    println!("=== Contrapunk Guitar Module Demo ===\n");

    // Create matcher with defaults
    let mut matcher = GuitarPitchMatcher::with_defaults();

    // Standard tuning reference
    println!("Standard Tuning:");
    println!("  {:>6}  {:>4}  {:>8}", "String", "MIDI", "Freq (Hz)");
    println!("  {}", "-".repeat(22));
    for (i, &base) in STRING_BASE_PITCH.iter().enumerate() {
        println!(
            "  {:>6}  {:>4}  {:>8.2}",
            STRING_NAMES[i],
            base,
            midi_to_freq(base)
        );
    }

    // Simulate playing open strings
    println!("\n--- Simulating Open String Plucks ---\n");
    let open_string_freqs = [82.41, 110.0, 146.83, 196.0, 246.94, 329.63];
    for (i, &freq) in open_string_freqs.iter().enumerate() {
        let time = i as f32 * 0.5;
        let result = matcher.feed(time, freq, 0.85, 0.08, 0.6);
        match result {
            Some(m) => println!(
                "  Freq {:>8.2} Hz -> {} fret {} (MIDI {}) [conf: {:.0}%]",
                freq,
                STRING_NAMES[m.string_idx],
                m.fret,
                m.midi_note,
                m.confidence * 100.0
            ),
            None => println!("  Freq {:>8.2} Hz -> no match", freq),
        }
    }

    // Simulate playing a C major chord (open position)
    // x-3-2-0-1-0 = C3, E3, G3, C4, E4
    println!("\n--- Simulating C Major Chord (open position) ---\n");
    let chord_notes = [
        (1, 3, "A string fret 3 -> C3"), // A string fret 3
        (2, 2, "D string fret 2 -> E3"), // D string fret 2
        (3, 0, "G string open   -> G3"), // G string open
        (4, 1, "B string fret 1 -> C4"), // B string fret 1
        (5, 0, "E string open   -> E4"), // high E string open
    ];
    for (string_idx, fret, desc) in &chord_notes {
        let midi = STRING_BASE_PITCH[*string_idx] + *fret as u8;
        let freq = midi_to_freq(midi);
        let time = 3.0 + *string_idx as f32 * 0.02; // slight strum spread
        let result = matcher.feed(time, freq, 0.80, 0.07, 0.5);
        match result {
            Some(m) => println!(
                "  {} -> detected: {} fret {} (MIDI {} = {})",
                desc,
                STRING_NAMES[m.string_idx],
                m.fret,
                m.midi_note,
                midi_to_note_name(m.midi_note)
            ),
            None => println!("  {} -> no match", desc),
        }
    }

    // Test fret identification across strings
    println!("\n--- Fret Identification Test ---\n");
    println!(
        "  {:>10}  {:>6}  {:>4}  {:>6}  {:>4}",
        "Note", "String", "Fret", "Expect", "OK?"
    );
    println!("  {}", "-".repeat(38));
    let test_cases: Vec<(u8, usize, u8)> = vec![
        (40, 0, 0), // E2 = Low E open
        (43, 0, 3), // G2 = Low E fret 3
        (45, 1, 0), // A2 = A open
        (48, 1, 3), // C3 = A fret 3
        (50, 2, 0), // D3 = D open
        (55, 3, 0), // G3 = G open
        (57, 3, 2), // A3 = G fret 2
        (59, 4, 0), // B3 = B open
        (60, 4, 1), // C4 = B fret 1
        (64, 5, 0), // E4 = High E open
        (67, 5, 3), // G4 = High E fret 3
    ];
    for (midi, expected_string, expected_fret) in &test_cases {
        let freq = midi_to_freq(*midi);
        let fresh_matcher = GuitarPitchMatcher::with_defaults();
        let result = fresh_matcher.identify_string(*midi, 0.9);
        match result {
            Some(m) => {
                let ok = m.string_idx == *expected_string && m.fret == *expected_fret;
                println!(
                    "  {:>10}  {:>6}  {:>4}  {:>2}/{:<2}  {:>4}",
                    midi_to_note_name(*midi),
                    STRING_NAMES[m.string_idx],
                    m.fret,
                    expected_string,
                    expected_fret,
                    if ok { "OK" } else { "DIFF" }
                );
            }
            None => println!("  {:>10}  no match", midi_to_note_name(*midi)),
        }
    }

    // Test time-windowed matching
    println!("\n--- Time-Windowed Note Matching ---\n");
    let mut matcher2 = GuitarPitchMatcher::with_defaults();
    // Simulate playing E2 at time 1.0s
    matcher2.feed(1.0, 82.41, 0.9, 0.08, 0.5);
    // Simulate playing A2 at time 1.5s
    matcher2.feed(1.5, 110.0, 0.85, 0.07, 0.5);

    // Try matching expected chart notes
    let match_tests = [
        (0, 0, 1.05, "E2 at 1.05s (should hit)"),
        (0, 0, 1.50, "E2 at 1.50s (too late)"),
        (1, 0, 1.52, "A2 at 1.52s (should hit)"),
        (2, 0, 1.00, "D3 at 1.00s (wrong pitch)"),
    ];
    for (string, fret, time, desc) in &match_tests {
        let result = matcher2.try_match_note(*string, *fret, *time);
        println!(
            "  {} -> {}",
            desc,
            match result {
                Some(t) => format!("HIT at {:.2}s", t),
                None => "MISS".to_string(),
            }
        );
    }

    // Profile JSON demo
    println!("\n--- Calibration Profile ---\n");
    let profile = GuitarCalibrationProfile::default();
    let json = profile.to_json().unwrap();
    println!(
        "  Default profile: {} strings, {} bytes JSON",
        profile.strings.len(),
        json.len()
    );
    println!("  Load your TheStringTheory profile with:");
    println!("    GuitarCalibrationProfile::from_json(&std::fs::read_to_string(\"guitar_calibration_profile.json\").unwrap())");

    println!("\n=== Done ===");
}
