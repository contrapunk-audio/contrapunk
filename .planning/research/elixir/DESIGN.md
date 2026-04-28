---
title: Elixir — Technical Design & Implementation
date: 2026-04-28
status: draft
supersedes: parts of `.planning/notes/elixir-design-decisions.md` (decision #4 — see Revisions)
---

# Elixir — Technical Design & Implementation

Open-source wavetable synthesizer in Rust with workflow parity to Serum, living in the contrapunk monorepo. Standalone Tauri + Svelte app first; contrapunk integration and external plugin formats follow later.

## 0. Document scope

This is the architecture-complete design for **all** of Elixir's intended feature surface, plus a phased implementation roadmap. Architecture is locked on day one so future engines, FX topologies, and modulation depth slot in without rewrite. Implementation is staged so a usable build ships early and grows toward parity over many months.

This doc is paired with:
- `.planning/notes/elixir-design-decisions.md` — six load-bearing decisions captured during `/gsd-explore`.
- `.planning/research/elixir/serum-features.md` — reference-synth feature inventory.
- `.planning/research/elixir/oss-prior-art.md` — OSS prior art + Rust plugin ecosystem.
- `.planning/seeds/elixir-serum-preset-re-gate.md` — gated future preset reverse-engineering.
- `.planning/todos/pending/elixir-prereqs.md` — pre-implementation reading list.

## 1. Vision and non-goals

### Vision

A Rust-native, permissively-licensed, MPE/microtuning-first wavetable synthesizer with workflow familiarity to Serum users. Five-engine hybrid (Wavetable / Multisample / Sample / Granular / Spectral) per oscillator slot. Deep modulation, three-bus parallel FX graph, full bidirectional Serum wavetable file compatibility. Composable Svelte UI built from reusable primitives so the same UI surface works in standalone, in contrapunk, and (later) in plugin shells.

### Non-goals (v1.0–v1.x)

- **Visual 1:1 replica of Serum's UI.** Workflow parity is the goal; the visual identity is Elixir's own. Trade-dress concerns rule out a literal copy. Layout and controls feel familiar; colors, type, branding are distinct.
- **Serum preset (`.SerumPreset`) import.** Gated post-MVP — see seed `elixir-serum-preset-re-gate.md`. Wavetable file compat (`clm`-chunk WAV) is in v1.
- **CLAP / VST3 / AU plugin formats.** Deferred. The standalone Tauri app is the first deliverable. External plugin formats are a later phase with its own design pass for webview-in-plugin tradeoffs.
- **Mobile / iOS targets.** Out of scope.

## 2. Revisions to prior decisions

The original decision #4 ("engine crate + two shells: contrapunk-first AudioBlock + nih-plug/vizia standalone") is **revised** by this doc:

| Aspect | Original (`elixir-design-decisions.md` #4) | Revised (this doc) |
|---|---|---|
| First deliverable | Contrapunk integration via `AudioBlock` | **Standalone Tauri + Svelte app** |
| Standalone UI | vizia (recommended by nih-plug docs) | **SvelteKit + Tauri** |
| Plugin formats | nih-plug CLAP/VST3 from the start | **Deferred** to a later phase |
| Contrapunk integration | First | **Later phase** (Phase 11 in §5) |
| UI language across shells | Two UIs (Svelte + vizia) | **One Svelte UI everywhere**, composable via shared component library |

The other five decisions (license, doc shape, ambition axes, file-format compat policy, anti-aliasing strategy) are **unchanged**.

### Why the revision

- Reuses contrapunk's existing Tauri + SvelteKit + cpal + midir stack: no new GUI toolkit to learn solo, faster path to first audible build.
- One Svelte component library can be reused by standalone Elixir, contrapunk's host UI, and future plugin webview shells. "Composable" in the user's sense.
- vizia is still a fine path for plugin shells later; ruling it out for v1 keeps the surface area small.
- Trade-off: dense real-time UI (oscilloscope, wavetable 3D view, spectrum) under web rendering needs care. Phase 1 spike must validate Canvas / WebGL performance for the wavetable view at 60fps under audio load.

## 3. Workspace topology

```
contrapunk/                          (existing)
├── Cargo.toml                       workspace root
├── crates/                          existing crates
│   ├── contrapunk-harmony/
│   ├── contrapunk-midi/
│   ├── contrapunk-audio/
│   ├── contrapunk-chord/
│   ├── contrapunk-transport/
│   └── elixir-engine/               NEW — pure DSP/state, no UI, no shell deps
├── elixir/                          NEW — standalone app subtree
│   ├── src-tauri/                   Tauri backend (consumes elixir-engine)
│   │   ├── Cargo.toml
│   │   └── src/main.rs
│   └── ui/                          SvelteKit frontend
│       ├── package.json
│       ├── svelte.config.js
│       └── src/
│           ├── app.html
│           ├── routes/
│           └── lib/
│               └── components/
│                   ├── synth/       reusable synth primitives (knobs, scopes, mod widgets)
│                   ├── osc/         oscillator panel components
│                   ├── filter/
│                   ├── env/
│                   ├── lfo/
│                   ├── fx/
│                   ├── mod-matrix/
│                   └── wavetable-view/
├── src-tauri/                       (existing contrapunk Tauri app, unchanged)
├── ui/                              (existing contrapunk Svelte UI, unchanged)
├── wasm/                            (existing)
└── ...
```

**Why this layout:**
- `crates/elixir-engine` is a normal workspace member: `cargo check -p elixir-engine` works, no Tauri/UI deps leak into the engine.
- `elixir/src-tauri` and `elixir/ui` mirror contrapunk's `src-tauri/` + `ui/` so the patterns are familiar and CI scripts can extend by analogy.
- Components under `elixir/ui/src/lib/components/` can be packaged as an internal `@elixir/ui` workspace package once stable, and consumed by contrapunk's host UI in the integration phase.

## 4. Engine architecture

### 4.1 Synth top-level

```rust
pub struct Synth {
    sample_rate: u32,
    voices: VoiceManager,         // polyphony, MPE allocation, voice stealing
    mod_global: GlobalMod,        // 8 macros, global LFOs (subset of 10), global envs (subset of 4)
    fx_graph: FxGraph,            // 3-bus parallel FX
    params: ParamStore,           // canonical parameter state (smoothed where needed)
    preset: PresetMeta,           // current preset metadata
    wavetables: WavetableBank,    // loaded WTs keyed by slot
    samples: SampleBank,          // loaded multisamples / samples
    tuning: TuningState,          // MTS-ESP, SCL/KBM, equal temperament
}

impl Synth {
    pub fn new(sample_rate: u32) -> Self;
    pub fn process(&mut self, output: &mut [f32], channels: usize);

    // MIDI 1.0 + MPE
    pub fn note_on(&mut self, ch: u8, note: u8, vel: u8);
    pub fn note_off(&mut self, ch: u8, note: u8);
    pub fn pitch_bend(&mut self, ch: u8, value: i16);
    pub fn cc(&mut self, ch: u8, cc: u8, value: u8);
    pub fn poly_pressure(&mut self, ch: u8, note: u8, value: u8);
    pub fn channel_pressure(&mut self, ch: u8, value: u8);

    // Param control (host/UI surface)
    pub fn set_param(&mut self, id: ParamId, value: f32);
    pub fn get_param(&self, id: ParamId) -> f32;
    pub fn iter_params(&self) -> impl Iterator<Item = (ParamId, ParamMeta)>;

    // Asset loading
    pub fn load_wavetable(&mut self, slot: OscSlot, wt: Wavetable);
    pub fn load_sample(&mut self, slot: OscSlot, sample: Sample);
    pub fn load_multisample(&mut self, slot: OscSlot, ms: Multisample);

    // Preset I/O
    pub fn load_preset(&mut self, preset: &Preset) -> Result<(), PresetError>;
    pub fn save_preset(&self) -> Preset;
}
```

`Synth` is the only thing a shell needs to drive. Everything else is internal.

### 4.2 Voice graph (per-voice DSP)

```
                       ┌─────────────────────────┐
                       │   Modulation evaluator    │  (per-voice, per-block; sample-accurate slots
                       │   - 4 envs (DAHDSR+curve) │   for vibrato + similar)
                       │   - LFOs (per-voice subset) │
                       │   - Sources: vel, key,    │
                       │     aftertouch, MPE Y/Z,  │
                       │     random, etc.          │
                       │   - Per-slot remap curves │
                       └───────────┬─────────────┘
                                   │  mod values
                                   ▼
   ┌───────────┐    ┌───────────┐    ┌───────────┐
   │  OSC A    │    │  OSC B    │    │  OSC C    │   each is an Engine impl
   │ (Engine)  │    │ (Engine)  │    │ (Engine)  │   (Wavetable | Multisample | Sample
   └─────┬─────┘    └─────┬─────┘    └─────┬─────┘    | Granular | Spectral)
         │                │                │
         └────────┬───────┴────────────────┘
                  ▼
              [Sub osc] + [Noise osc] mixed in
                  ▼
         ┌──────────────┐
         │  Filter 1    │   parallel/serial routing → Filter 2 (or split)
         └──────┬───────┘
                ▼
         ┌──────────────┐
         │  Filter 2    │
         └──────┬───────┘
                ▼
              voice output  ──→ FxGraph (post-voice, sample-summed)
```

### 4.3 Engine trait — the five-engine hybrid

```rust
pub trait Engine: Send {
    fn kind(&self) -> EngineKind;
    fn note_on(&mut self, ctx: &VoiceCtx);
    fn note_off(&mut self, ctx: &VoiceCtx);
    fn render(&mut self, mods: &EngineMods, out: &mut [f32]);
    fn reset(&mut self);
}

pub enum EngineKind { Wavetable, Multisample, Sample, Granular, Spectral }

pub struct EngineMods<'a> {
    pub pitch_hz: f32,
    pub level: f32,
    pub pan: f32,
    pub wt_pos: f32,         // 0..1, ignored by non-WT engines
    pub warp: WarpMode,      // ignored by non-WT engines
    pub warp_amount: f32,
    pub fm_index: f32,       // for FM warp
    pub unison: u8,
    pub detune: f32,
    pub blend: f32,
    pub phase: f32,
    pub stereo: f32,
    // engine-specific extras passed via `extras: &'a EngineExtras`
    pub extras: &'a EngineExtras,
}
```

v1 ships only `WavetableEngine`. The trait and dispatch infrastructure exist from day one; later engines slot in by implementing the trait.

### 4.4 Wavetable engine

- **Frame layout:** 2048 samples per cycle (Serum-compatible), 1–256 frames per WT (most are 256).
- **Anti-aliasing strategy v1: mipmap pyramid (Surge-style).** At WT load time, each frame is band-limited to N pre-computed mipmap levels indexed by playback frequency. Memory cost: ~log2(N_partials) × frame_size per frame. CPU cost at runtime: low.
- **Anti-aliasing strategy v2 (later): frequency-domain harmonic storage with per-voice on-the-fly bandlimiting (Vital-style).** Higher quality ceiling; selectable via `quality_mode` param. Loaded as a parallel `WavetableEngine` impl behind the same `Engine` trait.
- **Position interpolation:** linear by default, with optional "smooth" (cubic) mode.
- **Warp modes (v1 ships subset; full set ramps in):** Off, FM, RM, Sync, Bend, Mirror, Asym, Remap, Quantize, BendPlus, BendMinus, FlipFlop, ZeroFlip, Spectral *(spectral as warp routes through a small FFT path, distinct from the Spectral engine type)*.
- **Unison:** up to 16 voices per oscillator with detune, blend, phase scramble, stereo spread.

### 4.5 Filters

- v1: SVF (LP/HP/BP/Notch) with drive.
- v2+: ladder (Moog-style), comb, formant, MS-20-style, OB-style, phaser-as-filter, dual-band split.
- Per-voice. Two filters per voice with serial/parallel/split routing.

### 4.6 Modulation graph

```rust
pub struct ModSlot {
    pub source: ModSource,
    pub destination: ParamId,
    pub depth: f32,            // -1..1
    pub remap: RemapCurve,     // user-drawn curve, optional
    pub mode: ModMode,         // Bipolar | Unipolar | Mult
}

pub enum ModSource {
    Velocity, Note, Aftertouch, ChannelPressure, ModWheel, PitchBend,
    Lfo(LfoId),                // 10 LFOs, IDs 0..9
    Env(EnvId),                // 4 envs, IDs 0..3
    Macro(MacroId),            // 8 macros
    MpeY, MpeZ,
    Random,
    NoteOnRandom,
    // contrapunk-bridge sources (gated; populated only when contrapunk integration is active)
    HarmonyDegree, HarmonyTension, HarmonyKey, HarmonyMode,
}
```

- **10 LFOs** (Path-mode produces dual X/Y outputs; counted as 2 effective sources per Path-mode LFO).
- **4 envelopes** (DAHDSR with per-stage curve).
- **8 macros** (smoothed, global).
- **Mod matrix:** capacity ≥64 slots; each slot has source, destination ParamId, depth, optional remap curve, mode.
- **Evaluation:** per-block by default; sample-accurate option for sources marked audio-rate (e.g., LFO when sync'd as oscillator, env attack edges).

### 4.7 FX graph (3-bus parallel)

```
voice sum ─→ [Splitter: L/H | L/M/H | M/S | identity] ─┬→ Bus A: [fx, fx, fx, ...] ─┐
                                                       ├→ Bus B: [fx, fx, fx, ...] ─┼→ master out
                                                       └→ Bus C: [fx, fx, fx, ...] ─┘
```

- 3 buses, each with an ordered `Vec<Box<dyn FxModule>>`.
- Splitter modules feed into one or many buses.
- v1 FX list (subset): Compressor, EQ, Reverb, Delay, Distortion, Chorus.
- v2+ FX: Hyper, Dimension, Flanger, Phaser, Filter (as FX), Multiband, others matched by name from reference inventory.
- Each FxModule implements:
  ```rust
  pub trait FxModule: Send {
      fn name(&self) -> &str;
      fn process(&mut self, buf: &mut [f32], channels: usize);
      fn set_param(&mut self, id: u32, value: f32);
      fn reset(&mut self);
  }
  ```

### 4.8 File formats

#### Serum wavetable WAV (read + write, v1)

- 32-bit float WAV, 2048-sample frames per cycle.
- ASCII `clm ` chunk encodes metadata; format documented in `oss-prior-art.md`. Writer round-trip is bit-equivalent for files Elixir produced; reader is tolerant of vendor variants.
- Implementation: ~150 lines of Rust in `crates/elixir-engine/src/wavetable/serum_wav.rs`.

#### Native preset format

- **RON** (Rusty Object Notation) — chosen over JSON for: support for tagged enums (matches our enum-rich state), comments allowed, smaller diffs in version control, idiomatic in Rust ecosystems.
- Schema versioned (`version: u32` at root).
- Bundle layout: a preset is a directory `*.elx/` containing `preset.ron` + referenced wavetables in `wt/`. Optionally zipped to a single `.elx` file.
- Schema sketch:

```ron
ElixirPreset(
    version: 1,
    meta: Meta(name: "...", author: "...", category: "...", tags: ["..."]),
    osc: [
        OscSlot(engine: Wavetable(WtRef("wt/foo.wav")), enabled: true, /* ... */),
        OscSlot(engine: Wavetable(WtRef("wt/bar.wav")), enabled: false, /* ... */),
        OscSlot(engine: Off, enabled: false),
    ],
    sub: SubOsc( /* ... */ ),
    noise: NoiseOsc( /* ... */ ),
    filters: [Filter1( /* ... */ ), Filter2( /* ... */ )],
    envs: [ /* 4 envs */ ],
    lfos: [ /* 10 lfos */ ],
    macros: [ /* 8 macros */ ],
    mod_matrix: [ /* slots */ ],
    fx_graph: FxGraph(
        splitter: Identity,
        bus_a: [ /* fx modules */ ],
        bus_b: [ /* ... */ ],
        bus_c: [ /* ... */ ],
    ),
    tuning: TuningState( /* ... */ ),
)
```

- `.elx.bin` cache: optional binary serde for fast load; treat as derived, not authoritative.

#### Serum preset (`.SerumPreset`) — gated, post-MVP

See `.planning/seeds/elixir-serum-preset-re-gate.md`. Not part of any near-term phase.

### 4.9 Tuning and MPE

- **MTS-ESP:** ODDSound MTS-ESP (BSD-licensed C lib). Rust binding via `bindgen` (FFI in `crates/elixir-engine/src/tuning/mts_esp.rs`).
- **SCL/KBM:** native parser (no external dep needed).
- **MPE:** voice allocator handles channel-per-note (channels 2–16 with master 1 by convention). Per-note pitch bend, pressure (CC74 / channel pressure), Y axis. MPE detection auto-engages when MPE Configuration Message (RPN 6) seen.
- **MIDI 2.0:** voice allocator carries 32-bit per-note resolution under the hood; MIDI 1.0 + MPE map into it. MIDI 2.0 transport is a future input — same allocator.

## 5. Standalone Tauri + Svelte shell

### 5.1 Process model

- Single Tauri app: `elixir/src-tauri/` (Rust backend) + `elixir/ui/` (Svelte frontend).
- Audio thread: `cpal` output stream owned by the Tauri backend. Backend pulls frames and calls `Synth::process()` directly. No IPC on the audio path.
- MIDI input thread: `midir` callback feeds events into a SPSC ring buffer; audio thread drains the ring at the start of each callback.
- UI thread: SvelteKit. Communicates with backend via Tauri commands and events. **No shared memory, no audio data pulled to UI except telemetry** (RMS, oscilloscope ring, voice count).

### 5.2 Audio I/O

```rust
// elixir/src-tauri/src/audio_clock.rs (sketch)
pub struct AudioClock {
    synth: Arc<Mutex<Synth>>,        // protected only at config boundary; audio path uses lock-free param store
    midi_rx: ringbuf::Consumer<MidiEvent>,
    stream: cpal::Stream,
}
```

- `cpal` for output, matching contrapunk's existing pattern (`crates/contrapunk-audio`).
- Buffer size: 64–512 samples, host-configurable.
- Sample rate: device-driven, supports 44.1/48/88.2/96/176.4/192 kHz.

### 5.3 MIDI I/O

- `midir` for input (matching contrapunk). Multi-port aggregation supported.
- MIDI events serialized into a lock-free ring buffer (`ringbuf` crate).
- Future: virtual MIDI input from OSC, network MIDI (RTP-MIDI).

### 5.4 Tauri command surface

Naming convention: `elixir_*` prefix, mirrors contrapunk's `set_synth_*` pattern. Commands fall into:

- **Param control** — `elixir_set_param(id: u32, value: f32)`, `elixir_get_param(id) -> f32`, `elixir_iter_params() -> Vec<ParamSnapshot>`.
- **Asset I/O** — `elixir_load_wavetable(slot, path)`, `elixir_load_sample(slot, path)`, `elixir_save_preset(path)`, `elixir_load_preset(path)`.
- **MIDI inject** (for keyboard widget / testing) — `elixir_note_on(ch, note, vel)`, `elixir_note_off(ch, note)`.
- **Telemetry stream** — Tauri events: `elixir://oscilloscope` (Float32Array, ~30 Hz), `elixir://meters` (RMS L/R, voices, ~60 Hz), `elixir://wt-frame-snapshot` (single frame for the WT view).

The param store is the single source of truth. UI reflects state via initial fetch + diff events; UI never drives audio-path state directly.

### 5.5 Svelte composability model

Components live in `elixir/ui/src/lib/components/` and are deliberately context-free — they take props for state and emit events for changes. Top-level pages assemble them.

```svelte
<!-- elixir/ui/src/lib/components/synth/Knob.svelte -->
<script lang="ts">
    export let value: number;
    export let min = 0, max = 1, default_ = 0.5;
    export let label: string;
    export let modSlots: ModSlotInfo[] = [];   // mod indicators
    /* … */
</script>

<!-- elixir/ui/src/routes/+page.svelte -->
<script>
    import OscPanel from '$lib/components/osc/OscPanel.svelte';
    import FilterPanel from '$lib/components/filter/FilterPanel.svelte';
    import ModMatrix from '$lib/components/mod-matrix/ModMatrix.svelte';
    /* … */
</script>

<OscPanel slot="A" />
<OscPanel slot="B" />
<OscPanel slot="C" />
<FilterPanel id={1} />
<FilterPanel id={2} />
<ModMatrix />
```

**Composition rules:**
- No global stores in components. Stores live in `lib/stores/` and are imported explicitly.
- Components subscribe to a single param-id range; they don't reach across.
- All controls accept a `ModSlotInfo[]` so mod indicators (rings, halos) render uniformly across the UI.
- Drag-and-drop modulation: the mod-source UI element exposes a `data-mod-source` attribute; any component accepting modulation listens for drops with `data-mod-target`. Single drop handler in app shell. Same protocol works for any future component.

When the contrapunk integration phase comes, these components are imported as a workspace package (`@elixir/ui`) into contrapunk's UI without modification.

### 5.6 Real-time UI rendering

- **Knobs / sliders:** plain DOM/CSS. Cheap, accessible.
- **Oscilloscope, meters:** Canvas 2D, driven by Tauri telemetry events at 30–60 Hz.
- **Wavetable 3D view:** WebGL via `regl` or hand-written WebGL2. Telemetry-driven. **Phase 1 spike must validate 60 fps under audio load** (see Phase 0 prereqs).
- **Mod-matrix node graph:** SVG with d3-flavored layout, no library required.

If the WebGL view doesn't hold 60 fps, fallbacks: lower update rate (15–30 Hz), 2D top-down view only, or push the WT frame render to Rust (rasterize a PNG, ship as a Tauri event).

## 6. Implementation roadmap

Phases are ordered by dependency, not date. Each phase ends with a working, audible build. Solo timeline TBD; phases are sized to ship something demonstrable in ~2–6 weeks of part-time work each.

### Phase 0 — Prereqs (no code in `crates/elixir-engine`)

- Watch Tytel ADC 2021 talk; read EarLevel WT series; read Välimäki/Huovilainen.
- Document Serum `clm`-chunk WAV format → `crates/elixir-engine/docs/wavetable-format.md`.
- Throwaway spike: validate WebGL wavetable 3D view performance under simulated audio load.
- Throwaway spike: confirm `cpal` + `midir` end-to-end audio + MIDI in a minimal Tauri app skeleton.
- Tracked in `.planning/todos/pending/elixir-prereqs.md`.

### Phase 1 — "Hello synth": minimal standalone, end-to-end audio

- Crate `crates/elixir-engine` skeleton with `Synth` struct, `ParamStore`, sine osc placeholder, ADSR.
- Native preset format scaffold (RON, version 1 with sine osc only).
- Tauri app `elixir/src-tauri/`: `cpal` output, `midir` input, MIDI ring, `Synth::process()`.
- Svelte UI `elixir/ui/`: minimal — keyboard + ADSR knobs. Tauri commands for param set/get.
- **Acceptance:** play notes from a MIDI keyboard, hear sine osc with ADSR, change ADSR from UI.

### Phase 2 — Wavetable engine v1

- `WavetableEngine` impl behind `Engine` trait.
- Mipmap-pyramid antialiasing at WT load.
- Serum WAV reader (`clm ` chunk); writer.
- One OSC slot (A) wired up; B, C slots exist but disabled.
- WT browser in UI; load/save WT files.
- Subset of warp modes (Off, FM, RM, Sync).
- **Acceptance:** load a Serum WT, play it across the keyboard with no audible aliasing at the top of the range, change WT position from UI.

### Phase 3 — Filters + ADSR + LFO

- SVF filter (LP/HP/BP/Notch) with drive. Per-voice. One filter slot.
- Replace placeholder envs with full DAHDSR + curve.
- One LFO with shape selector (sine/saw/square/triangle/random/sample-and-hold).
- Filter cutoff modulatable from env and LFO via a minimal mod matrix (3 slots).
- **Acceptance:** classic subtractive sounds — filtered saw, modulated LP cutoff via env.

### Phase 4 — Voice manager + MPE + microtuning

- Polyphony, voice stealing strategy (oldest / quietest), legato/glide modes.
- MPE voice allocator: per-note channel detection, per-note pitch / pressure / Y.
- ODDSound MTS-ESP integration via FFI; SCL/KBM parsers.
- Tuning state in preset format.
- **Acceptance:** play with an MPE controller (Roli, Linnstrument, Osmose) — per-note pitch / pressure work; switch to a 24-edo MTS-ESP source and hear correct intonation.

### Phase 5 — Three-oscillator graph + sub + noise + unison

- Wire OSC B and C slots through `Engine` trait dispatch. Still WT-only as the engine type.
- Sub osc (sine/triangle, octave-down).
- Noise osc with band-pass post-filter.
- Unison up to 16 voices per oscillator with detune / blend / phase scramble / stereo spread.
- **Acceptance:** thick supersaw-class sounds.

### Phase 6 — FX rack v1 (single bus)

- `FxModule` trait + four modules: Compressor (FF, soft-knee), EQ (4-band parametric), Reverb (FDN), Delay (stereo with feedback filtering).
- Single bus only (Bus A); Bus B / C still scaffolded.
- FX rack UI panel.
- **Acceptance:** patches sound like finished sounds, not naked synth tones.

### Phase 7 — Full modulation system

- 10 LFOs with full shape set (incl. step sequencer mode, Path mode with X/Y outputs).
- 4 envelopes (full DAHDSR + per-stage curve, drawable in UI).
- 8 macros with smoothing.
- Mod matrix to ≥64 slots with remap curves, drag-and-drop assign from UI.
- **Acceptance:** full Vital-class modulation parity in workflow.

### Phase 8 — Three-bus parallel FX graph

- `FxGraph` with 3 buses + Splitter modules (L/H, L/M/H, M/S).
- More FX modules: Distortion (multi-mode), Chorus, Flanger, Phaser, Hyper (unison stack as FX).
- **Acceptance:** sidechain, parallel processing, multiband split work end to end.

### Phase 9 — Wavetable editor (in Svelte)

- Draw, formula, FFT-import modes.
- Audio import → resynthesis (FFT + frame slicing).
- WT export back to Serum-compatible WAV.
- This is a heavyweight UI phase; expect to extend `wavetable-view` component significantly.
- **Acceptance:** create a WT from scratch in-app; round-trip export/import preserves frames.

### Phase 10 — Additional engines

Order: Multisample → Sample → Granular → Spectral. Each lands as an `Engine` impl.

- **Multisample:** SFZ parser + voice region selection.
- **Sample:** single-sample playback with start/end/loop.
- **Granular:** classical grain cloud + texture mode; pitch/density/spread/jitter.
- **Spectral:** real-time STFT path, partial tracking, image-as-spectrum import. Highest-effort engine.

After this phase, Elixir reaches feature parity in oscillator-engine count.

### Phase 11 — Contrapunk integration

- `crates/contrapunk-audio` (or new `crates/contrapunk-elixir-bridge`) provides an `AudioBlock` wrapper around `elixir-engine::Synth`.
- HarmonyEngine state exposed as `ModSource::Harmony{Degree,Tension,Key,Mode}`.
- Contrapunk's host UI imports `@elixir/ui` workspace package and embeds Elixir's panels in its sound-source slots.
- Acceptance: load Elixir as a sound source in contrapunk, get the same UI, get harmony-aware modulation working.

### Phase 12 — Plugin shells (CLAP / VST3 / AU)

- Out of scope until earlier phases land. Design pass at that point will revisit:
  - Webview-in-plugin (Svelte UI) feasibility — `webview2` (Windows) / `wkwebview` (macOS) / `webkit2gtk` (Linux) embedded in the plugin window. Latency, resize, hi-DPI, multi-instance state — all need investigation.
  - Alternative: native UI via vizia for plugin shells only, accepting that the UI diverges from standalone.
  - Decision: defer.

### Phase 13+ — Stretch

- WaveEdit-compatible wavetable importer / exporter.
- Cloud preset library.
- Scriptable mod sources (Lua / WASM / Rhai — pick when phase scoped).
- Serum preset import (gated; see seed).

## 7. "Better than Serum" commitments — concrete

Mapping the four ambition axes from `elixir-design-decisions.md` decision #3 onto the roadmap:

| Axis | Phase(s) | Concrete commitment |
|---|---|---|
| Audio quality + perf | 2, 3, 5, 7 | Mipmap WT v1; FreqDomain quality mode in v2. Filter quality bench: aliasing < -90 dB at LP cutoff = 1 kHz when input is 22 kHz square. CPU bench: ≥256 voices on M1 Pro at 48 kHz / 256-sample buffer for stock saw + LP + reverb patch. |
| MPE / microtuning / MIDI 2.0 | 4 | Per-note pitch / pressure / Y from day one of voice work. MTS-ESP integrated. SCL/KBM. MIDI 2.0 internal voice resolution. |
| Open & inspectable | 1+ | RON preset format from Phase 1. WT format documented. Public stable plugin API for FX modules and engine types — third parties write modules in Rust, register at runtime. |
| Contrapunk integration | 11 | Harmony-aware mod sources (degree, tension, key, mode). `@elixir/ui` workspace package consumed by contrapunk UI. |

## 8. Risks and open questions

**Risks:**

- **Webview wavetable 3D view performance.** Mitigation: Phase 0 spike before committing to layout. Fallbacks documented in §5.6.
- **DSP scope vs solo bandwidth.** Mitigation: phased build; v1.0 is the wavetable engine alone; later engines slot in over time.
- **Plugin-shell UI strategy still open.** Mitigation: deferred until standalone ships; design pass will revisit.
- **`nih-plug` is solid but small-team.** Mitigation: plugin shell is deferred; revisit when needed; not on critical path.

**Open questions:**

- Exact license: MIT vs Apache-2.0 vs dual MIT/Apache. Decide before first push of `crates/elixir-engine`.
- Drag-and-drop modulation gesture for Svelte: HTML5 native vs `svelte-dnd-action` vs custom. Resolve in Phase 1.
- WebGL 3D view: hand-rolled vs `regl` vs `three.js`. Resolve in Phase 0 spike.
- FFT crate for spectral engine + WT FFT-import: `realfft`, `rustfft`, or `fundsp`'s wrapper. Resolve when Phase 2 starts.
- SCL/KBM parser: hand-rolled (small format) vs an existing crate. Hand-rolled likely.
- Audio-thread param smoothing: per-param config vs uniform 5 ms ramp default. Pick uniform as default with per-param override hatch.

## 9. References

- `.planning/notes/elixir-design-decisions.md` — locked decisions and revisions.
- `.planning/research/elixir/serum-features.md` — feature inventory (summary).
- `.planning/research/elixir/oss-prior-art.md` — OSS prior art + Rust ecosystem.
- `.planning/seeds/elixir-serum-preset-re-gate.md` — preset RE gate.
- `.planning/todos/pending/elixir-prereqs.md` — pre-implementation reading.
- contrapunk integration surface: `crates/contrapunk-audio/`, `crates/contrapunk-midi/`, `src-tauri/`, `ui/`.

---

End of design doc. Implementation should not begin before Phase 0 prereqs are complete.
