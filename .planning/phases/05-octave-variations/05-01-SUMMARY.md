---
phase: 05-octave-variations
plan: 01
subsystem: harmony-engine
tags: [mirror, octave, duplication, port-routing]
dependency_graph:
  requires: [04-server-mode]
  provides: [mirror-octave-duplication, port-aware-routing]
  affects: [05-02]
tech_stack:
  added: []
  patterns: [port-map-routing, note-duplication]
key_files:
  created: []
  modified:
    - src/harmony/engine.rs
    - src/harmony/config.rs
    - src/router.rs
decisions:
  - id: "05-01-01"
    description: "Port map stored alongside active notes for Note-Off restoration"
  - id: "05-01-02"
    description: "Server session unchanged - port routing is client-side concern"
metrics:
  duration: "3 min"
  completed: "2026-01-29"
---

# Phase 5 Plan 1: Mirror Octaves Duplication Summary

**Mirror mode now triples each harmony note (original + octave up + octave down), with all duplicates routing to the same output port as the source harmony voice.**

## What Was Done

### Task 1: Implement Mirror duplication in engine with port assignments
- Rewrote `apply_octave_mode` Mirror branch to perform true note duplication instead of alternating shifts
- Added `last_port_map: Vec<usize>` field to `HarmonyEngine` tracking output port for each note
- Added `active_port_maps: HashMap<u8, Vec<usize>>` for Note-Off port map restoration
- Added `pub fn last_port_map(&self) -> &[usize]` getter
- Mirror duplicates map back to original harmony voice's port index (e.g., E5 copy -> same port as E4 original)
- All clear methods updated to also clear `active_port_maps`
- Updated config.rs doc comment for Mirror mode
- Added 7 new tests: tripling, port map assignments, out-of-range handling, note-off release, port map restoration, identity map for non-mirror

### Task 2: Update router to use port-aware mirror routing
- Updated all 4 routing functions (GUI note-on/off, CLI note-on/off) to use `engine.last_port_map()` for port selection
- Fallback: if port_map index missing, uses identity mapping (backward compatible)
- Debug output now shows port assignments per note
- Server session left unchanged: it sends MIDI over TCP, port routing is client-side

## Decisions Made

| ID | Decision | Rationale |
|----|----------|-----------|
| 05-01-01 | Store port map in `active_port_maps` HashMap alongside `active_notes` | Simpler than changing `active_notes` type; clean separation of concerns |
| 05-01-02 | Server session unchanged | Server sends MIDI data over TCP; client handles local port routing. No protocol change needed. |

## Deviations from Plan

### Scoped Adjustments

**1. Server session not modified (plan Task 2 item 4)**
- Plan suggested updating session.rs for port-aware routing
- Server sends individual MIDI messages over TCP; it has no local output ports
- Port routing is a client-side concern handled by the existing client round-robin
- No protocol change required for current functionality

## Verification Results

1. `cargo test` -- 69 tests pass (7 new mirror tests)
2. `cargo build` -- CLI compiles
3. `cargo build --features gui` -- GUI compiles
4. Mirror branch creates duplicate notes (confirmed by test assertions)
5. Router uses `last_port_map()` in all 4 routing functions

## Commits

| Task | Commit | Message |
|------|--------|---------|
| 1 | df1e714 | feat(05-01): implement Mirror duplication in engine with port assignments |
| 2 | 162013e | feat(05-01): update router to use port-aware mirror routing |

## Next Phase Readiness

Mirror duplication is complete and tested. Ready for next plan in Phase 5.
No blockers or concerns.
