mod midi;

use anyhow::Result;
use midi::ports::{list_input_ports, list_output_ports, select_input_port, select_output_ports};

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

    println!("\nReady for MIDI routing. (Connection will be implemented in next plan)");

    Ok(())
}
