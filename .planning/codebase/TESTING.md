# Testing Patterns

**Analysis Date:** 2026-02-04

## Test Framework

**Runner:**
- Rust built-in test framework (no external test runner)
- Config: None required (uses `cargo test` defaults)
- Documentation tests supported via `cargo test --doc`

**Assertion Library:**
- Standard library: `assert!`, `assert_eq!`, `assert_ne!`
- No external assertion libraries (like `pretty_assertions`) detected

**Run Commands:**
```bash
cargo test              # Run all tests
cargo test --lib        # Library tests only
cargo test <name>       # Run specific test by name
```

## Test File Organization

**Location:**
- Co-located: Tests in same file as implementation using `#[cfg(test)] mod tests { ... }`
- Pattern: Every module with logic has inline tests at bottom of file
- No separate `tests/` directory detected (all unit tests)

**Naming:**
- Test modules always named `tests`
- Test functions use descriptive snake_case with `test_` prefix
- Pattern: `test_<what_is_being_tested>_<expected_outcome>`
- Examples: `test_c_major_scale_degrees`, `test_harmonize_smart_out_of_key`

**Structure:**
```
src/
├── chord.rs               # Implementation + #[cfg(test)] mod tests
├── harmony/
│   ├── engine.rs          # Implementation + 1523 lines (includes ~600 lines of tests)
│   ├── scale.rs           # Implementation + extensive tests
│   └── modes.rs           # Implementation + tests
```

## Test Structure

**Suite Organization:**
```rust
#[cfg(test)]
mod tests {
    use super::*;  // Import all from parent module

    #[test]
    fn test_specific_behavior() {
        // Arrange
        let scale = Scale::major(0);

        // Act
        let result = scale.transpose_diatonic(Note::C4, 2);

        // Assert
        assert_eq!(result, Some(Note::E4));
    }
}
```

**Patterns:**
- Arrange-Act-Assert structure (implicit, not labeled)
- Each test focuses on one behavior
- Descriptive test names eliminate need for comments
- Related tests grouped by functionality (e.g., all chord detection tests together)

**Test Coverage Groups:**
- Happy path tests first
- Edge cases (empty input, out of range)
- Error conditions
- Regression tests for bugs
- Integration tests between modules

## Mocking

**Framework:** None (no mocking library used)

**Patterns:**
- Rust's type system eliminates many mocking needs
- Trait objects not used extensively (concrete types preferred)
- Test doubles created manually when needed:
  ```rust
  // Example: Testing with controlled random behavior not observed
  // Code uses rand::thread_rng() directly (not injectable)
  ```

**What to Mock:**
- External MIDI I/O not mocked in unit tests (tested via integration)
- Random number generation not mocked (acceptable for these tests)

**What NOT to Mock:**
- Core domain logic (Scale, HarmonyEngine) - tested with real implementations
- Lightweight value objects (Note, Key, Mode enums)

## Fixtures and Factories

**Test Data:**
```rust
// Common pattern: Inline test data creation
#[test]
fn test_c_major() {
    let notes: HashSet<u8> = [60, 64, 67].into_iter().collect();
    assert_eq!(detect_chord(&notes), Some("Cmaj".to_string()));
}
```

**Location:**
- Test data created inline within each test
- No separate fixtures directory
- Constants used for reusable test values (e.g., `NOTE_NAMES`)

**Patterns:**
- Builder pattern for complex objects: `HarmonyEngine::with_voices(Key::C, mode, 4)`
- Direct construction for simple cases: `Scale::major(0)`
- Arrays converted to collections: `[60, 64, 67].into_iter().collect()`

## Coverage

**Requirements:** None enforced (no coverage tooling detected)

**View Coverage:**
```bash
# Using cargo-tarpaulin (if installed)
cargo tarpaulin --out Html

# Using cargo-llvm-cov (if installed)
cargo llvm-cov --html
```

**Actual Coverage:**
- Core modules well-tested: `chord.rs`, `harmony/engine.rs`, `harmony/scale.rs`
- Tests follow implementation (inline `#[cfg(test)]` modules)
- Integration scenarios tested via engine-level tests
- UI code (`app.rs`, `ui.rs`) not unit tested (GUI interaction)

## Test Types

**Unit Tests:**
- Scope: Individual functions and methods
- Approach: Test public API of each module
- Examples in `src/chord.rs`:
  - `test_c_major`: Basic chord detection
  - `test_slash_chord_c_over_e`: Slash chord inversion
  - `test_roman_numeral_iv`: Roman numeral conversion
  - `test_chord_display_with_analysis`: Complex formatting

**Integration Tests:**
- Scope: Multiple modules working together
- Approach: Test HarmonyEngine with various configurations
- Examples in `src/harmony/engine.rs`:
  - `test_key_change`: Engine + Scale interaction
  - `test_note_on_off_tracking`: Engine + active note management
  - `test_chained_harmonies_with_thirds`: Multi-voice generation
  - `test_vl_before_octave_mode`: Voice leading + octave transformations

**E2E Tests:**
- Framework: Not used
- No end-to-end tests detected (would require MIDI hardware/virtual ports)

## Common Patterns

**Async Testing:**
Not applicable (no async code in tested modules)

**Error Testing:**
```rust
#[test]
fn test_single_note() {
    let notes: HashSet<u8> = [60].into_iter().collect();
    assert_eq!(detect_chord(&notes), None);
}

#[test]
fn test_transpose_chromatic() {
    let scale = Scale::major(0);
    // In-range
    assert_eq!(scale.transpose_chromatic(Note::C4, 4), Some(Note::E4));
    // Out of range test would check None return
}
```

**State Testing:**
```rust
#[test]
fn test_stateful_reset_on_key_change() {
    let mut engine = HarmonyEngine::new(Key::C, HarmonyMode::ContraryMotion);

    // Build up state
    engine.harmonize(Note::C4);
    engine.harmonize(Note::E4);

    // Change key - state should reset
    engine.set_key(Key::G);

    // Verify reset behavior
    let result = engine.harmonize(Note::G4);
    assert_eq!(result.len(), 2);
    assert_eq!(result[1], Note::B4);  // First note behavior expected
}
```

**Parameterized Testing:**
```rust
#[test]
fn test_scale_new_with_each_mode() {
    for &mode in ScaleMode::all() {
        let scale = Scale::new(0, mode);
        assert_eq!(scale.degree_of(Note::C4), Some(0),
            "Tonic should be degree 0 for {:?}", mode);
    }
}

#[test]
fn test_vl_works_with_all_modes() {
    let modes = HarmonyMode::all();
    for &mode in modes {
        let mut engine = HarmonyEngine::with_voices(Key::C, mode, 3);
        engine.set_voice_leading_enabled(true);
        let result = engine.harmonize(Note::C4);
        assert_eq!(result[0], Note::C4, "Melody unchanged for mode {:?}", mode);
    }
}
```

**Regression Testing:**
```rust
// Example: Testing that voice leading doesn't modify melody (regression guard)
#[test]
fn test_vl_melody_never_modified() {
    let mut engine = HarmonyEngine::with_voices(Key::C, HarmonyMode::DiatonicThirds, 4);
    engine.set_voice_leading_enabled(true);

    for note in [Note::C4, Note::D4, Note::E4, Note::F4, Note::G4] {
        let result = engine.harmonize(note);
        assert_eq!(result[0], note, "Melody must never be modified by VL");
    }
}
```

**Comprehensive Test Suites:**
- `src/chord.rs`: 33 tests covering chord detection, slash chords, extended harmonies, roman numerals
- `src/harmony/engine.rs`: 80+ tests covering all modes, voice counts, octave modes, voice leading
- `src/harmony/scale.rs`: 40+ tests covering transposition, modal interchange, exotic scales

**Test Naming Conventions:**
- Format: `test_<component>_<scenario>_<outcome>`
- Examples:
  - `test_c_major` (simple case)
  - `test_harmonize_smart_out_of_key` (specific scenario)
  - `test_borrowing_range_clamp` (validation behavior)
  - `test_mirror_note_off_releases_all_duplicates` (complex interaction)

**Assertion Patterns:**
```rust
// Exact equality
assert_eq!(result, expected);

// Inequality
assert_ne!(result1[1], result1[2], "Chained harmonies should differ");

// Boolean conditions
assert!(result.is_some(), "Should find harmony via interchange");
assert!(scale.is_in_scale(Note::C4));

// Range checks
assert!([3, 4, 5, 7, 8, 9].contains(&interval),
    "Expected consonant interval, got {} semitones", interval);
```

## Test Quality Observations

**Strengths:**
- Comprehensive coverage of core harmony logic
- Tests document expected behavior clearly
- Edge cases well-covered (empty input, out-of-range MIDI)
- Regression prevention (state management, voice leading invariants)
- Parameterized tests ensure consistency across enums

**Gaps:**
- No tests for UI components (`ui.rs`, `app.rs`, `piano.rs`)
- Server/router code minimally tested (protocol parsing has tests)
- Random behavior not deterministically tested (acceptable trade-off)
- No performance/benchmark tests detected

---

*Testing analysis: 2026-02-04*
