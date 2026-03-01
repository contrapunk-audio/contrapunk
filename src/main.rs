//! Contrapunk binary entry point.
//!
//! The primary interfaces are now:
//! - **Desktop**: Tauri app (`cargo tauri dev` or built application)
//! - **Browser**: SvelteKit WASM app (`cd ui && npm run dev`)
//!
//! This binary retains server and client modes for network-based
//! MIDI harmony processing.

use anyhow::{anyhow, Result};
use clap::Parser;
use contrapunk::harmony::{HarmonyMode, Key, OctaveMode};

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

fn main() -> Result<()> {
    let args = Args::parse();

    // Server mode
    if args.server {
        let config = contrapunk::server::config::ServerConfig {
            port: args.port,
            ..Default::default()
        };
        return contrapunk::server::run_server(&config);
    }

    // Client mode
    if let Some(ref addr) = args.client {
        return run_client(addr);
    }

    // Default: inform user about the new GUI options
    println!("Contrapunk - Real-time MIDI Harmony Generation");
    println!("===============================================");
    println!();
    println!("The GUI has moved to Tauri (desktop) and SvelteKit (browser).");
    println!();
    println!("  Desktop app:  cd src-tauri && cargo tauri dev");
    println!("  Browser app:  cd ui && npm run dev");
    println!();
    println!("Server/client modes are still available:");
    println!("  Server:  contrapunk --server [--port 9900]");
    println!("  Client:  contrapunk --client <host:port>");

    Ok(())
}

/// Run as a client connecting to a remote Contrapunk server.
///
/// Connects to the server, prompts for local MIDI port selection and
/// harmony configuration, then streams local MIDI input to the server
/// and routes harmonized output back to local MIDI output ports.
fn run_client(addr: &str) -> Result<()> {
    use std::io;
    use std::net::TcpStream;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;
    use std::time::Duration;

    use contrapunk::midi::input::connect_input;
    use contrapunk::midi::output::OutputRouter;
    use contrapunk::midi::ports::{
        list_input_ports, list_output_ports, select_input_port, select_output_ports,
    };
    use contrapunk::server::protocol::{self, Message};

    // Connect to server
    let mut stream =
        TcpStream::connect(addr).map_err(|e| anyhow!("Failed to connect to {}: {}", addr, e))?;
    stream.set_nodelay(true)?;
    stream.set_read_timeout(Some(Duration::from_secs(30)))?;
    stream.set_write_timeout(Some(Duration::from_secs(5)))?;

    println!("Connected to Contrapunk server at {}", addr);

    // --- MIDI Port Selection ---
    let input_ports = list_input_ports()?;
    if input_ports.is_empty() {
        return Err(anyhow!("No MIDI input ports available"));
    }
    let selected_input = select_input_port(&input_ports)?;
    println!(
        "\nSelected input: {} - {}\n",
        selected_input, input_ports[selected_input].1
    );

    let output_ports = list_output_ports()?;
    if output_ports.is_empty() {
        return Err(anyhow!("No MIDI output ports available"));
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
    println!(
        "\nSelected mode: {} - {}\n",
        mode.number(),
        mode.description()
    );

    let octave_mode = select_octave_mode()?;
    println!("\nSelected octave mode: {}\n", octave_mode.description());

    let voice_count = selected_outputs.len() as u8;

    // Send Configure message
    let key_index = Key::all().iter().position(|k| *k == key).unwrap_or(0) as u8;
    let mode_index = HarmonyMode::all()
        .iter()
        .position(|m| *m == mode)
        .unwrap_or(0) as u8;
    let octave_index = OctaveMode::all()
        .iter()
        .position(|o| *o == octave_mode)
        .unwrap_or(0) as u8;

    protocol::write_message(
        &mut stream,
        &Message::Configure {
            key: key_index,
            mode: mode_index,
            octave_mode: octave_index,
            voice_count,
        },
    )?;

    // Wait for Ack
    match protocol::read_message(&mut stream)? {
        Message::Ack => println!("Server acknowledged configuration."),
        other => eprintln!("Unexpected response from server: {:?}", other),
    }

    // Connect to local MIDI input
    let (tx, rx) = std::sync::mpsc::channel::<Vec<u8>>();
    let _conn_in = connect_input(selected_input, tx)?;

    // Create local output router
    let mut output_router = OutputRouter::new(&selected_outputs)?;

    // Voice counter for round-robin output routing
    let voice_counter = Arc::new(AtomicUsize::new(0));
    let vc_reader = Arc::clone(&voice_counter);
    let vc_count = voice_count as usize;

    // Spawn reader thread for server responses
    let mut read_stream = stream
        .try_clone()
        .map_err(|e| anyhow!("Failed to clone stream: {}", e))?;

    let (response_tx, response_rx) = std::sync::mpsc::channel::<Vec<u8>>();

    std::thread::spawn(move || {
        loop {
            match protocol::read_message(&mut read_stream) {
                Ok(Message::MidiData(bytes)) => {
                    eprintln!(
                        "[client] received {} bytes from server: {:?}",
                        bytes.len(),
                        &bytes
                    );
                    if response_tx.send(bytes).is_err() {
                        break;
                    }
                }
                Ok(Message::Heartbeat) => {
                    let _ = protocol::write_message(&mut read_stream, &Message::Heartbeat);
                }
                Ok(Message::Disconnect) => {
                    eprintln!("[client] Server sent disconnect");
                    break;
                }
                Ok(_) => {} // Ignore unexpected messages
                Err(e) => {
                    // Timeout is not fatal -- keep waiting for server responses
                    let is_timeout = e.chain().any(|cause| {
                        if let Some(io_err) = cause.downcast_ref::<std::io::Error>() {
                            matches!(
                                io_err.kind(),
                                std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
                            )
                        } else {
                            false
                        }
                    });
                    if is_timeout {
                        continue;
                    }
                    break;
                }
            }
        }
    });

    println!("\n========================================");
    println!("Client mode active. Streaming to {}.", addr);
    println!("Press Enter to stop.");
    println!("========================================\n");

    // Spawn Enter-key detection thread
    let (stop_tx, stop_rx) = std::sync::mpsc::channel::<()>();
    std::thread::spawn(move || {
        let stdin = io::stdin();
        let mut line = String::new();
        let _ = stdin.read_line(&mut line);
        let _ = stop_tx.send(());
    });

    // Main loop: send MIDI input to server, route responses to outputs
    loop {
        // Check for stop signal
        if stop_rx.try_recv().is_ok() {
            println!("\nStopping client...");
            break;
        }

        // Send local MIDI input to server
        match rx.recv_timeout(Duration::from_millis(10)) {
            Ok(message) => {
                // Reset voice counter for each new input message
                voice_counter.store(0, Ordering::Relaxed);
                if let Err(e) = protocol::write_message(&mut stream, &Message::MidiData(message)) {
                    eprintln!("[client] Failed to send to server: {}", e);
                    break;
                }
            }
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {}
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                eprintln!("[client] MIDI input disconnected");
                break;
            }
        }

        // Route server responses to local outputs
        while let Ok(bytes) = response_rx.try_recv() {
            let output_index = vc_reader.fetch_add(1, Ordering::Relaxed) % vc_count;
            eprintln!(
                "[client] routing {} bytes to output {}",
                bytes.len(),
                output_index
            );
            if let Err(e) = output_router.send_to_port(output_index, &bytes) {
                eprintln!("[client] Failed to send to output {}: {}", output_index, e);
            }
        }
    }

    // Send disconnect
    let _ = protocol::write_message(&mut stream, &Message::Disconnect);
    println!("Client disconnected.");
    Ok(())
}

/// Prompts user to select a musical key.
fn select_key() -> Result<Key> {
    use std::io::{self, Write};

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

    let index: usize = input
        .parse()
        .map_err(|_| anyhow!("Invalid number: {}", input))?;

    Key::all()
        .get(index)
        .copied()
        .ok_or_else(|| anyhow!("Key index {} out of range (0-11)", index))
}

/// Prompts user to select a harmony mode.
fn select_mode() -> Result<HarmonyMode> {
    use std::io::{self, Write};

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

    let number: u8 = input
        .parse()
        .map_err(|_| anyhow!("Invalid number: {}", input))?;

    HarmonyMode::all()
        .iter()
        .find(|m| m.number() == number)
        .copied()
        .ok_or_else(|| anyhow!("Mode {} not found (valid: 1-7)", number))
}

/// Prompts user to select an octave mode.
fn select_octave_mode() -> Result<OctaveMode> {
    use std::io::{self, Write};

    println!("Select octave mode:");
    println!("  0: None (default pitch)");
    println!("  1: Spread (+1 octave per voice)");
    println!("  2: Bass/Treble split");
    println!("  3: Mirror (plus/minus 1 octave)");

    print!("\nEnter octave mode number [0-3, default 0 (None)]: ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let input = input.trim();

    if input.is_empty() {
        return Ok(OctaveMode::None);
    }

    let number: u8 = input
        .parse()
        .map_err(|_| anyhow!("Invalid number: {}", input))?;

    match number {
        0 => Ok(OctaveMode::None),
        1 => Ok(OctaveMode::Spread),
        2 => Ok(OctaveMode::BassTrebleSplit),
        3 => Ok(OctaveMode::Mirror),
        _ => Err(anyhow!("Octave mode {} not found (valid: 0-3)", number)),
    }
}
