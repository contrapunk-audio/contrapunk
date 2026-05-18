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

## 8. Interface & Visual Design

Elixir's interface is built around a single all-in-one wavetable-synth window: a wide canvas (default **1400 x 820**, minimum 350 x 205, freely scalable while preserving aspect via a `width_ratio`/`height_ratio` reconciliation pass) that packs five logical regions — top header strip, left modulation-source rail, central synthesis canvas, right voice / global section, and bottom keyboard — onto a dark, neutral background with hot per-section accent colours. The system is built on a JSON-driven palette ("skin") that fully parameterises colours, sizes, paddings, and per-section overrides, so theming is data, never code.

### Visual identity & palette

The default palette is unambiguously dark-mode with violet as the brand accent. Core tokens (alpha is the leading byte in the source `aarrggbb` hex):

- Background: `#262A2D`, Body: `#4C4F52`, Body Heading Background: `#3E4245`, Widget Background: `#1D2125`, Popup Background: `#1D2125`, Label Background: `#3E4245`
- Border (very low alpha): `#08FFFFFF`, Shadow: `#66000000`, Overlay Screen: `#22000000`, Lighten Screen: `#22FFFFFF`
- Heading Text: `#DFDFDF`, Body Text: `#D3D6D6`, Preset Text: `#FFFFFF`, Text Editor Caret: `#AAACAD`, Text Editor Selection: `#1FAAABAB`
- Primary brand accent (`Widget Primary 1` / `Rotary Arc` / `Icon Button On` / `Power Button On`): violet `#AA88FF`, with hover `#BDA3FF` and press `#906DE9`
- UI Action Button: `#AA88FF` (overridden teal in browsers, see below)

Each section overrides the brand accent with its own hue, producing an unmistakable colour code across the canvas:

- Oscillator / Sample / Sub: violet `#AA88FF` (inherits global)
- Filter (synth side) and Effects Filter: amber `#FFB74D` paired with a secondary `#FF8180` for a gradient fill
- Envelope, LFO, Random LFO, Modulation Matrix: teal `#1DE9B6` -> blue `#40C4FF` gradient with a bright center-line `#64FFDA`
- Compressor: teal `#1DE9B6` over deep teal secondaries `#10735B` / `#0B4039`
- Delay: pink `#FF99E9`; Distortion: red `#FF5252`; Flanger: yellow `#FFD740`; Phaser: cyan `#40CFFF`; Chorus: violet (inherits); Reverb: indigo `#8FA0FF`; Equalizer: cream `#FFF6E1`; Header banner accents: red `#FF1744`/`#FF5252` over `#161718`
- Modulation drag-drop state: green `#00E686` / blue `#0086E6` gradient with a 22% white "lighten screen" wash `#5637BEAC` while dragging; the in-flight connection paints a thick line in `#EA1616` so the user immediately sees it diverges from any real audio path
- Modulation meter rings around knobs: bipolar uses `#1DE952` (left) / `#1DC2E9` (right) with control overlay `#64FFDA`

Glow and shadow are kept deliberately minimal: a soft `#66000000` drop shadow under every panel (drawn as four side-gradient rects plus four radial corner gradients in `paintTabShadow`), and a per-widget rounded-rect background that uses a fragment shader to bake an inner soft fill so knobs and buttons feel slightly recessed without hard borders.

The **skin** itself is the canonical contract. It is a flat JSON dictionary with ~60 colour keys, ~40 numeric "value" keys, and an `overrides` map keyed by section name (`Oscillator`, `Filter`, `Envelope`, `Compressor`, ...). Both colours and sizes are parameterised — there is no magic hardcoded number anywhere in the widgets; every paint call resolves through `findColour(Skin::kRotaryArc, true)` and `findValue(Skin::kKnobBodySize)`. A new skin replaces the whole look wholesale.

**egui mapping.** Define a `Theme` struct with the same key set, deserialise from JSON with `serde_json`. Build a `SectionOverrides` enum mirroring the section list (Oscillator, Filter, Envelope, etc.) and resolve colours through a `theme.color(section, ColorId)` accessor that falls back to global. Store hex strings as `egui::Color32::from_rgba_premultiplied`. Push a `Visuals` snapshot per panel using `ui.style_mut().visuals = ...` or use `egui::Frame::none().fill(theme.body(section))` for explicit per-panel framing. Hot-reload the JSON in dev builds with `notify` watching the skin file path.

### Typography

Three font families ship by default: **Lato** (Regular + Light) for body text and parameter labels, **Montserrat** (Light + Regular) for section titles, and **Droid Sans Mono** for numeric value readouts inside knob popups and the modulation meter overlay. A monospace track is used wherever digits matter (parameter values mid-drag, envelope time displays, FFT cursor readouts) so digit widths don't jitter.

Hierarchy (point sizes are skin-driven, defaults shown after `size_ratio` of 1.0):

- Section title: Montserrat Light, ~14pt, sentence-case, centred in a `Title Width = 32` strip. Some sections (the vertical voice rail) render the title rotated 90 degrees inside that strip.
- Panel heading text: same family, ~11-13pt depending on the section override (`Button Font Size` = 11 globally, 14 in Oscillator / Modulation Matrix / Sample).
- Parameter label: Lato Light, ~11pt (`Label Height`), centred under each knob inside a rounded `Label Background` capsule of height 18 and rounding 9.
- Parameter value (knob hover popup): Droid Sans Mono, ~15pt (`Text Component Font Size`), white on `Text Component Background` `#2C3033`.
- Preset name (header): Lato Regular, scaled to the preset-selector height, in `Preset Text` white.
- Tooltip / popup display: Lato Light, ~13pt.
- Letter-spacing: native; no extra tracking. Case: section titles are typically rendered with the strings as-defined (sentence case for synthesis sections, UPPERCASE for the bottom-rail labels `BEND`, `PORTAMENTO`, `VOICE`).

**egui mapping.** Load Lato, Montserrat, and Droid Sans Mono via `egui::FontDefinitions::families` and assign them to four named `TextStyle` slots: `Heading`, `Body`, `Monospace`, and a custom `TextStyle::Name("Title".into())`. Build a helper `RichText::new(label).text_style(TextStyle::Name("Title".into())).size(theme.value(SkinValue::TitleFontSize) * scale)`.

### Layout grid

The window is divided in `FullInterface::resized()` against `kDefaultWindowWidth = 1400` / `kDefaultWindowHeight = 820`, then everything below is laid out in proportion to a `size_ratio` derived from current bounds. Key zones:

- **Top strip** (`kTopHeight = 48`): logo, preset selector with prev/next arrows, oscilloscope + spectrogram thumbnails, tab buttons.
- **Left vertical rail** (`Modulation Button Width = 68`, padded by `Large Padding = 10` either side): the eight modulation-source buttons stacked top-to-bottom (envelopes 1-3, LFOs 1-3, random LFOs, macros), with a small bend / mod-wheel section at the bottom.
- **Centre canvas** (split into `350 x size_ratio` per oscillator column, +`Padding = 4` gutter): hosts the active tab — Synthesis (3 oscillators + 2 filters + sub + sample), Effects bus, Modulation Matrix, or Master Controls. Always the same bounds, sections swap inside.
- **Right column**: modulation destination panels (envelope visualisers, LFO visualisers, random LFOs).
- **Bottom strip**: Voice section + Portamento section (each one `Knob Section Height = 71` tall), and a MIDI keyboard occupying the remainder (height ~50px, ~70% of knob-section height).
- **Vertical separator bars**: 6 * padding = 24px wide `Body`-coloured rectangles drawn between the modulation rail and the synthesis canvas, and between the canvas and the modulation destinations.

Padding tokens are also skin-driven: `Padding = 4`, `Large Padding = 10`, `Widget Margin = 5.5`, `Body Rounding = 4`, `Widget Rounded Corner = 9`, `Label Rounding = 9`. The most important relationship: **every visible panel uses the same 4px rounded-rect chrome with a 24px tall heading band** of `Body Heading Background` clipped to the top.

Z-order is strict and one-way: backgrounds (single offscreen `Image` re-rasterised on resize) at z=0, sub-section bodies + shadows on top, OpenGL line/quad layers above those, modulation-overlay components on top of widgets, and finally `setAlwaysOnTop(true)` modals: popup browser, save dialog, popup selectors, popup displays (parameter tooltips), download / auth overlays. The "Overlay Screen" `#22000000` dims everything behind modals.

**egui mapping.** Use a top-level `egui::TopBottomPanel::top("header")`, `SidePanel::left("mod_rail")`, `SidePanel::right("destinations")`, `TopBottomPanel::bottom("keyboard_bar")`, then `CentralPanel` for the tab canvas. Maintain an `AppLayout` struct with constants (`TOP_HEIGHT = 48.0`, `MOD_RAIL_WIDTH = 68.0`, `KNOB_SECTION_HEIGHT = 71.0`), and a `size_ratio` recomputed from `ctx.screen_rect()` to scale everything. For overlays use `egui::Area::new("modal").order(Order::Foreground)` plus a full-screen `Painter` that fills with `Color32::from_black_alpha(34)` to mimic the overlay dim. Z-order in egui is by layer/order; pre-allocate layer IDs (`background`, `panels`, `widgets`, `modulation_overlay`, `modal`) and pin paints to those.

### Widget vocabulary

**Knob.** The dominant control. Drawn entirely on a GPU quad with a custom fragment shader (`kRotarySliderFragment`) so it is crisp at any scale. Geometry: a round body of `Knob Body Size = 40` px with a subtle inner shadow and 1px border (`Rotary Body Border #4C4F52`), surrounded by a thin **arc** (`Knob Arc Size = 32`, `Knob Arc Thickness = 2.4`) drawn from `-0.8π` to `+0.8π`. Inside the arc is a short white "hand" line at `Knob Handle Length = 0.5` of body radius pointing to the current value. Sizes scale; some sections override `kKnobBodySize` for compact macro knobs. Drag axis is `RotaryHorizontalVerticalDrag` — vertical drag dominates, with horizontal drag also accepted; ctrl/cmd halves sensitivity (`kSlowDragMultiplier = 0.1`); double-click resets to default; right-click opens a popup with `Default Value / Manual Entry / MIDI Learn / Clear MIDI / Clear Modulations / Per-modulation route entries`. Hover state lights up a popup tooltip with the current value. Bipolar knobs render the arc centred on 12 o'clock with a `Widget Center Line` accent. Around each modulatable knob, a second concentric arc (`Knob Mod Meter Arc Size = 39`, thickness `3.0`) shows the live modulated value with `Modulation Meter` teal `#1DE9B6` (split left/right green/cyan in stereo mode). A third even thinner arc (`Knob Mod Amount Arc Size = 40`, thickness `0.5`) shows the per-route amount.

**Button.** Five styles in `OpenGlButtonComponent::ButtonStyle`: `kTextButton` (rectangular toggle with on/off colours and rounded corners `Widget Rounded Corner = 9`), `kJustText` (label only, transparent background), `kPowerButton` (filled circle, off `#606265` / on uses the section accent — violet, amber, teal, etc.), `kUiButton` (action button: solid pill with hover/press states), and `kLightenButton` (semi-transparent fill that becomes more opaque on hover). All buttons animate hover linearly (`kHoverInc = 0.2`) and use the section's `Icon Button On / Off / Hover / Pressed` colour quartet.

**Slider (non-knob).** Linear sliders (`Slider Width = 24`) are used sparingly — bend amount, mod-wheel display, and inside the equalizer / compressor editors. Track is `Linear Slider Unselected #262A2E`, filled portion uses `Linear Slider` `#848789` (or the section accent), thumb is white circle (`Linear Slider Thumb #FFFFFF`).

**Tab / section header.** `Body Heading Background #3E4245` strip clipped to the top of each panel, with the section name centred (or rotated for vertical rails). Top-level tabs in the header use `OpenGlToggleButton` with `kJustText` style and a thin underline accent when active.

**Popup / dropdown.** `PopupList` renders inside a `Popup Background #1D2125` panel with a 4px border (`Popup Border #000000`), rows 24px tall, hover row gets a `Lighten Screen` overlay. Selected row inherits `Popup Selector Background #2C3033`. Has its own `ScrollBar` rendered as a thin rounded thumb on the right.

**Text entry.** Knob right-click "Manual Entry" instantiates an `OpenGlTextEditor` inset inside the knob area (60% of knob height by default), no outline, `Text Editor Background #2C3033`, monospace, white text, caret `#AAACAD`.

**Mod-source draggable handle.** Each `ModulationButton` (in the left rail) is a pill-shaped control with the source name, a faint border, and on `mouseDown -> mouseDrag` enters `kDraggingOut` state: it paints a bright drag image (a tiny inverted-triangle "drop here" arrow plus a thick line `#EA1616` from source to cursor) and any knob hovered receives a teal/green halo (`#00E686` -> `#0086E6` `Modulation Drag Drop` palette) indicating it is a valid drop target. Drop completes the route.

**LED / activity indicators.** Tiny meter quads next to each modulation button show source activity (envelope output, LFO phase, etc.) using `Modulation Meter` colours; voice activity per keyboard note is drawn as a luminance pulse on the key.

**egui mapping.** Build a `custom_knob(ui, &mut value, options)` widget that allocates a square `Rect`, takes a `Response` (`ui.allocate_response(size, Sense::click_and_drag())`), and paints to `ui.painter()` with `Shape::circle_filled`, `Shape::CubicBezier`/`Shape::Path` for the arcs (use `Stroke::new(thickness, color)` and `PathShape::line(points, stroke)` after sampling an arc from `-0.8π` to `+0.8π`), and a single `Shape::line_segment` for the hand. Drive double-click reset via `response.double_clicked()`, manual entry via `if response.secondary_clicked() { open_text_entry = true; }`. For the modulation rings, paint additional arcs above the main arc using sampled `pos2`. For buttons, use `egui::Button::new(text).rounding(9.0).fill(theme.color(section, ColorId::IconButtonOn))` for toggles, and a custom widget for the power button (a filled circle plus a stylised `⏻` glyph). For drag-and-drop, set `response.dragged()` to enter a `DragState` stored in `Memory`; in the central canvas frame loop, query each knob's `Rect`, compute hover against the pointer position, and paint a `Color32::from_rgba_unmultiplied(0, 230, 134, 80)` halo when valid. The drag-line is just `painter.line_segment([source_rect.center(), pointer], Stroke::new(2.0, drag_color))`.

### Real-time visualizations

The synth uses one persistent `OpenGLContext` with continuous repainting (`setContinuousRepainting(true)`, swap interval 0) and a small zoo of GPU components:

- **Oscilloscope** (header, ~256px wide): renders the master output's `kResolution = 512` poly_float buffer as a single 512-segment line via `OpenGlLineRenderer`, coloured `Widget Primary 1`, drawn at every frame.
- **Spectrogram / FFT** (header, beside oscilloscope): 14-bit FFT (`kAudioSize = 16384`) windowed and converted to log-frequency bars from `9.2 Hz` to `21 kHz`, with a `3 dB/octave` tilt for pink-noise visual flattening, decay `0.008`, displayed as a smoothed line + filled area in the section accent.
- **Oscillator wave display** (per oscillator, `Wavetable3d`): the marquee visualisation. Renders the active wavetable as a **stacked-frames 3D view** by default — 256 background frames at low alpha behind the current frame, tilted using `Wavetable Vertical Angle = 1.22 rad`, `Horizontal Angle = -0.24 rad`, `Draw Width = 0.72`, `Wave Height = 0.083`, `Y Offset = 0.05`. Three render modes: `kWave3d`, `kWave2d` (flattened current frame at 0.25 height ratio), `kFrequencyAmplitudes` (live FFT of the active wave). Animated continuously. The current frame is highlighted at full saturation with a thick `Position Line` overlay.
- **Filter response curve**: `kResolution = 512`-point response of the actual filter model (Analog SVF, Dirty, Ladder, Digital, Diode, Formant, Comb, Phaser ...) at `kDefaultVisualSampleRate = 200000`, draggable in X (cutoff) and Y (resonance) directly on the curve.
- **Envelope curve** (live): the `EnvelopeEditor` draws the ADHSR/DAHDSR envelope as a smooth path with 98 points per section and 4 sections (`kTotalPoints = 393`). Each junction has a draggable handle (`kMarkerWidth = 9`) and each segment has a draggable "power" handle for curve concavity. A live position dot tracks the active voice with `kTailDecay = 0.965`.
- **LFO curve** (live): `LineEditor` with hand-drawn segments, draggable point handles, snap-to-grid; the live phase is rendered as an animated highlighted segment, with the playhead's frequency controlling the animation speed (`kSpeedDecayMult = 5.0`).
- **Voice activity / VU**: meters next to each modulation source pulse with the current output; the master output draws a `peak_meter_viewer` to the right of the spectrum.
- **Bar / harmonic painter** (wavetable editor): a `BarRenderer` shows per-harmonic amplitude and phase as bars that can be drag-painted to sculpt the spectrum.

**egui mapping.** Wrap every visualizer as a `CustomWidget` that calls `ctx.request_repaint()` every frame while visible. Sample the audio buffer into a ring (lock-free SPSC `rtrb::RingBuffer<f32>` written by the audio thread, read by UI). For the oscilloscope, `painter.add(PathShape::line(points, Stroke::new(1.5, accent)))`. For the spectrogram, run `realfft` on a 16384-sample window, convert to dB, paint as a line + a translucent fill polygon. For the 3D wavetable, project (x, frame, amp) to 2D using a precomputed `Mat3` derived from the two angle parameters, then paint each frame as a line — drop alpha linearly with depth. For the envelope and LFO, paint a sampled curve and overlay `Shape::circle_filled` handles; route mouse events through `ui.interact(handle_rect, id, Sense::drag())`. Throttle FFT computation with an `Instant::now()` check (~60Hz is enough).

### Modulation matrix UI

Routing has **two complementary UIs**. The first is the **direct, drag-driven view**: each modulation source (the pill buttons in the left rail) is grabbed and dragged onto any knob. While dragging, the `ModulationManager` paints (a) a vivid `#EA1616` line from the source button to the pointer, (b) a teal/green halo around every modulatable knob in the canvas (drawn as low-alpha overlay quads), and (c) on hover a preview modulation arc on the target knob. Release commits the route, and a small **modulation-amount sub-knob** appears next to the destination knob. The destination knob now shows a third concentric arc indicating modulation depth, the meter arc inside shows the live modulated value, and an inner colour-tinted ring (`Widget Accent 1 = #19AA88FF`, very low alpha) marks it as modulated even at rest.

The second view, the **Modulation Matrix tab**, is a wide table accessible from the header. Each row is a routing slot with: source dropdown, destination dropdown, an amount knob, polarity/stereo toggles, a bypass button, and a small live-value meter. Rows can be re-ordered. Right-click on a route's amount knob exposes `Disconnect / Bypass / Bipolar / Stereo`. The matrix view uses the same teal/blue gradient palette as the LFO/envelope sections.

For each connection there is also a per-source **callout**: hovering a source button briefly expands a panel of all its current destinations with their amount knobs in a `ModulationExpansionBox`, letting the user adjust depths without leaving the source.

**egui mapping.** Maintain a `Modulations { connections: Vec<ModConnection> }` state. Use `egui::Memory` to track a `DragGesture { source_id, started_at }`. During a drag, in the central panel's frame, paint the drag line via a foreground `Painter` and overlay halos on all `modulatable_widget_rects`. On `pointer.released()`, hit-test against those rects and call `modulations.connect(source, target)`. Render the per-knob mod meter arc by reading the latest mod value from a shared `Arc<Mutex<HashMap<ParamId, f32>>>` (or lock-free `dashmap`) populated by the audio thread. The Matrix tab is a `egui::Grid` or `TableBuilder` (from `egui_extras`) with one row per connection; each row hosts dropdowns (`ComboBox::from_id_source`), the same `custom_knob` for amount, and three small toggle buttons.

### Wavetable editor UI

Opening a wavetable's edit overlay fills the entire window with a focused editor:

- **Top strip** (~48px): preset-selector for wavetable name with prev/next, an exit-X button, a menu icon (`Paths::menu`), and a settings gear (`Paths::gear`).
- **Component list (left)**: a vertical list of `WavetableComponent` blocks — each source (Wave Source / Line Source / File Source) and each modifier (Phase Modifier, Frequency Filter, Slew Limiter, Wave Fold, Wave Warp, Wave Window) shown as a coloured pill that can be added, removed, reordered.
- **Timeline (top of the edit area)**: `WavetableOrganizer` shows each component's keyframes on a horizontal timeline (one row per component), with a draggable `WavetablePlayhead` indicating the current frame, plus an info readout (`WavetablePlayheadInfo`) of the frame index.
- **Frame editor (centre)**: three stacked surfaces depending on selection — `WaveSourceEditor` (time-domain waveform, click-and-drag to sculpt), `BarRenderer` for frequency amplitudes (drag bars), `BarRenderer` for frequency phases. Power-scale vs amplitude-scale toggleable; zoom 1x / 2x / 4x / 8x / 16x.
- **Component overlay (right)**: contextual editor for the selected component — file-source loader with three audio-import styles (Wavetable / Vocode / Pitch-Splice), math expression entry, harmonic preset picker.
- **Drop targets**: dropping an audio file on the editor opens a three-zone hint overlay (one for each import style) and previews the load style.
- **Undo/redo**: keyboard-driven via the global synth undo manager; the editor is fully snapshot-based (`stateToJson` / `jsonToState`).

**egui mapping.** Use a `Window::new("wavetable_editor").fullscreen().frame(Frame::none().fill(theme.background()))` or render in the central panel when a `wavetable_editor_open` flag is set. Compose four sub-areas with nested `SidePanel`s. The frame editor is a custom widget — for time-domain, draw the waveform as `PathShape::line` and intercept drags to overwrite the sample under the pointer (with a brush radius). For bar-renderer harmonics, draw filled rects per bar (`Shape::rect_filled`) and on drag set `harmonics[bin] = (1.0 - drag_y_norm)`. Handle audio file drops via `ctx.input(|i| i.raw.dropped_files.iter()`).

### Preset browser & saving

The preset browser slides over the synthesis canvas. Layout:

- **Search bar (top)**: text editor with magnifying-glass icon.
- **Filter chips**: category style buttons (Bass, Lead, Pad, Pluck, Keys, ...) plus author chips. Toggling is multi-select.
- **Two-column list**: rows show `Star | Name | Style | Author | Date` (`kStarWidthPercent = 0.04`, `kNameWidthPercent = 0.35`, `kStyleWidthPercent = 0.18`, `kAuthorWidthPercent = 0.25`, `kDateWidthPercent = 0.18`), 50 cached rows, row height ~4% of viewport. Hover row gets a lighten overlay, selected row gets the section accent.
- **Action buttons**: Open, Cancel, plus the Browser-section override uses teal `#1DE9B6` for the action button (replacing global violet) so commit actions feel decisive.
- **Save flow** (`SaveSection`, modal overlay 630 x 450): a name field, author field, comments field, three pages of style chips (multi-select), an "Add Folder" affordance, and Save / Cancel. Overwrite confirmation pops a smaller 340 x 160 modal.
- **User-vs-factory presets**: visually no chrome difference, but factory presets cannot be deleted (delete action hidden); a folder system overlays via `popupBrowser` with a tree of nested directories.

**egui mapping.** `egui::ScrollArea::vertical()` with virtualised row painting (only paint rows in view, indexed against scroll offset). Implement star toggle by hit-testing the star bounds in each row. Save dialog is a `Window::new("Save Preset").open(&mut save_open).resizable(false).fixed_size([630.0, 450.0])`. The chip lists are wrapped buttons via `egui::Layout::left_to_right(Align::Min).wrap(true)`.

### Interaction language

**Hover** transitions are linear, `kHoverInc = 0.2` per frame (so ~5 frames to fully lit at 60 Hz). Off colour smoothly lerps to hover colour via the OpenGL fragment shader. On press, the colour snaps to the `Pressed` variant. **Focus** on text editors highlights with `Text Editor Caret` and a thin border `Text Editor Border`. **Active modulation** is communicated by the secondary ring around a knob, not by changing the knob colour itself, so the underlying parameter colour stays stable.

**Animation** is everywhere but always purposeful: the LFO and envelope playheads continuously interpolate, the wavetable 3D view rotates subtly during morphing, mod amount sub-knobs ease in/out, popup tooltips fade, and a `LoadingWheel` (rotating arc, expanding width via `sinf(tick * 0.025)`) covers any async asset load. The cumulative effect is "alive but not busy".

**Drag-and-drop visual feedback**: dragged source paints a `Modulation Drag Drop`-coloured halo on every valid destination, an `EA1616` red drag line follows the cursor, and the dragged source itself dims with `Modulation Button Dragging` colour. Dropping outside any target cancels.

**Right-click as a uniform second action**: knobs -> manual-entry/MIDI-learn/clear modulations, mod-amount knobs -> disconnect/bypass/bipolar/stereo, route handles -> matrix view, presets -> open location/rename/delete, oscillator/sample -> copy/paste/init/resynthesize. The popup selector is the same `SinglePopupSelector` / `DualPopupSelector` component everywhere, so the language is consistent.

**egui mapping.** Drive hover transitions with per-widget `f32` interpolators stored in `egui::Memory`, advanced via `ctx.input(|i| i.stable_dt)` and damped toward target. `ctx.request_repaint_after(Duration::from_millis(16))` keeps animations smooth without burning the CPU when idle. Right-click everywhere via `response.context_menu(|ui| { ... })` or `response.secondary_clicked()` paired with a custom popup positioned at `pointer.interact_pos()`.

### Edge cases & polish

**Tooltips** are first-class. Every knob has a `popupDisplay(source, text, placement)` that opens a small `PopupDisplay` quad with monospace value text on hover, placed by `BubbleComponent::BubblePlacement` heuristics relative to the knob (prefer below, fall back to above near screen edges). Two display slots (`popup_display_1_`, `popup_display_2_`) exist so a primary value and a secondary modulation hint can coexist.

**Disabled / inactive** widgets desaturate. Each colour pair `kRotaryArc` / `kRotaryArcDisabled`, `kLinearSlider` / `kLinearSliderDisabled`, `kWidgetPrimary1` / `kWidgetPrimaryDisabled #4C4F52`, `Power Button Off #606265` provides the cold variant. Modulation bypass desaturates the amount knob via `Colour::withSaturation(0.0f)`.

**Error / out-of-range**: parameter text entry that fails to parse simply rejects and reverts. There is no inline error toast — invalid manual entry just clears the editor. Preset save with an empty name disables the Save button. Skin load failure falls back to `loadDefaultSkin`.

**High-contrast / accessibility**: the synth ships with no dedicated accessibility palette and no screen-reader hooks; contrast against `#262A2D` background is consistently >4.5:1 for primary text but knob arcs against widget backgrounds can drop below WCAG AA at small sizes. For the Rust rewrite, we should add an alternate high-contrast skin (boost arc thickness, raise label background opacity, use `#FFFFFF` for active states), expose `egui::Context::set_visuals` swapping, and tag every widget with `WidgetType` so a future screen-reader integration via `accesskit_egui` is possible.

**egui mapping.** Tooltips: `response.on_hover_text(format_value(value))` for free, or a custom hover popup via `Tooltip::for_widget(...)` for the styled monospace look. Disabled state: `ui.add_enabled(active, custom_knob(...))` plus `theme.color_dim(...)` resolvers. Skin reload: keep a `Theme` in `eframe::App` and replace it; call `ctx.request_repaint()` after to force a re-layout. AccessKit: enable `egui` accessibility feature, attach roles (`Role::Slider`, `Role::Button`) on each custom widget via `response.widget_info`.

---

## 9. Wavetable Creator Pipeline

The Creator is the offline graph that turns user intent — a hand-drawn line, an imported sample, or a stack of stored frames — into the data structure the audio thread will read in §4. Nothing here runs on the audio thread; everything renders ahead of time into a double-buffered `Wavetable` that the player atomically swaps in. The pipeline is built as **groups of components**, each component being either a *source* (produces a wave frame from nothing) or a *modifier* (mutates the wave frame in place). The Creator iterates positions 0..255, walks every component in order, mixes groups, and writes the resulting 2048-sample buffer plus its spectrum into the live wavetable.

### 9.1 Sources: Wave, Line, File

A **Wave Source** keyframe stores a literal 2048-sample buffer. The whole frame is the data; rendering is a memcpy followed by an FFT to populate the frequency-domain mirror. This is the source used by the predefined sin/triangle/square/saw set; it is also where round-trip imports end up after resynthesis. The Rust port keeps two parallel buffers per keyframe — `time: [f32; 2048]` and `freq: [Complex32; 1025]` — and reuses one `realfft::RealFftPlanner` for both directions.

A **Line Source** keyframe is a `LineGenerator`: an array of up to 100 control points `(x, y)` with per-point power-scale curvature. On `render()` the generator rasterises into a `[f32; 2048]` buffer, then the keyframe maps `[0, 1]` to `[-1, 1]` and FFTs. Interpolating two line keyframes interpolates **points and powers**, not the rasterised buffer — which is what lets the morph look like a moving cursor rather than a fade. A `pull_power` per keyframe biases the interpolation curve (more time spent near `t = 0` or near `t = 1`). In Rust, the line generator is just `Vec<(f32, f32)>` plus `Vec<f32>` powers; the rasteriser is a hand-rolled hot loop with `evalexpr` or `exmex` deliberately not pulled in — the math is small and adding a parser is overkill.

A **File Source** keyframe is a *cursor into a shared PCM buffer*, not a copy. Each keyframe stores `start_position` (sample offset), `window_size` (one cycle's length in samples), and `window_fade`. Rendering reads `window_size` samples from the parent buffer with Catmull–Rom cubic interpolation, then resamples them into 2048-sample land. Three import modes feed this source:

- **Wavetable mode** assumes the file is already a serial wavetable. `detectWaveEditTable()` checks the 2048-byte heuristic — if the buffer length equals 256 × 64 (the common wave-edit layout) and the FFT of frame 0 has more energy in the multiples of 8 than in the last bin, it sets the window size to 256 and the keyframes splice the file into N adjacent cycles. The fade style controls whether successive frames *blend* (Hann crossfade), hold (no-interpolate), or interpolate in time/frequency.
- **Vocode mode** runs the YIN-like pitch detector to find one cycle, then sets a per-frame randomised phase array. Each keyframe still pulls amplitude from the file's FFT, but `writePhaseOverrideBuffer()` overwrites every bin's phase with samples from a seeded `RandomGenerator` ranging over `(-π, π]`. Same seed in / same phases out, but every harmonic gets a random starting angle — the perceptual signature of vocoded wavetables.
- **Pitch-Splice mode** also runs pitch detection, then chops the buffer at multiples of the detected period. `kWaveBlend` is the default fade style, which does a half-cosine crossfade between adjacent windows so zero-crossings stitch without clicks.

There is no math-expression source in this codebase, so the Rust port should not pull in `evalexpr` / `exmex` for the source layer. (A future "Code Source" could; if it lands, hand-roll a tiny expression parser keyed to `sin`, `cos`, `tri`, `saw`, `t`, `n` — the surface area is too small to justify a dependency.)

### 9.2 Modifier chain

Every modifier follows the same shape: it owns a `Keyframe` per wavetable position with its own parameters, those parameters are linearly interpolated between adjacent keyframes, then `render()` is called on the wave frame produced by the upstream component. Order in the group matters — modifiers compose left-to-right.

- **Phase Modifier** operates entirely in the frequency domain. For each FFT bin it multiplies by `e^{-iφ_k}` where `φ_k` is one of: `k·phase` (normal — linear ramp across harmonics, equivalent to a time shift), `phase` (harmonic — same shift to every bin), even/odd variants that invert sign on odd harmonics for hollow-pulse shapes, and `clear` which zeroes every bin's imaginary part. A `mix` parameter blends each result with the unprocessed bin. Rust: this is one pass over `&mut [Complex32; 1025]`.
- **Frequency Filter** multiplies each bin's complex value by a real gain curve. `cutoff` is `2^c` so the slider is in octaves; `shape` morphs slope and shape. Four styles: low-pass `1 - slope·(k - cutoff)` clamped, band-pass `1 - |slope·(k - cutoff)|`, high-pass mirrored, and a comb whose period is `2·cutoff` and power scaling tilts the comb tooth. Optional time-domain renormalise after the FFT-back. Pure scalar math, no library needed.
- **Slew Limiter** runs in the time domain with separate up-rate and down-rate per sample. To avoid edge artefacts it scans `[0, 2·N)` so the limit wraps around the loop boundary — by the second pass the value has converged. The Rust port is a single-pass `for` loop over `[f32; 4096]` with a final wrap copy.
- **Wave Fold** computes `sin(boost · asin(x/peak))`, the classic foldback. The `boost` parameter is the only modulating value; `peak` is `max(|x|)` of the input. Past `boost = π/2` the signal starts mirroring back, giving the bright sidebands.
- **Wave Warp** is two power-scale curves — one for the horizontal (phase) axis, one for the vertical (amplitude). Each axis can be asymmetric or symmetric. The symmetric form `power(2t-1) / 2 + 0.5` keeps the centre at 0.5 and warps endpoints. The reusable kernel is `(e^{p·t} - 1) / (e^p - 1)` — Rust just exposes it as a `pow_scale(value, power)` helper.
- **Wave Window** multiplies the time-domain buffer by a window function tapered between a `left_position` and `right_position`. Four window shapes: half-cosine, half-sine, square (gate), and a "wiggle" shape `t·cos(π·(1.5t + 0.5))` that introduces a ripple. Useful for pulse-width-style sweeps.

In Rust, the modifier trait is one function `fn render(&self, frame: &mut WaveFrame)` plus an `interpolate(from, to, t)`; everything else (JSON, position) is on the base trait. Use `enum_dispatch` or an `enum Modifier` rather than `dyn Trait` to keep dispatch on the inline path.

### 9.3 Frame interpolation across the wavetable position axis

A user typically authors 2-8 keyframes; the Creator interpolates these to fill all 256 frames. Each `WavetableComponent` carries an `InterpolationStyle`: `None` (hold previous), `Linear` (Hermite endpoint), or `Cubic` (Catmull–Rom, four-point stencil). For sources rendering generic wave frames the interpolation is performed **per parameter** (line points, file cursor positions, modifier params). For raw sample buffers (Wave Source) there's a second choice — time-domain or frequency-domain.

Time-domain interpolation tweens each sample directly. Catmull–Rom uses the previous, current, next and next-next keyframes weighted by their position ranges so unequally-spaced keyframes still smooth. Frequency-domain interpolation tweens **amplitude** and **phase delta** independently per bin: amplitudes are linearly interpolated in `sqrt` space (so equal-energy crossfades feel uniform), and phase is integrated from the previous frame's phase plus the per-step phase delta `arg(conj(prev) · curr)`. This avoids the catastrophic phase wrapping a naïve linear `arg()` interp produces. The cubic variant builds phases left-to-right by accumulating the three deltas and Catmull-Rom-smoothing the accumulator. Rust port: `realfft` for the round-trip, store `(amp, phase)` pairs alongside the complex bins for sources that morph in spectrum mode.

### 9.4 Building the spectral mip-map

Once the 256 time-domain frames exist, the player needs them as harmonic triplets (amplitude, normalised-frequency, phase) so it can reconstruct each frame additively and band-limit by bin (covered in §4). The Creator's path is straightforward: every frame already had `toFrequencyDomain()` called on it during render, so `loadWaveFrame()` only has to translate the `Complex32` bins into three parallel arrays — `frequency_amplitudes[bin]`, `normalized_frequencies[bin]` (unit complex `e^{i·arg}`), and `phases[bin]` (the raw angle, doubled for SIMD pairing).

The interesting step is `postProcess()`. After all 256 frames are loaded it walks each harmonic top-down across frames, looks for the first frame where amplitude exceeds 0.1, anchors that phase, then for every subsequent above-threshold frame it computes the wrapped delta to the previous anchor and **linearly interpolates the normalised frequency for all frames in between** — i.e. it phase-unwraps along the position axis while ignoring frames where the harmonic is essentially silent (and therefore would have a garbage phase). Trailing silent frames inherit the last anchor. The same `max_span` it tracked during render is used here to apply a global gain so the loudest frame just reaches ±1.

The Rust equivalent stores three `[f32; 1025]` arrays per frame (or `[Complex32; 1025]` for the normalised version) and runs the post-process pass as a simple double loop. `realfft::RealFftPlanner<f32>` produces the bins; the only subtle bit is `Complex32::arg()` on near-zero magnitudes — guard with a `kMinAmplitudePhase` threshold matching the C++ constant.

### 9.5 Random source baking for spectral randomness

The "spectral randomness" knob in vocode mode is **not** a runtime modulator — it is pre-baked at preset save time. The File Source stores an integer `random_seed`; `writePhaseOverrideBuffer()` reseeds a uniform `(-π, π]` generator and fills a per-bin phase array. That array is held with the component and replayed every render. Changing the phase style increments the seed so successive toggles don't collapse to identical phases. Because the seed survives the JSON round-trip, the random pattern is byte-identical after reload. Rust: `rand::rngs::StdRng::seed_from_u64`, fill once into a `[f32; 2048]` cached on the component, never touch it from the audio thread.

### 9.6 Audio file import

The interface layer hands raw `f32` samples plus a sample rate into `initFromAudioFile()`. Decoding is the C++ framework's job — WAV / AIFF / FLAC are handled by the framework's reader factory which probes the stream, exposes `sampleRate`, `lengthInSamples`, and `numChannels`. Stereo files are read into a multi-channel buffer but only channel 0 is passed forward, so stereo→mono is a *drop the right channel* (not a sum). Length is normalised once — leading silence is stripped by `getFirstNonZeroSample()` before any analysis. The pitch detector (YIN-style) takes the centre of the buffer, scans periods from 300 samples up to a configurable max, scores each period by comparing two consecutive cycles at 2520 ÷ N sample points (squared diff plus a DC-delta penalty), then refines the best match with a 0.1-sample sweep ±1 sample around it. The result becomes the `window_size`. Rust: `symphonia` for WAV / AIFF / FLAC (broad format coverage, no FFmpeg dep) or `hound` if you only need WAV. The pitch detector is ~80 lines of straight numeric code.

### 9.7 Serialization

A wavetable preset is a single JSON document. At the top: `name`, `author`, `version` string, two booleans (`remove_all_dc`, `full_normalize`), and an array of `groups`. Each group is `{components: [...]}`. Each component is `{type, keyframes, interpolation_style, ...params}` where `type` is the component name string. Keyframes carry their own parameters plus an integer `position`.

Two binary payloads sit inside the JSON as base64 strings:

- **Wave Source keyframes** store `wave_data` — the literal 2048 `f32` samples, base64-encoded as raw `f32` little-endian. Older versions encoded as PCM-16 and the loader's version migration runs `convertPcmToFloatBuffer` for files older than `0.3.9`.
- **File Source** stores `audio_file` — the parent PCM buffer trimmed to `max_position + 2·window_size + 4` samples, quantised to `int16_t`, base64-encoded. Cheaper than full float, and the file is going to be FFT'd anyway so 16-bit precision is fine.

`updateJson()` does inline version migration: pre-`0.3.3` files used integer component types (mapped through a fixed string table); pre-`0.3.7` audio was raw float (converted to PCM); pre-`0.7.7` line points were `[-1,1]` and excluded endpoints (rebased to `[0,1]` with synthetic endpoints). Each migration is gated on `compareVersionStrings`.

Rust port: `serde` + `serde_json` with `#[serde(tag = "type")]` on the component enum, base64 via the `base64` crate, version field a string. Migrations are a chain of `fn migrate_0_3_3(v: &mut Value) -> Result<()>` functions that operate on the untyped `serde_json::Value` before strict typed deserialisation. Don't reach for `rkyv` / `bincode` — preset files are human-readable, edited by hand by patch designers, and exchanged across versions; binary buys nothing.

### 9.8 Resynthesis from a sample — full walkthrough

Three modes share the entry point `initFromAudioFile`. All clear the creator and add a single group containing one `FileSource`.

**Wavetable mode (`kWavetableSplice`)**: load the buffer; `detectWaveEditTable()` checks for the 256×64 layout and sets `window_size = 256` if it matches, otherwise leaves the default 2048. Add two keyframes (position 0 and last). Keyframe 0 starts at sample 0; keyframe 1's start is clamped to `num_samples - window_size`. Fade style is whatever the caller passed (typically `kNoInterpolate`). On render the file is sliced into N cycles of `window_size`, each cycle becomes a frame.

**Vocode mode (`kVocoded` / `kTtwt`)**: pitch-detect to set `window_size` (TTWT clamps to a maximum period of 20 ms at the input sample rate, which biases toward speech). Add two keyframes spanning the whole buffer; `setSamplesNeeded()` accounts for the crossfade region. Phase style is `kVocode`, so on every render the per-frame FFT's magnitudes are kept but the phases are replaced from the seeded random buffer. The result is a smooth amplitude morph from start to end with phase coherence randomised — characteristic vocoder/granular texture.

**Pitch-Splice mode (`kPitched`)**: pitch-detect, same two-keyframe layout, fade style `kWaveBlend`. No phase override — the FFT phases carry through. The Hann-crossfade at the cycle boundary is what makes the splice clickless. This is the path used internally by resynthesise-from-current-patch: render 4 seconds of synth output at MIDI note 16, pitch-detect, set `window_size = sample_rate / freq(note 16)`, then run the standard render.

### 9.9 Undo and snapshot

Undo is built on `stateToJson` / `jsonToState` at the WavetableCreator level. Every user-visible edit (add component, drag keyframe, change parameter) snapshots the entire creator to JSON and pushes it onto an `UndoableAction` stack; redo restores by re-applying. The cost is dominated by base64 encoding of any embedded PCM, which is why File Source clips the saved buffer to `max_position + 2·window_size + 4` samples — to keep snapshots small. Crucially, JSON serialisation does *not* save the rendered 256-frame output, only the inputs; restore re-runs `render()` so any change to the modifier code path takes effect on undo.

Rust port: keep undo at the same granularity. `serde_json::Value` snapshots are cheap to clone via `Arc<Value>`; the stack is `VecDeque<Arc<Value>>` with a configurable cap (~50 entries). For large File Source buffers, hash the PCM payload and intern it — most snapshots share the same audio file and only differ in keyframe parameters, so storing the audio once and referencing by hash makes undo essentially free.

---

## 10. Inter-Oscillator Cross-Modulation

The synth carries three full wavetable oscillators plus a dedicated sampler. Cross-modulation between these four sources is *not* drawn from the global modulation matrix. It is wired directly into each oscillator's per-sample phase / amplitude inner loop, and the topology — who modulates whom — is hard-coded by index in the producer module. Elixir's job is to preserve this audio-rate immediacy while replacing fixed wiring with a typed, allocation-free Rust graph.

### 10.1 Topology: fixed-pair, not free routing

Each oscillator carries two latent modulator inputs, "modulator A" and "modulator B". They are *not* user-selectable patch fields. The producer module computes them deterministically:

```
osc 0 -> A = osc 1, B = osc 2
osc 1 -> A = osc 0, B = osc 2
osc 2 -> A = osc 0, B = osc 1
```

The sampler is *always* the third modulator source for every oscillator (no routing decision required). A patch selects between A, B, or sample only by picking a `DistortionType` enum value (`FmOscillatorA`, `FmOscillatorB`, `FmSample`, `RmOscillatorA`, `RmOscillatorB`, `RmSample`). The carrier-vs-modulator role is therefore implicit in the enum, not in the matrix.

This is much narrower than a free any-to-any FM grid. The win is that the modulator buffer pointer is bound once at module init and dereferenced as a flat `poly_float*` in the inner loop — no per-sample lookup, no atomic, no virtual call.

Rust equivalent. Express modulator role as a const-resolved enum on the oscillator:

```rust
#[derive(Copy, Clone)]
enum ModSource { OscA, OscB, Sample }

struct OscWiring {
    mod_a: OscId,
    mod_b: OscId,
    sample: SampleId,
}
```

The wiring table is built once per voice and each `OscRender` holds three `&[f32]` slices into its peers' pre-attenuation output buffers. No indirection per sample.

### 10.2 The nine `DistortionType` modes

| Mode | Phase func | Window func | Modulator buffer |
|---|---|---|---|
| `Quantize`, `Bend`, `Squeeze` | distort | passthrough | none |
| `Sync`, `Formant`, `PulseWidth` | distort | passthrough / half-sin / pulse | none |
| `FmOscA`, `FmOscB`, `FmSample` | `fmPhase` | passthrough | A / B / sample |
| `RmOscA`, `RmOscB`, `RmSample` | passthrough | `rmWindow` | A / B / sample |

FM modes inject only into the integer phase accumulator. RM modes leave the phase alone and multiply the table-lookup result by `interpolate(1.0, modulator, depth)` — a bipolar ring mod that degrades to no-effect at `depth = 0` and to pure RM at `depth = 1`. The modulator is the peer oscillator's pre-attenuation output (the `Raw` port, not the `Levelled` one), so patch volume and pan don't bleed into modulation depth.

A second variant pair (`fmPhaseLeft` / `fmPhaseRight`, `rmWindowLeft` / `rmWindowRight`) is entered only when one stereo half of the SIMD lane is inactive, reading its missing pair through `swapVoices`.

### 10.3 Hard / soft sync

Sync isn't an inter-oscillator route — it's a self-distortion. `syncPhase` rescales the phase accumulator by a factor up to `kMaxSync = 16` and the bandlimited table is queried for the resulting frequency. There is no separate "master oscillator" slave-resetting another oscillator; the slave's drive frequency is encoded in the `distortion` parameter and looked up against a wavetable octave that already has the correct band-limit, sidestepping BLEP/BLIT. The Formant mode is the same `syncPhase` paired with a `halfSinWindow` so the cycle is shaped as a windowed sine-burst.

Rust: keep sync as a `PhaseFn` variant rather than a graph edge. The cap is `const MAX_SYNC: u32 = 16` and the wavetable mip-level is selected from `phase_inc * MAX_SYNC`.

### 10.4 FM depth cap and the aliasing budget

FM is the only mode that adds an unbounded signal-dependent offset to the phase accumulator. To keep aliasing tractable:

```cpp
const poly_float kFmPhaseMult = kPhaseMult / 8.0f;
const poly_int   kMaxFmModulation = 48;
phase += toInt(modulator * depth * kFmPhaseMult) * kMaxFmModulation;
```

The product `kFmPhaseMult * kMaxFmModulation` defines the maximum instantaneous phase jump per sample. A modulator of magnitude 1.0 at full depth produces a jump of `48 / 8 = 6` whole-cycles' worth of fraction — large enough for screaming FM, small enough that the wavetable interpolation between mip levels still produces a band-limited result.

Rust:

```rust
const FM_PHASE_MULT: f32   = PHASE_MULT / 8.0;
const MAX_FM_MOD: u32      = 48;
phase = phase.wrapping_add(((modulator * depth) * FM_PHASE_MULT) as u32 * MAX_FM_MOD);
```

The cap must be a `const`, not a runtime parameter, because its value is coupled to the wavetable's mip-pyramid band-limit assumption.

### 10.5 AM / RM

Ring modulation is implemented as an output window: `lerp(1.0, modulator, depth)`. Bipolar by default; at `depth = 1.0` it collapses to pure RM. There is no separate AM path — "AM" is just `Rm` with the modulator pre-offset to unipolar territory upstream (which the user controls via the modulator oscillator's wave choice and DC offset). The depth is interpolated sample-accurately across the block alongside the phase distortion to avoid zipper artifacts.

### 10.6 The "sub" question and the sampler-as-source

There is no discrete sub oscillator. The three full oscillators are symmetric; a "sub" is conventionally produced by setting one of them an octave or two below the lead and routing it through its own filter slot via the per-osc `destination` value. Elixir should preserve this — adding a fourth always-on osc just to ape vintage sub behaviour is wasted SIMD lanes.

The sampler is different: it can't be a *carrier* under FM (no phase accumulator into which to inject) but is universally available as a *modulator*. Because the sampler's output may already be band-limited or already noisy, it is the most aliasing-prone modulator source — the `kMaxFmModulation` cap is doing real work here.

### 10.7 Audio-rate vs control-rate routing

Inter-oscillator routes are strictly audio-rate. The global modulation matrix runs at the block boundary with smoothing; inter-osc routes run inside the same SIMD inner loop that produces the carrier sample, one modulator sample per carrier sample, no interpolation needed. Elixir keeps this split: a `ModMatrix` for control-rate destinations and a fixed compile-time `OscWiring` for audio-rate edges.

### 10.8 Topological ordering and feedback

FM/RM reads a peer's *current-block* output, so ordering matters. The producer module solves this with a tiny dependency-walk: a `kNumOscillators × kNumOscillators` loop scans oscillators, processes the next one whose modulator sources are already done, and skips otherwise. Three oscillators with at most two dependencies each converge in at most 9 iterations.

The topology guarantees no cycles **because each osc's two slots reference *other* oscillators only, never itself**. If two oscillators ever both selected each other as `FmOscA`, the current scheduler would silently drop the second one (its source never goes `processed`). A real footgun.

Rust — make the cycle explicit, not implicit:

```rust
enum OscEdge { Direct, OneSampleDelayed }   // matches the Feedback node in §3
```

Build the wiring graph each patch-load, run Kahn's algorithm, and for each remaining back-edge insert a single-sample-delay buffer (the `Feedback` node from §3). Two oscillators may now mutually modulate each other: A reads B's previous sample, B reads A's current sample. One sample of latency is below threshold for audio-rate FM (sub-millisecond at 48 kHz) and matches what hardware FM synths do.

### 10.9 SIMD inner-loop shape

The inner loop runs `poly_float::kSize` voices in parallel:

```
for i in 0..N:
    phase += toInt(phase_inc[i] * inc_mult)
    phase += toInt(mod_buf[i] * depth * kFmPhaseMult) * kMaxFmModulation
    sample = catmullInterp(wave_buffers, phase + dist_phase)
    out[i] += window(...) * sample
```

In Rust this maps cleanly to `std::simd::f32x4` / `u32x4`, with the wavetable lookup using gather instructions on supported targets and scalar fallback on M-series. `depth` and per-voice envelopes interpolate linearly across the block as `delta_*` accumulators, never recomputed. `Window` and `PhaseFn` selection should monomorphise per-mode through generics or enum-dispatch with `#[inline(always)]`, so the modulator-buffer dereference disappears on the no-FM/no-RM paths.

---

## 11. MPE and Per-Note Expression

In the reference C++ synth, MPE is not bolted on top of polyphonic MIDI — it is the dominant routing scheme, and the channel a note arrives on is preserved on the voice for the entirety of its life. Pitch-bend, channel pressure, and CC 74 (slide / Y-axis) all become per-voice signals that the modulation matrix can tap exactly like an LFO or envelope. Elixir adopts the same shape, with the data made explicit in Rust types and the broadcast vs. per-voice distinction lifted into the `ModSrc` enum itself.

### 11.1 Zone Layout and RPN Detection

The C++ side keeps a single `MpeZoneLayout` member on the MIDI manager and feeds every incoming controller message into an RPN detector. RPN 6 / RPN 0 configure the lower zone (master on channel 1, member channels 2..N) and the upper zone (master on channel 16, member channels 15..N descending), respectively. A predicate `isMpeChannelMasterLowerZone(ch)` / `isMpeChannelMasterUpperZone(ch)` then steers every subsequent pitch-bend, pressure and slide message — master-channel events broadcast to a channel range, member-channel events route to a single voice. Default is lower zone covering channels 2..16 with channel 1 as master.

Elixir mirrors this with:

```rust
pub struct MpeZoneLayout {
    pub lower: Option<MpeZone>,   // master = ch 1
    pub upper: Option<MpeZone>,   // master = ch 16
    pub enabled: bool,
}

pub struct MpeZone {
    pub master_channel: u8,
    pub first_member: u8,
    pub last_member: u8,
    pub pitch_bend_range: f32,    // default 48 for member, 2 for master
}
```

An `RpnDetector` consumes CC 100 / CC 101 / CC 6 / CC 38 and emits `RpnEvent::McmConfig { master_ch, member_count }` and `RpnEvent::PitchBendRange { ch, semitones }`. `MpeZoneLayout::apply_rpn` from the MIDI router rewrites zones in place — the only mutation path, so the audio thread never reads a torn layout.

### 11.2 Per-Note Dispatch

A note-on on a member channel calls `VoiceHandler::noteOn(note, vel, sample, channel)` and the channel is stamped directly onto `VoiceState::channel`. The voice picker then resolves a free voice and *seeds* it with the latest known pitch-bend, channel-pressure and slide value for that channel — so an MPE note that arrives milliseconds after the controller has already begun pressing into the keybed inherits the in-flight expression instead of jumping from zero.

```rust
pub struct MpeState {
    pub channel: u8,
    pub velocity: f32,         // 0..1
    pub lift: f32,             // note-off velocity (default 0.5)
    pub pressure: f32,         // 14-bit channel pressure
    pub slide: f32,            // 14-bit CC 74
    pub local_pitch_bend: f32, // bipolar -1..1, scaled by range
}
```

When a controller update arrives on a member channel, `MidiRouter::on_pressure(ch, value)` walks active voices and updates only those whose `state.channel == ch` *and* that are currently held. Updates carry a sample offset so the voice's `aftertouch` / `slide` trigger fires at the exact sub-block position.

### 11.3 Mod-Matrix Exposure

The C++ matrix registers per-voice control-rate outputs and exposes them by string keys: `velocity`, `aftertouch`, `slide`, `lift`, `mod_wheel`, `pitch_wheel`, `note`, `note_in_octave`, plus two generated sources `stereo` and `random`. The voice handler's `prepareVoiceValues` reads each voice's `MpeState` and stamps it into a per-aggregate-voice polyphonic register before processing.

Elixir lifts this implicit scope into the type:

```rust
pub enum ModSrc {
    // Per-voice (sampled from each voice's MpeState)
    Velocity, Aftertouch, Slide, Lift, ModWheel, PitchWheel,
    Note, NoteInOctave, Stereo, Random,

    // Global (broadcast identically to every voice)
    Lfo(u8), Env(u8), Macro(u8), RandomLfo(u8),
}

impl ModSrc {
    pub fn scope(&self) -> ModScope {
        match self {
            ModSrc::Velocity | ModSrc::Aftertouch | ModSrc::Slide
            | ModSrc::Lift | ModSrc::ModWheel | ModSrc::PitchWheel
            | ModSrc::Note | ModSrc::NoteInOctave
            | ModSrc::Stereo | ModSrc::Random => ModScope::PerVoice,
            _ => ModScope::Global,
        }
    }
}
```

`Stereo` is computed once per voice at allocation: voices indexed even get +1, odd get -1 (in C++ a literal `kLeftOne` poly-float constant interleaved into the SIMD lanes). `Random` is reseeded on every retrigger: a single uniform draw per voice on note-on, held for the voice lifetime — `Pcg32` seeded from a global counter, captured into `MpeState::random_value` inside `Voice::activate`.

### 11.4 Pitch-Wheel Range and Bipolarity

Pitch-bend values come off the wire as 14-bit unsigned and are converted to `-1.0..+1.0`. The default member-channel range in MPE 1.0 is ±48 semitones; the master-channel range is ±2 semitones. The C++ voice handler stores `kLocalPitchBendRange = 48.0f` and the matrix exposes both the raw bipolar `pitch_wheel` and a `pitch_wheel_percent` (`x * 0.5 + 0.5`) for users who want a unipolar source. Elixir stores `pitch_bend_range: f32` per `MpeZone` overridable via RPN 0, so non-MPE hosts that send a ±12 RPN still scale correctly.

### 11.5 Channel Pressure vs Poly Aftertouch

**Channel pressure** (status `0xDn`) routes via `setChannelAftertouch(ch, value)` — *every* held voice on that channel receives the update. In practice MPE allocates one note per member channel, so this collapses to a single voice; the routing doesn't assume that. **Poly key pressure** (`0xAn` with a note byte) routes via `setAftertouch(note, value, ch)` and walks active voices matching *both* the channel and the MIDI note number. Elixir keeps the split: `MidiRouter::on_channel_pressure` and `MidiRouter::on_poly_aftertouch` feed the same `MpeState.pressure` field via different entry points.

### 11.6 Master-Channel Broadcasts

When the host sends pitch-bend / pressure / slide on the master channel (1 for lower zone, 16 for upper), the C++ code broadcasts via `setZonedPitchWheel(value, lo, hi)` / `setChannelRangeAftertouch(lo, hi, value, 0)` / `setChannelRangeSlide(value, lo, hi, 0)` to every voice in the zone's member range. The "global slide" / "global bend" gesture.

```rust
fn broadcast_mpe(&mut self, zone: &MpeZone, update: MpeUpdate) {
    for voice in self.active_voices.iter_mut() {
        if zone.contains_member(voice.state.channel) {
            voice.apply(update);
        }
    }
}
```

The master-channel value is stored on a separate slot from the member-channel value (`local_pitch_bend` + `zone_pitch_bend`) so a per-note bend on channel 3 and a global bend on channel 1 sum cleanly at the voice.

### 11.7 CC Routing and 14-Bit Reassembly

Slide (CC 74) is the standard MPE high-resolution control. The C++ MIDI manager keeps `msb_slide_values_[16]` and `lsb_slide_values_[16]` and emits a 14-bit value whenever either half arrives — `(msb << 7) + lsb`, normalised by `(1 << 14) - 1`. CC 6 / 38 LSB pairs for channel pressure reassemble identically. Mod-wheel (CC 1) is per-channel and currently treated as 7-bit. Elixir mirrors with a per-channel `HiResControl { msb: u8, lsb: Option<u8> }` table and emits a single normalised `f32` whenever an MSB lands.

### 11.8 MIDI 2.0 / MPE 1.1 Forward Compat

Elixir v1 targets MPE 1.0 exclusively. The `MidiRouter` consumes a `MidiMessage` enum that already carries `channel: u8`, `note: u8`, `velocity: u16`, `value: u32`, so MIDI 2.0 per-note pitch / per-note controllers can later be routed by adding `MpeUpdate::PerNotePitch { note, channel, semitones }` variants without changing `MpeState`'s shape. Until then, MIDI 2.0 UMP packets are decoded down to MIDI 1.0 form at the transport layer.

### 11.9 Non-MPE Fallback

If the host never sends an RPN MCM configuration and `mpe_enabled == false`, every message arrives on channel 1 and Elixir collapses gracefully: channel pressure feeds *all* voices on channel 1, pitch-bend feeds every voice on channel 1, slide feeds every voice on channel 1. When `MpeZoneLayout::enabled == false`, the matrix's `PitchWheel` source reads from the global `zone_pitch_bend[0]` cell directly, skipping the per-voice walk, and `Aftertouch` / `Slide` similarly tap the global cell. One branch in `MpeState::sample()`, not a separate code path.

---

## 12. Plugin Latency, Transport Sync, and Host Integration

### 12.1 Latency reporting

The C++ code path is striking for what it does *not* do. Nowhere in the plugin processor or the sound engine is `setLatencySamples` invoked, and the override of `getTailLengthSeconds` simply returns `0.0`. The synth declares itself a zero-latency instrument. There is no lookahead limiter, no FFT analysis on the output bus, no impulse-response convolution. The one real source of group delay — polyphase oversampling in the voice path and effect chain — is treated as inaudible and is not surfaced to the host's PDC graph.

That is a conservative call with an audible cost when the synth is layered against an unprocessed dry signal: at 4x oversampling the FIR up/down-samplers introduce a few samples of group delay that the host cannot align. The Rust port should not inherit this. `nih-plug` exposes latency through `ProcessContext::set_latency_samples`. We compute the FIR's symmetric group delay once during `initialize()` (length / 2 per half-band stage, divided by the oversample factor to convert back to host-rate samples) and call `set_latency_samples` from `initialize` and again whenever the oversampling parameter changes inside `process`. The value is dynamic but quantized to a small set (one per oversampling tier).

### 12.2 Transport pull-model

Every audio block, the processor reads the play head once via `getCurrentPosition`, copies the result into a `CurrentPositionInfo`, and feeds two values forward: `bpm` is forwarded into the engine via `setBpm`, which writes a `beats_per_second` control value; and `ppqPosition` is divided by beats-per-second to produce a wall-clock seconds value `last_seconds_time_`, which is handed to the engine each sub-block through `correctToTime`. Bar number and time signature are *not* consumed; the engine has no concept of bar lines or downbeats.

Consumers of the transport value are the tempo-synced LFOs, the random LFO, the chorus / flanger / phaser modules, and the voice handler (for the `kSyncToPlayHead` retrigger style). Each implements `correctToTime(double seconds)` which writes a shared `sync_seconds_` slot; on the next block, the LFO computes `getCycleOffsetFromSeconds(seconds, frequency)` and masks it into its phase accumulator when the reset/sync mask fires.

The Rust port uses `ProcessContext::transport()` which returns `tempo`, `pos_samples`, `pos_seconds`, `pos_beats`, `time_sig_*`, `playing`. Capture this once per block, push `tempo / 60.0` into a `beats_per_second` atomic, route `pos_seconds` into the `correct_to_time` equivalent on each tempo-aware DSP node.

### 12.3 Tempo-sync subdivision math

Subdivision selection is a single integer index into a thirteen-entry ratio table from `1/128` to `16` beats per cycle. The `TempoChooser` operator multiplies the indexed ratio by `beats_per_second` to get cycles per second, then applies two optional multipliers: `2/3` for triplet, `2/3` for dotted (the dotted ratio is expressed as cycle-shortening). The output is masked against four modes — `kFrequencyMode`, `kTripletMode`, `kDottedMode`, `kKeytrack` — and the right branch is chosen per voice via SIMD masks.

A transport jump (locator move, loop boundary) causes `last_seconds_time_` to leap discontinuously. Each LFO recomputes `sync_phase = getCycleOffsetFromSeconds(seconds, frequency)` and snaps. No slewing, no crossfade, no anti-click envelope. Intentional for tight grid alignment; users who don't want it switch to free-running.

In Rust, the same table can be a `const [f32; 13]`. The `TempoChooser` becomes a small block-rate node that reads the `Transport` once per block and writes the resolved Hz into the LFO's frequency input. Phase snap-on-jump is implemented by detecting a non-linear `pos_seconds` delta (greater than one block's worth) and forcing the wrap.

### 12.4 Bypass handling

The plugin owns a dedicated `ValueBridge` registered as the host's bypass parameter via `getBypassParameter`. When non-zero at the top of `processBlock`, the processor calls the framework's `processBlockBypassed` and returns immediately — voices are *not* killed, MIDI is *not* consumed. Note tails freeze in place; on un-bypass the same voices continue. No zipper smoothing on the transition, which can click.

`nih-plug` has first-class bypass via `BoolParam` flagged with `.with_flags(ParamFlags::BYPASS)`. On a bypass transition we ramp a 5 ms crossfade between live output and the input bus copy to suppress the click. Tail-freeze vs tail-release becomes a user setting.

### 12.5 Sample-rate change

`prepareToPlay(sample_rate, buffer_size)` does three things: `engine_->setSampleRate`, `updateAllModulationSwitches` (re-evaluates control-rate cached coefficients), pushes the rate to `midi_manager_`. Buffer size is *not* propagated; the engine internally splits any block into chunks of `kMaxBufferSize`. The oversampling subsystem reacts on the next `process` call. Filter coefficients depending on sample rate are rebuilt lazily on first use after the rate change.

Rust: `Plugin::initialize(audio_io_layout, buffer_config, context)` fires whenever the host re-prepares — rebuild filter coefficients, resize delay-line allocations, re-emit `set_latency_samples`.

### 12.6 Buffer size change

The processor is robust against per-call buffer-size variability because of the internal sub-block loop: `for (sample_offset = 0; sample_offset < total_samples; sample_offset += num_samples)` where `num_samples = min(remaining, kMaxBufferSize)`. The engine never sees a block larger than its compile-time maximum. `releaseResources` is a no-op. No allocation on the audio thread tied to buffer size.

The Rust port keeps the sub-block loop (necessary for SIMD lane alignment) and pre-allocates all scratch buffers at the configured `max_buffer_size` reported in `BufferConfig`.

### 12.7 Process modes (offline rendering)

The plugin makes no distinction between realtime and non-realtime processing. Same code path whether the host is bouncing offline or streaming live. No optimization for offline (no quality-bump in oversampling, no exact tail rendering). `silenceInProducesSilenceOut` returns `false` because LFOs and self-oscillating filters can produce output from silence.

`nih-plug` surfaces process mode via `ProcessContext::process_mode()` returning `ProcessMode::Realtime` or `ProcessMode::Offline { tail_time }`. Use this to optionally bump oversampling one tier in offline mode.

### 12.8 MIDI clock / sync

No external MIDI clock support anywhere in the codebase. Tempo comes exclusively from the host transport (or a hard-coded 120 BPM fallback in the standalone build). Real-time MIDI bytes 0xF8/0xFA/0xFB/0xFC are ignored. The Rust port inherits this; if needed later it can be a second source feeding the same `beats_per_second` control.

### 12.9 Plugin parameters vs internal

The bridge object `ValueBridge` wraps every engine control whose name appears in the central parameter registry, presenting a 0.0–1.0 normalized range to the host, an indexed/continuous flag, a custom value-to-text formatter that honors the parameter's display multiplier and units, and skew curves (quadratic, cubic, quartic, exponential, square-root) for non-linear range mappings. Automation is at host block rate; smoothing is performed downstream inside the engine by the operators themselves, not at the parameter boundary. A few hundred parameters total.

The Rust port uses `FloatParam` and `IntParam` with `SmoothingStyle::Linear` or `Exponential` for audio-rate smoothing, `.with_value_to_string` / `.with_string_to_value` for the formatter, `.with_unit` for display. Skew functions move into `FloatRange::Skewed { factor }`.

### 12.10 State chunk

`getStateInformation` calls `LoadSave::stateToJson(this, getCallbackLock())` to capture every control value, every modulation connection, every LFO line generator, the wavetable creators, and the sample slot — then appends the tuning table under a `"tuning"` key. The result is `nlohmann::json::dump()`-ed to a string. `setStateInformation` reverses it: `pauseProcessing(true)`, parse, apply, tuning, `pauseProcessing(false)`, then `updateFullGui`. Errors are reported through a native alert (a UI side effect on the audio thread — a hazard the Rust port should fix).

In Rust, `nih-plug`'s `#[derive(Params)]` plus `PersistentField` with serde handles this declaratively: parameters serialize automatically, bespoke state (modulation graph, wavetable bytes, tuning table) is annotated with `#[persist = "key"]` on serde-able structs. Load errors propagate as `Result` back to the host instead of popping a modal.

---

## 13. Multi-Format Plugin Hosting in Contrapunk (Track C)

Track C is the complement to making Elixir a plugin: Contrapunk itself becomes a multi-format plugin host. A user with Elixir on slot 1, Diva on slot 2, and Pro-Q4 on slot 3 should be able to route harmony notes through that chain and hear the result, with Contrapunk's mod matrix steering every parameter on every slot. The chain abstraction we ship for Elixir is the same abstraction that hosts everyone else.

### 13.1 Status quo — what already exists in `src/plugin_host/`

The CLAP scaffold under `src/plugin_host/clap/` is roughly half-built. There is one module — `clap` — and `src/plugin_host/mod.rs` reserves room for VST3 and AU but does not declare them yet.

| File | Status | Notes |
|---|---|---|
| `clap/discovery.rs` | working | Walks the standard CLAP paths per OS, honours `$CLAP_PATH`, returns file-level `PluginDescriptor`s. |
| `clap/host.rs` | working | `ContrapunkHost` implements `HostLog`, `HostGui`, `HostAudioPorts`, `HostNotePorts`. `request_restart` / `request_process` / `request_callback` are no-ops. |
| `clap/controller.rs` | working for one plugin | `load_and_activate` instantiates a plugin, queries audio-port layout, activates, starts processing, exposes `take_processor()`. GUI lifecycle (`open_gui`, `close_gui`, `set_embed_frame`) is wired. |
| `clap/audio_block.rs` | working | The real audio block. Holds a `StartedPluginAudioProcessor`, drives `process()` per buffer with deinterleave → port buffers → CLAP `InputEvents` / `OutputEvents` → reinterleave. Send-safe. |
| `clap/block.rs` | **stub** | The earlier non-processing placeholder. Loads the entry and captures the descriptor but writes silence. Kept while `audio_block.rs` is the real path. Should be deleted once C0 is signed off. |
| `clap/embed.rs`, `clap/window.rs` | working, macOS | Bare `NSView` subview of Contrapunk's `contentView`, flipped to DOM coordinates. Non-macOS `EmbedInHost` returns `GuiError::CreateError`. Both files deliberately avoid `Drop` impls — autorelease pool drains race with `Retained` drop and produced overrelease crashes; we leak one `NSView` per plugin and that's accepted. |
| `clap/registry.rs` | working | `thread_local!` `HashMap<PluginId, ClapPluginController>`. Plugins are `!Send` so they live on whichever thread created them, accessed via `AppHandle::run_on_main_thread`. |

Two things are conspicuously absent: parameter automation routing and state save / restore. Both are listed as known v1 limitations in the module header and are the C2 deliverables.

### 13.2 The `AudioBlock` contract — how a hosted plugin slots in

`src/chain/block.rs` defines the only interface the chain knows about. `process(&mut [f32], channels)`, `midi_event(MidiBlockEvent)`, `reset()`, `set_sample_rate(u32)`, plus identity (`name`, `type_id`) and an `enabled` flag. Real-time-safe by contract — no allocs, no locks, no blocking I/O.

`crates/elixir-core` plugs in via `src/chain/elixir_block.rs::ElixirSynthBlock`. A hosted CLAP plugin plugs in via `src/plugin_host/clap/audio_block.rs::ClapAudioBlock`. The chain doesn't care which is which — both are `Box<dyn AudioBlock>` from its point of view, both go through the same `ChainCommand::PushBlock(Box<dyn AudioBlock>)` queue, both get drained by `Chain::drain_commands` at the top of every `process()` call.

**Track A and Track C land on the same chain by construction**. Nothing extra is needed to "integrate" Elixir with the plugin host; integration is the trait. The lock-free SPSC ring buffer (`HeapRb<ChainCommand>` in `src/chain/commander.rs`) carries boxed blocks across the thread boundary; `Box<dyn AudioBlock>` is `Send` because the trait requires `Send`.

Two policy gaps in the current queue need filling for multi-plugin work: there is no `InsertAt(idx, Box<dyn AudioBlock>)` variant (only `PushBlock` and `RemoveAt`), and there is no `MoveBlock(from, to)`. Both variants are one-line additions to `command.rs` and matching arms in `chain.rs::drain_commands`.

### 13.3 CLAP completion (C0–C2)

**C0 — activation + audio process loop.** Delete `clap/block.rs` (the stub) and route every caller to the controller / `ClapAudioBlock` pair already implemented in `controller.rs` and `audio_block.rs`. The flow:

1. Tauri command (`add_clap_plugin_to_chain` — needs to be written under `src-tauri/src/commands/plugin_host.rs`) calls `ClapPluginController::load_and_activate` on the main thread, receives a controller plus a started processor.
2. Insert the controller into the `registry` keyed by `PluginId`.
3. Construct a `ClapAudioBlock::new(name, processor, sample_rate, max_frames, port_layout)` and push it via `ChainCommander::push_block`.
4. Plugin instance stays main-thread-resident (GUI, params); audio processor lives in the chain.

The audio block already handles deinterleave / interleave, port-layout-aware buffer allocation, MIDI event translation (`MidiBlockEvent::NoteOn` → CLAP `NoteOnEvent` with PCKN addressing), and a 256-event MIDI ring sized for typical harmony load. `SustainPedal` is currently a TODO comment in `translate_events`; resolving it means inserting a CLAP CC event (controller 64) into the event buffer.

**C1 — GUI embedding.** Already working on macOS via `EmbeddedPluginView` (`clap/embed.rs`) and `PluginWindow` (`clap/window.rs`). Windows and Linux are not yet implemented. Structure mirrors macOS: a host-owned native parent (`HWND` via `windows-rs`, `Window` via `x11rb`) is created in `window.rs` under `#[cfg(target_os = "windows")]` / `#[cfg(target_os = "linux")]`, and a corresponding `EmbeddedPluginView::new` variant adds the plugin's view as a child. `GuiApiType::default_for_current_platform` already picks the right ABI; `gui.set_parent` accepts `ClapWindow::from_win32_hwnd(...)` and `ClapWindow::from_x11(...)`. The detached-window fallback in `GuiTarget::Detached` is the safety net.

**C2 — param automation + preset state.** Two CLAP extensions to wire:

1. `clack_extensions::params::PluginParams` on the controller. Discover `param_count`, walk `param_info_for_index`, build a `Vec<HostedParamInfo { id, name, range, default }>` exposed to the UI. To set a param, emit a `ParamValueEvent` into the audio block's event buffer alongside MIDI.
2. `clack_extensions::state::PluginState` on the controller. `save` writes the plugin's opaque blob to a `Vec<u8>`; `load` accepts the same blob back. Both run on the main thread.

### 13.4 VST3 hosting (C3) — new `src/plugin_host/vst3/`

Mirror the CLAP module file-for-file: `discovery.rs`, `host.rs`, `controller.rs`, `audio_block.rs`, `window.rs`, `embed.rs`, `registry.rs`. The Rust VST3 surface is rougher than CLAP — `vst3-sys` (raw `#[repr(C)]` COM-style bindings) is the realistic starting point; no `clack-host` analogue exists for VST3 so we may end up writing our own thin safe wrapper.

- `discovery.rs` — `~/Library/Audio/Plug-Ins/VST3/` (macOS), `%CommonProgramFiles%\VST3\` (Windows), `~/.vst3/` and `/usr/lib/vst3/` (Linux). VST3 bundles are directories on every OS.
- `host.rs` — implement `IHostApplication`, `IComponentHandler`, `IPlugFrame` so the plugin can talk back. `IPlugFrame::resizeView` ≈ CLAP's `HostGui::request_resize`.
- `controller.rs` — load via `vst3-sys`, instantiate `IComponent` + `IEditController`, `setupProcessing(ProcessSetup { sample_rate, max_samples_per_block, ... })`, `setActive(true)`, hand the `IAudioProcessor` to the audio thread.
- `audio_block.rs` — `process(ProcessData { ... })` with stereo input + stereo output buses; convert `MidiBlockEvent` into VST3 `Event` (NoteOn, NoteOff, ParamValue).
- `window.rs` / `embed.rs` — same `NSView` / `HWND` / `X11Window` host wrapper. `IPlugView::attached(parent, platform_type)` ≈ CLAP's `gui.set_parent`.

License story: the VST3 SDK headers are GPL3-compatible. Our `vst3` module is GPL3'd locally; the rest of Contrapunk stays on its current license, isolated via the module boundary. One-day legal check at the start of C3 per `ELIXIR-PLAN.md §10`. VST3 host lives in its own crate or `cfg(feature = "vst3-host")` so distributions without the feature are unaffected.

### 13.5 AU hosting (C4, macOS only) — new `src/plugin_host/au/`

Wrap the whole module in `#[cfg(target_os = "macos")]`. Two viable paths: the `audio-unit` crate (high-level, AU v2 only) or hand-rolling over `objc2` (more work, but `objc2` is already a dep). AU v2 (Component Manager) and AU v3 (App Extension) need separate detection — v2 from `AudioComponentInstanceNew`, v3 from `AVAudioUnitComponentManager` (sandboxed). v1 scope is v2 only. The block side is straightforward: `AURender` returns 32-bit interleaved float; MIDI is delivered through `MusicDeviceMIDIEvent`; GUI uses `AudioUnitGetProperty(kAudioUnitProperty_CocoaUI)` → an `NSView` we embed the same way as CLAP.

### 13.6 Plugin discovery — generalising the existing pattern

`clap/discovery.rs` is the template. Lift it to a shared trait in `src/plugin_host/mod.rs`:

```rust
pub trait PluginFormatScanner {
    fn standard_paths() -> Vec<PathBuf>;
    fn extension() -> &'static str;
    fn descriptor_from_path(p: &Path) -> PluginDescriptor;
}
```

Each format's `discovery.rs` implements it. The shared driver becomes one function: walk roots, find entries matching the extension, dedupe, return descriptors. Scanning runs off the audio thread (Tauri command spawned on the Tokio runtime) and writes a cache to `~/.config/contrapunk/plugins.json` keyed by `(path, mtime, size)`. Subsequent launches read the cache, re-walk only entries whose mtime has changed, and only fully introspect (factory load → real descriptor) on demand at instantiation time.

### 13.7 Multi-plugin chain UX

The current Svelte UI does not yet have a per-slot rack. Each `BlockDescriptor` (returned by `ChainCommander::snapshot()` in `src/chain/commander.rs`) needs a UI strip: name, format badge (`CLAP` / `VST3` / `AU` / `Elixir`), bypass toggle (driving `AudioBlock::enabled`), latency report, "open GUI" button, parameter expander, drag handle for reorder, "remove". Badge comes from the `type_id` prefix (`clap.audio`, `vst3.audio`, `au.audio`, `builtin.elixir-synth`).

Default slot order: **harmony source → synth (Elixir or hosted instrument) → FX (any mix of hosted and built-in) → output**. The Chain does not enforce ordering today — any block can go anywhere. The UI is the policy layer: drag-reorder calls a future `ChainCommand::MoveBlock { from, to }` and the UI refuses drops that would place an FX above the synth.

### 13.8 Parameter routing into the mod matrix

`crates/elixir-core/src/modulation.rs::ModDest` is currently a closed enum. For Track C extend it:

```rust
pub enum ModDest {
    // ...existing internals...
    Hosted { slot: u8, plugin_param_id: u32 },
}
```

`slot` is the chain index of the target plugin block; `plugin_param_id` is the plugin's own opaque ID (CLAP `clap_id`, VST3 `ParamID`, AU `AudioUnitParameterID`). The matrix evaluator stays SoA — accumulates `(amount * src_value)` into a per-block scratch buffer keyed by `(slot, plugin_param_id)`. Each hosted block reads its own scratch entries and emits ParamValue events into its event buffer before `process`.

Modulation amount smoothing happens at the matrix layer so plugins never see a stepped param value. Per-plugin namespacing is implicit in the `(slot, param_id)` pair.

### 13.9 State persistence — session preset

```json
{
  "chain": [
    { "kind": "harmony" },
    { "kind": "elixir", "preset": { /* elixir-core preset blob */ } },
    { "kind": "clap", "path": "~/Library/Audio/Plug-Ins/CLAP/Diva.clap",
      "state": "<base64 of CLAP state extension blob>" },
    { "kind": "vst3", "path": "...", "state": "<base64>" }
  ]
}
```

Load sequence: parse, scan-on-load for each plugin path (re-discover by id if path moved), instantiate, restore state from the base64 blob via the format's state extension, push onto the chain. Failures degrade gracefully — a missing plugin becomes a placeholder bypass block tagged with the original path.

### 13.10 Latency reporting

When Contrapunk itself is hosted as a plugin, chain latency is the sum of every block's latency plus internal lookahead. Add `fn latency_samples(&self) -> u32 { 0 }` to `AudioBlock` (default-implemented; trait-stable). CLAP plugins report via `latency.get`, VST3 via `IAudioProcessor::getLatencySamples`, AU via `kAudioUnitProperty_Latency`. `Chain` exposes `total_latency()` summing the lot, and the outer plugin wrapper passes it back to the host. When any hosted plugin signals a latency change (CLAP `host.latency.changed`), we call `host.request_restart`.

### 13.11 Plug-and-play with Elixir

Once the trait is the only contract, "swap Elixir for Diva" is `RemoveAt(1) + PushBlock(ClapAudioBlock)` on the queue. "Chain Elixir → Pro-Q4" is `PushBlock(ElixirSynthBlock) + PushBlock(ClapAudioBlock)`. The chain doesn't care, the audio callback doesn't care, the harmony engine doesn't care.

### 13.12 Sandboxing — current vs intended

**Current state**: every hosted plugin runs in-process. A misbehaving plugin (segfault inside `process`, dereference of a port the host didn't allocate) takes the whole app down. We saw this with Pro-C 2 before the port-layout query was added in `controller.rs`. Cost of in-process: fast (zero IPC overhead, shared audio buffers) but unsafe.

**Intended (post-v1)**: an out-of-process worker per plugin, audio shuttled through a shared-memory ring with SPSC frame-aligned producer/consumer, MIDI/params through a control SPSC. A crashed worker just stops producing; the chain runs its slot as a bypass, surfaced to the UI as "plugin crashed — reload?". Bitwig and Reaper both ship this; latency cost is ~1 buffer of pipelining (5.3 ms at 256/48k), well within our 30 ms target.

The architectural step that unlocks sandboxing later is keeping every `AudioBlock` implementation a thin shim over an opaque processor handle (which `ClapAudioBlock` already is — the only hot-path call is `processor.process(...)`). Swap that handle for an IPC client and the rest of the chain doesn't notice.

---

## 14. Scope, Policies, and Exclusions

This section is the explicit "what we *aren't* doing and why" so future contributors don't waste time re-litigating settled scope. Each subsection is short by design.

### 14.1 License / DRM / cloud features — **not shipping**

Elixir is **not** an account-gated product. No license server, no online activation, no DRM, no offline-license fallback that has to phone home. The binary you download is the binary that runs forever. Practical consequences:

- No third-party identity SDK (no Firebase, no Auth0, no Supabase). The dependency surface stays small and auditable.
- No "demo mode" with feature gates. The product is either open source (license TBD — `LICENSE` file pending) or paid via outright purchase, with no behavioural difference between binaries.
- No cloud-hosted preset library. Preset sharing happens through ordinary file exchange (Discord, forums, GitHub, email).
- No telemetry on preset opens, knob movements, or session duration (see §14.2).

This decision is final unless a future business model explicitly requires it; if it does, the conversation re-opens in a new design doc, not as a quiet code addition.

### 14.2 Telemetry, crash reporting, analytics — **opt-in only, off by default**

Default stance: Elixir collects nothing. No usage telemetry, no anonymous metrics, no PostHog / Mixpanel / Sentry hook in the default build.

Concrete rules:

- **Crash reporting**: a future build *may* offer an opt-in `--crash-reports` flag that uploads stack traces to a self-hosted endpoint. The first run dialog must clearly explain what's sent. Default off.
- **No phone-home on startup**: the binary does not contact any server during normal operation, including version checks. Update checks (§14.4) require explicit user action.
- **Local logs**: standard log output to stderr / a rotating file under `~/Library/Logs/Elixir/` (macOS) or equivalent. Stays on the user's machine.

Contrapunk's existing analytics posture (whatever it is) does not propagate to Elixir.

### 14.3 Documentation / parameter help — authored alongside the code

Every user-facing parameter ships with a one-line tooltip authored in the same source file as the parameter's declaration:

```rust
FloatParam::new("Cutoff", 8_000.0, FloatRange::Skewed { min: 20.0, max: 20_000.0, factor: 0.3 })
    .with_unit(" Hz")
    .with_value_to_string(format_hz())
    .with_help("Voice-filter cutoff. Modulating from an LFO yields the classic synth sweep.")
```

The reference manual is generated from the same source-of-truth via a `cargo xtask docs` subcommand that walks the parameter registry and emits a Markdown table per panel. No separate Wiki, no out-of-sync documentation. Diagrams (signal flow, mod-matrix topology) live as SVG under `docs/figures/` and are referenced in the generated manual.

Long-form tutorial content (sound design walkthroughs, "make a 90s pad in 4 minutes") lives on the public website and is *not* shipped in the binary.

### 14.4 Release, signing, auto-update

Per locked decision §5 in `ELIXIR-PLAN.md`, Elixir releases from this repo. The pipeline:

- **Tag scheme**: `elixir-vX.Y.Z` (vs Contrapunk's `vX.Y.Z`). The `release-patch` skill is extended to recognise both prefixes.
- **macOS signing**: Apple Developer ID Application certificate (Contrapunk's existing cert, separate bundle ID `com.contrapunk.elixir.standalone`, `com.contrapunk.elixir.plugin`).
- **Windows signing**: EV code-signing certificate. Same cert as Contrapunk; separate `Subject Name` per product.
- **Linux**: unsigned tarballs + `.deb` (no native signing convention).
- **Notarization (macOS)**: every release artifact passes `notarytool submit` before the GitHub Release is published.
- **Auto-update**: **none in v1**. The user manually downloads new releases. A `--check-for-updates` flag may be added post-v1 that hits a public GitHub release-feed JSON; no silent updates ever.
- **Distribution channels**: GitHub Releases is canonical. A `.dmg` for macOS, a `.exe` installer for Windows, a `.deb` / tarball for Linux. The plugin variant ships as `.vst3` / `.clap` / `.component` bundles inside the same artifact.

### 14.5 Localization — English-only for v1

The UI ships English-only. Strings are inlined in the source rather than going through a `t!("key")` macro — localization is a real cost (translation, font coverage, RTL layout) and v1 doesn't earn it.

If localization ever happens, the path is clear: extract every user-visible string into a `strings.toml` keyed by stable IDs, wrap with `t!`, and add language files alongside. The egui custom font loader already supports CJK and right-to-left scripts via additional `FontData` entries. None of this is wired in v1.

### 14.6 Community & sharing

- **Presets**: ordinary JSON files; users exchange via Discord / forum / GitHub gist / direct file transfer. A "Reveal in Finder / Explorer" affordance in the preset browser is the only friction-reducer.
- **Skins**: same JSON format as the default skin; users can author and share. Skin loader has a "Load Skin..." menu entry. No skin marketplace, no skin-format DRM.
- **Wavetables**: a wavetable file is its own JSON-embedded asset (`§9.7`); same sharing model as presets.
- **Bug reports & feature requests**: GitHub Issues on the public Elixir repo (or section of the Contrapunk repo until extraction happens). No paid support tier in v1.

### 14.7 Mobile / iOS — explicit non-goal

Elixir v1 is **desktop only** (macOS, Windows, Linux). iOS and Android are not on the roadmap. Rationale:

- The plugin story (VST3 / CLAP / AU) doesn't apply on iOS (AUv3 only) and requires sandboxing decisions we'd rather defer.
- The egui UI works on iOS in principle (via `eframe` mobile feature) but the workflow assumptions (mouse, keyboard, multi-window) don't transfer to a touch surface without a real redesign.
- Storage and entitlement constraints on iOS (sandbox, app-store-only distribution) conflict with the "no DRM, ordinary file exchange" position from §14.1 / §14.6.

If mobile ever happens, it's a separate product (`Elixir Mobile`), separate codebase entry point, separate UI. The shared piece is `elixir-core`, which already compiles to `wasm32` and would compile to `aarch64-apple-ios` with minimal porting.

### 14.8 Performance budget

The headline performance targets for the v1 release:

| Metric | Target | Notes |
|---|---|---|
| Polyphony | 16 voices at 48 kHz | Without dropping audio frames on a 2020+ M-series Mac or equivalent x86 |
| CPU @ 16 voices | < 15% of one core | Measured at idle (notes held, no UI interaction) |
| RAM at startup | < 60 MB resident | Without factory wavetable bank loaded |
| Cold startup time | < 800 ms | First UI frame visible |
| Patch load | < 50 ms | For any factory preset |
| Audio glitch budget | 0 over a 1 hour stress run | `assert_no_alloc` enabled in CI |

These are aspirational for v1 and become hard regressions thereafter — any commit that pushes any metric over its budget needs explicit justification.

### 14.9 Testing scope

Beyond the per-phase unit tests already in `crates/elixir-core/tests`:

- **Golden WAV tests**: a representative MIDI sequence rendered through every preset in the factory bank produces a reference WAV; CI compares new builds to the reference within a tolerance (`< -90 dBFS RMS` after migration is settled).
- **`pluginval` conformance**: for both `elixir-plugin` (Elixir as plugin) and the hosted plugins inside Contrapunk (Track C), the standard plugin-host conformance suite runs in CI on every macOS build.
- **Property tests** (`proptest`): invariants like "voice pool count == active + inactive", "envelope is bounded `[0, 1]`", "no NaN ever escapes `process`".
- **Fuzz**: preset parser is fuzzed via `cargo fuzz` for malformed inputs (no panics, only errors).

Stress tests (1-hour-render-without-glitches) are run weekly, not per-commit. CI tagging marks "regression-risk" commits (anything in `crates/elixir-core/src/voice.rs`, `osc.rs`, `filter.rs`) for an additional 30-second golden-WAV pass.

---

*End of document.*
