# Code Context

## Files Retrieved
1. `crates/contrapunk-companion/src/canon_lane.rs` (lines 328-398, 400-469, 823-1119, 1563-1725, 1996-2625) - Canon queue records, scheduling/release implementation, and all Canon HoldMode/lifecycle tests.
2. `crates/contrapunk-companion/src/lane.rs` (lines 24-116, 222-275) - HoldMode contract, default, input events, and runtime-reset contract.
3. `crates/contrapunk-companion/src/orchestrator.rs` (lines 42-104, 106-276) - global Hold owner, input/tick dispatch, and lane reset fanout.
4. `crates/contrapunk-transport/src/clock.rs` (lines 118-125, 209-215, 260-270) - total beat calculation and stopped-transport no-op behavior.
5. `src-tauri/src/commands/engine.rs` (lines 850-889; also 728-733) - native shutdown/panic sends AllNotesOff and resets Companion runtime.
6. `wasm/src/companion.rs` (lines 200-205) - WASM exposes reset, but the bridge itself does not make transport stop imply reset.
7. `src-tauri/src/commands/companion.rs` (lines 44-66, 137-166, 237-260) - global/lane/voice HoldMode callers and JSON pass-through.
8. `plugin/src/editor.rs` (lines 207-212) - plugin global HoldMode caller.
9. `crates/contrapunk-companion/src/counterpoint_lane.rs` (lines 348-410, 538-760) - sibling HoldMode behavior/tests (lane > global only), useful comparison.

## Key Code

`CanonLane::on_input(NoteOn)` computes one time coordinate per stack pitch:

```rust
fire_at = anchor + delay_beats + (now - anchor) * time_ratio
```

It records the complete emitted stack in `held[note]` and queues each pitch in `pending_on` (`canon_lane.rs:877-939`). No operation is emitted directly by input handling.

On player NoteOff, natural voice release is:

```rust
voice_on_fire = held.anchor + delay + (held.on_beat - held.anchor) * ratio;
voice_off_fire = voice_on_fire + (now - held.on_beat) * ratio;
effective = voice.hold_mode.or(lane.hold_mode).unwrap_or(global_hold);
release_at = Forever     => voice_off_fire
             Cancel      => now
             NearFuture  => min(voice_off_fire, now + tail)
             PhraseEnd   => min(voice_off_fire, phrase_end)
```

See `canon_lane.rs:944-1029`. Pending NoteOns are independently retained/cancelled at `canon_lane.rs:1032-1061`; cancelled, not-yet-fired entries have their newly-created orphan PendingOff removed at `canon_lane.rs:1062-1082`.

`tick()` compares both queues only with `transport.total_beats()` and drains NoteOns first, then NoteOffs (`canon_lane.rs:1094-1119`). Thus equal-time on/off produces ordered NoteOn then NoteOff in one tick.

Resolution is exactly **voice > lane > global**, at release time, for both pending NoteOn filtering and NoteOff timing (`canon_lane.rs:993-1001, 1041-1059`). Despite `orchestrator.rs:88-90` saying buffered emissions follow the mode in effect when scheduled, implementation intentionally reads the current global/overrides on NoteOff; the comment is internally contradictory.

## Architecture

- `Companion::on_input` forwards matching events to Canon because its filter is `All`; it does not inspect transport running state (`orchestrator.rs:213-276`, `canon_lane.rs:836-852`).
- `Companion::tick` likewise calls Decide lanes whenever Companion is enabled, regardless of transport state (`orchestrator.rs:106-146`).
- Running transport: audio advancement increases `total_beats`; queues mature deterministically at the first tick where `now >= fire_at`.
- Stopped transport: `Transport::advance` is a no-op, so `total_beats` freezes (`clock.rs:121-125, 212-215`). A tick can drain entries already due at the frozen beat, but any future PendingOn/PendingOff remains forever unless transport resumes or `reset_runtime` is called.
- Native router shutdown/panic compensates by broadcasting NoteOff/AllNotesOff and clearing queues (`engine.rs:850-876`). WASM merely exposes `reset_runtime`; correctness depends on the JS stop caller invoking both an audible all-notes-off path and this reset. Plugin/input-mode paths similarly need explicit lifecycle wiring.

## Likely Root Cause Candidates

1. **Highest-confidence missing-release reproducer: stopped clock + future release.** A zero-delay, `time_ratio=2`, Forever voice emits at beat 0; advance to beat 0.5, stop; player NoteOff schedules release at beat 1.0. Repeated ticks at frozen 0.5 never emit NoteOff. This is correct queue math but a real stuck note if the surface's stop path fails to send AllNotesOff/reset. Existing tests only keep transport running/reset it to target beats; none asserts stopped behavior.
2. **Lifecycle split across surfaces.** Core Canon has no running-state branch by design. Native exit is protected, while WASM reset is opt-in. Inspect each actual transport-stop caller before changing Canon; the minimal root fix is likely one stop-path reset/AllNotesOff, not altered musical scheduling.
3. **Same-note retrigger state collapse.** `held` is `HashMap<u8, HeldEntry>` (`canon_lane.rs:431-439`) and NoteOn uses `insert` (`canon_lane.rs:928-939`), so overlapping same-pitch NoteOns replace the first lifecycle. The later NoteOff removes the second entry; the first sounding generation has no independent release record. The current rapid-retrigger test (`canon_lane.rs:2550-2625`) is sequential on/off/on/off and does not cover overlapping note-on/note-on/off/off.
4. **Documentation mismatch only:** `HoldMode::Cancel` docs claim already-sounded voices use natural release (`lane.rs:39-44`), but Canon intentionally releases them at `now` and has a regression test proving that (`canon_lane.rs:2389-2441`). Avoid deriving behavior from the enum comment.

## Minimal Deterministic Regression

Add one unit test beside `hold_mode_cancel_releases_emitted_notes_at_now` (no production changes initially):

1. Enable Canon; one voice `delay=0`, `time_ratio=2`, HoldMode Forever.
2. At beat 0 send NoteOn and tick; assert NoteOn emitted.
3. Advance running transport to beat 0.5, then call `transport.stop()`.
4. Send NoteOff; assert PendingOff is at 1.0.
5. Call `transport.advance(...)` and `lane.tick()` repeatedly; assert no NoteOff and beat remains 0.5.
6. Then call the intended surface stop lifecycle (`AllNotesOff` plus `Companion::reset_runtime`) and assert no delayed emission can reappear after resume.

This test cleanly distinguishes: (a) core scheduling is behaving as implemented, and (b) a surface missing stop cleanup causes the audible missing release. If the reported symptom occurs without stop, use the overlapping same-pitch variant next; it is the smallest independent lifecycle bug.

Existing coverage already proves running behavior: delay boundaries and duration (`canon_lane.rs:1563-1725`), all four Hold filters and precedence (`canon_lane.rs:1996-2383`), immediate Cancel release and voice override (`canon_lane.rs:2389-2494`), orphan-off cleanup and sequential retrigger dedup (`canon_lane.rs:2498-2625`).

## Focused Test Commands

- `cargo test -p contrapunk-companion hold_mode --lib` — passed: 17 tests, 42 filtered out.
- `cargo test -p contrapunk-companion canon_lane::tests::note_off_preserves_per_voice_duration --lib`
- `cargo test -p contrapunk-companion canon_lane::tests::hold_mode_cancel_releases_emitted_notes_at_now --lib`
- Proposed stopped regression filter: `cargo test -p contrapunk-companion canon_lane::tests::stopped_transport_does_not_mature_future_release --lib`
- Proposed overlap filter: `cargo test -p contrapunk-companion canon_lane::tests::overlapping_same_pitch_releases_every_generation --lib`

## Start Here

Open `crates/contrapunk-companion/src/canon_lane.rs:944-1119` first: it contains the complete NoteOff resolution and tick drain path. Then trace the affected surface's transport-stop caller to verify it invokes audible AllNotesOff and `Companion::reset_runtime`.

```acceptance-report
{
  "criteriaSatisfied": [
    {
      "id": "criterion-1",
      "status": "satisfied",
      "evidence": "Read-only recon only; no project/source files modified. Findings written solely to the mandated artifact."
    },
    {
      "id": "criterion-2",
      "status": "satisfied",
      "evidence": "Exact line references, scheduling formulas, running/stopped distinction, precedence, existing coverage, two focused root-cause candidates, and deterministic regression steps are provided."
    }
  ],
  "changedFiles": [
    ".pi-subagents/artifacts/outputs/3723d3ea-2807-44fa-a94c-be69b5ec7d00/.planning/phases/10.2-arrangement-presets/recon/free-imitation-lifecycle.md"
  ],
  "testsAddedOrUpdated": [],
  "commandsRun": [
    {
      "command": "git status --short",
      "result": "passed",
      "summary": "Worktree already contains unrelated modifications and untracked files; this recon made no source edits."
    },
    {
      "command": "cargo test -p contrapunk-companion hold_mode --lib",
      "result": "passed",
      "summary": "17 passed, 0 failed, 42 filtered out."
    }
  ],
  "validationOutput": [
    "All focused existing Canon/Counterpoint HoldMode tests passed.",
    "Transport source confirms advance is a no-op while stopped and total_beats derives from frozen sample_pos."
  ],
  "residualRisks": [
    "Actual reported surface was not specified; native stop is protected, but WASM/plugin stop-call wiring must be traced before assigning the bug.",
    "No stopped-transport or overlapping same-pitch regression currently exists.",
    "Worktree was dirty before recon, so repository-wide no-change attribution requires reviewer awareness."
  ],
  "noStagedFiles": true,
  "diffSummary": "Added one read-only reconnaissance artifact; no project/source changes or tests added.",
  "reviewFindings": [
    "warning: canon_lane.rs:1094-1119 - future releases cannot mature while transport is stopped; surfaces must AllNotesOff/reset.",
    "warning: canon_lane.rs:928-939 - overlapping same-pitch NoteOn replaces the prior HeldEntry.",
    "warning: orchestrator.rs:88-90 - comment says scheduling-time Hold capture, implementation resolves at NoteOff.",
    "no source-code blocker established without identifying the affected surface stop path"
  ],
  "manualNotes": "Acceptance requires reviewer gate. The strongest first regression is stopped transport with time_ratio=2/Forever; it proves whether the issue is core math or missing surface lifecycle cleanup."
}
```
