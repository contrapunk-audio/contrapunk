# Brutal critique of research wave — 2026-05-11

**Reviewed:** issue-triage.md + 9 group-*.md files
**Goal:** prune over-engineering and premature plugin-ification before this becomes a roadmap

---

## Sharp findings to KEEP (don't relitigate)

These are the research wave's wins. The roadmap should bake them in without further debate.

- **Group F — #70 is obsolete.** The website already consumes this repo via a git submodule + Vite alias `@cp/*`. The "vendored copy drift" problem the issue was filed against no longer exists. Close it with a one-line comment. (`group-f-embed-ui.md` § Issue #70)
- **Group F — Hydra is AGPL-3.0.** Bundling AGPL into an MIT codebase served from `app.contrapunk.com` is a one-way door. The recommendation to pick Butterchurn (MIT) or roll a custom GLSL visualizer is correct. Anyone proposing "iframe sandbox to defuse AGPL §13" is asking for a lawyer bill; reject. (`group-f-embed-ui.md` § Issue #101)
- **Group G — `robbert-vdh/nih-plug` upstream is dead.** Verified via issue #265. The canonical successor is `BillyDM/nih-plug`. Anyone planning plugin work who hasn't internalized this is already wrong. (`group-g-vst-plugin.md` § External-library state)
- **Group G — webview-in-VST is the wrong path.** Path B (native VIZIA UI) is correct. The webview path has 4 forked repos, an unverified clap-wrapper AU GUI forwarding story, ~25-40 MB plugin bundles, and zero AU-loaded-in-Logic confirmations after weeks. The "one UI for all surfaces" dream is paid for by every user via plugin install size and instability. Cut losses. (`group-g-vst-plugin.md` § Three architectures evaluated)
- **Group B — #79 is mostly subsumed by #82.** The 1-line gate fix at `guitar_input.rs:743` is the only piece worth landing in isolation; the 15-bypass-flag debug window is wasted entropy if #82's Phase 11 deletes the legacy pipeline anyway. (`group-b-pitch-ml.md` § #79)
- **Group B — basic-pitch is not low-latency-streaming-capable.** NeuralNote ships offline-only for a reason (CQT needs >1s, ML adds 120ms, non-causal post-processing). Anyone planning "real-time polyphonic note transcription via basic-pitch" is going to spend 3 weeks discovering this. Use batch mode for chord/key detection (150-300ms is fine for chord changes); abandon the polyphonic-synth use case. (`group-b-pitch-ml.md` § #29)
- **Group H — #91 is partially obsolete.** The pattern programmer it was filed against was deleted in `1376c4c`. The companion `Lane::tick()` is the pure function the issue asked for. Re-scope or close. (`group-h-bugs-refactors.md` § #91)
- **Group D — Most DAW-integration issues already have a stable boundary.** The right move on #10 (openDAW) and #31 (TouchDesigner) is "zero source changes in this repo, stand up external bridges." On #11 (SonoBus), embedding JUCE+GPLv3 into MIT is rejected on sight. (`group-d-daw-integration.md`)
- **Group A — #4 is shipped; close after one tuning pass.** Don't relitigate Krumhansl vs current detector before the user has hands-on UAT on real-music phrases. (`group-a-core-harmony.md` § #4)
- **Group C — `fundsp` rejected for #105.** Adding a 200KB+ DSP framework for one tanh+quantize chain is exactly the kind of "we might need it later" that makes binaries bloat. Hand-roll, revisit if 3+ FX want it. (`group-c-fx-synth.md` § #105)
- **Group C — DDSP belongs external.** The user already runs an Elixir wavetable synth externally for the same reasons. tract-onnx in the workspace adds 3-5MB and a Rust 1.91+ MSRV bump for an opt-in feature. (`group-c-fx-synth.md` § #104)

---

## Verdicts I'd flip

### Flip: #97 SamplerAudioBlock — "in-repo plugin" → **in-repo core (single new module, no new crate)**

Group-c proposes `crates/contrapunk-sampler/` as a new workspace member. Their stated reasons:

1. "Sampler is large (~400-600 LOC)." A new module is ~600 LOC. A new crate is ~600 LOC of code plus a `Cargo.toml`, plus a place to drift, plus a separate test invocation, plus an extra `cargo check` target. The crate-vs-module decision should be driven by *circular dependency risk* or *consumer plurality*, not by line count.
2. "Independently testable via `cargo test -p contrapunk-sampler`." `cargo test --lib --package contrapunk` on a single module gives you that today, in 1/3 the time it takes to set up the crate.
3. "Independently versionable for future nih-plug." This is speculation. The plugin already pulls `contrapunk = { path = ".." }`; one more module on the dep tree is free. When the sampler genuinely needs its own version, *then* split the crate. Doing it now is premature optimization for a future that may not exist.
4. "`crates/contrapunk-audio/` already exists for shared types." Then put `SampleBuffer` and `Sampler` in `contrapunk-audio`. One crate is doing what two crates were proposed to do.

Net effect of the flip: same code, one less Cargo.toml, one less workspace member. The "in-repo plugin" framing here is cargo-cult — the BeatMachineLane case (#103) is in-repo-plugin because it implements an *abstraction* (`Lane`) and registers itself into orchestrator; that's the kind of "plugin" that earns its name. A `SampleBuffer` + `Sampler` + ADSR is a module, not a plugin.

### Flip: #103 BeatMachineLane — keep "in-repo plugin" but **defer until #97 ships and a second concrete Lane exists**

Building BeatMachineLane to "validate the Lane abstraction" is the wrong reason to build it. You don't get to "validate an abstraction" by writing exactly one consumer of it — the abstraction is whatever fits that one consumer, by tautology. Validation comes from seeing whether a *second, structurally different* Lane (LooperLane is on the jam-features-2026 calendar) reveals friction in the trait shape. Ship LooperLane first; if its `tick()` and `phase()` and `InputFilter` carry through to BeatMachine cleanly, you have a real abstraction. If not, you got a refactor for free.

### Flip: #15 Strudel route — "in-repo UI" → **separate AGPL build target or external sub-app**

`group-e-livecoding.md` calls this "in-repo UI (Svelte route + in-page bridge)" and waves at "lazy chunk = AGPL boundary." This is wishful AGPL reading. AGPL §13 cares about *whether the user interacts with the program over a network*, not how the bundler chunks the JS. Serving an AGPL-loaded route from `app.contrapunk.com` very plausibly triggers AGPL on the *page that loads it*, which is the entire SvelteKit app shell.

The safe path:
- Move the Strudel route to a separate Cloudflare Pages deployment (e.g. `strudel.contrapunk.com`) that links *back* to the main app for the actual harmony engine via the existing WASM bundle URL. The main MIT app stays uncontaminated; the AGPL boundary is at the origin level.
- Alternative: drop the Strudel integration entirely until the user has confirmed they want to commit to AGPL release obligations for an indie audio tool.

Either way, "lazy chunk on the same origin" is not the architectural boundary the research thinks it is. Get a 30-minute legal opinion before committing.

### Flip: #102 ListenLane — "in-repo plugin + feature flag" → **out-of-milestone; spike-only first**

Group-h proposes `crates/contrapunk-ml/` with `ort = "2.0.0-rc.12"`, plus a Demucs 80MB ONNX model fetched at runtime, plus basic-pitch 17MB, plus BlackHole capture, plus a new audio_capture module. That is L effort with three open questions still unanswered (can htdemucs even be exported as single-file ONNX, what's RTF on M-series, what's the binary delta of `ort`'s native lib download). This is research-only territory.

Phase a *spike* (XS, 1-2 days): pull `sevagh/demucs.onnx`, load via `ort`, run it on a 30-second clip, measure RTF. If RTF > 0.5 on M-series, write up the result and *defer the feature* to a future milestone. Don't add a new ML crate, a new feature flag, a new model-download pipeline, and a new audio-capture module to this milestone on the strength of an estimate.

### Flip: #8 Rhythm-aware triggers — "in-repo core (split into 8a/8b/8c)" → **only 8b this milestone**

Group-a's split is good. But shipping #8a (TempoEstimator) requires the *synthetic beat counter to consume the inferred BPM*, which is a structural change to the engine's clock model. That deserves its own design pass before code. #8b (per-voice phase offset) is one new `Vec<f32>` field on the engine + a slider; ship that alone and defer 8a until a clear use case justifies the engine clock-model change.

### Flip: #105 TextureFX — keep "in-repo core" but **reduce scope to "Bitcrush + Tape (tanh) only"**

The 4-stage chain (tanh → quantize → noise inject → LP filter) is fine as a final design, but proposing 4 presets ("Clean / Tape / Lo-Fi / Destroy") in the v1 ship is over-scoping. Ship the chain skeleton + 2 hand-tuned presets (Tape, Destroy) and let the user discover whether they actually want the noise stage. Saves ~1 day of preset tuning that you'd rather spend on the deadline-driven #106.

---

## Effort lies

T-shirt sizes that won't survive contact with the code:

| Issue | Research said | Reality |
|---|---|---|
| **#82 guitar pipeline rewrite** | L (1-3 weeks) | **XL (4-6 weeks).** 11 phases. WASM byte-identical lockstep proof is a CI-infrastructure project on its own. Phase 11's "delete 4030 Rust + 1313 TS lines in one commit" is going to have a long tail of fallout. The Phase 0.5 cleanup (gate logs, move inference.rs to examples) plus the fixture corpus from #27 (M on its own) are *not* in the 1-3 week estimate. |
| **#3 Canon mode** | M (3-7 days) | **L (7-12 days).** The router-tick path for "engine emits without user input" is a new real-time-safety code path. New emit infrastructure + per-voice delay buffers + UI for delay/transpose controls + 4-surface propagation + correctness tests for note-off lifecycle on key change. Plus the real-time-safety review the research punted on. |
| **#28 Performance Mode (basic-pitch)** | L (1-3 weeks) | **XL.** Phases A-E adding to 9-14 days, but blocked on "spike to confirm tract-onnx loads nmp.onnx cleanly" *and* "spike to verify Phase 7 Performance Mode harmony work scope" *and* "spike to confirm WASM tract-onnx binary size." Three spikes are not a 9-14 day estimate, they're "do the spikes, then estimate." |
| **#100 bass register suppression Approach B** | S (1-3 days) | **M.** "Plumb velocity through `harmonize_note_on` across 5+ call sites" sounds easy until you discover that one of those call sites is the WASM bindgen path, which means version-bumping the WASM API, which means the website's submodule-bumped UI needs to match. Cross-surface API changes are *never* S. |
| **#101 Hydra/Butterchurn** | M (3-7 days) | **L (10-14 days) IF license decision is made on day 1.** The research correctly flags the license blocker. But even after picking Butterchurn, the Rust-FFT-on-audio-thread + Tauri event throttling + lazy import + 3-4 presets + cross-surface AnalyserNode parity is a non-trivial integration with real-time-safety risk. |
| **#102 ListenLane** | L (1-3 weeks) | **XL or out-of-milestone.** See "Verdicts I'd flip." |
| **#9 plugin Path B** | M (5-7 days post-spike) | **L (10-14 days post-spike).** VIZIA UI rewrite + CI for *signed* macOS bundles (`xcrun notarytool` is a real ordeal) + Windows code-sign cert procurement + 5-DAW manual UAT matrix. "Build `release.sh`" is one line of plan covering several days of CI tinkering. |
| **#15 Strudel + OSC bridge** | M (5-9 days) | **L.** Add the AGPL boundary work + the actual Strudel `onTrigger` interception bug-hunting (Strudel's scheduler is well-documented but not trivial to wedge a custom output into) + per-language docs reality-checking. The OSC crate part *is* S; the Strudel part isn't. |
| **#106 Drone + Bitcrusher (Wk3 jam)** | S (1-3 days) | **S is correct** if-and-only-if the WASM "acceptance criterion" is dropped. The research correctly flags that WASM has no chain today. Don't try to wedge browser audio output into a 3-day jam ship. |
| **#42 AUTOPLAY** | XS | XS is honest. Believe it. |
| **#65 Presets UI redesign** | S | S is honest, *if* you don't get caught in a design rabbit hole on day 1. Force the pill-row pattern and ship; don't open the cmd-K palette debate. |

The bigger pattern: every research doc estimates as if the issue is the only thing in motion. The XL items (#82, #28, #102, #9 Path B if signing is in scope) are mutually-incompatible-in-the-same-milestone unless someone else is working in parallel on the others.

---

## Cross-doc conflicts

- **#102 ListenLane vs #28 Performance Mode** both want `crates/contrapunk-ml/` and the `ort`/tract-onnx infrastructure. Group-b proposes tract-onnx for #28; group-h proposes `ort = "2.0.0-rc.12"`. **These are different runtimes.** Pick one; building two ONNX-runtime paths through one repo is exactly the entropy `issue-triage.md` warned against. Recommend tract-onnx (group-b's recommendation is correct: pure-Rust, WASM-targetable, smaller). Group-h needs to update.

- **#82 fixture corpus vs #27 fixture corpus.** Both groups propose `tests/fixtures/guitar/`. Group-b's #82 plan implies the corpus is born from #79's wobble-repro and chromatic-run cases. Group-b's #27 plan implies a separate 6-fixture monophonic + 3-fixture polyphonic recording session. These are the same corpus and they conflict on who owns it. Resolution: #27 ships the *harness* (test runner + WAV reader + assertion shape), then #82 phases 1+ add fixtures incrementally as features land. Don't pre-record 9 fixtures and then build the pipeline around them.

- **#100 bass register suppression vs #102 ListenLane.** Both want to do "smart input filtering" — #100 at the engine input (gate by velocity + register), #102 at the audio-capture stage (separate stems, detect external bass). These are not conflicting features but they *both* propose adding fields to `HarmonyEngine` / `WorldState`. Decide which field-bag owns "input filtering" before either lands.

- **#3 Canon mode vs companion architecture.** Group-a proposes a new `tick_canon(beat_pos) -> Vec<...>` method on `HarmonyEngine` called from the router thread on each beat crossing. Group-h's companion-arch refactor (#91 successor) has Lanes emit `DispatchOp`s via the `Companion::tick()` orchestrator. Canon is by definition a Decide-phase Lane, *not* an engine method. Building Canon as an engine-level tick first and then refactoring to a Lane later is exactly the duplication the companion arch is supposed to prevent. Either build Canon as a Lane from day 1, or delay it until the companion-router wiring lands.

- **#106 Drone-as-AudioBlock vs companion-arch DroneLane.** Group-c's deadline-driven v0 makes Drone an `AudioBlock`. Companion arch (`01-companion-architecture.md:251`) lists DroneLane as a Decide-phase Lane on an audio graph that doesn't exist yet. The research correctly flags this as "ship v0 first, refactor to Lane later" but doesn't acknowledge that "refactor to Lane later" is itself S effort, not free. Budget the refactor.

- **#82 says "delete `inference.rs` to examples/" vs `CONCERNS.md` listing it as paused-but-existing.** Either move it or document a reason to keep it. The research recommendation is right; the next milestone should just *do it*, not wait for #82 phase 0.5.

- **#9 plugin says "rebase fork onto BillyDM/nih-plug" vs #82 plan doesn't acknowledge plugin surface.** When #82 changes `GuitarInput`, does the plugin's `Audio` mode (which uses guitar pitch-detect, plugin/src/lib.rs) inherit cleanly? If not, that's a plugin maintenance cost #82 hasn't priced in.

- **#65 Presets UI vs #100 StylePreset schema additions.** #65 wants to redesign the Presets UI surface; #100 wants to add two new `u8` fields to `StylePreset`. These conflict on the preset migration story: does the new UI render old presets that lack the new fields? Group-a's recommendation (`#[serde(default = "default_…")]`) handles the data side, but the *UI* should either ship after the schema settles or be explicit about which fields are exposed.

---

## Things that shouldn't be built at all (in this milestone)

- **#15 Strudel route.** AGPL boundary, deferred 30-minute legal opinion, scoped 5-9 days that's actually L. There's no user data this is needed for the v1.2.x window. Defer.
- **#15 OSC bridge for Sonic Pi.** Sonic Pi already speaks MIDI well via IAC. The research itself admits "docs make MIDI the default, OSC the power-user fallback." Ship the docs page. Build the `crates/contrapunk-osc/` crate only when one user has actually asked for it and confirmed that virtual MIDI doesn't work for them. Today: zero such users.
- **#11 SonoBus embedding.** Reject by default. Docs-only is XS and good. Anything else is L+ and walks into GPLv3.
- **#31 TouchDesigner.** External sub-project, blocked on td-rs license clarification. The research correctly flags this; don't let it back in via a workspace member.
- **#102 ListenLane full ship.** Spike only this milestone. The full feature is XL and depends on three unproven ONNX questions.
- **#28 Performance Mode full ship.** Spike-only this milestone. Phase A (tract-onnx loads nmp.onnx, RTF check) is XS and answers whether the whole approach pivots; ship that as research and revisit.
- **#101 Hydra-replacement visualizer.** Nice-to-have, no roadmap dependency. License decision + 10-14 days of integration is not worth it when the existing HLD afterimage already provides "visual identity." Defer until a user complains.
- **#3 Canon mode** unless the companion-arch wiring lands first. Canon-as-engine-tick is throwaway code.
- **#8c pattern detection** (the "1-3-5 eighths → arpeggio" piece). Group-a correctly defers; just don't let it sneak back in.

---

## Dependency footprint audit

| Dep / Package | Proposed by | Cost | Verdict |
|---|---|---|---|
| `tract-onnx = "0.21"` | Group-b (#28) | +2-3 MB WASM, +Rust 1.91 MSRV, ~30s CI build | **Keep, but only behind feature flag, only after Phase A spike passes.** Do not add for #102 — share the runtime. |
| `ort = "2.0.0-rc.12"` | Group-h (#102) | C++ ONNX Runtime binary + FFI surface, several MB native lib download | **Drop.** Use tract-onnx instead. Don't run two ONNX runtimes. |
| `rusty_link` | Group-d (#98) | ~200-400 KB, GPLv2+, hobby-maintained | **Keep behind `ableton-link` feature flag.** License-quarantined correctly. |
| `rosc = "0.11"` | Group-e (#15) | ~50 KB, MIT/Apache | **Drop for this milestone.** Build only after a user asks for OSC. |
| `tokio` (for OSC) | Group-e (#15) | Multi-MB transitive cost | **Drop.** Even if OSC ships, use `std::net::UdpSocket` + a thread (research itself prefers this). |
| `fundsp` | Group-c (#105) considered | +200 KB | **Drop.** Hand-roll the 40-line tanh+quantize. Revisit if 3+ FX want it. |
| `hound = "3.5"` | Group-c (#97), Group-h (#27, #102) | ~120 KB, pure Rust, ISC | **Keep.** Shared across 3 issues. Cheap. |
| `spectrum-analyzer` (Rust) | Group-f (#101) | ~80 KB, no_std, MIT/Apache | **Drop until #101 ships.** |
| `@strudel/web` + `@strudel/transpiler` | Group-e (#15) | ~250-500 KB, **AGPL-3.0** | **Drop or move to separate origin.** Single biggest license risk in the wave. |
| `hydra-synth` | Group-f (#101) | **AGPL-3.0**, 1.79 MB | **Reject.** Confirmed by research. |
| `butterchurn` | Group-f (#101) | 728 KB, MIT | **Keep IF #101 ships at all.** Defer to next milestone. |
| `nih_plug_vizia`, `vizia` | Group-g (#9) | bundled with nih-plug, MIT/ISC | **Keep IF plugin Path B is taken.** Confirmed by spike. |
| `td-rs` | Group-d (#31) | alpha, **NO LICENSE FILE** | **Reject until license clarified.** Research correctly flags. |
| `ableton-link-rs` (pure-Rust alt) | Group-d (#98) considered | early-stage, GPLv2+ | **Drop.** rusty_link is the right pick. |
| `wonnx` (WebGPU ONNX) | Group-b (#28) considered | WebGPU-only, no fallback | **Drop.** Cross-browser incompat. |
| `candle` (HuggingFace) | Group-b (#28) considered | Younger than tract, transformer-focused | **Drop for ONNX.** Use tract. |
| Demucs ONNX model | Group-h (#102) | ~80 MB external download | **Drop for milestone.** Spike-only. |
| basic-pitch ONNX (`nmp.onnx`) | Group-b (#28), Group-h (#102) | ~7-8 MB external | **Keep for #28 spike; not bundled.** |
| `spectrum-analyzer` (Rust) | Group-f (#101) | ~80 KB | **Drop with #101 deferral.** |

The "drop" column dominates because the research treated dependencies as line items to compare on technical merit instead of as long-term liabilities. Every `Cargo.toml` line is a year of maintenance.

---

## Refactor-debt warning signs

The research is piling new code on these already-high-entropy areas without proposing the refactor first:

1. **`guitar_input.rs` (4030 LOC, called out in CONCERNS.md).** #82 *does* propose the rewrite, but #79 proposes adding 15 bypass-flag fields to it as a stop-gap. If #82 is shipping, don't pay the #79 cost. Pick one path.

2. **`src-tauri/src/commands/engine.rs` (1041 LOC, the `run_tauri_router` body is 280 LOC).** #14 (bug fix), #90 (CC 123 + reconcile), and #91 (companion wiring) all touch this file. None of them propose extracting *just* the panic-replay / detune / knob-cc-raw / note-update-emit blocks into named functions first. Result: three concurrent diffs to the same 280-line block. Either serialize the work (#14 → #90 → #91) or extract the panic-replay block as a pure first PR.

3. **`wasm/src/lib.rs` (1012 LOC, per CONCERNS).** #82 plans to delete `guitarInputDsp.ts` (1313 LOC), good. But #100's velocity plumbing, #3's `set_mode("canon")` route, and #8's tempo state would all add new exports. Nobody has proposed grouping the WASM exports into namespaced modules. The file is a junk drawer.

4. **`harmony/engine.rs`.** #3 adds `tick_canon`. #8 adds `tempo_bpm`. #81 rewrites `KeyDetector`. #100 changes the `harmonize_note_on` signature. #90 adds `panic_clear`. That's 5 concurrent diffs to the same file. The companion arch claims to subsume the routing complexity — *use it*. Lanes that own beat-tick, tempo-detection, and canon would consolidate three of those five edits. If the companion arch isn't ready to absorb them, defer the engine-level additions until it is.

5. **`PolySynth::process_stereo` allocates per callback (CONCERNS.md).** #106 (Drone), #105 (TextureFX), #97 (Sampler) all add new AudioBlocks to the chain. None of them propose fixing the existing `vec![]` allocation in the audio callback. Fix that *first*; otherwise the new blocks inherit (and contribute to) audio glitches.

6. **`AppState::audio_out_producer` and the `try_lock` on `Arc<Mutex<AudioState>>`.** Same pattern: #97 and #103 inherit this lock. The lock should be removed (per CONCERNS recommendation) before new audio paths take a dependency on it.

The pattern across all six: **the research wave proposed features that touch hot, high-entropy modules without sequencing the cleanup work first.** A v1.2.x milestone that adds 7 new features on top of `engine.rs`, `wasm/src/lib.rs`, and `commands/engine.rs` without refactoring any of them will produce a v1.3.0 codebase that is meaningfully harder to work in.

---

## Recommendation: roadmap shape

This is the brutally narrow milestone I'd ship. 4 phases, deferred items named, refactors sequenced.

### Phase 1 — Refactor cleanup before features (1 week)

Goal: lower entropy in the modules the next phases will touch.

- Extract `panic_replay`, `detune_dispatch`, `knob_cc_raw_forward`, `note_update_emit` from `run_tauri_router` into named pure functions. ~150 LOC moves; unblocks #14/#90/#91 working concurrently.
- Fix `PolySynth::process_stereo` allocation (CONCERNS.md). Pre-allocate scratch buffer.
- Remove `try_lock` on `Arc<Mutex<AudioState>>` per CONCERNS recommendation. Move `PolySynth` ownership to audio thread, keep MIDI events on the lock-free ringbuffer.
- Move `inference.rs` (949-line dead CNN) to `examples/`.
- Gate `wasm/src/lib.rs:716-770` `console_log!` calls behind `cfg!(debug_assertions)`.
- Close #70 with a one-line comment. Close #4 after tuning pass.

### Phase 2 — Ship-fast features deadline-driven (1 week)

Goal: hit the May 14 Wk3 jam ship.

- **#106 Drone + Bitcrusher** — in-repo, 2 new AudioBlocks. Drop the WASM acceptance criterion explicitly. Ship desktop-only, document the gap.
- **#14 MIDI-out routing default fix** — UI hydration heuristic + log warning. XS.

### Phase 3 — Bugs + small features (2 weeks)

- **#79 patch path only** (1-line gate on initial bend). Skip the debug window — #82 owns that.
- **#90 CC 123 + reconcile** — companion `WorldState` cross-ref against `engine.active_notes`.
- **#91 successor** — wire `Companion::tick()` into `run_tauri_router` + golden tests. Defer concrete Lanes.
- **#100 bass register suppression Approach B** — velocity API plumb + early-return. Bumps WASM API; coordinate with website submodule bump.
- **#42 AUTOPLAY** — Svelte component, no Rust.
- **#65 Presets UI redesign** — pill-row pattern, no design rabbit hole.
- **#66 wave 3+4** — Piano wrapper swap + ChordReadout extract.
- **#81 Krumhansl detector** — pure-math rewrite of `key_detect.rs`. Ships profiles for the 21 modes in Diatonic + HarmonicMinor + MelodicMinor families.
- **#98 Ableton Link** — feature-flagged, Tauri-only. GPLv2+ quarantined.
- **#99 DAW side-by-side docs + InputRouter** — Phases 1-4 (skip Phase 5 audio-rate sidechain until Audio Foundation resumes).
- **#8b only** — per-voice phase offset slider. Skip 8a, 8c.

### Phase 4 — Plugin Path B spike + (conditional) ship (2-3 weeks)

- **#9 spike (1 day)** — VIZIA UI for 8 params loaded in Logic Pro. Measure bundle size + AU GUI forwarding.
- **If spike passes:** drop nih_plug_webview + wry + clap-wrapper GUI fork. Rebase onto `BillyDM/nih-plug`. Build signed bundles for macOS + Windows. 5-DAW UAT matrix.
- **If spike fails:** Path C (parameters-only generic UI). Don't reinvest in webview path.

### Deferred to v1.3.x or later (explicitly named)

- **#82 guitar pipeline rewrite** — XL. Either give it its own milestone or ship phases 0-3 only and stop at the byte-identical-WASM-lockstep barrier (Phase 6 is the schedule-killer).
- **#27 guitar recording fixtures** — pulled into #82's milestone.
- **#28 Performance Mode** — spike only (1 day Phase A: tract-onnx + nmp.onnx); full ship next milestone.
- **#29** — closed as "research output captured in group-b doc."
- **#3 Canon mode** — after companion router wiring lands in Phase 3, *then* build as a Decide-phase Lane in v1.3.x. Don't build as engine-tick.
- **#97 SamplerAudioBlock** — promote to v1.3.x. Build as module in `crates/contrapunk-audio/`, not as new crate.
- **#103 BeatMachineLane** — after #97 and after LooperLane (jam-features-2026 calendar). The Lane abstraction needs a second consumer to validate against.
- **#102 ListenLane** — spike only (1 day: load htdemucs.onnx via tract-onnx, measure RTF). Full feature explicitly out-of-milestone.
- **#104 DDSP** — external sub-project planning only. Spike RTF on Apple Silicon, then decide.
- **#10 openDAW** — XS velocity-export tweak in this repo. External bridge work is outside this milestone.
- **#11 SonoBus** — docs-only, ~1 hour. Optional.
- **#15 Strudel + OSC bridge** — defer. AGPL legal opinion is a prerequisite.
- **#31 TouchDesigner** — blocked on td-rs license clarification.
- **#101 Hydra-replacement visualizer** — defer.
- **#105 TextureFX** — defer to v1.3.x; #106 already ships a bitcrusher.
- **#106 v1 refactor** to Lane — deferred until audio graph lands.
- **#8a (TempoEstimator) and #8c (pattern detection)** — deferred indefinitely.

### What this milestone deliberately doesn't try to do

It doesn't try to "validate the companion architecture by building three Lanes." It doesn't try to "land the ML pipeline." It doesn't try to "ship every issue researchers wrote a doc about." It absorbs 14 of the 32 substantive issues with three rough buckets (refactor, ship-fast, bugfix+small-features) and saves the architectural decisions (Canon-as-Lane, Sampler-as-module-or-crate, plugin webview-vs-VIZIA) for *after* the cleanup phase has lowered the entropy of the files they'll touch.

Doing all 32 in one milestone is the only way to guarantee that none of them ship well.
