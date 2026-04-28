//! MIDI output connection and message routing.
//!
//! Provides the `OutputRouter` struct for managing multiple MIDI output
//! connections and routing messages to them.

use anyhow::{anyhow, Result};
use midir::{MidiOutput, MidiOutputConnection};

/// Manages multiple MIDI output connections for message routing.
///
/// `OutputRouter` holds connections to multiple MIDI output ports and
/// provides methods to send messages to specific ports or broadcast
/// to all connected ports.
pub struct OutputRouter {
    /// Active output connections
    connections: Vec<MidiOutputConnection>,
    /// Port indices for reference
    port_indices: Vec<usize>,
}

impl OutputRouter {
    /// Creates a new `OutputRouter` connected to the specified output ports.
    ///
    /// # Arguments
    ///
    /// * `port_indices` - Slice of port indices to connect to
    ///
    /// # Returns
    ///
    /// Returns an `OutputRouter` with all specified ports connected.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Any MIDI output cannot be initialized
    /// - Any port index is invalid
    /// - Any connection fails
    pub fn new(port_indices: &[usize]) -> Result<Self> {
        // Empty is allowed — voices may route entirely to the built-in synth
        // via the per-voice routing table. Returning an empty router keeps
        // the MIDI-output path no-op without blocking routing start.
        let mut connections = Vec::new();

        for (i, &idx) in port_indices.iter().enumerate() {
            // Create a new MidiOutput for each connection with unique client name
            let midi_out = MidiOutput::new(&format!("contrapunk-out-{}", i))?;
            let ports = midi_out.ports();

            let port = ports
                .get(idx)
                .ok_or_else(|| anyhow!("Invalid output port index: {}", idx))?;

            let port_name = midi_out
                .port_name(port)
                .unwrap_or_else(|_| "Unknown".to_string());

            println!("Connecting to output: {} (port {})", port_name, idx);

            let conn = midi_out
                .connect(port, &format!("contrapunk-{}", i))
                .map_err(|e| anyhow!("Failed to connect to output port {}: {}", idx, e))?;

            connections.push(conn);
        }

        println!(
            "Output router ready with {} connection(s).",
            connections.len()
        );

        Ok(Self {
            connections,
            port_indices: port_indices.to_vec(),
        })
    }

    /// Sends a MIDI message to the first connected output port.
    ///
    /// This is used for pass-through routing where all input messages
    /// are forwarded to a single output.
    ///
    /// # Arguments
    ///
    /// * `message` - MIDI message bytes to send
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the message was sent successfully, or an error
    /// if no connections exist or the send failed.
    pub fn send_to_first(&mut self, message: &[u8]) -> Result<()> {
        if let Some(conn) = self.connections.first_mut() {
            conn.send(message)
                .map_err(|e| anyhow!("Failed to send MIDI message: {:?}", e))?;

            println!(
                "[OUT] port {} | {:?} (len={})",
                self.port_indices[0],
                message,
                message.len()
            );

            Ok(())
        } else {
            Err(anyhow!("No output connections available"))
        }
    }

    /// Sends a MIDI message to all connected output ports.
    ///
    /// This is used for broadcasting messages to multiple outputs,
    /// such as when routing harmony voices.
    ///
    /// # Arguments
    ///
    /// * `message` - MIDI message bytes to send
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the message was sent to all ports successfully.
    /// Returns an error containing details of any failed sends.
    pub fn send_to_all(&mut self, message: &[u8]) -> Result<()> {
        let mut errors = Vec::new();

        for (i, conn) in self.connections.iter_mut().enumerate() {
            if let Err(e) = conn.send(message) {
                errors.push(format!("port {}: {:?}", self.port_indices[i], e));
            } else {
                println!(
                    "[OUT] port {} | {:?} (len={})",
                    self.port_indices[i],
                    message,
                    message.len()
                );
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(anyhow!(
                "Failed to send to some ports: {}",
                errors.join(", ")
            ))
        }
    }

    /// Returns the number of connected output ports.
    pub fn connection_count(&self) -> usize {
        self.connections.len()
    }

    /// Sends a MIDI message to a specific output port by index.
    ///
    /// This is used for harmony routing where different notes go to
    /// different outputs (e.g., original to port 0, harmony to port 1).
    ///
    /// # Arguments
    ///
    /// * `port_index` - Index into the connections array (not the original port number)
    /// * `message` - MIDI message bytes to send
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the message was sent successfully, or an error
    /// if the port index is out of range or the send failed.
    pub fn send_to_port(&mut self, port_index: usize, message: &[u8]) -> Result<()> {
        if let Some(conn) = self.connections.get_mut(port_index) {
            conn.send(message)
                .map_err(|e| anyhow!("Failed to send to port {}: {:?}", port_index, e))?;
            Ok(())
        } else {
            Err(anyhow!(
                "Port index {} out of range (have {} ports)",
                port_index,
                self.connections.len()
            ))
        }
    }
}
