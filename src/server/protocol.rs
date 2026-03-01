use anyhow::{anyhow, Result};
use std::io::{Read, Write};

/// Wire protocol message types for TCP-based MIDI streaming.
///
/// Messages use length-prefixed framing: [u16 BE length][u8 type][payload]
/// where length covers type byte + payload.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Message {
    /// Raw MIDI bytes (type 0x01)
    MidiData(Vec<u8>),
    /// Configuration update (type 0x02)
    Configure {
        key: u8,
        mode: u8,
        octave_mode: u8,
        voice_count: u8,
    },
    /// Acknowledgement (type 0x03)
    Ack,
    /// Client disconnect (type 0x04)
    Disconnect,
    /// Keep-alive heartbeat (type 0x05)
    Heartbeat,
}

const TYPE_MIDI_DATA: u8 = 0x01;
const TYPE_CONFIGURE: u8 = 0x02;
const TYPE_ACK: u8 = 0x03;
const TYPE_DISCONNECT: u8 = 0x04;
const TYPE_HEARTBEAT: u8 = 0x05;

/// Read a single length-prefixed message from a stream.
///
/// Wire format: [u16 BE length][u8 type][payload]
/// The length field covers type byte + payload bytes.
pub fn read_message(stream: &mut impl Read) -> Result<Message> {
    // Read 2-byte big-endian length
    let mut len_buf = [0u8; 2];
    stream.read_exact(&mut len_buf)?;
    let len = u16::from_be_bytes(len_buf) as usize;

    if len == 0 {
        return Err(anyhow!("invalid message: zero length"));
    }

    // Read type + payload
    let mut buf = vec![0u8; len];
    stream.read_exact(&mut buf)?;

    parse_message(&buf)
}

/// Write a length-prefixed message to a stream and flush.
pub fn write_message(stream: &mut impl Write, msg: &Message) -> Result<()> {
    let body = serialize_message(msg);
    let len = body.len() as u16;
    stream.write_all(&len.to_be_bytes())?;
    stream.write_all(&body)?;
    stream.flush()?;
    Ok(())
}

/// Parse a type byte + payload buffer into a Message.
pub fn parse_message(buf: &[u8]) -> Result<Message> {
    if buf.is_empty() {
        return Err(anyhow!("empty message buffer"));
    }

    let msg_type = buf[0];
    let payload = &buf[1..];

    match msg_type {
        TYPE_MIDI_DATA => Ok(Message::MidiData(payload.to_vec())),
        TYPE_CONFIGURE => {
            if payload.len() < 4 {
                return Err(anyhow!(
                    "configure message too short: {} bytes",
                    payload.len()
                ));
            }
            Ok(Message::Configure {
                key: payload[0],
                mode: payload[1],
                octave_mode: payload[2],
                voice_count: payload[3],
            })
        }
        TYPE_ACK => Ok(Message::Ack),
        TYPE_DISCONNECT => Ok(Message::Disconnect),
        TYPE_HEARTBEAT => Ok(Message::Heartbeat),
        _ => Err(anyhow!("unknown message type: 0x{:02x}", msg_type)),
    }
}

/// Serialize a Message to type byte + payload.
pub fn serialize_message(msg: &Message) -> Vec<u8> {
    match msg {
        Message::MidiData(data) => {
            let mut buf = Vec::with_capacity(1 + data.len());
            buf.push(TYPE_MIDI_DATA);
            buf.extend_from_slice(data);
            buf
        }
        Message::Configure {
            key,
            mode,
            octave_mode,
            voice_count,
        } => {
            vec![TYPE_CONFIGURE, *key, *mode, *octave_mode, *voice_count]
        }
        Message::Ack => vec![TYPE_ACK],
        Message::Disconnect => vec![TYPE_DISCONNECT],
        Message::Heartbeat => vec![TYPE_HEARTBEAT],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_roundtrip_midi_data() {
        let msg = Message::MidiData(vec![0x90, 0x3C, 0x7F]);
        let mut buf = Vec::new();
        write_message(&mut buf, &msg).unwrap();
        let mut cursor = Cursor::new(buf);
        let decoded = read_message(&mut cursor).unwrap();
        assert_eq!(decoded, msg);
    }

    #[test]
    fn test_roundtrip_configure() {
        let msg = Message::Configure {
            key: 0,
            mode: 2,
            octave_mode: 1,
            voice_count: 4,
        };
        let mut buf = Vec::new();
        write_message(&mut buf, &msg).unwrap();
        let mut cursor = Cursor::new(buf);
        let decoded = read_message(&mut cursor).unwrap();
        assert_eq!(decoded, msg);
    }

    #[test]
    fn test_roundtrip_ack() {
        let msg = Message::Ack;
        let mut buf = Vec::new();
        write_message(&mut buf, &msg).unwrap();
        let mut cursor = Cursor::new(buf);
        let decoded = read_message(&mut cursor).unwrap();
        assert_eq!(decoded, msg);
    }

    #[test]
    fn test_roundtrip_disconnect() {
        let msg = Message::Disconnect;
        let mut buf = Vec::new();
        write_message(&mut buf, &msg).unwrap();
        let mut cursor = Cursor::new(buf);
        let decoded = read_message(&mut cursor).unwrap();
        assert_eq!(decoded, msg);
    }

    #[test]
    fn test_roundtrip_heartbeat() {
        let msg = Message::Heartbeat;
        let mut buf = Vec::new();
        write_message(&mut buf, &msg).unwrap();
        let mut cursor = Cursor::new(buf);
        let decoded = read_message(&mut cursor).unwrap();
        assert_eq!(decoded, msg);
    }
}
