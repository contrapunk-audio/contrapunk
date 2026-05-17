# Graph Report - /tmp/elixir-graphify-input  (2026-05-18)

## Corpus Check
- Corpus is ~13,161 words - fits in a single context window. You may not need a graph.

## Summary
- 158 nodes · 261 edges · 14 communities detected
- Extraction: 85% EXTRACTED · 15% INFERRED · 0% AMBIGUOUS · INFERRED: 39 edges (avg confidence: 0.8)
- Token cost: 0 input · 0 output

## Community Hubs (Navigation)
- [[_COMMUNITY_Realtime Plumbing & Rationale|Realtime Plumbing & Rationale]]
- [[_COMMUNITY_Surfaces & Track B Rollout|Surfaces & Track B Rollout]]
- [[_COMMUNITY_Voices & Modulation|Voices & Modulation]]
- [[_COMMUNITY_Plugin Hosting & Audio Chain|Plugin Hosting & Audio Chain]]
- [[_COMMUNITY_Filter Topologies|Filter Topologies]]
- [[_COMMUNITY_Wavetable & Spectral Engine|Wavetable & Spectral Engine]]
- [[_COMMUNITY_FX Bus & Oversampling|FX Bus & Oversampling]]
- [[_COMMUNITY_Engine Framework Primitives|Engine Framework Primitives]]
- [[_COMMUNITY_Track A — Cutover|Track A — Cutover]]
- [[_COMMUNITY_Master Output Stage|Master Output Stage]]
- [[_COMMUNITY_Param Bridge (Synth↔Elixir)|Param Bridge (Synth↔Elixir)]]
- [[_COMMUNITY_SIMD Stereo Sum|SIMD Stereo Sum]]
- [[_COMMUNITY_DC Blocker|DC Blocker]]
- [[_COMMUNITY_dasp Slice Utils|dasp Slice Utils]]

## God Nodes (most connected - your core abstractions)
1. `elixir-core crate` - 15 edges
2. `Track B - Elixir standalone product` - 15 edges
3. `Phase A6 - Spectral oscillator + FX completion` - 15 edges
4. `Track A - Replacement of Contrapunk synth` - 13 edges
5. `Phase A5 - FX bus (minimum viable)` - 13 edges
6. `ReorderableEffectChain` - 12 edges
7. `Voice Filter (upstream of FX bus)` - 10 edges
8. `Modulation Matrix` - 10 edges
9. `Phase A3 - Modulation matrix v1` - 10 edges
10. `Wavetable Oscillator` - 8 edges

## Surprising Connections (you probably didn't know these)
- `Phase A6 - Spectral oscillator + FX completion` --delivers--> `Phase distortion (nine modes)`  [EXTRACTED]
  ELIXIR-PLAN.md → ELIXIR-DESIGN.md
- `Phase A2 - Polyphony + voice management` --delivers--> `AggregateVoice (SIMD-packed voices)`  [EXTRACTED]
  ELIXIR-PLAN.md → ELIXIR-DESIGN.md
- `Phase A5 - FX bus (minimum viable)` --delivers--> `Upsampler (2x)`  [EXTRACTED]
  ELIXIR-PLAN.md → ELIXIR-DESIGN.md
- `Phase A5 - FX bus (minimum viable)` --delivers--> `Decimator (3-pole IIR halfband)`  [EXTRACTED]
  ELIXIR-PLAN.md → ELIXIR-DESIGN.md
- `Phase A1 - Bare oscillator` --delivers--> `Wavetable Oscillator`  [EXTRACTED]
  ELIXIR-PLAN.md → ELIXIR-DESIGN.md

## Hyperedges (group relationships)
- **Voice signal path (oscillator → filter → upsampler → FX → decimator → master gain)** — elixir_design_wavetable_oscillator, elixir_design_voice_filter, elixir_design_upsampler, elixir_design_reorderable_effect_chain, elixir_design_decimator, elixir_design_smoothed_volume [EXTRACTED 1.00]
- **All sources of the modulation matrix** — elixir_design_envelope, elixir_design_synth_lfo, elixir_design_random_lfo, elixir_design_trigger_random, elixir_design_line_map, elixir_design_mpe_sources, elixir_design_macro_knobs, elixir_design_modulation_matrix [EXTRACTED 1.00]
- **All plugin formats supported in Track C (CLAP, VST3, AU)** — elixir_plan_clap_audio_block, elixir_plan_vst3_block, elixir_plan_au_block, contrapunk_audio_block, elixir_plan_track_c [EXTRACTED 1.00]
- **All surfaces Elixir compiles to (standalone, plugin, headless, contrapunk-embedded)** — elixir_design_standalone_target, elixir_design_plugin_host_target, elixir_design_headless_target, elixir_plan_elixir_synth_block, elixir_design_elixir_core_crate [EXTRACTED 1.00]
- **Real-time safety enforcement stack (allocation guard, arenas, lock-free queues, type-marker)** — elixir_design_assert_no_alloc, elixir_design_bumpalo, elixir_design_rtrb, elixir_design_arc_swap, elixir_design_crossbeam_channel, elixir_design_sound_engine [EXTRACTED 1.00]

## Communities

### Community 0 - "Realtime Plumbing & Rationale"
Cohesion: 0.09
Nodes (28): arc-swap, assert_no_alloc, bumpalo, crossbeam-channel, Elixir Synthesizer, elixir-core crate, enum_dispatch, flate2 (+20 more)

### Community 1 - "Surfaces & Track B Rollout"
Cohesion: 0.14
Nodes (22): cpal, elixir-host trait, Headless CLI Renderer, hound, midir, nih-plug, Plugin Host (VST3+CLAP via nih-plug), Standalone Desktop App (+14 more)

### Community 2 - "Voices & Modulation"
Cohesion: 0.14
Nodes (21): AggregateVoice (SIMD-packed voices), Multi-stage envelope (DAHDSR), LineGenerator (breakpoint curve), LineMap (line/curve mapper), Macro knobs, ModulationConnection / ModulationConnectionProcessor, Modulation Matrix, MPE/note value sources (+13 more)

### Community 3 - "Plugin Hosting & Audio Chain"
Cohesion: 0.2
Nodes (18): AudioBlock trait, src-tauri/src/audio_clock.rs, Chain (src/chain/), ChainCommand::PushBlock, src/plugin_host/ (CLAP host), AuBlock, clack-host, ClapAudioBlock (+10 more)

### Community 4 - "Filter Topologies"
Cohesion: 0.19
Nodes (16): Chorus effect, Comb / flanger filter, Digital state-variable filter, Diode filter, Dirty filter, Filter-FX (global insert), Flanger effect, Formant filter (+8 more)

### Community 5 - "Wavetable & Spectral Engine"
Cohesion: 0.14
Nodes (15): bytemuck, FFT (real-complex), OneDimLookup (LUT), Phase distortion (nine modes), rand_chacha, Rationale: No BLEP/polyBLEP - anti-aliasing exclusively via spectral band-limiting, Rationale: realfft over rustfft for real-input transforms, realfft (+7 more)

### Community 6 - "FX Bus & Oversampling"
Cohesion: 0.24
Nodes (14): Compressor effect, Decimator (3-pole IIR halfband), Delay effect, Distortion effect, EQ effect, FDN Reverb (16-line), Linkwitz-Riley crossover, MultibandCompressor (+6 more)

### Community 7 - "Engine Framework Primitives"
Cohesion: 0.22
Nodes (9): Feedback node, Operator (stateless Processor), Output (contiguous poly_float buffer), Processor (framework primitive), ProcessorRouter, Rationale: reject dyn-Processor design, Rationale: typestate builder compiles to flat Vec<Op> tape machine, SynthModule (+1 more)

### Community 8 - "Track A — Cutover"
Cohesion: 0.32
Nodes (8): LegacySynth (wrapper over old Synth), src/synth/voice.rs (legacy Contrapunk synth), Contrapunk-Default.elxprst preset, Phase A7 - Default flip + cleanup, Phase A-Cut - Cutover, Risk: .planning/ GSD milestone integration, Risk: Synth replacement changes Contrapunk tone, Track A - Replacement of Contrapunk synth

### Community 9 - "Master Output Stage"
Cohesion: 1.0
Nodes (2): PeakMeter / Clamp(-2.1, 2.1), SmoothedVolume / Master gain

### Community 10 - "Param Bridge (Synth↔Elixir)"
Cohesion: 1.0
Nodes (2): SynthParams, ElixirParams

### Community 11 - "SIMD Stereo Sum"
Cohesion: 1.0
Nodes (1): SIMD stereo sum

### Community 12 - "DC Blocker"
Cohesion: 1.0
Nodes (1): DC blocker

### Community 13 - "dasp Slice Utils"
Cohesion: 1.0
Nodes (1): dasp

## Knowledge Gaps
- **40 isolated node(s):** `Elixir Synthesizer`, `NoteHandler`, `SIMD stereo sum`, `SmoothedVolume / Master gain`, `PeakMeter / Clamp(-2.1, 2.1)` (+35 more)
  These have ≤1 connection - possible missing edges or undocumented components.
- **Thin community `Master Output Stage`** (2 nodes): `PeakMeter / Clamp(-2.1, 2.1)`, `SmoothedVolume / Master gain`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Param Bridge (Synth↔Elixir)`** (2 nodes): `SynthParams`, `ElixirParams`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `SIMD Stereo Sum`** (1 nodes): `SIMD stereo sum`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `DC Blocker`** (1 nodes): `DC blocker`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `dasp Slice Utils`** (1 nodes): `dasp`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `Track A - Replacement of Contrapunk synth` connect `Track A — Cutover` to `Realtime Plumbing & Rationale`, `Surfaces & Track B Rollout`, `Voices & Modulation`, `Plugin Hosting & Audio Chain`, `Filter Topologies`, `FX Bus & Oversampling`?**
  _High betweenness centrality (0.310) - this node is a cross-community bridge._
- **Why does `elixir-core crate` connect `Realtime Plumbing & Rationale` to `Track A — Cutover`, `Wavetable & Spectral Engine`?**
  _High betweenness centrality (0.258) - this node is a cross-community bridge._
- **Why does `Phase A6 - Spectral oscillator + FX completion` connect `Filter Topologies` to `Track A — Cutover`, `Surfaces & Track B Rollout`, `Wavetable & Spectral Engine`, `FX Bus & Oversampling`?**
  _High betweenness centrality (0.148) - this node is a cross-community bridge._
- **What connects `Elixir Synthesizer`, `NoteHandler`, `SIMD stereo sum` to the rest of the system?**
  _40 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `Realtime Plumbing & Rationale` be split into smaller, more focused modules?**
  _Cohesion score 0.09 - nodes in this community are weakly interconnected._
- **Should `Surfaces & Track B Rollout` be split into smaller, more focused modules?**
  _Cohesion score 0.14 - nodes in this community are weakly interconnected._
- **Should `Voices & Modulation` be split into smaller, more focused modules?**
  _Cohesion score 0.14 - nodes in this community are weakly interconnected._