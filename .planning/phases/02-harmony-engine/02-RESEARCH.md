# Phase 2: Harmony Engine - Research

**Researched:** 2026-01-28
**Domain:** Music theory algorithms for diatonic harmony generation in Rust
**Confidence:** HIGH

## Summary

This phase implements the core harmony generation engine that transforms incoming MIDI notes into harmonized output based on a selected key and mode (1-7). The research focuses on two domains: (1) music theory algorithms for computing diatonic intervals, and (2) Rust implementation patterns that integrate with the existing Phase 1 MIDI foundation.

The recommended approach uses the **wmidi** crate for MIDI note manipulation (already compatible with Phase 1's raw byte handling) combined with a **hand-rolled scale/interval engine** tailored to the specific requirements. While Rust music theory libraries exist (rust-music-theory, kord), they add complexity without matching our exact needs. The harmony algorithms are straightforward enough (< 200 lines) that a purpose-built implementation is cleaner.

The architecture follows a **stateless transformation model**: each incoming MIDI message is processed independently through a `HarmonyEngine` that holds the current key and mode configuration. This avoids complex state management while enabling real-time key/mode switching during playback.

**Primary recommendation:** Use wmidi for Note ↔ u8 conversion and the `step()` method for transposition. Build a custom `Scale` struct holding scale degree offsets, and a `HarmonyEngine` that selects the appropriate harmony algorithm based on mode number.

## Standard Stack

The established libraries/tools for this domain:

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| wmidi | 4.0.x | MIDI Note type with `step()` transposition | Already used in Phase 1 pattern; has exact API needed (Note enum, step method, u8 conversion) |
| (custom) | - | Scale/mode definitions | Specific to our 7 modes; ~50 lines of lookup tables |
| (custom) | - | HarmonyEngine | Routes to mode-specific algorithms; ~150 lines |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| rand | 0.8.x | Random number generation | Modes 4-5 need random diatonic interval selection |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Custom scale engine | rust-music-theory 0.3 | Heavier dependency (adds chord parsing, etc. we don't need); API doesn't match our exact needs (scale degree offsets vs. note lists) |
| Custom scale engine | kord | WASM-focused, more complex; overkill for our simple scale math |
| Custom scale engine | music-note | Less mature; similar feature gap |

**Installation:**
```bash
cargo add wmidi rand
```

**Cargo.toml additions:**
```toml
[dependencies]
wmidi = "4.0"
rand = "0.8"
```

## Architecture Patterns

### Recommended Project Structure
```
src/
├── main.rs              # Entry point (unchanged from Phase 1)
├── midi/                # Phase 1 MIDI I/O (unchanged)
│   ├── mod.rs
│   ├── ports.rs
│   ├── input.rs
│   └── output.rs
├── router.rs            # Updated: routes through HarmonyEngine
├── harmony/             # NEW: Phase 2 harmony engine
│   ├── mod.rs           # Re-exports
│   ├── engine.rs        # HarmonyEngine struct
│   ├── scale.rs         # Scale definitions and lookup
│   ├── modes.rs         # Mode-specific harmony algorithms
│   └── config.rs        # Key/Mode configuration types
└── state.rs             # NEW: Shared state for key/mode selection
```

### Pattern 1: Stateless Message Transformation
**What:** Each MIDI message is transformed independently; HarmonyEngine doesn't track note history.
**When to use:** Modes 1-5 (pass-through, thirds, fourths, random intervals).
**Example:**
```rust
// Source: Standard functional transformation pattern
use wmidi::Note;

pub struct HarmonyEngine {
    key: Key,
    mode: HarmonyMode,
    scale: Scale,  // Precomputed scale degrees for current key
}

impl HarmonyEngine {
    /// Transform a single note based on current mode
    pub fn harmonize(&self, input: Note) -> Vec<Note> {
        match self.mode {
            HarmonyMode::PassThrough => vec![input],
            HarmonyMode::DiatonicThirds => self.add_diatonic_third(input),
            HarmonyMode::DiatonicFourths => self.add_diatonic_fourth(input),
            // ...
        }
    }

    fn add_diatonic_third(&self, root: Note) -> Vec<Note> {
        let harmony = self.scale.transpose_diatonic(root, 2); // 2 scale degrees up
        vec![root, harmony]
    }
}
```

### Pattern 2: Scale Degree Lookup Table
**What:** Pre-compute scale note offsets for the current key; use for diatonic transposition.
**When to use:** All modes that need diatonic (in-key) intervals.
**Example:**
```rust
// Source: Standard music theory / diatonic harmony algorithms
/// Semitone offsets for each mode (from tonic)
/// Index = scale degree (0-6), Value = semitones from tonic
const IONIAN_OFFSETS: [u8; 7] = [0, 2, 4, 5, 7, 9, 11];    // Major scale
const DORIAN_OFFSETS: [u8; 7] = [0, 2, 3, 5, 7, 9, 10];
const PHRYGIAN_OFFSETS: [u8; 7] = [0, 1, 3, 5, 7, 8, 10];
const LYDIAN_OFFSETS: [u8; 7] = [0, 2, 4, 6, 7, 9, 11];
const MIXOLYDIAN_OFFSETS: [u8; 7] = [0, 2, 4, 5, 7, 9, 10];
const AEOLIAN_OFFSETS: [u8; 7] = [0, 2, 3, 5, 7, 8, 10];   // Natural minor
const LOCRIAN_OFFSETS: [u8; 7] = [0, 1, 3, 5, 6, 8, 10];

pub struct Scale {
    tonic: u8,           // MIDI note number of tonic (0-11 for pitch class)
    offsets: [u8; 7],    // Semitone offsets from tonic
}

impl Scale {
    /// Find the scale degree (0-6) for a given MIDI note
    pub fn degree_of(&self, note: Note) -> Option<usize> {
        let pitch_class = u8::from(note) % 12;
        let relative = (pitch_class + 12 - self.tonic) % 12;
        self.offsets.iter().position(|&o| o == relative)
    }

    /// Transpose a note by N scale degrees (diatonic transposition)
    pub fn transpose_diatonic(&self, note: Note, degrees: i8) -> Option<Note> {
        let current_degree = self.degree_of(note)?;
        let note_midi = u8::from(note);
        let octave_shift = (current_degree as i8 + degrees) / 7;
        let new_degree = ((current_degree as i8 + degrees) % 7 + 7) % 7;

        let current_offset = self.offsets[current_degree];
        let new_offset = self.offsets[new_degree as usize];
        let semitone_diff = (new_offset as i8 - current_offset as i8) + (octave_shift * 12);

        note.step(semitone_diff).ok()
    }
}
```

### Pattern 3: Contrary Motion Tracking
**What:** Track melody direction and generate harmony in opposite direction.
**When to use:** Mode 6 (Contrary Motion).
**Example:**
```rust
// Source: Music theory - contrary motion definition
pub struct ContraryMotionState {
    last_note: Option<Note>,
    last_harmony: Option<Note>,
}

impl ContraryMotionState {
    pub fn process(&mut self, scale: &Scale, input: Note) -> Vec<Note> {
        let harmony = match self.last_note {
            None => {
                // First note: start harmony a third below
                scale.transpose_diatonic(input, -2).unwrap_or(input)
            }
            Some(prev) => {
                let melody_direction = u8::from(input) as i8 - u8::from(prev) as i8;
                let last_harm = self.last_harmony.unwrap_or(input);

                if melody_direction > 0 {
                    // Melody went up, harmony goes down
                    scale.transpose_diatonic(last_harm, -1).unwrap_or(last_harm)
                } else if melody_direction < 0 {
                    // Melody went down, harmony goes up
                    scale.transpose_diatonic(last_harm, 1).unwrap_or(last_harm)
                } else {
                    // Melody repeated, harmony stays
                    last_harm
                }
            }
        };

        self.last_note = Some(input);
        self.last_harmony = Some(harmony);
        vec![input, harmony]
    }
}
```

### Pattern 4: Voice Leading Rules (Simplified Counterpoint)
**What:** Apply basic counterpoint rules to avoid parallel fifths/octaves.
**When to use:** Mode 7 (Strict Counterpoint).
**Example:**
```rust
// Source: Traditional voice leading rules
pub struct CounterpointState {
    last_melody: Option<Note>,
    last_harmony: Option<Note>,
}

impl CounterpointState {
    pub fn process(&mut self, scale: &Scale, input: Note) -> Vec<Note> {
        // Try intervals in order of preference: 3rd, 6th, 4th, 5th
        let preferred_intervals = [-2, -5, -3, -4, 2, 5, 3, 4]; // scale degrees

        let harmony = preferred_intervals.iter()
            .filter_map(|&interval| {
                let candidate = scale.transpose_diatonic(input, interval)?;

                // Check for parallel fifths/octaves with previous
                if let (Some(prev_m), Some(prev_h)) = (self.last_melody, self.last_harmony) {
                    let prev_interval = (u8::from(prev_m) as i8 - u8::from(prev_h) as i8).abs() % 12;
                    let new_interval = (u8::from(input) as i8 - u8::from(candidate) as i8).abs() % 12;

                    // Avoid parallel perfect intervals (unison, 5th, octave)
                    if (prev_interval == 0 || prev_interval == 7) && prev_interval == new_interval {
                        return None; // Parallel perfect - skip
                    }
                }
                Some(candidate)
            })
            .next()
            .unwrap_or(input);

        self.last_melody = Some(input);
        self.last_harmony = Some(harmony);
        vec![input, harmony]
    }
}
```

### Anti-Patterns to Avoid
- **Global mutable state for key/mode:** Use explicit state passing or Arc<RwLock> if shared across threads. Don't use static mutable.
- **Allocating in hot path:** Pre-allocate Vec for harmonized notes; avoid creating new Vecs per message.
- **Blocking on key/mode changes:** Use try_lock or atomic reads for configuration to avoid blocking MIDI callback.
- **Over-engineering mode selection:** Modes 1-7 are simple - use a match statement, not a complex strategy pattern.

## Don't Hand-Roll

Problems that look simple but have existing solutions:

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| MIDI Note ↔ number | Manual u8 casting | wmidi::Note | Handles octave wrapping, out-of-range errors, named variants |
| Note transposition | `note + semitones` arithmetic | wmidi::Note::step() | Handles bounds checking, returns Result for invalid transposition |
| Random selection | Manual rand implementation | rand crate | Proper randomness, thread-safe Rng |
| MIDI message parsing | Manual byte extraction | wmidi::MidiMessage | Handles running status, channel extraction, all message types |

**Key insight:** Scale/interval math is specific enough to our requirements that custom code is cleaner than adapting a general library. But MIDI fundamentals (Note type, message parsing) benefit from wmidi's battle-tested implementation.

## Common Pitfalls

### Pitfall 1: Note Out of Range on Transposition
**What goes wrong:** Transposing a high note up or low note down causes panic or silent failure.
**Why it happens:** MIDI notes are 0-127; wmidi::Note::step() returns Err for invalid results.
**How to avoid:** Always handle the Result from step(). Either clamp to valid range or skip harmonization.
**Warning signs:** Harmony suddenly stops on high/low notes.
```rust
// Good: Handle out-of-range gracefully
let harmony = note.step(interval).unwrap_or(note);

// Better: Return original note if harmony would be out of range
match note.step(interval) {
    Ok(h) => vec![note, h],
    Err(_) => vec![note], // Just play original
}
```

### Pitfall 2: Chromatic vs. Diatonic Confusion
**What goes wrong:** Harmony sounds wrong because intervals are chromatic (fixed semitones) instead of diatonic (scale-aware).
**Why it happens:** Using `step(4)` for "a third" gives major third always, but diatonic third varies (M3 or m3 depending on scale position).
**How to avoid:** Build scale degree lookup; transpose by degrees, not semitones.
**Warning signs:** Harmony sounds "too perfect" or "out of key."

### Pitfall 3: Note-Off Not Tracking Note-On Harmony
**What goes wrong:** Notes get stuck because Note-Off doesn't release the same harmony note that Note-On created.
**Why it happens:** Harmony computation is stateless but Note-Off must match Note-On.
**How to avoid:** Either (a) use same algorithm for Note-Off (deterministic), or (b) track active note mappings.
**Warning signs:** Stuck notes, especially after key/mode change.
```rust
// Solution A: Deterministic - recompute harmony for Note-Off
fn handle_note_off(&self, note: Note) -> Vec<Note> {
    // Same computation as Note-On ensures matching
    self.harmonize(note)
}

// Solution B: Track mappings (needed if random modes)
struct NoteTracker {
    active: HashMap<Note, Vec<Note>>,  // melody -> harmonies
}
```

### Pitfall 4: Race Condition on Key/Mode Change
**What goes wrong:** Harmony uses wrong key/mode during transition; inconsistent output.
**Why it happens:** Key/mode updated from UI thread while MIDI processes on callback thread.
**How to avoid:** Use atomic types or RwLock with try_read for non-blocking access.
**Warning signs:** Occasional wrong-sounding harmony during mode switching.

### Pitfall 5: Forgetting Running Status in Raw MIDI
**What goes wrong:** Note-On bytes misinterpreted when status byte omitted.
**Why it happens:** MIDI running status allows omitting repeated status bytes; raw byte handling may miss this.
**How to avoid:** Parse with wmidi::MidiMessage which handles running status.
**Warning signs:** Random note behavior, especially with fast note sequences.

## Code Examples

Verified patterns from official sources:

### wmidi Note Transposition
```rust
// Source: https://docs.rs/wmidi/latest/wmidi/enum.Note.html
use wmidi::Note;

// Build a chord using step() method
fn minor_chord(root: Note) -> Result<[Note; 3], wmidi::Error> {
    Ok([root, root.step(3)?, root.step(7)?])  // minor 3rd + perfect 5th
}

// Convert between u8 and Note
let midi_byte: u8 = 60;  // Middle C
let note = Note::from_u8_lossy(midi_byte);  // Note::C4
let back_to_byte: u8 = note.into();  // 60
```

### MIDI Message Handling with wmidi
```rust
// Source: https://github.com/RustAudio/wmidi
use wmidi::{MidiMessage, Note, Velocity};

fn process_message(bytes: &[u8]) -> Option<Vec<u8>> {
    let msg = MidiMessage::try_from(bytes).ok()?;

    match msg {
        MidiMessage::NoteOn(channel, note, velocity) => {
            // Generate harmony
            let harmony_note = note.step(4).ok()?;  // Major third

            // Create output message
            let harmony_msg = MidiMessage::NoteOn(channel, harmony_note, velocity);
            let mut output = vec![0u8; harmony_msg.bytes_size()];
            harmony_msg.copy_to_slice(&mut output).ok()?;
            Some(output)
        }
        MidiMessage::NoteOff(channel, note, velocity) => {
            // Same harmony computation for release
            let harmony_note = note.step(4).ok()?;
            let harmony_msg = MidiMessage::NoteOff(channel, harmony_note, velocity);
            let mut output = vec![0u8; harmony_msg.bytes_size()];
            harmony_msg.copy_to_slice(&mut output).ok()?;
            Some(output)
        }
        _ => None,  // Pass through other messages unchanged
    }
}
```

### Complete Scale Implementation
```rust
// Source: Standard music theory - diatonic scale construction
#[derive(Clone, Copy, Debug)]
pub enum ScaleMode {
    Ionian,      // Major
    Dorian,
    Phrygian,
    Lydian,
    Mixolydian,
    Aeolian,     // Natural minor
    Locrian,
}

impl ScaleMode {
    /// Semitone offsets for each scale degree (0-6)
    pub fn offsets(&self) -> [u8; 7] {
        match self {
            ScaleMode::Ionian     => [0, 2, 4, 5, 7, 9, 11],
            ScaleMode::Dorian     => [0, 2, 3, 5, 7, 9, 10],
            ScaleMode::Phrygian   => [0, 1, 3, 5, 7, 8, 10],
            ScaleMode::Lydian     => [0, 2, 4, 6, 7, 9, 11],
            ScaleMode::Mixolydian => [0, 2, 4, 5, 7, 9, 10],
            ScaleMode::Aeolian    => [0, 2, 3, 5, 7, 8, 10],
            ScaleMode::Locrian    => [0, 1, 3, 5, 6, 8, 10],
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Key {
    C, Db, D, Eb, E, F, Gb, G, Ab, A, Bb, B,
}

impl Key {
    pub fn semitones_from_c(&self) -> u8 {
        match self {
            Key::C  => 0,  Key::Db => 1,  Key::D  => 2,
            Key::Eb => 3,  Key::E  => 4,  Key::F  => 5,
            Key::Gb => 6,  Key::G  => 7,  Key::Ab => 8,
            Key::A  => 9,  Key::Bb => 10, Key::B  => 11,
        }
    }
}
```

### Random Diatonic Interval Selection
```rust
// Source: Derived from requirements HARM-04, HARM-05
use rand::Rng;
use wmidi::Note;

/// Select random diatonic interval below the input note
fn random_interval_below(scale: &Scale, note: Note, exclude_seconds: bool) -> Option<Note> {
    let mut rng = rand::thread_rng();

    // Intervals below: -1 to -6 scale degrees (skip unison)
    let intervals: Vec<i8> = if exclude_seconds {
        vec![-2, -3, -4, -5, -6]  // 3rds through 7ths
    } else {
        vec![-1, -2, -3, -4, -5, -6]  // 2nds through 7ths
    };

    let interval = intervals[rng.gen_range(0..intervals.len())];
    scale.transpose_diatonic(note, interval)
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| External harmony plugins | Built-in engine | Phase 2 | No external dependencies for harmony |
| Fixed chromatic intervals | Diatonic (scale-aware) intervals | Phase 2 | In-key harmony sounds more musical |
| Global mutable config | Thread-safe state with try_lock | Phase 1 pattern | Safe concurrent key/mode changes |

**Deprecated/outdated:**
- Pure chromatic harmonizers: Sound mechanical; diatonic is standard for musical results
- Blocking configuration updates: Causes audio glitches; use non-blocking patterns

## Open Questions

Things that couldn't be fully resolved:

1. **Velocity handling for harmonized notes**
   - What we know: Original velocity should likely pass through to harmony
   - What's unclear: Should harmony be quieter (velocity - 10) for "background" effect?
   - Recommendation: Start with same velocity; make it configurable if users request adjustment

2. **Polyphonic input handling**
   - What we know: Requirements don't specify; single-note examples shown
   - What's unclear: If user plays chord, should each note be harmonized independently?
   - Recommendation: Yes, harmonize each note independently. Track Note-On/Off mappings.

3. **Mode 7 counterpoint depth**
   - What we know: "Traditional voice leading rules" is broad; full species counterpoint is complex
   - What's unclear: How strict should counterpoint rules be?
   - Recommendation: Implement simplified rules (avoid parallel 5ths/octaves, prefer contrary motion). Flag for user feedback.

4. **Latency impact of harmony computation**
   - What we know: Computation is trivial (< 1us per note); not a concern
   - What's unclear: Whether users perceive any delay vs. direct pass-through
   - Recommendation: Measure in Phase 2 verification; should be imperceptible

## Sources

### Primary (HIGH confidence)
- [wmidi docs.rs](https://docs.rs/wmidi/latest/wmidi/enum.Note.html) - Note enum, step() method, MidiMessage
- [wmidi GitHub](https://github.com/RustAudio/wmidi) - Examples, version info
- Phase 1 RESEARCH.md - midir patterns, architecture decisions

### Secondary (MEDIUM confidence)
- [rust-music-theory](https://github.com/ozankasikci/rust-music-theory) - Scale/mode definitions (verified API exists, chose not to use)
- [Wikipedia: Contrapuntal motion](https://en.wikipedia.org/wiki/Contrapuntal_motion) - Contrary motion rules
- [hellomusictheory.com](https://hellomusictheory.com/learn/modes/) - Mode interval patterns
- [Open Music Theory](https://openmusictheory.github.io/motionTypes.html) - Voice leading types

### Tertiary (LOW confidence)
- WebSearch for "diatonic MIDI harmonizer" - Verified concept exists in commercial tools
- WebSearch for "species counterpoint algorithm" - Academic papers confirm optimization approaches exist

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - wmidi is well-documented, step() method verified
- Architecture: HIGH - Patterns derived from Phase 1 + standard Rust idioms
- Scale/mode theory: HIGH - Music theory is well-established; intervals verified
- Pitfalls: MEDIUM - Based on common MIDI programming issues; some inferred

**Research date:** 2026-01-28
**Valid until:** 60 days (music theory doesn't change; wmidi is stable)
