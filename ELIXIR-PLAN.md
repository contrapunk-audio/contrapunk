# Elixir Implementation Plan

**Status:** v0.2 — six core decisions locked (see §10)
**Companion to:** [ELIXIR-DESIGN.md](./ELIXIR-DESIGN.md)
**Premise:** Build Elixir incrementally so that every phase ships something audible, the existing Contrapunk synth keeps working until cutover, and we land plugin hosting (VST3 + CLAP + AU, multi-plugin chains) on the same chain abstraction.

### Locked decisions

1. Elixir crates live **inside the contrapunk workspace** (path deps; extractable to a sibling repo later if needed).
2. A-Cut is gated on **full feature parity (after A6)**, not minimum-viable (after A5). Cutover week shifts from ~8 → ~12.
3. Standalone UI uses **`egui`**. Elixir's standalone app opens as a **separate process / window** from Contrapunk — by design, not a regression.
4. Track C ships **all three plugin formats (CLAP + VST3 + AU)** for v1.
5. `elixir-standalone` is a **public product released from this (contrapunk) repo** — shared release infrastructure, shared signing certs, separate binaries.
6. This plan + the design doc **will be ingested into `.planning/`** as a real GSD milestone (run `/gsd-ingest-docs` after this commit).

---

## 0. TL;DR

Three parallel tracks. Each phase is a ship gate — nothing merges that doesn't pass the gate's audible/observable test.

| Track | What it produces | Phases | Wall-clock |
|---|---|---|---|
| **A. Elixir replaces Contrapunk's synth** | `src/synth/` deleted, replaced by `elixir-core` instance behind a feature flag, then flag flipped to default. **A-Cut waits until full feature parity (post-A6).** | A0 → A1 → … → A6 → **A-Cut** → A7 | ~13 weeks |
| **B. Elixir as standalone product** | `elixir-standalone` binary (cpal + midir), `elixir-plugin` (VST3 + CLAP + AU via nih-plug), `elixir-headless` renderer, `egui` UI. Released publicly from this repo. | B0 → B1 → … → B9 | ~19 weeks (overlaps A) |
| **C. Multi-plugin hosting in Contrapunk** | Finished CLAP host, new VST3 host, AU on macOS; user can load N plugins, chain them, route the harmony output through them | C0 → C1 → C2 → C3 → C4 | ~8 weeks (overlaps A+B) |

**The three tracks share one engineering substrate:** the `Chain` / `AudioBlock` abstraction in `src/chain/`. That's already in place and has a lock-free SPSC command queue (`ChainCommand::PushBlock` etc.). All three tracks plug into it.

**Total calendar time** if one developer works strictly serially: ~40 weeks. With Track B running parallel to A (most of B reuses A's core), and Track C interleaved on different files, **realistic shipping window is 24–28 weeks**.

---

## 1. Workspace layout

### Crates added (inside this repo)

```
crates/
  elixir-core/           # The DSP engine. no_std-friendly. No host code.
                         #   - voice handler, oscillator, filter, envelope, LFO,
                         #     modulation matrix, FX, preset structs
                         #   - compiles to wasm32 (constraint enforced in CI)
                         #   - depends on: rustfft/realfft, wide (SIMD),
                         #     serde, serde_json, base64, bytemuck, arc-swap
                         #   - NO cpal, NO midir, NO nih-plug
  elixir-standalone/     # Binary. cpal output, midir input, argv via clap,
                         #   egui UI in the same process.
                         #   - links elixir-core
                         #   - desktop only (cfg(not(wasm32)))
                         #   - public release artifact built from this repo
  elixir-plugin/         # nih-plug cdylib. VST3 + CLAP + AU + standalone wrap.
                         #   - links elixir-core, embeds egui editor
                         #   - separate from contrapunk_plugin
                         #   - distributes Elixir as an independent plugin
                         #     released from this repo's CI
  elixir-headless/       # Binary. argv + render MIDI file → WAV via hound.
                         #   - links elixir-core, hound, clap
```

### Crates and modules touched (existing tree)

```
src/synth/               # REPLACED at A-Cut (kept as legacy until then)
src/chain/               # EXTENDED — new ElixirSynthBlock impls AudioBlock
src/plugin_host/         # EXTENDED for Track C — fill CLAP, add vst3, au
src-tauri/src/audio_clock.rs # Re-wired to spawn an ElixirSynthBlock instead
                             # of a Synth in the chain
src-tauri/src/commands/synth.rs # Adapted to drive ElixirSynthBlock params
plugin/src/lib.rs        # contrapunk_plugin: same swap inside the plugin
ui/src/lib/adapter/      # Surface-agnostic param wiring picks up new keys
.github/workflows/       # ADD elixir-standalone + elixir-plugin release flows
                         # next to existing macos-build.yml
```

### Why inside the workspace (decision locked)

- One `cargo check` covers everything.
- Existing CI (clippy, wasm build, tauri build, plugin build) covers Elixir for free.
- Release infrastructure (code-signing, notarization, `.github/workflows/macos-build.yml`) is reused — Elixir gets DMG + plugin bundle builds out of the same pipeline.
- `elixir-standalone` and `elixir-plugin` are their own crates with no Contrapunk dependencies — they can be extracted to a separate repo later if desired with zero refactoring (a `git filter-repo` away).

### Release artifacts (locked: public from this repo)

Each release tag produces, in parallel CI jobs:

- Contrapunk DMG (existing flow) — unchanged
- Contrapunk plugin bundle (VST3 + CLAP) — unchanged
- **Elixir standalone DMG** — `cargo build -p elixir-standalone --release` + sign + notarize
- **Elixir plugin bundle** — `cargo xtask bundle elixir-plugin --release` + sign + notarize
- **Elixir headless binary** — multi-arch, attached to the GitHub Release

Versioning: Elixir tracks its own SemVer (`elixir-v0.1.0`, …) independent of Contrapunk's `v1.x`. Tags are namespaced. The `release-patch` skill is extended to know both prefixes.

---

## 2. Cutover boundary — what gets replaced and how

### Current synth surface

| Concept | Today | After Elixir |
|---|---|---|
| Audio block | `crate::synth::Synth` | `elixir_core::Engine` wrapped by `ElixirSynthBlock: AudioBlock` |
| Event input | `Receiver<SynthEvent>` (`NoteOn` / `NoteOff` / `AllNotesOff`) | Same MPSC, mapped to `elixir_core::NoteEvent` |
| Params | `Arc<SynthParams>` with atomics (`set_waveform`, `set_cutoff_hz`, etc.) | `Arc<ElixirParams>` with the same getter/setter shape but typed `ParamId` keys |
| Render | `Synth::render(buf, channels)` called per cpal callback | `ElixirSynthBlock::process(buf, channels)` |

### Default preset for cutover

The current synth is essentially "sine + one-pole LP + ADSR". To make A-Cut a *zero perceptual change*, Elixir ships a built-in factory preset called `Contrapunk-Default.elxprst` that:

- Single wavetable oscillator using a 256-frame sine-to-saw morphable wavetable, frame index = `Waveform::Sine = 0`, equivalent to a pure sine
- One amp envelope mapped to Contrapunk's existing `attack_ms` / `decay_ms` / `sustain_ppt` / `release_ms`
- One digital-SVF lowpass at `cutoff_hz` (no resonance unless `resonance_ppt > 0`)
- Master gain at `master_gain_ppt`
- No modulators, no FX, no unison

This preset is *byte-for-byte* the same DSP behaviour as `src/synth/voice.rs` runs today, just routed through Elixir's graph. A/B render of a fixed MIDI file should differ only by floating-point noise (`< -90 dBFS RMS`).

### Feature flag for safe cutover

In `Cargo.toml`:

```toml
[features]
default = []
elixir-synth = ["dep:elixir-core"]  # opt-in until A-Cut+2 weeks
```

In `src/chain/` (or wherever the synth block is instantiated):

```rust
#[cfg(feature = "elixir-synth")]
pub fn make_default_synth(sr: u32, rx: Receiver<SynthEvent>, params: Arc<SynthParams>)
    -> Box<dyn AudioBlock>
{
    Box::new(ElixirSynthBlock::new_with_default_preset(sr, rx, params))
}

#[cfg(not(feature = "elixir-synth"))]
pub fn make_default_synth(sr: u32, rx: Receiver<SynthEvent>, params: Arc<SynthParams>)
    -> Box<dyn AudioBlock>
{
    Box::new(LegacySynth::from(Synth::new(sr, rx, params)))
}
```

Flag stays opt-in for two stabilisation weeks after A-Cut. Then the default flips. Two further weeks of no regression reports, then `src/synth/` is deleted.

### Cutover timing (locked: post-A6 full feature parity)

A-Cut happens **after A6 lands**, not after A5. Rationale: shipping a partial Elixir as the default would mean Contrapunk users lose features that the design doc promises (spectral morphs, unison, chorus, phaser, full filter set) on a temporary basis. The risk profile of "ship the new synth missing features" is worse than "ship the old synth for 4 more weeks while we finish A6". Track B isn't blocked — `elixir-plugin` ships earlier and gives users the new sound through their DAW; A-Cut is purely about Contrapunk's built-in synth.

---

## 3. Track A — Elixir replaces Contrapunk's synth

| Phase | Goal | Ship gate | Effort |
|---|---|---|---|
| **A0** | Workspace bootstrap. Add `crates/elixir-core` with empty lib, an `Engine` struct that compiles and exposes `prepare(sr, max_block)` + `process(&mut [f32], channels)` writing silence. Wire it as a feature-flagged `AudioBlock` in `src/chain/`. CI green on all 4 surfaces (CLI, Tauri, WASM, plugin) with the flag off. | `cargo check --workspace` clean; both branches of the `cfg` compile. | 3 d |
| **A1** | **Bare oscillator.** Voice handler + one wavetable oscillator (Catmull-Rom + fixed-point phase). Spectral mip-mapping for anti-aliasing. Single pre-baked sine wavetable. One DAHDSR envelope, hard-coded amp routing. | Plug headphones into `cargo run --features elixir-synth` and the CLI; play notes via MIDI input; hear a sine, no aliasing across the keyboard. A/B against `src/synth/Sine` shows < -90 dBFS RMS difference. | 2 wk |
| **A2** | **Polyphony + voice management.** 16-voice pool, SIMD-packed AggregateVoice (2 voices per `f32x8`). Voice stealing (Newest priority). Sustain pedal. Per-block sample-offset for sample-accurate note-on. | 16-note chord plays cleanly; voice-steal at 17th note is click-free. Sustain pedal works through the harmony engine's full output. | 1 wk |
| **A3** | **Modulation matrix v1.** SoA `ModRoutes`. Two LFOs (custom-waveform + random), six envelopes per voice. `arc-swap` for breakpoint-table hot-swap. UI→audio command queue (`rtrb`) for route add/remove. | Map LFO → amp via the matrix from a test rig (no UI yet); click-free knob changes; modulation-of-modulation works (LFO speed modded by envelope). | 2 wk |
| **A4** | **Filter.** Digital SVF + analog ladder + comb. TPT coefficient LUT. Per-sample audio-rate cutoff modulation. | Cutoff sweep tracks an LFO with no zipper; resonance self-oscillates on the ladder; comb tracks pitch. | 2 wk |
| **A5** | **FX bus (minimum viable).** 2× oversampling (upsampler + 3-pole halfband decimator). Reorderable 4-slot chain (vs the full 8 — reverb / delay / EQ / distortion only; chorus/flanger/phaser/compressor defer to A6). FDN-8 reverb (FDN-16 deferred). | Full reverb tail audible, ping-pong delay, drive at -1 dBFS without clipping. Reorder via test rig works without crackle. | 3 wk |
| **A6** | **Spectral oscillator features + FX completion.** All 12 spectral morphs (vocode, smear, harmonic-scale, phase-disperse, shepard, skew, etc.). 9 phase-distortion modes (FM, RM, sync, pulsewidth…). Unison with stack styles. Chorus / flanger / phaser / compressor effects. FDN-16 reverb. Filter models: diode, dirty, formant, phaser. | Wavetable feature parity with the design doc. Existing tests still pass. New unit tests for each morph (golden WAV per morph). | 4 wk |
| **A-Cut** | **CUTOVER (full feature parity).** Map every `SynthParams` setter to an Elixir `ParamId`. Ship `Contrapunk-Default.elxprst` factory preset embedded in the binary. Audio chain instantiates `ElixirSynthBlock` instead of `Synth` when `--features elixir-synth`. Run audio-pipeline tests against both. Track B's `elixir-plugin` is already shipping by this point, so DAW users have been on the new sound for weeks. | Existing `tests/audio_pipeline.rs` passes with `--features elixir-synth`. Manual A/B on a fixed MIDI file in Tauri shows no perceptual change. Plugin (VST3/CLAP) renders identically. CLI binary renders identically. | 1 wk |
| **A7** | **Default flip + cleanup.** After 2 weeks of A-Cut in opt-in mode, flip the feature flag default. After another 2 weeks of no regressions, delete `src/synth/`. Update `src-tauri/src/commands/synth.rs` to use Elixir's typed params (drop the old `SynthParams` shim). | `git rm -r src/synth/`; CI green; no `--features` toggle needed; release-notes line item for Contrapunk v1.4. | 1 wk |

**Track A total:** ~13 weeks of focused work. **A-Cut is at week ~12** — Elixir has full feature parity with the design doc before becoming the default sound. Track B (`elixir-plugin` inside a DAW) gives users access to the new engine well before A-Cut.

---

## 4. Track B — Elixir standalone product

Track B runs in parallel with Track A. It shares the `elixir-core` crate; each phase below is a *thin layer on top* of an A-phase deliverable. Public releases ship from **this repo** (locked decision §5).

| Phase | Goal | Depends on | Effort |
|---|---|---|---|
| **B0** | Skeleton `elixir-standalone` binary. `cpal` opens default output, `midir` opens default input. Plays silence. Argv via `clap`. Window opens as a separate process from Contrapunk's desktop app. | A0 | 2 d |
| **B1** | `elixir-standalone` produces sound — single voice, default preset. Computer-keyboard fallback (`a w s e d f t g …` = chromatic). | A1 | 3 d |
| **B2** | Polyphony works in standalone. Voice meter logged to stderr. | A2 | 1 d |
| **B3** | `elixir-plugin` skeleton (nih-plug VST3 + CLAP + AU + standalone wrapper). Loads in Bitwig / Logic / Ableton; one parameter (master gain) automatable. | A1 | 5 d |
| **B4** | `elixir-plugin` exposes full param set (envelope, filter, mod matrix amount). Audio-thread parameter smoothing via `nih-plug`'s `SmoothedParam`. Earliest point Contrapunk users can demo Elixir inside their DAW — well before A-Cut. | A2 + A3 + A4 | 1 wk |
| **B5** | Preset save / load wired in plugin + standalone. JSON format from the design doc; embedded base64 PCM for wavetables and samples. ArcSwap preset hot-swap. | A3 onwards (preset structs evolve through A's phases) | 1 wk |
| **B6** | **Standalone UI v1 (`egui`).** Mod-matrix view, oscillator panel, envelope panel, filter panel, FX chain panel. No wavetable editor yet — pick from factory bank only. Same `egui` widget set is embedded in `elixir-plugin` so the in-DAW UI and standalone UI are the same code. | A5 minimum (so users have FX to use) | 4 wk |
| **B7** | **Wavetable editor.** UI for editing wavetable frames in time-domain and frequency-domain. Spectral morph parameter live preview. | A6 | 2 wk |
| **B8** | `elixir-headless` binary. Render `--midi foo.mid --preset bar.elxprst --out baz.wav --duration 60s`. | A2 minimum | 3 d |
| **B9** | **Public v0.1.0 release of Elixir.** Code-signing (macOS Developer ID, Windows EV cert), notarization, installers (`.dmg`, `.exe`, `.deb`). Released from this repo's CI under the `elixir-v0.1.0` tag. Distinct from any Contrapunk version. | B6 + B7 | 1 wk |

**Track B total:** ~10 weeks of *additive* work; ~19 weeks if Track A doesn't run in parallel. Most of B is "build a UI shell around the engine A already shipped".

### UI choice (B6, locked: `egui`)

`egui` is chosen for these reasons:
- Heavy custom-paint surfaces (mod-matrix, wavetable editor) where browser DOM offers no advantage.
- Single widget set shared between `elixir-standalone` and `elixir-plugin`'s in-DAW window — no double UI maintenance.
- Elixir's standalone app launches as a **separate window/process** from Contrapunk. Locked decision §3 — by design, not a regression. This keeps the two products independent (you can quit Contrapunk and keep Elixir open, run Elixir inside a DAW with Contrapunk closed, etc.).

Tauri+Svelte is rejected for B6 specifically. (It remains the right call for Contrapunk's main app; the two surfaces serve different users.)

### Public release from this repo (locked decision §5)

Tag `elixir-v0.1.0` triggers a release workflow that:

1. `cargo test -p elixir-core` + `cargo test -p elixir-plugin` + `cargo test -p elixir-standalone`.
2. Build matrix: macOS arm64 + x86_64 universal, Windows x86_64, Linux x86_64.
3. Sign + notarize macOS artifacts using existing Contrapunk certs.
4. Bundle: `elixir-standalone.dmg`, `Elixir.vst3`, `Elixir.clap`, `Elixir.component` (AU), `elixir-headless` binary.
5. Attach all artifacts to a new GitHub Release on this repo with hand-written release notes (use the `release-patch` skill, extended to recognise the `elixir-` tag prefix).

This means Elixir's release cadence is decoupled from Contrapunk's. v1.4 of Contrapunk can ship before `elixir-v0.1.0`. They're independent products from one monorepo.

---

## 5. Track C — Multi-plugin hosting

Track C extends `src/plugin_host/` which is currently CLAP-only and v1-limited. The goal: load any number of CLAP / VST3 / AU plugins as audio blocks in the chain, with parameter automation, GUI embedding, and preset save/load. Once landed, Contrapunk users can route the harmony stream through an Elixir instance, then through Diva, then through Pro-Q4, then to output.

| Phase | Goal | Ship gate | Effort |
|---|---|---|---|
| **C0** | **Finish CLAP activation.** Fill in the existing `block.rs` / `controller.rs` stubs so a discovered CLAP plugin actually instantiates, activates, and processes audio through `ClapAudioBlock`. Use the chain's existing `PushBlock` queue. | Load Surge XT (free CLAP), play notes from harmony engine, hear it. No GUI yet; plugin opens its own OS window. | 2 wk |
| **C1** | **Plugin GUI embedding (CLAP).** Implement the CLAP `gui` extension flow: query embedded-size hint, create child window inside Tauri's main window (macOS NSView, Windows HWND, Linux X11 window). Fallback: detached floating window. | Surge XT GUI shows inside Contrapunk's desktop window on macOS; resize works. | 2 wk |
| **C2** | **Parameter automation + preset state (CLAP).** Map plugin params to Contrapunk's mod matrix (so an LFO can drive a hosted plugin's filter cutoff). Serialise plugin state into Contrapunk's session file via the CLAP `state` extension. | Set up Diva CLAP, automate cutoff from Contrapunk's macro 1, save session, reload — automation and state survive. | 2 wk |
| **C3** | **VST3 hosting.** New `src/plugin_host/vst3/` module. Use a Rust VST3 host crate (candidates: `vst3-sys` raw bindings or roll our own thin wrapper). Mirror the CLAP module's API surface — `discovery`, `host`, `block`, `controller`, `window`. The `Chain` doesn't change. | Load FabFilter Pro-Q4 VST3, route harmony through it, GUI embeds, state saves. | 3 wk |
| **C4** | **AU hosting (macOS only).** `src/plugin_host/au/`. Use `audio-unit` crate or hand-roll over `objc2`. AU is macOS-only by definition — wrap the module in `#[cfg(target_os = "macos")]`. Locked decision §4: AU is part of v1, not deferred. | Load Apple's stock AU instruments, hear them through Contrapunk. Skip on Windows/Linux builds. | 2 wk |

**Track C total:** ~9 weeks (was 8, +1 for AU promoted from optional to v1 scope). Self-contained; doesn't depend on Elixir; runs in parallel with Tracks A+B once the chain plumbing is solid.

### Multi-plugin chain UI

When ≥ 2 plugins are loaded, Contrapunk needs a per-slot UI strip: name, format badge (CLAP/VST3/AU), bypass, latency report, "open GUI", parameter expander, and "remove". This is ~1 week of UI work and folds into Contrapunk's existing Svelte UI (not Track B's egui — different product, different users).

### Plugin discovery and the user library

Standard OS paths per format:

- **CLAP:** `~/Library/Audio/Plug-Ins/CLAP/` (macOS), `%CommonProgramFiles%\CLAP\` (Windows), `~/.clap/` and `/usr/lib/clap/` (Linux)
- **VST3:** `~/Library/Audio/Plug-Ins/VST3/` (macOS), `%CommonProgramFiles%\VST3\` (Windows), `~/.vst3/` and `/usr/lib/vst3/` (Linux)
- **AU:** `~/Library/Audio/Plug-Ins/Components/` (macOS only)

Scan is async, off the audio thread, cached to `~/.config/contrapunk/plugins.json` with mtime invalidation. Existing `discovery.rs` already does this for CLAP — generalise the pattern.

---

## 6. Cross-track integration points

These are the moments where two tracks touch the same code and need coordination:

| Touchpoint | When | What needs care |
|---|---|---|
| `AudioBlock` trait stability | A0, C0 | Both Elixir's `ElixirSynthBlock` and the plugin-host `ClapBlock` / `Vst3Block` / `AuBlock` implement this trait. Don't break the trait signature mid-track. Lock the trait shape at A0 and only add (never change) methods after. |
| `Chain::PushBlock` ordering | A0, C0 | The user expects: harmony → synth (Elixir or hosted) → FX (Elixir's built-in OR hosted FX). The default chain assembly needs an `insert_at(idx)` if it doesn't already, so users can place plugins between the synth and built-in FX. |
| Modulation routes targeting hosted plugin params | A3, C2 | Contrapunk's mod matrix needs to know about parameters owned by hosted plugins. Define a `ParamId` enum with `Internal(...) \| Hosted { slot, plugin_param_id }` variants. |
| Preset / session file format | A5/B5, C2/C3 | Contrapunk's session preset must store per-slot plugin state. JSON top-level key `chain: [{ kind: "elixir", preset: {...} }, { kind: "clap", path: ..., state: <base64> }, ...]`. Don't fork formats. |
| `egui` widget set vs Contrapunk's Svelte UI | B6, C1 | Elixir's UI (egui, separate window) and Contrapunk's UI (Svelte, in Tauri webview) are independent. No widget sharing. The boundary is the audio chain: Contrapunk drives Elixir via MIDI events, Elixir owns its own param state. |
| Release pipeline | B9 | `elixir-` tag prefix triggers Elixir build matrix; `v` tag prefix continues to trigger Contrapunk builds. Both share signing identities and notarization plumbing. |
| WASM compilation | Continuous | `elixir-core` must compile to wasm32. Plugin-hosting code (Track C) is `cfg(not(target_arch = "wasm32"))`. CI runs `cargo check --target wasm32-unknown-unknown -p elixir-core` on every PR. |

---

## 7. Surface matrix — what ships where

| Crate / module | CLI bin | Tauri desktop | WASM browser | contrapunk_plugin (VST3/CLAP) | elixir-standalone | elixir-plugin |
|---|---|---|---|---|---|---|
| `elixir-core` | yes (post-A1) | yes (post-A1) | yes (post-A1, smoke-tested in CI; not wired to audio output until/unless Contrapunk's WASM gets a Rust audio path) | yes (post-A1) | yes | yes |
| `src/synth/` (legacy) | until A7 | until A7 | n/a | until A7 | no | no |
| `src/plugin_host/clap/` | yes | yes | no | yes | no | no |
| `src/plugin_host/vst3/` (Track C) | yes | yes | no | yes | no | no |
| `src/plugin_host/au/` (Track C) | macOS only | macOS only | no | macOS only | no | no |
| `egui` UI widgets (`elixir-ui` if extracted) | no | no | no | no | yes | yes |

**WASM caveat:** Contrapunk's WASM surface doesn't render Rust audio today — the browser drives Web Audio from JS. Compiling `elixir-core` to wasm32 is enforced for the *standalone* Elixir web product (if Track B ever ships one), not for replacing Contrapunk's WASM synth (which doesn't exist).

---

## 8. Testing strategy

### Per-phase

1. **Unit tests** in `elixir-core` for each DSP component. Deterministic input → expected output buffer (small tolerance for FP noise). Golden WAVs for spectral-morph variants, FX algorithms.
2. **Property tests** (proptest) for invariants: no NaN, no allocation on the audio thread, voice-handler maintains `len(free) + len(active) + len(released) == capacity`.
3. **`assert_no_alloc`** wrapping every `process()` in debug builds. Any allocation crashes the test.

### Cutover (A-Cut)

1. A/B test rig in `tests/cutover_parity.rs`: render a fixed MIDI sequence through both the legacy `Synth` and `ElixirSynthBlock` with the `Contrapunk-Default` preset; assert RMS difference `< -90 dBFS`.
2. Smoke test on each surface: CLI render, Tauri "play 4-bar progression and metronome" flow, plugin loaded in `pluginval`.
3. Existing `tests/audio_pipeline.rs` runs against both feature configurations.

### Plugin hosting (Track C)

1. `pluginval` (the standard plugin-host fuzz / conformance tool) against Contrapunk acting as a host. CI runs the smoke suite.
2. Smoke load: a known-good CLAP instrument (Surge XT) and a known-good VST3 effect (TDR Nova free) on every macOS CI run.

### Continuous

1. Harmony-engine unit tests (existing 249) keep passing — Tracks A/B/C must not regress harmony correctness.
2. WASM build (`cd ui && npm run build:wasm`) green continuously — proves `elixir-core` is wasm-clean.

---

## 9. Risk register

| Risk | Likelihood | Mitigation |
|---|---|---|
| SIMD code uses nightly `std::simd` and stable users get blocked | Medium | Default to `wide` crate (stable). Gate `std::simd` behind `cfg(feature = "nightly-simd")`. CI runs both. |
| `nih-plug` upstream churn breaks `elixir-plugin` | Low (we use the forked `contrapunk-audio/nih-plug` already) | Pin both Contrapunk's and Elixir's nih-plug to the same fork SHA. |
| `clack-host` API breaks (it's git-only, not on crates.io) | Medium | Already pinned by `rev`. Bump deliberately, not opportunistically. |
| VST3 SDK licensing complications for Track C | Medium | VST3 hosting under GPL3 is permitted but distribution gets tricky. Plan: Track C's VST3 module is GPL3'd, contrapunk's own license stays as-is. Hold a 1-day legal review at the start of C3. |
| AU hosting (C4) needs deep ObjC bridging on macOS | Medium | `objc2` is already a contrapunk dep. Start with stock AU instruments; defer the more exotic AU v2/v3 differentiation until users ask. |
| Audio-thread allocation regression on Elixir | High | `assert_no_alloc` wrapping in debug builds; CI runs the full Elixir audio path in debug for one block per commit. |
| Replacing the synth changes Contrapunk's tone unintentionally | Low (full feature parity at A-Cut now) | The `Contrapunk-Default` preset is byte-for-byte parity. A-Cut is gated on the < -90 dBFS RMS A/B test. Ship behind a feature flag for 2 weeks before flipping the default. |
| Plugin GUI embedding (C1) is fragile per OS | High | Detached-window fallback ships first; embedding is best-effort. Document the fallback. |
| Track B's UI scope balloons | Medium (egui keeps it tight) | B6/B7 are explicitly the last things to land. If they slip, A and C still ship; Elixir's first public face can be "standalone with no UI, only headless CLI rendering" if needed. Or skip B6 entirely and ship Elixir as a plugin only — DAW provides UI free. |
| Releasing Elixir from contrapunk repo creates tag confusion | Low | Namespace tags: `v1.x` = Contrapunk, `elixir-v0.x` = Elixir. `release-patch` skill is extended to recognise both prefixes. CI workflow YAML pattern-matches the prefix and picks the right build matrix. |
| Codesigning Elixir requires separate Developer ID Application bundle | Medium | macOS notarization requires `bundle_id` per artifact. Reserve `com.contrapunk.elixir` and `com.contrapunk.elixir.plugin` early. |

---

## 10. Locked decisions (was: open questions)

All six questions are now answered. Recorded here for the audit trail.

1. **Workspace location** → *Inside the contrapunk workspace.* New Elixir crates ship as workspace members. Sibling-repo extraction remains a future option but is not planned.
2. **Cutover ambition** → *Full feature parity at A-Cut (post-A6, ~week 12).* Contrapunk's built-in synth flips to Elixir only after the full design-doc feature set lands. Track B's `elixir-plugin` gives DAW users early access during the build-out.
3. **Track B UI** → *`egui`.* Same widget set in standalone and plugin in-DAW window. Standalone app opens as a separate process/window from Contrapunk — by design, not a regression.
4. **Track C scope for v1** → *All three plugin formats (CLAP + VST3 + AU).* AU is promoted from "optional follow-up" to mandatory v1 scope. Track C duration revised 8 wk → 9 wk.
5. **`elixir-standalone` distribution** → *Public, released from this repo.* Same CI / signing / notarization plumbing as Contrapunk. Tags namespaced (`elixir-v0.x` vs `v1.x`). Bundle IDs reserved at `com.contrapunk.elixir{,.plugin}`.
6. **GSD milestone integration** → *Yes.* This plan and the design doc are about to be ingested into `.planning/` via `/gsd-ingest-docs`, becoming a real milestone with discuss/plan/execute phases.

---

## 11. Recommended start sequence (next 2 weeks)

Concrete first-2-weeks moves so you can start immediately:

**Week 1**
1. (1 d) Run `/gsd-ingest-docs ELIXIR-DESIGN.md ELIXIR-PLAN.md` so the planning artifacts land in `.planning/` and the GSD workflow picks them up.
2. (1 d) A0: add `crates/elixir-core` with empty `Engine`, wire it as a feature-flagged `AudioBlock` in `src/chain/`. Confirm CI green on all 4 surfaces.
3. (2 d) B0: skeleton `elixir-standalone` binary that opens cpal + midir and plays silence; egui window with a single "Hello Elixir" label.
4. (1 d) Reserve `com.contrapunk.elixir` and `com.contrapunk.elixir.plugin` bundle IDs in App Store Connect; extend `release-patch` skill to recognise the `elixir-` tag prefix.

**Week 2**
1. (5 d) A1 starts: voice handler + wavetable oscillator + DAHDSR. Daily commits; daily audible-progress demo against the previous day's build.
2. (in parallel, 2 d) C0 prep: read `src/plugin_host/clap/block.rs` and `controller.rs` stubs end-to-end; write the design note for finishing CLAP activation; confirm the 2-week budget for C0.

End of week 2 you have:
- Elixir crate compiling on every surface and ingested into `.planning/` as a GSD milestone
- Standalone binary opening audio + window (no notes yet)
- A clear go/no-go on C0's 2-week budget
- Bundle IDs reserved so signing isn't a last-minute blocker

---

*End of plan.*
