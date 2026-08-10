//! MIDI message routing from input to outputs with harmony generation.
//!
//! Provides the main routing loop that connects MIDI input to outputs
//! and processes messages through the HarmonyEngine.

use crate::harmony::HarmonyEngine;
use crate::midi::input::connect_input;
use crate::midi::output::OutputRouter;
use anyhow::Result;
use std::io::{self, BufRead};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use wmidi::{Channel, MidiMessage, Note, Velocity};

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

        // Try to receive MIDI message with timeout
        match rx.recv_timeout(Duration::from_millis(5)) {
            Ok(message) => {
                if let Err(e) = process_midi_message(&message, engine, &mut output_router) {
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
                handle_note_off(channel, note, velocity, engine, output)?;
            } else {
                handle_note_on(channel, note, velocity, engine, output)?;
            }
        }
        MidiMessage::NoteOff(channel, note, velocity) => {
            handle_note_off(channel, note, velocity, engine, output)?;
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
) -> Result<()> {
    let notes = engine.harmonize_note_on_owned(note, channel.index());
    let num_outputs = output.connection_count();

    let port_map = engine.last_port_map();
    for (i, &n) in notes.iter().enumerate() {
        let port = if i < port_map.len() { port_map[i] } else { i };
        if port >= num_outputs {
            continue;
        }

        let msg = MidiMessage::NoteOn(channel, n, velocity);
        let mut buf = vec![0u8; msg.bytes_size()];
        msg.copy_to_slice(&mut buf)?;
        output.send_to_port(port, &buf)?;
    }

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
) -> Result<()> {
    let notes = engine.harmonize_note_off_owned(note, channel.index());
    let num_outputs = output.connection_count();

    let port_map = engine.last_port_map();
    for (i, &n) in notes.iter().enumerate() {
        let port = if i < port_map.len() { port_map[i] } else { i };
        if port >= num_outputs {
            continue;
        }

        let msg = MidiMessage::NoteOff(channel, n, velocity);
        let mut buf = vec![0u8; msg.bytes_size()];
        msg.copy_to_slice(&mut buf)?;
        output.send_to_port(port, &buf)?;
    }

    Ok(())
}
