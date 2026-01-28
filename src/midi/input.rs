//! MIDI input connection and callback setup.
//!
//! Provides functions to connect to a MIDI input port and forward
//! received messages through a channel for processing.

use anyhow::{anyhow, Result};
use midir::{Ignore, MidiInput, MidiInputConnection};
use std::sync::mpsc;

/// Connects to a MIDI input port and sets up message forwarding.
///
/// Creates a connection to the specified input port index and configures
/// a callback that forwards all received MIDI messages through the provided
/// channel sender.
///
/// # Arguments
///
/// * `port_index` - Index of the input port to connect to
/// * `tx` - Channel sender for forwarding MIDI messages
///
/// # Returns
///
/// Returns the `MidiInputConnection` which must be kept alive for the
/// connection to remain active. Dropping this value closes the connection.
///
/// # Errors
///
/// Returns an error if:
/// - MIDI input cannot be initialized
/// - The port index is invalid
/// - Connection to the port fails
pub fn connect_input(
    port_index: usize,
    tx: mpsc::Sender<Vec<u8>>,
) -> Result<MidiInputConnection<()>> {
    let mut midi_in = MidiInput::new("contrapunk-in")?;

    // Receive all message types (don't filter SysEx, timing, active sensing)
    midi_in.ignore(Ignore::None);

    let ports = midi_in.ports();
    let port = ports
        .get(port_index)
        .ok_or_else(|| anyhow!("Invalid input port index: {}", port_index))?;

    let port_name = midi_in.port_name(port).unwrap_or_else(|_| "Unknown".to_string());

    println!("Connecting to input: {} (port {})", port_name, port_index);

    // Connect with callback that forwards messages via channel
    let conn = midi_in.connect(
        port,
        "contrapunk-read",
        move |timestamp, message, _| {
            // Clone message bytes (message is a borrowed slice)
            let msg_vec = message.to_vec();

            // Debug output
            println!(
                "[IN] t={:>10} | {:?} (len={})",
                timestamp,
                message,
                message.len()
            );

            // Forward through channel
            if let Err(e) = tx.send(msg_vec) {
                eprintln!("Error sending MIDI message through channel: {}", e);
            }
        },
        (),
    ).map_err(|e| anyhow!("Failed to connect to input port: {}", e))?;

    println!("Input connected successfully.");

    Ok(conn)
}
