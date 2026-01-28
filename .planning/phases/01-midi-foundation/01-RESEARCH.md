# Phase 1: MIDI Foundation - Research

**Researched:** 2026-01-28
**Domain:** Rust MIDI I/O with midir crate (cross-platform)
**Confidence:** HIGH

## Summary

This phase establishes MIDI input/output connectivity in Rust using the **midir** crate - the de facto standard for cross-platform real-time MIDI in Rust. The research confirms midir v0.10.x is actively maintained, well-documented, and provides all features needed for this phase: port enumeration, input callbacks, output sending, and cross-platform support (CoreMIDI on macOS, ALSA on Linux, WinMM on Windows).

The architecture follows a callback-based pattern where MIDI input runs on a separate thread managed by midir. Messages received in the callback can be forwarded to output ports using Rust's channel primitives (`std::sync::mpsc`) for thread-safe communication. This matches the pass-through requirement perfectly.

For Phase 1 (CLI/headless mode), no TUI framework is needed - simple stdin/stdout prompts suffice for device selection. The Python reference implementation uses `mido` (Python's rtmidi wrapper); midir provides equivalent functionality with a Rust-idiomatic API.

**Primary recommendation:** Use `midir` for MIDI I/O with `std::sync::mpsc` channels for forwarding messages from input callback to output sender thread. Keep Phase 1 minimal - CLI only, no TUI libraries.

## Standard Stack

The established libraries/tools for this domain:

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| midir | 0.10.x | MIDI I/O (ports, send/receive) | Only mature cross-platform MIDI crate; inspired by RtMidi; 350K+ downloads |
| std::sync::mpsc | stable | Thread communication for forwarding | Built-in, zero-overhead, perfect for callback-to-main communication |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| wmidi | 4.0.x | MIDI message parsing/encoding | If you need typed MIDI messages (NoteOn, NoteOff, etc.) instead of raw bytes |
| anyhow | 1.x | Error handling | Simplifies error propagation in application code |
| thiserror | 1.x | Custom error types | If defining library-style error enums |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| midir | coremidi (direct) | macOS-only, lower-level, more control but no cross-platform |
| wmidi | midi-msg | midi-msg has more features (MIDI 2.0, SysEx) but more complex; wmidi is no_std compatible |
| mpsc channels | crossbeam-channel | crossbeam is faster but overkill for MIDI message rates |

**Installation:**
```bash
cargo add midir
# Optional for typed messages:
cargo add wmidi
```

**Cargo.toml:**
```toml
[dependencies]
midir = "0.10"
# Optional:
wmidi = "4.0"     # For typed MIDI message parsing
anyhow = "1.0"    # For ergonomic error handling
```

## Architecture Patterns

### Recommended Project Structure
```
src/
├── main.rs          # Entry point, CLI argument handling
├── midi/
│   ├── mod.rs       # Module re-exports
│   ├── ports.rs     # Port enumeration and selection
│   ├── input.rs     # Input connection and callback setup
│   └── output.rs    # Output connection and message sending
└── router.rs        # Pass-through logic (input -> outputs routing)
```

### Pattern 1: Callback-to-Channel Forwarding
**What:** midir's input callback runs on a separate thread. Use mpsc channel to forward messages to main thread for routing to outputs.
**When to use:** Always for input processing - this is the standard pattern.
**Example:**
```rust
// Source: midir examples + standard Rust patterns
use std::sync::mpsc;
use midir::{MidiInput, MidiOutput, MidiInputConnection, MidiOutputConnection, Ignore};

fn setup_forwarding() -> Result<(), Box<dyn std::error::Error>> {
    // Create channel for forwarding
    let (tx, rx) = mpsc::channel::<Vec<u8>>();

    // Setup input
    let midi_in = MidiInput::new("contrapunk-input")?;
    let in_port = &midi_in.ports()[0]; // Select port (simplified)

    let _conn_in = midi_in.connect(
        in_port,
        "contrapunk-in",
        move |_timestamp, message, _| {
            // Forward message to main thread
            tx.send(message.to_vec()).unwrap();
        },
        (),
    )?;

    // Setup output
    let midi_out = MidiOutput::new("contrapunk-output")?;
    let out_port = &midi_out.ports()[0]; // Select port (simplified)
    let mut conn_out = midi_out.connect(out_port, "contrapunk-out")?;

    // Main loop: receive and forward
    loop {
        if let Ok(message) = rx.recv() {
            conn_out.send(&message)?;
        }
    }
}
```

### Pattern 2: Port Enumeration and Selection
**What:** List available ports, let user select by index.
**When to use:** At startup for device configuration.
**Example:**
```rust
// Source: midir test_list_ports.rs example
use midir::{MidiInput, MidiOutput};

fn list_and_select_input() -> Result<usize, Box<dyn std::error::Error>> {
    let midi_in = MidiInput::new("contrapunk")?;
    let ports = midi_in.ports();

    if ports.is_empty() {
        return Err("No MIDI input ports available".into());
    }

    println!("Available MIDI input ports:");
    for (i, port) in ports.iter().enumerate() {
        println!("  {}: {}", i, midi_in.port_name(port)?);
    }

    // Read user selection
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    let selection: usize = input.trim().parse()?;

    if selection >= ports.len() {
        return Err("Invalid port selection".into());
    }

    Ok(selection)
}
```

### Pattern 3: Multiple Output Connections
**What:** Maintain multiple output connections for routing to different ports.
**When to use:** Phase 1 requires 2-8 output ports.
**Example:**
```rust
// Source: Derived from midir patterns
use midir::{MidiOutput, MidiOutputConnection};

struct OutputRouter {
    connections: Vec<MidiOutputConnection>,
}

impl OutputRouter {
    fn new(port_indices: &[usize]) -> Result<Self, Box<dyn std::error::Error>> {
        let mut connections = Vec::new();

        for &idx in port_indices {
            let midi_out = MidiOutput::new(&format!("contrapunk-out-{}", idx))?;
            let ports = midi_out.ports();
            let port = ports.get(idx).ok_or("Invalid port index")?;
            let conn = midi_out.connect(port, &format!("contrapunk-{}", idx))?;
            connections.push(conn);
        }

        Ok(Self { connections })
    }

    fn send_to_first(&mut self, message: &[u8]) -> Result<(), midir::SendError> {
        if let Some(conn) = self.connections.first_mut() {
            conn.send(message)?;
        }
        Ok(())
    }

    fn send_to_all(&mut self, message: &[u8]) -> Result<(), midir::SendError> {
        for conn in &mut self.connections {
            conn.send(message)?;
        }
        Ok(())
    }
}
```

### Anti-Patterns to Avoid
- **Blocking in callback:** Never do heavy work in the MIDI input callback - it runs on a real-time thread. Just forward to channel.
- **Allocating in callback:** Avoid `Vec::new()` or string allocations in hot path. Pre-allocate or use fixed buffers.
- **Sharing MidiOutputConnection across threads:** It's `Send` but not `Sync`. Move ownership to one thread or use channel.
- **Ignoring connection lifetime:** The `MidiInputConnection` must stay alive. If dropped, connection closes. Use `_conn` prefix for intentional hold.

## Don't Hand-Roll

Problems that look simple but have existing solutions:

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| MIDI I/O abstraction | Custom FFI to CoreMIDI/ALSA | midir | Platform differences are complex; SysEx handling, virtual ports, timing |
| MIDI message parsing | Manual byte parsing | wmidi or handle raw bytes | Off-by-one errors, running status, SysEx boundaries |
| Thread-safe message passing | Arc<Mutex<VecDeque>> | std::sync::mpsc | Channels are purpose-built, lock-free sender |
| Cross-platform paths | cfg blocks everywhere | midir's backend abstraction | Already handles CoreMIDI/ALSA/WinMM |

**Key insight:** MIDI seems simple (just 3-byte messages) but has edge cases: running status, SysEx variable length, timing precision, platform quirks. Use established crates.

## Common Pitfalls

### Pitfall 1: Connection Dropped Prematurely
**What goes wrong:** MIDI stops working unexpectedly.
**Why it happens:** `MidiInputConnection` or `MidiOutputConnection` goes out of scope and drops.
**How to avoid:** Store connections in a struct or use `_conn_in` naming to indicate intentional hold. Keep in scope for entire runtime.
**Warning signs:** MIDI works briefly then stops; no error messages.

### Pitfall 2: Callback Closure Captures Wrong Data
**What goes wrong:** Callback can't access needed data or causes borrow checker errors.
**Why it happens:** Callback must be `'static + Send`. Can't capture references to local variables.
**How to avoid:** Use `move` closure. For shared state, use channels or `Arc<Mutex<T>>`.
**Warning signs:** Compiler errors about lifetimes or `Send` bounds.

### Pitfall 3: Port Index Changes Between Enumeration and Connection
**What goes wrong:** Connect to wrong device or get "invalid port" error.
**Why it happens:** Ports are enumerated once, but devices can be plugged/unplugged.
**How to avoid:** Keep `MidiInput`/`MidiOutput` alive between `ports()` call and `connect()`. Re-enumerate if significant time passes.
**Warning signs:** "Port no longer available" errors; wrong device selected.

### Pitfall 4: Ignoring Message Types (Note On with velocity 0)
**What goes wrong:** Notes get stuck or don't release.
**Why it happens:** MIDI spec allows Note On with velocity 0 as Note Off. Code only checks for Note Off message type.
**How to avoid:** Treat `Note On velocity=0` same as `Note Off`. For pass-through, not an issue - just forward raw bytes.
**Warning signs:** Stuck notes that won't stop until device restart.

### Pitfall 5: Client Name Collisions
**What goes wrong:** Multiple instances conflict or ports disappear.
**Why it happens:** CoreMIDI and some backends use client name for identification.
**How to avoid:** Use unique client names. Include instance ID if running multiple copies.
**Warning signs:** Ports show duplicate names; connections fail silently.

## Code Examples

Verified patterns from official sources:

### Complete MIDI Forward Example
```rust
// Source: Adapted from midir test_forward.rs
use std::error::Error;
use std::io::{stdin, stdout, Write};
use midir::{MidiInput, MidiOutput, Ignore};

fn main() -> Result<(), Box<dyn Error>> {
    let mut midi_in = MidiInput::new("contrapunk-in")?;
    midi_in.ignore(Ignore::None); // Don't filter any message types

    let midi_out = MidiOutput::new("contrapunk-out")?;

    // Get ports
    let in_ports = midi_in.ports();
    let out_ports = midi_out.ports();

    // Display and select input
    println!("Available input ports:");
    for (i, p) in in_ports.iter().enumerate() {
        println!("  {}: {}", i, midi_in.port_name(p)?);
    }
    print!("Select input port: ");
    stdout().flush()?;
    let mut input = String::new();
    stdin().read_line(&mut input)?;
    let in_idx: usize = input.trim().parse()?;
    let in_port = &in_ports[in_idx];

    // Display and select output
    println!("Available output ports:");
    for (i, p) in out_ports.iter().enumerate() {
        println!("  {}: {}", i, midi_out.port_name(p)?);
    }
    print!("Select output port: ");
    stdout().flush()?;
    input.clear();
    stdin().read_line(&mut input)?;
    let out_idx: usize = input.trim().parse()?;
    let out_port = &out_ports[out_idx];

    // Connect output first (will be moved into closure)
    let mut conn_out = midi_out.connect(out_port, "contrapunk-forward")?;

    // Connect input with forwarding callback
    let _conn_in = midi_in.connect(
        in_port,
        "contrapunk-read",
        move |stamp, message, _| {
            // Forward to output
            conn_out.send(message).unwrap_or_else(|e| {
                eprintln!("Error forwarding: {:?}", e);
            });
            println!("{}: {:?} (len={})", stamp, message, message.len());
        },
        (),
    )?;

    println!("Forwarding MIDI. Press Enter to exit.");
    input.clear();
    stdin().read_line(&mut input)?;

    Ok(())
}
```

### Port Selection with Validation
```rust
// Source: midir examples pattern
use midir::{MidiInput, MidiOutput, MidiInputPort, MidiOutputPort};
use std::io::{self, Write};

fn select_port<T: midir::MidiIO>(
    midi_io: &T,
    prompt: &str,
) -> Result<T::Port, Box<dyn std::error::Error>>
where
    T::Port: Clone,
{
    let ports = midi_io.ports();

    if ports.is_empty() {
        return Err(format!("No {} ports available", prompt).into());
    }

    println!("Available {} ports:", prompt);
    for (i, port) in ports.iter().enumerate() {
        println!("  {}: {}", i, midi_io.port_name(port)?);
    }

    loop {
        print!("Select {} port [0-{}]: ", prompt, ports.len() - 1);
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        match input.trim().parse::<usize>() {
            Ok(idx) if idx < ports.len() => {
                return Ok(ports[idx].clone());
            }
            _ => {
                println!("Invalid selection. Please try again.");
            }
        }
    }
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| rtmidi (C++) via FFI | midir (pure Rust) | 2016+ | No C++ dependency, better safety |
| portmidi-rs | midir | ~2018 | midir has better virtual port support |
| Manual platform cfg | midir backends | Always | Single API for all platforms |

**Deprecated/outdated:**
- `portmidi-rs`: Less maintained, midir is the standard
- Direct `coremidi` crate: Use only if macOS-specific features needed
- `alsa-rs` directly: Use only if Linux-specific features needed

## Open Questions

Things that couldn't be fully resolved:

1. **Timestamp precision for future scheduling**
   - What we know: midir provides timestamps on received messages but doesn't support scheduled future sending
   - What's unclear: If future phases need precise timing sync with audio, may need additional work
   - Recommendation: For Phase 1 pass-through, not an issue. Revisit if latency becomes problematic.

2. **Hot-plug device handling**
   - What we know: Ports are enumerated at a point in time; midir doesn't have device change notifications
   - What's unclear: Best pattern for handling device disconnect mid-session
   - Recommendation: For Phase 1, handle errors gracefully. Consider periodic re-enumeration for UI in Phase 3.

3. **Multiple clients with same name on macOS**
   - What we know: CoreMIDI uses client names; duplicates may cause issues
   - What's unclear: Exact behavior with multiple instances
   - Recommendation: Use unique client names (e.g., include PID or timestamp).

## Sources

### Primary (HIGH confidence)
- [midir GitHub repository](https://github.com/Boddlnagg/midir) - README, examples
- [midir docs.rs](https://docs.rs/midir/latest/midir/) - API documentation
- [wmidi GitHub/crates.io](https://github.com/RustAudio/wmidi) - MIDI message types

### Secondary (MEDIUM confidence)
- [Rust Forum MIDI Router discussion](https://users.rust-lang.org/t/a-simple-midi-router-application/123030) - Community patterns
- [RustAudio organization](https://github.com/RustAudio) - Ecosystem overview

### Tertiary (LOW confidence)
- WebSearch results for ecosystem trends - General direction verified with primary sources

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - midir is clearly the standard, well-documented
- Architecture: HIGH - Callback + channel pattern is well-established
- Pitfalls: MEDIUM - Based on documentation + common Rust patterns, not extensive production experience

**Research date:** 2026-01-28
**Valid until:** 60 days (midir is stable, slow-moving crate)
