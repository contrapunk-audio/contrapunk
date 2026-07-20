# Code Context

## Files Retrieved

1. `crates/contrapunk-companion/src/lane.rs` (lines 20-305) — existing transport-lane contract: `HoldMode`, `InputEvent`, `LaneOutput`, `Lane`, persistence, and mandatory `reset_runtime` hook.
2. `crates/contrapunk-companion/src/orchestrator.rs` (lines 20-279) — `Companion` owns all lanes and runs Sense → Mutate → Decide; tagged tick/input APIs preserve lane ownership through dispatch.
3. `crates/contrapunk-companion/src/lib.rs` (lines 1-67) — shared exports and `DispatchOp::{NoteOn, NoteOff, AllNotesOff}` used by both surfaces.
4. `crates/contrapunk-companion/src/voice_output.rs` (lines 1-27) — reusable `VoiceOutputTarget::{Synth, MidiPort, Off}`; do not invent new routing types.
5. `crates/contrapunk-companion/src/counterpoint_lane.rs` (lines 93-140, 282-375, 381-843; tests 843-1330) — closest concrete Decide lane: scheduled events, per-lane output target, config serialization, hold/release, tick, and runtime reset patterns.
6. `crates/contrapunk-companion/src/canon_lane.rs` (not fully retrieved; grep anchors lines 415 and 477) — existing delayed-event lane and target/config precedent.
7. `src-tauri/src/state.rs` (grep anchors lines 155-219) — constructs shared `WorldState`/`Companion` and registers canon and counterpoint lanes.
8. `src-tauri/src/commands/engine.rs` (lines 540-809; grep anchors 102-106, 1343-1400) — router advances/ticks Companion, dispatches tagged ops, tracks sounding lane notes, and clears Companion runtime on CC123 panic.
9. `src-tauri/src/commands/companion.rs` (grep anchors lines 19-256) — thin Tauri configuration commands funneled through `Companion::configure_lane(type_id, partial)`.
10. `wasm/src/companion.rs` (lines 35-264) — browser constructs the same shared lanes, mirrors harmony state, advances transport, exposes config/tick/input/reset, and serializes tagged ops.
11. `ui/src/lib/arrangement/presets.ts` (lines 1-213) — `ArrangementPresetV2`, `ArrangementConfig`, capability detection, and validation. `pattern_lane` is already a declared capability but has no config.
12. `ui/src/lib/arrangement/catalog.ts` (grep anchors lines 644-678, especially 652-654) — Pixel Trio is presently capability-locked because independent low-support/counterline timing is absent.
13. `ui/src/lib/components/EnsemblePresetBar.svelte` (grep anchors lines 8 and 117) — arrangement preset application/UI entry point.
14. `ui/src/lib/arrangement/persistence.ts` (grep anchors lines 10-140) — user-preset load/save/migration/type guard must tolerate and preserve the new config.
15. `ui/src/lib/arrangement/catalog.test.mjs` (grep anchors lines 418 and 518) — current catalog assertions, including Pixel Trio’s independent-low-support wording.

## Key Code

Existing shared contract is already sufficient; PatternLane should be one new `Lane` implementation, not a new scheduler/orchestrator:

```rust
pub trait Lane: Send + Sync {
    fn type_id(&self) -> &'static str;
    fn phase(&self) -> LanePhase;
    fn input_filter(&self) -> InputFilter { InputFilter::None }
    fn tick(&mut self, world: &WorldState) -> LaneOutput;
    fn on_input(&mut self, ev: InputEvent, world: &WorldState) -> LaneOutput;
    fn reset_runtime(&mut self) {}
    fn serialize_state(&self) -> serde_json::Value;
    fn deserialize_state(&mut self, state: serde_json::Value) -> Result<(), String>;
}
```

`Companion::tick_tagged` already invokes every Decide lane and returns `(lane.type_id(), DispatchOp)`; Tauri and WASM already consume that shape. `WorldState` already owns the shared `Transport` and harmony-engine snapshot, so degree resolution should read current key/scale there and timing should use `transport.total_beats()`/running state, as canon/counterpoint do.

### Smallest safe declarative model

Add to `ui/src/lib/arrangement/presets.ts` under `config.companion`:

```ts
pattern: {
  enabled: boolean;
  lengthBeats: number;
  lowSupport: PatternRoleConfig;
  counterline: PatternRoleConfig;
}

type PatternRoleConfig = {
  enabled: boolean;
  target: VoiceOutputTargetConfig;
  events: Array<{ beat: number; degree: number; octave: number; durationBeats: number; velocity: number }>;
};
```

Use one shared Rust `PatternLane` containing exactly two fixed roles. Each runtime role needs only `active: Option<(note, channel, off_at)>` plus loop position/last cycle. On each tick: if transport stopped, emit offs and clear; otherwise process crossed event boundaries, resolve scale degree against the current engine snapshot, emit the prior role off before replacement, then schedule its off. Fixed roles prevent arbitrary lane graphs and map directly to the requested low-support/counterline ownership.

Use integer/fixed tick positions if `contrapunk-transport` exposes them; otherwise copy canon/counterpoint’s existing crossed-beat comparison. Validate finite, non-negative beats/durations; positive loop length; MIDI-safe resolved notes; bounded events (a small hard ceiling) at deserialization. Reject malformed config rather than silently wrapping. Do not put Pixel Trio or any preset name in Rust.

## Architecture

### Exact implementation path

1. **Shared core**
   - Add `crates/contrapunk-companion/src/pattern_lane.rs` with `PatternLane`, `PatternRole`, declarative event/config structs, `Lane` implementation, degree resolver, serialization/deserialization, stop/reset behavior, and focused tests.
   - Change `crates/contrapunk-companion/src/lib.rs` to declare/export `pattern_lane` and `PatternLane`.
   - Reuse `VoiceOutputTarget`, `DispatchOp`, `WorldState`, and the existing Companion registry/configuration APIs. No HarmonyEngine preset branch and likely no change to `orchestrator.rs` or `lane.rs`.

2. **Lane registration/configuration**
   - Change `src-tauri/src/state.rs::AppState::default` registration block (lines 179-190) to push one `PatternLane::new()`.
   - Change `wasm/src/companion.rs::CompanionWasm::new` (lines 41-62) identically.
   - Add one generic `pattern_configure` Tauri command in `src-tauri/src/commands/companion.rs`, mirroring `counterpoint_configure`, calling `configure_lane("pattern", partial)`; register it in `src-tauri/src/main.rs` beside existing Companion commands.
   - Add `CompanionWasm::configure_pattern`, mirroring `configure_counterpoint`; optionally include its state in `debug_snapshot` only if debugging parity is desired.

3. **Dispatch/role targets**
   - No new output destination abstraction: each role uses existing `VoiceOutputTarget`.
   - Tauri’s `dispatch_companion_ops` already routes the target embedded in each op and `companion_output_notes` already tracks routing-aware sounding notes. Inspect its lane-id match before implementation: if it only recognizes `canon`/`counterpoint` for UI sets/mix constants, add `pattern_low_support` and `pattern_counterline` attribution or a role field. The clean minimum is two stable lane tags, which may mean two `PatternLane` instances sharing the same implementation/config shape; this preserves existing lane-tag ownership without changing `DispatchOp`.
   - WASM’s `serialize_tagged_ops` similarly forwards the lane tag. Extend UI decoding/color/mix mapping only where it currently enumerates canon/counterpoint. Do not collapse both roles to `pattern` if independent role output/mix attribution is required.

4. **Preset/UI contract**
   - Extend `ArrangementConfig` and validation/capability inference in `ui/src/lib/arrangement/presets.ts`; `arrangementConfigCapabilities` should add `pattern_lane` when enabled.
   - Extend the common `ContrapunkAdapter` and both implementations in `ui/src/lib/adapter/{tauri,wasm}.ts` with `configurePattern` (same JSON shape on both surfaces).
   - Update preset application in `ui/src/lib/components/EnsemblePresetBar.svelte` to configure/disable PatternLane alongside canon/counterpoint. Applying any preset must send the full pattern state so stale patterns cannot survive a preset switch.
   - Update `ui/src/lib/arrangement/persistence.ts` migration/type guard/default construction so legacy V2 records get `pattern.enabled = false`; because this changes the serialized V2 shape, either keep it optional with a default or bump schema. Optional/default is the smaller backward-compatible change.
   - Put declarative Pixel Trio events only in `ui/src/lib/arrangement/catalog.ts`; remove `pattern_lane` from its missing requirements only after runtime capability reporting includes it.

### Lifecycle, stop, and panic

- **Panic:** Tauri CC123 already broadcasts offs, clears `companion_output_notes`, clears engine active notes, and calls `Companion::reset_runtime()` (`commands/engine.rs` lines 735-760). PatternLane must clear active roles and all cycle cursors there; queued events must never reappear.
- **Transport stop:** `reset_runtime` documentation explicitly says transport stop calls it, but verify the actual stop command does so. PatternLane should also defensively detect `!transport.is_running()` in `tick`, emit NoteOff for active roles once, and reset its cycle anchor.
- **Disable/reconfigure/preset switch:** deserialization/configure must emit or arrange release of active notes before replacing runtime state. Existing `configure_lane` returns no ops, so safest is for adapters/preset application to call reset/panic before disabling/replacing, or minimally require the lane to queue releases for next tick. Silent state clearing alone causes stuck external MIDI notes.
- **WASM:** `reset_runtime` currently only clears state and cannot return NoteOff ops. The JS adapter must first release tracked Companion notes/all-notes-off, then call reset. Check its transport-stop path explicitly.
- **Rewind/tempo/meter:** anchor patterns to monotonic `total_beats`, not bar-relative `beat_position`; reset on backward jumps. Tempo changes then preserve beat placement naturally.

### Tests to reuse/add

- Follow `counterpoint_lane.rs`’s `fixture()` and hold/reset tests (from line 843) for shared `WorldState` + `Transport` setup.
- Follow orchestrator tagged-output/configure/restore tests in `orchestrator.rs` for registry ownership and JSON round-trip.
- Add one PatternLane test covering: two roles with different onset/rest grids; degree resolution after key/scale change; exact offs at duration; transport stop; skipped tick crossing multiple boundaries; reset/panic prevents resurrection; invalid config rejection; independent output targets.
- Extend WASM Companion tests for `configure_pattern` + tagged serialized ops, and Tauri `commands/companion.rs` command tests.
- Extend `ui/src/lib/arrangement/catalog.test.mjs` and preset validation tests: old records default disabled, capability inferred only when enabled, Pixel Trio contains no preset-specific runtime behavior.
- Run `cargo test -p contrapunk-companion`, relevant WASM/Tauri checks, and `npm --prefix ui run check`.

## Risks / Open Questions

1. **Attribution is the main design fork.** `Lane::type_id()` tags a whole lane, not individual ops. Two roles requiring distinct existing role mix/output tracking are simplest as two configured `PatternLane` instances (`pattern_low_support`, `pattern_counterline`) rather than expanding `DispatchOp`/orchestrator. Confirm `Companion::configure_lane` permits unique type IDs; if not, a tiny role tag extension is unavoidable.
2. **Config setters cannot presently emit NoteOff.** Reconfiguration while sounding can stick notes, especially external MIDI. Route changes through an existing panic/all-notes-off path; do not merely clear runtime.
3. **WASM clock quality:** `advance(frames)` is driven by animation/WebAudio glue. Catch-up logic must process crossed boundaries and avoid duplicate cycles after throttled tabs; scheduling precision is bounded by tick cadence.
4. **Scale-degree semantics:** define signed/zero-based versus musical 1-based degrees once, including negative degrees and octave. Reuse harmony scale/key helpers rather than duplicating mode tables.
5. **Preset schema compatibility:** making the new field required breaks stored schema-v2 presets. Prefer optional-on-read/default-disabled, then normalize on save.
6. **No allocation claim:** orchestrator currently collects `Vec` ops per tick, so PatternLane need not introduce a new real-time abstraction, but it should prevalidate/pre-sort events on configuration rather than sort/parse each tick.

## Start Here

Open `crates/contrapunk-companion/src/counterpoint_lane.rs` first. It is the closest complete precedent for a transport-aware Decide lane with output targets, scheduled note ownership, JSON configuration, Hold/NoteOff behavior, reset semantics, and tests. Implement the reusable lane thereabouts, then wire the same instance into Tauri and WASM constructors; keep all preset data in `ArrangementPresetV2`/catalog code.