# External Integrations

**Analysis Date:** 2026-02-04

## APIs & External Services

**Browser APIs (WASM builds only):**
- Web MIDI API - MIDI device access in browser
  - SDK/Client: web-sys crate (MidiAccess, MidiInput, MidiOutput, MidiPort, MidiMessageEvent)
  - Implementation: `src/midi/web.rs`
  - Auth: Browser permissions prompt via `navigator.requestMIDIAccess()`
  - Features used: MIDI input/output enumeration, message handling, device connections

**System APIs (Native builds only):**
- ALSA/CoreMIDI/Windows MIDI - Platform-native MIDI I/O
  - SDK/Client: midir 0.10
  - Implementation: `src/midi/ports.rs`, `src/midi/input.rs`, `src/midi/output.rs`
  - Auth: Direct system access (no credentials required)

## Data Storage

**Databases:**
- None - No external database

**File Storage:**
- Local filesystem only
- eframe persistence API for GUI state and presets
  - Storage location: Platform-specific (browser localStorage for WASM, OS-specific for native)
  - Implementation: `src/preset/storage.rs`
  - Data format: JSON serialization via serde_json
  - Stored data: Custom harmony presets, GUI state

**Caching:**
- None - No external caching layer

## Authentication & Identity

**Auth Provider:**
- None - No authentication system
- Application runs locally or as self-hosted service
- No user accounts or identity management

## Monitoring & Observability

**Error Tracking:**
- None - No external error tracking service
- WASM: console_error_panic_hook for browser console output
- Native: Standard Rust panic handling

**Logs:**
- Console output only (println!/eprintln! macros)
- No structured logging framework
- Debug output in client/server protocol: `[client]` prefixed messages in `src/main.rs` run_client()

## CI/CD & Deployment

**Hosting:**
- Fly.io - Static WASM build hosting
  - Config: `deploy/fly.toml`
  - App name: contrapunk
  - Region: ewr (US East)
  - Machine: shared-cpu-1x, 256MB memory
  - Auto-scaling: stop when idle, start on demand

**CI Pipeline:**
- GitHub Actions - `.github/workflows/ci.yml`
  - Jobs: check (cargo check), test (cargo test), wasm-check (WASM target validation)
  - Deploy: Automatic on main branch push
  - Build artifact: WASM bundle via Trunk
  - Deployment: flyctl deploy with FLY_API_TOKEN secret

**Container:**
- Docker - `deploy/Dockerfile`
  - Base image: nginx:alpine
  - Serves static files from `dist/`
  - Config: `deploy/nginx.conf`
  - Exposed port: 80

## Environment Configuration

**Required env vars:**
- None for runtime
- FLY_API_TOKEN - GitHub Actions secret for deployment (CI/CD only)

**Secrets location:**
- GitHub repository secrets (for CI/CD)
- No application secrets required

## Webhooks & Callbacks

**Incoming:**
- None

**Outgoing:**
- None

## Network Protocols

**Custom TCP Protocol:**
- Contrapunk Server Protocol - Real-time MIDI streaming over TCP
  - Implementation: `src/server/protocol.rs`, `src/server/session.rs`, `src/server/mod.rs`
  - Port: 9900 (default, configurable via `--port`)
  - Wire format: Length-prefixed messages `[u16 BE length][u8 type][payload]`
  - Message types:
    - 0x01: MidiData - Raw MIDI bytes
    - 0x02: Configure - Harmony engine settings (key, mode, octave_mode, voice_count)
    - 0x03: Ack - Acknowledgement
    - 0x04: Disconnect - Clean connection close
    - 0x05: Heartbeat - Keep-alive
  - Client mode: `--client <host:port>` streams local MIDI to remote server
  - Server mode: `--server` accepts MIDI streams and generates harmony
  - Connection: TCP with nodelay, 30s read timeout, 5s write timeout
  - Max clients: 10 (default, configured in `src/server/config.rs`)

---

*Integration audit: 2026-02-04*
