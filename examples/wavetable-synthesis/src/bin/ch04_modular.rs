use std::{error::Error, path::PathBuf};

use wavetable_synthesis_exercises::{
    bounded_feedback, midi_to_freq, write_wav, Adsr, OnePoleLowPass, SawOscillator, StepSequence,
    SAMPLE_RATE,
};

const MOTIF: [u8; 8] = [48, 55, 60, 63, 60, 55, 51, 55];

fn render() -> Vec<f32> {
    let sample_rate = SAMPLE_RATE as f32;
    let step_frames = (0.25 * sample_rate) as usize;
    let sequence = StepSequence::new(&MOTIF, step_frames, 0.8);
    let mut oscillator = SawOscillator::new();
    let mut envelope = Adsr::new(sample_rate, 0.005, 0.06, 0.6, 0.04);
    let mut filter = OnePoleLowPass::new();
    let frame_count = step_frames * MOTIF.len() * 2;
    let mut output = Vec::with_capacity(frame_count);
    let mut previous_gate = false;
    let mut previous_output = 0.0_f32;

    for frame in 0..frame_count {
        let step = sequence.at(frame);
        if step.gate != previous_gate {
            envelope.gate(step.gate);
            previous_gate = step.gate;
        }
        let env = envelope.next_sample();
        let source = oscillator.tick(midi_to_freq(step.midi), sample_rate);
        let filtered = filter.process(source, 600.0 + 2_400.0 * env, sample_rate);
        let input = bounded_feedback(filtered, previous_output, 0.35);
        let sample = (0.35 * env * input).tanh();
        previous_output = sample;
        output.push(sample);
    }
    output
}

fn main() -> Result<(), Box<dyn Error>> {
    let output_path = std::env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("ch04-modular-study.wav"));
    if let Some(parent) = output_path
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
    {
        std::fs::create_dir_all(parent)?;
    }

    let study = render();
    let peak = study.iter().copied().map(f32::abs).fold(0.0, f32::max);
    write_wav(&output_path, &study)?;
    println!("steps=16, frames={}, peak={peak:.6}", study.len());
    println!("wrote {}", output_path.display());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn modular_study_has_deterministic_length_and_headroom() {
        let output = render();
        assert_eq!(output.len(), SAMPLE_RATE as usize * 4);
        assert!(output
            .iter()
            .all(|sample| sample.is_finite() && sample.abs() <= 1.0));
        assert!(output.iter().copied().map(f32::abs).fold(0.0, f32::max) < 0.5);
        assert!((output[0] + 0.000_111_918_45).abs() < 1.0e-9);
        assert!((output[48_000] - 0.000_908_873_6).abs() < 1.0e-8);
    }
}
