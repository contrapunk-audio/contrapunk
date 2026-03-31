# Coding Conventions

**Analysis Date:** 2026-03-31

## Naming Patterns

**Files (TypeScript/Svelte):**
- Svelte components: PascalCase (`ControlPanel.svelte`, `MidiDevices.svelte`, `HumanizePanel.svelte`)
- Store files: camelCase with `.svelte.ts` suffix (`engine.svelte.ts`, `midi.svelte.ts`, `ui.svelte.ts`)
- Adapter files: camelCase (`tauri.ts`, `wasm.ts`, `types.ts`, `index.ts`)
- Route files: SvelteKit file-based routing (`+page.svelte`, `+layout.svelte`, `+layout.ts`)

**Files (Rust):**
- Modules: snake_case (`harmony_engine`, `voice_leading`, `beat_clock`)
- Files mirror module names exactly (`engine.rs`, `rules.rs`, `voicer.rs`)
- Test helper files: descriptive snake_case (`test_signals.rs`)

**TypeScript identifiers:**
- Interfaces and types: PascalCase (`ContrapunkAdapter`, `EngineState`, `MidiDevice`, `HumanizeState`)
- String literal union types: PascalCase values (`'PassThrough'`, `'DiatonicThirds'`, `'BachChorale'`)
- Constants: SCREAMING_SNAKE_CASE for module-level data (`ALL_KEYS`, `SCALE_FAMILIES`, `SETTINGS_KEY`, `SETTINGS_VERSION`)
- Functions: camelCase (`loadSettings`, `saveSettings`, `computeScaleNotes`, `isTauri`)
- Svelte store singletons: single-word camelCase (`engine`, `midi`, `ui`)
- Private class fields: underscore prefix (`_isRunning`, `_detuneCents`)

**Rust identifiers:**
- Types and structs: PascalCase (`HarmonyEngine`, `VoiceRegister`, `StyleRules`, `ChordPattern`)
- Functions and methods: snake_case (`note_on`, `set_key`, `build_registers`, `check_parallel_fifths`)
- Constants: SCREAMING_SNAKE_CASE (`NOTE_NAMES`, `CHORD_PATTERNS`, `STRING_BASE_PITCH`)
- Modules: snake_case (`voice_leading`, `harmony`, `chord`)
- Test functions: descriptive snake_case (`test_engine_creation`, `bacf_detect_440hz`, `test_parallel_fifths_detected`)

## Code Style

**Formatting (TypeScript/Svelte):**
- No `.prettierrc` or `.eslintrc` detected — formatting is enforced by `svelte-check` via `tsconfig.json`
- TypeScript strict mode enabled (`"strict": true` in `ui/tsconfig.json`)
- `moduleResolution: "bundler"` with `esModuleInterop` and `forceConsistentCasingInFileNames`
- Tabs used for indentation throughout TypeScript files
- Trailing commas used consistently in multi-line objects and arrays

**Formatting (Rust):**
- Standard `rustfmt` conventions (4-space indentation)
- Module-level doc comments use `//!` style
- Item-level doc comments use `///` style
- Inline comments use `//` with a space

**Linting:**
- TypeScript: `svelte-check` with strict TypeScript config
- ESLint: not configured (no `.eslintrc` or `eslint.config.*` found)
- Rust: standard `rustc` warnings enforced; no additional clippy config detected

## Import Organization

**TypeScript order (observed pattern):**
1. External packages (`@tauri-apps/api/core`, `@tauri-apps/api/event`)
2. Internal `$lib` path-aliased imports (`$lib/adapter`, `$lib/stores/engine.svelte`)
3. Relative imports (`./types`, `./tauri`, `./wasm`)

**Path Aliases:**
- `$lib` → `ui/src/lib/` (SvelteKit convention)
- `$lib/wasm-pkg` → `ui/src/lib/wasm-pkg/` (WASM output)

**Svelte component imports:**
- Always use `$lib/components/ComponentName.svelte` pattern
- Store imports always include both the singleton and needed type exports

**Rust imports:**
- `use super::*` in `#[cfg(test)]` modules to import the module under test
- Cross-module imports use full crate paths (`crate::harmony::config::HarmonyMode`)

## Error Handling

**TypeScript adapter pattern:**
- All async adapter methods wrap calls in `try/catch`
- Errors are rethrown as `new Error(\`Descriptive message: ${e}\`)` with context prefix
- Non-critical operations (inject note on/off, note polling) silently return fallback values on failure rather than throwing
- WASM stub methods (unimplemented features) silently accept calls rather than throwing

**Example pattern (`ui/src/lib/adapter/tauri.ts`):**
```typescript
async setKey(key: string): Promise<void> {
    try {
        await invoke('set_key', { key });
    } catch (e) {
        throw new Error(`Failed to set key: ${e}`);
    }
}
```

**Store optimistic update pattern (`ui/src/lib/stores/engine.svelte.ts`):**
```typescript
async setKey(newKey: KeyName) {
    const prev = this.key;
    this.key = newKey;   // optimistic update
    try {
        await adapter.setKey(newKey);
        this.persist();
    } catch (e) {
        this.key = prev; // rollback on failure
        throw e;
    }
}
```

**Rust error handling:**
- `anyhow::Result` used as the primary error type for fallible operations
- WASM-exported functions use `Result<T, JsValue>` for JavaScript interop
- Panics are acceptable for programming errors (wrong state); recoverable errors use `Result`
- Test assertions use descriptive messages: `assert!(cond, "explanation {}", value)`

## Logging

**TypeScript:**
- Logger: native `console` API
- Namespace prefix: `[contrapunk]` on all log calls
- `console.error` for initialization failures in `+page.svelte`
- `console.warn` for non-fatal restore/sync failures in stores
- No `console.log` used in production paths — only warn/error

**Rust:**
- No logging framework detected in core lib
- WASM uses `console_error_panic_hook` for panic messages in browser console

## Comments

**When to Comment:**
- Module-level `//!` doc comments on every Rust module explaining purpose and architecture
- Function-level `///` doc comments on all public Rust functions and methods
- JSDoc/TSDoc `/** ... */` block comments on all exported TypeScript interfaces and class methods
- Inline `//` comments explain non-obvious business logic (e.g., MIDI status byte bit masks, voice leading rules)

**TSDoc pattern (observed in `ui/src/lib/adapter/types.ts`):**
```typescript
/**
 * Subscribe to real-time note updates.
 * Returns an unsubscribe function.
 */
onNoteUpdate(callback: (state: NoteState) => void): () => void;
```

**Rust doc pattern (observed in `src/harmony/engine.rs`):**
```rust
//! Main harmony engine that routes notes through mode-specific algorithms.
//!
//! # Processing Pipeline
//!
//! ```text
//! Note-On -> Scale Check -> Mode Algorithm -> Voice Leading -> Octave Mode -> Output
//! ```

/// Builds register assignments based on voice position.
///
/// Assigns registers to the full voice arrangement (0=top to N-1=bass),
/// then reorders to match final_result layout.
fn build_registers_for_position(voice_count: usize, voice_position: usize) -> Vec<VoiceRegister>
```

**Section dividers:**
- TypeScript files use `// === Section Name ===` dividers between logical blocks
- Rust files use `// ====...====` comment blocks for major section separations

## Function Design

**Size:** Functions are kept focused. Private helper methods extract complex sub-logic (e.g., `mapEngineState`, `mapNoteState`, `sortVoices`, `centsToPitchBend`).

**Parameters:**
- TypeScript async methods prefer individual typed parameters over config objects for simple cases
- Partial config objects (`Partial<HumanizeState>`) used for update operations
- Optional parameters use `?` suffix with `?? defaultValue` at use site

**Return Values:**
- Async adapter methods return `Promise<void>` or `Promise<T>`; never `Promise<undefined>`
- Synchronous store actions (detune) are non-async when the backend call is fire-and-forget
- Rust public functions return `Vec<Note>` for harmony output, `Option<T>` for lookup operations

## Module Design

**TypeScript exports:**
- Types defined in `types.ts`, re-exported from `index.ts` for consumer convenience
- Store singletons exported as named `const` (not default exports): `export const engine = new EngineStore()`
- Adapter singleton: `export const adapter: ContrapunkAdapter = isTauri() ? new TauriAdapter() : new WasmAdapter()`
- No barrel files beyond the adapter `index.ts`

**Svelte component design:**
- Components import from stores directly, never from other components
- All state accessed via store singletons (`engine.key`, `midi.selectedInput`)
- Local reactive state uses `$state()`, derived values use `$derived()`
- Side effects use `$effect()`

**Rust module exports:**
- All public API re-exported via `pub mod` in `lib.rs`
- Platform-conditional modules use `#[cfg(not(target_arch = "wasm32"))]` guards
- Feature-gated dependencies use `#[cfg(feature = "web-midi")]`

## Data Mapping Conventions

**Snake_case ↔ camelCase boundary:**
- Rust backend uses snake_case for all serialized fields (`mode_number`, `voice_leading_enabled`)
- TypeScript layer maps these to camelCase on ingress (`mapEngineState`, `mapNoteState` functions in `ui/src/lib/adapter/tauri.ts`)
- WASM layer reads snake_case fields directly from WASM module return values with `?? default` fallback

**Validation on deserialization:**
- localStorage settings validated field-by-field against enum sets (`VALID_KEYS`, `VALID_MODES`)
- Schema version checked before use; mismatched versions trigger removal and return `null`
- Numeric bounds validated inline (e.g., `interchangeRange >= 1 && <= 5`)

---

*Convention analysis: 2026-03-31*
