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

Every feature in the 9-week jam pipeline that emits MIDI in time fits as a Lane. **The pattern programmer (cell-grid mask on held inputs from PR #89) was deleted (commit `1376c4c`) — wrong abstraction.** The companion's actual shape is **capture-and-replay**: LooperLane is the primary Lane impl, BeatMachine (Logic Drummer-style), Arpeggiator, Drone, Pad, ChordSeq, AutoKey all follow the same Lane shape.

**Part 2 — Audio architecture** (replaces today's linear chain). Audio graph (DAG) of typed Nodes:
- `Instrument` trait separates sources from processors. Multiple parallel instruments (synth, drum sampler, plugins, drone) coexist.
- `Mixer` Nodes sum sources and feed into FX chains.
- Routes have first-class gain/pan/mute. Sub-buses, parallel sends, sidechaining all expressible.
- `MidiRouter` matrix maps MIDI sources (router thread, BeatMachineLane, etc.) to Instrument destinations — no more broadcast.
- **`AudioGraphCommander`** with batched **transactions** — atomic multi-edge mutations applied at one block boundary. `insert_between`, `swap_nodes`, `move_node_in_chain` operations make the graph reorderable from the UI without code changes.
- **Sidechaining is first-class** — typed sidechain input ports on Compressor/Gate/Multiband, plus a dedicated `SidechainSense` envelope-follower Node. Standard patterns (kick→bass duck, vocal duck, multiband sidechain) ship as rig templates.
- **Elixir replaces BasicSynth** as the tonal Instrument over time. Wavetable + multisample + sample + granular + spectral hybrid; deep modulation; harmony-aware mod sources (`HarmonyDegree`/`Tension`/`Key`/`Mode`); shared `@elixir/ui` Svelte component library. Elixir is its own phase track, runs in parallel with Companion phases. BasicSynth is transitional until Elixir Phase E11 lands.

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
   LoopSlot Lane × N      ← primary companion abstraction (capture+replay)
   BeatMachine Lane
   Arpeggiator Lane
   Drone Lane
   AmbientPad Lane
```

**Concrete example:** AutoKey detects you're vibing in D minor (Sense). Engine snapshot now reflects the new key. ChordSeq's "next chord" lookup uses the updated key (Mutate). Drone Lane reads the new tonic from the engine snapshot and emits a low D drone (Decide). Each lane sees a coherent world.

Without phase ordering: Drone might read stale key while AutoKey is mid-update; ChordSeq fires harmony in the old key. Race conditions become latent musical glitches.

### Input filter — split keyboard / live channel

The user wants to play live melody while loops + drone + beats run in the background. Without `input_filter`, every Lane sees every press, and Lanes that capture (LooperLane) or transform (ArpeggiatorLane) live input would interfere with melody-register notes meant to fly free.

Default Lane filters:
- **LooperLane (Input source)**: `All` (records anything user plays). Configurable to `NoteRange` for chord-only capture.
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

### LooperLane × N (Phase 2 — Wk 1) — **the primary companion abstraction**

The pattern programmer (cell-grid mask on held inputs) was tried and removed
(commit `1376c4c`); it was the wrong shape. The companion's actual unit is
**capture-and-replay**: the user plays, a slot records, the slot loops back
through the harmony engine. Loops are how the companion "plays alongside" you.
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

The pattern-specific F2/F4/F5/M3/H3 + P1 invariants from PR #89 were retired
along with the pattern programmer (commit `1376c4c`). The remaining invariants
are looper- and beat-machine-specific.

### Input pipeline

- **P1 (still relevant for future Lanes)**: `handle_note_on` consults Companion before default harmonize. If a Lane returns `suppress_default=true`, default `harmonize_note_on` does not fire. Used by ArpeggiatorLane (suppresses input, emits arp pattern instead) and LooperLane Input source (captures press; harmony also fires through default).

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

---

## AudioGraph mutation API

The graph is *data*, not code — `nodes: Vec<NodeEntry>` + `routes: Vec<Route>`. Mutability is first-class. Reordering, inserting between, swapping FX positions are all data edits, not code rebuilds.

### `AudioGraphCommander`

Analogue of today's `ChainCommander`. Main-thread handle that pushes mutations onto a lock-free SPSC queue consumed by the audio thread.

```rust
pub struct AudioGraphCommander { /* ... */ }

impl AudioGraphCommander {
    // Node lifecycle
    pub fn add_node(&self, node: Box<dyn Node>) -> Result<NodeId, GraphError>;
    pub fn remove_node(&self, id: NodeId) -> Result<(), GraphError>;
    pub fn replace_node(&self, id: NodeId, new: Box<dyn Node>) -> Result<(), GraphError>;
    pub fn bypass_node(&self, id: NodeId, bypass: bool) -> Result<(), GraphError>;

    // Route lifecycle
    pub fn connect(
        &self,
        from: (NodeId, usize),
        to: (NodeId, usize),
        gain: f32,
    ) -> Result<RouteId, GraphError>;
    pub fn disconnect(&self, route: RouteId) -> Result<(), GraphError>;
    pub fn set_gain(&self, route: RouteId, gain: f32);
    pub fn set_mute(&self, route: RouteId, mute: bool);
    pub fn set_pan(&self, route: RouteId, pan: f32);

    // Convenience operations (compositions, sent atomically)
    pub fn insert_between(&self, edge: RouteId, new_node: Box<dyn Node>) -> Result<NodeId, GraphError>;
    pub fn move_node_in_chain(&self, id: NodeId, new_position: usize) -> Result<(), GraphError>;
    pub fn swap_nodes(&self, a: NodeId, b: NodeId) -> Result<(), GraphError>;

    // Transactions
    pub fn transaction(&self) -> GraphTransaction<'_>;
}
```

`insert_between` is the killer move: you have `Synth → Reverb → Output`, you grab the route between Reverb and Output, drop a Delay onto it, and it becomes `Synth → Reverb → Delay → Output` with no manual edge surgery.

### Transactions — atomic multi-edge mutations

A single reorder ("put the delay before the reverb instead of after") is multiple edge ops. Applied one-at-a-time on the audio thread, there's a brief moment where audio routes through nothing. The fix:

```rust
commander
    .transaction()
    .disconnect(synth_to_reverb_route)
    .disconnect(reverb_to_delay_route)
    .disconnect(delay_to_output_route)
    .connect((synth, 0), (delay, 0), 1.0)
    .connect((delay, 0), (reverb, 0), 1.0)
    .connect((reverb, 0), (output, 0), 1.0)
    .commit()?;
```

`commit()` packages all six ops into one `Vec<GraphMutation>` message sent over the SPSC queue. The audio thread applies them as a single batch at the next block boundary. No intermediate state; no audio glitch from a half-rerouted graph.

### Validation rules at insert time

`connect` returns `Err` if any of:

- Source node's `output_count <= from_port` (port doesn't exist)
- Target node's `input_count <= to_port`
- Either node doesn't exist (was removed)
- Target is the Output node and already has an input on that port (Output is a sink — exactly one input)
- New edge creates a cycle (DAG enforcement — runs a tarjan/khan check on the proposed graph)
- Sample-rate mismatch (Output node's rate vs source — currently single-rate, future-proofing the API)

UI surfaces validation errors so the user sees "can't connect this — would create a feedback loop" instead of silent failure.

### Live-edit safety (v1: brief mute; v2: gain ramps)

V1 (good enough for occasional rig-edit, NOT for live performance edits):
- Audio thread applies the transaction's mutations between two block callbacks.
- Output buffer briefly silences during the swap (~5-20ms). Audible click possible if there were sustained voices through the removed route.
- Acceptable for "stop, edit my rig, restart playing" workflow.

V2 (deferred to v0.2-ish):
- On `disconnect`: gain ramps from current value to 0 over 5ms, then route detaches.
- On `connect`: gain starts at 0, ramps to target over 5ms.
- Click-free live editing during performance.
- Adds 5ms latency to all mutations and ~10 LOC of state per route.

### UI implications

The AudioGraphCommander surface maps directly to user gestures:

| User gesture | Commander call(s) |
|---|---|
| Drag instrument from sidebar onto canvas | `add_node` |
| Right-click node → Delete | `remove_node` |
| Drag from one port to another | `connect` |
| Click-and-drag on edge to detach | `disconnect` |
| Drop new effect on existing edge | `insert_between` |
| Drag node onto another to swap chain position | `swap_nodes` |
| Right-click node → Bypass | `bypass_node` (toggle internal flag; route stays) |
| Multi-select edits (cut+paste a sub-graph) | `transaction()` batched |

---

## Sidechaining (first-class)

A real audio rig has signal flowing through **and** signal flowing alongside-but-not-summed (control signals). Sidechain compression, ducking, gating, multiband side processing — all of these need a route that taps a signal's *envelope* without summing it into the audible mix.

### Architecture

Sidechaining lives at the audio-graph level, not inside any one Node. Three primitives:

#### 1. Typed audio-rate input ports on relevant Nodes

Compressor, Gate, Multiband, Vocoder, Ducker — each has a dedicated `Sidechain` input port in addition to its main audio inputs. The port count is part of the Node's contract:

```rust
impl Node for Compressor {
    fn input_count(&self) -> usize { 2 }   // 0=main audio, 1=sidechain key
    fn output_count(&self) -> usize { 1 }  // 0=compressed audio
    fn process(&mut self, inputs: &[&[f32]], outputs: &mut [&mut [f32]], frames: usize, channels: usize) {
        let main = &inputs[0];
        let key  = &inputs[1];
        // Compute level from the key, apply gain reduction to main, write to outputs[0].
        ...
    }
}
```

If no sidechain route is connected, port 1 receives a silent buffer — compressor falls back to feed-forward (key = main).

#### 2. `SidechainSense` Node — envelope follower

For cases where the *level* is what matters (not the sample-accurate signal), a dedicated `SidechainSense` Node taps a route, computes a smoothed envelope (RMS or peak), and exposes that envelope as a control-rate output. Useful for routing one node's level to control another node's parameter:

```rust
pub struct SidechainSense {
    pub attack_ms: AtomicU32,
    pub release_ms: AtomicU32,
    pub mode: SidechainMode,    // RMS | Peak
}

impl Node for SidechainSense {
    fn input_count(&self) -> usize { 1 }
    fn output_count(&self) -> usize { 1 }   // 0=envelope (audio-rate, but typically used as control)
    ...
}
```

The output is audio-rate (sample-by-sample envelope value 0..1), so it can drive any audio-rate input or be sub-sampled for control-rate use.

#### 3. Documented patterns as rig templates

```
   Pattern 1 — Kick → bass duck (classic EDM):
      DrumSampler ─→ DrumBus ──→ MainBus ──→ Output
            │                       ▲
            └─→ SidechainSense ──→  │
                                    │
            BassSynth ─→ Compressor─┘ (Compressor's sidechain input = SidechainSense out)

   Pattern 2 — Multiband sidechain (only kick frequencies trigger comp):
      DrumSampler[kick] → MultibandSplit[low only] → SidechainSense → Compressor.sc
      BassSynth → Compressor → MainBus

   Pattern 3 — Parallel sidechain (NY-style drum bus):
      DrumSampler ─┬→ MainBus
                   └→ Compressor (heavy GR) → MainBus (mixed in parallel)

   Pattern 4 — Vocal duck (other tracks duck under vocals):
      VocalIn → SidechainSense → Compressor.sc on InstrumentBus
                              → Compressor.sc on PadBus
```

Patterns 1 and 4 are common enough that the rig editor should ship them as templates ("Add Kick Duck", "Add Vocal Duck") in the UI.

### Rig saving + sidechain

Sidechain edges are just routes with a typed source/destination. The route record (`from: (NodeId, port)`, `to: (NodeId, sidechain_port)`) serializes the same as audio routes — the typed port number captures the role. No special-case sidechain serialization needed.

### Validation

`connect` checks that the destination port accepts audio (most ports do). Sidechain ports specifically accept any audio source — no extra restriction. The Node's own `process` decides what to do with the sidechain signal.

---

### Built-in instruments shipped in v1

| Instrument | type_id | Role | Purpose |
|---|---|---|---|
| `BasicSynth` (transitional — see Elixir section) | `builtin.basic_synth` | Source | Harmony voices, until Elixir replaces it |
| `ElixirInstrument` (replaces BasicSynth long-term) | `elixir.synth` | Source | Tonal voices — wavetable + multisample + sample + granular + spectral hybrid |
| `DrumSampler` | `builtin.drum_sampler` | Source | BeatMachine drum playback |
| `Drone` (Wk 3) | `builtin.drone_synth` | Source | Sustained drone — single-voice synth optimized for hold-forever |
| `Pad` (Wk 9) | `builtin.pad_synth` | Source | Slow-evolving polyphonic pad |

### Plugin host Nodes (any plugin format — see "Plugin host" cross-cutting section)

| Node | type_id format | Role | Notes |
|---|---|---|---|
| `PluginNode` (CLAP) | `plugin.clap.<plugin-id>` | Source or Processor | Decided at runtime by the plugin |
| `PluginNode` (VST3) | `plugin.vst3.<guid>` | Source or Processor | macOS / Windows / Linux |
| `PluginNode` (AU) | `plugin.au.<type-subtype-mfr>` | Source or Processor | macOS only |
| `PluginNode` (LV2) | `plugin.lv2.<plugin-uri>` | Source or Processor | Linux primarily; works elsewhere with the right host runtime |

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

## Tonal Instrument: Elixir (replacement for current synth)

The current `src/synth/voice.rs::Synth` is a tier-1 placeholder — sine/saw/square/triangle osc, one filter, one ADSR, fixed 8-voice pool, all voices share global params. Sufficient for "hear harmony output of the box," insufficient for the product vision.

The user-locked replacement is **Elixir**: a Rust-native, permissively-licensed, Serum-class wavetable synthesizer with multi-engine hybrid, deep modulation, and 3-bus parallel FX. Full design lives at [`/.planning/research/elixir/DESIGN.md`](../research/elixir/DESIGN.md). This section covers only the parts where Elixir intersects the contrapunk audio architecture.

### Two-level audio architecture

```
   ╔══════════════════ Contrapunk audio graph (outer) ══════════════════╗
   ║                                                                     ║
   ║   Inputs ─→ MidiRouter ─→ ┌─────────────────┐                      ║
   ║                            │ ELIXIR          │ ─→ MainBus ─→ FX ─→ ║─→ Output
   ║                            │ (one Instrument │                      ║
   ║                            │  Node from      │                      ║
   ║                            │  outer's POV)   │                      ║
   ║                            └────────┬────────┘                      ║
   ║                                     │                                ║
   ║   ╔══════ Elixir internal architecture (inner) ══════╗              ║
   ║   ║                                                   ║              ║
   ║   ║   note_on → VoiceManager (MPE, polyphony)        ║              ║
   ║   ║              ↓                                    ║              ║
   ║   ║   Per-voice: 3 oscs (Engine trait: WT/MS/Sample/  ║              ║
   ║   ║              Gran/Spectral) + Sub + Noise + 2     ║              ║
   ║   ║              filters → voice output               ║              ║
   ║   ║              ↓                                    ║              ║
   ║   ║   Sum voices → 3-bus parallel FxGraph            ║              ║
   ║   ║              ↓                                    ║              ║
   ║   ║   Master mix                                      ║              ║
   ║   ║                                                   ║              ║
   ║   ║   Modulation graph: 10 LFOs, 4 envs, 8 macros,   ║              ║
   ║   ║   ≥64 mod slots feeds every parameter            ║              ║
   ║   ╚═══════════════════════════════════════════════════╝              ║
   ║                                                                     ║
   ╚═══════════════════════════════════════════════════════════════════════╝
```

The contrapunk audio graph treats Elixir as **one Instrument Node**. Elixir's internal complexity (voice manager, mod graph, internal FX buses) is opaque from outside. From outer's perspective: MIDI in via `Instrument::midi_event`, audio out via `Node::process`, params via `set_param`/`get_param`, preset via `serialize_state`/`deserialize_state`.

### Mapping Elixir → contrapunk Instrument trait

```rust
// crates/contrapunk-elixir-bridge/src/lib.rs (or inside elixir-engine itself)
pub struct ElixirInstrument {
    synth: elixir_engine::Synth,
}

impl Node for ElixirInstrument {
    fn type_id(&self) -> &str { "elixir.synth" }
    fn role(&self) -> NodeRole { NodeRole::Source }
    fn input_count(&self) -> usize { 0 }
    fn output_count(&self) -> usize { 1 }  // stereo via channels
    fn process(&mut self, _inputs: &[&[f32]], outputs: &mut [&mut [f32]], frames: usize, channels: usize) {
        self.synth.process(outputs[0], channels);
    }
    fn reset(&mut self) { self.synth.reset(); }
    fn set_sample_rate(&mut self, sr: u32) { self.synth.set_sample_rate(sr); }

    fn serialize_state(&self) -> serde_json::Value {
        // Elixir's native preset format is RON, but for rig-level consistency we
        // wrap as JSON. The RON-ness lives at Elixir's standalone preset I/O layer.
        let preset = self.synth.save_preset();
        serde_json::to_value(preset).unwrap()
    }
    fn deserialize_state(&mut self, state: serde_json::Value) -> Result<(), String> {
        let preset: elixir_engine::Preset = serde_json::from_value(state).map_err(|e| e.to_string())?;
        self.synth.load_preset(&preset).map_err(|e| e.to_string())
    }
}

impl Instrument for ElixirInstrument {
    fn midi_event(&mut self, ev: MidiBlockEvent) {
        match ev {
            MidiBlockEvent::NoteOn { note, velocity } => self.synth.note_on(0, note, velocity),
            MidiBlockEvent::NoteOff { note }          => self.synth.note_off(0, note),
            MidiBlockEvent::AllNotesOff               => self.synth.all_notes_off(),
        }
    }
    fn save_preset(&self) -> serde_json::Value { self.serialize_state() }
    fn load_preset(&mut self, p: serde_json::Value) -> Result<(), String> { self.deserialize_state(p) }
}
```

Note: MPE / poly pressure / pitch bend / CC pass-through requires extending `MidiBlockEvent` beyond today's `NoteOn/NoteOff/AllNotesOff` set. That's a small extension to the contrapunk MIDI types, made when the bridge lands.

### BasicSynth deprecation timeline

`BasicSynth` (current `src/synth/voice.rs`) stays in place as a transitional Instrument while Elixir builds out independently. Concretely:

| Stage | Status |
|---|---|
| Today | `BasicSynth` is the only tonal Instrument. Used by harmony engine voices, drone (placeholder), pad (placeholder). |
| **Elixir Phase E1-E7** ships | `BasicSynth` still default. Elixir runs as standalone app; not yet wired into contrapunk. |
| **Elixir Phase E11** ships (contrapunk integration) | `ElixirInstrument` becomes available in contrapunk. New rigs default to it. Old rigs with BasicSynth nodes auto-migrate via rig-load schema migration. |
| Elixir post-E11 stable | `BasicSynth` deleted. Migration shim in rig loader replaces any lingering `builtin.basic_synth` type_ids with `elixir.synth` + a default Elixir preset. |

This is intentionally slow rollout — Elixir is a multi-month build. Pretending we'll have it ready in two weeks would lock companion work behind a synth that doesn't exist. BasicSynth carries the load until Elixir is actually proven.

### Harmony-aware modulation bridge

Elixir's mod matrix exposes contrapunk-specific `ModSource` variants:

```rust
// In elixir-engine — gated behind a `contrapunk-bridge` cargo feature so
// standalone Elixir doesn't depend on contrapunk types at build time.
#[cfg(feature = "contrapunk-bridge")]
pub enum HarmonyModSource {
    HarmonyDegree,    // current scale degree of incoming note (1..7)
    HarmonyTension,   // dissonance score of current chord
    HarmonyKey,       // key root, normalized 0..1 across the 12 keys
    HarmonyMode,      // mode index normalized
}
```

The bridge crate (or contrapunk's adapter) updates these mod source values per audio block from `WorldState.engine_snapshot` + `WorldState.current_chord`. Result:

- **"Filter cutoff opens on tension chords"** — assign `HarmonyTension` → filter cutoff in the mod matrix.
- **"Wavetable position morphs through mode changes"** — `HarmonyMode` → osc wt_pos.
- **"Pad timbre shifts with key"** — `HarmonyKey` → reverb size or osc blend.

This is the contrapunk-specific value Elixir delivers over generic synths. Sells the integration story.

### `@elixir/ui` shared component package

Elixir's standalone Svelte UI uses composable primitives: `Knob`, `EnvelopeEditor`, `ModMatrix`, `WavetableView`, `Oscilloscope`, `LfoEditor`, `FilterPanel`, etc. These are designed component-first per Elixir's design doc — props in, events out, no global stores.

When contrapunk imports Elixir as an Instrument:

- The component library is published as an internal workspace package: `@elixir/ui` in `elixir/ui/src/lib/components/`
- Contrapunk's `ui/package.json` adds `"@elixir/ui": "workspace:*"`
- Contrapunk's CompanionPanel imports Elixir components for any UI that controls Elixir parameters
- `Knob`, `EnvelopeEditor` etc. become reusable across the whole app — same look, same feel

This is upside that doesn't show up in the audio architecture but matters for product cohesion.

### FX policy: Elixir-internal vs contrapunk-graph-level

Elixir has its own internal 3-bus FX graph (Compressor / EQ / Reverb / Delay / Distortion / Chorus / Hyper / Dimension / etc.). Contrapunk's audio graph also has standalone FX Nodes (Reverb, Delay, Bitcrusher, Distortion). Overlap is intentional, not redundant:

| Layer | Scope | Use case |
|---|---|---|
| **Elixir-internal FX** | Per-instrument, per-voice (mostly) | Each Elixir patch is a complete sound — a "preset" includes its own reverb, delay, modulation. Saving a patch saves its FX state too. |
| **Contrapunk-graph-level FX** | Cross-instrument, post-mix, on buses | Master reverb across all instruments. A drum bus compressor. A side-chained ducker. Things that can't live inside one instrument because they span multiple. |

**Code sharing — locked: shared `contrapunk-dsp` crate.** DSP algorithms (Reverb FDN, Delay, Compressor, EQ, Distortion, Bitcrusher, Chorus, etc.) live in `crates/contrapunk-dsp/` — pure DSP, no allocs, no Tauri/UI deps. Both Elixir-internal `FxModule` impls and contrapunk-graph Processor `Node` impls are thin wrappers around the same algorithms.

```
   crates/
     contrapunk-dsp/                         ← NEW. Pure DSP. No allocs, no UI deps.
       src/
         reverb.rs                            ← FDN reverb (shared)
         delay.rs                             ← stereo delay (shared)
         compressor.rs                        ← FF compressor (shared)
         eq.rs                                ← parametric EQ (shared)
         distortion.rs                        ← multi-mode (shared)
         bitcrusher.rs                        ← (shared)
         ...
     contrapunk-audio/
       src/fx/
         reverb.rs                            ← Node impl wrapping contrapunk_dsp::Reverb
         delay.rs                             ← ditto
         ...
     elixir-engine/
       src/fx/
         reverb.rs                            ← FxModule impl wrapping contrapunk_dsp::Reverb
                                                (with extra mod-matrix integration glue)
         ...
```

Single source of truth for DSP — bug fixes land in one place. Quality bar consistent across contrapunk and Elixir. Elixir's tighter mod-matrix integration becomes thin glue, not a parallel implementation. Trigger for the extract: Phase G2 (when migrating Reverb/Delay to Nodes anyway).

### Elixir as its own phase track

Per user direction: **Elixir is its own phase track**, not coupled to any specific jam-pipeline week. Elixir Phases E1-E11 (per Elixir DESIGN.md) run in parallel with the Companion phases:

- E0 prereqs (spike work) — non-blocking, can happen any time
- E1 hello-synth (standalone Tauri+Svelte minimal) — independent of Companion
- E2-E10 (engines, modulation, FX, file formats) — independent of Companion
- E11 contrapunk integration — when the user is ready to deprecate BasicSynth

Companion work doesn't block on Elixir, and Elixir's standalone shell ships its own demos. The two converge at E11.

The Elixir effort is real (~3-6 months part-time per the design doc's pacing). It is documented separately and tracked as its own track in the broader project plan, not interleaved into the day-by-day Companion phasing.

### Elixir license

**Locked: dual MIT / Apache-2.0.** Standard Rust ecosystem default (matches `cargo new`). Apache provides explicit patent grant; MIT for simplest adoption. Most upstream Rust crates pick this combo, so dependency licensing is automatic. `crates/elixir-engine/LICENSE-MIT` + `LICENSE-APACHE` from day one.

(Contrapunk currently has no LICENSE file — should match Elixir's dual-license to avoid mismatch when E11 integrates. Tracked as separate todo, not blocking.)

### Standalone Elixir vs early contrapunk integration

**Locked: standalone-first per Elixir DESIGN.md.** Standalone Elixir is a product in its own right — open-source Serum clone with independent users / demos / marketing. Contrapunk integration at E11 extends the standalone shell, doesn't replace it.

Re-evaluation gate: after E5-E7 (when Elixir has WT engine + filter + voice manager + FX), check whether the standalone shell still earns its keep or whether folding integration earlier would be cleaner. User-driven check, not pre-scheduled.

For now: standalone-first stays locked. BasicSynth carries contrapunk's tonal load through Companion + Audio + Rig phases. No coupling between Elixir's pace and contrapunk's pace until E11.

---

## Cross-cutting audio concerns

This section addresses architectural questions that span Part 2 — threading, parameter safety, engine boundary, plugin host design, multi-output routing, MIDI event extension. These are decisions that affect every Node and how the audio thread interacts with the rest of the system.

### Thread responsibilities

```
   Audio thread (cpal callback, realtime, no allocs / no locks)
   ─────────────────────────────────────────────────────────────
     • Owns AudioGraph + all Nodes
     • Drains MidiRouter SPSC queue at top of each block
     • Drains AudioGraphCommander SPSC queue at top of each block
     • Calls AudioGraph.process() — writes to output buffer
     • Reads Node params via lock-free atomics (smoothing layer applies)
     • NEVER touches WorldState, HarmonyEngine, Tauri commands

   Router thread (companion home, near-realtime ~60-200 Hz)
   ─────────────────────────────────────────────────────────────
     • Owns Companion + WorldState (Mutexes for held_inputs, sounding_voices, …)
     • Listens for transport beat-crosses (channel from audio thread)
     • Listens for MIDI input events (from midir thread + Tauri commands)
     • Runs Companion.tick() and Companion.on_input()
     • Pushes DispatchOps onto MidiRouter SPSC queue → audio thread
     • Reads/mutates HarmonyEngine briefly under Mutex

   Main thread (Tauri command handlers, UI)
   ─────────────────────────────────────────────────────────────
     • Handles Tauri commands from the Svelte UI
     • Dispatches to router thread via mpsc / signal channels
     • Reads WorldState briefly for display (chord_name, sounding voices)
     • Pushes graph mutations onto AudioGraphCommander SPSC queue
     • Handles file I/O for rig save / load (off the audio path entirely)
```

**Lock contract**:

| Resource | Audio thread | Router thread | Main thread |
|---|---|---|---|
| `AudioGraph.nodes` | full ownership; mutates by draining commander queue | — | — (mutates only via commander) |
| `Node::params()` (atomic store) | read (lock-free) | read/write (lock-free) | read/write (lock-free) |
| `WorldState.held_inputs` etc | — | brief Mutex | brief Mutex (UI display) |
| `HarmonyEngine` | — | brief Mutex (Sense reads, Mutate writes, default harmonize) | brief Mutex (UI commands) |
| `MidiRouter` SPSC | drain only | push only | — |
| `AudioGraphCommander` SPSC | drain only | — | push only |

**Audio thread realtime invariants**:

- No `Box::new`, `Vec::push`, `String` allocation
- No `Mutex::lock` (only `try_lock` if absolutely needed; even that's avoided)
- No I/O, no logging, no `println!`
- No `panic!` or unwinding (use `unwrap_or_default()` patterns; debug_assert in test builds)
- No floating-point denormals (clamp / underflow-prevent in DSP loops)

### Real-time parameter safety: standardized `ParamStore`

Every Node exposes its parameters through a uniform interface. UI commands, MIDI Learn, mod-matrix targets, and rig serialization all use the same surface.

```rust
pub trait Node: Send {
    // ... existing methods ...
    fn params(&self) -> &dyn ParamStore;
}

pub trait ParamStore: Send + Sync {
    fn get(&self, id: ParamId) -> f32;
    fn set(&self, id: ParamId, value: f32);
    fn iter(&self) -> Box<dyn Iterator<Item = (ParamId, ParamMeta)> + '_>;
}

pub type ParamId = u32;

pub struct ParamMeta {
    pub id: ParamId,
    pub name: String,            // "Filter Cutoff"
    pub min: f32,
    pub max: f32,
    pub default: f32,
    pub unit: String,            // "Hz", "ms", "dB", "ratio", ""
    pub log_scale: bool,
    pub smoothing_ms: f32,       // 0 = no smoothing; default 5.0
}
```

Default impl is `AtomicParamStore`:

```rust
pub struct AtomicParamStore {
    params: Vec<(ParamId, ParamMeta, AtomicU32)>,  // f32 stored as u32 bits
}

impl ParamStore for AtomicParamStore {
    fn get(&self, id: ParamId) -> f32 {
        let raw = self.find(id).unwrap().2.load(Ordering::Relaxed);
        f32::from_bits(raw)
    }
    fn set(&self, id: ParamId, value: f32) {
        let entry = self.find(id).unwrap();
        let clamped = value.clamp(entry.1.min, entry.1.max);
        entry.2.store(clamped.to_bits(), Ordering::Relaxed);
    }
    // ... iter
}
```

Audio-thread smoothing: each Node maintains a per-param `current_value: f32` field that ramps toward `params().get(id)` over the configured `smoothing_ms`. Parameter writes from UI take effect smoothly; sample-by-sample stepping prevents zipper noise.

**This unification unlocks:**
- UI parameter changes: `node.params().set(id, value)` from any thread
- Rig save/load: `for (id, meta) in node.params().iter() { ... }`
- MIDI Learn: stores `(node_id: NodeId, param_id: ParamId)` tuples
- Mod matrix destinations: `(node_id, param_id)` — same shape
- Plugin params: a CLAP/VST3 plugin's params expose the same `ParamStore` interface (the plugin host adapter implements it)

### HarmonyEngine boundary

**HarmonyEngine sits OUTSIDE the audio graph and OUTSIDE the Lane abstraction.** It's its own primitive.

Why not a Node: produces MIDI events, not audio samples. Putting it in the audio graph is a category error.
Why not a Lane: it's a stateful object that Lanes *consume*, not a decision-maker that Lanes orchestrate. AutoKey writes its state, ChordSeq writes its state, harmonize_note_on reads it. Engine is the data Lanes operate on, not a Lane itself.

Access model:

| Path | Who | How |
|---|---|---|
| Read engine config (key, mode, scale) | Sense Lanes, Decide Lanes (occasionally) | `WorldState.engine_snapshot.lock().<getter>()` |
| Mutate engine config | Mutate Lanes (e.g. ChordSeqLane on bar boundary) | `EngineMutation::SetKey(...)` enum returned in `LaneOutput.engine_mutations`; Companion applies via `engine.lock().apply(m)` |
| Default-harmonize on input | Companion's input pipeline (after Lane suppress check) | `engine.lock().harmonize_note_on(note)` directly in `handle_note_on` |
| UI engine commands (set_key, set_mode etc) | Main thread | Tauri command → engine Mutex |

`HarmonyEngine` lives in `AppState`, owned by main thread, Mutex-guarded. Router thread accesses via `Arc<Mutex<HarmonyEngine>>`. Brief critical sections — no audio-thread concern (audio thread never touches it).

### Plugin host (multi-format) integration

**Support CLAP first (Rust-native, easiest), VST3 + AU + LV2 next.** Plugins of any format are wrapped in a single `PluginNode` Node type — the format is implementation detail, not a separate Node class.

Plugins can be either Sources (instruments — note in, audio out) or Processors (effects — audio in, audio out). The `PluginNode::role()` is determined at instantiation by querying the plugin, not baked into the Rust type. This is why we don't call them `ClapInstrument` — a plugin isn't necessarily an instrument.

#### Architecture

```rust
// Format-agnostic Node wrapper — single type used in the audio graph
pub struct PluginNode {
    plugin_id: String,            // e.g. "com.u-he.Diva"
    instance: Box<dyn PluginInstance>,
}

impl Node for PluginNode {
    fn type_id(&self) -> &str { /* "plugin.clap.com.u-he.Diva" etc */ }
    fn role(&self) -> NodeRole { self.instance.role() }
    fn input_count(&self) -> usize { self.instance.input_count() }
    fn output_count(&self) -> usize { self.instance.output_count() }
    fn process(&mut self, inputs: &[&[f32]], outputs: &mut [&mut [f32]], frames: usize, channels: usize) {
        self.instance.process(inputs, outputs, frames, channels);
    }
    fn params(&self) -> &dyn ParamStore { self.instance.params() }
    fn serialize_state(&self) -> serde_json::Value {
        json!({
            "format": self.instance.format(),
            "plugin_id": self.plugin_id,
            "chunk": base64::encode(self.instance.save_state()),
        })
    }
    fn deserialize_state(&mut self, state: serde_json::Value) -> Result<(), String> {
        let chunk = base64::decode(state["chunk"].as_str().unwrap_or(""))?;
        self.instance.restore_state(&chunk)
    }
    // ... reset, set_sample_rate
}

// Format-specific instance trait
pub trait PluginInstance: Send {
    fn format(&self) -> PluginFormat;
    fn role(&self) -> NodeRole;          // queried from the plugin metadata at instantiation
    fn input_count(&self) -> usize;
    fn output_count(&self) -> usize;
    fn process(&mut self, inputs: &[&[f32]], outputs: &mut [&mut [f32]], frames: usize, channels: usize);
    fn midi_event(&mut self, ev: MidiBlockEvent);
    fn save_state(&self) -> Vec<u8>;
    fn restore_state(&mut self, chunk: &[u8]) -> Result<(), String>;
    fn params(&self) -> &dyn ParamStore;
    fn open_gui(&mut self, parent: Option<NativeWindowHandle>) -> Option<NativeWindowHandle>;
    fn close_gui(&mut self);
}

pub enum PluginFormat { Clap, Vst3, Au, Lv2 }

// Per-format implementations live in src/plugin_host/{clap,vst3,au,lv2}/
pub struct ClapInstance { /* ... */ }
pub struct Vst3Instance { /* ... */ }
pub struct AuInstance   { /* ... */ }   // macOS only
pub struct Lv2Instance  { /* ... */ }   // primarily Linux

// Per-format scan + factory exposed via a host module
pub trait PluginHost {
    fn format(&self) -> PluginFormat;
    fn scan(&self) -> Vec<PluginDescriptor>;
    fn instantiate(&self, id: &str, sample_rate: u32) -> Result<Box<dyn PluginInstance>, HostError>;
}

pub struct PluginDescriptor {
    pub format: PluginFormat,
    pub id: String,           // format-specific id
    pub name: String,
    pub vendor: String,
    pub role: NodeRole,       // peeked from metadata if available
    pub file_path: PathBuf,
}
```

#### Format-specific identifiers (used in rig type_ids)

| Format | id format | Example |
|---|---|---|
| CLAP | reverse-DNS plugin id | `plugin.clap.com.u-he.Diva` |
| VST3 | 16-byte UID hex-encoded | `plugin.vst3.5653544D726F6E5253796E5677617665` |
| AU | type/subtype/manufacturer 4cc tags | `plugin.au.aumu.div8.UHE0` |
| LV2 | plugin URI | `plugin.lv2.http://drobilla.net/plugins/mda/Piano` |

Rig loader picks the right host (`ClapHost`/`Vst3Host`/`AuHost`/`Lv2Host`) by parsing the type_id format prefix.

#### GUI hosting

| Format | GUI strategy |
|---|---|
| CLAP | Plugin opens its own native window via `clap_plugin_gui` extension. We track window handle for show/hide. |
| VST3 | Plugin's GUI via `IPlugView`. Native window, separate from contrapunk's. |
| AU | NSView via Audio Unit Cocoa View. Embeds in macOS native window. |
| LV2 | Plugin GUI via X11Window / WaylandSurface (Linux) or platform equivalent. |

No window-in-window embedding. Plugin GUI = separate native window. Contrapunk's UI shows a button "Open GUI" / "Close GUI" per `PluginNode`.

#### Plugin scan caching

- On startup, walk standard paths per format:
  - CLAP: `~/.clap`, `/Library/Audio/Plug-Ins/CLAP`, `%COMMONPROGRAMFILES%\CLAP`
  - VST3: `~/Library/Audio/Plug-Ins/VST3`, `%PROGRAMFILES%\Common Files\VST3`, `~/.vst3`
  - AU: `/Library/Audio/Plug-Ins/Components` (macOS only)
  - LV2: `/usr/lib/lv2`, `~/.lv2`
- Cache scan output in `~/.config/contrapunk/plugin-scan.json`, keyed by `(format, id, file_path, file_mtime)`.
- Re-scan on UI request or when paths change. Cache invalidated entries trigger re-scan.

#### MIDI Learn binding

A bound knob targets `(node_id: NodeId, param_id: ParamId)`. Plugin params discovered via the format's params extension at instantiation (`clap_plugin_params` for CLAP, `IEditController` for VST3, AUParameter for AU, LV2 ports). All formats expose the same `ParamStore` interface, so MIDI Learn doesn't care which format the plugin is.

#### Multi-instance

Each `PluginNode` is a separate plugin instance. Two Diva instances = two Nodes with different NodeIds, independent state, independent windows. Rig save captures all instances independently.

#### WASM / browser

Plugins do not run in browsers regardless of format (no Rust-callable host runtime in WebAudio). When a rig containing `plugin.*` type_ids loads in browser context:
- Loader replaces each `PluginNode` with a `PlaceholderNode` that passes audio through unchanged
- UI shows a warning per missing plugin
- Saving the rig in browser preserves the original `plugin.*` type_id (not the placeholder), so re-loading on native restores the real plugin

#### Phase scheduling

| Plugin format | Lands in | Effort |
|---|---|---|
| CLAP | Phase G3 (alongside MidiRouter — first format) | 3-4 d |
| VST3 | Phase G3.1 (after CLAP works) | 4-5 d (more complex API) |
| AU | Phase G3.2 (macOS-only, after VST3) | 3-4 d |
| LV2 | Phase G3.3 (after AU) | 2-3 d |

CLAP is first because it's Rust-native (clack / clap-host crates) and least mature ecosystem so it benefits most from contrapunk supporting it. VST3 brings the largest plugin universe. AU is macOS-only but mandatory for macOS jam users with AU-only plugins. LV2 is a courtesy for Linux users.

### Multi-output / channel routing

**V1: single stereo `Output` Node baked in.** AudioGraph supports multiple Output Nodes structurally (DAG with multiple sinks), but the v1 `AudioGraphCommander` restricts to one. UI similarly assumes one device.

V2 (deferred until user demand surfaces):

- `Output` Node parameterized: `Output { device_id: String, channel_count: u8, channel_offset: u8 }`
- Multiple `Output` Nodes in one AudioGraph (e.g. monitors out 1-2, headphones out 3-4, hardware FX send out 5-6)
- UI: device picker per Output, route from Buses/Mixers to specific Outputs
- Per-output sample-rate / buffer-size config

Use cases that motivate v2:
- "Drums on out 3-4 to my hardware sampler"
- "Wet/dry split: dry to monitors, wet to FX rack"
- "Stems out for live multi-track recording"

The v1 architecture explicitly does not preclude v2 — graph + commander + serialization all handle multiple Output Nodes already. The UI is the only thing that limits to one in v1.

### MidiBlockEvent — MPE-capable extension

The current `MidiBlockEvent` enum needs to grow to support MPE / pitch bend / CC / poly-pressure for Elixir + future MPE-aware Lanes:

```rust
pub enum MidiBlockEvent {
    NoteOn       { channel: u8, note: u8, velocity: u8 },
    NoteOff      { channel: u8, note: u8 },
    AllNotesOff,
    PitchBend    { channel: u8, value: i16 },                      // -8192..8191 (14-bit signed)
    PolyPressure { channel: u8, note: u8, value: u8 },             // per-note pressure
    ChannelPressure { channel: u8, value: u8 },                    // per-channel pressure
    Cc           { channel: u8, controller: u8, value: u8 },       // any CC, incl. CC74 (MPE Y)
}
```

All variants carry `channel` (was missing in NoteOn/NoteOff before — channel matters for MPE per-note routing).

Migration:
- `src/chain/block.rs` — extend enum
- `src/synth/voice.rs::Synth::midi_event` — handle new variants (BasicSynth ignores them; ok for now)
- `src-tauri/src/commands/engine.rs` — MidiRouter dispatches all variants
- ElixirInstrument / VST3 / AU plugins forward all variants to their plugin's MIDI input

Lands in **Phase G3** (MidiRouter introduction) — natural moment to extend the event surface.

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
| ~~`state.rs::PatternConfig`~~ | DELETED (commit `1376c4c`) — pattern programmer removed entirely |
| ~~`state.rs::PatternInputMode`~~ | DELETED |
| ~~`AppState::pattern_config`~~ | DELETED |
| ~~`AppState::pattern_enabled`~~ | DELETED |
| ~~`commands::set_pattern_enabled` / `set_pattern_config`~~ | DELETED |
| (new) | `AppState::companion: Arc<Companion>` (Phase 1.4) |
| (new) | `Companion::enabled: AtomicBool` (Phase 1.4) |
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
| `synth/voice.rs::Synth` | `audio/instruments/basic_synth.rs::BasicSynth` (transitional) |
| (eventually replaces BasicSynth) | `crates/elixir-engine::Synth` wrapped as `audio/instruments/elixir.rs::ElixirInstrument` |
| `chain/block.rs::AudioBlock` | `audio/node.rs::Node` (broader trait, with `params()` accessor) |
| `chain/block.rs::MidiBlockEvent` | extended to MPE-capable: adds `PitchBend`/`PolyPressure`/`ChannelPressure`/`Cc` variants; all carry `channel` |
| (new) | `audio/params.rs::ParamStore`, `ParamMeta`, `ParamId` |
| (new) | `audio/plugin/mod.rs::PluginNode` (format-agnostic Node wrapper) |
| (new) | `audio/plugin/mod.rs::PluginInstance` trait |
| (new) | `audio/plugin/mod.rs::PluginFormat { Clap, Vst3, Au, Lv2 }` |
| (new) | `audio/plugin/mod.rs::PluginHost` trait + `PluginDescriptor` |
| `plugin_host/clap/host.rs` | `audio/plugin/clap/instance.rs::ClapInstance` (impls `PluginInstance`) + `host.rs::ClapHost` (impls `PluginHost`) |
| (new) | `audio/plugin/vst3/instance.rs::Vst3Instance` + `host.rs::Vst3Host` |
| (new) | `audio/plugin/au/instance.rs::AuInstance` + `host.rs::AuHost` (macOS only) |
| (new) | `audio/plugin/lv2/instance.rs::Lv2Instance` + `host.rs::Lv2Host` |
| (new) | `crates/contrapunk-dsp/` — shared DSP algorithms (Reverb FDN, Delay, Compressor, EQ, etc.) |
| ~~`pattern_config_tests`~~ | DELETED with pattern (commit `1376c4c`) |

### TS

| Old | New |
|---|---|
| ~~`lib/stores/pattern.svelte.ts::PatternStore`~~ | DELETED (commit `1376c4c`) — replaced by upcoming `companion.svelte.ts::CompanionStore` (umbrella) |
| `lib/components/PatternPanel.svelte` | `lib/components/companion/CompanionPanel.svelte` (umbrella) + tabs |
| ~~Adapter `setPatternConfig` / `setPatternEnabled`~~ | DELETED (commit `1376c4c`) |
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

### WebAudio strategy: single AudioWorkletNode hosting the whole Rust graph

**Locked: Path α — single `AudioWorkletNode` runs the entire Rust audio graph.** Compile `contrapunk-audio` (the audio graph crate) to WASM. One `AudioWorkletNode` loads the WASM module, calls `AudioGraph::process()` per buffer. WebAudio nodes wrap it for input source + output destination. Web MIDI feeds events through the same MidiRouter SPSC pattern.

```
   Browser:
     [Web MIDI input] ─→ JS bridge ─→ MidiRouter SPSC ─→
                                                          ┌──────────────────────────┐
                                                          │   AudioWorkletNode        │
                                                          │   (Rust → WASM)           │
   [WebAudio AudioContext.destination] ←─────  output ←──│   AudioGraph::process()   │
                                                          └──────────────────────────┘
```

Rejected: Path β (map each Rust Node to a WebAudio node like ConvolverNode/DelayNode). Reasons:

- Sound parity: same Rust DSP runs native and browser → same audio
- Single audio architecture, no platform divergence in Node implementations
- WebAudio's built-in DSP has different params/quality from ours; using them creates "this reverb sounds different in browser" complaints
- Path α is simpler to maintain and test

CPU at our load (8 voices + 5 FX nodes + drum sampler) is well within modern browser AudioWorkletNode + WASM capacity.

### Plugins not available in browser

Plugins (CLAP / VST3 / AU / LV2) do not run in WASM regardless of format — no host runtime accessible from WebAudio. Rig loader behavior in browser context:

- Detects `plugin.*` type_ids in the rig
- Replaces each `PluginNode` with a `PlaceholderNode` (audio passes through unchanged; UI shows a warning per missing plugin)
- Saving the rig in browser preserves the original `plugin.*` type_id (not the placeholder), so re-loading on native restores the real plugin
- Native-only features the user might use in a plugin-using rig stay listed in the UI as "available on desktop"

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
| **1.6** | ~~PatternLane refactor~~ — pattern was deleted; first concrete Lane impl moves to Phase 2 (LooperLane) | — | superseded |
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

### Elixir phase track (parallel to everything above)

Elixir is its own phase track — runs independently of Companion / Audio / Rig work. The phases below come from Elixir's own [DESIGN.md §6](../research/elixir/DESIGN.md). Sized for solo part-time work; total ~3-6 months end to end.

| Phase | What | Effort |
|---|---|---|
| **E0** | Prereqs (DSP reading, WT format docs, WebGL spike, cpal+midir Tauri spike) | 1-2 wk part-time |
| **E1** | "Hello synth": minimal standalone Tauri+Svelte, sine osc + ADSR end-to-end | 2-3 wk |
| **E2** | Wavetable engine v1 (mipmap antialias, Serum WAV reader/writer, OSC slot A) | 3-4 wk |
| **E3** | Filters + ADSR + LFO (SVF filter, full DAHDSR, mod matrix v0) | 2-3 wk |
| **E4** | Voice manager + MPE + microtuning (MTS-ESP FFI, SCL/KBM) | 3-4 wk |
| **E5** | Three-osc graph + sub + noise + unison | 2-3 wk |
| **E6** | FX rack v1 (Comp, EQ, Reverb, Delay — single bus) | 2-3 wk |
| **E7** | Full modulation system (10 LFOs, 4 envs, 8 macros, ≥64 mod slots, drag-and-drop) | 3-4 wk |
| **E8** | 3-bus parallel FX graph (more FX modules, sidechain, multiband) | 2-3 wk |
| **E9** | Wavetable editor in Svelte (draw/formula/FFT-import) | 4-5 wk |
| **E10** | Additional engines (Multisample → Sample → Granular → Spectral) | 8-12 wk |
| **E11** | **Contrapunk integration** — `ElixirInstrument` Node, `@elixir/ui` workspace package, harmony-aware mod sources, BasicSynth deprecation | 2-3 wk |
| **E12+** | Plugin shells (CLAP/VST3/AU) — deferred indefinitely; design pass needed | TBD |

**Where E11 sits in the contrapunk plan:** scheduled as its own phase, after the Companion + Audio + Rig foundations are stable. The user's stance: "Elixir is its own phase" — not gated on or by any specific jam-pipeline week. Any contrapunk feature week that wants tonal voices (Wk 3 Drone, Wk 9 Pad, ambient pad in general) uses BasicSynth as the placeholder Instrument until E11; rigs migrate automatically when Elixir lands.

---

## Decisions locked

1. ✅ Companion as umbrella concept (loops + auto-key + beats as limbs; pattern programmer deleted in commit `1376c4c` — wrong abstraction)
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
20. ✅ AudioGraph mutation API with batched transactions (atomic multi-edge changes). UI gestures map directly to commander calls.
21. ✅ Validation-at-insert-time for routes (port existence, cycle detection, sink rules).
22. ✅ Sidechaining as a first-class concern — typed sidechain input ports + `SidechainSense` envelope-follower Node.
23. ✅ Standard sidechain patterns ship as rig templates (kick→bass duck, vocal duck, multiband, parallel comp).
24. ✅ Elixir replaces BasicSynth as Contrapunk's tonal Instrument over time. Multi-engine hybrid synth, harmony-aware mod sources.
25. ✅ BasicSynth stays as transitional placeholder until Elixir Phase E11. Auto-migration in rig loader at deprecation time.
26. ✅ Elixir is its own phase track — runs in parallel with Companion phases, not gated on or by jam-pipeline weeks.
27. ✅ Two-level audio architecture: contrapunk graph (outer) + Elixir's internal voice/mod/FX graph (inner). Outer treats Elixir as one Instrument Node.
28. ✅ `@elixir/ui` workspace package — Elixir's Svelte components reused by contrapunk UI for any Elixir-controlling panels.
29. ✅ Three-thread architecture: audio thread (realtime, lock-free) + router thread (companion + WorldState) + main thread (Tauri/UI). Lock contracts enumerated.
30. ✅ Standardized `Node::params() -> &dyn ParamStore` — every Node exposes parameters through a uniform atomic-backed interface. Per-param 5ms smoothing default, configurable per param.
31. ✅ HarmonyEngine sits OUTSIDE the audio graph and OUTSIDE the Lane abstraction. Owned by AppState. Read via `WorldState.engine_snapshot`, mutated via `EngineMutation` enum from Mutate-phase Lanes.
32. ✅ **Multi-format plugin support**: CLAP + VST3 + AU + LV2 — single `PluginNode` Node type wraps `Box<dyn PluginInstance>`. Per-format adapters (`ClapInstance`, `Vst3Instance`, `AuInstance`, `Lv2Instance`).
33. ✅ Plugins are NOT named after instrument/effect role — `PluginNode` is format-agnostic, `role()` is determined at instantiation by querying the plugin. CLAP can host effects; VST3 can host instruments; the Node wrapper doesn't presume.
34. ✅ Plugin format scheduling: CLAP first (Phase G3), VST3 next (G3.1), AU (G3.2 macOS-only), LV2 (G3.3).
35. ✅ Plugin GUI hosting via separate native window per platform (no window-in-window embedding).
36. ✅ Plugin scan caching: `~/.config/contrapunk/plugin-scan.json`, keyed by `(format, id, file_path, file_mtime)`.
37. ✅ V1 ships with single stereo Output Node. Multi-output deferred to v0.2; graph foundation already supports it (multi-sink DAG).
38. ✅ WASM port: single `AudioWorkletNode` hosting whole Rust graph (Path α). Rejected per-Node WebAudio mapping (Path β).
39. ✅ MidiBlockEvent extended to MPE-capable: `NoteOn`/`NoteOff`/`PitchBend`/`PolyPressure`/`ChannelPressure`/`Cc` all carry `channel`. Extension lands in Phase G3.
40. ✅ Shared `contrapunk-dsp` crate for FX algorithms — Elixir-internal FxModules and contrapunk-graph Processor Nodes both wrap the same DSP. Single source of truth.
41. ✅ Elixir license: dual MIT / Apache-2.0 (Rust ecosystem default). Contrapunk should match before any open release.
42. ✅ Standalone Elixir first per Elixir DESIGN.md. Re-evaluation gate after E5-E7. BasicSynth carries contrapunk's tonal load until E11.

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
12. **Plugin scan strategy on first launch**: scan blocking with progress UI, or scan on-demand when user opens plugin browser? Recommend on-demand to keep startup fast; scan once user enters the plugin picker.
13. **Sample rate change handling**: cpal occasionally renegotiates SR. Currently `set_sample_rate` ripples through every Node. With multi-output support (v0.2), per-Output SRs would be needed. For v1, single SR throughout the graph; document the assumption.
14. **Plugin sandboxing**: a buggy CLAP/VST3 plugin can crash the audio thread (and the whole app). Run plugins in a separate process? Defer — adds complexity. v1 accepts plugin-induced crash risk; user gets full restart.
15. **Buffer size + latency knob**: contrapunk currently uses cpal's default. Expose buffer size in Settings UI for users on slower machines (256/512/1024). Defer; recordable as v0.2.

(Resolved questions 16+ moved to "Decisions locked" — see #29-#42.)

---

## Architectural debts (deferred to separate sessions)

1. **`panic_pending` is a sledgehammer** — every engine setter triggers full reharm. Adds a Lane → adds another consumer that has to survive panic-replay. Replace with typed `EngineMutation` enum (Phase 6 ChordSeqLane is the natural home).

2. **Issue #90 (held_harmonies stale-entry recovery)** — defensive `CC123 AllNotesOff` ops in Companion-emitted state changes mitigate the symptom for the new code. Root fix (TTL or cross-reference) tracked separately.

3. **Audio thread lock contention** — `Mutex<HarmonyEngine>` reads on the Decide-phase hot path. Profile after Phase 5; if contention shows, swap for `ArcSwap<EngineSnapshot>` and have engine setters publish snapshots.

4. **Router thread testability** — Phase 1 unit-tests Lane impls against the trait. Integration tests for the *router thread itself* (spawn router + scripted MIDI input + assert dispatched events) tracked as a follow-up to issue #91.

5. **Per-voice modular synth (osc/filter/env sub-traits)** — Phase G2 extracts the `Instrument` trait. Sub-modularization (pluggable osc, filter, env per voice) is deferred until a feature needs it (Wk 9 Pad is the natural trigger).

6. **Rig sharing / cloud sync** — all rigs are local in v1. Cross-device sync, share-by-URL, jam.contrapunk.com hosted rig browser are future features. Architecture is ready (JSON serializable everything); product work pending.

7. **Audio graph live editing safety** — adding/removing a Node mid-playback can cause clicks if not handled gracefully. Initial v1: pause audio briefly during graph mutations (acceptable for occasional rig-edit, not for live performance). Crossfade-on-mutation deferred.

8. **Elixir Phase E12+ (plugin shells: CLAP/VST3/AU)** — out of scope. Will need a dedicated design pass on webview-in-plugin tradeoffs (latency, resize, hi-DPI, multi-instance state) vs falling back to vizia native UI for plugin shells. Tracked separately in Elixir DESIGN.md.

9. **Multi-output / multi-device audio routing** — graph supports multiple Output Nodes structurally; v1 commander + UI restrict to single stereo Output. v0.2 promotes this when user demand surfaces (drum-stems-out, FX-send-out, multichannel hardware). No rewrite needed — UI work + commander relaxation.

10. **Plugin sandboxing** — a buggy plugin can crash the audio thread (and the whole app). Out-of-process plugin hosting (à la Bitwig) is significant work. v1 accepts this risk; user gets full restart on plugin crash. Revisit if plugin instability becomes a recurring user complaint.

11. **Per-output sample-rate handling** — when multi-output lands, different devices may have different SRs. Currently single-SR pipeline. v0.2 needs SR conversion at the graph boundary or per-Output SR contexts.

12. **Buffer size + latency control** — Tauri Settings UI to override cpal's default buffer size (256/512/1024). Useful for users on slower machines. Defer to v0.2 polish.

---

## Acceptance criteria (Phase end-state)

### Companion + Audio + Rig (this doc's primary scope)

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
- [ ] AudioGraph mutation API supports `add_node`, `remove_node`, `connect`, `disconnect`, `insert_between`, `swap_nodes`, transactions
- [ ] Validation rejects cycles, port mismatches, output-as-source
- [ ] At least one sidechain template (kick→bass duck) ships as a default rig action
- [ ] Sidechain input ports work end-to-end on Compressor
- [ ] Three-thread architecture validated: audio thread takes no Mutexes during process; router thread holds Mutexes only briefly
- [ ] Every Node exposes parameters via `Node::params() -> &dyn ParamStore`; UI param changes go through this single surface
- [ ] HarmonyEngine accessible only via `WorldState.engine_snapshot` (Lanes) or `EngineMutation` enum (Mutate Lanes); no direct mutation paths leak
- [ ] **Plugin support**: CLAP works (load + play + save state); VST3 works; AU works on macOS; LV2 works on Linux
- [ ] Plugin scan caches to `~/.config/contrapunk/plugin-scan.json`; re-scan on demand
- [ ] Plugin GUI opens in separate native window per format (no window-in-window embedding)
- [ ] Plugins missing on rig load surface a placeholder + warning; rig stays loadable
- [ ] **`contrapunk-dsp` crate exists**; both Elixir-internal FX and contrapunk-graph FX consume the same Reverb/Delay impl
- [ ] **MidiBlockEvent extended** with PitchBend/PolyPressure/ChannelPressure/Cc; channel field on all variants
- [ ] **WASM port works**: single AudioWorkletNode hosts the Rust audio graph; companion + audio + rig save/load all functional
- [ ] **Save / Load rigs from native filesystem**
- [ ] **Save / Load rigs from browser IndexedDB**
- [ ] **Import / Export rigs as JSON files** (cross-platform)
- [ ] **At least one schema migration written + tested** (proves the framework)
- [ ] **CLAP plugin missing-bundle case handled gracefully on rig load**
- [ ] **BasicSynth → Elixir migration path tested** (rig with `builtin.basic_synth` loads after Elixir lands without user intervention)

### Elixir track (separate, deeper criteria in `.planning/research/elixir/DESIGN.md`)

- [ ] E1 ships: standalone Elixir Tauri+Svelte app plays sine osc + ADSR end-to-end
- [ ] E2 ships: wavetable engine renders Serum-format WAVs without audible aliasing
- [ ] E11 ships: `ElixirInstrument` lands as a Node in contrapunk's audio graph; harmony-aware mod sources work; `@elixir/ui` consumed by contrapunk's UI; new rigs default to Elixir; old rigs auto-migrate
- [ ] Acceptance criteria for individual Elixir phases (E0-E11) tracked in Elixir DESIGN.md, not duplicated here

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
- `.planning/research/elixir/DESIGN.md` — full Elixir wavetable synth design (replacement for current src/synth/voice.rs)
- `.planning/notes/elixir-design-decisions.md` — Elixir's six load-bearing decisions
- `.planning/research/elixir/serum-features.md` — feature inventory for the Elixir reference design
- `.planning/research/elixir/oss-prior-art.md` — OSS prior art + Rust plugin ecosystem
- `.planning/seeds/elixir-serum-preset-re-gate.md` — gated future Serum preset import (post-MVP)
- `.planning/todos/pending/elixir-prereqs.md` — Elixir Phase 0 reading list and spike work
- Serum (Xfer Records) — reference workflow for Elixir's wavetable engine

---

## Pre-flight gate manual UAT checklist

Pattern-specific UAT items are RETIRED — the pattern programmer was deleted in commit `1376c4c`. The pre-flight gate going forward is the simpler "metronome + harmony engine still work" smoke test.

### Setup
- [ ] `cargo tauri dev` launches, app window opens
- [ ] No errors in browser DevTools console at app startup
- [ ] StatusBar visible, transport controls work

### 1. Metronome click (CRITICAL — only thing left from "pattern stuff")
- [ ] Toggle metronome on
- [ ] Press play
- [ ] Audible click on each beat at the configured BPM
- [ ] Click stops cleanly on transport stop

### 2. Basic harmony still works
- [ ] Pick a key + harmony mode
- [ ] Hold middle C → harmony voices fire immediately and audibly
- [ ] Release C → all harmony notes release cleanly

### 3. Per-voice routing (CRITICAL — companion will inherit this)
- [ ] Configure: voice 0 → MIDI port 1, voice 1 → synth, voice 2 → off
- [ ] Hold middle C → only voice 0 (port 1) and voice 1 (synth) audible

### Cleanup
- [ ] Transport stop → all notes released
- [ ] App can be closed cleanly with no errors

---

**End of architecture document. Phase 1.4 (WorldState + Lane trait + Companion orchestrator) is next; first concrete Lane impl is LooperLane in Phase 2.**
