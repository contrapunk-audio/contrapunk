# Phase 4: Server Mode - Research

**Researched:** 2026-01-28
**Domain:** Rust TCP networking, real-time MIDI streaming, concurrent session management
**Confidence:** MEDIUM

## Summary

Phase 4 adds a server mode where the Contrapunk binary listens on a TCP port, accepts remote clients sending MIDI data, processes it through the existing HarmonyEngine, and returns harmonized MIDI output. The existing codebase is entirely synchronous (std::sync::mpsc, std::thread). The key architectural question is whether to introduce tokio async or stay with std::net + threads.

**Recommendation: Use std::net with OS threads.** The existing codebase is fully synchronous. The expected client count is low (likely <10 concurrent musicians). Introducing tokio would require restructuring the entire application and adding significant dependency weight. A thread-per-client model with std::net::TcpListener is simpler, matches the existing architecture, and is perfectly adequate for the expected load. Each client gets its own HarmonyEngine instance (they are cheap to create) and its own thread.

**Primary recommendation:** Thread-per-client TCP server using std::net with a simple length-prefixed binary protocol for MIDI messages.

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| std::net | (stdlib) | TCP listener/streams | No new dependencies; matches existing sync architecture |
| std::thread | (stdlib) | Per-client threads | Already used throughout codebase |
| std::sync::mpsc | (stdlib) | Inter-thread communication | Already used for MIDI routing |
| clap | 4.x | CLI argument parsing (--server, --port) | De facto standard for Rust CLI args |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| serde + bincode | latest | Binary serialization for protocol messages | If protocol needs structured messages beyond raw MIDI bytes |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| std::net threads | tokio async | Scales better to 1000s of clients but adds ~30 deps, requires restructuring entire app. Not needed for <10 musician clients. |
| Raw TCP | WebSocket (tungstenite) | Browser client support, but adds latency and complexity. Only if web client is a future requirement. |
| Custom binary protocol | OSC (Open Sound Control) | Industry standard for music networking, but heavier than needed for simple MIDI relay. Consider if interop with other music software is desired. |

**Installation:**
```bash
cargo add clap --features derive
```

## Architecture Patterns

### Recommended Project Structure
```
src/
├── server/
│   ├── mod.rs          # Server startup, TcpListener accept loop
│   ├── protocol.rs     # Wire protocol: framing, message types
│   ├── session.rs      # Per-client session: read MIDI, harmonize, write back
│   └── config.rs       # Server configuration (port, max clients)
├── harmony/            # (existing, unchanged)
├── midi/               # (existing, unchanged)
├── router.rs           # (existing, unchanged)
├── app.rs              # (existing GUI, unchanged)
└── main.rs             # Add --server flag to launch server mode
```

### Pattern 1: Thread-per-Client Server
**What:** Main thread accepts TCP connections in a loop. Each accepted connection spawns a new thread that owns a TcpStream and a HarmonyEngine.
**When to use:** Low client count (<100), simple request/response per message.
**Example:**
```rust
// Source: std::net documentation + existing codebase patterns
use std::net::TcpListener;
use std::thread;

fn run_server(addr: &str) -> Result<()> {
    let listener = TcpListener::bind(addr)?;
    println!("Server listening on {}", addr);

    for stream in listener.incoming() {
        let stream = stream?;
        let peer = stream.peer_addr()?;
        println!("Client connected: {}", peer);

        thread::spawn(move || {
            if let Err(e) = handle_client(stream) {
                eprintln!("Client {} error: {}", peer, e);
            }
        });
    }
    Ok(())
}

fn handle_client(mut stream: TcpStream) -> Result<()> {
    let mut engine = HarmonyEngine::default();
    // Read framed MIDI messages, harmonize, write back
    loop {
        let msg = read_message(&mut stream)?;
        match msg {
            Protocol::MidiData(bytes) => {
                let harmonized = process_through_engine(&bytes, &mut engine);
                write_message(&mut stream, &Protocol::MidiData(harmonized))?;
            }
            Protocol::Configure { key, mode, octave_mode } => {
                engine.set_key(key);
                engine.set_mode(mode);
                engine.set_octave_mode(octave_mode);
                write_message(&mut stream, &Protocol::Ack)?;
            }
            Protocol::Disconnect => break,
        }
    }
    Ok(())
}
```

### Pattern 2: Length-Prefixed Binary Protocol
**What:** Each message on the wire is: [2-byte big-endian length][message type byte][payload]. This is the simplest reliable framing for TCP.
**When to use:** Always for TCP protocols carrying variable-length messages.
**Example:**
```rust
// Wire format:
// [u16 BE length (of type + payload)] [u8 message type] [payload bytes...]
//
// Message types:
//   0x01 = MIDI data (payload = raw MIDI bytes, 1-3 bytes typically)
//   0x02 = Configure (payload = key:u8, mode:u8, octave_mode:u8, voice_count:u8)
//   0x03 = Ack (no payload)
//   0x04 = Disconnect (no payload)
//   0x05 = Heartbeat (no payload, for keepalive)

use std::io::{Read, Write};

fn read_message(stream: &mut TcpStream) -> Result<Message> {
    let mut len_buf = [0u8; 2];
    stream.read_exact(&mut len_buf)?;
    let len = u16::from_be_bytes(len_buf) as usize;

    let mut buf = vec![0u8; len];
    stream.read_exact(&mut buf)?;

    parse_message(&buf)
}

fn write_message(stream: &mut TcpStream, msg: &Message) -> Result<()> {
    let payload = serialize_message(msg);
    let len = payload.len() as u16;
    stream.write_all(&len.to_be_bytes())?;
    stream.write_all(&payload)?;
    stream.flush()?;
    Ok(())
}
```

### Pattern 3: Main Entry Point with Server Flag
**What:** Add `--server` and `--port` CLI flags. When `--server` is passed, launch server mode instead of GUI/CLI.
**When to use:** To integrate server mode into the existing binary.
**Example:**
```rust
// main.rs additions
fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();

    if args.contains(&"--server".to_string()) {
        let port = parse_port_arg(&args).unwrap_or(9900);
        return server::run_server(&format!("0.0.0.0:{}", port));
    }

    // existing GUI/CLI logic...
}
```

### Anti-Patterns to Avoid
- **Sharing HarmonyEngine across clients:** Engine has mutable state (active_notes, contrary_motion_state). Each client MUST have its own engine instance.
- **Unbounded reads:** Always use length-prefixed framing. Never read until newline or assume fixed-size messages.
- **Blocking the accept loop:** Never do heavy work in the accept loop. Spawn a thread immediately.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| CLI argument parsing | Manual arg parsing | clap derive macros | Edge cases with flags, help text, validation |
| TCP message framing | Ad-hoc read logic | Length-prefixed protocol (see Pattern 2) | TCP is a stream, not message-based; naive reads will split/merge messages |
| Graceful shutdown | Manual signal handling | ctrlc crate or std::sync::atomic flag | Cross-platform signal handling is tricky |

**Key insight:** The MIDI processing logic already exists in HarmonyEngine. The server is purely a network transport layer that frames MIDI bytes over TCP. Keep it thin.

## Common Pitfalls

### Pitfall 1: TCP Message Boundaries
**What goes wrong:** TCP is a byte stream, not a message stream. A single `read()` call may return half a MIDI message or two messages concatenated.
**Why it happens:** TCP coalesces small writes (Nagle's algorithm) and splits large ones.
**How to avoid:** Always use length-prefixed framing. Read exact number of bytes with `read_exact()`.
**Warning signs:** "Works locally but garbles messages over network" or "works with slow input but breaks with fast playing."

### Pitfall 2: Nagle's Algorithm Latency
**What goes wrong:** Small MIDI messages (3 bytes) get buffered by TCP for up to 200ms before sending.
**Why it happens:** Nagle's algorithm combines small writes into larger packets.
**How to avoid:** Set `TcpStream::set_nodelay(true)` on both server and client sockets.
**Warning signs:** Noticeable latency that disappears when sending larger messages.

### Pitfall 3: Blocked Reads Preventing Clean Shutdown
**What goes wrong:** `read_exact()` blocks forever when client disconnects silently (no FIN packet).
**Why it happens:** Network issues, client crash, firewall timeout.
**How to avoid:** Set read timeout with `stream.set_read_timeout(Some(Duration::from_secs(30)))`. Use heartbeat messages to detect dead clients.
**Warning signs:** Server threads accumulate over time, never cleaning up.

### Pitfall 4: Note-Off Leaks on Disconnect
**What goes wrong:** Client disconnects while notes are held. If client is routing harmonized output to local MIDI devices, stuck notes result.
**Why it happens:** No cleanup protocol on disconnect.
**How to avoid:** Server sends all-notes-off (MIDI CC 123) before closing. Client should also handle disconnect gracefully.
**Warning signs:** Stuck/hanging notes after network interruption.

### Pitfall 5: HarmonyEngine is Not Send+Sync
**What goes wrong:** Trying to share one engine across threads with Arc<Mutex<>> adds contention and latency.
**Why it happens:** Temptation to share state.
**How to avoid:** Each client thread creates its own HarmonyEngine. They are cheap (just a HashMap and some vecs).
**Warning signs:** Mutex contention warnings, increased latency under load.

## Code Examples

### TCP No-Delay Setup
```rust
// Critical for real-time MIDI: disable Nagle's algorithm
use std::net::TcpStream;

fn configure_stream(stream: &TcpStream) -> Result<()> {
    stream.set_nodelay(true)?;  // Disable Nagle's: send immediately
    stream.set_read_timeout(Some(Duration::from_secs(30)))?;
    stream.set_write_timeout(Some(Duration::from_secs(5)))?;
    Ok(())
}
```

### Processing MIDI Through Engine (Server Context)
```rust
// Reuses existing HarmonyEngine API directly
fn process_midi_bytes(bytes: &[u8], engine: &mut HarmonyEngine) -> Vec<Vec<u8>> {
    let msg = match MidiMessage::try_from(bytes) {
        Ok(m) => m,
        Err(_) => return vec![bytes.to_vec()], // Pass through unknown
    };

    match msg {
        MidiMessage::NoteOn(ch, note, vel) => {
            if vel == Velocity::MIN {
                let notes = engine.harmonize_note_off(note);
                notes.iter().map(|&n| {
                    let msg = MidiMessage::NoteOff(ch, n, vel);
                    let mut buf = vec![0u8; msg.bytes_size()];
                    msg.copy_to_slice(&mut buf).unwrap();
                    buf
                }).collect()
            } else {
                let notes = engine.harmonize_note_on(note);
                notes.iter().map(|&n| {
                    let msg = MidiMessage::NoteOn(ch, n, vel);
                    let mut buf = vec![0u8; msg.bytes_size()];
                    msg.copy_to_slice(&mut buf).unwrap();
                    buf
                }).collect()
            }
        }
        MidiMessage::NoteOff(ch, note, vel) => {
            let notes = engine.harmonize_note_off(note);
            notes.iter().map(|&n| {
                let msg = MidiMessage::NoteOff(ch, n, vel);
                let mut buf = vec![0u8; msg.bytes_size()];
                msg.copy_to_slice(&mut buf).unwrap();
                buf
            }).collect()
        }
        _ => vec![bytes.to_vec()],
    }
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Async everything (tokio) | Use threads for low-concurrency servers | Always valid | Simpler code, fewer deps, adequate perf for <100 clients |
| Custom MIDI-over-network | RTP-MIDI (RFC 6295) | 2011 | Industry standard but complex; overkill for this use case |

**Note on RTP-MIDI:** The industry standard for MIDI over network is RTP-MIDI (used by Apple's Network MIDI). It handles jitter compensation, packet loss recovery, and session management. However, it is extremely complex to implement. For Contrapunk's use case (direct TCP connection between known clients), a simple length-prefixed protocol is appropriate. RTP-MIDI could be a future enhancement.

## Open Questions

1. **Client implementation**
   - What we know: Server side is clear. Clients connect, send MIDI, receive harmonized MIDI.
   - What's unclear: Will there be a dedicated client binary/mode, or do users write their own clients? Should `contrapunk --client <host:port>` be part of this phase?
   - Recommendation: Include a `--client` mode that connects to a server, reads from local MIDI input, sends to server, receives harmonized output, and routes to local MIDI output. This makes the feature immediately usable.

2. **Configuration per client**
   - What we know: Each client gets their own HarmonyEngine. They need to configure key, mode, octave_mode.
   - What's unclear: Should configuration be sent as protocol messages, or set at connection time?
   - Recommendation: Protocol messages for configuration. Allows clients to change settings during a session (matches GUI behavior).

3. **Voice count / output routing on client side**
   - What we know: Server returns multiple harmonized notes per input note. Client needs to route them to local MIDI outputs.
   - What's unclear: How does voice count map when client has different number of MIDI outputs than the original local setup?
   - Recommendation: Client specifies desired voice count in configuration message. Server's HarmonyEngine generates that many voices. Client routes to its local outputs.

4. **Default port number**
   - Recommendation: Use port 9900 (not a well-known port, easy to remember). Make configurable via `--port`.

## Sources

### Primary (HIGH confidence)
- Existing Contrapunk codebase (router.rs, engine.rs, main.rs) - analyzed architecture patterns
- Rust std::net documentation - TCP server patterns

### Secondary (MEDIUM confidence)
- [Tokio TCP server patterns](https://tokio.rs/) - confirmed thread-per-client is simpler for low concurrency
- [Custom protocol over TCP](https://www.soup.dev/post/building-a-custom-protocol-over-tcp-with-rust-and-tokio) - length-prefixed framing pattern
- [Rust chat server tutorial](https://developerlife.com/2024/01/13/write-simple-chat-server-in-rust/) - multi-client TCP patterns

### Tertiary (LOW confidence)
- RTP-MIDI as industry standard - mentioned but not deeply investigated

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - stdlib networking is well-understood and matches existing architecture
- Architecture: MEDIUM - thread-per-client is straightforward but protocol design details need validation during implementation
- Pitfalls: HIGH - TCP framing and Nagle issues are well-documented, MIDI-specific issues (note-off leaks) identified from codebase analysis

**Research date:** 2026-01-28
**Valid until:** 2026-03-28 (stable domain, no fast-moving dependencies)
