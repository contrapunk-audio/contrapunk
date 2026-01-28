mod harmony;
mod midi;
mod router;

use anyhow::{anyhow, Result};
use std::io::{self, Write};

use crate::harmony::{Key, HarmonyMode, HarmonyEngine};
use crate::midi::ports::{list_input_ports, list_output_ports, select_input_port, select_output_ports};

fn main() -> Result<()> {
    println!("Contrapunk MIDI Router");
    println!("======================\n");

    // List and select MIDI input port
    let input_ports = list_input_ports()?;
    if input_ports.is_empty() {
        println!("No MIDI input ports available.");
        println!("Connect a MIDI device and restart the application.");
        return Ok(());
    }

    let selected_input = select_input_port(&input_ports)?;
    println!(
        "\nSelected input: {} - {}\n",
        selected_input, input_ports[selected_input].1
    );

    // List and select MIDI output ports (2-8 ports for harmony voices)
    let output_ports = list_output_ports()?;
    if output_ports.is_empty() {
        println!("No MIDI output ports available.");
        println!("Connect MIDI output devices and restart the application.");
        return Ok(());
    }

    let selected_outputs = select_output_ports(&output_ports, 2, 8)?;
    println!("\nSelected outputs:");
    for &idx in &selected_outputs {
        println!("  {} - {}", idx, output_ports[idx].1);
    }

    // Confirmation
    println!("\n--- Configuration Complete ---");
    println!("Input:  {} ({})", input_ports[selected_input].1, selected_input);
    println!("Outputs: {} ports selected", selected_outputs.len());
    for (i, &idx) in selected_outputs.iter().enumerate() {
        println!("  Voice {}: {} ({})", i + 1, output_ports[idx].1, idx);
    }

    // Start MIDI routing
    println!("\nStarting MIDI pass-through routing...\n");

    if let Err(e) = router::run_router(selected_input, &selected_outputs) {
        eprintln!("Error during MIDI routing: {}", e);
        return Err(e);
    }

    println!("\nContrapunk exited cleanly.");
    Ok(())
}

/// Prompts user to select a musical key.
///
/// Displays all 12 keys and returns the selected Key.
fn select_key() -> Result<Key> {
    println!("Select musical key:");
    for (i, key) in Key::all().iter().enumerate() {
        println!("  {}: {}", i, key);
    }

    print!("\nEnter key number [0-11, default 0 (C)]: ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let input = input.trim();

    if input.is_empty() {
        return Ok(Key::C);
    }

    let index: usize = input.parse()
        .map_err(|_| anyhow!("Invalid number: {}", input))?;

    Key::all()
        .get(index)
        .copied()
        .ok_or_else(|| anyhow!("Key index {} out of range (0-11)", index))
}
