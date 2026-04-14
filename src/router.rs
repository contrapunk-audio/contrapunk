//! MIDI message routing from input to outputs with harmony generation.
//!
//! Provides the main routing loop that connects MIDI input to outputs
//! and processes messages through the HarmonyEngine.

use crate::harmony::HarmonyEngine;
use crate::humanize::{DelayQueue, HumanizeConfig, HumanizedNote, Humanizer};
use crate::midi::input::connect_input;
use crate::midi::output::OutputRouter;
use anyhow::Result;
use std::io::{self, BufRead};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};
use wmidi::{Channel, MidiMessage, Note, Velocity};

/// Sends a humanized note to the appropriate output port.
fn send_humanized_note(note: &HumanizedNote, output: &mut OutputRouter) -> Result<()> {
    let msg = if note.is_note_off {
        MidiMessage::NoteOff(note.channel, note.note, note.velocity)
    } else {
        MidiMessage::NoteOn(note.channel, note.note, note.velocity)
    };
    let mut buf = vec![0u8; msg.bytes_size()];
    msg.copy_to_slice(&mut buf)?;
    output.send_to_port(note.port, &buf)?;
    Ok(())
}

/// Runs the MIDI routing loop with harmony generation.
pub fn run_router(
    input_port: usize,
    output_ports: &[usize],
    engine: &mut HarmonyEngine,
) -> Result<()> {
    // Create channel for message forwarding from input callback to main loop
    let (tx, rx) = mpsc::channel::<Vec<u8>>();

    // Connect to input port - IMPORTANT: Must keep connection alive
    let _conn_in = connect_input(input_port, tx)?;

    // Create output router
    let mut output_router = OutputRouter::new(output_ports)?;

    // Configure engine voice count to match output count
    let num_outputs = output_router.connection_count();
    engine.set_voice_count(num_outputs);

    // Create humanizer and delay queue (CLI uses default config = disabled)
    let mut humanizer = Humanizer::new(HumanizeConfig::default());
    let mut delay_queue = DelayQueue::new();
    let epoch = Instant::now();
    let now_ms = || epoch.elapsed().as_secs_f64() * 1000.0;

    humanizer.clock_mut().start(now_ms());

    println!("\n========================================");
    println!("MIDI harmony routing active.");
    println!(
        "Key: {:?}, Mode: {} - {}",
        engine.key(),
        engine.mode().number(),
        engine.mode().description()
    );
    println!(
        "Voices: {} (melody + {} chained harmonies)",
        num_outputs,
        num_outputs.saturating_sub(1)
    );
    println!("Press Enter to stop.");
    println!("========================================\n");

    // Create a channel to signal when Enter is pressed
    let (stop_tx, stop_rx) = mpsc::channel::<()>();

    // Spawn thread to wait for Enter key
    thread::spawn(move || {
        let stdin = io::stdin();
        let mut handle = stdin.lock();
        let mut line = String::new();
        let _ = handle.read_line(&mut line);
        let _ = stop_tx.send(());
    });

    // Main routing loop
    loop {
        // Check if user pressed Enter (non-blocking)
        if stop_rx.try_recv().is_ok() {
            println!("\nStopping MIDI router...");
            break;
        }

        // Tick humanizer
        let current_ms = now_ms();
        humanizer.tick(current_ms);

        // Push the current beat-phase position into the harmony engine so
        // beat-aware modes (Species 2-4 counterpoint) can react to it.
        engine.set_counterpoint_beat_phase(Some(humanizer.clock().beat_position()));

        // Drain delay queue
        let current_ms = now_ms();
        for hn in delay_queue.drain_ready(current_ms) {
            let _ = send_humanized_note(&hn, &mut output_router);
        }

        // Try to receive MIDI message with timeout
        match rx.recv_timeout(Duration::from_millis(5)) {
            Ok(message) => {
                let current_ms = now_ms();
                if let Err(e) = process_midi_message(
                    &message,
                    engine,
                    &mut output_router,
                    &mut humanizer,
                    &mut delay_queue,
                    current_ms,
                ) {
                    eprintln!("Error processing message: {}", e);
                }
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {
                // No message received, continue loop (allows checking for Enter)
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                eprintln!("Input channel disconnected unexpectedly.");
                break;
            }
        }
    }

    println!("MIDI router stopped.");
    Ok(())
}

/// Processes a single MIDI message through the harmony engine.
fn process_midi_message(
    bytes: &[u8],
    engine: &mut HarmonyEngine,
    output: &mut OutputRouter,
    humanizer: &mut Humanizer,
    delay_queue: &mut DelayQueue,
    now_ms: f64,
) -> Result<()> {
    let msg = match MidiMessage::try_from(bytes) {
        Ok(m) => m,
        Err(_) => {
            output.send_to_first(bytes)?;
            return Ok(());
        }
    };

    match msg {
        MidiMessage::NoteOn(channel, note, velocity) => {
            if velocity == Velocity::MIN {
                handle_note_off(
                    channel,
                    note,
                    velocity,
                    engine,
                    output,
                    humanizer,
                    delay_queue,
                    now_ms,
                )?;
            } else {
                handle_note_on(
                    channel,
                    note,
                    velocity,
                    engine,
                    output,
                    humanizer,
                    delay_queue,
                    now_ms,
                )?;
            }
        }
        MidiMessage::NoteOff(channel, note, velocity) => {
            handle_note_off(
                channel,
                note,
                velocity,
                engine,
                output,
                humanizer,
                delay_queue,
                now_ms,
            )?;
        }
        _ => {
            output.send_to_first(bytes)?;
        }
    }

    Ok(())
}

/// Handles Note-On: harmonize and send to outputs.
fn handle_note_on(
    channel: Channel,
    note: Note,
    velocity: Velocity,
    engine: &mut HarmonyEngine,
    output: &mut OutputRouter,
    humanizer: &mut Humanizer,
    delay_queue: &mut DelayQueue,
    now_ms: f64,
) -> Result<()> {
    let notes = engine.harmonize_note_on(note);
    let num_outputs = output.connection_count();

    let port_map = engine.last_port_map();
    for (i, &n) in notes.iter().enumerate() {
        let port = if i < port_map.len() { port_map[i] } else { i };
        if port >= num_outputs {
            continue;
        }

        if i == 0 {
            // Melody: send immediately
            let msg = MidiMessage::NoteOn(channel, n, velocity);
            let mut buf = vec![0u8; msg.bytes_size()];
            msg.copy_to_slice(&mut buf)?;
            output.send_to_port(port, &buf)?;
        } else {
            // Harmony: humanize
            let hn = humanizer.humanize_note_on(n, channel, velocity, port);
            if hn.delay_ms == 0 {
                send_humanized_note(&hn, output)?;
            } else {
                delay_queue.push(hn, now_ms);
            }
        }
    }

    // Debug output
    if notes.len() > 1 {
        let note_strs: Vec<String> = notes
            .iter()
            .zip(port_map.iter().chain(std::iter::repeat(&0)))
            .map(|(n, p)| format!("{:?}->p{}", n, p))
            .collect();
        println!("[CHAIN] {:?} => [{}]", note, note_strs.join(", "));
    } else {
        println!("[PASS] {:?}", note);
    }

    Ok(())
}

/// Handles Note-Off: release original and harmony notes.
fn handle_note_off(
    channel: Channel,
    note: Note,
    velocity: Velocity,
    engine: &mut HarmonyEngine,
    output: &mut OutputRouter,
    humanizer: &mut Humanizer,
    delay_queue: &mut DelayQueue,
    now_ms: f64,
) -> Result<()> {
    let notes = engine.harmonize_note_off(note);
    let num_outputs = output.connection_count();

    let port_map = engine.last_port_map();
    for (i, &n) in notes.iter().enumerate() {
        let port = if i < port_map.len() { port_map[i] } else { i };
        if port >= num_outputs {
            continue;
        }

        if i == 0 {
            // Melody: send immediately
            let msg = MidiMessage::NoteOff(channel, n, velocity);
            let mut buf = vec![0u8; msg.bytes_size()];
            msg.copy_to_slice(&mut buf)?;
            output.send_to_port(port, &buf)?;
        } else {
            // Harmony: humanize note-off
            let hn = humanizer.humanize_note_off(n, channel, velocity, port);
            if hn.delay_ms == 0 {
                send_humanized_note(&hn, output)?;
            } else {
                delay_queue.push(hn, now_ms);
            }
        }
    }

    Ok(())
}
