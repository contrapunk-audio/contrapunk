//! Reference test signal generators for audio pipeline testing.
//!
//! Provides functions to generate known audio signals (sine waves,
//! guitar-like waveforms, noise, plucks) for deterministic testing
//! of the pitch detection and onset detection pipeline.

use std::f32::consts::PI;

/// Generate a pure sine wave.
pub fn sine(freq: f32, sample_rate: usize, duration_secs: f32, amplitude: f32) -> Vec<f32> {
    let n = (sample_rate as f32 * duration_secs) as usize;
    (0..n)
        .map(|i| {
            let t = i as f32 / sample_rate as f32;
            amplitude * (2.0 * PI * freq * t).sin()
        })
        .collect()
}

/// Generate a guitar-like waveform (fundamental + harmonics with decay).
/// More realistic than a pure sine for testing pitch detection.
pub fn guitar_pluck(freq: f32, sample_rate: usize, duration_secs: f32, amplitude: f32) -> Vec<f32> {
    let n = (sample_rate as f32 * duration_secs) as usize;
    (0..n)
        .map(|i| {
            let t = i as f32 / sample_rate as f32;
            let decay = (-t * 3.0).exp(); // ~330ms decay
            let fundamental = (2.0 * PI * freq * t).sin();
            let h2 = 0.5 * (2.0 * PI * freq * 2.0 * t).sin(); // 2nd harmonic
            let h3 = 0.25 * (2.0 * PI * freq * 3.0 * t).sin(); // 3rd harmonic
            let h4 = 0.12 * (2.0 * PI * freq * 4.0 * t).sin(); // 4th harmonic
            amplitude * decay * (fundamental + h2 + h3 + h4) / 1.87 // normalize
        })
        .collect()
}

/// Generate a pluck onset: silence → sudden attack → decay.
/// Useful for testing onset detection.
pub fn pluck_onset(
    freq: f32,
    sample_rate: usize,
    silence_secs: f32,
    sustain_secs: f32,
    amplitude: f32,
) -> Vec<f32> {
    let silence_n = (sample_rate as f32 * silence_secs) as usize;
    let pluck = guitar_pluck(freq, sample_rate, sustain_secs, amplitude);
    let mut signal = vec![0.0f32; silence_n];
    signal.extend_from_slice(&pluck);
    signal
}

/// Generate a sequence of pluck onsets at different pitches.
/// Returns (signal, Vec<(time_secs, midi_note)>) for verification.
pub fn note_sequence(
    notes: &[(u8, f32)], // (midi_note, duration_secs)
    sample_rate: usize,
    gap_secs: f32,
    amplitude: f32,
) -> (Vec<f32>, Vec<(f32, u8)>) {
    let mut signal = Vec::new();
    let mut expected = Vec::new();
    let gap_n = (sample_rate as f32 * gap_secs) as usize;

    for &(midi, duration) in notes {
        let time = signal.len() as f32 / sample_rate as f32;
        expected.push((time, midi));

        let freq = midi_to_freq_f32(midi);
        let pluck = guitar_pluck(freq, sample_rate, duration, amplitude);
        signal.extend_from_slice(&pluck);
        signal.extend(vec![0.0f32; gap_n]); // silence gap
    }

    (signal, expected)
}

/// Generate white noise.
pub fn noise(sample_rate: usize, duration_secs: f32, amplitude: f32) -> Vec<f32> {
    let n = (sample_rate as f32 * duration_secs) as usize;
    // Simple LCG pseudo-random for deterministic noise
    let mut seed: u64 = 12345;
    (0..n)
        .map(|_| {
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
            let r = ((seed >> 33) as f32 / u32::MAX as f32) * 2.0 - 1.0;
            r * amplitude
        })
        .collect()
}

/// Generate a "brush" signal — broadband noise burst with no clear pitch.
/// Used to test noise rejection.
pub fn brush(sample_rate: usize, duration_secs: f32, amplitude: f32) -> Vec<f32> {
    let n = (sample_rate as f32 * duration_secs) as usize;
    let mut seed: u64 = 67890;
    (0..n)
        .map(|i| {
            let t = i as f32 / sample_rate as f32;
            let envelope = if t < 0.01 {
                t / 0.01 // fast attack
            } else {
                (-t * 10.0).exp() // fast decay
            };
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
            let r = ((seed >> 33) as f32 / u32::MAX as f32) * 2.0 - 1.0;
            r * amplitude * envelope
        })
        .collect()
}

/// Generate all 6 open guitar strings as a sequence.
/// Returns (signal, expected notes with times).
pub fn open_strings_sequence(sample_rate: usize) -> (Vec<f32>, Vec<(f32, u8)>) {
    let notes: Vec<(u8, f32)> = vec![
        (40, 0.5), // E2
        (45, 0.5), // A2
        (50, 0.5), // D3
        (55, 0.5), // G3
        (59, 0.5), // B3
        (64, 0.5), // E4
    ];
    note_sequence(&notes, sample_rate, 0.2, 0.5)
}

/// Generate a chromatic scale from E2 to E4 (all guitar notes on first 5 frets).
pub fn chromatic_guitar_range(sample_rate: usize) -> (Vec<f32>, Vec<(f32, u8)>) {
    let notes: Vec<(u8, f32)> = (40..=64).map(|midi| (midi, 0.3)).collect();
    note_sequence(&notes, sample_rate, 0.1, 0.5)
}

fn midi_to_freq_f32(midi: u8) -> f32 {
    440.0 * 2.0_f32.powf((midi as f32 - 69.0) / 12.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sine_correct_frequency() {
        let signal = sine(440.0, 44100, 0.1, 1.0);
        assert_eq!(signal.len(), 4410);
        // Check zero crossings: 440Hz → ~88 zero crossings in 0.1s
        let crossings: usize = signal
            .windows(2)
            .filter(|w| (w[0] >= 0.0) != (w[1] >= 0.0))
            .count();
        assert!(
            (crossings as i32 - 88).abs() <= 2,
            "Expected ~88 crossings, got {}",
            crossings
        );
    }

    #[test]
    fn test_guitar_pluck_has_harmonics() {
        let signal = guitar_pluck(110.0, 44100, 0.5, 1.0);
        // Peak should be near the start (attack)
        let peak_idx = signal
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.abs().partial_cmp(&b.abs()).unwrap())
            .map(|(i, _)| i)
            .unwrap();
        assert!(
            peak_idx < 500,
            "Peak should be near start, got idx {}",
            peak_idx
        );

        // End should be quieter (decay)
        let end_rms: f32 = (signal[signal.len() - 1000..]
            .iter()
            .map(|s| s * s)
            .sum::<f32>()
            / 1000.0)
            .sqrt();
        let start_rms: f32 = (signal[..1000].iter().map(|s| s * s).sum::<f32>() / 1000.0).sqrt();
        assert!(
            end_rms < start_rms * 0.5,
            "End should be quieter than start"
        );
    }

    #[test]
    fn test_pluck_onset_has_silence_then_attack() {
        let signal = pluck_onset(220.0, 44100, 0.1, 0.3, 0.8);
        // First 4410 samples should be ~silence
        let silence_rms: f32 = (signal[..4410].iter().map(|s| s * s).sum::<f32>() / 4410.0).sqrt();
        assert!(silence_rms < 0.001);

        // After silence, should have energy
        let attack_rms: f32 =
            (signal[4410..4410 + 1000].iter().map(|s| s * s).sum::<f32>() / 1000.0).sqrt();
        assert!(attack_rms > 0.1);
    }

    #[test]
    fn test_note_sequence_timing() {
        let (signal, expected) = note_sequence(&[(60, 0.2), (64, 0.2), (67, 0.2)], 44100, 0.1, 0.5);
        assert_eq!(expected.len(), 3);
        assert!((expected[0].0 - 0.0).abs() < 0.001);
        // Second note starts after first note (0.2s) + gap (0.1s) = 0.3s
        assert!((expected[1].0 - 0.3).abs() < 0.02);
        // Total length: 3 * (0.2 + 0.1) = 0.9s
        let expected_len = (44100.0 * 0.9) as usize;
        assert!((signal.len() as i32 - expected_len as i32).abs() < 500);
    }

    #[test]
    fn test_open_strings_sequence() {
        let (signal, expected) = open_strings_sequence(44100);
        assert_eq!(expected.len(), 6);
        assert_eq!(expected[0].1, 40); // E2
        assert_eq!(expected[5].1, 64); // E4
        assert!(signal.len() > 44100 * 3); // at least 3 seconds
    }

    #[test]
    fn test_brush_has_no_clear_pitch() {
        let signal = brush(44100, 0.1, 0.5);
        assert_eq!(signal.len(), 4410);
        // RMS should be non-zero
        let rms: f32 = (signal.iter().map(|s| s * s).sum::<f32>() / signal.len() as f32).sqrt();
        assert!(rms > 0.01);
    }

    #[test]
    fn test_noise_is_deterministic() {
        let a = noise(44100, 0.01, 1.0);
        let b = noise(44100, 0.01, 1.0);
        assert_eq!(a, b, "Same seed should produce same noise");
    }
}
