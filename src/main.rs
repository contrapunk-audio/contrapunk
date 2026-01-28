mod harmony;
mod midi;
mod router;
mod server;

#[cfg(feature = "gui")]
mod app;

#[cfg(feature = "gui")]
mod chord;

#[cfg(feature = "gui")]
mod piano;

#[cfg(feature = "gui")]
use eframe::egui;

#[cfg(not(feature = "gui"))]
use anyhow::anyhow;
use anyhow::Result;
#[cfg(not(feature = "gui"))]
use std::io::{self, Write};

use clap::Parser;

#[cfg(not(feature = "gui"))]
use crate::harmony::{Key, HarmonyMode, OctaveMode, HarmonyEngine};
#[cfg(not(feature = "gui"))]
use crate::midi::ports::{list_input_ports, list_output_ports, select_input_port, select_output_ports};

/// Contrapunk - Real-time MIDI harmony generation
#[derive(Parser)]
#[command(name = "contrapunk", about = "Real-time MIDI harmony generation")]
struct Args {
    /// Run as a harmony server
    #[arg(long)]
    server: bool,

    /// Connect to a server as client (host:port)
    #[arg(long)]
    client: Option<String>,

    /// Server port to listen on
    #[arg(long, default_value_t = 9900)]
    port: u16,
}

/// Run the GUI application.
#[cfg(feature = "gui")]
fn run_gui() -> Result<()> {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([500.0, 700.0])
            .with_min_inner_size([400.0, 500.0])
            .with_title("Contrapunk - MIDI Harmony Generator"),
        ..Default::default()
    };

    eframe::run_native(
        "Contrapunk",
        native_options,
        Box::new(|cc| Ok(Box::new(app::ContrapunkApp::new(cc)))),
    ).map_err(|e| anyhow::anyhow!("GUI error: {}", e))
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Server mode (works in both GUI and CLI builds)
    if args.server {
        let config = server::config::ServerConfig {
            port: args.port,
            ..Default::default()
        };
        return server::run_server(&config);
    }

    // Client mode
    if let Some(ref addr) = args.client {
        #[cfg(not(feature = "gui"))]
        {
            return run_client(addr);
        }
        #[cfg(feature = "gui")]
        {
            let _ = addr;
            eprintln!("Client mode requires CLI build (compile without --features gui)");
            std::process::exit(1);
        }
    }

    #[cfg(feature = "gui")]
    {
        return run_gui();
    }

    #[cfg(not(feature = "gui"))]
    {
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

        let octave_mode = select_octave_mode()?;
        println!("\nSelected octave mode: {}\n", octave_mode.description());

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
        println!("Octave:  {}", octave_mode.description());
        println!("========================================\n");

        // --- Create Harmony Engine and Start Routing ---

        // Create harmony engine with user's selections
        let mut engine = HarmonyEngine::new(key, mode);
        engine.set_octave_mode(octave_mode);

        println!("Starting MIDI harmony routing...\n");

        if let Err(e) = router::run_router(selected_input, &selected_outputs, &mut engine) {
            eprintln!("Error during MIDI routing: {}", e);
            return Err(e);
        }

        println!("\nContrapunk exited cleanly.");
        Ok(())
    }
}

/// Prompts user to select a musical key.
///
/// Displays all 12 keys and returns the selected Key.
#[cfg(not(feature = "gui"))]
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
#[cfg(not(feature = "gui"))]
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

/// Prompts user to select an octave mode.
///
/// Displays all octave modes with descriptions and returns the selected mode.
#[cfg(not(feature = "gui"))]
fn select_octave_mode() -> Result<OctaveMode> {
    println!("Select octave mode:");
    println!("  0: None (default pitch)");
    println!("  1: Spread (+1 octave per voice)");
    println!("  2: Bass/Treble split");
    println!("  3: Mirror (±1 octave)");

    print!("\nEnter octave mode number [0-3, default 0 (None)]: ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let input = input.trim();

    if input.is_empty() {
        return Ok(OctaveMode::None);
    }

    let number: u8 = input.parse()
        .map_err(|_| anyhow!("Invalid number: {}", input))?;

    match number {
        0 => Ok(OctaveMode::None),
        1 => Ok(OctaveMode::Spread),
        2 => Ok(OctaveMode::BassTrebleSplit),
        3 => Ok(OctaveMode::Mirror),
        _ => Err(anyhow!("Octave mode {} not found (valid: 0-3)", number)),
    }
}

/// Run as a client connecting to a remote Contrapunk server.
#[cfg(not(feature = "gui"))]
fn run_client(_addr: &str) -> Result<()> {
    // TODO: Full implementation in Task 2
    Err(anyhow!("Client mode not yet implemented"))
}
