# #91 — Companion::tick() router wiring (prep)

**Date:** 2026-05-11
**Status:** Skeleton wiring committed; full integration is multi-commit work.
**Phase:** v1.2.x Phase 2

## What `Companion` is

A three-phase orchestrator (`src-tauri/src/companion/orchestrator.rs:48`) that runs registered `Lane`s in fixed order each tick: **Sense** → **Mutate** → **Decide**. Decide-phase Lanes emit `DispatchOp`s that the router converts to MIDI / synth events.

```rust
pub struct Companion {
    pub enabled: AtomicBool,       // master kill switch — when false, tick() short-circuits
    pub world: Arc<WorldState>,    // shared observable
    pub lanes: Vec<Box<dyn Lane>>, // declaration-order registry
}

impl Companion {
    pub fn tick(&mut self, engine: &Mutex<HarmonyEngine>) -> Vec<DispatchOp>;
    pub fn on_input(&mut self, ev: InputEvent, engine: &Mutex<HarmonyEngine>) -> CompanionInputResult;
}
```

`enabled` defaults to `false` — Companion is invisible to users until Lanes register *and* something sets the flag.

## Key types

`DispatchOp` (`companion/mod.rs:44`):

```rust
pub enum DispatchOp {
    NoteOn { target: VoiceOutputTarget, note: u8, velocity: u8, channel: u8 },
    NoteOff { target: VoiceOutputTarget, note: u8, channel: u8 },
    AllNotesOff { ports: Vec<u8> },
}
```

Targets are the existing `VoiceOutputTarget` enum — `dispatch_voice` already handles `Synth | MidiPort | Off`. The translation from `DispatchOp` to `dispatch_voice` call is a tiny helper.

`InputEvent` (`companion/lane.rs:97`): `NoteOn { note, velocity, channel }`, `NoteOff { note, channel }`, `Cc { channel, .. }`.

`CompanionInputResult { ops: Vec<DispatchOp>, suppress_default: bool }` — `suppress_default` lets a Lane intercept and prevent the existing harmony-engine dispatch from running on the same input.

## Integration boundaries (the design questions)

1. **Ownership**. `Companion` lives on `AppState` as `Arc<Mutex<Companion>>`. The router thread clones the Arc and ticks it; Tauri command handlers (future: enable/disable, register Lane, snapshot state) take brief locks on the same Arc. This matches the `engine: Arc<Mutex<HarmonyEngine>>` pattern already established.
2. **Tick cadence**. Once per outer router-loop iteration (i.e. every ~5ms wall-clock, gated by the existing `rx.recv_timeout(5ms)`). Decide-Lanes that want bar-aligned firing read `WorldState.transport` themselves and short-circuit on off-beats. Centralised cadence keeps the router loop simple.
3. **Dispatch translation**. New helper `dispatch_companion_ops(ops, num_ports, synth_tx, output)` converts each `DispatchOp` to the existing `dispatch_voice` / per-port broadcast. Pure routing — no engine state mutation. Testable in isolation.
4. **Input-pipeline interleave**. `Companion::on_input` runs **before** `process_midi_message`. If it returns `suppress_default: true`, the existing harmony dispatch is skipped. Otherwise both paths run (companion's ops + harmony's notes). This deferred to a follow-up commit — the skeleton wiring lands first to verify the plumbing compiles + tests pass, then `on_input` integrates without touching the tick path.

## Sequencing across commits

Locked decision: ship #91 in three atomic commits, each green on its own.

**Commit A — skeleton (this commit, e8367e0+):**
- `Arc<Mutex<Companion>>` field on `AppState`, constructed with a fresh `WorldState` in `AppState::default()`.
- `start_routing` clones the Arc, passes it to `run_tauri_router`.
- Main router loop calls `companion.lock().tick(&engine)` once per iteration. Returned ops dispatched via a new `dispatch_companion_ops` helper.
- Companion stays `enabled = false` by default → tick short-circuits, behavior bit-identical to pre-#91.
- Test for `dispatch_companion_ops` translating each `DispatchOp` variant to the right `dispatch_voice` call.

**Commit B — input-pipeline interleave (follow-up):**
- `Companion::on_input` called from `process_midi_message` before the harmony dispatch.
- Honor `suppress_default`.
- Tests cover: no-Lane companion → no-op; Lane that suppresses → harmony skipped; Lane that adds ops → both run.

**Commit C — enable/disable + snapshot Tauri commands (follow-up):**
- `companion_set_enabled(bool)`, `companion_get_state() -> CompanionState`, `companion_load_state(state)`.
- Lets the UI toggle the companion and persist Lane state across sessions.

Commit A is what lands in this session. B and C are next-session work.

## Files this touches

- `src-tauri/src/state.rs` — `AppState` gains a `companion` field + initialization.
- `src-tauri/src/commands/engine.rs` — `start_routing` clones the Arc; `run_tauri_router` ticks it; `dispatch_companion_ops` helper + tests.
- `src-tauri/src/main.rs` — no change in commit A (no new Tauri commands yet).

## Anti-pattern to avoid

Don't wire a default Lane in this commit. The temptation is to register e.g. a no-op LooperLane "to prove tick runs". That bakes a Lane into the default code path and makes future Lane work harder to A/B against the empty baseline. Companion stays empty here; Lanes register in their own commits.

## Open questions, deferred to commit B/C

- How does `Companion::tick`'s `DispatchOp::NoteOn` interact with the existing `held_harmonies` tracking? Currently zero overlap (Companion is empty), but as soon as a Lane emits notes the question becomes real.
- Where does the UI register Lanes? Tauri command + serializable Lane factory function? Or pre-register a fixed set at build time? Probably the latter for v1.2.x.
- `WorldState::sounding_voices` vs the router's `harmony_notes` HashSet — two sources of truth. Plan to converge on one in the audio-graph milestone (v1.3.x); for now both live and Companion writes to WorldState while the router writes to its HashSets.
