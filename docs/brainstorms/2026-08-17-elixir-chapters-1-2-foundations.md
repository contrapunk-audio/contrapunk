---
date: 2026-08-17
topic: elixir-chapters-1-2-foundations
---

# Elixir Chapters 1 and 2 Foundations

## What We Are Building

Elixir becomes a playable companion to the first two Wavetable Synthesis chapters. It will turn Chapter 1 harmonic construction and Chapter 2 phase, nonlinear interaction, articulation, and continuous pitch into production instrument controls.

The current 16-voice, role-aware, fixed-sine engine remains the base. Every Contrapunk function gets its own six-partial harmonic recipe and articulation. A constrained second sine operator provides additive phase summation and ring multiplication. Existing Slide remains the only portamento system. A fixed vibrato route and ADSR provide independent pitch and amplitude trajectories.

## Signal Path

```text
note + exact tuning
  -> existing Slide
  -> pitch bend + fixed vibrato
  -> six-partial harmonic oscillator A
  -> optional sine oscillator B
  -> A only / Add / Ring
  -> ADSR x velocity x expression
  -> role gain
  -> master gain
```

## Chapter 1 Surface

- Six harmonic amplitudes and six phase offsets per role.
- Sine, Three harmonics, Odd only, Saw-like, Dark, and Custom recipes using the exact published coefficients.
- Harmonic components at `k * f0`; components at or above Nyquist do not render.
- A centered dual scope showing oscillator inputs and the settled final oscillator output on one shared amplitude scale, plus a one-sided spectrum.
- Independent patches for Input, Harmony, Canon, and Counterpoint so timbre can organize an arrangement.

## Chapter 2 Surface

- One secondary sine operator with semitone offset, fine cents, phase, and level.
- Three interaction modes: Primary only, Add, and Ring.
- Add mode preserves phase-dependent reinforcement and cancellation.
- Ring mode creates sum and difference components without a general modulation matrix.
- Per-role ADSR with sustain represented as a level, velocity sensitivity, and expression sensitivity.
- Per-role vibrato depth and rate with a direct pitch route.
- Existing Slide settings remain authoritative for glide, trigger, and curve behavior.
- Pitch bend, channel pressure or CC11, mod wheel, velocity, and sustain map to the appropriate fixed destinations.

## Factory Demonstrations

Recording-ready states will cover:

1. One note through all five Chapter 1 recipes.
2. One phrase with fixed pitch and changing role colour.
3. Equal sine operators at 0, 90, and 180 degrees.
4. Add versus Ring at a known interval.
5. Passive-style ring-down versus maintained sustain.
6. The same phrase with continuous and detached articulation.
7. Glide alone, vibrato alone, and both together.

These examples will be deterministic and named so they can later drive website video scripts. This change does not record or publish media.

## Key Decisions

- Textbook provenance stays in documentation. The product surface uses synthesizer language, factory-patch names, and no chapter badges.
- Range controls update while dragged, and the central 48 kHz reference scope displays source A, source B, and their settled output on a shared scale.
- Role-scoped patches, not one global patch: Contrapunk is an ensemble and Chapter 1 explicitly uses timbre as musical identity.
- Six partials, not an arbitrary large additive bank: six matches the published chapter and is sufficient for the foundation.
- No LC component knobs: ring-down and maintained oscillation map to articulation presets, while the physical circuit remains a teaching model.
- No spatial theremin controller: keyboard, MIDI bend, pressure, expression, and Slide provide the Chapter 2 control concepts.
- No general LFO or modulation matrix: hard-wired vibrato is the only new modulation route before Chapter 4.
- No filters, spectral morphing, wavetable editing, or new FX: those depend on concepts introduced later.
- Do not resurrect the removed all-at-once Elixir engine. Extend the current small realtime-safe core.

## Compatibility

Preset schema v4 stores four role patches. Schema v3 migrates to the exact current sound: sine recipe, operator B off, five-millisecond attack and release, sustain level one, vibrato off, and preserved master and role gains.

Core processing remains allocation-free, lock-free, finite, and host-neutral. Desktop, browser AudioWorklet, Contrapunk plugin, and the independent Elixir plugin consume the same engine model.

## Definition of Done

- Harmonic, phase-sum, ring-sideband, envelope, vibrato, migration, and phase-continuity tests pass.
- Parameter edits are smoothed and cannot create non-finite output.
- Desktop, WASM, and plugin wrappers expose the same saved sound.
- The Elixir UI lets a musician hear and inspect every included concept without reading an engineering panel first.
- The repository is clean and every completed change is committed on a feature branch.
