//! Per-client session handling for the Contrapunk TCP server.
//!
//! Each connected client gets its own HarmonyEngine instance and
//! processes MIDI messages independently.

use std::net::TcpStream;
use std::time::Duration;

use anyhow::Result;
use wmidi::MidiMessage;

use crate::harmony::{HarmonyEngine, HarmonyMode, Key, OctaveMode};
use crate::server::protocol::{self, Message};

/// Handle a single client connection.
///
/// Configures the stream, creates a per-client HarmonyEngine, and loops
/// reading protocol messages. MIDI NoteOn/NoteOff are harmonized and
/// sent back; Configure messages update engine settings; Heartbeat is
/// echoed; Disconnect breaks the loop.
///
/// Sends all-notes-off (CC 123) on disconnect.
pub fn handle_client(mut stream: TcpStream) -> Result<()> {
    let peer = stream
        .peer_addr()
        .map(|a| a.to_string())
        .unwrap_or_else(|_| "unknown".into());
    eprintln!("[server] client connected: {}", peer);

    // Configure stream
    stream.set_nodelay(true)?;
    stream.set_read_timeout(Some(Duration::from_secs(30)))?;
    stream.set_write_timeout(Some(Duration::from_secs(5)))?;

    // Per-client harmony engine
    let mut engine = HarmonyEngine::new(Key::C, HarmonyMode::PassThrough);
    engine.set_voice_count(4);

    loop {
        let msg = match protocol::read_message(&mut stream) {
            Ok(m) => m,
            Err(e) => {
                eprintln!("[server] client {} read error: {}", peer, e);
                break;
            }
        };

        match msg {
            Message::MidiData(bytes) => {
                if let Err(e) = process_midi(&bytes, &mut engine, &mut stream) {
                    eprintln!("[server] client {} midi error: {}", peer, e);
                }
            }
            Message::Configure {
                key,
                mode,
                octave_mode,
                voice_count,
            } => {
                apply_configure(&mut engine, key, mode, octave_mode, voice_count);
                // Best-effort ack
                let _ = protocol::write_message(&mut stream, &Message::Ack);
            }
            Message::Heartbeat => {
                let _ = protocol::write_message(&mut stream, &Message::Heartbeat);
            }
            Message::Disconnect => {
                eprintln!("[server] client {} sent disconnect", peer);
                break;
            }
            Message::Ack => {
                // Unexpected from client, ignore
            }
        }
    }

    // Send all-notes-off (CC 123, value 0, channel 0) as best-effort
    send_all_notes_off(&mut stream);

    eprintln!("[server] client disconnected: {}", peer);
    Ok(())
}

/// Process a MIDI data message through the harmony engine and write
/// resulting messages back to the client stream.
fn process_midi(
    bytes: &[u8],
    engine: &mut HarmonyEngine,
    stream: &mut TcpStream,
) -> Result<()> {
    let midi_msg = match MidiMessage::try_from(bytes) {
        Ok(m) => m,
        Err(_) => {
            // Non-parseable MIDI: echo back unchanged
            protocol::write_message(stream, &Message::MidiData(bytes.to_vec()))?;
            return Ok(());
        }
    };

    match midi_msg {
        MidiMessage::NoteOn(channel, note, velocity) => {
            if velocity == wmidi::Velocity::MIN {
                // Velocity 0 = Note-Off
                let notes = engine.harmonize_note_off(note);
                for &n in &notes {
                    let msg = MidiMessage::NoteOn(channel, n, velocity);
                    let mut buf = vec![0u8; msg.bytes_size()];
                    msg.copy_to_slice(&mut buf)?;
                    protocol::write_message(stream, &Message::MidiData(buf))?;
                }
            } else {
                let notes = engine.harmonize_note_on(note);
                for &n in &notes {
                    let msg = MidiMessage::NoteOn(channel, n, velocity);
                    let mut buf = vec![0u8; msg.bytes_size()];
                    msg.copy_to_slice(&mut buf)?;
                    protocol::write_message(stream, &Message::MidiData(buf))?;
                }
            }
        }
        MidiMessage::NoteOff(channel, note, velocity) => {
            let notes = engine.harmonize_note_off(note);
            for &n in &notes {
                let msg = MidiMessage::NoteOff(channel, n, velocity);
                let mut buf = vec![0u8; msg.bytes_size()];
                msg.copy_to_slice(&mut buf)?;
                protocol::write_message(stream, &Message::MidiData(buf))?;
            }
        }
        _ => {
            // Non-note messages: echo back unchanged
            protocol::write_message(stream, &Message::MidiData(bytes.to_vec()))?;
        }
    }

    Ok(())
}

/// Apply a Configure message to the engine.
fn apply_configure(
    engine: &mut HarmonyEngine,
    key: u8,
    mode: u8,
    octave_mode: u8,
    voice_count: u8,
) {
    // Convert key index (0-11) to Key
    let keys = Key::all();
    if let Some(&new_key) = keys.get(key as usize) {
        if new_key != engine.key() {
            engine.set_key(new_key);
        }
    }

    // Convert mode index (0-6) to HarmonyMode
    let modes = HarmonyMode::all();
    if let Some(&new_mode) = modes.get(mode as usize) {
        if new_mode != engine.mode() {
            engine.set_mode(new_mode);
        }
    }

    // Convert octave_mode index (0-3) to OctaveMode
    let octave_modes = OctaveMode::all();
    if let Some(&new_oct) = octave_modes.get(octave_mode as usize) {
        if new_oct != engine.octave_mode() {
            engine.set_octave_mode(new_oct);
        }
    }

    // Voice count (1-based, at least 1)
    let vc = (voice_count as usize).max(1);
    if vc != engine.voice_count() {
        engine.set_voice_count(vc);
    }
}

/// Send all-notes-off (CC 123, channel 0) to the client. Best-effort.
fn send_all_notes_off(stream: &mut TcpStream) {
    // MIDI CC: status 0xB0 (channel 0), controller 123, value 0
    let cc_bytes = vec![0xB0, 123, 0];
    let _ = protocol::write_message(stream, &Message::MidiData(cc_bytes));
}
