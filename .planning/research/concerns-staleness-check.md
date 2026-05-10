# CONCERNS.md staleness check — 2026-05-11

While starting Phase 1 of the v1.2.x roadmap I discovered that two of the highest-priority refactor tasks reference code that no longer exists. CONCERNS.md was snapshotted 2026-04-15; the audio architecture has been reworked since.

## Items resolved by intervening refactors

### "PolySynth::process_stereo allocates on every audio callback" — RESOLVED

CONCERNS.md (line 81-86) flagged a `vec![0.0_f32; frames]` allocation per audio callback in `src/audio_out/sine_synth.rs:192`.

**Current state:**
- `src/audio_out/` directory does not exist.
- The audio output path is now `src/synth/voice.rs::Synth::render(output: &mut [f32], channels: usize)`.
- The render path writes in-place to the caller-provided buffer. No `vec![]` allocations in the hot path. The only `vec![0.0f32; ...]` calls in `voice.rs` are inside `#[cfg(test)]` blocks (lines 351, 371, 375, 392, 408 — all test fixtures).

**Verification:** `grep -n "vec!" src/synth/voice.rs` — only test code.

**Roadmap impact:** Phase 1 task "Fix PolySynth::process_stereo allocation" is removed from v1.2.x. No work needed.

### "process_callback uses try_lock on Arc<Mutex<AudioState>>" — RESOLVED

CONCERNS.md (line 87-91) flagged a `try_lock()` in the audio callback at `src/audio_out/engine.rs:166`.

**Current state:**
- `src/audio_out/` does not exist.
- Repo-wide `grep -rn "try_lock\|AudioState"` returns zero matches in `src/` or `src-tauri/`.
- The current synth callback path takes an `mpsc::Receiver<SynthEvent>` (lock-free SPSC) and writes the output buffer in-place.

**Verification:** `grep -rn "try_lock\|AudioState" --include="*.rs" src/ src-tauri/` returns nothing.

**Roadmap impact:** Phase 1 task "Remove try_lock on Arc<Mutex<AudioState>>" is removed from v1.2.x. No work needed.

## Items NOT verified (probably also stale, not checked yet)

CONCERNS.md was written against the pre-companion-architecture codebase. Other entries in it may also be stale:

- The "Stuck MIDI Notes on Settings Change" bug pattern (lines 52-59) — predates the `pending_reharm_inputs` / `clear_active_for_reharm` mechanism that ships now.
- The Tauri JS-approximation beat clock concern (lines 99-103) — predates the companion-architecture `Transport`.
- The `wasm/src/lib.rs` per-frame `console.log` concern (line 94-97) — RESOLVED in this session (commit `34271df`, gated behind `cfg!(debug_assertions)`).

A future session should do a full pass through CONCERNS.md and either flag each item as RESOLVED, CURRENT, or REWORD-AGAINST-CURRENT-CODE. The pattern matters: tech-debt docs that age silently into half-fiction stop being useful as priority signals — and they pollute roadmap planning by making fixed-already work look like in-flight work.

## Recommendation

When the next `/gsd-map-codebase` runs, it should reset the entire CONCERNS.md snapshot rather than incrementally amending it. Cross-reference each entry against actual current code paths before keeping it.

This doc is intentionally NOT a fix for CONCERNS.md — it's a research artifact documenting one staleness discovery so a future session knows to do the full audit.

## Updated Phase 1 task list

Phase 1 of v1.2.x effectively becomes 5 tasks (was 7):

1. ✓ Move `inference.rs` (commit `f5e5f16`)
2. ✓ Gate WASM `console_log!` (commit `34271df`)
3. ✓ Close obsolete issues (#70, #2)
4. ◐ Extract router pure functions (commits `79aca33`, `d78ff39` — 2 of ~4 done, panic_replay deferred to its own commit)
5. ⊘ Group/namespace WASM exports — still TODO

Tasks 5 and 6 from the original plan (PolySynth alloc, try_lock removal) are RESOLVED-OBSOLETE per this audit.

Net result: Phase 1 is ~80% complete after one short session of work. Phase 2 (bugs + small features) can start sooner than the roadmap estimated.
