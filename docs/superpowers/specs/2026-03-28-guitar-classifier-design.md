# Guitar String+Fret Classifier — Design Spec

**Date:** 2026-03-28
**Guitar:** Ibanez Artcore AG85 (hollow body, 22 frets)
**Interface:** Audient iD14 (DI input)
**Goal:** Classify which string (0-5) AND fret (0-22) produced a pluck, plus noise rejection. 139 classes total.

---

## 1. Data Capture

### What
- 6 strings x 23 positions (open + 22 frets) x 100 plucks = **13,800 pluck samples**
- 6 noise categories x 30 seconds each = **~300+ noise samples**
- 2-3 minutes raw noise recording for augmentation injection
- Each sample: **500ms audio, onset-forward** (attack at the beginning)
- Capture time: ~11.5 hours across multiple sessions

### How
- `examples/guitar_capture.rs` — guided fretboard walk
- Manual advance (Enter between positions)
- Pitch validation gate: reject plucks where `|detected - expected| > 1 semitone`
- VecDeque ring buffer (O(1) on audio thread)
- Playback verification, reset/skip/quit per position
- Guitar name stored in metadata (per-guitar models)
- Dataset appends across sessions via MessagePack

### Noise Categories
1. **Ambient** — room noise, fan, cable hum (30s)
2. **String muting** — palm on strings, shifting hand (30s)
3. **Pick scrapes** — scrape along strings, tap body (30s)
4. **Finger slides** — slide up/down neck without plucking (30s)
5. **String brushes** — brush across strings without clean plucks (30s)
6. **Accidental touches** — bump strings, fret without plucking (30s)

### Storage
- Format: MessagePack (`guitar_training_data.msgpack`)
- Size: ~600MB for full dataset
- Location: project root, `.gitignore`d
- Metadata: guitar name, sample rate, device, channel, capture date

---

## 2. Data Processing

Every step produces visualizations + concept explainers in `ml/processing/`.

### Pipeline
1. **Raw analysis** — waveform grids, amplitude histograms, clipping/silence detection
2. **Onset alignment** — detect onset within each sample, align all samples to attack start
3. **Feature extraction** (multiple views per sample):
   - Full mel-spectrogram (FFT=2048, hop=512, 64 mel bins, 60-8000Hz)
   - Goertzel harmonic features (first 10 harmonics, ratios, centroid, inharmonicity) = 15 features
   - Attack transient (first 50ms mel-spectrogram)
   - Temporal envelope (RMS contour over time)
4. **Normalization** — global (training set mean/std)
5. **Noise injection augmentation**:
   - Load captured raw noise
   - For each clean pluck, generate 4 noisy copies at SNR 30/20/15/10 dB
   - Additional: gain ±3dB, time jitter ±10ms, elastic time warp ±5%
   - **NO pitch shift, NO SpecAugment**
   - 13,800 clean → ~69,000 total pluck samples
6. **Dataset validation**:
   - Class balance bar chart
   - t-SNE/UMAP of Goertzel features
   - Mel-spectrogram grids per class
   - Confusion zone analysis (which string/fret combos share pitch)

### Visual Output at Every Step
Each processing step writes to `ml/processing/NN_step_name/`:
- `WHAT_IS_HAPPENING.md` — plain English + physics/math explanation
- Visualization PNGs showing what the step does to the data
- Before/after comparisons

---

## 3. Training

Three models trained on the same data, compared head-to-head. All results documented in `ml/training/results/`.

### Model A: Random Forest on Goertzel Features
- **Input:** 15 Goertzel harmonic features
- **Architecture:** 500 trees, max_depth=20, min_samples_leaf=3
- **Parameters:** N/A (decision trees)
- **Why try:** Works well with limited data, interpretable, fast inference, zero dependencies, works in WASM
- **Training time:** <1 minute on CPU

### Model B: Hybrid CNN (Goertzel + Attack Spectrogram)
- **Input A:** 15 Goertzel features (physics-informed)
- **Input B:** Attack spectrogram (64 mel bins x ~5 time frames from first 100ms)
- **Architecture:**
  ```
  Goertzel features → Linear(15 → 32) → ReLU
  Attack spectrogram → Conv2d(1→16) → BN → ReLU → Pool
                     → Conv2d(16→16) → BN → ReLU → AdaptiveAvgPool → 16-dim
  Concatenate (32 + 16 = 48) → Linear(48 → 64) → ReLU → Dropout(0.3) → Linear(64 → 139)
  ```
- **Parameters:** ~5-8K
- **Why try:** Combines physics knowledge (Goertzel) with learned features (attack shape)
- **Training time:** 5-10 minutes on CPU

### Model C: Pure CNN on Full Mel-Spectrogram
- **Input:** (64, ~22) mel-spectrogram
- **Architecture:**
  ```
  Conv2d(1→16, 3x3) → BN → ReLU → MaxPool(2)
  Conv2d(16→32, 3x3) → BN → ReLU → MaxPool(2)
  Conv2d(32→32, 3x3) → BN → ReLU → MaxPool(2)
  AdaptiveAvgPool(1,1) → Dropout(0.3) → Linear(32→139)
  ```
- **Parameters:** ~25K
- **Why try:** Baseline comparison — does end-to-end learning beat hand-crafted features?
- **Training time:** 5-10 minutes on CPU

### Training Setup (shared)
- Optimizer: Adam, lr=1e-3
- Epochs: 100, early stopping patience 15
- Split: 80/20 stratified random (20% test = random samples per class)
- Loss: CrossEntropyLoss
- Regularization: Dropout 0.3, BatchNorm, early stopping
- Augmentation: noise injection + gain + time jitter (applied on-the-fly)

### Ensemble
After individual training, combine the best two:
```
Audio frame
  ├──→ Goertzel features → Model A (RF) → prediction + confidence
  ├──→ Goertzel + attack → Model B (Hybrid CNN) → prediction + confidence
  └──→ Ensemble:
       If both agree → high confidence output
       If disagree → use higher confidence
       If both low → reject as noise
```

### Training Visualization
- Train/val loss + accuracy curves (per epoch)
- Per-class F1 scores
- Confusion matrix (139-class, grouped by string)
- Per-fret accuracy heatmap (string x fret)
- Worst predictions (highest-loss samples for debugging)
- Feature importance (RF: which Goertzel features matter)
- Confidence calibration plot
- Model comparison table (accuracy, F1, params, inference time)

---

## 4. Deployment

### Inference: Pure Rust (No ONNX)
- Export trained weights as binary blob
- Implement forward pass in pure Rust (~80-100 lines for the hybrid CNN)
- RF: export as nested if/else or serialized tree structure
- Embed weights via `include_bytes!` — zero filesystem dependency
- Works on native (Tauri) AND WASM (browser) with same code
- Inference time: <1ms per classification

### Integration: Three Detection Modes

The user chooses between three detection modes in the UI:

**Mode 1: Heuristic Only** (existing behavior)
```
Audio → PluckDetector (onset) → pitch detect → identify_string() → note on
```
Fast, no model needed, works out of the box.

**Mode 2: ML Only** (the goal)
```
Audio → PluckDetector (onset) → accumulate 100ms audio
      → extract Goertzel features + attack spectrogram
      → classifier → string+fret prediction → note on
```
More accurate, requires trained model for the specific guitar.

**Mode 3: Both** (heuristic + ML ensemble)
```
Audio → PluckDetector (onset) → heuristic fires immediately (fast path)
                               → ML classifies in parallel
                               → if agree: confirmed
                               → if disagree: correct within 50ms
                               → if ML low confidence: keep heuristic
```
Maximum reliability, graceful degradation.

**Default:** Mode 2 when a trained model exists, auto-fallback to Mode 1 when no model is available.

### Feature Extraction Parity
Critical: the Goertzel features must be computed identically in training (Python) and inference (Rust). Solution: compute features in Rust during capture, store them alongside raw audio. Training script loads pre-computed features. No Python/Rust feature extraction drift.

---

## 5. Per-Guitar Model Management

- Each guitar gets its own dataset + model: `ibanez_artcore_ag85.msgpack`, `ibanez_artcore_ag85.model`
- Guitar name stored in dataset metadata and model metadata
- When switching guitars: run capture tool again, retrain
- Future: self-supervised pre-training on all guitar data, fine-tune per guitar

---

## 6. Visual Learning App (SvelteKit)

A dedicated SvelteKit app for visualizing every step of the ML pipeline — from raw audio to trained model. Not deferred — built alongside the pipeline so every step is visible as it's developed.

### Step-by-Step Visualization Pages

**Page 1: Raw Data Explorer**
- Browse all captured samples by string/fret
- Waveform display for each sample — click to play audio
- Amplitude histogram per string
- Per-sample metadata (RMS, peak, confidence, detected vs expected MIDI)
- Filter by: string, fret, validated/rejected, confidence range

**Page 2: Onset Alignment**
- Before/after waveform comparison — raw capture vs onset-aligned
- Onset detection visualization (where the algorithm found the pluck start)
- Distribution of onset positions across all samples
- Alignment quality check — are all attacks starting at sample 0?

**Page 3: Feature Extraction**
- For any selected sample, show side-by-side:
  - Raw waveform
  - Full mel-spectrogram (color heatmap)
  - Goertzel harmonic bar chart (h1 through h10 amplitudes)
  - Harmonic ratio profile (h2/h1, h3/h1, ..., h10/h1)
  - Attack transient spectrogram (first 50ms zoomed in)
  - Temporal envelope (RMS over time curve)
  - Spectral centroid trajectory
  - Inharmonicity measurement
- Compare features across: same string different frets, different strings same note, pluck vs noise

**Page 4: Normalization**
- Before/after spectrogram comparison for selected samples
- Feature distribution plots (histograms) before and after global normalization
- Show how normalization affects different strings differently
- Verify noise samples stay distinguishable after normalization

**Page 5: Augmentation**
- For any clean sample, show all augmented versions:
  - Original (clean)
  - SNR 30dB (barely noisy) — waveform + spectrogram
  - SNR 20dB (moderate) — waveform + spectrogram
  - SNR 15dB (noisy) — waveform + spectrogram
  - SNR 10dB (very noisy) — waveform + spectrogram
  - Gain +3dB / -3dB variants
  - Time-jittered variants
- Play each augmented version back
- Dataset size growth visualization (before/after bar chart)

**Page 6: Dataset Validation**
- Class balance bar chart (all 139 classes + noise)
- t-SNE/UMAP scatter plot — colored by string, click to hear samples
- t-SNE/UMAP per string — colored by fret, see if frets separate
- Confusion zone map — which string/fret combos share the same pitch
- Outlier detection — samples that cluster with the wrong class
- Audio quality summary (clipping, silence, low-confidence counts)

**Page 7: Training**
- Live (or replayed) training curves: loss + accuracy per epoch for all 3 models
- Visual explanation of what each model layer does:
  - CNN: show the convolutional filters, what patterns they detect
  - RF: show a single decision tree, which features it splits on
  - Hybrid: show both branches feeding into the fusion layer
- Per-epoch confusion matrix animation — watch the model learn
- "What does it mean for a model to fit a note?" — for a selected note:
  - Show the model's internal activations
  - Show which features the model uses to classify it
  - Show the training samples for that note and how the model's confidence grows

**Page 8: Model Comparison**
- Side-by-side accuracy/F1/latency for all 3 models
- Per-fret accuracy heatmap (string x fret) for each model
- Worst predictions for each model — play the audio, see why it failed
- Feature importance (RF: bar chart, CNN: gradient-based attribution)
- Confidence calibration plot for each model

**Page 9: Ensemble**
- Show how individual predictions combine
- Cases where models disagree — play the audio, see both predictions
- Before-ensemble vs after-ensemble accuracy comparison
- Noise rejection visualization — what the ensemble catches that individuals miss

**Page 10: Live Demo**
- Plug in your guitar, play live
- See real-time: waveform → spectrogram → Goertzel features → model prediction
- Switch between detection modes: heuristic / ML / both
- Side-by-side comparison of heuristic vs ML predictions in real time

---

## 7. Success Criteria

| Metric | Target | Measurement |
|--------|--------|-------------|
| String identification accuracy | >95% | Held-out test set |
| String+fret accuracy | >85% | Held-out test set |
| Noise rejection rate | >98% | Noise samples in test set |
| False trigger rate | <2% | Noise samples misclassified as plucks |
| Inference latency | <5ms | Benchmark in Rust |
| Model size | <500KB | Embedded weights |
| Works in WASM | Yes | Browser test |
| Beats heuristic | Yes | A/B on same test data |

---

## 8. File Structure

```
ml/
├── CONCEPTS.md                  — master glossary (already created)
├── REVIEW_LEARNINGS.md          — 8-reviewer audit findings (already created)
├── capture/
│   └── guitar_training_data.msgpack
├── processing/
│   ├── 01_raw_analysis/
│   ├── 02_onset_alignment/
│   ├── 03_feature_extraction/
│   ├── 04_normalization/
│   ├── 05_augmentation/
│   └── 06_validation/
├── training/
│   ├── train.py
│   ├── models/
│   │   ├── ARCHITECTURES.md
│   │   ├── hybrid_cnn.py
│   │   ├── pure_cnn.py
│   │   └── random_forest.py
│   ├── results/
│   │   ├── comparison/
│   │   ├── hybrid_cnn/
│   │   ├── pure_cnn/
│   │   └── random_forest/
│   └── checkpoints/
├── inference/
│   ├── src/classifier.rs        — pure Rust inference
│   └── weights/                 — exported weight files
└── app/                         — SvelteKit visual learning app
    ├── src/routes/
    │   ├── raw-data/            — Page 1: browse samples, play audio
    │   ├── onset/               — Page 2: onset alignment visualization
    │   ├── features/            — Page 3: feature extraction explorer
    │   ├── normalization/       — Page 4: before/after normalization
    │   ├── augmentation/        — Page 5: noise injection explorer
    │   ├── validation/          — Page 6: t-SNE, class balance, outliers
    │   ├── training/            — Page 7: live training visualization
    │   ├── comparison/          — Page 8: model comparison dashboard
    │   ├── ensemble/            — Page 9: ensemble behavior explorer
    │   └── live/                — Page 10: live guitar demo with mode switch
    └── static/                  — processed data for visualization
```

---

## 9. End-to-End Goal

The final deliverable is **guitar audio in → MIDI notes out** inside Contrapunk, with accurate string+fret identification powered by the trained model.

```
Guitar (Ibanez Artcore AG85)
  → Audient iD14 (DI input)
  → Contrapunk audio pipeline
  → PluckDetector (onset)
  → Classifier (trained model)
  → String + Fret identification
  → MIDI note with correct pitch
  → Harmony engine
  → MIDI output to DAW
```

The SvelteKit visual learning app is the **development, testing, and verification environment** — where you see every step of the pipeline, verify the model works, and test it with live audio before importing into Contrapunk.

### What gets imported into Contrapunk
- `src/audio/classifier.rs` — pure Rust inference code (~100 lines)
- `assets/models/ibanez_artcore_ag85.model` — trained weights (~500KB, embedded via `include_bytes!`)
- Detection mode selector in the UI (Heuristic / ML / Both)
- The model replaces or augments the existing `identify_string()` heuristic

### What stays in the SvelteKit app
- Training pipeline visualization
- Dataset exploration
- Model comparison dashboards
- Concept explainers
- Live demo with side-by-side heuristic vs ML

---

## 10. Open Items for Future Discussion

- **Pre-trained models:** Self-supervised pre-training on own data for multi-guitar transfer learning. Design for it now, build when we have 2+ guitars.
- **Visual learning app:** SvelteKit vs Streamlit decision. Deferred until training pipeline works.
- **Tekton CI integration:** Run accuracy/latency tests as Tekton Pipeline Tasks against reference audio.

## 10. Implementation Order

1. Fix capture tool (done — ring buffer, pitch validation, onset-forward, guitar name)
2. Scaffold SvelteKit visual learning app (build alongside pipeline, not after)
3. Capture first session of data (start with a few strings)
4. Build Python processing pipeline — each step feeds into the SvelteKit app
5. Benchmark existing heuristic on captured data (visible in app)
6. Continue capturing data across sessions until complete
7. Train 3 models — training visualization live in app
8. Compare models — comparison dashboard in app
9. Build ensemble — ensemble explorer in app
10. Implement pure Rust inference (all 3 modes: heuristic / ML / both)
11. Integrate mode selection into Contrapunk UI
12. Live demo page in SvelteKit app
