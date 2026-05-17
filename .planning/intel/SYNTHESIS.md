# Synthesis Summary

Entry point for `gsd-roadmapper`. Single source for what was ingested, what conflicted, and where to read per-type detail.

**Synthesizer run:** `/gsd-ingest-docs` on 2026-05-18
**Mode:** merge
**Initiative:** Elixir — major new milestone-sized addition (synthesizer engine + standalone product + multi-plugin hosting)

---

## Doc counts by type

| Type | Count | Sources |
|---|---|---|
| ADR | 0 | — |
| SPEC | 2 | `ELIXIR-DESIGN.md` (precedence 0), `ELIXIR-PLAN.md` (precedence 1, LOCKED) |
| PRD | 0 | — |
| DOC | 0 | — |
| **Total** | **2** | |

No UNKNOWN-confidence-low docs. No type-tag disputes.

## Decisions locked

**Count:** 6 locked + 10 non-locked technical decisions extracted = 16 decisions total.

LOCKED decisions (from `ELIXIR-PLAN.md` §10):
- **ELIX-DEC-01** — Workspace location: inside contrapunk workspace
- **ELIX-DEC-02** — Cutover ambition: full feature parity at A-Cut (post-A6, ~week 12)
- **ELIX-DEC-03** — Track B UI: egui, separate process/window from Contrapunk
- **ELIX-DEC-04** — Track C scope for v1: all three plugin formats (CLAP + VST3 + AU)
- **ELIX-DEC-05** — `elixir-standalone` distribution: public, released from this repo
- **ELIX-DEC-06** — GSD milestone integration: this ingest

Non-locked technical decisions (from both docs, recording transparency for downstream):
- ELIX-DEC-07 — AudioBlock trait stability at A0
- ELIX-DEC-08 — ParamId namespacing for hosted plugin parameters
- ELIX-DEC-09 — Session preset format includes per-slot plugin state
- ELIX-DEC-10 — Tag prefix routing for release builds
- ELIX-DEC-11 — WASM compilation constraint for elixir-core
- ELIX-DEC-12 — Feature flag gates Elixir synth swap
- ELIX-DEC-13 — Default preset for cutover: `Contrapunk-Default.elxprst`
- ELIX-DEC-14 — Real-time safety enforcement (4 layers)
- ELIX-DEC-15 — Lock-free state architecture
- ELIX-DEC-16 — DSP crate-stack composition

Full detail: `.planning/intel/decisions.md`

## Requirements extracted

**Count:** 24 requirements with phase ship gates.

Breakdown by track:
- **Track A** (replace Contrapunk's built-in synth, ~13 weeks): 9 reqs — A0 workspace bootstrap, A1 bare oscillator, A2 polyphony, A3 mod matrix, A4 filter, A5 FX bus MVP, A6 spectral + FX completion, A-Cut, A7 default flip + cleanup
- **Track B** (standalone Elixir product, ~10 weeks parallel): 10 reqs — B0 skeleton through B9 public v0.1.0 release
- **Track C** (multi-plugin hosting in Contrapunk, ~9 weeks parallel): 7 reqs — C0 CLAP activation, C1 GUI embedding, C2 param automation, C3 VST3, C4 AU, multi-plugin strip UI, plugin discovery
- **Cross-cutting:** 3 reqs — WASM core compile constraint, release-pipeline extension, bundle IDs reserved

All requirements use ID convention `REQ-elixir-{slug}` and trace to `source: ELIXIR-PLAN.md §{N}`.

Full detail: `.planning/intel/requirements.md`

## Constraints

**Count:** 23 constraint entries.

Breakdown by type:
- **structural:** 4 (overall architecture, voice management, surface targets, workspace layout)
- **api-contract:** 10 (engine signal flow, framework primitives, block processing, oscillator playback, spectral morphs, filter topologies, FX chain, mod matrix, mod sources, MIDI/MPE, cutover contract, cross-track integration)
- **threading:** 1 (lock-free 2-thread model)
- **nfr:** 3 (real-time safety, testing strategy, risk register)
- **schema:** 2 (wavetable representation, preset format)
- **crate-stack:** 1 (locked dependency set)

Full detail: `.planning/intel/constraints.md`

## Context topics

**Count:** 11 topic-keyed notes (SPEC prose extracted as background context).

Topics: why Elixir is major; distinguishing trait (spectral warping); workspace location rationale; cutover ambition rationale; UI choice rationale (egui for Elixir / Svelte for Contrapunk); public-release-cadence; plugin discovery (Track C); multi-plugin chain UI (Track C); WASM caveat; surface matrix; recommended 2-week start sequence; implementation notes; relationship to existing Contrapunk planning.

Full detail: `.planning/intel/context.md`

## Conflicts

- **Blockers:** 0
- **Competing variants:** 0
- **Auto-resolved / INFO:** 7

INFO entries (none gate the workflow):
1. Companion document pair (DESIGN + PLAN)
2. Auto-resolved: PLAN precedence override on §10 locked decisions
3. Existing locked decision: Contrapunk UI is Tauri+Svelte (Phase 06.10.1) — disjoint scope from ELIX-DEC-03
4. Synth subsystem replacement scope (CLAUDE.md footgun #3 will need post-execution update)
5. Existing ROADMAP Phase 16 vs Elixir Track C scope overlap (complementary, not duplicative)
6. STATE.md "Plugin Companion lanes deferred to v1.4" vs Elixir Tracks (independent work streams)
7. Cross-ref graph is acyclic

Full detail: `.planning/INGEST-CONFLICTS.md`

## Pointers

- **Per-type intel:**
  - Decisions: `.planning/intel/decisions.md`
  - Requirements: `.planning/intel/requirements.md`
  - Constraints: `.planning/intel/constraints.md`
  - Context: `.planning/intel/context.md`
- **Conflicts report:** `.planning/INGEST-CONFLICTS.md`
- **Source documents (verbatim):**
  - `ELIXIR-DESIGN.md` (repo root)
  - `ELIXIR-PLAN.md` (repo root)
- **Existing planning surface that intersects:**
  - `.planning/PROJECT.md` — Contrapunk Rust core value, key decisions table
  - `.planning/ROADMAP.md` — current phases; Phase 16 (plugin packaging) is complementary to Track C (plugin hosting)
  - `.planning/STATE.md` — v1.3.x pre-tag; "Plugin Companion lanes deferred to v1.4" is independent of Elixir tracks
  - `.planning/phases/06.10.1-ui-modernization/06.10.1-CONTEXT.md` — Contrapunk's Tauri+Svelte UI lock; disjoint scope from Elixir egui

## Status for downstream

**READY** — safe to route to `gsd-roadmapper`. No blockers, no competing variants requiring user resolution.

The roadmapper should:
1. Treat the six ELIX-DEC-0{1..6} as LOCKED key decisions in the merged PROJECT.md.
2. Map the 24 REQ-elixir-* entries into REQUIREMENTS.md and into a new milestone in ROADMAP.md (suggested milestone label: "Elixir" or "v1.5 + elixir-v0.1.0" since Track A targets Contrapunk v1.4 cleanup and Track B targets `elixir-v0.1.0`).
3. Preserve the three-track structure (A / B / C) — these are parallel work streams sharing the `Chain` / `AudioBlock` substrate, not a linear phase sequence.
4. Recognize Track C as a continuation of existing Phase-16-adjacent plugin-hosting work (`src/plugin_host/clap/` stubs); do not double-count.
5. Note the surface matrix (constraint ELIX-CON-23 / context topic "Surface matrix"): `elixir-core` must compile to wasm32; plugin-hosting code is `cfg(not(target_arch = "wasm32"))`; Elixir's egui UI does not affect Contrapunk's Svelte UI.

---

*Synthesized: 2026-05-18.*
