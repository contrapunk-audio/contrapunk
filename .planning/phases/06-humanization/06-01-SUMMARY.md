---
phase: 06-humanization
plan: 01
subsystem: audio-processing
tags: [humanization, midi, velocity, jitter, swing, beat-clock, rand]

requires:
  - phase: 02-harmony-engine
    provides: wmidi types (Note, Channel, Velocity) used in HumanizedNote
provides:
  - HumanizeConfig struct with all humanization parameters
  - BeatClock for tempo-aware beat position tracking
  - Humanizer engine with humanize_note_on/off methods
  - HumanizedNote output type with delay, velocity, duration delta
affects: [06-02 scheduler integration, 06-03 GUI controls]

tech-stack:
  added: []
  patterns: [f64 millisecond time for WASM compatibility, active_humanization HashMap for Note-On/Off pairing]

key-files:
  created:
    - src/humanize/mod.rs
    - src/humanize/config.rs
    - src/humanize/engine.rs
    - src/humanize/beat_clock.rs
  modified:
    - src/main.rs
    - src/lib.rs

key-decisions:
  - "Use f64 milliseconds for all time (WASM-compatible, no Instant)"
  - "Velocity clamped 1..=127 via TryFrom, never 0"
  - "Note-Off inherits jitter from Note-On via active_humanization HashMap"
  - "Duration variation is extension-only (positive values)"
  - "Swing delay computed from BeatClock offbeat detection"

patterns-established:
  - "Humanization pairing: Note-On stores record, Note-Off retrieves and removes"
  - "Abstract time: caller provides now_ms, no platform-specific time APIs"

duration: 3min
completed: 2026-01-29
---

# Phase 6 Plan 1: Humanization Core Summary

**HumanizeConfig, BeatClock, and Humanizer engine for velocity variation, timing jitter, swing delay, and duration extension of harmony notes**

## Performance

- **Duration:** 3 min
- **Started:** 2026-01-29T00:00:00Z
- **Completed:** 2026-01-29T00:03:00Z
- **Tasks:** 2
- **Files modified:** 6

## Accomplishments
- HumanizeConfig with toggleable jitter, velocity, duration, and swing parameters
- BeatClock tracks beat position from absolute elapsed time with offbeat detection and beat crossing
- Humanizer transforms notes with velocity variation (clamped 1-127), jitter, swing, and duration delta
- Note-Off inherits humanization from corresponding Note-On via active_humanization map

## Task Commits

1. **Task 1: Create humanize module with config types and beat clock** - `3831904` (feat)
2. **Task 2: Create Humanizer engine with note transformation logic** - `81cefdc` (feat)

## Files Created/Modified
- `src/humanize/mod.rs` - Module declaration and re-exports
- `src/humanize/config.rs` - HumanizeConfig and HumanizedNote structs
- `src/humanize/engine.rs` - Humanizer with humanize_note_on/off methods
- `src/humanize/beat_clock.rs` - BeatClock with tick, is_offbeat, beat_crossed
- `src/main.rs` - Added `mod humanize`
- `src/lib.rs` - Added `mod humanize`

## Decisions Made
- Used `Velocity::try_from(clamped).unwrap_or(velocity)` instead of unsafe for velocity clamping
- Module added to both main.rs (native binary) and lib.rs (WASM entry) since project uses both

## Deviations from Plan
None - plan executed exactly as written.

## Issues Encountered
- `Velocity::from_u8_unchecked` does not exist in wmidi 4.0; used `TryFrom<u8>` with fallback instead.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Humanize module exports all types needed for scheduler integration (Plan 02)
- BeatClock ready for tick-based scheduling
- No blockers

---
*Phase: 06-humanization*
*Completed: 2026-01-29*
