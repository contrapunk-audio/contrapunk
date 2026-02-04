# Coding Conventions

**Analysis Date:** 2026-02-04

## Naming Patterns

**Files:**
- Snake case for module files: `harmony_engine.rs`, `voice_leading.rs`, `midi_defaults.rs`
- Match module name to directory: `src/harmony/mod.rs` contains `harmony` module
- Test modules use `mod.rs` or are co-located with implementation

**Functions:**
- Snake case: `harmonize_note_on()`, `detect_chord()`, `transpose_diatonic()`
- Predicates use `is_` prefix: `is_in_scale()`, `is_flat_key()`
- Conversion methods use `from_` or `to_`: `semitones_from_c()`, `to_vec()`
- Builder pattern uses `with_`: `with_voices()`, `with_analysis()`

**Variables:**
- Snake case: `pitch_classes`, `current_offset`, `last_borrowed_from`
- Mutable variables clearly marked with `mut` keyword
- Iterator variables typically short: `i`, `n`, `pc`

**Types:**
- Pascal case for structs/enums: `HarmonyEngine`, `ScaleMode`, `VoiceRegister`
- Trait names are adjectives when possible: (no custom traits observed)
- Type aliases rare; prefer explicit types

**Constants:**
- SCREAMING_SNAKE_CASE: `NOTE_NAMES`, `CONSONANT_INTERVALS_ABOVE`, `TYPE_MIDI_DATA`
- Array constants for lookup tables: `CHORD_PATTERNS`, `NOTE_NAMES_FLAT`

## Code Style

**Formatting:**
- Tool used: rustfmt (default Rust formatter)
- No custom config detected (using Rust defaults)
- Line length: appears to respect 100-char soft limit
- Indentation: 4 spaces (Rust standard)

**Linting:**
- Tool used: clippy (Rust's linter)
- No custom clippy.toml detected (using defaults)
- Code follows standard Rust idioms
- Warnings treated seriously (no `#[allow]` attributes observed except for necessary cases)

## Import Organization

**Order:**
1. Standard library (`std::collections::HashMap`, `std::io`)
2. External crates (`wmidi::Note`, `rand::Rng`, `anyhow::Result`)
3. Local crate modules (`crate::harmony::config`, `crate::midi::ports`)

**Path Aliases:**
- Use `crate::` for absolute paths within project
- Relative imports use `super::` or explicit module paths
- Common pattern: `use crate::harmony::{Key, HarmonyMode, OctaveMode}`

**Grouping:**
- Related imports grouped together
- Wildcard imports avoided (use explicit item imports)
- Example from `src/harmony/engine.rs`:
  ```rust
  use std::collections::HashMap;
  use wmidi::Note;

  use crate::harmony::config::{Key, HarmonyMode, OctaveMode, ScaleMode};
  use crate::harmony::modes;
  use crate::harmony::scale::Scale;
  ```

## Error Handling

**Patterns:**
- Primary: `anyhow::Result<T>` for operations that can fail
- Fallible operations return `Option<T>` for simple cases (e.g., `transpose_diatonic()` returns `Option<Note>`)
- Pattern matching on `Result`/`Option` preferred over unwrapping
- Early returns with `?` operator for error propagation
- Example from `src/harmony/scale.rs`:
  ```rust
  pub fn transpose_diatonic(&self, note: Note, degrees: i8) -> Option<Note> {
      let current_degree = self.degree_of(note)? as i8;
      // ... computation ...
      Note::try_from(new_midi as u8).ok()
  }
  ```

**Fallback Strategy:**
- Out-of-range MIDI notes return `None` instead of panicking
- Invalid input falls back to safe defaults (e.g., pass-through mode returns original note)
- MIDI range checked explicitly: `if !(0..=127).contains(&midi)`

## Logging

**Framework:** Standard library `eprintln!` for errors; no structured logging framework

**Patterns:**
- Errors printed to stderr with context
- Debug info uses `#[derive(Debug)]` for struct inspection
- No verbose logging in production code paths
- Console output in WASM via `console_error_panic_hook::set_once()`

## Comments

**When to Comment:**
- Module-level doc comments (`//!`) explain purpose and key concepts
- Public functions have doc comments (`///`) with Args/Returns sections
- Complex algorithms explained inline (e.g., chord detection, voice leading)
- Non-obvious optimizations documented

**Doc Comments:**
- Full `///` documentation for all public API surfaces
- Arguments section: `/// # Arguments` with `* name - description` format
- Returns section: `/// # Returns` describes output
- Examples included for complex functions
- Example from `src/chord.rs`:
  ```rust
  /// Detects the chord from a set of MIDI note numbers.
  ///
  /// Analyzes the pitch classes (ignoring octaves) and attempts to match
  /// against known chord patterns. Detects slash chords when the lowest
  /// note differs from the chord root.
  ///
  /// # Arguments
  /// * `notes` - Set of MIDI note numbers (0-127)
  ///
  /// # Returns
  /// The chord name (e.g., "Cmaj", "Am7", "Cmaj/E") or None if no chord detected.
  pub fn detect_chord(notes: &HashSet<u8>) -> Option<String>
  ```

**Inline Comments:**
- Clarify intent, not implementation (unless algorithm is complex)
- Used sparingly; prefer self-documenting code
- Mark algorithm steps in complex functions (e.g., `// First pass: try to find...`)

## Function Design

**Size:**
- Functions typically 20-50 lines
- Complex functions (like `harmonize()`) reach 100+ lines but are well-structured
- Helper functions extracted when logic is reused

**Parameters:**
- Pass small types by value (`note: Note`, `degrees: i8`)
- Pass collections by reference (`notes: &HashSet<u8>`, `scale: &mut Scale`)
- Mutable references used sparingly and clearly (`&mut self`, `&mut Scale`)
- Builder pattern for complex initialization (`with_voices()`)

**Return Values:**
- Prefer `Option<T>` for operations that may fail gracefully
- Use `Result<T>` for operations with detailed error context
- Return owned `Vec` for generated collections
- Return references when borrowing from self (`&[usize]` for port maps)

## Module Design

**Exports:**
- Public API clearly marked with `pub`
- Internal helpers remain private (module-scoped)
- Re-export key types in parent modules: `pub use self::scale::Scale;`

**Barrel Files:**
- `mod.rs` used to organize submodules
- Example from `src/harmony/mod.rs`:
  ```rust
  pub mod config;
  pub mod engine;
  pub mod modes;
  pub mod scale;
  pub mod stateful;
  pub mod voice_leading;

  pub use config::{Key, HarmonyMode, OctaveMode, ScaleMode};
  pub use engine::HarmonyEngine;
  pub use scale::Scale;
  ```

**Module Structure:**
- Flat when possible; nested only when grouping related functionality
- Example: `src/harmony/voice_leading/` contains `mod.rs`, `rules.rs`, `voicer.rs`, `styles.rs`, `suspension.rs`
- Each module focused on single responsibility

## Type System Usage

**Enums:**
- Used for finite sets of options: `HarmonyMode`, `ScaleMode`, `OctaveMode`
- Derive traits liberally: `#[derive(Debug, Clone, Copy, PartialEq, Eq)]`
- Pattern matching exhaustive (no wildcard fallbacks in mode selection)

**Structs:**
- Organize related state: `HarmonyEngine`, `Scale`, `ContraryMotionState`
- Private fields with public accessors (getters/setters)
- Builder pattern for complex types (`HarmonyEngine::with_voices()`)

**Type Safety:**
- Newtype pattern not used; rely on semantic naming
- MIDI note numbers wrapped in `wmidi::Note` type (external crate)
- Pitch classes represented as `u8` (0-11) by convention

## Conditional Compilation

**Feature Flags:**
- `#[cfg(feature = "gui")]` gates GUI-only modules (`app.rs`, `ui.rs`, `piano.rs`)
- `#[cfg(not(target_arch = "wasm32"))]` excludes native-only code (server, router)
- `#[cfg(target_arch = "wasm32")]` includes WASM entry point (`lib.rs`)

**Platform-Specific:**
- MIDI backend selection via target arch
- Server mode disabled in WASM builds
- Example from `src/main.rs`:
  ```rust
  #[cfg(not(target_arch = "wasm32"))]
  use clap::Parser;

  #[cfg(target_arch = "wasm32")]
  use wasm_bindgen::prelude::*;
  ```

---

*Convention analysis: 2026-02-04*
