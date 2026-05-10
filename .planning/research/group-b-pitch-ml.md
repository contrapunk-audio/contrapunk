# Research: Group B — Pitch Detection / ML / Guitar Pipeline

**Issue(s):** #82, #79, #29, #28, #27
**Date:** 2026-05-11
**Researcher:** issue-researcher
**Verdict (group-level):** mixed — see per-issue verdicts.

| # | Verdict | T-shirt |
|---|---|---|
| #82 | in-repo core (replaces existing) | L |
| #79 | in-repo bug fix — **skip if #82 lands same milestone** | S |
| #29 | research-only — no code | XS |
| #28 | mixed — runtime in-repo, model file external | L |
| #27 | in-repo tests | M |

Group framing: pitch / DSP / ML / guitar capture all live in `crates/contrapunk-audio/`. The pipeline rewrite (#82) is the spine — everything else either fixes a bug in the legacy pipeline (#79), builds tests against it (#27), or layers polyphonic ML on top (#28, #29). #82 has already shipped its Phase 0 skeleton at `crates/contrapunk-audio/src/guitar_pipeline/{mod.rs,stages.rs,new/}` — five traits (OnsetDetector, PitchDetector, OctaveCorrector, StateMachine, EventEmitter) are defined; `new/` is empty.

---

## #82 — Rewrite guitar→MIDI pipeline (monophonic, native + WASM lockstep)

### Problem
Legacy `crates/contrapunk-audio/src/guitar_input.rs` is 4,030 lines accumulated since v1.0.0, with no fixture-based regression coverage, audible pitch wobble (#79), known octave-error leaks on fresh onsets, broken string ID without calibration, and a parallel TS port (`ui/src/lib/audio/guitarInputDsp.ts`, 1,313 lines) that has drifted from the Rust source. User wants a fresh design built **in parallel** with stage-by-stage A/B testing against legacy, and one Rust source compiled to both native and WASM.

### Touchpoints
- `crates/contrapunk-audio/src/guitar_input.rs:27-100` — `GuitarInputConfig` (extend with stage selector enums; rename `bends_enabled` → split `initial_bend_enabled` / `continuous_bend_enabled` per #79 H1)
- `crates/contrapunk-audio/src/guitar_input.rs:741-797` — the unconditional initial-bend emission (#79's root cause)
- `crates/contrapunk-audio/src/guitar_pipeline/` — Phase 0 skeleton already in place; `stages.rs` defines five traits; `new/mod.rs` is empty
- `crates/contrapunk-audio/src/lib.rs:20-26` — module wiring
- `wasm/src/lib.rs:693-947` — `WasmGuitarInput` re-exports `contrapunk::audio::guitar_input::*`. Lockstep means *delete* the TS port and route the worklet straight into the same Rust source compiled to wasm.
- `ui/src/lib/audio/guitarInputDsp.ts` — **delete in Phase 11** (1,313-line parallel impl is the lockstep failure mode)
- `ui/src/lib/audio/guitarCapture.ts` — already uses AudioWorklet → Float32Array → WASM; this stays.
- `.planning/codebase/CONCERNS.md:93-104` — per-frame `console.log` in `guitarCapture.ts:153-155,169,195,199`; same module is logging from inside `wasm/src/lib.rs:716-770` (`[wasm-guitar] Created`, every 20 frames `[analyze]`, every event batch). All this needs a `DEBUG` gate before #82 ships.
- `.planning/codebase/CONCERNS.md:144-147` — `ScriptProcessorNode` deprecation. Worklet path is already implemented at `guitarCapture.ts:244`; #82 should drop the SP fallback or feature-flag it.

### Architecture verdict
**In-repo core, replaces the existing implementation.** Stays inside `crates/contrapunk-audio/`. The pipeline is the heart of the guitar input feature — moving it to a separate crate or sub-project trades zero benefit (it has to ship on all surfaces) for a release boundary. The new pipeline lives at `guitar_pipeline/new/` and the legacy code at `guitar_input.rs` stays untouched until Phase 11.

**Lockstep:** WASM is a compile target, not a parallel TS rewrite. After #82, the TS DSP file is deleted and `guitarCapture.ts` calls `WasmGuitarInput.process_block()` for every block. The fixture corpus (see Test strategy) is run against the *same Rust binary* on both native and wasm32-unknown-unknown — any byte-difference in emitted `MidiEvent` sequences fails CI. This is the H5 fix from #79.

### Implementation outline
The issue body itself is the plan; phases 0-11 are explicit. The research-level adjustments:

1. **Resolve the five open questions in the issue body before Phase 1 lands:**
   - Detector default: McLeod-only for MVP. Already in `pitch-detection = "0.3"` (`Cargo.toml:45`). pMPM/vote can be added behind a config enum.
   - Velocity model: peak-RMS in the first 5ms of the onset window (already what `guitar_input.rs:706-718` does); revisit after Phase 4 fixtures.
   - Cooldown default: 30ms (split the difference between v1.0.0 50ms and HEAD 20ms).
   - Pitch-bend range default: 48 (MPE). Standard MIDI mode is a config flip, not a default change. Document the producer/consumer contract in a `// SAFETY` comment.
   - Trait shape: per-stage traits (already shipped in `stages.rs`). Don't unify.
2. **Phase 0.5 (insert before Phase 1): gate all `console_log!` calls in `wasm/src/lib.rs:716-770` behind a `cfg!(debug_assertions)` or runtime flag.** Per-frame logging is in CONCERNS.md; #82 inherits the issue if not fixed now.
3. **Phase 6 lockstep enforcement: byte-identical event sequences.** Add a CI job that runs the fixture corpus on both `cargo test` (native) and `wasm-pack test --headless` (wasm) and diffs JSON output. Anything non-byte-identical is a bug.
4. **Phase 11 deletes both legacy modules in one commit** — `guitar_input.rs` (Rust legacy) **and** `guitarInputDsp.ts` (TS port). The TS port stays in the dead-weight category until then because Phase 6 fixture testing needs the lockstep proof first.
5. **Debug window** is #79's deliverable — folded into Phase 10 per the issue body. See #79 below.

### Test strategy
**Fixture corpus committed to `tests/fixtures/guitar/`** (new directory). One subdir per scenario containing `input.wav` (5 s mono), `expected.json` (`Vec<MidiEvent>`), `config.json` (which stage variant ran).

TDD-first fixtures (write before code):
- `wobble_initial_bend/` — #79 H1 repro: pluck, hold, no bend cents in emitted events when `initial_bend_enabled=false`.
- `octave_lock_first_onset/` — #79 H3 repro: synthesized E2 + strong H2 (164 Hz) component; expects MIDI 40, not 52.
- `string_scrape_should_not_trigger/` — accidental string slide; expects zero `NoteOn`.
- `no_calibration_single_channel/` — #79 H4: every event on channel 0 when calibration is `None`.
- `chromatic_run_clean/` — 12-note chromatic from E2 to E3; expects 12 clean NoteOn/NoteOff pairs.
- `held_vibrato_bend/` — sustained note with periodic ±20-cent oscillation; expects PitchBend events when `continuous_bend_enabled=true`, none when `false`.

Runner pattern: load wav → push blocks through `GuitarInput::process_block` (or new pipeline) → compare emitted Vec<MidiEvent> to expected.json with exact equality. Same fixture must produce byte-identical output under both `all-legacy` and `all-new` stage configs *eventually*; until then, the A/B is the manual UAT.

Existing benchmark `tests/pitch_accuracy_benchmark.rs` covers synthetic signals — keep it; it's complementary.

### Dependencies
No new dependencies. `pitch-detection = "0.3"` is already pinned. pMPM is in the same crate. `hound` (WAV reader) is likely already pulled transitively; if not, `hound = "3.5"` is ~50 KB.

### Entropy impact
- Net: **negative** at Phase 11 — deletes 4,030 Rust lines + 1,313 TS lines, replaced by ~2,000 Rust lines (estimate, based on stage trait sizes). Removes the WASM lockstep failure class entirely.
- Net: **positive during phases 1-10** — both pipelines coexist. Compile time +10-20% during the rewrite window. Worth it for stage-by-stage A/B.
- Surfaces touched: native (Tauri + CLI), WASM. No nih-plug impact (not yet wired). Touches one Tauri command (`src-tauri/src/commands/guitar.rs`) only if `GuitarInputConfig` schema changes — keep the public schema stable.
- Risk of regression: high during phases 1-10 (default stays legacy, so prod is safe); low at Phase 11 (fixture corpus is the safety net).

### Open questions / blockers
- The TS port `guitarInputDsp.ts` — does anything still call it after `WasmGuitarInput` was introduced? `wasm/src/lib.rs:693-727` strongly implies WASM already goes through Rust. Spike: 30-minute audit of `guitarCapture.ts` to confirm `processGuitarBlock()` no longer calls the TS DSP. If confirmed, delete the TS file in Phase 0.5 (not Phase 11) — that immediately closes #79 H5 without waiting for the full rewrite.
- Whether to keep the `inference.rs` CNN code in the new pipeline (Concerns says it's a 949-line dead branch). Recommend: move it to `examples/` in Phase 0.5 to reduce the surface being touched.

### Estimated effort
**L (1-3 weeks)** — 11 phases. Most phases are 1-2 days each; the WASM lockstep proof (Phase 6) and the debug window (Phase 10) are the longest single phases.

---

## #79 — Guitar pipeline pitch-stability bug + debug window

### Problem
Initial pitch bend fires unconditionally on every NoteOn at `guitar_input.rs:743`, regardless of `config.bends_enabled`. With software synths and no string-body mask, this produces audible micro-wobble per pluck. Plus the issue proposes a debug Tauri window with 15 per-stage bypass flags and live fixture-recording.

### Touchpoints
- `crates/contrapunk-audio/src/guitar_input.rs:741-755` — the bug
- `crates/contrapunk-audio/src/guitar_input.rs:39` — current `bends_enabled` field (conflates initial vs continuous)
- `src-tauri/tauri.conf.json` — would need a second window definition for `/debug/guitar`
- `ui/src/lib/components/` — new `DebugGuitarPanel.svelte` (debug window UI)
- `ui/src/routes/debug/guitar/+page.svelte` — new route
- The 15 bypass-flag wiring touches every stage in `guitar_input.rs`

### Architecture verdict
**In-repo bug fix.** The 1-line gate (`if self.config.bends_enabled && initial_bend.abs() > 0`) is trivial. The debug window is in-repo too — it shares process state with the live pipeline; an out-of-process debugger would need IPC and would lag.

**Critical recommendation:** **If #82 lands in the same milestone, skip #79 entirely.** The debug window's value is for the rewrite (per-stage A/B), and the wobble bug is #82's wobble_initial_bend fixture from day 1. Bolting 15 bypass flags onto the legacy pipeline only to throw them away in Phase 11 is wasted entropy. The issue itself says "Supersedes #79" lives in #82's body.

**Only do #79 if #82 is more than ~2 weeks out.** In that case the scope shrinks to:
1. The 1-line gate at `guitar_input.rs:743` — ship as a patch release. Done in <1 hour.
2. **Skip the debug window.** Build it as Phase 10 of #82 instead.

### Implementation outline (minimal patch path)
1. Edit `guitar_input.rs:743`: `if self.config.bends_enabled && initial_bend.abs() > 0 { ... }`.
2. Add a fixture `tests/fixtures/guitar/wobble_initial_bend/` (same one as #82 Phase 1). Asserts no PitchBend in events when `bends_enabled=false`.
3. Bump patch version, ship.

The H1-H5 followup tickets (per the issue body) can be filed as separate small issues. H5 (TS port drift) is closed by #82.

### Test strategy
One fixture: `wobble_initial_bend/`. Plays a soft pluck, expects events with no PitchBend cents when bends_enabled=false. Two assertions: (a) no `PitchBend` event in the sequence, (b) NoteOn note matches the played note ±1 semitone.

### Dependencies
None.

### Entropy impact
Patch path: zero. Full path (debug window): +1 Tauri window config, +1 Svelte route, +15 boolean fields on `GuitarInputConfig`, +15 if-gates in `process_block`. Discardable in Phase 11 of #82.

### Open questions / blockers
- Confirm `config.bends_enabled` semantics with the user before splitting it. The issue suggests `initial_bend_enabled` + `continuous_bend_enabled` — that's the better long-term design, but it's a config schema change (breaks persisted profiles).

### Estimated effort
**S (1-3 days)** for patch path. **M (3-7 days)** for full debug window — **don't pay this if #82 is imminent**.

---

## #29 — Real-time polyphonic pitch detection (research-only)

### Problem
Decide what #28's polyphonic Performance Mode should use: tract-onnx running the basic-pitch model, NeuralNote's frame-by-frame CNN decomposition pattern (4 sub-models + circular buffers), or hand-coded Conv2D kernels with RTNeural-style pre-trained weights.

### Touchpoints
None — this is a research-only issue. Output is `.planning/research/realtime-polyphonic-decision.md` (or this section serves as that doc).

### Architecture verdict
**Research-only.** No code lands from #29 itself. Output is a recommendation that #28's plan adopts.

### Findings (this is the research)
**Real-time polyphonic pitch detection with basic-pitch is structurally hard.** From NeuralNote's own README: "Basic Pitch uses the Constant-Q transform (CQT) as input feature. The CQT requires really long audio chunks (>1s) to get amplitudes for the lowest frequency bins. The neural network introduces approximately 120ms of additional delay. The note detection algorithm processes data non-causally (backward in time)." NeuralNote ships as **offline-only**.

basicpitch.cpp (sevagh, C++/ONNXRuntime) ships only batch inference. Spotify's `basic-pitch-ts` (browser TS) is batch-only — the streaming proposal in `spotify/basic-pitch#171` is still open.

**NeuralNote's frame-by-frame decomposition pattern is real:**
- The basic-pitch CNN is split into 4 sequential sub-models wired via circular buffers
- RTNeural runs the CNN part; ONNXRuntime runs the CQT feature extraction
- 10-frame lookahead (~116ms at the 86 fps stride)
- Required custom 2D-convolution support in RTNeural (the project contributed it upstream)

But: **NeuralNote itself doesn't ship the decomposed version for real-time use**. The issue title mentions `myk-polypitch-detect-plugin` as "working real-time VST (38x real-time on desktop)" — that's throughput (offline speed), not streaming latency.

**Recommendation for #28:**
1. **For Performance Mode polyphonic chord/key detection** (the actual #28 use case), 150-300 ms latency is **acceptable** — chord changes happen on bar boundaries, not on per-frame timing. Use **basic-pitch with tract-onnx in batch mode**: collect 1-2 seconds of audio in a ringbuffer, run inference on each chunk, emit chord/key updates. **No NeuralNote-style decomposition needed.**
2. **For low-latency polyphonic note transcription** (the harder use case, more like a polyphonic guitar synth), the NeuralNote pattern is the only known design. **This is XL effort and not what #28 needs.** Defer indefinitely.
3. **Spike** (Phase 0 of #28): benchmark tract-onnx loading `nmp.onnx` and running CQT+model on a 22050 Hz 1-second buffer. Target: <100 ms wall-time on M-series. If it hits, ship batch-mode Performance Mode and move on.

### Dependencies
None for this issue. Drives #28.

### Entropy impact
Zero — research-only.

### Open questions / blockers
- Whether anyone has measured tract-onnx inference latency for the 17K-parameter basic-pitch model on a 1-second buffer. Likely fast (the model is tiny). Spike confirms.

### Estimated effort
**XS (≤1 day)** — this section is the research output.

---

## #28 — Performance Mode: basic-pitch polyphonic analysis via tract-onnx

### Problem
Performance Mode needs polyphonic analysis for chord detection, key detection, and backing-track transcription. The issue proposes basic-pitch (17K params, Apache-2.0, ONNX format) running in parallel to the monophonic pMPM path via tract-onnx.

### Touchpoints
- New module: `crates/contrapunk-audio/src/polyphonic/` (or extend `inference.rs` after #82 Phase 0.5 cleanup)
- The Performance Mode harmony hook lives in `crates/contrapunk-harmony/` (per Phase 7 in ROADMAP.md) — not yet built
- Model file: **does not ship in the binary** — downloaded on first launch and cached in app-data dir
- `crates/contrapunk-audio/Cargo.toml` — add `tract-onnx = "0.21"` under a `polyphonic` feature flag

### Architecture verdict
**Mixed: runtime in-repo, model file external.**
- Runtime (tract-onnx + CQT feature extractor + post-processing): in-repo behind a `polyphonic` feature flag. Compiles into all four surfaces but only enabled by default for desktop.
- Model file (`nmp.onnx`, ~7-8 MB): **does not ship in the binary**. Downloaded on first launch to `~/.config/contrapunk/models/basic-pitch/nmp.onnx` (per the credentials-location convention in user memory). User-controllable: "Performance Mode" first-launch prompt asks consent before download. SHA-256 verified.

Rationale: 7-8 MB is too much to bake into every WASM bundle for users who never touch Performance Mode. External hosting (a Cloudflare R2 bucket or GitHub Release asset) is one HTTP fetch and a one-time cache; release boundary cost is low because the model itself rarely changes.

### Implementation outline
**Phase A: ONNX runtime spike** (1-2 days)
1. Add `tract-onnx = "0.21"` to `crates/contrapunk-audio/Cargo.toml` under `[features] polyphonic = []`.
2. Wire `nmp.onnx` loader using `tract_onnx::onnx().model_for_path(...)`.
3. Write a smoke test: load the model, run a 22050-sample zeros input, verify it produces three output heads (contour, note, onset) without panicking.
4. Benchmark inference latency on a 1-second buffer; target <100ms wall time on M-series. **If this fails, abandon tract-onnx and re-evaluate `ort` or `candle`.** See ONNX runtime comparison below.

**Phase B: CQT feature extraction in Rust** (2-3 days)
5. Port the harmonic-CQT preprocessing from basic-pitch-ts (TypeScript reference) to Rust. This is ~150 LoC of FFT-based filterbank computation. No new deps (use existing `rustfft`).
6. Verify CQT output matches the Python reference on a known input (write a fixture).

**Phase C: Post-processing** (2-3 days)
7. Port `outputToNotesPoly()` from basic-pitch-ts: activation matrices → note events. ~200 LoC.
8. Emit `Vec<PolyphonicNote { midi: u8, start_ms: u32, end_ms: u32, velocity: u8 }>`.

**Phase D: WASM build** (1-2 days)
9. Compile crate with `polyphonic` feature for `wasm32-unknown-unknown`. tract supports this target (confirmed via `webonnx/wonnx` ecosystem and tract's prior browser deployments). Binary size delta: ~2-3 MB compressed wasm.
10. Implement model-file loader: fetch from CDN URL, store in IndexedDB on web, app-data dir on Tauri.

**Phase E: Performance Mode integration** (3-5 days, depends on Phase 7 roadmap)
11. Wire output into chord/key detector in `crates/contrapunk-harmony/`.

### Test strategy
- Smoke test: load `nmp.onnx`, infer on zeros — no panic, correct output shape.
- Fixture: a 3-second C major arpeggio at 22050 Hz, mono → expected chord events `[C, E, G]` with timing.
- Determinism test: same input on native and wasm must produce byte-identical output activations (lockstep, same principle as #82).
- Performance test: latency <150ms on M1 Pro, <300ms on Apple Silicon WebKit (WASM).

### Dependencies
- **`tract-onnx = "0.21"`** — verified active (0.21.15 released 2026-03-09). Dual MIT/Apache-2.0. Sonos production-tested. Supports wasm32-unknown-unknown (prior deployments documented; the 2020 WASM-support issue confirmed it worked even then with no source changes). Pure Rust — no C/C++ deps. Supports Conv2D, BatchNorm, ReLU, Sigmoid (everything basic-pitch needs).
- **basic-pitch `nmp.onnx` model** — Apache-2.0 (verified). ~7-8 MB. Hosted by us on a CDN; we vendor a SHA-256.
- Binary size delta:
  - tract-onnx + model loader code: ~2-3 MB wasm gzipped (estimate from comparable tract deployments)
  - Model file: ~7-8 MB (not in binary; downloaded)
- `rustfft` already in the dep tree (for guitar harmonic analysis).

### ONNX-runtime-in-Rust-WASM comparison (sub-investigation)
The user explicitly requested a dedicated comparison. Three Rust ONNX runtime candidates:

| Runtime | License | WASM? | Pure Rust? | basic-pitch viable? | Notes |
|---|---|---|---|---|---|
| **tract-onnx 0.21.15** | MIT/Apache-2.0 | **Yes** (wasm32-unknown-unknown, documented working) | Yes | **Yes** — model uses Conv2D + BN + Sigmoid only; tract supports these | Sonos production runtime. Last release 2026-03-09. ~2-3 MB wasm. Recommended. |
| **ort 2.0.0-rc.12** (pyke) | MIT/Apache-2.0 | Partial — "supports deployment to WASM with tract or candle backends" per pyke docs; the native ORT binary is **not** WASM-targetable; ort wraps Microsoft's ONNX Runtime C++ which doesn't compile to wasm32-unknown-unknown | No (C/C++ deps) | Yes for native; for WASM you end up using tract under ort anyway | ort is great for *native* — up to 9x faster than naive setups. But for WASM you fall back to tract; ort adds wrapper complexity for no benefit. Skip for our use case. |
| **candle 0.9.x** (HuggingFace) | MIT/Apache-2.0 | **Yes** (production demos: YOLO, Whisper, T5 in-browser) | Yes | Partial — candle-onnx supports core ops but is younger than tract; not all ONNX models load cleanly. The 17K-parameter basic-pitch model is simple enough it'd likely work, but tract has a deeper track record for ONNX import. | Better suited to transformer/attention workloads. Slower than PyTorch for some CNNs (candle issue #942). Not first choice for this. |
| wonnx | Apache-2.0 | WebGPU only, no fallback | Yes | Untested with basic-pitch | WebGPU-accelerated; great for browsers with WebGPU, but iOS Safari WebGPU is recent and uneven. Not first choice for cross-browser. |

**Verdict: tract-onnx.** Best Rust ONNX runtime for this specific model in WASM. ort has no WASM advantage (it'd use tract internally anyway). candle is fine but newer and less battle-tested for ONNX import. wonnx is platform-restricted.

### Entropy impact
- New crate dep `tract-onnx` (~2-3 MB wasm). Behind `polyphonic` feature flag — disabled by default for nih-plug surface (plugins don't want heavy ML).
- Model file is external: zero compile-time impact, +one cold-start fetch on first Performance Mode use.
- Touches: `contrapunk-audio` crate (new module), `contrapunk-harmony` (chord detection hook), Tauri command for model download, WASM adapter for IndexedDB fetch.
- Risk: tract-onnx WASM binary growth. Mitigation: feature-flag the polyphonic module out of the default WASM build; load on demand from a separate wasm bundle (split-build).

### Open questions / blockers
- Where to host `nmp.onnx` for first-launch download? Recommend Cloudflare R2 (we already have the CF Pages relationship per user memory).
- Whether the polyphonic module should compile in by default for desktop Tauri (probably yes) and WASM (probably no, lazy-load).
- Whether to ship a smaller-quantized basic-pitch model. Spotify only publishes the float32 version; quantization is a separate optimization.

### Estimated effort
**L (1-3 weeks)** — depends on Phase 7 (Performance Mode harmony work) being scoped first. ONNX runtime spike alone is XS (≤1 day) and **should run first** — if tract-onnx can't load `nmp.onnx` cleanly, the whole approach pivots.

---

## #27 — Integration tests: real guitar recordings + basic-pitch ground truth

### Problem
Current `tests/pitch_accuracy_benchmark.rs` and `tests/audio_pipeline.rs` use synthetic signals (sine sweeps, additive harmonics). They miss real-world failure modes: string scrape, fret buzz, pick noise, palm-mute partials. Need real guitar recordings with reference MIDI to catch regressions.

### Touchpoints
- `tests/fixtures/guitar/` — new directory. Same one #82 Phase 1+ will use.
- `tests/audio_pipeline.rs` — extend or add `tests/guitar_real_recordings.rs`
- `crates/contrapunk-audio/Cargo.toml` — add `hound = "3.5"` if not already present
- `.gitattributes` — mark WAVs as binary via Git LFS (recommendation below)

### Architecture verdict
**In-repo tests.** Tests belong with code; fixtures live under `tests/fixtures/guitar/`. Concrete sub-decisions:

1. **Recording source: record our own.** Don't redistribute IDMT-SMT-Guitar (academic license, restricted redistribution per Fraunhofer terms) or GuitarSet (CC license but bundled redistribution complicates a public repo). Recording 30-60 seconds total of fixture audio is ~5 minutes of guitar time per session. User memory ("test after every increment") confirms this fits the workflow.
2. **Ground truth: hybrid.** For monophonic fixtures (most of them), hand-author the `expected.json` from sheet music + manual review — cheaper and more accurate than running basic-pitch on a guitar pluck sequence. For polyphonic fixtures (chord strumming), run basic-pitch offline once, manually audit and correct, commit the corrected JSON.
3. **No copyrighted music.** Originals only: scales, arpeggios, public-domain melodies (Bach inventions, traditional folk tunes ≥100 years old). Document in `tests/fixtures/guitar/README.md`.
4. **Git LFS or raw?** WAVs at 48 kHz mono 16-bit are ~96 KB/sec → 5-second fixtures are ~480 KB each. 20 fixtures = ~10 MB total. Manageable as raw blobs without LFS for the first batch; add LFS at the 50-fixture mark.

### Implementation outline
1. **Phase 1: harness** (1 day)
   - Add `tests/guitar_real_recordings.rs`. Use `hound` to load WAV, push through `GuitarInput` (or the #82 pipeline), compare emitted MidiEvent sequence to `expected.json`.
   - Equality is exact: same channel, same note, same velocity (±2), same cents (±5).
2. **Phase 2: record the first 6 fixtures** (1-2 days)
   - Open strings (E2-E4): one fixture per string.
   - Chromatic scale low E to high E.
   - Simple riff (e.g., "Mary Had a Little Lamb" first bar).
   - Hand-author `expected.json` for each.
3. **Phase 3: integration** (1 day)
   - Wire into `cargo test` default. Add `--test-threads=1` if the harness reads global audio config.
   - Add to CI; flag any latency regression beyond ±15 ms.
4. **Phase 4: chord fixtures** (2-3 days)
   - 3 strummed chords (C, G, Am) recorded with a metronome click track.
   - Run basic-pitch offline (Python or `basic-pitch-ts`), audit output, hand-correct, commit.

### Test strategy
This *is* the test strategy for everything else. Recursive — the fixture corpus is itself the regression net. New bugs in the pipeline add fixtures **before** fixes.

### Dependencies
- `hound = "3.5"` (~50 KB, MIT, last released 2024-Q4). Pure Rust WAV reader. Likely already a transitive dep; if not, trivially added.
- Recording chain: user's existing setup (mag pickup → audio interface → 48 kHz mono WAV). No new hardware.
- For Phase 4: `basic-pitch` Python CLI (one-shot tool, not a runtime dep) — run locally, commit the corrected output.

### Entropy impact
- +10 MB repo size at 20 fixtures, +50 MB at 100 fixtures. Add Git LFS when crossing 30 MB.
- +1 test file. Slows `cargo test` by ~5-10 s (loading 20 WAVs sequentially). Acceptable.
- Affects nothing in production code; affects CI runtime budget.
- Risk: the fixtures encode current pipeline behavior; if a fix changes a fixture's expected output, every fixture downstream needs an audit. Document this in `tests/fixtures/guitar/README.md`.

### Open questions / blockers
- Whether to require a specific recording setup (one guitar, one interface, fixed gain) for reproducibility, or allow varied conditions. Recommend: document the rig once, use it consistently, note any deviations per-fixture.
- Whether #82 should share these fixtures verbatim or add its own under `guitar_pipeline/` subdirs. Recommend: shared corpus at `tests/fixtures/guitar/`, both pipelines consume it. Each `config.json` says which pipeline variant runs.

### Estimated effort
**M (3-7 days)** — harness + 6 monophonic + 3 polyphonic fixtures. Can grow incrementally; only the harness is blocker for #82 Phase 1.
