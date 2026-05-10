---
name: perf-check
description: Run Contrapunk performance checks and surface regressions. Use when the user asks "is performance still good?", "did this regress latency?", "check bundle size", "any audio glitches?", or before tagging a release. Also called automatically by the contrapunk-harmony-fixer agent before commit. Covers harmony-engine throughput, audio-callback allocation, WASM/UI bundle size, and known hot-path concerns from CONCERNS.md.
---

# perf-check

A repeatable performance sanity check for Contrapunk. Designed to be run after non-trivial changes to harmony, audio, DSP, or WASM/UI code — and before every release.

This skill does **not** invent new benchmarks. It runs what already exists, plus a small set of cheap structural checks that catch the regression patterns this codebase has historically hit.

## Where Contrapunk's perf sensitivity lives

| Surface | What matters | Why |
|---|---|---|
| `cpal` audio callback (`src/audio_out/`) | No heap allocations, no mutex contention, no blocking | Hard real-time — a glitch is audible |
| Harmony engine (`crates/contrapunk-harmony/`) | `harmonize_note_on` p99 latency, `process_with_beat` throughput | Sits in the MIDI router hot path; called per note |
| Guitar DSP pipeline (`src/audio/guitar_input.rs`, `wasm/src/lib.rs`) | `process_block` time per audio buffer | Called every ~23ms at 44.1 kHz / 1024 samples |
| WASM bundle (`ui/src/lib/wasm-pkg/`) | Total `.wasm` size | Affects first-load time on `app.contrapunk.com` |
| UI bundle (`ui/build/`) | Total JS + CSS size | Same |

The known footguns are documented in `.planning/codebase/CONCERNS.md` under "Performance Bottlenecks" — read it before assuming a regression is new.

## Steps

### 1. Harmony engine — fast unit tests + spot benchmark

```bash
cargo test -p contrapunk-harmony --lib --release
```

All 249 lib tests should pass in `--release` mode in well under 5 seconds total. If any test takes >100ms, look at what it's doing — that's a smell.

For micro-benchmarking, the project does not yet have `criterion` set up. If the user wants quantitative numbers, propose adding it (`crates/contrapunk-harmony/benches/harmonize.rs`) as a follow-up, **don't add it inside this skill run**.

### 2. Audio callback — allocation guard

The audio callback at `src/audio_out/engine.rs::process_callback` must not heap-allocate. `CONCERNS.md` flags `PolySynth::process_stereo` at `src/audio_out/sine_synth.rs:192` as a known offender (`let mut mono = vec![0.0_f32; frames];`). Check that no new `vec![` / `Vec::new()` / `Box::new` / `String::new` calls have been introduced in any function transitively reachable from `process_callback`.

```bash
rg -n 'vec!|Vec::new|Box::new|String::new|String::from|format!|to_string' src/audio_out/ | grep -v test
```

Compare against the previous commit:

```bash
git diff HEAD~1 -- src/audio_out/ | grep -E '^\+.*\b(vec!|Vec::new|Box::new|String::new|format!)\b'
```

Surface any new lines as a P0 — they will glitch audio under load.

### 3. Guitar DSP — block throughput sanity

```bash
cargo test -p contrapunk --lib audio::guitar_input --release 2>&1 | tail -10
```

If `pitch_accuracy_benchmark.rs` (`tests/pitch_accuracy_benchmark.rs`) is runnable:

```bash
cargo test --test pitch_accuracy_benchmark --release -- --nocapture 2>&1 | tail -30
```

Look for printed timings; flag any block that takes more than 1ms (one block is ~23ms of audio, so anything above 1ms eats >4% of the audio budget on a single core).

### 4. WASM bundle size

```bash
cd ui && npm run build:wasm 2>&1 | tail -5
ls -lh src/lib/wasm-pkg/*.wasm 2>/dev/null
```

Compare against the previous commit:

```bash
git show HEAD~1:ui/src/lib/wasm-pkg/contrapunk_wasm_bg.wasm 2>/dev/null | wc -c
ls -l ui/src/lib/wasm-pkg/contrapunk_wasm_bg.wasm 2>/dev/null
```

Flag a >10% size growth as a regression worth surfacing. Acceptable in absolute terms is roughly <1.5 MB compressed.

### 5. UI bundle size

```bash
cd ui && npm run build 2>&1 | tail -20
du -sh build/_app/immutable/chunks/ 2>/dev/null
```

If Vite's per-chunk size output mentions any chunk >500 KB uncompressed, surface it.

### 6. Report

Output a structured summary (don't ramble):

```
## perf-check summary (commit: <SHA>)

| Check | Result | Notes |
|---|---|---|
| Harmony lib tests | <pass/fail, time> | <slow tests if any> |
| Audio-callback allocation diff | <clean / N new alloc sites> | <file:line list> |
| Guitar DSP block timing | <ok / regress> | <block ms> |
| WASM bundle | <size>, <delta from HEAD~1> | |
| UI bundle | <size>, biggest chunk | |

### Regressions to address
- ...

### Acceptable / no action
- ...
```

## What this skill does NOT do

- Doesn't add new benches — that's a one-time setup task with its own commit.
- Doesn't profile (perf, instruments, dtrace) — that's an investigation, not a sanity check.
- Doesn't compare against historical baselines beyond `HEAD~1` — there's no perf history database in this repo yet.
- Doesn't fail loudly. Output is informational. The agent or user decides whether to block on it.

If you find a real regression, write a `.planning/todos/pending/perf-<slug>.md` capturing it with file:line citations and the measured delta. Don't try to fix it as part of this skill run.
