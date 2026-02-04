# Codebase Structure

**Analysis Date:** 2026-02-04

## Directory Layout

```
contrapunk/
├── src/               # Rust source code
│   ├── harmony/       # Musical harmony generation core
│   ├── midi/          # MIDI I/O abstraction
│   ├── generator/     # Pattern-based note generation
│   ├── humanize/      # Timing humanization
│   ├── preset/        # Configuration save/load
│   ├── server/        # TCP server for remote clients
│   ├── theme/         # GUI visual styling
│   ├── app.rs         # Main GUI application
│   ├── router.rs      # MIDI routing orchestration
│   ├── main.rs        # Native entry point
│   ├── lib.rs         # WASM entry point
│   ├── ui.rs          # GUI layout and controls
│   ├── piano.rs       # Visual piano keyboard widget
│   ├── chord.rs       # Chord display and analysis
│   └── midi_defaults.rs  # Default MIDI configurations
├── assets/            # Static resources
│   └── fonts/         # Custom fonts for UI
├── deploy/            # Deployment artifacts
│   └── dist/          # Built WASM/web files for hosting
├── dist/              # Local build output for web target
├── target/            # Cargo build artifacts (gitignored)
├── .planning/         # GSD planning documents
├── Cargo.toml         # Rust project manifest
├── Cargo.lock         # Dependency lock file
├── index.html         # WASM application shell
├── Trunk.toml         # Trunk build configuration
└── README.md          # Project documentation
```

## Directory Purposes

**src/**
- Purpose: All Rust application source code
- Contains: Modules, entry points, domain logic
- Key files: `main.rs` (native), `lib.rs` (WASM), `app.rs` (GUI)

**src/harmony/**
- Purpose: Core musical theory and harmony generation algorithms
- Contains: `engine.rs` (orchestration), `config.rs` (types), `modes.rs` (algorithms), `scale.rs`, `stateful.rs` (stateful modes), `voice_leading/` (counterpoint rules)
- Key files: `mod.rs` (public API), `engine.rs` (HarmonyEngine)

**src/harmony/voice_leading/**
- Purpose: Voice leading rules and chord voicing logic
- Contains: `rules.rs` (interval checks), `styles.rs` (style presets), `voicer.rs` (revoicing algorithm), `suspension.rs` (suspension tracking)
- Key files: `mod.rs` (public exports), `voicer.rs` (main revoicing function)

**src/midi/**
- Purpose: Platform-specific MIDI input/output abstraction
- Contains: `input.rs`, `output.rs`, `ports.rs` (native via midir), `web.rs` (WASM via Web MIDI API)
- Key files: `mod.rs` (conditional compilation), `ports.rs` (device enumeration)

**src/generator/**
- Purpose: Automated note pattern generation (arpeggios, chords, sequences)
- Contains: `config.rs` (generator settings), `engine.rs` (note generation logic)
- Key files: `mod.rs` (public API), `engine.rs` (NoteGenerator)

**src/humanize/**
- Purpose: Timing variation and natural feel for MIDI output
- Contains: `config.rs`, `engine.rs` (Humanizer), `scheduler.rs` (DelayQueue), `metronome.rs`, `beat_clock.rs`
- Key files: `engine.rs` (humanization logic), `scheduler.rs` (delay queue)

**src/preset/**
- Purpose: Save/load harmony configurations as named presets
- Contains: `mod.rs` (PresetManager), `builtins.rs` (factory presets), `storage.rs` (persistence layer)
- Key files: `mod.rs` (preset CRUD), `builtins.rs` (default presets)

**src/server/**
- Purpose: TCP server for remote MIDI streaming and harmony processing
- Contains: `config.rs`, `protocol.rs` (wire format), `session.rs` (client handler), `mod.rs` (server loop)
- Key files: `protocol.rs` (message serialization), `session.rs` (client session)

**src/theme/**
- Purpose: GUI visual styling (colors, widgets, effects)
- Contains: `colors.rs` (color palette), `widgets.rs` (custom UI elements), `mod.rs` (theme configuration)
- Key files: `mod.rs` (ContrapunkTheme), `widgets.rs` (ornate frames, scanlines)

**assets/**
- Purpose: Static files bundled with application
- Contains: Fonts for UI rendering
- Generated: No
- Committed: Yes

**deploy/dist/**
- Purpose: Production-ready web deployment files
- Contains: Compiled WASM, JS, HTML for hosting
- Generated: Yes (via Trunk)
- Committed: No (deployment artifacts)

**dist/**
- Purpose: Local development web build output
- Contains: WASM build artifacts for testing
- Generated: Yes (via Trunk)
- Committed: No

**target/**
- Purpose: Rust build cache and binaries
- Contains: Compiled artifacts for debug/release/WASM targets
- Generated: Yes (via Cargo)
- Committed: No

**.planning/**
- Purpose: GSD (Getting Stuff Done) workflow planning documents
- Contains: Roadmaps, phase plans, summaries, codebase analysis
- Generated: Partially (by GSD commands)
- Committed: Yes

## Key File Locations

**Entry Points:**
- `src/main.rs`: Native desktop entry (CLI and GUI modes)
- `src/lib.rs`: WASM browser entry
- `index.html`: WASM application HTML shell

**Configuration:**
- `Cargo.toml`: Rust dependencies, features, build settings
- `Trunk.toml`: WASM build configuration
- `.gitignore`: Build artifacts exclusion

**Core Logic:**
- `src/harmony/engine.rs`: Harmony transformation orchestration (1476 lines)
- `src/router.rs`: MIDI routing and message processing (701 lines)
- `src/app.rs`: GUI application state and UI (1356 lines)

**Testing:**
- No dedicated test directory (tests colocated in source files via `#[cfg(test)]`)
- Example: `src/harmony/engine.rs` contains `mod tests` at line 793

## Naming Conventions

**Files:**
- Module files: `snake_case.rs` (e.g., `voice_leading.rs`, `midi_defaults.rs`)
- Module index: `mod.rs` in subdirectories
- Special entries: `main.rs` (native), `lib.rs` (WASM)

**Directories:**
- All lowercase `snake_case` (e.g., `harmony`, `voice_leading`, `humanize`)
- Domain-aligned naming (musical concepts: `harmony`, `chord`; technical: `server`, `midi`)

## Where to Add New Code

**New Harmony Mode:**
- Algorithm function: `src/harmony/modes.rs` (add function like `harmonize_mode_N`)
- Mode enum variant: `src/harmony/config.rs` (add to `HarmonyMode` enum)
- Engine integration: `src/harmony/engine.rs` (add case in `harmonize()` match)
- Tests: `src/harmony/engine.rs` (add to `mod tests`)

**New GUI Control:**
- UI layout: `src/ui.rs` (add widget in appropriate section function)
- State binding: `src/app.rs` (add field to `AppState`, update in `refresh_state()`)
- Styling: `src/theme/` if custom widget needed

**New MIDI Processing Feature:**
- Router integration: `src/router.rs` (modify main loop or add to pipeline)
- Shared state: `src/router.rs` (add field to `GUIRouterState` if GUI-accessible)

**New Preset:**
- Built-in preset: `src/preset/builtins.rs` (add to `default_presets()` function)
- Preset type extension: `src/preset/mod.rs` (modify `Preset` struct)

**Utilities:**
- Shared helpers: Place in relevant module (e.g., MIDI utilities in `src/midi/`, harmony utilities in `src/harmony/`)
- Cross-cutting: Create new top-level module in `src/` (e.g., `src/utils.rs` if truly general)

**New Server Feature:**
- Protocol extension: `src/server/protocol.rs` (add `Message` variant, update serialization)
- Session handling: `src/server/session.rs` (modify `handle_client()`)

## Special Directories

**target/**
- Purpose: Cargo compilation artifacts (debug, release, WASM targets)
- Generated: Yes
- Committed: No
- Size: Large (hundreds of MB)

**dist/** and **deploy/dist/**
- Purpose: Trunk web build output (WASM, JS, processed HTML)
- Generated: Yes (via `trunk build`)
- Committed: No (dist), deployment decision (deploy/dist)

**assets/**
- Purpose: Static resources bundled by Trunk for web build
- Generated: No (manually placed)
- Committed: Yes

**.planning/**
- Purpose: GSD planning workflow documentation
- Generated: Partially (via `/gsd:*` commands)
- Committed: Yes
- Structure: `ROADMAP.md`, `STATE.md`, phase folders, `codebase/` analysis

## Module Organization Pattern

**Typical module structure:**
```
src/module_name/
├── mod.rs           # Public API, re-exports
├── config.rs        # Types, enums, configuration
├── engine.rs        # Main logic/algorithms
└── submodule/       # Optional sub-organization
    ├── mod.rs
    └── feature.rs
```

**Examples:**
- `src/harmony/`: Follows pattern (mod.rs, config.rs, engine.rs, scale.rs, etc.)
- `src/generator/`: Minimal variant (mod.rs, config.rs, engine.rs only)
- `src/midi/`: Platform-split variant (mod.rs with conditional compilation, separate files per platform)

## File Size Distribution

**Large files (complexity hotspots):**
- `src/harmony/engine.rs`: 1476 lines (harmony orchestration)
- `src/app.rs`: 1356 lines (GUI application)
- `src/harmony/stateful.rs`: 990 lines (stateful mode algorithms)
- `src/ui.rs`: 693 lines (UI layout)
- `src/router.rs`: 701 lines (MIDI routing)
- `src/harmony/scale.rs`: 543 lines (scale definitions)

**Module total lines:** ~4023 lines in src root files

---

*Structure analysis: 2026-02-04*
