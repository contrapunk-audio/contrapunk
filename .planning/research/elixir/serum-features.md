# Serum — Feature Inventory (research summary)

**Status:** Summary captured from a research-agent run on 2026-04-28. The full long-form output was not preserved on disk; this file holds the agent's distilled findings. **Re-run a deeper research pass before locking the design doc** if more granular detail is needed (manual deep-dive, exhaustive parameter list, etc.).

## Top design-critical findings

### 1. Serum is a five-engine hybrid, not a wavetable synth

Each of OSC A / OSC B / OSC C independently selects between:
- **Wavetable** (the classic Serum engine)
- **Multisample** (SFZ-style)
- **Sample** (single-sample playback)
- **Granular**
- **Spectral** (real-time STFT resynthesis, partial tracking, transient detection, PNG-image-as-spectrum import)

**Architectural implication for Elixir:** the engine layer must abstract over an `Engine` trait from day one. The Spectral engine alone is a significant DSP undertaking (estimated 30–50% of total DSP effort).

### 2. Preset format change — `.SerumPreset`, not `.fxp`

- Header: XferJson (trivially parseable).
- Body: zlib-compressed undocumented binary blob.
- **Not publicly reverse-engineered.**
- Serum 1 `.fxp` is also only partially documented.
- Wavetable format (`.wav` + `clm ` ASCII chunk) is **fully open and reimplementable** — a free compatibility win.

### 3. Modulation scope is larger than headlines

- **10 LFOs** (with Path mode producing dual X/Y outputs → ~20 effective streams).
- **4 envelopes.**
- **8 macros.**
- Per-mod-slot remap curves with curve-preset saving.
- Closer in capability to Vital / Phase Plant than to Serum 1.

(Conflict in sources: gearnews says 6 LFOs; Sonic Weaponry / Databroth / EDMProd / Star Samples all say 10. Going with **10**.)

### 4. FX is a 3-bus parallel graph

- Three independent FX buses, multiple instances allowed.
- Splitter modules: L/H, L/M/H, M/S.
- This is a fundamentally different graph topology from Serum 1's single linear chain.
- **Architectural implication:** Elixir's FX module needs a graph engine, not a list — must support arbitrary parallel sub-chains from day one.

### 5. MPE and microtuning are weaponizable gaps

- Serum's own MPE is reportedly still maturing post-launch.
- **MTS-ESP support is unconfirmed** in any public source consulted.
- Dropping in ODDSound's BSD-licensed MTS-ESP library plus a properly-designed MPE voice allocator from day one would let Elixir leapfrog Serum on expression and microtonal use cases.

## Open follow-ups

- Get the Serum manual when next available and produce an exhaustive parameter table.
- Confirm filter model list (LP/HP/BP/notch/comb/formant/MS-20-style/OB-style/etc.).
- Confirm exact FX module list and routing semantics.
- Capture Serum's UI panel layout (high-level prose, no copyrighted imagery).
- Verify polyphony cap, mono/legato/glide modes, microtuning support, MPE modes.

## Sources consulted (per agent report)

- xferrecords.com (official Serum page)
- Sonic Weaponry overview
- Databroth feature breakdown
- EDMProd review
- Star Samples writeup
- gearnews announcement (conflicts noted above)

A subsequent research pass should cite each claim against a source URL.
