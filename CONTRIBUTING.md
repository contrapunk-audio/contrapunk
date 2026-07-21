# Contributing to Contrapunk

Welcome to Contrapunk! This guide covers development setup, architecture overview, and contribution guidelines.

## Development Setup

### Prerequisites

- **Rust stable** (via [rustup](https://rustup.rs/))
- **Tauri CLI**: `cargo install tauri-cli`
- **Node.js** and **npm** (for the SvelteKit frontend)
- **macOS**: Xcode Command Line Tools (`xcode-select --install`)
- **Linux**: ALSA development libraries (`sudo apt-get install libasound2-dev`) + WebKitGTK

### Building

Install UI dependencies once with `npm --prefix ui ci`, then build the surface you are changing:

```bash
# Native CLI
cargo build --release

# Desktop app (Tauri)
(cd src-tauri && cargo tauri dev)

# Browser/WASM
npm --prefix ui run build:wasm
npm --prefix ui run dev

# VST3/CLAP with the production UI
npm --prefix ui run build
CONTRAPUNK_PLUGIN_UI_DIR="$PWD/ui/build" \
  cargo xtask bundle contrapunk_plugin --release --features embed-ui

# Universal macOS VST3/CLAP + Logic Audio Units
CONTRAPUNK_PLUGIN_UI_DIR="$PWD/ui/build" \
  cargo xtask bundle-universal contrapunk_plugin --release --features embed-ui
./au-wrapper/build.sh
```

### Running tests

Use focused checks first:

```bash
cargo test -p contrapunk-harmony --lib
cargo test -p contrapunk-audio --lib
cargo test -p contrapunk-companion --lib
npm --prefix ui run check
```

Before submitting a cross-surface change:

```bash
cargo check --workspace --message-format=short
cargo test --workspace --lib --bins --tests --examples
cargo clippy --all-targets -- -D warnings
cargo fmt --all -- --check
npm --prefix ui run check
```

Harmony-crate doctests currently reference the top-level `contrapunk` crate; use `--lib` for focused harmony work.

### Documentation

```bash
cargo doc --all-features     # Generate docs
cargo doc --all-features --open  # Generate and open in browser
```

## Architecture overview

One Rust core feeds four distribution surfaces. A core fix should land once, but each surface has its own adapter:

| Surface | Entry point | Adapter boundary |
|---|---|---|
| Native CLI | `src/main.rs` | `src/router.rs` and `src/server/` |
| Tauri desktop | `src-tauri/` | `src-tauri/src/commands/` |
| Browser/WASM | `wasm/src/lib.rs` | `ui/src/lib/adapter/wasm.ts` |
| VST3/CLAP/AU | `plugin/src/lib.rs` | `ui/src/lib/adapter/plugin.ts` and `au-wrapper/` |

### Core crates

| Crate/path | Purpose |
|---|---|
| `crates/contrapunk-harmony/` | Scales, harmony modes, stateful voice leading, and exact NoteOn/NoteOff ownership |
| `crates/contrapunk-companion/` | Canon, counterpoint, and pattern lanes driven by transport |
| `crates/contrapunk-audio/` | Guitar onset/pitch analysis and audio input support |
| `crates/contrapunk-transport/` | Shared beat/transport state |
| `crates/contrapunk-midi/` | MIDI types and native routing support |
| `src/chain/` | Built-in audio/MIDI processing chain |
| `src/synth/` and `src/fx/` | Built-in sound and effects |
| `ui/src/lib/` | Production Svelte workspace, stores, adapters, and arrangement catalog |

### Data flow

```text
MIDI or guitar audio
  → surface adapter/router
  → HarmonyEngine immediate voices
  → Companion transport-scheduled voices
  → humanization/routing
  → owned MIDI NoteOn/NoteOff events
  → built-in audio, hardware/virtual MIDI, or DAW host
```

When adding a setting, trace it through the shared core and every supported adapter. Unsupported surfaces must report a capability gap instead of silently accepting a no-op. Parameters that change active harmony state must preserve the panic/reharmonization cleanup contract.

## Harmony Algorithm Deep Dive

The harmony engine uses a multi-stage evaluation process to determine which notes to generate. This section explains the algorithms in detail.

### 1. Scale Membership Check

When a note arrives, the engine first checks if it belongs to the current scale using `Scale::is_in_scale()`:

```rust
// Pitch class (0-11) is checked against scale intervals
let pitch_class = (note_midi - tonic_midi + 12) % 12;
scale.intervals.contains(&pitch_class)
```

This determines the harmonization path:
- **In-scale notes**: Use diatonic transposition (move by scale degrees)
- **Out-of-scale notes**: Use modal interchange or chromatic consonant intervals

### 2. Diatonic Transposition

For in-scale notes, `Scale::transpose_diatonic()` moves by scale degrees:

```
C4 + 2 degrees in C Major = E4 (major 3rd, 4 semitones)
E4 + 2 degrees in C Major = G4 (minor 3rd, 3 semitones)
```

The interval size in semitones varies based on position in the scale, which is what makes the harmony sound "diatonic" rather than mechanical.

### 3. Modal Interchange (Borrowing)

When enabled, out-of-scale notes search parallel modes for borrowing:

```
Eb4 in C Ionian -> Not in scale
                -> Search C Aeolian (has Eb)
                -> Borrow harmonization from that mode
                -> Track borrowed mode for UI display (amber highlight)
```

The borrowing range (1-5) controls how many parallel modes to search:
- Range 1: Only natural minor
- Range 2-3: Adds Dorian, Phrygian
- Range 4-5: Adds harmonic/melodic minor modes

### 4. Windowing Algorithms for Stateful Modes

Modes 6 (Contrary Motion) and 7 (Strict Counterpoint) use **sliding window history** via `VecDeque` for context-aware harmony generation.

#### Contrary Motion (Mode 6)

Tracks `last_melody` and `last_harmony` to determine direction:

```
Melody: C4 -> E4 (ascending +4 semitones)
Harmony: A3 -> G3 (descending -2 semitones, contrary motion)

When melody repeats:
  repeat_direction alternates (-1, +1, -1, ...)
  Creates oblique motion (melody static, harmony moves)
```

#### Strict Counterpoint (Mode 7) - Window Buffers

The `CounterpointState` maintains multiple sliding windows:

| Buffer | Type | Size | Purpose |
|--------|------|------|---------|
| `interval_history` | `VecDeque<i8>` | 4 | Track last 4 interval types for variety |
| `melody_contour` | `VecDeque<MelodicDirection>` | 3 | Track melodic direction trend |
| `harmony_range_low/high` | `Option<u8>` | - | Track harmony voice range |

**Interval History Window** (size 4):
```rust
// Push new interval, evict oldest if at capacity
fn push_interval(&mut self, interval_class: i8) {
    if self.interval_history.len() >= 4 {
        self.interval_history.pop_front();  // VecDeque O(1) pop
    }
    self.interval_history.push_back(interval_class);
}

// Penalize overused intervals (3+ of same in last 4)
fn is_interval_overused(&self, interval_class: i8) -> bool {
    self.count_recent_interval(interval_class) >= 3
}
```

**Melodic Contour Window** (size 3):
```rust
// Determine dominant trend for contrary motion preference
fn dominant_contour(&self) -> Option<MelodicDirection> {
    // Returns Ascending/Descending if majority, None if mixed
    let ascending_count = self.melody_contour.iter()
        .filter(|d| **d == MelodicDirection::Ascending).count();
    // Majority = (len + 1) / 2
}
```

### 5. Candidate Scoring System

For each candidate harmony note, the counterpoint mode calculates a score:

| Factor | Score | Condition |
|--------|-------|-----------|
| Parallel perfect 5ths/octaves | -100 (reject) | Previous and new interval both P5 or P8 |
| Static harmony on repeated melody | -100 (reject) | Melody same, harmony same |
| Different note from previous | +3 | Harmony moved |
| Stepwise motion (1-2 semitones) | +4 | Small step preferred |
| Small leap (3-4 semitones) | +2 | Moderate leap OK |
| Interval overused (3+ in last 4) | -3 | Variety penalty |
| Fresh interval (not in history) | +2 | Variety bonus |
| Contrary motion to melody trend | +3 | Move opposite to dominant contour |
| Parallel motion to melody trend | -1 | Slight penalty |
| Large leap when range narrow | +2 | Encourage range expansion |
| 3rds and 6ths (vs 4ths/5ths) | +1 | Traditional preference |

The candidate with the highest positive score wins. Negative scores (-100) are hard rejections.

### 6. Voice Position and Chaining

Multi-voice output uses chained harmony:

```
4-voice with position=2 (tenor):

Voice 0 [Soprano]  <-- harmony_of(harmony_of(input))
Voice 1 [Alto]     <-- harmony_of(input)
Voice 2 [Tenor]    <-- USER INPUT (voice_position)
Voice 3 [Bass]     <-- harmony_of(input, below)
```

Each voice pair gets independent state (for stateful modes like Counterpoint).

### 7. Octave Mode Post-Processing

After harmony generation, octave transformations are applied:

| Mode | Effect |
|------|--------|
| None | No modification |
| Spread | Each voice +1 octave from previous |
| BassTrebleSplit | Below melody -1 oct, above +1 oct |
| Mirror | Duplicate each harmony at +1 and -1 octave (tripling) |

## Making Changes

### Workflow

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/your-feature`)
3. Make changes with tests
4. Run `cargo test` and `cargo clippy`
5. Submit a pull request

### Before submitting

- [ ] Focused tests for the changed crate pass
- [ ] `cargo check --workspace --message-format=short` passes
- [ ] `cargo clippy --all-targets -- -D warnings` passes
- [ ] `cargo fmt --all -- --check` passes
- [ ] `npm --prefix ui run check` passes for UI/adapter changes
- [ ] Every emitted NoteOn has deterministic NoteOff/reset coverage
- [ ] New tunables have real controls on supported surfaces and capability gates elsewhere
- [ ] Relevant user/developer documentation is updated

### Commit Style

Use conventional commits:

```
feat(harmony): add lydian dominant scale mode
fix(humanize): correct swing delay calculation
docs(readme): update installation instructions
refactor(midi): simplify port selection logic
test(counterpoint): add parallel fifth rejection test
```

## Code Style

### Formatting

Follow `rustfmt` defaults. Run `cargo fmt` before committing.

### Documentation

Add doc comments (`///`) to all public items:

```rust
/// Transposes a note by N scale degrees (diatonic transposition).
///
/// # Arguments
///
/// * `note` - The MIDI note to transpose
/// * `degrees` - Number of scale degrees (positive = up, negative = down)
///
/// # Returns
///
/// The transposed note, or `None` if out of MIDI range.
///
/// # Example
///
/// ```ignore
/// let result = scale.transpose_diatonic(Note::C4, 2);
/// assert_eq!(result, Some(Note::E4));
/// ```
pub fn transpose_diatonic(&self, note: Note, degrees: i8) -> Option<Note>
```

### Error Handling

- Use `Result<T, E>` for fallible operations
- Use `Option<Note>` for transpositions that might go out of range
- Avoid `unwrap()` in production code; use `expect()` with context if needed

### Testing

- Unit tests go in the same file (`#[cfg(test)] mod tests`)
- Integration tests go in `tests/`
- Use descriptive test names: `test_contrary_motion_melody_repeats`

## Questions?

Open an issue on GitHub for questions or suggestions. Contributions are welcome!
