## Review

### Correct
- `CounterpointLane` already uses transport-total-beat scheduling and sorted pending attacks, providing the right clock basis for beat-grid preparation/resolution (`crates/contrapunk-companion/src/counterpoint_lane.rs:173-181,213-217,413-424`).
- Reset callers generally pair `Companion::reset_runtime()` with system-wide note draining or CC123, so transport stop/panic can safely discard lane queues (`src-tauri/src/commands/engine.rs:162-180,728-759`).
- Preset 08 correctly declares transport as required, matching fourth-species timing needs (`ui/src/lib/arrangement/catalog.ts:264`).

### Blockers

- **Blocker — existing lifecycle cannot safely own a P-S-R chain:** `HeldCpEntry.emitted_notes` contains pitches as soon as they are scheduled, not when their NoteOn fires (`counterpoint_lane.rs:258-348`). On release, retained future attacks receive NoteOffs at `now` or `now + tail_beats` (`counterpoint_lane.rs:370-405`). An Off can therefore precede its On, leaving the later attack stuck. `Forever` is especially broken: it preserves all future attacks but schedules their Offs at `now`. A fourth-species implementation must not reuse this bookkeeping.

- **Blocker — disable can strand sounding notes:** `set_enabled(false)` clears pending, held, and state without emitting NoteOff (`counterpoint_lane.rs:124-132`). The Tauri configuration command invokes this directly without the panic/drain path (`src-tauri/src/commands/companion.rs:203-224`). `tick()` then refuses to run while disabled (`counterpoint_lane.rs:413-415`), so a deferred flush cannot currently occur.

- **Blocker — current Species 4 is only delayed attack:** it schedules one attack at `now + 0.5`; there is no preparation held through a strong beat, no resolution, and no tied ownership (`counterpoint_lane.rs:323-337`).

- **Blocker — source-note ownership is incompatible with legato:** pending events are owned by the note that originally prepared them, while `held` is keyed only by MIDI pitch (`counterpoint_lane.rs:48-50,93-95,341-348`). In normal legato ordering—new NoteOn, then old NoteOff—the old release can cancel the pitch that should become the new note’s suspension. Same-pitch retriggers also overwrite the previous entry.

- **Blocker to preset delivery:** preset 08 is currently only a `DRAFT_SPECS` metadata entry, not an executable `ArrangementPresetV2` configuration (`ui/src/lib/arrangement/catalog.ts:256-264`). A Lane fix alone will not make selecting preset 08 configure Species 4.

- **Review-input blocker:** the requested root `plan.md` does not exist. `progress.md` exists but contains unrelated AU/REAPER debugging notes, so no proposed plan assumptions could be verified.

## Smallest safe design

Keep the patch primarily inside `CounterpointLane`, but give Species 4 a dedicated **single-voice cycle state** rather than extending `pending_on`/`held`:

1. Add one `Species4Cycle` containing:
   - generation/token and current owner `(note, channel)`;
   - prepared/current pitch and whether it has actually sounded;
   - preparation, expected-strong, and resolution beats;
   - resolution pitch when the next cantus becomes known;
   - optional release deadline.

2. Quantize from absolute transport beats:
   - first strong-beat input computes a consonant pitch;
   - attack it on the next half-beat as **P**;
   - retain it with no reattack across the following integer beat as **S**;
   - only when the next live melody NoteOn arrives near/after that strong boundary can the Lane compute **R**, scheduled for the following half-beat.

3. At resolution, emit one ordered atomic transition:
   - `NoteOff(prepared)`;
   - then `NoteOn(resolution)`;
   - update the cycle’s active pitch before processing any subsequent release.
   
   The resolved pitch becomes the next preparation, avoiding a second attack.

4. Respect live-input causality:
   - never predict the next cantus;
   - when it arrives, accept a suspension only if the held pitch was consonant against the previous cantus, is dissonant against the new cantus, and a descending diatonic step resolves consonantly;
   - otherwise release/revoice on the weak beat and do not claim a true suspension for that cycle.

5. Transfer ownership on the new NoteOn **before** the old legato NoteOff can arrive. Match releases by generation plus owner, not pitch alone. An old owner’s NoteOff must not terminate the transferred cycle.

6. Apply HoldMode to the whole cycle ledger:
   - `Cancel`: cancel future transitions and immediately Off the active pitch.
   - `NearFuture`: allow transitions strictly before `now + tail_beats`, then Off whichever pitch is active.
   - `PhraseEnd`: same, using the current bar boundary.
   - `Forever`: allow the armed resolution to finish, then release on the next half-beat boundary. It must not retain attacks without a later Off.

7. Safe disable:
   - disabling cancels unfired events but retains an immediate flush record for every actually active pitch;
   - `tick()` must drain that flush before checking `enabled`;
   - configuration/species changes use the same flush path.
   - `reset_runtime()` may continue relying on the documented outer CC123/AllNotesOff contract, but must erase all future events so nothing reappears.

This avoids modifying the core harmony engine or predicting future melody. The unavoidable limitation is that live fourth species is opportunistic: arbitrary future melody cannot guarantee that a previously prepared consonance becomes a valid suspension with a stepwise consonant resolution.

### Required checks
- Exact grid: P at `n+0.5`, no reattack at `n+1`, ordered Off(P)/On(R) at `n+1.5`.
- No resolution before the next cantus NoteOn.
- Invalid or late next cantus falls back without claiming a suspension.
- Legato `On(B) → Off(A)` transfers ownership correctly.
- Same-pitch retrigger does not overwrite an earlier generation.
- Every HoldMode balances attacks and releases, including release before P.
- Disable/species change flushes sounded notes and cancels unfired notes.
- Reset plus outer AllNotesOff produces no delayed reappearance.
- Duplicate MIDI pitches are balanced by generation, not pitch-set deduplication.