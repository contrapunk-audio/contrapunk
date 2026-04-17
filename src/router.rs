//! MIDI message routing from input to outputs with harmony generation.
//!
//! Provides the main routing loop that connects MIDI input to outputs
//! and processes messages through the HarmonyEngine.

use crate::audio_out::{MidiEvent, MidiProducer};
use crate::harmony::HarmonyEngine;
use crate::humanize::{DelayQueue, HumanizeConfig, HumanizedNote, Humanizer};
use crate::midi::input::connect_input;
use crate::midi::output::OutputRouter;
use anyhow::Result;
use std::io::{self, BufRead};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};
use wmidi::{Channel, MidiMessage, Note, Velocity};

/// Sends a humanized note to the appropriate output port.
/// If `audio_out` is Some, also pushes the event to the audio synth queue.
fn send_humanized_note(
    note: &HumanizedNote,
    output: &mut OutputRouter,
    audio_out: Option<&mut MidiProducer>,
    voice_index: u8,
) -> Result<()> {
    let msg = if note.is_note_off {
        MidiMessage::NoteOff(note.channel, note.note, note.velocity)
    } else {
        MidiMessage::NoteOn(note.channel, note.note, note.velocity)
    };
    let mut buf = vec![0u8; msg.bytes_size()];
    msg.copy_to_slice(&mut buf)?;
    output.send_to_port(note.port, &buf)?;

    // Parallel fanout to audio synth queue (fire-and-forget; drop on full).
    if let Some(producer) = audio_out {
        let event = if note.is_note_off {
            MidiEvent::NoteOff {
                voice: voice_index,
                note: u8::from(note.note),
            }
        } else {
            MidiEvent::NoteOn {
                voice: voice_index,
                note: u8::from(note.note),
                velocity: u8::from(note.velocity),
            }
        };
        let _ = producer.push(event);
    }

    Ok(())
}

/// Runs the MIDI routing loop with harmony generation.
///
/// `audio_out` is the producer half of the lock-free SPSC queue that feeds
/// the audio synth. Pass `None` to disable audio output (MIDI-only mode).
/// When provided, ownership is moved into the router loop for its lifetime.
pub fn run_router(
    input_port: usize,
    output_ports: &[usize],
    engine: &mut HarmonyEngine,
    mut audio_out: Option<MidiProducer>,
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

    // Create humanizer and delay queue (CLI uses default config = disabled)
    let mut humanizer = Humanizer::new(HumanizeConfig::default());
    let mut delay_queue = DelayQueue::new();
    let epoch = Instant::now();
    let now_ms = || epoch.elapsed().as_secs_f64() * 1000.0;

    humanizer.clock_mut().start(now_ms());

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

        // Tick humanizer
        let current_ms = now_ms();
        humanizer.tick(current_ms);

        // Push the current beat-phase position into the harmony engine so
        // beat-aware modes (Species 2-4 counterpoint) can react to it.
        engine.set_counterpoint_beat_phase(Some(humanizer.clock().beat_position()));

        // Drain delay queue — includes fanout to audio synth for delayed notes.
        let current_ms = now_ms();
        for hn in delay_queue.drain_ready(current_ms) {
            let _ =
                send_humanized_note(&hn, &mut output_router, audio_out.as_mut(), hn.voice_index);
        }

        // Try to receive MIDI message with timeout
        match rx.recv_timeout(Duration::from_millis(5)) {
            Ok(message) => {
                let current_ms = now_ms();
                if let Err(e) = process_midi_message(
                    &message,
                    engine,
                    &mut output_router,
                    &mut humanizer,
                    &mut delay_queue,
                    current_ms,
                    audio_out.as_mut(),
                ) {
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
    humanizer: &mut Humanizer,
    delay_queue: &mut DelayQueue,
    now_ms: f64,
    audio_out: Option<&mut MidiProducer>,
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
                handle_note_off(
                    channel,
                    note,
                    velocity,
                    engine,
                    output,
                    humanizer,
                    delay_queue,
                    now_ms,
                    audio_out,
                )?;
            } else {
                handle_note_on(
                    channel,
                    note,
                    velocity,
                    engine,
                    output,
                    humanizer,
                    delay_queue,
                    now_ms,
                    audio_out,
                )?;
            }
        }
        MidiMessage::NoteOff(channel, note, velocity) => {
            handle_note_off(
                channel,
                note,
                velocity,
                engine,
                output,
                humanizer,
                delay_queue,
                now_ms,
                audio_out,
            )?;
        }
        _ => {
            output.send_to_first(bytes)?;
        }
    }

    Ok(())
}

/// Handles Note-On: harmonize and send to outputs.
#[allow(clippy::too_many_arguments)]
fn handle_note_on(
    channel: Channel,
    note: Note,
    velocity: Velocity,
    engine: &mut HarmonyEngine,
    output: &mut OutputRouter,
    humanizer: &mut Humanizer,
    delay_queue: &mut DelayQueue,
    now_ms: f64,
    mut audio_out: Option<&mut MidiProducer>,
) -> Result<()> {
    let notes = engine.harmonize_note_on(note);
    let num_outputs = output.connection_count();

    let port_map = engine.last_port_map();
    for (i, &n) in notes.iter().enumerate() {
        let port = if i < port_map.len() { port_map[i] } else { i };
        if port >= num_outputs {
            continue;
        }

        if i == 0 {
            // Melody: send immediately
            let msg = MidiMessage::NoteOn(channel, n, velocity);
            let mut buf = vec![0u8; msg.bytes_size()];
            msg.copy_to_slice(&mut buf)?;
            output.send_to_port(port, &buf)?;

            if let Some(ref mut producer) = audio_out {
                let _ = producer.push(MidiEvent::NoteOn {
                    voice: i as u8,
                    note: u8::from(n),
                    velocity: u8::from(velocity),
                });
            }
        } else {
            // Harmony: humanize with correct voice index
            let hn = humanizer.humanize_note_on(n, channel, velocity, port, i as u8);
            if hn.delay_ms == 0 {
                send_humanized_note(&hn, output, audio_out.as_deref_mut(), hn.voice_index)?;
            } else {
                delay_queue.push(hn, now_ms);
            }
        }
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
#[allow(clippy::too_many_arguments)]
fn handle_note_off(
    channel: Channel,
    note: Note,
    velocity: Velocity,
    engine: &mut HarmonyEngine,
    output: &mut OutputRouter,
    humanizer: &mut Humanizer,
    delay_queue: &mut DelayQueue,
    now_ms: f64,
    mut audio_out: Option<&mut MidiProducer>,
) -> Result<()> {
    let notes = engine.harmonize_note_off(note);
    let num_outputs = output.connection_count();

    let port_map = engine.last_port_map();
    for (i, &n) in notes.iter().enumerate() {
        let port = if i < port_map.len() { port_map[i] } else { i };
        if port >= num_outputs {
            continue;
        }

        if i == 0 {
            // Melody: send immediately
            let msg = MidiMessage::NoteOff(channel, n, velocity);
            let mut buf = vec![0u8; msg.bytes_size()];
            msg.copy_to_slice(&mut buf)?;
            output.send_to_port(port, &buf)?;

            // Fanout to audio synth queue (fire-and-forget).
            if let Some(ref mut producer) = audio_out {
                let _ = producer.push(MidiEvent::NoteOff {
                    voice: i as u8,
                    note: u8::from(n),
                });
            }
        } else {
            // Harmony: humanize note-off with correct voice index
            let hn = humanizer.humanize_note_off(n, channel, velocity, port, i as u8);
            if hn.delay_ms == 0 {
                send_humanized_note(&hn, output, audio_out.as_deref_mut(), hn.voice_index)?;
            } else {
                delay_queue.push(hn, now_ms);
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::audio_out::{midi_queue, MidiEvent};
    use wmidi::{Note, Velocity};

    // -------------------------------------------------------------------------
    // Helper
    // -------------------------------------------------------------------------

    /// Drains all events from the consumer into a Vec.
    fn drain(consumer: &mut crate::audio_out::MidiConsumer) -> Vec<MidiEvent> {
        let mut out = Vec::new();
        while let Some(e) = consumer.pop() {
            out.push(e);
        }
        out
    }

    /// Core fanout test: verifies that when the router's fanout code fires,
    /// a NoteOn event arrives in the consumer with the correct voice/note/velocity.
    ///
    /// We can't easily call handle_note_on without a real MIDI output port
    /// (OutputRouter::new requires at least one valid port on the host system),
    /// so this test exercises the fanout push path directly — identical to the
    /// code that runs inside handle_note_on and handle_note_off after the
    /// MIDI-port send succeeds.
    #[test]
    fn test_fanout_note_on_arrives_in_consumer() {
        let (mut producer, mut consumer) = midi_queue(128);

        // This is the exact push expression used by handle_note_on for voice 0:
        let n = Note::C4;
        let vel = Velocity::MAX;
        let _ = producer.push(MidiEvent::NoteOn {
            voice: 0,
            note: u8::from(n),
            velocity: u8::from(vel),
        });

        let events = drain(&mut consumer);
        assert_eq!(events.len(), 1, "expected exactly one NoteOn event");
        assert_eq!(
            events[0],
            MidiEvent::NoteOn {
                voice: 0,
                note: 60, // C4 = MIDI 60
                velocity: 127
            }
        );
    }

    /// Verifies that when a MidiProducer is provided AND a note is actually
    /// routed (simulated by calling the push path directly as the router
    /// would), NoteOn and NoteOff events both arrive with correct fields.
    #[test]
    fn test_fanout_note_on_and_note_off_queue_mechanics() {
        let (mut producer, mut consumer) = midi_queue(128);

        // Simulate what handle_note_on does for voice 0:
        let _ = producer.push(MidiEvent::NoteOn {
            voice: 0,
            note: u8::from(Note::C4),
            velocity: u8::from(Velocity::MAX),
        });

        // Simulate what handle_note_off does for voice 0:
        let _ = producer.push(MidiEvent::NoteOff {
            voice: 0,
            note: u8::from(Note::C4),
        });

        let events = drain(&mut consumer);
        assert_eq!(events.len(), 2);
        assert_eq!(
            events[0],
            MidiEvent::NoteOn {
                voice: 0,
                note: 60,
                velocity: 127
            }
        );
        assert_eq!(events[1], MidiEvent::NoteOff { voice: 0, note: 60 });
    }

    /// Verifies that when the queue is full, push is dropped without panicking.
    #[test]
    fn test_fanout_drops_on_full_queue() {
        let (mut producer, _consumer) = midi_queue(2);

        // Fill the queue
        producer
            .push(MidiEvent::NoteOn {
                voice: 0,
                note: 60,
                velocity: 100,
            })
            .unwrap();
        producer
            .push(MidiEvent::NoteOn {
                voice: 1,
                note: 62,
                velocity: 100,
            })
            .unwrap();

        // This should be silently dropped (mimicking `let _ = producer.push(...)`)
        let result = producer.push(MidiEvent::NoteOn {
            voice: 2,
            note: 64,
            velocity: 100,
        });
        assert!(result.is_err(), "expected QueueFull error on overflow");
    }
}
