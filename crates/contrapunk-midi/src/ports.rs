//! MIDI port enumeration and selection functions.
//!
//! Provides functions to list available MIDI input and output ports,
//! and to prompt the user for port selection via stdin.

use anyhow::{anyhow, Result};
use midir::{MidiInput, MidiOutput};
use std::io::{self, Write};

/// Lists all available MIDI input ports.
///
/// Returns a vector of (index, port_name) tuples for display and selection.
pub fn list_input_ports() -> Result<Vec<(usize, String)>> {
    let midi_in = MidiInput::new("contrapunk-in")?;
    let ports = midi_in.ports();

    if ports.is_empty() {
        return Ok(Vec::new());
    }

    let mut port_list = Vec::new();
    for (i, port) in ports.iter().enumerate() {
        let name = midi_in
            .port_name(port)
            .unwrap_or_else(|_| "Unknown".to_string());
        port_list.push((i, name));
    }

    Ok(port_list)
}

/// Lists all available MIDI output ports.
///
/// Returns a vector of (index, port_name) tuples for display and selection.
pub fn list_output_ports() -> Result<Vec<(usize, String)>> {
    let midi_out = MidiOutput::new("contrapunk-out")?;
    let ports = midi_out.ports();

    if ports.is_empty() {
        return Ok(Vec::new());
    }

    let mut port_list = Vec::new();
    for (i, port) in ports.iter().enumerate() {
        let name = midi_out
            .port_name(port)
            .unwrap_or_else(|_| "Unknown".to_string());
        port_list.push((i, name));
    }

    Ok(port_list)
}

/// Prompts the user to select a single MIDI input port.
///
/// Displays a numbered list of available input ports and waits for user input.
/// Validates that the selection is within the valid range.
///
/// Returns the selected port index.
pub fn select_input_port(ports: &[(usize, String)]) -> Result<usize> {
    if ports.is_empty() {
        return Err(anyhow!("No MIDI input ports available"));
    }

    println!("Available MIDI input ports:");
    for (i, name) in ports {
        println!("  {}: {}", i, name);
    }

    loop {
        print!("Select input port [0-{}]: ", ports.len() - 1);
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        match input.trim().parse::<usize>() {
            Ok(idx) if idx < ports.len() => {
                return Ok(idx);
            }
            _ => {
                println!(
                    "Invalid selection. Please enter a number between 0 and {}.",
                    ports.len() - 1
                );
            }
        }
    }
}

/// Prompts the user to select multiple MIDI output ports.
///
/// Displays a numbered list of available output ports and waits for user input.
/// Accepts comma-separated port numbers (e.g., "0,2,3").
/// Validates that the selection count is within min/max bounds.
///
/// Returns a vector of selected port indices.
pub fn select_output_ports(
    ports: &[(usize, String)],
    min: usize,
    max: usize,
) -> Result<Vec<usize>> {
    if ports.is_empty() {
        return Err(anyhow!("No MIDI output ports available"));
    }

    if ports.len() < min {
        return Err(anyhow!(
            "Need at least {} output ports, but only {} available",
            min,
            ports.len()
        ));
    }

    println!("\nAvailable MIDI output ports:");
    for (i, name) in ports {
        println!("  {}: {}", i, name);
    }

    loop {
        print!(
            "Select output ports (comma-separated, {}-{} ports): ",
            min, max
        );
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        // Parse comma-separated numbers
        let selections: Result<Vec<usize>, _> = input
            .trim()
            .split(',')
            .map(|s| s.trim().parse::<usize>())
            .collect();

        match selections {
            Ok(indices) => {
                // Validate count
                if indices.len() < min || indices.len() > max {
                    println!(
                        "Please select between {} and {} ports. You selected {}.",
                        min,
                        max,
                        indices.len()
                    );
                    continue;
                }

                // Validate all indices are in range
                let all_valid = indices.iter().all(|&idx| idx < ports.len());
                if !all_valid {
                    println!(
                        "Invalid port number. Please use numbers between 0 and {}.",
                        ports.len() - 1
                    );
                    continue;
                }

                // Check for duplicates
                let mut sorted = indices.clone();
                sorted.sort();
                sorted.dedup();
                if sorted.len() != indices.len() {
                    println!("Duplicate port numbers detected. Please select unique ports.");
                    continue;
                }

                return Ok(indices);
            }
            Err(_) => {
                println!("Invalid input. Please enter comma-separated numbers (e.g., 0,1,2).");
            }
        }
    }
}
