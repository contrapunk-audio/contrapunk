use std::{error::Error, path::PathBuf};

use wavetable_synthesis_exercises::{append_silence, render_phrase, write_wav, Note, PhraseStyle};

fn main() -> Result<(), Box<dyn Error>> {
    let output = std::env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("ch01-harmonic-melody.wav"));
    if let Some(parent) = output
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        std::fs::create_dir_all(parent)?;
    }

    // Opening of the traditional French melody "Ah! vous dirai-je, maman".
    // This is a new monophonic teaching rendition, not a borrowed recording or arrangement.
    let melody = [
        Note::new(60, 1.0),
        Note::new(60, 1.0),
        Note::new(67, 1.0),
        Note::new(67, 1.0),
        Note::new(69, 1.0),
        Note::new(69, 1.0),
        Note::new(67, 2.0),
        Note::new(65, 1.0),
        Note::new(65, 1.0),
        Note::new(64, 1.0),
        Note::new(64, 1.0),
        Note::new(62, 1.0),
        Note::new(62, 1.0),
        Note::new(60, 2.0),
    ];

    let mut audio = render_phrase(&melody, &[1.0], 108.0, PhraseStyle::detached(0.92));
    append_silence(&mut audio, 0.75);
    audio.extend(render_phrase(
        &melody,
        &[1.0, 0.5, 0.25, 0.125],
        108.0,
        PhraseStyle::detached(0.92),
    ));
    append_silence(&mut audio, 0.75);

    // Original eight-note timbre call-and-response at one fixed pitch.
    for index in 0..8 {
        let recipe: &[f32] = if index % 2 == 0 {
            &[1.0, 0.5, 0.25]
        } else {
            &[1.0, 0.0, 0.33, 0.0, 0.2]
        };
        audio.extend(render_phrase(
            &[Note::new(57, 0.5)],
            recipe,
            108.0,
            PhraseStyle::detached(0.78),
        ));
    }

    write_wav(&output, &audio)?;
    println!("wrote {}", output.display());
    Ok(())
}
