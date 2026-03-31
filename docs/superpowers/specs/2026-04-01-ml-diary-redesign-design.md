# ML Diary Redesign — Design Spec

## What

Rebuild the Contrapunk development diary as an interactive learning experience. The diary covers the full Contrapunk project journey — from MIDI foundation to ML guitar classification — with the ML chapter as the flagship interactive section featuring audio playback, spectrogram visualization, dataset exploration, and live WASM-based model inference.

## Why

The current diary page scored 4/10 in review: hardcoded data, static matplotlib PNGs, no audio on an audio classification project, no interactivity, disconnected visual identity. It needs to become a world-class educational product that serves both musicians and ML practitioners.

## Core Identity

**Contrapunk is an improvisation companion.** The diary tells the story of building that companion — how it learns to understand your instrument.

## Design Decisions

| Decision | Choice |
|----------|--------|
| Audience | Both musicians and ML practitioners equally |
| Structure | Hybrid: narrative spine with embedded interactive explorer stations |
| Visual system | Pixel font for headers/numbers/labels, system sans-serif for body text, full HLD color palette (magenta/cyan/teal on deep indigo) |
| Landing hook | Auto-playing demo: guitar pluck → spectrogram → classification result |
| Interactivity depth | Full live playground with WASM inference in browser |
| Scope | Full Contrapunk dev journal (all phases), ML chapter is the first deep chapter |
| Concept teaching | Inline explanations with collapsible "learn more" sections |
| Icons | Functional SVG icons only, no emojis |
| Tone | Direct, factual. Not emotional or marketing-speak |
| Data source | All metrics loaded from results.json via fetch. Never hardcoded. |

## Information Architecture

```
/diary                                  Landing page with animated demo + chapter grid
/diary/machine-learning                 ML chapter overview: approach, progress chart, round timeline
/diary/machine-learning/round-1         Round 1 narrative with 6 interactive stations
/diary/machine-learning/round-2         Round 2 narrative (onset alignment, comparison to R1)
/diary/machine-learning/round-N         Future rounds added as they happen
/diary/machine-learning/explore         Dataset explorer: guitar neck nav, filter, hear, see spectrograms
/diary/machine-learning/playground      Live WASM inference: record or upload, classify in browser
```

Future chapters (not in scope for this build):
- `/diary/midi-foundation` — Phases 1-3
- `/diary/harmony-engine` — Phase 2 + 6.x
- `/diary/wasm-browser` — Phase 5.1
- `/diary/rust-inference` — Integration story

## Page Designs

### /diary — Landing Page

1. **Hero section:** "Contrapunk is an improvisation companion" tagline. Animated demo: pre-recorded pluck auto-plays, waveform draws, spectrogram fills, classification result appears. "Play Demo" button to replay with different strings.
2. **Stats bar:** Accuracy (96.2%), Inference (2.1ms), Classes (138), Dependencies (0).
3. **Chapter grid:** Cards for each project phase. Completed phases muted. Active phase (ML) has glow border + progress bar. Future phases dashed outline. Click navigates to chapter.

### /diary/machine-learning — Chapter Overview

1. **Header:** Chapter title, one-line description of the classification challenge.
2. **The Approach:** 4-step visual: Train → Change one thing → Measure → Repeat.
3. **Progress chart:** SVG line chart of Pure CNN accuracy across rounds. Completed rounds as solid dots, future projected as dashed.
4. **Round timeline:** Vertical timeline. Each round card shows: title, accuracy, delta from previous, date, completion status. Click navigates to round page.
5. **Bottom links:** "Explore Data" and "Live Playground" cards.

### /diary/machine-learning/round-N — Training Round Page

Scrollable narrative with 6 embedded interactive stations:

**Station 1 — Hear the Data:** A/B audio comparison. Two samples of the same note on different strings. Waveform display synced to playback. Side-by-side spectrogram comparison. Question: "Can you hear the difference?"

**Station 2 — See the Spectrogram:** Canvas-rendered mel-spectrogram viewer. Hover shows frequency bin, time frame, dB value. Playhead syncs with audio. Comparison view for confused pairs or quality (normal vs clipped).

**Station 3 — How the Models Work:** SVG architecture diagrams for each model (RF pipeline, Hybrid CNN, Pure CNN). Data flows through with dimensional annotations. Click a layer to see what it does. Inline explanation of why Pure CNN wins (GAP vs dense bottleneck, parameter count).

**Station 4 — The Results:** Model selector toggle (RF / Hybrid / Pure). SVG per-string accuracy bars. SVG 6x23 fret accuracy heatmap — click a cell to hear the sample and see if the model got it right. All data from results.json.

**Station 5 — Where It Struggles:** String-level confusion view. Click confused pair to hear both samples. Highlights why E2 is hardest (thick wound string, tight fret spacing, complex harmonics).

**Station 6 — What We Learned:** Key insights from the round. For Round 2: "Onset alignment had zero impact — the capture tool was already triggering at the pluck. This tells us the bottleneck is data quantity, not alignment."

**Round comparison banner** (Round 2+): Shows previous round accuracy → current round accuracy → what changed → why.

### /diary/machine-learning/explore — Dataset Explorer

1. **Filter bar:** String buttons (E2/A2/D3/G3/B3/E4), fret range slider, sample count display.
2. **Guitar neck visualization:** Clickable grid representing the guitar neck. Shows note names. Same-note positions highlight together (e.g., all A2 positions glow when one is clicked). Accuracy color overlay available as toggle.
3. **Sample detail panel:** Selected position shows waveform (with playback), spectrogram (Canvas), metadata (string, fret, note, frequency, RMS, peak), and model prediction with confidence.
4. **Data loaded from:** `static/samples/index.json` for metadata, WAV files for playback, `static/spectrograms/` for pre-computed spectrogram JSON.

### /diary/machine-learning/playground — Live Playground

1. **Input options:** Record (Web Audio API microphone capture) or Upload (.wav file).
2. **Processing visualization:** Audio → waveform display → mel-spectrogram extraction (shown in real time) → model inference.
3. **Result display:** Top prediction with confidence, top-3 ranked list, spectrogram of the input, "How it works" explainer showing pipeline timing.
4. **Technology:** 23KB ONNX model running via WASM. Zero server calls. Mel-spectrogram extraction in JavaScript. Entirely client-side.

## Components

All in `ui/src/lib/components/diary/`:

### Audio
- `audioContext.svelte.ts` — singleton AudioContext store, autoplay policy handling
- `AudioPlayer.svelte` — play/pause, waveform progress bar, accepts WAV URL
- `WaveformDisplay.svelte` — Canvas waveform renderer, playhead sync
- `AudioComparison.svelte` — A/B side-by-side or toggle playback

### Visualization
- `SpectrogramViewer.svelte` — Canvas 64x94 mel-spectrogram, HLD color scale, hover tooltips, playhead
- `AccuracyHeatmap.svelte` — SVG 6x23 string-fret grid, click-to-play, accuracy color coding
- `PerStringBars.svelte` — SVG horizontal bars, animated, color-coded by threshold
- `TrainingCurves.svelte` — SVG dual-axis line chart (loss + accuracy over epochs)
- `ConfusionExplorer.svelte` — Canvas confusion matrix with zoom/pan, click-to-hear
- `ModelComparison.svelte` — model selector driving child visualization updates
- `ProgressChart.svelte` — SVG round-over-round accuracy line chart
- `colorScale.ts` — HLD-palette color mapping utility

### Navigation / Layout
- `DiaryNav.svelte` — sticky breadcrumb: CONTRAPUNK > Diary > Machine Learning > Round 1
- `GuitarNeck.svelte` — interactive guitar neck grid for dataset explorer
- `ConceptInline.svelte` — inline explanation with collapsible depth

### Diagrams
- `ArchitectureDiagram.svelte` — SVG parametric model architecture (3 presets)
- `PipelineDiagram.svelte` — SVG flow: Audio → Onset → Spectrogram → CNN → Class

### Playground
- `AudioRecorder.svelte` — Web Audio capture with onset detection
- `WasmInference.svelte` — ONNX WASM runtime, mel extraction in JS, inference display

## Data Pipeline

### Existing (already exported)
- `ui/static/samples/` — 166 WAV files + index.json (7.63 MB)
- `ui/static/spectrograms/` — 15 JSON files + index.json (616 KB)
- `ui/static/training/round_01/results.json` — model accuracies and per-string data
- `ml/training/round_01/pure_cnn.onnx` — 23KB ONNX model for WASM inference

### Needs export
- `training_curves.json` — epoch-by-epoch loss/accuracy for CNN models
- `confusion_matrices.json` — raw 138x138 matrices per model (for Canvas renderer)
- `fret_accuracy.json` — 6x23 per-position accuracy arrays per model
- `concepts.json` — structured content from CONCEPTS.md for inline explanations
- Round 2 results and visualizations copied to static/

### Loading pattern
All data via `fetch()` in `$effect` or `onMount`. No server-side. No build-time imports. Browser caches independently.

## Design System Extension

Add to `ui/src/lib/theme/tokens.css`:
```css
--font-reading: system-ui, -apple-system, sans-serif;
```

Diary layout uses `--color-bg-deep` (not hardcoded #030712), `--font-reading` for body, `--font-pixel` for headers/labels/numbers. All accent colors from existing HLD tokens.

SVG icons only. No emojis. Functional icon set: play, pause, upload, record, expand, collapse, filter, link.

## Build Phases

### Phase 1: Foundation
Restructure routes, HLD theming, DiaryNav, ConceptInline, landing page (static), ML overview page, Round 1 page with data from results.json. No interactive viz yet — plain text + styled sections.

### Phase 2: Audio
audioContext store, AudioPlayer, WaveformDisplay, AudioComparison. Integrate into Round 1 Stations 1 and 5. Users can hear samples.

### Phase 3: Spectrograms
SpectrogramViewer with Canvas rendering, colorScale utility, playhead sync. Integrate into Stations 1 and 2. Replace text explanations with rendered spectrograms.

### Phase 4: Native Charts
Export training_curves.json, confusion_matrices.json, fret_accuracy.json. Build PerStringBars, AccuracyHeatmap, TrainingCurves, ConfusionExplorer, ModelComparison, ProgressChart. Replace all matplotlib PNGs.

### Phase 5: Explorer + Diagrams
GuitarNeck, dataset explorer page, ArchitectureDiagram, PipelineDiagram. Full dataset browsing with audio and spectrograms.

### Phase 6: Playground
AudioRecorder, WasmInference. ONNX model in browser. Record or upload, classify live.

### Phase 7: Polish
Responsive layout, keyboard nav, loading states, page transitions, a11y, performance, meta tags.

## Not in Scope
- Other diary chapters (MIDI, Harmony, WASM) — structure supports them but content deferred
- Real-time continuous audio stream classification (playground is single-pluck)
- Mobile mic recording (desktop browsers only for playground)
- Server-side rendering or API endpoints
