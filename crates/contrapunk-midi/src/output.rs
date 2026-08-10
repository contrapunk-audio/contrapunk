//! MIDI output connection and message routing.
//!
//! Provides the `OutputRouter` struct for managing multiple MIDI output
//! connections and routing messages to them.

use anyhow::{anyhow, Result};
use midir::{MidiOutput, MidiOutputConnection};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MidiWireEvent {
    pub device_port: usize,
    pub message: Vec<u8>,
}

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
    /// Optional adapter-boundary wire capture used by deterministic diagnostics.
    trace: Option<Vec<MidiWireEvent>>,
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
            trace: None,
        })
    }

    /// Construct a no-hardware router that records the exact bytes each stable
    /// device port would receive.
    pub fn recording(port_indices: &[usize]) -> Self {
        Self {
            connections: Vec::new(),
            port_indices: port_indices.to_vec(),
            trace: Some(Vec::new()),
        }
    }

    pub fn take_trace(&mut self) -> Vec<MidiWireEvent> {
        self.trace.as_mut().map(std::mem::take).unwrap_or_default()
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
        if self.trace.is_some() {
            return self.send_to_port(0, message);
        }
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
        if self.trace.is_some() {
            for index in 0..self.port_indices.len() {
                self.send_to_port(index, message)?;
            }
            return Ok(());
        }
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
        self.port_indices.len()
    }

    /// Original system MIDI port indices represented by the connection pool.
    pub fn connected_port_indices(&self) -> &[usize] {
        &self.port_indices
    }

    /// Send to a stable system MIDI port index rather than a mutable position
    /// in the connection pool.
    pub fn send_to_device_port(&mut self, device_port_index: usize, message: &[u8]) -> Result<()> {
        let connection_index = connection_index(&self.port_indices, device_port_index)
            .ok_or_else(|| anyhow!("MIDI output port {device_port_index} is not connected"))?;
        self.send_to_port(connection_index, message)
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
        if let Some(trace) = self.trace.as_mut() {
            let device_port = *self.port_indices.get(port_index).ok_or_else(|| {
                anyhow!(
                    "Port index {} out of range (have {} ports)",
                    port_index,
                    self.port_indices.len()
                )
            })?;
            trace.push(MidiWireEvent {
                device_port,
                message: message.to_vec(),
            });
            return Ok(());
        }
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

fn connection_index(port_indices: &[usize], device_port_index: usize) -> Option<usize> {
    port_indices
        .iter()
        .position(|&index| index == device_port_index)
}

#[cfg(test)]
mod tests {
    use super::{connection_index, MidiWireEvent, OutputRouter};

    #[test]
    fn device_port_identity_survives_connection_reordering() {
        assert_eq!(connection_index(&[8, 4, 6], 4), Some(1));
        assert_eq!(connection_index(&[6, 8], 4), None);
    }

    #[test]
    fn recording_router_reports_unavailable_device_instead_of_falling_back() {
        let mut router = OutputRouter::recording(&[8]);
        let error = router.send_to_device_port(4, &[0x90, 60, 100]).unwrap_err();
        assert!(error.to_string().contains("not connected"));
        assert!(router.take_trace().is_empty());
    }

    #[test]
    fn recording_router_captures_stable_device_ports_and_exact_bytes() {
        let mut router = OutputRouter::recording(&[8, 4]);
        router.send_to_device_port(4, &[0x91, 60, 99]).unwrap();
        router.send_to_all(&[0xb0, 64, 0]).unwrap();

        assert_eq!(
            router.take_trace(),
            [
                MidiWireEvent {
                    device_port: 4,
                    message: vec![0x91, 60, 99],
                },
                MidiWireEvent {
                    device_port: 8,
                    message: vec![0xb0, 64, 0],
                },
                MidiWireEvent {
                    device_port: 4,
                    message: vec![0xb0, 64, 0],
                },
            ]
        );
    }
}
