# UX Audit: Species + Counterpoint Controls

**Date:** 2026-05-11
**Trigger:** User support report — "I can only hear 1:1 and the others are not really there"
**Status:** Engine-side bug (silent fallback to Species 1) fixed in a parallel commit. This doc captures the UX failure modes that turned a one-line dispatch bug into a confused user.

## Findings

### 1. Species selector is gated behind a mode dropdown

**Evidence:** `ui/src/lib/components/ControlPanel.svelte:135` —
```svelte
{#if engine.mode === 'StrictCounterpoint'}
    <div class="card">
        <div class="row-2col-cells">
            <div class="cell">
                <span class="cell-label font-ui">Species</span>
```

The whole Species + Strictness card is conditional on `engine.mode === 'StrictCounterpoint'`. A user exploring the Mode dropdown (`ControlPanel.svelte:112-118`) has no signal that picking "Strict Counterpoint" unlocks an entire second control surface — the card simply does not exist in the DOM until then.

**User impact:** Discovery cliff. The species controls advertise the product's signature feature (Fux species counterpoint), but a new user landing on the default mode never sees them and has no idea they exist. Compounds the just-fixed engine bug because users who *do* find the dropdown have nothing else to fall back on — there is no second entry point.

**Fix sketch:** Either (a) surface a disabled-state stub of the Species card with a one-line hint "Switch Mode → Strict Counterpoint to enable", or (b) decorate the `StrictCounterpoint` option in the Mode dropdown with a small `+ species` badge so the dependency is visible at choice time.

### 2. PixelSelect does not surface option tooltips

**Evidence:** `ui/src/lib/components/PixelSelect.svelte:2` defines the option type as
```ts
type Option = { value: string; label: string };
```
and the option button itself (`PixelSelect.svelte:77-84`) renders only `{opt.label}` with no `title=` attribute, no aria-description, nothing:
```svelte
{#each options as opt}
    <button
        class="pixel-select-option font-ui"
        class:active={opt.value === value}
        onclick={() => select(opt.value)}
        type="button"
    >
        {opt.label}
    </button>
{/each}
```

Meanwhile `COUNTERPOINT_SPECIES` in `ui/src/lib/stores/engine.svelte.ts:147-177` declares a `tooltip` field on every option, and `COUNTERPOINT_STRICTNESS` (`engine.svelte.ts:179-194`) does the same. The call site in `ControlPanel.svelte:30` flattens away the tooltip before passing options in: `speciesOptions = COUNTERPOINT_SPECIES.map((sp) => ({ value: sp.name, label: sp.label }))`.

**User impact:** Every species/strictness tooltip we have written is dead data. A user staring at "Species 2 (2:1)" with no idea what 2:1 means cannot hover, long-press, or otherwise extract the explanation that was written for them. This is the single biggest reason a user defaults to "the one I recognise (1:1)" and then complains the others "are not really there" — they have no way to form an expectation of what the others should sound like.

**Fix sketch:** Extend `PixelSelect`'s `Option` type to `{ value: string; label: string; tooltip?: string }`, render `title={opt.tooltip}` on the option button (and ideally a `title` on the trigger reflecting the currently-selected option's tooltip). Stop the `.map()` strip at the call sites. ~10 line change across two files.

### 3. Species 2 tooltip text is misleading

**Evidence:** `ui/src/lib/stores/engine.svelte.ts:163` claims
```ts
tooltip: 'Two harmony notes per melody note; passing tones on weak beats.'
```

But the engine implementation in `crates/contrapunk-harmony/src/stateful.rs:937-965` (Species 2 path of `process_with_beat`) returns either one or two notes per *call*, not two per melody note:
- On strong beats: returns `self.process(scale, melody)` — one harmony note per melody note (line 941, 946).
- On weak beats: tries to produce a diatonic passing tone derived from `last_harmony` and returns `vec![melody, pt]` (line 960), or falls back to just `vec![melody]` (line 964) if no passing tone is available.

There is never a code path that emits "two harmony notes per melody note" the way the tooltip implies. The 2:1 ratio is across two melody notes (one consonance + one passing tone), not within one.

**User impact:** A user reading the tooltip (once finding #2 is fixed) expects chord-stack thickness to double on Species 2 vs. Species 1. What they actually hear is the same chord density with rhythmic decoration on the weak beat. The mismatch reads as "Species 2 is broken / not really there" — verbatim the support thread.

**Fix sketch:** Rewrite as `'2:1 motion — consonance on strong beats, diatonic passing tones on weak beats. Same density as 1:1, more horizontal movement.'` Apply the same lens to Species 3 (`engine.svelte.ts:169`, similar over-promise of "four harmony notes per melody note") and Species 4 (`engine.svelte.ts:175`).

### 4. Strictness labels lack musical context

**Evidence:** `ui/src/lib/stores/engine.svelte.ts:179-194`:
```ts
{ name: 'Relaxed', label: 'Relaxed',
  tooltip: 'Lighter penalties — more permissive harmonic choices.' },
{ name: 'Strict',  label: 'Strict',
  tooltip: 'Fux-aligned scoring — enforces species rules strictly.' }
```

"Relaxed" and "Strict" are pure adjectives — they tell a user *how* the dial behaves but not *what changes audibly*. The tooltips name a scoring concept ("penalties", "Fux-aligned") that a non-musicologist user will not connect to a sonic outcome. (And per finding #2, the tooltips are not rendered anyway.)

**User impact:** A user toggles Strict ↔ Relaxed, hears something subtle change, and cannot tell whether the change is meaningful or a placebo. With Species 2-4 silently broken (now fixed), this was the only knob that *did* respond, which set the expectation that "subtle = broken".

**Fix sketch:** Either (a) rename labels to be outcome-led: `'Permissive'` / `'Textbook'`, or (b) keep the names and reword tooltips with concrete examples: `'Strict: forbids parallel fifths & octaves, forces stepwise resolution.'` Pair with a `?` info pip next to the cell label.

### 5. No way to A/B audition species without re-playing

**Evidence:** There is no UI affordance in `ControlPanel.svelte` (lines 135-160, the species card) to capture-and-replay the last phrase under a different species. Changing the species via `onSpeciesChange` (line 73-75) → `engine.setCounterpointSpecies` (`engine.svelte.ts:897-907`) only updates state going forward; past notes are not re-rendered.

**User impact:** To compare Species 1 vs. Species 2 a user has to: play a phrase, listen, switch the dropdown, play the same phrase again from memory, listen, switch back, play it a third time. Each comparison loses the previous audio reference. This is exactly the workflow that produced the support thread — the user *did* switch dropdowns, *did* re-play, and concluded "the others are not really there" because they could not hold the two renditions side by side mentally.

**Fix sketch:** A "Last phrase: replay" button on the Species card that captures the last 4-8 seconds of melody input and re-runs it through the engine with whichever species is currently selected. Out-of-scope for a pure UX patch but worth filing — link to a separate phase.

### 6. Prior dependency on Transport was invisible to the user

**Evidence:** Pre-fix, `process_with_beat` (`crates/contrapunk-harmony/src/stateful.rs:935-936`) routed every species back to plain `self.process(scale, melody)` whenever `beat_phase` was `None`:
```rust
match (self.species, beat_phase) {
    (CounterpointSpecies::Species1, _) | (_, None) => self.process(scale, melody),
```
`beat_phase` is `None` whenever the transport is not running. The UI exposed no signal that the Species selector required transport playback to take effect.

**User impact:** This is the root cause of the support thread. Even users who found the dropdown and picked Species 2 heard 1:1 output because the transport was idle. Now resolved engine-side — keeping this finding here because the *UI* still gives no indication that any species control is transport-dependent, and a future regression in that contract would re-create the same silent failure.

**Fix sketch:** Add a subtle "needs transport" indicator (e.g. a tiny dimmed pip on the Species card label when transport is stopped). Cheap insurance against the bug class. Or: assert in the engine that Species 2-4 + `beat_phase = None` is a logic error rather than silently downgrading.

### 7. Cell labels lack target size and have no keyboard discovery

**Evidence:** `ui/src/lib/components/ControlPanel.svelte:302-307` — the `.cell-label` class is `font-size: var(--font-size-xs)` with no hover/focus styling; `PixelSelect.svelte:46-54` uses `role="listbox"` with `tabindex="-1"`, meaning the wrap itself is not in the natural tab order. The trigger button (`PixelSelect.svelte:55-64`) is focusable but there is no visible focus ring beyond the `.open` border-color swap on lines 119-124.

**User impact:** A keyboard user cannot Tab to the Species dropdown predictably, and a user scanning the UI by hover gets no "this label means something" feedback. Combined with finding #2 (no tooltips), the whole control surface reads as decorative.

**Fix sketch:** Add `tabindex="0"` to the wrap and ensure trigger receives `:focus-visible` outline. Render `cell-label` with a `title=` (or a `?` pip) that surfaces the cell's purpose.

## Recommended Fixes (priority order)

| Priority | Fix | Files | Effort |
|---|---|---|---|
| P0 | Render option tooltips in `PixelSelect` (finding #2) | `PixelSelect.svelte`, `ControlPanel.svelte` | S (~30 min) |
| P0 | Rewrite Species 2/3/4 tooltip copy to match engine behaviour (finding #3) | `engine.svelte.ts:147-177` | S (~15 min) |
| P1 | Surface Species card existence from the Mode dropdown (finding #1) | `ControlPanel.svelte` | M (~1-2 h) |
| P1 | Rewrite Strictness labels/tooltips with concrete examples (finding #4) | `engine.svelte.ts:179-194` | S (~15 min) |
| P1 | Add "needs transport" indicator on Species card (finding #6) | `ControlPanel.svelte` | S (~30 min) |
| P2 | Keyboard focus + cell-label hover affordances (finding #7) | `PixelSelect.svelte`, `ControlPanel.svelte` | M (~1 h) |
| P2 | Last-phrase replay button for A/B auditioning (finding #5) | new component + buffer in engine store | L (separate phase) |

## Out of Scope

- Redesigning the entire ControlPanel layout — current single-card-per-row chrome works; only the contents need fixing.
- Renaming `StrictCounterpoint` mode itself — the mode name is fine, the issue is that its sub-controls are invisible until you commit.
- Engine-side behaviour of Species 2/3/4 — fixed in the parallel commit; this audit is strictly UX surface.
- Tooltip i18n / localisation — copy is English-only across the UI; not a regression introduced here.
