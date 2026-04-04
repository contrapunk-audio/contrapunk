# Phase 7: Performance Mode - Context

**Gathered:** 2026-02-25
**Status:** Ready for planning

<domain>
## Phase Boundary

Beat-aware performance mode with two major capabilities:

1. **Backing Track Analysis** — Listen to a separate audio input (backing track, song, or room mic) and auto-detect chords, key/scale, and BPM in real-time. Detected parameters auto-configure the harmony engine (root note, scale, tempo). The user's guitar plays through the normal pitch→MIDI pipeline on one audio channel while the backing track analysis runs on a second channel.

2. **Accompaniment Generation** — Accumulate played notes over bars (using BeatClock) and generate musically-contextual accompaniment patterns based on phrase-level state, rather than harmonizing note-by-note. The system analyzes chord progressions, rhythmic density, melodic contour, and key/mode from accumulated input to drive intelligent accompaniment generation.

</domain>

<decisions>
## Implementation Decisions

### Accumulation Window
- Two selectable window modes: **Fixed bar count** and **Sliding + phrase-detect**, which can be active simultaneously
- When both modes are active, they use **merged context** — fixed resets add hard boundary markers, phrase-detect triggers the actual response
- Fixed bar count range: **1-16 bars (any integer)**, including odd counts for asymmetric phrases
- **Time signature must be configurable** (3/4, 4/4, 5/4, 6/8, 7/8, etc.) — extends the existing BeatClock
- Sliding window uses **weighted decay** — recent bars influence the response more, older bars gradually fade
- Phrase-detect silence gate: **dynamic, combining tempo-relative baseline with density-adaptive adjustment**
  - At faster tempos / denser playing, gate threshold shrinks
  - At slower tempos / sparser playing, gate threshold widens
  - **User sets min/max bounds** on the dynamic gate (e.g., 0.5-4 beats)

### Response Generation
- Generates **accompaniment patterns** (not call-and-response or melodic variation)
- Four selectable pattern types: **chord comping**, **bass line**, **arpeggiated pads**, **rhythmic ostinato**
- **Multiple patterns can be layered simultaneously** — each routes to its own MIDI output
- Each pattern outputs to a **user-assigned MIDI output port** (full routing flexibility)
- **Single style per pattern type** (no sub-variants for now — keep simple)
- Patterns have **pre-built groove** (e.g., swing eighths for jazz comping) — humanization adds more on top
- Generated accompaniment passes through the **full pipeline** (voice leading + humanization)
- **Density/intensity control**: auto-match by default (mirrors user's playing intensity) with manual override slider
- When user stops playing: accompaniment **fades out gracefully** over 1-2 bars
- Accompaniment adapts to context changes at **next bar boundary** (stays stable within a bar)
- **Separate preset system** for Performance Mode (independent from style presets)

### Response Timing
- Accompaniment starts **immediately (next bar)** — no lead-in period required
- Patterns can be **multi-bar** (1, 2, or 4 bars) for more composed, less repetitive feel
- If context changes mid-pattern: **adapt at next bar boundary** even if mid-pattern

### Activation & Coexistence
- Three coexistence modes (user toggle): **Harmony + Performance**, **Performance only**, **Harmony only**
- Controls live in a **slide-out side panel** in the main (single) view
- **Both MIDI CC and keyboard shortcut** for quick toggle during performance
- Deactivation: **fade out over 1-2 bars** for smooth transition

### Backing Track Analysis (Audio DSP)
- **Second audio input**: separate device/channel from guitar (e.g., Audient iD14 ch3/4 for backing, ch1/2 for guitar)
- **Chromagram extraction**: FFT → 12-bin pitch class histogram (C, C#, D, ..., B), updated every ~100ms
- **Chord detection** (~500ms updates): template-match chromagram against chord profiles (Maj, min, 7, m7, Maj7, dim, aug, sus2, sus4, add9, etc.)
  - Auto-sets harmony engine root note + chord quality
- **Key/scale detection** (~4-8 second window): Krumhansl-Schmuckler algorithm — correlate accumulated chroma with 24 key profiles (12 major + 12 minor)
  - Auto-sets scale/mode in harmony engine
  - Confidence threshold required before locking (prevent false switching)
- **BPM detection** (~4-8 second window): onset detection on backing track → inter-onset interval histogram → dominant tempo
  - Autocorrelation on onset function as secondary estimator
  - Locks BeatClock tempo for rhythmic features (arpeggiator, humanize timing, accompaniment)
- **Lock buttons**: user can lock any auto-detected parameter to prevent it from changing (e.g., lock key but let chords float)
- **Confidence indicators**: visual display showing how certain each detection is
- Must work in both native (cpal second input) and WASM (browser second getUserMedia / system audio)
- Chromagram computation reuses FFT data already computed in the audio pipeline

### Musical Analysis (MIDI-based, from user's playing)
- Four analysis features, all active during Performance Mode:
  1. **Chord progression detection** with **pattern matching** (ii-V-I, I-vi-IV-V, 12-bar blues, etc.)
  2. **Rhythmic density/pattern** tracking (notes per beat, motif detection)
  3. **Melodic contour** — both **direction and intervals** (stepwise vs leaps)
  4. **Key/mode estimation** — **auto-switches** the active key when confident (high threshold, ~80%+ note fit over several bars)
- **Predictive accompaniment**: if a common progression is detected (e.g., ii-V), accompaniment anticipates the next chord (e.g., I)
- When prediction is wrong: **smooth transition** blending from predicted to actual over 1-2 beats
- **Velocity/dynamics-aware** — accompaniment matches the player's dynamic level
- **Separate decay weighting** for different analysis types (key estimation gets longer memory, rhythmic density is more responsive)
- Time signature is user-set (no automatic meter detection)
- User selects accompaniment style manually (no genre auto-detection)
- All analysis results **visible in the UI** (detected key, chord, density, contour)
- **Backing track analysis takes priority** over MIDI-based analysis when both are active (the song defines the key/chords, user's playing is harmonized against it)

### Visual Feedback
- Performance Mode side panel (resizable) contains:
  1. **Grid/sequencer-style bar position indicator** — beats x bars grid showing accumulated notes in context (like a real-time piano roll)
  2. **Visual meters/graphs** for analysis: VU meter for density, mini line graph for contour, colored label for chord
  3. **Active patterns display** showing which patterns are generating and their current notes
  4. **Pattern controls** — toggle each type, output routing, density slider
- Accompaniment notes shown on the **piano keyboard with a distinct color** (separate from input green and harmony orange)

### Claude's Discretion
- Mute/solo per pattern (vs simple on/off)
- Exact MIDI CC number for toggle
- Keyboard shortcut key choice
- Specific groove patterns for each accompaniment type
- Weighted decay curve shape (linear, exponential, etc.)
- Grid/sequencer resolution (per-beat, per-eighth, per-sixteenth)
- Exact color for accompaniment notes on piano

</decisions>

<specifics>
## Specific Ideas

- The accompaniment should feel like a backing musician who's always listening — not a rigid loop machine
- Prediction with smooth recovery mirrors how real jazz musicians anticipate chord changes
- Dynamic silence gate should make phrase detection feel natural regardless of tempo or playing style
- Weighted decay creates a "musical memory" where recent playing matters most but context is maintained

</specifics>

<deferred>
## Deferred Ideas

- **UI modernization from egui** — User wants to explore moving to a more sophisticated GUI framework beyond egui. Separate phase.
- **Call-and-response phrase generation** — Trading melodic phrases with the player (selected but scoped out of this phase)
- **Melodic variation responses** — Transforming user melody (inversion, retrograde, augmentation)
- **Rhythmic fills** — Drum-like percussive fills complementing user's rhythmic density
- **Style sub-variants per pattern** — e.g., walking bass vs root-fifth vs chromatic approach (keep single style for now, expand later)
- **Automatic meter/time-signature detection** — Detecting if player shifts from 4/4 to 3/4 feel
- **Genre auto-detection** — Detecting jazz/blues/classical from playing to auto-adjust accompaniment style

</deferred>

---

*Phase: 07-performance-mode*
*Context gathered: 2026-02-25*
