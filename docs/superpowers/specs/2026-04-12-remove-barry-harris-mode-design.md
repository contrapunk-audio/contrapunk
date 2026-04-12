# Phase A: Remove BarryHarris + Honest Mode Labeling — Design Spec

## What

Two changes in one PR:

1. **Remove `HarmonyMode::BarryHarris`** — delete the variant and its duplicate wrapper functions. The `BHMajor6thDim`/`BHMinor6thDim` scales stay. Old presets silently coerce to `DiatonicThirds` via `#[serde(alias)]`.

2. **Honest relabeling of all modes** — update display labels and add a `tooltip()` method to `HarmonyMode` with accurate algorithm descriptions. Rename misleading modes:

| Mode (enum variant) | Old label | New label | Tooltip |
|---------------------|-----------|-----------|---------|
| PassThrough | "Pass-through (no harmony)" | **unchanged** | "Notes pass unchanged to output" |
| DiatonicThirds | "Diatonic thirds above" | **"Parallel Thirds"** | "Adds +2 scale degrees per voice. Multiple voices stack into 7th chords." |
| DiatonicFourths | "Diatonic fourths above" | **"Parallel Fourths"** | "Adds +3 scale degrees per voice. Multiple voices stack into extended chords." |
| RandomBelow | "Random diatonic below" | **unchanged** | "Random diatonic interval (2nd-7th) below the melody" |
| RandomBelowNoSeconds | "Random below (no 2nds)" | **"Random Below (consonant)"** | "Random diatonic interval below, excluding dissonant 2nds" |
| ContraryMotion | "Contrary motion" | **unchanged** | "Harmony moves opposite to melody direction. Stateful — tracks previous notes." |
| StrictCounterpoint | "Strict counterpoint" | **"Counterpoint (Species 1, basic)"** | "Note-against-note voice leading with scoring. No parallel 5ths/octaves, prefers contrary/stepwise motion. Partial Species 1 rules." |
| ~~BarryHarris~~ | ~~"Barry Harris (6th dim movement)"~~ | **REMOVED** | — |

## Why

The HarmonyMode variant is a **six-line copy** of `diatonic_thirds`. Proof, from `src/harmony/modes.rs`:

```rust
pub fn diatonic_thirds(note: Note, scale: &mut Scale) -> Vec<Note> {
    match scale.harmonize_smart(note, 2, true) {        // (degrees=2, above=true)
        Some(harmony) => vec![note, harmony],
        None => vec![note],
    }
}

pub fn barry_harris(note: Note, scale: &mut Scale) -> Vec<Note> {
    match scale.harmonize_smart(note, 2, true) {        // IDENTICAL CALL
        Some(harmony) => vec![note, harmony],
        None => vec![note],
    }
}
```

`barry_harris_directed` and `diatonic_thirds_directed` are likewise byte-identical twins. The existing test `test_barry_harris_with_7note_scale` (`src/harmony/engine.rs:1693`) explicitly confirms this: on Ionian, `BarryHarris` produces `C4 → E4`, which is just a diatonic third.

The only reason the mode *feels* different on an 8-note BH scale is that `transpose_diatonic` walks by scale degrees, so `+2 degrees` lands on a chord-tone / passing-tone correctly in an 8-note scale — but that behavior belongs entirely to the **scale**, not the mode. The mode variant adds zero unique logic.

This was noted in the harmony-rework research (`.planning/research/harmony_05_jazz_barry_harris.md`) as "~5% of the real Barry Harris method" and flagged on HN by Lycomedes1814 as a mislabelled feature. The honest fix is to stop pretending the mode exists. Users who want the parity flavor already get it today by picking `BHMajor6thDim` scale + `DiatonicThirds` mode.

A proper Barry Harris implementation (drop-2 4-voice output, sister chord substitutions, beat-phase input, 8-note scale guard) remains planned as its own future phase — this spec does **not** ship that. Removing the placeholder now gives the future implementation a clean slot instead of a legacy variant to rip out.

## Design Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Remove the mode entirely? | Yes | Zero unique logic; synonym for DiatonicThirds |
| Keep BH scales? | Yes (`BHMajor6thDim`, `BHMinor6thDim`, `ScaleFamily::BarryHarris`) | They're real musical content; users compose BH flavor via scale + DiatonicThirds |
| Preset migration strategy | `#[serde(alias = "barry_harris")]` on `DiatonicThirds` | Silent, zero-code, zero-UX-impact; output is byte-identical |
| String-parser migration (tauri / wasm / plugin) | Accept `"BarryHarris" \| "barry_harris" \| "8"` → coerce to `DiatonicThirds` | Matches serde alias behavior; keeps external callers working |
| UI migration (Svelte store) | Remove `BarryHarris` from `HarmonyModeName` union + mode list; if a persisted config still has it, coerce on load | Keeps types honest, no orphan variants in the UI |
| Delete or reframe BH tests? | Delete `test_barry_harris_with_7note_scale` (redundant with DiatonicThirds test). Rename the two 8-note-scale tests to `test_diatonic_thirds_on_bh_scale_preserves_parity` — they're still musically interesting: they prove that diatonic thirds on an 8-note BH scale emergently preserves chord-tone/passing-tone parity. | Preserve the musical insight without the misleading mode name |
| Tag for the PR | `kind/refactor` + `release-note` block explaining the removal | HN visibility — release notes pick this up |

## What changes (exhaustive file list)

### Rust core

| File | Change |
|------|--------|
| `src/harmony/config.rs` | Remove `HarmonyMode::BarryHarris` variant (line 590), its `number()` arm (613), its `all()` entry (627), its `description()` arm (641). Add `#[serde(alias = "barry_harris")]` above `DiatonicThirds`. Update the doc-comment mode table (lines 543-553) to remove row 8. Update the `number()` doctest (line 602) from `BarryHarris.number() == 8` to `StrictCounterpoint.number() == 7`. |
| `src/harmony/modes.rs` | Delete `barry_harris` function (246-251) and `barry_harris_directed` function (253-260). |
| `src/harmony/engine.rs` | Remove the two dispatcher match arms (836, 865). Delete `test_barry_harris_with_7note_scale` (1693-1702). Rename the two 8-note-scale tests (1637, 1650) to use `DiatonicThirds` as the mode — same assertions, same musical point, honest label. |
| `src/harmony/mod.rs:38` | Update the module-level doc-comment mode table to remove the BarryHarris row. |

### Plugin (VST3 / CLAP)

| File | Change |
|------|--------|
| `plugin/src/lib.rs:79` | Remove `PluginMode::BarryHarris` variant. |
| `plugin/src/lib.rs:92` | Remove the mapping arm `Self::BarryHarris => HarmonyMode::BarryHarris`. |
| `plugin/src/editor.rs:257` | Remove the `"BarryHarris" => Some(PluginMode::BarryHarris)` parser arm. If external preset-state strings pass `"BarryHarris"`, coerce to `PluginMode::DiatonicThirds`. |

### Tauri desktop

| File | Change |
|------|--------|
| `src-tauri/src/commands/harmony.rs:180` | Change `"BarryHarris" \| "barry_harris" \| "8" => Ok(HarmonyMode::BarryHarris)` to `=> Ok(HarmonyMode::DiatonicThirds)`. Keeps legacy Tauri IPC calls working. |

### CLI binary

| File | Change |
|------|--------|
| `src/main.rs:333` | The CLI lookup `HarmonyMode::all().iter().find(\|m\| m.number() == number)` returns `None` for `--mode 8` after removal. Add an explicit coercion: if `number == 8`, map to `DiatonicThirds` (the same-behavior replacement). Keeps shell aliases and automation working. |

### WASM bridge

| File | Change |
|------|--------|
| `wasm/src/lib.rs:53` | `"BarryHarris" => Ok(HarmonyMode::DiatonicThirds)` (silent coerce). |
| `wasm/src/lib.rs:148` | Remove the `HarmonyMode::BarryHarris => "BarryHarris"` arm of the `mode_to_string` function (no longer reachable once the variant is gone). |

### UI (Svelte)

| File | Change |
|------|--------|
| `ui/src/lib/stores/engine.svelte.ts:50` | Remove `'BarryHarris'` from the `HarmonyModeName` type union. |
| `ui/src/lib/stores/engine.svelte.ts:145` | Remove the `{ name: 'BarryHarris', label: 'Barry Harris', shortLabel: 'Barry' }` entry from the modes list. |
| Any persisted `localStorage` config with `mode: 'BarryHarris'` | On engine-store load, coerce to `'DiatonicThirds'` before sending to WASM (defense in depth — the WASM side also handles it). |

### Examples / Docs

| File | Change |
|------|--------|
| `examples/guitar_harmony.rs:518` | Remove the `("5", "Barry Harris", HarmonyMode::BarryHarris)` demo entry. |
| `docs/ENGINE_DEEP_DIVE.md:247, 704` | Remove BarryHarris rows from the mode tables. Add a footnote noting the removal and pointing to the ScaleFamily::BarryHarris scales. |
| `.planning/phases/harmony-rework/.continue-here.md` | Update `<remaining_work>` to remove the "Phase A — Honest renaming" step for BarryHarris (it's now "removal", not "rename"). |

## Architecture / dispatch after the change

```
                            HarmonyEngine::harmonize_above / _below
                                          │
                                          ▼
                         ┌────────────────────────────────┐
                         │   match HarmonyMode             │
                         │                                 │
                         │   PassThrough                   │
                         │   DiatonicThirds  ◄──────┐      │
                         │   DiatonicFourths        │      │
                         │   RandomBelow            │      │
                         │   RandomBelowNoSeconds   │      │
                         │   ContraryMotion         │      │
                         │   StrictCounterpoint     │      │
                         │                          │      │
                         │   (BarryHarris removed) ─┘      │
                         │    ^^^ legacy presets coerce    │
                         │        here via serde alias     │
                         └────────────────────────────────┘
                                          │
                                          ▼
                              scale.harmonize_smart(...)
```

Dispatch table drops from 8 arms to 7. Mode `number()` method now returns 1-7 instead of 1-8 — **this is a breaking change for any code that hard-codes mode numbers**. Search confirms no such callers exist in-tree; external API consumers (if any) would see `number() == 8` no longer exist.

## Preset / persistence migration

**Rust side (core + plugin + tauri + wasm):** one `#[serde(alias = "barry_harris")]` on `DiatonicThirds` plus coercion in the tauri/wasm string parsers. Serde's `alias` attribute allows deserialization to accept `"barry_harris"` as a valid representation of `DiatonicThirds` — existing JSON presets round-trip cleanly, except on re-save they'll persist as `"diatonic_thirds"`. This is fine: the output is byte-identical.

**Svelte side (UI localStorage):** when the engine store loads persisted state, if `mode === 'BarryHarris'` it coerces to `'DiatonicThirds'` before any other use. One line.

**Rationale for silent migration:** the two modes ran identical code today. A user with `barry_harris` saved is already hearing diatonic thirds output. Renaming the label they see is not a regression — it's the label finally matching the behavior they've been hearing all along.

## Testing strategy

1. **Delete** `test_barry_harris_with_7note_scale` — redundant with existing DiatonicThirds tests.
2. **Rename** `test_barry_harris_chord_tone_to_chord_tone` → `test_diatonic_thirds_on_bh_scale_chord_tone_parity` (same assertions; honest label).
3. **Rename** `test_barry_harris_passing_tone_to_passing_tone` → `test_diatonic_thirds_on_bh_scale_passing_tone_parity` (same).
4. **Add** `test_legacy_preset_barry_harris_deserializes_as_diatonic_thirds` — parses a JSON string `{"mode":"barry_harris"}` and asserts the field is `HarmonyMode::DiatonicThirds`. Guards the serde alias.
5. **Add** `test_tauri_harmony_command_accepts_legacy_barry_harris_string` — calls the command's string parser with `"BarryHarris"` and asserts it returns `DiatonicThirds`.
6. **Verify full test suite passes** — `cargo test --all-features` should stay green.
7. **Run the plugin build** — `cargo xtask bundle-universal contrapunk-plugin` (or equivalent) to confirm the VST/CLAP plugin still compiles with the variant removed.
8. **UI smoke test** — load the WASM build, verify the mode selector shows 7 modes, pick each one, play a note, confirm notes emit.

## Out of scope

- Implementing the real Barry Harris method (drop-2 4-voice, sister chords, beat-phase). That's Phase D of the harmony-rework plan, tracked separately.
- Renaming the `ScaleFamily::BarryHarris` grouping or the `BHMajor6thDim` / `BHMinor6thDim` scale variants. They stay — they're real musical content.
- Writing the HN reply to Lycomedes1814. That happens after the PR ships, and should reference this design doc.
- Any changes to `DiatonicThirds` itself, its tests, or its docs.

## Follow-ups (not part of this spec)

- **HN reply**: draft a reply once the PR lands. Something like: *"You were right — our 'Barry Harris' harmony mode was a relabelled diatonic-thirds wrapper. We've removed it. The 8-note BH scales remain, and real Barry Harris method (drop-2 voicings, sister chord substitutions, beat-phase movement) is planned as its own phase."*
- **Phase D planning**: when we get to real Barry Harris implementation, this spec becomes the reference for "what we intentionally removed and why."
- **Release notes**: the PR should include a `release-note` block so monthly release notes pick up the removal with context, not just a diff summary.
