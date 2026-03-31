# Architecture

**Analysis Date:** 2026-03-31

## Pattern Overview

**Overall:** Multi-platform library/core with platform-specific frontend shells

**Key Characteristics:**
- A single Rust library crate (`src/`) contains all harmony logic, audio processing, and MIDI primitives, usable on all platforms
- Three deployment targets share the same core: Tauri desktop app, browser WASM app, and TCP server/client CLI
- The Svelte frontend (`ui/`) is shared across both Tauri and browser targets via a platform-adapter pattern
- Conditional compilation (`#[cfg(not(target_arch = "wasm32"))]`) keeps native-only code (midir, networking) out of the WASM build
- An ML subsystem (`ml/`) with Python + a separate SvelteKit dashboard is a standalone side-project for guitar classifier work

---

## Layers

**Core Library (`src/`):**
- Purpose: All harmony logic, MIDI processing, audio analysis, humanization, and preset management
- Location: `src/`
- Contains: `HarmonyEngine`, scale definitions, voice leading, audio pipeline, MIDI I/O primitives, humanizer, note generator, chord detection, server protocol
- Depends on: `wmidi`, `cpal`, `midir` (native only), `pitch-detection`, `rand`, `serde`
- Used by: `src-tauri/`, `wasm/`, CLI entry point `src/main.rs`

**WASM Bridge (`wasm/`):**
- Purpose: Thin wasm-bindgen wrapper that exposes the core library to JavaScript
- Location: `wasm/src/lib.rs`
- Contains: `Engine` struct (wraps `HarmonyEngine`), string-to-enum parsers, JS-friendly API surface
- Depends on: core library at `src/`, `wasm-bindgen`, `serde-wasm-bindgen`
- Used by: `ui/src/lib/wasm-pkg/` (the compiled `.wasm` + JS glue files checked into source)

**Tauri Backend (`src-tauri/`):**
- Purpose: Desktop app shell — registers commands, manages shared state, spawns router thread
- Location: `src-tauri/src/`
- Contains: `AppState` (Mutex-wrapped engine + config), command modules (`harmony`, `midi`, `engine`, `presets`)
- Depends on: core library at `src/`, `tauri` v2, `midir`
- Used by: Tauri runtime; communicates with `ui/` via Tauri IPC invoke/event system

**Svelte UI (`ui/`):**
- Purpose: Single shared frontend for both Tauri desktop and browser WASM deployment
- Location: `ui/src/`
- Contains: SvelteKit single-page app, platform adapter, Svelte 5 rune stores, components
- Depends on: platform adapter (`ui/src/lib/adapter/`), Tailwind CSS v4, `@tauri-apps/api` v2
- Used by: Tauri (served via devUrl / `ui/build`), browser (deployed as static site)

**Platform Adapter (`ui/src/lib/adapter/`):**
- Purpose: Abstracts all backend communication so components never call Tauri IPC or WASM directly
- Location: `ui/src/lib/adapter/`
- Contains: `ContrapunkAdapter` interface (`types.ts`), `TauriAdapter` (`tauri.ts`), `WasmAdapter` (`wasm.ts`), factory `index.ts`
- Depends on: `@tauri-apps/api` (Tauri path), `$lib/wasm-pkg` (browser path)
- Used by: Svelte stores (`engine.svelte.ts`, `midi.svelte.ts`) and components

**TCP Server (`src/server/`):**
- Purpose: Network mode — accepts remote MIDI client connections, runs harmony engine server-side
- Location: `src/server/`
- Contains: `run_server()` accept loop, `session::handle_client()`, `protocol::Message` enum with length-prefixed wire framing
- Depends on: core library (harmony engine), stdlib TCP
- Used by: `src/main.rs` when `--server` flag is passed

**ML Subsystem (`ml/`):**
- Purpose: Guitar string+fret classification data pipeline (standalone, not integrated with main app)
- Location: `ml/`
- Contains: Python data loader (`loader.py`), raw analysis scripts (`processing/01_raw_analysis/analyze.py`), SvelteKit dashboard (`ml/app/`)
- Depends on: Python packages (`requirements.txt`), separate SvelteKit app (not shared with main `ui/`)

---

## Data Flow

**MIDI Harmony (Tauri Desktop):**

1. User selects MIDI input/output devices and presses "Start" in the Svelte UI
2. `MidiDevices.svelte` calls `adapter.startRouting(inputIdx, outputIndices)` → `TauriAdapter` → `invoke('start_routing', ...)`
3. `src-tauri/src/commands/engine.rs::start_routing` spawns a background thread running the router loop
4. Router thread reads raw MIDI bytes from `midir` input callback via `mpsc::channel`
5. Each `NoteOn` is passed to `HarmonyEngine::harmonize_note_on(note)` → returns `Vec<Note>` (melody + harmonies)
6. Harmony notes are optionally passed through `Humanizer` → `DelayQueue` for timing offsets
7. Notes are sent to `OutputRouter` which routes each voice to its designated MIDI output port
8. Router thread emits `"note-update"` Tauri events at ~30fps with `NoteUpdatePayload` (input/harmony/borrowed notes, chord name)
9. `WasmAdapter` / `TauriAdapter.onNoteUpdate()` callbacks update `engine` store in the UI; components react

**MIDI Harmony (Browser WASM):**

1. Same Svelte UI; `WasmAdapter` is active because `window.__TAURI__` is absent
2. `WasmAdapter.init()` dynamically imports `$lib/wasm-pkg` and constructs a WASM `Engine` instance
3. Web MIDI API (`navigator.requestMIDIAccess()`) enumerates devices; adapter routes `MIDIMessageEvent` directly to `engine.injectNoteOn/Off`
4. WASM `Engine` wraps `HarmonyEngine` in Rust compiled to WASM; harmony runs in the browser process
5. Note outputs are sent to `MIDIOutput.send()` via the Web MIDI API

**Audio Pipeline (Guitar-to-MIDI):**

1. `cpal` opens audio input; raw PCM samples fill `DualBufferAnalyzer`
2. `PluckDetector` (HFC + spectral flux) fires `NoteEvent::Attack` on onset
3. `PitchDetector` (autocorrelation / BACF) identifies pitch; `freq_to_midi()` converts to MIDI note
4. `GuitarPitchMatcher` cross-references calibration profile to identify string+fret
5. `OnsetGrouper` batches simultaneous events into `ChordEvent`
6. Output flows into `HarmonyEngine` for real-time harmonization

**State Management:**

- Tauri: `AppState` struct with `Mutex<HarmonyEngine>` guards all engine state; command handlers lock before mutating; router thread holds its own copy after `clone()` at start-routing time
- Browser: Svelte 5 rune stores (`$state`) in `engine.svelte.ts`, `midi.svelte.ts`, `ui.svelte.ts` are the reactive state layer; mutations call adapter then optimistically update local state
- Persistence: MIDI device selections are persisted to `localStorage` by `midi.svelte.ts` (keyed `"contrapunk-midi"`); detune value also persisted

---

## Key Abstractions

**`HarmonyEngine` (`src/harmony/engine.rs`):**
- Purpose: Transforms incoming MIDI notes into multi-voice harmonies using the configured mode and scale
- Examples: `src/harmony/engine.rs`, `wasm/src/lib.rs` (wrapped as `Engine`), `src-tauri/src/state.rs` (held in `AppState`)
- Pattern: Stateful engine with `harmonize_note_on(note) -> Vec<Note>` / `harmonize_note_off(note) -> Vec<Note>`; tracks active notes for correct note-off generation

**`ContrapunkAdapter` (`ui/src/lib/adapter/types.ts`):**
- Purpose: Interface isolating all Svelte components from platform specifics (Tauri vs WASM)
- Examples: `ui/src/lib/adapter/tauri.ts`, `ui/src/lib/adapter/wasm.ts`
- Pattern: Factory pattern in `ui/src/lib/adapter/index.ts` detects `window.__TAURI__` and exports a singleton adapter

**`Scale` / `HarmonyMode` / `ScaleMode` (`src/harmony/`):**
- Purpose: Type-safe representations of musical concepts; `Scale::transpose_diatonic()` drives all interval generation
- Examples: `src/harmony/config.rs` (enums), `src/harmony/scale.rs` (operations), `src/harmony/modes.rs` (per-mode logic)
- Pattern: Enum-based dispatch; `HarmonyMode::all()` and `ScaleMode::all()` return ordered slices for UI enumeration

**`StylePreset` (`src/preset/mod.rs`):**
- Purpose: Serializable bundle of all engine settings that can be loaded/saved as a named style
- Examples: `src/preset/builtins.rs` (built-in presets), `src/preset/storage.rs` (custom persistence)
- Pattern: `PresetManager` holds builtins + custom presets; applies to `HarmonyEngine` + `HumanizeConfig`

**`OutputRouter` (`src/midi/output.rs`):**
- Purpose: Routes voice-indexed MIDI bytes to the correct physical MIDI output port
- Examples: `src/router.rs`, `src-tauri/src/commands/engine.rs`
- Pattern: Index-based port dispatch; `send_to_port(voice_index, bytes)` / `send_to_first(bytes)`

---

## Entry Points

**Tauri Desktop App:**
- Location: `src-tauri/src/main.rs`
- Triggers: `cargo tauri dev` (dev) or built binary
- Responsibilities: Register `AppState`, register all `#[tauri::command]` handlers via `invoke_handler!`, launch Tauri runtime with the SvelteKit frontend served from `ui/build`

**Browser WASM App:**
- Location: `ui/src/routes/+page.svelte` (SvelteKit root page)
- Triggers: `cd ui && npm run dev` or `npm run build` (static site)
- Responsibilities: Initialize adapter via `adapter.init()`, sync engine + MIDI state, render all UI panels

**CLI / Server Binary:**
- Location: `src/main.rs`
- Triggers: `cargo run -- --server` or `cargo run -- --client <host:port>`
- Responsibilities: Parse `clap` args; dispatch to `contrapunk::server::run_server()` or inline `run_client()`; informs user about GUI alternatives when run with no args

**WASM Build:**
- Location: `wasm/src/lib.rs` (compiled via `wasm-pack`)
- Triggers: `cd ui && npm run build:wasm` (runs `ui/scripts/build-wasm.sh`)
- Responsibilities: Expose `Engine` struct and free functions to JavaScript; initializes panic hook

---

## Error Handling

**Strategy:** `anyhow::Result<T>` for all fallible Rust functions; command handlers return `Result<T, String>` to Tauri (errors serialized as strings); WASM returns `Result<T, JsValue>`

**Patterns:**
- Native router errors are logged with `eprintln!` and break the routing loop
- Tauri commands propagate errors as `Err(String)` to the frontend via IPC
- WASM errors are thrown as JS exceptions via `JsValue::from_str(...)`
- Svelte stores catch adapter errors and expose them on `error` state fields

---

## Cross-Cutting Concerns

**Logging:** `println!` / `eprintln!` to stdout/stderr in CLI and Tauri backend; browser WASM uses `console_error_panic_hook` for panic backtraces in the browser console; no structured logging framework

**Validation:** Musical enum validation happens in `parse_key()`, `parse_mode()`, `parse_scale_mode()` string-to-enum helpers in `wasm/src/lib.rs` and `src/main.rs`; out-of-range MIDI notes are clamped by `wmidi`

**Authentication:** Not applicable — single-user local desktop/browser app; no network auth in server mode

---

*Architecture analysis: 2026-03-31*
