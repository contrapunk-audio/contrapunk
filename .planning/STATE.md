---
gsd_state_version: 1.0
milestone: v1.5
milestone_name: / elixir-v0.1.0)
status: executing
stopped_at: Proceeding to complete Elixir implementation; first code target is Phase 21.A6 spectral/FX completion.
last_updated: "2026-05-18T17:21:30.965Z"
last_activity: 2026-05-18 — pi-gsd installed and project state reconciled. HANDOFF.json is treated as consumed after this update.
progress:
  total_phases: 54
  completed_phases: 17
  total_plans: 82
  completed_plans: 65
---

# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-05-18 — added Elixir Milestone Decisions subsection)

**Core value:** Real-time harmony generation with minimal latency
**Current focus:** Complete Elixir v1.5 / elixir-v0.1.0 — synth engine replacement + standalone product + plugin path.
**Previous Contrapunk release state:** v1.3.0 work is effectively wrapped; user has chosen to continue Elixir implementation now.

## Current Position

Phase: 06.4 of 23 (modal harmony chord detection)
Status: Executing Phase 06.4
Last activity: 2026-05-18 — pi-gsd installed and project state reconciled. HANDOFF.json is treated as consumed after this update.

### Phase 21 Resume — 2026-05-18

**Source consumed:** `.planning/HANDOFF.json` (now deleted as one-shot resume artifact) + git history (`71045556 wip: Phase 21 Elixir — paused at A5+B6 complete`).

**Completed from handoff:**

- **A0** — Workspace bootstrap + feature flag (`6a36f311`)
- **A1** — Bare oscillator + ADSR + sine wavetable + Catmull-Rom (`6a36f311`)
- **B0** — Standalone cpal demo binary (`6a36f311`)
- **B1** — midir live MIDI input (`6a36f311`)
- **A2** — 16-voice polyphony + voice stealing + sustain + kill ramp (`880cc852`)
- **A3** — Modulation matrix v1, LFO + routes + mod-of-mod (`ee0b587d`)
- **A4** — SVF lowpass filter + cutoff modulation (`3f28adbd`)
- **A5** — FX bus: Drive + Delay + Reverb + 4-slot chain (`52a34152`)
- **B6** — Standalone egui UI v1 (`39cbf546`)
- **B6.1/B6.2** — UI polish + visual rewrite (captured in `71045556`)
- **DESIGN-DOC** — ELIXIR-DESIGN.md expansion through policy/scope sections (captured in `71045556`)

**Next implementation objective:** Complete Elixir implementation. Default sequence:

1. **A6** — Spectral oscillator + FX completion: spectral morphs, phase distortion, unison, chorus/flanger/phaser/compressor, FDN-16 reverb, dirty/diode/formant/phaser filters.
2. **A-Cut** — Wire Elixir into Contrapunk's built-in synth path behind feature flag after A6 parity.
3. **B3/B4/B5/B7/B8/B9** — DAW plugin, full params, preset format, wavetable editor, headless renderer, public release.
4. **C0-C4** — Multi-plugin hosting in Contrapunk (CLAP, GUI embedding, automation/state, VST3, AU).

**Open human actions:**

- Reserve bundle IDs `com.contrapunk.elixir.standalone` and `com.contrapunk.elixir.plugin` before B9 signing.
- Keep Elixir FX self-contained for A6 (Option A); revisit extraction/shared-FX only after A-Cut.

### Elixir Milestone Ingest — 2026-05-18

**Source docs:** [ELIXIR-DESIGN.md](../ELIXIR-DESIGN.md), [ELIXIR-PLAN.md](../ELIXIR-PLAN.md)
**Mode:** merge-into-existing-from-ingest
**Status:** Executing Phase 06.4

**Track structure** (three parallel work streams sharing `Chain` / `AudioBlock` substrate):

- **Track A** (~13 weeks): replace Contrapunk's built-in synth. Phases 21.A0 → 21.A7 + 21.A-Cut (9 phases)
- **Track B** (~10 weeks parallel): public Elixir product (standalone + plugin + headless). Phases 21.B0 → B9 (10 phases)
- **Track C** (~9 weeks parallel): multi-plugin hosting in Contrapunk (CLAP + VST3 + AU). Phases 21.C0 → C4 (5 phases)

**Locked decisions (ELIX-DEC-0{1..6}):**

1. Workspace location: inside contrapunk workspace
2. Cutover ambition: full feature parity at A-Cut (post-A6, not post-A5)
3. Track B UI: egui, separate process/window from Contrapunk (Contrapunk's Svelte UI unaffected)
4. Track C v1 scope: all three plugin formats (CLAP + VST3 + AU)
5. `elixir-standalone` distribution: public, released from this repo under `elixir-v0.1.0` tag
6. GSD milestone integration: ingest as real milestone

### Recently Shipped — 2026-05-14 session

**Core features (v1.3.0):**

- **#11 HoldMode end-to-end** — enum + Companion scaffolding (`5aed550`), Rust core plumbing (`12fe5ec`), WASM/Tauri/JS bridge (`bfa6854`), master HoldMode picker (`32eac55`), 6-test regression suite (`725a270`), per-lane + per-voice overrides (`953fdd4`), Tauri pass-through tests (`738921d`), per-voice NoteOff fix (`1a484b8`), UI persistence + counterpoint lime recolor (`df50e0c`), C1+C2 orphan-NoteOff and dedup fix (`f580372`/`b065eb5`)
- **Tempo-synced delay FX** (`e7f59d3`) — sync toggle + 6 subdivisions (1/4, 1/8d, 1/8, 1/8t, 1/16, 1/16t); transport-anchored hard re-tap on BPM change
- **Per-lane piano colors on Tauri** (`73afe0e`) — canon = gold, counterpoint = lime; was WASM-only before today
- **Transport metronome BPM re-anchor** (`d5e04be`) — fixes silent metronome death on BPM change mid-play

**Plugin parity work:**

- **Plugin emits noteUpdate** (`130b63e`) — `PluginNoteState` shared between audio thread and editor `on_frame`; Piano + Fretboard light up in DAW
- **DAW-managed sections hidden** via `transportControl` / `midiDevicePicker` / `audioFx` / `companionLanes` adapter capabilities (`130b63e`, `54df700`) — plugin webview no longer shows inert transport bar, MIDI pickers, synth/reverb/delay, or Companion tab

**Brutal-critic punch list (`54df700`):**

- canon_lane `.expect` → `.unwrap_or(now)` (was trusting housekeeping invariant)
- Delay set_mix/set_feedback truncation → round (was silently flooring 0.0009 → 0)
- Subdivision::from_u8 eprintln on junk byte (was silent fallback to Quarter)
- with_transport built via struct-update syntax (was fake builder)
- type_id `&'static str` end-to-end (was allocating String per emitted op per tick)
- WasmAdapter destroy() cancels companionTickHandle + pollingHandle + _clockTimer + closes _audioCtx
- TauriAdapter startRouting + init drop pre-existing listener handles (was leaking on double-start)
- Knob disabled gates tabindex + aria-disabled
- guitarCapture.ts stripped 4 per-frame console.log calls (1.7×/sec in prod)
- guitar.rs deleted dead-code calibration write to CWD (real save_calibration_profile uses app_data_dir)
- presets.rs::load_preset raises panic_pending (commit b065eb5; regression test in `54df700`)

**Tests + housekeeping:**

- New: `load_preset_raises_panic_pending`, `load_preset_unknown_name_does_not_raise_panic`, `test_drain_all_tracked_notes_survives_poisoned_lock` (first poison-recovery test in project)
- 30 canon_lane tests pass (was 28; +2 C1+C2 regression tests)
- 29 Tauri command tests pass (was 22; +3 above + 4 from earlier b065eb5)
- Untracked `ui/node_modules` (11,443 files) and `ui/build` (66 files) from git tracking; added to `.gitignore` (`8af66ff`)

### Phase 2 issues from v1.2.x

| # | Status |
|---|---|
| #81 Krumhansl auto-key (3 slices) | ✅ Shipped (slice 3 in `f1ab6ba`) |
| #65 Presets UI redesign | ✅ Shipped (`3b516bb`); intentionally unmounted on main page per PERFORMANCE-VIEW.md design decision |
| #66 Canonical embed wave 3+4 | ✅ Shipped (`4ec21c9`) — PianoEmbed + ChordReadout in tree; Piano.svelte wrapper-swap descoped |
| #11 HoldMode (Companion + persistence) | ✅ Shipped this session |
| #8b Per-voice phase offset | Shipped + reverted with no body in `bb6c691` (humanize scrap); SKIPPED for v1.3, re-apply candidate for v1.4 |
| #9 Plugin spike | ✅ Shipped (VST3/CLAP scaffolding; Companion lanes deferred to v1.4) |

### Deferred to v1.4 / later

- **Plugin Companion lanes** — 2.5-3 days work; canon/counterpoint emission inside DAW. UI is already hidden via `companionLanes: false` capability so the visible-lie is solved.
- **Plugin audio FX wiring** — synth/reverb/delay params in nih-plug; currently hidden via `audioFx: false` capability.
- **Bonus: no `load_calibration_profile`** — the Tauri `save_calibration_profile` writes a struct that no consumer reads. Two unrelated `GuitarCalibrationProfile` types in the tree. Filed for triage.
- **AudioWorklet fallback removal** — guitarCapture.ts already uses AudioWorklet as primary; ScriptProcessor is a vestigial fallback that could be deleted now that Safari ≥14.1 / Chrome ≥66 / Firefox ≥76 ship AudioWorklet.

### Queued — Elixir Milestone (v1.5 / elixir-v0.1.0)

See full structure in ROADMAP.md "Phase 21: Elixir Milestone" section.

**Quick reference:**

- 24 phase blocks (A0-A7+A-Cut, B0-B9, C0-C4) running in parallel as three tracks
- 24 REQ-elixir-* requirements (Track A: 9, Track B: 10, Track C: 7, cross-cutting: 3 — 4 reqs span both Track C and cross-cutting buckets)
- 6 LOCKED decisions in PROJECT.md Key Decisions table
- Source docs: `ELIXIR-DESIGN.md` (precedence 0), `ELIXIR-PLAN.md` (precedence 1, LOCKED)
- Sync intel under `.planning/intel/` (decisions.md, requirements.md, constraints.md, context.md, SYNTHESIS.md)

## Performance Metrics

**Velocity:**

- Total plans completed: 37
- Average duration: 3.4 min
- Total execution time: 126.5 min

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
| 06.10.1-ui-modernization | 10 | 55 min | 5.5 min |

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
- [06.10.1-08]: wasm-pack output to ui/src/lib/wasm-pkg/ under $lib for native SvelteKit resolution
- [06.10.1-08]: build-wasm.sh creates dev stub when wasm-pack unavailable, enabling builds without Rust toolchain
- [06.10.1-08]: Docker build from repo root with flyctl -c flag; nginx on port 8080
- [06.10.1-08]: Gate old egui WASM modules behind feature='wasm' for contrapunk-wasm crate isolation
- [06.10.1-09]: Keep server/client modes in main.rs binary (useful for network MIDI)
- [06.10.1-09]: Remove all egui GUI router code from router.rs, keep CLI router for potential future use
- [06.10.1-09]: Remove WASM-specific deps from root Cargo.toml (contrapunk-wasm crate has its own)
- [ELIX-DEC-01]: Workspace location: inside contrapunk workspace (LOCKED)
- [ELIX-DEC-02]: Cutover ambition: full feature parity at A-Cut, post-A6 not post-A5 (LOCKED)
- [ELIX-DEC-03]: Track B UI: egui, separate process/window from Contrapunk (LOCKED)
- [ELIX-DEC-04]: Track C v1 scope: all three plugin formats (CLAP + VST3 + AU) (LOCKED)
- [ELIX-DEC-05]: elixir-standalone distribution: public, released from this repo (LOCKED)
- [ELIX-DEC-06]: GSD milestone integration: ingest as real milestone (LOCKED — completed 2026-05-18)

### Prioritized Backlog (as of 2026-04-15)

User direction: fix everything that needs fixing EXCEPT pitch detection and guitar, then improve input/output routing, then metronome/beatmaker and humanization.

**P0 — Close the gaps in already-shipped work**

1. **Finish Audio Foundation Task 11** — manual end-to-end smoke test (play → MIDI out AND audio out through cpal) + tag `milestone/audio-foundation`. Blocks plugin hosting sub-project 2.
2. **Stuck MIDI notes on settings change** — root cause: engine's `active_notes.clear()` in every setter (12+ call sites in `src/harmony/engine.rs`) drops Note-Off tracking. Candidate fix: send MIDI CC 123 (All Notes Off) on every port when any engine setting changes, then clear. Also flush delay queue, also clear `active_humanization` in Humanizer.
3. **Wire Generator into WASM + Tauri** — `src/generator/` is 949 lines of Rust unwired to anything. Add bindings in `wasm/src/lib.rs` and `src-tauri/src/commands/`, remove the `{#if false}` gate in `GeneratorPanel.svelte`. Closes the other half of the WASM-parity memory bug.
4. **Tauri `setDetune` has no backend command** — UI stores `_detuneCents` locally, never calls `invoke()`. Add `set_detune` Tauri command.
5. **Guitar calibration file written to CWD** — use `tauri::path::app_data_dir()` in `src-tauri/src/commands/guitar.rs:244`; surface write errors. (Security/hygiene; does NOT require reopening guitar input work.)
6. **`stopTickLoop` never called → RAF leak** — add `destroy()` on `WasmAdapter`, call from SvelteKit root layout `onDestroy`.
7. **Mutex unwrap cascade** in `src-tauri/src/commands/engine.rs:422-425,605-632` — replace `.unwrap()` with logged `unwrap_or_default()` to stop router thread from crashing silently on lock poison.
8. **Rigor-gate stash audit** — 19 `rigor-gate-*` stashes exist after the one recovery commit. Audit whether any still contain real work. Drop the rest.

**P1 — Input/Output Routing improvements (user direction: "make routing better")**

Current routing lives in two places: `src/router.rs` (CLI) and `src-tauri/src/commands/engine.rs` (Tauri runtime — the one users actually hit). They drifted during Audio Foundation.

1. **Unify voice_index vs port_index** — remove the `FIXME(sub-project-2)` substitution in `src/router.rs:134` and `src-tauri/src/commands/engine.rs:384`. Add `voice_index: u8` to `HumanizedNote` and populate from harmony voice index at `humanize_note_on`. GitHub issue #33.
2. **Route MIDI queue overflow gracefully** — `src/audio_out/midi_queue.rs` `push()` on full currently leads to `.unwrap()` panics at `router.rs:452,459`. Change to drop-oldest or log+skip.
3. **Real-time safety in audio callback** — `PolySynth::process_stereo` heap-allocates a scratch buffer per callback (`sine_synth.rs:192`). Pre-allocate a max-frames scratch at construction. Also kill `Mutex<AudioState>` on the hot path (give PolySynth ownership to the audio thread; MIDI events already flow lock-free via SPSC ringbuffer).
4. **Browser audio output (WASM)** — `WasmAdapter.startAudioOutput` is a no-op warning. Wire a minimal Web Audio AudioWorklet synth so browser users can hear harmony without hardware MIDI. Mirrors native PolySynth.
5. **Beat-update event from Rust to UI** — `TauriAdapter` currently approximates beats with `setInterval` (drifts from Rust `BeatClock`). Emit real `beat-update` Tauri event from router thread at each `beat_crossed()` boundary; remove `_beatInterval` approximation.
6. **AudioWorkletNode migration** — `ui/src/lib/audio/guitarCapture.ts` uses deprecated `ScriptProcessorNode`. Refactor to AudioWorklet feeding `Float32Array` blocks to WASM DSP. (This is routing INTO the engine, not guitar-detection logic — allowed under the "no guitar" exclusion since the change is plumbing.)
7. **Kill console.log spam** in `guitarCapture.ts:153-155,169,195,199` on the hot path.

**P2 — Metronome, beatmaker, humanization (user direction: "a lot better")**

Current state is minimal. `src/humanize/metronome.rs` is 35 lines: `enabled` bool + downbeat/offbeat click. `BeatClock::is_offbeat` is hardcoded to fractional 0.4-0.6. No subdivision control, no accent patterns, no groove templates.

1. **Metronome: accent model + subdivision clicks** — upgrade from a two-note (downbeat/offbeat) stub to configurable: accent pattern per beat, optional 8th/16th subdivision clicks with distinct notes, volume, sound selection (woodblock/cowbell/rim), count-in (1-2 bars).
2. **Humanize: richer swing model** — `is_offbeat` hardcoded 0.4-0.6 only catches 8th offbeats crudely. Move to per-subdivision swing (8ths AND 16ths), curve shape (linear / triplet / Logic-style), per-voice swing amount.
3. **Humanize: groove templates** — named presets ("straight", "swing light/heavy", "push", "pull", "drag", "rush") that set jitter + swing + velocity curves together.
4. **Humanize: per-voice variation** — current config applies the same humanization to all harmony voices. Add per-voice config so bass can be straighter than upper voices (real ensemble feel).
5. **Velocity dynamics curves** — current is uniform random ± variation. Add weighted curves (Gaussian, Pareto) and accent-aware variation (stronger on beats 1, 3).
6. **Tempo tap** — tap-tempo input to set BPM from the UI (common DAW feature, minimal effort in Svelte + Tauri event bridge).
7. **Metronome UI parity** — expose all the above in `HumanizePanel.svelte`; metronome currently has no dedicated UI panel.

**Q (Queued) — Elixir Milestone (v1.5 / elixir-v0.1.0)**

Ingested 2026-05-18. Behind v1.3.0 tag. See ROADMAP.md Phase 21.

Recommended start sequence per ELIXIR-PLAN.md §11 (once v1.3.0 ships):

- **Week 1:** A0 (workspace bootstrap, 1d), B0 (skeleton standalone, 2d), bundle-ID reservation + release-patch skill extension (1d each)
- **Week 2:** A1 (voice handler + wavetable osc + DAHDSR, 5d), C0 prep (2d reading `src/plugin_host/clap/block.rs` + `controller.rs` stubs to confirm 2-week C0 budget)

### Pending Todos (resolved in backlog above)

_Previous entry — stuck MIDI notes — now tracked as P0 item 2._

### Recent Fixes

- [2026-04-13] Harmony rework audit findings C1-C4, S2-S7, Q1-Q5 fixed (`2794e98`)
- [2026-04-13] WASM humanize + beat clock + metronome ported (`4f695f6`)
- [2026-04-13] MIDI input now routed through humanized_note_on/off (`7ff989f`)
- [2026-04-14] Windows desktop build infrastructure landed (PR #32)
- [2026-04-15] Audio Foundation Tasks 5-10 shipped end-to-end
- [2026-02-04] Theme is PICO-8 retro pixel art (green accents), not steampunk gold/copper — user preference

### Blockers/Concerns

- **Rigor-gate stashes (19 remaining)** — all prefixed `rigor-gate-*`, originating from feat/windows-build and main. Need an audit pass before next dev session to confirm nothing material is stashed. The one already recovered (stash@{4}) came in as `33a3621 wip: recovered stash@{4}`.

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
- **Phase 21 added: Elixir Milestone (v1.5 / elixir-v0.1.0)** [2026-05-18] — ingested ELIXIR-DESIGN.md + ELIXIR-PLAN.md via `/gsd-ingest-docs`. Three parallel tracks (A: replace built-in synth, B: standalone product, C: multi-plugin hosting). 24 phase blocks across A0-A7+A-Cut, B0-B9, C0-C4. Queued behind v1.3.0 tag.

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

Last session: 2026-05-18 — Resumed Phase 21 Elixir in pi after installing pi-gsd and reconciling stale state.
Stopped at: Proceeding to complete Elixir implementation; first code target is Phase 21.A6 spectral/FX completion.
Resume file: HANDOFF.json consumed by this state update.
Next: Implement A6 incrementally with tests, then A-Cut/B-track plugin and release work.

## A6 Closure — 2026-05-18 (this session)

**Phase 21.A6 complete (A6.1 → A6.8).** Next: A-Cut — wire Elixir into Contrapunk built-in synth path behind feature flag.

- **A6.4** — `FilterCoeffs` enum + `FilterModel::tick_prepared`. Engine pre-computes coefficients per block; the audio callback is now `tanf`-free for every voice. Slow `FilterModel::tick(&FilterParams)` is retained for tests and pinned to the prepared path by `prepared_path_matches_slow_path_for_all_kinds` (<1e-5 drift across 2048 samples for all five filter kinds).
- **A6.5** — `all_a6_modes_are_sample_rate_invariant` exercises every public oscillator control (12 morphs, 9 phase modes, 11 unison styles) at 44.1 kHz AND 48 kHz; locks finite output, peak ≤ 1.01, bounded DC (< 0.95 to allow legitimate PWM duty asymmetry).
- **A6.6** — elixir-standalone UI now exposes A6 controls end-to-end:
  - OSC card: spectral-morph dropdown + morph_amount knob, phase-distortion dropdown + phase_amount knob, unison style dropdown + voices/detune knobs.
  - FILTER card: filter-kind dropdown (DigitalSvf/Diode/Dirty/Formant/Phaser), drive + gain knobs, morph X/Y knobs that appear when Formant or Phaser is selected.
  - New `filter_drive` / `filter_gain` / `filter_morph` getters on `Engine`; `FilterKind` / `SpectralMorph` / `PhaseDistortionMode` / `UnisonStyle` now `Default`-derivable for snapshot construction.
- **A6.7** — Decision: **DEFER** `contrapunk-fx` extraction. Keep Elixir FX in `crates/elixir-core/src/fx/`; revisit after A-Cut. Triggers and rationale captured in `.planning/phases/21-elixir-milestone/A6-STATUS.md`.
- **A6.8** — Decision: ship A6 spectral morphs as documented scalar approximations. Full FFT/IFFT spectral wavetable infrastructure is queued for **B7 (Wavetable Editor)**. Honesty docs in `A6-STATUS.md` (per-morph implementation table + path to full parity).

**Validation:** `cargo test -p elixir-core --lib` = 85 passed. `cargo check -p elixir-core --target wasm32-unknown-unknown` = clean. `cargo build -p elixir-standalone` = clean.

**Housekeeping done this session:**

- Repo-root `context.md` (subagent reuse audit) moved to `.planning/intel/elixir/reuse-audit.md`.
- Root `rigor.yaml` added to `.gitignore` (local rigor-gate config, not project state).
- `HANDOFF.json` consumed; remains deleted as a one-shot resume artifact.

**Open human actions (non-blocking, carry forward):**

1. Reserve bundle IDs `com.contrapunk.elixir.standalone` and `com.contrapunk.elixir.plugin` before B9 signing.
2. Decide on backlog re-prioritization: continue Track A (A-Cut next) or open Track B/C in parallel.

## A-Cut Closure — 2026-05-18 (this session)

**Phase 21.A-Cut initial Tauri cutover complete behind feature flag.**

- `src-tauri/Cargo.toml` now defines `elixir-synth = ["contrapunk/elixir-synth"]`.
- `src-tauri/src/audio_clock.rs` selects the initial synth block by cfg:
  - default: legacy `Synth` with descriptor `builtin.synth`
  - `--features elixir-synth`: `ElixirSynthBlock` with descriptor `builtin.elixir-synth`
- `src/chain/elixir_block.rs` now drains the existing router→audio `SynthEvent` receiver, so Tauri routing code can keep sending note events unchanged.
- Existing `SynthParams` are bridged into Elixir each block: enabled, master gain, ADSR, filter cutoff/resonance, plus approximate waveform mapping into Elixir morph/phase controls. This prevents the existing synth UI controls from going inert under the feature flag.
- Added regression tests for queued `SynthEvent::NoteOn` and `SynthParams::enabled(false)` mute behavior.

**Validation:**

- `cargo check -p contrapunk` — clean (pre-existing warnings only)
- `cargo check -p contrapunk --features elixir-synth` — clean (pre-existing warnings only)
- `cargo check -p contrapunk-tauri` — clean (pre-existing warnings only)
- `cargo check -p contrapunk-tauri --features elixir-synth` — clean (pre-existing warnings only)
- `cargo test -p contrapunk --features elixir-synth chain::elixir_block --lib` — 7 passed

**Next:** B3 — create `elixir-plugin` skeleton using existing nih-plug plugin crate patterns, then B4 full params/preset surface.

## B3 Skeleton — 2026-05-18 (this session)

**Phase 21.B3 initial elixir-plugin skeleton created.**

- Added workspace member `crates/elixir-plugin`.
- New `elixir-plugin` crate exports CLAP + VST3 via nih-plug.
- Plugin surface is intentionally minimal for B3:
  - MIDI input (`MidiConfig::Basic`)
  - mono/stereo instrument output layouts (no audio input)
  - `elixir_core::Engine` renders audio
  - one Gain parameter as the placeholder DAW parameter surface
  - sample-accurate MIDI handling by iterating `buffer.iter_samples()` and applying events at their timing
- No editor/preset matrix yet; B4/B5 own full parameter and preset expansion.
- Captured reusable nih-plug skeleton pattern in `.claude/skills/contrapunk-nih-plug-skeleton/SKILL.md`.

**Validation:** `cargo check -p elixir-plugin` = clean.

**Next:** B4 — expose full Elixir oscillator/filter/FX/amp parameter surface to DAW automation, then B5 preset format.
