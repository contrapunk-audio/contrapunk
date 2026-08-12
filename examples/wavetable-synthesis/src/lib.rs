//! Small cumulative DSP helpers for Chapters 1–2 of the wavetable workbook.
//!
//! The examples favor visible mathematics over production abstractions. They
//! write offline WAV files; they are not an audio-callback implementation.

use std::path::Path;

pub const SAMPLE_RATE: u32 = 48_000;

/// Equal-tempered MIDI note to frequency. Kept here so the workbook bundle is
/// runnable on its own; production Contrapunk code uses `contrapunk_dsp::pitch`.
pub fn midi_to_freq(note: u8) -> f32 {
    440.0 * 2.0_f32.powf((note as f32 - 69.0) / 12.0)
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Note {
    pub midi: u8,
    pub beats: f32,
}

impl Note {
    pub const fn new(midi: u8, beats: f32) -> Self {
        Self { midi, beats }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Connection {
    Detached { gate: f32 },
    Glide { final_portion: f32 },
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PhraseStyle {
    pub connection: Connection,
    pub accent_note: Option<usize>,
    pub final_vibrato_cents: f32,
}

impl PhraseStyle {
    pub const fn detached(gate: f32) -> Self {
        Self {
            connection: Connection::Detached { gate },
            accent_note: None,
            final_vibrato_cents: 0.0,
        }
    }

    pub const fn legato(final_portion: f32) -> Self {
        Self {
            connection: Connection::Glide { final_portion },
            accent_note: None,
            final_vibrato_cents: 0.0,
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct SineOscillator {
    phase: f32,
}

impl SineOscillator {
    pub const fn new() -> Self {
        Self { phase: 0.0 }
    }

    /// Advance phase once. Passing a new frequency each sample correctly
    /// integrates a glide or vibrato trajectory.
    pub fn tick(&mut self, frequency_hz: f32, sample_rate: f32) -> f32 {
        let sample = self.phase.sin();
        self.phase = (self.phase + std::f32::consts::TAU * frequency_hz / sample_rate)
            .rem_euclid(std::f32::consts::TAU);
        sample
    }
}

pub fn period_seconds(frequency_hz: f32) -> Option<f32> {
    (frequency_hz.is_finite() && frequency_hz > 0.0).then_some(1.0 / frequency_hz)
}

pub fn harmonic_frequency(fundamental_hz: f32, harmonic: usize) -> Option<f32> {
    (fundamental_hz.is_finite() && fundamental_hz > 0.0 && harmonic > 0)
        .then_some(fundamental_hz * harmonic as f32)
}

pub fn heterodyne_components(a_hz: f32, b_hz: f32) -> (f32, f32) {
    ((a_hz - b_hz).abs(), a_hz + b_hz)
}

pub fn cents_ratio(cents: f32) -> f32 {
    2.0_f32.powf(cents / 1_200.0)
}

pub fn log_frequency_lerp(start_hz: f32, end_hz: f32, t: f32) -> f32 {
    start_hz * (end_hz / start_hz).powf(t.clamp(0.0, 1.0))
}

/// Sum one oscillator per harmonic, skip components at or above Nyquist, and
/// normalize coefficient energy so recipes have comparable steady-state RMS.
pub fn additive_sample(
    oscillators: &mut [SineOscillator],
    amplitudes: &[f32],
    fundamental_hz: f32,
    sample_rate: f32,
) -> f32 {
    let mut sample = 0.0;
    let mut coefficient_energy = 0.0;

    for (index, (oscillator, amplitude)) in oscillators
        .iter_mut()
        .zip(amplitudes.iter().copied())
        .enumerate()
    {
        let frequency_hz = fundamental_hz * (index + 1) as f32;
        let component = oscillator.tick(frequency_hz, sample_rate);
        if frequency_hz < sample_rate / 2.0 {
            sample += amplitude * component;
            coefficient_energy += amplitude * amplitude;
        }
    }

    if coefficient_energy > 0.0 {
        sample / coefficient_energy.sqrt()
    } else {
        0.0
    }
}

pub fn render_phrase(notes: &[Note], amplitudes: &[f32], bpm: f32, style: PhraseStyle) -> Vec<f32> {
    let sample_rate = SAMPLE_RATE as f32;
    let seconds_per_beat = 60.0 / bpm;
    let mut oscillators = vec![SineOscillator::new(); amplitudes.len()];
    let mut output = Vec::new();

    for (note_index, note) in notes.iter().enumerate() {
        let frames = (note.beats * seconds_per_beat * sample_rate).round() as usize;
        let current_hz = midi_to_freq(note.midi);
        let next_hz = notes
            .get(note_index + 1)
            .map_or(current_hz, |next| midi_to_freq(next.midi));

        for frame in 0..frames {
            let position = frame as f32 / frames.max(1) as f32;
            let (frequency_hz, envelope) = match style.connection {
                Connection::Detached { gate } => {
                    let gate = gate.clamp(0.05, 1.0);
                    let active = (frames as f32 * gate) as usize;
                    let fade = (sample_rate * 0.005).min(active as f32 / 2.0) as usize;
                    let envelope = if frame >= active {
                        0.0
                    } else if frame < fade {
                        frame as f32 / fade.max(1) as f32
                    } else if frame + fade >= active {
                        (active - frame) as f32 / fade.max(1) as f32
                    } else {
                        1.0
                    };
                    (current_hz, envelope)
                }
                Connection::Glide { final_portion } => {
                    let portion = final_portion.clamp(0.01, 1.0);
                    let glide_start = 1.0 - portion;
                    let glide_t = ((position - glide_start) / portion).clamp(0.0, 1.0);
                    let mut envelope = 1.0;
                    let edge = (sample_rate * 0.005) as usize;
                    if note_index == 0 && frame < edge {
                        envelope *= frame as f32 / edge as f32;
                    }
                    if note_index + 1 == notes.len() && frame + edge >= frames {
                        envelope *= (frames - frame) as f32 / edge as f32;
                    }
                    (log_frequency_lerp(current_hz, next_hz, glide_t), envelope)
                }
            };

            let vibrato = if note_index + 1 == notes.len() && style.final_vibrato_cents != 0.0 {
                let time = output.len() as f32 / sample_rate;
                cents_ratio(style.final_vibrato_cents * (std::f32::consts::TAU * 5.0 * time).sin())
            } else {
                1.0
            };
            let accent = if style.accent_note == Some(note_index) {
                1.25
            } else {
                1.0
            };
            output.push(
                envelope
                    * accent
                    * additive_sample(
                        &mut oscillators,
                        amplitudes,
                        frequency_hz * vibrato,
                        sample_rate,
                    ),
            );
        }
    }

    output
}

pub fn append_silence(samples: &mut Vec<f32>, seconds: f32) {
    samples.resize(
        samples.len() + (seconds * SAMPLE_RATE as f32).round() as usize,
        0.0,
    );
}

pub fn write_wav(path: impl AsRef<Path>, samples: &[f32]) -> Result<(), hound::Error> {
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: SAMPLE_RATE,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut writer = hound::WavWriter::create(path, spec)?;
    for sample in samples {
        writer.write_sample(((sample * 0.25).clamp(-1.0, 1.0) * i16::MAX as f32) as i16)?;
    }
    writer.finalize()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chapter_one_math_is_executable() {
        assert_eq!(midi_to_freq(69), 440.0);
        assert_eq!(period_seconds(250.0), Some(0.004));
        assert_eq!(harmonic_frequency(110.0, 5), Some(550.0));
        assert_eq!(harmonic_frequency(110.0, 0), None);
    }

    #[test]
    fn oscillator_stays_bounded_during_retuning() {
        let mut oscillator = SineOscillator::new();
        for frequency_hz in 220..880 {
            assert!(
                oscillator
                    .tick(frequency_hz as f32, SAMPLE_RATE as f32)
                    .abs()
                    <= 1.0
            );
        }
    }

    #[test]
    fn nyquist_components_are_silent_and_recipes_are_rms_matched() {
        let mut nyquist_oscillators = [SineOscillator::new(); 2];
        for _ in 0..128 {
            assert_eq!(
                additive_sample(
                    &mut nyquist_oscillators,
                    &[0.0, 1.0],
                    12_000.0,
                    SAMPLE_RATE as f32,
                ),
                0.0
            );
        }

        fn recipe_rms(amplitudes: &[f32]) -> f32 {
            let mut oscillators = vec![SineOscillator::new(); amplitudes.len()];
            let square_sum: f32 = (0..SAMPLE_RATE)
                .map(|_| {
                    additive_sample(&mut oscillators, amplitudes, 220.0, SAMPLE_RATE as f32).powi(2)
                })
                .sum();
            (square_sum / SAMPLE_RATE as f32).sqrt()
        }

        assert!((recipe_rms(&[1.0]) - recipe_rms(&[1.0, 0.5, 0.25, 0.125])).abs() < 1.0e-4);
    }

    #[test]
    fn wav_writer_preserves_accent_headroom_and_limits_extremes(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let path = std::env::temp_dir().join(format!(
            "wavetable-synthesis-exercises-{}.wav",
            std::process::id()
        ));
        write_wav(&path, &[1.25, -1.25, 4.0])?;
        let samples: Vec<i16> = hound::WavReader::open(&path)?
            .into_samples::<i16>()
            .collect::<Result<_, _>>()?;
        std::fs::remove_file(path)?;

        let accented = (1.25 * 0.25 * i16::MAX as f32) as i16;
        assert_eq!(samples, vec![accented, -accented, i16::MAX]);
        Ok(())
    }

    #[test]
    fn chapter_two_math_is_executable() {
        assert_eq!(
            heterodyne_components(260_000.0, 259_560.0),
            (440.0, 519_560.0)
        );
        assert!((cents_ratio(1_200.0) - 2.0).abs() < 1.0e-6);
        assert!((log_frequency_lerp(220.0, 880.0, 0.5) - 440.0).abs() < 1.0e-3);
    }

    #[test]
    fn phrase_styles_keep_duration_but_change_samples() {
        let notes = [Note::new(69, 1.0), Note::new(72, 1.0)];
        let detached = render_phrase(&notes, &[1.0], 120.0, PhraseStyle::detached(0.8));
        let legato = render_phrase(&notes, &[1.0], 120.0, PhraseStyle::legato(0.25));
        assert_eq!(detached.len(), SAMPLE_RATE as usize);
        assert_eq!(detached.len(), legato.len());
        assert_ne!(detached, legato);
    }
}
