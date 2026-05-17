//! Elixir standalone — desktop demo (Phase 21.B0 + B1).
//!
//! Opens the default cpal output device and routes MIDI input into
//! `elixir_core::Engine`. Behaviour:
//!
//! - If at least one MIDI input port is available, the first one is
//!   opened and the binary stays open until Ctrl-C, playing whatever
//!   you send it.
//! - If no MIDI input is found (or `--demo` is passed), runs a hardcoded
//!   C-major arpeggio + A4 reference tone and exits.
//!
//! Notes:
//! - Mutex-guarded engine on the audio callback is demo-grade. Real
//!   threading lands in B4 (lock-free param atoms + SPSC event ringbuf).
//! - Only `f32` sample formats are handled; non-f32 devices error out.

use std::env;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::SampleFormat;
use midir::{Ignore, MidiInput};

use elixir_core::fx::{Delay, Drive, FxSlot, Reverb};
use elixir_core::modulation::{ModDest, ModRoute, ModSrc};
use elixir_core::Engine;

fn main() -> anyhow::Result<()> {
    println!("Elixir standalone v0.0.1 — Phase 21.A1 + B1");

    let args: Vec<String> = env::args().collect();
    let force_demo = args.iter().any(|a| a == "--demo");
    let list_only = args.iter().any(|a| a == "--list");

    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .ok_or_else(|| anyhow::anyhow!("no default output device available"))?;
    let supported = device.default_output_config()?;
    let sample_rate = supported.sample_rate();
    let channels = supported.channels() as usize;
    let format = supported.sample_format();
    #[allow(deprecated)]
    let device_name = device.name().unwrap_or_else(|_| "<unknown>".into());
    let stream_config: cpal::StreamConfig = supported.into();

    println!();
    println!("  device       : {}", device_name);
    println!("  sample rate  : {} Hz", sample_rate);
    println!("  channels     : {}", channels);
    println!("  sample fmt   : {:?}", format);
    println!();

    if format != SampleFormat::F32 {
        anyhow::bail!(
            "this demo only supports f32 sample format (device reported {:?})",
            format
        );
    }

    let engine = Arc::new(Mutex::new(Engine::new()));
    {
        let mut e = engine.lock().unwrap();
        e.prepare(sample_rate, 2048);
        e.set_master_gain(0.30);
    }

    let engine_cb = Arc::clone(&engine);
    let stream = device.build_output_stream(
        &stream_config,
        move |buffer: &mut [f32], _info: &cpal::OutputCallbackInfo| {
            if let Ok(mut e) = engine_cb.lock() {
                e.process(buffer, channels);
            } else {
                for s in buffer.iter_mut() {
                    *s = 0.0;
                }
            }
        },
        |err| eprintln!("audio stream error: {err}"),
        None,
    )?;
    stream.play()?;

    // Enumerate MIDI inputs.
    let midi_ports = list_midi_ports();
    if list_only {
        return Ok(());
    }

    if force_demo || midi_ports.is_empty() {
        if midi_ports.is_empty() {
            println!("(no MIDI input ports detected — running canned demo)");
        }
        run_demo(&engine);
        return Ok(());
    }

    // Connect to first MIDI input.
    let (port_idx, port_name) = (0usize, midi_ports[0].clone());
    let midi_in = MidiInput::new("elixir-standalone")?;
    let port =
        midi_in.ports().get(port_idx).cloned().ok_or_else(|| {
            anyhow::anyhow!("MIDI port disappeared between enumeration and connect")
        })?;

    println!("► MIDI input: {}", port_name);
    println!("  Play notes on your MIDI controller. Press Ctrl-C to exit.");
    println!();

    let engine_midi = Arc::clone(&engine);
    let _conn = midi_in
        .connect(
            &port,
            "elixir-in",
            move |_ts, msg, _| {
                handle_midi(msg, &engine_midi);
            },
            (),
        )
        .map_err(|e| anyhow::anyhow!("midir connect failed: {}", e))?;

    // Sleep forever; audio + MIDI callbacks run on their own threads.
    loop {
        thread::sleep(Duration::from_secs(3600));
    }
}

/// Enumerate MIDI input ports and print them. Returns the list of port
/// names (parallel to `MidiInput::ports()`).
fn list_midi_ports() -> Vec<String> {
    let mut midi_in = match MidiInput::new("elixir-standalone-enum") {
        Ok(m) => m,
        Err(_) => return vec![],
    };
    midi_in.ignore(Ignore::None);
    let ports = midi_in.ports();
    if ports.is_empty() {
        return vec![];
    }
    println!("MIDI input ports:");
    let mut names = Vec::with_capacity(ports.len());
    for (i, p) in ports.iter().enumerate() {
        let name = midi_in
            .port_name(p)
            .unwrap_or_else(|_| format!("port-{}", i));
        println!("  [{}] {}", i, name);
        names.push(name);
    }
    println!();
    names
}

/// Parse a MIDI message and drive the engine.
fn handle_midi(msg: &[u8], engine: &Arc<Mutex<Engine>>) {
    if msg.is_empty() {
        return;
    }
    let status = msg[0] & 0xF0;
    match status {
        0x90 => {
            // Note-on. Velocity 0 = note-off per MIDI 1.0.
            if msg.len() >= 3 {
                let note = msg[1];
                let vel = msg[2];
                if let Ok(mut e) = engine.lock() {
                    if vel == 0 {
                        e.note_off(note);
                    } else {
                        e.note_on(note, vel);
                    }
                }
            }
        }
        0x80 => {
            // Note-off.
            if msg.len() >= 3 {
                let note = msg[1];
                if let Ok(mut e) = engine.lock() {
                    e.note_off(note);
                }
            }
        }
        0xB0 => {
            // Control Change. We care about:
            //   CC 64 = sustain pedal (>= 64 → on, < 64 → off)
            //   CC 120 = all sound off, CC 123 = all notes off
            if msg.len() >= 3 {
                let cc = msg[1];
                let val = msg[2];
                if let Ok(mut e) = engine.lock() {
                    match cc {
                        64 => e.set_sustain_pedal(val >= 64),
                        120 | 123 => e.all_notes_off(),
                        _ => {}
                    }
                }
            }
        }
        _ => {}
    }
}

fn run_demo(engine: &Arc<Mutex<Engine>>) {
    println!("► C major arpeggio: C E G C (300 ms each, mono)");
    for &note in &[60u8, 64, 67, 72] {
        send_note(engine, note, 100, Duration::from_millis(300));
    }

    thread::sleep(Duration::from_millis(200));

    println!("► C major chord (C E G C, 4-voice polyphony) — 1500 ms");
    {
        let mut e = engine.lock().unwrap();
        for &n in &[60u8, 64, 67, 72] {
            e.note_on(n, 90);
        }
    }
    thread::sleep(Duration::from_millis(1500));
    {
        let mut e = engine.lock().unwrap();
        for &n in &[60u8, 64, 67, 72] {
            e.note_off(n);
        }
    }
    thread::sleep(Duration::from_millis(400));

    println!("► Sustain pedal demo: hold C, lift fingers, release pedal");
    {
        let mut e = engine.lock().unwrap();
        e.set_sustain_pedal(true);
        e.note_on(60, 100);
    }
    thread::sleep(Duration::from_millis(400));
    {
        let mut e = engine.lock().unwrap();
        e.note_off(60); // key up, but pedal still down → still rings
    }
    thread::sleep(Duration::from_millis(800));
    {
        let mut e = engine.lock().unwrap();
        e.set_sustain_pedal(false); // pedal up → release
    }
    thread::sleep(Duration::from_millis(500));

    println!("► Tremolo via LFO→MasterGain @ 8 Hz, amount 0.5 (2.5 s)");
    let lfo_route_idx = {
        let mut e = engine.lock().unwrap();
        if let Some(lfo) = e.lfo_mut(0) {
            lfo.set_rate_hz(8.0);
        }
        let idx = e
            .add_mod_route(ModRoute::new(ModSrc::Lfo(0), ModDest::MasterGain, 0.5))
            .expect("matrix has room");
        e.note_on(72, 100);
        idx
    };
    thread::sleep(Duration::from_millis(2500));
    {
        let mut e = engine.lock().unwrap();
        e.note_off(72);
        e.remove_mod_route(lfo_route_idx);
    }
    thread::sleep(Duration::from_millis(400));

    println!("► Filter sweep: LFO→Cutoff @ 0.6 Hz, amount 3 kHz, base 1.5 kHz");
    let sweep_route_idx = {
        let mut e = engine.lock().unwrap();
        if let Some(lfo) = e.lfo_mut(0) {
            lfo.set_rate_hz(0.6);
        }
        e.set_filter_cutoff_hz(1_500.0);
        e.set_filter_resonance(0.55);
        let idx = e
            .add_mod_route(ModRoute::new(
                ModSrc::Lfo(0),
                ModDest::FilterCutoff,
                3_000.0,
            ))
            .expect("matrix has room");
        // A C-major chord with the filter sweeping — classic synth bread
        // and butter.
        for &n in &[48u8, 52, 55, 60, 64, 67] {
            e.note_on(n, 80);
        }
        idx
    };
    thread::sleep(Duration::from_millis(4500));
    {
        let mut e = engine.lock().unwrap();
        for &n in &[48u8, 52, 55, 60, 64, 67] {
            e.note_off(n);
        }
        e.remove_mod_route(sweep_route_idx);
        e.set_filter_cutoff_hz(8_000.0);
        e.set_filter_resonance(0.0);
    }
    thread::sleep(Duration::from_millis(600));

    println!("► FX chain: Drive (gentle) → Delay (3/8) → Reverb (hall)");
    {
        let mut e = engine.lock().unwrap();
        let mut drive = Drive::with_drive(2.5);
        drive.mix = 0.4;
        let mut delay = Delay::new(48_000); // up to 1 s @ 48 kHz
        delay.set_delay_secs(0.375, 48_000.0); // 3/8 of a bar at 120 BPM
        delay.set_feedback(0.55);
        delay.set_mix(0.30);
        let mut reverb = Reverb::new(48_000.0);
        reverb.set_decay(0.88);
        reverb.set_damping(0.4);
        reverb.set_mix(0.35);
        e.set_fx_slot(0, FxSlot::Drive(drive));
        e.set_fx_slot(1, FxSlot::Delay(delay));
        e.set_fx_slot(2, FxSlot::Reverb(reverb));
        e.set_master_gain(0.22); // headroom for FX summing
    }
    // Pluck a melody over the FX
    for &n in &[60u8, 64, 67, 72, 67, 64, 60] {
        send_note(engine, n, 100, Duration::from_millis(180));
    }
    // Hold a chord and let the tail bloom
    {
        let mut e = engine.lock().unwrap();
        for &n in &[55u8, 59, 62, 67] {
            e.note_on(n, 90);
        }
    }
    thread::sleep(Duration::from_millis(2200));
    {
        let mut e = engine.lock().unwrap();
        for &n in &[55u8, 59, 62, 67] {
            e.note_off(n);
        }
    }
    // Let the reverb / delay tail decay
    thread::sleep(Duration::from_millis(3000));
    {
        let mut e = engine.lock().unwrap();
        e.clear_fx_chain();
        e.set_master_gain(0.30);
    }

    println!("done");
}

fn send_note(engine: &Arc<Mutex<Engine>>, note: u8, velocity: u8, hold: Duration) {
    {
        let mut e = engine.lock().unwrap();
        e.note_on(note, velocity);
    }
    thread::sleep(hold);
    {
        let mut e = engine.lock().unwrap();
        e.note_off(note);
    }
    thread::sleep(Duration::from_millis(80));
}
