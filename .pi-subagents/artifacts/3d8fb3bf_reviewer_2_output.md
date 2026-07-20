## Review

### Correct
- The existing architecture already provides the right reuse seam: UI code imports one runtime-selected `ContrapunkAdapter` (`ui/src/lib/adapter/index.ts:28-55`), while Tauri exposes the full command surface from one backend entry (`src-tauri/src/main.rs:24-180`).
- A dedicated route is supported by the static build: `adapter-static` uses `index.html` fallback and prerenders wildcard routes (`ui/svelte.config.js:10-17`).
- Existing feature components cover most native functionality:
  - Input, guitar, and calibration: `ui/src/lib/components/InputPanel.svelte:217-390`
  - Per-voice routing, synth, FX, and chain: `ui/src/lib/components/OutputPanel.svelte:74-130`
  - Harmony, Companion, Voices, Piano, Fretboard, and History are already independently composable from `ui/src/routes/+page.svelte:1-16`.
- The backend already emits useful comparator data at approximately 30 fps: continuous detected `frequency` in `guitar-signal` and active emitted note sets in `note-update` (`src-tauri/src/commands/engine.rs:785-829`).

### Blocker
- **Blocker — current capability-based branch is not isolated.** `ui/src/routes/+page.svelte:226-229` selects `PluginWorkspace` when `inputSourcePicker` is true; Tauri explicitly sets that capability true at `ui/src/lib/adapter/tauri.ts:103-123`. Consequently, the normal desktop root route is replaced by the prototype workspace. This directly violates “separate prototype” and “do not change production behavior.”
- Do not use adapter capabilities as a design-selection flag. Capabilities describe backend functionality, not which UI experiment should render.

### Notes and risks
- **High — current workspace violates monochrome-idle rules.** Global particles render magenta/cyan/teal continuously (`ui/src/lib/components/Particles.svelte:5-20,55-101`), global tokens make navigation and controls colorful (`ui/src/lib/theme/tokens.css:74-110`), and `PluginWorkspace` applies colored headings, active controls, and relationship borders even without sound (`ui/src/lib/components/PluginWorkspace.svelte:205-324,360-375`).
- **High — continuous expression is discarded before reaching stores.** The backend payload contains `frequency`, clarity, note state, and rounded MIDI note (`src-tauri/src/commands/engine.rs:99-108`), but `TauriAdapter` only forwards RMS and clarity to the guitar store (`ui/src/lib/adapter/tauri.ts:495-515`). The store retains only amplitude and clarity histories (`ui/src/lib/stores/guitar.svelte.ts:89-119`). `LiveLines` therefore operates solely on integer note gates (`ui/src/lib/components/LiveLines.svelte:82-127`) and cannot show continuous guitar pitch.
- **Medium — `PluginWorkspace` is not “all app features.”** It limits strict counterpoint choices to Species 1 (`ui/src/lib/components/PluginWorkspace.svelte:37-41`), while the normal page explicitly leaves `PresetManager` unmounted (`ui/src/routes/+page.svelte:5-6`) and the current workspace omits the full transport surface.
- **Medium — comparator fidelity ceiling.** Both existing telemetry streams are UI-coalesced around 30 fps. They can compare detected fractional pitch with emitted NoteOn/NoteOff gates, but not exact event timing, pitch-bend bytes, CC, or channel pressure. Do not label this “raw MIDI.”
- **Note — requested `plan.md` was absent.** `/Users/vibhavbobade/go/src/github.com/contrapunk-audio/contrapunk/plan.md` could not be read. `progress.md` contained unrelated AU/REAPER investigation, so this review used the task contract and current code.

## Recommended isolation boundary

Use a **dedicated Svelte route plus a Tauri config override**:

- `ui/src/routes/prototype/+page.svelte`
- `ui/src/lib/prototype/prototype.css`
- `ui/src/lib/prototype/GuitarMidiComparator.svelte`
- `ui/src/lib/prototype/nativeTelemetry.ts`
- `src-tauri/tauri.prototype.conf.json`
- `ui/tests/prototype.spec.ts`

The override should:

- Keep window label `main`, avoiding capability changes.
- Set the window URL to `"prototype"`.
- Use a distinct title, product name, and bundle identifier so it cannot overwrite the production app.
- Retain the same `src-tauri/src/main.rs`, `AppState`, handlers, adapter singleton, and stores.

Launch target:

```text
cd src-tauri && cargo tauri dev --config tauri.prototype.conf.json
```

The prototype route should initialize the same adapter and stores locally. For the first slice, duplicate only the small startup orchestration rather than extracting or modifying production startup. Consolidate it only if the prototype is promoted.

`nativeTelemetry.ts` should be the sole prototype-only Tauri import. It can subscribe to the existing `guitar-signal` event without widening every adapter implementation. The rest of the prototype continues through real adapters and stores.

### Alternative comparison

| Alternative | Isolation | Risk | Recommendation |
|---|---:|---:|---|
| `/prototype` route + Tauri config override | High | Low | **Use this** |
| `/?prototype=1` query branch | Low | High | Reject: changes root rendering and is easy to ship accidentally |
| Build-time environment flag | Medium | Medium | Reject: hidden build variance and cache/config mistakes |
| Capability-based branch | None | High | Reject: currently changes normal Tauri behavior |
| Second Rust/Tauri binary | Very high | High maintenance | Reject: duplicates handler and state registration |

## Proposed navigation

Persistent monochrome header:

- Routing Start/Stop
- Compact transport
- Panic
- Settings
- Current backend/error state using icon, text, and border—not hue

Top-level tabs:

1. **Perform**
   - Active notes
   - Continuous guitar-expression versus emitted-MIDI comparator
   - Retained time-based piano roll as the primary visualization
   - Optional Piano/Fretboard/History views
2. **Harmony**
   - Full `ControlPanel`
   - Performance controls only as an optional simplified subview
3. **Ensemble**
   - Companion lanes
   - Voice library
   - Presets
4. **I/O**
   - Input/Guitar/Calibration subtab
   - Output/Routing/Synth/FX/Chain subtab

Settings remains a utility action rather than a fifth content tab. Keep navigation state route-local so production `ui` persistence keys are not changed.

## Visual, accessibility, and performance constraints

- Override accent tokens only inside `.prototype-shell`; do not alter global theme files.
- Disable and restore `ui.animationsEnabled` for the prototype route so global colored particles neither render nor consume animation frames.
- Idle controls, focus, selection, errors, and warnings remain grayscale. Use weight, inversion, border style, icons, and text.
- Color is reserved for sounding or historically emitted material:
  - Detected guitar expression: continuous line
  - Emitted MIDI: stepped blocks
  - Harmony/canon/counterpoint: distinct sound-role colors
- Do not rely on color alone: preserve labels, line/block shapes, dash patterns, and `<title>`/text summaries.
- Use WAI-ARIA tabs with roving tabindex and Arrow/Home/End behavior, following the existing implementation at `ui/src/routes/+page.svelte:132-190`.
- Provide visible monochrome focus rings and at least 24×24 CSS-pixel targets.
- Avoid an `aria-live` update every telemetry frame; announce only note-gate/status transitions.
- Comparator implementation:
  - Convert frequency to fractional MIDI using `69 + 12 * log2(f / 440)`.
  - Use one bounded ring buffer and one canvas.
  - Cap refresh at 30 fps.
  - Pause when hidden or when guitar routing is inactive.
  - Keep samples outside reactive Svelte arrays.
  - Bound history by time and count.
- The current `LiveLines` ceiling is 512 SVG segments updated from a 30 fps clock (`ui/src/lib/components/LiveLines.svelte:17-18,129-141,334-349`). Retain it for the emitted-note roll, but do not add continuous pitch samples as hundreds of SVG nodes.

## Smallest testable vertical slice

1. Add the prototype route and Tauri config override.
2. Render the monochrome shell with Perform, Harmony, Ensemble, and I/O navigation.
3. Make Perform functional with:
   - Real Input panel
   - Real Start/Stop and Panic
   - Existing note-update store
   - Existing `guitar-signal` subscription
   - One combined fractional-pitch line and emitted-note-gate plot
   - Existing piano roll retained below it
4. Mount the existing feature components behind the remaining tabs without redesigning them yet.
5. Add one Playwright smoke test asserting:
   - `/prototype` renders all navigation destinations
   - Tabs are keyboard operable
   - Only one panel is mounted at a time
   - Idle shell tokens are grayscale
   - Particle canvas is absent
6. Manually validate native guitar routing because hardware telemetry cannot be proven by browser Playwright.

Exact raw MIDI bends/CC telemetry is intentionally deferred. Add a prototype-gated backend event only if the first comparator proves the 30 fps note-gate representation insufficient.