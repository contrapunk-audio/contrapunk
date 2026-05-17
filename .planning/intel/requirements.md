# Requirements Intel

Synthesized from ingested PRDs and SPEC-style ship gates. Each entry has an ID, source, description, and acceptance criteria. IDs use `REQ-elixir-{slug}` convention.

The Elixir docs are SPECs, not PRDs — but their phase ship gates function as user-facing acceptance criteria, so they're surfaced here for the roadmapper to consume. Where a ship gate is technical (e.g., "trait is locked at A0"), it lives in `constraints.md` instead.

---

## Track A — Elixir replaces Contrapunk's built-in synth

### REQ-elixir-a0-workspace-bootstrap

- **Source:** `ELIXIR-PLAN.md` §3 (Phase A0)
- **Description:** `crates/elixir-core` exists with an empty `Engine` struct exposing `prepare(sr, max_block)` and `process(&mut [f32], channels)` that writes silence. Wire it as a feature-flagged `AudioBlock` in `src/chain/`.
- **Acceptance:**
  - `cargo check --workspace` is clean
  - Both branches of `#[cfg(feature = "elixir-synth")]` compile
  - CI green on all 4 surfaces (CLI, Tauri, WASM, plugin) with the flag off
- **Effort:** 3 days

### REQ-elixir-a1-bare-oscillator

- **Source:** `ELIXIR-PLAN.md` §3 (Phase A1)
- **Description:** Voice handler + one wavetable oscillator (Catmull-Rom + fixed-point phase). Spectral mip-mapping for anti-aliasing. Single pre-baked sine wavetable. One DAHDSR envelope, hard-coded amp routing.
- **Acceptance:**
  - User can plug headphones into `cargo run --features elixir-synth` and the CLI; play notes via MIDI input; hear a sine
  - No aliasing across the keyboard range
  - A/B against `src/synth/Sine` shows < -90 dBFS RMS difference
- **Effort:** 2 weeks

### REQ-elixir-a2-polyphony-voice-management

- **Source:** `ELIXIR-PLAN.md` §3 (Phase A2)
- **Description:** 16-voice pool. SIMD-packed AggregateVoice (2 voices per `f32x8`). Voice stealing (Newest priority). Sustain pedal. Per-block sample-offset for sample-accurate note-on.
- **Acceptance:**
  - 16-note chord plays cleanly
  - Voice-steal at 17th note is click-free
  - Sustain pedal works through the harmony engine's full output
- **Effort:** 1 week

### REQ-elixir-a3-modulation-matrix-v1

- **Source:** `ELIXIR-PLAN.md` §3 (Phase A3)
- **Description:** SoA `ModRoutes`. Two LFOs (custom-waveform + random), six envelopes per voice. `arc-swap` for breakpoint-table hot-swap. UI→audio command queue (`rtrb`) for route add/remove.
- **Acceptance:**
  - Map LFO → amp via the matrix from a test rig (no UI yet)
  - Click-free knob changes
  - Modulation-of-modulation works (LFO speed modded by envelope)
- **Effort:** 2 weeks

### REQ-elixir-a4-filter

- **Source:** `ELIXIR-PLAN.md` §3 (Phase A4)
- **Description:** Digital SVF + analog ladder + comb filters. TPT coefficient LUT. Per-sample audio-rate cutoff modulation.
- **Acceptance:**
  - Cutoff sweep tracks an LFO with no zipper
  - Resonance self-oscillates on the ladder
  - Comb tracks pitch
- **Effort:** 2 weeks

### REQ-elixir-a5-fx-bus-mvp

- **Source:** `ELIXIR-PLAN.md` §3 (Phase A5)
- **Description:** 2× oversampling (upsampler + 3-pole halfband decimator). Reorderable 4-slot chain (vs the full 8 — reverb / delay / EQ / distortion only; chorus/flanger/phaser/compressor defer to A6). FDN-8 reverb (FDN-16 deferred).
- **Acceptance:**
  - Full reverb tail audible
  - Ping-pong delay
  - Drive at -1 dBFS without clipping
  - Reorder via test rig works without crackle
- **Effort:** 3 weeks

### REQ-elixir-a6-spectral-and-fx-completion

- **Source:** `ELIXIR-PLAN.md` §3 (Phase A6)
- **Description:** All 12 spectral morphs (vocode, smear, harmonic-scale, phase-disperse, shepard, skew, etc.). 9 phase-distortion modes (FM, RM, sync, pulsewidth…). Unison with stack styles. Chorus / flanger / phaser / compressor effects. FDN-16 reverb. Filter models: diode, dirty, formant, phaser.
- **Acceptance:**
  - Wavetable feature parity with the design doc
  - Existing tests still pass
  - New unit tests for each morph (golden WAV per morph)
- **Effort:** 4 weeks

### REQ-elixir-a-cut-full-parity-cutover

- **Source:** `ELIXIR-PLAN.md` §3 (Phase A-Cut), §2
- **Description:** Map every `SynthParams` setter to an Elixir `ParamId`. Ship `Contrapunk-Default.elxprst` factory preset embedded in the binary. Audio chain instantiates `ElixirSynthBlock` instead of `Synth` when `--features elixir-synth`. Run audio-pipeline tests against both. Track B's `elixir-plugin` is shipping by this point.
- **Acceptance:**
  - Existing `tests/audio_pipeline.rs` passes with `--features elixir-synth`
  - Manual A/B on a fixed MIDI file in Tauri shows no perceptual change
  - Plugin (VST3/CLAP) renders identically
  - CLI binary renders identically
- **Effort:** 1 week

### REQ-elixir-a7-default-flip-and-cleanup

- **Source:** `ELIXIR-PLAN.md` §3 (Phase A7), §2
- **Description:** After 2 weeks of A-Cut in opt-in mode, flip the feature flag default. After another 2 weeks of no regressions, delete `src/synth/`. Update `src-tauri/src/commands/synth.rs` to use Elixir's typed params (drop the old `SynthParams` shim).
- **Acceptance:**
  - `git rm -r src/synth/`; CI green
  - No `--features` toggle needed
  - Release-notes line item for Contrapunk v1.4
- **Effort:** 1 week

## Track B — Elixir as standalone product

### REQ-elixir-b0-standalone-skeleton

- **Source:** `ELIXIR-PLAN.md` §4 (Phase B0)
- **Description:** Skeleton `elixir-standalone` binary. `cpal` opens default output, `midir` opens default input. Plays silence. Argv via `clap`. Window opens as a separate process from Contrapunk's desktop app.
- **Acceptance:** Binary runs; cpal opens output; midir opens input; egui window opens with "Hello Elixir" label.
- **Effort:** 2 days

### REQ-elixir-b1-standalone-first-sound

- **Source:** `ELIXIR-PLAN.md` §4 (Phase B1)
- **Description:** `elixir-standalone` produces sound — single voice, default preset. Computer-keyboard fallback (`a w s e d f t g …` = chromatic).
- **Acceptance:** User can press a computer keyboard key and hear a note from the standalone binary.
- **Effort:** 3 days

### REQ-elixir-b2-standalone-polyphony

- **Source:** `ELIXIR-PLAN.md` §4 (Phase B2)
- **Description:** Polyphony works in standalone. Voice meter logged to stderr.
- **Acceptance:** 16-note chord plays cleanly in standalone; voice meter prints.
- **Effort:** 1 day

### REQ-elixir-b3-plugin-skeleton

- **Source:** `ELIXIR-PLAN.md` §4 (Phase B3)
- **Description:** `elixir-plugin` skeleton (nih-plug VST3 + CLAP + AU + standalone wrapper). One parameter (master gain) automatable.
- **Acceptance:** Plugin loads in Bitwig / Logic / Ableton; master gain is automatable from each host.
- **Effort:** 5 days

### REQ-elixir-b4-plugin-full-params

- **Source:** `ELIXIR-PLAN.md` §4 (Phase B4)
- **Description:** `elixir-plugin` exposes full param set (envelope, filter, mod matrix amount). Audio-thread parameter smoothing via nih-plug's `SmoothedParam`. Earliest point Contrapunk users can demo Elixir inside their DAW.
- **Acceptance:** All params automatable; smoothing is click-free.
- **Effort:** 1 week

### REQ-elixir-b5-preset-save-load

- **Source:** `ELIXIR-PLAN.md` §4 (Phase B5)
- **Description:** Preset save / load wired in plugin + standalone. JSON format from the design doc; embedded base64 PCM for wavetables and samples. ArcSwap preset hot-swap.
- **Acceptance:** User can save preset in plugin, load it in standalone (and vice versa). Hot-swap is click-free.
- **Effort:** 1 week

### REQ-elixir-b6-standalone-ui-v1

- **Source:** `ELIXIR-PLAN.md` §4 (Phase B6)
- **Description:** Standalone UI v1 (`egui`). Mod-matrix view, oscillator panel, envelope panel, filter panel, FX chain panel. No wavetable editor yet — pick from factory bank only. Same `egui` widget set is embedded in `elixir-plugin`.
- **Acceptance:** All five panels render and function in both standalone and in-DAW window. Factory bank loadable from UI.
- **Effort:** 4 weeks

### REQ-elixir-b7-wavetable-editor

- **Source:** `ELIXIR-PLAN.md` §4 (Phase B7)
- **Description:** UI for editing wavetable frames in time-domain and frequency-domain. Spectral morph parameter live preview.
- **Acceptance:** User can drag wavetable points in both time and frequency domains; morph live-previews.
- **Effort:** 2 weeks

### REQ-elixir-b8-headless-renderer

- **Source:** `ELIXIR-PLAN.md` §4 (Phase B8)
- **Description:** `elixir-headless` binary. Render `--midi foo.mid --preset bar.elxprst --out baz.wav --duration 60s`.
- **Acceptance:** CLI rendering produces deterministic WAV from MIDI + preset.
- **Effort:** 3 days

### REQ-elixir-b9-public-v0-1-0-release

- **Source:** `ELIXIR-PLAN.md` §4 (Phase B9)
- **Description:** Public v0.1.0 release of Elixir. Code-signing (macOS Developer ID, Windows EV cert), notarization, installers (`.dmg`, `.exe`, `.deb`). Released from this repo's CI under the `elixir-v0.1.0` tag. Distinct from any Contrapunk version.
- **Acceptance:**
  - `cargo test -p elixir-core` + `cargo test -p elixir-plugin` + `cargo test -p elixir-standalone` all pass
  - Build matrix: macOS arm64 + x86_64 universal, Windows x86_64, Linux x86_64
  - macOS artifacts signed + notarized using existing Contrapunk certs
  - GitHub Release attaches: `elixir-standalone.dmg`, `Elixir.vst3`, `Elixir.clap`, `Elixir.component` (AU), `elixir-headless` binary
- **Effort:** 1 week

## Track C — Multi-plugin hosting in Contrapunk

### REQ-elixir-c0-clap-activation

- **Source:** `ELIXIR-PLAN.md` §5 (Phase C0)
- **Description:** Fill in the existing `block.rs` / `controller.rs` stubs so a discovered CLAP plugin actually instantiates, activates, and processes audio through `ClapAudioBlock`. Use the chain's existing `PushBlock` queue.
- **Acceptance:** Load Surge XT (free CLAP), play notes from harmony engine, hear it. No GUI yet; plugin opens its own OS window.
- **Effort:** 2 weeks

### REQ-elixir-c1-plugin-gui-embedding

- **Source:** `ELIXIR-PLAN.md` §5 (Phase C1)
- **Description:** Implement the CLAP `gui` extension flow: query embedded-size hint, create child window inside Tauri's main window (macOS NSView, Windows HWND, Linux X11 window). Fallback: detached floating window.
- **Acceptance:** Surge XT GUI shows inside Contrapunk's desktop window on macOS; resize works.
- **Effort:** 2 weeks

### REQ-elixir-c2-clap-param-automation-state

- **Source:** `ELIXIR-PLAN.md` §5 (Phase C2)
- **Description:** Map plugin params to Contrapunk's mod matrix (so an LFO can drive a hosted plugin's filter cutoff). Serialise plugin state into Contrapunk's session file via the CLAP `state` extension.
- **Acceptance:** Set up Diva CLAP, automate cutoff from Contrapunk's macro 1, save session, reload — automation and state survive.
- **Effort:** 2 weeks

### REQ-elixir-c3-vst3-hosting

- **Source:** `ELIXIR-PLAN.md` §5 (Phase C3)
- **Description:** New `src/plugin_host/vst3/` module. Use a Rust VST3 host crate. Mirror the CLAP module's API surface (`discovery`, `host`, `block`, `controller`, `window`).
- **Acceptance:** Load FabFilter Pro-Q4 VST3, route harmony through it, GUI embeds, state saves.
- **Effort:** 3 weeks

### REQ-elixir-c4-au-hosting

- **Source:** `ELIXIR-PLAN.md` §5 (Phase C4)
- **Description:** `src/plugin_host/au/`. Use `audio-unit` crate or hand-roll over `objc2`. AU is macOS-only — wrap module in `#[cfg(target_os = "macos")]`.
- **Acceptance:** Load Apple's stock AU instruments, hear them through Contrapunk. Skip on Windows/Linux builds.
- **Effort:** 2 weeks

### REQ-elixir-c-ui-multi-plugin-strip

- **Source:** `ELIXIR-PLAN.md` §5 multi-plugin chain UI block
- **Description:** When ≥ 2 plugins are loaded, Contrapunk needs a per-slot UI strip: name, format badge (CLAP/VST3/AU), bypass, latency report, "open GUI", parameter expander, "remove". Folds into Contrapunk's existing Svelte UI (NOT Track B's egui — different product, different users).
- **Acceptance:** User can chain two plugins and see/manage both via Contrapunk's Svelte UI.
- **Effort:** ~1 week

### REQ-elixir-c-plugin-discovery

- **Source:** `ELIXIR-PLAN.md` §5 discovery block
- **Description:** Standard OS paths per format (CLAP: `~/Library/Audio/Plug-Ins/CLAP/` etc.; VST3 / AU paths analogous). Scan is async, off the audio thread, cached to `~/.config/contrapunk/plugins.json` with mtime invalidation. Existing `discovery.rs` already does this for CLAP — generalize the pattern.
- **Acceptance:** All three formats discovered; cache survives restart; new installs are picked up.

## Cross-cutting Elixir requirements

### REQ-elixir-wasm-core-compiles

- **Source:** `ELIXIR-PLAN.md` §6, §7
- **Description:** `elixir-core` MUST compile to wasm32. Plugin-hosting code stays `cfg(not(target_arch = "wasm32"))`. CI runs `cargo check --target wasm32-unknown-unknown -p elixir-core` on every PR.
- **Acceptance:** WASM CI green on every commit.

### REQ-elixir-release-pipeline-extension

- **Source:** `ELIXIR-PLAN.md` §1, §4 public release block
- **Description:** `.github/workflows/` adds elixir-standalone + elixir-plugin release flows next to existing `macos-build.yml`. `release-patch` skill is extended to recognize the `elixir-` tag prefix; CI workflow YAML pattern-matches the prefix and picks the right build matrix.
- **Acceptance:** Tagging `elixir-v0.1.0` produces signed, notarized macOS + Windows + Linux artifacts attached to a GitHub Release.

### REQ-elixir-bundle-ids-reserved

- **Source:** `ELIXIR-PLAN.md` §11 week 1 step 4
- **Description:** Reserve `com.contrapunk.elixir` and `com.contrapunk.elixir.plugin` bundle IDs in App Store Connect early.
- **Acceptance:** Signing isn't a last-minute blocker.

---

*Synthesized: 2026-05-18 from `/gsd-ingest-docs` run on ELIXIR-DESIGN.md + ELIXIR-PLAN.md.*
