//! A/B parity tests: legacy `Synth` vs `ElixirSynthBlock`.
//!
//! Skipped entirely when the `elixir-synth` feature is off, since
//! `ElixirSynthBlock` isn't compiled.
//!
//! ### Threshold roadmap (per `ELIXIR-PLAN.md`)
//!
//! | Phase | Threshold  | Why                                       |
//! |-------|------------|-------------------------------------------|
//! | A-Cut | < -90 dBFS | Production adapter replacement gate.      |
//!
//! Each test prints the actual RMS so we can watch it drop as more
//! parity work lands.

#![cfg(feature = "elixir-synth")]

use std::sync::Arc;

use contrapunk::chain::{AudioBlock, ElixirSynthBlock, MidiBlockEvent};
use contrapunk::synth::{synth_event_channel, Synth, SynthParams, Waveform};

const SAMPLE_RATE: u32 = 48_000;
const CHANNELS: usize = 2;
const NOTE: u8 = 69; // A4 = 440 Hz
const VELOCITY: u8 = 100;
const FRAMES: usize = SAMPLE_RATE as usize; // one second
const A_CUT_RMS_DBFS_GATE: f32 = -90.0;

fn rms(samples: &[f32]) -> f32 {
    if samples.is_empty() {
        return 0.0;
    }
    let sum_sq: f64 = samples.iter().map(|s| (*s as f64).powi(2)).sum();
    (sum_sq / samples.len() as f64).sqrt() as f32
}

fn db(level: f32) -> f32 {
    if level <= 1e-12 {
        -240.0
    } else {
        20.0 * level.log10()
    }
}

fn render_legacy_sine(buf: &mut [f32]) {
    let params = Arc::new(SynthParams::default());
    params.set_waveform(Waveform::Sine);
    // Pull the LP filter as close to unity as the legacy clamp allows.
    params.set_cutoff_hz(20_000);
    params.set_resonance(0.0);
    params.set_master_gain(0.25);
    let (_tx, rx) = synth_event_channel();
    let mut s = Synth::new(params, rx, SAMPLE_RATE);
    s.midi_event(MidiBlockEvent::NoteOn {
        note: NOTE,
        velocity: VELOCITY,
    });
    s.process(buf, CHANNELS);
}

fn render_elixir_sine(buf: &mut [f32]) {
    let mut e = ElixirSynthBlock::new(SAMPLE_RATE);
    // Match the legacy LP filter setting so the test compares apples
    // to apples. With both filters at 20 kHz and no resonance the
    // 440 Hz signal sees near-identity response.
    e.set_filter_cutoff_hz(20_000.0);
    e.set_filter_resonance(0.0);
    e.midi_event(MidiBlockEvent::NoteOn {
        note: NOTE,
        velocity: VELOCITY,
    });
    e.process(buf, CHANNELS);
}

#[test]
fn both_synths_produce_audio_on_a4() {
    let mut legacy = vec![0.0f32; FRAMES * CHANNELS];
    let mut elixir = vec![0.0f32; FRAMES * CHANNELS];
    render_legacy_sine(&mut legacy);
    render_elixir_sine(&mut elixir);

    let rms_legacy = rms(&legacy);
    let rms_elixir = rms(&elixir);

    println!(
        "legacy A4 RMS : {:.4} ({:.1} dBFS)",
        rms_legacy,
        db(rms_legacy)
    );
    println!(
        "elixir A4 RMS : {:.4} ({:.1} dBFS)",
        rms_elixir,
        db(rms_elixir)
    );

    assert!(rms_legacy > 0.01, "legacy synth produced near-silence");
    assert!(rms_elixir > 0.01, "elixir synth produced near-silence");
}

#[test]
fn legacy_vs_elixir_a4_rms_meets_a_cut_gate() {
    let mut legacy = vec![0.0f32; FRAMES * CHANNELS];
    let mut elixir = vec![0.0f32; FRAMES * CHANNELS];
    render_legacy_sine(&mut legacy);
    render_elixir_sine(&mut elixir);

    let diff: Vec<f32> = legacy
        .iter()
        .zip(elixir.iter())
        .map(|(a, b)| a - b)
        .collect();
    let diff_rms = rms(&diff);
    let diff_dbfs = db(diff_rms);

    println!(
        "A/B parity: diff RMS = {:.5} ({:.2} dBFS, A-Cut gate < {:.0} dBFS)",
        diff_rms, diff_dbfs, A_CUT_RMS_DBFS_GATE
    );

    assert!(
        diff_dbfs < A_CUT_RMS_DBFS_GATE,
        "production adapter A/B diff {:.2} dBFS exceeds A-Cut gate {:.0} dBFS",
        diff_dbfs,
        A_CUT_RMS_DBFS_GATE
    );
}

#[test]
fn parity_rms_diff_metric_is_recorded() {
    // Pure measurement test — always passes, just emits the number to
    // stdout so CI can grep it. Useful for tracking parity over time
    // as A2/A3/A4 land.
    let mut legacy = vec![0.0f32; FRAMES * CHANNELS];
    let mut elixir = vec![0.0f32; FRAMES * CHANNELS];
    render_legacy_sine(&mut legacy);
    render_elixir_sine(&mut elixir);

    let diff: Vec<f32> = legacy
        .iter()
        .zip(elixir.iter())
        .map(|(a, b)| a - b)
        .collect();
    println!("PARITY_METRIC rms_dbfs={:.3}", db(rms(&diff)));
}

#[derive(Clone, Copy)]
enum SequenceEvent {
    On { note: u8, velocity: u8 },
    Off { note: u8 },
}

const SEQUENCE: [(usize, SequenceEvent); 4] = [
    (
        0,
        SequenceEvent::On {
            note: 60,
            velocity: 100,
        },
    ),
    (
        6_000,
        SequenceEvent::On {
            note: 64,
            velocity: 90,
        },
    ),
    (18_000, SequenceEvent::Off { note: 60 }),
    (30_000, SequenceEvent::Off { note: 64 }),
];
const SEQUENCE_FRAMES: usize = 60_000;

fn render_legacy_sequence(output: &mut [f32]) {
    let params = Arc::new(SynthParams::default());
    let (_tx, rx) = synth_event_channel();
    let mut synth = Synth::new(params, rx, SAMPLE_RATE);
    let mut cursor = 0;
    for (frame, event) in SEQUENCE {
        synth.process(&mut output[cursor * CHANNELS..frame * CHANNELS], CHANNELS);
        match event {
            SequenceEvent::On { note, velocity } => {
                synth.midi_event(MidiBlockEvent::NoteOn { note, velocity })
            }
            SequenceEvent::Off { note } => synth.midi_event(MidiBlockEvent::NoteOff { note }),
        }
        cursor = frame;
    }
    synth.process(&mut output[cursor * CHANNELS..], CHANNELS);
}

fn render_elixir_sequence(output: &mut [f32]) {
    let mut synth = ElixirSynthBlock::new(SAMPLE_RATE);
    let mut cursor = 0;
    for (frame, event) in SEQUENCE {
        synth.process(&mut output[cursor * CHANNELS..frame * CHANNELS], CHANNELS);
        match event {
            SequenceEvent::On { note, velocity } => {
                synth.midi_event(MidiBlockEvent::NoteOn { note, velocity })
            }
            SequenceEvent::Off { note } => synth.midi_event(MidiBlockEvent::NoteOff { note }),
        }
        cursor = frame;
    }
    synth.process(&mut output[cursor * CHANNELS..], CHANNELS);
}

#[test]
fn contrapunk_default_fixed_sequence_meets_a_cut_gate() {
    let mut legacy = vec![0.0; SEQUENCE_FRAMES * CHANNELS];
    let mut elixir = vec![0.0; SEQUENCE_FRAMES * CHANNELS];
    render_legacy_sequence(&mut legacy);
    render_elixir_sequence(&mut elixir);
    let diff: Vec<_> = legacy
        .iter()
        .zip(&elixir)
        .map(|(legacy, elixir)| legacy - elixir)
        .collect();
    let diff_dbfs = db(rms(&diff));
    println!("A-Cut fixed sequence diff RMS: {diff_dbfs:.3} dBFS");
    assert!(
        diff_dbfs < A_CUT_RMS_DBFS_GATE,
        "A-Cut diff {diff_dbfs:.3} dBFS exceeds {A_CUT_RMS_DBFS_GATE} dBFS"
    );
}
