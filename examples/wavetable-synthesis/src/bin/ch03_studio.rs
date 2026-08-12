use std::{error::Error, path::PathBuf};

use wavetable_synthesis_exercises::{
    append_silence, apply_envelope, fir_filter, ring_modulate, splice_crossfade, write_wav,
    SineOscillator, SAMPLE_RATE,
};

#[derive(Debug)]
struct Event {
    start: usize,
    duration: usize,
    frequency_hz: f32,
    level: f32,
    attack: usize,
    release: usize,
    bright: bool,
    modulation_hz: f32,
}

fn parse_events(source: &str) -> Result<Vec<Event>, Box<dyn Error>> {
    source
        .lines()
        .skip(1)
        .map(|line| {
            let columns: Vec<_> = line.split(',').collect();
            if columns.len() != 8 {
                return Err(format!("invalid event: {line}").into());
            }
            Ok(Event {
                start: columns[0].parse()?,
                duration: columns[1].parse()?,
                frequency_hz: columns[2].parse()?,
                level: columns[3].parse()?,
                attack: columns[4].parse()?,
                release: columns[5].parse()?,
                bright: columns[6] == "bright",
                modulation_hz: columns[7].parse()?,
            })
        })
        .collect()
}

fn render(events: &[Event]) -> Vec<f32> {
    let frames = events
        .iter()
        .map(|event| event.start + event.duration)
        .max()
        .unwrap_or(0);
    let mut output = vec![0.0; frames];
    for event in events {
        let mut oscillator = SineOscillator::new();
        let mut clip: Vec<f32> = (0..event.duration)
            .map(|_| event.level * oscillator.tick(event.frequency_hz, SAMPLE_RATE as f32))
            .collect();
        apply_envelope(&mut clip, event.attack, event.release);
        if event.modulation_hz > 0.0 {
            ring_modulate(&mut clip, event.modulation_hz);
        }
        let coefficients: &[f32] = if event.bright {
            &[0.75, 0.25]
        } else {
            &[0.25, 0.5, 0.25]
        };
        for (target, sample) in output[event.start..]
            .iter_mut()
            .zip(fir_filter(&clip, coefficients))
        {
            *target += sample;
        }
    }
    output
}

fn main() -> Result<(), Box<dyn Error>> {
    let output_path = std::env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("ch03-paper-roll-study.wav"));
    if let Some(parent) = output_path
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
    {
        std::fs::create_dir_all(parent)?;
    }
    let events = parse_events(include_str!("../../data/ch03-paper-roll.csv"))?;
    let rendered = render(&events);
    let reversed: Vec<_> = rendered.iter().rev().copied().collect();
    let mut study = splice_crossfade(&rendered, &reversed, 480);
    append_silence(&mut study, 0.5);
    let peak = study.iter().copied().map(f32::abs).fold(0.0, f32::max);
    write_wav(&output_path, &study)?;
    println!(
        "events={}, frames={}, peak={peak:.6}",
        events.len(),
        study.len()
    );
    println!("wrote {}", output_path.display());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn event_roll_has_integer_schedule_and_bounded_output() {
        let events = parse_events(include_str!("../../data/ch03-paper-roll.csv")).unwrap();
        let output = render(&events);
        assert_eq!(events.len(), 5);
        assert_eq!(output.len(), 108_000);
        assert!(output
            .iter()
            .all(|sample| sample.is_finite() && sample.abs() < 1.0));
    }
}
