# Roadmap: Contrapunk Rust

## Overview

Port Contrapunk from Python to Rust for real-time MIDI harmony generation. The journey builds incrementally: first establish MIDI connectivity (hear notes pass through), then implement harmony algorithms (hear harmonies), finally wrap in a native GUI and ship as a single binary.

## Phases

**Phase Numbering:**
- Integer phases (1, 2, 3): Planned milestone work
- Decimal phases (2.1, 2.2): Urgent insertions (marked with INSERTED)

Decimal phases appear between their surrounding integers in numeric order.

- [x] **Phase 1: MIDI Foundation** - Establish MIDI input/output connectivity with pass-through
- [x] **Phase 2: Harmony Engine** - Implement music theory core and all 7 harmony modes
- [x] **Phase 3: GUI and Distribution** - Native egui interface and single-binary packaging
- [x] **Phase 4: Server Mode** - Network server for remote MIDI harmony processing
- [x] **Phase 5: Octave Variations** - Sub-modes for counterpoint with octave placement options
- [x] **Phase 5.1: WASM and In-Browser Support** - Compile to WebAssembly for browser-based use with Fly.io deployment (INSERTED)
- [ ] **Phase 6: Humanization** - Add timing jitter, velocity variation, groove, and internal beat clock/metronome to generated notes
- [ ] **Phase 6.1: Humanization UI Fix** - Fix humanization controls visibility in browser, ensure effects are audible end-to-end, redesign humanization UI (INSERTED)
- [x] **Phase 6.2: Voice Leading** - Improved voice leading with smooth transitions, minimal motion, and voice independence (INSERTED)
- [x] **Phase 6.3: Style Update** - UI visual redesign and musical style presets (jazz, classical, pop, etc.) (INSERTED)
- [ ] **Phase 6.4: Modal Harmony & Chord Detection** - All church modes + harmonic/melodic minor, modal interchange with visual feedback, comprehensive chord detection (INSERTED)
- [ ] **Phase 6.5: Note Generator** - Virtual MIDI input source: beat-synced note generation (arpeggiator, scale runner, random, chord player), selectable as IN source alongside physical devices (INSERTED)
- [x] **Phase 6.6: Default MIDI Selection** - Persist default MIDI input/output device selections across sessions on both native and WASM (INSERTED)
- [x] **Phase 6.7: Extended Scale Modes & Barry Harris** - All harmonic/melodic minor modes, exotic scales, Barry Harris 6th diminished scales and movement rules (INSERTED)
- [x] **Phase 6.8: CI Fix** - Fix CI pipeline issues (INSERTED)
- [x] **Phase 6.9: Repo Cleanup & Documentation** - Remove legacy Python scripts, clean up README, create comprehensive docs (INSERTED)
- [ ] **Phase 6.10: Docs** - Additional documentation work (INSERTED)
- [ ] **Phase 6.10.1: UI Modernization** - Move from egui to a more sophisticated GUI framework (INSERTED)
- [ ] **Phase 6.11: Logo** - Gold illuminated manuscript C logo — integrated in app, favicon, and docs (INSERTED, design complete)
- [ ] **Phase 7: Performance Mode** - Auto-detect chords/key/BPM from backing track + beat-aware accompaniment generation from accumulated playing
- [ ] **Phase 8: Mic Input** - Audio capture with pitch detection for audio-to-MIDI conversion and raw audio passthrough
- [ ] **Phase 9: Vocoder** - Classic vocoder (carrier modulated by voice) and harmony vocoder (real-time vocal harmonization)
- [ ] **Phase 10: Guitar Input** - Audio input from guitar with pitch detection (runtime exists; broad original claims remain unverified)
- [x] **Phase 10.1: Guitar Reliability** - Automated corpus, lifecycle, realtime-safety, and truthful-calibration gates complete; manual release-surface smoke remains (INSERTED)
- [ ] **Phase 10.2: Composer-Informed Arrangement Presets** - Research, author, and implement 50 technique-named global arrangements; hard-stop for user testing after the first 12 operational presets (INSERTED)
- [ ] **Phase 10.3: Internal MIDI Looper** - Build and performance-test one bar-quantized, pre-arrangement MIDI phrase loop before any Daisy firmware or hardware UX work resumes (INSERTED)
- [ ] **Phase 6.12: DMG Distribution** - Ship Contrapunk as a signed macOS DMG with app icon, codesigning, and notarization (INSERTED)
- ~~**Phase 11: Trackpad Beat Input**~~ - DROPPED
- [ ] **Phase 16: VST3/CLAP/AU Plugin** - nih-plug plugin with webview GUI, AU wrapper via clap-wrapper (GitHub #15)
- [ ] **Phase 17: Integration Test Pipeline** - Real guitar recordings + basic-pitch ground truth, pitch accuracy benchmarks (GitHub #27)
- [ ] **Phase 18: Basic-Pitch Polyphonic Analysis** - tract-onnx integration for Performance Mode chord/key detection (GitHub #28)
- [ ] **Phase 19: NeuralNote Real-Time Research** - Frame-by-frame CNN decomposition for real-time polyphonic detection (GitHub #29)
- [ ] **Phase 20: Release Engineering** - Weekly patch + monthly minor releases, codename generator, backport workflow, Dependabot
- [ ] **Phase 21: Elixir Milestone (v1.5 / elixir-v0.1.0)** - Synthesizer engine + standalone product + multi-plugin hosting. Three parallel tracks (A/B/C) sharing the `Chain`/`AudioBlock` substrate. Queued behind v1.3.0. (INGESTED 2026-05-18)

## Phase Details

### Phase 1: MIDI Foundation
**Goal**: User can connect MIDI devices and hear notes pass through the application
**Depends on**: Nothing (first phase)
**Requirements**: MIDI-01, MIDI-02, MIDI-03, MIDI-04
**Success Criteria** (what must be TRUE):
  1. User can select a MIDI input device from a list of available ports
  2. User can select 2-8 MIDI output ports from available ports
  3. User can play a note on input device and hear it on the first output port
  4. Application runs without GUI (CLI or headless mode) for testing MIDI flow
**Plans**: 3 plans

Plans:
- [x] 01-01-PLAN.md — Project setup and MIDI port enumeration
- [x] 01-02-PLAN.md — MIDI connections and pass-through routing
- [x] 01-03-PLAN.md — Hardware verification checkpoint

### Phase 2: Harmony Engine
**Goal**: User can play notes and hear harmonies generated in any of the 7 modes
**Depends on**: Phase 1
**Requirements**: CONF-01, CONF-02, CONF-03, HARM-01, HARM-02, HARM-03, HARM-04, HARM-05, HARM-06, HARM-07
**Success Criteria** (what must be TRUE):
  1. User can select musical key (C through B) and it affects harmony output
  2. User can switch between all 7 harmony modes and hear different results
  3. User can change key and mode while playing without stopping or restarting
  4. Mode 1 passes notes through unchanged
  5. Modes 2-7 produce audibly different harmonies following their algorithms
**Plans**: 6 plans

Plans:
- [x] 02-01-PLAN.md — Foundation types (Key, HarmonyMode, Scale with diatonic transposition)
- [x] 02-02-PLAN.md — Stateless modes 1-5 and HarmonyEngine struct
- [x] 02-03-PLAN.md — Stateful modes 6-7 (ContraryMotion, StrictCounterpoint)
- [x] 02-04-PLAN.md — Router integration with harmony processing
- [x] 02-05-PLAN.md — CLI key and mode selection
- [x] 02-06-PLAN.md — Hardware verification checkpoint

### Phase 3: GUI and Distribution
**Goal**: User has a complete native application with visual interface as a single binary
**Depends on**: Phase 2
**Requirements**: GUI-01, GUI-02, GUI-03, GUI-04, GUI-05, DIST-01
**Success Criteria** (what must be TRUE):
  1. Application opens as a native window (not terminal)
  2. User can see current configuration (key, mode, active notes) in the GUI
  3. User can change all settings (input device, output ports, key, mode) via GUI controls
  4. Application compiles to single binary that runs without external dependencies
  5. Virtual piano keyboard shows input notes and generated harmony notes
  6. Chord detection displays what chord the combined notes form
**Plans**: 6 plans

Plans:
- [x] 03-01-PLAN.md — GUI foundation with eframe/egui setup and basic window
- [x] 03-02-PLAN.md — Configuration controls and MIDI routing integration
- [x] 03-03-PLAN.md — Real-time active notes display with shared state
- [x] 03-04-PLAN.md — Virtual piano keyboard widget
- [x] 03-05-PLAN.md — Chord detection and display
- [x] 03-06-PLAN.md — Release build optimization and verification

### Phase 4: Server Mode
**Goal**: Binary runs as a server allowing remote users to connect and receive MIDI harmony generations back to their output devices
**Depends on**: Phase 3
**Requirements**: SRV-01, SRV-02, SRV-03, SRV-04, SRV-05
**Success Criteria** (what must be TRUE):
  1. Application can start in server mode, listening on a configurable network port
  2. Remote clients can connect and send MIDI input to the server
  3. Server processes MIDI through harmony engine and returns harmonized output to clients
  4. Multiple clients can connect simultaneously (server handles concurrent sessions)
  5. Clients receive harmonized MIDI output routable to their local output devices
**Plans**: 4 plans

Plans:
- [x] 04-01-PLAN.md — Wire protocol and server configuration types
- [x] 04-02-PLAN.md — Session handler and server accept loop
- [x] 04-03-PLAN.md — CLI integration (clap) and client mode
- [x] 04-04-PLAN.md — End-to-end verification with MIDI hardware

### Phase 5: Octave Variations
**Goal**: User can apply octave placement variations to counterpoint and harmony modes
**Depends on**: Phase 2 (Harmony Engine)
**Requirements**: OCT-01, OCT-02, OCT-03
**Success Criteria** (what must be TRUE):
  1. User can enable "Octave Spread" mode where each harmony voice is in progressively different octaves
  2. User can enable "Bass/Treble Split" where harmonies below melody go low octave, above go high
  3. User can enable "Mirror Octaves" where harmonies duplicate across multiple octaves
  4. Octave variations can be combined with any existing harmony mode
  5. GUI displays which octave variation is active
**Plans**: 1 plan

Plans:
- [x] 05-01-PLAN.md — True Mirror Octaves duplication with port-aware routing

### Phase 5.1: WASM and In-Browser Support (INSERTED)
**Goal**: Compile Contrapunk to WebAssembly for browser-based use and deploy via Fly.io
**Depends on**: Phase 5
**Requirements**: WASM-01, WASM-02, WASM-03
**Success Criteria** (what must be TRUE):
  1. Application compiles to WebAssembly and runs in a modern browser
  2. Web MIDI API provides input/output device access in-browser
  3. GUI renders correctly via egui's WASM backend
  4. Application is deployed and accessible via Fly.io
  5. Browser users can use harmony engine with acceptable latency
**Plans**: 3 plans

Plans:
- [x] 05.1-01-PLAN.md — WASM compilation foundation and Trunk build setup
- [x] 05.1-02-PLAN.md — Web MIDI backend and frame-based polling
- [x] 05.1-03-PLAN.md — Fly.io deployment and browser verification

### Phase 6: Humanization
**Goal**: User can add human-like imperfections to generated harmony notes, with an internal beat clock for musically-aware timing
**Depends on**: Phase 2 (Harmony Engine)
**Requirements**: HUM-01, HUM-02, HUM-03, HUM-04, HUM-05, HUM-06
**Success Criteria** (what must be TRUE):
  1. User can enable timing jitter (5-30ms random delays on note onsets)
  2. User can enable velocity variation (±10-20 randomization)
  3. User can enable note duration variation (slight sustain changes)
  4. User can enable swing/groove (off-beat note shifting using beat clock)
  5. Humanization parameters can be adjusted via GUI sliders
  6. Multiple humanization effects can be combined
  7. Internal beat clock tracks BPM and beat position (user-adjustable tempo)
  8. Optional audible metronome click on a dedicated MIDI output channel
**Plans**: 3 plans

Plans:
- [x] 06-01-PLAN.md — Humanize module: config types, beat clock, and humanizer engine
- [x] 06-02-PLAN.md — Delay queue scheduler, metronome, and router integration
- [x] 06-03-PLAN.md — GUI humanization controls and WASM compatibility

### Phase 6.1: Humanization UI Fix (INSERTED)
**Goal**: Fix humanization controls visibility in browser/WASM, ensure humanization audio effects work end-to-end (user can hear the difference), and redesign humanization UI controls for better UX
**Depends on**: Phase 6 (Humanization)
**Requirements**: HUM-UI-01, HUM-UI-02, HUM-UI-03
**Success Criteria** (what must be TRUE):
  1. Humanization controls are visible and functional in the browser/WASM build
  2. Humanization effects (timing jitter, velocity variation, groove) produce audible differences in output
  3. UI controls are redesigned for clarity and ease of use
  4. Native and WASM builds have consistent humanization behavior
**Plans**: 2 plans

Plans:
- [x] 06.1-01-PLAN.md — Wire humanizer into WASM MIDI processing path
- [x] 06.1-02-PLAN.md — Redesign humanization UI with collapsible sections

### Phase 6.2: Voice Leading (INSERTED)
**Goal**: Implement proper counterpoint and voice leading based on deep research into classical counterpoint theory (Fux's species counterpoint, Bach chorale voice leading, Palestrina style rules)
**Depends on**: Phase 6 (Humanization), Phase 2 (Harmony Engine)
**Requirements**: VL-01, VL-02, VL-03, VL-04
**Research-heavy**: This phase requires extensive research into counterpoint writing before implementation. Key areas:
  - Species counterpoint (1st through 5th species, Fux's Gradus ad Parnassum)
  - Bach chorale voice leading conventions (SATB part writing)
  - Palestrina-style rules (consonance treatment, dissonance preparation/resolution)
  - Modern jazz voice leading (drop voicings, guide tones, chromatic approach)
  - Real-time constraints on counterpoint algorithms (must run <5ms per note)
**Success Criteria** (what must be TRUE):
  1. Harmony voices move by the smallest possible interval when chords change (smooth voice leading)
  2. Parallel fifths and octaves are detected and avoided
  3. Voice crossing is minimized (each voice stays in its assigned register — soprano, alto, tenor, bass)
  4. Common tones are held when moving between chords
  5. Dissonance handling follows species counterpoint rules (preparation, suspension, resolution)
  6. Multiple counterpoint styles selectable (strict Palestrina, Bach chorale, jazz, free)
  7. Voice leading mode can be toggled on/off via GUI
  8. Works with all existing harmony modes
**Plans**: 4 plans

Plans:
- [x] 06.2-01-PLAN.md — Core voice leading types, rules, styles, and voicer algorithm
- [x] 06.2-02-PLAN.md — VoiceLeadingProcessor, suspension state machine, and engine integration
- [x] 06.2-03-PLAN.md — GUI voice leading controls (native and WASM)
- [x] 06.2-04-PLAN.md — Human verification checkpoint

### Phase 6.3: Style Update (INSERTED)
**Goal**: Modernize GUI appearance with retro pixel-art theme, add musical style presets, tabbed navigation, and ambient animations
**Depends on**: Phase 6 (Humanization), Phase 3 (GUI)
**Requirements**: STY-01, STY-02, STY-03, STY-04, STY-05
**Success Criteria** (what must be TRUE):
  1. GUI has PICO-8 retro dark theme (green accents, pixel font, sharp corners)
  2. Tabbed navigation (Play/Craft/Settings) with always-visible piano keyboard
  3. Musical style presets selectable from GUI (11+ built-in with character personas)
  4. Each preset configures harmony mode, voice leading, humanization, and octave settings as a bundle
  5. Users can customize and save their own presets (JSON persistence)
  6. Style presets provide audibly distinct musical character
  7. Ambient animations (rotating gears, decorative frame, music-reactive visuals)
**Plans**: 7 plans

Plans:
- [x] 06.3-01-PLAN.md — Theme system, color palette, serde deps, tab navigation skeleton
- [x] 06.3-02-PLAN.md — Preset data model, built-in presets with character personas, PresetManager
- [x] 06.3-03-PLAN.md — Layout refactor: tabbed navigation (Play/Craft/Settings) with always-visible piano
- [x] 06.3-04-PLAN.md — Custom steampunk widgets (ornate sliders, decorative frame, gear helpers)
- [x] 06.3-05-PLAN.md — Preset persistence, JSON export/import, preset management UI in Craft tab
- [x] 06.3-06-PLAN.md — Ambient animations (rotating gears, particles) and music-reactive visuals
- [x] 06.3-07-PLAN.md — Human verification checkpoint

**Scoped out:** Dark/light toggle (dark-only by decision), 3D harmonic visualizer (deferred)

### Phase 6.4: Modal Harmony & Chord Detection (INSERTED)
**Goal**: Add all 7 church modes plus harmonic/melodic minor scales, modal interchange for smarter chromatic note handling with visual feedback, and comprehensive chord detection (extended chords, slash chords, add chords)
**Depends on**: Phase 6.3 (Style Update), Phase 2 (Harmony Engine)
**Requirements**: MOD-01, MOD-02, MOD-03, CHD-01, CHD-02
**Current status**: Code 100% implemented (audit 2026-04-04). UX/usage review needed — user wants to reconsider how modal harmony is surfaced and used in practice.
**Success Criteria** (what must be TRUE):
  1. User can select scale mode (Ionian, Dorian, Phrygian, Lydian, Mixolydian, Aeolian, Locrian) in addition to key
  2. Harmonic minor and melodic minor scale variants are available
  3. Out-of-key notes use modal interchange (borrowing from parallel modes) instead of generic consonant intervals
  4. GUI visually indicates when modal interchange is happening (e.g., highlight borrowed notes, show source mode)
  5. Chord detection recognizes extended chords (9th, 11th, 13th), altered dominants (b9, #9, #11, b13)
  6. Chord detection recognizes slash chords (C/E), add chords (Cadd9), and 6th chords
  7. All previously unrecognized note combinations now show a chord name instead of raw note names
  8. Scale mode selection works with all existing harmony modes and voice leading
**Plans**: 4 plans

Plans:
- [x] 06.4-01-PLAN.md — ScaleMode enum, Scale::new() parameterization, modal interchange, engine wiring
- [x] 06.4-02-PLAN.md — Expanded chord detection (40+ patterns, slash chords, roman numerals)
- [x] 06.4-03-PLAN.md — GUI integration (scale dropdown, interchange controls, piano tinting, chord display)
- [ ] 06.4-04-PLAN.md — Human verification checkpoint (UX review pending)

### Phase 6.5: Note Generator + WASM Feature Parity (INSERTED)
**Goal**: Wire up the existing note generator engine to UI/routing, AND fix WASM feature parity so humanize and generator work in the browser — not just Tauri desktop.
**Depends on**: Phase 6 (Humanization — beat clock), Phase 6.3 (Style Update — UI tabs/layout)
**Requirements**: GEN-01, GEN-02, GEN-03, GEN-04, GEN-05, WASM-PARITY-01
**Current status**: Rust engine complete + tested. Not wired to UI/routing. WASM adapter stubs out humanize.
**Success Criteria** (what must be TRUE):
  1. "Note Generator" appears as a selectable option in the IN device dropdown
  2. User can select specific notes (click piano keys or note names) to feed into the harmony engine
  3. User can select a chord (e.g., Cmaj, Am7) and all chord tones feed into the harmony engine
  4. Beat-synced generator modes work: arpeggiator (up/down/up-down), scale runner, random diatonic
  5. Generator notes flow through the full harmony engine pipeline (harmonized, voice-led, humanized)
  6. Generator can run alongside a physical MIDI input (both sources merge into the engine)
  7. Generator uses the existing BeatClock (shared BPM/time-signature with metronome)
  8. Works in both native and WASM builds
  9. Humanize panel works in WASM/browser (not just Tauri) — expose humanize config via wasm-bindgen
  10. Generator works in WASM/browser — expose generator state/events via wasm-bindgen
**Plans**: 4 plans

Plans:
- [ ] 06.5-01-PLAN.md — Generator module: config types (modes, chords, events) and NoteGenerator engine
- [ ] 06.5-02-PLAN.md — Integration: wire generator into app update loop and IN device dropdown
- [ ] 06.5-03-PLAN.md — Generator UI: mode selector, chord picker, piano click-to-select
- [ ] 06.5-04-PLAN.md — Human verification checkpoint

### Phase 6.6: Default MIDI Selection (INSERTED)
**Goal**: Persist default MIDI input and output device selections so they are remembered across application restarts, on both native (file-based eframe::Storage) and WASM (localStorage)
**Depends on**: Phase 6.4 (Modal Harmony — current phase), Phase 3 (GUI)
**Requirements**: DEF-01, DEF-02
**Success Criteria** (what must be TRUE):
  1. User can select MIDI input and output devices, and those selections are remembered on next launch
  2. Device matching uses port names (not indices) since indices can change between sessions
  3. If a previously-saved device is unavailable on launch, selection falls back to "Select..." (no silent failure)
  4. Works on native (eframe file storage) and WASM (localStorage) identically
  5. A "Set as Default" action or automatic save on selection change persists the choice
**Plans**: 1 plan

Plans:
- [ ] 06.6-01-PLAN.md — MidiDefaults persistence module and app integration

### Phase 6.7: Extended Scale Modes & Barry Harris (INSERTED)
**Goal**: Add all modes of harmonic minor (7), all modes of melodic minor (7), popular exotic scales (5), and Barry Harris 6th diminished 8-note scales (2) with Barry Harris movement rules as a new harmony mode
**Depends on**: Phase 6.4 (Modal Harmony & Chord Detection), Phase 2 (Harmony Engine)
**Requirements**: ESM-01, ESM-02, ESM-03
**Success Criteria** (what must be TRUE):
  1. All 7 modes of harmonic minor are selectable as scale modes
  2. All 7 modes of melodic minor are selectable as scale modes
  3. Popular exotic scales (Double Harmonic, Hungarian Minor, Enigmatic, Neapolitan Minor/Major) are available
  4. Barry Harris Major 6th Diminished and Minor 6th Diminished 8-note scales are available
  5. Scale dropdown groups modes by family (Church, Harmonic Minor, Melodic Minor, Exotic, Barry Harris)
  6. Barry Harris harmony mode implements movement rules (chord tones to chord tones, passing tones to passing tones)
  7. Scale system supports variable-length scales (7 and 8 note) without breaking existing modes
  8. All existing harmony modes work with all new scale types
**Plans**: 3 plans

Plans:
- [x] 06.7-01-PLAN.md — Generalize Scale to variable-length offsets, add ~28 ScaleMode variants with ScaleFamily
- [x] 06.7-02-PLAN.md — Barry Harris harmony mode with 6th diminished movement rules
- [x] 06.7-03-PLAN.md — Grouped scale dropdown UI and Barry Harris in harmony mode list

### Phase 6.8: CI Fix (INSERTED)
**Goal**: Fix CI pipeline issues
**Depends on**: Phase 6.7
**Plans**: 1 plan

Plans:
- [x] 06.8-01-PLAN.md — Fix WASM build and improve CI caching

### Phase 6.9: Repo Cleanup & Documentation (INSERTED)
**Goal**: Consolidate CI/CD, remove legacy Python files, rewrite README for Rust project, update .gitignore
**Depends on**: Phase 6.8
**Plans**: 3 plans

Plans:
- [x] 06.9-01-PLAN.md — Merge ci.yml and deploy.yml into unified CI/CD workflow
- [x] 06.9-02-PLAN.md — Remove legacy Python files and update .gitignore for Rust
- [x] 06.9-03-PLAN.md — Rewrite README.md for the Rust project

### Phase 6.10: Docs (INSERTED)
**Goal**: Improve rustdoc coverage for public APIs and add CONTRIBUTING.md for developer onboarding
**Depends on**: Phase 6.9
**Plans**: 2 plans

Plans:
- [ ] 06.10-01-PLAN.md — Enhance harmony module rustdoc (mod.rs, config.rs, engine.rs)
- [ ] 06.10-02-PLAN.md — Humanize module docs and CONTRIBUTING.md

### Phase 06.10.1: UI Modernization (INSERTED)

**Goal:** Replace egui with Tauri v2 + Svelte 5 for a polished Hyper Light Drifter-inspired DAW interface, with equal desktop and browser support
**Depends on:** Phase 6.10 (Docs), Phase 3 (GUI)
**Plans:** 9/9 plans complete (human verification pending)

Plans:
- [x] 06.10.1-01-PLAN.md — Cargo workspace + Tauri v2 backend with commands and state
- [x] 06.10.1-02-PLAN.md — SvelteKit project scaffolding + HLD design system
- [x] 06.10.1-03-PLAN.md — Platform adapter layer + Svelte 5 rune stores + WASM bridge
- [x] 06.10.1-04-PLAN.md — 88-key piano keyboard + control panel + Ableton layout
- [x] 06.10.1-05-PLAN.md — MIDI device selection + preset management panels
- [x] 06.10.1-06-PLAN.md — Humanization + generator + active notes panels
- [x] 06.10.1-07-PLAN.md — HLD atmospheric effects (particles, glow, beat indicator)
- [x] 06.10.1-08-PLAN.md — WASM build pipeline + Fly.io deployment + CI update
- [x] 06.10.1-09-PLAN.md — Remove egui/CLI/Trunk + human verification

### Phase 6.11: Logo (INSERTED)
**Goal**: Create a programmatic SVG logo for Contrapunk — designed in code, usable as app icon, favicon, splash, and docs header
**Depends on**: Phase 6.10.1 (UI Modernization — need to know the design language)
**Requirements**: LOGO-01
**Success Criteria** (what must be TRUE):
  1. Logo is a pure SVG file generated programmatically (not a bitmap)
  2. Logo reflects Contrapunk's identity: music, counterpoint, harmony, guitar-to-MIDI
  3. Works at all sizes: 16x16 favicon, 64x64 app icon, 512x512 splash, wide header
  4. Matches the existing HLD (Hyper Light Drifter) inspired dark theme — neon accents on dark background
  5. SVG is clean, hand-editable, no external dependencies (no fonts that need loading)
  6. Integrated into: Tauri app window icon, browser favicon, README header, Fly.io deployment
  7. Variant: monochrome version for light backgrounds
**Plans**: TBD

Plans:
- [x] Design gold illuminated manuscript C logo with HLD accents (brainstorming complete)
- [x] Save logo SVG to ui/static/logo.svg
- [x] Integrate logo into StatusBar brand area
- [x] Add SVG favicon to app.html
- [ ] Generate PNG app icons from SVG for Tauri (32x32, 128x128, 128x128@2x, icon.png)
- [ ] Create monochrome variant for light backgrounds

### Phase 6.12: DMG Distribution (INSERTED)
**Goal**: Ship Contrapunk as a properly signed and notarized macOS DMG installer with custom app icon, background, and drag-to-Applications UX
**Depends on**: Phase 6.11 (Logo — need app icon), Phase 6.10.1 (UI Modernization — Tauri build)
**Requirements**: DMG-01
**Success Criteria** (what must be TRUE):
  1. `cargo tauri build` produces a working .app bundle
  2. .app has the Contrapunk logo as its icon (icns format)
  3. DMG is created with custom background showing drag-to-Applications arrow
  4. DMG is code-signed with a valid Apple Developer ID
  5. DMG is notarized with Apple and passes Gatekeeper
  6. DMG file size is reasonable (< 50MB)
  7. Users can download, open DMG, drag to /Applications, and launch without security warnings
**Plans**:
- [ ] TBD (run /gsd:plan-phase 6.12 to break down)

### Phase 7: Performance Mode
**Goal**: Two-part performance mode: (A) auto-detect chords, key/scale, and BPM from a backing track audio stream to configure the harmony engine in real-time, and (B) beat-aware accompaniment generation from accumulated played notes over bars
**Depends on**: Phase 6 (Humanization — BeatClock), Phase 2 (Harmony Engine), Phase 10 (Guitar Input — audio DSP infrastructure)
**Requirements**: PERF-01, PERF-02, PERF-03, PERF-04, PERF-05
**Research-heavy**: This phase requires deep research into:
  - Chromagram extraction from FFT for chord/key detection
  - Krumhansl-Schmuckler key-finding algorithm
  - Onset-based BPM detection (inter-onset interval histograms)
  - Chord template matching (major, minor, 7th, extended chords)
  - Second audio input management (separate from guitar channel)
  - Temporal MIDI buffering tied to BeatClock (accumulating notes across bars)
  - Phrase-level state management (what the user played in last N bars)
  - Generative algorithms that respond to musical context vs individual notes
  - How performance state interacts with existing harmony modes, voice leading, humanization
  - Real-time constraints on bar-aware processing
**Success Criteria** (what must be TRUE):
  1. User can select a second audio input (backing track source) independent of guitar input
  2. System detects chords from backing track audio (~500ms updates) with confidence scores
  3. System detects key/scale from backing track (~4-8s window) using Krumhansl-Schmuckler
  4. System detects BPM from backing track (~4-8s window) using onset-based tempo estimation
  5. Detected chord auto-sets harmony engine root note; detected key auto-sets scale/mode; detected BPM auto-sets tempo
  6. Lock buttons allow user to freeze any auto-detected parameter
  7. Metronome runs in background and Contrapunk tracks bar boundaries
  8. System accumulates user-played notes across configurable bar windows (1-4 bars)
  9. Generated notes respond to the musical context of recent performance (not just current note)
  10. Performance mode produces musically distinct output from standard harmony modes
  11. Works with existing voice leading, humanization, and octave variation settings
  12. GUI displays performance state (detected chord, key, BPM, confidence, current bar, accumulated context)
  13. Works in both native and WASM builds
**Plans**: 0 plans

Plans:
- [ ] TBD (run /gsd:plan-phase 7 to break down)

### Phase 8: Mic Input
**Goal**: Capture audio from microphone for pitch-to-MIDI conversion and raw audio passthrough for vocoder
**Depends on**: Phase 1 (MIDI Foundation)
**Requirements**: MIC-01, MIC-02, MIC-03
**Success Criteria** (what must be TRUE):
  1. User can select an audio input device (microphone) from available sources
  2. Audio-to-MIDI: Detected pitch is converted to MIDI notes feeding the harmony engine
  3. Raw audio capture provides a signal buffer accessible by the vocoder phase
  4. Pitch detection works with acceptable latency (<50ms) for real-time use
  5. GUI displays detected pitch and confidence level
**Plans**: 8 plans

Plans:
- [ ] 08-01-PLAN.md — Audio module foundation with config types, pitch detection types, and note tracker
- [ ] 08-02-PLAN.md — Native audio capture with cpal and lock-free ring buffer
- [ ] 08-03-PLAN.md — Pitch detection engine with YIN algorithm
- [ ] 08-04-PLAN.md — WASM audio capture with Web Audio API (getUserMedia + AnalyserNode)
- [ ] 08-05-PLAN.md — App integration: MicState, unified IN dropdown, frame processing
- [ ] 08-05b-PLAN.md — WASM app integration: WebAudioCapture wiring, permissions
- [ ] 08-06-PLAN.md — UI: Mic Settings, pitch display, level meter, piano coloring
- [ ] 08-07-PLAN.md — Human verification checkpoint

### Phase 9: Vocoder
**Goal**: Apply vocoder effects using mic audio and harmony engine output
**Depends on**: Phase 8 (Mic Input), Phase 2 (Harmony Engine)
**Requirements**: VOC-01, VOC-02, VOC-03, VOC-04
**Success Criteria** (what must be TRUE):
  1. Classic vocoder: Synth/harmony carrier signal modulated by voice input produces robot-voice effect
  2. Harmony vocoder: Singing a note produces harmonized copies of the user's own voice in real-time
  3. User can switch between classic and harmony vocoder modes via GUI
  4. Vocoder parameters (band count, formant shift, wet/dry mix) adjustable via GUI sliders
  5. Vocoder works with any active harmony mode
  6. Audio output routable to system audio device
**Plans**: TBD

Plans:
- [ ] TBD (run /gsd:plan-phase 9 to break down)

### Phase 10: Guitar Input
**Goal**: Accept guitar audio input with pitch detection optimized for guitar frequency range and playing styles, plus ML-based string+fret identification and browser calibration
**Depends on**: Phase 8 (Mic Input — shares audio capture infrastructure)
**Requirements**: GTR-01, GTR-02, GTR-03, GTR-04, GTR-05, GTR-06
**Current status**: ~95% complete on `guitar-input-clean` branch (monophonic DSP pipeline, WASM bindings, browser capture)
**Success Criteria** (what must be TRUE):
  1. User can select audio input for guitar (direct-in or mic'd amp)
  2. Monophonic pitch detection tracks single-note lines with low latency (<30ms)
  3. Polyphonic detection identifies chords (at least triads) from strummed input
  4. Detected notes feed into harmony engine like any other MIDI input
  5. GUI displays detected guitar notes with string/fret visualization
  6. Works with electric guitar (direct input) and acoustic (via mic)
  7. ML string+fret classifier (139 classes: 6 strings x 23 frets + noise) integrated into pipeline as refinement layer
  8. Hybrid DSP+ML architecture: DSP primary (inharmonicity B-coefficient), ML for ambiguous cases
  9. MiGiC-style AI calibration: play notes → auto-set latency, gain, and confidence threshold
  10. Browser calibration path: set_calibration exposed via WASM bindings
  11. NoteOff timing fix: sustain_threshold tuned for real guitar RMS levels (current 0.0045 too low)
**Plans**: TBD

Plans:
- [ ] TBD (run /gsd:plan-phase 10 to break down)

### Phase 10.1: Guitar Reliability (INSERTED)
**Goal**: Make clean, monophonic, standard-tuned six-string guitar input measurable, reliable, deterministic, and real-time safe without claiming unsupported polyphony or instrument coverage.
**Depends on**: Phase 10 (Guitar Input), Phase 16 (plugin worker architecture provides the real-time pattern)
**Requirements**: GTR-REL-01 through GTR-REL-06
**Success Criteria** (what must be TRUE):
  1. A permanent evaluator runs the shipping `GuitarInput` against all 138 checked-in labeled WAV files with a fixed development/holdout split and reports accuracy, octave errors, retriggers, NoteOff cleanup, latency, determinism, and processing speed.
  2. The automated corpus gate requires >=95% exact first-note accuracy, <5% files with false retriggers, 100% matching NoteOff cleanup, deterministic repeated output, no crashes/stuck notes, faster-than-real-time processing, and first-correct onset-relative p95 <=120 ms. These claims apply only to this corpus.
  3. First-note pitch selection, legato history ordering, vibrato hop timing, duplicate/stale bends, and structural live reconfiguration are correct and regression-tested.
  4. Attack consensus, harmonic rejection, octave correction, sustained-note retrigger suppression, and low-string handling meet the gate on both development and untouched holdout partitions.
  5. Tauri's cpal callback only deinterleaves into a bounded lock-free queue; a worker owns and runs `GuitarInput`, sends MIDI/signal updates, handles overflow safely, and shuts down cleanly.
  6. Calibration has one truthful persisted model and the UI says **Tune Guitar** unless/until it measures and applies a real DSP calibration.
**Product boundary**: Clean monophonic single notes, standard tuning, six-string guitar. Chords, double-stops, alternate tunings, bass, extended range, and broad microphone support are deferred.
**Plans**: 1 plan

Plans:
- [x] 10.1-01-PLAN.md — Evaluator, locked gates, corrected corpus, lifecycle fixes, Tauri worker, and truthful Tune Guitar terminology complete; manual release smoke remains

### Phase 10.2: Composer-Informed Arrangement Presets (INSERTED)

**Goal:** Ship 50 research-backed global arrangement presets with Result/Play guidance, composer references, editable Save As metadata, honest capability gating, reusable musical engines, and deterministic note cleanup.
**Depends on:** Phase 10.1 lifecycle discipline, Phase 6.2 voice leading, Phase 6.7 extended scales, current Companion Lane architecture
**Requirements:** ARRP-01 through ARRP-12
**Context:** [10.2-CONTEXT.md](./phases/10.2-arrangement-presets/10.2-CONTEXT.md)
**Catalog:** [10.2-PRESET-CATALOG.md](./phases/10.2-arrangement-presets/10.2-PRESET-CATALOG.md)
**Loop:** [10.2-LOOP.md](./phases/10.2-arrangement-presets/10.2-LOOP.md)
**Research template:** [10.2-RESEARCH-TEMPLATE.md](./phases/10.2-arrangement-presets/10.2-RESEARCH-TEMPLATE.md)

**Research gate:** Before any preset becomes operational, three independent fresh-context researchers cover artist/style history and primary sources, music theory and temporal behavior, and performer/interaction guidance. Research scopes specific works and periods, traces career evolution, separates persistent from period-specific traits, distinguishes shared vocabulary from characteristic choices, and models how material moves within phrases and sections. A parent synthesis records stylistic invariants, evolution states/transitions, caricatures to avoid, acceptable approximations, configuration, and acceptance examples.

**Human checkpoint:** Execution must stop after the first 12 operational presets (catalog IDs 2, 4, 7, 8, 12, 14, 23, 25, 27, 36, 43, 48). Wave 3 cannot begin until the user performs them and explicitly approves continuation.

**Success Criteria:**
1. Exactly 50 stable, unique built-in records include Result, Play guidance, references, requirements, availability, and immutable built-in status.
2. User Save As captures arrangement configuration plus musical guidance and references; built-ins can only be duplicated.
3. Presets preserve tonic, tempo, devices, routing, current sound, master level, mute/solo, and transport state.
4. Preset application validates and applies atomically after one Panic/runtime reset; failure cannot leave partial state or orphan notes.
5. Every operational preset has three cited research reports and a parent synthesis.
6. Every operational preset passes a fixed NoteOn/NoteOff lifecycle simulation and ends with zero active/pending notes.
7. Unsupported presets remain visible but cannot Apply and identify the missing capability/surface.
8. Temporal features reuse Companion Lane phases; stateless note mapping uses shared harmony strategies, not preset-specific branches.
9. The first 12 are audibly differentiated, automatically validated, and delivered to the user for a mandatory performance checkpoint.
10. After user approval, reusable voicing, motif, pattern/timing, stable-group, harmonic-timeline, adaptive, and experimental capabilities unlock the remaining catalog.

**Plans / waves:**
- [ ] 10.2-01 — Safety and V2 foundation: Free Imitation lifecycle, typed schema, centralized state, transactional apply, capability gating
- [ ] 10.2-02 — Three-researcher packs, catalog/authoring UX, first 12 operational presets, automated report, then mandatory user stop
- [ ] 10.2-03 — Reusable interval/voicing strategies (only after user approval)
- [ ] 10.2-04 — Bounded phrase and motif Lane
- [ ] 10.2-05 — Data-driven Pattern Lane and advanced timing
- [ ] 10.2-06 — Stable lane instances/groups, harmonic timeline, and three-state adaptive scenes
- [ ] 10.2-07 — Experimental capabilities, all-50 tuning, and final cross-surface acceptance

### Phase 10.3: Internal MIDI Looper (INSERTED)

**Goal:** Build and test one reliable volatile MIDI phrase loop in Contrapunk before continuing Daisy hardware firmware or screenless hardware UX.
**Depends on:** Current Companion Lane/transport architecture, Phase 10.1 guitar-detected MIDI lifecycle, and checkpointed Phase 10.2 Phrase Context/arrangement work
**Requirements:** LOOP-01 through LOOP-10
**Context:** [10.3-CONTEXT.md](./phases/10.3-internal-midi-looper/10.3-CONTEXT.md)
**Priority gate:** Daisy firmware, Pod/MPK mappings, and further hardware UX remain paused until automated lifecycle evidence and a user-run desktop performance test pass.

**Success Criteria:**
1. One volatile slot follows `Empty → Armed → Recording → Playing → Stopped → Playing`; long-clear returns it safely to Empty.
2. First press starts a stopped transport with a one-bar count-in or arms a running transport for the next downbeat; second press closes on the next downbeat.
3. Capture preserves live pre-arrangement NoteOn, NoteOff, velocity, channel, and sustain CC64 at raw beat-relative offsets.
4. Notes held at closure receive matching boundary NoteOffs; no loop-owned note or sustain state survives stop, clear, panic, transport stop/reset, seek, disable, or reconfigure.
5. Replay passes through the current full arrangement and reharmonizes after key/scale/arrangement changes without mutating the recorded source.
6. Loop replay uses an explicit non-live origin and isolated arrangement/Phrase Context, preventing recursive capture and live phrase-state contamination.
7. Live MIDI or clean monophonic guitar-detected notes can layer over playback without ownership collisions.
8. Coarse/fine/delayed tick simulations emit every due event exactly once across boundaries, and tempo changes preserve beat phase without accumulated drift.
9. Minimal Tauri controls expose press, clear, and truthful state; unsupported WASM/plugin surfaces are capability-gated rather than pretending parity.
10. Focused Rust tests, workspace/UI checks, and a manual desktop MPK-plus-guitar performance session pass before hardware work resumes.

**Explicit non-goals:** audio looping, multiple slots, overdub/undo, output capture, arbitrary CC/pitch/aftertouch capture, cross-boundary ties, persistence, Ableton Link, WASM/plugin/Daisy parity, and hardware control mapping.

**Plans:** 3 plans

Plans:
- [x] 10.3-01-PLAN.md — Continuous transport beats and deterministic one-slot MIDI looper core
- [ ] 10.3-02-PLAN.md — Isolated full-arrangement replay, exact ownership, Tauri routing, and cleanup
- [ ] 10.3-03-PLAN.md — Capability-gated desktop controls, automated gate, and user performance checkpoint

### Phase 14: openDAW Device Integration

**Depends on**: Phase 10 (Guitar Input — WASM API), Phase 5.1 (WASM)

**Goal**: Integrate Contrapunk as a MIDI effect device inside openDAW, enabling real-time counterpoint harmony generation within the openDAW web-based DAW.

**Research**: Complete — see `.planning/research/opendaw-integration.md`

**Success Criteria**:
- [ ] Contrapunk appears as a MIDI effect device in openDAW
- [ ] MIDI notes in → original + harmony voices out
- [ ] Parameters exposed: key, scale, mode, voice count, voice position, voice leading style
- [ ] WASM core runs inside AudioWorklet (NAM pattern)
- [ ] Works with openDAW's automation system

**Steps**:
1. Book Calendly call with André Michelle to discuss integration
2. Prototype with `@opendaw/studio-sdk` (headless, LGPL)
3. Fork openDAW, implement 5-layer device (Schema → Adapter → Processor → Editor → Registration)
4. Submit focused PRs upstream
5. Package as standalone device when runtime loading ships

**Reference implementations**: NeuralAmpDevice (TONE3000), PitchDeviceProcessor, ArpeggioDeviceProcessor

### Phase 15: Contrapunk Cloud

**Depends on**: Phase 14 (openDAW integration), Phase 10 (Guitar Input)

**Goal**: Online jamming platform with real-time counterpoint harmony across players.

**Tagline**: "AI is not going to kill music till people keep playing music together. So let's jam!"

**Success Criteria**:
- [ ] Low-latency audio networking (Rust-based)
- [ ] Harmony engine generates counterpoint across multiple player inputs
- [ ] Shared sessions with automatic key/chord detection
- [ ] Async collaboration mode (record, share, layer)

**Waitlist**: Live at contrapunk.com/cloud — Cloudflare Worker + KV backend

### Phase 11: Trackpad Beat Input
**Goal**: Use computer trackpad as a MIDI beat pad for triggering notes and drums, similar to hardware MIDI pad controllers
**Depends on**: Phase 3 (GUI), Phase 6 (Humanization — beat clock)
**Requirements**: TBD
**Success Criteria** (what must be TRUE):
  1. User can enable trackpad as a virtual MIDI pad input
  2. Trackpad surface divided into configurable grid (e.g., 4x4, 8x2) of trigger zones
  3. Touch/click in a zone triggers a MIDI note (velocity from pressure if available)
  4. Zones can be mapped to any MIDI note (drums, melody notes, chord triggers)
  5. Visual feedback shows which zone is being pressed
  6. Works in both native and WASM builds
  7. Triggered notes flow through harmony engine like any other input
**Plans**: TBD

Plans:
- [ ] TBD (run /gsd:plan-phase 11 to break down)

### Phase 12: Advanced Voice Leading
**Goal**: Comprehensive voice leading techniques covering jazz voicings, motion control, harmonic techniques, resolution-aware processing, and density control
**Depends on**: Phase 6.2 (Voice Leading)
**Requirements**: AVL-01 through AVL-25
**Success Criteria** (what must be TRUE):
  1. Jazz voicings available: Drop 2, Drop 3, rootless, shell, upper structure triads, So What/quartal, locked hands, spread triads
  2. Motion control modes: contrary motion preference, oblique motion, voice exchange, bass-led, tenor-lead
  3. Harmonic techniques: Neo-Riemannian transforms (P, L, R), negative harmony, linear chromaticism, planing/parallelism, chromatic approach
  4. Resolution-aware: tension resolution (9→8, ♯11→5, 13→12), cadential voicing (ii-V-I), dominant preparation, avoid doubling leading tone
  5. Density control: close/open position toggle, register-locked SATB, dynamic density, doubled roots/fifths
  6. All techniques selectable via GUI dropdown or preset system
  7. Techniques combine with existing harmony modes and basic voice leading
  8. Works in both native and WASM builds
**Plans**: TBD

Plans:
- [ ] TBD (run /gsd:plan-phase 12 to break down)

### Phase 13: Voice Leading Test Suite
**Goal**: Comprehensive automated tests for voice leading to replace manual UAT and catch regressions
**Depends on**: Phase 6.2 (Voice Leading)
**Requirements**: VLT-01 through VLT-08
**Success Criteria** (what must be TRUE):
  1. Parallel 5th/octave detection tests verify rules are enforced per style
  2. Style differentiation tests prove Palestrina/Bach/Jazz/Free produce distinct outputs for same input
  3. Voice crossing prevention tests confirm voices stay in assigned registers
  4. Common tone retention tests verify shared notes are held across chord changes
  5. Suspension resolution tests verify Palestrina-style suspensions resolve stepwise down
  6. Integration tests confirm voice leading works with all harmony modes (1-8)
  7. Regression tests for stuck notes on config changes (key, mode, style switches)
  8. All tests run in CI and block merges on failure
**Plans**: TBD

Plans:
- [ ] TBD (run /gsd:plan-phase 13 to break down)

---

## Phase 21: Elixir Milestone (v1.5 / elixir-v0.1.0)

**Source:** [ELIXIR-DESIGN.md](../ELIXIR-DESIGN.md) + [ELIXIR-PLAN.md](../ELIXIR-PLAN.md)
**Ingested:** 2026-05-18 via `/gsd-ingest-docs`
**Status:** Queued behind v1.3.0 tag.

**Overview:** Major new milestone with three PARALLEL tracks sharing the `Chain` / `AudioBlock` substrate in `src/chain/`. These tracks are NOT linear — they run concurrently against the same locked `AudioBlock` trait (locked at A0, additive-only after).

- **Track A** (replace Contrapunk's built-in synth, ~13 weeks): 9 phases A0 → A7 + A-Cut
- **Track B** (standalone Elixir product, ~10 weeks parallel): 10 phases B0 → B9
- **Track C** (multi-plugin hosting in Contrapunk, ~9 weeks parallel): 5 phases C0 → C4

Realistic calendar window with concurrent tracks: **24-28 weeks**.

**Cross-cutting requirements** (not phase-specific): WASM core compile constraint, release-pipeline extension, bundle IDs reserved. See `REQ-elixir-wasm-core-compiles`, `REQ-elixir-release-pipeline-extension`, `REQ-elixir-bundle-ids-reserved` — these are assigned to A0 (bootstrap) and B9 (release) respectively.

### Track A — Replace Contrapunk's built-in synth

#### Phase 21.A0: Workspace Bootstrap
**Goal**: New Elixir workspace exists and compiles to silence on all 4 surfaces with feature flag off
**Depends on**: Nothing (foundation phase)
**Requirements**: REQ-elixir-a0-workspace-bootstrap, REQ-elixir-wasm-core-compiles, REQ-elixir-bundle-ids-reserved
**Success Criteria** (what must be TRUE):
  1. `crates/elixir-core` exists with empty `Engine` exposing `prepare(sr, max_block)` and `process(&mut [f32], channels)` writing silence
  2. `ElixirSynthBlock` wired as a feature-flagged `AudioBlock` in `src/chain/`
  3. Both branches of `#[cfg(feature = "elixir-synth")]` compile
  4. CI green on all 4 surfaces (CLI, Tauri, WASM, plugin) with flag off
  5. `cargo check --target wasm32-unknown-unknown -p elixir-core` runs in CI on every PR
  6. `AudioBlock` trait signature locked (additive-only changes thereafter)
  7. Bundle IDs `com.contrapunk.elixir` and `com.contrapunk.elixir.plugin` reserved in App Store Connect
**Effort**: 3 days
**Plans**: TBD

#### Phase 21.A1: Bare Oscillator
**Goal**: Headphones-plugged-in moment — user plays MIDI notes via CLI and hears a sine
**Depends on**: Phase 21.A0
**Requirements**: REQ-elixir-a1-bare-oscillator
**Success Criteria** (what must be TRUE):
  1. Voice handler + one wavetable oscillator with Catmull-Rom interpolation + fixed-point phase
  2. Spectral mip-mapping for anti-aliasing across the keyboard range
  3. Single pre-baked sine wavetable; one DAHDSR envelope hard-coded to amp
  4. User plays MIDI input through `cargo run --features elixir-synth` and hears the sine
  5. No aliasing across keyboard range
  6. A/B render against `src/synth/Sine` shows < -90 dBFS RMS difference
**Effort**: 2 weeks
**Plans**: TBD

#### Phase 21.A2: Polyphony + Voice Management
**Goal**: User can hold a 16-note chord cleanly with sustain pedal support
**Depends on**: Phase 21.A1
**Requirements**: REQ-elixir-a2-polyphony-voice-management
**Success Criteria** (what must be TRUE):
  1. 16-voice pool with SIMD-packed AggregateVoice (2 voices per `f32x8`)
  2. Voice stealing on 17th note is click-free (Newest priority)
  3. Sustain pedal works end-to-end through harmony engine output
  4. Sample-accurate note-on within a block via per-block sample-offset
**Effort**: 1 week
**Plans**: TBD

#### Phase 21.A3: Modulation Matrix v1
**Goal**: LFOs and envelopes can route to any parameter without clicks or glitches
**Depends on**: Phase 21.A2
**Requirements**: REQ-elixir-a3-modulation-matrix-v1
**Success Criteria** (what must be TRUE):
  1. SoA `ModRoutes` storage with sparse fixed-bank addressing
  2. Two LFOs (custom-waveform + random) and six envelopes per voice
  3. `arc-swap` breakpoint-table hot-swap is click-free
  4. UI→audio command queue (`rtrb`) handles route add/remove without dropout
  5. Modulation-of-modulation works (LFO speed modded by envelope)
**Effort**: 2 weeks
**Plans**: TBD

#### Phase 21.A4: Filter
**Goal**: User hears warm analog ladder + crisp digital SVF + comb filters with audio-rate cutoff modulation
**Depends on**: Phase 21.A3
**Requirements**: REQ-elixir-a4-filter
**Success Criteria** (what must be TRUE):
  1. Digital SVF + analog ladder + comb filters implemented
  2. Cutoff sweep tracks an LFO with no zipper noise
  3. Resonance self-oscillates on the ladder
  4. Comb filter tracks pitch
  5. TPT coefficient LUT eliminates per-sample `tan()`
**Effort**: 2 weeks
**Plans**: TBD

#### Phase 21.A5: FX Bus MVP
**Goal**: User hears a full reverb tail, ping-pong delay, EQ, and drive without clipping at -1 dBFS
**Depends on**: Phase 21.A4
**Requirements**: REQ-elixir-a5-fx-bus-mvp
**Success Criteria** (what must be TRUE):
  1. 2× oversampling (upsampler + 3-pole halfband decimator)
  2. Reorderable 4-slot FX chain (reverb, delay, EQ, distortion)
  3. FDN-8 reverb produces audible tail
  4. Ping-pong delay works in stereo
  5. Drive at -1 dBFS without clipping
  6. Reorder via test rig works without crackle
**Effort**: 3 weeks
**Plans**: TBD

#### Phase 21.A6: Spectral + FX Completion
**Goal**: Full design-doc feature set lands — all 12 spectral morphs, 9 phase-distortion modes, unison, full FX chain, FDN-16 reverb
**Depends on**: Phase 21.A5
**Requirements**: REQ-elixir-a6-spectral-and-fx-completion
**Success Criteria** (what must be TRUE):
  1. All 12 spectral morphs implemented (vocode, smear, harmonic-scale, phase-disperse, shepard, skew, etc.)
  2. 9 phase-distortion modes work (FM, RM, sync, pulsewidth…)
  3. Unison with 11 stack styles
  4. Chorus, flanger, phaser, compressor effects available
  5. FDN-16 reverb replaces FDN-8 (FDN-8 fallback retained for WASM)
  6. Filter models complete: diode, dirty, formant, phaser-filter
  7. Wavetable feature parity with design doc
  8. New unit tests for each morph (golden WAV per morph) pass
  9. All existing tests continue to pass
**Effort**: 4 weeks
**Plans**: TBD

#### Phase 21.A-Cut: Full Parity Cutover
**Goal**: Contrapunk's built-in synth flips to Elixir under `--features elixir-synth` with byte-for-byte parity on `Contrapunk-Default.elxprst`
**Depends on**: Phase 21.A6, Phase 21.B3 (plugin shipping by this point)
**Requirements**: REQ-elixir-a-cut-full-parity-cutover
**Success Criteria** (what must be TRUE):
  1. Every `SynthParams` setter mapped to an Elixir `ParamId`
  2. `Contrapunk-Default.elxprst` factory preset embedded in the binary
  3. Audio chain instantiates `ElixirSynthBlock` instead of `Synth` when `--features elixir-synth`
  4. Existing `tests/audio_pipeline.rs` passes with `--features elixir-synth`
  5. Manual A/B on a fixed MIDI file in Tauri shows no perceptual change
  6. Plugin (VST3/CLAP) renders identically with the new synth
  7. CLI binary renders identically
**Effort**: 1 week
**Plans**: TBD

#### Phase 21.A7: Default Flip + Cleanup
**Goal**: Elixir is the default synth shipped to all Contrapunk users; legacy `src/synth/` deleted
**Depends on**: Phase 21.A-Cut
**Requirements**: REQ-elixir-a7-default-flip-and-cleanup
**Success Criteria** (what must be TRUE):
  1. After 2 weeks of A-Cut opt-in, feature flag default flipped to ON
  2. After another 2 weeks of no regressions, `src/synth/` deleted from tree
  3. `git rm -r src/synth/`; CI green
  4. No `--features` toggle needed by users
  5. `src-tauri/src/commands/synth.rs` uses Elixir's typed params (old `SynthParams` shim dropped)
  6. Release-notes line item for Contrapunk v1.5 (or whichever cycle ships this)
**Effort**: 1 week
**Plans**: TBD

### Track B — Elixir as standalone product

#### Phase 21.B0: Standalone Skeleton
**Goal**: `elixir-standalone` binary exists, opens audio + MIDI + window in a separate process from Contrapunk
**Depends on**: Phase 21.A0 (shares `elixir-core` substrate)
**Requirements**: REQ-elixir-b0-standalone-skeleton
**Success Criteria** (what must be TRUE):
  1. Skeleton binary runs and exits cleanly
  2. `cpal` opens default audio output
  3. `midir` opens default MIDI input
  4. Argv parsing via `clap`
  5. `egui` window opens with "Hello Elixir" label
  6. Window opens as a separate process from Contrapunk's desktop app
**Effort**: 2 days
**Plans**: TBD
**UI hint**: yes

#### Phase 21.B1: Standalone First Sound
**Goal**: User can press a computer keyboard key and hear a note from `elixir-standalone`
**Depends on**: Phase 21.B0, Phase 21.A1 (need oscillator)
**Requirements**: REQ-elixir-b1-standalone-first-sound
**Success Criteria** (what must be TRUE):
  1. `elixir-standalone` produces sound — single voice, default preset
  2. Computer-keyboard fallback works (`a w s e d f t g …` = chromatic)
  3. Notes from MIDI input also produce sound
**Effort**: 3 days
**Plans**: TBD

#### Phase 21.B2: Standalone Polyphony
**Goal**: User can hold a 16-note chord cleanly in `elixir-standalone` with voice meter visible
**Depends on**: Phase 21.B1, Phase 21.A2 (need polyphony from core)
**Requirements**: REQ-elixir-b2-standalone-polyphony
**Success Criteria** (what must be TRUE):
  1. 16-note chord plays cleanly in standalone
  2. Voice meter logged to stderr (active voices count visible)
**Effort**: 1 day
**Plans**: TBD

#### Phase 21.B3: Plugin Skeleton
**Goal**: `elixir-plugin` loads in Bitwig / Logic / Ableton with one automatable parameter
**Depends on**: Phase 21.B2
**Requirements**: REQ-elixir-b3-plugin-skeleton
**Success Criteria** (what must be TRUE):
  1. `elixir-plugin` skeleton built via nih-plug (VST3 + CLAP + AU + standalone wrapper)
  2. Plugin loads in Bitwig (CLAP), Logic (AU), Ableton (VST3)
  3. Master gain parameter automatable from each host
**Effort**: 5 days
**Plans**: TBD

#### Phase 21.B4: Plugin Full Params
**Goal**: Earliest point Contrapunk users can demo Elixir inside their DAW — full parameter set automatable and smooth
**Depends on**: Phase 21.B3, Phase 21.A3 (need mod matrix), Phase 21.A4 (need filter)
**Requirements**: REQ-elixir-b4-plugin-full-params
**Success Criteria** (what must be TRUE):
  1. `elixir-plugin` exposes full parameter set (envelope, filter, mod matrix amount, etc.)
  2. Audio-thread parameter smoothing via nih-plug `SmoothedParam` is click-free
  3. All params automatable from each tested host
**Effort**: 1 week
**Plans**: TBD

#### Phase 21.B5: Preset Save/Load
**Goal**: User can save a preset in `elixir-plugin` and load it in `elixir-standalone` (and vice versa); hot-swap is click-free
**Depends on**: Phase 21.B4
**Requirements**: REQ-elixir-b5-preset-save-load
**Success Criteria** (what must be TRUE):
  1. Preset save / load wired in plugin
  2. Preset save / load wired in standalone
  3. JSON format per design doc; embedded base64 PCM for wavetables and samples
  4. ArcSwap preset hot-swap is click-free
  5. Plugin-saved preset loads in standalone (and vice versa)
**Effort**: 1 week
**Plans**: TBD

#### Phase 21.B6: Standalone UI v1
**Goal**: User can twiddle knobs and see the synth respond — mod-matrix, oscillator, envelope, filter, FX panels work in both standalone and plugin in-DAW window
**Depends on**: Phase 21.B5
**Requirements**: REQ-elixir-b6-standalone-ui-v1
**Success Criteria** (what must be TRUE):
  1. Mod-matrix view renders and is functional
  2. Oscillator panel renders and is functional
  3. Envelope panel renders and is functional
  4. Filter panel renders and is functional
  5. FX chain panel renders and is functional
  6. Same `egui` widget set works in both standalone and `elixir-plugin` in-DAW window
  7. Factory wavetable bank loadable from UI (no editor yet)
**Effort**: 4 weeks
**Plans**: TBD
**UI hint**: yes

#### Phase 21.B7: Wavetable Editor
**Goal**: User can drag wavetable points in time and frequency domains with live spectral morph preview
**Depends on**: Phase 21.B6, Phase 21.A6 (need spectral morphs)
**Requirements**: REQ-elixir-b7-wavetable-editor
**Success Criteria** (what must be TRUE):
  1. UI for editing wavetable frames in time-domain
  2. UI for editing wavetable frames in frequency-domain
  3. Spectral morph parameter live-previews while editing
  4. Frame interpolation visualizes correctly during edit
**Effort**: 2 weeks
**Plans**: TBD
**UI hint**: yes

#### Phase 21.B8: Headless Renderer
**Goal**: User can render `--midi foo.mid --preset bar.elxprst --out baz.wav --duration 60s` and get a deterministic WAV
**Depends on**: Phase 21.B5 (need preset format)
**Requirements**: REQ-elixir-b8-headless-renderer
**Success Criteria** (what must be TRUE):
  1. `elixir-headless` binary builds
  2. CLI rendering produces a WAV from MIDI + preset
  3. Output is deterministic (same input → same output bytes)
  4. `--duration` flag is honored
**Effort**: 3 days
**Plans**: TBD

#### Phase 21.B9: Public v0.1.0 Release
**Goal**: Public v0.1.0 of Elixir released from this repo with signed/notarized artifacts on a GitHub Release under `elixir-v0.1.0` tag
**Depends on**: Phase 21.B6, Phase 21.B8 (must have UI + headless before public release)
**Requirements**: REQ-elixir-b9-public-v0-1-0-release, REQ-elixir-release-pipeline-extension
**Success Criteria** (what must be TRUE):
  1. `cargo test -p elixir-core`, `-p elixir-plugin`, `-p elixir-standalone` all pass
  2. Build matrix: macOS arm64 + x86_64 universal, Windows x86_64, Linux x86_64
  3. macOS artifacts signed (Developer ID) and notarized using existing Contrapunk certs
  4. Windows artifacts signed (EV cert)
  5. GitHub Release attaches: `elixir-standalone.dmg`, `Elixir.vst3`, `Elixir.clap`, `Elixir.component` (AU), `elixir-headless` binary
  6. `release-patch` skill recognizes the `elixir-` tag prefix
  7. CI workflow YAML pattern-matches the prefix and picks the right build matrix
**Effort**: 1 week
**Plans**: TBD

### Track C — Multi-plugin hosting in Contrapunk

**Cross-reference:** Track C is complementary to existing ROADMAP Phase 16 (VST3/CLAP/AU Plugin). Phase 16 = Contrapunk packaged AS a plugin. Track C = Contrapunk hosting OTHER plugins. They share `src/plugin_host/`-related plumbing but ship different products. Existing `src/plugin_host/clap/block.rs` and `controller.rs` stubs are finished by Phase 21.C0.

#### Phase 21.C0: CLAP Activation
**Goal**: User loads Surge XT (free CLAP) in Contrapunk, plays notes from the harmony engine, and hears it
**Depends on**: Phase 21.A0 (need `AudioBlock` trait locked)
**Requirements**: REQ-elixir-c0-clap-activation, REQ-elixir-c-plugin-discovery, REQ-elixir-c-ui-multi-plugin-strip
**Success Criteria** (what must be TRUE):
  1. Existing `block.rs` / `controller.rs` stubs in `src/plugin_host/clap/` filled in
  2. A discovered CLAP plugin instantiates, activates, and processes audio through `ClapAudioBlock`
  3. Chain's existing `PushBlock` queue routes audio through the plugin
  4. User loads Surge XT, plays notes from harmony engine, hears it (no GUI yet; plugin opens its own OS window)
  5. Plugin discovery scans standard OS paths and caches to `~/.config/contrapunk/plugins.json` with mtime invalidation
  6. When ≥ 2 plugins loaded, Contrapunk's Svelte UI shows per-slot strip (name, format badge, bypass, latency, "open GUI", expander, "remove")
**Effort**: 2 weeks (+ ~1 week UI strip)
**Plans**: TBD
**UI hint**: yes

#### Phase 21.C1: Plugin GUI Embedding
**Goal**: Surge XT GUI shows inside Contrapunk's desktop window with resize
**Depends on**: Phase 21.C0
**Requirements**: REQ-elixir-c1-plugin-gui-embedding
**Success Criteria** (what must be TRUE):
  1. CLAP `gui` extension flow implemented: query embedded-size hint
  2. Child window created inside Tauri's main window (macOS NSView, Windows HWND, Linux X11)
  3. Fallback to detached floating window when embed fails
  4. Surge XT GUI renders inside Contrapunk's desktop window on macOS; resize works
**Effort**: 2 weeks
**Plans**: TBD
**UI hint**: yes

#### Phase 21.C2: CLAP Param Automation + State
**Goal**: User can automate a hosted plugin's parameter from Contrapunk's mod matrix and save the plugin state in Contrapunk's session
**Depends on**: Phase 21.C1, Phase 21.A3 (need mod matrix from Track A)
**Requirements**: REQ-elixir-c2-clap-param-automation-state
**Success Criteria** (what must be TRUE):
  1. Plugin params mapped to Contrapunk's mod matrix (`ParamId::Hosted { slot, plugin_param_id }`)
  2. LFO can drive a hosted plugin's filter cutoff
  3. Plugin state serializes into Contrapunk's session file via CLAP `state` extension
  4. Set up Diva CLAP, automate cutoff from Contrapunk's macro 1, save session, reload — automation and state survive
**Effort**: 2 weeks
**Plans**: TBD

#### Phase 21.C3: VST3 Hosting
**Goal**: Contrapunk hosts generic VST3 instruments directly, beginning with Analog Lab V, then extends the same host boundary to effects
**Depends on**: Phase 21.C2
**Requirements**: REQ-elixir-c3-vst3-hosting
**Status note (2026-07-20)**: Parked by user decision. The immediate workflow is Contrapunk compiled as a MIDI-output VST3/CLAP and placed before Analog Lab V in the DAW. Resume direct hosting only when Track C is active.
**Success Criteria** (what must be TRUE):
  1. New `src/plugin_host/vst3/` module exists
  2. Rust VST3 host crate integrated after license review
  3. VST3 module mirrors the CLAP module API surface (`discovery`, `host`, `block`, `controller`, `window`)
  4. Instrument-first slice discovers and loads the installed `Analog Lab V.vst3`, sends Contrapunk MIDI events, and receives stereo audio without allocating or locking on the audio callback
  5. Analog Lab V's floating editor opens and resizes safely; embedded GUI remains optional
  6. VST3 component/controller state saves and reloads in Contrapunk sessions
  7. A later effect slice validates audio-input bus negotiation with FabFilter Pro-Q or equivalent; sidechains remain out of the instrument-first slice
**Effort**: 3 weeks
**Plans**: TBD
**UI hint**: yes

#### Phase 21.C4: AU Hosting
**Goal**: macOS user loads Apple's stock AU instruments through Contrapunk and hears them
**Depends on**: Phase 21.C3
**Requirements**: REQ-elixir-c4-au-hosting
**Success Criteria** (what must be TRUE):
  1. `src/plugin_host/au/` exists, wrapped in `#[cfg(target_os = "macos")]`
  2. `audio-unit` crate or hand-rolled `objc2` bindings work
  3. Apple stock AU instruments load and play through Contrapunk
  4. Windows/Linux builds skip the AU module cleanly
**Effort**: 2 weeks
**Plans**: TBD
**UI hint**: yes

## Progress

**Execution Order:**
Phases execute in numeric order: 1 -> 2 -> 3 -> 4 -> 5 -> 5.1 -> 6 -> 6.1 -> 6.2 -> 6.3 -> 6.4 -> 6.5 -> 6.6 -> 6.7 -> 6.8 -> 6.9 -> 6.10 -> 6.10.1 -> 7 -> 8 -> 9 -> 10
(Phase 11 dropped. Phases 12, 13 closed — already implemented.)

Phase 21 (Elixir Milestone) is QUEUED behind v1.3.0 release. The three tracks (A/B/C) run in parallel internally, sharing the `Chain`/`AudioBlock` substrate.

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 1. MIDI Foundation | 3/3 | Complete | 2026-01-28 |
| 2. Harmony Engine | 6/6 | Complete | 2026-01-28 |
| 3. GUI and Distribution | 6/6 | Complete | 2026-01-28 |
| 4. Server Mode | 4/4 | Complete | 2026-01-29 |
| 5. Octave Variations | 1/1 | Complete | 2026-01-29 |
| 5.1 WASM & Browser (INSERTED) | 3/3 | Complete | 2026-01-29 |
| 6. Humanization | 3/3 | Complete | 2026-01-29 |
| 6.1 Humanization UI Fix (INSERTED) | 2/2 | Complete | 2026-01-30 |
| 6.2 Voice Leading (INSERTED) | 4/4 | Complete | 2026-01-30 |
| 6.3 Style Update (INSERTED) | 7/7 | Complete | 2026-01-31 |
| 6.4 Modal Harmony & Chord Detection (INSERTED) | 4/4 | Code complete — UX/usage review needed | - |
| 6.5 Note Generator + WASM Parity (INSERTED) | 3/4 | Deferred (engine done, not wired, WASM humanize/gen stubs) | - |
| 6.6 Default MIDI Selection (INSERTED) | 1/1 | Complete | 2026-02-02 |
| 6.7 Extended Scale Modes & Barry Harris (INSERTED) | 3/3 | Complete | 2026-02-02 |
| 6.8 CI Fix (INSERTED) | 1/1 | Complete | 2026-02-02 |
| 6.9 Repo Cleanup & Documentation (INSERTED) | 3/3 | Complete | 2026-02-02 |
| 6.10 Docs (INSERTED) | 2/2 | Complete | 2026-02-05 |
| 6.10.1 UI Modernization (INSERTED) | 9/9 | Complete (code done, verified) | 2026-04-04 |
| 6.11 Logo (INSERTED) | 0/? | Not started | - |
| 7. Performance Mode | 0/? | Not started | - |
| 8. Mic Input | 0/8 | Planning complete | - |
| 9. Vocoder | 0/? | Not started (requires audio synthesis) | - |
| 10. Guitar Input | ~95% | Code complete on guitar-input-clean branch | - |
| ~~11. Trackpad Beat Input~~ | — | DROPPED (no research, niche) | - |
| ~~12. Advanced Voice Leading~~ | — | CLOSED: already shipped (4 styles, motion control, suspensions) | 2026-01-30 |
| ~~13. Voice Leading Test Suite~~ | — | CLOSED: tests exist inline in voice leading code | - |
| 14. openDAW Device Integration | 0/? | Research complete | - |
| 15. Contrapunk Cloud | 0/? | Waitlist live, concept defined | - |
| 16. VST3/CLAP/AU Plugin | 4/7 | Plugin builds, AU works in Logic, webview GUI WIP | 2026-04-07 |
| 17. Integration Test Pipeline | 1/3 | Synthetic benchmarks done (100%), real audio fixtures pending | 2026-04-10 |
| 18. Basic-Pitch Polyphonic | 0/? | Research complete, tract-onnx path chosen | - |
| 19. NeuralNote Research | 0/? | Research complete, NeuralNote frame-by-frame pattern documented | - |
| 20. Release Engineering | ~80% | Workflows created, codename gen done, manual dispatch | 2026-04-10 |
| **21. Elixir Milestone (v1.5 / elixir-v0.1.0)** | **0/24** | **Queued (ingested 2026-05-18; behind v1.3.0)** | **-** |
| 21.A0 Workspace Bootstrap | 0/? | Queued | - |
| 21.A1 Bare Oscillator | 0/? | Queued | - |
| 21.A2 Polyphony + Voice Mgmt | 0/? | Queued | - |
| 21.A3 Modulation Matrix v1 | 0/? | Queued | - |
| 21.A4 Filter | 0/? | Queued | - |
| 21.A5 FX Bus MVP | 0/? | Queued | - |
| 21.A6 Spectral + FX Completion | 0/? | Queued | - |
| 21.A-Cut Full Parity Cutover | 0/? | Queued | - |
| 21.A7 Default Flip + Cleanup | 0/? | Queued | - |
| 21.B0 Standalone Skeleton | 0/? | Queued | - |
| 21.B1 Standalone First Sound | 0/? | Queued | - |
| 21.B2 Standalone Polyphony | 0/? | Queued | - |
| 21.B3 Plugin Skeleton | 0/? | Queued | - |
| 21.B4 Plugin Full Params | 0/? | Queued | - |
| 21.B5 Preset Save/Load | 0/? | Queued | - |
| 21.B6 Standalone UI v1 | 0/? | Queued | - |
| 21.B7 Wavetable Editor | 0/? | Queued | - |
| 21.B8 Headless Renderer | 0/? | Queued | - |
| 21.B9 Public v0.1.0 Release | 0/? | Queued | - |
| 21.C0 CLAP Activation | 0/? | Queued | - |
| 21.C1 Plugin GUI Embedding | 0/? | Queued | - |
| 21.C2 CLAP Param Automation + State | 0/? | Queued | - |
| 21.C3 VST3 Hosting | 0/? | Queued | - |
| 21.C4 AU Hosting | 0/? | Queued | - |

## Known Cross-Cutting Issues (from memory, not phase-specific)

- [ ] **Desktop MIDI device enumeration** — Tauri may not enumerate devices properly (needs verification, may already be fixed)
- [ ] **Fly.io redeployment** — latest changes from guitar-input-clean and main need redeployment to contrapunk.fly.dev
- [ ] **Desktop settings persistence** — likely fixed by Phase 6.6, needs verification on Tauri build

---
*Roadmap created: 2026-01-28*
*Last updated: 2026-05-18 — Added Phase 21: Elixir Milestone (v1.5 / elixir-v0.1.0) via `/gsd-ingest-docs` from ELIXIR-DESIGN.md + ELIXIR-PLAN.md. Three parallel tracks (A: replace built-in synth, B: standalone product, C: multi-plugin hosting). 24 new phase blocks across A0-A7+A-Cut, B0-B9, C0-C4. Queued behind v1.3.0 tag.*
