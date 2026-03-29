# Learnings from the 8-Reviewer ML Pipeline Audit

This documents what we learned from having 4 principal engineers + 4 evil twins review the guitar classifier pipeline.

---

## The Reviewers

| Role | What They Reviewed | Key Insight |
|------|-------------------|-------------|
| **Principal ML Architect** | Model architecture, training methodology | "Benchmark the heuristic before building ML" |
| **ML Evil Twin** | Challenged the Architect | "Overfitting IS the strategy for a personal model" |
| **Principal Audio ML Engineer** | Audio features, spectrogram params | "FFT resolution matters for frequency discrimination" |
| **Audio Evil Twin** | Challenged Audio ML | "DI signal path makes most audio ML concerns irrelevant" |
| **Principal Data Engineer** | Data pipeline, storage, integrity | "Label validation is the ONE thing that matters at this scale" |
| **Data Evil Twin** | Challenged Data Engineer | "A single MessagePack file with serde is the right choice, not a database" |
| **Principal MLOps Engineer** | Deployment, inference, latency | "500ms correction is musically unacceptable. ONNX has no WASM backend." |
| **MLOps Evil Twin** | Challenged MLOps | "Pure Rust inference with include_bytes! is the entire deployment" |

---

## Unanimous Findings (All 8 Agreed)

These are the highest-confidence findings — when both the expert and their adversary agree, it's signal:

### 1. The O(n) Ring Buffer Bug
`Vec::remove(0)` on the audio thread does a memmove of 88KB per sample at 44.1kHz = 3.9 GB/s of memory copies. **Every reviewer flagged this.** Fix: use VecDeque or a proper ring buffer.

**Lesson:** Never use Vec as a FIFO queue. VecDeque exists for a reason. In real-time audio, O(n) operations on the callback thread cause sample drops.

### 2. Pitch Validation is Essential
The capture tool labeled plucks based on what it *told* you to play, not what was *detected*. With 13,800 samples, even a 2% error rate = ~276 mislabeled samples, which is 2 per class — enough to measurably degrade accuracy.

**Lesson:** Ground truth labels should be prompted position (what you intended to play), but validated by the pitch detector (what was actually heard). Reject mismatches.

### 3. Onset-Forward, Not Onset-Backward
The ring buffer snapshot captured 500ms *ending* near the pluck. The attack transient (most discriminative) was in the last 10% of the window. The model would learn from 90% irrelevant audio.

**Lesson:** Start recording AFTER onset detection, not before. The attack is at the beginning of a pluck — put it at the beginning of the capture.

### 4. 139 Classes is Right, Not 7
If you collapse to 7 classes (6 strings + noise), the ML model does less than the existing heuristic. 139 classes (string + fret) is the only granularity that justifies the complexity.

**Lesson:** Don't simplify the problem to make the model easier to train. Simplify the model to match the data you have.

---

## Where Experts Fought (Open Questions)

### Overfitting: Problem or Strategy?
- **Principal:** "25K parameters on 14K samples will memorize, not generalize."
- **Evil Twin:** "The training distribution IS the deployment distribution. Memorization is the goal."

**Resolution:** For a personal model on one guitar, moderate overfitting is acceptable. But still need a test set to catch catastrophic failures. Use 80/20 split, not 100% training.

### CNN vs Simpler Models
- **Principal:** "Random Forest on hand-crafted features will outperform CNN at this data scale."
- **Evil Twin:** "CNN captures attack transient patterns that hand-crafted features miss."

**Resolution:** Build both. Run them as an ensemble — RF on Goertzel features (fast, physics-informed) + CNN on attack spectrogram (learns what RF can't). Compare and combine.

### Pre-trained Audio Models
- **Principals:** "Consider transfer learning from PANNs/AST."
- **Evil Twins:** "Catastrophically wrong. 80M params on 14K samples = memorization in 3 epochs. 320MB model for a web app. No WASM support."

**Resolution:** Pre-trained models from AudioSet are the wrong domain. Self-supervised pre-training on YOUR data (for future multi-guitar transfer) is worth designing for but not building yet.

---

## Key Technical Lessons

### DI Signal Path Changes Everything
Most audio ML literature assumes microphone recordings with room acoustics, background noise, and source separation challenges. A DI guitar signal through an Audient iD14 is:
- Extremely clean (no room, no mic coloring)
- Highly consistent (same guitar = same signal)
- Rich in harmonics (pickup directly transduces string vibration)

This means: HPSS is irrelevant, aggressive noise augmentation is counterproductive, and hand-crafted harmonic features work better than end-to-end learning.

### Goertzel > Mel-Spectrogram for String Identification
The Goertzel filter bank directly computes energy at specific harmonic frequencies — exactly what distinguishes strings physically. A mel-spectrogram is a general-purpose representation that the model must learn to interpret. With limited data, giving the model pre-computed physics features wins.

### Real-Time Audio Constraints
- **Onset detection must be <10ms** — this is the trigger path. ML can't be in this path.
- **ML classification needs 300-500ms of audio** — too slow for triggering, fine for correction/refinement.
- **500ms correction is musically unacceptable** — if ML corrects a wrong note after 500ms, the user hears the wrong note ring for half a second. ML correction must happen within ~50ms or it's useless.
- **ONNX Runtime has no WASM backend** — pure Rust inference is the only option for web deployment.

### Data > Model Complexity
With 100 plucks per position (13,800 samples), the data is rich enough for multiple approaches. More data with natural playing variation (different pick angles, attack strengths) is better than fewer samples with synthetic augmentation. The real-world variation in 100 plucks captures what no augmentation algorithm can simulate.

### Safe vs Dangerous Augmentations
| Augmentation | Safe? | Why |
|---|---|---|
| Gain variation ±3dB | Yes | Simulates pick attack variation |
| Noise injection (real captured noise) | Yes | Simulates environmental noise |
| Time jitter ±10ms | Yes | Simulates onset detection timing |
| Elastic time warp ±5% | Yes | Simulates playing speed variation |
| Pitch shift ≥0.5 semitones | **NO** | Changes the fret class — corrupts labels |
| SpecAugment freq masking | **NO** | Can mask the exact harmonic that distinguishes strings |
| Mixup/CutMix | **NO** | Creates impossible intermediate classes |

### Normalization Matters
Per-sample z-normalization destroys loudness information — makes quiet noise look like loud plucks. Use global normalization (train set mean/std) to preserve relative amplitudes.

---

## Process Lesson: Evil Twins Are Valuable

The evil twin pattern (adversarial review of the review) caught several places where standard ML wisdom was wrong for our specific case:
- "More general is better" → wrong for a personal model
- "End-to-end learning is always superior" → wrong with limited data and known physics
- "Use pre-trained models" → wrong for this domain/deployment
- "ONNX is the deployment standard" → wrong for WASM

**Takeaway:** When reviewing ML pipelines, always consider whether the advice applies to YOUR specific constraints (data size, deployment target, user count, domain knowledge available).
