# OSS Wavetable Synth Prior Art + Rust Plugin Ecosystem (research summary)

**Status:** Summary captured from a research-agent run on 2026-04-28. The full long-form output was not preserved on disk; this file holds the agent's distilled findings. Re-run a deeper research pass if more granular per-project detail is needed.

## Top design-critical takeaways

### 1. License reality — every meaningful prior-art wavetable synth is GPLv3

GPLv3 (or GPLv3-only): **Vital, Vitalium, Surge XT, Helm, Odin 2, Bespoke, ZynAddSubFX, Dexed, OB-Xd.**

For a permissively-licensed Elixir, this means:
- Read these projects only at the *concept* level. Do not copy code.
- Clean-room derivation paths:
  - Watch Tytel's ADC 2021 talk (concepts, not code).
  - Read papers (Välimäki/Huovilainen on antialiasing oscillators).
  - Read EarLevel articles (mipmap-pyramid wavetable construction).
  - Implement against published file-format specs.
- The Serum `clm`-chunk WAV format is openly documented; reimplementing the parser is permitted.

### 2. Two viable anti-aliasing strategies — pick deliberately

- **Vital approach:** frequency-domain harmonic storage + per-voice on-the-fly bandlimiting (Tytel ADC 2021).
  - Best quality.
  - Higher per-voice CPU.
- **Surge XT approach:** mipmap pyramids computed at load time.
  - Battle-tested.
  - Low CPU.
  - Higher memory.

**Recommendation:** ship Surge-style mipmap first; add Tytel-style as a quality mode later. (Confirmed in the design decisions note.)

### 3. `nih-plug` is the right plugin shell

- Author: Robbert van der Helm.
- License: ISC (permissive — license-compatible with Elixir).
- Exports CLAP + VST3.
- Has shipped multiple production plugins (`Spectral Compressor`, `Diopser`, etc. in its `plugins/` directory).
- Sample-accurate automation.
- Provides adapters for **vizia / iced / egui** GUI frameworks.

### 4. GUI — `vizia` is the strongest pick for Serum-class dense UI

- Uses Skia for rendering.
- Supports custom-drawn knobs and wavetable views.
- Recommended over egui in nih-plug docs for plugin UIs.
- **Iced** is the fallback (more mature ecosystem; `iced_audio` widget pack exists).
- **egui** is fine for a debug overlay but not the production UI.

### 5. Serum WAV format is an implementable de-facto standard

- 32-bit float WAV.
- 2048-sample frames per cycle.
- ASCII `clm ` chunk encodes metadata: `<!>2048 BC000000 vendor` (or similar — verify exact byte pattern when implementing).
- Read by: Vital, Pigments, Korg modwave, Surge, Ableton Wavetable.
- Implementation: ~100 lines of Rust to parse and serialize.

**Decision implication:** implement this importer/exporter first. Skip Serum `.fxp` and `.SerumPreset` import (proprietary, not publicly RE'd — see seed `elixir-serum-preset-re-gate.md`).

## Single must-do before writing any oscillator code

Watch and take notes on:
- **Matt Tytel — "Practical Guide to Optimized High-Quality Wavetable Oscillators" (ADC 2021)**
  - Video: https://www.youtube.com/watch?v=qlinVx60778
  - Slides: https://data.audio.dev/talks/2021/guide-to-optimized-wavetable-oscillators/slides.pdf

For Surge-style mipmap implementation, read EarLevel's wavetable series:
- https://www.earlevel.com/main/2012/05/04/a-wavetable-oscillator%E2%80%94introduction/

## Stack — recommended set

| Layer | Pick | License | Notes |
|---|---|---|---|
| Plugin shell (standalone) | `nih-plug` | ISC | CLAP + VST3 export |
| GUI (standalone) | `vizia` | MPL-2.0 | Custom knob/WT rendering |
| GUI fallback | `iced` + `iced_audio` | MIT | If vizia ergonomics fail in spike |
| DSP primitives | `fundsp` / `dasp` | MIT/Apache | Evaluate; may roll our own |
| Audio I/O (standalone host) | `cpal` | Apache/MIT | Already used by contrapunk |
| Microtuning | ODDSound MTS-ESP | BSD | C library, FFI via bindgen |

## Open follow-ups

- Spike `nih-plug` + `vizia` "hello sine" plugin in Reaper / Bitwig CLAP.
- Survey `fundsp` and `dasp` for what we can pull in vs. write ourselves.
- Confirm Rust binding strategy for ODDSound MTS-ESP (`bindgen` vs hand-written FFI).
- Identify the closest publicly-permissively-licensed reference implementation for each Serum feature area (oscillators, filters, FX, mod matrix) — most will be from-paper, no reference impl.
