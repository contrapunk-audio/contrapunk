# Coding Conventions

**Analysis Date:** 2026-04-15

## Naming Patterns

**Rust files:**
- `snake_case` for all filenames: `guitar_input.rs`, `voice_leading.rs`, `sine_synth.rs`
- One public struct/trait per module file; the filename matches the concept, not the type
- Sub-modules grouped in directories with `mod.rs` as the public re-export surface

**Rust types:**
- `PascalCase` structs: `HarmonyEngine`, `GuitarInput`, `SineVoice`, `PluckDetector`
- `PascalCase` enums: `HarmonyMode`, `ScaleMode`, `OctaveMode`, `MidiEvent`
- `SCREAMING_SNAKE_CASE` constants: `SAMPLE_RATE`, `STRING_BASE_PITCH`, `CONSONANT_INTERVALS_ABOVE`
- `snake_case` functions and methods: `freq_to_midi`, `transpose_diatonic`, `process_block`
- Boolean fields spelled out fully: `interchange_enabled`, `bends_enabled`, `is_running`

**TypeScript files:**
- `PascalCase.svelte` for components: `ControlPanel.svelte`, `GuitarInputPanel.svelte`
- `camelCase.ts` for pure logic: `pitchDetector.ts`, `guitarCapture.ts`, `guitarInputDsp.ts`
- `camelCase.svelte.ts` for Svelte 5 rune stores: `engine.svelte.ts`, `guitar.svelte.ts`, `beat.svelte.ts`
- `PascalCase` for adapter classes: `TauriAdapter`, `WasmAdapter`
- `camelCase` for interfaces: `ContrapunkAdapter`, `EngineState`, `NoteState` (interfaces are PascalCase but read naturally)

**TypeScript identifiers:**
- `camelCase` for properties and methods throughout: `voiceLeadingEnabled`, `modeNumber`, `startRouting`
- Backend Rust uses `snake_case`; the adapter layer maps explicitly: `raw.mode_number as number` → `modeNumber`
- Svelte store instances are lowercase singletons: `engine`, `guitar`, `beat`, `midi`, `ui`

## Code Style

**Rust formatting:**
- Tool: `rustfmt` (enforced in CI via `cargo fmt --all -- --check`)
- Standard Rust style; no custom rustfmt.toml found in repo root
- Enforced as the first CI job; blocks merge if formatting fails

**Rust linting:**
- Tool: `cargo clippy -- -W clippy::all` in CI; pre-commit hook runs `cargo clippy --all-targets -D warnings`
- No `#![allow(...)]` blanket suppressions observed
- WASM target compiled separately: `cargo check -p contrapunk-wasm --target wasm32-unknown-unknown`

**TypeScript:**
- `tsconfig.json` uses `"strict": true` — all strict checks enabled
- `"moduleResolution": "bundler"` (SvelteKit/Vite bundler resolution)
- `svelte-check` runs against tsconfig; included in CI `frontend` job via `npm run check`
- No ESLint config found at root or in `ui/`; `svelte-check` handles TS type checking
- Occasional `// eslint-disable-next-line @typescript-eslint/no-explicit-any` comments indicate `any` is consciously avoided but sometimes required for WASM module interop

## Import Organization

**Rust imports:**
- Standard library (`use std::...`) first, then external crates, then internal (`use crate::...`)
- Internal imports use explicit `crate::` paths, never relative `super::` chains more than one level
- Example from `src/harmony/engine.rs`:
  ```rust
  use std::collections::HashMap;
  use wmidi::Note;

  use crate::harmony::config::{BeatPhase, HarmonyMode, Key, OctaveMode, ScaleMode};
  use crate::harmony::functional;
  use crate::harmony::scale::Scale;
  ```

**TypeScript imports:**
- External packages first, then `$lib/` path-alias imports
- Path alias `$lib/` maps to `ui/src/lib/`; all internal imports use this alias, never relative `../`
- Example from `ui/src/lib/adapter/tauri.ts`:
  ```typescript
  import { invoke } from '@tauri-apps/api/core';
  import { listen, type UnlistenFn } from '@tauri-apps/api/event';
  import { guitar } from '$lib/stores/guitar.svelte';
  import type { ContrapunkAdapter, EngineState } from './types';
  ```

**Re-exports:**
- Each `mod.rs` / `harmony/mod.rs` explicitly re-exports public types with `pub use`:
  ```rust
  pub use config::{HarmonyMode, Key, OctaveMode, ScaleMode};
  pub use engine::HarmonyEngine;
  pub use scale::Scale;
  ```
- Adapter layer uses barrel re-export in `ui/src/lib/adapter/index.ts`

## Error Handling

**Rust — library core (`src/`):**
- `anyhow::Result<T>` for fallible functions that can fail for multiple reasons (MIDI port operations)
- `anyhow!("message")` macro for constructing errors with context strings
- `?` operator throughout for error propagation; no `unwrap()` in production paths
- Example from `src/midi/output.rs`:
  ```rust
  use anyhow::{anyhow, Result};
  .ok_or_else(|| anyhow!("Invalid output port index: {}", idx))?;
  ```

**Rust — Tauri commands (`src-tauri/`):**
- All `#[tauri::command]` functions return `Result<T, String>` for IPC serialization
- Internal errors converted via `.map_err(|e| e.to_string())` at the IPC boundary
- Mutex poisoning handled explicitly: `state.engine.lock().map_err(|e| e.to_string())?`
- `anyhow::Result<()>` used for internal helper functions inside command modules (not the exported commands themselves)
- Example from `src-tauri/src/commands/harmony.rs`:
  ```rust
  #[tauri::command]
  pub fn get_engine_state(state: State<AppState>) -> Result<EngineStateResponse, String> {
      let engine = state.engine.lock().map_err(|e| e.to_string())?;
      Ok(EngineStateResponse { ... })
  }
  ```

**Rust — WASM bridge (`wasm/`):**
- WASM-exported functions return `Result<T, JsValue>` for JS interop
- Errors constructed via `Err(JsValue::from_str(&format!("message")))`

**TypeScript — adapter layer:**
- Every adapter method wraps its backend call in `try/catch` and rethrows as `new Error("Failed to X: ${e}")`
- Components never call Tauri/WASM directly; always through the adapter interface
- Error boundaries not present as formal Svelte constructs; errors surface as rejected promises in calling code
- Example from `ui/src/lib/adapter/tauri.ts`:
  ```typescript
  async setKey(key: string): Promise<void> {
      try {
          await invoke('set_key', { key });
      } catch (e) {
          throw new Error(`Failed to set key: ${e}`);
      }
  }
  ```

## Logging

**Rust:**
- `eprintln!("[module-tag] message", ...)` for runtime errors and warnings in non-realtime paths
- Tags use bracket notation with module context: `[tauri-router]`, `[calibration]`, `[guitar-bridge]`
- No structured logging crate (no `tracing`, `log`, `env_logger`); `eprintln!` is used throughout
- `println!` used in test output paths (test helpers that print detection results)

**WASM:**
- `console_log!` macro defined in `wasm/src/lib.rs` wrapping `web_sys::console::log_1`
- Panic hook installed via `console_error_panic_hook::set_once()` in `#[wasm_bindgen(start)]`

**TypeScript:**
- `console.warn` and `console.error` used directly in adapter edge cases; no logging abstraction

## Comments

**Module-level doc comments (`//!`):**
- Every Rust file starts with `//!` module documentation describing purpose, pipeline, and key types
- Complex modules include ASCII art pipeline diagrams inline in the doc comment
- Example from `src/audio/guitar_input.rs`:
  ```rust
  //! DSP-based guitar input pipeline: pitch detection, string/fret identification, MIDI output.
  //!
  //! # Pipeline
  //! 1. **Onset detection** -- RMS spike + spectral flux with cooldown
  //! 2. **Note state machine** -- Idle/Attack/Sustain/Decay hysteresis
  ```

**Item-level doc comments (`///`):**
- Every public struct, enum, and function has `///` doc comments
- Struct fields have inline `///` or end-of-line comments explaining units and valid ranges
- Example from `src/audio/guitar_input.rs`:
  ```rust
  /// Analysis window in samples (256-2048).
  pub buffer_size: usize,
  /// RMS threshold for pluck / onset detection (seed for adaptive mode).
  pub onset_threshold: f32,
  ```

**Inline comments:**
- `//` used for non-obvious logic explanations; code that is self-explanatory is uncommented
- Section dividers use `// ──────────────────────────` or `// ===` to delineate logical sections within long files
- `// TODO:` and `// FIXME:` used sparingly for known gaps; some include sub-project tags: `// FIXME(sub-project-2):`

**TypeScript:**
- JSDoc-style `/** */` block comments on classes, interface definitions, and non-obvious functions
- `/** @deprecated */` not observed; stubs use `// Desktop X deferred` inline comments
- `// eslint-disable-next-line` used as override comments when unavoidable

## Function Design

**Size:** Functions stay focused; long algorithms are factored into private helpers within the same file

**Parameters:** Prefer taking owned values or references explicitly; configs passed as value types (`GuitarInputConfig` is `Clone`)

**Return values:**
- `Option<T>` for "might not be present" (pitch detection returning `None` when no pitch found)
- `Result<T, E>` for operations that can fail with meaningful error context
- `Vec<Note>` for harmony output (always returns melody note as first element)
- Avoid returning empty types as `()` when the call has a side-effect worth confirming

## Module Design

**Rust:**
- Public API re-exported at the module root via `pub use` in `mod.rs`
- Internal implementation types kept private (no `pub` on helpers)
- Platform-gated modules use `#[cfg(not(target_arch = "wasm32"))]` at the `mod` declaration in `mod.rs`, not inside files
- Feature-gated modules use `#[cfg(feature = "web-midi")]`

**TypeScript:**
- Svelte 5 runes stores (`$state`, `$derived`) are defined in `.svelte.ts` files (not `.ts`), co-located in `ui/src/lib/stores/`
- The adapter pattern is the only integration seam: components import from `$lib/adapter`, never from `$lib/adapter/tauri` or `$lib/adapter/wasm` directly
- `$lib/adapter/index.ts` selects the correct adapter at runtime based on Tauri availability

## Feature Flagging Patterns

**Rust Cargo features:**
- `web-midi` feature enables WASM-compatible Web MIDI API (declared in root `Cargo.toml`)
- Platform split is primarily via `#[cfg(not(target_arch = "wasm32"))]` rather than named features
- WASM-incompatible modules (`router`, `server`, `midi/input`, `midi/output`, `midi/ports`) are all gated with this cfg
- Native-only crates (`midir`, `cpal`, `clap`, `rmp-serde`) declared under `[target.'cfg(not(target_arch = "wasm32"))'.dependencies]`

**TypeScript:**
- No compile-time feature flags; runtime adapter selection via `typeof window.__TAURI__ !== 'undefined'` (resolved in `$lib/adapter/index.ts`)
- UI stubs for desktop-only features return empty arrays or no-ops, with comments like `// Desktop X deferred`

## Commit Message Conventions

Format: `type(scope): description`

**Types observed:**
- `feat` — new capability added
- `fix` — bug correction
- `docs` — documentation only
- `refactor` — structural change without behavior change
- `wip` — work in progress / stash recovery (rare; used for interim commits)

**Scope examples:**
- `(tauri)` — Tauri IPC layer changes
- `(audio-out)` — audio output subsystem
- `(harmony)` — harmony engine
- `(wasm)` — WASM bridge
- `(ui)` — frontend-only changes
- `(windows)` — platform-specific build changes

**Rules (from PR template checklist and observed history):**
- Lowercase type prefix with parenthesized scope
- Imperative mood in description: "add", "fix", "port", "expose", not "adding" or "added"
- No period at end of subject line
- No Claude attribution in commit messages or PR descriptions

---

*Convention analysis: 2026-04-15*
