# Architecture

**Analysis Date:** 2026-02-04

## Pattern Overview

**Overall:** Modular Event-Driven Architecture with Multi-Target Compilation

**Key Characteristics:**
- Domain-driven modules organized by musical and technical concerns
- Conditional compilation for multiple targets (native CLI, native GUI, WASM)
- Event-driven MIDI routing with pluggable harmony processing pipeline
- Stateful and stateless harmony generation modes with voice leading post-processing

## Layers

**MIDI I/O Layer:**
- Purpose: Abstract platform-specific MIDI communication
- Location: `src/midi/`
- Contains: Input/output port management, native (midir) and web (Web MIDI API) implementations
- Depends on: Platform-specific APIs (midir for native, web-sys for WASM)
- Used by: Router, Application UI

**Harmony Engine Layer:**
- Purpose: Core musical transformation logic - converts input notes to harmonized output
- Location: `src/harmony/`
- Contains: Scale definitions, mode algorithms (stateless and stateful), voice leading rules, engine coordination
- Depends on: wmidi for MIDI note representation
- Used by: Router, Generator, Application

**Router/Orchestration Layer:**
- Purpose: Connects MIDI input to harmony engine to MIDI output, manages message flow
- Location: `src/router.rs`
- Contains: Main event loop, note tracking, humanization scheduling, generator integration
- Depends on: MIDI I/O layer, Harmony engine, Humanizer, Generator
- Used by: CLI main (`src/main.rs`), GUI app (`src/app.rs`)

**Presentation Layer:**
- Purpose: User interfaces for controlling harmony parameters
- Location: `src/app.rs`, `src/ui.rs`, `src/piano.rs`, `src/theme/`
- Contains: eframe/egui GUI components, MIDI device selectors, parameter controls, visual feedback
- Depends on: eframe, Application state, Router state
- Used by: Main entry points (native and WASM)

**Supporting Services:**
- Purpose: Cross-cutting features for generation, timing, persistence
- Location: `src/generator/`, `src/humanize/`, `src/preset/`, `src/server/`
- Contains: Note generator, humanization timing, preset storage, TCP server protocol
- Depends on: Core domain types (harmony, MIDI)
- Used by: Router, Application

## Data Flow

**Real-time MIDI Flow (Native):**

1. MIDI hardware → `midi::input::connect_input()` → channel sender
2. Channel receiver in `router.rs` loop receives raw bytes
3. Parse bytes into `wmidi::MidiMessage` (NoteOn/NoteOff/etc)
4. For NoteOn: `HarmonyEngine::harmonize(note, voice_count)` → Vec of harmony notes
5. Apply humanization: `Humanizer::humanize()` → schedule delayed notes
6. `DelayQueue::tick()` releases notes when delay expires
7. Route notes to outputs: `OutputRouter::send_to_port(index, bytes)`
8. MIDI hardware receives harmonized output

**Real-time MIDI Flow (WASM):**

1. Web MIDI API → `midi::web` event handlers → `keyboard_events` queue
2. Application polls queue in UI update loop
3. Convert events to `wmidi::Note`, send to simulated harmony pipeline
4. Output via Web MIDI API (if available) or visual feedback only

**Configuration Flow:**

1. User selects parameters in GUI (`app.rs`) or CLI prompts (`main.rs`)
2. Parameters stored in `AppState` (GUI) or directly passed (CLI)
3. For GUI: Update `GUIRouterState` (Arc<Mutex>) shared with router thread
4. Router thread polls state, applies `engine.set_key()`, `engine.set_mode()`, etc.
5. Subsequent harmony generation uses updated configuration

**State Management:**
- GUI mode: Shared state via `Arc<Mutex<GUIRouterState>>` between UI thread and router thread
- CLI mode: Direct engine ownership in main thread or router thread
- WASM mode: `Rc<RefCell>` for single-threaded async state

## Key Abstractions

**HarmonyEngine:**
- Purpose: Encapsulates all harmony generation logic and state
- Examples: `src/harmony/engine.rs`
- Pattern: Stateful service with mode strategy pattern - delegates to mode-specific functions in `src/harmony/modes.rs` or stateful processors (`ContraryMotionState`, `CounterpointState`)

**OutputRouter:**
- Purpose: Manages multiple MIDI output connections, routes messages to specific ports
- Examples: `src/midi/output.rs`
- Pattern: Resource manager with indexed access - holds vector of `MidiOutputConnection`, provides `send_to_port(index, bytes)`

**VoiceLeadingProcessor:**
- Purpose: Post-processes harmony output for smooth voice transitions following counterpoint rules
- Examples: `src/harmony/engine.rs` (internal struct), `src/harmony/voice_leading/`
- Pattern: Stateful filter with rule-based transformations - tracks previous voicing, applies style-specific rules

**NoteGenerator:**
- Purpose: Generates MIDI notes from patterns (arpeggios, chords, sequences) independently of live input
- Examples: `src/generator/engine.rs`
- Pattern: Iterator-like event stream - advances on tick, emits `GeneratorEvent` (NoteOn/NoteOff)

**Humanizer/DelayQueue:**
- Purpose: Adds timing variation to MIDI output for natural feel
- Examples: `src/humanize/engine.rs`, `src/humanize/scheduler.rs`
- Pattern: Priority queue scheduler - notes scheduled with randomized delays, released by tick() calls

**Preset:**
- Purpose: Serializable snapshot of harmony configuration for save/load
- Examples: `src/preset/mod.rs`, `src/preset/builtins.rs`
- Pattern: Data transfer object with builder - contains key, mode, octave mode, voice settings; stored via eframe persistence

## Entry Points

**Native CLI Entry:**
- Location: `src/main.rs` (when compiled without `gui` feature)
- Triggers: `cargo run` or binary execution
- Responsibilities: Parse CLI args (clap), prompt for MIDI device selection and harmony config, create `HarmonyEngine`, call `router::run_router()` blocking loop

**Native GUI Entry:**
- Location: `src/main.rs` → `run_gui()` → `app::ContrapunkApp` (when compiled with `gui` feature)
- Triggers: `cargo run --features gui` or GUI binary execution
- Responsibilities: Initialize eframe window, create `ContrapunkApp`, enter eframe event loop

**WASM Entry:**
- Location: `src/lib.rs` (WASM-only compilation, automatically included via `#[wasm_bindgen(start)]`)
- Triggers: Trunk build loads WASM module, calls `main()` via wasm-bindgen bootstrap
- Responsibilities: Set panic hook, spawn async eframe WebRunner on canvas element

**Server Mode Entry:**
- Location: `src/main.rs` → `server::run_server()`
- Triggers: CLI flag `--server`
- Responsibilities: Bind TCP listener, spawn thread per client running `session::handle_client()`, forward MIDI messages through harmony engine

**Client Mode Entry:**
- Location: `src/main.rs` → `run_client()`
- Triggers: CLI flag `--client <addr>`
- Responsibilities: Connect to server TCP socket, stream local MIDI input to server, route server responses to local outputs

## Error Handling

**Strategy:** Result-based propagation with anyhow for CLI/server, GUI-specific error display

**Patterns:**
- CLI/Server: Functions return `anyhow::Result<T>`, errors propagate to main with `?`, printed via `eprintln!`
- GUI: MIDI operations return `Result`, errors captured in `AppState.last_error: Option<String>`, displayed in error panel
- WASM: Panic hook redirects to browser console via `console_error_panic_hook::set_once()`
- Router: Errors logged to stderr but loop continues (non-fatal MIDI errors don't crash routing)

## Cross-Cutting Concerns

**Logging:** stderr output via `eprintln!` macro (CLI/server), browser console via panic hook (WASM), no structured logging framework

**Validation:**
- MIDI port indices validated on selection (check against available ports list)
- Harmony parameters validated via enum types (Key, HarmonyMode, OctaveMode)
- Voice count validated in `HarmonyEngine` (min 1, max typically 8)
- Note ranges validated when parsing MIDI messages (0-127)

**Authentication:** Not applicable - local desktop/WASM application, server has no auth (intended for localhost)

---

*Architecture analysis: 2026-02-04*
