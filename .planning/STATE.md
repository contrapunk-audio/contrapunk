# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-01-28)

**Core value:** Real-time harmony generation with minimal latency
**Current focus:** Phase 06.10.1 — UI Modernization (SvelteKit + HLD Design System)

## Current Position

Phase: 06.10.1 of 13 (UI Modernization)
Plan: 7 of 9 in current phase (01, 02, 03, 04, 05, 06, 07 complete)
Status: In progress
Last activity: 2026-02-25 - Completed 06.10.1-07-PLAN.md (HLD atmospheric effects: particles, glow, beat indicator, FX toggle)

Progress: [███████████████░░░░░] (06.10.1 plan 7/9 complete)

## Performance Metrics

**Velocity:**
- Total plans completed: 36
- Average duration: 3.3 min
- Total execution time: 119.5 min

**By Phase:**

| Phase | Plans | Total | Avg/Plan |
|-------|-------|-------|----------|
| 01-midi-foundation | 3 | 6 min | 2 min |
| 02-harmony-engine | 6 | 18 min | 3 min |
| 03-gui-distribution | 6 | 15 min | 2.5 min |
| 04-server-mode | 4 | 12 min | 3 min |
| 05-octave-variations | 1 | 3 min | 3 min |
| 05.1-wasm-browser | 2 | 11 min | 5.5 min |
| 06.5-note-generator | 4 | 16 min | 4 min |
| 06.8-ci-fix | 1 | 3 min | 3 min |
| 06.9-repo-cleanup | 2 | 2.5 min | 1.3 min |
| 06.10-docs | 2 | 12 min | 6 min |
| 06.10.1-ui-modernization | 8 | 43 min | 5.4 min |

*Updated after each plan completion*

## Accumulated Context

### Decisions

Decisions are logged in PROJECT.md Key Decisions table.
Recent decisions affecting current work:

- [Init]: Port to Rust for performance and single-binary distribution
- [Init]: Use egui/eframe for native GUI
- [Init]: Drop audio-to-MIDI to reduce complexity
- [01-01]: Use midir crate for cross-platform MIDI I/O
- [01-01]: Port lists as (index, name) tuples for display flexibility
- [01-01]: Validate multi-output selection with min/max bounds
- [01-02]: Use recv_timeout pattern for non-blocking message loop
- [01-02]: Spawn background thread for Enter-key detection
- [01-02]: OutputRouter stores port indices for debug output
- [01-03]: Hardware verification confirms MIDI foundation is solid
- [02-01]: Use wmidi crate for Note type with step() method
- [02-01]: Major scale offsets stored as semitones from tonic
- [02-01]: Diatonic transposition calculates octave shift for cross-octave intervals
- [02-02]: Mode functions take (note, scale) and return Vec<Note> for uniform interface
- [02-02]: Out-of-range harmonies fail gracefully by returning only original note
- [02-03]: Contrary motion first note uses third below as default harmony
- [02-03]: Counterpoint prefers thirds/sixths over fourths/fifths for consonance
- [02-03]: State resets on both key change and mode change
- [02-05]: Default key is C (index 0) for most common use case
- [02-05]: Default mode is Pass-through (1) for safe no-change starting point
- [02-05]: Voice roles labeled in summary (melody for first output, harmony for rest)
- [02-04]: Track active notes in HashMap<u8, Vec<Note>> for Note-Off handling
- [02-04]: Clear note tracking on key/mode change to prevent stale harmonies
- [02-04]: Use send_to_port() for targeted output routing
- [02-enh]: Use VecDeque for sliding window history (O(1) push/pop at both ends)
- [02-enh]: Chained harmonies: each voice pair gets independent CounterpointState
- [02-enh]: Out-of-key: chromatic intervals (3rds, 6ths, 4ths, 5ths) preferring scale landing
- [03-01]: Use eframe::egui re-export for egui types
- [03-01]: Feature gate CLI code with #[cfg(not(feature = "gui"))]
- [03-01]: AppState contains all harmony and MIDI state for future use
- [03-02]: Use output_slots Vec<Option<usize>> for 8 configurable output ports
- [03-02]: Background router thread with Arc<Mutex<GUIRouterState>> for note state sharing
- [03-02]: cfg(feature = gui) guards separate GUI and CLI router code paths
- [03-02]: Router thread receives initial key/mode (dynamic config changes require restart)
- [03-03]: Input notes track melody only, harmony notes track generated harmonies (skip index 0)
- [03-03]: midi_to_name uses standard MIDI convention: 60 = C4
- [03-03]: Notes displayed sorted by pitch for visual consistency
- [03-05]: Chord patterns ordered by specificity (7ths before triads) for correct matching
- [03-05]: Pitch class reduction (mod 12) for octave-independent chord detection
- [03-05]: Unknown chord combinations show individual note names
- [04-03]: clap Args struct unconditional (both builds parse args); only client mode is CLI-gated
- [04-03]: AtomicUsize voice counter for lock-free round-robin output routing in client
- [04-04]: Read timeouts handled as non-fatal in server/client loops
- [05-01]: Port map stored alongside active notes for Note-Off port restoration
- [05-01]: Server session unchanged for mirror routing (port routing is client-side)
- [05.1-01]: index.html at project root (Trunk expects Cargo.toml alongside HTML)
- [05.1-01]: WASM stubs return error messages; full Web MIDI deferred to Plan 02
- [05.1-01]: Entire midi module gated (not individual files) since all depend on midir
- [05.1-02]: HarmonyEngine owned directly by ContrapunkApp in WASM (no thread, no Arc<Mutex>)
- [05.1-02]: Frame-based polling drains Rc<RefCell<Vec<Vec<u8>>>> each update() frame
- [05.1-02]: Output device IDs stored as Vec<String> for Web MIDI routing
- [06.1-01]: Melody (index 0) bypasses humanizer; only harmony notes (index 1+) humanized
- [06.1-01]: Beat clock starts on first running frame, stops on stop()
- [06.3-01]: Apply theme once via bool guard, not every frame
- [06.3-01]: Tab enum with Play/Craft/Settings; tab bar renders but does not change content yet
- [06.3-03]: Tab content as impl ContrapunkApp methods in separate files
- [06.3-03]: Deferred start/stop via pending flags consumed in update()
- [06.3-03]: Piano keyboard in BottomPanel always visible across all tabs
- [06.3-04]: Gear cogs at frame corners instead of plain circles for steampunk feel
- [06.5-01]: HeldNotes and Chord modes both play all selected notes (chord tones pre-resolved by UI)
- [06.5-01]: Beat wrap-around detected by beat_pos < last_beat_position
- [06.5-02]: Virtual inputs use sentinel values (usize::MAX for Note Generator, usize::MAX-1 for Computer Keyboard)
- [06.5-02]: QWERTY piano: Z-M maps C3-B3, Q-U maps C4-C5 (standard DAW layout)
- [06.5-02]: Virtual inputs skip physical MIDI connection in both native and WASM
- [06.5-03]: Magenta highlight for generator-selected piano keys (distinct from input/harmony colors)
- [06.5-03]: Chord resolution in octave 4 (C4=MIDI 60)
- [06.4-01]: harmonize_smart takes &mut self to track last_borrowed_from for UI
- [06.4-01]: Borrowing range 1-5 maps to progressively more parallel modes
- [06.4-01]: All mode functions take &mut Scale for consistency with interchange
- [06.4-02]: Smart match priority prefers longer patterns then root-position chords over inversions
- [06.4-03]: Borrowed note amber on piano when interchange enabled and harmony sounding
- [06.4-03]: Subtle gold scale tinting on in-scale keys (warm wash white, lighter black)
- [06.4-03]: Roman numeral chord analysis in display (e.g., "Fmaj7 (IVmaj7 in C)")
- [bugfix]: ScaleFamily re-exported from harmony module (GUI build fix)
- [bugfix]: last_borrowed_from cleared on in-key notes (stuck interchange display fix)
- [bugfix]: borrowing_sources expanded with harmonic/melodic minor at all ranges, Phrygian at range 3+
- [bugfix]: GUIRouterState now tracks borrowed_notes and last_borrowed_from for native UI
- [06.8-01]: Use Swatinem/rust-cache@v2 instead of manual actions/cache for Rust deps
- [06.8-01]: Cache Trunk binary with version-keyed key (trunk-OS-v0.21)
- [06.8-01]: Three parallel CI jobs: check, test, wasm-check
- [06.9-01]: Unified CI/CD in single ci.yml with deploy gated behind all checks, main-only
- [06.10-01]: Use ignore attribute for complex examples requiring full setup
- [06.10-01]: Document VecDeque sliding windows in module overview for discoverability
- [06.10-01]: Add counterpoint scoring table to module docs for GitHub readability
- [06.10-02]: Include harmony algorithm deep dive in CONTRIBUTING.md per user request
- [06.10-02]: Document VecDeque sliding windows for counterpoint/contrary motion
- [06.10-02]: Use ignore attribute for doc examples (private modules in binary crate)
- [06.10.1-01]: Gate midi_defaults behind cfg(all(gui, wasm32)) since it depends on app module
- [06.10.1-01]: Gate piano behind cfg(gui) since it depends on eframe::egui
- [06.10.1-01]: Use Mutex for all AppState fields (simple correctness over performance)
- [06.10.1-01]: Router thread spawned per start_routing call with Arc-shared note state
- [06.10.1-01]: Note-update events emitted at ~30fps from router thread
- [06.10.1-02]: Used @sveltejs/vite-plugin-svelte v5.x for Vite 6 compatibility
- [06.10.1-02]: Press Start 2P via Google Fonts CDN, body background anti-flash in app.html
- [06.10.1-02]: CSS custom properties for all HLD colors, TypeScript constants mirror CSS tokens
- [06.10.1-02]: Pixel art CSS: image-rendering pixelated, font-smoothing none, .reduced-motion toggle
- [06.10.1-03]: Externalize contrapunk-wasm in Vite config (wasm-pack output not available at build time)
- [06.10.1-03]: Optimistic state updates with rollback on error in engine store
- [06.10.1-03]: WASM Engine wrapper tracks note state internally for get_note_state() polling
- [06.10.1-03]: String-based API in WASM bridge (parse_key/parse_mode) for JS interop simplicity
- [06.10.1-04]: Piano keyboard 90px height, black keys absolute-positioned from white key index
- [06.10.1-04]: Scale selector uses expandable family accordion (Church expanded by default)
- [06.10.1-04]: Ableton layout uses CSS Grid 3-row (auto 1fr auto), content area 3-column (1fr 1.4fr 1fr)
- [06.10.1-04]: Note labels only on active keys; card/pixel-btn/toggle-btn patterns for control sections
- [06.10.1-06]: HLD health-bar sliders: transparent range input layered over cyan-gradient CSS fill with magenta square thumb
- [06.10.1-06]: Humanize sub-toggles linked to master enable; generator modes match all 7 from Rust enum
- [06.10.1-06]: Right column order: ActiveNotes (compact, top) > HumanizePanel (sliders) > GeneratorPanel (modes)
- [06.10.1-05]: Virtual input sentinel values use Number.MAX_SAFE_INTEGER matching Rust app.rs usize::MAX pattern
- [06.10.1-05]: Preset delete uses inline toggle confirmation rather than browser confirm() dialog
- [06.10.1-05]: Svelte 5 event modifiers use inline e.stopPropagation() (pipe syntax removed in Svelte 5)
- [06.10.1-07]: Canvas particle system instead of @tsparticles/svelte for zero-dependency 30-particle ambient animation
- [06.10.1-07]: FX toggle persists to localStorage under 'contrapunk-fx' key; restored on mount
- [06.10.1-07]: BeatIndicator in StatusBar with 4-pip display (magenta downbeats, cyan others)
- [06.10.1-07]: GlowEffects uses display:contents; CSS classes (.glow-magenta, .glow-cyan, .glow-teal) usable standalone

### Pending Todos

- Stuck MIDI notes when changing settings mid-play (voice leading, key, mode, etc.) — active_notes tracking cleared without sending Note-Offs. Needs a proper fix that doesn't introduce new stuck notes.

### Recent Fixes

- [2026-02-04] Theme is PICO-8 retro pixel art (green accents), not steampunk gold/copper — user preference

### Blockers/Concerns

None.

### Quick Tasks Completed

| # | Description | Date | Commit | Directory |
|---|-------------|------|--------|-----------|
| 001 | Embed Press Start 2P pixel font and configure typography | 2026-01-30 | pending | [001-nice-fonts-typography](./quick/001-nice-fonts-typography/) |

### Roadmap Evolution

- Phase 4 added: Server Mode - Network server for remote MIDI harmony processing
- Phase 5.1 inserted after Phase 5: WASM and In-Browser Support with Fly.io deployment (URGENT)
- Phase 6.1 inserted as Humanization UI Fix (URGENT) — previous 6.1 (Voice Leading) moved to 6.2, previous 6.2 (Style Update) moved to 6.3
- Phase 6.4 inserted after Phase 6.3: Modal Harmony & Chord Detection — all church modes + harmonic/melodic minor, modal interchange with visual feedback, comprehensive chord detection (extended, slash, add chords)
- Phase 6.5 inserted after Phase 6.4: Note Generator — virtual MIDI input source with note/chord selection and beat-synced patterns (arpeggiator, scale runner, random), feeds into harmony engine
- Phase 6.6 inserted after Phase 6.5: Default MIDI Selection — persist MIDI device choices across sessions using eframe::Storage (native file + WASM localStorage)
- Phase 6.7 inserted after Phase 6.6: Extended Scale Modes & Barry Harris — all harmonic/melodic minor modes, exotic scales, Barry Harris 6th diminished scales and movement rules
- Phase 6.8 inserted after Phase 6.7: CI Fix (URGENT)
- Phase 6.9 inserted after Phase 6.8: Repo Cleanup & Documentation — remove Python scripts, clean README, comprehensive docs
- Phase 6.10 inserted after Phase 6.9: Docs (URGENT)
- Phase 10 added: Trackpad Beat Input — use trackpad as MIDI beat pad
- Phase 11 added: Advanced Voice Leading — comprehensive jazz voicings, Neo-Riemannian, negative harmony, motion control, resolution-aware processing
- Phase 12 added: Voice Leading Test Suite — automated tests for parallel motion, style differentiation, voice crossing, suspensions, integration with harmony modes

## Phase 1 Completion Summary

**MIDI Foundation - COMPLETE**

All success criteria verified:
1. User can select a MIDI input device from a list of available ports
2. User can select 2-8 MIDI output ports from available ports
3. User can play a note on input device and hear it on the first output port
4. Application runs without GUI (CLI mode) for testing MIDI flow

Hardware tested: Akai MPK Mini -> 4 IAC Driver buses (macOS)

## Phase 2 Completion Summary

**Harmony Engine - COMPLETE**

All success criteria verified:
1. User can select musical key (C through B) and it affects harmony output
2. User can switch between all 7 harmony modes and hear different results
3. User can change key and mode while playing without stopping or restarting
4. Mode 1 passes notes through unchanged
5. Modes 2-7 produce audibly different harmonies following their algorithms

Hardware tested: Akai MPK Mini -> IAC Driver buses (macOS)
All 7 modes verified working with no stuck notes.

## Phase 3 Completion Summary

**GUI and Distribution - COMPLETE**

All success criteria verified (6/6):
1. Application opens as a native window (egui/eframe)
2. GUI displays current configuration, active notes
3. All settings changeable via GUI controls
4. Single binary (2.9 MB) with no external dependencies
5. Full 88-key piano keyboard with color-coded notes
6. Chord detection displays detected chord name

Human-verified: approved.

## Phase 4 Completion Summary

**Server Mode - COMPLETE**

All success criteria verified (5/5):
1. Application can start in server mode, listening on a configurable network port
2. Remote clients can connect and send MIDI input to the server
3. Server processes MIDI through harmony engine and returns harmonized output to clients
4. Multiple clients can connect simultaneously (server handles concurrent sessions)
5. Clients receive harmonized MIDI output routable to their local output devices

Hardware tested: Akai MPK Mini -> Server (port 9900) -> Client -> IAC Driver buses (macOS)
Timeout bug fixed during verification: read timeouts now non-fatal.

## Phase 5 Completion Summary

**Octave Variations - COMPLETE**

All success criteria verified (5/5):
1. User can enable "Octave Spread" mode
2. User can enable "Bass/Treble Split" mode
3. User can enable "Mirror Octaves" with true note duplication (3x harmony notes)
4. Octave variations combine with any existing harmony mode
5. GUI displays which octave variation is active

Mirror Octaves tripling implemented with port-aware routing via last_port_map().
69 tests pass including 8 mirror-specific tests.

## Phase 5.1 Completion Summary

**WASM and In-Browser Support - COMPLETE**

All success criteria verified (5/5):
1. Application compiles to WebAssembly and runs in a modern browser
2. Web MIDI API provides input/output device access in-browser
3. GUI renders correctly via egui's WASM backend
4. Application is deployed and accessible via Fly.io (https://contrapunk.fly.dev/)
5. Browser users can use harmony engine with acceptable latency

Human-verified: "it works well in the browser"

## Phase 6.5 Outcome

**Note Generator - DEFERRED**

Human verification failed:
1. Note Generator feature is non-functional
2. Voice leading modes not distinct enough between styles
3. User chose to defer note generation work

Feedback: "voice leading isn't unique enough between different voice leading modes" and "note generator just doesn't work"

## Phase 6.3 Completion Summary

**Style Update - COMPLETE**

Delivered:
1. Steampunk dark theme with gold/copper/amber palette
2. Tabbed navigation (Play/Craft/Settings) with always-visible piano
3. 11 built-in musical style presets with character personas
4. Custom steampunk widgets (ornate sliders, decorative frame, gear cogs)
5. Preset persistence (JSON save/load, create/delete custom presets)
6. Ambient animations (rotating gears, decorative frame, music-reactive visuals)
7. Press Start 2P pixel font, three-column layout (post-plan rehaul)

Scoped out: Dark/light toggle (dark-only), 3D harmonic visualizer (deferred)
Human-verified: approved

## Phase 6.10 Completion Summary

**Docs - COMPLETE**

Plan 01 (Harmony API Documentation):
1. Comprehensive rustdoc for harmony module with 8-mode table and 28-scale families
2. Chord/note evaluation algorithm fully documented with processing pipeline
3. VecDeque sliding window algorithms documented (interval_history, melody_contour)
4. Counterpoint scoring system table in module docs
5. All public types have examples and cross-references

Plan 02 (Humanize Docs + CONTRIBUTING.md):
1. Humanize module fully documented (mod.rs, config.rs, engine.rs, beat_clock.rs)
2. CONTRIBUTING.md created with Development Setup, Architecture Overview
3. Harmony Algorithm Deep Dive section with windowing explanation
4. Data flow diagrams and module table for developer onboarding

All 180 tests pass, `cargo doc --all-features` completes without warnings.
Verified: 13/13 must-haves passed.

## Session Continuity

Last session: 2026-02-25
Stopped at: Completed 06.10.1-07-PLAN.md (HLD atmospheric effects: particles, glow, beat indicator, FX toggle)
Resume file: None
Next: 06.10.1-08-PLAN.md
