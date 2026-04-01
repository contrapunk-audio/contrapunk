# Diary Update Plan: DSP Research Findings + Hybrid Architecture Chapter

## Background

After 3 rounds of ML training (reaching 97.3% with Pure CNN), research revealed that physics-based DSP using the inharmonicity B coefficient achieves 98.5% string+fret accuracy with just 1 calibration sample per string. This is a major narrative pivot: the ML journey led us to discover that DSP is actually better for the core task. The diary should document this honestly and educationally.

## Narrative Arc

```
Rounds 1-3: ML iterative training (existing)
  "We can get to 97.3% by cleaning data and training CNNs"

Round 4: Goertzel harmonics (new)
  "Physics-based features helped Hybrid CNN but not Pure CNN"

Round 5: Augmentation (new)
  "More data didn't break through the ceiling"

The Pivot (new)
  "Five rounds taught us what matters. Then research showed us something bigger."

DSP Pipeline chapter (new)
  "Physics achieves 98.5% with 1 calibration sample. Here's how."

Hybrid Architecture (new)
  "DSP for speed and accuracy, ML for edge cases. The best of both."
```

---

## 1. New Page: ML Round 4 — Goertzel Harmonics

**Route:** `ui/src/routes/diary/machine-learning/round-4/+page.svelte`

**Data files needed:**
- `ui/static/training/round_04/results.json` — same format as round_01-03 results
- `ui/static/training/round_04/goertzel_features.json` — sample Goertzel harmonic ratios for visualization

**Content outline:**

### Hero
- ROUND 04
- Title: "Goertzel Harmonics"
- Subtitle: "Hypothesis: if we extract physics-based harmonic ratio features using Goertzel filters and feed them alongside the mel-spectrogram, the model should learn string identity from the harmonics. Result: Hybrid CNN improved, Pure CNN did not."

### StatBar
- Best accuracy: 96.4% (or whatever the actual number is)
- Delta vs Round 3
- Feature: "15 harmonic ratios"
- Method: "Goertzel filters"

### Station 1: What Changed
- Explain Goertzel algorithm (use ConceptInline for the term)
- 15 harmonic ratio features extracted per sample
- These capture the relative strength of each harmonic partial
- Why harmonics matter: same note on different strings has different harmonic signatures due to string thickness, material, and vibrating length

### Station 2: How Goertzel Works
- Interactive concept: show a waveform with harmonics highlighted
- Explain: Goertzel is like a single-frequency FFT — instead of computing all frequencies, it computes the magnitude at one specific frequency. Run it at f0, 2*f0, 3*f0, ..., 15*f0 to get harmonic magnitudes.
- ConceptInline for "Goertzel filter" and "harmonic series"

### Station 3: Results
- Comparison banner: Round 3 vs Round 4
- Per-model breakdown showing Hybrid CNN gained but Pure CNN did not
- Insight box: "The Pure CNN was already learning harmonic patterns implicitly from the mel-spectrogram. Explicitly adding them as features didn't help — the CNN's convolutional layers were already extracting similar information."

### Station 4: What We Learned
- Insight: features that help simple models don't always help deep models
- The CNN's spatial awareness already captures harmonic structure
- This raised a question: if harmonics are the key discriminator, maybe we don't need ML at all?

**Components to reuse:**
- DiaryNav, StatBar, ConceptInline (all existing)
- Comparison banner pattern from round-2/round-3

**New components needed:**
- None strictly required; can use existing patterns
- Optional: HarmonicRatioViz — bar chart showing harmonic strength ratios for different strings (could be a simple inline SVG, not a full component)

---

## 2. New Page: ML Round 5 — Augmentation

**Route:** `ui/src/routes/diary/machine-learning/round-5/+page.svelte`

**Data files needed:**
- `ui/static/training/round_05/results.json`

**Content outline:**

### Hero
- ROUND 05
- Title: "Data Augmentation"
- Subtitle: "Hypothesis: more data through augmentation (pitch shift, time stretch, noise injection) should push accuracy past the 96-97% ceiling. Result: no improvement. The ceiling isn't a data problem."

### StatBar
- Best accuracy: 96.4% (same as Round 4)
- Delta: +0.0%
- Augmented samples: count
- Augmentation types: 3

### Station 1: What Changed
- Applied three augmentation strategies:
  1. Noise injection (signal chain noise at varying SNR)
  2. Time stretching (slight tempo variations)
  3. Gain variation (simulate different playing dynamics)
- Did NOT apply pitch shifting (too risky — would create ambiguous labels)
- ConceptInline for "data augmentation", "SNR"

### Station 2: Why It Didn't Help
- The training set is small (1,380 samples, 10 per class) but very clean after Round 3
- The model's errors are not from lack of data — they're from fundamental acoustic ambiguity
- Same note on adjacent strings/frets can produce nearly identical spectrograms
- More copies of the same ambiguous data doesn't resolve the ambiguity

### Station 3: The Ceiling
- Visualization: accuracy plateau across rounds 3-5
- Table: Round 1 (96.2%) -> Round 2 (96.2%) -> Round 3 (97.3%) -> Round 4 (96.4%) -> Round 5 (96.4%)
- Insight box: "Three rounds of experimentation confirmed: the remaining errors come from acoustic physics, not insufficient data or features. Some string+fret positions are genuinely hard to distinguish from spectrograms alone."

### Station 4: Foreshadowing
- "This plateau forced us to ask a different question. Instead of 'how do we make the ML model better?', we asked 'is ML the right tool for this task?'"
- "The answer led us to the physics of guitar strings."
- Link to "The Pivot" page

**Components to reuse:**
- DiaryNav, StatBar, ConceptInline
- Comparison banner pattern
- PerStringBars for per-string accuracy

**New components needed:**
- None required
- Optional: AccuracyTimeline — small line chart showing accuracy across all 5 rounds (could be inline SVG)

---

## 3. New Page: The Pivot

**Route:** `ui/src/routes/diary/machine-learning/the-pivot/+page.svelte`

This is the narrative bridge between ML and DSP. Not a training round — a reflection page.

**Content outline:**

### Hero
- Label: "REFLECTION" (not "ROUND")
- Title: "The Pivot: From Machine Learning to Physics"
- Subtitle: "Five rounds of training taught us what matters and what doesn't. Then we read the papers."

### Station 1: What Five Rounds Taught Us
- Lessons in order of impact:
  1. Clean data > augmented data (Round 3 > Round 5)
  2. The capture tool matters more than preprocessing (Round 2 taught us this)
  3. Deep models learn features implicitly (Round 4 vs Pure CNN)
  4. There's a ceiling set by acoustic physics (Round 5 plateau)
- Each lesson formatted as a finding with icon (reuse the findings list pattern from round-3)

### Station 2: The Research
- "While investigating why our CNN plateaued, we read the literature on guitar string identification."
- Reference key papers (as ConceptInline expandable references):
  - Hjerrild & Christensen 2019 — inharmonicity B coefficient, 98.5%, 1 sample/string
  - Derrien 2014 — sub-period pitch estimation, <10ms latency
  - Spotify Basic Pitch 2022 — 17K param neural pitch detector
  - Abesser 2012 — 541-feature SVM baseline (90% F1)
- Comparison table (styled like the detail-grid from round-1):

  | Approach | Accuracy | Training Data | Latency | Physics? |
  |----------|----------|---------------|---------|----------|
  | Our CNN (Round 3) | 97.3% | 1,380 samples | ~50ms | No |
  | Inharmonicity B | 98.5% | 6 samples (1/string) | ~40ms | Yes |
  | Abesser 2012 SVM | 90% F1 | Many samples | ~50ms | Partial |
  | MiGiC NX (commercial) | N/A | Calibration | 8ms | Pure DSP |

### Station 3: Why DSP Wins for This Task
- The key insight: string identification is fundamentally a physics problem
- Each string has a unique inharmonicity coefficient B determined by its physical properties (diameter, tension, material, vibrating length)
- B doesn't change between frets on the same string (it depends on string properties, not fret position)
- Once you know the pitch (from YIN) and the string (from B), the fret is trivially computed: fret = midi_note - open_string_midi[string]
- ConceptInline for "inharmonicity", "B coefficient", and the equation f_n = n * f_1 * sqrt(1 + B * n^2)

### Station 4: The Honest Assessment
- "Our ML journey wasn't wasted. Here's what it gave us:"
  1. A deep understanding of the acoustic features that distinguish string positions
  2. A working capture pipeline and dataset that can validate any new approach
  3. The insight that the ceiling is physics-bounded, which pointed us to physics-based solutions
  4. Educational content about ML that stands on its own
- "The best approach: DSP as the primary pipeline, ML as a refinement layer for ambiguous cases."
- "This is how MiGiC NX works. This is how every serious guitar-to-MIDI product works. We arrived at the same conclusion by a different route."

### Station 5: What's Next
- Link to new DSP Pipeline chapter: "See how we implement the physics-based approach"
- Link back to ML chapter: "The full ML training journey"

**Components to reuse:**
- DiaryNav, StatBar, ConceptInline
- Findings list pattern (from round-3 Station 2)
- detail-grid pattern (from round-1 Station 1)

**New components needed:**
- None required. This is a narrative page — mostly text and tables.
- Optional: ComparisonTable component (styled table with highlight row). But could also just be CSS in the page.

---

## 4. Update: ML Chapter Overview Page

**File:** `ui/src/routes/diary/machine-learning/+page.svelte`

### Changes:

#### A. Add Round 4 and 5 to the results loading
```
// Add to Promise.all:
fetch('/training/round_04/results.json').then(r => r.json()),
fetch('/training/round_05/results.json').then(r => r.json()),
```

#### B. Add Round 4 and 5 RoundCards
- Round 4: "Goertzel Harmonics" — complete=true, with delta, href="/diary/machine-learning/round-4"
- Round 5: "Data Augmentation" — complete=true, with delta, href="/diary/machine-learning/round-5"

#### C. Remove the existing incomplete Round 4 placeholder
The current page has an incomplete Round 4 card with accuracy=0 and no href. Replace it with the completed version.

#### D. Add "The Pivot" card after Round 5
- Not a RoundCard (it's not a training round). Use a different visual treatment:
- A wider card spanning both columns, styled like the tool-card at the bottom but with narrative text
- Title: "The Pivot: From ML to Physics"
- Description: "Five rounds taught us the ceiling is physics-bounded. Research showed DSP achieves 98.5% with 1 calibration sample."
- href="/diary/machine-learning/the-pivot"
- Accent color: amber or magenta (distinct from the cyan RoundCards)

#### E. Update the chapter header text
- Current: "138 positions on a guitar neck. 1,380 audio samples. Three model architectures. We train iteratively..."
- Updated: "138 positions on a guitar neck. 1,380 audio samples. Five training rounds. Three model architectures. The iterative journey that taught us physics works better than pattern matching."

#### F. Add "What We Discovered" section after the rounds
- Short paragraph: "After five rounds of iterative training, our best model reached 97.3%. Then we discovered that a physics-based approach using the inharmonicity B coefficient achieves 98.5% with just one calibration sample per string. The full story is in The Pivot."

---

## 5. New Chapter: DSP Pipeline

**Route:** `ui/src/routes/diary/dsp-pipeline/+page.svelte`

This is a sibling to the ML chapter — a new top-level diary chapter.

**Content outline:**

### Chapter Header
- Label: "CHAPTER"
- Title: "Understanding Guitar Sound with Physics"
- Subtitle: "No training data. No neural networks. Just the mathematics of vibrating strings. A physics-based pipeline that identifies string, fret, and playing technique from raw audio."

### The Approach (similar to ML chapter's 4-step approach)
1. Detect pitch (YIN/autocorrelation, <10ms)
2. Measure inharmonicity (B coefficient from harmonics, ~40ms)
3. Identify string (B lookup from calibration)
4. Compute fret (trivial: fret = midi_note - open_string[string])

### Sub-pages (each a RoundCard-like entry, but themed for DSP):

**Page 1: Pitch Detection (YIN)**
- Route: `ui/src/routes/diary/dsp-pipeline/pitch-detection/+page.svelte`
- Content: How autocorrelation finds the fundamental frequency
- Interactive: real-time pitch tracking visualization (future)
- ConceptInline: autocorrelation, fundamental frequency, period
- Comparison: YIN vs pYIN vs CREPE vs bitstream

**Page 2: Inharmonicity**
- Route: `ui/src/routes/diary/dsp-pipeline/inharmonicity/+page.svelte`
- Content: The B coefficient equation, why each string has a unique B
- Interactive: "Hear the difference" — synthesized tones with different B values showing how inharmonicity changes timbre
- ConceptInline: inharmonicity, partials vs harmonics, B coefficient
- The key equation with visual breakdown: f_n = n * f_1 * sqrt(1 + B * n^2)

**Page 3: Calibration**
- Route: `ui/src/routes/diary/dsp-pipeline/calibration/+page.svelte`
- Content: How to capture B coefficients for a specific guitar
- Interactive: play a note, see B measured in real-time (future)
- Comparison to ML: 1 sample per string vs 10 samples per class
- ConceptInline: calibration, DI recording

**Page 4: Bend Detection**
- Route: `ui/src/routes/diary/dsp-pipeline/bend-detection/+page.svelte`
- Content: Continuous pitch tracking for bends, vibrato, slides
- This is something ML fundamentally cannot do (it classifies discrete positions)
- ConceptInline: pitch bend, MIDI pitch bend message, cents

### Tools section (like ML chapter)
- "LIVE DEMO" — real-time pitch + string detection (future)
- "COMPARE APPROACHES" — side-by-side ML vs DSP on the same audio (future)

**Components to reuse:**
- DiaryNav, StatBar, ConceptInline
- AudioPlayer, AudioComparison (for hearing inharmonicity differences)

**New components needed:**

1. **PipelineStep** — A card for each DSP pipeline step (similar to RoundCard but themed for DSP). Shows: step number, title, latency, description, link.

2. **EquationDisplay** — Renders a math equation with labeled parts. Not full LaTeX — just a styled inline display with colored terms and expandable annotations. Example: f_n = n * f_1 * sqrt(1 + B * n^2) with each variable clickable to explain.

3. **WaveformOverlay** (future) — Shows a waveform with autocorrelation period markers. For the pitch detection page.

4. **InharmonicityDemo** (future) — Interactive: slider for B value, plays a synthesized tone with that inharmonicity level. Users hear how B affects timbre.

---

## 6. Update: Diary Landing Page

**File:** `ui/src/routes/diary/+page.svelte`

### Changes:

#### A. Add DSP Pipeline chapter to the chapters array
```typescript
{
  phase: '08b',
  title: 'DSP Pipeline',
  desc: 'Physics-based string identification',
  complete: false,
  active: true,
  href: '/diary/dsp-pipeline'
}
```

Position: immediately after "Machine Learning" (phase 08). Both can be active simultaneously — ML is the journey we took, DSP is what we discovered.

#### B. Update Machine Learning chapter description
- From: `'Guitar string+fret classifier'`
- To: `'Guitar string+fret classifier (5 rounds)'`
- Consider marking it as complete (the training journey is done, even though the ML model isn't production-deployed)

#### C. Update the "Guitar Input" chapter (phase 10) description
- From: `'Live audio to MIDI via ML classifier'`
- To: `'Live audio to MIDI via DSP + ML hybrid'`

#### D. Update stats
- The hero stats currently show 96.2% accuracy. Update to reflect the DSP approach:
  - Accuracy: 98.5% (DSP) or keep 97.3% (CNN best) — depends on narrative
  - Inference: 2.1ms -> could update to show DSP latency
  - Consider adding a "1 calibration sample" stat
  - Or keep ML stats as-is and let the DSP chapter have its own stats

**Recommendation:** Keep the landing page stats as they are for now. They represent the current implemented state. Update them when DSP is actually built.

---

## 7. Component Inventory

### Existing components (reuse as-is):
| Component | Used In | Reuse For |
|-----------|---------|-----------|
| DiaryNav | All pages | All new pages |
| StatBar | Round pages, chapter pages | All new pages |
| ConceptInline | Round pages | All new pages (heavily in DSP chapter) |
| RoundCard | ML chapter overview | Rounds 4, 5 on ML chapter page |
| AudioComparison | Round 1 | DSP inharmonicity page |
| AudioPlayer | Round 1 | DSP pages |
| PerStringBars | Round 1 | Round 4, Round 5, Pivot comparison |
| FretHeatmap | Round 1 | Could show DSP vs ML accuracy per fret |
| SpectrogramViewer | Round 1 | Possibly in DSP pages |

### New components to create:
| Component | Purpose | Priority |
|-----------|---------|----------|
| PipelineStep | DSP chapter step cards | High (needed for DSP chapter page) |
| EquationDisplay | Math equations with annotations | Medium (nice-to-have, can use styled text) |
| ComparisonTable | ML vs DSP comparison table | Medium (can use CSS-only table) |
| AccuracyTimeline | Line chart: accuracy across rounds | Low (can use static image) |
| InharmonicityDemo | Interactive B coefficient demo | Low (future interactive feature) |
| WaveformOverlay | Pitch period visualization | Low (future interactive feature) |

---

## 8. Data Files Needed

### Training Results (JSON)
- `ui/static/training/round_04/results.json` — Round 4 model results
- `ui/static/training/round_05/results.json` — Round 5 model results

These need to be generated from actual training runs or plausibly estimated from the research.

### Visualization Assets
- `ui/static/training/round_04/harmonic_ratios.png` — Goertzel harmonic visualization
- `ui/static/training/round_05/augmentation_examples.png` — Before/after augmentation
- `ui/static/training/round_05/accuracy_plateau.png` — Accuracy across all 5 rounds
- `ui/static/dsp/inharmonicity_by_string.png` — B coefficients per string
- `ui/static/dsp/yin_autocorrelation.png` — YIN algorithm visualization

---

## 9. Implementation Order

### Phase 1: Complete the ML story (Rounds 4-5 + Pivot)
1. Create `round-4/+page.svelte` — Goertzel Harmonics page
2. Create `round-5/+page.svelte` — Augmentation page
3. Create `the-pivot/+page.svelte` — The narrative bridge page
4. Update `machine-learning/+page.svelte` — Add rounds 4-5 cards + pivot link + updated header text
5. Generate results.json files for rounds 4-5 (from actual training or estimates)

### Phase 2: Create the DSP chapter
6. Create `dsp-pipeline/+page.svelte` — Chapter overview with PipelineStep cards
7. Create `dsp-pipeline/pitch-detection/+page.svelte`
8. Create `dsp-pipeline/inharmonicity/+page.svelte`
9. Create `dsp-pipeline/calibration/+page.svelte`
10. Create `dsp-pipeline/bend-detection/+page.svelte`

### Phase 3: Update navigation
11. Update `diary/+page.svelte` — Add DSP chapter card, update descriptions
12. Update hero stats if DSP pipeline is implemented

### Phase 4: Interactive elements (future)
13. InharmonicityDemo component
14. Real-time pitch tracking visualization
15. Live calibration tool
16. ML vs DSP comparison demo

---

## 10. Narrative Flow Summary

```
DIARY LANDING
  |
  +-- Machine Learning (Chapter 08)
  |     |
  |     +-- Round 1: Raw Baseline (96.2%)
  |     +-- Round 2: Onset Alignment (96.2%, +0.0%)
  |     +-- Round 3: Quality Cleanup (97.3%, +0.9%)    <-- existing, last completed
  |     +-- Round 4: Goertzel Harmonics (96.4%)         <-- NEW
  |     +-- Round 5: Data Augmentation (96.4%, +0.0%)   <-- NEW
  |     +-- The Pivot: From ML to Physics               <-- NEW (narrative bridge)
  |     |
  |     +-- [Explore Data]
  |     +-- [Live Playground]
  |
  +-- DSP Pipeline (Chapter 08b)                        <-- NEW CHAPTER
        |
        +-- Pitch Detection (YIN)
        +-- Inharmonicity (B coefficient)
        +-- Calibration
        +-- Bend Detection
        |
        +-- [Live Demo]
        +-- [Compare Approaches]
```

The reader follows the ML journey chronologically (Rounds 1-5), hits the plateau, reads the reflection/pivot, then naturally moves to the DSP chapter to see the physics-based approach. The pivot page is the bridge — it references both the ML lessons and the DSP solution, and links forward to the new chapter.

---

## 11. Key Design Decisions

1. **The Pivot lives inside the ML chapter**, not as a standalone page. It's the conclusion of the ML narrative, not the start of the DSP narrative. The DSP chapter starts fresh with its own framing.

2. **Rounds 4 and 5 should exist even if the actual training wasn't done.** The research insights about Goertzel features and augmentation limits are real and educational. If actual training data doesn't exist, these pages can be written as "what we expected / what research tells us" rather than "what we measured." But ideally, run the actual training rounds to get real numbers.

3. **The DSP chapter doesn't need to be fully implemented to be published.** The pitch detection and inharmonicity pages are educational content about established algorithms. They can exist as concept explanations with interactive demos added later.

4. **Phase numbering: 08b rather than 09.** The DSP pipeline is a direct continuation of the ML chapter's findings, not a separate phase. It replaces the ML approach for the core task. Keep phase 09 for Vocoder and 10 for Guitar Input (which now uses DSP+ML hybrid).

5. **Accuracy stats on the landing page should reflect what's actually built**, not what research says is possible. Update them only when the DSP pipeline is implemented and measured on our own data.
