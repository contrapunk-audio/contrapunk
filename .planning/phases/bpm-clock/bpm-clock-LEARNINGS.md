---
phase: bpm-clock
phase_name: "BPM-aware Performance + Pattern Programmer (PR #89)"
project: "contrapunk"
generated: "2026-04-30"
counts:
  decisions: 13
  lessons: 9
  patterns: 8
  surprises: 9
missing_artifacts:
  - "PLAN.md (no formal phase plan — this work was PR-driven, not phase-planned)"
  - "VERIFICATION.md (no formal phase verification — UAT happened against running Tauri dev)"
  - "UAT.md (no formal UAT artifact — interactive testing in dev session)"
sources:
  - "PR #89 commit messages (11 commits, 0bb6953..ddab892, squashed to 495529d on main)"
  - "PR #89 review history (8 brutal-code-critic + graphify reviews)"
  - "HANDOFF.json (frozen 2026-04-28)"
  - "GitHub issues #90 (held_harmonies stale-entry recovery), #91 (router-loop pure-function extraction)"
---

# Phase BPM-Clock Learnings

Feature work spanned ~3 days of active development (2026-04-27 through 2026-04-30) wrapped in 8 brutal-code-critic + graphify review iterations that converged from BLOCK to MERGE. The verdict trajectory: BLOCK → MERGE WITH FIXES (×2) → MERGE WITH FOLLOWUPS (×2) → MERGE (×3). 21 commits squashed to one on main; 11 of those were review-driven fixes.

## Decisions

### Bit-identical cell-index math in Rust + TS
The `PatternConfig::cell_index_at(total_beats: f64) -> usize` algorithm is implemented in both `src-tauri/src/state.rs` (router thread reads it for cell-boundary decisions) and `ui/src/lib/stores/pattern.svelte.ts` (UI uses it for the playing-cell highlight). Algorithm: `((total_beats % beats_per_loop) + beats_per_loop) % beats_per_loop * subdivision % cell_count`.

**Rationale:** Router thread needs deterministic per-iteration math without IPC; UI needs frame-rate cell highlighting from `transport.totalBeat` reactivity. Single-IPC-query-per-frame approach was rejected as too costly at sub-beat granularity.
**Source:** Phase 1-3 commit message; D3 review finding; commit `62eed31` (lockstep test pinning)

### Panel-pip visibility = master enable lifecycle
`PatternPanel.svelte` calls `pattern.setEnabled(true)` on `onMount` and `false` on `onDestroy`. Mount = enable, unmount = disable. The user has no separate enable toggle — the panel-pip in StatusBar is the single source of truth.

**Rationale:** Eliminates user confusion about "panel visible but feature off" or vice versa. Lifecycle stays in one place.
**Source:** Phase 1-3 commit message; reaffirmed in D1 fix (commit `867de49`)

### Step-sequencer NoteOff-before-NoteOn semantics on retrigger
For Live/Quantized input modes, every cell-on boundary fires `NoteOff` (for previously-held harmony) followed by `NoteOn` (re-attack), even when the same notes are sounding. Synths see explicit re-attack rather than NoteOn-on-already-held silently no-op.

**Rationale:** Step-sequencer feel — staccato re-attacks are the obvious behavior for a programmer-style cell grid. NoteOn-on-already-held is implementation-defined per MIDI spec; some synths retrigger envelopes, some no-op.
**Source:** Commit `00d1fa1` (step-sequencer NoteOff semantics)

### BeatGrid 3-variant API: `pip` / `cell` / `mini`
A single `BeatGrid.svelte` component exposes three visual variants. PatternPanel uses `cell` (large clickable), HistoryStrip header uses `pip` (small fixed) and `mini` (read-only preview).

**Rationale:** Three call-sites with shared cell-strip primitive structure (currentIndex highlight, divider predicates, click handling). Refute the over-abstraction concern — graphify community detection confirmed the abstraction earned its keep across two distinct callers.
**Source:** Commit `0bb6953` (BeatGrid extraction); review 1 (BeatGrid verdict)

### TransportBar pips NOT migrated to BeatGrid
TransportBar's beat indicators stay inline. Distinctive `class:downbeat` + `pixel-btn` styling tied to the StatusBar pip-row aesthetic.

**Rationale:** Refactor would be visual regression for marginal code-org gain. Three callers is the canonical "extract" threshold; TransportBar is a fourth that doesn't fit the BeatGrid contract.
**Source:** Commit `0bb6953` body; review 1 D4

### Quantized mode falls through to Live in router; hidden in UI
Backend `PatternInputMode::Quantized` arm in the pattern-tick match treats Quantized identically to Live (the same `(prev_was_on, cell_is_on)` tuple). Frontend `INPUT_MODE_OPTIONS` excludes `quantized`; `VALID_INPUT_MODES` set in restore() also excludes it, so persisted "quantized" silently migrates to "live" on hydrate.

**Rationale:** True input-onset quantization needs a router-side MIDI buffer that wasn't built in this PR. Shipping a UI control with zero observable effect was the original sin (review 1 H1: "UI lying about a control"); hide it until the buffer ships, document the intent in code so re-adding is one-line.
**Source:** Commit `e337152` (hide Quantized); review 1 H1; review 3 H1 verification

### HeldVoice tracker as parallel routing-aware structure
`held_harmonies: Arc<Mutex<HashMap<u8 input_note, Vec<HeldVoice>>>>` lives alongside `harmony_notes: Mutex<HashSet<u8>>` (UI/display) and `borrowed_notes: Mutex<HashSet<u8>>` (chord analysis). Each `HeldVoice` carries `note: u8`, `target: VoiceOutputTarget`, `channel: u8`, `velocity: u8`.

**Rationale:** Changing `harmony_notes` to a richer type would have rippled through chord display, the note-update event payload, and reharm-replay diff math. The parallel tracker preserves the existing UI surface while adding the routing precision pattern dispatch needs. Memory cost is trivial (max 8 voices × MAX_INPUTS).
**Source:** Commit `c0f2b01` (D2 fix); review 3 D2 verification

### dispatch_voice helper with exhaustive match (no `_` fallthrough)
Six dispatch sites (real-time NoteOn/NoteOff, pattern tick on/off, drain, orphan release, panic-replay attack) collapsed into one helper. Match arms are explicit: `(Synth, NoteOn)`, `(Synth, NoteOff)`, `(MidiPort{port}, _) if port >= num_ports`, `(MidiPort, NoteOn)`, `(MidiPort, NoteOff)`, `(Off, _)`. No `_` arm — adding a fourth `VoiceOutputTarget` variant produces a compile error at the helper site.

**Rationale:** Reviewer's "extract after third site appears" threshold was hit and exceeded (5 sites). Centralizes byte encoding, port-bounds check, and synth dispatch in one place. The exhaustive match is a small but real safety win — silent routing-to-Off via fallthrough is impossible.
**Source:** Commit `36093b9` (P2 extraction); review 6 helper-correctness verdict

### `broadcast_note_off` reuses dispatch_voice via thin loop
Originally a separate helper with raw byte construction. After review 8 R2, rewritten as a loop calling `dispatch_voice(VoiceOutputTarget::MidiPort{port}, 0, NoteOff{note, 0}, ...)` for each port plus one Synth call. Byte-for-byte identical output; encoding now lives in exactly one place.

**Rationale:** Adding a fourth dispatch destination requires updating one helper, not two. Side benefit: dispatch_voice's `debug_assert!(channel < 16)` / `debug_assert!(velocity < 128)` invariants now guard the broadcast path too.
**Source:** Commit `ddab892` (R2 fix); review 10 byte-level verification

### Move transport.play() out of set_pattern_enabled
`set_pattern_enabled` was originally a 9-line Tauri command that toggled `pattern_enabled` AND called `state.transport.play()` if not running. After D1 fix, it's 3 lines: store the bool. `PatternPanel.onMount` now starts the transport explicitly before calling `pattern.setEnabled(true)`.

**Rationale:** Louvain community detection (graphify) caught the coupling — `set_pattern_enabled` clustered with `TransportStore` as a 2-node island, separate from its 15 Tauri-command siblings. Setters that mutate unrelated subsystems become "wait, what side effects does this have?" graveyards over time. Frontend desync was also real: transport.play() never touched the Svelte TransportStore.running field.
**Source:** Commit `867de49`; review 2 D1 (graphify-detected divergence)

### `set_auto_key` raises panic_pending; pattern setters do NOT
Pre-existing miss surfaced by graphify edge analysis: 5 of 16 Tauri setters skipped `raise_panic`. The fix scope analysis classified each:
- `set_auto_key` mutates engine state (auto-key flip changes selected key on next note) → must raise panic. Fixed.
- `set_pattern_enabled` / `set_pattern_config` mutate UI flags only, not engine state → correctly skip panic.
- `set_routing_mode` / `set_detune` are atomic-only mutations → correctly skip panic.

**Rationale:** Symmetry with engine-mutating sibling setters (15+ that do raise panic). Distinguishing "mutates engine" from "mutates AppState bookkeeping" is the right axis.
**Source:** Commit `867de49` (D5 fix); review 2 D5 partial-confirmation analysis

### Lockstep test pinning between Rust and TS cell math
Rust unit tests in `state.rs::pattern_config_tests` pin `cell_index_at` and `cell_count` against a 13-vector reference table. TS dev-mode self-check at `pattern.svelte.ts` module load runs a 9-vector subset against a fresh `PatternStore` instance, logs `console.error` on drift.

**Rationale:** Both implementations work today (verified by trace). The risk is future drift — someone changes one side without the other. Tests catch drift in CI (Rust) and at module load in dev (TS). Production builds skip the dev-mode check (zero release-build cost).
**Source:** Commit `e337152` (D3 implementation); review 5 D3 verification

### Debounce pattern config IPC at 75ms trailing edge
Frontend pattern store's `persist()` writes localStorage synchronously and pushes `setPatternConfig` to backend via debounced `setTimeout(75ms)`. Cell-paint of 64 cells produces ~2 IPCs instead of 60-100.

**Rationale:** localStorage stays sync so a crash mid-paint loses at most one frame of state. IPC is the expensive part; collapsing them with trailing-edge debounce gives natural batching without leading-edge lag (router would freeze on first cell of a paint).
**Source:** Commit `e337152` (M4 fix); review 5 M4 verification

---

## Lessons

### Pattern code paths can completely bypass established invariants without line-by-line review catching it
The original PR shipped pattern attack/release dispatch that ignored `voice_outputs`, `last_port_map`, and `OutputRouter` entirely — sending only to the synth via `synth_tx`. Voices set to "Off" still played; voices routed to "MidiPort{N}" produced no audible re-trigger. Every cell flip / disable / panel close stuck notes on external MIDI gear. This was 4 CRITICAL findings in the first review and would have been the merge-killer.

**Context:** Line-by-line review reads "what does this code do" but not "what does this code DON'T do compared to its sibling code". Graphify community detection visualized the bypass — `PatternStore` (frontend god node, degree 18) had zero edges into the `Tauri Router Runtime` community where `target_for(i)` lives.
**Source:** Review 1 C1-C3 findings; review 2 D2 verification via graphify

### Architectural drift is detectable by Louvain community detection before it becomes an obvious bug
`set_pattern_enabled` calling `transport.play()` was hidden in plain sight — the doc comment even cops to it ("auto-starts the transport clock"). The bug was that the function name claimed "set a flag" but the implementation did "set a flag AND mutate an unrelated subsystem". Graphify's Louvain put `set_pattern_enabled` and `TransportStore` in their own 2-node community, separate from the 15-node `harmony.rs` setter cluster. The clustering algorithm detected coupling that was below the threshold of human "this feels wrong" intuition.

**Context:** Communities reflect *who-talks-to-whom* — when a node clusters with the wrong neighborhood, the abstraction is misshapen. Graphify is therefore a complement to line-by-line review, not a substitute.
**Source:** Review 2 D1 (architectural divergence); commit `867de49` D1 fix

### Pre-existing bugs surface when new code paths exercise dormant invariants
The `get_note_state` Tauri command at `engine.rs:82` had been reading from stale AppState mutexes for an unknown duration. Frontend never called it (UI uses `note-update` event stream). The bug existed but was invisible because of the unused code path. Surfaced during PR #89 review when the agent traced the get/set surface looking for adjacent issues.

**Context:** Code that's never called silently rots. The fix was deletion (~80 LOC removed), not repair — there's no point fixing dead code. Trace-based reviews surface these because graphify and code-aware agents follow edges that humans don't.
**Source:** Review 5 (pre-existing P1); commit `188d86b` (O1 deletion)

### Brutal review × graphify produces convergence; verdict trajectory is systematic
8 review iterations, BLOCK → MERGE WITH FIXES → MERGE WITH FIXES → MERGE WITH FOLLOWUPS → MERGE WITH FOLLOWUPS → MERGE → MERGE → MERGE. Each pass surfaced 2-6 numbered concerns; each round of fixes addressed all of them and added at most 1-3 new ones (which were always smaller in scope). Convergence was real, not an infinite loop.

**Context:** The pattern: round N raises concerns, round N+1 fixes them, round N+1 surfaces new concerns at smaller scope. Eventually round N+M surfaces zero new concerns — that's the terminal state. For PR #89 it took 8 rounds.
**Source:** All 8 review bodies; brutal-code-critic agent updated to require graphify Step 0 mid-cycle

### Velocity hardcoding for pattern-driven attacks must be intentional and documented
First fix passed `velocity: 100` to all pattern NoteOn dispatches (matching the first commit's hardcode). Reviewer flagged this as invisible musical degradation: "a user assigning a soft pad to the synth and a hard hit to a drum module will hear 100/100 for both on every pattern tick, with no way to tell why." Fix: capture the input event's velocity into HeldVoice, dispatch with that velocity.

**Context:** "Step-sequencer feel" is a defensible reason for uniform velocity, but if you want it, write it in a comment. Otherwise reviewers (and future-you) read the code as having a bug.
**Source:** Review 4 N2; commit `62eed31`

### FFI dual-source-of-truth needs explicit lockstep tests
`PatternConfig::cell_index_at` exists in both Rust (`state.rs`) and TS (`pattern.svelte.ts`). Both work today. There was no test pinning them to the same outputs. Reviewer flagged: "the next time someone changes the wrap behavior, fixes a sign issue, or adds a swing offset, they'll change one and forget the other."

**Context:** Cross-language clones are drift bait. The fix is cheap (~30 LOC of unit tests + a TS dev-mode self-check) and prevents the inevitable. Solved here; the same pattern applies anywhere data structures are mirrored across an FFI.
**Source:** Review 2 D3; commit `e337152` D3 fix

### Auto-start side effects in setters mask the user-visible coupling
"Open Pattern panel → transport starts" is the desired UX. Implementing it via `transport.play()` inside `set_pattern_enabled` made the coupling invisible to anyone reading the Tauri command surface. The fix moved the side effect to `PatternPanel.onMount`, where the relationship is explicit. Same UX, clearer code.

**Context:** When the UX requires "do X, then do Y in another subsystem," put the orchestration where it's visible (UI mount/destroy hooks, explicit command sequences). Hiding it in a setter named "set_X_enabled" buys nothing and costs traceability.
**Source:** Review 2 D1 architectural finding; commit `867de49`

### Some bugs are P2 standalone projects, not same-PR fixes
Stale-entry recovery in `held_harmonies` (dropped Note-Off via USB glitch / MPE rotation) needs design discussion: TTL false-releases sustains; `input_notes` cross-ref is updated by the same path so it's redundant; CC 123 needs device support. None of the cheap fixes are obviously correct. Same shape: router-loop pure-function extraction would unlock testing for F2/F4/F5 invariants but needs integration tests landed first as regression net.

**Context:** "I would rather fix everything here" is a sympathetic stance, but it conflates "outstanding" with "fixable now." Cramming design-discussion-tier work into a PR turns a green merge into a regression. The right move is filing follow-up issues with enough specificity that the next dev can pick up cold (#90, #91 do this).
**Source:** Review 7 scope assessment; issues #90, #91

### Convergence requires "everything tracked," not "everything fixed"
The user's preference was "no outstanding issues." The seventh-pass scope assessment classified outstanding items: O1 (in-scope, fixed), O3 (in-scope, fixed), O2 (out-of-scope → #90), O4 (out-of-scope → #91). Filing the issues with enough detail (LOC estimates, fix-shape evaluation, references to existing TODO markers) was substitutable for fixing them — the work is now tracked, just not done in this PR.

**Context:** Triage discipline: "in-scope, low risk, ~30min" is a clear yes; "design discussion + multi-PR refactor" is a clear no. Make the distinction explicit before the user pushes back, then propose the path.
**Source:** Review 7 final scope assessment; issues #90, #91 issue bodies

---

## Patterns

### Routing-aware parallel tracker
**Pattern:** When a flat data structure (e.g., `HashSet<u8>`) needs to grow a per-element attribute (routing target, channel, velocity) without rippling through every consumer, add a parallel `HashMap<key, Vec<RichValue>>` indexed by the originating event. Update both structures on the same code path; never let them diverge.

**When to use:** Cross-cutting attribute additions where the existing flat structure has many readers (UI display, chord detection, network emission) that would otherwise need updating. Memory cost is bounded by N events × M attributes; lock cost is one extra brief acquisition per event.

**Source:** `held_harmonies` design (commit `c0f2b01`); review 3 D2 verification

### Enum-driven exhaustive dispatch with no fallthrough
**Pattern:** When multiple call sites share a `match v.target { Synth | MidiPort | Off }` skeleton, extract a `dispatch(target, channel, event, ...)` helper. Use explicit match arms for every (target, event) combination — no `_` fallthrough. Adding a new target variant produces a compile error at the helper, not a silent route-to-default.

**When to use:** 3+ call sites with the same target-match structure. The exhaustive match is the safety win, not the LOC reduction.

**Source:** `dispatch_voice` extraction (commit `36093b9`); review 6 verdict

### Cross-language lockstep test
**Pattern:** When the same algorithm must run in two languages (e.g., Rust router thread + TS UI for cell-index math), maintain a single reference table of (input → expected_output) tuples. Implement a unit test in each language that runs the table through its implementation. Also embed a dev-mode self-check at module load on the higher-level language so drift is visible immediately, not only in CI.

**When to use:** Any FFI'd algorithm that must produce identical outputs on both sides — pitch detection, audio buffer math, MIDI byte construction, time-signature math.

**Source:** `cell_index_at_matches_reference_table` (commit `e337152`); review 5 D3

### Trailing-edge debounce with synchronous side effect
**Pattern:** When a UI mutation has both a fast persistence path (localStorage / in-memory) and a slow propagation path (IPC / network), keep the fast one synchronous and trailing-edge-debounce the slow one. Crash recovery is preserved (fast path captures intent immediately); user-perceived lag is bounded by debounce window; downstream load is collapsed.

**When to use:** Any "user is dragging / typing / clicking fast" UI pattern where each event has a meaningful per-event side effect plus an aggregate side effect.

**Source:** `pattern.svelte.ts` debounced `persist()` (commit `e337152`); review 5 M4

### Phase-alignment seed (lifecycle-aware first-iter handling)
**Pattern:** When a stateful loop tracks "previous state for transition detection," the first iteration after enable has no real previous state — yet the transition logic will fire as if there were one. Fix: detect the "uninitialized" case (often `Option::None` / `is_none()`) and seed the tracker without firing transition logic. Trade-off: cold-start case (genuinely first execution) loses one transition.

**When to use:** Any router/loop body where transitions matter musically, audio glitches matter, or the difference between "fire on enable" and "fire on first real boundary" is user-perceptible.

**Source:** M3 fix in `engine.rs` pattern tick (commit `c0f2b01`); review 3 M3

### Mutex-with-explicit-drop before slow I/O
**Pattern:** When a critical section must (a) read shared state, (b) compute, (c) commit a result, AND (d) trigger downstream I/O (network send, MIDI port write), do (a)-(c) under a single lock guard, then drop the guard explicitly before (d). Pattern: `let result = { let mut hh = state.lock(); ...; hh.insert(...); computed_value };  do_io(result);`

**When to use:** Hot-path critical sections where the I/O latency could couple to USB / network / disk and back-pressure other lock acquirers.

**Source:** Orphan-release pattern in `handle_note_on` (commit `36093b9`); review 5 N4

### Brutal-critic + graphify dual-pass review
**Pattern:** For non-trivial review targets, run graphify on the diff scope first (community detection + god-node analysis), then have brutal-code-critic do the line-by-line trace using graphify output as input. Graphify catches architectural drift (cross-subsystem coupling, isolated communities, god nodes growing); brutal-critic catches code-level bugs (missed match arms, wrong byte encodings, race conditions). Neither alone catches both classes.

**When to use:** Any PR > 200 LOC, any cross-cutting concern (routing, dispatch, state machines), any feature that touches multiple subsystems (transport + harmony + UI in PR #89's case). Ineffective for one-line bug fixes.

**Source:** brutal-code-critic agent definition updated mid-cycle (commit to `~/.claude/agents/brutal-code-critic.md`); 8 review iterations on PR #89

### Iterative review-fix-review with verdict trajectory tracking
**Pattern:** Run review-fix cycles until the verdict converges to MERGE without qualifier. Track the verdict explicitly (BLOCK, MERGE WITH FIXES, MERGE WITH FOLLOWUPS, MERGE). Each fix round addresses ALL flagged concerns; new concerns surfaced by the next round must be smaller in scope than the prior round. If they're not, you've hit a design-level issue and should stop fixing and start designing.

**When to use:** Any feature work where correctness matters more than speed, or where a single review pass is unlikely to catch all the issues in one go (cross-subsystem features, real-time / concurrency, security-sensitive code).

**Source:** PR #89 review trajectory (8 passes, BLOCK→MERGE)

---

## Surprises

### Pattern code path was a *complete* bypass of per-voice routing, not a partial miss
First review found 4 CRITICAL items; all four were the same root cause: pattern attack/release dispatched to `synth_tx` and ONLY `synth_tx`, never invoking `output_router.send_to_port`. Voices set to "Off" still played; voices routed to "MidiPort{1}" produced no audible re-trigger. Every cell flip / disable / panel close stuck notes on external MIDI gear. The fix wasn't "add a missed branch" — it was structurally rebuilding how pattern dispatch knows about routing.

**Impact:** Without graphify community detection visualizing the bypass, a typical line-by-line review would have caught maybe one of the four sites (the most obvious cell-on attack). The other three (cell-off, disable-drain, retrigger) would have shipped. Stuck notes on external MIDI gear is the kind of bug that erodes trust in the whole product on first use.
**Source:** Review 1 C1-C4; review 2 D2 verification via graphify

### Louvain detected the auto-start coupling without being told to look for it
`set_pattern_enabled` clustered with `TransportStore` as a 2-node community (C8 in graphify v2), separate from the 15-node `harmony.rs` setter cluster. The algorithm caught the coupling automatically — community detection works without any prompt about "find me cross-subsystem leakage."

**Impact:** Same architectural finding emerged independently from line-by-line review (review 2 found D1 by reading the function body). Both methods converged on the same call. That's signal — when graph structure and code-level analysis agree, the finding is high-confidence.
**Source:** Review 2 (graphify-augmented audit); commit `867de49` D1 fix

### `get_note_state` was dead code with zero callers
Tauri command `get_note_state` was registered, exposed via the adapter interface, and implemented in TauriAdapter. Frontend never called it. UI populates note state exclusively via `adapter.onNoteUpdate(...)` (the `note-update` event stream from the router thread). The polling pathway existed and was always returning stale data — but nothing ever observed the staleness because nothing called it.

**Impact:** Discovered during fifth-pass review trace. Fix was deletion (~80 LOC removed across Rust + TS), not repair. Saved future maintenance cost on dead code that would otherwise have rotted further.
**Source:** Review 5 P1 (pre-existing); commit `188d86b` O1

### `& 0x0F` mask survived 4 reviews as defensive code
The byte construction `[0x80 | (channel & 0x0F), note, 0]` masks channel to its low 4 bits. The mask is unreachable: every caller passes `channel.index()` (returns 0-15 by definition of wmidi::Channel) or a literal 0. The mask is belt-and-suspenders against a future caller violating the invariant; it doesn't fix any current bug.

**Impact:** Survived multiple reviews because reviewers (and the implementing agent) defaulted to "leave defensive code in." The reviewer eventually flagged it explicitly in pass 6 as "the mask is now load-bearing if anyone passes a raw byte from an untrusted source — currently nobody does." Acceptable in the end, but noteworthy as an example of belt-and-suspenders calcifying.
**Source:** Review 6 (commit `36093b9`)

### `broadcast_note_off` refactor silently fixed a pre-existing bug
The panic-replay block's `to_release` overspray loop previously sent raw MIDI bytes to every external port BUT did NOT push `SynthEvent::NoteOff` to `synth_tx`. So after a parameter change (key, mode, scale), harmonies dropped from the new diff would keep ringing on the internal Rust synth even though external MIDI was correctly released. Refactoring to use `broadcast_note_off` (which always hits both synth and external ports) fixed it as a side-effect. The bug was never observed because the synth's voice allocator gracefully handles unbalanced NoteOn/NoteOff via voice-stealing.

**Impact:** A latent bug fixed accidentally during a cleanup commit. Not in the original commit message; eighth-pass reviewer flagged it as worth surfacing in the PR description.
**Source:** Review 8 O3 verification (commit `188d86b`)

### Each review cycle surfaced 0-3 new minor concerns; convergence was real
The convergence pattern: round 1 raised 15+ concerns; round 2 raised D1-D5 + N1-N6 framework; round 3 raised 6 N-concerns; round 4 raised P1-P3; round 5 raised Q1-Q4; round 6 raised R1-R4 (most cosmetic); round 7 was a scope analysis that produced no NEW concerns; round 8 raised S1-S2 (both pre-existing or constant-folding); round 9 raised zero new concerns. The trajectory was monotone-decreasing in scope, never an infinite-loop pattern.

**Impact:** Validates "iterate until no new concerns" as a stopping rule. The reviewer's verdict went from BLOCK to MERGE in 8 passes; the 9th pass was redundant verification.
**Source:** All 8 review bodies; final summary in main message thread

### `cell_index_at` math was bit-identical in Rust and TS without any test
The original PR shipped `PatternConfig::cell_index_at` in `state.rs` and `cellIndexAt` method in `pattern.svelte.ts`. Algorithm was identical; both worked. There was no test pinning them. Survived review 1 (the brutal-code-critic actually noted the duplication as M2 medium concern but didn't fail the review on it). The drift-detection test was added in commit `e337152` as part of D3 fix.

**Impact:** A real-world example of "untested code that happens to be correct." The cost of NOT having the test was zero (the algorithm hadn't drifted), but the cost of writing the test was 30 LOC and the value of the test grows over time as code evolves. Cheap insurance.
**Source:** Review 2 D3; commit `e337152` D3

### Debounce window of 75ms cleanly collapsed N→1 IPCs without perceptible lag
Initial implementation pushed pattern config on every cell-toggle. A 64-cell drag-paint produced ~60-100 IPCs in ~1 second. The 75ms trailing-edge debounce collapsed this to 1 IPC at end of paint. User-perceptible lag from "click stops" to "router sees new config" is bounded at 75ms — well under the human threshold for perceiving "did the cell turn on?"

**Impact:** A textbook debounce win. The number 75 wasn't arrived at by measurement; it was the agent's best guess, validated by mental simulation. Could be tuned higher (less IPC pressure) or lower (faster response) but 75ms hit the right point.
**Source:** Commit `e337152` (M4); review 5 M4 verification

### Graphify community count fluctuated ±1-2 between iterations on stable code
Across 6 graphify runs (v1-v6), community count varied: 11, 11, 11, 14, 15, 12, 16, 15, 12. Stable code, fluctuating Louvain output. The structural signals (god-node degrees, specific community membership shifts) were stable; the raw community count was noise.

**Impact:** Reading raw community count as "topology change" is a mistake. Read god-node degree changes (HarmonyEngine 50, PatternStore 18, raise_panic 12→13) and specific community membership (set_pattern_enabled moving from C8 to C2 after D1 fix) — those are the real signals.
**Source:** All 8 graphify analyses (graphify-out/pr-89-fixes-*-analysis/)
