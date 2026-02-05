# Codebase Structure

**Analysis Date:** 2026-02-05

## Directory Layout

```
contrapunk/
├── src/               # All Rust source code
│   ├── harmony/       # Core harmony algorithms and musical logic
│   ├── midi/          # Platform-specific MIDI I/O
│   ├── humanize/      # Timing/velocity variation engine
│   ├── generator/     # Note generator (arpeggiators, chord patterns)
│   ├── server/        # TCP server/client for network mode
│   ├── preset/        # Preset management and storage
│   ├── theme/         # GUI theme (colors, widgets, steampunk styling)
│   ├── app.rs         # Main GUI application (eframe)
│   ├── ui.rs          # UI component implementations
│   ├── router.rs      # MIDI routing and message processing
│   ├── main.rs        # Native entry point (CLI/GUI/server)
│   └── lib.rs         # WASM entry point
├── assets/            # Static assets (fonts, etc.)
├── deploy/            # Deployment config (Dockerfile, nginx, fly.toml)
├── dist/              # WASM build output (generated)
├── target/            # Cargo build artifacts (generated)
├── .planning/         # Project planning and documentation
├── Cargo.toml         # Rust project manifest
├── Trunk.toml         # WASM build config
└── index.html         # WASM entry HTML
```

## Directory Purposes

**src/**
- Purpose: All Rust source modules
- Contains: .rs files organized by functional area
- Key files: `main.rs`, `lib.rs`, `app.rs`, `router.rs`

**src/harmony/**
- Purpose: Musical transformation algorithms
- Contains: `engine.rs` (main HarmonyEngine), `scale.rs`, `modes.rs`, `config.rs`, `stateful.rs`, `voice_leading/`
- Key files: `engine.rs` (58KB, core logic), `stateful.rs` (counterpoint state), `scale.rs` (modal transposition)

**src/midi/**
- Purpose: MIDI device communication (platform-specific)
- Contains: `input.rs`, `output.rs`, `ports.rs`, `web.rs` (WASM), `mod.rs`
- Key files: `ports.rs` (device enumeration), `output.rs` (OutputRouter)

**src/humanize/**
- Purpose: Timing and velocity humanization
- Contains: `engine.rs`, `scheduler.rs` (DelayQueue), `beat_clock.rs`, `metronome.rs`, `config.rs`, `mod.rs`
- Key files: `engine.rs` (Humanizer), `scheduler.rs` (DelayQueue)

**src/generator/**
- Purpose: Virtual MIDI input generators
- Contains: `engine.rs`, `config.rs`, `mod.rs`
- Key files: `engine.rs` (NoteGenerator with arpeggiator, chord, scale modes)

**src/server/**
- Purpose: Network MIDI processing (native-only)
- Contains: `session.rs`, `protocol.rs`, `config.rs`, `mod.rs`
- Key files: `session.rs` (TCP session handling), `protocol.rs` (message serialization)

**src/preset/**
- Purpose: Musical style presets
- Contains: `builtins.rs` (11+ predefined presets), `storage.rs` (eframe persistence), `mod.rs`
- Key files: `builtins.rs` (preset definitions)

**src/theme/**
- Purpose: GUI styling and visual effects
- Contains: `colors.rs`, `widgets.rs` (scanlines, ornate frame), `mod.rs`
- Key files: `widgets.rs` (custom pixel-art decorations)

**deploy/**
- Purpose: Production deployment configuration
- Contains: `Dockerfile`, `nginx.conf`, `fly.toml`, `dist/` (deployed WASM)
- Key files: `fly.toml` (Fly.io config)

**.planning/**
- Purpose: Project planning and codebase documentation
- Contains: `codebase/` (this document), `phases/`, `ROADMAP.md`, `STATE.md`
- Key files: `ROADMAP.md` (feature roadmap), `STATE.md` (current development state)

## Key File Locations

**Entry Points:**
- `src/main.rs`: Native desktop entry (CLI/GUI/server/client)
- `src/lib.rs`: WASM entry point (browser)

**Configuration:**
- `Cargo.toml`: Rust dependencies, features, build profile
- `Trunk.toml`: WASM build settings
- `index.html`: WASM canvas container

**Core Logic:**
- `src/harmony/engine.rs`: HarmonyEngine (note harmonization)
- `src/router.rs`: MIDI routing loop (28KB, 792 lines)
- `src/app.rs`: ContrapunkApp (GUI state, 53KB, 1279 lines)

**Testing:**
- No dedicated test directory (tests co-located or missing)

## Naming Conventions

**Files:**
- Module files: `mod.rs` (re-exports), snake_case for others (`beat_clock.rs`, `voice_leading.rs`)
- Large modules: Split into subdirectories with `mod.rs` (e.g., `harmony/mod.rs`)

**Directories:**
- Lowercase, snake_case: `voice_leading/`, `humanize/`

**Types:**
- PascalCase: `HarmonyEngine`, `OutputRouter`, `DelayQueue`
- Enums: `HarmonyMode`, `ScaleMode`, `GeneratorMode`

**Functions:**
- snake_case: `harmonize_note_on`, `spawn_gui_router`, `drain_ready`

**Constants:**
- SCREAMING_SNAKE_CASE: `MAX_OUTPUT_SLOTS`, `INPUT_NOTE_GENERATOR`

## Where to Add New Code

**New Harmony Mode:**
- Primary code: `src/harmony/modes.rs` (algorithm implementation)
- Config: `src/harmony/config.rs` (add variant to `HarmonyMode` enum)
- Engine: `src/harmony/engine.rs` (add match arm in `harmonize_note_on`)

**New Scale/Mode:**
- Implementation: `src/harmony/config.rs` (add to `ScaleMode` enum and intervals)
- Tests: Co-locate in same file or add test module

**New UI Tab:**
- Implementation: `src/ui.rs` (add tab enum variant, render function)
- State: `src/app.rs` (add fields to `ContrapunkApp` if needed)

**New Preset:**
- Builtin: `src/preset/builtins.rs` (add to `builtin_presets()` function)
- Custom: User-created via GUI, stored in eframe storage

**New MIDI Feature:**
- Input handling: `src/router.rs` (message processing functions)
- Output: `src/midi/output.rs` (OutputRouter methods)
- Platform-specific: `src/midi/web.rs` (WASM), `src/midi/input.rs` (native)

**New Generator Mode:**
- Primary code: `src/generator/engine.rs` (add variant to `GeneratorMode`, implement in `tick()`)
- Config: `src/generator/config.rs` if new settings needed

**Utilities:**
- Shared helpers: Add to appropriate module (no dedicated utils/ directory)
- MIDI utilities: `src/midi/mod.rs`
- Music theory utilities: `src/harmony/scale.rs` or `src/harmony/config.rs`

## Special Directories

**target/**
- Purpose: Cargo build artifacts
- Generated: Yes (by cargo)
- Committed: No (.gitignore)

**dist/**
- Purpose: WASM build output (Trunk)
- Generated: Yes (by trunk build)
- Committed: No (.gitignore), but `deploy/dist/` is committed for Fly.io

**assets/**
- Purpose: Static assets (fonts)
- Generated: No
- Committed: Yes

**.planning/**
- Purpose: Project planning documentation
- Generated: No (manually maintained)
- Committed: Yes

**src/harmony/voice_leading/**
- Purpose: Voice leading algorithms (Palestrina, Jazz, Bach styles)
- Generated: No
- Committed: Yes
- Contains: `voicer.rs`, `rules.rs`, `suspension.rs`, `styles.rs`, `mod.rs`

## Module Organization Pattern

**Typical module structure:**
```
module_name/
├── mod.rs          # Public API, re-exports
├── config.rs       # Configuration structs/enums
├── engine.rs       # Main implementation
└── [specific].rs   # Domain-specific logic
```

**Examples:**
- `harmony/` → `mod.rs`, `config.rs`, `engine.rs`, `scale.rs`, `modes.rs`, `stateful.rs`
- `generator/` → `mod.rs`, `config.rs`, `engine.rs`
- `server/` → `mod.rs`, `config.rs`, `session.rs`, `protocol.rs`

**Flat modules (no subdirectory):**
- `app.rs`, `ui.rs`, `router.rs`, `piano.rs`, `chord.rs`, `midi_defaults.rs`

## Conditional Compilation Patterns

**Feature flags:**
- `#[cfg(feature = "gui")]` → GUI-specific code (app, ui, piano, theme)
- `#[cfg(not(feature = "gui"))]` → CLI-specific code

**Target architecture:**
- `#[cfg(target_arch = "wasm32")]` → Browser-specific (Web MIDI, WASM bindings)
- `#[cfg(not(target_arch = "wasm32"))]` → Native-specific (midir, threads, server)

**Common pattern in main.rs, lib.rs, app.rs:**
```rust
#[cfg(not(target_arch = "wasm32"))]
use midir;

#[cfg(target_arch = "wasm32")]
use web_sys;
```

## File Size Guidelines

**Large files (>1000 lines):**
- `src/app.rs` (1279 lines) → Main GUI application
- `src/ui.rs` (27KB) → UI rendering
- `src/router.rs` (792 lines) → MIDI routing
- `src/harmony/engine.rs` (1400+ lines) → Core harmony logic
- `src/harmony/stateful.rs` (900+ lines) → Counterpoint state

**Typical module files: 200-500 lines**

**Small config files: <200 lines**

---

*Structure analysis: 2026-02-05*
