# Coding Conventions

**Analysis Date:** 2026-02-05

## Naming Patterns

**Files:**
- Modules use snake_case: `voice_leading.rs`, `midi_defaults.rs`, `beat_clock.rs`
- Main entry points: `main.rs`, `lib.rs`, `app.rs`
- Config modules: `config.rs` (inside feature subdirectories)

**Functions:**
- snake_case for all functions: `detect_chord()`, `revoice_chord()`, `harmonize()`
- Constructor pattern: `new()` for associated functions
- Getter pattern: simple name without `get_` prefix (e.g., `key()`, `mode()`, `range()`)
- Setter pattern: `set_` prefix (e.g., `set_key()`, `set_mode()`)
- Boolean accessors: no `is_` prefix in method names typically

**Variables:**
- snake_case for locals and fields: `note_duration_beats`, `last_beat_position`, `harmony_pitch_classes`
- Abbreviations lowercase: `bpm`, `ms`, `pc` (pitch class)

**Types:**
- PascalCase for structs and enums: `HarmonyEngine`, `NoteGenerator`, `VoiceRegister`, `StylePreset`
- Enum variants: PascalCase (e.g., `Key::C`, `HarmonyMode::DiatonicThirds`, `OctaveMode::None`)
- Trait implementations for Display use `std::fmt::Display`

## Code Style

**Formatting:**
- Standard Rust formatting (no custom rustfmt.toml detected)
- 4-space indentation (Rust default)
- Line length: appears to follow default 100-character limit
- Trailing commas in multi-line lists

**Linting:**
- No custom clippy config detected (uses Rust defaults)
- Code follows standard Rust idioms
- No `#[allow(...)]` attributes observed in sampled code

## Import Organization

**Order:**
1. Standard library imports (e.g., `use std::collections::HashSet;`, `use std::io::{Read, Write};`)
2. External crate imports (e.g., `use wmidi::{MidiMessage, Note, Channel, Velocity};`, `use rand::Rng;`, `use serde::{Serialize, Deserialize};`)
3. Internal module imports with `crate::` prefix (e.g., `use crate::harmony::{Key, HarmonyMode};`)
4. Relative imports with `super::` (e.g., `use super::config::HumanizeConfig;`)

**Path Aliases:**
- No custom path aliases
- Crate-relative imports use `crate::module::Type`
- Parent module imports use `super::module`

## Error Handling

**Patterns:**
- Uses `anyhow` crate for error handling in application code
- Result types: `anyhow::Result<T>` for functions that can fail
- Explicit error context with `anyhow!()` macro for creating errors
- Propagate errors with `?` operator
- Example from `src/server/protocol.rs`:
  ```rust
  pub fn read_message(stream: &mut impl Read) -> Result<Message> {
      let mut len_buf = [0u8; 2];
      stream.read_exact(&mut len_buf)?;
      let len = u16::from_be_bytes(len_buf) as usize;
      if len == 0 {
          return Err(anyhow!("invalid message: zero length"));
      }
      // ...
  }
  ```

## Logging

**Framework:** Standard output (no dedicated logging framework detected)

**Patterns:**
- No structured logging observed in core modules
- Error messages via `anyhow!()` for propagation
- Console output in CLI portions (`std::io::{self, Write}`)

## Comments

**When to Comment:**
- Module-level documentation for every file using `//!` doc comments
- Public API functions documented with `///` doc comments
- Complex algorithms explained inline (e.g., voice leading rules)
- Intent documented for non-obvious code (e.g., borrowing sources in scales)

**Doc Comments:**
- Module docs (`//!`) appear at top of each file with purpose and context
- Function docs (`///`) include:
  - Purpose description
  - `# Arguments` section with parameter descriptions
  - `# Returns` section for return value
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

## Function Design

**Size:** Functions range from small helpers (~10 lines) to larger complex algorithms (~100-200 lines for voice leading)

**Parameters:**
- Borrow by reference for read-only access: `&HashSet<u8>`, `&[u8]`
- Mutable references for modification: `&mut self`, `&mut impl Write`
- Owned parameters for consuming: `Vec<u8>`
- Option types for optional parameters: `Option<&VoiceAnchor>`, `Option<usize>`

**Return Values:**
- Use `Result<T>` for fallible operations
- Use `Option<T>` for operations that may not produce a value
- Return `Vec<T>` for collections
- Return owned types for newly constructed values

## Module Design

**Exports:**
- Public re-exports in `mod.rs` files for clean API surface
- Example from `src/harmony/mod.rs`:
  ```rust
  mod config;
  mod engine;
  mod modes;
  mod scale;
  mod stateful;
  pub mod voice_leading;

  pub use config::{Key, HarmonyMode, OctaveMode, ScaleFamily, ScaleMode};
  pub use engine::HarmonyEngine;
  pub use scale::Scale;
  pub use stateful::{ContraryMotionState, CounterpointState};
  pub use voice_leading::VoiceLeadingStyle;
  ```

**Barrel Files:**
- `mod.rs` serves as barrel file for each module
- Re-exports primary types for clean imports
- Submodules remain private unless explicitly made public

## Serialization

**Pattern:**
- Use `serde` with derive macros for data structures
- Enum serialization uses snake_case: `#[serde(rename_all = "snake_case")]`
- Default values: `#[serde(default)]` or custom default functions
- Skip fields in serialization: `#[serde(skip)]`
- Example from `src/harmony/config.rs`:
  ```rust
  #[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
  #[serde(rename_all = "snake_case")]
  pub enum Key {
      C, Db, D, Eb, E, F, Gb, G, Ab, A, Bb, B,
  }
  ```

## Constants

**Pattern:**
- SCREAMING_SNAKE_CASE for constants
- Const arrays for lookup tables and patterns
- Example from `src/chord.rs`:
  ```rust
  const NOTE_NAMES: [&str; 12] = ["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"];
  const NOTE_NAMES_FLAT: [&str; 12] = ["C", "Db", "D", "Eb", "E", "F", "Gb", "G", "Ab", "A", "Bb", "B"];
  ```
- Private const for internal use, public const for API exposure

## Platform-Specific Code

**Pattern:**
- Use `#[cfg(target_arch = "wasm32")]` for WASM-specific code
- Use `#[cfg(not(target_arch = "wasm32"))]` for native-only code
- Use `#[cfg(feature = "gui")]` for GUI-specific code
- Example from `src/main.rs`:
  ```rust
  #[cfg(not(target_arch = "wasm32"))]
  mod router;

  #[cfg(feature = "gui")]
  mod app;
  ```

---

*Convention analysis: 2026-02-05*
