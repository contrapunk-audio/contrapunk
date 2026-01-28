//! MIDI message routing from input to outputs.
//!
//! Provides the main routing loop that connects MIDI input to outputs
//! and forwards all received messages.

use crate::midi::input::connect_input;
use crate::midi::output::OutputRouter;
use anyhow::Result;
use std::io::{self, BufRead};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

/// Runs the MIDI routing loop.
///
/// Connects to the specified input port and output ports, then enters
/// a loop that forwards all received MIDI messages from the input to
/// the first output port.
///
/// The routing continues until the user presses Enter.
///
/// # Arguments
///
/// * `input_port` - Index of the MIDI input port to connect to
/// * `output_ports` - Slice of output port indices to connect to
///
/// # Returns
///
/// Returns `Ok(())` when the user exits, or an error if connection fails.
pub fn run_router(input_port: usize, output_ports: &[usize]) -> Result<()> {
    // Create channel for message forwarding from input callback to main loop
    let (tx, rx) = mpsc::channel::<Vec<u8>>();

    // Connect to input port - IMPORTANT: Must keep connection alive
    let _conn_in = connect_input(input_port, tx)?;

    // Create output router
    let mut output_router = OutputRouter::new(output_ports)?;

    println!("\n========================================");
    println!("MIDI routing active. Press Enter to stop.");
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
        match rx.recv_timeout(Duration::from_millis(100)) {
            Ok(message) => {
                // Forward to first output
                if let Err(e) = output_router.send_to_first(&message) {
                    eprintln!("Error forwarding message: {}", e);
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
