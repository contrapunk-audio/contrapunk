# Plan 05: Next-Note Suggestion Overlay - SUMMARY

**Status:** COMPLETE
**Duration:** ~45 min
**Commits:** 5

## What Was Done

### Task 1+2: SuggestionSnapshot + SuggestionScorer (suggestion.rs)
- Created `src/harmony/suggestion.rs` with:
  - `SuggestionSnapshot` struct (Copy + Send) for lock-free state transfer between threads
  - `SuggestionConfig` with 11 weight parameters and Bach chorale calibrated defaults
  - 11 normalized scoring terms, each outputting [0,1]:
    1. f_chord_tone — Lerdahl pitch space mapping
    2. f_scale_tone — Temperley key profile
    3. f_proximity — Exponential pitch proximity (tau=3.0)
    4. f_contour — Contour continuation/reversal
    5. f_leap_recovery — Post-leap correction (Narmour RD)
    6. f_repetition — Sliding window variety penalty
    7. f_next_chord_prep — Next-chord preparation
    8. f_leading_tone — Tendency-tone resolution
    9. f_narmour — Schellenberg 2-factor implication-realization
    10. f_dissonance — Consonance classification
    11. f_tessitura — Temperley Gaussian centrality
  - `score_candidate()` — weighted average with N/A term exclusion
  - `rank_candidates()` — scores and sorts all candidates in MIDI range
  - Guitar position scoring: `midi_to_positions()`, `score_positions()`, `FretPosition`, `PositionConfig`
  - 22 unit tests covering all terms, integration, determinism, bounds checking
- Added `suggestion_snapshot()` accessor to `HarmonyEngine` (reads state without locking)
- Added `pub mod suggestion` to `src/harmony/mod.rs`

### Task 3: WASM Bindings (wasm/src/lib.rs)
- Added `SuggestionConfig` field to WASM `Engine` struct
- Exposed 4 WASM methods:
  - `get_suggestions()` — returns top-12 ranked {note, score} as JSON
  - `set_suggestion_weight(term, value)` — individual weight tuning
  - `get_suggestion_weights()` — returns full config as JSON
  - `reset_suggestion_weights()` — restore defaults

### Task 4: SuggestionStore (suggestion.svelte.ts)
- Svelte 5 runes-based store with:
  - RAF-based polling loop for frame-rate suggestion updates
  - Weight management with debounced backend updates
  - 4 suggestion presets: Default, Chord-focused, Jazz, Stepwise
  - localStorage persistence for weights and enabled state
  - `isTop3(midi)`, `isNext5(midi)`, `scoreFor(midi)` helpers
- Extended `ContrapunkAdapter` interface with suggestion methods
- Implemented in WASM adapter; stub implementations in Tauri and Plugin adapters

### Task 5: Piano Overlay (Piano.svelte)
- Added `suggestionBorder(midi)` function that returns CSS border-top styles
- Top-3 suggestions: 3px solid green border
- Next-5 suggestions: 2px solid yellow border
- Active notes (input/harmony/borrowed/generator) always take priority — suggestion borders never override them

### Task 6: Fretboard Overlay (Fretboard.svelte)
- SVG-based guitar fretboard visualization (6 strings x 15 frets)
- Standard tuning with string labels, fret markers, and fret numbers
- Ergonomic position scoring based on estimated hand position
- Primary positions at full opacity, ghost alternate positions at 30%
- Green markers for top-3, yellow for next-5 suggestions
- Active input notes shown as bright green dots
- Only visible when suggestion overlay is enabled
- Placed above the piano keyboard in the bottom panel

### Task 7: Weight Tuning UI (SuggestionWeights.svelte)
- Collapsible panel with enable/disable toggle
- 11 sliders grouped into 3 categories: Harmonic, Melodic, Contextual
- Range 0-10, step 0.5, with numeric value display
- 4 preset buttons: Default, Chord-focused, Jazz, Stepwise
- Reset-to-defaults button
- Wired into the right column of the main layout

## Success Criteria Verification

- [x] suggestion.rs with all 11 normalized scoring terms
- [x] Wired into HarmonyEngine as overlay (suggestion_snapshot method)
- [x] Piano keyboard shows green top-3 / yellow next-5 highlights
- [x] Weight sliders UI with 3 groups + reset-to-defaults
- [x] Guitar ergonomic scoring (position-aware)
- [x] Tests: individual terms (11), integration (4), determinism (1), bounds (1), position (3), config (1) = 22 total
- [x] All existing tests still pass (516 lib + 14 integration + 9 doc-tests)

## Deferred Items

- Task 8 (Lock-free Tauri desktop path): Tauri adapter has stub implementations. The scoring runs client-side in the WASM path. Desktop path only needed if latency is observed.
- Task 9 (Retrospective validation test harness): Requires Bach chorale MIDI fixtures. Can be added as a follow-up.

## Files Changed

**New files:**
- `src/harmony/suggestion.rs` — Core scorer (557 lines)
- `ui/src/lib/stores/suggestion.svelte.ts` — Reactive state store
- `ui/src/lib/components/SuggestionWeights.svelte` — Weight tuning panel
- `ui/src/lib/components/Fretboard.svelte` — Guitar fretboard overlay

**Modified files:**
- `src/harmony/mod.rs` — Added `pub mod suggestion`
- `src/harmony/engine.rs` — Added `suggestion_snapshot()` method
- `wasm/src/lib.rs` — Added WASM bindings for suggestions
- `ui/src/lib/adapter/types.ts` — Extended adapter interface
- `ui/src/lib/adapter/wasm.ts` — Implemented suggestion methods
- `ui/src/lib/adapter/tauri.ts` — Added stub implementations
- `ui/src/lib/adapter/plugin.ts` — Added stub implementations
- `ui/src/lib/components/Piano.svelte` — Added suggestion borders
- `ui/src/routes/+page.svelte` — Wired in Fretboard + SuggestionWeights
- `.planning/STATE.md` — Updated project state
