use std::{error::Error, path::PathBuf};

use wavetable_synthesis_exercises::{
    append_silence, heterodyne_components, render_phrase, write_wav, Note, PhraseStyle,
    SineOscillator, SAMPLE_RATE,
};

fn render_scaled_pair(multiply: bool) -> Vec<f32> {
    let mut fixed = SineOscillator::new();
    let mut variable = SineOscillator::new();
    (0..SAMPLE_RATE)
        .map(|_| {
            let a = fixed.tick(1_000.0, SAMPLE_RATE as f32);
            let b = variable.tick(560.0, SAMPLE_RATE as f32);
            if multiply {
                a * b
            } else {
                0.5 * (a + b)
            }
        })
        .collect()
}

fn main() -> Result<(), Box<dyn Error>> {
    let output = std::env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("ch02-gesture-phrase.wav"));
    if let Some(parent) = output
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        std::fs::create_dir_all(parent)?;
    }

    let (difference_hz, sum_hz) = heterodyne_components(300_000.0, 299_560.0);
    println!("RF model: difference={difference_hz:.0} Hz, sum={sum_hz:.0} Hz");

    // Linear addition retains 1000 and 560 Hz; multiplication creates 440 and 1560 Hz.
    // Real theremin RF oscillators are above the 48 kHz teaching renderer's Nyquist limit.
    let mut audio = render_scaled_pair(false);
    append_silence(&mut audio, 0.4);
    audio.extend(render_scaled_pair(true));
    append_silence(&mut audio, 0.75);

    // Pitch incipit 5-1-3-1-3-2-1-6-5-5 of the public-domain NEW BRITAIN
    // tune. Durations are simplified for this original articulation study.
    let melody = [
        Note::new(67, 0.5),
        Note::new(72, 1.5),
        Note::new(76, 0.5),
        Note::new(72, 0.5),
        Note::new(76, 1.5),
        Note::new(74, 0.5),
        Note::new(72, 0.5),
        Note::new(69, 1.5),
        Note::new(67, 0.5),
        Note::new(67, 1.0),
    ];
    let mut continuous = PhraseStyle::legato(0.35);
    continuous.final_vibrato_cents = 18.0;
    audio.extend(render_phrase(&melody, &[1.0, 0.18], 84.0, continuous));
    append_silence(&mut audio, 0.75);

    let mut articulated = PhraseStyle::detached(0.76);
    articulated.accent_note = Some(4);
    articulated.final_vibrato_cents = 18.0;
    audio.extend(render_phrase(&melody, &[1.0, 0.18], 84.0, articulated));

    write_wav(&output, &audio)?;
    println!("wrote {}", output.display());
    Ok(())
}
