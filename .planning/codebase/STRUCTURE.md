# Codebase Structure

**Analysis Date:** 2026-03-31

## Directory Layout

```
contrapunk/
├── src/                        # Core Rust library (all platforms)
│   ├── lib.rs                  # Crate root; declares all modules
│   ├── main.rs                 # CLI binary entry point (server/client modes)
│   ├── chord.rs                # Chord detection (standalone module)
│   ├── router.rs               # Native MIDI routing loop (non-WASM)
│   ├── harmony/                # Harmony engine and algorithms
│   │   ├── mod.rs              # Public re-exports + module docs
│   │   ├── config.rs           # Key, HarmonyMode, ScaleMode, OctaveMode enums
│   │   ├── engine.rs           # HarmonyEngine struct (core harmonization logic)
│   │   ├── modes.rs            # Per-mode algorithm implementations
│   │   ├── scale.rs            # Scale struct + diatonic transposition
│   │   ├── stateful.rs         # ContraryMotionState, CounterpointState
│   │   └── voice_leading/      # Voice leading post-processor
│   │       ├── mod.rs
│   │       ├── rules.rs        # Voice leading rules
│   │       ├── styles.rs       # VoiceLeadingStyle enum
│   │       ├── suspension.rs   # Suspension handling
│   │       └── voicer.rs       # Main voice revoicing logic
│   ├── audio/                  # Audio capture and pitch detection
│   │   ├── mod.rs              # Public exports
│   │   ├── buffer.rs           # DualBufferAnalyzer, RingBuffer, OverlapManager
│   │   ├── config.rs           # MicConfig (user-adjustable thresholds)
│   │   ├── detectors.rs        # BacfDetector, AmdfDetector, GoertzelBank
│   │   ├── guitar.rs           # Guitar calibration, string matching, chord grouping
│   │   ├── onset.rs            # PluckDetector, RunningAutocorrelation
│   │   ├── pitch.rs            # NoteTracker, DetectedPitch, freq_to_midi
│   │   ├── profiles.rs         # Built-in MicProfile presets
│   │   ├── single_cycle.rs     # Low-latency single-cycle pitch detector
│   │   └── test_signals.rs     # Test signal generators
│   ├── generator/              # Beat-driven note generator
│   │   ├── mod.rs
│   │   ├── config.rs           # GeneratorMode, GeneratorConfig
│   │   └── engine.rs           # NoteGenerator tick-based event emitter
│   ├── humanize/               # Timing humanization
│   │   ├── mod.rs              # Public exports + module docs
│   │   ├── beat_clock.rs       # BeatClock (BPM, beat position tracking)
│   │   ├── config.rs           # HumanizeConfig, HumanizedNote
│   │   ├── engine.rs           # Humanizer (applies jitter, velocity, swing)
│   │   ├── metronome.rs        # Optional audible click track
│   │   └── scheduler.rs        # DelayQueue (timed note dispatch)
│   ├── midi/                   # MIDI I/O primitives
│   │   ├── mod.rs              # Conditional re-exports
│   │   ├── input.rs            # connect_input (midir, native-only)
│   │   ├── output.rs           # OutputRouter (midir, native-only)
│   │   ├── ports.rs            # list/select port helpers (native-only)
│   │   └── web.rs              # Web MIDI API wrapper (wasm feature)
│   ├── preset/                 # Style presets
│   │   ├── mod.rs              # StylePreset, PresetManager
│   │   ├── builtins.rs         # Hard-coded built-in presets
│   │   └── storage.rs          # Custom preset persistence
│   └── server/                 # TCP server/client mode (native-only)
│       ├── mod.rs              # run_server() accept loop
│       ├── config.rs           # ServerConfig
│       ├── protocol.rs         # Message enum, wire framing (length-prefixed)
│       └── session.rs          # handle_client() per-connection handler
│
├── wasm/                       # WASM bridge crate
│   ├── Cargo.toml              # cdylib + rlib; depends on ../
│   └── src/
│       └── lib.rs              # Engine struct + wasm-bindgen exports
│
├── src-tauri/                  # Tauri v2 desktop backend
│   ├── Cargo.toml
│   ├── tauri.conf.json         # App config (window size, build commands, devUrl)
│   ├── build.rs
│   ├── capabilities/
│   └── src/
│       ├── main.rs             # Tauri builder: manage(AppState) + invoke_handler!
│       ├── state.rs            # AppState (Mutex<HarmonyEngine>, AtomicBool, etc.)
│       └── commands/
│           ├── mod.rs
│           ├── engine.rs       # start_routing, stop_routing, get_note_state
│           ├── harmony.rs      # set_key, set_mode, set_scale_mode, etc.
│           ├── midi.rs         # list_midi_inputs, list_midi_outputs, refresh
│           └── presets.rs      # list_presets, load_preset, save_preset, delete_preset
│
├── ui/                         # Shared SvelteKit frontend (Tauri + browser)
│   ├── package.json            # Build scripts including build:wasm
│   ├── vite.config.ts
│   ├── svelte.config.js
│   ├── scripts/
│   │   └── build-wasm.sh       # Runs wasm-pack and copies output to wasm-pkg
│   ├── static/
│   └── src/
│       ├── app.html            # SvelteKit HTML shell
│       ├── app.css             # Global styles
│       ├── routes/
│       │   ├── +layout.svelte  # Root layout (Particles background)
│       │   ├── +layout.ts      # SSR disabled (prerender/static)
│       │   └── +page.svelte    # Main app page (keyboard input, init, renders panels)
│       └── lib/
│           ├── adapter/        # Platform abstraction layer
│           │   ├── index.ts    # Factory: detects Tauri vs browser, exports singleton
│           │   ├── types.ts    # ContrapunkAdapter interface + all shared types
│           │   ├── tauri.ts    # TauriAdapter: calls invoke() + listens to events
│           │   ├── wasm.ts     # WasmAdapter: calls WASM Engine directly + Web MIDI
│           │   └── wasm-types.d.ts
│           ├── components/     # Svelte UI components
│           │   ├── ActiveNotes.svelte
│           │   ├── BeatIndicator.svelte
│           │   ├── ControlPanel.svelte  # Main harmony settings (key, mode, scale)
│           │   ├── GeneratorPanel.svelte
│           │   ├── GlowEffects.svelte
│           │   ├── HumanizePanel.svelte
│           │   ├── MidiDevices.svelte
│           │   ├── Particles.svelte
│           │   ├── Piano.svelte
│           │   ├── PixelSelect.svelte
│           │   ├── PresetManager.svelte
│           │   └── StatusBar.svelte
│           ├── stores/         # Svelte 5 rune reactive stores
│           │   ├── engine.svelte.ts  # Harmony engine config + note state
│           │   ├── midi.svelte.ts    # MIDI device list + selection + localStorage
│           │   └── ui.svelte.ts      # Platform, animations, panel state
│           ├── theme/
│           │   ├── colors.ts   # Color palette constants
│           │   ├── pixel.css   # Pixel/retro visual style
│           │   └── tokens.css  # CSS custom properties (design tokens)
│           └── wasm-pkg/       # Compiled WASM output (checked in)
│               ├── contrapunk_wasm.js
│               ├── contrapunk_wasm.d.ts
│               ├── contrapunk_wasm_bg.wasm
│               └── contrapunk_wasm_bg.wasm.d.ts
│
├── ml/                         # Guitar ML classifier (standalone)
│   ├── loader.py               # Training data loader
│   ├── requirements.txt
│   ├── CONCEPTS.md
│   ├── REVIEW_LEARNINGS.md
│   ├── processing/
│   │   └── 01_raw_analysis/
│   │       ├── analyze.py      # Raw dataset analysis script
│   │       └── *.png / *.txt   # Analysis output artifacts
│   └── app/                    # Separate SvelteKit ML dashboard
│       └── src/
│           └── routes/
│               ├── +page.svelte
│               ├── raw-data/
│               ├── features/
│               ├── training/
│               ├── validation/
│               ├── augmentation/
│               ├── comparison/
│               ├── ensemble/
│               ├── normalization/
│               ├── onset/
│               └── live/
│
├── examples/                   # Standalone Rust example binaries
│   ├── guitar_calibrate.rs     # Guitar calibration tool
│   ├── guitar_capture.rs       # Training data capture
│   ├── guitar_demo.rs          # Guitar harmonization demo
│   ├── guitar_harmony.rs       # Guitar + harmony integration
│   └── guitar_tuner.rs         # Real-time guitar tuner
│
├── tests/
│   └── audio_pipeline.rs       # Integration tests for audio pipeline
│
├── deploy/                     # Production deployment config
│   ├── Dockerfile
│   ├── fly.toml                # Fly.io deployment
│   ├── nginx.conf
│   └── dist/                   # Built static files for deployment
│
├── assets/
│   └── fonts/                  # Custom font files
│
├── docs/                       # Project documentation
│   └── superpowers/
│       ├── plans/
│       └── specs/
│
├── Cargo.toml                  # Workspace root: members = [".", "src-tauri", "wasm"]
├── Cargo.lock
├── guitar_calibration_profile.json  # Active guitar calibration data
├── guitar_training_data.msgpack     # Captured training samples
└── .planning/                  # GSD planning documents
    ├── codebase/               # Codebase maps (this file's home)
    └── phases/                 # Implementation phase plans
```

---

## Directory Purposes

**`src/`:**
- Purpose: The heart of the application — all cross-platform Rust logic
- Contains: Library crate root (`lib.rs`), CLI binary (`main.rs`), and 8 submodules: `harmony`, `audio`, `generator`, `humanize`, `midi`, `preset`, `server`, plus top-level `chord.rs` and `router.rs`
- Key files: `src/lib.rs` (module declarations), `src/harmony/engine.rs` (core engine, ~1600 lines), `src/audio/guitar.rs` (guitar pipeline, ~1500 lines)

**`wasm/`:**
- Purpose: A thin crate that wraps `src/` for wasm-pack compilation
- Contains: Single `lib.rs` with wasm-bindgen exports; no business logic
- Key files: `wasm/src/lib.rs`, `wasm/Cargo.toml`

**`src-tauri/`:**
- Purpose: Tauri v2 desktop shell; all IPC command handlers live here
- Contains: `AppState`, four command modules, Tauri config
- Key files: `src-tauri/src/main.rs`, `src-tauri/src/state.rs`, `src-tauri/tauri.conf.json`

**`ui/`:**
- Purpose: SvelteKit frontend served by both Tauri and deployed as static site
- Contains: One route (`+page.svelte`), 13 components, 3 stores, platform adapter, theme tokens, pre-built WASM package
- Key files: `ui/src/lib/adapter/index.ts` (adapter factory), `ui/src/lib/stores/engine.svelte.ts` (engine state)

**`ui/src/lib/wasm-pkg/`:**
- Purpose: Compiled wasm-pack output — JavaScript glue and `.wasm` binary
- Generated: Yes, by `npm run build:wasm` (calls `ui/scripts/build-wasm.sh`)
- Committed: Yes (so the browser app works without a local Rust toolchain)

**`ml/`:**
- Purpose: Standalone ML pipeline for guitar string+fret classification; unrelated to the main MIDI app
- Contains: Python scripts, SvelteKit dashboard, venv
- Key files: `ml/loader.py`, `ml/processing/01_raw_analysis/analyze.py`

**`examples/`:**
- Purpose: Standalone Rust example binaries that exercise the audio pipeline; run with `cargo run --example <name>`
- Generated: No
- Committed: Yes

**`deploy/`:**
- Purpose: Docker + Fly.io config for hosting the browser WASM version
- Key files: `deploy/Dockerfile`, `deploy/fly.toml`, `deploy/nginx.conf`

---

## Key File Locations

**Entry Points:**
- `src/main.rs`: CLI binary (server and client modes)
- `src-tauri/src/main.rs`: Tauri desktop app bootstrap
- `ui/src/routes/+page.svelte`: SvelteKit app root (runs in browser and Tauri webview)
- `wasm/src/lib.rs`: WASM module init + exported API

**Configuration:**
- `Cargo.toml`: Workspace root, workspace member list, feature flags (`web-midi`)
- `src-tauri/tauri.conf.json`: Window dimensions, dev URL, frontend dist path
- `ui/package.json`: Build scripts (`build:wasm`, `build`, `dev`)
- `ui/vite.config.ts`: Vite + Tailwind + WASM plugin config

**Core Logic:**
- `src/harmony/engine.rs`: `HarmonyEngine` — the main harmonization algorithm
- `src/harmony/config.rs`: All musical enum definitions (`Key`, `HarmonyMode`, `ScaleMode`, `OctaveMode`)
- `src/harmony/scale.rs`: `Scale::transpose_diatonic()` — interval generation
- `src/harmony/stateful.rs`: Stateful mode implementations (contrary motion, counterpoint)
- `src/harmony/voice_leading/voicer.rs`: Voice leading revoicing logic
- `src/audio/guitar.rs`: Guitar calibration + string/fret matching
- `src/humanize/engine.rs`: `Humanizer` — applies timing jitter and velocity variation
- `src/router.rs`: Native MIDI routing loop (NoteOn → HarmonyEngine → Humanizer → OutputRouter)
- `src-tauri/src/commands/engine.rs`: Tauri command that spawns the router thread
- `ui/src/lib/adapter/types.ts`: `ContrapunkAdapter` interface definition
- `ui/src/lib/adapter/wasm.ts`: `WasmAdapter` — WASM + Web MIDI API implementation
- `ui/src/lib/adapter/tauri.ts`: `TauriAdapter` — Tauri IPC implementation

**Testing:**
- `tests/audio_pipeline.rs`: Integration tests for the audio module
- `src/server/protocol.rs`: Inline unit tests for TCP message round-trips

---

## Naming Conventions

**Files (Rust):**
- `snake_case.rs` for all modules: `engine.rs`, `beat_clock.rs`, `single_cycle.rs`
- `mod.rs` for module roots, e.g., `src/harmony/mod.rs`
- Top-level flat modules (chord, router) live directly in `src/` as single files

**Files (TypeScript/Svelte):**
- `PascalCase.svelte` for components: `ControlPanel.svelte`, `MidiDevices.svelte`
- `camelCase.svelte.ts` for Svelte 5 rune stores: `engine.svelte.ts`, `midi.svelte.ts`
- `camelCase.ts` for plain TypeScript modules: `types.ts`, `index.ts`, `colors.ts`

**Directories:**
- `snake_case` for Rust module directories: `voice_leading/`, `src-tauri/`
- `kebab-case` for SvelteKit route segments: `raw-data/`, `wasm-pkg/`
- `camelCase` for non-route lib directories: `wasm-pkg/` is an exception (follows wasm-pack output)

**Types (Rust):**
- `PascalCase` for structs and enums: `HarmonyEngine`, `NoteTracker`, `ScaleMode`
- Enum variants are `PascalCase`: `HarmonyMode::DiatonicThirds`, `OctaveMode::BassTrebleSplit`

**Types (TypeScript):**
- `PascalCase` for interfaces and type aliases: `ContrapunkAdapter`, `EngineState`, `MidiDevice`
- `camelCase` for functions and class instances: `adapter`, `engine`, `midi`
- Store classes are `PascalCase` internally, exported as `camelCase` singletons: `export const engine = new EngineStore()`

---

## Where to Add New Code

**New Harmony Algorithm:**
- Add variant to `HarmonyMode` enum: `src/harmony/config.rs`
- Implement algorithm in: `src/harmony/modes.rs` (stateless) or `src/harmony/stateful.rs` (stateful)
- Add string parsing in: `wasm/src/lib.rs::parse_mode()` and `src-tauri/src/commands/harmony.rs`
- Add TypeScript type in: `ui/src/lib/stores/engine.svelte.ts::HarmonyModeName`

**New Scale Mode:**
- Add variant to `ScaleMode` enum: `src/harmony/config.rs`
- Add interval definition in: `src/harmony/scale.rs`
- Add string parsing in: `wasm/src/lib.rs::parse_scale_mode()`
- Add TypeScript type in: `ui/src/lib/stores/engine.svelte.ts::ScaleModeName`

**New UI Component:**
- Implementation: `ui/src/lib/components/ComponentName.svelte`
- Import in: `ui/src/routes/+page.svelte` or parent component

**New Svelte Store:**
- Implementation: `ui/src/lib/stores/storeName.svelte.ts` (use Svelte 5 `$state` runes)

**New Adapter Method:**
- Add to interface: `ui/src/lib/adapter/types.ts::ContrapunkAdapter`
- Implement in both: `ui/src/lib/adapter/tauri.ts` and `ui/src/lib/adapter/wasm.ts`
- Add Tauri command if needed: `src-tauri/src/commands/` + register in `src-tauri/src/main.rs`
- Add WASM export if needed: `wasm/src/lib.rs`

**New Preset:**
- Add to: `src/preset/builtins.rs::all()` function

**New Example Binary:**
- Create: `examples/example_name.rs`
- Run with: `cargo run --example example_name`

**New Audio Detector:**
- Implementation: `src/audio/detectors.rs` or a new file in `src/audio/`
- Export from: `src/audio/mod.rs`

---

## Special Directories

**`.planning/`:**
- Purpose: GSD planning system — phases, codebase maps, quick tasks
- Generated: No
- Committed: Yes

**`ui/src/lib/wasm-pkg/`:**
- Purpose: wasm-pack output (WASM binary + JS glue)
- Generated: Yes, by `npm run build:wasm`
- Committed: Yes (allows browser deployment without Rust toolchain; `.gitignore` inside excludes nothing)

**`target/`:**
- Purpose: Rust build artifacts
- Generated: Yes
- Committed: No

**`ml/venv/`:**
- Purpose: Python virtual environment for ML scripts
- Generated: Yes
- Committed: No

**`ui/.svelte-kit/`:**
- Purpose: SvelteKit internal build cache
- Generated: Yes
- Committed: No

**`ui/build/`:**
- Purpose: Static site output from `npm run build`; served by Tauri and Nginx
- Generated: Yes
- Committed: No

---

*Structure analysis: 2026-03-31*
