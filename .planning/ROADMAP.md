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
- [ ] **Phase 7: Performance Mode** - Beat-aware performance where Contrapunk accumulates played notes over bars and generates context-aware responses
- [ ] **Phase 8: Mic Input** - Audio capture with pitch detection for audio-to-MIDI conversion and raw audio passthrough
- [ ] **Phase 9: Vocoder** - Classic vocoder (carrier modulated by voice) and harmony vocoder (real-time vocal harmonization)
- [ ] **Phase 10: Guitar Input** - Audio input from guitar with pitch detection for monophonic and polyphonic note tracking
- [ ] **Phase 11: Trackpad Beat Input** - Use computer trackpad as a MIDI beat pad for triggering notes and drums

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
- [ ] 06.4-01-PLAN.md — ScaleMode enum, Scale::new() parameterization, modal interchange, engine wiring
- [ ] 06.4-02-PLAN.md — Expanded chord detection (40+ patterns, slash chords, roman numerals)
- [ ] 06.4-03-PLAN.md — GUI integration (scale dropdown, interchange controls, piano tinting, chord display)
- [ ] 06.4-04-PLAN.md — Human verification checkpoint

### Phase 6.5: Note Generator (INSERTED)
**Goal**: Provide a virtual MIDI input source that generates notes using the shared beat clock, selectable in the IN dropdown alongside physical MIDI devices. Users can pick individual notes, chords, or algorithmic patterns that feed into the harmony engine.
**Depends on**: Phase 6 (Humanization — beat clock), Phase 6.3 (Style Update — UI tabs/layout)
**Requirements**: GEN-01, GEN-02, GEN-03, GEN-04, GEN-05
**Success Criteria** (what must be TRUE):
  1. "Note Generator" appears as a selectable option in the IN device dropdown
  2. User can select specific notes (click piano keys or note names) to feed into the harmony engine
  3. User can select a chord (e.g., Cmaj, Am7) and all chord tones feed into the harmony engine
  4. Beat-synced generator modes work: arpeggiator (up/down/up-down), scale runner, random diatonic
  5. Generator notes flow through the full harmony engine pipeline (harmonized, voice-led, humanized)
  6. Generator can run alongside a physical MIDI input (both sources merge into the engine)
  7. Generator uses the existing BeatClock (shared BPM/time-signature with metronome)
  8. Works in both native and WASM builds
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
**Plans:** 9 plans

Plans:
- [x] 06.10.1-01-PLAN.md — Cargo workspace + Tauri v2 backend with commands and state
- [x] 06.10.1-02-PLAN.md — SvelteKit project scaffolding + HLD design system
- [ ] 06.10.1-03-PLAN.md — Platform adapter layer + Svelte 5 rune stores + WASM bridge
- [x] 06.10.1-04-PLAN.md — 88-key piano keyboard + control panel + Ableton layout
- [ ] 06.10.1-05-PLAN.md — MIDI device selection + preset management panels
- [ ] 06.10.1-06-PLAN.md — Humanization + generator + active notes panels
- [ ] 06.10.1-07-PLAN.md — HLD atmospheric effects (particles, glow, beat indicator)
- [ ] 06.10.1-08-PLAN.md — WASM build pipeline + Fly.io deployment + CI update
- [ ] 06.10.1-09-PLAN.md — Remove egui/CLI/Trunk + human verification

### Phase 7: Performance Mode
**Goal**: Beat-aware performance mode where Contrapunk accumulates played notes over bars (using BeatClock) and generates musically-contextual responses based on phrase-level state, rather than harmonizing note-by-note
**Depends on**: Phase 6 (Humanization — BeatClock), Phase 2 (Harmony Engine)
**Requirements**: PERF-01, PERF-02, PERF-03
**Research-heavy**: This phase requires deep research into:
  - Temporal MIDI buffering tied to BeatClock (accumulating notes across bars)
  - Phrase-level state management (what the user played in last N bars)
  - Generative algorithms that respond to musical context vs individual notes
  - How performance state interacts with existing harmony modes, voice leading, humanization
  - Real-time constraints on bar-aware processing
**Success Criteria** (what must be TRUE):
  1. Metronome runs in background and Contrapunk tracks bar boundaries
  2. System accumulates user-played notes across configurable bar windows (1-4 bars)
  3. Generated notes respond to the musical context of recent performance (not just current note)
  4. Performance mode produces musically distinct output from standard harmony modes
  5. Works with existing voice leading, humanization, and octave variation settings
  6. GUI displays performance state (current bar, accumulated context)
  7. Works in both native and WASM builds
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
**Goal**: Accept guitar audio input with pitch detection optimized for guitar frequency range and playing styles
**Depends on**: Phase 8 (Mic Input — shares audio capture infrastructure)
**Requirements**: GTR-01, GTR-02, GTR-03
**Success Criteria** (what must be TRUE):
  1. User can select audio input for guitar (direct-in or mic'd amp)
  2. Monophonic pitch detection tracks single-note lines with low latency (<30ms)
  3. Polyphonic detection identifies chords (at least triads) from strummed input
  4. Detected notes feed into harmony engine like any other MIDI input
  5. GUI displays detected guitar notes with string/fret visualization
  6. Works with electric guitar (direct input) and acoustic (via mic)
**Plans**: TBD

Plans:
- [ ] TBD (run /gsd:plan-phase 10 to break down)

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

## Progress

**Execution Order:**
Phases execute in numeric order: 1 -> 2 -> 3 -> 4 -> 5 -> 5.1 -> 6 -> 6.1 -> 6.2 -> 6.3 -> 6.4 -> 6.5 -> 6.6 -> 6.7 -> 6.8 -> 6.9 -> 6.10 -> 6.10.1 -> 7 -> 8 -> 9 -> 10 -> 11 -> 12 -> 13

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
| 6.4 Modal Harmony & Chord Detection (INSERTED) | 0/4 | Planning complete | - |
| 6.5 Note Generator (INSERTED) | 3/4 | Deferred (non-functional) | - |
| 6.6 Default MIDI Selection (INSERTED) | 1/1 | Complete | 2026-02-02 |
| 6.7 Extended Scale Modes & Barry Harris (INSERTED) | 3/3 | Complete | 2026-02-02 |
| 6.8 CI Fix (INSERTED) | 1/1 | Complete | 2026-02-02 |
| 6.9 Repo Cleanup & Documentation (INSERTED) | 3/3 | Complete | 2026-02-02 |
| 6.10 Docs (INSERTED) | 2/2 | Complete | 2026-02-05 |
| 6.10.1 UI Modernization (INSERTED) | 4/9 | In progress | - |
| 7. Performance Mode | 0/? | Not started | - |
| 8. Mic Input | 0/8 | Planning complete | - |
| 9. Vocoder | 0/? | Not started | - |
| 10. Guitar Input | 0/? | Not started | - |
| 11. Trackpad Beat Input | 0/? | Not started | - |
| 12. Advanced Voice Leading | 0/? | Not started | - |
| 13. Voice Leading Test Suite | 0/? | Not started | - |

---
*Roadmap created: 2026-01-28*
*Last updated: 2026-02-25 — Renumbered: Performance Mode is Phase 7, Mic Input pushed to Phase 8, all subsequent phases shifted +1*
