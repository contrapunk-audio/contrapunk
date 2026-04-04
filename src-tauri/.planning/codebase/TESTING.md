# Testing Patterns

**Analysis Date:** 2026-04-04

## Test Framework

**Runner:**
- Rust's built-in test harness via `cargo test`
- Tests compiled and run by: `cargo test --lib` or `cargo test`
- Config: No explicit `Cargo.toml` test configuration in `src-tauri/` subproject

**Assertion Library:**
- Standard Rust assertions: `assert!`, `assert_eq!`, `assert_ne!`
- No external assertion crate (relies on standard library `assert_*!` macros)

**Run Commands:**
```bash
cargo test                    # Run all tests (from src-tauri or parent crate)
cargo test --lib             # Run library tests only
cargo test --release         # Run tests in release mode
cargo test -- --nocapture    # Show println! output during tests
```

## Test File Organization

**Location:**
- Tests defined inline within the module they test, not in separate files
- Uses `#[cfg(test)]` attribute to gate test modules (compiled only for test builds)
- This co-location pattern visible throughout parent crate: `src/harmony/engine.rs`, `src/harmony/scale.rs`, `src/audio/guitar_input.rs`

**Naming:**
- Test functions prefixed with `test_`: `test_engine_creation()`, `test_engine_pass_through()`, `test_diatonic_thirds()`
- Test module always named `tests`: `mod tests { ... }`
- Descriptive test names indicate what is being tested and expected outcome

**Structure:**
```
src/
├── commands/
│   ├── harmony.rs         # Implementation + tests
│   ├── engine.rs          # Implementation + tests
│   └── ...
├── state.rs               # Implementation (no tests)
└── main.rs               # Entry point (no tests)

src/  (parent crate)
├── harmony/
│   ├── engine.rs          # Implementation + #[cfg(test)] mod tests
│   ├── scale.rs           # Implementation + #[cfg(test)] mod tests
│   └── ...
├── audio/
│   ├── guitar_input.rs    # Implementation + #[cfg(test)] mod tests
│   └── ...
└── lib.rs
```

## Test Structure

**Suite Organization:**
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine_creation() {
        // Arrange
        let engine = HarmonyEngine::new(Key::C, HarmonyMode::DiatonicThirds);
        
        // Act & Assert
        assert_eq!(engine.key(), Key::C);
        assert_eq!(engine.mode(), HarmonyMode::DiatonicThirds);
    }

    #[test]
    fn test_engine_pass_through() {
        let mut engine = HarmonyEngine::new(Key::C, HarmonyMode::PassThrough);
        let result = engine.harmonize(Note::C4);
        assert_eq!(result, vec![Note::C4]);
    }
}
```

**Patterns:**
- `use super::*;` imports the parent module's public items
- Simple Arrange-Act-Assert (AAA) structure
- Minimal setup; tests focus on single behavior
- Test names describe the scenario: `test_engine_creation`, `test_diatonic_thirds`, `test_key_change`

## Mocking

**Framework:** No external mocking crate (e.g., `mockito`, `mockall`)

**Patterns:**
- Struct behaviors tested via public methods; no mocks needed for most cases
- For components with dependencies (e.g., `HarmonyEngine` with `Scale`), tests instantiate real objects
- No dependency injection pattern; structures are initialized directly

**What to Mock:**
- Nothing currently mocked in tests; tests exercise real implementations
- Rationale: Core logic (harmony, scale, voice leading) has no external I/O dependencies
- Tests can run deterministically without network, file, or async concerns

**What NOT to Mock:**
- Audio/MIDI I/O (not tested in unit tests; would require integration tests)
- Tauri state and command handlers (tested via black-box integration, not direct unit tests)
- Network operations (server module tests likely integration-focused)

## Fixtures and Factories

**Test Data:**
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_c_major_scale_degrees() {
        let scale = Scale::major(0); // C major
        assert_eq!(scale.degree_of(Note::C4), Some(0));
        assert_eq!(scale.degree_of(Note::D4), Some(1));
        assert_eq!(scale.degree_of(Note::Db4), None);
    }

    #[test]
    fn test_diatonic_third_up() {
        let scale = Scale::major(0); // C major
        // ... test behavior
    }
}
```

**Location:**
- Fixtures defined locally within test functions (minimal setup)
- No external fixture files or factories
- Simple constructors like `Scale::major(0)`, `HarmonyEngine::new(Key::C, HarmonyMode::DiatonicThirds)` used directly

**Pattern:**
- Constructor functions with sensible defaults: `Scale::major()`, `Default::default()`
- Tests instantiate expected state directly inline

## Coverage

**Requirements:** Not enforced. No coverage target or CI requirement specified in `Cargo.toml`

**View Coverage:**
```bash
# Using tarpaulin (requires separate installation)
cargo tarpaulin --out Html

# Using llvm-cov (requires separate installation)
cargo llvm-cov
```

**Notes:**
- No coverage measurement in the project yet
- Main focus areas for testing: harmony algorithms, scale operations, voice leading rules (core logic)
- Tauri command handlers have lower test coverage (rely on integration tests)
- Audio pipeline (guitar_input) has extensive test coverage in parent crate

## Test Types

**Unit Tests:**
- **Scope:** Single struct/function behavior in isolation
- **Approach:** Direct instantiation of object under test, call public method, assert result
- **Examples:** 
  - `test_engine_creation` - tests HarmonyEngine initialization
  - `test_engine_pass_through` - tests PassThrough mode returns input unchanged
  - `test_c_major_scale_degrees` - tests Scale degree calculation
  - `test_diatonic_third_up` - tests diatonic transposition logic
- **Coverage:** Harmony algorithms, scale operations, parsing logic

**Integration Tests:**
- **Scope:** Not yet visible in codebase structure; likely future focus
- **Approach:** Would test Tauri IPC with full router thread, audio capture, MIDI I/O
- **Status:** No explicit integration test directory or files found in `src-tauri/`

**E2E Tests:**
- **Framework:** Not used
- **Status:** Desktop app testing via Tauri likely done manually or via platform-specific UI automation (not in repo)

## Common Patterns

**Async Testing:**
- Not needed for current tests (no async/await in test logic)
- Router thread uses `thread::spawn()`, but tests don't directly test threading (would require synchronization assertions)
- If needed in future, would use `#[tokio::test]` with `tokio` dev-dependency

**Error Testing:**
- Harmony algorithm tests verify error conditions implicitly via assertions
- Parsing functions (`parse_key`, `parse_harmony_mode`) tested via match patterns
- No explicit error propagation tests with `?` operator

**Example: Implicit error testing via input validation**
```rust
#[test]
fn test_parse_key_unknown() {
    let result = parse_key("X");
    // Parser returns error for unknown key
    // (implicitly tested via match behavior)
}
```

## Test Execution Context

**Test Files in src-tauri:**
- No explicit test files in `src-tauri/src/` yet
- Tauri backend primarily tested via Tauri IPC calls (integration testing)
- Core harmony logic tested in parent crate

**Parent Crate Tests:**
- Extensive tests in `/src/harmony/engine.rs`, `/src/harmony/scale.rs`, etc.
- Tests instantiate HarmonyEngine, call methods, verify output
- No mocks; all state is deterministic and reproducible

**Running Tauri Tests:**
```bash
cd src-tauri
cargo test                    # Run tests (currently none in this directory)
cd ..
cargo test --lib             # Run parent crate unit tests
```

---

*Testing analysis: 2026-04-04*
