# v1.2.x Milestone Roadmap (LOCKED)

**Date:** 2026-05-11
**Status:** All five open questions answered 2026-05-11. Locked. Phase 1 in progress.

## Locked decisions

| Q | Decision | Impact |
|---|---|---|
| 1 | Defer plugin signing — ship unsigned for v1.2.x; revisit if user pull demands it | Plugin Phase doesn't carry $400+/yr cert burden |
| 2 | #82 guitar pipeline rewrite gets its own milestone (v1.3.x+) | Out of v1.2.x scope entirely |
| 3 | Jam dropped — #106 and #105 deferred indefinitely | Phase 2 of original plan collapses |
| 4 | **Zero AGPL/GPL stance for everything** including the main app | #15 deferred, #101 → Butterchurn or clean-room, **#98 dropped** (rusty_link is GPLv2+, contaminates the binary) |
| 5 | Companion architecture ready to absorb new code as Lanes | Phase 2's #91 successor wiring proceeds |

## License posture (CRITICAL — applies repo-wide)

**Zero GPL of any kind in `contrapunk-audio/contrapunk` or any binary it links.** This includes:
- GPLv2, GPLv2+, GPLv3, AGPL — all rejected
- LGPL is acceptable only via dynamic linking with documented load path
- MIT, Apache-2.0, ISC, BSD-2/3, MPL-2.0 — all fine

License audits become a pre-commit consideration. Any new Cargo or npm dep must show its license. The license verdicts table in `CLEAN-ROOM-CANDIDATES.md` is authoritative.

Implications for the four clean-room candidates: they remain MIT, in their own repos. Their presence is now a hard requirement (not optional) for the features they cover, because the alternative is "drop the feature entirely."
**Inputs:**
- `.planning/research/issue-triage.md` — all 37 open issues categorized
- `.planning/research/group-{a..i}-*.md` — 9 parallel research docs
- `.planning/research/CRITIQUE.md` — brutal review of the research wave

This roadmap is the synthesis of the research wave + brutal critique. It deliberately ships less than the research proposed, because the research treated each issue in isolation and ignored that they all queue diffs against the same hot files. The roadmap leads with a refactor phase so the feature phases land cleanly.

---

## The principle: entropy first, features second

The brutal critique surfaced a structural problem: **5 features queue diffs against `engine.rs` (2548 LOC)** (#3 Canon, #8 tempo, #81 Krumhansl, #100 velocity, #90 panic clear) and **3 against `run_tauri_router`** (#14, #90, #91 successor). Without sequencing the refactor first, v1.2.x would compound the entropy these files already carry.

Phase 1 lowers entropy. Phases 2-4 ship features on top of the refactored modules.

Code-entropy metrics (from `.claude/skills/code-entropy/`) confirm the priorities:

| File | LOC | 90-day churn | Hotspot score | Action |
|---|---|---|---|---|
| `ui/src/lib/adapter/wasm.ts` | 1012 | 42 | 42504 | Group exports; planned in Phase 1 |
| `src-tauri/src/commands/engine.rs` | 1041 | 35 | 36435 | Extract router blocks in Phase 1 |
| `wasm/src/lib.rs` | 947 | 27 | 25569 | Namespace exports in Phase 3 |
| `crates/contrapunk-audio/src/guitar_input.rs` | 4030 | (deferred) | — | Subsumed by #82, deferred to v1.3 |
| `crates/contrapunk-harmony/src/engine.rs` | 2548 | (high) | — | Lane refactor absorbs new code |

---

## Phase 1 — Refactor before features (1 week)

Goal: lower entropy in the modules Phases 2-4 will touch. No new features.

| Task | Files | Effort |
|---|---|---|
| Extract `panic_replay`, `detune_dispatch`, `knob_cc_raw_forward`, `note_update_emit` from `run_tauri_router` into pure functions | `src-tauri/src/commands/engine.rs` | S |
| Fix `PolySynth::process_stereo` heap allocation per audio callback (CONCERNS.md) | `src/audio_out/sine_synth.rs` | S |
| Remove `try_lock` on `Arc<Mutex<AudioState>>` — move `PolySynth` ownership to audio thread | `src/audio_out/engine.rs` | M |
| Move `inference.rs` (949-line dead CNN) to `examples/` | `crates/contrapunk-audio/src/inference.rs` | XS |
| Gate `console_log!` in `wasm/src/lib.rs:716-770` behind `cfg!(debug_assertions)` | `wasm/src/lib.rs` | XS |
| Group/namespace WASM exports (audio / harmony / transport / preset) | `wasm/src/lib.rs` | S |
| Close #70 (obsolete) and #4 (already shipped, tune only) | GitHub | XS |

Acceptance criteria: tests still pass, no regressions in `app.contrapunk.com`, hotspot scores drop on the entropy snapshot.

---

## Phase 2 — Bugs + small features (2 weeks)

The bulk of v1.2.x's user-visible work. Phase 2 of the original recommendation (jam ship-fast) collapsed into here after the jam was dropped.

| Task | Issue | Verdict | Effort | Notes |
|---|---|---|---|---|
| MIDI-out routing default fix (UI heuristic + log warning) | #14 | bug | XS | Promoted to top — high user impact. Root cause: `VoiceOutputTarget::default() = Synth` |
| #79 patch path only (1-line gate on initial bend) | #79 | bug | XS | Skip the debug window — owned by future #82 milestone |
| CC 123 + companion-vs-engine reconcile | #90 | bug | S | Cross-ref `WorldState` ↔ `engine.active_notes` |
| Wire `Companion::tick()` into router; defer concrete Lanes | #91 successor | refactor | M | Foundation for Canon/BeatMachine later |
| Bass register suppression (velocity API + early-return) | #100 | in-repo core | M | Cross-surface API break — coordinate with website submodule bump |
| AUTOPLAY scale walker (Svelte only) | #42 | UI | XS | `src/generator/` doesn't exist; UI-only via `injectNoteOn` |
| Presets UI redesign — pill-row pattern | #65 | in-repo core | S | No cmd-K palette debate |
| Canonical embed wave 3+4 (Piano wrapper + ChordReadout) | #66 | in-repo core | S | Prereq closure of #70 |
| Krumhansl auto-key detector | #81 | in-repo core | M | Pure-math rewrite of `key_detect.rs` for 21 modes |
| DAW side-by-side docs + InputRouter (Phases 1-4 only) | #99 | mixed | M | Skip audio-rate sidechain |
| Per-voice phase offset slider only | #8b | in-repo core | S | Defer #8a (TempoEstimator), #8c (pattern detection) |
| Chord mini app | #12 | in-repo UI | XS | Three Svelte files + Playwright spec — trivial |

**Removed from Phase 2** (had been planned, deferred by locked decisions):
- **#98 Ableton Link** — `rusty_link` is GPLv2+, contaminates binary. Either commit to clean-room `contrapunk-link` as a side project or skip Link sync entirely.

Acceptance: all features land with tests written first (`.claude/skills/tdd-workflow/`). The TDD-discipline hook flags any new pub fn shipped without a test in the same diff.

---

## Phase 3 — Plugin spike + conditional ship, unsigned (2 weeks)

Per locked decision #1: ship unsigned for v1.2.x. Code signing deferred to v1.3.x if plugin has real user pull.

| Task | Issue | Verdict | Effort | Notes |
|---|---|---|---|---|
| **Spike Day 1**: VIZIA UI for 8 plugin params, load in Logic Pro | #9 | spike | XS | Pass/fail decides whether to invest |
| If spike passes: drop webview path, rebase onto `BillyDM/nih-plug`, build **unsigned** bundles for macOS + Windows, 3-DAW UAT matrix | #9 Path B | in-repo | M | `robbert-vdh/nih-plug` upstream is dead — must rebase. Users see Gatekeeper warnings — document the bypass |
| If spike fails: Path C (parameters-only generic UI) | #9 Path C | in-repo | S | Don't reinvest in webview |

Phase 3 has an explicit kill switch: if the spike fails on day 1, drop the path-B commitment. Don't sink effort into webview-based plugin work that the dead-upstream + 4-fork stack can't sustain.

Effort dropped from L→M because the signing/notarization ordeal is off the critical path for v1.2.x.

---

## Deferred to v1.3.x or later (explicitly named)

These are not lost. They're recorded as `.planning/seeds/` or as backlog items with their architectural decisions already made. v1.3.x picks up where v1.2.x left off.

| Issue | Why deferred | Resumed how |
|---|---|---|
| #82 guitar pipeline rewrite | XL effort, deserves its own milestone | Ship phases 0-3 only if started; full rewrite is a milestone |
| #27 guitar recording fixtures | Pulled into #82's milestone | Co-owned with #82 |
| #28 Performance Mode (basic-pitch) | Spike-only Phase A: tract-onnx loads nmp.onnx, RTF check | Full feature next milestone |
| #29 polyphonic pitch research | Closed as "research output captured in group-b doc" | — |
| #3 Canon mode | Build as Decide-phase Lane after #91 successor wiring lands | v1.3.x |
| #97 SamplerAudioBlock | Build as module in `crates/contrapunk-audio/`, NOT new crate | v1.3.x |
| #103 BeatMachineLane | Needs LooperLane + #97 before the abstraction is validated | v1.3.x |
| #98 Ableton Link | rusty_link is GPLv2+, contaminates the MIT binary. Needs clean-room `contrapunk-link` (separate side project) | When `contrapunk-link` clean-room ships |
| #106 Drone + Bitcrusher | Jam dropped (decision #3); no longer deadline-pressured. Rebuild as DroneLane after audio-graph lands | v1.3.x |
| #105 TextureFX | Same as #106 | v1.3.x |
| #102 ListenLane | Spike-only this milestone (htdemucs RTF on M-series) | Full feature out-of-milestone |
| #104 DDSP | External sub-project planning only | Separate repo, separate cadence |
| #10 openDAW | XS velocity-export tweak in this repo; bridge is external | External sub-project |
| #11 SonoBus | Docs-only (~1h) | Embedding rejected on GPLv3 grounds |
| #15 Strudel + OSC bridge | AGPL legal opinion required; no current users for OSC | Defer indefinitely |
| #31 TouchDesigner | Blocked on td-rs license clarification (no LICENSE file in upstream) | When upstream resolves |
| #101 Hydra-replacement | Defer; existing HLD afterimage provides visual identity | When a user requests |
| #105 TextureFX | #106 already ships a bitcrusher; revisit if user wants more | v1.3.x |
| #8a TempoEstimator | Engine clock-model change needs its own design pass | When justified by a use case |
| #8c pattern detection | Group-a deferred correctly; don't let it sneak back | Indefinite |
| #106 v1 refactor to Lane | After audio graph lands | v1.3.x |
| #12 Chord mini app | XS (3 Svelte files + Playwright spec); land in Phase 3 if time | v1.3.x if not |

---

## Closures (don't build, close the issue)

| Issue | Reason |
|---|---|
| #70 | Obsolete — website already consumes via git submodule + Vite alias |
| #2 | Done in v1.0.0; species fallback bug fixed in v1.1.1 |
| #4 | Detector already shipped; close after tuning pass |
| #29 | Research-only; verdict captured in `group-b-pitch-ml.md` |

---

## Cross-cutting work that runs through every phase

- **TDD discipline.** Every feature commit ships with the test that fails first. The post-edit hook flags new pub fns without a matching test in the diff. The `tdd-workflow` skill is the procedure.
- **Entropy monitoring.** Each phase ends with a `code-entropy` snapshot. The hotspot scores should drop after Phase 1 and stay flat through Phases 2-4. If a feature drives a hotspot up, that's a signal the refactor was incomplete.
- **Per-phase verification UAT.** Phase 2 deadline-driven work especially — desktop-only ship needs a 30-minute hands-on UAT before tagging. The species fix in v1.1.1 was caught in user testing, not CI; bake that loop in.
- **One brutal-code-critic pass before each phase's merge.** Spawn the agent on the phase's PR. It found 5 verdict flips and one structural anti-pattern in the research wave — it'll find more during implementation.

---

## Open questions for the user before this becomes the active roadmap

1. **Plugin Path B commitment.** If the Phase 4 spike passes, are you willing to absorb the macOS notarization + Windows code-signing cert procurement work? That's a real ordeal (`xcrun notarytool`, DigiCert account, ~$500/yr). If no, narrow to Path C parameters-only.
2. **#82 guitar rewrite — separate milestone or stop at phase 3?** XL effort is real. Phases 0-3 might be useful as standalone shippable work without the full rewrite.
3. **#106 desktop-only acceptance.** Confirming that you're OK shipping the May 14 Wk3 jam without WASM browser support for the new FX. (The research / critique recommends this; the user is the one who pays the social cost on launch day.)
4. **AGPL exposure on #15.** Even if you defer Strudel, are you sure you want zero AGPL-touching code in `ui/`? That decision also bears on #101 (Hydra) and any future livecoding bridge.
5. **Companion architecture readiness.** Phase 3's #91 successor wiring assumes the companion Lane abstraction is far enough along to absorb new code. Is it? The recent `companion: Phase 1.4 abstraction — lanes, orchestrator, world` commit (`2c796ab`) suggests yes; confirm.

Once these are settled, write the locked version to `.planning/ROADMAP.md` under a `v1.2.x` heading and start Phase 1.

---

## Process artifacts created this session

- `.planning/research/issue-triage.md` (all 37 issues categorized)
- `.planning/research/group-{a..i}-*.md` (9 grouped research docs)
- `.planning/research/CRITIQUE.md` (brutal critique)
- `.planning/research/ROADMAP-v1.2.x.md` (this file)
- `.claude/skills/code-entropy/SKILL.md` (4-axis entropy measurement)
- `.claude/skills/tdd-workflow/SKILL.md` (test-first procedure)
- `.claude/agents/issue-researcher.md` (reusable researcher agent)
- `.claude/agents/contrapunk-harmony-fixer.md` (harmony specialist with test gate)
- `.claude/hooks/entropy-snapshot.sh` (SessionStart hotspot surfacing)
- `.claude/hooks/post-edit.sh` (TDD-discipline nudge added)
- `.claude/settings.json` (hooks + permission allowlist)
- `CLAUDE.md` (project-level instructions)

The harness compounds: each subsequent session opens with hotspot data, surfaces TDD gaps inline, and has the reusable researcher available for the issues this roadmap defers.
