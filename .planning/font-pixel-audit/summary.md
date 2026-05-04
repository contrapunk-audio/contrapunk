# Font-Pixel Migration Audit — Issue #51

Total occurrences classified: **250** across **33** files.

## Totals by target class

| Target class | Count |
|---|---:|
| `--font-ui` | 183 |
| `--font-code` | 65 |
| `--font-body` | 0 |
| `--font-display` | 0 |
| `--ambiguous` | 2 |

## Per-file breakdown

| File | Total | ui | code | body | display | ambiguous |
|---|---:|---:|---:|---:|---:|---:|
| `ui/src/lib/components/ActiveNotes.svelte` | 9 | 3 | 6 | 0 | 0 | 0 |
| `ui/src/lib/components/ChainPanel.svelte` | 25 | 24 | 0 | 0 | 0 | 1 |
| `ui/src/lib/components/ClapPluginPicker.svelte` | 8 | 7 | 0 | 0 | 0 | 1 |
| `ui/src/lib/components/ControlPanel.svelte` | 16 | 14 | 2 | 0 | 0 | 0 |
| `ui/src/lib/components/GuitarInputPanel.svelte` | 36 | 28 | 8 | 0 | 0 | 0 |
| `ui/src/lib/components/Knob.svelte` | 2 | 1 | 1 | 0 | 0 | 0 |
| `ui/src/lib/components/MidiDevices.svelte` | 5 | 4 | 1 | 0 | 0 | 0 |
| `ui/src/lib/components/Piano.svelte` | 3 | 0 | 3 | 0 | 0 | 0 |
| `ui/src/lib/components/PixelSelect.svelte` | 3 | 3 | 0 | 0 | 0 | 0 |
| `ui/src/lib/components/PresetManager.svelte` | 6 | 6 | 0 | 0 | 0 | 0 |
| `ui/src/lib/components/SettingsModal.svelte` | 14 | 12 | 2 | 0 | 0 | 0 |
| `ui/src/lib/components/SignalGraphs.svelte` | 2 | 2 | 0 | 0 | 0 | 0 |
| `ui/src/lib/components/StatusBar.svelte` | 7 | 4 | 3 | 0 | 0 | 0 |
| `ui/src/lib/components/TransportBar.svelte` | 4 | 3 | 1 | 0 | 0 | 0 |
| `ui/src/lib/components/diary/AudioComparison.svelte` | 2 | 2 | 0 | 0 | 0 | 0 |
| `ui/src/lib/components/diary/AudioPlayer.svelte` | 1 | 0 | 1 | 0 | 0 | 0 |
| `ui/src/lib/components/diary/DemoAnimation.svelte` | 3 | 2 | 1 | 0 | 0 | 0 |
| `ui/src/lib/components/diary/DiaryNav.svelte` | 1 | 1 | 0 | 0 | 0 | 0 |
| `ui/src/lib/components/diary/RoundCard.svelte` | 2 | 0 | 2 | 0 | 0 | 0 |
| `ui/src/lib/components/diary/SpectrogramComparison.svelte` | 2 | 2 | 0 | 0 | 0 | 0 |
| `ui/src/lib/components/diary/SpectrogramViewer.svelte` | 2 | 1 | 1 | 0 | 0 | 0 |
| `ui/src/lib/components/diary/StatBar.svelte` | 1 | 0 | 1 | 0 | 0 | 0 |
| `ui/src/lib/components/diary/WaveformDisplay.svelte` | 1 | 1 | 0 | 0 | 0 | 0 |
| `ui/src/routes/+page.svelte` | 4 | 4 | 0 | 0 | 0 | 0 |
| `ui/src/routes/diary/+page.svelte` | 3 | 2 | 1 | 0 | 0 | 0 |
| `ui/src/routes/diary/machine-learning/+page.svelte` | 4 | 3 | 1 | 0 | 0 | 0 |
| `ui/src/routes/diary/machine-learning/playground/+page.svelte` | 12 | 4 | 8 | 0 | 0 | 0 |
| `ui/src/routes/diary/machine-learning/round-1/+page.svelte` | 12 | 8 | 4 | 0 | 0 | 0 |
| `ui/src/routes/diary/machine-learning/round-2/+page.svelte` | 13 | 9 | 4 | 0 | 0 | 0 |
| `ui/src/routes/diary/machine-learning/round-3/+page.svelte` | 11 | 7 | 4 | 0 | 0 | 0 |
| `ui/src/routes/diary/machine-learning/round-4/+page.svelte` | 11 | 7 | 4 | 0 | 0 | 0 |
| `ui/src/routes/diary/machine-learning/round-5/+page.svelte` | 14 | 10 | 4 | 0 | 0 | 0 |
| `ui/src/routes/diary/machine-learning/the-pivot/+page.svelte` | 11 | 9 | 2 | 0 | 0 | 0 |

## Ambiguous cases (require human review)

1. **ui/src/lib/components/ChainPanel.svelte:352** — `<div class="plugin-body font-pixel"><Type:CLAP / Plugin ID: b.typeId></div>`
   - **Reason**: Parent wraps a Type:CLAP label (UI) + plugin-id value (code). Split: labels -> font-ui, .plugin-value.plugin-id -> font-code
2. **ui/src/lib/components/ClapPluginPicker.svelte:68** — `<button class="plugin-row font-pixel" …>name/vendor/path</button>`
   - **Reason**: Row contains plugin-name (UI) + path (code). Needs splitting — child spans should get font-ui / font-code per role

## Recommended edit strategy

### Bulk-editable files (all hits -> single target class)

| File | Target | Hits |
|---|---|---:|
| `ui/src/lib/components/PresetManager.svelte` | `--font-ui` | 6 |
| `ui/src/routes/+page.svelte` | `--font-ui` | 4 |
| `ui/src/lib/components/Piano.svelte` | `--font-code` | 3 |
| `ui/src/lib/components/PixelSelect.svelte` | `--font-ui` | 3 |
| `ui/src/lib/components/SignalGraphs.svelte` | `--font-ui` | 2 |
| `ui/src/lib/components/diary/AudioComparison.svelte` | `--font-ui` | 2 |
| `ui/src/lib/components/diary/RoundCard.svelte` | `--font-code` | 2 |
| `ui/src/lib/components/diary/SpectrogramComparison.svelte` | `--font-ui` | 2 |
| `ui/src/lib/components/diary/AudioPlayer.svelte` | `--font-code` | 1 |
| `ui/src/lib/components/diary/DiaryNav.svelte` | `--font-ui` | 1 |
| `ui/src/lib/components/diary/StatBar.svelte` | `--font-code` | 1 |
| `ui/src/lib/components/diary/WaveformDisplay.svelte` | `--font-ui` | 1 |

These files can be mechanically swept: global find-replace `font-pixel` -> the single target class (for `.svelte` class attributes) or `var(--font-pixel)` -> `var(--font-<role>)` inside `<style>` blocks.

### Files needing per-site reasoning (mixed targets)

| File | Total | Breakdown |
|---|---:|---|
| `ui/src/lib/components/GuitarInputPanel.svelte` | 36 | font-code:8, font-ui:28 |
| `ui/src/lib/components/ChainPanel.svelte` | 25 | ambiguous:1, font-ui:24 |
| `ui/src/lib/components/ControlPanel.svelte` | 16 | font-code:2, font-ui:14 |
| `ui/src/lib/components/SettingsModal.svelte` | 14 | font-code:2, font-ui:12 |
| `ui/src/routes/diary/machine-learning/round-5/+page.svelte` | 14 | font-code:4, font-ui:10 |
| `ui/src/routes/diary/machine-learning/round-2/+page.svelte` | 13 | font-code:4, font-ui:9 |
| `ui/src/routes/diary/machine-learning/playground/+page.svelte` | 12 | font-code:8, font-ui:4 |
| `ui/src/routes/diary/machine-learning/round-1/+page.svelte` | 12 | font-code:4, font-ui:8 |
| `ui/src/routes/diary/machine-learning/round-3/+page.svelte` | 11 | font-code:4, font-ui:7 |
| `ui/src/routes/diary/machine-learning/round-4/+page.svelte` | 11 | font-code:4, font-ui:7 |
| `ui/src/routes/diary/machine-learning/the-pivot/+page.svelte` | 11 | font-code:2, font-ui:9 |
| `ui/src/lib/components/ActiveNotes.svelte` | 9 | font-code:6, font-ui:3 |
| `ui/src/lib/components/ClapPluginPicker.svelte` | 8 | ambiguous:1, font-ui:7 |
| `ui/src/lib/components/StatusBar.svelte` | 7 | font-code:3, font-ui:4 |
| `ui/src/lib/components/MidiDevices.svelte` | 5 | font-code:1, font-ui:4 |
| `ui/src/lib/components/TransportBar.svelte` | 4 | font-code:1, font-ui:3 |
| `ui/src/routes/diary/machine-learning/+page.svelte` | 4 | font-code:1, font-ui:3 |
| `ui/src/lib/components/diary/DemoAnimation.svelte` | 3 | font-code:1, font-ui:2 |
| `ui/src/routes/diary/+page.svelte` | 3 | font-code:1, font-ui:2 |
| `ui/src/lib/components/Knob.svelte` | 2 | font-code:1, font-ui:1 |
| `ui/src/lib/components/diary/SpectrogramViewer.svelte` | 2 | font-code:1, font-ui:1 |

For these, walk each occurrence using `audit.json` and apply the per-site `target_class`.

## Rules applied

- **`--font-ui`** — panel headers/titles, section labels (caps), card headers, nav links, form labels/inputs, dropdowns, toggle buttons, transport buttons, tab buttons, chrome glyphs (arrows, X close), loading/error/status captions, hints.
- **`--font-code`** — MIDI note names, chord names (Cmaj7), BPM/detune/cents numerics, percentages, accuracy stats, round-number badges, ASCII meters, technical file paths, tech-detail values, axis labels with numeric ticks, table cells holding numeric stats.
- **`--font-body`** — (not observed; diary prose already uses `var(--font-reading)` / default, not `--font-pixel`).
- **`--font-display`** — (not observed; diary H1s already use `var(--font-reading)`).
