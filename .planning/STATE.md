# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-01-28)

**Core value:** Real-time harmony generation with minimal latency
**Current focus:** Phase 3 complete — next: Phase 4 (Server Mode)

## Current Position

Phase: 4 of 6 (Server Mode)
Plan: 2 of 4 in current phase
Status: In progress
Last activity: 2026-01-28 - Completed 04-02-PLAN.md

Progress: [█████████████████░░░░░░░] 71% (17 of 24 plans complete)

## Performance Metrics

**Velocity:**
- Total plans completed: 15
- Average duration: 2.6 min
- Total execution time: 39 min

**By Phase:**

| Phase | Plans | Total | Avg/Plan |
|-------|-------|-------|----------|
| 01-midi-foundation | 3 | 6 min | 2 min |
| 02-harmony-engine | 6 | 18 min | 3 min |
| 03-gui-distribution | 6 | 15 min | 2.5 min |

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

### Pending Todos

None.

### Blockers/Concerns

None.

### Roadmap Evolution

- Phase 4 added: Server Mode - Network server for remote MIDI harmony processing

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

## Session Continuity

Last session: 2026-01-28
Stopped at: Completed 04-02-PLAN.md
Resume file: None
Next: Execute 04-03-PLAN.md (integration tests and CLI wiring)
