# Companion Architecture

**Status:** Draft, locked at session start 2026-05-04 (Sun)
**Supersedes:** `01-looper.md` (the original brief — kept for reference, scope expanded)
**Jam target:** Thu 2026-05-07
**Author:** Vibhav, drafted with Claude Code

---

## TL;DR

Promote **Companion** as the umbrella concept: an automated bandmate that plays alongside the user. The existing pattern programmer becomes one limb of the companion. We add multi-slot loops (input + output sources) as a second limb. Auto-key joins as a third limb. The router thread executes whatever ops the companion emits each tick, instead of embedding pattern decision logic directly.

Practical wins:

- Naturally subsumes follow-up issue #91 (pure-function extraction for testability).
- Single `companion.tick()` callback instead of three independent in-router subsystems.
- Multi-slot loops sized as `Vec<LoopSlot>` so the count is configurable per session.
- One `CompanionPanel` UI tab structure replaces the disjoint PatternPanel + (future) LooperPanel + AutoKey controls.

---

## Why "Companion"?

Reading the user's framing of the feature: *a companion that plays stuff on time in the background based on what you're playing in the key you're playing in, combined with auto-key, gives you a tool that comps and helps you jam better*.

Pattern-as-rhythmic-gate is one expression of this. Loops-as-recorded-phrases is another. Auto-key as a passive "follow what I play" mode is a third. They share an audience (the soloing musician), a venue (the jam), and a tick model (the transport clock). Treating them as siblings under a `Companion` umbrella is the cleaner mental model and the cleaner code.

This naming is breaking: existing `pattern_*` Tauri commands, `PatternStore`, `PatternConfig`, etc. all get renamed under the companion namespace. We have no external customers, so this is a one-time migration cost.

---

## Component composition

```
                    ┌────────────────────────────────────────────────────┐
                    │                  COMPANION                          │
                    │  master_enabled: AtomicBool                         │
                    │  pattern: PatternConfig (renamed: CompanionPattern) │
                    │  loops:   Vec<LoopSlot>                             │
                    │  auto_key: AutoKeyConfig                            │
                    │                                                    │
                    │  fn tick(&self,                                    │
                    │     transport: &Transport,                         │
                    │     held: &HeldHarmonies)                          │
                    │     -> Vec<DispatchOp>                             │
                    └────────────────────────────────────────────────────┘
                                          │
                                          ↓ ops
                                ┌──────────────────┐
                                │   Router thread  │
                                │   executes ops   │
                                │   via dispatch_  │
                                │   voice helper   │
                                └──────────────────┘
                                          ↓
                                  harmony engine
                                          ↓
                                  per-voice routing
                                          ↓
                          MIDI ports / synth / outputs
```

**Invariant**: the router thread does no pattern/loop/key decisions of its own. It pulls ops from `companion.tick()`, executes them, returns to next iteration.

---

## Data model

### `Companion` (Rust, `src-tauri/src/companion/mod.rs` — new module)

```rust
pub struct Companion {
    pub enabled: AtomicBool,
    pub pattern: Mutex<CompanionPattern>,
    pub loops: Mutex<Vec<LoopSlot>>,
    pub auto_key: Mutex<AutoKeyConfig>,
}

impl Companion {
    /// Pure function: given current transport position and held inputs,
    /// emit the dispatch ops for this tick. Called once per router-loop
    /// iteration. Side-effect-free — does not touch ports, synth, engine.
    pub fn tick(
        &self,
        transport: &Transport,
        held: &HeldHarmonies,
    ) -> Vec<DispatchOp> {
        let mut ops = Vec::new();
        if !self.enabled.load(Acquire) { return ops; }

        // Pattern lane: gates harmony for held inputs
        let pattern = self.pattern.lock().unwrap();
        ops.extend(pattern.tick(transport, held));

        // Loop lanes: each slot emits its own ops
        let loops = self.loops.lock().unwrap();
        for slot in loops.iter() {
            ops.extend(slot.tick(transport));
        }

        // Auto-key state mutation happens elsewhere (in handle_note_on).
        // Companion.tick is read-only on auto_key state.

        ops
    }
}

pub enum DispatchOp {
    NoteOn  { target: VoiceOutputTarget, note: u8, velocity: u8, channel: u8 },
    NoteOff { target: VoiceOutputTarget, note: u8, channel: u8 },
    AllNotesOff { ports: Vec<u8> },  // CC 123 broadcast
}
```

### `LoopSlot`

```rust
pub struct LoopSlot {
    pub id: u32,
    pub source: LoopSource,
    pub length_bars: u8,
    pub state: LoopState,
    pub buffer: Option<LoopBuffer>,
    pub recorded_at_bar: Option<u32>,  // global bar when recording started
}

pub enum LoopSource { Input, Output }

pub enum LoopState {
    Empty,
    Armed { length_bars: u8 },          // waiting for next bar
    Recording { started_at_bar: u32 },  // capturing
    Playing,                            // looped
    Stopped,                            // buffer present, not playing
}

pub struct LoopBuffer {
    pub source: LoopSource,             // capture-time decision
    pub length_beats: f64,              // length_bars * beats_per_bar at record time
    pub events: Vec<LoopEvent>,
}

pub struct LoopEvent {
    pub beat_offset: f64,               // 0.0 .. length_beats
    pub kind: LoopEventKind,
}

pub enum LoopEventKind {
    NoteOn  { note: u8, velocity: u8, channel: u8 },
    NoteOff { note: u8, channel: u8 },
}
```

### `CompanionPattern` (renamed from `PatternConfig`)

Same struct, same algorithm, same lockstep test. Renamed only:

- File: `src-tauri/src/state.rs::PatternConfig` → `src-tauri/src/companion/pattern.rs::CompanionPattern`
- Method: `cell_index_at` stays
- Algorithm bit-identical Rust↔TS lockstep test stays
- TS mirror: `pattern.svelte.ts::PatternStore::cellIndexAt` → `companion/pattern.svelte.ts::CompanionPatternStore::cellIndexAt`

### `AutoKeyConfig`

Currently a flat `AtomicBool` on AppState plus engine internal state. Pull into a struct under Companion. Keep `set_auto_key` raising `panic_pending` (verified correct in PR #89 review).

---

## Pure function contract

`companion.tick()` invariants (subsume issue #91's F2/F4/F5/M3/H3):

- **F2**: ops emitted by pattern lane do not double-fire when polyphonic input produces overlapping harmonies (e.g., C+E both producing G in Mirror mode).
- **F4**: when pattern lane re-fires a held voice with new routing, the old voice gets a NoteOff op before the new voice's NoteOn op.
- **F5**: when a setter raises `panic_pending` and the pattern lane was about to fire on this tick, the panic-replay's `to_release` set covers the pattern-attacked notes; pattern lane skips this tick.
- **M3**: the first tick after `companion.enabled` flips true seeds `last_pattern_cell` without firing — phase-alignment.
- **H3**: pattern lane skipped when `panic_pending` on this tick.

Plus new looper-specific invariants:

- **L1**: a slot in `Recording` captures only events whose `beat_offset` is within `[0, length_beats)`. Events past the boundary close out the buffer and transition to `Playing`.
- **L2**: a slot in `Playing` emits ops with `beat_offset == (transport.totalBeat - recorded_at_bar*beats_per_bar) mod length_beats` for each event matching this tick's window.
- **L3**: a slot in `Stopped` emits a single `AllNotesOff` op on transition.
- **L4**: a slot transition `Empty → Armed` schedules the recording to start at the next bar boundary (`Armed.armed_at_bar + 1`).
- **L5**: `LoopSource::Output` slots, on replay, emit ops that bypass the pattern lane (since pattern was applied at capture time). `LoopSource::Input` slots emit ops that re-enter the pattern lane (their notes are treated as new held inputs).

All invariants get unit tests. The pure-function shape makes them all reachable without standing up the router thread.

---

## Naming migration (concrete checklist)

**Rust**

| Old | New |
|---|---|
| `state.rs::PatternConfig` | `companion/pattern.rs::CompanionPattern` |
| `AppState::pattern_config` | `AppState::companion: Arc<Companion>` |
| `AppState::pattern_enabled: AtomicBool` | `Companion::enabled: AtomicBool` |
| `commands/engine.rs::set_pattern_enabled` | `commands/companion.rs::set_companion_enabled` |
| `commands/engine.rs::set_pattern_config` | `commands/companion.rs::set_companion_pattern` |
| `commands/engine.rs::set_auto_key` | `commands/companion.rs::set_companion_auto_key` |
| (new) | `commands/companion.rs::set_companion_loop_arm` |
| (new) | `commands/companion.rs::set_companion_loop_stop` |
| (new) | `commands/companion.rs::set_companion_loop_clear` |
| (new) | `commands/companion.rs::set_companion_loop_count` |
| `pattern_config_tests` | `companion_pattern_tests` |

**TS**

| Old | New |
|---|---|
| `lib/stores/pattern.svelte.ts::PatternStore` | `lib/stores/companion.svelte.ts::CompanionStore` (umbrella), with `companion.pattern: PatternLane`, `companion.loops: LoopSlot[]`, `companion.autoKey: AutoKeyLane` |
| `pattern.svelte.ts::cellIndexAt` | `companion/pattern-lane.ts::cellIndexAt` (still lockstepped to Rust) |
| `lib/components/PatternPanel.svelte` | `lib/components/companion/CompanionPanel.svelte` (umbrella) + `companion/PatternTab.svelte`, `companion/LoopsTab.svelte`, `companion/AutoKeyTab.svelte` |
| Adapter `setPatternConfig` | `setCompanionPattern` |
| Adapter `setPatternEnabled` | `setCompanionEnabled` |
| Adapter (new) | `armCompanionLoop(slotId, source, lengthBars)` |
| Adapter (new) | `stopCompanionLoop(slotId)`, `clearCompanionLoop(slotId)`, `setCompanionLoopCount(n)` |

**localStorage keys**

| Old | New |
|---|---|
| `'contrapunk-pattern'` | `'contrapunk-companion'` (one blob: `{pattern, loops:[...lengthAndSource], autoKey, enabled}`) |

Loops persist their length and source preference but **not** the buffer (matches loop-pedal mental model).

**Migration shim**: on first hydrate after the rename, read `'contrapunk-pattern'` if present, copy into the new `'contrapunk-companion'` shape under `companion.pattern`, delete the old key. One shot, then forget.

---

## UI surface

```
┌─ Status bar pip row ──────────────────────────┐
│  ◉  Transport      ◉  Companion (master)      │
└───────────────────────────────────────────────┘

┌─ CompanionPanel ──────────────────────────────┐
│  Tabs: [ ▣ Pattern ] [ Loops ] [ Auto-Key ]   │
│  ─────────────────────────────────────────    │
│  PatternTab content here, identical layout    │
│  to existing PatternPanel:                    │
│    - subdivision selector                     │
│    - length selector                          │
│    - input mode (live | gated)                │
│    - cell strip                               │
│    - clear / fillAll                          │
└───────────────────────────────────────────────┘

┌─ CompanionPanel ──────────────────────────────┐
│  Tabs: [ Pattern ] [ ▣ Loops ] [ Auto-Key ]   │
│  ─────────────────────────────────────────    │
│  Slot count: [ 1 | 2 | 3 | ▣ 4 | + Add ]      │
│                                                │
│  ┌─ Slot 1 ──┐  ┌─ Slot 2 ──┐                 │
│  │ Capture:  │  │ Capture:  │                 │
│  │ [In|▣Out] │  │ [▣In|Out] │                 │
│  │ Length:   │  │ Length:   │                 │
│  │ [1 ▣2 4 8]│  │ [▣1 2 4 8]│                 │
│  │ ── State ─│  │ ── State ─│                 │
│  │  ●Record  │  │  Playing  │                 │
│  │  bar 1/2  │  │  bar 3/1  │                 │
│  └───────────┘  └───────────┘                 │
│  ┌─ Slot 3 ──┐  ┌─ Slot 4 ──┐                 │
│  │  empty    │  │  empty    │                 │
│  └───────────┘  └───────────┘                 │
└───────────────────────────────────────────────┘
```

**MIDI Learn integration** (PR #88): each slot exposes its arm/stop/clear commands as discoverable knob targets. Stable command names: `companion_loop_arm_slot_{N}` so MPK pads can map persistently. The MIDI learn map regenerates when slot count changes.

---

## WASM parity

Browser-side companion lives in `wasm/src/lib.rs` mirroring the native shape. JS-side stores `LoopBuffer` in a `Map<slotId, LoopBuffer>` since the WASM engine doesn't have its own router thread (events flow synchronously through the wasm engine's `note_on`/`note_off` calls).

Pattern parity already shipped (we believe — verify in pre-flight). Loop parity adds the same `arm`/`stop`/`clear` surface to the wasm bindings.

---

## Phases (build order)

The user has confirmed: all 6 phases, no shortcuts.

### Phase 0 — Pre-flight gate
**Why first:** validate pattern infra before stacking loops on it. Catch known issues #90, #91, P0 #2 hazards.
**Output:** automated gate green + manual UAT checklist passed
**Artifacts:** updates to this doc with any pattern bugs surfaced
**Effort:** 45 min

### Phase 1 — Pure-function extract + Companion skeleton + tests
**Why second:** issue #91 is the regression net. The Companion abstraction *is* the extracted function. Any further phase that adds emitters (loop slots, auto-key passive accompaniment) attaches as another op-source.
**Output:**
- New `src-tauri/src/companion/` module
- `Companion::tick() -> Vec<DispatchOp>` pure function with F2/F4/F5/M3/H3 unit tests
- Router thread refactored to call `companion.tick()` and execute ops
- Pattern behavior bit-identical to today (regression test: same lockstep table + new behavior tests)
- Full naming migration (Rust + TS + Tauri commands + localStorage key)
**Effort:** 1.5–2 days

### Phase 2 — Single-slot looper, both Input + Output sources
**Why third:** demo-quality minimum. Both sources because user locked dual-mode. One slot because slot-count is a UI concern, not a backend concern (`Vec<LoopSlot>` already supports N).
**Output:**
- `LoopSlot::tick()` with L1–L5 invariants and unit tests
- Capture taps: input mode (4 sites in `handle_note_on/off`/`inject_note_on/off`); output mode (1 tap at `dispatch_voice` site, post-harmony, post-pattern)
- LoopsTab UI (single slot for now)
- Tauri commands: `arm_companion_loop`, `stop_companion_loop`, `clear_companion_loop`
- Adapter methods + types
**Effort:** 1.5–2 days

### Phase 3 — WASM parity
**Why fourth:** browser users get the feature. Native users already have it from Phase 2 if they update.
**Output:**
- WASM bindings for companion + loop slots
- JS-side LoopBuffer storage
- Browser smoke test: record + replay a 4-bar loop in `app.contrapunk.com`
**Effort:** 0.5–1 day

### Phase 4 — Multi-slot (N configurable)
**Why fifth:** Phase 2's single slot is `Vec<LoopSlot>::with_capacity(1)` already; this phase exposes N to the UI. Most of the work is UI + stable MIDI Learn slot identifiers.
**Output:**
- Slot count selector in LoopsTab
- Per-slot UI rendering
- Per-slot MIDI Learn binding stability (stable command names, regenerated on count change)
- Per-slot persistence (length + source toggle, not buffer)
**Effort:** 1 day

### Phase 5 — UI consolidation: CompanionPanel umbrella
**Why sixth:** the existing PatternPanel becomes a tab inside CompanionPanel. AutoKey gets its tab. Single companion-master pip in StatusBar replaces the panel-pip-per-feature pattern.
**Output:**
- `CompanionPanel.svelte` with tabs (Pattern | Loops | Auto-Key)
- Existing pattern panel content moved verbatim into PatternTab.svelte
- AutoKey controls extracted into AutoKeyTab.svelte
- StatusBar pip becomes single Companion master toggle (replaces pattern-pip)
**Effort:** 1 day

### Phase 6 — Pattern + Loop combo polish
**Why last:** the killer combo (Input loop + pattern programmer = chord track + auto-strum) needs edge-case work. Capture-during-pattern-on-cells, replay during pattern-off, transport stop mid-loop, etc.
**Output:**
- Documented and tested edge cases in the LoopSlot + CompanionPattern interaction
- Visual loop indicator on piano keyboard during playback (if confirmed)
- Demo video script tested
**Effort:** 0.5 day

**Total realistic estimate: 5.5–8 days.**

### Timeline reality check vs jam (Sun → Thu = 3 days)

The user said "all 6 phases, no shortcuts." Realistic estimate exceeds the jam window. Three honest paths:

1. **Move the jam date** — push it to ~Thu 2026-05-14 (10 days). Phases 0-6 fit comfortably.
2. **Ship through Phase 3 by Thu, finish 4-6 the following week** — jam has working single-slot looper (both modes) on the new Companion architecture, multi-slot + UI consolidation lands ~05-09. Demo video scripts use single-slot. The "companion" naming is in by jam day.
3. **Ship through Phase 2 by Thu (native only)**, then Phases 3-6 the following week — jam has native looper but no browser parity by demo. Risky if jam attendees expect web access.

**Recommended path: 2.** It uses the 3 days for the architecturally hardest work (Phase 0+1+2+3) and saves the additive UI work for next session. The jam demo doesn't need 4 slots — 1 slot recorded twice tells the same story.

---

## Decisions locked

1. ✅ Companion as umbrella concept (pattern + loops + auto-key as limbs)
2. ✅ Both Input + Output loop sources, per-slot toggle
3. ✅ N configurable slots (UI selector, default starts at 4)
4. ✅ Pre-flight gate before Phase 1
5. ✅ Architecture artifact (this document)
6. ✅ Full rename (Tauri commands + types + storage keys + files), no backwards-compat shim except the one-shot localStorage migration

---

## Open questions

1. **Timeline path** — option 1, 2, or 3 from the deadline reality check above.
2. **Recording start** — arm-and-wait-for-next-bar (recommend) or start-immediately on Record-button press?
3. **Default loop slot count on first run** — 1, 2, or 4?
4. **MIDI Learn binding for per-slot loop arm/stop on day 1** — or punt to Phase 6?
5. **Visual loop indicator on piano during playback** — Phase 2 polish or Phase 6 polish?
6. **AutoKey tab scope in Phase 5** — just expose the existing `set_auto_key` toggle, or also surface key/mode confidence + visualizations?

---

## Acceptance criteria (Phase 6 gate)

- [ ] All 6 phases shipped, all unit tests green
- [ ] No regressions in existing pattern behavior (validated by lockstep test + visual comparison against pre-rename behavior)
- [ ] User can: arm a loop slot, record N bars, hear it loop. Toggle between Input and Output sources. Stop and clear. Multiple slots playing simultaneously without timing drift.
- [ ] Native + WASM parity (record + replay a 4-bar loop in `app.contrapunk.com`)
- [ ] No stuck notes on loop stop/clear (defensive CC 123 broadcast)
- [ ] Per-slot MIDI Learn discoverable on knob/pad map
- [ ] Companion master toggle in StatusBar replaces the panel-pip pattern
- [ ] AutoKey functionality preserved and accessible from Companion AutoKey tab

---

## References

- `01-looper.md` — original brief, superseded
- `.planning/phases/bpm-clock/bpm-clock-LEARNINGS.md` — pattern infra learnings, F2-H3 invariants
- Issue [#90](https://github.com/contrapunk-audio/contrapunk/issues/90) — `held_harmonies` stale-entry recovery
- Issue [#91](https://github.com/contrapunk-audio/contrapunk/issues/91) — router-loop pure-function extraction (subsumed by Phase 1)
- `.planning/STATE.md` P0 #2 — stuck MIDI notes on settings change (looper inherits this hazard; defensive CC 123 mitigation in this work)
- HANDOFF.json — paused-session context that triggered this design
- PR #87 (PerformanceView), PR #88 (MIDI Learn), PR #89 (BPM-clock + Pattern programmer)

---

## Pre-flight gate manual UAT checklist

Run this in `cargo tauri dev` before Phase 1 starts. Each item below has a clear pass/fail criterion. If items 1, 3, or 5 fail, stop and fix the underlying pattern bug before continuing.

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

### Cleanup
- [ ] Close PatternPanel → pattern disables → all held harmony released
- [ ] Press transport stop → all notes released
- [ ] App can be closed cleanly with no errors

---

**End of architecture document. Phase 1 begins after Phase 0 (pre-flight gate) passes.**
