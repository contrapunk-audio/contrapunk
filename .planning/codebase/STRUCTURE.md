# Codebase Structure

> **⚠️ STALE — pre-crate-split (2026-04-15)**
>
> The `src/audio/`, `src/audio_out/`, `src/harmony/`, `src/humanize/`, `src/midi/`,
> and `src/generator/` paths cited below **no longer exist**. As of v1.2.x the
> shared library was split into 7 workspace crates under `crates/`:
>
> - `crates/contrapunk-audio/` — guitar pipeline + AudioNormalizer
> - `crates/contrapunk-harmony/` — harmony engine + voice leading + scales
> - `crates/contrapunk-midi/` — MIDI wrappers
> - `crates/contrapunk-transport/` — sample-accurate clock
> - `crates/contrapunk-chord/` — chord detection
> - `crates/contrapunk-companion/` — Companion + Canon/Counterpoint lanes
> - `crates/contrapunk-preset/` — preset manager
>
> `src/` now contains only the binary entry points + chain, fx, plugin_host,
> synth, server, router subdirs.
>
> **GeneratorPanel.svelte does not exist** in the current UI tree.
>
> **To regenerate this doc:** `/gsd-map-codebase` (project skill). Until then,
> treat anything below as approximate intent, NOT a file-citation reference.

**Analysis Date:** 2026-04-15 _(stamped STALE 2026-05-15 — see banner above)_

## Directory Layout

```
contrapunk/
├── src/                    # Core Rust library (all targets)
│   ├── audio/              # Guitar audio capture + DSP pipeline
│   ├── audio_out/          # cpal output stream + MIDI→synth queue
│   ├── harmony/            # Harmony engine, scales, voice leading
│   ├── humanize/           # Timing/velocity humanization + beat clock
│   ├── midi/               # MIDI I/O wrappers (midir, Web MIDI)
│   ├── generator/          # Note generator engine
│   ├── preset/             # Preset storage + builtins
│   ├── server/             # TCP server mode (native only)
│   ├── lib.rs              # Crate root, module declarations
│   ├── main.rs             # CLI binary (--server / --client)
│   ├── router.rs           # CLI MIDI routing loop
│   └── chord.rs            # Chord detection and display
├── src-tauri/              # Tauri v2 desktop app backend
│   ├── src/
│   │   ├── main.rs         # Tauri entry point, command registration
│   │   ├── state.rs        # AppState (Tauri managed state)
│   │   ├── guitar_bridge.rs# cpal audio capture → MIDI bytes bridge
│   │   └── commands/       # Tauri IPC command handlers
│   │       ├── engine.rs   # start_routing, stop_routing, get_note_state
│   │       ├── harmony.rs  # get_engine_state, set_key, set_mode, ...
│   │       ├── guitar.rs   # set_guitar_device, list_audio_devices
│   │       ├── audio_out.rs# start/stop audio output, list devices
│   │       ├── midi.rs     # list_midi_inputs, list_midi_outputs
│   │       ├── presets.rs  # list/load/save/delete presets
│   │       └── mod.rs
│   ├── tauri.conf.json     # Tauri app config (window size, bundle)
│   ├── Cargo.toml
│   └── build.rs
├── wasm/                   # wasm-bindgen bridge crate
│   └── src/lib.rs          # Engine + WasmGuitarInput exports
├── ui/                     # SvelteKit frontend (shared Tauri + WASM)
│   ├── src/
│   │   ├── routes/
│   │   │   ├── +layout.ts      # SSR=false, PostHog init
│   │   │   ├── +layout.svelte  # Root layout (particles, app shell)
│   │   │   ├── +page.svelte    # Main app page (component assembly)
│   │   │   ├── debug/          # Debug routes (/debug/guitar-midi)
│   │   │   └── diary/          # Dev diary pages
│   │   └── lib/
│   │       ├── adapter/        # Platform adapter layer
│   │       │   ├── index.ts    # Platform detection + singleton export
│   │       │   ├── types.ts    # ContrapunkAdapter interface + all types
│   │       │   ├── tauri.ts    # TauriAdapter (invoke/listen)
│   │       │   ├── wasm.ts     # WasmAdapter (WASM direct calls)
│   │       │   └── plugin.ts   # PluginAdapter (window.plugin IPC)
│   │       ├── stores/         # Svelte 5 rune stores
│   │       │   ├── engine.svelte.ts   # Harmony engine state
│   │       │   ├── guitar.svelte.ts   # Guitar input state
│   │       │   ├── midi.svelte.ts     # MIDI device selection
│   │       │   ├── beat.svelte.ts     # Beat clock / metronome
│   │       │   ├── suggestion.svelte.ts # Next-note suggestions
│   │       │   └── ui.svelte.ts       # UI preferences
│   │       ├── components/     # Svelte UI components
│   │       ├── audio/          # JS guitar capture (browser path)
│   │       │   ├── guitarCapture.ts   # getUserMedia + WasmGuitarInput
│   │       │   ├── guitarInputDsp.ts  # JS DSP utilities
│   │       │   └── pitchDetector.ts   # JS pitch detection (McLeod JS port)
│   │       ├── wasm-pkg/       # wasm-pack build output (gitignored)
│   │       └── theme/          # CSS theme tokens
│   ├── package.json
│   ├── svelte.config.js
│   └── vite.config.ts
├── plugin/                 # nih-plug VST3/CLAP plugin
│   └── src/
│       ├── lib.rs          # Plugin process() + parameter handling
│       └── editor.rs       # Webview GUI editor
├── ml/                     # Python ML pipeline (training only)
│   ├── training/           # PyTorch training scripts (rounds 01-05+)
│   ├── app/                # SvelteKit data collection app
│   ├── loader.py           # Training data loader (msgpack)
│   └── requirements.txt
├── au-wrapper/             # macOS AU wrapper (CMake, clap-wrapper)
├── examples/               # Standalone Rust binary examples
├── tests/                  # Integration tests
│   ├── audio_pipeline.rs
│   └── pitch_accuracy_benchmark.rs
├── scripts/                # Build/deploy helper scripts
├── assets/                 # Static assets (fonts)
├── deploy/                 # Fly.io deploy config + nginx
├── docs/                   # Documentation
├── Cargo.toml              # Workspace root (members: . src-tauri wasm plugin xtask)
├── Cargo.lock
├── guitar_calibration_profile.json   # Calibration data file
└── guitar_training_data.msgpack      # ML training dataset (156MB)
```

## Directory Purposes

**`src/audio/`:**
- Purpose: The full guitar-to-MIDI DSP pipeline
- Key files:
  - `guitar_input.rs` — `GuitarInput` struct (9-stage pipeline), `process_block()`, `MidiEvent` enum
  - `inference.rs` — Pure-Rust CNN forward pass (138-class mel-spectrogram classifier, 6 strings × 23 frets)
  - `guitar.rs` — `GuitarPitchMatcher`, inharmonicity B-coefficient string ID, `GuitarCalibrationProfile`
  - `pitch.rs` — `NoteTracker`, `DetectedPitch`, `freq_to_midi()`
  - `onset.rs` — `PluckDetector` (RMS + spectral flux), `RunningAutocorrelation`
  - `detectors.rs` — `BacfDetector`, `AmdfDetector`, `GoertzelBank`
  - `buffer.rs` — `DualBufferAnalyzer`, `RingBuffer`, `OverlapManager`

**`src/audio_out/`:**
- Purpose: Native audio output — cpal stream lifecycle, polyphonic sine synth, lock-free MIDI queue
- Key files:
  - `engine.rs` — `AudioOutEngine` (owns `cpal::Stream`), `start()` returns `MidiProducer`
  - `midi_queue.rs` — `midi_queue()`, `MidiProducer`, `MidiConsumer`, `MidiEvent` (audio synth variant)
  - `sine_synth.rs` — `PolySynth` (32-voice), `SineVoice` (ADSR sine oscillator)
  - `config.rs` — `AudioConfig` (sample rate, buffer size, channels, device)

**`src/harmony/`:**
- Purpose: All harmony generation logic
- Key files:
  - `engine.rs` — `HarmonyEngine`, `harmonize_note_on()`, `harmonize_note_off()`, voice count/leading/interchange
  - `config.rs` — `Key`, `HarmonyMode`, `ScaleMode`, `OctaveMode`, `RoutingMode` enums (57 scale modes)
  - `scale.rs` — `Scale`, diatonic transposition, interval math
  - `stateful.rs` — Stateful counterpoint (Species 1-4), voice state tracking
  - `modes.rs` — Per-mode harmonize implementations
  - `suggestion.rs` — `SuggestionScorer`, next-note ranking
  - `key_detect.rs` — Auto-key detection from note histogram
  - `barry_harris.rs` — Barry Harris chord/scale logic
  - `functional/` — Functional harmony chord progression engine
  - `voice_leading/` — Voice leading algorithms

**`src/humanize/`:**
- Purpose: Timing/velocity humanization and beat clock
- Key files:
  - `engine.rs` — `Humanizer`, `humanize_note_on()`, `humanize_note_off()`
  - `config.rs` — `HumanizeConfig` (jitter, velocity variation, swing, BPM)
  - `beat_clock.rs` — `BeatClock`, tempo tracking, beat position
  - `metronome.rs` — `Metronome` (MIDI channel 10 click output)

**`src/midi/`:**
- Purpose: MIDI port I/O wrappers
- Key files:
  - `input.rs` — `connect_input(port_idx, sender)` → `MidiInputConnection` (midir native)
  - `output.rs` — `OutputRouter` with `send_to_port()`, `send_to_first()`, `send_to_all()`
  - `ports.rs` — `list_input_ports()`, `list_output_ports()`, interactive selection helpers
  - `web.rs` — Web MIDI API wrappers (WASM target)

**`src-tauri/src/commands/`:**
- Purpose: All Tauri IPC surface. Each file is a logical domain.
- `engine.rs` — Routing lifecycle (`start_routing`, `stop_routing`, `get_note_state`), contains full `run_tauri_router` implementation
- `harmony.rs` — All harmony engine parameter setters + `get_engine_state`
- `guitar.rs` — Guitar device/config setters, audio device enumeration
- `audio_out.rs` — Audio output start/stop/status
- `midi.rs` — MIDI device enumeration and refresh
- `presets.rs` — Preset CRUD

**`ui/src/lib/adapter/`:**
- Purpose: Single stable API used by all Svelte components; hides Tauri vs WASM difference
- `index.ts` — Platform detection (`isTauri()`, `isPlugin()`) + singleton `adapter` export
- `types.ts` — `ContrapunkAdapter` interface (the only import components should need)
- `tauri.ts` — `TauriAdapter` using `@tauri-apps/api/core` `invoke` + `listen`
- `wasm.ts` — `WasmAdapter` using dynamic `import('$lib/wasm-pkg')` + Web MIDI API

**`ui/src/lib/stores/`:**
- Purpose: Svelte 5 rune (`$state`, `$derived`, `$effect`) reactive stores; source of truth for UI state
- `engine.svelte.ts` — `EngineStore` (key, mode, scale, voice settings, note state, chord name)
- `guitar.svelte.ts` — `GuitarInputStore` (latency, gain, device, live detection state)
- `midi.svelte.ts` — `MidiStore` (selected input/output indices, device lists)
- `beat.svelte.ts` — `BeatStore` (BPM, beat phase for UI indicator)

**`ui/src/lib/audio/`:**
- Purpose: Browser-side audio capture and pitch detection (WASM path only)
- `guitarCapture.ts` — `GuitarAudioCapture` class: `getUserMedia` → `ScriptProcessorNode` → `WasmGuitarInput.process_block()` → callbacks
- `pitchDetector.ts` — Pure JS McLeod pitch detector (used in debug page)
- `guitarInputDsp.ts` — JS DSP utilities (onset, spectrum)

**`wasm/src/lib.rs`:**
- Purpose: All wasm-bindgen exported types
- `Engine` — Harmony engine + humanizer + metronome for WASM; `note_on()`, `note_off()`, `tick()`, `get_state()`, `inject_note_on/off()`
- `WasmGuitarInput` — Thin wrapper over `GuitarInput`; `process_block(Float32Array) → JSON`

**`ml/`:**
- Purpose: Offline training pipeline only (Python + PyTorch). Not compiled into any runtime target.
- `training/` — Training scripts organized in rounds (01-05+); each round improves the CNN classifier
- `loader.py` — Reads `guitar_training_data.msgpack` (156MB dataset in repo root)
- The trained weights are exported to `src/audio/inference.rs` as hardcoded `Vec<f32>` constants (no external model file at runtime)

**`plugin/`:**
- Purpose: nih-plug VST3/CLAP plugin binary. Uses the same `contrapunk` core library as the Tauri app. GUI is a `nih_plug_webview` window embedding the same SvelteKit frontend using `PluginAdapter`.

## Key File Locations

**Entry Points:**
- `src-tauri/src/main.rs`: Tauri desktop app entry (`fn main()`)
- `wasm/src/lib.rs`: WASM module entry (`#[wasm_bindgen(start)] fn init_panic_hook()`)
- `ui/src/routes/+layout.ts`: Frontend bootstrap (SSR config, analytics)
- `ui/src/routes/+page.svelte`: Main app page (component mount, adapter init)
- `src/main.rs`: CLI binary entry (`fn main()`)
- `plugin/src/lib.rs`: nih-plug plugin entry

**Configuration:**
- `src-tauri/tauri.conf.json`: Window dimensions, bundle targets, dev server URL
- `Cargo.toml` (root): Workspace members, shared dependencies, WASM feature flags
- `ui/svelte.config.js`: SvelteKit adapter (static for Tauri, node for server)
- `ui/vite.config.ts`: Vite config

**Core Logic:**
- `src/harmony/engine.rs`: `HarmonyEngine` — central harmony transformation logic
- `src/audio/guitar_input.rs`: `GuitarInput::process_block()` — entire DSP pipeline in one file (~3800 lines)
- `src/audio/inference.rs`: CNN inference forward pass (no external model dependency)
- `src-tauri/src/commands/engine.rs`: `run_tauri_router` — Tauri router loop with audio fan-out (~600 lines)
- `src/router.rs`: CLI router loop (similar to Tauri version, used for `--client` mode)
- `ui/src/lib/adapter/index.ts`: Platform detection and adapter selection

**Audio Thread Boundary:**
- `src/audio_out/midi_queue.rs`: `midi_queue()` — creates the SPSC ringbuffer (lock-free bridge)
- `src/audio_out/engine.rs`: `AudioOutEngine::start()` — opens cpal stream, moves `MidiConsumer` into callback closure
- `src-tauri/src/commands/engine.rs` (line ~192): Where `MidiProducer` is taken from `AppState` and passed into router thread

**Testing:**
- `tests/audio_pipeline.rs`: Integration tests for DSP pipeline
- `tests/pitch_accuracy_benchmark.rs`: Pitch detection accuracy benchmarks
- Inline `#[cfg(test)]` modules in: `src/audio_out/midi_queue.rs`, `src/audio_out/engine.rs`, `src/audio_out/config.rs`, `src/router.rs`, `src-tauri/src/commands/audio_out.rs`

## Naming Conventions

**Files:**
- Rust modules: `snake_case.rs` (e.g., `guitar_input.rs`, `beat_clock.rs`)
- Svelte stores: `noun.svelte.ts` (e.g., `engine.svelte.ts`, `guitar.svelte.ts`)
- Svelte components: `PascalCase.svelte` (e.g., `ControlPanel.svelte`, `MidiDevices.svelte`)
- Adapter implementations: `platform.ts` (e.g., `tauri.ts`, `wasm.ts`, `plugin.ts`)

**Directories:**
- Rust: `snake_case/` for modules with multiple files
- Frontend: `kebab-case/` for lib subdirs (e.g., `wasm-pkg/`), `camelCase/` for route dirs (e.g., `debug/`)

**Types:**
- Rust public structs: `PascalCase` (e.g., `GuitarInput`, `HarmonyEngine`, `AudioOutEngine`)
- TypeScript interfaces: `PascalCase` (e.g., `ContrapunkAdapter`, `EngineState`)
- TypeScript store instances: `camelCase` singleton exports (e.g., `export const engine`, `export const guitar`)

## Where to Add New Code

**New Harmony Mode:**
- Add variant to `HarmonyMode` enum: `src/harmony/config.rs`
- Add harmonize implementation: `src/harmony/modes.rs` or new file in `src/harmony/`
- Add string parse: `wasm/src/lib.rs::parse_mode()`
- Add to `HarmonyModeName` union type: `ui/src/lib/stores/engine.svelte.ts`
- Update UI selector: `ui/src/lib/components/ControlPanel.svelte`

**New Tauri IPC Command:**
- Implement handler function in appropriate file under `src-tauri/src/commands/`
- Register in `src-tauri/src/main.rs` `invoke_handler![]` macro
- Add corresponding method to `ContrapunkAdapter` interface: `ui/src/lib/adapter/types.ts`
- Implement in `TauriAdapter`: `ui/src/lib/adapter/tauri.ts`
- Implement stub/equivalent in `WasmAdapter`: `ui/src/lib/adapter/wasm.ts`

**New Svelte Component:**
- Add file to `ui/src/lib/components/PascalCase.svelte`
- Import adapter via `import { adapter } from '$lib/adapter'`
- Read state via store imports: `import { engine } from '$lib/stores/engine.svelte'`

**New DSP Sub-module:**
- Add file to `src/audio/name.rs`
- Declare in `src/audio/mod.rs` with `pub mod name;` and `pub use name::...`

**New Store:**
- Add `ui/src/lib/stores/name.svelte.ts`
- Export a singleton class instance: `export const name = new NameStore()`
- Call `adapter.*()` for all mutations; sync state from backend in constructor or `init()`

**New Integration Test:**
- Add file to `tests/name.rs`
- Reference in `Cargo.toml` as `[[test]] name = "name" path = "tests/name.rs"`

## Special Directories

**`ui/src/lib/wasm-pkg/`:**
- Purpose: wasm-pack build output — `.wasm` binary + JS bindings
- Generated: Yes (by `wasm-pack build --target bundler` from `wasm/`)
- Committed: Yes (checked in so the UI works without rebuilding WASM)
- Key files: `contrapunk_wasm_bg.wasm` (492KB), `contrapunk_wasm.js`, `contrapunk_wasm.d.ts`

**`.planning/`:**
- Purpose: GSD planning documents, phase plans, research notes
- Generated: No
- Committed: Yes

**`target/`:**
- Purpose: Cargo build cache
- Generated: Yes
- Committed: No (gitignored)

**`ml/venv/`:**
- Purpose: Python virtualenv for training
- Generated: Yes
- Committed: No

**`au-wrapper/build/`:**
- Purpose: CMake build artifacts for the macOS AU wrapper (CLAP → AU bridge)
- Generated: Yes
- Committed: Partially (CMake config committed, build artifacts not)

---

*Structure analysis: 2026-04-15*
