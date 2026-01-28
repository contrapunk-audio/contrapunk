---
phase: 04-server-mode
plan: 01
subsystem: server
tags: [tcp, protocol, config, binary-framing]
dependency-graph:
  requires: []
  provides: [server-module, wire-protocol, server-config]
  affects: [04-02, 04-03, 04-04]
tech-stack:
  added: []
  patterns: [length-prefixed-framing, generic-read-write]
key-files:
  created: [src/server/mod.rs, src/server/protocol.rs, src/server/config.rs]
  modified: [src/main.rs]
decisions:
  - id: 04-01-01
    decision: "Length-prefixed binary protocol with u16 BE length field"
    reason: "Reliable message framing over TCP streams"
  - id: 04-01-02
    decision: "Generic Read/Write traits instead of TcpStream"
    reason: "Protocol testable with in-memory buffers"
metrics:
  duration: "2 min"
  completed: 2026-01-28
---

# Phase 04 Plan 01: Server Wire Protocol and Config Summary

**One-liner:** Length-prefixed binary protocol over generic Read/Write with 5 message types and ServerConfig defaults

## What Was Done

### Task 1: Create server config and protocol types
- Created `ServerConfig` with `port: u16` (default 9900) and `max_clients: usize` (default 10)
- Created `Message` enum: MidiData, Configure, Ack, Disconnect, Heartbeat
- Implemented `read_message`/`write_message` with `[u16 BE length][u8 type][payload]` framing
- Implemented `parse_message`/`serialize_message` for type+payload encoding
- Added 5 roundtrip tests covering all message variants
- Commit: `1adf8f6`

### Task 2: Add server module to main.rs
- Added unconditional `mod server;` declaration
- Verified both `cargo check` and `cargo check --features gui` pass
- Commit: `1477bbd`

## Deviations from Plan

None - plan executed exactly as written.

## Decisions Made

1. **Length-prefixed framing** - `[u16 BE length][u8 type][payload]` where length covers type+payload
2. **Generic streams** - Functions use `impl Read`/`impl Write` for testability without TCP

## Verification Results

- `cargo check` passes (CLI build)
- `cargo check --features gui` passes (GUI build)
- 5/5 protocol roundtrip tests pass
- `read_message` and `write_message` present in protocol.rs
- `ServerConfig` with default port 9900 present in config.rs

## Next Phase Readiness

Plan 04-02 (TCP listener and session management) can proceed. Protocol types are ready for use.
