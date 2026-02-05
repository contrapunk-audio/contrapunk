# External Integrations

**Analysis Date:** 2026-02-05

## APIs & External Services

**MIDI Devices (Hardware/Virtual):**
- Native MIDI I/O - Direct system MIDI device access
  - SDK/Client: `midir` crate (0.10)
  - Auth: No authentication (OS-level device access permissions)
  - Location: `src/midi/ports.rs`, `src/midi/input.rs`, `src/midi/output.rs`
  - Platforms: macOS, Windows, Linux (requires ALSA)

**Web MIDI API (Browser):**
- Browser MIDI access for WASM builds
  - SDK/Client: `web-sys` crate with MIDI features
  - Auth: Browser permission prompt via `navigator.requestMIDIAccess()`
  - Location: `src/midi/web.rs`
  - Sysex: Disabled by default in `MidiOptions`

**No external cloud services or APIs detected.**

## Data Storage

**Databases:**
- None

**File Storage:**
- Local filesystem only (native builds)
- Browser localStorage (WASM builds)
  - Used by eframe persistence feature for app state
  - Preset storage: `src/preset/storage.rs` uses `eframe::Storage` trait
  - Keys: "custom_presets" (JSON-serialized preset array)

**Caching:**
- None

## Authentication & Identity

**Auth Provider:**
- None (standalone application, no user accounts)

## Monitoring & Observability

**Error Tracking:**
- None

**Logs:**
- Native: stderr output via `eprintln!` macros
- WASM: Browser console via `console_error_panic_hook`
  - Panic handler installed in `src/lib.rs` and `src/main.rs` for WASM builds

## CI/CD & Deployment

**Hosting:**
- Fly.io (production)
  - Config: `deploy/fly.toml`
  - App: "contrapunk"
  - Region: ewr (US East)
  - Deployment trigger: Push to main branch after CI passes

**CI Pipeline:**
- GitHub Actions
  - Workflow: `.github/workflows/ci.yml`
  - Jobs:
    - `check` - Runs `cargo check` on native target
    - `test` - Runs `cargo test` on native target
    - `wasm-check` - Verifies WASM compilation with `--target wasm32-unknown-unknown --features wasm`
    - `deploy` - Builds WASM with Trunk and deploys to Fly.io (main branch only)
  - Rust cache: Uses `Swatinem/rust-cache@v2` with separate caches for native and WASM
  - Trunk cache: Caches `~/.cargo/bin/trunk` to avoid reinstalling
  - Linux dependencies: Installs `libasound2-dev` for ALSA support

**Deployment Process:**
1. Trunk builds WASM bundle to `dist/`
2. `dist/` copied to `deploy/dist/`
3. Docker image built from `deploy/Dockerfile` (nginx:alpine + static files)
4. `flyctl deploy` pushes to Fly.io
5. Requires `FLY_API_TOKEN` secret in GitHub repository

## Environment Configuration

**Required env vars:**
- None for application runtime
- `FLY_API_TOKEN` - Required for CI/CD deployment to Fly.io (GitHub Actions secret)

**Secrets location:**
- GitHub repository secrets (for deployment only)

## Webhooks & Callbacks

**Incoming:**
- None

**Outgoing:**
- None

## Network Services

**TCP Server Mode (Optional):**
- Custom harmony server for multi-client MIDI processing
  - Protocol: Custom binary protocol over TCP
  - Location: `src/server/mod.rs`, `src/server/protocol.rs`, `src/server/session.rs`
  - Default port: 9900 (configurable via `--port` CLI flag)
  - Bind address: 0.0.0.0 (all interfaces)
  - Max clients: Configurable via `ServerConfig` (default in `src/server/config.rs`)
  - Message types: Configure, MidiData, Heartbeat, Ack, Disconnect
  - Use case: Run harmony engine on server, connect multiple clients for processing
  - Enabled by: `--server` CLI flag (native builds only)

**TCP Client Mode (Optional):**
- Connects to remote Contrapunk server for distributed MIDI processing
  - Location: `run_client()` function in `src/main.rs`
  - Protocol: Same custom TCP protocol as server
  - Flow:
    1. Connect to server at specified address
    2. Select local MIDI I/O devices
    3. Configure remote harmony engine (key, mode, octave, voice count)
    4. Stream local MIDI input to server
    5. Route harmonized output from server to local MIDI devices
  - Enabled by: `--client <host:port>` CLI flag (native builds only, requires non-GUI build)
  - Timeouts: Read 30s, Write 5s
  - Keep-alive: Responds to server Heartbeat messages

---

*Integration audit: 2026-02-05*
