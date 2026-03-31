# Phase 9: Scale Detection — Context

**Status:** Planned

## Feature: Real-Time Scale Detection from Played Notes

As you play notes, Contrapunk detects which scale(s) your notes belong to and displays matches in real-time.

### Mode 1: Passive Detection
- Track unique pitch classes from recent playing
- Compare against all 336 possible scales (28 modes × 12 keys)
- Rank by match percentage, display top 3-5 matches
- Reset button to clear and start fresh

### Mode 2: Active Feedback  
- User sets a target scale, notes light up green (in scale) or red (out of scale)
- Hit/miss counter and accuracy percentage
- Extends existing `inScaleNotes` piano highlighting

### Algorithm
For each of 336 scale/key combos: score = |played ∩ scale| / |played|, penalize extra notes. Sort, return top 5.

### Existing Infrastructure
- `SCALE_INTERVALS` map (all 28 modes) already in engine.svelte.ts
- `KEY_TO_PITCH_CLASS` map already exists
- `computeScaleNotes()` already exists for piano highlighting
- Needs: pitch class tracking, comparison logic, UI display
