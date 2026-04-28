---
title: Elixir — pre-implementation reading & evaluation list
date: 2026-04-28
priority: high
---

# Elixir — pre-implementation prerequisites

Concrete tasks to complete **before writing any oscillator code**. Clean-room implementation under permissive license requires a paper-trail of derivation sources; doing the reading first prevents inadvertent GPL contamination and avoids the rewrite tax of building on the wrong foundation.

## DSP foundations (must-read before WT engine code)

- [ ] **Watch:** Matt Tytel — "Practical Guide to Optimized High-Quality Wavetable Oscillators" (ADC 2021).
  - Video: https://www.youtube.com/watch?v=qlinVx60778
  - Slides: https://data.audio.dev/talks/2021/guide-to-optimized-wavetable-oscillators/slides.pdf
  - Why: defines modern state of the art for wavetable anti-aliasing and the technique behind Vital. Take notes; we're implementing the *concepts*, not the code.
- [ ] **Read:** Nigel Redmon's EarLevel wavetable series (multi-part): https://www.earlevel.com/main/2012/05/04/a-wavetable-oscillator%E2%80%94introduction/
  - Why: clearest public explanation of mipmap-pyramid wavetable construction (Surge XT's approach, our v1 strategy).
- [ ] **Read:** Välimäki & Huovilainen — "Antialiasing Oscillators in Subtractive Synthesis" (IEEE Signal Processing Magazine, 2007).
  - Why: BLEP/MinBLEP background for non-wavetable oscillator types and reference filter quality.
- [ ] **Read:** Will Pirkle — "Designing Software Synthesizer Plug-Ins in C++" (relevant chapters on filter topologies, ADSR shapes, mod matrix).
  - Why: textbook treatment of synth architecture; useful as a vocabulary baseline even though the code examples are GPL-incompatible to copy.

## File-format specs (before file I/O code)

- [ ] **Document:** Serum wavetable WAV `clm` chunk format. The prior-art research file has a partial spec; expand into a standalone `docs/wavetable-format.md` so the parser can be built from a written spec, not from another implementation.
- [ ] **Survey:** existing public docs/notes on the format (Vital wiki, KVR threads, modwave docs). Cite all sources in the doc.
- [ ] **Defer (do NOT start yet):** Serum `.SerumPreset` format. Gated by seed `elixir-serum-preset-re-gate.md`. No work until trigger conditions are met.

## Stack evaluation (before crate skeleton)

- [ ] **Build a "hello sine" with `nih-plug` + `vizia`** as a throwaway spike. Goal: confirm plugin loads in a real DAW (Reaper / Bitwig CLAP), confirm vizia renders a custom-drawn widget at 60fps. Estimated 1–2 days. If vizia struggles with custom rendering or the toolchain is rough, fall back to `iced` + `iced_audio` (also evaluated in this spike).
- [ ] **Survey:** `fundsp` and `dasp` for DSP primitives we can pull in vs. write ourselves. Note their licenses and quality bar.
- [ ] **Survey:** ODDSound MTS-ESP integration in Rust. Library is BSD-licensed but written in C — confirm Rust binding strategy (`bindgen` vs hand-written FFI).

## Architecture spike (before design doc finalization)

- [ ] **Sketch:** voice graph data structure. Per-voice mod evaluation order: per-sample vs. per-block, how the mod matrix is wired, where SIMD applies. One page of pseudocode is enough.
- [ ] **Sketch:** `Engine` trait that accommodates all five Serum engine types (Wavetable, Multisample, Sample, Granular, Spectral). Confirm trait signature doesn't break when engines beyond WT are added later.
- [ ] **Sketch:** 3-bus FX graph topology (parallel sub-chains, splitter modules). Confirm whether existing contrapunk `Chain` abstraction (`src/chain/chain.rs`) can host this or whether Elixir needs its own internal FX graph.

## Contrapunk integration audit

- [ ] **Read:** `src/chain/block.rs` (lines 35–67) — `AudioBlock` trait, the integration point.
- [ ] **Read:** `src/synth/voice.rs` — existing 8-voice polyphonic synth as a reference pattern. Understand how it gets MIDI events and how its parameters are exposed via Tauri commands.
- [ ] **Read:** `src/harmony/engine.rs` (lines 30–100) — public API of HarmonyEngine. Identify what state we'd need to expose to Elixir as harmony-aware mod sources.
- [ ] **Decide:** Tauri command naming conventions for Elixir parameters (likely `elixir_*` prefix — confirm with existing `set_synth_*` patterns).

## Outcome

When all boxes are checked, the design doc can be written with confidence and the engine crate skeleton can be created without rework. Estimated total: 1–2 weeks of part-time prep.
