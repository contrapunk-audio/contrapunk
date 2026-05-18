# Phase 21.A6 — Status & Decisions

_Last updated: 2026-05-18_

## Completion Snapshot

| Task | Status | Notes |
|---|---|---|
| A6.1 oscillator controls scaffold | ✅ | morph/phase/unison enums + params wired through Engine → Voice → Oscillator |
| A6.2 FX family scaffold | ✅ | Chorus / Flanger / Phaser / Compressor / FdnReverb; `FX_SLOTS = 8` |
| A6.3 filter model scaffold | ✅ | `FilterKind` with DigitalSvf / Diode / Dirty / Formant / Phaser |
| A6.4 hoist filter coefficients out of hot path | ✅ | `FilterCoeffs` per-block; `FilterModel::tick_prepared` no `tanf` in audio callback |
| A6.5 golden metric tests for 12 morphs / 9 phase modes / 11 unison styles | ✅ | `all_a6_modes_are_sample_rate_invariant` locks finite + peak + DC at 44.1k and 48k |
| A6.6 expose A6 controls in elixir-standalone UI | ✅ | OSC card + filter kind/drive/gain/morph controls in FILTER card |
| A6.7 contrapunk-fx extraction decision | ✅ (this doc) | **DEFER** — see below |
| A6.8 spectral wavetable parity docs | ✅ (this doc) | **APPROXIMATION** — see below |

## A6.7 — `contrapunk-fx` extraction decision

**Decision: defer extraction. Keep Elixir FX in `crates/elixir-core/src/fx/`. Continue extracting only neutral DSP primitives into `crates/contrapunk-dsp`.**

**Rationale**

1. `elixir-core` is `no_std` + wasm-ready. The root `src/fx/{delay,reverb,...}.rs` modules pull in `Transport`, atomic-param machinery, and tempo sync built around the root chain runtime. Wrapping those for `no_std` would mean fork-or-refactor on multiple modules at once.
2. Reuse-audit verdict (see `.planning/intel/elixir/reuse-audit.md`): root FX are mature but coupled to runtime concerns Elixir's voice path doesn't need yet. Extraction is a multi-day refactor that adds risk to A-Cut without a corresponding correctness win.
3. The A5/A6 FX already share the only pieces that genuinely repeat — delay lines, allpass stages, sat curves, hadamard mixing matrices, equal-power crossfade, fractional-sample interpolation — and those now live in `contrapunk-dsp`. Drift across implementations is bounded to wrappers around shared primitives.

**Triggers to revisit**

- A-Cut ships and Elixir is the default Contrapunk synth → consolidating two FX trees becomes a maintenance win, not a setup cost.
- A new FX must be authored that already exists in `src/fx/` and can be reused as-is (e.g. tempo-synced delay parity for B-track plugin). At that point extract `contrapunk-fx` on demand, one block at a time, behind a `no_std` cfg.
- Plugin (B3) or Track C consumes both root and Elixir FX simultaneously → unify to one source then.

## A6.8 — Spectral wavetable approximation vs final FFT/IFFT parity

**Decision: A6 ships as documented scalar approximations. Full FFT/IFFT wavetable infrastructure is queued for B7 (Wavetable Editor).**

**Where the approximation lives**

`crates/elixir-core/src/osc.rs::apply_spectral_morph` derives each of the 12 morphs from the shared sine table via additive scalar math:

| Morph | Implementation |
|---|---|
| Passthrough | bypass (verbatim sine table lookup) |
| Vocode | additive 1f/2f/4f sine partials |
| FormScale | phase-modulated 1f sine (`sin(2πx + 0.45·sin(2π·3x))`) |
| HarmonicScale / InharmonicScale | weighted partial stacks |
| Smear | three-tap weighted blend of the shared sine table |
| RandomAmplitudes | fixed partial stack at h2/h3/h5/h8 |
| LowPass / HighPass | partial weighting |
| PhaseDisperse | quadratic-phase modulated sine |
| ShepardTone | logarithmically spaced partial taps |
| Skew | piecewise-powered phase warp through sine |

This is **not** the full design-doc behavior, which calls for offline FFT analysis of a wavetable frame, per-bin amplitude/phase warping, and inverse FFT regeneration of each morphed frame. The scalar form gives the same *musical* axis (timbral colour shifts under the morph slider) while keeping the audio path allocation-free and `no_std`.

**What the user gets today**

- Every morph audibly differs from passthrough at amount=1.0 (locked by `all_spectral_morphs_change_the_wave_when_full_amount` test).
- Output is bounded to `[-1, 1]` and sample-rate-invariant for finiteness + peak + bounded DC across 44.1k / 48k (locked by `all_a6_modes_are_sample_rate_invariant`).
- All 12 variants are reachable from `SpectralMorph::ALL` and exposed in the standalone UI (A6.6).

**What's missing for design-doc parity**

- True frequency-domain shaping: e.g. `HighPass` currently boosts mid harmonics rather than zeroing the fundamental.
- `ShepardTone` octave-stack does not currently re-window across pitch — a real Shepard tone requires Gaussian envelope per octave layer, which needs a multi-frame wavetable.
- `RandomAmplitudes` uses a fixed deterministic seed (good for tests) but should pull a new seed per voice or per note for the design-doc result.

**Path to full parity**

1. Land B7 wavetable editor → adds a `WaveTable` type that holds multiple time-domain frames + cached FFT/IFFT scratch.
2. Generalize `apply_spectral_morph` to operate on a `WaveTableFrame` instead of the shared `SineTable`.
3. Replace the scalar morph implementations with their FFT-domain equivalents one by one; gate behind a per-preset flag so existing presets render identically until they opt in.
4. Retire the scalar form once every preset in the wild has migrated.

Until then, callers see exactly what the controls promise — a colour change under `morph_amount` — without the synth ever allocating in the audio path or pretending the morph is the final algorithm.
