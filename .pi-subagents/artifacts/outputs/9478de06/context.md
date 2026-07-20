# Code Context

## Files Retrieved
1. `crates/contrapunk-companion/src/counterpoint_lane.rs` (lines 41-181, 191-490, 510-755) - all current scheduling, ownership, release, reset, serialization, and tests; Species IV is explicitly only a half-beat delay.
2. `crates/contrapunk-companion/src/lane.rs` (lines 26-70, 145-180, 228-290) - `HoldMode`, input routing, `LaneOutput`, and cleanup contract.
3. `crates/contrapunk-companion/src/orchestrator.rs` (lines 45-125, 126-254, 256-370) - master gate, tick/input dispatch, configuration, and runtime reset ownership.
4. `crates/contrapunk-companion/src/world.rs` (lines 35-142) - shared transport, held input, sounding voice, and engine snapshot ownership.
5. `crates/contrapunk-transport/src/clock.rs` (lines 1-232) - atomic sample clock, fractional `total_beats`, play/stop/reset/seek behavior.
6. `src-tauri/src/commands/engine.rs` (lines 158-190, 733-765) - Panic/Stop CC123 path drains tracked downstream notes before resetting lanes.
7. `src-tauri/src/commands/companion.rs` (lines 20-80) - master disable only flips an atomic; lane configuration returns no dispatch ops.
8. `wasm/src/companion.rs` (lines 45-75, 125-206) - WASM clock/input/tick/configuration; disable and reset do not return NoteOffs.
9. `plugin/src/lib.rs` (lines 1215-1250) - plugin clears surface note counters and synth notes independently.

## Key Code

Current Species IV branch (`counterpoint_lane.rs:324-344`) computes a pitch immediately from the current input, then only queues it at `now + 0.5`. There is no retained-suspension or resolution state. `PendingCpOn` is source-note tagged, while `HeldCpEntry` owns a vector of nominal emitted pitches (`counterpoint_lane.rs:41-69`). Importantly, that vector includes pitches merely scheduled, not necessarily sounded.

The smallest honest musical state machine belongs entirely in `CounterpointLane`, driven by `world.transport.total_beats()` in `tick`:

- On a monophonic NoteOn, quantize to the **next weak half-beat** (integer beat + 0.5), not `now + 0.5`; choose a consonant preparation against the note currently known.
- Queue preparation `NoteOn` at that weak boundary.
- Retain the same sounding pitch through the following integer strong boundary (no retrigger and no NoteOff).
- Queue/emit preparation `NoteOff` plus resolution `NoteOn` one half-beat later; resolution is exactly `scale.transpose_diatonic(preparation, -1)` (downward diatonic step), with a bounded fallback that aborts the gesture rather than repeating the preparation.
- The resolution should remain owned by the same gesture/source input and receive its eventual NoteOff. A single `Species4Gesture { source_note, channel, velocity, prep, resolution, prepare_at, strong_at, resolve_at, stage }` is simpler and safer than representing the tie as unrelated `PendingCpOn`s. For the stated monophonic scope, allow at most one gesture; a new NoteOn cancels/releases the prior gesture before replacing it.
- Phase boundaries should be calculated from absolute transport beats. `total_beats()` is monotonic only between reset/seek operations (`clock.rs:109-124, 191-213`), so Stop/Reset/seek must clear the gesture.

No new clock abstraction is warranted: the existing atomic `Transport` is the canonical clock and router/WASM/plugin callers already advance/mirror it.

## Architecture

Input reaches `Companion::on_input[_tagged]`, which forwards to the lane. The lane currently computes harmony at input time, buffers beat timestamps, and emits only from `tick`. Tauri/router, WASM, and plugin consume tagged/untagged `DispatchOp`s and maintain their own sounding-note registries. Thus the lane should own **gesture scheduling and pitch lifecycle**, while each surface continues to own actual output dispatch/tracking.

### Findings

- **blocker — `crates/contrapunk-companion/src/counterpoint_lane.rs:324-344`:** current Species IV is only delayed onset; it cannot produce preparation → retained suspension → resolution.
- **blocker — musical correctness limit:** with live monophonic input, the lane can guarantee timing, retention, and downward-step resolution, but cannot guarantee that the retained pitch is a valid dissonant suspension against the *next* strong-beat cantus note. That note does not exist when preparation must sound. Nor can it guarantee the resolution is consonant with a still-later/changed live note. Honest labeling is “transport-scheduled suspension gesture”; strict Fux Species IV needs lookahead, a score/buffer, or delayed cantus playback.
- **high — `counterpoint_lane.rs:124-132, 204-210`:** lane disable/reset clears bookkeeping without emitting NoteOff for already-sounded lane notes. Per-lane disable can strand sound.
- **high — `orchestrator.rs:84-91, 102-106` and `src-tauri/src/commands/companion.rs:30-33`:** master disable short-circuits tick/input and only flips an atomic, so queued cleanup cannot run and sounding notes can stick.
- **high — `wasm/src/companion.rs:69-74, 132-138, 204-206`:** WASM disable/configure/reset cannot return cleanup ops. JS must explicitly silence tracked counterpoint notes, or the API must expose a drain returning tagged NoteOffs.
- **medium — `counterpoint_lane.rs:133-142`:** species change resets scorer history but leaves pending/held Species IV runtime alive under the new configuration.
- **medium — `counterpoint_lane.rs:346-407`:** HoldMode semantics conflict with a three-phase Species IV gesture. Default NearFuture/Cancel may cancel preparation or release it before strong retention. For Species IV, once preparation has actually sounded, the lane must own it through resolution (unless Panic/Stop/disable); HoldMode should only govern a not-yet-sounded gesture, or Species IV must document an override.
- **medium — `counterpoint_lane.rs:61-69, 346-407`:** `emitted_notes` contains scheduled pitches, so NoteOffs may be queued for notes that never sounded; duplicate pitch values also make cancellation by value ambiguous.
- **medium — `world.rs:102-117`:** `held_inputs`/`sounding_voices` are keyed by note, not `(channel,note)`. This is acceptable only under the explicitly monophonic scope and is unsafe for same pitch across channels/retriggers.

### Cleanup recommendation

The clean shared fix is a lifecycle drain, not more silent `clear()` calls: add a lane/orchestrator operation that returns NoteOff `DispatchOp`s for actually-sounded owned pitches and then clears runtime. Use it before master disable, per-lane disable/species change, Stop, Panic, and transport discontinuity. Tauri already has the stronger CC123 tracked-note drain (`engine.rs:158-190, 733-765`), so retain that and reset after dispatch. WASM/plugin must consume returned cleanup ops or explicitly all-notes-off their surface registries. Merely pushing `pending_off` is insufficient because disabled `Companion::tick` never drains it.

## Exact Tests Recommended

In `counterpoint_lane.rs` unit tests:

1. `species4_quantizes_preparation_to_next_weak_boundary`: input at beat 0.2; no output before 0.5; preparation NoteOn at/after 0.5.
2. `species4_retains_same_pitch_across_strong_boundary`: preparation at 0.5; tick through 1.0 emits neither retrigger nor NoteOff.
3. `species4_resolves_down_one_diatonic_step_on_weak_boundary`: at 1.5 emits prep NoteOff then resolution NoteOn; assert scale degree is exactly -1 and op order is off-before-on.
4. `species4_does_not_claim_future_cantus_correctness`: next player note introduced at strong beat can make retained interval consonant/non-suspending; test documents temporal guarantee only.
5. `species4_new_monophonic_input_replaces_prior_gesture_without_orphan`: second NoteOn releases/cancels the first gesture deterministically.
6. `species4_noteoff_before_preparation_cancels_unsounded_gesture`; and `noteoff_after_preparation_does_not_break_required_tie` (unless explicit panic).
7. `species4_stop_panic_disable_and_species_change_emit_off_for_sounded_pitch_and_clear_all_stages`; assert a later tick cannot resurrect notes.
8. `species4_transport_reset_or_backward_seek_clears_gesture`: no stale event fires when beat returns to an earlier value.
9. `species4_scale_lower_bound_aborts_resolution_without_stuck_note`: prep is released even when downward transposition fails.

In `orchestrator.rs`: `disable_drains_before_master_gate_closes` and `reset/drain_collects_each_lane_cleanup_once`.

Surface contract tests: Tauri CC123/Stop includes counterpoint owned note and clears it; WASM disable/reset returns or causes corresponding tagged NoteOff; plugin host stop/seek clears worker/lane ownership and note counters.

## Start Here

Open `crates/contrapunk-companion/src/counterpoint_lane.rs:41-181` first. Replace the Species IV use of generic pending entries with one explicit monophonic gesture state, then make cleanup observable before touching pitch selection.

## Residual Risks

Router tick granularity means dispatch is transport-referenced but not sample-offset accurate: events fire on the first router tick after a boundary. Strict sample-accurate output would require scheduling offsets in the audio/plugin callback and is not the smallest implementation. Tempo changes preserve beat calculation but can shift wall-clock timing; transport reset/seek needs explicit discontinuity cleanup. Full Species IV correctness remains impossible without future cantus knowledge.

```acceptance-report
{
  "criteriaSatisfied": [
    {
      "id": "criterion-1",
      "status": "satisfied",
      "evidence": "Concrete severity-ranked findings cite counterpoint_lane.rs, orchestrator.rs, world.rs, transport clock, and Tauri/WASM/plugin callers; residual risks and exact tests are included."
    }
  ],
  "changedFiles": [],
  "testsAddedOrUpdated": [],
  "commandsRun": [],
  "validationOutput": [
    "Read-only inspection completed; no preset research files were read and no project source files were edited."
  ],
  "residualRisks": [
    "Future live player notes cannot be predicted, so strict suspension dissonance and resolution consonance cannot be guaranteed.",
    "Router-loop dispatch is boundary-late rather than sample-offset accurate.",
    "Current disable/reset APIs can strand already-sounded lane notes unless cleanup ops are dispatched first."
  ],
  "noStagedFiles": true,
  "diffSummary": "No source diff; wrote the requested read-only architecture report artifact only.",
  "reviewFindings": [
    "blocker: crates/contrapunk-companion/src/counterpoint_lane.rs:324-344 - Species IV only delays onset and has no retained suspension/resolution.",
    "blocker: live-input architecture - strict Species IV interval correctness is impossible without future cantus lookahead or delaying the cantus.",
    "high: crates/contrapunk-companion/src/counterpoint_lane.rs:124-132 - disable clears ownership without emitting NoteOff.",
    "high: crates/contrapunk-companion/src/orchestrator.rs:84-106 - master disable prevents queued cleanup from ticking."
  ],
  "manualNotes": "Artifact-only write per task; implementation should be gated as a temporal suspension gesture unless lookahead is added."
}
```
