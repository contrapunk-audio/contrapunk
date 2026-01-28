# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-01-28)

**Core value:** Real-time harmony generation with minimal latency
**Current focus:** Phase 2 - Harmony Engine

## Current Position

Phase: 2 of 4 (Harmony Engine)
Plan: 1 of 5 in current phase
Status: In progress
Last activity: 2026-01-28 - Completed 02-01-PLAN.md (harmony types)

Progress: [###-------] 30% (4 of 13 plans complete)

## Performance Metrics

**Velocity:**
- Total plans completed: 4
- Average duration: 2 min
- Total execution time: 8 min

**By Phase:**

| Phase | Plans | Total | Avg/Plan |
|-------|-------|-------|----------|
| 01-midi-foundation | 3 | 6 min | 2 min |
| 02-harmony-engine | 1 | 2 min | 2 min |

**Recent Trend:**
- Last 5 plans: 01-01 (4 min), 01-02 (2 min), 01-03 (checkpoint), 02-01 (2 min)
- Trend: stable

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

### Pending Todos

None.

### Blockers/Concerns

None - Phase 2 in progress.

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

## Phase 2 Progress

**Harmony Engine - IN PROGRESS**

Plan 1 complete:
- Key enum (12 keys), HarmonyMode enum (7 modes), Scale struct with diatonic transposition

Next: Plan 02-02 (HarmonyEngine trait and mode processors)

## Session Continuity

Last session: 2026-01-28 15:24 UTC
Stopped at: Completed 02-01-PLAN.md
Resume file: None
