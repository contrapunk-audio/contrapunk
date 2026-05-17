# Constraints Intel

Technical constraints synthesized from SPEC documents. Each entry has a type (api-contract | schema | nfr | protocol | crate-stack | structural | threading), source, and content block.

---

## ELIX-CON-01 — Synthesizer architecture (5 subsystems)

- **Type:** structural
- **Source:** `ELIXIR-DESIGN.md` §1
- **Content:** Elixir is a polyphonic wavetable synthesizer with frequency-domain warping. Each wavetable frame is editable in the spectral domain. An oscillator can morph the spectrum at audio rate through one of twelve operations (vocode, formant-shift, smear, harmonic-scale, phase-disperse, shepard, skew, etc.) before the IFFT writes time-domain samples. Five subsystems:
  1. Engine + voicing + framework — processing graph, voice allocator, module abstractions
  2. Wavetable oscillator + spectral engine — FFT-based warping
  3. Filters + effects — eight filter topologies + reorderable eight-slot FX bus
  4. Modulators + modulation matrix
  5. Build targets, hosting, presets, concurrency

## ELIX-CON-02 — Engine signal flow

- **Type:** api-contract
- **Source:** `ELIXIR-DESIGN.md` §3 engine architecture
- **Content:**
  ```
  MIDI in → NoteHandler → VoiceHandler → per-voice graph (osc / env / filter / mods)
         → SIMD stereo sum → Upsampler (2x) → ReorderableEffectChain
         → Decimator (3-pole IIR halfband) → stereo encode → SmoothedVolume
         → PeakMeter → Clamp(-2.1, 2.1) → output buffer
  ```
  Top-level `SoundEngine` owns a `ProcessorRouter`. Oscillators, envelopes, voice filters, and per-voice modulators run inside the voice graph. Global modulators (master LFO, BPM clock), FX chain, and master gain run outside. Oversampling default 2x. Internal rate held roughly constant against host: at 44.1kHz the requested oversample applied verbatim; at 88.2/176.4 it is divided down.

## ELIX-CON-03 — Voice management

- **Type:** structural
- **Source:** `ELIXIR-DESIGN.md` §3 voice management
- **Content:** A `VoiceHandler` owns pre-allocated `Voice` slots (`MAX_POLYPHONY + PARALLEL_VOICES`). Voices packed into `AggregateVoice` of `PARALLEL_VOICES = SIMD_WIDTH / 2` lanes. Allocation walks: free-parallel-slot → free voice → released → sustained → held → triggering. Voice priority: `Newest / Oldest / Highest / Lowest / RoundRobin`. `voice_override`: `Kill` or `Steal`. Sustain pedal moves voices to `Sustained`. Sostenuto latches at pedal-down. Per-block voice events carry sample offset for sample-accurate note-on within the block.

## ELIX-CON-04 — Framework primitives

- **Type:** api-contract
- **Source:** `ELIXIR-DESIGN.md` §3 framework primitives
- **Content:** Five abstractions: `Processor`, `Output` (with trigger side-channel), `Value`, `Operator`, `ProcessorRouter`. Reject `Box<dyn Processor>` in inner loop. Use enum-dispatch over all node types OR `Box<dyn Processor>` only for outer effect-chain slots (block-rate dispatch acceptable) OR typestate-builder compiling to flat `Vec<Op>`.

## ELIX-CON-05 — Block processing model

- **Type:** api-contract
- **Source:** `ELIXIR-DESIGN.md` §3 block processing
- **Content:** `const MAX_BLOCK: usize = 512` (or 1024). Stack-friendly `[f32x4; MAX_BLOCK / 4]` buffers held in per-engine arena. Sub-block events: event list sorted by `sample_offset`, replayed via inner `for sample in start..end` loop per region. Trigger side-channel for sample-accurate events. Control-rate modulators run once per block (`num_samples = 1`); audio-rate modulators run at full block.

## ELIX-CON-06 — Threading model (lock-free)

- **Type:** threading
- **Source:** `ELIXIR-DESIGN.md` §3 threading, §7 concurrency
- **Content:** Exactly two threads of concern: **audio thread** (never allocates, never locks) and **UI/control thread**. Communication patterns:
  - Scalar params: `AtomicU32` storing `f32::to_bits()` with relaxed reads
  - Event streams: `crossbeam::queue::ArrayQueue<NoteEvent>` or `rtrb` SPSC ringbuffer
  - Meters / status outputs (audio → UI): `triple_buffer` or `arc-swap::ArcSwap<MeterSnapshot>`
  - Full preset / wavetable swaps: `arc-swap::ArcSwap` (eliminates pause-processing dropout)
  - Mod-matrix mutations: SPSC ring buffer (`rtrb`) carrying `MatrixEdit { route_id, src, dst, op }`
  Avoid `Mutex` and `RwLock` anywhere on the audio path. If shared mutable state unavoidable, use `arc-swap` or double-buffered `[T; 2]` with atomic generation counter.

## ELIX-CON-07 — Real-time safety enforcement (4 layers)

- **Type:** nfr
- **Source:** `ELIXIR-DESIGN.md` §7 real-time safety
- **Content:** Four enforcement layers:
  1. **Allocation guard.** Wrap audio callback in `assert_no_alloc::assert_no_alloc(|| { ... })` under `#[cfg(debug_assertions)]`.
  2. **Type-level safety.** Marker trait `RealtimeSafe`; audio-thread entry point generic over `R: RealtimeSafe` so borrow checker rejects `Vec`, `String`, `HashMap`, `std::sync::*`.
  3. **Two-arena pattern.** Pre-allocate two `bumpalo::Bump` arenas, alternate per block, reset at block start.
  4. **Disk persistence physically impossible.** Route "save to disk" through `crossbeam-channel` to dedicated config thread — engine doesn't link to `std::fs`.

## ELIX-CON-08 — Wavetable representation

- **Type:** schema
- **Source:** `ELIXIR-DESIGN.md` §4 wavetable representation
- **Content:**
  - `WAVEFORM_BITS = 11`, `WAVEFORM_SIZE = 2048` samples per frame
  - Frame count typically ≤ 256
  - `WaveFrame` (editable): `time_domain[2 * WAVEFORM_SIZE]` (double-length scratch), `frequency_domain[WAVEFORM_SIZE]` of `Complex<f32>`, freq ratio + SR
  - `Wavetable` (audio-side, per frame): raw `f32[WAVEFORM_SIZE]` time-domain + three SIMD-packed parallel arrays — `frequency_amplitudes`, `normalized_frequencies`, `phases`
  - Harmonics: `NUM_HARMONICS = WAVEFORM_SIZE/2 + 1 = 1025` bins
  - Mip-mapping implicit in spectral pipeline (lazy, double-buffered Fourier output frames `fourier_frames1` / `fourier_frames2`)
  - Frame interpolation: Catmull-Rom in time domain (static morphs), per-bin linear in frequency (spectral "skew" morph)
  - Hot-swap: `arc-swap::ArcSwap<WavetableData>` (wait-free `load_full`)
  - Phase unwrapping in `post_process()`: bins below amplitude floor inherit linearly-interpolated complex direction from neighbours

## ELIX-CON-09 — Oscillator playback

- **Type:** api-contract
- **Source:** `ELIXIR-DESIGN.md` §4 oscillator playback
- **Content:**
  - Phase: 32-bit unsigned (`poly_int`), top 11 bits = sample index, bottom 21 (`INTERMEDIATE_BITS`) = interpolation fraction
  - `phase += to_int(phase_inc_buffer[i] * phase_inc_mult)` per sample
  - NO BLEP/polyBLEP — anti-aliasing exclusively via band-limited synthesis at spectral stage
  - Four-point Catmull-Rom between adjacent waveform samples
  - Outer Catmull blend between adjacent frames via `interpolate_multiple_buffers` (4x4 SIMD lane matrix)
  - Unison: up to `MAX_UNISON = 16` detuned voices, 8 SIMD pairs (`NUM_POLY_PHASE`)
  - Detune: `cents_to_ratio(t^power * range_cents)`, 11 stack styles via `STACK_MULTIPLIERS`
  - 9 phase-distortion modes: `quantize`, `bend`, `squeeze`, `sync`, `pulse_width`, `fm_oscillator_a/b/sample`, `rm_oscillator_a/b/sample`
  - FM phase offset scaled by `FM_PHASE_MULT = PHASE_MULT / 8`, capped at `MAX_FM_MODULATION = 48`

## ELIX-CON-10 — Spectral morph operations (12 variants)

- **Type:** api-contract
- **Source:** `ELIXIR-DESIGN.md` §4 spectral operations
- **Content:** Twelve operations per voice, selected from: `passthrough`, `vocode`, `form_scale`, `harmonic_scale`, `inharmonic_scale`, `smear`, `random_amplitudes`, `low_pass`, `high_pass`, `phase_disperse`, `shepard_tone`, `skew`. Real-FFT length 2048 with no explicit window (rectangular = correct for periodic signal). `realfft` crate gives forward+inverse plans with allocation-free `process(&mut input, &mut output, &mut scratch)` after first call. Wrap-pad trick: `SIZE` samples copied head/tail per IFFT pass keeps Catmull-Rom playback branch-free.

## ELIX-CON-11 — Filter topologies (8 models)

- **Type:** api-contract
- **Source:** `ELIXIR-DESIGN.md` §5 filter models
- **Content:** Eight filter models, common base publishing `(cutoff_midi, resonance, drive, gain, style, pass_blend, interp_x, interp_y, transpose, spread)` plus per-block `setup_filter` and per-sample `tick`:
  1. Analog Sallen-Key (12/24 dB)
  2. Ladder (12/24 dB)
  3. Diode (12/24 dB)
  4. "Dirty" (12/24 dB)
  5. Digital state-variable (Zavalishin TPT SVF)
  6. Comb / flanger
  7. Formant (vowel-grid bilinear)
  8. Phaser-filter (12 cascaded all-pass)
  Plus structural helpers: one-zero DC blocker, Linkwitz-Riley 4th-order LP/HP crossover, FIR/IIR half-band decimators + upsampler. Coefficient `g(f)` computed via 2048-entry cubic-interpolated LUT (NOT inline `tan()`).

## ELIX-CON-12 — Effects chain (reorderable 8-slot)

- **Type:** api-contract
- **Source:** `ELIXIR-DESIGN.md` §5 effects chain
- **Content:** `ReorderableEffectChain` runs eight effect modules; order is an automated parameter. Each effect implements `Processor::process_with_input`:
  1. Chorus — multi-tap modulated delay
  2. Flanger — same delay-line core as comb-filter
  3. Phaser — 12-stage all-pass cascade with internal LFO
  4. Distortion — six nonlinearities (soft-clip, hard-clip, linear-fold, sin-fold, bit-crush, downsample)
  5. EQ — Linkwitz-Riley crossover + parametric SVF bands
  6. Compressor — RMS-envelope, separate upper+lower threshold/ratio; `MultibandCompressor` adds 3-band LR splitter
  7. Reverb — 16-line FDN (FDN-8 fallback for WASM)
  8. Delay — modes: mono / stereo / ping-pong / mid-ping-pong / clamped-dampened / clamped-unfiltered / unclamped-unfiltered
  Plus Filter-FX exposing any of the eight filter models as a global insert.
  Wet/dry: equal-power-fade `wet = sin(π/2 * w)`, `dry = cos(π/2 * w)`, per-block-smoothed. Effects are **serial** (no send-bus architecture).

## ELIX-CON-13 — Modulation matrix (sparse, SoA)

- **Type:** api-contract
- **Source:** `ELIXIR-DESIGN.md` §6 modulation matrix
- **Content:** Matrix is fixed bank of `MAX_MODULATION_CONNECTIONS` slots. Each slot a `ModulationConnectionProcessor` with inputs `MOD_INPUT`, `MOD_AMOUNT`, `MOD_POWER`, `RESET` and outputs `MOD_OUTPUT`, `MOD_PRE_SCALE`, `MOD_SOURCE`. Persistent flags: `bipolar`, `stereo`, `bypass`. `LineGenerator` for optional per-route response curve. Storage:
  ```rust
  struct ModRoutes {
      src:    Vec<ModSrcId>,
      dst:    Vec<ModDstId>,
      amount: Vec<Smoothed<f32>>,
      power:  Vec<Smoothed<f32>>,
      curve:  Vec<Option<CurveId>>,
      flags:  Vec<RouteFlags>,
  }
  ```
  Typed IDs (newtype `u16`). Destinations are `VariableAdd` summers (additive combine). Inner-loop variants: linear / morphed / remapped / both. Modulation-of-modulation: amount and power inputs are themselves modulatable.

## ELIX-CON-14 — Modulation source types (7 kinds)

- **Type:** api-contract
- **Source:** `ELIXIR-DESIGN.md` §6 modulation source types
- **Content:** Seven mod source kinds, each its own processor with per-voice state:
  1. Multi-stage envelopes (`Envelope`) — six-stage DAHDSR + forced kill tail; signed power-curve per segment
  2. Custom-waveform LFO (`SynthLfo`) — phase-tracking, breakpoint curve via Catmull-Rom, 5 run modes (Trigger/Sync/Envelope/SustainEnvelope/LoopPoint/LoopHold)
  3. Stochastic LFO (`RandomLfo`) — 4 flavors: Perlin / S&H / sine-interp / 3-state Lorenz attractor
  4. One-shot trigger random (`TriggerRandom`)
  5. Line/curve mapper (`LineMap`) — for velocity/aftertouch/mod-wheel/note/slide/lift/pitch-wheel response curves
  6. MPE/note value sources — `velocity`, `aftertouch`, `slide`, `lift`, `mod_wheel`, `pitch_wheel`, `note`, `note_in_octave`, `stereo`, `random`
  7. Macro knobs — mono control-rate `Value` processors

## ELIX-CON-15 — Surface targets (4 binaries from one core)

- **Type:** structural
- **Source:** `ELIXIR-DESIGN.md` §7 surface targets, `ELIXIR-PLAN.md` §1
- **Content:** Elixir compiles to four binaries from one shared `SynthBase`:
  - Plugin (VST3 + CLAP via nih-plug)
  - Standalone desktop (cpal + midir + egui)
  - Headless CLI renderer (argv + hound WAV)
  - AU plugin (macOS only, via nih-plug or wrapper)
  Crates: `elixir-core` (no_std-friendly, no host code, compiles to wasm32), `elixir-standalone`, `elixir-plugin`, `elixir-headless`.

## ELIX-CON-16 — MIDI / MPE / CC-learn / tuning

- **Type:** api-contract
- **Source:** `ELIXIR-DESIGN.md` §7 MIDI handling
- **Content:** `MidiManager` consumes host MIDI buffers (plugin path) and OS MIDI inputs (standalone path). Routes CCs specifically: mod wheel (1), sustain (64), sostenuto (66), slide / LSB-slide (74 / 106), LSB-pressure (102), bank/folder select (0 / 32), panic CCs. CC 74 + channel pressure use `(msb << 7) | lsb` for 14-bit precision. MPE first-class: `MPEZoneLayout` snoops controller messages; engine exposes zoned setters. CC learn map persisted via `crossbeam-channel` to config thread (not directly from audio thread). Tuning: `.scl`, `.kbm`, `.tun` files loaded into 256-entry MIDI-to-frequency table. Optional `mts-esp-rs` behind a feature flag.

## ELIX-CON-17 — Preset format (JSON + base64 PCM)

- **Type:** schema
- **Source:** `ELIXIR-DESIGN.md` §7 preset format
- **Content:** UTF-8 JSON files. Top-level: `synth_version`, `preset_name`, `author`, `comments`, `preset_style` (enum: Bass/Lead/Keys/Pad/Percussion/Sequence/Experimental/SFX/Template), per-macro names, nested `settings` with all flat params, modulation connections array (`{source, destination, line_mapping}`), per-oscillator wavetable definitions, LFO line-generator shapes array, sample. Wavetable + sample buffers as base64 int16 PCM embedded in JSON. Plugin state = same JSON + tuning state, gzipped via `flate2` for binary host chunks. Version migration: sequential ladder rewriting JSON tree forward. Search/tag sidecar `index.json`.

## ELIX-CON-18 — Crate stack (locked dependency set)

- **Type:** crate-stack
- **Source:** `ELIXIR-DESIGN.md` §2
- **Content:**
  - Plugin host: `nih-plug`
  - Standalone audio: `cpal`
  - Standalone MIDI: `midir`
  - SIMD: `std::simd` (nightly) with `wide` as stable fallback (default to `wide`; gate `std::simd` behind `cfg(feature = "nightly-simd")`)
  - FFT: `realfft` (wraps `rustfft`)
  - Resampling: `rubato`; or hand-rolled half-band IIR
  - Lock-free shared state: `arc-swap`, `rtrb` (SPSC), `crossbeam-channel` (multi-producer)
  - Realtime allocation guard: `assert_no_alloc` (debug-only)
  - Bump arenas: `bumpalo`
  - Serialization: `serde` + `serde_json` + `base64` + `flate2`
  - Deterministic RNG: `rand_chacha` (ChaCha8)
  - WAV writing: `hound`
  - Tuning parser: hand-rolled (~300 LOC); optional `mts-esp-rs` behind feature flag
  - Argv: `clap`
  - Static dispatch: `enum_dispatch`

## ELIX-CON-19 — Workspace layout (new crates inside contrapunk)

- **Type:** structural
- **Source:** `ELIXIR-PLAN.md` §1
- **Content:**
  ```
  crates/
    elixir-core/         # DSP engine. no_std-friendly. wasm32-compatible. NO cpal/midir/nih-plug.
                         #   Depends: rustfft/realfft, wide (SIMD), serde, serde_json, base64, bytemuck, arc-swap
    elixir-standalone/   # Binary. cpal + midir + clap argv + egui UI in same process. Desktop only.
    elixir-plugin/       # nih-plug cdylib. VST3 + CLAP + AU + standalone wrap. Links elixir-core, embeds egui.
    elixir-headless/     # Binary. argv + MIDI file → WAV via hound. Links elixir-core, hound, clap.
  ```
  Existing tree touched: `src/synth/` replaced at A-Cut; `src/chain/` extended (new `ElixirSynthBlock` impls `AudioBlock`); `src/plugin_host/` extended for Track C; `src-tauri/src/audio_clock.rs` rewired; `src-tauri/src/commands/synth.rs` adapted; `plugin/src/lib.rs` same swap; `ui/src/lib/adapter/` picks up new param keys; `.github/workflows/` adds elixir release flows.

## ELIX-CON-20 — Cutover contract (legacy synth ↔ ElixirSynthBlock)

- **Type:** api-contract
- **Source:** `ELIXIR-PLAN.md` §2 cutover boundary
- **Content:** Surface-by-surface mapping between today's synth and Elixir:
  | Concept | Today | After Elixir |
  |---|---|---|
  | Audio block | `crate::synth::Synth` | `elixir_core::Engine` wrapped by `ElixirSynthBlock: AudioBlock` |
  | Event input | `Receiver<SynthEvent>` (NoteOn/NoteOff/AllNotesOff) | Same MPSC, mapped to `elixir_core::NoteEvent` |
  | Params | `Arc<SynthParams>` with atomics | `Arc<ElixirParams>` with same getter/setter shape, typed `ParamId` keys |
  | Render | `Synth::render(buf, channels)` per cpal callback | `ElixirSynthBlock::process(buf, channels)` |

## ELIX-CON-21 — Testing strategy

- **Type:** nfr
- **Source:** `ELIXIR-PLAN.md` §8
- **Content:**
  - **Per-phase unit tests** in `elixir-core` for each DSP component (deterministic input → expected output buffer with FP tolerance). Golden WAVs for spectral-morph variants, FX algorithms.
  - **Property tests** (proptest) for invariants: no NaN; no allocation on audio thread; voice-handler maintains `len(free) + len(active) + len(released) == capacity`.
  - **`assert_no_alloc`** wrapping every `process()` in debug builds. Any allocation crashes the test.
  - **Cutover A/B** (`tests/cutover_parity.rs`): render fixed MIDI sequence through both `Synth` and `ElixirSynthBlock` with `Contrapunk-Default` preset; assert RMS difference `< -90 dBFS`.
  - **Smoke tests per surface:** CLI render, Tauri "play 4-bar progression + metronome", plugin loaded in `pluginval`.
  - **Existing `tests/audio_pipeline.rs`** runs against both feature configurations.
  - **Track C:** `pluginval` against Contrapunk as host; CI runs smoke suite. Known-good plugins on every macOS CI run: Surge XT (CLAP), TDR Nova free (VST3).
  - **Continuous:** existing harmony-engine 249 unit tests must keep passing; WASM build green continuously.

## ELIX-CON-22 — Risk register (top 8)

- **Type:** nfr
- **Source:** `ELIXIR-PLAN.md` §9
- **Content:** Top risks with mitigations:
  - **SIMD nightly blocking stable users** — default to `wide`; gate `std::simd` behind `cfg(feature = "nightly-simd")`; CI runs both.
  - **`nih-plug` upstream churn** — pin both Contrapunk's and Elixir's nih-plug to the same fork SHA.
  - **`clack-host` API breaks** — pinned by `rev`; bump deliberately not opportunistically.
  - **VST3 SDK licensing** — Track C's VST3 module GPL3'd; hold 1-day legal review at start of C3.
  - **Audio-thread allocation regression** — `assert_no_alloc` wrapping in debug builds; CI runs full audio path in debug for one block per commit.
  - **Replacing synth changes Contrapunk's tone** — `Contrapunk-Default` preset is byte-for-byte parity; A-Cut gated on < -90 dBFS RMS A/B test; ship behind feature flag 2 weeks before flipping default.
  - **Plugin GUI embedding fragility per OS** — detached-window fallback ships first; embedding is best-effort.
  - **Track B UI scope balloons** — B6/B7 explicitly last; if slip, A and C still ship; first public face can be "standalone w/ no UI, headless only" or "plugin only — DAW provides UI free".

## ELIX-CON-23 — Cross-track integration contracts

- **Type:** api-contract
- **Source:** `ELIXIR-PLAN.md` §6
- **Content:**
  - `AudioBlock` trait: shape **locked at A0**, additive-only after that
  - `Chain::PushBlock` ordering: harmony → synth (Elixir or hosted) → FX (Elixir built-in OR hosted FX); default chain assembly needs `insert_at(idx)` if not present
  - Mod-matrix `ParamId` enum gets `Internal(...) | Hosted { slot, plugin_param_id }` variants
  - Session preset format: top-level `chain: [{ kind: "elixir", preset: {...} }, { kind: "clap", path: ..., state: <base64> }, ...]`; no format fork
  - `egui` (Elixir UI) and Svelte (Contrapunk UI) are INDEPENDENT. Boundary is audio chain — Contrapunk drives Elixir via MIDI; Elixir owns its param state.
  - Release pipeline: `elixir-` tag prefix triggers Elixir matrix; `v` tag prefix triggers Contrapunk builds; both share signing identities.
  - WASM compilation: `elixir-core` must compile to wasm32 on every PR. Plugin-hosting is `cfg(not(target_arch = "wasm32"))`.

---

*Synthesized: 2026-05-18 from `/gsd-ingest-docs` run on ELIXIR-DESIGN.md + ELIXIR-PLAN.md.*
