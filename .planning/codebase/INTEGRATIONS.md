# External Integrations

**Analysis Date:** 2026-03-31

## APIs & External Services

**MIDI (Browser):**
- Web MIDI API - Real-time MIDI input/output in browsers
  - SDK/Client: `web-sys` (Rust WASM) + `navigator.requestMIDIAccess()` (TypeScript `WasmAdapter`)
  - Auth: Browser permission prompt (no API key)
  - Implementation: `src/midi/web.rs` (Rust), `ui/src/lib/adapter/wasm.ts` (TypeScript)

**MIDI (Desktop):**
- CoreMIDI (macOS) / ALSA (Linux) / WinMM (Windows) - OS-level MIDI I/O
  - SDK/Client: `midir` 0.10 crate
  - Implementation: `src/midi/`, `src-tauri/src/commands/midi.rs`
  - Auth: None (OS-level device access)

**No third-party SaaS APIs** detected (no Stripe, Supabase, OpenAI, etc.)

## Data Storage

**Databases:**
- None - No database is used

**File Storage:**
- Local filesystem only
  - Presets stored as JSON files via Tauri's file system APIs on desktop
  - In browser/WASM mode, presets are held in-memory in the WASM engine instance
  - ML training dataset: MessagePack binary files (`ml/` directory); loaded by `ml/loader.py` using `rmp-serde` format

**Caching:**
- None (no Redis, Memcache, etc.)

## Authentication & Identity

**Auth Provider:**
- None - No user authentication exists
  - The application is a local-first MIDI tool with no accounts or login
  - Tauri capabilities are limited to `core:default` + `core:event:default` (see `src-tauri/capabilities/default.json`)

## Monitoring & Observability

**Error Tracking:**
- None (no Sentry, Datadog, etc. detected)

**Logs:**
- Native: `eprintln!()` / `println!()` to stderr/stdout directly in Rust code
- WASM: `console_error_panic_hook` 0.1 routes Rust panics to browser `console.error`
- Frontend: `console.warn()` used in the WASM development stub (`ui/scripts/build-wasm.sh`)

## CI/CD & Deployment

**Hosting:**
- Fly.io - Production web deployment
  - App name: `contrapunk`
  - Region: `ewr` (Newark)
  - VM: `shared-cpu-1x`, 512 MB RAM
  - HTTPS enforced (`force_https = true`)
  - Auto stop/start machines configured
  - Config: `deploy/fly.toml`

**Container:**
- Docker multi-stage build (`deploy/Dockerfile`)
  - Stage 1: `rust:1.88` - builds WASM with wasm-pack
  - Stage 2: `node:22` - builds SvelteKit SPA
  - Stage 3: `nginx:alpine` - serves static files on port 8080

**CI Pipeline:**
- GitHub Actions (`.github/workflows/ci.yml`)
  - Jobs: `fmt` (rustfmt), `clippy`, `check` (cargo check native), `test` (cargo test), `wasm-check` (wasm32 target), `tauri-check`, `frontend` (SvelteKit build)
  - Rust cache: `Swatinem/rust-cache@v2` with separate shared keys (`native`, `wasm`, `tauri`)
  - Deploy job runs on push to `main` only, after all other jobs pass
  - Uses: `actions/checkout@v4`, `dtolnay/rust-toolchain@stable`, `actions/setup-node@v4`

## Environment Configuration

**Required env vars:**
- `FLY_API_TOKEN` - GitHub Actions secret for Fly.io deployment (only needed in CI)

**No application-level env vars** - the app has no runtime configuration via environment variables

**Secrets location:**
- `FLY_API_TOKEN` stored as a GitHub Actions secret (`secrets.FLY_API_TOKEN`), referenced only in `.github/workflows/ci.yml`

## Platform IPC (Tauri)

**Tauri Commands (Desktop ↔ Webview):**
- Pattern: `invoke('command_name', { ...args })` from TypeScript → Rust handler
- Commands exposed: `get_engine_state`, `set_key`, `set_mode`, `set_scale_mode`, `set_octave_mode`, `set_voice_leading`, `set_interchange`, `set_voice_position`, `list_midi_inputs`, `list_midi_outputs`, `refresh_midi_devices`, `start_routing`, `stop_routing`, `get_note_state`, `list_presets`, `load_preset`, `save_preset`, `delete_preset`
- Implementation: `src-tauri/src/commands/` (Rust), `ui/src/lib/adapter/tauri.ts` (TypeScript)

**Tauri Events (Rust → Webview):**
- `note-update` - Emitted by Rust backend when harmony notes change
  - Payload: `{ input_notes, harmony_notes, borrowed_notes, chord_name, last_borrowed_from }`
  - Subscribed via `listen('note-update', callback)` in `ui/src/lib/adapter/tauri.ts`

## TCP Server/Client (Network Mode)

**Incoming:**
- `contrapunk --server [--port 9900]` - Binds TCP on `0.0.0.0:{port}`
  - Protocol: custom MessagePack binary framing (`src/server/protocol.rs`)
  - Messages: `Configure`, `Ack`, `MidiData`, `Heartbeat`, `Disconnect`
  - Max clients: configurable (`src/server/config.rs`)

**Outgoing:**
- `contrapunk --client <host:port>` - Connects to a remote server
  - Streams local MIDI input to server, routes harmonized MIDI back to local output ports
  - Implementation: `src/main.rs` (`run_client` function)

## Webhooks & Callbacks

**Incoming:**
- None (no webhook endpoints)

**Outgoing:**
- None (no outgoing webhooks)

---

*Integration audit: 2026-03-31*
