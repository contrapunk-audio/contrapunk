---
phase: 03-gui-distribution
plan: 03
subsystem: ui
tags: [eframe, egui, rust, gui, midi, real-time, visualization]

# Dependency graph
requires:
  - phase: 03-gui-distribution/02
    provides: GUIRouterState with input_notes/harmony_notes, spawn_gui_router, ctx.request_repaint
provides:
  - Real-time active notes display with note names (e.g., C4, E4, G4)
  - Visual distinction between input (blue) and harmony (green) notes
  - midi_to_name() helper function for MIDI-to-note conversion
  - Notes sorted by pitch for consistent display order
affects: [03-04, 03-05, 03-06]

# Tech tracking
tech-stack:
  added: []
  patterns: [MIDI-to-name conversion, sorted HashSet display, color-coded labels]

key-files:
  created: []
  modified: [src/router.rs, src/app.rs]

key-decisions:
  - "Input notes track melody only, harmony notes track generated harmonies (skip index 0)"
  - "midi_to_name uses standard MIDI convention: 60 = C4"
  - "Notes displayed sorted by pitch (low to high) for visual consistency"
  - "Active Notes group always visible (shows '(none)' when empty)"

patterns-established:
  - "midi_to_name(midi: u8) -> String for note name conversion"
  - "get_router_notes() returns cloned HashSets for thread-safe display"

# Metrics
duration: 1min
completed: 2026-01-28
---

# Phase 03 Plan 03: Active Notes Display Summary

**Real-time note name display with color-coded input (blue) and harmony (green) notes**

## Performance

- **Duration:** 1 min
- **Started:** 2026-01-28T17:15:13Z
- **Completed:** 2026-01-28T17:16:36Z
- **Tasks:** 2
- **Files modified:** 2

## Accomplishments
- GUI displays currently active input notes in real-time (light blue)
- GUI displays currently active harmony notes in real-time (light green)
- Notes appear when pressed and disappear when released
- Display updates immediately without user interaction (via request_repaint)
- Notes shown as names (C4, D#5) not just counts
- Notes sorted by pitch for consistent visual ordering

## Task Commits

Each task was committed atomically:

1. **Task 1: Ensure router updates shared state on note events** - `8d73454` (feat)
   - Fixed router to properly separate input vs harmony notes
   - Harmony notes now skip index 0 (melody) to avoid double-counting
2. **Task 2: Display active notes in GUI** - `1767c5a` (feat)
   - Added midi_to_name() helper function
   - Added Active Notes group with color-coded display
   - Changed get_router_notes() to return HashSets for individual note access

## Files Created/Modified
- `src/router.rs` - Fixed harmony_notes population to skip melody (index 0)
- `src/app.rs` - Added midi_to_name() function, Active Notes display group, color-coded labels

## Decisions Made
- Input notes (HashSet) tracks what user plays - the melody notes
- Harmony notes (HashSet) tracks only generated harmonies (notes[1..])
- MIDI note 60 = C4 (standard convention)
- Notes always sorted ascending by pitch for visual consistency
- Active Notes group shown even when empty with "(none)" placeholder

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed harmony notes including melody note**
- **Found during:** Task 1 analysis
- **Issue:** Existing code added ALL notes to harmony_notes, including the melody (index 0)
- **Fix:** Changed to `notes.iter().skip(1)` to exclude melody from harmony set
- **Files modified:** src/router.rs
- **Commit:** 8d73454

## Issues Encountered
None - plan executed smoothly after identifying the existing bug.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness
- Active notes display complete, ready for piano roll visualization (03-04)
- midi_to_name() function available for reuse in other visualizations
- Router state properly separates input and harmony notes

---
*Phase: 03-gui-distribution*
*Completed: 2026-01-28*
