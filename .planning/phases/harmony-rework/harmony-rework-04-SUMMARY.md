---
phase: harmony-rework
plan: 04
subsystem: harmony, ui
tags: [scales, music-theory, taxonomy, pentatonic, blues, symmetric, wasm, serde]

# Dependency graph
requires:
  - phase: harmony-rework-01
    provides: honest renaming of existing 28 scales
provides:
  - 57 total ScaleMode variants (28 existing + 29 new)
  - 10 ScaleFamily variants with proper taxonomy (was 5)
  - intervals() returns &'static [u8] (zero heap allocation)
  - Property-based tests for all 57 scales
  - All surfaces updated (WASM, UI, Tauri, plugin, docs)
affects: [harmony-rework-05, ui-scale-picker, preset-system]

# Tech tracking
tech-stack:
  added: []
  patterns: [static-slice-return, property-based-testing, family-taxonomy]

key-files:
  modified:
    - src/harmony/config.rs
    - src/harmony/scale.rs
    - src/harmony/engine.rs
    - src/harmony/key_detect.rs
    - wasm/src/lib.rs
    - ui/src/lib/stores/engine.svelte.ts
    - docs/ENGINE_DEEP_DIVE.md

key-decisions:
  - "Dissolved Exotic family into DoubleHarmonic (2 moved) + World (3 moved + 2 new)"
  - "Changed intervals() from Vec<u8> to &'static [u8] while touching all match arms"
  - "Skipped BebopMajor, BebopMinor, RomanianMinor (duplicates of existing scales)"
  - "Tauri parse_scale_mode needs no update (serde + Debug fallback handles new variants)"
  - "Plugin has no PluginScaleMode enum, so no changes needed"

patterns-established:
  - "Static slice return: intervals() returns &'static [u8] not Vec<u8>"
  - "10-family taxonomy: Diatonic, HarmonicMinor, MelodicMinor, HarmonicMajor, DoubleHarmonic, Pentatonic, Blues, Symmetric, World, BarryHarris"
  - "Property tests: all scales validated for invariants (ascending, unique, correct family)"

requirements-completed: [scale-expansion, family-taxonomy, performance-optimization, property-tests, surface-updates]

# Metrics
duration: 25min
completed: 2026-04-12
---

# Plan 04: Langoy Scale Expansion Summary

**29 new scales across 5 new families (Pentatonic, Blues, Symmetric, Harmonic Major, World) with zero-alloc intervals() and property-based tests**

## Performance

- **Duration:** 25 min
- **Tasks:** 8 (enum variants, intervals, family, display, all, surfaces, tests, docs)
- **Files modified:** 7

## Accomplishments
- 57 total ScaleMode variants: pentatonic (8), blues/bebop (3), symmetric (4), harmonic major (7), double harmonic modes (5), world (2)
- 10 ScaleFamily taxonomy replacing 5 families (Exotic dissolved into proper categories)
- intervals() returns &'static [u8] -- zero heap allocation per call
- 10 property-based tests: invariants, uniqueness, family round-trip, modal relations, spot checks
- All surfaces updated: WASM bridge, TypeScript types/constants, ENGINE_DEEP_DIVE.md

## Task Commits

1. **All tasks:** `7f9da31` (feat: expand scales to 57 with 10-family taxonomy across all surfaces)

## Files Created/Modified
- `src/harmony/config.rs` - 29 new ScaleMode variants, 5 new ScaleFamily variants, &'static [u8] intervals, 10 property tests
- `src/harmony/scale.rs` - .to_vec() for offsets field, updated test count assertion
- `src/harmony/engine.rs` - Fixed iterator for &'static [u8] return type
- `src/harmony/key_detect.rs` - Removed needless borrow on intervals
- `wasm/src/lib.rs` - 29 new entries in parse_scale_mode and scale_mode_to_string
- `ui/src/lib/stores/engine.svelte.ts` - ScaleFamilyName (10), ScaleModeName (57), SCALE_FAMILIES (10 groups), SCALE_INTERVALS (57 entries)
- `docs/ENGINE_DEEP_DIVE.md` - Updated scale catalog and variant listing

## Decisions Made
- Dissolved Exotic family: DoubleHarmonic and HungarianMinor moved to DoubleHarmonic family, Enigmatic/Neapolitan moved to World
- Persian and HungarianMajor added to World family (not DoubleHarmonic, despite similar IC vectors -- different parent scales)
- Fixed pre-existing clippy error in voicer_bach.rs (|| true assert)
- No serde version bump needed: existing validation falls back to Ionian for unknown variants

## Deviations from Plan

### Auto-fixed Issues

**1. Pre-existing clippy error in voicer_bach.rs**
- **Found during:** Pre-commit hook
- **Issue:** `voicing[1] >= voicing[2] || true` flagged as logic bug by clippy --all-targets
- **Fix:** Replaced with comment (inner crossing already intentionally allowed)
- **Verification:** clippy passes, all tests pass

---

**Total deviations:** 1 auto-fixed (pre-existing issue)
**Impact on plan:** Unrelated to scale expansion. Required for pre-commit hook to pass.

## Issues Encountered
- Pre-commit hook was already broken before this plan (clippy --all-targets fails due to pre-existing `|| true` assertion). Fixed as part of this work.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- 57 scales ready for harmony engine use
- UI scale picker may benefit from search/filter for 57 entries (cosmetic, not blocking)
- All downstream phases can reference new scale families

---
*Phase: harmony-rework*
*Plan: 04*
*Completed: 2026-04-12*
