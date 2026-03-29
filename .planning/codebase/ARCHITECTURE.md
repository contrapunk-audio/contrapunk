# Architecture

**Analysis Date:** 2026-03-28

## Pattern Overview

**Overall:** Library-core with platform-specific frontends (Rust core library + WASM bridge + Tauri desktop + SvelteKit browser + CLI server/client)

**Key Characteristics:**
- Rust `contrapunk` crate (`src/lib.rs`) is the shared core -- compiled natively for Tauri/CLI and to WASM for the browser
- Platform adapter layer in the UI (`ui/src/lib/adapter/`) abstracts Tauri IPC vs WASM direct calls behind a single `ContrapunkAdapter` interface
- Three deployment surfaces: Tauri desktop, SvelteKit WASM browser app, CLI server/client TCP mode
- The harmony engine is completely platform-agnostic; all platform-specific code (MIDI I/O, audio capture) is behind `#[cfg(not(target_arch = "wasm32"))]` gates

---

## System Architecture Diagram

```
                    ┌─────────────────────────────────────────────────────────┐
                    │                    Contrapunk                           │
                    │                                                         │
  ┌─────────┐      │  ┌──────────────────────────────────────────────┐       │
  │  MIDI   │──────│──│  MIDI Input Layer                            │       │
  │Controller│     │  │  Native: midir (src/midi/input.rs)           │       │
  └─────────┘      │  │  Browser: Web MIDI API (src/midi/web.rs)     │       │
                    │  └──────────────┬───────────────────────────────┘       │
                    │                 │ raw MIDI bytes                        │
  ┌─────────┐      │                 ▼                                       │
  │  Guitar  │─────│──┐  ┌──────────────────────────────────────┐            │
  │  Audio   │     │  │  │  Router / Message Processing         │            │
  └─────────┘      │  │  │  Native: src/router.rs               │            │
                    │  │  │  Tauri:  src-tauri/src/commands/     │            │
                    │  │  │          engine.rs                   │            │
                    │  │  │  WASM:   ui/src/lib/adapter/wasm.ts  │            │
                    │  │  │  Server: src/server/session.rs       │            │
                    │  │  └──────────┬───────────────────────────┘            │
                    │  │             │ parsed Note-On/Off                     │
                    │  │             ▼                                        │
                    │  │  ┌──────────────────────────────────────────────┐    │
  Audio pipeline   │  └─▶│         HARMONY ENGINE (src/harmony/)        │    │
  (cpal + pitch    │     │                                              │    │
   detection)      │     │  Scale Check ──▶ Mode Algorithm ──▶ Voice   │    │
                    │     │       │              │              Leading │    │
                    │     │  [in-scale?]    [Stateful?]       [revoice]│    │
                    │     │  [interchange]  [VecDeque]        [rules]  │    │
                    │     │       │              │              │       │    │
                    │     │       ▼              ▼              ▼       │    │
                    │     │  Octave Mode ──▶ Port Mapping ──▶ Output   │    │
                    │     └──────────────────────────────────────────────┘    │
                    │                 │                                       │
                    │                 ▼                                       │
                    │  ┌──────────────────────────────────────────────┐       │
                    │  │  Humanizer (src/humanize/)                   │       │
                    │  │  Timing jitter, velocity variation, swing    │       │
                    │  │  Beat clock, delay queue, metronome          │       │
                    │  └──────────────┬───────────────────────────────┘       │
                    │                 │                                       │
                    │                 ▼                                       │
                    │  ┌──────────────────────────────────────────────┐       │
                    │  │  MIDI Output Layer                           │       │
                    │  │  Native: OutputRouter (src/midi/output.rs)   │       │
                    │  │  Browser: Web MIDI API (adapter/wasm.ts)     │       │
                    │  │  Server: TCP stream (server/protocol.rs)     │       │
                    │  └──────────────────────────────────────────────┘       │
                    └─────────────────────────────────────────────────────────┘

  ┌────────────────────────────────────────────────────────────────────────┐
  │  UI LAYER (SvelteKit)                                                  │
  │                                                                        │
  │  ui/src/routes/+page.svelte  -- Main page, QWERTY keyboard input       │
  │  ui/src/lib/stores/          -- Svelte 5 state management              │
  │  ui/src/lib/adapter/         -- Platform abstraction                   │
  │       index.ts  --> detects Tauri vs Browser at runtime                │
  │       tauri.ts  --> TauriAdapter (IPC to Rust backend)                 │
  │       wasm.ts   --> WasmAdapter (direct WASM calls)                    │
  │       types.ts  --> ContrapunkAdapter interface                        │
  │  ui/src/lib/components/      -- ControlPanel, Piano, MidiDevices...    │
  └────────────────────────────────────────────────────────────────────────┘
```

---

## Layers

### 1. Core Library (`src/`)

**Purpose:** Platform-agnostic harmony generation, audio processing, and music theory primitives.

**Location:** `src/lib.rs` (module root), compiled as `contrapunk` crate

**Contains:**
- `src/harmony/` -- Harmony engine, scale system, mode algorithms, voice leading
- `src/audio/` -- Pitch detection, onset detection, guitar calibration, buffer management
- `src/midi/` -- MIDI I/O (native: `midir`; browser: Web MIDI API via `web-sys`)
- `src/humanize/` -- Timing jitter, velocity variation, swing, beat clock
- `src/chord.rs` -- Chord detection from MIDI note combinations (extended chords, roman numerals)
- `src/generator/` -- Beat-driven note generator (arpeggios, sequences, held chords)
- `src/preset/` -- Style presets with persona names (Jazz/Alchemist, Classical/Architect, etc.)
- `src/router.rs` -- Native MIDI routing loop (input -> harmony -> output)
- `src/server/` -- TCP server/client for network-based MIDI processing

**Depends on:** `wmidi`, `serde`, `rand`, `anyhow`; native-only: `midir`, `clap`, `cpal`, `pitch-detection`, `rmp-serde`

**Used by:** WASM bridge, Tauri backend, CLI binary

### 2. WASM Bridge (`wasm/`)

**Purpose:** Exposes `HarmonyEngine` to JavaScript via wasm-bindgen with string-based APIs.

**Location:** `wasm/src/lib.rs`, compiled as `contrapunk-wasm` crate (cdylib)

**Contains:**
- `Engine` struct wrapping `HarmonyEngine` and `PresetManager`
- String-to-enum parsers for all config types (keys, modes, scales, octave modes, voice leading styles)
- `note_on()` / `note_off()` methods returning `Vec<u8>` of MIDI note numbers
- `get_state()` / `get_note_state()` serialized via `serde-wasm-bindgen`
- `midi_to_name()` helper (MIDI number -> note name string)

**Built with:** `wasm-pack build --target web --out-dir ../ui/src/lib/wasm-pkg`

**Depends on:** `contrapunk` (path dep), `wasm-bindgen`, `serde-wasm-bindgen`, `console_error_panic_hook`

**Used by:** SvelteKit UI (imported as `$lib/wasm-pkg` in `ui/src/lib/adapter/wasm.ts`)

### 3. Tauri Desktop Backend (`src-tauri/`)

**Purpose:** Rust backend for the Tauri v2 desktop app. Wraps `contrapunk` with IPC command handlers.

**Location:** `src-tauri/src/main.rs`

**Contains:**
- `src-tauri/src/commands/harmony.rs` -- Set key, mode, scale, octave mode, voice leading, interchange
- `src-tauri/src/commands/engine.rs` -- Start/stop MIDI routing, get note state; spawns router thread
- `src-tauri/src/commands/midi.rs` -- List/refresh MIDI devices via midir
- `src-tauri/src/commands/presets.rs` -- Load/save/delete presets
- `src-tauri/src/state.rs` -- `AppState` with Mutex-guarded HarmonyEngine, note tracking, humanize config

**Depends on:** `contrapunk` (path dep), `tauri` v2, `midir`, `wmidi`

**Used by:** SvelteKit UI (via Tauri IPC when running as desktop app)

### 4. SvelteKit UI (`ui/`)

**Purpose:** Shared UI for both Tauri desktop and browser WASM modes.

**Location:** `ui/src/`

**Contains:**
- `ui/src/lib/adapter/` -- Platform adapter layer (factory pattern, runtime detection)
  - `index.ts` -- Detects `__TAURI__` global, exports singleton `adapter`
  - `types.ts` -- `ContrapunkAdapter` interface (20+ methods)
  - `tauri.ts` -- `TauriAdapter` class (IPC via `@tauri-apps/api`)
  - `wasm.ts` -- `WasmAdapter` class (direct WASM calls + Web MIDI API)
- `ui/src/lib/stores/` -- Svelte 5 runes-based state
  - `engine.svelte.ts` -- Engine state, note state, initialization
  - `midi.svelte.ts` -- MIDI device lists, selection state
  - `ui.svelte.ts` -- UI mode (light/dark, panel visibility)
- `ui/src/lib/components/` -- UI components
  - `ControlPanel.svelte` -- Key, mode, scale mode, voice leading, interchange controls
  - `Piano.svelte` -- Interactive piano keyboard visualization
  - `MidiDevices.svelte` -- Device selection (input/output)
  - `ActiveNotes.svelte` -- Real-time note display
  - `PresetManager.svelte` -- Preset load/save/delete
  - `HumanizePanel.svelte` -- Humanization parameter controls
  - `GeneratorPanel.svelte` -- Beat generator controls
  - `StatusBar.svelte` -- Connection status, platform info
- `ui/src/routes/+page.svelte` -- Main page, QWERTY keyboard-to-MIDI input mapping

### 5. CLI Binary (`src/main.rs`)

**Purpose:** Command-line entry point for server/client TCP modes.

**Location:** `src/main.rs`

**Modes:**
- `--server [--port 9900]` -- TCP server accepting MIDI stream clients
- `--client <host:port>` -- Connects to server, routes local MIDI through remote harmony engine
- Default: prints usage instructions pointing to Tauri/browser UIs

### 6. Audio Pipeline (`src/audio/`)

**Purpose:** Guitar-to-MIDI conversion via real-time pitch detection and onset analysis.

**Location:** `src/audio/`

**Contains:**
- `pitch.rs` -- `NoteTracker` with octave-error rejection, 3-frame median voting, onset gating
- `onset.rs` -- `PluckDetector` combining HFC, spectral flux, amplitude slope with adaptive thresholds
- `detectors.rs` -- Three alternative pitch detectors:
  - `BacfDetector` -- Bitstream autocorrelation (XOR + popcount, 64 samples/op)
  - `AmdfDetector` -- Average Magnitude Difference Function
  - `GoertzelBank` -- Goertzel filter bank tuned to 49 guitar note frequencies
- `single_cycle.rs` -- Ultra-low-latency single-cycle detector (dual-predictor, no buffering)
- `buffer.rs` -- `RingBuffer`, `OverlapManager`, `DualBufferAnalyzer` (512-sample fast + 1024-sample slow paths)
- `guitar.rs` -- Guitar-specific calibration profiles, string matching, chord grouping, audio normalization
- `config.rs` -- User-adjustable detection thresholds
- `profiles.rs` -- Named presets for different recording environments
- `test_signals.rs` -- Synthetic test signal generation for unit tests

### 7. ML Pipeline (`ml/`)

**Purpose:** Machine learning pipeline for guitar string classification (in development).

**Location:** `ml/`

**Contains:**
- `ml/loader.py` -- Python dataset loader for MessagePack training data captured by Rust
- `ml/CONCEPTS.md` -- Comprehensive guide to audio ML concepts (DI signals, FFT, mel-spectrograms, Goertzel, harmonic ratios)
- `ml/processing/01_raw_analysis/` -- Raw data analysis scripts
- `ml/app/` -- SvelteKit-based data visualization app
- `ml/requirements.txt` -- Python dependencies (`msgpack`, `numpy`)

**Data format:** `guitar_training_data.msgpack` -- Training samples captured via `examples/guitar_capture.rs` containing audio, labels, class IDs, string/fret info, harmonic features

---

## Key Code Paths

### Code Path 1: MIDI In -> Harmony -> MIDI Out (Native/Tauri)

```
1. MIDI Controller sends Note-On bytes [0x90, note, velocity]
   --> midir callback in src/midi/input.rs:connect_input()
       --> mpsc::channel forwards Vec<u8>

2. Router thread receives bytes
   --> src-tauri/src/commands/engine.rs:start_routing() (Tauri)
   --> OR src/router.rs:run_router() (CLI)
       --> wmidi::MidiMessage::try_from(&bytes) parses raw MIDI
       --> Extracts Note, Channel, Velocity

3. Harmony Engine processes note
   --> src/harmony/engine.rs:HarmonyEngine::harmonize_note_on(note)
       --> harmonize(note) returns Vec<Note>
           a. Build Scale from Key + ScaleMode
           b. Apply mode algorithm (src/harmony/modes.rs)
              - Stateless: diatonic_thirds(), barry_harris(), etc.
              - Stateful: ContraryMotionState::process(), CounterpointState::process()
           c. Chain harmonies for multi-voice (harm2 = harmony_of(harm1))
           d. Voice leading revoicing (src/harmony/voice_leading/voicer.rs)
           e. Octave mode post-processing (spread/split/mirror)
       --> Stores result in active_notes HashMap for Note-Off tracking
       --> Returns Vec<Note> with port_map for output routing

4. Humanizer (optional)
   --> src/humanize/engine.rs:Humanizer::humanize_note_on()
       --> Applies timing jitter, velocity variation, swing offset
       --> Returns HumanizedNote with delay_ms

5. Output routing
   --> src/midi/output.rs:OutputRouter::send_to_port(port_index, midi_bytes)
       --> Each harmony voice routed to its assigned output port
```

### Code Path 2: MIDI In -> Harmony -> MIDI Out (Browser/WASM)

```
1. Web MIDI API receives MIDIMessageEvent
   --> ui/src/lib/adapter/wasm.ts:startRouting()
       --> activeInput.onmidimessage callback

2. WASM Engine processes note
   --> wasm/src/lib.rs:Engine::note_on(note_u8) -> Vec<u8>
       --> Calls contrapunk::harmony::HarmonyEngine::harmonize_note_on()
       --> Converts Vec<Note> to Vec<u8>

3. Output routing in JavaScript
   --> wasm.ts sorts notes ascending (bass on output 0)
   --> Sends [0x90, note, velocity] to MIDIOutput devices
       --> Round-robin: output[i % outputCount]
```

### Code Path 3: Guitar Audio -> Pitch Detection -> MIDI

```
1. Audio capture via cpal (native only)
   --> examples/guitar_harmony.rs or examples/guitar_calibrate.rs
       --> cpal::Stream captures f32 audio samples at 44,100 Hz

2. Onset detection
   --> src/audio/onset.rs:PluckDetector::process_frame(magnitudes)
       --> Three signals: HFC, spectral flux, amplitude slope
       --> Adaptive thresholds via running mean
       --> Returns true when pluck detected

3. Pitch detection (multiple strategies)
   --> src/audio/detectors.rs:GoertzelBank::detect() -- 49-note filter bank
   --> src/audio/detectors.rs:BacfDetector::detect() -- bitstream autocorrelation
   --> src/audio/single_cycle.rs:SingleCycleDetector -- ultra-low-latency
   --> OR pitch-detection crate (pyin)

4. Note tracking and filtering
   --> src/audio/pitch.rs:NoteTracker::update(frequency, confidence)
       --> Octave-error rejection (T1): reject +/-12 jumps with low confidence
       --> Median voting (T3): 3-frame median filter
       --> Onset gating (T5): only emit NoteOn after pluck detected
       --> Emits NoteEvent::NoteOn(midi) or NoteEvent::NoteOff(midi)

5. Guitar-specific matching
   --> src/audio/guitar.rs:GuitarPitchMatcher
       --> Maps detected pitch to specific guitar string using calibration profile
       --> OnsetGrouper collects near-simultaneous notes into ChordEvents

6. Feed into harmony engine
   --> HarmonyEngine::harmonize_note_on(detected_midi_note)
   --> Same pipeline as MIDI controller input from here
```

---

## The Harmony Engine in Detail

### Processing Pipeline (`src/harmony/engine.rs`)

```
harmonize(input_note: Note) -> Vec<Note>
|
+-- 1. SCALE CHECK
|   +-- Scale::is_in_scale(note) -- checks pitch class against scale offsets
|       +-- In-scale: use diatonic transposition
|       +-- Out-of-scale: try modal interchange (if enabled) or chromatic consonants
|
+-- 2. MODE ALGORITHM DISPATCH
|   +-- Stateless modes (1-5, 8): src/harmony/modes.rs
|   |   +-- pass_through() -- returns [note]
|   |   +-- diatonic_thirds() -- Scale::harmonize_smart(note, 2, true)
|   |   +-- diatonic_fourths() -- Scale::harmonize_smart(note, 3, true)
|   |   +-- random_below() -- random interval -1 to -6 degrees
|   |   +-- random_below_no_seconds() -- random -2 to -6 degrees
|   |   +-- barry_harris() -- 2 degrees (preserves chord/passing parity in 8-note scales)
|   |
|   +-- Stateful modes (6-7): src/harmony/stateful.rs
|       +-- ContraryMotionState::process_directed()
|       |   +-- Tracks last_melody, last_harmony; moves opposite direction
|       |       When melody repeats: alternates direction (oblique motion)
|       |
|       +-- CounterpointState::process()
|           +-- Sliding windows:
|               - interval_history: VecDeque<i8> (size 4)
|               - melody_contour: VecDeque<MelodicDirection> (size 3)
|               +-- Scoring: parallel 5ths/8ves=-100, stepwise=+4, contrary=+3, etc.
|
+-- 3. VOICE POSITION & CHAINING
|   +-- Multi-voice generation from voice_position outward:
|       voice_position=2 in 4-voice:
|         Soprano [0]  <-- harmony_of(harmony_of(input)) above
|         Alto    [1]  <-- harmony_of(input) above
|         Tenor   [2]  <-- USER INPUT
|         Bass    [3]  <-- harmony_of(input) below
|       Each harmony is chained: harm_above2 = mode_fn(harm_above1)
|
+-- 4. VOICE LEADING POST-PROCESSING (optional)
|   +-- src/harmony/voice_leading/voicer.rs:revoice_chord()
|       +-- Generates valid placements per voice register (Soprano: 60-81, Alto: 55-76, etc.)
|       +-- Evaluates all candidate voicings holistically (not greedily)
|       +-- Applies StyleRules scoring:
|       |   +-- Palestrina: hard-reject parallel 5ths/8ves, max leap 5 semitones, tight spacing
|       |   +-- BachChorale: hard-reject parallels, common-tone bonus=70, max leap 12
|       |   +-- Jazz: no hard rejects, spread preference=+5, leaps OK
|       |   +-- Free: minimal constraints, closest note wins
|       +-- Rule checks: check_parallel_fifths(), check_voice_crossing(), check_spacing()
|       +-- Palestrina suspensions: SuspensionState (prepare -> suspend -> resolve)
|
+-- 5. OCTAVE MODE
|   +-- None: no change
|   +-- Spread: voice[i] += i * 12 semitones
|   +-- BassTrebleSplit: below melody -12, above melody +12
|   +-- Mirror: each harmony duplicated at +12 and -12 (triples harmony notes)
|
+-- 6. NOTE TRACKING
    +-- active_notes: HashMap<u8, Vec<Note>>
        Maps input MIDI -> produced harmony notes
        Critical for random modes: Note-Off must release exact same notes as Note-On
```

---

## The Voice Leading System

### Architecture (`src/harmony/voice_leading/`)

```
voice_leading/
+-- mod.rs          -- Re-exports all public types
+-- styles.rs       -- VoiceLeadingStyle enum (Palestrina, BachChorale, Jazz, Free)
|                      StyleRules struct with ~12 scoring parameters per style
+-- rules.rs        -- Pure rule-checking functions:
|                      check_parallel_fifths(), check_parallel_octaves(),
|                      check_voice_crossing(), check_spacing(),
|                      check_motion_independence(), interval_class()
+-- voicer.rs       -- revoice_chord() -- the main voicing algorithm
|                      VoiceRegister (Soprano 60-81, Alto 55-76, Tenor 48-69, Bass 40-64)
|                      VoiceAnchor (constrains user's note position in arrangement)
|                      Deterministic: same input always produces same output
+-- suspension.rs   -- SuspensionState -- Palestrina-only suspension handling
                       3-phase cycle: None -> Suspended -> NeedResolution
                       Holds one voice from previous chord, resolves stepwise down
```

### Voice Leading Styles

| Style | Parallel 5ths/8ves | Max Leap | Stepwise Bonus | Spread | Use Case |
|-------|--------------------|----------|----------------|--------|----------|
| **Palestrina** | Hard reject | 5 semitones | +60 | -4 (tight) | Renaissance polyphony, choir in close harmony |
| **BachChorale** | Hard reject | 12 semitones (octave) | +25 | -1 (slight tight) | Hymn harmonization, held inner voices |
| **Jazz** | Soft penalty (-2) | 127 (unlimited) | +3 | +5 (wide) | Drop-2/drop-3 voicings, open spacious sound |
| **Free** | No penalty | 127 (unlimited) | +1 | 0 (neutral) | Minimal intervention, closest available note |

### Scoring Parameters (StyleRules)

Each style defines these parameters (`src/harmony/voice_leading/styles.rs`):

| Parameter | Palestrina | BachChorale | Jazz | Free |
|-----------|-----------|-------------|------|------|
| `hard_reject_parallel_fifths` | true | true | false | false |
| `hard_reject_parallel_octaves` | true | true | false | false |
| `parallel_fifths_penalty` | -200 | -100 | -2 | 0 |
| `parallel_octaves_penalty` | -200 | -100 | -2 | 0 |
| `voice_crossing_penalty` | -150 | -80 | -10 | -2 |
| `stepwise_bonus` | 60 | 25 | 3 | 1 |
| `common_tone_bonus` | 45 | 70 | 2 | 1 |
| `leap_penalty_per_semitone` | -15 | -4 | 0 | 0 |
| `max_leap_semitones` | 5 | 12 | 127 | 127 |
| `all_parallel_motion_penalty` | -120 | -60 | -5 | 0 |
| `spacing_violation_penalty` | -200 | -80 | 0 | 0 |
| `spread_preference` | -4 | -1 | 5 | 0 |
| `contrary_motion_bonus` | 40 | 20 | 3 | 0 |

---

## Scale Modes System

### 28 Modes Across 5 Families (`src/harmony/config.rs`)

**Family: Church Modes (7)** -- modes of the major scale
| Mode | Intervals | Character |
|------|-----------|-----------|
| Ionian (Major) | 0-2-4-5-7-9-11 | Bright, happy |
| Dorian | 0-2-3-5-7-9-10 | Minor with raised 6th, jazzy |
| Phrygian | 0-1-3-5-7-8-10 | Spanish/Middle Eastern flavor |
| Lydian | 0-2-4-6-7-9-11 | Dreamy, raised 4th |
| Mixolydian | 0-2-4-5-7-9-10 | Bluesy, dominant 7th |
| Aeolian (Minor) | 0-2-3-5-7-8-10 | Natural minor |
| Locrian | 0-1-3-5-6-8-10 | Diminished tonic, unstable |

**Family: Harmonic Minor (7)** -- augmented 2nd interval
| Mode | Intervals |
|------|-----------|
| HarmonicMinor | 0-2-3-5-7-8-11 |
| LocrianNat6 | 0-1-3-5-6-9-10 |
| IonianAug | 0-2-4-5-8-9-11 |
| DorianSharp4 | 0-2-3-6-7-9-10 |
| PhrygianDominant | 0-1-4-5-7-8-10 |
| LydianSharp2 | 0-3-4-6-7-9-11 |
| SuperLocrianDim | 0-1-3-4-6-8-9 |

**Family: Melodic Minor (7)** -- jazz minor modes
| Mode | Intervals |
|------|-----------|
| MelodicMinor | 0-2-3-5-7-9-11 |
| DorianFlat2 | 0-1-3-5-7-9-10 |
| LydianAug | 0-2-4-6-8-9-11 |
| LydianDominant | 0-2-4-6-7-9-10 |
| MixolydianFlat6 | 0-2-4-5-7-8-10 |
| LocrianNat2 | 0-2-3-5-6-8-10 |
| SuperLocrian | 0-1-3-4-6-8-10 |

**Family: Exotic (5)** -- non-Western and synthetic
| Mode | Intervals |
|------|-----------|
| DoubleHarmonic | 0-1-4-5-7-8-11 |
| HungarianMinor | 0-2-3-6-7-8-11 |
| Enigmatic | 0-1-4-6-8-10-11 |
| NeapolitanMinor | 0-1-3-5-7-8-11 |
| NeapolitanMajor | 0-1-3-5-7-9-11 |

**Family: Barry Harris (2)** -- 8-note bebop scales
| Mode | Intervals | Notes |
|------|-----------|-------|
| BHMajor6thDim | 0-2-4-5-7-8-9-11 | 8 notes: major scale + b6 passing tone |
| BHMinor6thDim | 0-2-3-5-7-8-9-11 | 8 notes: minor scale + b6 passing tone |

**Barry Harris key insight:** In 8-note BH scales, even-index degrees (0,2,4,6) are chord tones and odd-index degrees (1,3,5,7) are passing tones. Moving by exactly 2 scale degrees preserves parity -- chord tone maps to chord tone, passing tone maps to passing tone. This creates smooth voice leading while maintaining harmonic clarity.

### Modal Interchange (`src/harmony/scale.rs`)

When a note is out-of-scale and interchange is enabled:
1. Search parallel modes from the same tonic (borrowing_range controls how many: 1-5)
2. Range 1: Aeolian, HarmonicMinor
3. Range 2: + Dorian, MelodicMinor
4. Range 3: + Mixolydian, Phrygian
5. Range 4: + Lydian, PhrygianDominant
6. Range 5: + Locrian, Ionian, LydianDominant
7. If a parallel mode contains the note, borrow its harmonization
8. Track `last_borrowed_from` for UI display
9. Fallback: use consonant chromatic intervals (M3, m3, M6, m6, P5, P4)

---

## Audio Pipeline in Detail (`src/audio/`)

### Onset Detection (`src/audio/onset.rs`)

`PluckDetector` combines three complementary signals:
1. **High-Frequency Content (HFC):** `sum(magnitude[k] * k)` -- plucks inject broadband energy weighting upper bins
2. **Spectral flux:** half-wave-rectified frame-to-frame magnitude difference -- captures sudden spectral changes
3. **Amplitude slope:** frame-to-frame total-energy jump -- guards against slow swells

All three use running mean for adaptive thresholding (self-calibrates to noise floor).

### Pitch Detection Strategies (`src/audio/detectors.rs`, `src/audio/single_cycle.rs`)

| Detector | Approach | Speed | Best For |
|----------|----------|-------|----------|
| `GoertzelBank` | 49 filters tuned to guitar notes E2-E6 | O(N) per filter | Known frequency set, low overhead |
| `BacfDetector` | Bitstream autocorrelation (XOR + popcount) | Very fast (64 samples/op) | General pitch, binary comparison |
| `AmdfDetector` | Average Magnitude Difference Function | Faster than ACF | Mid-range accuracy |
| `SingleCycleDetector` | Peak-to-peak measurement, dual predictor | Ultra-low latency (1 cycle) | Fastest initial detection |
| `pitch-detection` crate (pyin) | Standard probabilistic YIN | Moderate | High accuracy reference |

### Dual-Buffer Analysis (`src/audio/buffer.rs`)

`DualBufferAnalyzer` routes audio to two parallel paths:
- **Fast path (512 samples):** For high strings (B3, high E4) -- faster response, sufficient frequency resolution
- **Slow path (1024 samples):** For low strings (low E2, A2) -- better frequency resolution needed for low fundamentals
- `OverlapManager` produces 50% or 75% overlapping frames from continuous stream

### Guitar Calibration (`src/audio/guitar.rs`)

Per-string calibration profiles capturing:
- Soft/strong pluck samples with frequency, confidence, peak, RMS
- Brightness (high-frequency) band analysis
- Onset indicators (main_delta, main_ratio, main_slope)
- Standard tuning: E2(40), A2(45), D3(50), G3(55), B3(59), E4(64)

---

## The ML Pipeline (`ml/`)

### Purpose
Building a guitar string classifier that identifies which string was plucked from audio features (harmonic ratios, inharmonicity, spectral centroid).

### Data Capture
- `examples/guitar_capture.rs` -- Rust binary that captures labeled training samples
- Outputs `guitar_training_data.msgpack` (MessagePack format, ~45MB)
- Each sample: audio waveform, label, class_id, string_idx, fret, expected/detected MIDI, confidence, RMS, peak, Goertzel harmonics, harmonic ratios, spectral centroid

### Data Loading
- `ml/loader.py` -- Python loader (`GuitarDataset` class)
- Handles rmp-serde's positional (array) serialization format
- Provides `.summary()`, filtering, and numpy array extraction

### Feature Concepts (from `ml/CONCEPTS.md`)
- **Harmonic ratio features:** h2/h1, h3/h1, ... h10/h1 ratios as string fingerprints
- **Inharmonicity:** deviation of harmonics from perfect integer multiples (differs by string gauge/tension)
- **Mel-spectrograms:** 64 mel bins, 60-8000 Hz, FFT 2048, hop 512
- **Goertzel bank:** 49 filters for each guitar note frequency

### Status
Early stage -- data capture and raw analysis infrastructure complete; model training and inference not yet implemented.

---

## Deployment Architecture

### Browser (WASM)
```
wasm-pack build  -->  ui/src/lib/wasm-pkg/ (JS+WASM bundle)
cd ui && npm run build  -->  ui/build/ (static SvelteKit)
deploy/Dockerfile  -->  nginx serving static files
deploy/fly.toml  -->  Fly.io deployment
```

### Desktop (Tauri)
```
cd src-tauri && cargo tauri build  -->  native binary with embedded webview
SvelteKit UI served from Tauri's asset protocol
Rust backend runs natively with full midir/cpal access
```

### Server/Client (TCP)
```
contrapunk --server --port 9900  -->  TCP accept loop
contrapunk --client host:port  -->  Streams MIDI over TCP
Wire protocol: [u16 BE length][u8 type][payload]
Message types: MidiData(0x01), Configure(0x02), Ack(0x03), Disconnect(0x04), Heartbeat(0x05)
Per-client HarmonyEngine instance on server
```

---

## Preset System (`src/preset/`)

Built-in presets with character personas:
| Preset | Persona | Genre | Mode | Voice Leading | Scale | Humanize |
|--------|---------|-------|------|--------------|-------|----------|
| Jazz | The Alchemist | Jazz | StrictCounterpoint | Jazz | Ionian | Swing 0.55, jitter 5-25ms |
| Classical | The Architect | Classical | DiatonicThirds | BachChorale | Ionian | Jitter 1-8ms, vel var 5 |

Custom presets store all engine settings (key, mode, scale, octave, voice leading, interchange, humanize config). Preset data model: `src/preset/mod.rs` (`StylePreset` struct), builtins: `src/preset/builtins.rs`, persistence: `src/preset/storage.rs`.

---

## Humanization Pipeline (`src/humanize/`)

```
Harmony Engine  -->  Humanizer  -->  DelayQueue  -->  MIDI Output
                         |               |
                    [BeatClock]     [Scheduled events]
                    [Config]       [Timing offsets]
```

**Effects applied to harmony voices only (melody bypasses):**
| Effect | Range | Purpose |
|--------|-------|---------|
| Timing jitter | 1-30ms random delay | Human imprecision |
| Velocity variation | +/-10-20% | Dynamic variation |
| Swing/groove | 0.0-0.5 off-beat shift | Genre feel |
| Duration variation | 0-50ms extension | Natural sustain |

`BeatClock` (`src/humanize/beat_clock.rs`) tracks BPM, beat position, and bar boundaries. `DelayQueue` (`src/humanize/scheduler.rs`) schedules delayed note events. `Metronome` (`src/humanize/metronome.rs`) provides optional audible click.

---

## Chord Detection (`src/chord.rs`)

Analyzes active MIDI notes to identify chord types:
- Extended chords: 9th, 11th, 13th (up to 6-note patterns)
- Altered dominants: 7b9, 7#9, 7#5, 7b5
- 6th chords, add chords, slash chords
- Patterns ordered by specificity (longest match first)
- Roman numeral analysis relative to current key
- `chord_display_with_analysis()` used by Tauri engine commands for UI display

---

## Note Generator (`src/generator/`)

Beat-driven virtual MIDI input:
- `GeneratorMode::HeldNotes` -- sustains all selected notes
- `GeneratorMode::Chord` -- plays selected notes as chord
- `GeneratorMode::Arpeggio(direction)` -- sequences through notes (up/down/updown/random)
- Configurable note duration (in beats), velocity, and note selection
- Synchronized to `BeatClock` for tempo-locked output

---

## Cross-Cutting Concerns

**Conditional Compilation:**
- `#[cfg(not(target_arch = "wasm32"))]` guards all native-only code (midir, cpal, TCP server, router)
- `#[cfg(feature = "web-midi")]` gates browser Web MIDI API code
- Core harmony engine compiles on all targets without feature flags

**Error Handling:**
- Core library: `anyhow::Result` for native, `Result<T, JsValue>` for WASM
- Tauri commands: return `Result<T, String>` (serialized for IPC)
- WASM adapter: catch-and-rethrow with descriptive `Error` messages

**Serialization:**
- Serde throughout for config types (JSON for Tauri IPC, MessagePack for ML data)
- `serde-wasm-bindgen` for Rust->JS object transfer in WASM bridge
- TCP protocol: custom length-prefixed binary framing

**State Management:**
- Tauri: `AppState` with `Mutex<HarmonyEngine>` + atomic flags for routing thread
- WASM: `Engine` struct holds mutable `HarmonyEngine` directly (single-threaded)
- UI: Svelte 5 runes (`$state`, `$derived`) in `ui/src/lib/stores/`

**Logging:** `eprintln!` for debug output (CLI/native), `console_error_panic_hook` for WASM browser console

---

*Architecture analysis: 2026-03-28*
