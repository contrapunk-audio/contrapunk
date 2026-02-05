# Testing Patterns

**Analysis Date:** 2026-02-05

## Test Framework

**Runner:**
- Built-in Rust test framework (cargo test)
- No external test harness detected

**Assertion Library:**
- Standard library `assert_eq!`, `assert!` macros

**Run Commands:**
```bash
cargo test                  # Run all tests
cargo test --lib           # Run only library tests
cargo test <test_name>     # Run specific test
cargo test -- --nocapture  # Show stdout during tests
```

## Test File Organization

**Location:**
- Tests are co-located with source code (not in separate test files)
- Tests live at the bottom of each module file inside `#[cfg(test)] mod tests`
- No separate `tests/` directory for integration tests

**Naming:**
- No `*_test.rs` files (all tests inline)
- Test module always named `tests`
- Test functions prefixed with `test_`

**Structure:**
```
src/
├── chord.rs              # Contains #[cfg(test)] mod tests
├── harmony/
│   ├── engine.rs         # Contains #[cfg(test)] mod tests
│   ├── scale.rs          # Contains #[cfg(test)] mod tests
│   ├── modes.rs          # Contains #[cfg(test)] mod tests
│   ├── stateful.rs       # Contains #[cfg(test)] mod tests
│   └── voice_leading/
│       ├── voicer.rs     # Contains #[cfg(test)] mod tests
│       ├── rules.rs      # Contains #[cfg(test)] mod tests
│       ├── styles.rs     # Contains #[cfg(test)] mod tests
│       └── suspension.rs # Contains #[cfg(test)] mod tests
└── generator/
    └── engine.rs         # Contains #[cfg(test)] mod tests
```

## Test Structure

**Suite Organization:**
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_descriptive_name() {
        // Arrange
        let input = create_test_data();

        // Act
        let result = function_under_test(input);

        // Assert
        assert_eq!(result, expected);
    }
}
```

**Patterns:**
- Each test module imports parent with `use super::*;`
- Test functions are simple and focused on single behavior
- Tests follow Arrange-Act-Assert pattern implicitly
- No test setup/teardown fixtures (tests are self-contained)

## Test Coverage

**Coverage Analysis:**
```
Files with tests (11 detected):
- src/chord.rs (29 tests)
- src/harmony/engine.rs (68 tests)
- src/harmony/scale.rs (22 tests)
- src/harmony/modes.rs (4 tests)
- src/harmony/stateful.rs (21 tests)
- src/harmony/voice_leading/voicer.rs (11 tests)
- src/harmony/voice_leading/rules.rs (11 tests)
- src/harmony/voice_leading/styles.rs (6 tests)
- src/harmony/voice_leading/suspension.rs (6 tests)
- src/generator/engine.rs (7 tests)
- src/server/protocol.rs (5 tests)
```

**Coverage by Module:**
- Core harmony logic: Extensively tested (68 tests in `harmony/engine.rs`)
- Voice leading: Well tested (34 tests across submodules)
- Chord detection: Well tested (29 tests)
- UI components: Not tested (`src/app.rs`, `src/ui.rs`, `src/piano.rs`)
- MIDI I/O: Not tested (`src/midi/` modules)

## Mocking

**Framework:** None (no mocking library used)

**Patterns:**
- Tests use real implementations, not mocks
- Pure functions tested with concrete inputs
- No dependency injection for testing
- WMIDI types used directly in tests

**Example (no mocking):**
```rust
#[test]
fn test_engine_diatonic_thirds() {
    let mut engine = HarmonyEngine::new(Key::C, HarmonyMode::DiatonicThirds);
    let result = engine.harmonize(Note::C4);
    assert_eq!(result, vec![Note::C4, Note::E4]);
}
```

## Fixtures and Factories

**Test Data:**
- Tests create data inline using simple constructors
- HashSet from array literals: `[60, 64, 67].into_iter().collect()`
- Direct enum usage: `Key::C`, `HarmonyMode::DiatonicThirds`, `Note::C4`
- No factory functions or builders

**Example:**
```rust
#[test]
fn test_c_major() {
    let notes: HashSet<u8> = [60, 64, 67].into_iter().collect();
    assert_eq!(detect_chord(&notes), Some("Cmaj".to_string()));
}
```

**Location:**
- No dedicated fixture files
- Test data created inline within each test function

## Common Patterns

**Enum Testing:**
```rust
#[test]
fn test_key_change() {
    let mut engine = HarmonyEngine::new(Key::C, HarmonyMode::DiatonicThirds);
    let result = engine.harmonize(Note::C4);
    assert_eq!(result[1], Note::E4);

    engine.set_key(Key::G);
    let result = engine.harmonize(Note::G4);
    assert_eq!(result[1], Note::B4);
}
```

**Option Testing:**
```rust
#[test]
fn test_single_note() {
    let notes: HashSet<u8> = [60].into_iter().collect();
    assert_eq!(detect_chord(&notes), None);
}
```

**Collection Testing:**
```rust
#[test]
fn test_chord_display_unknown() {
    let notes: HashSet<u8> = [60, 61].into_iter().collect();
    let display = chord_display(&notes);
    assert!(display.contains("C") && display.contains("C#"));
}
```

**State Machine Testing:**
```rust
#[test]
fn test_mode_change() {
    let mut engine = HarmonyEngine::new(Key::C, HarmonyMode::PassThrough);
    let result = engine.harmonize(Note::C4);
    assert_eq!(result.len(), 1);

    engine.set_mode(HarmonyMode::DiatonicThirds);
    let result = engine.harmonize(Note::C4);
    assert_eq!(result.len(), 2);
}
```

## Test Naming

**Convention:**
- `test_` prefix required by Rust
- Descriptive names: `test_c_major`, `test_key_change`, `test_parallel_fifths_violation`
- Pattern: `test_<scenario>` or `test_<function>_<scenario>`

**Examples:**
- `test_engine_creation` - Constructor behavior
- `test_engine_pass_through` - Mode-specific behavior
- `test_parallel_fifths_violation` - Rule validation
- `test_common_tone_preference` - Voice leading preference

## Assertions

**Equality:**
```rust
assert_eq!(result, expected);
assert_eq!(engine.key(), Key::C);
```

**Boolean:**
```rust
assert!(display.contains("C"));
assert!(!has_parallel_fifths);
```

**Vector/Collection:**
```rust
assert_eq!(result, vec![Note::C4, Note::E4]);
assert_eq!(result.len(), 2);
```

## Test Types

**Unit Tests:**
- Primary testing strategy
- Tests pure functions and stateful objects
- Examples: chord detection, harmony generation, scale operations
- Location: Inline at bottom of source files in `#[cfg(test)] mod tests`

**Integration Tests:**
- Not detected (no `tests/` directory)
- No end-to-end workflow tests

**E2E Tests:**
- Not used (GUI and MIDI I/O not tested)

## Coverage Gaps

**Untested Areas:**
- GUI components (`src/app.rs`, `src/ui.rs`, `src/piano.rs`, `src/theme/`)
- MIDI I/O (`src/midi/input.rs`, `src/midi/output.rs`, `src/midi/ports.rs`, `src/midi/web.rs`)
- Router/Server (`src/router.rs`, `src/server/session.rs`, `src/server/config.rs`)
- Humanization (`src/humanize/` modules)
- Preset management (`src/preset/`)
- Generator tick logic (only basic tests in `src/generator/engine.rs`)

**Well-Tested Areas:**
- Core harmony algorithms
- Voice leading rules and voicing
- Chord detection
- Scale operations

## Running Tests

**All tests:**
```bash
cargo test
```

**Specific module:**
```bash
cargo test harmony::engine
cargo test chord
```

**Specific test:**
```bash
cargo test test_c_major
```

**With output:**
```bash
cargo test -- --nocapture
```

**Documentation tests:**
- No doc tests detected (no `/// # Examples` sections in sampled code)

---

*Testing analysis: 2026-02-05*
