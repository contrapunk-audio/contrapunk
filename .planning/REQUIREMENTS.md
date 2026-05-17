# Requirements: Contrapunk Rust

**Defined:** 2026-01-28
**Core Value:** Real-time harmony generation with minimal latency

## v1 Requirements

### MIDI I/O

- [x] **MIDI-01**: User can select MIDI input device from available ports
- [x] **MIDI-02**: User can select 2-8 MIDI output ports for harmony voices
- [x] **MIDI-03**: Original note passes through to first output port
- [x] **MIDI-04**: Harmony notes route to additional output ports

### Configuration

- [x] **CONF-01**: User can select musical key (C through B)
- [x] **CONF-02**: User can select harmony mode (1-7)
- [x] **CONF-03**: User can change key/mode during playback without stopping

### Harmony Modes

- [x] **HARM-01**: Mode 1 - Forward MIDI as-is (pass-through)
- [x] **HARM-02**: Mode 2 - Diatonic thirds above input note
- [x] **HARM-03**: Mode 3 - Diatonic fourths above input note
- [x] **HARM-04**: Mode 4 - Random diatonic interval below input
- [x] **HARM-05**: Mode 5 - Random diatonic below (excluding seconds)
- [x] **HARM-06**: Mode 6 - Contrary motion (harmony moves opposite to melody)
- [x] **HARM-07**: Mode 7 - Strict counterpoint (traditional voice leading rules)

### GUI

- [x] **GUI-01**: Native window renders with egui/eframe
- [x] **GUI-02**: Display active notes and current configuration
- [x] **GUI-03**: Controls for device selection, key selection, mode selection
- [x] **GUI-04**: Virtual piano keyboard showing input and harmony notes
- [x] **GUI-05**: Chord detection displaying what chord the combined notes form

### Distribution

- [x] **DIST-01**: Compiles to single binary with no runtime dependencies

### Octave Variations

- [x] **OCT-01**: Octave Spread - each harmony voice in progressively different octaves
- [x] **OCT-02**: Bass/Treble Split - harmonies below melody go low, above go high
- [x] **OCT-03**: Mirror Octaves - harmonies duplicate across multiple octaves simultaneously

### Humanization

- [x] **HUM-01**: Timing jitter - random delays (5-30ms) on harmony note onsets
- [x] **HUM-02**: Velocity variation - randomize note velocity within ±10-20 range
- [x] **HUM-03**: Note duration variation - slight sustain changes on harmony notes
- [x] **HUM-04**: Swing/groove - shift off-beat notes for rhythmic feel
- [x] **HUM-05**: Internal beat clock with adjustable BPM and time signature, tracking beat position
- [x] **HUM-06**: Optional audible metronome click on dedicated MIDI channel (GM percussion)

## v2 Requirements

### Extended Features

- **EXT-01**: MIDI file input for offline processing
- **EXT-02**: Preset save/load for configurations
- **EXT-03**: Audio-to-MIDI conversion (pitch detection)
- **EXT-04**: Algorithmic melody generation

## Elixir Milestone (v1.5 + elixir-v0.1.0)

**Source:** [ELIXIR-DESIGN.md](../ELIXIR-DESIGN.md) + [ELIXIR-PLAN.md](../ELIXIR-PLAN.md)
**Ingested:** 2026-05-18 via `/gsd-ingest-docs`
**Tracks:** A (replace Contrapunk's built-in synth), B (standalone Elixir product), C (multi-plugin hosting in Contrapunk)
**Status:** Queued behind v1.3.0 release. Three parallel-track work streams sharing the `Chain` / `AudioBlock` substrate. Calendar window 24-28 weeks with concurrent track execution.

### Track A — Elixir replaces Contrapunk's built-in synth

- [ ] **REQ-elixir-a0-workspace-bootstrap**: `crates/elixir-core` exists with empty `Engine` exposing `prepare(sr, max_block)` and `process(&mut [f32], channels)` writing silence; wired as feature-flagged `AudioBlock` in `src/chain/`. (source: ELIXIR-PLAN.md §3 Phase A0)
- [ ] **REQ-elixir-a1-bare-oscillator**: Voice handler + one wavetable oscillator (Catmull-Rom + fixed-point phase) with spectral mip-mapping. Single pre-baked sine wavetable. One DAHDSR envelope, hard-coded amp routing. A/B against `src/synth/Sine` < -90 dBFS RMS. (source: ELIXIR-PLAN.md §3 Phase A1)
- [ ] **REQ-elixir-a2-polyphony-voice-management**: 16-voice pool. SIMD-packed AggregateVoice (2 voices per `f32x8`). Voice stealing (Newest priority). Sustain pedal. Per-block sample-offset for sample-accurate note-on. (source: ELIXIR-PLAN.md §3 Phase A2)
- [ ] **REQ-elixir-a3-modulation-matrix-v1**: SoA `ModRoutes`. Two LFOs (custom-waveform + random), six envelopes per voice. `arc-swap` for breakpoint-table hot-swap. UI→audio command queue (`rtrb`) for route add/remove. (source: ELIXIR-PLAN.md §3 Phase A3)
- [ ] **REQ-elixir-a4-filter**: Digital SVF + analog ladder + comb filters. TPT coefficient LUT. Per-sample audio-rate cutoff modulation. (source: ELIXIR-PLAN.md §3 Phase A4)
- [ ] **REQ-elixir-a5-fx-bus-mvp**: 2× oversampling. Reorderable 4-slot chain (reverb / delay / EQ / distortion). FDN-8 reverb. Ping-pong delay. Drive at -1 dBFS without clipping. (source: ELIXIR-PLAN.md §3 Phase A5)
- [ ] **REQ-elixir-a6-spectral-and-fx-completion**: All 12 spectral morphs (vocode, smear, harmonic-scale, phase-disperse, shepard, skew, etc.). 9 phase-distortion modes. Unison with stack styles. Chorus / flanger / phaser / compressor. FDN-16 reverb. Filter models: diode, dirty, formant, phaser. (source: ELIXIR-PLAN.md §3 Phase A6)
- [ ] **REQ-elixir-a-cut-full-parity-cutover**: Map every `SynthParams` setter to Elixir `ParamId`. Ship `Contrapunk-Default.elxprst` factory preset embedded in binary. Audio chain instantiates `ElixirSynthBlock` instead of `Synth` when `--features elixir-synth`. (source: ELIXIR-PLAN.md §3 Phase A-Cut, §2)
- [ ] **REQ-elixir-a7-default-flip-and-cleanup**: After 2 weeks of A-Cut in opt-in mode, flip feature flag default. After another 2 weeks of no regressions, delete `src/synth/`. (source: ELIXIR-PLAN.md §3 Phase A7, §2)

### Track B — Elixir as standalone product

- [ ] **REQ-elixir-b0-standalone-skeleton**: Skeleton `elixir-standalone` binary. `cpal` opens default output, `midir` opens default input. Plays silence. Argv via `clap`. egui window opens with "Hello Elixir" label. (source: ELIXIR-PLAN.md §4 Phase B0)
- [ ] **REQ-elixir-b1-standalone-first-sound**: `elixir-standalone` produces sound — single voice, default preset. Computer-keyboard fallback (`a w s e d f t g …` = chromatic). (source: ELIXIR-PLAN.md §4 Phase B1)
- [ ] **REQ-elixir-b2-standalone-polyphony**: Polyphony works in standalone. 16-note chord plays cleanly. Voice meter logged to stderr. (source: ELIXIR-PLAN.md §4 Phase B2)
- [ ] **REQ-elixir-b3-plugin-skeleton**: `elixir-plugin` skeleton (nih-plug VST3 + CLAP + AU + standalone wrapper). One parameter (master gain) automatable. Loads in Bitwig / Logic / Ableton. (source: ELIXIR-PLAN.md §4 Phase B3)
- [ ] **REQ-elixir-b4-plugin-full-params**: `elixir-plugin` exposes full param set (envelope, filter, mod matrix amount). Audio-thread parameter smoothing via nih-plug's `SmoothedParam`. Earliest point Contrapunk users can demo Elixir inside their DAW. (source: ELIXIR-PLAN.md §4 Phase B4)
- [ ] **REQ-elixir-b5-preset-save-load**: Preset save / load wired in plugin + standalone. JSON format from design doc; embedded base64 PCM for wavetables and samples. ArcSwap preset hot-swap. Click-free. (source: ELIXIR-PLAN.md §4 Phase B5)
- [ ] **REQ-elixir-b6-standalone-ui-v1**: Standalone UI v1 (`egui`). Mod-matrix view, oscillator panel, envelope panel, filter panel, FX chain panel. No wavetable editor yet — pick from factory bank only. Same `egui` widget set embedded in `elixir-plugin`. (source: ELIXIR-PLAN.md §4 Phase B6)
- [ ] **REQ-elixir-b7-wavetable-editor**: UI for editing wavetable frames in time-domain and frequency-domain. Spectral morph parameter live preview. (source: ELIXIR-PLAN.md §4 Phase B7)
- [ ] **REQ-elixir-b8-headless-renderer**: `elixir-headless` binary. Render `--midi foo.mid --preset bar.elxprst --out baz.wav --duration 60s`. Deterministic WAV output. (source: ELIXIR-PLAN.md §4 Phase B8)
- [ ] **REQ-elixir-b9-public-v0-1-0-release**: Public v0.1.0 release of Elixir. Code-signing (macOS Developer ID, Windows EV cert), notarization, installers (`.dmg`, `.exe`, `.deb`). Released from this repo's CI under `elixir-v0.1.0` tag. (source: ELIXIR-PLAN.md §4 Phase B9)

### Track C — Multi-plugin hosting in Contrapunk

- [ ] **REQ-elixir-c0-clap-activation**: Fill in existing `block.rs` / `controller.rs` stubs so a discovered CLAP plugin actually instantiates, activates, and processes audio through `ClapAudioBlock`. Use chain's existing `PushBlock` queue. Load Surge XT, play notes from harmony engine, hear it. (source: ELIXIR-PLAN.md §5 Phase C0)
- [ ] **REQ-elixir-c1-plugin-gui-embedding**: Implement CLAP `gui` extension flow: query embedded-size hint, create child window inside Tauri's main window (macOS NSView, Windows HWND, Linux X11). Fallback: detached floating window. Surge XT GUI shows inside Contrapunk; resize works. (source: ELIXIR-PLAN.md §5 Phase C1)
- [ ] **REQ-elixir-c2-clap-param-automation-state**: Map plugin params to Contrapunk's mod matrix (so an LFO can drive a hosted plugin's filter cutoff). Serialise plugin state into Contrapunk's session file via CLAP `state` extension. Save session, reload — automation and state survive. (source: ELIXIR-PLAN.md §5 Phase C2)
- [ ] **REQ-elixir-c3-vst3-hosting**: New `src/plugin_host/vst3/` module. Use a Rust VST3 host crate. Mirror CLAP module's API surface (`discovery`, `host`, `block`, `controller`, `window`). Load FabFilter Pro-Q4 VST3, route harmony through it, GUI embeds, state saves. (source: ELIXIR-PLAN.md §5 Phase C3)
- [ ] **REQ-elixir-c4-au-hosting**: `src/plugin_host/au/`. Use `audio-unit` crate or hand-roll over `objc2`. AU is macOS-only — wrap module in `#[cfg(target_os = "macos")]`. Load Apple stock AU instruments through Contrapunk. (source: ELIXIR-PLAN.md §5 Phase C4)
- [ ] **REQ-elixir-c-ui-multi-plugin-strip**: When ≥ 2 plugins are loaded, Contrapunk needs a per-slot UI strip: name, format badge (CLAP/VST3/AU), bypass, latency report, "open GUI", parameter expander, "remove". Folds into Contrapunk's existing Svelte UI (NOT Track B's egui). (source: ELIXIR-PLAN.md §5 multi-plugin chain UI block)
- [ ] **REQ-elixir-c-plugin-discovery**: Standard OS paths per format. Scan is async, off audio thread, cached to `~/.config/contrapunk/plugins.json` with mtime invalidation. Existing `discovery.rs` already does this for CLAP — generalize the pattern. (source: ELIXIR-PLAN.md §5 discovery block)

### Cross-cutting Elixir requirements

- [ ] **REQ-elixir-wasm-core-compiles**: `elixir-core` MUST compile to wasm32. Plugin-hosting code stays `cfg(not(target_arch = "wasm32"))`. CI runs `cargo check --target wasm32-unknown-unknown -p elixir-core` on every PR. (source: ELIXIR-PLAN.md §6, §7)
- [ ] **REQ-elixir-release-pipeline-extension**: `.github/workflows/` adds elixir-standalone + elixir-plugin release flows next to existing `macos-build.yml`. `release-patch` skill extended to recognize `elixir-` tag prefix; CI workflow YAML pattern-matches the prefix and picks the right build matrix. (source: ELIXIR-PLAN.md §1, §4 public release block)
- [ ] **REQ-elixir-bundle-ids-reserved**: Reserve `com.contrapunk.elixir` and `com.contrapunk.elixir.plugin` bundle IDs in App Store Connect early. (source: ELIXIR-PLAN.md §11 week 1 step 4)

## Out of Scope

| Feature | Reason |
|---------|--------|
| Audio-to-MIDI conversion | Reduces complexity, removes heavy dependencies |
| Algorithmic melody generation | Scope reduction for v1, focus on live performance |
| TUI/curses interface | Replaced by native GUI |
| Tkinter GUI | Legacy Python UI not being ported |
| MIDI file output | Real-time focus for v1 |

## Traceability

| Requirement | Phase | Status |
|-------------|-------|--------|
| MIDI-01 | Phase 1 | Complete |
| MIDI-02 | Phase 1 | Complete |
| MIDI-03 | Phase 1 | Complete |
| MIDI-04 | Phase 1 | Complete |
| CONF-01 | Phase 2 | Complete |
| CONF-02 | Phase 2 | Complete |
| CONF-03 | Phase 2 | Complete |
| HARM-01 | Phase 2 | Complete |
| HARM-02 | Phase 2 | Complete |
| HARM-03 | Phase 2 | Complete |
| HARM-04 | Phase 2 | Complete |
| HARM-05 | Phase 2 | Complete |
| HARM-06 | Phase 2 | Complete |
| HARM-07 | Phase 2 | Complete |
| GUI-01 | Phase 3 | Complete |
| GUI-02 | Phase 3 | Complete |
| GUI-03 | Phase 3 | Complete |
| GUI-04 | Phase 3 | Complete |
| GUI-05 | Phase 3 | Complete |
| DIST-01 | Phase 3 | Complete |
| OCT-01 | Phase 5 | Complete |
| OCT-02 | Phase 5 | Complete |
| OCT-03 | Phase 5 | Complete |
| HUM-01 | Phase 6 | Complete |
| HUM-02 | Phase 6 | Complete |
| HUM-03 | Phase 6 | Complete |
| HUM-04 | Phase 6 | Complete |
| HUM-05 | Phase 6 | Complete |
| HUM-06 | Phase 6 | Complete |
| REQ-elixir-a0-workspace-bootstrap | Phase 21.A0 | Not started |
| REQ-elixir-a1-bare-oscillator | Phase 21.A1 | Not started |
| REQ-elixir-a2-polyphony-voice-management | Phase 21.A2 | Not started |
| REQ-elixir-a3-modulation-matrix-v1 | Phase 21.A3 | Not started |
| REQ-elixir-a4-filter | Phase 21.A4 | Not started |
| REQ-elixir-a5-fx-bus-mvp | Phase 21.A5 | Not started |
| REQ-elixir-a6-spectral-and-fx-completion | Phase 21.A6 | Not started |
| REQ-elixir-a-cut-full-parity-cutover | Phase 21.A-Cut | Not started |
| REQ-elixir-a7-default-flip-and-cleanup | Phase 21.A7 | Not started |
| REQ-elixir-b0-standalone-skeleton | Phase 21.B0 | Not started |
| REQ-elixir-b1-standalone-first-sound | Phase 21.B1 | Not started |
| REQ-elixir-b2-standalone-polyphony | Phase 21.B2 | Not started |
| REQ-elixir-b3-plugin-skeleton | Phase 21.B3 | Not started |
| REQ-elixir-b4-plugin-full-params | Phase 21.B4 | Not started |
| REQ-elixir-b5-preset-save-load | Phase 21.B5 | Not started |
| REQ-elixir-b6-standalone-ui-v1 | Phase 21.B6 | Not started |
| REQ-elixir-b7-wavetable-editor | Phase 21.B7 | Not started |
| REQ-elixir-b8-headless-renderer | Phase 21.B8 | Not started |
| REQ-elixir-b9-public-v0-1-0-release | Phase 21.B9 | Not started |
| REQ-elixir-c0-clap-activation | Phase 21.C0 | Not started |
| REQ-elixir-c1-plugin-gui-embedding | Phase 21.C1 | Not started |
| REQ-elixir-c2-clap-param-automation-state | Phase 21.C2 | Not started |
| REQ-elixir-c3-vst3-hosting | Phase 21.C3 | Not started |
| REQ-elixir-c4-au-hosting | Phase 21.C4 | Not started |
| REQ-elixir-c-ui-multi-plugin-strip | Phase 21.C0 | Not started |
| REQ-elixir-c-plugin-discovery | Phase 21.C0 | Not started |
| REQ-elixir-wasm-core-compiles | Phase 21.A0 | Not started |
| REQ-elixir-release-pipeline-extension | Phase 21.B9 | Not started |
| REQ-elixir-bundle-ids-reserved | Phase 21.A0 | Not started |

**Coverage:**
- v1 requirements: 28 total
- Mapped to phases: 28
- Elixir milestone requirements: 24 total
- Elixir mapped to phases: 24
- Unmapped: 0

---
*Requirements defined: 2026-01-28*
*Last updated: 2026-05-18 — Ingested 24 Elixir milestone requirements (Tracks A/B/C + cross-cutting) via `/gsd-ingest-docs`. Queued behind v1.3.0 tag.*
