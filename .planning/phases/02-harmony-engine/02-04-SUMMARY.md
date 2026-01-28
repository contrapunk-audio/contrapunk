---
phase: 02-harmony-engine
plan: 04
subsystem: midi, harmony
tags: [wmidi, midi-routing, note-tracking, harmony-integration]

# Dependency graph
requires:
  - phase: 02-02
    provides: Stateless harmony modes (pass_through, diatonic_thirds/fourths, random_below)
  - phase: 02-03
    provides: Stateful harmony modes (contrary motion, counterpoint) and HarmonyEngine
  - phase: 01-02
    provides: MIDI router with OutputRouter and routing loop
provides:
  - Harmony-aware MIDI routing (original to output 0, harmony to output 1)
  - Note-On/Off tracking for random mode consistency
  - wmidi-based MIDI message parsing in router
affects: [03-gui, 02-05]

# Tech tracking
tech-stack:
  added: []
  patterns: [note-tracking-hashmap, midi-message-parsing]

key-files:
  created: []
  modified:
    - src/harmony/engine.rs
    - src/router.rs
    - src/midi/output.rs
    - src/main.rs

key-decisions:
  - "Track active notes in HashMap<u8, Vec<Note>> for Note-Off handling"
  - "Clear note tracking on key/mode change to prevent stale harmonies"
  - "Use send_to_port() for targeted output routing instead of broadcast"

patterns-established:
  - "Note tracking pattern: harmonize_note_on stores, harmonize_note_off retrieves"
  - "MIDI message routing: parse with wmidi, dispatch by message type"

# Metrics
duration: 3min
completed: 2026-01-28
---

# Phase 2 Plan 4: Harmony Router Integration Summary

**HarmonyEngine integrated into MIDI router with wmidi parsing, Note-On/Off tracking for random modes, and separate outputs for melody/harmony**

## Performance

- **Duration:** 3 min
- **Started:** 2026-01-28T15:37:18Z
- **Completed:** 2026-01-28T15:40:09Z
- **Tasks:** 2
- **Files modified:** 4

## Accomplishments

- Added Note-On/Off tracking to HarmonyEngine using HashMap for random mode consistency
- Integrated HarmonyEngine into router with wmidi message parsing
- Original notes route to output 0, harmony notes to output 1
- Non-note messages pass through unchanged

## Task Commits

Each task was committed atomically:

1. **Task 1: Add Note-On/Off tracking to HarmonyEngine** - `57ce998` (feat)
2. **Task 2: Update router to use HarmonyEngine** - `62a4d5e` (feat)

## Files Created/Modified

- `src/harmony/engine.rs` - Added active_notes HashMap, harmonize_note_on/off methods, 6 tracking tests
- `src/router.rs` - Complete rewrite with HarmonyEngine integration, wmidi parsing, note routing
- `src/midi/output.rs` - Added send_to_port() method for targeted output
- `src/main.rs` - Updated to pass HarmonyEngine reference to router

## Decisions Made

1. **HashMap<u8, Vec<Note>> for note tracking** - Key is melody MIDI number, value is harmony notes. Simple and O(1) lookup for Note-Off.

2. **Clear tracking on key/mode change** - When user changes key or mode, tracked harmonies would be invalid. Clearing prevents stuck notes with wrong harmonies.

3. **send_to_port() instead of broadcast** - More explicit routing than send_to_all(). Output 0 always gets melody, output 1 gets harmony (when available).

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Harmony routing fully functional with all 7 modes
- Ready for Phase 2 Plan 5: User configuration interface refinements
- Ready for Phase 3: GUI development (engine.set_key/set_mode ready for UI binding)

---
*Phase: 02-harmony-engine*
*Completed: 2026-01-28*
