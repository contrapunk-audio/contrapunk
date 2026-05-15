# Architecture

> **⚠️ STALE — pre-crate-split (2026-04-15)**
>
> File-level citations below (e.g. `src/audio/guitar_input.rs`,
> `src/harmony/engine.rs`, `src/audio_out/sine_synth.rs`) point to paths that
> have been moved. The shared library is now seven workspace crates under
> `crates/contrapunk-{audio,harmony,midi,transport,chord,companion,preset}/`.
>
> The conceptual architecture (multi-target library + pluggable platform
> adapters, lock-free SPSC for MIDI↔audio, etc.) is still accurate. Specific
> "look here for X" line refs are NOT — verify via grep before relying on them.
>
> The WASM guitar path also changed: `ScriptProcessorNode` was replaced by
> AudioWorklet as primary (ScriptProcessor remains as fallback).
>
> **To regenerate this doc:** `/gsd-map-codebase` (project skill).

**Analysis Date:** 2026-04-15 _(stamped STALE 2026-05-15 — see banner above)_

## Pattern Overview

**Overall:** Multi-target library + pluggable platform adapters

**Key Characteristics:**
- Core Rust library (`contrapunk` crate) compiles to native binary, WASM, Tauri backend, and nih-plug plugin — same harmony/DSP code on all targets
- Platform adapters in TypeScript (`ui/src/lib/adapter/`) abstract all backend calls behind a single `ContrapunkAdapter` interface; Svelte components never call Tauri or WASM directly
- Two parallel real-time paths: MIDI routing thread (harmony engine) and audio output thread (cpal callback) communicate exclusively through a lock-free SPSC ringbuffer (`ringbuf::HeapRb`)
- Guitar audio pipeline is feature-symmetrical across Tauri (cpal via `GuitarBridge`) and WASM (Web Audio `ScriptProcessorNode` + `WasmGuitarInput`)

## Layers

**Core Library (`src/`):**
- Purpose: All platform-agnostic logic — harmony engine, DSP pipeline, humanizer, MIDI routing utilities
- Location: `src/`
- Contains: `harmony/`, `audio/`, `audio_out/`, `humanize/`, `midi/`, `generator/`, `preset/`, `chord.rs`, `router.rs`
- Depends on: `wmidi`, `pitch-detection`, `ringbuf`, `serde`
- Used by: `src-tauri/`, `wasm/`, `plugin/`, CLI binary

**Tauri Backend (`src-tauri/`):**
- Purpose: Desktop application shell — IPC command handlers, managed `AppState`, cpal audio I/O, guitar audio bridge
- Location: `src-tauri/src/`
- Key files: `main.rs` (entry, `tauri::Builder`), `state.rs` (`AppState`), `commands/` (8 command modules), `guitar_bridge.rs`
- Depends on: core library, `tauri`, `cpal`, `midir`
- Used by: Tauri desktop app only

**WASM Bridge (`wasm/`):**
- Purpose: wasm-bindgen wrapper that exposes `Engine` and `WasmGuitarInput` to JavaScript
- Location: `wasm/src/lib.rs`
- Key exports: `Engine` (wraps `HarmonyEngine` + `Humanizer` + `Metronome`), `WasmGuitarInput` (wraps `GuitarInput` DSP)
- Depends on: core library, `wasm-bindgen`, `web-sys`
- Built to: `ui/src/lib/wasm-pkg/` (via `wasm-pack`)

**Frontend (`ui/`):**
- Purpose: SvelteKit app — UI components, Svelte 5 rune stores, platform adapter layer
- Location: `ui/src/`
- Sub-structure: `lib/adapter/` (platform detection + implementations), `lib/stores/` (reactive state), `lib/components/` (Svelte components), `lib/audio/` (JS guitar capture), `routes/` (SvelteKit pages)
- Depends on: WASM pkg or Tauri IPC depending on runtime

**Plugin (`plugin/`):**
- Purpose: nih-plug VST3/CLAP plugin wrapping the core library with a webview GUI
- Location: `plugin/src/lib.rs`, `plugin/src/editor.rs`
- Depends on: core library, `nih_plug`, `nih_plug_webview`

## Data Flow

**Guitar Audio → MIDI (Tauri path):**

1. cpal input callback fires on OS audio thread (`src-tauri/src/guitar_bridge.rs`, `build_input_stream` closure)
2. Callback extracts mono channel, calls `GuitarInput::process_block(&mono)` (`src/audio/guitar_input.rs`)
3. `process_block` runs 9-stage DSP pipeline: onset detection → note state machine → McLeod pitch detection → octave correction → Goertzel harmonic measurement → inharmonicity string ID → note-off gating → bend tracking → MIDI event generation
4. Returns `Vec<MidiEvent>` (NoteOn/NoteOff/PitchBend/CC/ChannelPressure)
5. `GuitarBridge` converts each event to raw MIDI bytes via `MidiEvent::to_midi_bytes()` and sends on `mpsc::Sender<Vec<u8>>`
6. Router thread (in `run_tauri_router`, `src-tauri/src/commands/engine.rs`) receives on the same channel as physical MIDI input

**Guitar Audio → MIDI (WASM/browser path):**

1. `getUserMedia` stream feeds `ScriptProcessorNode` (`ui/src/lib/audio/guitarCapture.ts`, `GuitarAudioCapture`)
2. `onaudioprocess` collects overlap buffer (75% overlap), calls `WasmGuitarInput.process_block(samples)` (`wasm/src/lib.rs`)
3. Returns JSON-serialized `MidiEvent` array parsed in JS
4. `GuitarCaptureCallbacks` fire (`onNoteOn`, `onNoteOff`, etc.) → forwarded to Web MIDI output or WASM `Engine.note_on()`

**MIDI In → Harmonized MIDI Out (Tauri path):**

1. `start_routing` Tauri command (`src-tauri/src/commands/engine.rs`) spawns `run_tauri_router` on a dedicated `std::thread`
2. Router connects to physical MIDI input via `connect_input()` (`src/midi/input.rs`) **or** launches `GuitarBridge` when `input_idx == GUITAR_AUDIO_SENTINEL (999_997)`
3. Both paths deliver `Vec<u8>` MIDI bytes on same `mpsc::Receiver`
4. `process_midi_message()` parses bytes → calls `engine.harmonize_note_on(note)` → `HarmonyEngine` returns `Vec<Note>` (melody + harmony voices)
5. Melody voice sent immediately via `OutputRouter::send_to_port()` (`src/midi/output.rs`)
6. Harmony voices go through `Humanizer::humanize_note_on()` → either sent immediately or pushed to `DelayQueue` for timed release
7. Fan-out: each sent note **also** pushes `MidiEvent::NoteOn` into `MidiProducer` (lock-free SPSC), picked up by audio thread
8. Router emits "note-update" Tauri event at ~30fps to frontend via `app_handle.emit()`

**MIDI Events → Audio Output (Tauri native synth):**

1. `AudioOutEngine::start()` (`src/audio_out/engine.rs`) opens cpal output stream with `MAX_POLYPHONY=32` voice `PolySynth`
2. Returns `MidiProducer` end of `ringbuf::HeapRb<MidiEvent>` (capacity 1024) — stored in `AppState::audio_out_producer`
3. `start_routing` takes the producer out of `AppState` and passes ownership into the router thread via `run_tauri_router`
4. cpal `process_callback` runs on real-time OS audio thread: drains `MidiConsumer`, calls `PolySynth::handle_event()`, then `PolySynth::process_stereo(output)` (`src/audio_out/sine_synth.rs`)
5. No heap allocation or mutex contention in the audio callback; `try_lock()` on `Arc<Mutex<AudioState>>` — returns silence on contention

**MIDI In → Harmonized MIDI Out (WASM/browser path):**

1. Web MIDI API (`navigator.requestMIDIAccess`) or `GuitarAudioCapture` delivers note events to `WasmAdapter` (`ui/src/lib/adapter/wasm.ts`)
2. Adapter calls `engine.note_on(note, velocity)` on WASM `Engine` directly (no thread, single-threaded WASM)
3. WASM `Engine` runs `HarmonyEngine::harmonize_note_on()` + `Humanizer` in-process, returns humanized MIDI bytes
4. Adapter forwards bytes to active `MIDIOutput` connections

**State Management:**
- Tauri: `AppState` (`src-tauri/src/state.rs`) is Tauri-managed state, all fields `Mutex<T>` or `AtomicBool`. Router thread owns a snapshot of engine config copied before spawn; no shared mutable access to `HarmonyEngine` during routing.
- WASM: `Engine` struct in `wasm/src/lib.rs` is a JS-side object; all mutations are synchronous single-threaded calls from the adapter.
- Frontend: Svelte 5 rune stores (`engine.svelte.ts`, `guitar.svelte.ts`, `midi.svelte.ts`, `ui.svelte.ts`) hold reactive copies of backend state. All mutations go through `adapter.*()` first, then update local state optimistically.

## Key Abstractions

**`HarmonyEngine` (`src/harmony/engine.rs`):**
- Purpose: Transform a single input `Note` into `Vec<Note>` (melody + harmonies) based on key, mode, scale, voice leading, interchange, counterpoint settings
- Pattern: Stateful struct, mutable borrow for `harmonize_note_on(note)` / `harmonize_note_off(note)` pair — retains active note map for Note-Off tracking

**`GuitarInput` (`src/audio/guitar_input.rs`):**
- Purpose: Full DSP pipeline from raw `f32` audio samples to `Vec<MidiEvent>`
- Pattern: Stateful struct with ring buffer and note state machine. Call `process_block(&[f32])` each audio callback.
- Key sub-systems: `McLeod` pitch detector (via `pitch-detection` crate), Goertzel harmonic analysis, inharmonicity B-coefficient matching for string ID

**`ContrapunkAdapter` interface (`ui/src/lib/adapter/types.ts`):**
- Purpose: Uniform API surface for all platform backends
- Implementations: `TauriAdapter` (Tauri IPC `invoke`/`listen`), `WasmAdapter` (direct WASM calls), `PluginAdapter` (`window.plugin.send/listen`)
- Selection: `ui/src/lib/adapter/index.ts` detects `__TAURI_INTERNALS__`, `window.plugin.send`, or falls back to WASM

**`MidiProducer`/`MidiConsumer` (`src/audio_out/midi_queue.rs`):**
- Purpose: Lock-free SPSC bridge between harmony router thread (producer) and cpal audio callback (consumer)
- Pattern: `ringbuf::HeapRb<MidiEvent>` split at construction; producer held by router thread, consumer moved into audio callback closure. Push returns `Err(QueueFull)` on overflow (fire-and-forget).

**`OutputRouter` (`src/midi/output.rs`):**
- Purpose: Multi-port MIDI output — routes each voice to its designated physical MIDI port
- Pattern: `send_to_port(port_index, bytes)` for voice-mapped output; `send_to_first()` for passthrough messages

## Entry Points

**Tauri Desktop (`src-tauri/src/main.rs`):**
- Location: `src-tauri/src/main.rs`, `fn main()`
- Triggers: OS launches Tauri process
- Responsibilities: Calls `tauri::Builder::default().manage(AppState::default()).invoke_handler(...)`.  Registers 23 IPC commands across 5 modules. Loads SvelteKit app from `http://localhost:5173` (dev) or `../ui/build` (release).

**WASM Module Initialization (`wasm/src/lib.rs`):**
- Location: `#[wasm_bindgen(start)] pub fn init_panic_hook()`
- Triggers: `wasm-pack` generated JS calls the `start` function when the `.wasm` binary is instantiated
- Responsibilities: Installs `console_error_panic_hook`. Thereafter the JS adapter calls `new Engine()` to instantiate the harmony engine.

**Frontend Bootstrap (`ui/src/routes/+layout.ts`, `+layout.svelte`, `+page.svelte`):**
- Location: `ui/src/routes/+layout.ts` — disables SSR (`ssr = false`), conditionally loads PostHog analytics
- `+layout.svelte` — renders background particles layer, applies motion preference from `ui` store
- `+page.svelte` — calls `adapter.init()` on mount, syncs engine state and MIDI devices, sets up keyboard shortcuts for virtual piano input

**CLI Binary (`src/main.rs`):**
- Location: `src/main.rs`, `fn main()`
- Triggers: Direct binary execution
- Responsibilities: Parses `--server` / `--client` flags via `clap`. Routes to `server::run_server()` or TCP client loop. No GUI; prints instructions to use Tauri/SvelteKit instead.

## Thread Model

**Tauri Desktop (3 thread categories):**

| Thread | Owned by | Real-time? | Notes |
|--------|----------|-----------|-------|
| Main (Tauri event loop) | OS | No | Handles IPC commands via `#[tauri::command]` handlers; holds `AppState` |
| Router thread | `std::thread::spawn` in `start_routing` | No | Runs `run_tauri_router`; processes MIDI I/O, humanization, calls `HarmonyEngine`; stopped via `Arc<AtomicBool>` stop signal |
| cpal audio callback | OS audio subsystem (CoreAudio/WASAPI) | **Yes** | Runs `process_callback`; no allocation, no blocking; only `try_lock` on `Arc<Mutex<AudioState>>`; drains `MidiConsumer` ringbuffer |
| Guitar audio callback | cpal input stream (spawned in `GuitarBridge::new`) | **Yes** | Runs `GuitarInput::process_block()`; results sent via `mpsc::Sender` to router thread |

**IPC across thread boundary:**
- Router → Audio thread: `ringbuf::HeapProd<MidiEvent>` (lock-free, owned by router thread after `start_routing` takes it from `AppState`)
- Guitar audio → Router: `std::sync::mpsc::channel::<Vec<u8>>()` (same channel type as physical MIDI input)
- Router → Frontend: Tauri `app_handle.emit("note-update", payload)` at ~30fps

## IPC Boundaries

**Frontend ↔ Tauri Rust (Tauri IPC):**
- Mechanism: `invoke('command_name', args)` from TypeScript; `#[tauri::command]` annotated `fn` in Rust
- Commands registered in `src-tauri/src/main.rs::main()` via `tauri::generate_handler![]`
- Note update events: Rust emits `"note-update"` and `"guitar-signal"` events; TypeScript subscribes via `listen()` in `TauriAdapter`
- Serialization: serde `snake_case` JSON on Rust side; `mapEngineState()`/`mapNoteState()` in `ui/src/lib/adapter/tauri.ts` converts to camelCase

**Frontend ↔ WASM (direct JS ↔ WASM call):**
- Mechanism: `wasm-bindgen` generated JS bindings in `ui/src/lib/wasm-pkg/contrapunk_wasm.js`
- `WasmAdapter.init()` dynamically imports `$lib/wasm-pkg`, calls `new Engine()`
- All method calls are synchronous JS function calls into WASM heap; no serialization overhead for numeric types
- Guitar DSP: `WasmGuitarInput.process_block(Float32Array)` returns JSON string of MIDI events (serialized in Rust)

## Error Handling

**Strategy:** Rust `anyhow::Result` at library boundaries; `String` errors across Tauri IPC (Tauri serializes `Err(String)` as a rejected promise in JS); panics only in programmer-error paths.

**Patterns:**
- Tauri commands return `Result<T, String>` — adapter wraps in try/catch, surfaces to Svelte store as `initError` string state
- Audio callback: never panics; `try_lock()` failure returns silence; queue-full returns `Err(QueueFull)` silently dropped with `let _ =`
- WASM: `console_error_panic_hook` installed at init; JS adapter throws `Error("Failed to initialize WASM: ...")`

## Cross-Cutting Concerns

**Logging:** `eprintln!("[module]")` throughout Rust (no structured logging framework). WASM uses `console_log!` macro wrapping `web_sys::console::log_1`.

**Validation:** Input validation happens at adapter layer (TypeScript type narrowing) and at Rust enum parse boundaries (`parse_key()`, `parse_mode()` in `wasm/src/lib.rs`).

**Authentication:** None. Local desktop app + browser app with no user accounts.

**Feature flags:** `cfg(not(target_arch = "wasm32"))` gates native-only modules (`router`, `server`, `audio_out::engine`). `[features] web-midi` in `Cargo.toml` gates `wasm-bindgen` / `web-sys` dependencies.

---

*Architecture analysis: 2026-04-15*
