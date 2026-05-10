# Research: Group C — FX / Synth Sub-projects

**Issue(s):** #106, #105, #103, #97, #104
**Date:** 2026-05-11
**Researcher:** issue-researcher (manual, single-agent run)
**Verdict (one-liner per issue):**

| # | Title | Verdict |
|---|---|---|
| 106 | Wk3 jam: Drone + Bitcrusher (ship May 14) | **in-repo core** for v0 (deadline-driven) → **in-repo plugin** Lane/AudioBlock refactor for v1 |
| 105 | TextureFX corrosion-style distortion chain | **in-repo core** — extends `src/fx/`, one new AudioBlock |
| 103 | BeatMachineLane (step seq + sample playback) | **in-repo plugin** — companion `Lane` impl, depends on #97 |
| 97 | SamplerAudioBlock (sample-based playback) | **in-repo plugin** — new crate `crates/contrapunk-sampler/` |
| 104 | DDSP neural tone transfer | **external sub-project** — heavy ML, MIDI/OSC boundary |

This is a "mixed" group. The justification per issue is below.

## Shared architectural context (read once)

The codebase has three primitives relevant to every issue in this group:

- **`AudioBlock`** trait at `src/chain/block.rs:35-67` — synchronous DSP node, `process(&mut [f32], channels)` per audio buffer, RT-safe. Holds the contract every FX and every in-repo synth has implemented since `Synth`/`Reverb`/`Delay`.
- **`Chain`** at `src/chain/chain.rs:19-101` — linear pipeline of `Box<dyn AudioBlock>`, mutated via a lock-free SPSC `ChainCommand` queue (`src/chain/chain.rs:58-76`). The startup chain is assembled in `src-tauri/src/audio_clock.rs:128-141` as `[Synth → Delay → Reverb]`.
- **`Lane`** trait at `src-tauri/src/companion/lane.rs:144-189` (companion arch, commit `2c796ab`) — high-level *behavioral* unit: `phase()` ∈ {Sense, Mutate, Decide}, `tick(world) → LaneOutput`. Lanes emit `DispatchOp`s; the audio graph evolution (Part 2 of `.planning/jam-features-2026/01-companion-architecture.md`) replaces today's linear `Chain` with a typed DAG of `Instrument`s + `Mixer`s. Until that lands, Lanes route via the existing router → synth path.

The companion arch doc (`.planning/jam-features-2026/01-companion-architecture.md:441-552`) **explicitly catalogues BeatMachineLane** and bundles `DrumSampler` as an `Instrument` source on the future audio graph. That doc is the authoritative reference for #103 and the sampler discussion in #97.

Cross-surface note: the audio chain lives in `src-tauri` only. **WASM has no chain, no synth, no FX** (`wasm/src/lib.rs` is a harmony-engine binding only). Anything in `src/fx/` or `src/synth/` that the issues claim "works in WASM" is aspirational — `WasmAdapter.startAudioOutput` is a no-op (`CONCERNS.md:153-156`). This needs to be acknowledged in #105's and #106's "acceptance criteria" — they read as if WASM audio out exists.

CONCERNS-relevant: `PolySynth::process_stereo` allocates per callback (`CONCERNS.md:81-86`), and `process_callback` does `try_lock` on `Arc<Mutex<AudioState>>` (`CONCERNS.md:87-91`). Any sample-based work (#97, #103) must not compound that.

---

# Issue #106 — Wk3 jam: Drone layer + Bitcrusher FX (ship May 14)

## Problem
A Disasterpeace-themed jam needs (a) a sustained-pitch drone voice the user can pick + hold indefinitely, and (b) a bitcrusher FX block. Hard deadline: Thu May 14, 2026 EOD (3 days from today, 2026-05-11). Live on `app.contrapunk.com` for the jam Fri May 15.

## Touchpoints
- `src/synth/voice.rs:73-290` — existing 8-voice `Synth`, the closest reference for a new `DroneVoice` (same `AudioBlock` shape, same atomic-params pattern).
- `src/fx/mod.rs:16-21` — only exports `delay` + `reverb`; bitcrusher goes here.
- `src/chain/block.rs:35-67` — `AudioBlock` trait both pieces implement.
- `src-tauri/src/audio_clock.rs:128-141` — startup chain assembly; bitcrusher inserts between Synth and Delay or after Reverb. Drone is a *parallel* source, not a chain block — see entropy note below.
- `src-tauri/src/state.rs:127-174` — pattern for `Arc<DroneParams>` / `Arc<BitcrusherParams>` registration.
- UI: pattern of `ReverbPanel.svelte` / `DelayPanel.svelte` (assumed; `src/fx/reverb.rs:37-89` shows the atomic param shape Tauri commands feed).
- Prior brief: `.planning/jam-features-2026/03-drone-bitcrusher.md` (1-week-out skeleton — day-by-day plan exists, file diffs not yet filled in).

## Architecture verdict

**v0 (deadline path, MUST hit May 14):** in-repo core. Two new files inside the existing surface: `src/synth/drone.rs` (a second `AudioBlock` that emits a sustained two-osc tone, modeled exactly on `voice.rs`), and `src/fx/bitcrusher.rs` (a third AudioBlock alongside delay/reverb). Wire both into `audio_clock.rs:start` like Delay/Reverb are wired. No new crate, no new abstraction. The drone is added as a **second source block** in front of the synth — `Chain::process` iterates linearly, so a drone block that writes to the buffer at position 0 + a synth block that mixes-in at position 1 is a one-line change in `Synth::render` (it already overwrites; change to additive when not the first source) **or** a single mixer block. The cheapest path: temporarily make Drone its own `AudioBlock` and accept that it's only audible when Synth is muted, then ship a 2-into-1 mixer block on Tue. The brief at `.planning/jam-features-2026/03-drone-bitcrusher.md:11-22` is consistent with this.

**v1 (post-jam refactor, do not block ship):** the companion arch doc (`01-companion-architecture.md:251`) lists **DroneLane** as a tick-only Decide-phase Lane, and the audio-graph migration moves Drone into a separate `Instrument` source. Refactor `Drone` from `AudioBlock` source to a `DroneLane` emitting `DispatchOp::NoteOn` on its own Instrument once the audio graph lands. Bitcrusher stays as an `AudioBlock` regardless (FX = chain node, not Lane).

Entropy framing: this is the right call because (a) the deadline forecloses anything more ambitious, (b) the AudioBlock surface is the trait the user already knows + has parameter-binding scaffolding for, (c) v1 refactor is a same-author rename, not a redesign. The "tax" is that for ~2-4 weeks Drone is a chain block with a one-source assumption; the architecture doc (`01-companion-architecture.md:593`) already flags the single-synth assumption as a known limitation.

## Implementation outline (3-day plan)

**Mon May 11 (today)** — scaffold:
1. Create `src/fx/bitcrusher.rs` and `src/synth/drone.rs` skeletons. Both implement `AudioBlock` with name/type_id/process. Both have an `Arc<{Name}Params>` with atomic fields (mirror `ReverbParams` at `src/fx/reverb.rs:37-89`).
2. Add to `src/fx/mod.rs` exports.
3. Add `bitcrusher_params: Arc<BitcrusherParams>` and `drone_params: Arc<DroneParams>` fields to `src-tauri/src/state.rs:127`. Register block descriptors in `audio_clock.rs:128-141`.

**Tue May 12** — DSP:
1. `DroneVoice`: two phase-accumulator oscs (sine/saw/square selectable), detune via `osc2_phase_inc = osc1_phase_inc * (1.0 + detune_hz/freq)`. ADSR with `sustain_level = 1.0` and infinite hold (release on `set_active(false)`). Polyphony = 1; voice is owned by the block. Use `midi_to_freq` from `voice.rs:292`.
2. `Bitcrusher`: bit depth `q = 2^bits / 2`, output = `(x * q).round() / q`. Sample-rate crush via sample-and-hold counter (issue body code is correct as-is).
3. Tauri commands: `drone_on(note: u8, waveform: u8, detune: f32)`, `drone_off()`, `set_bitcrusher_bits(f32)`, `set_bitcrusher_rate(f32)`. Pattern from `set_reverb_mix`-style commands.
4. Unit tests at this point: `bitcrusher_8bit_quantizes_to_256_levels`, `drone_held_silence_after_off`, `bitcrusher_disabled_is_passthrough`.

**Wed May 13** — UI + descope checkpoint:
1. `ui/src/lib/components/DronePanel.svelte` (pitch picker, waveform 3-button radio, detune slider 0-2Hz, on/off).
2. Bitcrusher knobs in existing `FxPanel.svelte` (bits 4-24, rate 0.01-1.0, enabled toggle).
3. **Descope gate**: if either piece isn't audibly working by 18:00, drop bitcrusher per brief `03-drone-bitcrusher.md:30-32`.

**Thu May 14** — polish + ship:
1. Wire to `app.contrapunk.com` (Cloudflare Pages auto-deploy via website repo).
2. Demo recording: low D drone + Lydian melody + bitcrush sweep → `cover/demos/03-drone.mp4` in website repo (private repo per memory).
3. Manual smoke test on macOS + browser. CI green check.

## Test strategy
- **First test (TDD entry)**: `bitcrusher_disabled_is_passthrough` — copy-paste from `reverb.rs:316-322`.
- Unit: bit-depth produces ≤ `2^bits` unique output levels; rate-crush at ratio 0.5 holds 2 consecutive samples; drone with `active=true` produces sustained nonzero output for 5 seconds of buffer; drone `off` → silence within `release` ms.
- Integration: chain `Drone → Bitcrusher → Delay → Reverb` doesn't NaN; CPU < 5%.
- Manual UAT: hold a low D for 30s, sweep bit depth 24→4 audibly, no clicks on engage/disengage. Done.

## Dependencies
None new. DSP is hand-rolled (issue body provides full math). No `fundsp`, no `hound`. Estimated diff: ~400 lines Rust, ~120 lines Svelte, zero new Cargo deps.

## Entropy impact
- New surfaces: 2 AudioBlock files, 4 Tauri commands, 2 UI panels. No new crate.
- Affects: `src/synth/mod.rs`, `src/fx/mod.rs`, `src-tauri/src/state.rs`, `src-tauri/src/audio_clock.rs`, `src-tauri/src/main.rs` (param wiring). All single-line additions next to existing patterns.
- WASM: **not affected** — there's no Chain in `wasm/src/lib.rs`. Acceptance criterion "Works in WASM" in the issue is **inaccurate today**; flag back to the user. Browser users will not hear the drone or the bitcrusher unless the no-op `WasmAdapter.startAudioOutput` (`CONCERNS.md:153-156`) is fixed first — out of scope for this ship.
- Risk of regression: tiny. Drone block sits parallel-ish to Synth (see v0 trade-off); bitcrusher inserts between blocks but pass-throughs when disabled. Existing chain tests stay green.

## Open questions / blockers
- **WASM acceptance criterion is wrong** — get user confirmation that desktop-only ship is acceptable for the Wk3 jam, or scope in a quick "browser fallback note" panel.
- Drone-as-source vs Drone-as-second-source: confirm v0 path Tuesday morning before writing the mixing logic. If user wants drone *over* a regular harmony jam, the cleanest hack is a `Mixer` AudioBlock between Drone+Synth and Bitcrusher.

## Estimated effort
**S (1-3 days)** — scoped to the deadline. v1 Lane refactor is a separate **S** later.

---

# Issue #105 — TextureFX Corrosion-style distortion chain

## Problem
Add a multi-stage texture FX block — tape saturation → bitcrush → noise injection → post-filter — to give Contrapunk harmony voices a Disasterpeace / lo-fi / industrial aesthetic. Complements Reverb + Delay in `src/fx/`.

## Touchpoints
- `src/fx/mod.rs:16-21` — adds `texture` module + export.
- `src/fx/reverb.rs:37-89, 176-305` — copy the `XxxParams` atomic struct + `AudioBlock` impl shape verbatim.
- `src-tauri/src/audio_clock.rs:128-141` — register a `builtin.texture` block descriptor; chain becomes `[Synth → Texture → Delay → Reverb]` (or Texture-after-Reverb depending on preset goals; default before-Delay matches Logic Corrosion's "guitar amp pre-delay" position).
- `src-tauri/src/state.rs:127-174` + `commands/` — `texture_params` field, knob commands.

## Architecture verdict
**in-repo core.** TextureFX is a single `AudioBlock` in `src/fx/texture.rs` — identical surface to `Reverb`/`Delay`, identical lifetime (params atomics, lock-free dispatch). It needs zero abstractions the codebase doesn't already have, ships on the desktop surface only (FX chain doesn't run in WASM today), and has no independent release cadence pressure. The issue body proposes `fundsp` as the implementation vehicle — that's the only meaningful decision.

**`fundsp` recommendation: don't add it for this issue.** The proposed chain (tanh + quantize + white-noise sum + 1-pole LP) is ~40 lines of hand-rolled DSP. `fundsp` is dual-licensed Apache/MIT with active development (latest 0.23.0, no_std-capable per the GitHub README), so it's a future-safe option, but pulling it in for *one* FX is a ~200KB+ compile-time cost and a new dep surface. Defer `fundsp` until 3+ FX want it (Elixir's design doc already eyes it, see `.planning/research/elixir/` prereqs). For #105 specifically: hand-roll, file the "should we standardize on fundsp" decision for a separate sweep.

## Implementation outline
1. **TDD start**: `texture_disabled_is_passthrough` test, copy from `reverb.rs:316-322`. Failing.
2. `TextureParams` struct with atomics: `enabled`, `drive_ppt`, `bits_x100` (4-24 bits = u32 0-2400), `rate_ratio_ppt`, `noise_level_ppt`, `cutoff_hz`. Mirror `ReverbParams::default()` defaults from issue table.
3. `Texture` struct holds `params: Arc<TextureParams>`, sample-rate, internal state (1-pole LP memory per channel, sample-hold counter).
4. `process()` per frame:
   - `pre = x * drive`
   - `sat = pre.tanh()` (cheap soft clipper)
   - `crushed = (sat * q).round() / q` where `q = 2^bits / 2`
   - `held = sample_hold(crushed, rate_ratio)`
   - `noisy = held + (rng() * 2.0 - 1.0) * noise_level`
   - `out = lp.process(noisy, cutoff)` — same `alpha = 1 - exp(-2π·cutoff/sr)` one-pole as `voice.rs:185`.
5. Presets (Clean / Tape / Lo-Fi / Destroy from issue): JSON in `src-tauri/src/commands/texture.rs` or hardcoded constants. Simple `set_texture_preset(name)` command applies all params.
6. UI: `TextureFxPanel.svelte` with 5 knobs + 4 preset buttons. Mirror existing FX panel layout.
7. RNG note: `rand::thread_rng()` allocates; use a `XorShiftRng` seeded once at construction and store in the struct. Audio-thread safe.

## Test strategy
- Unit (each preset): bit reduction at 8 bits produces ≤ 256 unique samples in 1024 of swept-sine input; lowpass at 1kHz attenuates 8kHz sine by >20dB; pass-through when disabled.
- Golden: 100ms of sine through "Destroy" preset → snapshot peak / spectral centroid / RMS into a `tests/fixtures/texture_destroy.json`; assert tolerances.
- Manual: A/B Reverb-only vs Texture+Reverb on a held chord. Subjective but the "Tape" preset should sound warmer not louder.

## Dependencies
None new. `fundsp` is **rejected for this issue** (rationale above) — flag for a future "FX framework consolidation" research item.

## Entropy impact
- One new file (`src/fx/texture.rs`), one export line, one descriptor in startup chain, ~10 lines in state.rs, ~3 Tauri commands. Identical shape to `Reverb` so review burden is minimal.
- WASM: same as #106 — not in scope. Issue's acceptance "Works in WASM build" is inaccurate; flag to user.
- nih-plug surface: TextureFx is an `AudioBlock` and `plugin/` (CLAP plugin host) consumes the Chain, so it ships everywhere the chain ships. Free.

## Open questions
- Default chain position: pre-Delay (Logic Corrosion convention) or post-Reverb (heavy-handed master FX)? Recommend pre-Delay; defer to user.
- Should presets live as JSON files for user-shareable presets or as code constants? Match whatever `reverb`/`delay` do (currently constants); revisit if presets explode.

## Estimated effort
**S (1-3 days)** — 1 day DSP, 0.5 day tests, 0.5 day UI panel, buffer for polish.

---

# Issue #103 — BeatMachineLane (step sequencer + sample playback)

## Problem
A step-sequencer-driven drum lane in the companion: tempo-synced, sample-playing, swing-enabled, with per-pad sample assignment. Needed for Fred Again / Justice production workflow inside Contrapunk.

## Touchpoints
- `src-tauri/src/companion/lane.rs:144-189` — `Lane` trait. BeatMachine is a **Decide-phase** Lane (catalog: `01-companion-architecture.md:251` lists it explicitly).
- `src-tauri/src/companion/orchestrator.rs:1-664` — Companion runs lanes; BeatMachine registers here. Read this file before implementing.
- `crates/contrapunk-transport/` — beat clock the Lane reads (`world.transport.totalBeat`).
- `01-companion-architecture.md:441-552` — **the design is already drafted in the arch doc**, including data model, lane behavior, and 6-phase build plan (A through F, 10-15d total). The issue body re-states a subset of this. Read the arch doc as source of truth.
- Depends on #97 — `SamplerAudioBlock` provides the `Sampler` / `SampleBuffer` types BeatMachine triggers. Without #97, BeatMachine has no way to make sound.

## Architecture verdict
**in-repo plugin** — implements as `companion/beat_machine.rs`, registers as a `Box<dyn Lane>` in the companion orchestrator. This is the **canonical** Lane impl beyond the (deleted) pattern programmer; building it validates the abstraction for everything downstream (LooperLane, ArpeggiatorLane, DroneLane). The companion arch doc was explicitly drafted with BeatMachine as the second concrete Lane (after LooperLane).

**Do not** put BeatMachine in `src/chain/` or `src/synth/`. The Lane abstraction exists *precisely* to keep behavioral logic (step scheduling, swing, fills) outside the audio-buffer-processing layer. The Lane emits `DispatchOp::NoteOn` events; a `Sampler` AudioBlock (#97) renders them into audio. Clean separation.

Entropy framing: this is a free architectural win — BeatMachineLane fits a pattern the codebase has invested in, in a phase (Decide) the orchestrator runs every iteration anyway, with zero new abstraction needed. The cost is one new file (~300 LOC for v1) plus one UI panel. Lane state serializes via existing `serialize_state` (`lane.rs:179`) for rig persistence.

## Implementation outline (matches arch doc § Build phases)
1. **Phase A (prereq)**: #97 ships first. Without `Sampler` registered as an instrument target, BeatMachineLane has no output.
2. **Phase B — Lane skeleton**: `BeatMachineLane { steps: Vec<Step>, current_step: usize, step_length_beats: f32, swing: f32, samples: Vec<Arc<SampleBuffer>>, target: VoiceOutputTarget }`. Impl `Lane::phase() → LanePhase::Decide`, `input_filter() → InputFilter::None`, `tick(world) → LaneOutput`. Compute step index from `world.transport.totalBeat / step_length_beats`; on step change, push `DispatchOp::NoteOn` if active.
3. **Phase C — Pattern data**: hardcode a 16-step rock pattern at first. JSON-loadable later.
4. **Phase D — Swing + groove**: offset even steps by `swing * step_length_beats`. Jitter parameter (0-10ms) per arch doc § "Groove quantise".
5. **Phase E — UI**: `BeatMachinePanel.svelte` 4x16 grid, drag-drop WAV-to-pad, swing knob, on/off per step. Per-step velocity via drag-y-axis.
6. **Phase F — Polish**: voice-stealing handled by the `Sampler` (Phase 97), not the Lane. Mute/solo per pad. Pattern length 8/16/32 switch.

The arch doc projects 10-15d for BeatMachine v1 + a separate `DrumSampler` Instrument; #97's sampler is a near-superset, so a tight implementation can collapse those to ~8-10d.

## Test strategy
- **First test**: `beat_machine_lane_emits_kick_on_beat_one` — construct lane with `[kick, off, off, ..]`, tick at `total_beat = 0.0`, assert `LaneOutput.ops` contains a `NoteOn`. From there fan out per step.
- Unit: step advancement at varied step_length (1/8 vs 1/16); swing shifts even steps by exactly `swing * length`; pattern-length-8 wraps at index 7; serialize/deserialize roundtrip preserves all 16 steps.
- Integration: register Lane in a test Companion + mock `WorldState`, advance transport over 16 beats, assert sequence of NoteOns matches pattern.
- Manual UAT: 120 BPM, pad 1 kick, pad 2 snare on 2 & 4; sounds like a beat. Swing 0.25 audibly behind. Done.

## Dependencies
- `crates/contrapunk-sampler/` (#97) for sample playback. Hard dep.
- Optionally: `serde_json` for pattern persistence (already in workspace).

## Entropy impact
- One new file in `src-tauri/src/companion/`, one new UI panel, ~5 Tauri commands (load pattern, toggle step, set swing, set pad sample, on/off lane).
- Zero impact on `src/chain/`, `src/fx/`, `src/synth/`, WASM, `plugin/` (companion is desktop-only today).
- Future migration: when audio graph (Part 2 of arch doc) lands, `target: VoiceOutputTarget` becomes `target: InstrumentId`. Trivial rename.

## Open questions
- Ableton Link sync (issue acceptance criterion) **depends on #99** — out of scope; gracefully degrade to transport-only.
- Default kit: bundle 4-8 small WAVs in `assets/drums/`, or require user-supplied? Recommend bundle (≤500KB), `hound` decode once at startup (#97 ships the loader).
- "Logic Drummer-style smart drummer" path in the arch doc (`01-companion-architecture.md:445`) is far more ambitious than the issue body's step grid. Confirm scope: v1 = step grid (issue body), v2 = preset+intensity smart drummer (arch doc). Recommend ship v1 first.

## Estimated effort
**M (3-7 days)** for v1 step grid after #97 lands.

---

# Issue #97 — Sample-based playback engine (SamplerAudioBlock)

## Problem
A polyphonic, pitch-shifting WAV sampler so harmony voices can play through user-loaded instrument samples (cello, strings, piano) instead of the built-in oscillator synth. Enables instant string-ensemble from a single keyboard input.

## Touchpoints
- `crates/contrapunk-audio/` — workspace member, currently exists; this is the natural home for `SampleBuffer`. Confirm contents (issue suggests adding `sample_buffer.rs` here).
- `src/synth/voice.rs:73-290` — reference for the polyphonic voice-management pattern (8 voices, retrigger, voice-steal by age). Sampler reuses this nearly verbatim.
- `src/chain/block.rs:35-67` — Sampler is an `AudioBlock`, registered like Synth.
- `src-tauri/src/audio_clock.rs:128-141` — registers Sampler either alongside or instead of Synth (config-driven). Today's chain assumes one source at position 0 (`01-companion-architecture.md:593`) — when Sampler is enabled, Synth needs to bypass or coexist via a future Mixer block.
- File loading: new dep `hound = "3.5"` per issue. Lives in `crates/contrapunk-sampler/` (new) or `crates/contrapunk-audio/` (existing).

## Architecture verdict
**in-repo plugin** — new workspace crate `crates/contrapunk-sampler/`. The reasoning:

1. Sampler is **large** (issue scopes ~400-600 LOC across `voice.rs`, `sampler.rs`, `sample_buffer.rs`, interpolation, WAV loader). Putting it in `src/sampler/` pollutes the top-level binary crate; putting it in `crates/contrapunk-sampler/` keeps a clean module boundary.
2. It's **independently testable** — `cargo test -p contrapunk-sampler` runs DSP unit tests without pulling in Tauri / cpal.
3. It's **independently versionable** — when nih-plug plugin format ships, the sampler crate is reusable as a standalone plugin without depending on `src-tauri`.
4. **`crates/contrapunk-audio/` already exists** for shared audio types; `SampleBuffer` (issue suggests there) belongs there. The voice/sampler implementation belongs in a new `contrapunk-sampler` crate that depends on `contrapunk-audio`. This mirrors the existing split (`contrapunk-chord`, `contrapunk-harmony`).

This is *not* an external sub-project candidate (unlike Elixir): the sampler is small, harmony-adjacent (it's the sound source for harmony voices), and synchronous with the audio thread. No release cadence pressure. The Elixir precedent in `.planning/todos/pending/elixir-prereqs.md` is for a **full synthesis platform** (5 engines, mod matrix, FX graph), not a sampler.

Entropy framing: one new crate is the marginal cost. The crate replaces 0 existing files (`src/sampler/` doesn't exist) and unblocks #103, future ListenLane (#102), and future DDSP-target-sample-bank (#104 fallback). High ROI for a small surface.

## Implementation outline
1. **TDD start**: in `crates/contrapunk-sampler/tests/`, write `sampler_plays_root_pitch_unchanged` — load a 440Hz sine WAV at root note A4, trigger NoteOn(69), assert output is approximately the same buffer.
2. **`crates/contrapunk-sampler/Cargo.toml`**: deps = `contrapunk-audio` (workspace path), `hound = "3.5"`, no Tauri.
3. **`SampleBuffer`** in `crates/contrapunk-audio/src/sample_buffer.rs` per issue body. `load_wav(path: &Path) -> Result<SampleBuffer>` using `hound::WavReader`. Convert int formats to f32.
4. **`SamplerVoice`** with `playhead: f64`, `playback_rate = 2f64.powf((target - root) / 12.0)`, ADSR copied from `synth/voice.rs:22-69`, Catmull-Rom interpolation:
   ```
   y = 0.5 * ((2*p1) + (-p0 + p2)*t + (2p0 - 5p1 + 4p2 - p3)*t² + (-p0 + 3p1 - 3p2 + p3)*t³)
   ```
   where `p0..p3 = buffer[idx-1..idx+2]`, `t = playhead.fract()`. Standard formulation.
5. **`Sampler`** struct: `voices: [Option<SamplerVoice>; 8]`, `slots: Vec<SampleSlot>` (multisample, v2), `params: Arc<SamplerParams>`. Impl `AudioBlock`. Voice-steal-by-age: copy from `voice.rs:104-115`.
6. **Tauri commands**: `sampler_load_wav(path)`, `sampler_set_root_note(slot, note)`, `sampler_set_enabled(bool)`, `sampler_clear()`.
7. **UI**: `SamplerPanel.svelte` with drag-drop file zone + WaveformDisplay (canvas) + root-note picker. Loop point editor is v2.
8. **Chain integration**: register `builtin.sampler` block descriptor in `audio_clock.rs`. **Toggle Synth ↔ Sampler at the chain level** for v1 (only one source active at a time); parallel sources wait for the audio-graph migration (Part 2 of arch doc).

## Test strategy
- **First tests** (in order):
  1. `wav_loader_roundtrips_f32_data` — write a 1kHz sine via hound, load it back, samples match.
  2. `sampler_plays_root_pitch_unchanged`.
  3. `pitch_shift_one_octave_doubles_rate` — assert `playback_rate ≈ 2.0` for `target=72, root=60`.
  4. `voice_steal_does_not_panic` — copy from `voice.rs:382-397`.
- Quality: render 440Hz sample at 880Hz (one octave up) for 1 sec, assert peak spectral bin at ~880Hz (within ±10Hz). Catmull-Rom should beat naive linear here.
- Integration: load a real cello WAV, play a C-E-G chord via MIDI, listen — manual UAT. 8 voices simultaneous, no clipping at default gain.
- WASM build: omit (sampler doesn't ship to WASM in v1 — Web Audio output is a no-op).

## Dependencies
- **`hound = "3.5"`** — pure-Rust WAV decoder/encoder. ISC license, stable, last release 2023 but feature-complete (WAV is a frozen format). ~120KB compiled. Safe pick.
- **No** `symphonia` for v1: WAV-only is fine, multi-format decode (FLAC, MP3, OGG) is v2 — flag as a follow-up.
- No `rubato` / `samplerate` — Catmull-Rom interpolation in-line is enough quality and avoids a heavy dep.

## Entropy impact
- One new crate (`crates/contrapunk-sampler/`), one new file in `crates/contrapunk-audio/` (`sample_buffer.rs`), ~5 Tauri commands, one UI panel.
- Affects existing files: `src-tauri/src/audio_clock.rs` (register block), `src-tauri/src/state.rs` (`sampler_params: Arc<SamplerParams>`), `Cargo.toml` (workspace member, hound dep).
- nih-plug surface: `Sampler` is an `AudioBlock`, the plugin host consumes the chain, so it ships with the plugin build. Free.
- WASM: doesn't ship (no audio output in WASM). Eventually ports when Web Audio sampler lands.
- Audio-thread risk: WAV loading is **not** RT-safe (file I/O, allocation). Load happens on main thread, hand `Arc<SampleBuffer>` to the audio thread via `ChainCommand::PushBlock`. Standard pattern.

## Open questions
- Multi-sample zones (issue mentions `slots: Vec<SampleSlot>` for v2) — recommend single-sample-per-voice v1, multisample v2. Document in module-doc.
- Sample-format support beyond WAV: defer to v2, `symphonia` is the standard pick.
- Memory budget per loaded sample: enforce a 100MB-per-buffer soft cap? Surface to UI on load failure.

## Estimated effort
**M (3-7 days)** for v1: WAV-only, single-zone, 8-voice, Catmull-Rom. Multisample + loop points add another **M** in v2.

---

# Issue #104 — DDSP tone transfer via neural DSP

## Problem
Render harmony voices through neural-DSP instrument models (violin, cello, flute, trumpet) trained per-instrument. A ~2MB ONNX decoder maps `(pitch_hz, loudness_db)` → harmonic amplitudes + noise filter coefficients → additive synthesis. Lightweight alternative to multi-GB sample libraries.

## Touchpoints
- `crates/contrapunk-ml/` — **does not exist yet**. Issue body says "depends on #102 (ListenLane)" — that issue likely creates the crate. If #102 hasn't shipped, #104 inherits the cost of standing up `crates/contrapunk-ml/`.
- `src/chain/block.rs` — DDSPVoice would be an `AudioBlock` *if* in-repo.
- Tauri webview fallback path (issue body): `@magenta/ddsp-tfjs` in Svelte UI, audio via postMessage. Lives entirely in `ui/`.

## Architecture verdict
**External sub-project** — DDSP does *not* belong in the Contrapunk binary or workspace. Justification:

1. **Binary size**. `tract` (latest 0.21.15 March 2026, dual MIT/Apache, active per GitHub) is the smallest pure-Rust ONNX runtime and **still adds several MB to the binary plus its dependency tree** (the README highlights Raspberry-Pi-Zero performance, not WASM/desktop binary size). `ort` (ONNX Runtime FFI) is faster but adds a 20-40MB native lib + system dependency. Either choice **doubles or triples** the Contrapunk install size for a feature that's only used per-voice on opt-in.
2. **WASM target**. The Magenta DDSP-tfjs path is browser-only; the Rust `tract` path doesn't currently advertise WASM support (sonos/tract README is silent on wasm32). So in-repo Rust DDSP fragments across surfaces — desktop-only, with a separate webview implementation. Two codepaths, one feature.
3. **Release cadence**. DDSP models churn (new instruments, retrained weights). A separate `contrapunk-ddsp` repo + standalone process means model updates ship without a Contrapunk release.
4. **Real-time math is heavy**. Issue claims "~3ms per voice" — that's *if* a 60-harmonic additive synth runs inline after the ONNX inference. 4 voices × (inference + 60-osc add) on the audio thread is ambitious; one missed callback = audible glitch. Better to render off-thread and stream samples in.
5. **Elixir precedent applies here**. The user already runs a heavy synthesis sub-project externally (Elixir, see `.planning/research/elixir/DESIGN.md`). DDSP fits the same "heavy DSP, opt-in, separate release cadence, communicates via MIDI/audio" pattern.

**Recommended architecture**: standalone Rust binary `contrapunk-ddsp` (separate repo, eventually) that:
- Loads ONNX models via `tract` (best size/perf, pure-Rust, MIT/Apache).
- Listens on a UDP/OSC port for `(voice_id, pitch_hz, loudness_db, gate)` messages.
- Renders to a virtual audio device (BlackHole/Soundflower on macOS, JACK on Linux, ASIO loopback on Windows) **or** streams audio back over UDP.
- Contrapunk's `MidiRouter` (forthcoming in the audio-graph migration) targets `InstrumentId::External(ddsp_id)` and dispatches MIDI to it.

Boundary cost: one UDP socket, one schema document, one Tauri command to launch/connect the helper. Far cheaper than 5-15MB of ONNX runtime in every Contrapunk install.

**v0 fallback (if user insists on in-repo)**: ship the **`@magenta/ddsp-tfjs` browser path only**, gated to the WASM/web surface, since the issue says it can already run in the Tauri webview. Zero Rust dep cost. Limitation: webview latency, no desktop-CLAP-plugin support. Document as "experimental, browser-only" and revisit when an external sub-project is feasible.

Entropy framing: a Rust ONNX runtime is the **single largest dep we could add** to the workspace right now. Compared to `fundsp` (~200KB) or `hound` (~120KB), `tract` is 1-2 orders of magnitude heavier. The external-sub-project boundary is the right place to pay that cost.

## Implementation outline
**External path (recommended):**
1. Create new repo `contrapunk-audio/contrapunk-ddsp` (parallel to `contrapunk-audio/website`). Rust binary, MIT-licensed.
2. Pull Magenta's pretrained ONNX models (Apache 2.0 — issue confirms license).
3. Build: `tract`-based decoder + additive synth + filtered noise → samples. Output via `cpal` to a virtual device OR over UDP.
4. Define wire protocol: small OSC schema, `/ddsp/voice <id:i> <pitch_hz:f> <loud_db:f> <gate:i>`.
5. In Contrapunk: add `DdspExternalInstrument` to `MidiRouter` matrix (when audio-graph lands). Until then, prototype via a Tauri command that just connects/disconnects.
6. UI: instrument selector per voice + "connect to contrapunk-ddsp" launch button in `TimbrePanel.svelte`.

**Browser-only v0 path (interim):**
1. `npm install @magenta/ddsp-tfjs` in `ui/`.
2. Svelte component loads model, takes pitch/loudness from a Tauri/WASM event stream, renders audio via Web Audio.
3. Limited to WASM/web build; document as "experimental".

## Test strategy
- **Spike (BEFORE building anything)**: 1-day proof-of-concept loading Magenta's violin ONNX in `tract`, running 100 inferences in a benchmark, measuring real-time-factor. If < 1.0 (slower than realtime) for 4 voices on the user's M-series Mac, the whole architecture changes (need ORT, GPU, or model quantization).
- Integration: external helper running, send a chromatic scale via OSC, record output, FFT for pitch accuracy.
- A/B: violin DDSP vs sine-osc on the same MIDI input — subjective UAT.
- Latency: measure round-trip MIDI→OSC→helper→audio→speaker. Target < 30ms end-to-end (issue says < 20ms additional — tight).

## Dependencies (external helper)
- `tract = "0.21"` (March 2026, MIT/Apache, active, ~3-5MB compiled). **Or** `ort` (faster, larger, system-dep on ONNX Runtime).
- `rosc` for OSC, `cpal` for audio out. Both already known to the user.
- Magenta pretrained ONNX models, Apache 2.0.

## Entropy impact (in-repo path, for comparison)
- Adding `tract` to the Contrapunk workspace: 3-5MB binary growth, ~20s extra CI build time, new Rust 1.91+ MSRV requirement (per tract README) — current MSRV is presumably lower. Big footprint.
- Adding the magenta-tfjs path: ~5MB of JS/WASM in the UI bundle. Manageable but still meaningful.
- **External path entropy in main repo: zero.** One OSC schema doc + one Tauri command. Versus 3-5MB binary growth.

## Open questions / blockers
- **Critical spike needed**: tract real-time-factor on Apple Silicon for a Magenta DDSP violin model. If RTF >> 1.0, in-repo path is dead and only external (with potential GPU acceleration) survives.
- Does Magenta still publish pretrained ONNX (issue assumes yes)? Verify via `https://github.com/magenta/ddsp` releases — they shifted focus around 2023-2024, models may be archived.
- #102 (ListenLane) prerequisite: does it create `crates/contrapunk-ml/`, and is that crate sized for the ONNX runtime weight? If #102 is doing audio-feature ML (pitch tracking?), DDSP's needs may differ.
- User's appetite: Elixir is already running parallel as an external project. Is DDSP-as-external-helper acceptable, or is the user trying to keep this inside the main repo for some reason?

## Estimated effort
- **Spike**: **XS (≤1 day)**.
- **External sub-project**: **XL (>3 weeks)** for v1 — new repo, build system, ONNX integration, OSC/audio bridge, model loading, UI wiring on the Contrapunk side.
- **Browser-only v0**: **M (3-7 days)** — magenta-tfjs is well-documented; can ship as a curiosity in the WASM build without committing to architecture.

---

## Cross-issue notes

### Dedup opportunity: #97 ↔ #103
The arch doc (`01-companion-architecture.md:540-552`) splits sampling into **two pieces**: a "DrumSampler Instrument" (Phase A, 2-3d) and a `BeatMachineLane` (Phase B, 1-2d). The issue body of #97 describes a *general-purpose* sampler that pitch-shifts melodically. These are the same substrate with different config:
- **Pitch sampler (#97)**: pitch-shift on NoteOn, sustain-loop, melodic voice allocation.
- **Drum sampler (#103's helper)**: no pitch-shift (or coarse-tuning only), one-shot trigger, percussive voice allocation.

Recommend **#97 ships first as a single `Sampler` AudioBlock with a "mode" toggle** (Melodic / Drum). #103 then **reuses #97's Sampler** with `mode = Drum` and just adds the step-grid scheduling Lane on top. Saves ~3-5 days vs implementing them independently.

The issue body of #97 already lists "Enables: BeatMachineLane (drum sample playback)" — author already saw this. Confirm with user before splitting effort.

### What's *not* in this group
The issue numbers suggest there are sibling issues (e.g. #99 Ableton Link, #102 ListenLane) referenced as dependencies but not in scope here. #103's Link sync and #104's `contrapunk-ml` crate prerequisite both depend on issues outside this group. Note these as cross-group blockers for the planner.

### Tempo: 5 issues, 5 architectural verdicts, ~1 "external" decision
The verdict mix (3 in-repo + 1 mixed + 1 external) feels right for this group:
- Drone + Bitcrusher (#106) + TextureFX (#105): trivially in-repo, AudioBlock pattern is the right home.
- BeatMachineLane (#103) + Sampler (#97): in-repo *plugin* — Lane abstraction + new crate, exactly the surface the codebase has been building toward (`crates/`, companion arch).
- DDSP (#104): the **one** issue where heavy deps and release-cadence pressure justify an external sub-project. The other four don't.

If the verdict mix had been "everything in-repo" or "everything external", that would be a red flag — the cost/benefit asymmetry across these issues is what the entropy framing exists to surface.
