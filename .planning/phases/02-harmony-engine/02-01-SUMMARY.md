---
phase: 02-harmony-engine
plan: 01
subsystem: harmony
tags: [wmidi, rand, scale, music-theory, diatonic]

# Dependency graph
requires:
  - phase: 01-midi-foundation
    provides: MIDI I/O with midir crate
provides:
  - Key enum with 12 musical keys
  - HarmonyMode enum with 7 harmony modes
  - Scale struct with diatonic transposition
affects: [02-02, 02-03, 02-04, 02-05]

# Tech tracking
tech-stack:
  added: [wmidi 4.0, rand 0.8]
  patterns: [diatonic transposition via scale degrees]

key-files:
  created:
    - src/harmony/mod.rs
    - src/harmony/config.rs
    - src/harmony/scale.rs
  modified:
    - Cargo.toml
    - src/main.rs

key-decisions:
  - "Use wmidi crate for Note type with step() method"
  - "Major scale offsets stored as semitones from tonic"
  - "Diatonic transposition calculates octave shift for cross-octave intervals"

patterns-established:
  - "Scale degree calculation: pitch_class relative to tonic mapped to offset array"
  - "Diatonic transposition: degree arithmetic with octave handling"

# Metrics
duration: 2min
completed: 2026-01-28
---

# Phase 2 Plan 1: Harmony Types Summary

**Core harmony types: Key enum (12 keys), HarmonyMode enum (7 modes), Scale struct with diatonic transposition**

## Performance

- **Duration:** 2 min
- **Started:** 2026-01-28T15:22:07Z
- **Completed:** 2026-01-28T15:24:22Z
- **Tasks:** 2
- **Files modified:** 5

## Accomplishments
- Key enum with all 12 musical keys (C through B) and semitones_from_c() method
- HarmonyMode enum with 7 harmony modes (PassThrough, DiatonicThirds, DiatonicFourths, RandomBelow, RandomBelowNoSeconds, ContraryMotion, StrictCounterpoint)
- Scale struct with degree_of() for identifying scale degrees and transpose_diatonic() for diatonic interval transposition
- 4 unit tests validating scale degree detection and diatonic transposition in C major and G major

## Task Commits

Each task was committed atomically:

1. **Task 1: Add wmidi and rand dependencies** - `32419f3` (chore)
2. **Task 2: Create harmony module with Key, HarmonyMode, Scale** - `691164f` (feat)

## Files Created/Modified
- `Cargo.toml` - Added wmidi 4.0 and rand 0.8 dependencies
- `src/main.rs` - Added `mod harmony;` declaration
- `src/harmony/mod.rs` - Module re-exports (Key, HarmonyMode, Scale)
- `src/harmony/config.rs` - Key enum (12 variants) and HarmonyMode enum (7 variants)
- `src/harmony/scale.rs` - Scale struct with degree_of(), transpose_diatonic(), snap_to_scale()

## Decisions Made
- Used wmidi crate for Note type (provides step() method and MIDI note constants like Note::C4)
- Major scale stored as semitone offsets [0, 2, 4, 5, 7, 9, 11] for interval calculations
- Diatonic transposition handles octave shifts for intervals crossing octave boundaries

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed useless comparison warning in transpose_diatonic()**
- **Found during:** Task 2 (harmony module creation)
- **Issue:** Comparison `new_midi > 127` for i8 type always false (i8 max is 127)
- **Fix:** Changed to `!(0..=127).contains(&new_midi)` for correct range check
- **Files modified:** src/harmony/scale.rs
- **Verification:** cargo check passes without unused_comparisons warning
- **Committed in:** 691164f (Task 2 commit)

---

**Total deviations:** 1 auto-fixed (1 bug)
**Impact on plan:** Minor code quality fix. No scope creep.

## Issues Encountered
None - plan executed as expected.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Key, HarmonyMode, and Scale types ready for harmony algorithm implementation
- Scale.transpose_diatonic() is the core operation for modes 2-7
- Next plan (02-02) will implement HarmonyEngine trait and individual mode processors

---
*Phase: 02-harmony-engine*
*Completed: 2026-01-28*
