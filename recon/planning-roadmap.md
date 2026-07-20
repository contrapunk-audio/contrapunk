# Contrapunk Planning & Roadmap Recon

Generated from the requested read-only planning/docs sources only. No `.planning/` or source files were modified.

## Executive snapshot

- **Product baseline:** Contrapunk is positioned as a real-time counterpoint harmony generator + OSS plugin host for guitar/MIDI input, with rule-based voice-leading styles and a shared Rust core across native desktop, browser/WASM, and plugin surfaces. Evidence: `README.md:14`, `README.md:53-59`; market claim in `docs/MARKET_ANALYSIS.md:5`, `docs/MARKET_ANALYSIS.md:84-86`.
- **Current planning focus:** `.planning/STATE.md` says the current focus is **Elixir v1.5 / elixir-v0.1.0**: synth engine replacement + standalone product + plugin path. Evidence: `.planning/STATE.md:22-24`.
- **Important state conflict:** the top of `STATE.md` still says “Phase 06.4” and `ROADMAP.md` still says Phase 21 is queued/0 of 24, but later `STATE.md` appendices show Phase 21 execution has advanced through A6, A-Cut, B3, B4, and B5 slices. Treat the later `STATE.md` closure notes as the freshest evidence. Evidence: stale/top entries `.planning/STATE.md:28-29`, `.planning/ROADMAP.md:983-1007`; newer closures `.planning/STATE.md:542-706`.
- **Latest explicit next work:** `STATE.md` ends with **plugin preset state serialization / loading path, then B7 wavetable parity**. Evidence: `.planning/STATE.md:706`.
- **Planning docs caveat:** several `.planning/codebase/*.md` files are explicitly marked stale after crate split/I-O restructure. Use them as conceptual context only; verify paths before implementation. Evidence: `.planning/codebase/ARCHITECTURE.md:3-7`, `.planning/codebase/STRUCTURE.md:3-15`, `.planning/codebase/CONCERNS.md:3-21`.

## Planned features by area

### Core Contrapunk product roadmap

- **Modal harmony UX review:** Phase 6.4 is code-complete but still needs UX/usage review/human verification. Evidence: `.planning/ROADMAP.md:250-263`, `.planning/ROADMAP.md:960`.
- **Note Generator + WASM parity:** virtual MIDI generator, chord/note selection, beat-synced arpeggiator/scale/random modes, full harmony/humanize pipeline, native + WASM. Currently deferred. Evidence: `.planning/ROADMAP.md:265-287`, `.planning/ROADMAP.md:961`.
- **Logo/branding completion:** SVG/logo/favicons partly done; remaining Tauri PNG app icons and monochrome variant. Evidence: `.planning/ROADMAP.md:368-388`, status mismatch at `.planning/ROADMAP.md:968`.
- **DMG distribution:** signed/notarized macOS DMG with app icon, background, Gatekeeper-safe launch. Plans still TBD. Evidence: `.planning/ROADMAP.md:390-403`.
- **Performance Mode:** backing-track chord/key/BPM detection plus bar-aware accompaniment from accumulated user playing. Not started. Evidence: `.planning/ROADMAP.md:405-437`, `.planning/ROADMAP.md:969`.
- **Mic Input:** microphone capture, pitch-to-MIDI, raw audio buffer for vocoder; planning complete with 8 plans. Evidence: `.planning/ROADMAP.md:439-459`, `.planning/ROADMAP.md:970`.
- **Vocoder:** classic and harmony vocoder using mic input and harmony engine output. Not started. Evidence: `.planning/ROADMAP.md:461-475`, `.planning/ROADMAP.md:971`.
- **Guitar Input:** audio input, monophonic/polyphonic pitch detection, string/fret visualization, ML refinement, browser calibration; roadmap says ~95% complete on `guitar-input-clean` branch. Evidence: `.planning/ROADMAP.md:477-497`, `.planning/ROADMAP.md:972`.
- **openDAW device integration:** MIDI effect device inside openDAW; research complete, prototype/fork/PR steps listed. Evidence: `.planning/ROADMAP.md:499-518`, `.planning/ROADMAP.md:976`.
- **Contrapunk Cloud:** online jamming platform with low-latency audio networking and shared harmony sessions; waitlist/concept exists. Evidence: `.planning/ROADMAP.md:520-535`, `.planning/ROADMAP.md:977`.
- **Contrapunk as plugin:** VST3/CLAP/AU plugin with webview GUI; roadmap says 4/7 complete, AU works in Logic, webview GUI WIP. Evidence: `.planning/ROADMAP.md:40`, `.planning/ROADMAP.md:978`.
- **Integration test pipeline / MIR research:** real guitar fixtures pending, Basic-Pitch path chosen, NeuralNote research documented. Evidence: `.planning/ROADMAP.md:979-981`.
- **Release engineering:** weekly patch/monthly minor releases, codename generator, backport workflow, Dependabot; ~80%. Evidence: `.planning/ROADMAP.md:982`.

### Elixir milestone: v1.5 / elixir-v0.1.0

Locked decisions:

1. Elixir crates live inside this workspace.
2. A-Cut waits for full feature parity after A6.
3. Track B UI uses egui and opens separately from Contrapunk.
4. Track C v1 scope includes CLAP + VST3 + AU.
5. `elixir-standalone` is public and released from this repo under `elixir-v*` tags.
6. The Elixir plan/design were ingested into `.planning/`.

Evidence: `.planning/PROJECT.md:73-84`, `ELIXIR-PLAN.md:7-14`, `ELIXIR-PLAN.md:325-330`.

Planned tracks:

| Track | Planned output | Evidence |
|---|---|---|
| **A — Replace built-in synth** | `elixir-core` behind `elixir-synth`, then A-Cut, then default flip and legacy synth deletion. | `ELIXIR-PLAN.md:24`, `ELIXIR-PLAN.md:156-168`; roadmap detail `.planning/ROADMAP.md:610-737` |
| **B — Standalone Elixir product** | `elixir-standalone`, `elixir-plugin`, preset save/load, egui UI, wavetable editor, headless renderer, public v0.1.0 release. | `ELIXIR-PLAN.md:25`, `ELIXIR-PLAN.md:174-189`; roadmap detail `.planning/ROADMAP.md:739-866` |
| **C — Multi-plugin hosting in Contrapunk** | CLAP activation/GUI/automation/state, then VST3 hosting, then AU hosting. | `ELIXIR-PLAN.md:26`, `ELIXIR-PLAN.md:216-226`; roadmap detail `.planning/ROADMAP.md:868-936` |

Fresh Elixir execution status from `STATE.md`:

- Handoff had A0-A5 plus B0/B1/B6/B6.1/B6.2 complete. Evidence: `.planning/STATE.md:32-48`.
- **A6 complete**: spectral/FX completion closed; validation passed for `elixir-core` tests/check/build. Evidence: `.planning/STATE.md:542-560`.
- **A-Cut initial Tauri cutover complete** behind feature flag; checks/tests passed. Evidence: `.planning/STATE.md:568-586`.
- **B3 plugin skeleton complete**; `cargo check -p elixir-plugin` clean. Evidence: `.planning/STATE.md:590-605`.
- **B4.1/B4.2 parameter + FX automation complete** for `elixir-plugin`; checks clean. Evidence: `.planning/STATE.md:609-645`.
- **B5.1/B5.2 preset import + standalone Vital import UI complete**; next remains plugin preset serialization/loading, then B7 wavetable parity. Evidence: `.planning/STATE.md:649-706`.
- Remaining Elixir work still implied by plan: B5 plugin save/load completion, B7 wavetable editor/full wavetable parity, B8 headless renderer, B9 public release, A7 default flip/legacy cleanup, and C0-C4 hosted-plugin work.

Key Elixir validation expectations:

- Per-phase DSP unit/property/no-allocation checks. Evidence: `ELIXIR-PLAN.md:277-283`.
- A-Cut parity via fixed MIDI render against legacy synth with RMS diff `< -90 dBFS`, surface smoke tests, and existing audio pipeline tests under both feature configs. Evidence: `ELIXIR-PLAN.md:287-289`.
- Plugin hosting tests via `pluginval`, Surge XT CLAP, TDR Nova VST3. Evidence: `ELIXIR-PLAN.md:293-294`.
- Continuous harmony tests and WASM build must stay green. Evidence: `ELIXIR-PLAN.md:296-299`.

### Golem proposal: separate adaptive drummer product

Golem is not in the current GSD roadmap table yet, but the design/research docs define a separate product/crate family like Elixir.

Planned/desired features:

- Audio-native adaptive drummer, not MIDI-first; listens to guitar/clock/future Contrapunk state and outputs drum audio. Evidence: `GOLEM-DESIGN.md:3-18`, `GOLEM-DESIGN.md:7`.
- Svelte/Tauri standalone UI from day one; Rust owns realtime audio. Evidence: `GOLEM-DESIGN.md:25-45`.
- Initial scaffold: `crates/golem-core/` and `apps/golem/`; future `golem-preset`, `golem-kit`, `golem-plugin`, and `src/chain/golem_block.rs`. Evidence: `GOLEM-DESIGN.md:48-67`.
- MVP “Golem Jam v0.1”: guitar input device, start/stop, BPM, styles, 2D drummer pad, swing/fill/follow/master volume, live meters, procedural drums. Acceptance includes separate Tauri app, drum audio without MIDI, stable internal sample clock, guitar RMS/onset following, continues when guitar stops, no allocation/blocking in output callback. Evidence: `GOLEM-DESIGN.md:70-93`.
- Research direction: deterministic adaptive grooves first; avoid ML/search and audio-thread inference until product loop is proven. Evidence: `GOLEM-RESEARCH.md:18`, `GOLEM-RESEARCH.md:31`, `GOLEM-RESEARCH.md:298-314`.
- Open Golem questions: procedural vs embedded WAVs, kit format timing, source of guitar-follow input, and first Contrapunk integration bus placement. Evidence: `GOLEM-RESEARCH.md:316-321`.

## Current paused, deferred, or unresolved work

| Work | State | Evidence |
|---|---|---|
| Phase 6.4 Modal Harmony UX | Code complete; UX/usage review pending. | `.planning/ROADMAP.md:250-263`, `.planning/ROADMAP.md:960` |
| Phase 6.5 Note Generator + WASM parity | Deferred; engine done/not wired; WASM humanize/gen stubs noted. | `.planning/ROADMAP.md:265-287`, `.planning/ROADMAP.md:961`; narrative in `docs/PROJECT_JOURNEY.md:173-181` |
| Plugin Companion lanes | Deferred to v1.4/later; UI hidden via adapter capability. | `.planning/STATE.md:129-132` |
| Plugin audio FX wiring | Deferred; synth/reverb/delay params in nih-plug currently hidden. | `.planning/STATE.md:131-133` |
| Calibration profile load path | Triage item: save exists but no consumer/load path; duplicate profile types. | `.planning/STATE.md:132-133`; stale-but-recent caveat in `.planning/codebase/CONCERNS.md:15-21` |
| AudioWorklet fallback removal | Candidate cleanup; ScriptProcessor fallback vestigial. | `.planning/STATE.md:134` |
| Elixir bundle IDs | Human action before B9 signing/release. | `.planning/STATE.md:57-60`, `.planning/STATE.md:563-566`; `ELIXIR-PLAN.md:342` |
| Track priority | `STATE.md` asked whether to continue A-Cut or open B/C; subsequent B3-B5 work suggests B track was opened, but formal state line remains. | `.planning/STATE.md:563-566`, later `.planning/STATE.md:588-706` |
| Roadmap cross-cutting issues | Verify desktop MIDI enumeration, Fly.io redeploy, desktop settings persistence. | `.planning/ROADMAP.md:1011-1013` |
| Audio Foundation / plugin hosting | Concerns doc says in-flight/paused, but file is stale; verify before acting. | `.planning/codebase/CONCERNS.md:219-222`, stale warning `.planning/codebase/CONCERNS.md:3-21` |
| Golem | Design/research exists; no `GOLEM-PLAN.md` in requested docs and open questions remain. | `GOLEM-RESEARCH.md:316-327` |

## Roadmap phase inventory

Use this table as a quick map; prefer `STATE.md` closure notes for the freshest Elixir progress.

| Phase | Planned feature | Roadmap status |
|---|---|---|
| 1 | MIDI Foundation | Complete (`.planning/ROADMAP.md:15`, progress `.planning/ROADMAP.md:949`) |
| 2 | Harmony Engine | Complete (`.planning/ROADMAP.md:16`, progress `.planning/ROADMAP.md:950`) |
| 3 | GUI and Distribution | Complete (`.planning/ROADMAP.md:17`, progress `.planning/ROADMAP.md:951`) |
| 4 | Server Mode | Complete (`.planning/ROADMAP.md:18`, progress `.planning/ROADMAP.md:952`) |
| 5 | Octave Variations | Complete (`.planning/ROADMAP.md:19`, progress `.planning/ROADMAP.md:953`) |
| 5.1 | WASM and browser | Complete (`.planning/ROADMAP.md:20`, progress `.planning/ROADMAP.md:954`) |
| 6 | Humanization | Progress table says complete, but top checklist remains unchecked; treat as complete per summaries. Evidence: `.planning/ROADMAP.md:21`, `.planning/ROADMAP.md:955`, `.planning/STATE.md` Phase 6 summary around humanization. |
| 6.1 | Humanization UI Fix | Progress table says complete, top checklist unchecked. Evidence: `.planning/ROADMAP.md:22`, `.planning/ROADMAP.md:956` |
| 6.2 | Voice Leading | Complete (`.planning/ROADMAP.md:23`, `.planning/ROADMAP.md:957`) |
| 6.3 | Style Update | Complete (`.planning/ROADMAP.md:24`, `.planning/ROADMAP.md:959`) |
| 6.4 | Modal Harmony & Chord Detection | Code complete; UX review needed (`.planning/ROADMAP.md:25`, `.planning/ROADMAP.md:960`) |
| 6.5 | Note Generator + WASM parity | Deferred (`.planning/ROADMAP.md:26`, `.planning/ROADMAP.md:961`) |
| 6.6 | Default MIDI Selection | Complete (`.planning/ROADMAP.md:27`, `.planning/ROADMAP.md:962`) |
| 6.7 | Extended Scale Modes & Barry Harris | Complete (`.planning/ROADMAP.md:28`, `.planning/ROADMAP.md:963`) |
| 6.8 | CI Fix | Complete (`.planning/ROADMAP.md:29`, `.planning/ROADMAP.md:964`) |
| 6.9 | Repo Cleanup & Documentation | Complete (`.planning/ROADMAP.md:30`, `.planning/ROADMAP.md:965`) |
| 6.10 | Docs | Complete per progress table, top checklist unchecked (`.planning/ROADMAP.md:31`, `.planning/ROADMAP.md:966`) |
| 6.10.1 | UI Modernization | Complete per progress table (`.planning/ROADMAP.md:32`, `.planning/ROADMAP.md:967`) |
| 6.11 | Logo | Mixed: progress table says 0/? not started; detail shows SVG/brand/favicon done, icons/monochrome pending (`.planning/ROADMAP.md:33`, `.planning/ROADMAP.md:383-388`, `.planning/ROADMAP.md:968`) |
| 6.12 | DMG Distribution | Plans TBD (`.planning/ROADMAP.md:38`, `.planning/ROADMAP.md:390-403`) |
| 7 | Performance Mode | Not started (`.planning/ROADMAP.md:34`, `.planning/ROADMAP.md:969`) |
| 8 | Mic Input | Planning complete; 0/8 implementation (`.planning/ROADMAP.md:35`, `.planning/ROADMAP.md:970`) |
| 9 | Vocoder | Not started (`.planning/ROADMAP.md:36`, `.planning/ROADMAP.md:971`) |
| 10 | Guitar Input | ~95% on `guitar-input-clean` branch (`.planning/ROADMAP.md:37`, `.planning/ROADMAP.md:972`) |
| 11 | Trackpad Beat Input | Dropped (`.planning/ROADMAP.md:39`, `.planning/ROADMAP.md:973`) |
| 12 | Advanced Voice Leading | Closed/already shipped (`.planning/ROADMAP.md:974`) |
| 13 | Voice Leading Test Suite | Closed; inline tests exist (`.planning/ROADMAP.md:975`) |
| 14 | openDAW Device Integration | Research complete (`.planning/ROADMAP.md:976`) |
| 15 | Contrapunk Cloud | Waitlist live, concept defined (`.planning/ROADMAP.md:977`) |
| 16 | VST3/CLAP/AU Plugin | 4/7, plugin builds, AU works in Logic, webview GUI WIP (`.planning/ROADMAP.md:978`) |
| 17 | Integration Test Pipeline | 1/3; synthetic benchmarks done, real fixtures pending (`.planning/ROADMAP.md:979`) |
| 18 | Basic-Pitch Polyphonic | Research complete, tract-onnx path chosen (`.planning/ROADMAP.md:980`) |
| 19 | NeuralNote Research | Research complete (`.planning/ROADMAP.md:981`) |
| 20 | Release Engineering | ~80% (`.planning/ROADMAP.md:982`) |
| 21 | Elixir Milestone | Roadmap table says queued/0 of 24, but `STATE.md` later shows active progress through A6/A-Cut/B5 slices. Evidence: `.planning/ROADMAP.md:983-1007`, `.planning/STATE.md:542-706` |

## Evidence paths read

Primary planning:

- `.planning/PROJECT.md` — core value, active/out-of-scope initial requirements, Elixir locked decisions.
- `.planning/STATE.md` — freshest current focus and Elixir closure notes.
- `.planning/ROADMAP.md` — full phase list, planned requirements, progress table, cross-cutting issues.

Codebase context docs:

- `.planning/codebase/ARCHITECTURE.md` — conceptual multi-surface architecture; explicitly stale for file paths.
- `.planning/codebase/STRUCTURE.md` — stale path map but useful crate split warning.
- `.planning/codebase/CONCERNS.md` — stale in parts, but contains deferred/paused work and caveats.
- `.planning/codebase/STACK.md` — workspace/surface stack.
- `.planning/codebase/INTEGRATIONS.md`, `CONVENTIONS.md`, `TESTING.md` — integration/test/style context; some paths may predate crate split.

External docs:

- `ELIXIR-PLAN.md` — locked Elixir plan and validation strategy.
- `GOLEM-DESIGN.md`, `GOLEM-RESEARCH.md` — proposed adaptive drummer product.
- `docs/MARKET_ANALYSIS.md` — product gap/positioning.
- `docs/PROJECT_JOURNEY.md` — historical narrative, older than May Elixir work; useful but stale on future phases.
- `README.md` — current public positioning and build/test summary.
