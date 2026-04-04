# Coding Conventions

**Analysis Date:** 2026-04-04

## Naming Patterns

**Files:**
- Rust modules use `snake_case` with `.rs` extension
- Public command modules grouped in `commands/` directory with descriptive names: `harmony.rs`, `engine.rs`, `guitar.rs`, `presets.rs`, `midi.rs`
- Private helper modules like `state.rs`, `guitar_bridge.rs` follow same snake_case convention
- Example: `src/commands/harmony.rs`, `src/state.rs`, `src/guitar_bridge.rs`

**Functions:**
- Public functions use `snake_case`: `get_engine_state()`, `set_guitar_device()`, `start_routing()`, `list_presets()`
- Private helper functions use `snake_case`: `parse_key()`, `process_midi_message()`, `send_humanized_note()`
- Tauri command functions marked with `#[tauri::command]` attribute
- Examples: `get_engine_state()`, `set_scale_mode()`, `run_tauri_router()`

**Variables:**
- Local variables use `snake_case`: `input_notes`, `harmony_notes`, `borrowed_notes`, `guitar_device`, `engine_config`
- Mutable variables explicitly declared with `mut` keyword
- State references use descriptive names: `state`, `engine`, `output_router`, `humanizer`
- Examples: `let mut engine = state.engine.lock().map_err(|e| e.to_string())?;`

**Types:**
- Struct names use `PascalCase`: `AppState`, `NoteUpdatePayload`, `EngineStateResponse`, `GuitarBridge`, `PresetInfo`
- Enum names use `PascalCase`: `HarmonyMode`, `ScaleMode`, `OctaveMode`, `Key`, `VoiceLeadingStyle`
- Type aliases use `PascalCase`: `EngineConfig`
- Examples: `pub struct AppState { ... }`, `enum HarmonyMode { PassThrough, DiatonicThirds, ... }`

## Code Style

**Formatting:**
- Uses Rust standard formatting conventions (implied by project structure)
- 4-space indentation (Rust standard)
- Line length targets 80-100 characters but not strictly enforced
- Brace style: opening braces on same line (K&R style): `fn name() {`

**Linting:**
- No explicit `.clippy.toml` or `rustfmt.toml` configuration file in repository
- Assumes standard Rust conventions via `cargo clippy` and `cargo fmt` defaults
- Code follows idiomatic Rust patterns: ownership, borrowing, Result/Option semantics

## Import Organization

**Order:**
1. Standard library imports (`std::...`)
2. External crate imports (third-party dependencies)
3. Local crate imports (internal modules via `crate::...` or relative paths)
4. Re-exports marked with `pub use`

**Pattern examples:**
```rust
use std::collections::HashSet;
use std::sync::atomic::AtomicBool;
use std::sync::Mutex;

use tauri::State;
use serde::Serialize;

use contrapunk::harmony::{HarmonyEngine, HarmonyMode, Key};
use contrapunk::audio::guitar_input::GuitarInputConfig;

use crate::state::AppState;
```

**Path Aliases:**
- No path aliases configured (uses absolute imports)
- Imports from parent crate `contrapunk` use full path: `use contrapunk::harmony::HarmonyEngine;`
- Internal module imports via `crate::` prefix: `use crate::state::AppState;`

## Error Handling

**Patterns:**
- Tauri command functions return `Result<T, String>` with `String` as error type (Tauri IPC serialization requirement)
- String conversion pattern for lock errors: `state.engine.lock().map_err(|e| e.to_string())?`
- Router thread uses `anyhow::Result<()>` for context-rich error handling
- Propagation via `?` operator throughout
- `unwrap()` used sparingly in hot paths (event emission timing loop) where lock cannot fail
- `unwrap_or_default()` for fallback values in audio configuration
- Descriptive error messages: `"Routing is already active"`, `"At least one output port required"`

**Examples:**
```rust
// Tauri command style
pub fn set_key(key: String, state: State<AppState>) -> Result<(), String> {
    let parsed = parse_key(&key)?;
    let mut engine = state.engine.lock().map_err(|e| e.to_string())?;
    engine.set_key(parsed);
    Ok(())
}

// Router thread style
fn run_tauri_router(...) -> anyhow::Result<()> {
    let output_router = OutputRouter::new(output_ports)?;
    // ...
}

// Hot path with unwrap (after checking lock safety)
let payload = {
    let in_notes = input_notes.lock().unwrap();
    // ...
};
```

## Logging

**Framework:** Standard Rust `eprintln!` macro for errors and diagnostic output

**Patterns:**
- Errors logged to stderr with context prefix: `eprintln!("[tauri-router] Error: {}", e);`
- Audio device fallback logged: `eprintln!("[guitar_bridge] Device '{}' not found, using default", device_name);`
- Audio stream errors logged: `eprintln!("Guitar audio error: {}", err);`
- Used only for error conditions and fallback diagnostics, not general logging

## Comments

**When to Comment:**
- Module-level documentation via `//!` comments explaining purpose and architecture
- Complex algorithms documented with diagrams (e.g., processing pipeline in `guitar_input.rs`)
- Non-obvious implementation details explained (e.g., sentinel values for virtual inputs)
- Configuration constants documented with purpose

**JSDoc/TSDoc:**
- Rust uses `///` for public API documentation
- All public functions documented with purpose and parameters
- Example from `state.rs`:
  ```rust
  /// Managed Tauri state wrapping HarmonyEngine and related components.
  ///
  /// AppState is registered with Tauri's managed state system and accessed
  /// via `State<AppState>` in command handlers.
  pub struct AppState { ... }
  ```

**Module documentation example:**
```rust
//! Tauri commands for harmony engine control.
//!
//! Get/set key, mode, scale mode, octave mode, voice leading, interchange,
//! and voice position.
```

## Function Design

**Size:**
- Public command functions typically 10-30 lines (simple lock + delegation pattern)
- Helper functions like `parse_*()` vary 5-20 lines
- Router thread functions (`run_tauri_router`, `process_midi_message`) can be 50-100+ lines for state management and branching
- No explicit size limits enforced

**Parameters:**
- Tauri command functions take `state: State<AppState>` as last parameter
- Utility functions use explicit Arc/Mutex parameters when needed
- Multiple related parameters sometimes grouped (e.g., `engine_config` tuple)
- Named parameters preferred over positional for clarity

**Return Values:**
- Tauri commands: `Result<T, String>` where T is Serialize-able type
- Router operations: `anyhow::Result<()>` for context-rich errors
- Option types used for optional values: `Option<GuitarInputConfig>`
- Collections returned as Vec or references depending on ownership needs

## Module Design

**Exports:**
- Tauri commands marked public and listed in `main.rs` invoke_handler
- State struct and helper structs marked `pub` for Tauri access
- Helper functions like `parse_*()` kept private (module-scoped)
- Example structure: `pub mod harmony;` in `commands/mod.rs`

**Barrel Files:**
- `commands/mod.rs` acts as barrel export for command submodules
- Single line per module: `pub mod harmony;`, `pub mod engine;`, etc.
- No re-exports of external types, just module organization

## Threading & Concurrency

**Patterns:**
- Arc<Mutex<T>> for shared state across threads (command handlers to router thread)
- Arc<AtomicBool> for simple flags like `is_running`
- mpsc::channel<Vec<u8>> for MIDI message communication between input and routing
- Ownership captured in thread closures via `move` keyword
- Lock-hold times kept minimal (lock, copy data, unlock immediately)

**Example:**
```rust
let input_notes = Arc::new(Mutex::new(HashSet::<u8>::new()));
let in_notes = Arc::clone(&input_notes);  // Clone for thread

thread::spawn(move || {
    // Use in_notes in thread
    let mut notes = in_notes.lock().unwrap();
    notes.insert(note_val);
});
```

## Tauri Integration Patterns

**Command Handler Pattern:**
```rust
#[tauri::command]
pub fn command_name(param: Type, state: State<AppState>) -> Result<ReturnType, String> {
    // Acquire state lock
    let data = state.field.lock().map_err(|e| e.to_string())?;
    // Process
    // Return Result
}
```

**State Management:**
- All mutable state wrapped in Mutex or AtomicBool
- Locks acquired briefly, data copied before releasing
- Shared state between command handlers and router thread via Arc references

---

*Convention analysis: 2026-04-04*
