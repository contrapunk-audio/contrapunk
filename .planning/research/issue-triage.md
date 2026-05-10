# Issue Triage — May 2026

**Date:** 2026-05-11
**Trigger:** Backlog review pre-research wave. v1.1.1 just shipped.
**Total open issues:** 37
**Substantive (non-docs, non-already-done):** 32
**Research-dispatched this wave:** 32 across 9 grouped research agents

## Method

Every open issue placed into one of eight buckets. Architectural verdict (in-repo core / in-repo plugin / external sub-project / refactor / bug / docs / superseded) decided per-issue, with the **code-entropy** lens: every "in-repo" decision touches all four distribution surfaces (CLI / Tauri / WASM / nih-plug); every "external" decision creates a release boundary. Both are costs. Verdicts justified in the per-issue research docs at `.planning/research/group-*.md`.

## The 8 buckets

### A — In-repo core harmony features (7 issues)

Touch `crates/contrapunk-harmony/` or are tightly bound to the harmony engine's state. Default verdict: in-repo, low entropy.

| # | Title | Verdict (provisional) | Notes |
|---|---|---|---|
| #2 | Species counterpoint | **DONE** | Shipped in v1.1.0; bug fixed in v1.1.1 (fb3e7b9). Close. |
| #4 | Auto-detect key | in-repo core | Engine already has `key_detect.rs`; this is likely "expose / polish". |
| #3 | Canon mode | in-repo core | New `HarmonyMode::Canon` variant. |
| #42 | AUTOPLAY scale walker | in-repo core | Wires existing generator + harmony engine. |
| #65 | Re-introduce Presets UI | in-repo core | UI redesign; `PresetManager` already exists. |
| #100 | Smart bass register suppression | in-repo core | Engine input filter; cheap. |
| #8 | Rhythm-aware triggers | in-repo core | Builds on existing `BeatClock`/`Transport`. |
| #81 | Auto-key via Krumhansl profiles | in-repo core | Upgrade of #4's detector. |

Research doc: `group-a-core-harmony.md`

### B — Pitch detection / ML / guitar pipeline (5 issues)

DSP-heavy, ML-adjacent, hot-path. Verdict spectrum: most in-repo (`crates/contrapunk-audio/`), some external (model files).

| # | Title | Verdict (provisional) | Notes |
|---|---|---|---|
| #82 | Rewrite guitar→MIDI pipeline | in-repo (replaces current) | Monophonic, native + WASM lockstep — big refactor. |
| #28 | Performance Mode: basic-pitch + tract-onnx | mixed — model external, runtime in-repo | ~8MB ONNX model lives outside the binary; loader in-repo. |
| #29 | Real-time polyphonic pitch detection | research-only | Picks the approach for #28. |
| #27 | Integration tests: real guitar recordings | in-repo (tests) | Test data lives in `tests/fixtures/`. |
| #79 | Guitar pipeline pitch-stability bug | in-repo bug fix | Stop-gap until #82 lands. |

Research doc: `group-b-pitch-ml.md`

### C — Synth / FX sub-projects (5 issues)

Heavy DSP, often standalone-able. Verdict heavily skewed toward in-repo plugin (separate crate) or external — depends per-issue on how tightly it integrates with the harmony engine.

| # | Title | Verdict (provisional) | Notes |
|---|---|---|---|
| #105 | TextureFX corrosion-style distortion | in-repo plugin | New `crates/contrapunk-fx/` or extend existing FX chain. |
| #106 | Drone layer + Bitcrusher (Wk3 jam ship May 14) | in-repo (deadline-driven) | Ship-fast; refactor to plugin later if successful. |
| #104 | DDSP tone transfer (neural DSP) | **external** | Heavy ML pipeline; separate sub-project. |
| #97 | Sample-based playback engine (SamplerAudioBlock) | in-repo plugin | New AudioBlock variant; reuse `Chain` abstraction. |
| #103 | BeatMachineLane (step sequencer + samples) | in-repo plugin | Standalone lane in companion architecture. |

Research doc: `group-c-fx-synth.md`

### D — DAW integration & sync (5 issues)

Bridges to external systems. All require stable boundaries; some can be in-repo (Tauri-only), others want their own crate or sub-project.

| # | Title | Verdict (provisional) | Notes |
|---|---|---|---|
| #10 | openDAW device integration | external bridge | OpenDAW is a separate ecosystem. |
| #11 | SonoBus integration (Contrapunk Cloud) | external bridge | Networking, peer-to-peer audio. |
| #98 | Ableton Link tempo sync | in-repo (Tauri) | `ableton-link` crate; Tauri-only, gated behind feature. |
| #99 | DAW side-by-side (IAC MIDI + BlackHole) | docs-mostly | Mostly a "how to configure" guide; minimal code. |
| #31 | TouchDesigner via td-rs | external bridge | `td-rs` crate; separate plugin binary. |

Research doc: `group-d-daw-integration.md`

### E — Livecoding bridges (1 issue)

| # | Title | Verdict (provisional) | Notes |
|---|---|---|---|
| #15 | Livecoding language integration (Strudel, Sonic Pi, TidalCycles) | external bridge(s) | OSC / MIDI bridge — one crate per protocol or one universal bridge. |

Research doc: `group-e-livecoding.md`

### F — Embed / UI / visualization (3 issues)

UI-layer; verdict varies between in-repo UI and a separate npm package.

| # | Title | Verdict (provisional) | Notes |
|---|---|---|---|
| #70 | Extract `@contrapunk/embed` package | npm sub-project | Separate npm release; consumed by website + others. |
| #66 | Canonical embed components in `contrapunk/ui/src/lib/embed/` | in-repo (UI) | Prereq for #70. |
| #101 | Hydra visualiser (WebGL audio FFT) | in-repo (UI feature flag) | Heavy npm dep; gate behind a setting. |

Research doc: `group-f-embed-ui.md`

### G — VST/AU plugin architecture (1 issue, big decision)

| # | Title | Verdict (provisional) | Notes |
|---|---|---|---|
| #9 | VST/AU plugin version for DAW integration | **mixed — needs deep research** | Plugin crate already exists (`plugin/src/lib.rs`); question is what it SHOULD be: nih-plug only? webview UI? full Tauri reuse? |

Research doc: `group-g-vst-plugin.md`

### H — Bugs + refactors (4 issues)

Tactical work, no architectural ambiguity.

| # | Title | Verdict | Notes |
|---|---|---|---|
| #14 | MIDI out not producing messages (some users) | bug | High user impact; affects existing functionality. |
| #90 | held_harmonies stale-entry recovery | bug | Edge case in MPE + dropped-NoteOff scenarios. |
| #91 | Extract router-loop pattern-tick into pure function | refactor | Testability win. |
| #102 | ListenLane (stem separation via Demucs ONNX) | refactor (new lane) | Architectural, but the lane pattern exists. |

Research doc: `group-h-bugs-refactors.md`

### I — Standalone tool (1 issue)

| # | Title | Verdict | Notes |
|---|---|---|---|
| #12 | Chord mini app | in-repo (UI route) OR external | Question: does it deserve its own URL / npm? |

Research doc: `group-i-chord-app.md`

### Skipped (already done or pure docs)

| # | Title | Why skipped |
|---|---|---|
| #2 | Species counterpoint | Done in v1.1.0 (audited / bug fixed in v1.1.1). Close. |
| #30 | Windows desktop support | Per STATE.md / commit `1e15518`, build infra shipped. Re-evaluate test-ready installer status separately. |
| #5 | Publish sample recordings | Marketing; no research needed. |
| #6 | Document end-to-end latency pipeline | Docs only. |
| #13 | Publish video demo | Marketing. |

## Code-entropy lens

The deciding question for the 32 substantive issues isn't "is this a good feature?" — most are. It's "where does this code BELONG so we don't end up with a 200k-LOC blob that's painful to ship?".

The entropy budget I'd protect:

1. **Core harmony engine** (`crates/contrapunk-harmony/`) — must stay surgical. New harmony modes are OK; new IO concerns are not.
2. **Audio I/O paths** (`src/audio_out/`, `crates/contrapunk-audio/`) — real-time safety budget is small. New allocations or heavy deps here = audio glitches.
3. **WASM bundle size** — currently ~1MB-ish. Every ML model or heavy npm dep cuts into first-load time on `app.contrapunk.com`. Hard cap at ~3MB.
4. **`src-tauri/` Cargo.lock** — adding deps here doesn't affect WASM but does balloon the desktop installer + CI time. Soft cap on new transitive deps.
5. **Number of crates / workspace members** — every new crate is a place to put code; also a place for things to drift. Adding `crates/contrapunk-fx/` is fine; adding `crates/contrapunk-fx-distortion/` separately from `crates/contrapunk-fx-bitcrush/` is over-fragmentation.

When a research doc proposes "external sub-project", it should mean: this feature has a real reason to live on its own release cadence, not just "the core repo is getting big". The Elixir wavetable synth is a legitimate external sub-project (different release cadence, different audience, different license posture). A canon mode is not — it belongs in the harmony engine.

## Dependency graph (provisional)

Researched issues feed into a roadmap. Sequencing constraints to watch:

- #66 (canonical embed components) → #70 (extract npm package). Don't extract before consolidating.
- #29 (research polyphonic pitch) → #28 (implement Performance Mode). Decision before code.
- #82 (rewrite guitar pipeline) supersedes #79 (stop-gap bug fix). If #82 ships fast, skip #79.
- #4 (auto-key UI / polish) and #81 (Krumhansl detector) are sequential — Krumhansl is the better detector for #4's UI.
- #106 (Wk3 jam ship May 14) is deadline-driven — research should prioritize a "ship something playable" path even if not architecturally pristine.

## Next steps

1. 9 research agents dispatched in parallel (this wave).
2. After research returns, `brutal-code-critic` agent reviews proposals for over-engineering / premature plugin-ification.
3. Aggregate into a v1.2.x milestone roadmap in `.planning/ROADMAP.md` (or a fresh milestone doc).
4. Surface findings via the entropy-metric hooks added in this same wave (see `.claude/skills/code-entropy/SKILL.md`).
