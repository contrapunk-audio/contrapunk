---
name: tdd-workflow
description: Enforce a TDD-driven workflow for Contrapunk feature/bug work. Use whenever a new feature or non-trivial bug fix is being implemented. Specifies the red-green-refactor loop, where tests live, what counts as "tested first", and how to plug into the project's existing test infrastructure. Also called automatically by the contrapunk-harmony-fixer agent.
---

# tdd-workflow

Test-first means **the test exists and fails before the production code exists**. Not "test added in the same PR". Not "test added in the same commit". The test fails first; the test passes second; refactor last.

This skill is the procedure. It's invoked by humans before starting any non-trivial change and by some agents (like `contrapunk-harmony-fixer`) automatically.

## The loop

### 1. Name the behavior

Write one sentence describing the **observable behavior** you're going to introduce. Not "implement function X" — "When the user picks Species 2 with no transport, the harmony output diverges from Species 1 within 4 notes."

If you can't name the behavior in one sentence, you don't know what you're building yet. Stop and clarify.

### 2. Write the failing test FIRST

Pick the right test surface:

| What you're changing | Test goes in | Pattern |
|---|---|---|
| `crates/contrapunk-harmony/` | Same file, `mod tests` block at bottom | See `engine.rs:2148+` for counterpoint test patterns |
| `crates/contrapunk-audio/` DSP | Inline `mod tests` | Block-throughput tests common |
| `src-tauri/src/commands/` | Inline `mod tests` if possible; integration test in `src-tauri/tests/` if it needs IPC | |
| `wasm/src/` | `wasm-bindgen-test` in same module, gated `#[cfg(target_arch = "wasm32")]` | See `wasm/src/lib.rs:879+` |
| `ui/src/lib/` Svelte / TS | `*.test.ts` next to the file | Vitest |
| Cross-surface integration | `tests/<feature>.rs` for Rust, `ui/playwright/<feature>.spec.ts` for UI | |

Run it:

```bash
# Rust:
cargo test -p <crate> --lib test_name_of_your_new_test

# UI:
npm --prefix ui test -- <pattern>
```

It must **fail** with a clear assertion message. "test not found" doesn't count — that means you forgot to compile-link it.

### 3. Make it pass — minimum code

Write the **smallest** production change that turns the failing test green. No "while I'm here" cleanup. No bonus features. No premature abstractions.

Re-run the test. It must pass. Run the full crate's tests — they must all still pass.

```bash
cargo test -p <crate> --lib
```

### 4. Refactor

Now and only now, look at the production code with a critical eye. Improve names, extract helpers, deduplicate. Run the test again after each refactor step — they must stay green.

If you're not sure what to refactor, invoke the `code-entropy` skill on the file you just touched. It'll point at structural entropy you may have introduced.

### 5. Commit

Conventional message. One logical change per commit. The test and the production code go in the **same** commit — they're a unit. Don't split them.

## Anti-patterns

- **"I'll add the test later."** No. Now. Otherwise it doesn't happen.
- **"The test would be hard to write."** Then the design is wrong. Test-pain is design-pain leaking out.
- **Testing implementation details.** Test the observable behavior, not the internal function call graph. If you find yourself mocking 10 things to test 1 thing, the unit you're testing is too coupled.
- **"This change is too small for a test."** Sometimes true (e.g. version bumps). Almost never true for behavior changes. If you're not adding a test, write down WHY in the commit message so the next person can audit the call.
- **Snapshot tests for everything.** Snapshots are fine for stable serialization formats; brittle for behaviors that evolve.
- **Flaky tests.** A test that sometimes fails is worse than no test — it teaches the team to ignore failures. Fix the flake or delete the test the same day you discover it.

## Project-specific patterns

### Rust crate test conventions

- Inline `mod tests { use super::*; ... }` at the bottom of the file.
- `#[test] fn test_<verb>_<condition>()` naming. Reads as English.
- Use `assert!(...)` and `assert_eq!(...)` with descriptive messages: `assert_eq!(got, want, "Species 2 should diverge — config: ...")`.
- For property-style tests, see `crates/contrapunk-harmony/src/voice_leading/voicer.rs::tests::test_determinism_100_times` — runs the same code 100x to catch nondeterminism.

### WASM tests

Only run on `target_arch = "wasm32"`. The `wasm/src/lib.rs:879-947` block shows the pattern: most tests are plain `#[test]` and run on host; the ones that call `JsValue::from_str` need `#[cfg(target_arch = "wasm32")]` gating.

To actually run WASM tests: `wasm-pack test --node wasm` or via `cargo test -p contrapunk-wasm`.

### UI tests (Vitest)

Co-locate next to the file: `Component.svelte` → `Component.test.ts`. Use `@testing-library/svelte` for component tests.

### Manual UAT

For UI-flow validation that's hard to automate, write a UAT checklist in the PR description and reference it. The user runs through it before merging. See `.planning/phases/*/UAT-*.md` for prior examples.

## When NOT to TDD

- Tiny config/docs/version bumps (state in commit msg: "no test, X is config").
- Throwaway spikes (`/gsd-spike`). The whole point is exploratory — but if the spike becomes real code, port it through TDD before promoting.
- Surgery that's literally untestable (e.g. an editor reformatting; a build script tweak). Same caveat: justify in commit msg.

## When TDD is hardest, lean harder

- **Real-time audio**: write a test that pushes a known signal through the DSP and asserts on the output. Use the `test_signals` helpers in `contrapunk-audio` if they cover your case.
- **MIDI routing**: the router is hard to test end-to-end. Use the pure-function extraction pattern (#91 in the open issue list) — split the routing decision from the IO so the decision is unit-testable.
- **UI**: focus on the store / adapter layer. Components are render-heavy and brittle to test; stores are pure state.

## Report back format (for agents using this skill)

When an agent invokes this skill, the agent should report:

```
TDD loop completed:
- Behavior: <one-sentence>
- Failing test: <path:line>, failure msg: "<msg>"
- Production change: <path:line> (N lines added)
- Now passing: yes (cargo test -p X --lib: M passed, 0 failed)
- Refactor done: <yes/no/skipped — reason>
- Commit: <SHA>
```
