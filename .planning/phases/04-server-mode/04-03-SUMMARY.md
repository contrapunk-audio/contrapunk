---
phase: 04-server-mode
plan: 03
subsystem: cli-integration
tags: [clap, cli, client-mode, tcp, midi-streaming]

dependency-graph:
  requires: [04-02]
  provides: ["CLI argument parsing", "server launch via --server", "client mode via --client", "three-mode binary"]
  affects: [04-04]

tech-stack:
  added: ["clap 4 (derive)"]
  patterns: ["clap derive Args struct", "cfg-gated client mode", "AtomicUsize voice counter for output routing"]

file-tracking:
  key-files:
    created: []
    modified: ["Cargo.toml", "src/main.rs"]

decisions:
  - id: "04-03-01"
    decision: "clap Args struct is unconditional (both GUI and CLI builds parse args)"
    rationale: "Server mode works in both builds; only client mode is CLI-gated"
  - id: "04-03-02"
    decision: "Client mode cfg-gated to CLI build only"
    rationale: "Client requires stdin prompts for MIDI port selection, incompatible with GUI event loop"
  - id: "04-03-03"
    decision: "AtomicUsize voice counter for round-robin output routing"
    rationale: "Simple lock-free coordination between writer and reader threads"

metrics:
  duration: "2 min"
  completed: "2026-01-28"
---

# Phase 4 Plan 3: CLI Integration and Client Mode Summary

**One-liner:** clap CLI with --server/--client/--port flags and full client mode with MIDI I/O routing via AtomicUsize voice counter

## What Was Done

### Task 1: Add clap and CLI argument parsing
- Added `clap = { version = "4", features = ["derive"] }` to Cargo.toml
- Defined `Args` struct with `--server`, `--client <host:port>`, `--port` flags
- Updated `main()` to route: server mode -> `run_server()`, client mode -> `run_client()`, default -> GUI or CLI
- Server mode works in both GUI and CLI builds
- Client mode in GUI build prints error and exits with code 1
- `cargo run -- --help` shows all three flags

### Task 2: Implement client mode
- Full `run_client(addr)` function (~140 lines) gated with `#[cfg(not(feature = "gui"))]`
- Connects TcpStream with nodelay, read timeout 30s, write timeout 5s
- Reuses existing `select_input_port`, `select_output_ports`, `select_key`, `select_mode`, `select_octave_mode`
- Sends Configure message with key/mode/octave_mode/voice_count indices
- Waits for Ack from server
- Reader thread receives MidiData from server, routes to local outputs via `voice_counter.fetch_add(1) % voice_count`
- Main thread reads local MIDI input, resets voice_counter to 0 on each new message, sends MidiData to server
- Enter-key detection for clean shutdown with Disconnect message

## Deviations from Plan

None - plan executed exactly as written.

## Commits

| # | Hash | Type | Description |
|---|------|------|-------------|
| 1 | 2932c65 | feat | Add clap CLI argument parsing |
| 2 | b967860 | feat | Implement client mode |

## Verification

- `cargo check` passes (CLI build)
- `cargo check --features gui` passes (GUI build)
- `cargo run -- --help` shows --server, --client, --port flags
- run_client is cfg-gated out of GUI build
- GUI build with --client prints "Client mode requires CLI build" error

## Next Phase Readiness

Plan 04-04 (integration testing) can proceed. All server and client code compiles and is wired together. The binary supports three modes: GUI (default with feature), CLI (default without), server (--server), client (--client host:port).
