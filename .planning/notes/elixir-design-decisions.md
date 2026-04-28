---
title: Elixir — load-bearing design decisions
date: 2026-04-28
context: Pre-design-doc Socratic exploration outcomes for the Elixir wavetable synth (open-source Serum clone in Rust, living inside contrapunk).
---

# Elixir — load-bearing design decisions

Captured during `/gsd-explore` on 2026-04-28, after two parallel research streams landed:
- `.planning/research/elixir/serum-features.md` — Serum feature inventory
- `.planning/research/elixir/oss-prior-art.md` — OSS wavetable prior art + Rust plugin ecosystem

These six decisions are the load-bearing constraints that the design doc must respect. Re-open them only with explicit reason; everything downstream is built on top.

## 1. License posture — Elixir MIT/Apache, contrapunk stays MIT

**Decision:** Elixir is permissive (MIT or Apache-2.0, exact choice in design doc). Contrapunk does not relicense to GPL.

**Why:** Preserves contrapunk's permissive surface for any future downstream embedding. Elixir is a crate inside contrapunk's MIT workspace; both must be license-compatible.

**Consequence (non-negotiable):** Every DSP technique must be **clean-room** — derived from papers, conference talks, public file-format specs, or BSD/MIT/Zlib/ISC code only. **No reading Vital, Vitalium, Surge XT, Helm, Odin 2, Bespoke, ZynAddSubFX, Dexed, or OB-Xd source code line-by-line.** Concepts and approaches discussed in talks/blog posts are fair; copy-paste of GPL code is not. Prior-art research file flags GPL boundaries.

**How to apply:** When implementing DSP, cite the paper/talk/spec the implementation derives from in code comments. If a feature has no permissive prior art, it's clean-room from first principles or paper.

## 2. Doc shape — architecture-complete, phased build

**Decision:** Design doc specifies *every* Serum feature at the architecture level on day one. Implementation ships in phases. v1.0 is partial; subsequent versions grow into parity.

**Why:** "All features from day one" + solo + clean-room is mathematically incompatible. Architecture-complete keeps the engine extensible (so we don't paint ourselves into a corner) while implementation is realistically staged.

**How to apply:** Doc has a complete "Feature Architecture" section covering all 5 engines, 3-bus FX graph, full mod matrix, etc. A separate "Implementation Roadmap" section sequences delivery. Code structure must accommodate every future feature without major rewrite — e.g. `Engine` trait with WT impl shipped first, Spectral/Granular/Sample/Multisample as later impls of the same trait.

## 3. "Better than Serum" axes — four named ambitions

**Decision:** Elixir aims to *exceed* Serum on these four axes specifically. Other axes match Serum baseline, no more.

1. **Audio quality + performance** — Lower aliasing, better filters, lower CPU per voice via Rust + SIMD, higher polyphony cap. Doc commits to specific quality/CPU benchmarks.
2. **MPE / microtuning / MIDI 2.0 first-class** — Per-note expression as primary input model. ODDSound MTS-ESP (BSD-licensed) integrated from day one. MIDI 2.0 readiness in voice allocator. Serum's MPE is reportedly still maturing post-launch and MTS-ESP support is unconfirmed in any public source — clear lead opportunity.
3. **Open & inspectable** — Human-readable native preset format (JSON or RON), scriptable mod graph (Lua/WASM/Rhai TBD in doc), themeable UI, fully documented file formats, public stable plugin API for extensions.
4. **Tight contrapunk integration** — HarmonyEngine-aware modulation sources (scale/key as mod sources, harmony-driven preset variations, "harmonize-then-synthesize" as first-class). Direct Rust API access to contrapunk's harmony state — only possible because Elixir is a crate, not a black-box plugin.

**Why:** These four were explicitly chosen by the user; the rest of Serum's surface is "match, don't exceed." This keeps ambition focused.

**How to apply:** Each of these four axes gets a dedicated design doc section with concrete commitments (benchmarks for #1, MPE feature matrix for #2, schema for #3, mod-source list for #4).

## 4. Code structure — engine crate + two shells

**Decision:**
- `crates/elixir-engine` — Pure Rust DSP/state crate. No UI. No plugin-shell dependencies.
- Contrapunk integration: `AudioBlock` impl + Svelte UI (matches existing contrapunk synth pattern at `src/synth/voice.rs`).
- `crates/elixir-plugin` — `nih-plug` CLAP/VST3 wrapper around the same engine, with vizia UI. This is the standalone build.

**Why:** Contrapunk needs direct Rust API access for harmony-aware mod sources (decision #3.4) — a generic CLAP plugin host can't deliver that. Standalone needs a real plugin shell. One engine, two shells, two UIs serving two contexts.

**How to apply:** Engine crate is the *only* place DSP and state live. Both shells consume the same public API. UI duplication is accepted cost.

**Stack confirmed by research:**
- `nih-plug` (ISC, Robbert van der Helm) for plugin shell — has shipped multiple production plugins; not GPL-tainted.
- `vizia` for standalone GUI — recommended over `egui`/`iced` for dense Serum-class UI; uses Skia, supports custom-drawn knobs/wavetable views.
- `iced` is fallback if vizia doesn't pan out.

## 5. File-format compatibility — wavetables yes, presets gated

**Decision:**
- **Wavetables: full bidirectional read+write** for Serum's `clm`-chunk WAV format. ~100 LOC of Rust per the prior-art research. Free win.
- **Native preset format**: Elixir defines its own human-readable format (JSON or RON, decided in doc). Aligns with "open & inspectable" ambition.
- **Serum preset (`.SerumPreset`) reverse-engineering: deferred to post-MVP, gated.** Format is XferJson + zlib + undocumented binary blob; not publicly reverse-engineered. Xfer's EULA likely prohibits RE; legal posture varies by jurisdiction. See seed `elixir-serum-preset-re-gate.md` for trigger conditions and required gate before any public release.

**Why:** Wavetable compat is technically trivial and legally clean. Preset compat is the opposite on both axes. Splitting them lets Elixir ship the marketing/UX win (Serum WT library "just works") without the risk.

**How to apply:** Design doc commits to wavetable compat in v0; preset import is a placeholder labelled "post-MVP, see seed."

## 6. Anti-aliasing strategy — Surge-style first, Vital-style as quality mode

**Decision:** Mipmap pyramid built at wavetable load time (Surge XT approach) is the v1 anti-aliasing strategy. Vital-style frequency-domain harmonic storage with per-voice on-the-fly bandlimiting is documented as a future "quality mode."

**Why:** Surge approach is battle-tested, low CPU, higher memory — best fit for solo first-implementation. Vital approach has higher quality ceiling but higher per-voice CPU and is more complex to get right. Both are well-documented in talks (Tytel ADC 2021 for the Vital approach; Surge XT's `wavetable.cpp` is GPL but the technique is generic and discussed in EarLevel articles).

**How to apply:** Engine has an `AntiAliasStrategy` enum or trait so the second implementation slots in as a setting/quality mode rather than a rewrite.

---

## Open questions (tracked elsewhere, not blocking design doc)

- Exact license: MIT vs Apache-2.0 vs dual MIT/Apache — decide in doc preamble.
- Mod-graph scripting language: Lua vs WASM vs Rhai — discuss in doc.
- Native preset format: JSON vs RON — minor; discuss in doc.
- Plugin formats sequencing (CLAP-only first vs CLAP+VST3 day one) — decide when standalone shell phase is planned.

## Pointers

- Serum features: `.planning/research/elixir/serum-features.md`
- OSS prior art + stack rationale: `.planning/research/elixir/oss-prior-art.md`
- Contrapunk integration surface: see `src/chain/block.rs` (`AudioBlock` trait, lines 35–67), `src/synth/voice.rs` (existing synth as reference pattern), `src/harmony/engine.rs` (harmony source), `src-tauri/src/commands/` (Tauri command surface).
- Pre-implementation reading list: `.planning/todos/pending/elixir-prereqs.md`
- Preset RE gate: `.planning/seeds/elixir-serum-preset-re-gate.md`
