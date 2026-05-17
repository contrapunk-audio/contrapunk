# Decisions Intel

Synthesized from ingested ADRs/SPECs. Each entry records the decision, its status, scope, and source path so downstream consumers can trace provenance.

This file does NOT itself record locked status as an enforcement mechanism — the conflict report (`.planning/INGEST-CONFLICTS.md`) and downstream artifacts (PROJECT.md key decisions table) carry that authority.

---

## ELIX-DEC-01 — Workspace location: Inside contrapunk workspace

- **Source:** `ELIXIR-PLAN.md` §10 #1 (LOCKED)
- **Scope:** repository layout, crate organization
- **Decision:** New Elixir crates (`elixir-core`, `elixir-standalone`, `elixir-plugin`, `elixir-headless`) ship as workspace members inside the contrapunk monorepo. Sibling-repo extraction remains a future option (zero-refactor via `git filter-repo`) but is not planned.
- **Rationale:** One `cargo check` covers everything; existing CI (clippy, wasm build, tauri build, plugin build) covers Elixir for free; release infrastructure reused; crates have no Contrapunk dependencies so extraction is later-trivial.

## ELIX-DEC-02 — Cutover ambition: full feature parity at A-Cut (post-A6)

- **Source:** `ELIXIR-PLAN.md` §10 #2 (LOCKED), §2 cutover timing block
- **Scope:** Contrapunk's built-in synth replacement schedule
- **Decision:** Contrapunk's built-in synth (`src/synth/`) flips to Elixir only after the full design-doc feature set lands (after A6, ~week 12), NOT after minimum-viable parity (A5). Cutover week shifts from ~8 → ~12. Feature flag `elixir-synth` is opt-in for two stabilization weeks after A-Cut, then default flips; two more weeks of no regressions, then `src/synth/` is deleted.
- **Rationale:** Shipping a partial Elixir as the default would mean Contrapunk users lose features that the design doc promises (spectral morphs, unison, chorus, phaser, full filter set) on a temporary basis. The risk profile of "ship the new synth missing features" is worse than "ship the old synth for 4 more weeks while we finish A6". Track B's `elixir-plugin` gives DAW users early access during the build-out.

## ELIX-DEC-03 — Track B UI: egui (separate process/window from Contrapunk)

- **Source:** `ELIXIR-PLAN.md` §10 #3 (LOCKED), §4 UI choice block
- **Scope:** Elixir standalone + plugin UI framework; relationship to Contrapunk's Svelte UI
- **Decision:** Elixir's standalone app and `elixir-plugin`'s in-DAW window share one `egui` widget set. Elixir's standalone app opens as a **separate process/window** from Contrapunk's desktop app — by design, not a regression. Tauri+Svelte is rejected for Elixir specifically; it remains the right call for Contrapunk's main app.
- **Rationale:** Heavy custom-paint surfaces (mod-matrix, wavetable editor) where browser DOM offers no advantage. Single widget set shared between standalone and plugin in-DAW window — no double UI maintenance. Two products serve different users; keeping them independent (quit one, keep the other open) requires separate UI surfaces.
- **NOTE:** This decision applies **only** to Elixir's own surface. Contrapunk's UI remains Tauri+Svelte per Phase 06.10.1 (no conflict — different product, different surface).

## ELIX-DEC-04 — Track C scope for v1: all three plugin formats (CLAP + VST3 + AU)

- **Source:** `ELIXIR-PLAN.md` §10 #4 (LOCKED), §5
- **Scope:** Contrapunk's plugin hosting capability (loading external plugins inside Contrapunk)
- **Decision:** Track C ships CLAP + VST3 + AU plugin hosting in Contrapunk's `src/plugin_host/`. AU is promoted from "optional follow-up" to mandatory v1 scope. Track C duration revised 8 wk → 9 wk.
- **Rationale:** Users routing harmony through hosted plugins (e.g., Diva, Pro-Q4) need format coverage that matches what they actually own. AU is mandatory on macOS for Logic users.
- **Constraint:** AU module wrapped in `#[cfg(target_os = "macos")]`; AU is macOS-only by definition.

## ELIX-DEC-05 — elixir-standalone distribution: public, released from this repo

- **Source:** `ELIXIR-PLAN.md` §10 #5 (LOCKED), §1 Release artifacts block, §4 public release block
- **Scope:** Elixir release model, repo policy, tagging, signing
- **Decision:** `elixir-standalone` is a **public product released from this (contrapunk) repo**. Same CI / signing / notarization plumbing as Contrapunk. Tags namespaced (`elixir-v0.x` vs Contrapunk's `v1.x`). Bundle IDs reserved at `com.contrapunk.elixir` and `com.contrapunk.elixir.plugin`. Each release tag produces parallel CI jobs: Elixir standalone DMG, Elixir plugin bundle (VST3+CLAP+AU), Elixir headless binary.
- **Rationale:** Shared infrastructure (codesigning, notarization) avoids duplication. Independent product lifecycles via namespaced tags. Versioning: Elixir tracks its own SemVer independent of Contrapunk's.

## ELIX-DEC-06 — GSD milestone integration: ingest as real milestone

- **Source:** `ELIXIR-PLAN.md` §10 #6 (LOCKED)
- **Scope:** planning-system integration
- **Decision:** ELIXIR-PLAN.md and ELIXIR-DESIGN.md are ingested into `.planning/` via `/gsd-ingest-docs`, becoming a real GSD milestone with discuss/plan/execute phases.

## ELIX-DEC-07 — AudioBlock trait stability at A0

- **Source:** `ELIXIR-PLAN.md` §6 cross-track integration table
- **Scope:** trait contract for chain plumbing
- **Decision:** The `AudioBlock` trait signature is locked at A0 and only additive (never breaking) methods are added afterward. Both Elixir's `ElixirSynthBlock` and the plugin-host `ClapBlock` / `Vst3Block` / `AuBlock` implement this trait.
- **Rationale:** Two tracks (A and C) touch this trait concurrently; breaking changes mid-track destabilize both.

## ELIX-DEC-08 — ParamId namespacing for hosted plugin parameters

- **Source:** `ELIXIR-PLAN.md` §6
- **Scope:** modulation-routing parameter addressing
- **Decision:** Contrapunk's mod matrix uses a `ParamId` enum with `Internal(...) | Hosted { slot, plugin_param_id }` variants so an LFO can target a hosted plugin's filter cutoff.

## ELIX-DEC-09 — Session preset format includes per-slot plugin state

- **Source:** `ELIXIR-PLAN.md` §6
- **Scope:** session/preset serialization
- **Decision:** Session preset top-level key `chain: [{ kind: "elixir", preset: {...} }, { kind: "clap", path: ..., state: <base64> }, ...]`. No format fork between Elixir and hosted-plugin state.

## ELIX-DEC-10 — Tag prefix routing for release builds

- **Source:** `ELIXIR-PLAN.md` §6, §1 versioning block
- **Scope:** CI workflow dispatch
- **Decision:** `elixir-` tag prefix triggers Elixir build matrix; `v` tag prefix continues to trigger Contrapunk builds. Both share signing identities and notarization plumbing. `release-patch` skill extended to recognize both prefixes.

## ELIX-DEC-11 — WASM compilation constraint for elixir-core

- **Source:** `ELIXIR-PLAN.md` §6, §7 surface matrix
- **Scope:** cross-platform compile target
- **Decision:** `elixir-core` MUST compile to wasm32. Plugin-hosting code (Track C) is `cfg(not(target_arch = "wasm32"))`. CI runs `cargo check --target wasm32-unknown-unknown -p elixir-core` on every PR.
- **WASM caveat:** Contrapunk's WASM surface doesn't render Rust audio today — browser drives Web Audio from JS. Compiling `elixir-core` to wasm32 is enforced for the *standalone* Elixir web product (if Track B ever ships one), not for replacing Contrapunk's WASM synth (which doesn't exist).

## ELIX-DEC-12 — Feature flag gates Elixir synth swap

- **Source:** `ELIXIR-PLAN.md` §2 feature flag block
- **Scope:** Cargo features, cutover safety
- **Decision:** `Cargo.toml` adds `elixir-synth = ["dep:elixir-core"]` feature. `make_default_synth` returns `ElixirSynthBlock` when feature is on, `LegacySynth` otherwise. Flag stays opt-in for 2 stabilization weeks after A-Cut, then default flips, then 2 more weeks of no regressions, then `src/synth/` deleted.

## ELIX-DEC-13 — Default preset for cutover: Contrapunk-Default.elxprst

- **Source:** `ELIXIR-PLAN.md` §2 default preset block
- **Scope:** A-Cut acceptance criterion
- **Decision:** Ship a built-in `Contrapunk-Default.elxprst` factory preset: single wavetable oscillator (sine-to-saw morphable, frame=0=pure sine), one amp envelope mapped to existing `attack_ms` / `decay_ms` / `sustain_ppt` / `release_ms`, one digital-SVF lowpass at `cutoff_hz`, master gain at `master_gain_ppt`, no modulators, no FX, no unison.
- **Acceptance:** A/B render of a fixed MIDI file shows < -90 dBFS RMS difference vs `src/synth/voice.rs`.

## ELIX-DEC-14 — Real-time safety enforcement strategy

- **Source:** `ELIXIR-DESIGN.md` §7 real-time safety block
- **Scope:** audio-thread allocation discipline
- **Decision:** Four enforcement layers — (1) `assert_no_alloc` wrapping audio callback under `cfg(debug_assertions)`; (2) marker trait `RealtimeSafe` with audio-thread entry point generic over it; (3) two-arena `bumpalo::Bump` pattern for scratch memory; (4) disk persistence routed through `crossbeam-channel` to dedicated config thread.

## ELIX-DEC-15 — Lock-free state architecture

- **Source:** `ELIXIR-DESIGN.md` §7, §3 threading block
- **Scope:** UI↔audio data plumbing
- **Decision:** Three primitives. (1) Scalar parameters: `AtomicU32` storing `f32::to_bits()` with relaxed reads; nih-plug provides this. (2) Event streams: `rtrb` SPSC ringbuffer. (3) Full preset / wavetable swaps: `arc-swap::ArcSwap`. Eliminates the `pause_processing(true)` dropout window of reference implementations.

## ELIX-DEC-16 — DSP crate-stack composition

- **Source:** `ELIXIR-DESIGN.md` §2
- **Scope:** dependency choices for Elixir
- **Decision:** `nih-plug` (host abstraction), `cpal` (standalone audio), `midir` (standalone MIDI), `realfft` (FFT), `arc-swap` (lock-free hot-swap), `rtrb` (SPSC audio queue), `crossbeam-channel` (multi-producer), `bumpalo` (bump arenas), `assert_no_alloc` (debug guard), `serde`+`serde_json`+`base64`+`flate2` (preset format), `rand_chacha` (deterministic RNG), `hound` (WAV write), `clap` (argv), `enum_dispatch` (static dispatch). Hand-roll DSP rather than `fundsp`. Default SIMD `wide`; gate `std::simd` behind `cfg(feature = "nightly-simd")`.

---

*Synthesized: 2026-05-18 from `/gsd-ingest-docs` run on ELIXIR-DESIGN.md + ELIXIR-PLAN.md.*
