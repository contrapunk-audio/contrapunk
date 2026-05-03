# Contrapunk Architecture (Companion + Audio + Rig)

**Status:** Living document — rewritten 2026-05-04 (Sun) with state machine + audio graph + rig saving
**Supersedes:** `01-looper.md` (the original brief — kept for reference, scope expanded)
**Author:** Vibhav, drafted with Claude Code

---

## TL;DR

Three intertwined architecture pieces, designed together:

**Part 1 — Companion** (an automated bandmate). State machine with three primitives:
- `WorldState` — the observable. Holds transport, engine snapshot, held inputs, currently-sounding harmony, current chord, recent input window.
- `Lane` — a trait. The unit of decision-making. Each Lane declares an `input_filter`, runs in one of three phases (Sense / Mutate / Decide), reads `WorldState`, and emits `Vec<DispatchOp>`.
- `Companion` — the orchestrator. Holds the WorldState, owns a `Vec<Box<dyn Lane>>`, runs them in phase order on every router-loop tick and on every input event.

Every feature in the 9-week jam pipeline that emits MIDI in time fits as a Lane. Pattern (existing PR #89) becomes the first Lane impl. Looper, BeatMachine (Logic Drummer-style), Arpeggiator, Drone, Pad, ChordSeq, AutoKey all follow the same shape.

**Part 2 — Audio architecture** (replaces today's linear chain). Audio graph (DAG) of typed Nodes:
- `Instrument` trait separates sources from processors. Multiple parallel instruments (synth, drum sampler, plugins, drone) coexist.
- `Mixer` Nodes sum sources and feed into FX chains.
- Routes have first-class gain/pan/mute. Sub-buses, parallel sends, sidechaining all expressible.
- `MidiRouter` matrix maps MIDI sources (router thread, BeatMachineLane, etc.) to Instrument destinations — no more broadcast.

**Part 3 — Rig saving** (first-class persistence of the entire performance setup):
- Versioned JSON schema. Audio graph topology + Instrument states + Companion config + MIDI Learn map + engine state.
- Component registry (`type_id` → factory) for restoration.
- Migration framework for forward/backward compat.
- Native + WASM storage paths (filesystem vs IndexedDB).

Practical wins:
- Subsumes follow-up issue #91 (pure function extraction for testability) — `Lane::tick()` *is* the pure function.
- Subsumes the "press during off-cell still fires harmony" pattern bug — `handle_note_on` consults Companion via `on_input`, Lanes can `SuppressDefault`.
- 6/9 jam features fit as Lane impls. 3/9 (audio FX) sit safely outside Companion but inside the audio graph.
- Multi-instrument architecture lets the BeatMachine's DrumSampler coexist with the harmony synth; Wk 9 ambient pad becomes a separate Instrument with its own preset.
- Rig saving converts the jam from a stateless tool into a kit you can recall — pre-stage rigs per song, per genre, per composer.

Architectural debts that remain (deferred to separate sessions):
1. `panic_pending` sledgehammer — replace with typed `EngineMutation` enum once Wk 2 ChordSeq Lane lands.
2. Issue #90 root fix (held_harmonies stale-entry recovery) — defensive CC123 ops mitigate; root fix later.
3. Audio thread lock contention — defer until profiling shows it.

---

## Why "Companion"?

Reading the user's framing: *a companion that plays stuff on time in the background based on what you're playing in the key you're playing in, combined with auto-key, gives you a tool that comps and helps you jam better*.

Pattern-as-rhythmic-gate is one expression. Loops-as-recorded-phrases is another. Auto-key as passive "follow what I play" is a third. Logic-Drummer-style smart drummer is a fourth. They share an audience (the soloing musician), a venue (the jam), and a tick model (the transport clock). Treating them as siblings under a `Companion` umbrella is the cleaner mental model and the cleaner code.

This naming is breaking: existing `pattern_*` Tauri commands, `PatternStore`, `PatternConfig`, etc. all get renamed under the companion namespace. We have no external customers, so this is a one-time migration cost.

---

## System architecture (high-level)

```
        ┌────────────────────────── COMPANION ─────────────────────────┐
        │                                                               │
   ┌────│──→ on_input(ev) ──→ Lanes ─→ DispatchOps ──┐                │
   │    │   (handle_note_on,                          │                │
   │    │    inject_note_on,                          ▼                │
INPUTS  │    note_off, MIDI in)                MidiRouter             │
keyboard│                                            │                 │
virtual │    ┌─── WorldState (observable) ──┐       │                 │
MIDI in │    │  • transport                  │       ▼                 │
   │    │    │  • engine_snapshot            │  ╔════════════════════╗ │
   │    │    │  • held_inputs                │  ║   AUDIO GRAPH      ║ │
   │    │    │  • sounding_voices            │  ║                    ║ │
   │    │    │  • current_chord              │  ║   Synth ─→─┐       ║ │
   │    │    │  • recent_input_window        │  ║   Drums ─→─├─→Mix ─║─┼─→ FX ─→ Output
   │    │    └────────────────────────────────┘  ║   Drone ─→─┤       ║ │  (audio device)
   │    │                                  ▲     ║   Pad   ─→─┘       ║ │
   └────│──→ tick() ────────→ Lanes ────→──┘     ║   Plugin ─→ Mix    ║ │
        │   (every router-loop                    ╚════════════════════╝ │
        │    iteration)                                                  │
        └────────────────────────────────────────────────────────────────┘

   transport tick ──────────────────────────────────────────────────────┘
   (Arc<Transport>, sample-accurate, audio-thread-driven)

                           ┌─── RIG ────────────────┐
                           │  Save / load any time  │
                           │  Captures: graph,      │
                           │  Instrument states,    │
                           │  Companion config,     │
                           │  MIDI Learn map,       │
                           │  engine settings       │
                           └────────────────────────┘
```

Two entry points into Companion: `on_input` (event-driven) and `tick` (clock-driven). Both consult the same WorldState. Both produce `Vec<DispatchOp>`. The MidiRouter forwards each op to the correct Instrument in the audio graph. Audio mixes through the graph to the output device.

---

# PART 1 — Companion subsystem

## WorldState — the observable

```rust
pub struct WorldState {
    /// Sample-accurate position. Read-only handle; updated by audio thread.
    pub transport: Arc<Transport>,

    /// Snapshot of HarmonyEngine config — read often, mutated rarely.
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

pub struct HeldInput { pub note: u8, pub velocity: u8, pub channel: u8, pub pressed_at: Instant }
pub struct InputEntry { pub at: Instant, pub note: u8, pub velocity: u8 }
pub struct DetectedChord { pub root: Option<u8>, pub quality: Option<ChordQuality>, pub display: String }
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
    /// any Lane fall through to default harmonize. Enables split-keyboard /
    /// live-channel routing.
    fn input_filter(&self) -> InputFilter { InputFilter::None }

    /// Called once per router-loop iteration with current WorldState.
    fn tick(&mut self, world: &WorldState) -> LaneOutput;

    /// Called when an input event arrives that this Lane's filter matched.
    fn on_input(&mut self, _ev: InputEvent, _world: &WorldState) -> LaneOutput {
        LaneOutput::default()
    }

    /// Save this Lane's state for rig persistence. Default: empty (stateless lanes).
    fn serialize_state(&self) -> serde_json::Value { serde_json::Value::Null }
    fn deserialize_state(&mut self, _state: serde_json::Value) -> Result<(), String> { Ok(()) }
    fn type_id(&self) -> &str;  // for rig restoration registry
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
    NoteOn { instrument: InstrumentId, note: u8, velocity: u8, channel: u8 },
    NoteOff { instrument: InstrumentId, note: u8, channel: u8 },
    AllNotesOff { instrument: InstrumentId },  // CC 123 broadcast scope
}
// (NB: DispatchOp now keys to InstrumentId — see MidiRouter section in Part 2.
//  Phase 1 keeps the existing VoiceOutputTarget model and migrates after audio graph lands.)

pub enum EngineMutation {
    SetKey(Key), SetMode(HarmonyMode), SetScale(ScaleMode), SetVoiceLeading(VoiceLeadingStyle),
}

pub enum WorldWrite {
    UpdateChord(DetectedChord), UpdateDetectedKey(Key), UpdateDetectedMode(HarmonyMode),
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

**Concrete example:** AutoKey detects you're vibing in D minor (Sense). Engine snapshot now reflects the new key. ChordSeq's "next chord" lookup uses the updated key (Mutate). Drone Lane reads the new tonic from the engine snapshot and emits a low D drone (Decide). Each lane sees a coherent world.

Without phase ordering: Drone might read stale key while AutoKey is mid-update; ChordSeq fires harmony in the old key. Race conditions become latent musical glitches.

### Input filter — split keyboard / live channel

The user wants to play live melody while pattern + loops + drone + beats run in the background. Without `input_filter`, every Lane sees every press, and Pattern Lane will try to gate the live melody just like the chord-register inputs.

Default Lane filters:
- **PatternLane**: `All` (today's behavior). User can configure to `NoteRange(C2..B3)` for split-keyboard.
- **LooperLane (Input source)**: `All` (records anything user plays).
- **LooperLane (Output source)**: `None` (taps engine output, not input).
- **ArpLane**: `NoteRange(C2..B3)` default (chord register).
- **DroneLane / PadLane / BeatMachineLane**: `None` (tick-only).
- **AutoKeyLane**: writes via `world_writes`; doesn't suppress (lets harmony fire normally).

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
              → voices → MidiRouter → Instruments
              → WorldState.sounding_voices.update
              → next-tick Sense lanes pick up the change
```

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
        self.update_held(&ev);

        let mut suppress_default = false;
        let mut ops = Vec::new();

        for lane in self.lanes.iter_mut() {
            if !lane.input_filter().matches(&ev) { continue; }
            let out = lane.on_input(ev.clone(), &self.world);
            ops.extend(out.ops);
            if out.suppress_default { suppress_default = true; }
            self.apply_world_writes(out.world_writes);
            for m in out.engine_mutations { engine.lock().unwrap().apply(m); }
        }

        CompanionInputResult { ops, suppress_default }
    }

    pub fn save(&self) -> CompanionState {
        CompanionState {
            enabled: self.enabled.load(Ordering::Acquire),
            lanes: self.lanes.iter().map(|l| LaneState {
                type_id: l.type_id().to_string(),
                state: l.serialize_state(),
            }).collect(),
        }
    }

    pub fn restore(&mut self, state: CompanionState) -> Result<(), String> {
        self.enabled.store(state.enabled, Ordering::Release);
        for ls in state.lanes {
            // find Lane in self.lanes by type_id, deserialize state into it
            ...
        }
        Ok(())
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

    // 2. Apply Lane-emitted ops via MidiRouter (loop captures, arp pattern, etc.).
    for op in result.ops {
        state.midi_router.dispatch(op, &state.audio_graph);
    }

    // 3. Default harmonize unless a Lane suppressed it.
    if !result.suppress_default {
        let voices = state.engine.lock().unwrap().harmonize_note_on(note);
        for (i, v) in voices.iter().enumerate() {
            let target = state.midi_router.target_for_voice(i);
            state.midi_router.dispatch(
                DispatchOp::NoteOn { instrument: target, note: u8::from(*v), velocity, channel },
                &state.audio_graph,
            );
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
- Reads: transport.totalBeat, sounding_voices, current_cell index
- Emits: NoteOn/NoteOff per held harmony voice on cell boundaries
- Modes: Live (staccato retrigger), Gated (legato, edges only)
- F2/F4/F5/M3/H3 invariants port over; new P1 invariant added.

### LooperLane × N (Phase 2 — Wk 1)
**Phase**: Decide. **Filter** depends on source: `All` for Input source, `None` for Output source.
- Slot lifecycle: Empty → Armed → Recording → Playing / Stopped
- Replay paths: Input source re-enters via MidiRouter → harmony engine; Output source emits direct dispatch ops bypassing harmony
- L1-L5 invariants. N configurable.

### MotifTransposerLane (before Wk 6)
Extends LooperLane. Adds transpose semitone control + reads `world.current_chord` for auto-fit.

### ChordSeqLane (before Wk 2)
**Phase**: Mutate. **Filter**: `None`.
- Reads: transport.totalBeat
- Emits: `EngineMutation::SetKey(...)` + `SetMode(...)` on bar boundary, cycling user-typed progression.
- First Mutate-phase Lane. Validates the EngineMutation enum.

### DroneLane (before Wk 3)
**Phase**: Decide. **Filter**: `None`.
- Reads: engine_snapshot.tonic, transport (running flag)
- Emits: sustained NoteOn at tonic to a configured Instrument
- Tracks "is currently emitting" to fire NoteOff on disable / tonic change.

### ArpeggiatorLane (before Wk 7)
**Phase**: Decide. **Filter**: `NoteRange(C2..B3)` default.
- Reads: held_inputs, transport.totalBeat, current_chord, ARP config
- Emits: NoteOn/NoteOff per arp step
- on_input: returns `SuppressDefault=true` and stores chord; tick emits the arp.
- Sustain pedal handled at input pipeline filter level.

### AmbientPadLane (Wk 9)
**Phase**: Decide. **Filter**: `None`.
- Reads: engine_snapshot.key, transport.totalBeat
- Emits: slowly-evolving polyphonic NoteOns; morphs between presets over 8-16 bars
- Could share infrastructure with DroneLane.

### AutoKeyLane (Sense — before Wk 6)
**Phase**: Sense. **Filter**: `All` (observes; doesn't claim — non-suppressing).
- Reads: recent_input_window
- Writes: detected key/mode based on Krumhansl scale fitting (issue #81)
- Hysteresis to prevent flipping per note. Replaces the current `set_auto_key` AtomicBool.

### ChordDetectLane (Sense — Phase 1)
**Phase**: Sense. **Filter**: `None`.
- Reads: sounding_voices
- Writes: `WorldWrite::UpdateChord(DetectedChord)`
- Replaces existing `chord_name: Arc<Mutex<String>>` update site, lifts to WorldState.

### BeatMachineLane (see dedicated section below)

---

## BeatMachine — Logic Drummer-style smart drummer

**User vision**: Logic Pro's Drummer. *Not* a basic step sequencer — a smart adaptive drummer with style presets, intensity controls, and auto-fills.

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
   │   variant. Fill style adapts to intensity X.                │
   └────────────────────────────────────────────────────────────┘

   ┌─ Drum kit selector ────────────────────────────────────────┐
   │   Acoustic (default)  Electronic  Hybrid  Brushes  ...     │
   │   Each kit = a sample bank loaded into the drum sampler.    │
   └────────────────────────────────────────────────────────────┘

   ┌─ Per-element overrides (advanced) ─────────────────────────┐
   │   Kick: busy / sparse                                       │
   │   Snare: ghost notes / rim / accents                        │
   │   Hat: closed-only / open-on-2&4 / pumping                  │
   │   Percussion: on / off / fills-only                         │
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
    pub follow_target: Option<String>,    // Lane name for groove lock
    target: InstrumentId,                  // routing for drum events (DrumSampler instance)
}

pub struct ElementOverrides {
    pub kick: ElementBias,    // -1.0 sparse → 0.0 default → +1.0 busy
    pub snare: ElementBias,
    pub hat: ElementBias,
    pub perc: ElementBias,
}

pub struct DrummerPreset {
    pub name: String,                      // "Rock", "Hip-Hop", etc.
    pub patterns: PatternBank,             // 5 intensity levels × 4 element tracks
    pub fills: FillBank,                   // 5 fill variants
    pub style_constraints: StyleSettings,  // swing, push/pull, dynamics
}

pub struct PatternBank {
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
   - Interpolate between adjacent intensity levels.
   - At fill boundary, substitute fill pattern.
4. Apply per-element overrides.
5. Apply intensity_y (overall velocity multiplier).
6. Emit DispatchOp::NoteOn for each track that fires this cell.

### Build phases for BeatMachine

| Phase | What | Effort |
|---|---|---|
| **A. DrumSampler Instrument** (see Part 2) | Internal Rust sample-based Instrument with bundled kit. | 2-3 d |
| **B. BeatMachineLane skeleton** | Lane impl, basic per-track cell grid, no presets yet. Validates Lane abstraction with 2nd impl beyond Pattern. | 1-2 d |
| **C. Pattern library authoring** | 3-5 drummer presets × 5 intensity levels × kick/snare/hat/perc. | 2-3 d |
| **D. Intensity X/Y interpolation + auto-fills** | The "smart drummer" logic. | 2-3 d |
| **E. UI: BeatsTab in CompanionPanel** | X/Y pad, preset picker, kit picker, per-element overrides. | 2 d |
| **F. Polish + edge cases** | Mute/solo per element, follow-target wiring, smooth transitions. | 1-2 d |

**Total: 10-15 days for v1 BeatMachine.**

---

## Pure-function contracts (Lane invariants)

### Pattern invariants (existing, ported from PR #89)

- **F2**: ops emitted by pattern lane do not double-fire when polyphonic input produces overlapping harmonies.
- **F4**: when pattern lane re-fires a held voice with new routing, the old voice gets a NoteOff op before the new voice's NoteOn op.
- **F5**: when a setter raises `panic_pending` and pattern was about to fire on this tick, the panic-replay's `to_release` set covers the pattern-attacked notes; pattern lane skips this tick.
- **M3**: the first tick after `companion.enabled` flips true seeds `last_pattern_cell` without firing.
- **H3**: pattern lane skipped when `panic_pending` on this tick.

### Input pipeline

- **P1**: `handle_note_on` consults Companion before default harmonize. If a Lane returns `suppress_default=true`, default `harmonize_note_on` does not fire. PatternLane.on_input returns suppress_default=true when current cell is off in Live or Gated mode.

### Looper

- **L1**: a slot in `Recording` captures only events whose `beat_offset` is within `[0, length_beats)`.
- **L2**: a slot in `Playing` emits ops with `beat_offset == (transport.totalBeat - recorded_at_bar*beats_per_bar) mod length_beats` matching this tick's window.
- **L3**: a slot in `Stopped` emits a single `AllNotesOff` op on transition.
- **L4**: a slot transition `Empty → Armed` schedules the recording to start at the next bar boundary.
- **L5**: `LoopSource::Output` slots, on replay, emit ops that bypass the pattern lane. `LoopSource::Input` slots emit ops that re-enter the pattern lane.

### BeatMachine

- **B1**: tick emits at most one NoteOn per element per cell.
- **B2**: intensity X interpolation is monotonic (intensity 0.5 produces "between level 2 and level 3" patterns, never something outside that range).
- **B3**: auto-fill substitutes the *next* bar's pattern when fill boundary is crossed; the bar after that returns to base pattern.
- **B4**: on disable, all currently-sounding drum notes get NoteOff (no stuck snare).

All invariants get unit tests. Pure-function shape makes them reachable without standing up the router thread.

---

# PART 2 — Audio subsystem

## Current state assessment

The existing `src/chain/` + `src/synth/` were sized for tier-1 audio output: one synth, one linear FX chain. The 9-week pipeline + the BeatMachine + rig saving need more.

### Seven problems

1. **Linear-only chain** — `Vec<Box<dyn AudioBlock>>` cannot express multiple sources mixed before FX, parallel FX sends, sub-buses, or sidechaining.
2. **Single-synth assumption** — `Chain` allows one source at position 0; `voice.rs:147` makes the assumption explicit. BeatMachine + DrumSampler immediately need parallel sources.
3. **No source/processor distinction** — `AudioBlock` is one trait. Synths overwrite the buffer; FX modify it. Mismatched ordering silently destroys upstream signal.
4. **MIDI is broadcast** — `Chain::midi_event` delivers every event to every block. Drum NoteOn(36) plays as low-C2 on the harmony synth.
5. **Monolithic `Voice`** — osc/filter/envelope hardcoded. No path to FM, wavetable, granular, sample-based without parallel codepaths.
6. **Per-voice presets unsupported** — all 8 voices share one `SynthParams`. Pad ≠ Bass ≠ Lead can't coexist.
7. **No rig serialization** — each block exposes `type_id()` for future serialization, but `save()`/`restore()` is missing. Plugin params + chain layout + MIDI routing aren't persistable.

## Proposed: Audio Graph (DAG)

Replace the linear chain with a directed acyclic graph of typed nodes connected by routes. Linear chain becomes the degenerate case (one path through the graph).

### Architecture

```
   ┌─────────────────────────── AUDIO GRAPH (DAG) ──────────────────────────┐
   │                                                                        │
   │   nodes: Source | Processor | Mixer | Output                          │
   │   edges: Route { from: (node, port), to: (node, port), gain, mute }   │
   │                                                                        │
   │     ╭── Synth (Instrument) ─→─╮                                       │
   │     │                          │                                       │
   │     │  Drums (Instrument) ─→──├─→ MainBus (Mixer) ─→ Reverb ─→ Output│
   │     │                          │       │                              │
   │     │  Drone (Instrument) ─→──╯       ├──→ ReverbSend ─→ Reverb (wet)╮
   │     │                                  │                              ││
   │     │  PadSynth (Instrument) ─→ PadBus (Mixer) ─→ Chorus ─────────────┘│
   │     │                                                                  │
   │     ╰─ ClapPlugin (Instrument) ─→ MainBus                              │
   │                                                                        │
   └────────────────────────────────────────────────────────────────────────┘
```

### Node trait

```rust
pub trait Node: Send {
    fn type_id(&self) -> &str;       // stable string — for rig serialization
    fn name(&self) -> &str;           // human-readable
    fn role(&self) -> NodeRole;
    fn input_count(&self) -> usize;
    fn output_count(&self) -> usize;
    fn process(
        &mut self,
        inputs: &[&[f32]],            // input_count buffers, frames samples each
        outputs: &mut [&mut [f32]],   // output_count buffers
        frames: usize,
        channels: usize,
    );
    fn reset(&mut self);
    fn set_sample_rate(&mut self, sr: u32);
    fn serialize_state(&self) -> serde_json::Value;
    fn deserialize_state(&mut self, state: serde_json::Value) -> Result<(), String>;
}

pub enum NodeRole {
    Source,      // 0 inputs, ≥1 outputs (synth, sampler, plugin generator)
    Processor,   // M inputs, M outputs (FX: reverb, delay, distortion)
    Mixer,       // ≥2 inputs, 1 output (sums + per-input gain/pan/mute)
    Output,      // 1 input, 0 outputs (the audio device — exactly one in graph)
}
```

### AudioGraph

```rust
pub struct AudioGraph {
    nodes: Vec<NodeEntry>,
    routes: Vec<Route>,
    output_node: NodeId,
    /// Topological order, recomputed when graph mutates.
    process_order: Vec<NodeId>,
}

struct NodeEntry {
    id: NodeId,
    node: Box<dyn Node>,
    /// Each output port allocates a scratch buffer of `max_block_size * channels`.
    output_buffers: Vec<Vec<f32>>,
}

pub struct Route {
    pub from: (NodeId, usize /* output port */),
    pub to: (NodeId, usize /* input port */),
    pub gain: AtomicU32,    // ppt 0..1000 = 0.0..1.0 (lock-free)
    pub mute: AtomicBool,
}

pub type NodeId = u32;

impl AudioGraph {
    pub fn process(&mut self, output_buffer: &mut [f32], frames: usize, channels: usize) {
        for &node_id in &self.process_order {
            // Collect this node's inputs (sum of incoming routes, per-input port).
            let inputs = self.collect_inputs(node_id, frames);
            let outputs = self.scratch_outputs_for(node_id, frames);
            self.nodes[node_id as usize].node.process(&inputs, outputs, frames, channels);
        }
        // Copy the output_node's first output buffer into output_buffer.
        ...
    }
}
```

### Instrument trait (specialization of Node for Source role)

```rust
pub trait Instrument: Node {
    fn midi_event(&mut self, ev: MidiBlockEvent);

    /// Optional preset abstraction. Saved/loaded as JSON; structure is
    /// instrument-specific. The rig format wraps it in a typed envelope.
    fn save_preset(&self) -> serde_json::Value { serde_json::Value::Null }
    fn load_preset(&mut self, _preset: serde_json::Value) -> Result<(), String> { Ok(()) }
}

// Built-in instruments:
pub struct BasicSynth { /* current Synth refactored as Instrument */ }
pub struct DrumSampler { /* sample-based Instrument */ }
pub struct ClapInstrument { /* CLAP plugin wrapper */ }
```

### Mixer

```rust
pub struct Mixer {
    pub input_count: usize,
    pub gains: Vec<AtomicU32>,    // per-input gain
    pub pans: Vec<AtomicU32>,     // per-input pan, 500=center
    pub mutes: Vec<AtomicBool>,
}

impl Node for Mixer {
    fn role(&self) -> NodeRole { NodeRole::Mixer }
    fn process(&mut self, inputs: &[&[f32]], outputs: &mut [&mut [f32]], frames: usize, channels: usize) {
        // Sum all unmuted inputs with their gains and pans into outputs[0].
        ...
    }
}
```

### MIDI Router matrix

```rust
pub struct MidiRouter {
    /// Maps voice slot index (0..MAX_VOICES) → which Instrument receives it.
    /// Replaces today's VoiceOutputTarget per voice with a more general
    /// "MIDI source → Instrument" mapping.
    voice_routing: Vec<InstrumentId>,

    /// Lane-specific routing. BeatMachineLane's events go to the drum_sampler
    /// instrument; LooperLane (Output source) emits into pre-routed targets;
    /// etc.
    lane_routing: HashMap<String /* lane name */, InstrumentId>,

    /// Inverse: given an InstrumentId, find the corresponding audio Node.
    instruments: HashMap<InstrumentId, NodeId>,
}

impl MidiRouter {
    pub fn dispatch(&self, op: DispatchOp, graph: &AudioGraph) {
        match op {
            DispatchOp::NoteOn { instrument, note, velocity, channel } => {
                let node_id = self.instruments[&instrument];
                if let Some(inst) = graph.get_instrument_mut(node_id) {
                    inst.midi_event(MidiBlockEvent::NoteOn { note, velocity });
                }
            }
            DispatchOp::NoteOff { instrument, note, .. } => { ... }
            DispatchOp::AllNotesOff { instrument } => { ... }
        }
    }
}

pub type InstrumentId = String;  // stable across rig saves: "main_synth", "drum_kit", "drone_synth", ...
```

### Built-in instruments shipped in v1

| Instrument | type_id | Role | Purpose |
|---|---|---|---|
| `BasicSynth` (refactored from current) | `builtin.basic_synth` | Source | Harmony voices |
| `DrumSampler` | `builtin.drum_sampler` | Source | BeatMachine drum playback |
| `Drone` (Wk 3) | `builtin.drone_synth` | Source | Sustained drone — single-voice synth optimized for hold-forever |
| `Pad` (Wk 9) | `builtin.pad_synth` | Source | Slow-evolving polyphonic pad |
| `ClapInstrument` | `clap.<plugin-id>` | Source | CLAP plugin host wrapper |

### Built-in processors shipped in v1

| Processor | type_id | Notes |
|---|---|---|
| `Reverb` (existing) | `builtin.reverb` | Refactor to fit new Node interface |
| `Delay` (existing) | `builtin.delay` | Same |
| `Bitcrusher` (Wk 3) | `builtin.bitcrusher` | New |
| `ReverseDelay` (Wk 4) | `builtin.reverse_delay` | New |
| `Shimmer` (Wk 4) | `builtin.shimmer` | New |
| `Distortion` (Wk 8) | `builtin.distortion` | New |

---

## DrumSampler subsystem (now an Instrument)

```rust
pub struct DrumSampler {
    pub kit: DrumKit,
    pub voices: Vec<SampleVoice>,
    pub master_gain: AtomicU32,
}

pub struct DrumKit {
    pub name: String,
    pub kit_id: String,                                   // for serialization, e.g. "acoustic_default"
    pub samples: HashMap<u8 /* MIDI note */, DrumSample>,
}

pub struct DrumSample {
    pub buffer: Arc<Vec<f32>>,    // mono PCM at engine sample rate
    pub envelope: SampleEnvelope, // ADSR for natural release
}

impl Node for DrumSampler {
    fn role(&self) -> NodeRole { NodeRole::Source }
    fn input_count(&self) -> usize { 0 }
    fn output_count(&self) -> usize { 1 }
    fn process(&mut self, _inputs: &[&[f32]], outputs: &mut [&mut [f32]], frames: usize, _channels: usize) {
        // Render active sample voices, sum into outputs[0].
        ...
    }
    fn type_id(&self) -> &str { "builtin.drum_sampler" }
    fn serialize_state(&self) -> serde_json::Value {
        json!({ "kit_id": self.kit.kit_id, "master_gain_ppt": self.master_gain.load(...) })
    }
    fn deserialize_state(&mut self, state: serde_json::Value) -> Result<(), String> {
        let kit_id = state["kit_id"].as_str().unwrap_or("acoustic_default");
        self.kit = DrumKit::load(kit_id)?;
        self.master_gain.store(state["master_gain_ppt"].as_u64().unwrap_or(700) as u32, ...);
        Ok(())
    }
}

impl Instrument for DrumSampler {
    fn midi_event(&mut self, ev: MidiBlockEvent) {
        match ev {
            MidiBlockEvent::NoteOn { note, velocity } => {
                if let Some(sample) = self.kit.samples.get(&note) {
                    self.trigger_voice(sample, velocity);
                }
            }
            ...
        }
    }
}
```

### Default kit content (bundled)

- 5 elements minimum: kick, snare, closed hi-hat, open hi-hat, ride bell
- 8 elements ideal: + crash, tom, perc/clap
- Sourced from CC0 / public domain libraries
- Bundled at compile time via `include_bytes!` for native; downloaded once for WASM (cached via Service Worker)

---

## Audio architecture phase plan

| Phase | What | Effort |
|---|---|---|
| **G1** | `Node` trait + `AudioGraph` skeleton + topo-sort + buffer allocation | 3-4 d |
| **G2** | Migrate `Synth` to `BasicSynth: Instrument`. Migrate `Reverb`/`Delay` to `Node`. Replace `Chain` callers with `AudioGraph`. | 2-3 d |
| **G3** | `Mixer` Node + `MidiRouter` + multi-instrument support | 2 d |
| **G4** | `DrumSampler` Instrument + bundled kit | 2-3 d |
| **G5** | UI: graph editor (initially limited — add/remove/reroute via Tauri commands; visual editor later) | 2 d |
| **G6** | WASM parity: AudioGraph runs in browser via WebAudio AudioWorklet | 2-3 d |

**Total: 13-17 days** for the audio architecture rework. DrumSampler (G4) is a hard prerequisite for BeatMachine Phase B.

---

# PART 3 — Rig saving

## What's in a rig

A *rig* is a complete snapshot of the user's performance setup: everything they could lose if the app crashes mid-jam, plus everything they'd want to recall when they sit down to jam tomorrow.

```rust
pub struct Rig {
    pub schema_version: u32,            // for migration
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,

    // Audio
    pub audio_graph: AudioGraphState,   // node list + routes + output_node
    pub midi_router: MidiRouterState,

    // Companion
    pub companion: CompanionState,      // master enable + per-Lane state

    // Engine
    pub engine: EngineState,            // key, mode, scale, voice_leading_style

    // Hardware mapping
    pub midi_learn: MidiLearnMap,       // PR #88 knob bindings

    // Transport (settings, NOT play state)
    pub transport_settings: TransportSettings,  // bpm, time signature, metronome_enabled
}

pub struct AudioGraphState {
    pub nodes: Vec<NodeState>,
    pub routes: Vec<RouteState>,
    pub output_node: NodeId,
}

pub struct NodeState {
    pub id: NodeId,
    pub type_id: String,                // for the registry to instantiate
    pub name: String,
    pub state: serde_json::Value,       // node-specific serialized state
}

pub struct RouteState {
    pub from: (NodeId, usize),
    pub to: (NodeId, usize),
    pub gain: f32,
    pub mute: bool,
}

pub struct CompanionState {
    pub enabled: bool,
    pub lanes: Vec<LaneState>,
}

pub struct LaneState {
    pub type_id: String,
    pub state: serde_json::Value,
}
```

What's **not** in a rig:
- Currently-held inputs (ephemeral session state)
- Loop buffers (takes — ephemeral)
- Transport play/stop position (current sample_pos)
- Live UI panel visibility states (those go in localStorage separately)

---

## Schema versioning + migration framework

```rust
pub const CURRENT_SCHEMA_VERSION: u32 = 1;

pub fn migrate(rig: serde_json::Value) -> Result<Rig, RigError> {
    let mut version = rig.get("schema_version")
        .and_then(|v| v.as_u64()).unwrap_or(0) as u32;
    let mut working = rig;

    while version < CURRENT_SCHEMA_VERSION {
        working = match version {
            0 => migrate_v0_to_v1(working)?,
            // future: 1 => migrate_v1_to_v2(working)?,
            _ => return Err(RigError::UnsupportedVersion(version)),
        };
        version += 1;
    }

    serde_json::from_value(working).map_err(RigError::Json)
}

fn migrate_v0_to_v1(mut rig: serde_json::Value) -> Result<serde_json::Value, RigError> {
    // Example future migration shape: rename `pattern` → `companion.pattern`
    if let Some(p) = rig.get("pattern").cloned() {
        let companion = rig.entry("companion").or_insert(json!({}));
        companion["pattern"] = p;
        rig.as_object_mut().unwrap().remove("pattern");
    }
    Ok(rig)
}
```

Each migration is a deliberate, reviewed change. Tests for each migration step using sample fixture rigs.

---

## Component registry

Loader needs to instantiate Nodes/Lanes/Instruments from `type_id`. Each implementation registers a factory function.

```rust
pub type NodeFactory = fn() -> Box<dyn Node>;
pub type LaneFactory = fn() -> Box<dyn Lane>;

pub fn node_registry() -> &'static HashMap<&'static str, NodeFactory> {
    static REGISTRY: OnceLock<HashMap<&'static str, NodeFactory>> = OnceLock::new();
    REGISTRY.get_or_init(|| {
        let mut m = HashMap::new();
        m.insert("builtin.basic_synth", BasicSynth::factory as NodeFactory);
        m.insert("builtin.drum_sampler", DrumSampler::factory as NodeFactory);
        m.insert("builtin.reverb", Reverb::factory as NodeFactory);
        m.insert("builtin.delay", Delay::factory as NodeFactory);
        m.insert("builtin.mixer", Mixer::factory as NodeFactory);
        // CLAP plugins are instantiated via a different path (need their bundle path)
        m
    })
}

pub fn lane_registry() -> &'static HashMap<&'static str, LaneFactory> {
    // Similar for PatternLane, LooperLane, BeatMachineLane, etc.
}
```

CLAP plugins are special — `type_id` like `"clap.com.u-he.Diva"` references an external plugin bundle. The loader either:
- Locates the bundle by id (in standard CLAP search paths) and instantiates, OR
- If not found, replaces with a "missing plugin" placeholder Node and warns the user.

---

## Save / load API

```rust
impl AppState {
    pub fn save_rig(&self, name: &str) -> Result<Rig, RigError> {
        Ok(Rig {
            schema_version: CURRENT_SCHEMA_VERSION,
            name: name.to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            audio_graph: self.audio_graph.lock().unwrap().save(),
            midi_router: self.midi_router.lock().unwrap().save(),
            companion: self.companion.lock().unwrap().save(),
            engine: self.engine.lock().unwrap().save(),
            midi_learn: self.midi_learn.lock().unwrap().save(),
            transport_settings: self.transport.save_settings(),
        })
    }

    pub fn load_rig(&self, rig: Rig) -> Result<(), RigError> {
        let rig = migrate(serde_json::to_value(rig)?)?;
        // Drop current state cleanly (panic to release stuck notes).
        self.panic_pending.store(true, Ordering::Release);

        // Restore engine first (key/mode/scale).
        self.engine.lock().unwrap().restore(rig.engine)?;

        // Restore audio graph (instantiates Nodes via registry, applies states).
        let new_graph = AudioGraph::from_state(rig.audio_graph, node_registry())?;
        *self.audio_graph.lock().unwrap() = new_graph;

        // Restore MIDI router.
        self.midi_router.lock().unwrap().restore(rig.midi_router)?;

        // Restore companion (instantiates Lanes via registry).
        self.companion.lock().unwrap().restore(rig.companion, lane_registry())?;

        // Restore MIDI Learn + transport settings.
        self.midi_learn.lock().unwrap().restore(rig.midi_learn)?;
        self.transport.restore_settings(rig.transport_settings);

        Ok(())
    }
}
```

---

## Storage

### Native (Tauri)

- Path: `~/.config/contrapunk/rigs/<slug>.rig.json` (macOS / Linux)
                    `%APPDATA%\contrapunk\rigs\<slug>.rig.json` (Windows)
- One JSON file per rig.
- Rig directory listed by the loader for the "Open Rig" UI.
- Atomic write: write to `.tmp`, rename. Crash mid-save doesn't corrupt prior rig.

### WASM (`app.contrapunk.com`)

- IndexedDB store `rigs`: `{ slug: string, rig: Rig }`.
- Export: download as `.rig.json` via `Blob` + `<a download>`.
- Import: file picker → `JSON.parse` → `load_rig`.

Import/export interop: same JSON format on both sides. Drag a file from native install onto the browser app and it loads.

---

## UI considerations

```
┌─ Top bar / status row ───────────────────────────────┐
│  ▣ Companion  ▣ Pattern    Rig: [ My Jam ▼ ]  💾    │
└──────────────────────────────────────────────────────┘

Click rig dropdown:
┌──────────────────────────────┐
│  Rigs                        │
│  ────────────────────────    │
│  ● My Jam (current)          │
│  ○ Kondo Mode                │
│  ○ Mick Gordon Setup          │
│  ○ Quiet Pads                 │
│  ────────────────────────    │
│  + New Rig                   │
│  ⤓ Import Rig                │
│  ⤴ Export Current Rig         │
└──────────────────────────────┘

Click 💾:
┌──────────────────────────────┐
│  Save Rig                    │
│  Name: [ Kondo Mode      ]   │
│  ☐ Save as new                │
│  ☑ Overwrite "My Jam"         │
│  [ Save ]    [ Cancel ]      │
└──────────────────────────────┘
```

Save modes:
- **Overwrite current** (fast, default behavior)
- **Save as new** (asks for name)
- **Auto-save current rig on every change** (settings toggle, default off — destructive editing without confirmation can lose state)

Loading a rig with stuck notes from the previous setup is handled by the panic_pending broadcast in `load_rig`.

---

## Phase plan for rig saving

| Phase | What | Effort |
|---|---|---|
| **R1** | `Node::serialize_state` + `Lane::serialize_state` impls — every component knows how to save itself | 2 d (spread across G1-G4 work) |
| **R2** | `Rig` struct + Save/Load API + Migration framework + node/lane registries | 2 d |
| **R3** | Native filesystem storage layer | 1 d |
| **R4** | WASM IndexedDB storage layer + export/import | 1-2 d |
| **R5** | UI: Rig dropdown + Save dialog | 1-2 d |

**Total: 7-10 days**, partially overlapping with G1-G4 (the per-Node serialize work).

---

# PART 4 — Operations

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
| (new) | `commands/rig.rs::save_rig`, `load_rig`, `list_rigs`, `delete_rig`, `import_rig`, `export_rig` |
| `chain.rs::Chain` | `audio/graph.rs::AudioGraph` |
| `chain.rs::ChainCommander` | `audio/graph.rs::AudioGraphCommander` |
| `synth/voice.rs::Synth` | `audio/instruments/basic_synth.rs::BasicSynth` |
| `chain/block.rs::AudioBlock` | `audio/node.rs::Node` (broader trait) |
| `pattern_config_tests` | `companion_pattern_tests` ✅ done in 1.2 |

### TS

| Old | New |
|---|---|
| `lib/stores/pattern.svelte.ts::PatternStore` | `lib/stores/companion.svelte.ts::CompanionStore` (umbrella) |
| `lib/components/PatternPanel.svelte` | `lib/components/companion/CompanionPanel.svelte` (umbrella) + tabs |
| Adapter `setPatternConfig` | `setCompanionPattern` |
| Adapter `setPatternEnabled` | `setCompanionEnabled` |
| Adapter (new) | `armCompanionLoop`, `setCompanionBeatIntensity`, `saveRig`, `loadRig`, etc. |

### localStorage keys

| Old | New |
|---|---|
| `'contrapunk-pattern'` | `'contrapunk-companion'` (one blob) |
| (new) | `'contrapunk-current-rig-slug'` (which rig is loaded) |

---

## UI surface

```
┌─ Status bar pip row ──────────────────────────────────────────┐
│  ◉ Transport   ◉ Companion (master)   Rig: [ My Jam ▼ ]  💾  │
└───────────────────────────────────────────────────────────────┘

┌─ CompanionPanel ──────────────────────────────────────┐
│  [ ▣ Pattern ] [ Loops ] [ Beats ] [ Auto-Key ]       │
│  ───────────────────────────────────────────────────  │
│  PatternTab:                                           │
│    subdivision, length, input mode, cells              │
│    + Input range (split-keyboard) configurable         │
└────────────────────────────────────────────────────────┘

┌─ CompanionPanel ──────────────────────────────────────┐
│  [ Pattern ] [ ▣ Loops ] [ Beats ] [ Auto-Key ]       │
│  ───────────────────────────────────────────────────  │
│  Slot count: [ 1 | 2 | 3 | ▣ 4 | + Add ]              │
│  Per-slot: capture toggle, length, state, record      │
└────────────────────────────────────────────────────────┘

┌─ CompanionPanel ──────────────────────────────────────┐
│  [ Pattern ] [ Loops ] [ ▣ Beats ] [ Auto-Key ]       │
│  ───────────────────────────────────────────────────  │
│  Drummer:  [ Rock ▼ ]    Kit: [ Acoustic ▼ ]         │
│                                                        │
│  ┌─ Intensity ──────┐   ┌─ Element overrides ─┐       │
│  │     Loud          │   │  Kick    [─●─────]  │       │
│  │  ┌────●─────┐    │   │  Snare   [───●───]  │       │
│  │  │ X/Y pad  │     │   │  Hat     [─────●─]  │       │
│  │  │ (drag)   │     │   │  Perc    [─●─────]  │       │
│  │  └──────────┘     │   └─────────────────────┘       │
│  │     Soft          │                                  │
│  │  Simple ←→ Complex│   Auto-fill every: [ 8 ▼ ]      │
│  └───────────────────┘                                  │
└────────────────────────────────────────────────────────┘

┌─ CompanionPanel ──────────────────────────────────────┐
│  [ Pattern ] [ Loops ] [ Beats ] [ ▣ Auto-Key ]       │
│  ───────────────────────────────────────────────────  │
│  ☑ Enable auto-detect                                  │
│  Detected: D minor (confidence 0.84)                   │
│  Hysteresis: ●●●○○ (medium — 3 changes/min cap)        │
└────────────────────────────────────────────────────────┘

┌─ Audio graph editor (new panel — basic v1) ──────────┐
│  Sources:    Synth, DrumSampler, Drone               │
│  Mixers:     MainBus                                  │
│  FX:         Reverb, Delay                            │
│  Output:     Speakers                                 │
│  ──────────────────────────────────────────────────  │
│  + Add Instrument    + Add Mixer    + Add FX         │
│  Load CLAP plugin... [Browse]                        │
└──────────────────────────────────────────────────────┘
```

**MIDI Learn integration** (PR #88): each Lane and rig action exposes its key actions as discoverable knob targets. Stable command names for persistence across rig swaps:
- `companion_loop_arm_slot_{N}`
- `companion_beat_intensity_x`, `companion_beat_intensity_y`, `companion_beat_preset`
- `companion_pattern_enable`
- `rig_load_slot_{N}` (preset rig hot-swap)

---

## WASM parity

Browser-side companion + audio graph + drum sampler all run in WASM (Rust → WASM compile). Sample bank streamed once on first load; cached via Service Worker.

WebAudio integration: AudioGraph topo-sorts Rust-side, then renders into a `Float32Array` consumed by an AudioWorkletNode. Web MIDI feeds the input pipeline.

CLAP plugins are NOT supported in WASM. The loader detects `clap.*` type_ids in a rig loaded in browser context and replaces with placeholder + warns.

---

## Feature → architectural layer mapping

```
   FEATURE                          LAYER                    LANDS IN

   Pattern (current PR #89)         Decide Lane              Phase 1 (refactor)

   Wk 1: Looper (input + output)    Decide Lane × N          Phase 2

   Wk 2: Chord progression seq      Mutate Lane              Before Wk 2

   Wk 3: Drone                      Decide Lane              Before Wk 3
                                    + Drone Instrument       (audio source)
   Wk 3: Bitcrusher                 Processor Node           audio graph extension

   Wk 4: Reverse delay              Processor Node           audio graph extension
   Wk 4: Shimmer reverb             Processor Node           audio graph extension

   Wk 5: Exotic scales              pure scale data          no companion change

   Wk 6: Motif transposer           extends Looper Lane      Before Wk 6
   Wk 6: auto-fit-to-chord          reads current_chord      (foundation already there)

   Wk 7: Arpeggiator                Decide Lane              Before Wk 7
   Wk 7: Sustain pedal              input pipeline filter    small input-layer change

   Wk 8: Distortion                 Processor Node           audio graph extension
   Wk 8: Power-chord mode           HarmonyEngine option     one engine flag
   Wk 8: Drop-tune preset           guitar_pipeline config   subsystem-local

   Wk 9: Ambient pad                Decide Lane              Before Wk 9
                                    + Pad Instrument         (audio source)

   AutoKey rewrite (issue #81)      Sense Lane               Before Wk 6
                                    + recent_input_window    (motif transpose dep)

   BeatMachine (Logic Drummer)      Decide Lane              See dedicated section
   DrumSampler subsystem            Source Instrument        Phase G4

   Audio graph (DAG)                Foundation rework        Phase G1-G6
   Rig saving                       Foundation feature       Phase R1-R5
```

---

## Phase plan (revised, comprehensive)

| Phase | What | Effort | Status |
|---|---|---|---|
| **0** | Pre-flight gate (cargo test + UI check + manual UAT) | 45 min | ✅ automated done; manual UAT pending user |
| **1.1** | Companion module skeleton, type stubs | 1 hr | ✅ done (`2ca0291`) |
| **1.2** | Move PatternConfig into companion/pattern.rs as CompanionPattern | 30 min | ✅ done (`19053a6`) |
| **1.3** | Update consumers, drop migration aliases | 30 min | ✅ done (`b4f8940`) |
| **1.4** | `WorldState` struct, move held trackers from router thread | 1 d | next |
| **1.5** | `Lane` trait + `Companion` orchestrator + phase ordering | 1 d | |
| **1.6** | PatternLane impl wrapping existing logic + tests | 1 d | |
| **1.7** | Companion-mediated input pipeline (P1 fix) | 0.5 d | |
| **1.8** | TS adapter rename + frontend store split (CompanionStore umbrella) | 1 d | |
| **1.9** | localStorage migration shim, Tauri command rename | 0.5 d | |
| **2** | LooperLane: single + multi-slot, both Input/Output sources | 2-3 d | |
| **3** | WASM parity for Pattern + Looper | 1 d | |
| **G1** | `Node` trait + `AudioGraph` skeleton + topo-sort | 3-4 d | |
| **G2** | Migrate Synth/Reverb/Delay to Node/Instrument | 2-3 d | |
| **G3** | `Mixer` Node + `MidiRouter` + multi-instrument | 2 d | |
| **G4** | DrumSampler Instrument + bundled kit | 2-3 d | |
| **G5** | UI: graph editor (basic add/remove/reroute) | 2 d | |
| **G6** | WASM AudioWorklet integration | 2-3 d | |
| **R1** | `Node`/`Lane`::serialize_state impls (overlaps G1-G4) | 2 d | |
| **R2** | Rig struct + Save/Load API + Migration framework + registries | 2 d | |
| **R3** | Native filesystem storage | 1 d | |
| **R4** | WASM IndexedDB storage + export/import | 1-2 d | |
| **R5** | UI: Rig dropdown + Save dialog | 1-2 d | |
| **A** | (replaces standalone DrumSampler phase — handled by G4) | — | |
| **B** | BeatMachineLane skeleton (basic step-seq behavior) | 1-2 d | |
| **C** | Drummer pattern library (3-5 presets × 5 intensity levels) | 2-3 d | |
| **D** | Intensity X/Y interpolation + auto-fills | 2-3 d | |
| **E** | UI: BeatsTab in CompanionPanel | 2 d | |
| **F** | BeatMachine polish + edge cases | 1-2 d | |
| **5** | UI consolidation: CompanionPanel umbrella with all tabs | 1-2 d | |
| **6** | ChordSeqLane (Wk 2) | 1-2 d | |
| **7** | DroneLane + Drone Instrument (Wk 3) | 1-2 d | |
| **8** | AutoKeyLane (Sense) rewrite — issue #81 | 2 d | |
| **9** | MotifTransposerLane (Wk 6) | 1-2 d | |
| **10** | ArpeggiatorLane + sustain pedal (Wk 7) | 1-2 d | |
| **11** | AmbientPadLane + Pad Instrument (Wk 9) | 1-2 d | |
| **12** | FX additions (Bitcrusher Wk 3, Reverse+Shimmer Wk 4, Distortion Wk 8) | 2-3 d each, ~6-9 d total | |

**Total: 45-60 days** for everything. The user has confirmed "no shortcuts, all phases." Timeline is the user's call — this doc just states the realistic effort.

**Key prerequisite ordering:**
- Phase 1 completes before Phase 2 (LooperLane needs Lane trait)
- G1 → G2 → G3 → G4 ordered (G2 migrates existing FX before G3 introduces Mixer; G4 needs G3 for drum routing)
- G4 (DrumSampler) before B (BeatMachineLane) — drum sampler is the audio target
- R1 happens *during* G1-G4 (each new Node implements serialize_state as it's built)
- R2-R5 can land any time after R1 stabilizes
- Phase 7 (DroneLane) needs G2 (Drone Instrument depends on Instrument trait)
- Phase 11 (Pad) needs G2 same way

---

## Decisions locked

1. ✅ Companion as umbrella concept (pattern + loops + auto-key + beats as limbs)
2. ✅ Both Input + Output loop sources, per-slot toggle
3. ✅ N configurable slots
4. ✅ Pre-flight gate before Phase 1
5. ✅ State machine with Lane trait + WorldState + 3-phase orchestration
6. ✅ Lanes declare `input_filter` for split-keyboard / live-channel
7. ✅ `handle_note_on` consults companion → fixes press-during-off-cell bug (P1)
8. ✅ BeatMachine = Logic Drummer-style smart drummer (NOT a basic step sequencer)
9. ✅ Internal Rust drum sampler
10. ✅ Audio graph (DAG) replaces linear chain
11. ✅ `Instrument` trait separates source Nodes from generic Nodes
12. ✅ MIDI router matrix instead of broadcast
13. ✅ Mixer Node as a typed role (not just another Block)
14. ✅ Rig saving as a first-class feature (not deferred)
15. ✅ Versioned JSON schema with migration framework for rig files
16. ✅ Native (filesystem) + WASM (IndexedDB) storage with import/export interop
17. ✅ Component registry pattern for type_id → factory
18. ✅ Full rename (Tauri commands + types + storage keys + files), no backwards-compat shim except one-shot localStorage migration
19. ✅ Audio FX framework refactored into Processor Nodes (was: stays separate). FX still extend `src/fx/` but become Nodes in the graph.

---

## Open questions

1. **BeatMachine schedule**: build alongside Phase 1 (Phase B basic only), replace Wk 7 Arp, insert Wk 3.5, or post-jam? — recommend: alongside Phase 1 with Phase B (validates Lane abstraction), Phase C-F lands as dedicated feature.
2. **Recording start**: arm-and-wait-for-next-bar (recommend) or start immediately on Record press?
3. **Default loop slot count on first run**: 1, 2, or 4?
4. **MIDI Learn binding for per-slot loop arm/stop on day 1** or punt to polish phase?
5. **Visual loop indicator on piano during playback** in Phase 2 polish or later?
6. **AutoKey tab scope**: just the existing toggle, or also surface key/mode confidence + visualizations?
7. **Drum kit selection in v1 BeatMachine**: ship one kit (Acoustic), or 3 (Acoustic / Electronic / Brushes)?
8. **Audio graph editor UI**: text-based first (list of nodes + routes JSON-style) or visual node-graph editor (more work)?
9. **Rig auto-save**: opt-in setting (recommend off), opt-out (recommend on), or always-on?
10. **Rig sharing**: should rigs be shareable URLs (`app.contrapunk.com/?rig=base64...`) for the jam? Future feature; not v1.
11. **Plugin scan caching**: rigs reference CLAP plugins by id. If user moves plugins after saving a rig, fallback handling? Recommend: warn-and-continue with placeholder Node.

---

## Architectural debts (deferred to separate sessions)

1. **`panic_pending` is a sledgehammer** — every engine setter triggers full reharm. Adds a Lane → adds another consumer that has to survive panic-replay. Replace with typed `EngineMutation` enum (Phase 6 ChordSeqLane is the natural home).

2. **Issue #90 (held_harmonies stale-entry recovery)** — defensive `CC123 AllNotesOff` ops in Companion-emitted state changes mitigate the symptom for the new code. Root fix (TTL or cross-reference) tracked separately.

3. **Audio thread lock contention** — `Mutex<HarmonyEngine>` reads on the Decide-phase hot path. Profile after Phase 5; if contention shows, swap for `ArcSwap<EngineSnapshot>` and have engine setters publish snapshots.

4. **Router thread testability** — Phase 1 unit-tests Lane impls against the trait. Integration tests for the *router thread itself* (spawn router + scripted MIDI input + assert dispatched events) tracked as a follow-up to issue #91.

5. **Per-voice modular synth (osc/filter/env sub-traits)** — Phase G2 extracts the `Instrument` trait. Sub-modularization (pluggable osc, filter, env per voice) is deferred until a feature needs it (Wk 9 Pad is the natural trigger).

6. **Rig sharing / cloud sync** — all rigs are local in v1. Cross-device sync, share-by-URL, jam.contrapunk.com hosted rig browser are future features. Architecture is ready (JSON serializable everything); product work pending.

7. **Audio graph live editing safety** — adding/removing a Node mid-playback can cause clicks if not handled gracefully. Initial v1: pause audio briefly during graph mutations (acceptable for occasional rig-edit, not for live performance). Crossfade-on-mutation deferred.

---

## Acceptance criteria (Phase end-state)

- [ ] All planned Lanes shipped (Pattern, Looper × N, BeatMachine, ChordSeq, Drone, Pad, Arp, AutoKey, MotifTransposer)
- [ ] Audio graph runs the full instrument set (BasicSynth, DrumSampler, Drone, Pad, CLAP plugin)
- [ ] At least one bundled drum kit
- [ ] BeatMachine has 3+ Drummer presets, intensity X/Y, auto-fills
- [ ] Live-play-on-top works: chord-register patterns + melody-register live notes coexist
- [ ] No regressions in existing pattern behavior (lockstep test + visual comparison)
- [ ] Native + WASM parity (everything in this doc works in `app.contrapunk.com`)
- [ ] No stuck notes on Lane state changes (defensive CC 123 broadcast)
- [ ] Per-slot/per-Lane MIDI Learn discoverable
- [ ] Companion master toggle in StatusBar replaces panel-pip pattern
- [ ] AutoKey functionality preserved + improved with Krumhansl-style detection
- [ ] All Lane invariants (P1, F2-H3, L1-L5, B1-B4) covered by unit tests
- [ ] **Save / Load rigs from native filesystem**
- [ ] **Save / Load rigs from browser IndexedDB**
- [ ] **Import / Export rigs as JSON files** (cross-platform)
- [ ] **At least one schema migration written + tested** (proves the framework)
- [ ] **CLAP plugin missing-bundle case handled gracefully on rig load**

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
- Reaper / Ableton / Bitwig — reference inspirations for the audio graph + rig model

---

## Pre-flight gate manual UAT checklist

Run this in `cargo tauri dev` before Phase 1.4 starts. If items 1, 3, or 5 fail, stop the gate and we fix the underlying pattern bug first.

### Setup
- [ ] `cargo tauri dev` launches, app window opens
- [ ] No errors in browser DevTools console at app startup
- [ ] PatternPanel pip visible in StatusBar
- [ ] Click pip → PatternPanel opens

### 1. Pattern fires correctly in 'live' mode (CRITICAL)
- [ ] Default 16 cells all on
- [ ] Toggle off cells 2, 6, 10, 14
- [ ] Press transport play
- [ ] Hold middle C
- [ ] Audible: harmony fires only on cells 0, 1, 3, 4, 5, 7, 8, 9, 11, 12, 13, 15
- [ ] Visual: highlighted cell matches audible cell

### 2. Pattern fires correctly in 'gated' mode
- [ ] Switch input mode to 'gated'
- [ ] Hold middle C continuously through 1 full bar
- [ ] Audible: harmony NoteOff fires when entering an off-cell, NoteOn fires when entering an on-cell

### 3. Stop/start mid-pattern doesn't stick notes (CRITICAL)
- [ ] With pattern playing, hold middle C, let it run for 4 bars
- [ ] Press transport stop
- [ ] Wait 2 seconds — visual: piano UI shows no held harmony notes; audible: nothing ringing
- [ ] Press transport play again — pattern resumes correctly

### 4. Long-running drift check (5 min @ 120 BPM, 8-bar pattern)
- [ ] Set length to 8 bars, BPM to 120
- [ ] Hold middle C, let pattern run for 5 minutes
- [ ] At 5-min mark: no audible drift, console clean of warnings, beat counter aligned

### 5. Per-voice routing (CRITICAL)
- [ ] Configure: voice 0 → MIDI port 1, voice 1 → synth, voice 2 → off
- [ ] Hold middle C → only voice 0 (port 1) and voice 1 (synth) audible
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
