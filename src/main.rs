mod harmony;
mod midi;
mod router;

use anyhow::{anyhow, Result};
use std::io::{self, Write};

use crate::harmony::{Key, HarmonyMode, HarmonyEngine};
use crate::midi::ports::{list_input_ports, list_output_ports, select_input_port, select_output_ports};

fn main() -> Result<()> {
    println!("Contrapunk MIDI Harmony Generator");
    println!("==================================\n");

    // --- MIDI Port Selection ---

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

    // --- Harmony Configuration ---

    println!("\n--- Harmony Configuration ---\n");

    let key = select_key()?;
    println!("\nSelected key: {}\n", key);

    let mode = select_mode()?;
    println!("\nSelected mode: {} - {}\n", mode.number(), mode.description());

    // --- Configuration Summary ---

    println!("\n========================================");
    println!("         Configuration Summary");
    println!("========================================");
    println!("Input:   {} ({})", input_ports[selected_input].1, selected_input);
    println!("Outputs: {} ports", selected_outputs.len());
    for (i, &idx) in selected_outputs.iter().enumerate() {
        let role = if i == 0 { "melody" } else { "harmony" };
        println!("  Voice {}: {} ({}) [{}]", i + 1, output_ports[idx].1, idx, role);
    }
    println!("Key:     {}", key);
    println!("Mode:    {} - {}", mode.number(), mode.description());
    println!("========================================\n");

    // --- Create Harmony Engine and Start Routing ---

    // Create harmony engine with user's selections
    let mut engine = HarmonyEngine::new(key, mode);

    println!("Starting MIDI harmony routing...\n");

    if let Err(e) = router::run_router(selected_input, &selected_outputs, &mut engine) {
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

/// Prompts user to select a harmony mode.
///
/// Displays all 7 modes with descriptions and returns the selected mode.
fn select_mode() -> Result<HarmonyMode> {
    println!("Select harmony mode:");
    for mode in HarmonyMode::all() {
        println!("  {}: {}", mode.number(), mode.description());
    }

    print!("\nEnter mode number [1-7, default 1 (Pass-through)]: ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let input = input.trim();

    if input.is_empty() {
        return Ok(HarmonyMode::PassThrough);
    }

    let number: u8 = input.parse()
        .map_err(|_| anyhow!("Invalid number: {}", input))?;

    HarmonyMode::all()
        .iter()
        .find(|m| m.number() == number)
        .copied()
        .ok_or_else(|| anyhow!("Mode {} not found (valid: 1-7)", number))
}
