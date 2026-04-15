# External Integrations

**Analysis Date:** 2026-04-15

## APIs & External Services

**Analytics:**
- PostHog — Product analytics for browser/web deployment
  - SDK: `posthog-js` 1.364.7 (`ui/package.json`)
  - Init: `ui/src/routes/+layout.ts` — dynamically imported, browser-only
  - API host: `https://us.i.posthog.com`
  - Auth: `PUBLIC_POSTHOG_KEY` environment variable (Vite `import.meta.env`)
  - Behaviour: silently skipped if key is absent; not loaded in Tauri desktop context

## Data Storage

**Databases:**
- None — no database layer. All user data is file-based or in-memory.

**File Storage (local):**
- Preset files — serialised as JSON via `serde_json` in `src/preset/storage.rs`
- Guitar training data — `guitar_training_data.msgpack` at repo root; MessagePack blob read by `ml/loader.py` and Rust inference pipeline
- Guitar calibration profile — `guitar_calibration_profile.json` at repo root; loaded by native Rust pipeline and ML export scripts
- ML model weights — binary files exported by `ml/training/export_weights_bin.py`, loaded by hand-written CNN in `src/audio/inference.rs`

**Caching:**
- None — no caching layer.

## Audio Hardware Interfaces

**Audio I/O (native only — excluded from WASM):**
- cpal 0.15.3 — abstracts CoreAudio (macOS), WASAPI (Windows), ALSA/JACK (Linux) for both microphone capture and audio output stream
  - Mic capture: `src/audio/guitar_input.rs` via `cpal::Stream`
  - Audio output: `src/audio_out/engine.rs` — polyphonic sine synth driven by MIDI ringbuffer

**MIDI I/O (native only):**
- midir 0.10.3 — enumerates and connects to physical MIDI ports
  - Input: `src/midi/input.rs` — connects to a named MIDI input port
  - Output: `src/midi/output.rs` + `src/midi/ports.rs` — routes harmony voices to 1–4 MIDI output ports
  - Tauri commands: `commands/midi.rs` — `list_midi_inputs`, `list_midi_outputs`, `refresh_midi_devices`

**Web MIDI API (browser / WASM adapter):**
- `navigator.requestMIDIAccess()` — used by `WasmAdapter` at `ui/src/lib/adapter/wasm.ts`
- Not available in all browsers; adapter falls back gracefully (allows "running" for virtual/keyboard input)

**Web Audio API (browser / WASM adapter):**
- `getUserMedia` + `AudioContext` + `MediaStreamAudioSourceNode` — guitar microphone capture in browser
  - `ui/src/lib/audio/guitarCapture.ts` — `GuitarAudioCapture` class
  - `ui/src/lib/stores/guitar.svelte.ts` — store-level device enumeration via `navigator.mediaDevices`
  - Sample rate: 48000 Hz, buffer size: 1024 frames

## Authentication & Identity

**Auth Provider:**
- None — no user authentication system present in the codebase.

## Plugin Hosting (DAW Integration)

**VST3 / CLAP plugin:**
- `nih_plug` (git fork `contrapunk-audio/nih-plug`, pinned commit `28b149ec`) — plugin framework generating VST3 and CLAP binaries
- `nih_plug_webview` (git fork `contrapunk-audio/nih-plug-webview`) — embeds SvelteKit UI inside the plugin via native webview
- Communication: `window.plugin.send()` / `listen()` injected by webview host, detected by `PluginAdapter` at `ui/src/lib/adapter/plugin.ts`
- Build: `cargo xtask bundle contrapunk_plugin` via `xtask/` crate

## Networking (Custom TCP Server)

**Collaboration/Cloud layer (in-progress):**
- Custom TCP server in `src/server/` — listens on configurable port, accepts clients up to `max_clients` limit, one thread per client
- Wire protocol: length-prefixed binary framing (`[u16 BE length][u8 type][payload]`) in `src/server/protocol.rs`
- Message types: MidiData (0x01), Configure (0x02), Ack (0x03), Disconnect (0x04), Heartbeat (0x05)
- Session handling: `src/server/session.rs`
- Activated via `--server` CLI flag (Cargo binary `contrapunk`, native only)

## CI/CD & Deployment

**Hosting:**
- Fly.io — web app hosting (`fly.toml`)
  - App: `contrapunk`
  - Primary region: `bom` (Mumbai)
  - VM: 256 MB RAM, shared CPU, single core
  - Auto-stop/start machines enabled; minimum 0 machines running
  - Force HTTPS; internal port 8080

**Build pipeline:**
- No CI service configured (no `.github/workflows/` or similar found)
- Deploy Dockerfile: `deploy/Dockerfile` — multi-stage: Rust wasm-pack build (rust:1.88) → Node vite build (node:22) → nginx:alpine serve
- Simpler UI-only Dockerfile: `ui/Dockerfile` — copies pre-built `ui/build/` directly into nginx

**nginx configuration:**
- `deploy/nginx.conf` / `ui/nginx.conf` — serves static SPA with SPA fallback routing, port 8080

## Monitoring & Observability

**Error Tracking:**
- None — no Sentry or similar service integrated.

**Logs:**
- Rust: `eprintln!()` to stderr (server accept loop, audio errors)
- Frontend: `console.warn()` for WASM stub detection
- No structured logging framework in place

## Webhooks & Callbacks

**Incoming:**
- None

**Outgoing:**
- None (PostHog analytics is the only outbound call, from browser to `us.i.posthog.com`)

## Environment Configuration

**Required env vars for full functionality:**
- `PUBLIC_POSTHOG_KEY` — PostHog project API key; analytics disabled if absent

**No env vars required for core operation** — all audio, MIDI, harmony, and WASM functionality runs without any external service.

**Secrets location:**
- No `.env` files committed to the repository
- PostHog key expected to be injected at Fly.io deploy time as an environment variable

---

*Integration audit: 2026-04-15*
