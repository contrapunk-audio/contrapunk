# Architecture

**Analysis Date:** 2026-02-05

## Pattern Overview

**Overall:** Multi-target layered architecture with platform-specific entry points

**Key Characteristics:**
- Clean separation between harmony logic, MIDI I/O, and UI
- Conditional compilation for native vs. WASM targets
- Message-passing for MIDI routing (channels, threads)
- Stateful harmony engine with modal transformation
- Frame-based processing in GUI, event-driven in CLI

## Layers

**Entry Points:**
- Purpose: Platform-specific initialization and runtime setup
- Location: `src/main.rs`, `src/lib.rs`
- Contains: CLI arg parsing, GUI/CLI mode selection, WASM bootstrap
- Depends on: app, router, server modules
- Used by: Binary executables, WASM loader

**Application Layer:**
- Purpose: GUI application state and user interaction
- Location: `src/app.rs`, `src/ui.rs`
- Contains: eframe/egui GUI, app state, preset management, theme
- Depends on: harmony, router, MIDI, generator, humanize
- Used by: main.rs (native), lib.rs (WASM)

**Router Layer:**
- Purpose: MIDI message routing and harmony pipeline orchestration
- Location: `src/router.rs`
- Contains: Message forwarding, note-on/off handling, thread spawning
- Depends on: harmony engine, MIDI I/O, humanizer
- Used by: app (GUI mode), main (CLI mode)

**Harmony Engine:**
- Purpose: Core musical transformation logic
- Location: `src/harmony/`
- Contains: Scale-aware transposition, mode algorithms, voice leading, modal interchange
- Depends on: wmidi (note types), internal state
- Used by: router, app (WASM direct path)

**MIDI I/O:**
- Purpose: Platform-specific MIDI device communication
- Location: `src/midi/`
- Contains: Port enumeration, input callbacks, output routing
- Depends on: midir (native), web-sys (WASM)
- Used by: router, app

**Generator & Humanizer:**
- Purpose: Note generation and timing variation
- Location: `src/generator/`, `src/humanize/`
- Contains: Arpeggiators, delay queues, beat clock, metronome
- Depends on: harmony engine, MIDI types
- Used by: router, app

**Server (Native-only):**
- Purpose: Network-based MIDI processing (client/server mode)
- Location: `src/server/`
- Contains: TCP protocol, session management, remote harmonization
- Depends on: harmony engine, MIDI I/O
- Used by: main.rs (--server, --client flags)

## Data Flow

**Native Desktop (GUI mode):**

1. User configures input/output ports in GUI (`app.rs`)
2. GUI spawns router thread (`router::spawn_gui_router`)
3. MIDI input callback pushes raw bytes to channel (`midi/input.rs`)
4. Router thread drains channel, parses MIDI messages (`router.rs`)
5. Note-on/off → Harmony engine generates harmony notes (`harmony/engine.rs`)
6. Humanizer applies timing/velocity variation (`humanize/`)
7. Notes sent to output ports via OutputRouter (`midi/output.rs`)
8. Router updates shared state (Arc<Mutex<GUIRouterState>>)
9. GUI polls shared state each frame to display active notes (`app.rs::update`)

**WASM Browser:**

1. `lib.rs` initializes Web MIDI access asynchronously
2. Input callback pushes MIDI bytes to Rc<RefCell<Vec<Vec<u8>>>> queue
3. Each GUI frame (`app.rs::update`), drain queue and process messages
4. Direct harmony engine invocation (no separate thread)
5. Humanized notes pushed to delay queue with timestamps
6. Delay queue drains ready notes each frame
7. Web MIDI outputs receive processed messages

**CLI mode:**

1. User selects ports interactively via stdin prompts (`main.rs`)
2. Router loop receives MIDI from channel (`router::run_router`)
3. Messages processed through harmony engine
4. Output sent directly to OutputRouter (humanization disabled by default)
5. Enter key signals stop via separate channel

**Server/Client mode:**

1. Server listens on TCP port, accepts client connections (`server/session.rs`)
2. Client sends Configure message with harmony settings (`server/protocol.rs`)
3. Client streams MIDI input to server
4. Server processes through local harmony engine
5. Server sends harmonized MIDI back to client
6. Client routes to local output ports

**State Management:**
- GUI: Arc<Mutex<GUIRouterState>> for thread communication
- WASM: Direct engine mutation in update() loop (single-threaded)
- CLI: Owned HarmonyEngine, no shared state

## Key Abstractions

**HarmonyEngine:**
- Purpose: Stateful musical transformation with voice tracking
- Examples: `src/harmony/engine.rs`
- Pattern: Builder-style setters (key, mode, octave_mode), stateful note tracking (note_on/note_off pairs)

**OutputRouter:**
- Purpose: Manage multiple MIDI output connections, route notes to ports
- Examples: `src/midi/output.rs`
- Pattern: Vec of MidiOutputConnection, indexed send

**GUIRouterState (native GUI):**
- Purpose: Thread-safe communication between router and GUI
- Examples: `src/router.rs:37`
- Pattern: Arc<Mutex<T>> with HashSet for active notes, Option fields for config updates

**NoteGenerator:**
- Purpose: Virtual MIDI input for arpeggios, chords, scales
- Examples: `src/generator/engine.rs`
- Pattern: Beat-synced event emission, mode-specific algorithms

**Humanizer:**
- Purpose: Add musical imperfection to harmony notes
- Examples: `src/humanize/engine.rs`
- Pattern: BeatClock for timing, DelayQueue for scheduled notes, per-note jitter/velocity variation

**DelayQueue:**
- Purpose: Schedule MIDI events for future delivery
- Examples: `src/humanize/scheduler.rs`
- Pattern: BinaryHeap of (timestamp, note), drain_ready() for time-based popping

**Scale:**
- Purpose: Modal note transposition with chromatic fallback
- Examples: `src/harmony/scale.rs`
- Pattern: Interval-based transformation, degree calculation, modal interchange borrowing

## Entry Points

**Native Desktop (main.rs):**
- Location: `src/main.rs`
- Triggers: Direct binary execution
- Responsibilities: CLI arg parsing, mode selection (GUI/CLI/server/client), device enumeration

**WASM (lib.rs):**
- Location: `src/lib.rs`
- Triggers: Trunk-generated JS bootstrap calls `#[wasm_bindgen(start)]`
- Responsibilities: Panic hook setup, canvas acquisition, eframe WebRunner launch

**Server:**
- Location: `src/server/mod.rs`
- Triggers: `--server` flag in main.rs
- Responsibilities: TCP listener, session spawning, heartbeat loop

**Client:**
- Location: `src/main.rs::run_client`
- Triggers: `--client <addr>` flag
- Responsibilities: TCP connection, config sync, bidirectional MIDI streaming

## Error Handling

**Strategy:** Result<T> with anyhow::Error for propagation, user-facing strings in GUI

**Patterns:**
- MIDI errors: Log to stderr (CLI) or set app.state.last_error (GUI)
- Thread panics: Isolated (router thread failure doesn't crash GUI)
- WASM: console_error_panic_hook for browser console visibility
- Server/client: Timeout-based error recovery, graceful disconnect

## Cross-Cutting Concerns

**Logging:** `eprintln!` for debug output (CLI), console.log via web-sys (WASM), no structured logging framework

**Validation:** Port index bounds checking, MIDI message parsing via wmidi::MidiMessage::try_from

**Authentication:** None (local MIDI devices, optional network server has no auth)

---

*Architecture analysis: 2026-02-05*
