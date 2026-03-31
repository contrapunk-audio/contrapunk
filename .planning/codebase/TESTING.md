# Testing Patterns

**Analysis Date:** 2026-03-31

## Test Framework

**Rust (primary test suite):**
- Runner: built-in `cargo test`
- No external test framework dependencies
- Assertion library: built-in `assert!`, `assert_eq!`, `assert_ne!`
- Config: `Cargo.toml` workspace at `/Users/vibhavbobade/go/src/github.com/waveywaves/contrapunk/Cargo.toml`

**TypeScript/Svelte:**
- No test framework installed (no `jest.config.*`, `vitest.config.*`, or test files in `ui/src/`)
- `svelte-check` runs type checking only, not behavioral tests
- Run commands:
```bash
cargo test                          # Run all Rust tests
cargo test harmony                  # Run harmony module tests only
cargo test --lib                    # Run lib tests only (excludes bin)
npm run check                       # TypeScript type checking only (not tests)
```

## Test File Organization

**Location:** Co-located with source in Rust — all tests live in `#[cfg(test)] mod tests` blocks at the bottom of each `.rs` file.

**Coverage across modules:**
- `src/harmony/engine.rs` — 55+ tests covering all harmony modes, key changes, voice leading, interchange, octave modes
- `src/harmony/scale.rs` — 25+ tests covering scale degrees, diatonic transposition, all scale families
- `src/harmony/stateful.rs` — 25+ tests covering ContraryMotion and StrictCounterpoint state machines
- `src/harmony/modes.rs` — Tests for individual mode algorithms
- `src/harmony/voice_leading/rules.rs` — 11 tests covering parallel fifths/octaves, voice crossing, spacing
- `src/harmony/voice_leading/voicer.rs` — 11 tests covering chord revoicing and register assignment
- `src/harmony/voice_leading/styles.rs` — 6 tests for style configuration
- `src/harmony/voice_leading/suspension.rs` — 6 tests for Palestrina suspension logic
- `src/audio/detectors.rs` — 20+ tests covering BACF, AMDF, Goertzel pitch detectors
- `src/audio/guitar.rs` — 15+ tests covering pitch/note conversion, string identification, calibration

**Naming:** `test_<behavior_description>` in snake_case.
Examples: `test_engine_creation`, `bacf_detect_440hz`, `test_parallel_fifths_detected`, `test_common_tone_retention`

## Test Structure

**Standard Rust unit test block:**
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine_diatonic_thirds() {
        let mut engine = HarmonyEngine::new(Key::C, HarmonyMode::DiatonicThirds);
        let result = engine.harmonize(Note::C4);
        assert_eq!(result, vec![Note::C4, Note::E4]);
    }
}
```

**Helper functions in test modules:**
Test modules define private helper functions to generate test input data. These are not `#[test]` annotated — they are plain `fn` called by tests.

**Audio test helpers (from `src/audio/detectors.rs`):**
```rust
fn sine_wave(freq: f64, sample_rate: usize, duration_secs: f64, amplitude: f32) -> Vec<f32> {
    let num_samples = (sample_rate as f64 * duration_secs) as usize;
    (0..num_samples)
        .map(|i| {
            let t = i as f64 / sample_rate as f64;
            (amplitude as f64 * (2.0 * PI * freq * t).sin()) as f32
        })
        .collect()
}

fn silence(num_samples: usize) -> Vec<f32> {
    vec![0.0; num_samples]
}

fn noise(num_samples: usize, amplitude: f32) -> Vec<f32> {
    // LCG pseudo-random for deterministic tests — no rand dependency needed
    let mut state: u64 = 0xDEAD_BEEF_CAFE_BABE;
    ...
}
```

**Voice leading test helpers (from `src/harmony/voice_leading/voicer.rs`):**
```rust
fn default_registers_3() -> Vec<VoiceRegister> {
    vec![
        VoiceRegister::Soprano, // melody placeholder
        VoiceRegister::Soprano,
        VoiceRegister::Alto,
        VoiceRegister::Tenor,
    ]
}
```

**Custom assertion helpers (from `src/audio/detectors.rs`):**
```rust
const FREQ_TOLERANCE_PERCENT: f64 = 3.0;

fn assert_freq_close(detected: f32, expected: f64, label: &str) {
    let error_pct = ((detected as f64 - expected) / expected).abs() * 100.0;
    assert!(
        error_pct < FREQ_TOLERANCE_PERCENT,
        "{}: detected {:.1} Hz, expected {:.1} Hz (error {:.2}%)",
        label, detected, expected, error_pct
    );
}
```

## Mocking

**Framework:** None — no mocking library used.

**Approach:**
- Tests use real struct instances constructed with default or test-specific configurations
- Deterministic pseudo-random data generated with LCG (linear congruential generator) inline — avoids `rand` crate dependency in test code
- WASM-specific code paths are isolated via `#[cfg(not(target_arch = "wasm32"))]` feature flags; tests run on native target only

**What to mock:**
- Audio input data: replaced with `sine_wave()`, `silence()`, `noise()` helpers that produce `Vec<f32>`
- MIDI events: constructed directly as `Note::C4`, `Note::E4` (using `wmidi::Note` enum values)
- Time: passed as `f64` seconds parameter to functions that need timestamps

**What NOT to mock:**
- The harmony engine itself — tests verify full end-to-end harmonization behavior
- Scale/chord logic — tested directly, not abstracted behind interfaces

## Fixtures and Factories

**Test Data:**
No separate fixture files. All test data is constructed inline or via local helper functions within each `mod tests` block.

**Pattern for audio signal tests:**
```rust
// Provide frequency, sample rate, duration in seconds, amplitude
let samples = sine_wave(440.0, 44100, 0.1, 0.8);
let mut det = BacfDetector::new();
let result = det.detect(&samples, 44100);
```

**Pattern for harmony engine tests:**
```rust
// Construct with specific musical parameters
let mut engine = HarmonyEngine::new(Key::C, HarmonyMode::DiatonicThirds);
let result = engine.harmonize(Note::C4);
assert_eq!(result, vec![Note::C4, Note::E4]);
```

**Pattern for voice leading tests:**
```rust
// Represent MIDI note numbers as u8 vectors
let prev = vec![72u8, 60, 53]; // melody=C5, soprano=C4, alto=F3
let curr = vec![74u8, 62, 55]; // melody=D5, soprano=D4, alto=G3
let result = check_parallel_fifths(&prev, &curr);
assert_eq!(result, vec![(1, 2)]);
```

**Location:** Inline within `mod tests` blocks in each `.rs` file. No separate fixtures directory.

## Coverage

**Requirements:** Not enforced — no coverage tooling configured.

**View Coverage:**
```bash
cargo test 2>&1 | grep "test result"   # Summary of pass/fail counts
# For detailed coverage, install cargo-tarpaulin:
cargo install cargo-tarpaulin
cargo tarpaulin --out Html
```

## Test Types

**Unit Tests (primary):**
- Scope: individual functions and methods in isolation
- All Rust tests are unit tests within the module they test
- Tests directly construct the struct being tested and assert on outputs

**Integration Tests:**
- No `tests/` directory at crate root — no integration test suite
- The Tauri/WASM adapter boundary is not tested (no mock Tauri or mock WASM environment)

**E2E Tests:**
- Not used. No Playwright, Cypress, or similar tooling configured.

**TypeScript/Svelte Tests:**
- None. The `ui/` frontend has no test suite. `svelte-check` provides type safety only.

## Common Patterns

**Stateful algorithm testing (harmony engine):**
Tests verify stateful behavior by calling harmonize() multiple times and checking accumulated state:
```rust
#[test]
fn test_key_change() {
    let mut engine = HarmonyEngine::new(Key::C, HarmonyMode::DiatonicThirds);
    let result = engine.harmonize(Note::C4);
    assert_eq!(result[1], Note::E4);   // In C major

    engine.set_key(Key::G);
    let result = engine.harmonize(Note::G4);
    assert_eq!(result[1], Note::B4);   // In G major
}
```

**Boundary / edge case testing:**
```rust
#[test]
fn bacf_silence_returns_none() {
    let samples = silence(4410);
    let mut det = BacfDetector::new();
    let result = det.detect(&samples, 44100);
    assert!(result.is_none(), "Should return None for silence");
}

#[test]
fn bacf_very_low_amplitude() {
    let samples = sine_wave(440.0, 44100, 0.1, 0.0001);
    // Very quiet signal should not be detected
}
```

**Determinism testing (for stochastic algorithms):**
```rust
#[test]
fn test_determinism_100_times() {
    let pcs = vec![4, 7];   // E, G pitch classes
    let prev = vec![72u8, 64, 67];
    let first = revoice_chord(&pcs, Some(&prev), &registers, &rules, None);
    for _ in 0..100 {
        let result = revoice_chord(&pcs, Some(&prev), &registers, &rules, None);
        assert_eq!(result, first, "Determinism violated!");
    }
}
```

**Round-trip testing:**
```rust
#[test]
fn test_profile_json_roundtrip() {
    let profile = GuitarCalibrationProfile::default();
    let json = profile.to_json().unwrap();
    let restored = GuitarCalibrationProfile::from_json(&json).unwrap();
    assert_eq!(profile, restored);
}

#[test]
fn test_roundtrip_note_name() {
    for midi in 21..=108 {
        let name = midi_to_note_name(midi);
        let back = note_name_to_midi(&name).unwrap();
        assert_eq!(back, midi);
    }
}
```

**Multi-detector consensus testing:**
```rust
#[test]
fn all_detectors_agree_on_440hz() {
    let samples = sine_wave(440.0, 44100, 0.1, 0.8);
    // Test all three detectors (BACF, AMDF, Goertzel) on same signal
    // Assert all return results within tolerance
}
```

## Testing Gaps

**No TypeScript/Svelte tests:** The entire `ui/` frontend is untested beyond type checking. Adapter implementations (`TauriAdapter`, `WasmAdapter`), store logic (optimistic updates, persistence, rollback), and component behavior have no automated tests.

**No Tauri IPC tests:** The `src-tauri/` command handlers are not tested in isolation.

**No integration tests:** The Rust lib ↔ WASM bridge is not tested end-to-end.

---

*Testing analysis: 2026-03-31*
