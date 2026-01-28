# Phase 5: Octave Variations - Research

**Researched:** 2026-01-29
**Domain:** MIDI note octave transformation in Rust (wmidi crate)
**Confidence:** HIGH

## Summary

This phase is **already substantially implemented** in the existing codebase. The `OctaveMode` enum, `apply_octave_mode()` engine method, GUI dropdown, CLI menu, server protocol support, and tests all exist. The implementation covers OCT-01 (Spread) and OCT-02 (Bass/Treble Split) fully.

However, **OCT-03 (Mirror Octaves) has a semantic gap**: the requirement states "harmonies duplicate across multiple octaves simultaneously" (implying additional notes are generated), but the current implementation merely shifts alternating voices +/- 1 octave without duplicating. This is the only remaining work.

**Primary recommendation:** Decide whether OCT-03 "Mirror" should truly duplicate notes (increasing voice count) or if the current shift-based behavior is acceptable. If duplication is desired, `apply_octave_mode` must produce additional notes rather than just transforming existing ones.

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| wmidi | (already in use) | Note type, MIDI range (0-127), `try_from(u8)` | Already the project's MIDI note representation |

No new libraries needed. This phase is pure logic on existing `Note` / `u8` types.

## Architecture Patterns

### Existing Architecture (Already Implemented)

The octave variation system follows a **post-processing transform** pattern:

```
Input Note -> HarmonyEngine.harmonize() -> mode generates harmony notes -> apply_octave_mode() transforms pitches -> output
```

Key design decisions already made:
- Melody (index 0) is NEVER modified by octave transforms
- Octave mode is applied AFTER harmony generation
- Changing octave mode clears `active_notes` tracking (correct for Note-Off consistency)
- `OctaveMode` is stored in `HarmonyEngine` alongside `HarmonyMode`

### Pattern: Saturating MIDI Arithmetic
```rust
// Already used in engine.rs - clamp to valid MIDI range
let shifted = midi.saturating_add(12).min(127);  // up one octave
let shifted = midi.saturating_sub(12);            // down one octave (saturates at 0)
// Convert back, keeping original if out of range
if let Ok(new_note) = Note::try_from(shifted) {
    *note = new_note;
}
```

### Anti-Patterns to Avoid
- **Modifying melody note:** Octave transforms must skip index 0
- **Forgetting to clear active_notes:** When octave mode changes, tracked Note-On/Off pairs become invalid

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| MIDI range clamping | Manual bounds checking | `saturating_add/sub` + `.min(127)` + `Note::try_from` | Already proven in codebase |
| Octave mode UI | Custom widget | egui `ComboBox` with `OctaveMode::all()` | Already implemented |

**Key insight:** This phase requires almost no new code. The main question is whether Mirror mode's semantics match the requirement.

## Common Pitfalls

### Pitfall 1: Mirror Mode Semantic Mismatch
**What goes wrong:** OCT-03 says "duplicate across multiple octaves simultaneously" but current code just shifts alternating voices +/- 1 octave (no duplication)
**Why it happens:** "Mirror" was interpreted as "spread symmetrically" rather than "add duplicate notes in other octaves"
**How to avoid:** Clarify requirement. If true duplication is needed, `apply_octave_mode` must return a longer `Vec<Note>` for Mirror mode, and the Note-On/Off tracking must account for the extra notes.
**Warning signs:** Mirror mode produces the same number of output notes as other modes instead of more.

### Pitfall 2: Note-Off Mismatch with Duplicated Notes
**What goes wrong:** If Mirror mode adds extra notes, `harmonize_note_on` stores them in `active_notes`, but `harmonize_note_off` must release ALL of them
**Why it happens:** The current tracking stores `result[1..]` which works if octave mode only transforms (same count). If Mirror duplicates (more notes), the tracking still works because it stores the final result.
**How to avoid:** The current `harmonize_note_on` already stores the post-transform result, so duplication is safe as long as `apply_octave_mode` is called before storing.

### Pitfall 3: Output Port Count vs Note Count
**What goes wrong:** With duplication, Mirror mode may produce more notes than available output ports
**Why it happens:** Router sends `notes[i]` to `port[i]`, extra notes beyond port count are silently dropped
**How to avoid:** Either send all Mirror duplicates to the same port as the original harmony, or document that Mirror requires more output ports.

## Code Examples

### Current apply_octave_mode (already in engine.rs)
```rust
// Source: src/harmony/engine.rs lines 215-260
fn apply_octave_mode(&self, notes: &mut Vec<Note>) {
    if notes.len() <= 1 || self.octave_mode == OctaveMode::None {
        return;
    }
    let melody = notes[0];
    let melody_midi = u8::from(melody);
    for (i, note) in notes.iter_mut().enumerate().skip(1) {
        let midi = u8::from(*note);
        let shifted = match self.octave_mode {
            OctaveMode::Spread => {
                let octaves_up = i as u8;
                midi.saturating_add(octaves_up * 12).min(127)
            }
            OctaveMode::BassTrebleSplit => {
                if midi < melody_midi {
                    midi.saturating_sub(12)
                } else {
                    midi.saturating_add(12).min(127)
                }
            }
            OctaveMode::Mirror => {
                if i % 2 == 1 { midi.saturating_add(12).min(127) }
                else { midi.saturating_sub(12) }
            }
            OctaveMode::None => midi,
        };
        if let Ok(new_note) = Note::try_from(shifted) {
            *note = new_note;
        }
    }
}
```

### If Mirror Needs True Duplication (proposed approach)
```rust
// Would need to change signature or push additional notes
// For each harmony note, add both +1 and -1 octave copies
OctaveMode::Mirror => {
    let original_harmonies: Vec<Note> = notes[1..].to_vec();
    for harm in &original_harmonies {
        let midi = u8::from(*harm);
        if let Ok(up) = Note::try_from(midi.saturating_add(12).min(127)) {
            notes.push(up);
        }
        if midi >= 12 {
            if let Ok(down) = Note::try_from(midi - 12) {
                notes.push(down);
            }
        }
    }
}
```

## State of the Art

| Aspect | Status | Notes |
|--------|--------|-------|
| OctaveMode enum | DONE | All 4 variants defined in config.rs |
| apply_octave_mode | DONE | Transforms harmony pitches post-generation |
| GUI dropdown | DONE | ComboBox in app.rs side panel |
| GUI status display | DONE | Shows active octave mode in status bar |
| CLI selection | DONE | select_octave_mode() in main.rs |
| Server protocol | DONE | octave_mode field in Configure message |
| Tests | DONE | 4 octave mode tests in engine.rs |
| OCT-01 (Spread) | DONE | Each voice +i octaves |
| OCT-02 (BassTrebleSplit) | DONE | Below melody -1oct, above +1oct |
| OCT-03 (Mirror) | PARTIAL | Shifts alternately, does not truly duplicate |

## Open Questions

1. **Does OCT-03 "Mirror Octaves" require true note duplication?**
   - What we know: Current implementation shifts alternating voices +/- 1 octave. The requirement says "duplicate across multiple octaves."
   - What's unclear: Whether "duplicate" means additional notes or just repositioning.
   - Recommendation: If the user wants true duplication (each harmony note also plays in +1 and -1 octaves), this is the only code change needed. If current behavior is acceptable, this phase is already complete.

2. **Output port routing for duplicated notes**
   - What we know: Router maps notes[i] to port[i]. Extra notes beyond port count are dropped.
   - What's unclear: Should Mirror duplicates go to the same port as the original harmony, or require additional ports?
   - Recommendation: Send duplicates to the same port as the original harmony note (simplest, no port count change needed).

## Sources

### Primary (HIGH confidence)
- Source code: `src/harmony/config.rs` - OctaveMode enum definition
- Source code: `src/harmony/engine.rs` - apply_octave_mode implementation and tests
- Source code: `src/app.rs` - GUI octave mode dropdown and status display
- Source code: `src/main.rs` - CLI octave mode selection

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - no new dependencies, pure logic on existing types
- Architecture: HIGH - already implemented, pattern is clear from source code
- Pitfalls: HIGH - identified from reading actual implementation vs requirements

**Research date:** 2026-01-29
**Valid until:** 2026-03-01 (stable, no external dependencies involved)
