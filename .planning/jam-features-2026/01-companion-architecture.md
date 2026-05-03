# Companion Architecture

**Status:** Living document — rewritten 2026-05-04 (Sun) with state machine + Lane abstraction
**Supersedes:** `01-looper.md` (the original brief — kept for reference, scope expanded)
**Author:** Vibhav, drafted with Claude Code

---

## TL;DR

Promote **Companion** as the umbrella concept: an automated bandmate that plays alongside the user.

Implemented as a **state machine** with three primitives:
- `WorldState` — the observable. Holds transport, engine snapshot, held inputs, currently-sounding harmony, current chord, recent input window.
- `Lane` — a trait. The unit of decision-making. Each Lane declares an `input_filter`, runs in one of three phases (Sense / Mutate / Decide), reads `WorldState`, and emits `Vec<DispatchOp>`.
- `Companion` — the orchestrator. Holds the WorldState, owns a `Vec<Box<dyn Lane>>`, runs them in phase order on every router-loop tick and on every input event.

Every feature in the 9-week jam pipeline that emits MIDI in time fits as a Lane. Pattern (existing PR #89) becomes the first Lane impl. Looper, BeatMachine (Logic Drummer-style), Arpeggiator, Drone, Pad, ChordSeq, AutoKey all follow the same shape. Audio FX (Bitcrusher, Reverse, Shimmer, Distortion) stays in the existing `src/fx/` framework — separate from Companion.

Practical wins:
- Subsumes follow-up issue #91 (pure function extraction for testability) — `Lane::tick()` *is* the pure function.
- Subsumes the "press during off-cell still fires harmony" pattern bug — `handle_note_on` consults Companion via `on_input`, Lanes can `SuppressDefault`.
- 6/9 jam features fit cleanly as Lane impls. Audio FX features (3/9) sit safely outside.
- Multi-slot loops, beat machine, arp all become incremental Lane adds, not router-thread surgery.

Architectural debts surfaced this session, deferred to separate sessions:
1. **Chain rework** — `src/chain/` not architected the way the user wants. Investigate, redesign in dedicated session.
2. **Synth rework** — `src/synth/` similar concern. Investigate, redesign.
3. `panic_pending` sledgehammer — replace with typed `EngineMutation` enum once Wk 2 ChordSeq Lane lands.

---

## Why "Companion"?

Reading the user's framing of the feature: *a companion that plays stuff on time in the background based on what you're playing in the key you're playing in, combined with auto-key, gives you a tool that comps and helps you jam better*.

Pattern-as-rhythmic-gate is one expression of this. Loops-as-recorded-phrases is another. Auto-key as a passive "follow what I play" mode is a third. A Logic-Drummer-style smart drummer is a fourth. They share an audience (the soloing musician), a venue (the jam), and a tick model (the transport clock). Treating them as siblings under a `Companion` umbrella is the cleaner mental model and the cleaner code.

This naming is breaking: existing `pattern_*` Tauri commands, `PatternStore`, `PatternConfig`, etc. all get renamed under the companion namespace. We have no external customers, so this is a one-time migration cost.

---

## System architecture

```
        ┌───────────────────────────────────────────────────────────────┐
        │                         COMPANION                              │
        │                                                                │
   ┌────│──→ on_input(ev) ──→ Lanes ─→ ops ──┐                         │
   │    │   (handle_note_on,                  │                         │
   │    │    inject_note_on,                  ▼                         │
INPUTS  │    note_off, MIDI in)         dispatch_voice ─────────┐       │
keyboard│                                     ▲                  │       │
virtual │    ┌─── WorldState (observable) ───┴───────┐          ├─→ MIDI / synth out
MIDI in │    │  • transport      • engine_snapshot   │          │       │
   │    │    │  • held_inputs    • sounding_voices   │          │       │
   │    │    │  • current_chord  • recent_input_     │          │       │
   │    │    │                     window            │          │       │
   │    │    │                                       │          │       │
   │    │    │  ▲ all Lanes read                     │          │       │
   │    │    │  ▲ Sense + Mutate Lanes write         │          │       │
   │    │    └───────────────────────────────────────┘          │       │
   │    │                                     ▲                  │       │
   └────│──→ tick() ────────→ Lanes ────→ ops ┘                 │       │
        │   (every router-loop                                    │       │
        │    iteration)                                           │       │
        │                                                          │       │
        └──────────────────────────────────────────────────────────┘       │
                                                                            │
   transport tick ──────────────────────────────────────────────────────────┘
   (Arc<Transport>, sample-accurate, audio-thread-driven)
```

Two entry points into Companion: `on_input` (event-driven) and `tick` (clock-driven). Both consult the same WorldState. Both produce `Vec<DispatchOp>` which the router thread executes through `dispatch_voice`.

---

## WorldState — the observable

```rust
pub struct WorldState {
    /// Sample-accurate position. Read-only handle; updated by audio thread.
    pub transport: Arc<Transport>,

    /// Snapshot of HarmonyEngine config — read often, mutated rarely.
    /// Wrapped so reads are lock-free atomic loads where possible.
    /// Phase 1: just `Arc<Mutex<HarmonyEngine>>` — defer ArcSwap until profiling shows contention.
    pub engine_snapshot: Arc<Mutex<HarmonyEngine>>,

    /// Currently-held inputs. Keyed by MIDI note. Updated on every
    /// handle_note_on/off via Companion::on_input.
    pub held_inputs: Arc<Mutex<HashMap<u8, HeldInput>>>,

    /// Currently-sounding harmony voices (per input note → routing-aware list).
    /// Was `held_harmonies` in router-thread-local. Now owned by WorldState.
    pub sounding_voices: Arc<Mutex<HashMap<u8, Vec<HeldVoice>>>>,

    /// Detected current chord (from sounding voices). Updated by ChordDetect Sense Lane.
    /// Powers Wk 6 motif transpose, Wk 7 arp, Wk 9 pad.
    pub current_chord: Arc<Mutex<DetectedChord>>,

    /// Time-windowed input history for auto-key + motif detection.
    /// Bounded VecDeque; old entries pruned on insert.
    /// Phase 1: defer until AutoKey Sense Lane needs it (before Wk 6).
    pub recent_input_window: Arc<Mutex<VecDeque<InputEntry>>>,
}

pub struct HeldInput {
    pub note: u8,
    pub velocity: u8,
    pub channel: u8,
    pub pressed_at: Instant,
}

pub struct InputEntry {
    pub at: Instant,
    pub note: u8,
    pub velocity: u8,
}

pub struct DetectedChord {
    pub root: Option<u8>,
    pub quality: Option<ChordQuality>,
    pub display: String,  // human-readable, e.g. "Cmaj7"
}
```

**Phase 1 must build:** `transport`, `engine_snapshot`, `held_inputs`, `sounding_voices`, `current_chord`.
**Phase 1 defers:** `recent_input_window` (add when AutoKey rewrite needs it before Wk 6).

---

## Lane abstraction

```rust
pub trait Lane: Send + Sync {
    fn name(&self) -> &str;

    /// Phase the Lane participates in.
    /// Sense lanes write to WorldState (auto-key, chord-detect).
    /// Mutate lanes write engine state (chord seq).
    /// Decide lanes only emit ops (pattern, loops, arp, drone, beats).
    fn phase(&self) -> LanePhase;

    /// What input events this Lane wants to handle. Inputs not matched by
    /// any Lane fall through to default harmonize. Enables split-keyboard
    /// / live-channel routing.
    fn input_filter(&self) -> InputFilter { InputFilter::None }

    /// Called once per router-loop iteration with current WorldState.
    fn tick(&mut self, world: &WorldState) -> LaneOutput;

    /// Called when an input event arrives that this Lane's filter matched.
    /// Lane can suppress dispatch (return SuppressDefault), modify, or pass through.
    fn on_input(&mut self, ev: InputEvent, world: &WorldState) -> LaneOutput {
        LaneOutput::default()  // default: do nothing on input
    }
}

pub enum LanePhase {
    Sense,    // updates WorldState (e.g. auto-key writes detected key)
    Mutate,   // mutates HarmonyEngine state (e.g. chord seq sets key/mode on bar)
    Decide,   // emits dispatch ops only (pattern, loops, arp, drone, beats, pad)
}

pub enum InputFilter {
    None,                                   // tick-only Lane (Drone, Pad — never claims input)
    All,                                    // grabs everything (Pattern v1 default)
    NoteRange(u8, u8),                      // split-keyboard mode (chord register only)
    Channel(u8),                            // live-channel routing
    Predicate(Box<dyn Fn(&InputEvent) -> bool + Send + Sync>),
}

pub struct LaneOutput {
    pub ops: Vec<DispatchOp>,
    pub engine_mutations: Vec<EngineMutation>, // only used by Mutate-phase lanes
    pub world_writes: Vec<WorldWrite>,         // only used by Sense-phase lanes
    pub suppress_default: bool,                // input-event only: skip default harmonize
}

pub enum DispatchOp {
    NoteOn { target: VoiceOutputTarget, note: u8, velocity: u8, channel: u8 },
    NoteOff { target: VoiceOutputTarget, note: u8, channel: u8 },
    AllNotesOff { ports: Vec<u8> },
}

pub enum EngineMutation {
    SetKey(Key),
    SetMode(HarmonyMode),
    SetScale(ScaleMode),
    SetVoiceLeading(VoiceLeadingStyle),
}

pub enum WorldWrite {
    UpdateChord(DetectedChord),
    UpdateDetectedKey(Key),
    UpdateDetectedMode(HarmonyMode),
}

pub enum InputEvent {
    NoteOn { note: u8, velocity: u8, channel: u8 },
    NoteOff { note: u8, channel: u8 },
    Cc { number: u8, value: u8, channel: u8 },  // for sustain pedal etc.
}
```

### Phase ordering — why it matters

```
   PHASE 1: SENSE        ─→ writes WorldState
   ─────────────────       (engine_snapshot, current_chord, recent inputs)
   AutoKey Lane
   ChordDetect Lane
                                    │
                                    ▼
   PHASE 2: MUTATE       ─→ writes HarmonyEngine state
   ─────────────────       (key, mode, scale)
   ChordSeq Lane
                                    │
                                    ▼
   PHASE 3: DECIDE       ─→ emits dispatch ops
   ─────────────────       (NoteOn, NoteOff, AllNotesOff)
   Pattern Lane
   LoopSlot Lane × N
   Arpeggiator Lane
   Drone Lane
   AmbientPad Lane
   BeatMachine Lane
```

**Concrete example of why ordering matters:** AutoKey detects you're vibing in D minor (Sense). Engine snapshot now reflects the new key. ChordSeq's "next chord" lookup uses the updated key (Mutate). Drone Lane reads the new tonic from the engine snapshot and emits a low D drone (Decide). Each lane sees a coherent world.

Without phase ordering: Drone might read stale key while AutoKey is mid-update; ChordSeq fires harmony in the old key. Race conditions become latent musical glitches.

### Input filter — split keyboard / live channel

The user wants to play live melody while pattern + loops + drone + beats run in the background. Without `input_filter`, every Lane sees every press, and the Pattern Lane will try to gate the live melody just like the chord-register inputs.

```
                        on_input(ev)
                             │
                             ▼
   ┌─ for each Lane ───────────────────────────────────┐
   │  if lane.input_filter().matches(&ev):             │
   │      lane.on_input(ev, &world)                    │
   │          → ops + maybe SuppressDefault            │
   └────────────────────────────────────────────────────┘
                             │
              if no Lane SuppressDefault
                             ▼
              HarmonyEngine.harmonize_note_on(note)
              → voices → dispatch_voice
              → WorldState.sounding_voices.update
              → next-tick Sense lanes pick up the change
```

Default Lane filters:
- **PatternLane**: `All` (today's behavior). User can configure to `NoteRange(C2..B3)` for split-keyboard.
- **LooperLane (Input source)**: `All` (records anything user plays).
- **LooperLane (Output source)**: `None` (taps engine output, not input).
- **ArpLane**: `NoteRange(C2..B3)` default (chord register).
- **DroneLane**: `None` (tick-only).
- **PadLane**: `None` (tick-only).
- **BeatMachineLane**: `None` (tick-only).
- **AutoKeyLane**: writes via `world_writes`; doesn't suppress (lets harmony fire normally).

---

## Companion orchestrator

```rust
pub struct Companion {
    pub enabled: AtomicBool,
    pub world: Arc<WorldState>,
    pub lanes: Vec<Box<dyn Lane>>,  // sorted by phase: Sense → Mutate → Decide
}

impl Companion {
    pub fn tick(&mut self, engine: &Mutex<HarmonyEngine>) -> Vec<DispatchOp> {
        if !self.enabled.load(Ordering::Acquire) { return vec![]; }

        // Phase 1: Sense lanes update WorldState.
        for lane in self.lanes.iter_mut().filter(|l| l.phase() == LanePhase::Sense) {
            let out = lane.tick(&self.world);
            self.apply_world_writes(out.world_writes);
        }

        // Phase 2: Mutate lanes change engine state.
        for lane in self.lanes.iter_mut().filter(|l| l.phase() == LanePhase::Mutate) {
            let out = lane.tick(&self.world);
            for m in out.engine_mutations { engine.lock().unwrap().apply(m); }
        }

        // Phase 3: Decide lanes emit dispatch ops.
        let mut all_ops = Vec::new();
        for lane in self.lanes.iter_mut().filter(|l| l.phase() == LanePhase::Decide) {
            let out = lane.tick(&self.world);
            all_ops.extend(out.ops);
        }
        all_ops
    }

    pub fn on_input(&mut self, ev: InputEvent, engine: &Mutex<HarmonyEngine>) -> CompanionInputResult {
        // Update WorldState.held_inputs (if NoteOn/NoteOff).
        self.update_held(&ev);

        let mut suppress_default = false;
        let mut ops = Vec::new();

        // Run only Lanes whose filter matches.
        for lane in self.lanes.iter_mut() {
            if !lane.input_filter().matches(&ev) { continue; }
            let out = lane.on_input(ev.clone(), &self.world);
            ops.extend(out.ops);
            if out.suppress_default { suppress_default = true; }
            // Sense + Mutate writes apply same as in tick().
            self.apply_world_writes(out.world_writes);
            for m in out.engine_mutations { engine.lock().unwrap().apply(m); }
        }

        CompanionInputResult { ops, suppress_default }
    }
}
```

---

## Input pipeline (companion-mediated)

```rust
fn handle_note_on(state: &AppState, note: u8, velocity: u8, channel: u8, ...) {
    // 1. Notify Companion (updates WorldState, runs Lanes that filter-match).
    let result = state.companion.lock().unwrap().on_input(
        InputEvent::NoteOn { note, velocity, channel },
        &state.engine,
    );

    // 2. Apply Lane-emitted ops (loop captures, arp pattern, etc.).
    for op in result.ops { dispatch_voice(op, ...); }

    // 3. Default harmonize unless a Lane suppressed it.
    if !result.suppress_default {
        let voices = state.engine.lock().unwrap().harmonize_note_on(note);
        for (i, v) in voices.iter().enumerate() {
            dispatch_voice(DispatchOp::NoteOn { target: target_for(i), note: u8::from(*v), velocity, channel }, ...);
        }
        // Update WorldState.sounding_voices.
        state.world.sounding_voices.lock().unwrap().insert(note, voices_with_targets);
    }
}
```

This fixes:
- **P1 (the press-during-off-cell bug)**: PatternLane.on_input checks current cell — if off, returns `SuppressDefault=true`. Press during off-cell does not fire harmony for Live or Gated mode.
- **Looper capture**: LooperLane.on_input pushes the press into its capture buffer in addition to default harmonize.
- **Arpeggiator behavior**: ArpLane.on_input returns `SuppressDefault=true` and keeps the held chord internally; emits arp pattern on tick.
- **Live melody on top**: PatternLane filtered to chord register, melody-register notes don't match any filter, fall through to default harmonize unchanged.

---

## Lane catalog

### PatternLane (Phase 1 — refactor of existing PR #89 logic)
**Phase**: Decide. **Filter**: `All` default; configurable to `NoteRange(C2..B3)` for split-keyboard.
- Reads: transport.totalBeat, sounding_voices (the dedupe set), current_cell index
- Emits: NoteOn/NoteOff per held harmony voice on cell boundaries
- Modes: Live (staccato retrigger), Gated (legato, edges only)
- Existing F2/F4/F5/M3/H3 invariants port over; new P1 invariant added.

### LooperLane × N (Phase 2 — Wk 1)
**Phase**: Decide. **Filter** depends on source: `All` for Input source (captures user's notes), `None` for Output source (taps engine output via observation of sounding_voices on tick).
- Slot lifecycle: Empty → Armed (waiting next bar) → Recording → Playing / Stopped
- Replay paths: Input source re-enters via dispatch_voice → harmony engine; Output source emits direct dispatch ops
- L1-L5 invariants from previous draft.
- N configurable. `Vec<LoopSlot>` so adding slots is one operation.

### MotifTransposerLane (before Wk 6)
Extends LooperLane. Adds a transpose semitone control + reads `world.current_chord` to auto-fit a transposed loop to current chord. Auto-fit-to-chord descopable per Wk 6 brief.

### ChordSeqLane (before Wk 2)
**Phase**: Mutate. **Filter**: `None`.
- Reads: transport.totalBeat (current bar)
- Emits: `EngineMutation::SetKey(...)` + `SetMode(...)` on bar boundary, cycling through user-typed progression (`Am F C G`)
- This is the first Mutate-phase Lane. Validates the EngineMutation enum.

### DroneLane (before Wk 3)
**Phase**: Decide. **Filter**: `None`.
- Reads: engine_snapshot.tonic, transport (only to know we're running)
- Emits: sustained NoteOn at tonic to configured voice (synth or external port)
- Tracks "is currently emitting" to fire NoteOff on disable / tonic change.

### ArpeggiatorLane (before Wk 7)
**Phase**: Decide. **Filter**: `NoteRange(C2..B3)` default.
- Reads: held_inputs (the chord), transport.totalBeat (subdivisions), current_chord, ARP config (pattern, octaves, rate)
- Emits: NoteOn/NoteOff per arp step
- on_input: returns `SuppressDefault=true` and stores the chord notes; tick emits the arp.
- Sustain pedal handled at input pipeline filter level (not a Lane — see "Input filters").

### AmbientPadLane (Wk 9)
**Phase**: Decide. **Filter**: `None`.
- Reads: engine_snapshot.key, transport.totalBeat (for slow morph timing)
- Emits: slowly-evolving polyphonic NoteOns; morphs between 2-3 pad presets over 8-16 bars
- Could share infrastructure with DroneLane.

### AutoKeyLane (Sense — before Wk 6)
**Phase**: Sense. **Filter**: `All` (observes inputs; doesn't claim — non-suppressing).
- Reads: recent_input_window
- Writes: `WorldWrite::UpdateDetectedKey(...)` + `UpdateDetectedMode(...)` based on Krumhansl scale fitting (issue #81)
- Hysteresis to prevent flipping keys per note.
- Replaces the current `set_auto_key` AtomicBool + set_auto_key panic_pending logic.

### ChordDetectLane (Sense — Phase 1)
**Phase**: Sense. **Filter**: `None`.
- Reads: sounding_voices
- Writes: `WorldWrite::UpdateChord(DetectedChord)` based on currently-sounding pitches
- Replaces the existing `chord_name: Arc<Mutex<String>>` update site, lifts it to WorldState.

### **BeatMachineLane (Logic Drummer-style — major feature, see dedicated section)**

---

## BeatMachine — Logic Drummer-style smart drummer

**User vision**: Logic Pro's Drummer behavior. *Not* a basic step sequencer — a smart adaptive drummer with style presets, intensity controls, and auto-fills.

### Behavior model

```
   ┌─ Drummer presets ──────────────────────────────────────────┐
   │  Rock       Soul        Hip-Hop      Electro              │
   │  Songwriter Jazz/Brush  Latin        Indie                 │
   │  (start with 3-5; expand later)                            │
   └────────────────────────────────────────────────────────────┘

   ┌─ Intensity X/Y pad (live UI control) ──────────────────────┐
   │   Y axis: Soft ←──────→ Loud  (velocity, density)          │
   │   X axis: Simple ←────→ Complex (pattern busyness, fills)  │
   │                                                             │
   │   User drags the puck around during the jam — drummer       │
   │   adapts the pattern in real time. Quantized to bar         │
   │   boundaries so transitions feel musical.                   │
   └────────────────────────────────────────────────────────────┘

   ┌─ Auto-fills ───────────────────────────────────────────────┐
   │   Every N bars (4, 8, 16) the drummer plays a fill          │
   │   variant of the current pattern. Fill style adapts to      │
   │   intensity X (simple → minimal; complex → busy fill).     │
   └────────────────────────────────────────────────────────────┘

   ┌─ Drum kit selector ────────────────────────────────────────┐
   │   Acoustic (default)  Electronic  Hybrid  Brushes  ...     │
   │   Each kit = a sample bank loaded into the internal         │
   │   drum sampler.                                             │
   └────────────────────────────────────────────────────────────┘

   ┌─ Per-element overrides (advanced) ─────────────────────────┐
   │   Kick: busy / sparse                                       │
   │   Snare: ghost notes / rim / accents                        │
   │   Hat: closed-only / open-on-2&4 / pumping                  │
   │   Percussion: on / off / fills-only                         │
   │   These are bias offsets layered on top of the preset.      │
   └────────────────────────────────────────────────────────────┘
```

### Data model

```rust
pub struct BeatMachineLane {
    pub enabled: bool,
    pub preset: DrummerPreset,    // Rock / Soul / Jazz / etc
    pub kit: KitSelector,          // Acoustic / Electronic / Hybrid
    pub intensity_x: f32,          // 0.0 simple → 1.0 complex
    pub intensity_y: f32,          // 0.0 soft → 1.0 loud
    pub fill_every_bars: u8,       // 0=disabled, 4, 8, 16
    pub element_overrides: ElementOverrides,
    pub follow_target: Option<String>,  // Lane name to lock groove with (advanced)
    target: VoiceOutputTarget,    // routing for drum events (drum sampler or external)
}

pub struct ElementOverrides {
    pub kick: ElementBias,    // -1.0 sparse → 0.0 default → +1.0 busy
    pub snare: ElementBias,
    pub hat: ElementBias,
    pub perc: ElementBias,
}

pub struct DrummerPreset {
    pub name: String,                      // "Rock", "Hip-Hop", etc.
    pub patterns: PatternBank,             // pre-authored patterns at 5 intensity levels
    pub fills: FillBank,                   // pre-authored fills at 5 intensity levels
    pub style_constraints: StyleSettings,  // swing, push/pull, dynamics
}

pub struct PatternBank {
    /// 5 intensity levels × 4 element tracks (kick, snare, hat, perc)
    /// Each cell = (active, velocity, accent).
    pub levels: [PatternLevel; 5],  // intensity 0.0-0.2, 0.2-0.4, ... 0.8-1.0
}

pub struct PatternLevel {
    pub kick: Vec<DrumCell>,
    pub snare: Vec<DrumCell>,
    pub hat: Vec<DrumCell>,
    pub perc: Vec<DrumCell>,
}

pub struct DrumCell { pub on: bool, pub velocity: u8, pub accent: bool }
```

### Lane behavior

`BeatMachineLane::tick(world)` per iteration:
1. Read transport.totalBeat → current cell within the pattern.
2. Determine current bar within the phrase (for auto-fill detection).
3. Look up pattern for this cell, given current preset + intensity:
   - Interpolate between adjacent intensity levels (e.g. intensity_x = 0.65 → blend levels[2] + levels[3]).
   - At fill boundary (every N bars), substitute fill pattern.
4. Apply per-element overrides (bias kick busier, snare sparser, etc.).
5. Apply intensity_y (overall velocity multiplier).
6. Emit DispatchOp::NoteOn for each track that fires this cell.

### Build phases for BeatMachine

This is multi-phase work — significantly more than a step sequencer.

| Phase | What | Effort |
|---|---|---|
| **A. DrumSampler subsystem** | Internal Rust sampler with bundled kit (see next section). | 2-3 d |
| **B. BeatMachineLane skeleton + step-sequencer behavior** | Lane impl, basic per-track cell grid, no presets/intensity yet. Validates Lane abstraction with a 2nd impl beyond Pattern. | 1-2 d |
| **C. Pattern library authoring** | 3-5 drummer presets × 5 intensity levels × kick/snare/hat/perc. MIDI-style data. Done by hand initially. | 2-3 d |
| **D. Intensity X/Y interpolation + auto-fills** | The "smart drummer" logic. Pattern selection by intensity, fill substitution at boundaries. | 2-3 d |
| **E. UI: BeatsTab in CompanionPanel** | X/Y intensity pad, preset picker, kit picker, per-element overrides. | 2 d |
| **F. Polish + edge cases** | Mute/solo per element, follow-target wiring, smooth transitions. | 1-2 d |

**Total: 10-15 days** for v1 BeatMachine. Significantly more than fitting into a single jam-week slot.

### Where this lands in the schedule

The 9-week pipeline doesn't have a BeatMachine. Insertion options:

| Option | Detail | Tradeoff |
|---|---|---|
| **Build alongside Phase 1 foundation** | Phase B only (basic step seq Lane). Validates Lane abstraction with 2 impls. v1 (full Drummer) lands later. | Phase 1 grows; "step sequencer for drums" might confuse users vs the promised Drummer experience |
| **Replace Wk 7 Arpeggiator** | Slot the BeatMachine into Wk 7. Arp pushes to a later patch week. | Lose arp until later |
| **New week between existing weeks** | E.g., Wk 3.5 BeatMachine. Pushes Wk 4-9 by half a week. | 9-week pipeline becomes 9.5 |
| **Post-jam dedicated cycle** | Build it properly after Wk 9 ends. | Loses jam demo opportunity |

User decision needed. Recommendation in dedicated decisions section below.

---

## Drum Sampler subsystem

**Decision locked**: internal Rust drum sampler. Self-contained, works in browser via WASM, no external dependency.

### Scope

```rust
// src/synth/drum_sampler.rs (NEW)
pub struct DrumSampler {
    pub kit: DrumKit,        // sample bank (kick, snare, hat, …)
    pub voices: Vec<Voice>,  // polyphonic voice allocator
    pub master_gain: f32,
}

pub struct DrumKit {
    pub name: String,
    pub samples: HashMap<u8 /* MIDI note */, DrumSample>,
}

pub struct DrumSample {
    pub buffer: Arc<Vec<f32>>,    // mono PCM at engine sample rate
    pub original_velocity: u8,    // for velocity-scaled multisamples (later)
    pub envelope: SampleEnvelope, // ADSR for natural release
}
```

### Default kit content (bundled)

- 5 elements minimum: kick, snare, closed hi-hat, open hi-hat, ride bell
- 8 elements ideal: + crash, tom, perc/clap
- Sourced from CC0 / public domain libraries (avoid licensing issues at jam time)
- Bundled at compile time via `include_bytes!` for native; downloaded once for WASM

### Dispatch path

BeatMachineLane emits `DispatchOp::NoteOn { target: VoiceOutputTarget::DrumSampler, note: 36 /* kick */, velocity: 100, ... }`.

Adds a new `VoiceOutputTarget::DrumSampler` variant alongside `Synth`, `MidiPort`, `Off`. The existing `dispatch_voice` helper extends with one new match arm.

### Phase

Phase A above (2-3 days). Lands before BeatMachineLane Phase B.

---

## Audio FX framework — out of scope for Companion

`src/fx/` and `src/chain/` already do this right. Bitcrusher (Wk 3), Reverse delay (Wk 4), Shimmer (Wk 4), Distortion (Wk 8) all extend that framework. **Don't conflate audio sample-rate processing with MIDI Lanes** — different cycles, different constraints, different abstractions.

(See "Architectural debts" section: chain + synth need their own rework. Investigate in a separate session.)

---

## Pure-function contracts (Lane invariants)

### Pattern invariants (existing, ported from PR #89)

- **F2**: ops emitted by pattern lane do not double-fire when polyphonic input produces overlapping harmonies.
- **F4**: when pattern lane re-fires a held voice with new routing, the old voice gets a NoteOff op before the new voice's NoteOn op.
- **F5**: when a setter raises `panic_pending` and the pattern lane was about to fire on this tick, the panic-replay's `to_release` set covers the pattern-attacked notes; pattern lane skips this tick.
- **M3**: the first tick after `companion.enabled` flips true seeds `last_pattern_cell` without firing — phase-alignment.
- **H3**: pattern lane skipped when `panic_pending` on this tick.

### New: input pipeline invariant

- **P1**: `handle_note_on` consults Companion before default harmonize. If a Lane returns `suppress_default=true`, default `harmonize_note_on` does not fire. PatternLane.on_input returns suppress_default=true when current cell is off in Live or Gated mode. Press during off-cell does not fire harmony.

### Looper invariants

- **L1**: a slot in `Recording` captures only events whose `beat_offset` is within `[0, length_beats)`. Events past the boundary close out the buffer and transition to `Playing`.
- **L2**: a slot in `Playing` emits ops with `beat_offset == (transport.totalBeat - recorded_at_bar*beats_per_bar) mod length_beats` for each event matching this tick's window.
- **L3**: a slot in `Stopped` emits a single `AllNotesOff` op on transition.
- **L4**: a slot transition `Empty → Armed` schedules the recording to start at the next bar boundary.
- **L5**: `LoopSource::Output` slots, on replay, emit ops that bypass the pattern lane. `LoopSource::Input` slots emit ops that re-enter the pattern lane.

### BeatMachine invariants (TBD when phase B lands)

- **B1**: tick emits at most one NoteOn per element per cell.
- **B2**: intensity X interpolation is monotonic (intensity 0.5 produces "between level 2 and level 3" patterns, never something outside that range).
- **B3**: auto-fill substitutes the *next* bar's pattern when fill boundary is crossed; the bar after that returns to base pattern.
- **B4**: on disable, all currently-sounding drum notes get NoteOff (no stuck snare).

All invariants get unit tests. Pure-function shape makes them reachable without standing up the router thread.

---

## Naming migration

### Rust

| Old | New |
|---|---|
| `state.rs::PatternConfig` | `companion/pattern.rs::CompanionPattern` ✅ done in 1.2 |
| `state.rs::PatternInputMode` | `companion/pattern.rs::CompanionInputMode` ✅ done in 1.2 |
| `AppState::pattern_config` | `AppState::companion: Arc<Companion>` (Phase 1) |
| `AppState::pattern_enabled: AtomicBool` | `Companion::enabled: AtomicBool` (Phase 1) |
| `commands/engine.rs::set_pattern_enabled` | `commands/companion.rs::set_companion_enabled` |
| `commands/engine.rs::set_pattern_config` | `commands/companion.rs::set_companion_pattern` |
| `commands/engine.rs::set_auto_key` | `commands/companion.rs::set_companion_auto_key` |
| (new) | `commands/companion.rs::set_companion_loop_arm` |
| (new) | `commands/companion.rs::set_companion_loop_stop` |
| (new) | `commands/companion.rs::set_companion_loop_clear` |
| (new) | `commands/companion.rs::set_companion_loop_count` |
| (new) | `commands/companion.rs::set_companion_beat_intensity` |
| (new) | `commands/companion.rs::set_companion_beat_preset` |
| (new) | `commands/companion.rs::set_companion_beat_kit` |
| `pattern_config_tests` | `companion_pattern_tests` ✅ done in 1.2 |

### TS

| Old | New |
|---|---|
| `lib/stores/pattern.svelte.ts::PatternStore` | `lib/stores/companion.svelte.ts::CompanionStore` (umbrella, with `companion.pattern`, `companion.loops`, `companion.beats`, `companion.autoKey`) |
| `lib/components/PatternPanel.svelte` | `lib/components/companion/CompanionPanel.svelte` (umbrella) + `companion/PatternTab.svelte`, `companion/LoopsTab.svelte`, `companion/BeatsTab.svelte`, `companion/AutoKeyTab.svelte` |
| Adapter `setPatternConfig` | `setCompanionPattern` |
| Adapter `setPatternEnabled` | `setCompanionEnabled` |
| Adapter (new) | `armCompanionLoop(slotId, source, lengthBars)` etc. |
| Adapter (new) | `setCompanionBeatIntensity(x, y)`, `setCompanionBeatPreset(name)`, etc. |

### localStorage keys

| Old | New |
|---|---|
| `'contrapunk-pattern'` | `'contrapunk-companion'` (one blob: `{enabled, pattern, loops, beats, autoKey}`) |

Migration shim on first hydrate after the rename: read `'contrapunk-pattern'` if present, copy into the new shape under `companion.pattern`, delete old key.

---

## UI surface

```
┌─ Status bar pip row ──────────────────────────┐
│  ◉  Transport      ◉  Companion (master)      │
└───────────────────────────────────────────────┘

┌─ CompanionPanel ──────────────────────────────────┐
│  [ ▣ Pattern ] [ Loops ] [ Beats ] [ Auto-Key ]   │
│  ───────────────────────────────────────────────  │
│  PatternTab — existing PatternPanel content       │
│    subdivision, length, input mode, cells         │
│    + new: input range selector (split-keyboard)   │
└───────────────────────────────────────────────────┘

┌─ CompanionPanel ──────────────────────────────────┐
│  [ Pattern ] [ ▣ Loops ] [ Beats ] [ Auto-Key ]   │
│  ───────────────────────────────────────────────  │
│  Slot count: [ 1 | 2 | 3 | ▣ 4 | + Add ]          │
│  Per-slot: capture toggle, length, state, record  │
└───────────────────────────────────────────────────┘

┌─ CompanionPanel ──────────────────────────────────┐
│  [ Pattern ] [ Loops ] [ ▣ Beats ] [ Auto-Key ]   │
│  ───────────────────────────────────────────────  │
│  Drummer:  [ Rock ▼ ]    Kit: [ Acoustic ▼ ]     │
│                                                    │
│  ┌─ Intensity ──────┐   ┌─ Element overrides ─┐   │
│  │     Loud          │   │  Kick    [─●─────]  │   │
│  │  ┌────●─────┐    │   │  Snare   [───●───]  │   │
│  │  │          │     │   │  Hat     [─────●─]  │   │
│  │  │  X/Y pad │     │   │  Perc    [─●─────]  │   │
│  │  │ (drag)   │     │   └─────────────────────┘   │
│  │  └──────────┘     │                              │
│  │     Soft          │   Auto-fill every: [ 8 ▼ ]  │
│  │  Simple ←→ Complex│                              │
│  └───────────────────┘                              │
└───────────────────────────────────────────────────┘

┌─ CompanionPanel ──────────────────────────────────┐
│  [ Pattern ] [ Loops ] [ Beats ] [ ▣ Auto-Key ]   │
│  ───────────────────────────────────────────────  │
│  ☑ Enable auto-detect                              │
│  Detected: D minor (confidence 0.84)               │
│  Hysteresis: ●●●○○ (medium — 3 changes/min cap)    │
└───────────────────────────────────────────────────┘
```

**MIDI Learn integration** (PR #88): each Lane exposes its key actions as discoverable knob targets. Stable command names (`companion_loop_arm_slot_{N}`, `companion_beat_intensity_x`, `companion_pattern_enable`) so MPK pads / knobs map persistently. The MIDI learn map regenerates when slot count or beat configuration changes.

---

## WASM parity

Browser-side companion lives in `wasm/src/lib.rs` mirroring the native shape. JS-side stores LoopBuffer in a `Map<slotId, LoopBuffer>` since the WASM engine doesn't have its own router thread (events flow synchronously through the wasm engine's `note_on`/`note_off` calls).

For the BeatMachine, the drum sampler runs in WASM the same as native (Rust → WASM compile). Sample bank streamed once on first load; cached via Service Worker.

Audio FX in WASM requires WebAudio primitives, which is its own concern — covered in the chain/synth rework session.

---

## Feature → architectural layer mapping

```
   FEATURE                          LAYER                    LANDS IN

   Pattern (current PR #89)         Decide Lane              Phase 1 (refactor)

   Wk 1: Looper (input + output)    Decide Lane × N          Phase 2

   Wk 2: Chord progression seq      Mutate Lane              Before Wk 2

   Wk 3: Drone                      Decide Lane              Before Wk 3
   Wk 3: Bitcrusher                 src/fx/  (audio FX)      no companion change

   Wk 4: Reverse delay              src/fx/                  no companion change
   Wk 4: Shimmer reverb             src/fx/                  no companion change

   Wk 5: Exotic scales              pure scale data          no companion change

   Wk 6: Motif transposer           extends Looper Lane      Before Wk 6
   Wk 6: auto-fit-to-chord          reads current_chord      (foundation already there)

   Wk 7: Arpeggiator                Decide Lane              Before Wk 7
   Wk 7: Sustain pedal              input pipeline filter    small input-layer change

   Wk 8: Distortion                 src/fx/                  no companion change
   Wk 8: Power-chord mode           HarmonyEngine option     one engine flag
   Wk 8: Drop-tune preset           guitar_pipeline config   subsystem-local

   Wk 9: Ambient pad                Decide Lane              Before Wk 9

   AutoKey rewrite (issue #81)      Sense Lane               Before Wk 6
                                    + recent_input_window    (motif transpose dep)

   BEAT MACHINE (Logic Drummer)     Decide Lane              See dedicated section
   DRUM SAMPLER subsystem            src/synth/drum_sampler   Independent of Lanes

   ─────────────────────────────────────────────────────────────────────────
   FEATURES NEEDING THE COMPANION ARCHITECTURE: 1, 2, 3, 6, 7, 9 + Beat = 7
   FEATURES SAFELY OUTSIDE COMPANION:           4, 5, 8 sub-pieces       = 3
```

---

## Phase plan (revised)

| Phase | What | Effort | Status |
|---|---|---|---|
| **0** | Pre-flight gate (cargo test + UI check + manual UAT) | 45 min | ✅ automated done; manual UAT pending user |
| **1.1** | Companion module skeleton, type stubs | 1 hr | ✅ done (`2ca0291`) |
| **1.2** | Move PatternConfig into companion/pattern.rs as CompanionPattern | 30 min | ✅ done (`19053a6`) |
| **1.3** | Update consumers, drop migration aliases | 30 min | ✅ done (`b4f8940`) |
| **1.4** | Define `WorldState`, move held trackers from router thread | 1 d | next |
| **1.5** | `Lane` trait + `Companion` orchestrator + phase ordering | 1 d | |
| **1.6** | PatternLane impl wrapping existing logic + tests | 1 d | |
| **1.7** | Companion-mediated input pipeline (P1 fix for press-during-off-cell) | 0.5 d | |
| **1.8** | TS adapter rename + frontend store split (CompanionStore umbrella) | 1 d | |
| **1.9** | localStorage migration shim, Tauri command rename | 0.5 d | |
| **2** | LooperLane: single + multi-slot, both Input/Output sources | 2-3 d | |
| **3** | WASM parity for Pattern + Looper | 1 d | |
| **A** | DrumSampler subsystem (internal Rust sampler + bundled kit) | 2-3 d | |
| **B** | BeatMachineLane skeleton (basic step-seq behavior) | 1-2 d | |
| **C** | Drummer pattern library (3-5 presets × 5 intensity levels) | 2-3 d | |
| **D** | Intensity X/Y interpolation + auto-fills | 2-3 d | |
| **E** | UI: BeatsTab in CompanionPanel | 2 d | |
| **F** | BeatMachine polish + edge cases | 1-2 d | |
| **5** | UI consolidation: CompanionPanel umbrella with all tabs | 1-2 d | |
| **6** | ChordSeqLane (Wk 2) | 1-2 d | |
| **7** | DroneLane (Wk 3) | 0.5-1 d | |
| **8** | AutoKeyLane (Sense) rewrite — issue #81 | 2 d | |
| **9** | MotifTransposerLane (Wk 6) | 1-2 d | |
| **10** | ArpeggiatorLane + sustain pedal (Wk 7) | 1-2 d | |
| **11** | AmbientPadLane (Wk 9) | 1 d | |

**Total: 26-37 days** for everything in this doc. Note: Phases A-F (BeatMachine + DrumSampler) alone are 10-15 days — a major component.

The user has confirmed "no shortcuts, all phases." Timeline is the user's call — this doc just states the realistic effort.

---

## Decisions locked

1. ✅ Companion as umbrella concept (pattern + loops + auto-key + beats as limbs)
2. ✅ Both Input + Output loop sources, per-slot toggle
3. ✅ N configurable slots
4. ✅ Pre-flight gate before Phase 1
5. ✅ Architecture artifact (this document)
6. ✅ Full rename (Tauri commands + types + storage keys + files), no backwards-compat shim except one-shot localStorage migration
7. ✅ State machine with Lane trait + WorldState + 3-phase orchestration
8. ✅ Lanes declare `input_filter` for split-keyboard / live-channel
9. ✅ `handle_note_on` consults companion → fixes press-during-off-cell bug (P1 invariant)
10. ✅ BeatMachine = Logic Drummer-style smart drummer (NOT a basic step sequencer)
11. ✅ Internal Rust drum sampler (NOT external GM MIDI)
12. ✅ Audio FX framework (`src/fx/`) stays separate from Companion

---

## Open questions

1. **BeatMachine schedule**: build alongside Phase 1 (Phase B basic only), replace Wk 7 Arp, insert Wk 3.5, or post-jam? — recommend: alongside Phase 1 with Phase B (validates Lane abstraction), Phase C-F lands later as dedicated feature.
2. **Recording start**: arm-and-wait-for-next-bar (recommend) or start immediately on Record press?
3. **Default loop slot count on first run**: 1, 2, or 4?
4. **MIDI Learn binding for per-slot loop arm/stop on day 1** or punt to polish phase?
5. **Visual loop indicator on piano during playback** in Phase 2 polish or later?
6. **AutoKey tab scope**: just the existing toggle, or also surface key/mode confidence + visualizations?
7. **Drum kit selection in v1 BeatMachine**: ship one kit (Acoustic), or 3 (Acoustic / Electronic / Brushes)?

---

## Architectural debts (out of scope for this doc; flagged for separate sessions)

1. **Chain rework** — `src/chain/` is not architected the way the user wants. Investigate, redesign in dedicated session. Likely impacts how voice routing, FX insertion, and plugin hosting compose.

2. **Synth rework** — `src/synth/` similar. Current synth is tonal; adding the DrumSampler exposes how the synth subsystem extends. Probably needs a `Synth` trait or a multi-source mixer, not a hard-coded single voice generator.

3. **`panic_pending` is a sledgehammer** — every engine setter triggers full reharm. Adds a Lane → adds another consumer that has to survive the panic-replay. Replace with typed `EngineMutation` enum (Phase 6 ChordSeqLane is the natural home for this work).

4. **Issue #90 (held_harmonies stale-entry recovery)** — defensive `CC123 AllNotesOff` ops in Companion-emitted state changes mitigate the symptom for the new code. Root fix (TTL or cross-reference) tracked separately.

5. **Audio thread lock contention** — `Mutex<HarmonyEngine>` reads on the Decide-phase hot path. Profile after Phase 5; if contention shows, swap for `ArcSwap<EngineSnapshot>` and have engine setters publish snapshots.

6. **Router thread testability** — Phase 1 unit-tests Lane impls against the trait. Integration tests for the *router thread itself* (spawn router + scripted MIDI input + assert dispatched events) tracked as a follow-up to issue #91.

---

## Acceptance criteria (Phase end-state)

- [ ] All planned Lanes shipped (Pattern, Looper × N, BeatMachine, ChordSeq, Drone, Pad, Arp, AutoKey, MotifTransposer)
- [ ] Drum sampler ships with at least one bundled kit
- [ ] BeatMachine has 3+ Drummer presets, intensity X/Y, auto-fills
- [ ] Live-play-on-top works: chord-register patterns + melody-register live notes coexist
- [ ] No regressions in existing pattern behavior (validated by lockstep test + visual comparison against pre-rename behavior)
- [ ] Native + WASM parity (everything in this doc works in `app.contrapunk.com`)
- [ ] No stuck notes on Lane state changes (defensive CC 123 broadcast)
- [ ] Per-slot/per-Lane MIDI Learn discoverable
- [ ] Companion master toggle in StatusBar replaces the panel-pip pattern
- [ ] AutoKey functionality preserved + improved with Krumhansl-style detection
- [ ] All Lane invariants (P1, F2-H3, L1-L5, B1-B4) covered by unit tests

---

## References

- `01-looper.md` — original brief, superseded
- `.planning/phases/bpm-clock/bpm-clock-LEARNINGS.md` — pattern infra learnings, F2-H3 invariants
- Issue [#81](https://github.com/contrapunk-audio/contrapunk/issues/81) — Auto-key Krumhansl detection (becomes AutoKeyLane)
- Issue [#90](https://github.com/contrapunk-audio/contrapunk/issues/90) — `held_harmonies` stale-entry recovery
- Issue [#91](https://github.com/contrapunk-audio/contrapunk/issues/91) — router-loop pure-function extraction (subsumed by Phase 1)
- `.planning/STATE.md` P0 #2 — stuck MIDI notes on settings change (defensive mitigation in Lanes)
- `.planning/jam-features-2026/README.md` — 9-week feature pipeline + cross-feature dependencies
- HANDOFF.json — paused-session context that triggered this design
- PR #87 (PerformanceView), PR #88 (MIDI Learn), PR #89 (BPM-clock + Pattern programmer)
- Logic Pro Drummer (Apple) — reference behavior for the BeatMachine

---

## Pre-flight gate manual UAT checklist

Run this in `cargo tauri dev` before Phase 1.4 starts. Each item below has a clear pass/fail criterion. If items 1, 3, or 5 fail, stop the gate and we fix the underlying pattern bug first.

### Setup
- [ ] `cargo tauri dev` launches, app window opens
- [ ] No errors in browser DevTools console at app startup
- [ ] PatternPanel pip visible in StatusBar
- [ ] Click pip → PatternPanel opens

### 1. Pattern fires correctly in 'live' mode (CRITICAL)
- [ ] Default 16 cells all on
- [ ] Toggle off cells 2, 6, 10, 14 (every 4th cell starting at 1)
- [ ] Press transport play
- [ ] Hold middle C on keyboard
- [ ] Audible: harmony fires only on cells 0, 1, 3, 4, 5, 7, 8, 9, 11, 12, 13, 15
- [ ] Visual: highlighted cell on PatternPanel matches the cell that's audibly firing

### 2. Pattern fires correctly in 'gated' mode
- [ ] Switch input mode to 'gated'
- [ ] Hold middle C continuously through 1 full bar
- [ ] Audible: harmony NoteOff fires when entering an off-cell, NoteOn fires when entering an on-cell, all without releasing the key

### 3. Stop/start mid-pattern doesn't stick notes (CRITICAL)
- [ ] With pattern playing, hold middle C, let it run for 4 bars
- [ ] Press transport stop
- [ ] Wait 2 seconds
- [ ] Visual: piano UI shows no held harmony notes
- [ ] Audible: nothing ringing
- [ ] Press transport play again
- [ ] Pattern resumes correctly from totalBeat=0 (assumes transport.reset on stop — verify behavior)

### 4. Long-running drift check (5 min @ 120 BPM, 8-bar pattern)
- [ ] Set length to 8 bars
- [ ] Set BPM to 120
- [ ] Hold middle C, let pattern run for 5 minutes
- [ ] At 5-min mark: no audible drift, console clean of warnings, beat counter still aligned

### 5. Per-voice routing (CRITICAL)
- [ ] Configure: voice 0 → MIDI port 1, voice 1 → synth, voice 2 → off
- [ ] Hold middle C
- [ ] Pattern plays
- [ ] Audible: only voice 0 (port 1) and voice 1 (synth). Voice 2 silent.
- [ ] Stopped pattern → no stuck notes on port 1 or in synth

### 6. Press during off-cell (P1 — KNOWN CURRENT BUG)
- [ ] Pattern: only cells 0, 4, 8, 12 on (downbeats only)
- [ ] Press middle C during cell 1 (off)
- [ ] Currently: harmony fires immediately. **This is the bug Phase 1.7 fixes.**
- [ ] Document current behavior; do not fail the gate on this.

### Cleanup
- [ ] Close PatternPanel → pattern disables → all held harmony released
- [ ] Press transport stop → all notes released
- [ ] App can be closed cleanly with no errors

---

**End of architecture document. Phase 1.4 (WorldState + Lane trait) is next after pre-flight UAT passes (manual UAT items 1, 3, 5).**
