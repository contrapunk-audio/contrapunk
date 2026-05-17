# Elixir — Synthesizer Design Document

**Status:** Draft v0.1
**Target:** Cross-platform spectral-warping wavetable synthesizer
**Surfaces:** VST3 + CLAP plugin, standalone desktop app (macOS / Windows / Linux), headless CLI renderer
**Language:** Rust workspace (one DSP core crate, multiple surface crates)

---

## 1. Overview

Elixir is a polyphonic wavetable synthesizer whose distinguishing trait is **frequency-domain warping**: each wavetable frame is editable in the spectral domain, and an oscillator can morph the spectrum at audio rate through one of twelve operations (vocode, formant-shift, smear, harmonic-scale, phase-disperse, shepard, skew, etc.) before the IFFT writes time-domain samples for playback. The result feels like a wavetable synth and a spectral-modeling synth fused into one.

The architecture is built on five subsystems:

1. **Engine + voicing + framework** — the processing graph, voice allocator, and module abstractions
2. **Wavetable oscillator + spectral engine** — the core sound source with FFT-based warping
3. **Filters + effects** — eight filter topologies and a reorderable eight-slot FX bus
4. **Modulators + modulation matrix** — envelopes, LFOs, randoms, MPE sources, and the routing system that connects them to any destination
5. **Build targets, hosting, presets, and concurrency** — non-DSP plumbing: how MIDI lands, how presets serialize, how UI talks to audio

The remainder of this document specifies each subsystem and the Rust implementation strategy.

---

## 2. Crate stack at a glance

| Concern | Crate / approach |
|---|---|
| Plugin host (VST3 + CLAP + standalone wrapper) | `nih-plug` |
| Standalone audio I/O | `cpal` |
| Standalone MIDI I/O | `midir` |
| SIMD | `std::simd` (portable, nightly) with `wide` as the stable fallback |
| FFT | `realfft` (wraps `rustfft`, specialized for real input/output) |
| Resampling (where polyphase needed) | `rubato`; otherwise hand-rolled half-band IIR |
| Lock-free shared state | `arc-swap` for whole-object hot-swap; `rtrb` (SPSC) for audio-bound events; `crossbeam-channel` for multi-producer |
| Realtime allocation guard | `assert_no_alloc` (debug-only) |
| Bump arenas for "safe" audio-thread "allocation" | `bumpalo` |
| Serialization | `serde` + `serde_json` for presets, `base64` for embedded PCM, `flate2` for plugin state chunks |
| Deterministic RNG | `rand_chacha` (ChaCha8) |
| WAV writing (headless surface) | `hound` |
| Tuning (.scl / .kbm / .tun) | Hand-rolled parser (~300 LOC); optional `mts-esp-rs` behind a feature flag |
| Argv (headless surface) | `clap` |
| Static dispatch over closed enums | `enum_dispatch` |

---

## 3. Engine, Voicing, and Framework

### Engine architecture

The top-level synth is a single class — call it `SoundEngine` — that owns a `ProcessorRouter`. The router owns a topologically-ordered list of child processors wired at construction time. Signal flow is:

```
MIDI in → NoteHandler → VoiceHandler → per-voice graph (osc / env / filter / mods)
       → SIMD stereo sum → Upsampler (2x) → ReorderableEffectChain
       → Decimator (3-pole IIR halfband) → stereo encode → SmoothedVolume
       → PeakMeter → Clamp(-2.1, 2.1) → output buffer
```

Oscillators, envelopes, voice filters, and per-voice modulators run inside the voice graph; global modulators (master LFO, BPM clock), the FX chain, and the master gain run outside it. Oversampling is a power-of-two multiplier (default 2x), and the effective internal rate is held roughly constant against host sample rate: at 44.1 kHz the requested oversample is applied verbatim; at 88.2/176.4 kHz it is divided down. Each `process(num_samples)` call walks the router in dependency order, skipping disabled processors, and short-circuits to silence when zero voices are active (but mono modulation processors still tick so their phase / state stays coherent). Sample-rate and oversample changes propagate recursively via `set_sample_rate` / `set_oversample_amount`, resizing per-output buffers to `MAX_BUFFER_SIZE * oversample`.

**Rust implementation.** Wrap the engine as a `nih-plug` `Plugin` (DAW target) and a `cpal` stream (standalone). Avoid runtime polymorphism in the hot path: rather than a `Vec<Box<dyn Processor>>`, use enum dispatch:

```rust
enum Node { Osc(WtOsc), Filter(SvFilter), Env(Adsr), Lfo(SynthLfo), ... }
```

with the `enum_dispatch` crate or hand-rolled `match`. The topo-sorted list becomes a `Vec<Node>` plus a parallel `Vec<NodeEdges>` mapping each node's input slots to producer indices. Oversampling is best handled by `rubato` (polyphase resampler) or a hand-rolled 2x/4x half-band IIR. Denormal flushing via `core::arch` FTZ/DAZ intrinsics on x86 and the AArch64 FPCR FZ bit on Apple Silicon. Use `dasp` only for slice utilities; `fundsp` is tempting but its `AudioUnit` trait does virtual dispatch and allocates — fine for prototypes, not for the production hot path.

### Voice management

A `VoiceHandler` owns three input ports (`polyphony`, `voice_priority`, `voice_override`), an array of pre-allocated `Voice` slots (capacity `MAX_POLYPHONY + PARALLEL_VOICES`), and three circular-queue lists: `all_voices`, `free_voices`, `active_voices`. Voices are packed into `AggregateVoice` structs of `PARALLEL_VOICES = SIMD_WIDTH / 2` lanes (typically two SIMD voices share a 4-wide register), each backed by a cloned copy of the per-voice processor subgraph.

Allocation walks: free-parallel-slot in an already-active aggregate, then free voice, then released, sustained, held, triggering (in that order). Voice priority is one of `Newest / Oldest / Highest / Lowest / RoundRobin`. On note-on the pressed-notes queue is sorted by the active priority and the new voice is appended to `active_voices` then re-sorted. `voice_override` is `Kill` (cut the stolen voice instantly) or `Steal` (retrigger it with the new note).

Note-off either deactivates the voice or, if a sustain pedal is held on that channel, moves it to `Sustained`. When sustain releases, all sustained-not-sostenuto voices on that channel are deactivated. Sostenuto latches at pedal-down via a per-voice flag. Legato is implemented by a `LegatoFilter` that suppresses retriggers when `last_key_state == Held` and a `PortamentoSlope` that slews the `note` value toward target. Voice death is detected by sampling a "voice killer" output (typically the amp envelope) for an all-silent SIMD mask across the block, then returning that voice to `free_voices`. Per-block voice events carry a sample offset so note-on / note-off precision is sample-accurate within the block.

**Rust implementation.** Pre-allocate `voices: [Voice; MAX_POLY]` in a fixed-size array — never `Vec::push` on the audio thread. Free/active lists become `ArrayDeque` (the `arraydeque` crate) or two `tinyvec::ArrayVec<u8, MAX_POLY>` indices. The priority/override selection is a small `match` over `enum VoicePriority`. SIMD packing: use `std::simd` or `wide::f32x4`/`f32x8`, with two voices interleaved into each lane pair (one stereo voice = 2 lanes; 4-wide = 2 voices). Sustain / sostenuto state is a `[bool; 16]` per-channel array. The `NoteHandler` trait maps cleanly to a Rust trait with `&mut self` methods — but on the audio thread keep it object-free: feed events as `enum NoteEvent { On{note,vel,sample,chan}, Off{...}, Sustain{...} }` drained from a `crossbeam::queue::ArrayQueue` populated by the host/MIDI-in thread.

### Framework primitives

Five abstractions form the graph:

- **Processor** — virtual `process(num_samples)`, owned `Input*`/`Output*` slots, ref-counted `ProcessorState` for sample-rate / enabled / oversample.
- **Output** — a contiguous `poly_float[]` buffer with a separate trigger side-channel (`trigger_mask`, `trigger_value`, `trigger_offset`) used for sample-accurate events out-of-band from the audio stream.
- **Value** — a `Processor` whose output is a constant `poly_float`. The canonical parameter primitive.
- **Operator** — a stateless `Processor` that auto-enables when at least one input is connected; clear-and-tick-once when disabled.
- **ProcessorRouter** — a container `Processor` that owns children and runs them in topo order.

Wiring is by pointer: `dest.plug(source_output, index)` sets `inputs[index].source = source`, then the owning router calls `connect()` which either reorders the topology or, if a cycle is detected, inserts a `Feedback` node that double-buffers last block's output. The router keeps a `global_order` (shared via `Arc` across cloned per-voice copies) and a `local_order` (the actual processor instances for this voice), with `local_changes` vs `*global_changes` counters used to lazily resync cloned subgraphs after edits. `SynthModule` extends `ProcessorRouter` with a `ModuleData` map of named controls, mod sources, mod destinations (mono and poly), modulation switches, and `StatusOutput` taps used to feed UI meters. Parameter changes are just `Value::set()`; smoothing is layered on by `SmoothValue` / `cr::SmoothValue` (control-rate vs audio-rate variants) which low-pass toward the target each block.

**Rust implementation.** Reject any dyn-Processor design — virtual calls per sample-block per node tank the optimiser. Use one of:

- (a) An enum of all node types with `enum_dispatch`,
- (b) `Box<dyn Processor>` only for the *outer* effect-chain slots (block-rate dispatch is fine, sample-rate is not), or
- (c) A typestate builder that compiles the graph into a flat `Vec<Op>` of small POD instructions (closest to a tape machine, best codegen).

Inputs/outputs are `&[f32x4]` slices owned by an arena (`bumpalo` at startup, never resized after `prepare`). Cycles are still handled with an explicit `Feedback` node holding a `[f32x4; MAX_BLOCK]` ring. Value parameters: an `AtomicU32` per parameter set by the UI/host thread, read once per block by the audio thread, fed into a `SmoothedValue` (one-pole, `1 - exp(-2π·fc/sr)`) — `nih-plug`'s `SmoothedParam` already does this; for standalone use `cpal` + a hand-rolled one-pole.

### Block processing model

A single audio callback delivers `num_samples` (clamped to `MAX_BUFFER_SIZE`). The router multiplies that by per-processor `oversample_amount` so individual nodes can process at different rates inside the same call. Control-rate outputs are special-cased: their `buffer_size == 1` and they alias their backing storage to the trigger value so writes are scalar.

MIDI events that landed mid-block do **not** subdivide the block; instead each `Voice` records `event_sample` and `aftertouch_sample` (the sample-offset within the upcoming block), and `prepare_voice_triggers` translates that into an `Output::trigger(mask, value, offset)` call. Downstream stateful processors (envelopes, oscillators) read `get_reset_mask(input_idx)` against `VOICE_ON`, then use `clear_output_buffer_for_reset(mask, in, out)` to zero everything *before* `trigger_offset` so the previous voice's tail doesn't leak past the new note. The block-rate event queue is therefore sample-accurate without ever splitting the inner loop.

Control-rate modulators run once per block (`num_samples = 1`); audio-rate modulators run at full block. Routing a modulator into an audio destination is done by a `ModulationConnectionProcessor` that adapts rates and applies a destination scale; promoting a modulator from control-rate to audio-rate is a runtime flag flip when *any* connection demands it.

**Rust implementation.** Define `const MAX_BLOCK: usize = 512` (or 1024) and use stack-friendly `[f32x4; MAX_BLOCK / 4]` buffers held in a per-engine arena. Sub-block events are an event list sorted by `sample_offset` and replayed via a small inner `for sample in start..end` loop per region — or, copying the side-channel trigger approach, a `Trigger { mask: u8, value: f32x4, offset: u16 }` on each output edge. Control-rate vs audio-rate is a const generic or `enum Rate { Control, Audio }` chosen at graph-build time. Block iteration uses `chunks_exact_mut(N)` over the output buffer.

### Threading

There are exactly two threads of concern: the **audio thread** (driven by the host or by `cpal`) and the **UI/control thread** (DAW automation, mouse drags, MIDI-in if not on the audio thread). The audio thread *never* allocates, never locks; it reads atomically-published parameter values, drains a lock-free MIDI event queue, runs the graph, and writes to the output buffer.

Parameter writes from the UI go to `Value::set()` (relaxed atomic store) and are smoothed by `SmoothValue` at audio-rate or `cr::SmoothValue` at control-rate. Modulation-graph mutations (connect / disconnect a modulator) are funnelled through `SoundEngine::connect_modulation`, which is documented as audio-thread-safe: it `plug`s into the destination and increments the router's `global_changes` counter, so per-voice clones notice on their next `process()` and lazily clone the new processor. The `ReorderableEffectChain` reorder is similarly latched per-block. UI meters read from `StatusOutput` objects updated once per block.

**Rust implementation.** Mirror the two-thread model exactly. Use `crossbeam::queue::ArrayQueue<NoteEvent>` (SPSC, lock-free, bounded, pre-allocated) for UI→audio events. For meters / status outputs going audio→UI, use a `triple_buffer` (the `triple_buffer` crate) or `arc-swap::ArcSwap<MeterSnapshot>` so the UI never blocks the audio thread. Parameter atoms: `AtomicU32` of `f32::to_bits()` per parameter. Wrap the entire audio callback in `assert_no_alloc::assert_no_alloc(|| { ... })` in debug builds. Graph mutations (modulation connect/disconnect, effect reorder) are encoded as commands pushed through the same SPSC queue and applied by the audio thread at the top of `process()`. Voice-graph cloning for new polyphony slots happens off-thread: build the new graph on a worker, then `ArcSwap::swap` the whole `VoiceGraph` in. Avoid `Mutex` and `RwLock` anywhere on the audio path; if shared mutable state is unavoidable (e.g. wavetable data swap), use `arc-swap` or a double-buffered `[T; 2]` with an atomic generation counter.

---

## 4. Wavetable Oscillator and Spectral Engine

### Wavetable representation

A wavetable is a stack of **frames** stored as parallel time-domain and frequency-domain buffers. The compile-time constants are `WAVEFORM_BITS = 11`, `WAVEFORM_SIZE = 2048` samples per frame, and a frame count typically bounded at 256.

A `WaveFrame` is the editable representation handed to producers and modifiers. It carries `time_domain[2 * WAVEFORM_SIZE]` (double-length to give the inverse FFT a write-without-wrap scratch zone), `frequency_domain[WAVEFORM_SIZE]` of `Complex<f32>`, plus a frequency ratio and sample rate so resampled imports keep their pitch metadata. The compiled-for-audio `Wavetable` is a different structure: per frame it stores the raw `f32[WAVEFORM_SIZE]` time-domain waveform, and three SIMD-packed parallel arrays: `frequency_amplitudes`, `normalized_frequencies` (unit-magnitude complex per harmonic), and `phases`. Harmonics are stored as `NUM_HARMONICS = WAVEFORM_SIZE/2 + 1 = 1025` bins.

Frames are not mip-mapped per frame in the classical sense; **mip-mapping is implicit in the spectral pipeline** — at playback time the oscillator computes a frequency bin from the phase increment (`get_frequency_bin = floor(log2(1/phase_inc))`) and band-limits the IFFT to `last_harmonic = WAVEFORM_SIZE * 2^-(FREQ_BINS+1-bin)` so high pitches synthesize from fewer harmonics. The result is anti-aliasing equivalent to per-octave mip-maps, but generated lazily into a small set of double-buffered Fourier output frames (`fourier_frames1` / `fourier_frames2`) sized `SPECTRAL_BUFFER_SIZE = 2 * WAVEFORM_SIZE / SIZE + SIZE`.

Frame interpolation across the table-position axis is Catmull-Rom in time domain for static morph types and per-bin linear interpolation in frequency domain for the spectral "skew" morph. A double-buffer hot-swap via `std::atomic<WavetableData*>` (`current_data` vs `active_audio_data`) lets the UI thread rebuild frames while the audio thread holds a stable pointer. `post_process()` performs **phase unwrapping** across frames: bins below an amplitude floor inherit a linearly-interpolated complex direction from neighbouring loud bins, preventing phase discontinuities when amplitude crosses zero.

**Rust implementation.** Represent `WaveFrame` as a struct with `time_domain: Box<[f32; 4096]>` and `frequency_domain: Box<[Complex<f32>; 2048]>` aligned to 32 bytes via `#[repr(align(32))]`. The audio-side `WavetableData` becomes three `Box<[[f32x8; POLY_FREQ_SIZE]]>` slabs (one per frame) plus the raw time-domain frames. Use `arc-swap::ArcSwap<WavetableData>` for the lock-free hot-swap: it eliminates any busy-wait `yield()` loop and gives the audio thread a wait-free `load_full()` that returns an `Arc` it can keep for the duration of a block. Make `WavetableData` `Send + Sync`. Frame-axis interpolation defaults to four-point Catmull-Rom — same quality as linear+slope alternatives, two FMAs deeper but still SIMD-friendly with `std::simd::f32x8`.

### Oscillator playback

Phase is a 32-bit unsigned integer (`poly_int`) treated as a fixed-point fraction of a wavetable period: the top 11 bits index a sample, the bottom 21 bits (`INTERMEDIATE_BITS`) are the interpolation fraction. Each sample, `phase += to_int(phase_inc_buffer[i] * phase_inc_mult)`. There is **no BLEP/polyBLEP** — anti-aliasing is exclusively band-limited synthesis at the spectral stage; the oscillator merely interpolates a pre-band-limited frame.

Sample interpolation is **four-point Catmull-Rom** between adjacent waveform samples and, when two adjacent table frames differ, an outer Catmull blend between `from_buffers` and `to_buffers` (`interpolate_multiple_buffers`), all SIMD-evaluated through a 4x4 matrix of SIMD lanes.

Unison stacks up to `MAX_UNISON = 16` detuned voices arranged as 8 SIMD pairs (`NUM_POLY_PHASE`), each with its own phase accumulator, detune ratio, distortion-phase, and pair of from/to waveform pointers. Detune is computed as `cents_to_ratio(t^power * range_cents)` per stack slot, with eleven unison stack styles (octave, fifth, power chord, harmonic series, etc.) supplied as a static `STACK_MULTIPLIERS` table.

Phase distortion comes in nine flavours dispatched via template/function-pointer specialisation plus a `window` function: `quantize`, `bend` (cubic ease), `squeeze`, `sync` (multiplied phase ramp), `pulse_width` (clamped phase), `fm_oscillator_a/b/sample` (additive phase offset from another oscillator's output buffer), and `rm_oscillator_a/b/sample` (window multiplied into output). FM phase offset is scaled by `FM_PHASE_MULT = PHASE_MULT / 8` and capped at `MAX_FM_MODULATION = 48` to bound aliasing.

**Rust implementation.** Keep the fixed-point phase trick — it is excellent. Define `type Phase = u32; const FRAC_BITS: u32 = 21;`. The phase accumulator is intrinsically wrapping (`Wrapping<u32>` or just `u32` with `wrapping_add`). For SIMD, use `std::simd::Simd<u32, 8>` and `Simd<f32, 8>`; both `wide::u32x8`/`f32x8` and portable `std::simd` produce identical AVX2 code on Apple Silicon NEON and x86.

Replace template-over-function-pointer dispatch with a small `enum DistortionMode` and a `match` in the chunk function — the LLVM optimiser hoists the match outside the inner loop when the function is `#[inline]` and the variant is a compile-time constant. For genuine per-block monomorphisation, use a generic over a `trait PhaseDistort { fn distort(...) -> Simd<u32,8>; }` with zero-sized impl types. Catmull-Rom should stay; it's preferable to Hermite (extra slope params, no audible benefit at 2048 samples per frame) and plain linear (audible HF aliasing at unison detune). Pre-allocate the scratch `Box<[f32x8; SPECTRAL_BUFFER_SIZE]>` at voice construction; the audio thread allocates nothing.

### Spectral / warping operations

The frequency-domain pipeline is a pluggable `spectral_morph` function selected per voice from twelve variants:

- `passthrough`
- `vocode` (even/odd formant shift)
- `form_scale`
- `harmonic_scale`
- `inharmonic_scale` (per-bin power-law shift)
- `smear` (recursive amplitude blur with `(i+0.25)/i` tilt)
- `random_amplitudes` (interpolated between sixteen pre-baked random staircases held in `RandomValues`)
- `low_pass`/`high_pass` (cutoff `pow(2, (FREQ_BINS-1) * t)` with sub-bin fractional last tap)
- `phase_disperse` (per-bin quadratic phase shift derived from `(i - CENTER_MORPH)^2`)
- `shepard_tone` (octave-folded amplitude/phase morph)
- `skew` (per-bin frame-position remap)

Each morph fills a destination buffer of SIMD complex pairs starting at `dest + 1` (the leading slot becomes the IFFT's "below-zero" wrap pad), then `transform_and_wrap_buffer` calls `FourierTransform::transform_real_inverse` and copies `SIZE` (SIMD width) samples from the buffer tail to the head and another `SIZE` from the head to the tail — this gives every Catmull-Rom read a contiguous neighbourhood without bounds checks.

The FFT is a real-complex transform templated on `WAVEFORM_BITS` (length 2048). There is no explicit window — the wavetable is treated as one period of a periodic signal, so a rectangular window is correct. Frequency-domain editing feeds back to time-domain on every block whenever wave-frame index, spectral-morph amount, distortion, or phase-increment frequency-bin changes, with double-buffering (`fourier_frames1` / `fourier_frames2`) so the previously-active buffer remains valid for the cross-fade Catmull-Rom blend.

**Rust implementation.** `realfft` (a thin wrapper over `rustfft` specialised for real input/output) gives identical complexity and avoids the `Complex<f32>` packing dance. Build one `RealFftPlanner<f32>` per voice at construction; the forward and inverse plans expose `process(&mut input, &mut output, &mut scratch)` which is allocation-free after the first call. Store morphs as an `enum SpectralMorph` with an `apply(&self, table, frame, dst, scratch, shift, last_harmonic)` method; dispatch via match on the enum variant. The wrap-pad trick (`SIZE` samples copied head/tail) ports verbatim — it is what lets the playback loop run branch-free. Use `bytemuck::cast_slice_mut` to flip between `&mut [f32]` and `&mut [Complex<f32>]` views with zero copies.

### Lookup tables

A single generic `OneDimLookup<function, resolution>` bakes any `f(t)` into a `f32[resolution + 4]` array at construction (the extra 4 slots support Catmull-Rom's `i-1, i, i+1, i+2` stencil without bounds checks). Lookups are SIMD: `cubic_lookup(simd)` computes `boost = value * scale`, splits into integer index and fraction `t`, builds a 4x4 Catmull interpolation matrix and a 4x4 value-gather matrix, transposes, and reduces with `multiply_and_sum_rows` — four FMA-heavy steps, no branches.

The same `OneDimLookup` underpins `futils::sin`, `futils::pow`, `futils::log2`, `futils::tanh`, `futils::exp2`, equal-power crossfade curves, and the cents-to-ratio conversion used hot-path in unison detune. The detune `STACK_MULTIPLIERS` table is a static 2D array. Random-amplitude staircases are baked once at process-construction into a `RandomValues` singleton sized `(RANDOM_AMP_STAGES+1) * (NUM_HARMONICS+1) / SIZE` SIMD lanes seeded with `0x4`. The fixed-point phase→interpolation-fraction conversion extracts the bottom 21 bits and multiplies by `2^-21` — equivalent to a bit-level cast, free.

**Rust implementation.** Express lookups as `const fn` initialised arrays where possible: `const SIN_TABLE: [f32; 2052] = build_sin_table();` with a `const fn` that runs a polynomial or CORDIC approximant at compile time. The `libm` `f32::sin` is not `const`, so either roll a small `const fn` polynomial or fall back to `std::sync::OnceLock<[f32; N]>` initialised on first use — pay the init cost once, never branch in the hot loop.

Avoid `lazy_static!`/`once_cell::Lazy` for hot-path tables: they emit an atomic check per access. Instead, build a `pub struct Lookups { sin: [f32; 2052], ... }` constructed once at audio-engine startup and passed by `&'static` reference into every voice. SIMD lookup: `std::simd::Simd::<f32,8>::gather_or_default(&table, indices)` for the four-tap gather, then four FMAs against a hardcoded Catmull-Rom basis. The seeded RNG for the random-amplitude staircase should use `rand_chacha::ChaCha8Rng::seed_from_u64(0x4)` — deterministic, fast, well-tested.

### Sample playback and granular

The sampler path (`Sample` + `SampleSource`) does not do classical granular or phase-vocoder time-stretch. Instead it builds **a band-limited mip-map pyramid** when a sample is loaded: one 2x-upsampled copy (52-tap windowed-sinc upsampler with hand-tuned coefficients) plus a chain of half-rate downsampled copies (55-tap windowed-sinc decimator) down to a minimum size of 4. For each copy two variants are stored — a "play" buffer zero-padded at the boundaries and a "loop" buffer with the head/tail wrapped — so loop and one-shot reads share an interpolation kernel.

At playback the active mip level is chosen from `ilog2(phase_inc)`, so a sample played up an octave reads from the half-rate level and so on. Pitch shift is pure phase increment plus Catmull-Rom interpolation (same kernel as the wavetable oscillator); there is no time-stretch separable from pitch. Forward/reverse bounce-loop is a single SIMD mask flip per voice; stereo is interleaved into adjacent SIMD lanes.

**Rust implementation.** Keep the mip-map approach — it is the cleanest anti-aliasing strategy for one-shot sample playback. Use `realfft` to design the FIR coefficients offline as a `const [f32; 55]` rather than hand-typing them. If granular or independent time-stretch is wanted later, layer a phase-vocoder on top: forward STFT with a Hann window of 1024 / 75% overlap via `realfft`, peak-locked phase advance (Laroche–Dolson) for monophonic material, and inverse STFT with the same window — implement as a `GranularEngine` consuming the same mip-map pyramid so pitch shift and time stretch can be combined or used separately. Boundary handling: replace the C-style padding with Rust slice indexing through a `read(idx: i64) -> f32` accessor that masks against `len.next_power_of_two() - 1` for looped reads and clamps to `[0, len)` for one-shots — the inner loop stays branch-free because the mask/clamp choice is hoisted into a closure picked once at voice start.

---

## 5. Filters and Effects

### Filter models

Elixir exposes eight filter models, all derived from a common abstract base that publishes `(cutoff_midi, resonance, drive, gain, style, pass_blend, interp_x, interp_y, transpose, spread)` and a per-block `setup_filter` plus a per-sample `tick`. Cutoff is delivered as a per-sample MIDI-cutoff buffer (audio-rate); resonance / drive / gain are sampled once per block and linearly ramped to target across the block.

- **Analog Sallen-Key (12/24 dB, dual-notch-band, peak-band-notch).** Two cascaded TPT one-pole stages with a feedback path, ZDF formulation. The one-pole is the standard trapezoidal integrator: `g = tan(π * f / fs)` (precomputed by a 2048-point cubic-interpolated lookup), `y = s + g*(x - s)/(1+g)` and `s += 2*g*(x - s)/(1+g)`. Two of those plus a stage-1 feedback term implement a 2-pole; a second pair stacks for 4-pole. Resonance self-oscillates around 2.15; the inner saturation is an inline `tanh` on the resonance feedback path.
- **Ladder (12/24 dB, notch-pass-swap, dual-notch-band, band-peak-notch).** Four cascaded saturating one-poles (template-parameterized with `futils::algebraic_sat`) closed by a global resonance feedback. The transfer is the classic four-pole-cascade with the all-pass-blend trick: per-stage outputs `s0..s4` are linearly combined by style-specific scaling vectors (e.g. `{0,0,0,0,1}` for LP24, `{1,-4,6,-4,1}` for HP24) to morph filter shape continuously. Saturation lives both inside the per-stage state update and as a `tanh` on the input mixer.
- **Diode (12/24 dB).** Four cascaded one-poles with cross-stage averaging (`stage[n] = 0.5*(prev + next_sat_state)`) plus a feedback high-pass to model the ladder-bias asymmetry. Includes a tunable input high-pass (one-pole pair) whose ratio sweeps an 8-octave range. Saturation: `tanh` on stage 1, hard clip on stage 4.
- **"Dirty" (12/24 dB, multiple styles).** Two pre-stages and two saturated post-stages (`quick_tanh` per stage); resonance is "tuned" against the cutoff coefficient (`resonance / max(1, 0.25*g + 0.97)`) to keep self-oscillation pitch stable, with a drive-resonance boost that pushes the filter into screaming territory.
- **Digital state-variable.** Vadim Zavalishin's trapezoidal SVF. Per-tick: `c² = g²`, `c0 = 1/(g² + g*R + 1)`, `c1 = c0*g`, `c2 = c0*g²`, then `v3 = in - ic2; v1 = c0*ic1 + c1*v3; v2 = ic2 + c1*ic1 + c2*v3; ic1 = 2*v1 - ic1; ic2 = 2*v2 - ic2`. Output is `blend.v0*in + blend.v1*v1 + blend.v2*v2`. A second cascaded SVF gives the 24 dB variant with an inter-stage `hard_tanh` for cheap drive coloration. Shelving and dual-notch-band are style switches that re-derive the `(v0, v1, v2)` blend coefficients.
- **Comb / flanger (positive / negative / unbiased feedback, low-high blend or band-spread).** A circular delay-line `Memory` with fractional-tap read, plus a two-band shelving filter inside the feedback loop (two one-poles) and a `hard_tanh` on the loop. The delay length is `period = fs / midi_to_hz(cutoff)`, clamped to `[2, memory_size-5]`. Band-spread mode runs two cutoffs offset by ±N octaves.
- **Formant (two vowel-quad styles + vocal tract).** A `FormantManager` of four parallel digital-SVF bandpass stages whose `(midi_cutoff, resonance, gain)` triplets are bilinearly interpolated across a 2-D `(interp_x, interp_y)` vowel grid (A/O/I/E or A/I/U/O corners). Each formant rides the digital-SVF kernel at 12 dB with the `pass_blend` parameter cross-fading toward a neutral 80-MIDI center for "blend".
- **Phaser-filter.** 12 cascaded all-pass one-poles (`y = 2*onepole(x, g) - x`) grouped into three 4-stage banks; outputs at the 4th, 8th, and 12th stage are weighted (`peak1, peak3, peak5`) into a single sum, optionally saturated. The "clean" template specialization uses identity; the dirty one inserts `tanh` on the resonance and input paths. A low-shelf/high-shelf pair removes DC/HF energy from the recirculation.

In addition to the user-facing models, the filter directory contains three structural helpers: a one-zero DC blocker (`y = x - x[-1] + a*y[-1]`, `a ≈ 0.999`), a Linkwitz-Riley 4th-order LP/HP crossover (biquad squared, used by the multiband compressor), and FIR/IIR half-band decimators plus an upsampler — the entire FX bus runs at 2× oversampling.

### Filter modulation, smoothing, path

Every filter follows the same pattern. `setup_filter` is called once per audio block at the start of `process`. It captures the *target* coefficient set; the previous block's coefficients are saved into `current_*` locals; per-sample deltas `delta = (target - current) / num_samples` are computed; the inner sample loop then linearly ramps `current += delta` every tick. The MIDI cutoff itself is a full per-sample buffer (so modulators can move it at audio rate); the rest of the parameter set is per-block-linearly-smoothed. The TPT coefficient `g(f)` is *not* computed inline — it's a 2048-entry cubic-interpolated lookup keyed on `frequency/fs`, which is critical because `tan()` is otherwise the per-sample bottleneck.

Oversampling: the entire effects-engine output runs through an `Upsampler` (linear interpolation, prior to FX) and a 3-stage IIR halfband `Decimator` after FX. The voice-internal filter sits *upstream* of the FX bus and runs at the voice's native rate (still 2× upsampled because the whole engine is upsampled). Voice signal path is `oscillators → voice filter(s) → upsampler → FX chain → decimator → stereo encode → smooth volume → DC + clamp`.

Saturation strategy is per-filter: ladder/diode use `futils::tanh` (a polynomial+rational approximation), digital SVF uses `hard_tanh` (clamp-style for cheap sigmoid), dirty uses `quick_tanh` (table). All saturation is inside the unit delay (`get_next_sat_state()`) so ZDF stability is preserved across the implicit feedback.

### Effects chain

A `ReorderableEffectChain` runs eight effect modules; their order is itself an automated parameter. Each effect implements `Processor::process_with_input`.

- **Chorus** — multi-tap modulated delay; up to 4 delay pairs with LFO-modulated taps; outputs the tap delays as status meters.
- **Flanger** — same delay-line core as the comb-filter with a swept short delay, internal LFO; a single `output_frequency` status meter.
- **Phaser** — 12-stage all-pass cascade (the `PhaserFilter` above) wrapped with a built-in LFO (`rate`, `mod_depth`, `phase_offset`, stereo `0.5`-stereo-split) and a sample-accurate cutoff buffer driven by an internal 32-bit phase counter. Mix is equal-power smoothed.
- **Distortion** — six time-invariant nonlinearities selected by an integer: soft-clip (`tanh(x*d)`), hard-clip, linear-fold (triangular wave-folder), sin-fold, bit-crush (`round(x/q)*q`), and downsample (sample-and-hold at `1/d * 88200` rate). Drive is dB-scaled and smoothed at audio rate via a per-sample drive buffer. The two stereo voices are packed into one SIMD register for the inner loop, then expanded back.
- **EQ** — a `LinkwitzRileyFilter` crossover plus parametric SVF bands (the same digital-SVF used as filter model, in shelving style); state is captured into a stereo memory for the UI spectrum readout.
- **Compressor** — RMS-envelope compressor with separate upper *and* lower threshold/ratio (so it does upward + downward in one stage), exponential attack/release `samples = exp(8*p - 4) * base_ms * fs/1000`. There are three preset attack/release "speeds" (low/band/high). The `MultibandCompressor` splits the input with two cascaded LR crossovers into three bands, each fed to its own `Compressor`.
- **Reverb** — 16-line **FDN** (feedback delay network). Allpass diffusion stage first (4×4 SIMD allpass taps with prime-ish delays in samples), then 16 feedback delays (lengths in samples scaled by sample-rate ratio; canonical lengths 6753.2, 9278.4, 7704.5, 11328.5, ...). Per-line T60 decay is `dec[i] = 0.001^(delay[i]*size_mult/(decay_seconds*fs))`. Each feedback path has a one-pole low-shelf and high-shelf for HF/LF damping; chorus drift is implemented as a complex-rotator LFO modulating each delay-tap offset. Pre-filters (low + high one-poles) shape the input. Each of the 16 feedback lines is a `f32[max_feedback_size + extra]` with power-of-two masking; the 4 allpass taps share SIMD interleaved arrays. Sample-rate-dependent buffer sizes are rounded up to the next power of two and *reallocated* (not resized in-place) on sample-rate change.
- **Delay** — templated on `StereoMemory` / `Memory`. Modes: mono, stereo, ping-pong, mid-ping-pong, clamped-dampened, clamped-unfiltered, unclamped-unfiltered. Internal one-pole HP + one-pole LP (or single LP in damped mode) inside the feedback loop. Period is derived from frequency (`samples = fs / freq`), exponentially smoothed (half-life = 20 ms), clamped to `[3, max_period]`, with a `0.5` interpolation against the previous block period to avoid zipper.
- **Filter-FX** — exposes any of the eight filter models as a global insert.

### Routing, ordering, parameter rates

Wet/dry inside every effect uses equal-power-fade `wet = sin(π/2 * w)`, `dry = cos(π/2 * w)`, both per-block-smoothed. Almost all FX parameters are *control-rate* (one SIMD value per block, ramped); a handful — cutoff buffers on filter-fx, the LFO-driven cutoff out of the phaser, the per-sample drive on distortion — are full audio-rate buffers. Effects are *serial* (chain order is a permutation of the eight slots), not parallel; there is no send-bus architecture. The `EffectsModulationHandler` is a `VoiceHandler` that accumulates polyphonic modulation outputs into mono control-rate values feeding the FX chain — this is how poly-modulated effect parameters become effectively mono at the FX boundary.

### Resource management

Filter `Processor`s pre-allocate fixed-size state at construction; nothing on the audio thread allocates. The Reverb is the exception: `setup_buffers_for_sample_rate` reallocates the 16 feedback lines and 4 allpass interleaved buffers when the sample-rate-derived `buffer_scale` changes, gated by an early-exit if the size is unchanged. `set_max_samples` on `Delay` rebuilds its `Memory` on the audio thread when the max-time control changes — a footgun worth removing. `reset(mask)` carries a polyphonic mask so polyphonic voices clear only the lanes that re-triggered; `hard_reset` clears all lanes. On preset load and transport jump, every processor's `correct_to_time(seconds)` drives LFO phase reset; reverb / delay state is *not* cleared on transport jump (preserves tails).

### Rust implementation

**Crates.** `fundsp` is tempting (it bundles SVFs, Schroeder / FDN reverbs, biquads), but its `An`/`Frame`-based combinator model fights audio-rate parameter modulation and per-block ramp smoothing. **Hand-roll the DSP**; lift `dasp_sample` for sample conversion; optionally use `realfft` for analysis; use `fundsp` only for prototyping reference filters in tests. `wide` / `core::simd` is the SIMD layer.

**Filter type design.** Each topology is a plain `struct` of POD state (typically 2-12 SIMD fields) with a `process_block(&mut self, &mut [f32x4], cutoff_per_sample: &[f32x4], state: &FilterState)` method that consumes a fresh `FilterState` derived once at block boundary and ramps internally. Avoid `Box<dyn Filter>` in the inner loop. The selector is a static-dispatch `enum`:

```rust
enum FilterModel {
    Analog(SallenKey), Ladder(Ladder), Digital(Svf), Diode(Diode),
    Dirty(Dirty), Comb(Comb), Formant(Formant), Phase(Phaser)
}
impl FilterModel {
    fn process_block(&mut self, buf: &mut [f32x4], cut: &[f32x4], st: &FilterState) {
        match self { /* ... */ }
    }
}
```

Compiles to a jump table or, with `#[inline(always)]` on the variant methods, an inlined branch — both are dramatically cheaper than vtable indirection. Use `enum_dispatch` if hand-writing the match is tedious, but a hand-match is fine for eight variants. Saturation curves (`tanh`, `hard_tanh`, `algebraic_sat`) are free functions; parameterize via const generics if a topology needs multiple saturator choices.

**Coefficient lookup.** Build a 2048-entry `[f32; 2048]` cubic-spline LUT for `tan(π·f)` at startup; look up with `f32x4::cubic_lookup(ratio)`. The LUT is `Lazy<&'static [f32; 2048]>`.

**Smoothing / modulation.** A small `Ramp<T>` helper with `start`, `delta`, `current` plus `step()` per sample matches the per-block-ramp idiom and keeps allocations zero. Audio-rate cutoff stays as a `&[f32x4]` slice owned by the modulation graph; control-rate parameters are `f32x4`s on the filter struct.

**Reverb recommendation.** FDN-16 is the right CPU/quality target on desktop / DAW plugin, but FDN-8 with a 16-tap allpass diffuser is the right call for any future WASM surface (≈40% the inner-loop cost, perceptually nearly indistinguishable on dense material). Use a Hadamard mixing matrix (not the identity sum) for cheaper, denser modal density. Keep the per-line low-shelf/high-shelf and the chorus-LFO complex-rotator. A Schroeder reverb (4 comb + 2 allpass) is only worth shipping if CPU-bound; quality drop is audible.

**Pre-allocation.** Delay lines are `Box<[f32x4]>` with capacity rounded up to power-of-two at construction; index masking replaces modulo. The reverb's 16 feedback lines + 4 allpass buffers are a single `Vec<Vec<f32x4>>` allocated in `prepare(sample_rate, max_block)` on the *prepare* thread, never the audio thread. The audio thread's `process` method takes `&mut self` and `&mut [f32x4]` only — no allocator calls. Convolution kernels (if added for IR reverb later) load into `AlignedBox<[f32]>` at preset-load time on a worker thread; the audio thread swaps an `ArcSwap<KernelHandle>` pointer.

**Reset semantics.** A Rust `fn reset(&mut self, voice_mask: u8)` mirrors the masked reset; `hard_reset` clears all lanes. Transport jumps call `correct_to_time(secs)` on every stateful effect; reverb / delay tails are intentionally preserved unless the host explicitly requests a flush.

---

## 6. Modulators and Modulation Matrix

Elixir's modulation system is intentionally orthogonal to its audio path: every modulator is a small `Processor` that writes a per-block buffer of normalized values, and every routing is a separately-owned `ModulationConnection` that reads one source buffer, applies amount / curve / polarity transforms, and adds its result into a destination input. The matrix is therefore not a dense `N × M` array but a sparse, append-only list of typed connections evaluated once per processing block.

### Modulation source types

Seven kinds of modulation sources, each implemented as its own processor with its own per-voice state:

- **Multi-stage envelopes** (`Envelope`). Six-stage DAHDSR (delay, attack, hold, decay, sustain, release) plus a forced "kill" tail. Each non-sustain segment has its own signed exponent (`attack_power`, `decay_power`, `release_power`) used by a power-curve shaper to morph between exponential, linear, and logarithmic shape inside a single segment. State is `position`, `value`, a `poly_state` enum `{Idle, On, Hold, Decay, Off, Kill}`, plus a `start_value` captured at every transition so release / kill can interpolate from wherever the envelope was when note-off arrived. Runs at block rate by default, but the amp envelope upgrades itself to per-sample.
- **Custom-waveform LFO** (`SynthLfo`). A phase-tracking LFO whose waveform is a breakpoint curve evaluated via Catmull-Rom interpolation over a pre-rendered 2048-sample lookup. Supports five run modes (`Trigger`, `Sync`, `Envelope`, `SustainEnvelope`, `LoopPoint`, `LoopHold`), tempo-sync (straight/dotted/triplet) plus keytrack, optional smoothing (one-pole low-pass with half-life in seconds), and an attack-style "fade-in" amplitude ramp with delay.
- **Stochastic LFO** (`RandomLfo`). Four flavors: Perlin (smoothstep between successive samples), Sample-and-Hold, sine-interpolated, and a 3-state Lorenz attractor integrated per sample. All share the same phase/sync machinery as the custom LFO and own per-voice RNG state.
- **One-shot trigger random** (`TriggerRandom`). Single SIMD value latched on every note-on; nothing else.
- **Line/curve mapper** (`LineMap`). Reads any input phase 0..1 and looks it up through the same breakpoint curve used by the LFOs — used so that velocity, aftertouch, mod-wheel, note, slide, lift and pitch-wheel each pass through their own user-editable response curve before becoming mod sources.
- **MPE/note value sources**. The voice handler exposes `velocity`, `aftertouch`, `slide`, `lift`, `mod_wheel`, `pitch_wheel`, `note`, `note_in_octave`, `stereo` and `random` as named control-rate outputs.
- **Macro knobs**. Mono control-rate `Value` processors whose output is just the knob's smoothed setting, registered into the same source map.

### Envelope design

The envelope is a small state machine driven by a `trigger_mask` polyphonic SIMD register: on note-on it loads `VOICE_ON`, on note-off it loads `VOICE_OFF`, and a `VOICE_KILL` state is forced when voices are stolen so the tail fades in `VOICE_KILL_TIME` seconds regardless of the user release. `start_value` is snapshotted on every transition (and the trigger of a new note) so a retrigger during the release segment smoothly continues from the current value instead of clicking back to zero — this is the legato / retrigger behavior. Stage shaping is `value = power_scale(position, power)`; positive power bows the curve up (exponential), negative bows it down (logarithmic), zero is linear. Powers themselves are interpolated across the block to avoid zippering when the user sweeps the curve knob. Two process paths exist (`process_control_rate` for block-rate per-voice modulation, `process_audio_rate` for the amp envelope) sharing a `process_section` inner loop that handles power, position and target-end deltas together. The release segment interpolates `start_value * (1 - shaped_position)`, so release always returns the envelope to zero from wherever note-off caught it.

### LFO design

The LFO's waveform lives in a `LineGenerator`: a fixed-capacity `points[100]` array of `(x, y)` breakpoints plus per-segment `powers[100]` (curve shape per segment) and a `bool smooth` flag toggling between piecewise power curves and sinusoidal-smoothed transitions. On any edit the generator re-renders into a 2048-sample float buffer with three guard samples on each side for cubic interpolation. The audio thread never touches the breakpoints; it only reads the rendered buffer through `get_value_at_phase`. Phase is a per-voice `simd offset` in `[0,1)`, advanced by `frequency * (1 / sample_rate)` per sample. Tempo-sync converts beats-per-second to frequency upstream (a `TempoSyncSwitch` chooses between hertz, straight, dotted, triplet, or keytrack-scaled frequency). Smoothing is a one-pole `y[n] = lerp(x[n], y[n-1], exp2(-dt / half_life))`. Five sync modes each have a dedicated audio-rate inner loop differing only in how phase wraps, freezes at sustain, or loops between two breakpoint x-positions.

### Modulation matrix and routing

The matrix is a fixed bank of `MAX_MODULATION_CONNECTIONS` slots; each slot holds a `ModulationConnectionProcessor` (a small SynthModule with `MOD_INPUT`, `MOD_AMOUNT`, `MOD_POWER`, `RESET` inputs and `MOD_OUTPUT`, `MOD_PRE_SCALE`, `MOD_SOURCE` outputs) plus persistent `bipolar`, `stereo`, `bypass` flags and a `LineGenerator` for an optional per-route response curve. Connecting a route plugs the named source's `Output*` into the processor's input and `plug_next`s the processor's output onto the destination's input list — destinations are themselves `VariableAdd` summers, so any number of routes additively combine into one input. There is no per-block re-walking of the routing graph: the audio thread iterates a `CircularQueue<ModulationConnectionProcessor*> enabled_modulation_processors` populated only on UI connect/disconnect.

Each route picks one of four inner-loop variants at the top of `process`: linear (raw amount), morphed (power-curve shaping via `futils::power_scale`), remapped (lookup through the per-route line curve), or both. All four interpolate `amount` and `power` linearly across the block (`delta_amount = (new - old) / num_samples`) for click-free knob changes. Bipolar sources are biased by `-0.5` so unipolar destinations see `[0,1]` and bipolar destinations see `[-0.5, 0.5]`. Modulation-of-modulation is automatic: the amount and power inputs are themselves `create_poly_mod_control(...)` outputs, so any other route can target `modulation_3_amount` just like any audio parameter. To avoid per-sample allocation every processor pre-allocates its output buffer at construction; the inner loops only read/write into these fixed buffers.

The engine knows which destinations to recompute because (a) only enabled routes are in the iteration queue, and (b) every destination is plugged into via `VariableAdd`, which has no recomputation — it simply sums whatever its inputs hold this block. Mod-source enable/disable is also lazy: at construction `disable_unnecessary_mod_sources()` parks every LFO / envelope / random until a route actually targets one.

### Per-voice vs global scope

Sources owned by the voice handler are intrinsically polyphonic; macros, the master / global LFOs and the BPM clock live on the parent router and are mono. A connection's polyphony is decided at `connect_modulation`: if the source is polyphonic AND a polyphonic destination output exists for the parameter, it routes to the per-voice input bus; otherwise it routes to the mono input bus. Both buses exist for almost every parameter (a `create_poly_mod_control` allocates them together). The voice handler reconciles scopes at block end by masking each enabled route's output buffer against the active-voice mask and OR-folding the two voice lanes (`buffer + swap_voices(buffer)`) so the meter UI sees one stable value per logical voice rather than ghosts of voices that have ended.

### Rust implementation

**Modulator representation.** With seven concrete kinds and no plug-in author API, a sum-type `enum ModSource { Envelope(Env), Lfo(Lfo), Random(Rng), Curve(LineMap), Note(NoteSource), Macro(MacroId), MpeAxis(AxisId) }` evaluated through `enum_dispatch` is preferable to `Box<dyn Modulator>`. Trait objects buy nothing here because the variant set is closed and we want all modulators stored inline in a `Vec<ModSource>` to keep them cache-warm; `enum_dispatch` collapses the vtable indirection into a `match` the compiler can hoist out of the per-sample loop. Each variant carries its own POD state struct (no `Box`), with anything cross-thread (UI-editable waveform, smoothed knobs) stored separately under an `Arc<ArcSwap<...>>` so the audio thread reads a shared snapshot pointer with no locks.

**Matrix storage.** SoA, not AoS:

```rust
struct ModRoutes {
    src:    Vec<ModSrcId>,    // typed newtype index into mod_sources
    dst:    Vec<ModDstId>,    // typed newtype index into param table
    amount: Vec<Smoothed<f32>>,
    power:  Vec<Smoothed<f32>>,
    curve:  Vec<Option<CurveId>>,
    flags:  Vec<RouteFlags>,  // bipolar | stereo | bypass | poly
}
```

Typed IDs (newtype `u16`) are far safer than raw indices and free at runtime. Sources and destinations are each in their own `Vec`, and `ModDstId` is the array index into a parallel `Vec<f32>` of base parameter values; the audio thread evaluates each block by walking active routes, summing each route's contribution into a scratch `Vec<f32>` of destination accumulators, then adding accumulators to base values. A `routes_enabled: SmallBitVec` masks which routes participate this block — flipping a single bit on connect/disconnect avoids any reallocation. Destinations needing recomputation are tracked by a `dirty: SmallBitVec` set whenever a route writes into them; downstream consumers (filter cutoff, oscillator pitch, etc.) check the bit and skip their parameter-smoothing pass when nothing moved.

**Smoothed values.** Two patterns cover all needs. For knob parameters use a linear ramp `Smoothed { target, current, step }` updated once per block (`step = (target - current) / block_size`). For envelopes / LFO smoothing and modulation-power crossfade use a one-pole `OnePole { y, coeff = exp2(-dt / half_life) }`. For zipper-free integer/enum changes use a hard slew gated on note-off.

**Lock-free UI → audio transfer.** For scalar parameters use `AtomicF32` + relaxed loads on the audio thread; the per-block ramp absorbs the latency. For complex objects (the breakpoint table backing an LFO waveform, the per-route curve, a preset swap) use `arc_swap::ArcSwap<LfoTable>`: the UI thread renders a new table off-thread, `store`s a new `Arc`, and the audio thread `load`s a `Guard` once per block. The old `Arc` drops on the UI thread, never on the audio thread. Route topology changes (connect/disconnect) go through a single-producer single-consumer ring buffer (`rtrb`) carrying `MatrixEdit { route_id, src, dst, op }` messages applied by the audio thread at block boundaries — this guarantees no torn reads of `src` / `dst` arrays mid-block and keeps the matrix mutation lock-free. Per-voice modulators are stored inside each `Voice` struct in a fixed-size `[ModSource; N]`; global modulators live on the engine and are indexed through the same `ModSrcId` space with a high bit distinguishing "global" from "voice-local" so route evaluation can dispatch without a branch on every route.

---

## 7. Build Targets, Hosting, Presets, and Concurrency

### Surface targets and shared-core layering

Elixir compiles a single engine to four binaries: a desktop standalone, a headless CLI renderer, and two plugin formats (VST3 + CLAP). All four link the same `SynthBase` type, which owns the `SoundEngine`, a `MidiManager`, a `MidiKeyboardState`, wavetable creators, a `Tuning` object, and the modulation bank. Each surface adds the host-specific lifecycle:

- The **plugin** variant implements the host's `AudioProcessor` interface and wires a per-parameter `ValueBridge` adapter that converts between the host's normalized `[0..1]` automation domain and the engine's native range (with quadratic, cubic, quartic, exponential, and square-root skew modes).
- The **standalone** variant adds a window, hardware audio device setup, and a computer-keyboard MIDI source.
- The **headless** variant uses the same `SynthBase` with an argv parser and a direct call into the offline render path.

A fix to the engine propagates to all four; each surface has its own adapter and one or two host-specific lifecycle hooks (`prepare_to_play` / `release_resources` in plugin mode, an audio device opened over the OS abstraction in standalone, a render-to-file loop in headless).

**Rust implementation.** Use `nih-plug` as the single host abstraction: one `Plugin` impl produces VST3, CLAP, and a standalone binary from the same crate via feature flags, eliminating the per-format build directories. The shared engine lives in a `no_std`-friendly `elixir-core` crate, depending only on `core::f32` math, `wide` / portable-simd for SIMD, and no host types. A thin `elixir-host` trait abstracts the things only the host can provide (transport position, sample rate, MPE config, parameter automation begin/end gestures) and is implemented twice: once on `nih_plug::context::ProcessContext`, once on a standalone driver built on `cpal` for audio and `midir` for MIDI. The headless binary is a fourth target in the workspace that links `elixir-core` plus a `clap`-based argv parser and writes WAV via `hound`. The four-way split becomes three crates (`elixir-core`, `elixir-plugin`, `elixir-standalone` / `elixir-headless`) and one feature matrix.

### MIDI handling, MPE, CC learn, and tuning

The `MidiManager` consumes both host MIDI buffers (plugin path) and OS MIDI inputs (standalone path). It decodes raw status bytes into the standard 7-bit MIDI types (`NoteOn` / `NoteOff` / `Aftertouch` / `ChannelPressure` / `PitchWheel` / `Controller`), and on the CC path it routes specifically: mod wheel (CC 1), sustain (64), sostenuto (66), slide / LSB-slide (74 / 106), LSB-pressure (102), bank/folder select (0 / 32), and the all-notes/all-controllers/all-sounds-off panic CCs. CC 74 (slide) and channel pressure are combined with their LSB partners through a high-resolution `(msb << 7) | lsb` reconstruction giving 14-bit precision; everything else is 7-bit.

MPE is first-class: an `MPEZoneLayout` snoops controller messages to detect lower/upper zone configuration and the engine exposes zoned setters (`set_zoned_pitch_wheel`, `set_channel_range_aftertouch`, `set_channel_range_slide`) so per-note pitch, pressure, and slide go to the right per-channel voice. CC learn is an "arm a parameter, next CC binds it" map (`midi_learn_map`) persisted to a JSON config; the persistence path is the only known violation of audio-thread safety in this subsystem and the Rust port closes the hole at the type level.

Tuning is a separate object loading `.scl` (Scala), `.kbm` (keyboard map), and `.tun` files into a 256-entry MIDI-to-frequency table; it serializes into the preset's JSON state and reloads with it.

**Rust implementation.** In plugin mode, MIDI lands as `NoteEvent`s from the host through `nih-plug`; in standalone mode, `midir` opens OS inputs and the same decoder consumes both. Define one enum-typed event stream (`NoteOn`, `NoteOff`, `Cc { ch, num, value }`, `PitchBend { ch, value14 }`, `ChannelPressure`, `PolyAftertouch`) and feed it into the engine independent of source. MPE state lives in a small struct keyed by zone (lower/upper master channel, member range), updated on RPN 6/0 and pitch-bend-range messages. CC learn uses a `HashMap<u8, ParamId>` snapshotted into an `ArcSwap<HashMap>` so the audio thread only reads — and the persistence write happens on a dedicated config thread that drains a `crossbeam-channel`. Tuning gets a hand-rolled `.scl` / `.kbm` / `.tun` parser (~300 lines); add `mts-esp-rs` client support behind a feature flag so users with retuning hosts get system-wide microtuning automatically.

### Preset format, versioning, and metadata

Presets are UTF-8 JSON files with a custom extension and a single top-level object: `synth_version`, `preset_name`, `author`, `comments`, `preset_style` (an enum: Bass, Lead, Keys, Pad, Percussion, Sequence, Experimental, SFX, Template), per-macro names, plus a nested `settings` object containing every flat parameter value, an array of modulation connections (`{source, destination, line_mapping}`), per-oscillator wavetable definitions, an LFO array of line-generator shapes, and the sample. Wavetable and sample audio buffers are embedded as base64-encoded PCM (int16) inside the JSON to keep presets a single portable file; a converter swaps PCM and float on save / load.

Plugin state for the host (the chunk passed to `get_state_information` / `set_state_information`) is the same JSON plus the tuning state appended at the top level. Version migration is a big sequential ladder of `compare_version_strings(version, "0.X.Y") < 0 { ... }` blocks that rewrite the JSON tree forward to current schema — dozens of these, each rewriting renamed keys, splitting fields, or filling in new defaults. Newer presets than the running build are rejected at the feature-version boundary. Factory presets, user presets, wavetables, samples, LFOs, and skins each live in their own directories with parallel "user directory" overlays. Favorites are a separate JSON file; tags are derived from `author`, `preset_style`, and filename.

**Rust implementation.** Use `serde` with `serde_json` as the canonical wire format — the JSON-with-embedded-base64 approach is worth preserving because it makes presets diffable, hand-editable, and trivially shareable on forums and Discord, which matters more than the size cost. Define each version of the schema as its own struct, derive `Serialize` / `Deserialize`, and use the `#[serde(tag = "schema_version")]` pattern or a hand-written `migrate(value: serde_json::Value) -> CurrentPreset` ladder that mirrors the sequential rewrite approach but reuses Rust's exhaustiveness checking to prove every old field is handled. Embedded PCM blobs go through `base64` + an explicit `Vec<i16>`. For binary state on plugin save (where the chunk doesn't need to be human-readable), gzip the JSON with `flate2` rather than switching to `bincode` / `rkyv` — the forward-compat story stays identical to disk presets, and you avoid maintaining two serializers. `rkyv` is the right choice only if you discover a real perf hit deserializing factory banks at boot; that's an optimization, not a v1 decision. Search/tag metadata stays as a sidecar `index.json` regenerated on factory install.

### Audio-thread concurrency and lock-free state

Two things have to cross the UI → audio boundary without dropouts: scalar parameter changes (a knob moved) and structural changes (a modulation route added, a preset loaded). The engine handles these differently.

Scalar parameter writes go directly through `Value::set(simd)` on the parameter node — a plain write to a SIMD member with no synchronization, relying on the fact that a single SIMD-register write on x86_64 and ARM64 is atomic enough that a torn read won't propagate audible artifacts past the next block's smoothing filter (`SmoothValue` runs a one-pole smoother toward the target every block, so even a torn intermediate value is masked within a few milliseconds).

Structural changes go through a lock-free MPMC queue instantiated twice on `SynthBase`: one for `control_change` events and one for `modulation_change` events. The UI enqueues; the audio thread drains via `try_dequeue_non_interleaved` at the top of `process_block` (`process_modulation_changes`). Preset loads, oversampling changes, and explicit "I'm doing something heavy" operations cannot be expressed through the queue, so those routes call `pause_processing(true)` which acquires the host's audio callback lock; the audio thread blocks until the load finishes. This is the explicit dropout window the design accepts in exchange for correctness on large state swaps — and the Rust port eliminates it via `arc-swap`.

**Rust implementation.** Three primitives, picked by use case.

1. **Scalar parameters.** Use `AtomicU32` storing `f32::to_bits()` with `Ordering::Relaxed` reads on the audio thread and `Ordering::Relaxed` writes from the UI; pair every parameter with a one-pole smoother in the engine so jump discontinuities are inaudible. `nih-plug` already gives you this for declared parameters; use it.
2. **Event streams.** For mod-matrix changes, MIDI learn map updates, "panic" notifications, use `rtrb` (a true SPSC ringbuffer designed for audio threads) — `no_std`-compatible, zero allocation on push. MPMC queues are MPMC and heap-allocate internally on growth; `rtrb` is a strict downgrade in flexibility and a strict upgrade in real-time guarantees.
3. **Full preset swaps.** Use `arc-swap`: build the entire new engine state on the UI thread, then `ArcSwap::store(Arc::new(new_state))`; the audio thread does an `ArcSwap::load()` once per block (a single relaxed atomic pointer read) and operates on whichever `Arc` it grabbed. Old state's `Drop` runs on whichever thread releases the last reference — schedule that to be the UI thread by keeping the previous `Arc` alive there for one frame. This eliminates the `pause_processing(true)` dropout window entirely for preset loads, a real user-visible improvement.

### Real-time safety enforcement

Enforce realtime safety with the type system and a runtime guard:

1. **Allocation guard.** Wrap the audio callback body in `assert_no_alloc::assert_no_alloc(|| { ... })` under `#[cfg(debug_assertions)]`; this installs a thread-local global allocator hook that panics on any `alloc::alloc` call inside the closure. Ship release builds without the guard.
2. **Type-level safety.** Define a marker trait `RealtimeSafe` and implement it only for types whose methods are provably allocation-free; have the audio-thread entry point be generic over `R: RealtimeSafe` so the borrow checker rejects passing a `Vec`, `String`, `HashMap`, or anything from `std::sync` (mutex, rwlock) into the hot path.
3. **Two-arena pattern.** For the rare case where the audio thread genuinely needs scratch memory beyond a fixed buffer (e.g. a wavetable resynth on note-on), pre-allocate two `bumpalo::Bump` arenas at engine construction sized for the worst case, alternate between them per block, and reset (not free) at block start. "Allocation" semantics on the audio thread without ever calling the system allocator.
4. **Disk persistence is physically impossible from the audio thread.** Route all "save to disk" requests through a `crossbeam-channel` whose sender is the only handle the engine holds and whose receiver lives on a dedicated config thread — the engine doesn't even link to `std::fs`. This turns the known C-style TODO into a compile error.

---

## 8. Phased implementation roadmap

Build Elixir in vertical slices so each phase produces audible output that can be QA'd before adding scope.

**Phase 1 — Bare oscillator (≈2 weeks)**
- `elixir-core` crate: voice handler (single voice), one wavetable oscillator with band-limited spectral mip-map, fixed-point phase accumulator, Catmull-Rom interpolation
- `elixir-standalone` crate: `cpal` audio out, `midir` MIDI in, computer-keyboard fallback
- **Exit:** play a single sine/saw/square wavetable from MIDI keyboard with no aliasing across the keyboard range

**Phase 2 — Polyphony + envelope + amp (≈1 week)**
- AggregateVoice SIMD packing (2 voices per `f32x8`)
- One DAHDSR envelope per voice, voice stealing, sustain pedal
- **Exit:** 16-voice polyphony, clean note steal, no clicks on retrigger

**Phase 3 — Modulation matrix (≈2 weeks)**
- SoA `ModRoutes` storage, enum-dispatched modulators, smoothing wrappers
- LFO (custom waveform), random LFO
- Modulation-of-modulation
- **Exit:** assign LFO → cutoff (once cutoff exists in Phase 4 stub) and envelope → amp, all click-free

**Phase 4 — Filter (≈2 weeks)**
- Three topologies first: digital SVF (cheapest), analog ladder, comb
- Per-sample audio-rate cutoff modulation
- TPT coefficient LUT
- **Exit:** cutoff sweep tracks LFO, no zipper, resonance self-oscillates

**Phase 5 — FX bus (≈3 weeks)**
- Upsampler + decimator (2x)
- Reorderable 8-slot chain
- Distortion, EQ, compressor, delay, reverb (FDN-8 first, FDN-16 later)
- **Exit:** full reverb tail, ping-pong delay, drive at -1 dBFS without clipping

**Phase 6 — Spectral oscillator features (≈2 weeks)**
- All 12 spectral morphs
- Phase distortion modes (FM, RM, sync, pulsewidth)
- Unison with stack styles
- **Exit:** full wavetable feature parity

**Phase 7 — Plugin surface (≈2 weeks)**
- `elixir-plugin` crate via `nih-plug`
- Host parameter automation
- MPE mapping (lower-zone master + per-note channels)
- **Exit:** plugin loads in Bitwig / Ableton / Logic, parameters automate

**Phase 8 — Presets & UI (≈4 weeks)**
- Preset JSON schema + serde round-trip
- ArcSwap preset hot-swap
- Wavetable editor / mod-matrix UI (probably egui or web-based via Tauri)
- **Exit:** load/save preset, edit wavetable shape, ship a factory bank

**Phase 9 — Headless surface (≈1 week)**
- `elixir-headless` crate: argv, render-to-WAV, batch preset rendering
- **Exit:** render a MIDI file + preset to WAV from CLI

**Total wall-clock estimate:** ≈19 weeks of focused work for one developer to feature-complete v1. The phasing front-loads the parts that gate every other phase (graph, voices, modulation), which is where most synth projects stall.

---

## 9. Open questions

- **SIMD width.** Portable-simd lets you write width-agnostic code but is nightly. Stable + `wide::f32x4` / `f32x8` works but requires picking a width. Recommend `f32x8` (256-bit) for desktop, `f32x4` (128-bit) for any WASM build, gated by `cfg(target_feature)`.
- **GUI choice.** `egui` is the fastest path but limits visual polish. Web-based (Tauri / WebView) gives unlimited polish at the cost of an IPC boundary. Pick early; the mod-matrix and wavetable editor are large UI surface area.
- **WASM target.** Every DSP component above compiles to WASM today, but oversampling and the reverb are expensive; consider FDN-8 + 1x oversampling for WASM gated behind a `cfg` flag.
- **Tuning.** Ship with `.scl` / `.kbm` from v1; defer MTS-ESP until a user asks.
- **Plugin distribution.** `nih-plug`'s CLAP support is solid; VST3 needs signing on Windows and notarization on macOS. Budget a week for code-signing logistics before v1.

---

*End of document.*
