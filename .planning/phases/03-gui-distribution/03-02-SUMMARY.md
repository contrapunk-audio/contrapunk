---
phase: 03-gui-distribution
plan: 02
subsystem: ui
tags: [eframe, egui, rust, gui, midi, controls, threading]

# Dependency graph
requires:
  - phase: 03-gui-distribution/01
    provides: eframe/egui foundation, ContrapunkApp, AppState
  - phase: 02-harmony-engine
    provides: Key, HarmonyMode enums, HarmonyEngine
provides:
  - SidePanel with MIDI device selection (input + 8 output slots)
  - ComboBox controls for Key and HarmonyMode selection
  - Start/Stop button for MIDI routing
  - Background router thread with shared state (GUIRouterState)
  - spawn_gui_router() function for GUI-mode routing
  - Live note activity display during routing
affects: [03-03, 03-04, 03-05, 03-06]

# Tech tracking
tech-stack:
  added: []
  patterns: [Arc<Mutex<T>> for GUI-thread communication, background MIDI routing thread]

key-files:
  created: []
  modified: [src/app.rs, src/router.rs]

key-decisions:
  - "Use output_slots Vec<Option<usize>> for 8 configurable output ports"
  - "Background router thread with Arc<Mutex<GUIRouterState>> for note state sharing"
  - "cfg(feature = gui) guards separate GUI and CLI router code paths"
  - "Router thread receives initial key/mode (dynamic config changes require restart)"

patterns-established:
  - "GUI controls in SidePanel, status display in CentralPanel"
  - "spawn_gui_router pattern for background MIDI processing"
  - "ctx.request_repaint() to refresh GUI on MIDI events"

# Metrics
duration: 5min
completed: 2026-01-28
---

# Phase 03 Plan 02: MIDI Control Panel Summary

**SidePanel device/key/mode controls with spawnable background router thread and live note activity display**

## Performance

- **Duration:** 5 min
- **Started:** 2026-01-28T17:07:41Z
- **Completed:** 2026-01-28T17:12:50Z
- **Tasks:** 3
- **Files modified:** 2

## Accomplishments
- User can select MIDI input device from dropdown in GUI
- User can select up to 8 MIDI output ports via individual slot dropdowns
- User can select musical key (C through B) via dropdown
- User can select harmony mode (1-7) via dropdown with descriptions
- MIDI routing starts when user clicks Start button
- Status display shows ACTIVE/STOPPED, current key, mode, and connected devices
- Live note count display shows input and harmony notes during routing

## Task Commits

Each task was committed atomically:

1. **Task 1: Extend AppState with device lists and add controls UI** - `71bd41e` (feat)
2. **Task 2: Refactor router for GUI integration with shared state** - `72ec8f2` (feat)
3. **Task 3: Display current configuration in CentralPanel header** - (included in Task 1 and 2)

## Files Created/Modified
- `src/app.rs` - ContrapunkApp with SidePanel controls, CentralPanel status display, router integration (410 lines)
- `src/router.rs` - GUIRouterState struct, spawn_gui_router function, GUI-mode MIDI processing (450 lines)

## Decisions Made
- Used output_slots as Vec<Option<usize>> with 8 fixed slots for flexible multi-output configuration
- GUIRouterState uses Arc<Mutex<T>> pattern for safe GUI-thread communication
- Background router thread receives initial key/mode at spawn time; changing config requires stop/restart
- Separate #[cfg] guards for GUI and CLI router functions to avoid code duplication issues
- Note activity display shows counts only when notes are active (clean UI when idle)

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered
- Initial import error for `io` module in GUI mode due to conditional compilation - resolved by adding #[cfg] guards to CLI-only functions (run_router, process_midi_message, handle_note_on, handle_note_off)

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness
- GUI controls complete, ready for piano roll visualization (03-03)
- Router state includes input_notes and harmony_notes for visualization
- AppState.available_inputs/outputs populated on startup and via Refresh Devices button

---
*Phase: 03-gui-distribution*
*Completed: 2026-01-28*
