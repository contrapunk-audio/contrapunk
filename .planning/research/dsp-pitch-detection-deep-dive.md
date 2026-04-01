# DSP-Based Guitar Pitch Detection: Deep Dive Research Report

> Generated 2026-03-31. Companion to `guitar-pitch-detection-research.md`.
> Focus: implementation-level details extracted from actual papers and codebases.

---

## Table of Contents

1. [Derrien 2014 -- ESPRIT-Based Very Low Latency Pitch Tracker](#1-derrien-2014--esprit-based-very-low-latency-pitch-tracker)
2. [Cycfi Bitstream Autocorrelation & Q Library](#2-cycfi-bitstream-autocorrelation--q-library)
3. [McLeod Pitch Method (MPM)](#3-mcleod-pitch-method-mpm)
4. [pYIN -- Probabilistic YIN](#4-pyin--probabilistic-yin)
5. [Spotify Basic Pitch](#5-spotify-basic-pitch)
6. [CREPE -- CNN Pitch Estimation](#6-crepe--cnn-pitch-estimation)
7. [Abesser 2012 -- Automatic String Detection](#7-abesser-2012--automatic-string-detection)
8. [pitch-detection Rust Crate & pitchlite WASM](#8-pitch-detection-rust-crate--pitchlite-wasm)
9. [Comparative Analysis for Contrapunk](#9-comparative-analysis-for-contrapunk)
10. [Implementation Roadmap](#10-implementation-roadmap)

---

## 1. Derrien 2014 -- ESPRIT-Based Very Low Latency Pitch Tracker

**Paper:** "A Very Low Latency Pitch Tracker for Audio to MIDI Conversion"
**Author:** Olivier Derrien, Universite de Toulon & CNRS LMA
**Venue:** DAFx-14, September 2014
**PDF:** https://hal.science/hal-01089401/document

### 1.1 Core Idea

Unlike YIN (time-domain, non-parametric), Derrien uses a **parametric** approach. Since a guitar pickup produces quasi-harmonic sound with very low noise, a parametric sinusoidal model is justified. The algorithm has two stages:

1. **ESPRIT** -- extract sinusoidal components (partials) from the signal
2. **Statistical f0 estimator** -- find the most probable fundamental frequency from the partials

### 1.2 The ESPRIT Algorithm (Stage 1)

The signal is modeled as a sum of K Exponentially Damped Sinusoids (EDS):

```
x[n] = sum_{k=0}^{K-1} alpha_k * z_k^n + w[n]
```

Where:
- `alpha_k = a_k * exp(i * phi_k)` -- complex amplitude (amplitude + phase)
- `z_k = exp(-d_k + 2*i*pi*nu_k)` -- poles (damping + normalized frequency)
- `w[n]` -- Gaussian white noise

**Algorithm steps:**

1. Form the Hankel signal matrix X from the input buffer (N samples)
   - X is R x Q where Q = N/2, R = N/2 + 1
2. Compute SVD of X: `X = [U1 U2] * diag(Sigma1, Sigma2) * [V1; V2]`
   - U1, Sigma1 correspond to the K largest singular values (signal space)
   - U2, Sigma2 correspond to the remaining singular values (noise space)
3. Exploit the **shift invariance property**: `U1_down * Phi = U1_up`
   - Where U1_down discards last row, U1_up discards first row
4. Solve: `Phi = pinv(U1_down) * U1_up`
5. **Eigenvalues of Phi give the poles z_k** -- from which you get frequencies and dampings
6. Recover amplitudes: `alpha = pinv(Z^N) * x`

**Model order K:** Constant K=6 is a good default for guitar. Variable K from 4-12 using the modified ESTER criterion (equation 11 in paper) for automatic model order selection. Bass notes benefit from higher K (more harmonics observable).

**Periodicity measure:** The modified ESTER criterion J multiplied by signal energy:
```
J = (K-1)^2 / ||U1_up - U1_down * Phi||^2
```
High J = good match between signal and harmonic model. Used for note-on/note-off gating.

### 1.3 Statistical f0 Estimation (Stage 2)

Given extracted partials at frequencies f_k, find the most probable f0 using a likelihood function:

```
L(f0) = [product over m in M: g(f_m^h / f0 - m)] * P_S(f0) * P_E(f0)
```

Where:
- f_m^h is the "harmonic partial" closest to m*f0 in interval [(m-0.5)*f0, (m+0.5)*f0]
- g is a Gaussian: `g(x) ~ exp(-x^2 / sigma^2)` with **sigma = 1/8**
- P_S penalizes "supplementary partials" (extra partials not near any harmonic): `P_S = (1 - N_S/K)^alpha_S`, **alpha_S = 8**
- P_E penalizes empty intervals (expected harmonics missing): `P_E = (1 - N_E/M)^alpha_E`, **alpha_E = 4**

**Key octave-error resistance:** A wrong octave generates either extra supplementary partials (octave too low) or empty intervals (octave too high), both lowering the likelihood. This is a built-in anti-octave-error mechanism.

**Implementation shortcut:** Test only discrete f0 values on the tempered scale (not continuous), which is much faster.

### 1.4 Concrete Latency Numbers (On Guitar)

| Metric | ESPRIT | YIN |
|--------|--------|-----|
| Minimum buffer length N | 260 samples | 300 samples |
| At 11.025 kHz sample rate | 23.6 ms | 27.2 ms |
| Raw MIDI note correct (E2, lowest) | **12 ms** after onset | 20 ms after onset |
| Periodicity threshold crossed (E2) | **12 ms** after onset | **35 ms** after onset |
| Periodicity threshold crossed (E4) | 12-24 ms (shelf) | ~12 ms (sharper onset for high notes) |
| Theoretical minimum latency | 12.5 ms (1/80 Hz) | 25 ms (2/80 Hz) |

**Critical finding:** ESPRIT achieves latency very close to the theoretical minimum (one period of the lowest pitch = 12.5 ms for E2 at 80 Hz). YIN needs approximately two periods.

**Ghost note comparison:** ESPRIT's periodicity measure is much more stable than YIN's. YIN's periodicity fluctuates, making threshold-based note segmentation unreliable (ghost notes). ESPRIT allows clean thresholding at 100 (vs 60 for YIN), significantly reducing ghost notes.

**Tradeoff on high notes:** For E4, ESPRIT shows a periodicity "shelf" between 12-24 ms that could cause ghost notes. YIN's periodicity onset is sharper for high-pitched notes.

### 1.5 Complexity

- Naive ESPRIT: O(N^3) -- too slow for real-time
- Fast ESPRIT (Badeau 2005): **O(K*N*log(N))** where K~6, N<300
- At these sizes, this is comparable to a single FFT
- With high-overlap adaptive ESPRIT, complexity can be reduced further
- **Not tested in real-time in this paper** (offline Matlab only)

### 1.6 Implementability Assessment for Contrapunk

**Pros:**
- Achieves near-theoretical-minimum latency on guitar
- Excellent periodicity measure (clean note segmentation, fewer ghost notes)
- Naturally avoids octave errors via the likelihood function
- Parametric model is perfectly suited for DI guitar (clean, quasi-harmonic signal)

**Cons:**
- Complex to implement (SVD, eigendecomposition, Hankel matrices)
- No existing Rust implementation
- Paper only tested offline; real-time viability is theoretical
- Requires downsampling to 11.025 kHz (which is fine for guitar range)

**Verdict:** Interesting for a second-generation pitch tracker, but too complex for v1. Start with YIN/MPM, revisit ESPRIT if latency is unsatisfactory.

---

## 2. Cycfi Bitstream Autocorrelation & Q Library

**Sources:**
- https://www.cycfi.com/2018/03/fast-and-efficient-pitch-detection-bitstream-autocorrelation/
- https://www.cycfi.com/2017/10/fast-and-efficient-pitch-detection/
- https://github.com/cycfi/q

> Note: The Cycfi blog posts use JavaScript-heavy rendering that prevents full content extraction. Details below are reconstructed from the Q library source code, README, and blog post summaries available in secondary sources.

### 2.1 The Bitstream Autocorrelation Concept

Standard autocorrelation computes:
```
R(tau) = sum_{n=0}^{N-1-tau} x[n] * x[n+tau]
```
This is O(N * tau_max) with floating-point multiplications.

**Bitstream autocorrelation** replaces this with:

1. **1-bit conversion:** Convert the signal to a binary bitstream using zero-crossing detection. Each sample becomes 0 (negative) or 1 (positive).
2. **Pack into machine words:** 64 samples per uint64 on a 64-bit machine.
3. **Autocorrelation via XNOR + POPCNT:**
   ```
   R_bit(tau) = sum POPCNT(word_n XNOR word_{n+tau})
   ```
   - XNOR gives 1 where both bits are equal (both positive or both negative)
   - POPCNT counts the 1s in a machine word -- single CPU instruction, executes in 1 cycle
   - This computes 64 sample correlations in a single instruction pair

### 2.2 Performance Advantage

For a 4096-sample buffer:
- Standard autocorrelation: ~4096 * 1024 = 4M floating-point multiplies
- Bitstream: ~64 * 16 = 1K POPCNT+XNOR operations (64x fewer words, plus bit-level parallelism)
- **Theoretical speedup: ~100-1000x** over naive autocorrelation
- Even vs FFT-based autocorrelation, bitstream can be faster for small N

### 2.3 Limitations of 1-Bit Conversion

- Loses all amplitude information -- cannot distinguish loud from quiet partials
- Harmonic structure is preserved only in the zero-crossing pattern
- More susceptible to noise (a small noise spike can flip a zero crossing)
- Inharmonicity information (needed for string ID) is degraded
- Not suitable as the sole input for string identification

### 2.4 Q Library Status (2024-2026)

The Q library is undergoing a major transition:
- **v1.x:** Used BACF (Bitstream Autocorrelation) as the primary pitch detector
- **v1.5+:** Retiring BACF in favor of a **new "Hz system"** pitch detector with integrated onset detection
- The Hz system is described in "Pitch Perfect: Enhanced Pitch Detection Techniques (Part 1)"

**Architecture:**
- `q_lib`: Header-only C++ DSP library, zero dependencies, runs on microcontrollers
- `q_io`: Cross-platform audio/MIDI I/O via PortAudio/PortMidi
- Uses modern C++ with functional composition patterns

**Key insight:** Even the creator of bitstream autocorrelation moved away from it to a "much better" algorithm. This suggests BACF alone is not sufficient for production-quality pitch detection.

### 2.5 Implementability for Contrapunk

**Pros:**
- Extremely fast -- could enable sub-millisecond pitch detection on each audio callback
- Simple to implement (zero-crossing + POPCNT)
- Good for a "first guess" that gets refined

**Cons:**
- The original author deprecated it in favor of a better approach
- Loses amplitude/harmonic info needed for string ID
- More noise-sensitive than standard autocorrelation
- No ready-made Rust implementation

**Verdict:** Interesting as a pre-filter or fast initial estimate, but not sufficient as the primary pitch detector. Use standard autocorrelation (YIN/MPM) instead.

---

## 3. McLeod Pitch Method (MPM)

**Paper:** "A Smarter Way to Find Pitch" (2005)
**Authors:** Philip McLeod, Geoff Wyvill (University of Otago)
**Software:** Tartini pitch analysis tool

### 3.1 How MPM Differs from YIN

Both YIN and MPM are autocorrelation-based, but they differ in normalization and peak selection.

**YIN** uses the Cumulative Mean Normalized Difference Function (CMND):
```
d'(tau) = d(tau) / [(1/tau) * sum_{j=1}^{tau} d(j)]
```
This produces dips at pitch periods. You look for the first dip below a threshold (typically 0.1-0.2).

**MPM** uses the Normalized Square Difference Function (NSDF):
```
NSDF(tau) = 2 * r(tau) / (sum_{n=0}^{N-tau-1} x[n]^2 + x[n+tau]^2)
```

Where r(tau) is the standard autocorrelation. The NSDF:
- Ranges from -1 to +1 (like a correlation coefficient)
- Has clear peaks at pitch periods (value near +1)
- The normalization accounts for the decreasing energy at larger lags

### 3.2 Key Innovation: Peak Picking via Key Maximums

MPM introduces the concept of **key maximums**:

1. Find all positive peaks in the NSDF
2. A "key maximum" is the highest point between consecutive zero crossings
3. Apply parabolic interpolation around each key maximum for sub-sample precision
4. Select the first key maximum that exceeds a **clarity threshold** (typically 0.9)
5. If multiple peaks exceed the threshold, prefer the first (lowest-lag = highest frequency) unless a later peak has significantly higher clarity

**Why this matters for guitar:** The first key maximum at the fundamental period typically has the highest clarity. Octave errors occur when harmonics produce higher peaks -- MPM's ordered peak search with a high threshold naturally avoids this.

### 3.3 Parabolic Interpolation

For sub-sample precision, MPM interpolates around the peak at index p:
```
delta = (NSDF[p-1] - NSDF[p+1]) / (2 * (NSDF[p-1] - 2*NSDF[p] + NSDF[p+1]))
tau_refined = p + delta
```

This gives pitch accuracy better than 1 cent even at modest sample rates.

### 3.4 Benchmarks from pitch-detection Crate

From the sevagh/pitch-detection repository testing on degraded audio (Viola E3 at 164.81 Hz, 26 x 0.1s slices):

| Degradation Level | MPM Correct | YIN Correct |
|-------------------|-------------|-------------|
| 0 (clean) | **26/26** | 22/26 |
| 1 | 23/26 | 21/26 |
| 2 | 19/26 | 21/26 |
| 3 | 18/26 | 19/26 |
| 4 | 19/26 | 19/26 |
| 5 | 18/26 | 19/26 |

**Key finding:** MPM is better on clean audio; YIN is slightly more robust to heavy degradation. For DI guitar (clean signal from Audient iD14), MPM's clean-audio advantage is relevant.

### 3.5 MPM vs YIN Summary

| Aspect | MPM | YIN |
|--------|-----|-----|
| Normalization | NSDF (-1 to +1) | CMND (dips toward 0) |
| Peak type | Peaks (positive peaks) | Dips (valleys below threshold) |
| Clarity metric | Built-in (peak height) | Derived from "aperiodicity" |
| Clean audio accuracy | Slightly better | Slightly worse |
| Noisy audio robustness | Slightly worse | Slightly better |
| Octave error handling | First key maximum | Cumulative mean trick |
| Complexity | O(N log N) via FFT | O(N log N) via FFT |
| Latency | Same (needs ~2 periods) | Same (needs ~2 periods) |

**Verdict for Contrapunk:** MPM is the better choice for DI guitar. The `pitch-detection` Rust crate implements both. Start with MPM, fall back to YIN for noisy conditions.

---

## 4. pYIN -- Probabilistic YIN

**Paper:** "pYIN: A Fundamental Frequency Estimator Using Probabilistic Threshold Distributions"
**Authors:** Matthias Mauch, Simon Dixon (Queen Mary University of London)
**Venue:** ICASSP 2014
**PDF:** https://www.eecs.qmul.ac.uk/~simond/pub/2014/MauchDixon-PYIN-ICASSP2014.pdf

> Note: PDF was not successfully fetched, but the algorithm is well-documented in secondary sources and the pitch-detection crate implements it.

### 4.1 Core Innovation: Multiple Pitch Candidates

Standard YIN picks a single pitch per frame using a fixed threshold. pYIN instead:

1. **Try multiple thresholds** (beta distribution over [0,1])
2. For each threshold, YIN may produce a different pitch candidate
3. Weight each candidate by the probability of that threshold
4. Result: a **probability distribution over pitch** for each frame, not a single estimate

### 4.2 HMM Viterbi Decoding

The per-frame pitch distributions become observations in a Hidden Markov Model:

- **States:** discretized pitch values (e.g., 480 values from 55-1760 Hz at 10-cent resolution)
- **Observations:** the probability distribution from the multi-threshold YIN at each frame
- **Transition model:** penalizes large pitch jumps between frames (smoothness prior)
- **Viterbi decoding:** finds the globally optimal pitch sequence

This temporal smoothing:
- Eliminates isolated octave errors (an octave jump that immediately returns is penalized)
- Handles vibrato gracefully (smooth pitch variation is allowed)
- Resolves ambiguous frames where multiple pitches are plausible

### 4.3 Why pYIN is Better for Guitar

| Scenario | YIN Problem | pYIN Solution |
|----------|-------------|---------------|
| Note attack | Chaotic signal, wrong pitch | HMM smooths through attack |
| Vibrato | Pitch fluctuates, may trigger octave error | HMM tracks smooth modulation |
| Bend release | Rapid pitch change | HMM follows the trajectory |
| String resonance | May detect wrong fundamental | HMM penalizes sudden jumps |
| Quiet decay | Signal-to-noise drops, errors increase | HMM keeps previous pitch estimate |

### 4.4 Available Implementations

- **C++ (pitch-detection):** sevagh/pitch-detection implements PYIN. 228/228 tests passing.
- **Rust:** The pitch-detection Rust crate does NOT implement pYIN directly. The C++ library does.
- **pyin-rs:** https://github.com/Sytronik/pyin-rs -- dedicated Rust pYIN implementation, but WASM compatibility is unclear.

### 4.5 Latency Impact

pYIN adds latency because Viterbi decoding is non-causal (needs future frames for optimal decode). In practice:
- Real-time pYIN uses a **look-ahead of 2-5 frames** (20-50 ms at typical hop sizes)
- This is on top of the base YIN latency
- Total: ~40-80 ms for full pYIN with Viterbi

**For Contrapunk:** pYIN is excellent for the continuous pitch tracking / bend detection path but too slow for the initial note-on trigger. Use plain MPM/YIN for the fast note trigger, then refine with pYIN for smooth pitch tracking.

---

## 5. Spotify Basic Pitch

**Paper:** "A Lightweight Instrument-Agnostic Model for Polyphonic Note Transcription and Multipitch Estimation"
**Authors:** Rachel M. Bittner, Juan Jose Bosch, David Rubinstein, Gabriel Meseguer-Brocal, Sebastian Ewert
**Venue:** ICASSP 2022
**Code:** https://github.com/spotify/basic-pitch
**TypeScript:** https://github.com/spotify/basic-pitch-ts

### 5.1 Model Architecture (from source code)

**Input:** Raw audio waveform, shape `(batch, AUDIO_N_SAMPLES, 1)`

**Feature extraction:**
1. Harmonic Constant-Q Transform (CQT) -- not a standard spectrogram
2. Batch normalization
3. Harmonic stacking (captures multiple octaves of each frequency)

**Three output heads:**

| Head | Filters | Kernel | Stride | Activation | Purpose |
|------|---------|--------|--------|------------|---------|
| Contour | 8 | (3, 39) | (1, 1) | Sigmoid | Continuous pitch (for bends) |
| Note | 32 | (7, 7) | (1, 3) | Sigmoid | Discrete note presence |
| Onset | 32 | (5, 5) | (1, 3) | Sigmoid | Note attack detection |

**Architecture flow:**
```
Audio -> CQT -> BatchNorm -> HarmonicStacking
    |
    +-> Conv2D(8, 3x39) -> BN -> ReLU -> Conv2D(1) -> Sigmoid -> CONTOUR
    |
    +-> [Contour features] -> Conv2D(32, 7x7, stride 1x3) -> ReLU -> Conv2D(1) -> Sigmoid -> NOTE
    |
    +-> Conv2D(32, 5x5, stride 1x3) -> BN -> ReLU
        + [Note features concatenated]
        -> Conv2D(1) -> Sigmoid -> ONSET
```

### 5.2 Key Specifications

| Spec | Value |
|------|-------|
| Parameters | **< 17,000** |
| Peak memory | < 20 MB |
| Internal sample rate | 22,050 Hz (auto-resampled) |
| Input channels | Mono (auto-downmixed) |
| Speed | Faster than real-time on most hardware |
| Supported formats | MP3, OGG, WAV, FLAC, M4A |
| Model formats | TensorFlow, CoreML, TFLite, **ONNX** |
| Output formats | MIDI (with pitch bend), CSV, NPZ, sonified WAV |
| Pitch range | ~30 Hz to ~4000 Hz (covers full guitar range) |
| Polyphony | Yes (multipitch) |
| Instrument-agnostic | Yes (guitar, piano, vocals, etc.) |

### 5.3 Pitch Bend Detection

Basic Pitch uniquely supports **pitch bend detection** via the contour head:
- The contour output provides a continuous pitch estimate per frame
- Post-processing: `addPitchBendsToNoteEvents()` converts contour to MIDI pitch bend messages
- This captures vibrato, glissando, slides, and bends
- Most competing AMT systems output only discrete notes

### 5.4 TypeScript / Browser Version

The `basic-pitch-ts` package:
- Uses ONNX model loaded in-browser
- Processes via Web Audio API (`AudioContext.decodeAudioData()`)
- API: `new BasicPitch(model)` -> `evaluateModel(audioBuffer, callbacks)`
- Exports utilities: `outputToNotesPoly()`, `addPitchBendsToNoteEvents()`, `noteFramesToTime()`
- Handles any audio size/length (streaming windowed processing for large files)
- Auto-resamples to 22,050 Hz, mono

**Bundle size:** Not explicitly documented, but the model itself is ~7-8 MB ONNX. Total with runtime likely 10-15 MB.

### 5.5 How Basic Pitch Could Fit Contrapunk

**Option A: Replace the custom CNN entirely**
- Basic Pitch handles pitch detection, onset detection, AND pitch bend
- 17K params vs our custom CNN which is likely larger
- Already has ONNX and TypeScript versions
- Handles polyphony (future chord support)

**Option B: Use as the "slow path" complement to DSP fast path**
- DSP (MPM/YIN) provides instant note-on trigger (< 15 ms)
- Basic Pitch runs in parallel, providing:
  - String disambiguation (via multi-note context)
  - Pitch bend tracking (contour head)
  - Onset refinement
  - Polyphonic support

**Option C: Use contour output only for pitch bend**
- Keep our DSP pitch + ML string ID pipeline
- Use Basic Pitch's contour model for high-quality pitch bend detection

### 5.6 Limitations for Contrapunk

1. **No string identification:** Basic Pitch outputs notes, not string+fret. We still need string ID.
2. **Latency unknown:** "Faster than real-time" is about throughput, not latency. The CQT + CNN inference adds significant latency (likely 50-100ms).
3. **Instrument-agnostic = guitar-unoptimized:** A guitar-specific model could outperform it on guitar.
4. **22 kHz sample rate:** Loses information above 11 kHz. Fine for pitch but may lose timbral cues for string ID.

---

## 6. CREPE -- CNN Pitch Estimation

**Paper:** "CREPE: A Convolutional Representation for Pitch Estimation" (2018)
**Authors:** Jong Wook Kim, Justin Salamon, Peter Li, Juan Pablo Bello
**Code:** https://github.com/marl/crepe

### 6.1 Architecture

- **Input:** 1024 samples at 16 kHz (64 ms window), time-domain waveform directly
- **Network:** 6-layer CNN
- **Output:** 360-dimensional vector (pitch bins spanning 20 cents each, ~32.7 Hz to 1975.5 Hz)
- **Pitch extraction:** argmax-local weighted averaging around peak activation

### 6.2 Model Variants

| Variant | Parameters | Speed | Accuracy |
|---------|-----------|-------|----------|
| Full | Large | Slow | Best |
| Large | Medium | Medium | Very good |
| Medium | Medium | Medium | Good |
| Small | Small | Fast | Good |
| **Tiny** | **Smallest** | **Fastest** | **Adequate** |

### 6.3 Performance

- **Outperforms** pYIN and SWIPE on multiple benchmarks (as of 2018)
- Default hop size: 10 ms (100 frames/second)
- Internal resampling: 16 kHz
- Outputs: timestamps, frequency (Hz), voicing confidence (0-1)

### 6.4 Relevance to Contrapunk

CREPE is **monophonic only** and requires a neural network inference per frame. The Tiny model could run in real-time, but:
- No ONNX export (TensorFlow/Keras)
- No Rust implementation
- No string ID capability
- 64 ms input window = high latency for note onset

**Verdict:** Inferior to Basic Pitch for our use case (Basic Pitch handles polyphony, has ONNX, has pitch bend). Inferior to DSP for latency. Skip for now.

---

## 7. Abesser 2012 -- Automatic String Detection

**Paper:** "Automatic String Detection for Bass Guitar and Electric Guitar"
**Author:** Jakob Abesser (Fraunhofer IDMT)
**Venue:** CMMR 2012

### 7.1 The 48 Features (Not 541)

Contrary to the earlier research document's claim of "541 features", Abesser 2012 uses **48 features** organized in 6 categories:

| Feature Category | Dimension | Description |
|-----------------|-----------|-------------|
| Inharmonicity coefficient beta_hat | 1 | Estimated from partial frequencies via polynomial fit |
| Relative partial amplitudes {a_r,k} | 15 | Amplitude of first 15 partials relative to fundamental |
| Statistics over {a_r,k} | 8 | max, min, mean, median, mode, variance, skewness, kurtosis |
| Normalized partial frequency deviations {delta_f_norm,k} | 15 | How much each partial deviates from ideal harmonic |
| Statistics over {delta_f_norm,k} | 8 | max, min, mean, median, mode, variance, skewness, kurtosis |
| Partial amplitude slope s_a | 1 | Linear regression slope of amplitude vs harmonic number |

**Total: 48 features**

### 7.2 Inharmonicity Estimation

The key physical equation for string inharmonicity:
```
f_k = k * f0 * sqrt(1 + beta * k^2)    for k >= 1
```

Where beta depends on string physical properties (Young's modulus, radius of gyration, tension, length). **Beta is different for each string** because string diameter varies (0.1-0.41 mm for electric guitar).

**Estimation procedure:**
1. From the AR spectral model, extract frequencies f_k of first 15 partials
2. Compute `(f_k / f0)^2 = k^2 + beta * k^4`
3. Fit a 4th-order polynomial: `(f_k / f0)^2 ~ p_0 + p_1*k + p_2*k^2 + p_3*k^3 + p_4*k^4`
4. Estimate: `beta_hat = p_4`

### 7.3 Most Discriminative Features (Feature Ranking via IRMFSP)

| Rank | Bass Guitar | Electric Guitar |
|------|-------------|-----------------|
| 1 | delta_f_norm,9 | delta_f_norm,15 |
| 2 | beta_hat | mean{a_r,k} |
| 3 | delta_f_norm,3 | var{delta_f_norm,k} |
| 4 | var{delta_f_norm,k} | max{a_r,k} |
| 5 | a_r,4 | skew{delta_f_norm,k} |

**Key insight:** The **normalized partial frequency deviations** (how much each harmonic deviates from the ideal) are the most discriminative features for both instruments. The inharmonicity coefficient beta is #2 for bass. The deviation pattern encodes the string's physical characteristics more richly than beta alone.

### 7.4 Classification Pipeline

1. **Spectral Modeling:** AR model with modified covariance method (least-squares)
   - Downsampled to 5.5 kHz (bass) / 10.1 kHz (guitar) to capture first 15 harmonics
   - Block size N=256, hop H=64
2. **Note Detection:** Find first frame after attack (where process variance sigma^2 peaks)
3. **Feature Extraction:** Extract 48 features from first 5 frames after note onset
4. **Dimensionality Reduction:** LDA to N_strings - 1 dimensions (3 for bass, 5 for guitar)
5. **Classification:** SVM with RBF kernel, grid-search optimized C and gamma
6. **Plausibility Filter:** Zero out probabilities for strings that can't physically play the detected pitch
7. **Frame Aggregation:** Sum probabilities over 5 adjacent frames, then argmax

### 7.5 Results

| Configuration | Bass (4 strings) | Electric (6 strings) |
|---------------|------------------|----------------------|
| Best F-measure | **0.93** | **0.90** |
| Baseline (MFCC + SVM) | 0.46 | 0.37 |
| Without plausibility filter | 0.85-0.92 | 0.63-0.72 |
| Without frame aggregation | 0.85-0.87 | 0.64-0.70 |

**Critical finding:** The plausibility filter and frame aggregation each provide ~5-8% improvement. MFCCs alone are terrible for string detection (0.37 F-measure on guitar).

### 7.6 Dataset

- 1034 isolated note recordings
- 2 bass guitars + 2 electric guitars
- 2 playing styles (plectrum, fingerstyle)
- 2 pickup settings (neck, bridge)
- Evaluated with cross-validation across instruments, techniques, and pickup settings

### 7.7 Relevance to Contrapunk

**What we can use directly:**
1. The 48-feature design is a good template for our feature extraction
2. Plausibility filter is trivially implementable (just a string-fret lookup table)
3. Frame aggregation (voting over 5 frames) significantly improves accuracy
4. The feature ranking tells us what to prioritize: normalized frequency deviations > inharmonicity > amplitude ratios

**What differs for us:**
- We use DI guitar (Audient iD14), not amplified -- signal is cleaner
- We have only 1 guitar, not cross-instrument generalization
- We can calibrate per-guitar (Abesser tested cross-instrument)
- Our CNN approach learns features automatically; Abesser's hand-crafted features are more interpretable

---

## 8. pitch-detection Rust Crate & pitchlite WASM

### 8.1 pitch-detection Crate

**Repository:** https://github.com/sevagh/pitch-detection
**Rust API:** https://docs.rs/pitch-detection

**Available detectors:**

| Detector | Method | Best For |
|----------|--------|----------|
| `AutocorrelationDetector` | Standard autocorrelation | General purpose |
| `McLeodDetector` | MPM (Normalized Square Difference) | **Accuracy-efficiency balance** |
| `YINDetector` | YIN (Cumulative Mean Normalized Difference) | **Monophonic audio** |

**Note:** The C++ version also has PYIN and PMPM (Probabilistic MPM), but the Rust crate only exposes the three above.

**API:**
```rust
use pitch_detection::detector::mcleod::McLeodDetector;
use pitch_detection::detector::PitchDetector;

let mut detector = McLeodDetector::<f32>::new(buffer_size, padding);
let pitch = detector.get_pitch(
    &audio_samples,
    sample_rate,
    power_threshold,  // minimum signal power
    clarity_threshold, // 0.0-1.0, MPM peak height threshold
);

if let Some(pitch) = pitch {
    println!("Frequency: {} Hz, Clarity: {}", pitch.frequency, pitch.clarity);
}
```

**Return type:** `Option<Pitch>` with `frequency: f32` and `clarity: f32`

**Dependencies:** Only `rustfft ^6.0.1` at runtime. Pure Rust, no C bindings.

**WASM compatibility:** Designed for WASM. The `rustfft` dependency is pure Rust and compiles to WASM.

### 8.2 pitchlite -- WASM Real-Time Pitch Detection

**Repository:** https://github.com/sevagh/pitchlite

**Architecture:**
- Ring buffer of 4096 samples
- Subdivided into 512-sample chunks (8 subdivisions)
- AudioWorklet sends 128 samples at a time (Web Audio API standard)
- Returns 9 simultaneous pitch values: 8 sub-chunk pitches + 1 overall

**Key specs:**
- Algorithm: McLeod Pitch Method (MPM)
- FFT: KissFFT (real FFTs via r2c/c2r)
- Buffer: 4096 samples total, 512 per sub-chunk
- At 44.1 kHz: 4096/44100 = **92.8 ms total buffer**, 512/44100 = **11.6 ms per sub-chunk**
- Sub-chunk detection: gives temporal pitch evolution within the larger buffer

**Limitation:** The web implementation is described as "low quality and only an example."

### 8.3 Recommended Configuration for Contrapunk

For 44.1 kHz sampling (Audient iD14):

| Parameter | Value | Rationale |
|-----------|-------|-----------|
| Detector | McLeodDetector | Best for clean DI guitar |
| Buffer size | 2048 samples (46.4 ms) | Covers ~3.5 periods of E2 (82 Hz) |
| Padding | 1024 | Zero-padding for FFT efficiency |
| Hop size | 128 samples (2.9 ms) | Match AudioWorklet quantum |
| Power threshold | 0.01 | Reject silence |
| Clarity threshold | 0.7-0.9 | Tune empirically. Higher = fewer false positives |
| Expected latency | ~23 ms for E2, ~5 ms for E4 | Depends on where in buffer the onset falls |

---

## 9. Comparative Analysis for Contrapunk

### 9.1 Contrapunk's Specific Requirements

| Requirement | Detail |
|-------------|--------|
| Input | DI electric guitar, Audient iD14, mono 44.1/48 kHz |
| Signal quality | Clean, high SNR, quasi-harmonic |
| Pitch range | E2 (82 Hz) to E6 (1319 Hz) -- full guitar range |
| Monophonic | Primary use case is single-note playing |
| Need string ID | Yes -- for tablature generation |
| Need pitch bend | Yes -- for expressive MIDI control |
| Target latency | < 15 ms for note onset, < 50 ms for string ID |
| Platforms | Desktop (Tauri/Rust) + Browser (WASM) |

### 9.2 Algorithm Comparison Matrix

| Algorithm | Note Onset Latency | String ID | Pitch Bend | Complexity | WASM Ready | Rust Crate |
|-----------|-------------------|-----------|------------|------------|------------|------------|
| **MPM** | ~23 ms (E2) | No | Via tracking | O(N log N) | Yes | `pitch-detection` |
| **YIN** | ~24 ms (E2) | No | Via tracking | O(N log N) | Yes | `pitch-detection` |
| **pYIN** | ~40-80 ms | No | Yes (HMM) | O(N log N + Viterbi) | Maybe | `pyin-rs` |
| **ESPRIT** | **~12 ms (E2)** | No | Via tracking | O(KN log N) | No | None |
| **Bitstream AC** | ~5 ms (fast) | No | No | O(N) bit-ops | No | None |
| **Basic Pitch** | ~50-100 ms | No | Yes (contour) | CNN inference | Yes (TS) | No |
| **CREPE Tiny** | ~64 ms | No | No | CNN inference | No | No |
| **Inharmonicity B** | ~40 ms | **Yes** | No | O(K*N) | Implementable | None |
| **Abesser 48-feat** | ~50 ms | **Yes** | No | AR model + SVM | Implementable | None |

### 9.3 Recommended Architecture

```
Audio Input (44.1 kHz, mono, DI guitar)
    |
    v
[Buffer: 2048 samples, hop 128]
    |
    +---> [MPM Pitch Detection]  .............. ~15-23 ms
    |         |
    |         +-> frequency (Hz)
    |         +-> clarity (confidence)
    |         +-> MIDI note = 69 + 12*log2(freq/440)
    |         +-> Note-on when clarity crosses threshold
    |         +-> Note-off when clarity drops or energy drops
    |
    +---> [Onset Detection]  .................. ~5 ms
    |         |
    |         +-> Energy envelope spike
    |         +-> Confirms note-on trigger
    |
    +---> [Inharmonicity Estimation]  ......... ~40 ms (parallel)
    |         |
    |         +-> AR spectral model (first 15 harmonics)
    |         +-> Estimate beta from harmonic frequencies
    |         +-> Compare to calibrated per-string beta values
    |         +-> String number (0-5)
    |         +-> Fret = MIDI_note - open_string_MIDI[string]
    |
    +---> [Continuous Pitch Tracking]  ........ ongoing
              |
              +-> Frame-by-frame MPM/pYIN
              +-> Frequency trajectory -> pitch bend MIDI
              +-> Detect vibrato, slides, bends

FUSION TIMELINE:
  t=0      Note onset (energy spike)
  t=15ms   Pitch detected (MPM), MIDI note-on sent
  t=40ms   String identified, MIDI updated with string routing
  t=...    Continuous pitch bend tracking ongoing
```

### 9.4 Where Basic Pitch Fits

Basic Pitch is **not a replacement for the DSP pipeline** but a powerful complement:

| Role | DSP Pipeline | Basic Pitch |
|------|-------------|-------------|
| Note onset | MPM (15 ms) | Too slow (~50-100 ms) |
| Pitch accuracy | MPM/pYIN (< 1 cent) | Good but overkill |
| String ID | Inharmonicity B | Cannot do this |
| Pitch bend | Continuous MPM tracking | Contour head (good) |
| Polyphonic | Not yet | Yes (future) |
| Offline analysis | Overkill | Perfect |
| WASM playground | DSP is better for real-time | TypeScript version for non-real-time |

**Recommendation:** Use Basic Pitch for the non-real-time WASM playground (offline analysis, transcription). Use DSP for the real-time Tauri desktop app.

---

## 10. Implementation Roadmap

### Phase 1: Core Pitch Detection (Immediate)

**Goal:** Real-time note-on/note-off with MIDI output

1. **Add `pitch-detection` crate** to Rust dependencies
2. **Implement McLeod detector** with:
   - Buffer: 2048 samples at 44.1 kHz
   - Hop: 128 samples (match AudioWorklet)
   - Clarity threshold: 0.8 (tune empirically)
3. **Frequency-to-MIDI conversion:** `midi = 69 + 12 * log2(freq / 440)`
4. **Note-on/off gating:**
   - Note-on: clarity > threshold AND energy > minimum AND stable for 2+ frames
   - Note-off: clarity < threshold OR energy < minimum for 3+ frames
5. **WASM build:** `pitch-detection` compiles to WASM via `wasm-pack`

**Expected result:** ~15-23 ms note onset latency, MIDI note output, works in both Tauri and browser.

### Phase 2: String Identification (Week 2)

**Goal:** Determine which string a note was played on

1. **Implement AR spectral estimation:**
   - Modified covariance method (Abesser 2012)
   - Extract first 15 harmonic frequencies from the AR model
2. **Estimate inharmonicity B:**
   - Polynomial fit: `(f_k/f0)^2 ~ p_0 + p_1*k + p_2*k^2 + p_3*k^3 + p_4*k^4`
   - `beta = p_4`
3. **Calibration mode:**
   - User plays each open string once
   - Record B coefficient for each string
   - Store as per-guitar calibration
4. **String classification:**
   - Compare estimated B to calibrated values
   - Plausibility filter: zero out strings that can't play the detected pitch
   - Confidence threshold: reject if B doesn't match any string well
5. **Fret derivation:** `fret = midi_note - open_string_midi[string_number]`

**Expected result:** ~40 ms string ID after note onset, 95%+ accuracy with per-guitar calibration.

### Phase 3: Pitch Bend Tracking (Week 3)

**Goal:** Continuous pitch-to-MIDI-bend conversion

1. **Frame-by-frame MPM tracking** at ~3 ms hop
2. **Pitch deviation from nearest semitone:** `bend = 8192 + (cents_deviation / 200) * 8192`
3. **Smoothing:** Low-pass filter on the pitch trajectory (remove jitter)
4. **Vibrato detection:** periodic modulation of pitch (> 4 Hz, < 8 Hz, < 50 cents)
5. **Bend detection:** monotonic pitch change > 50 cents

### Phase 4: Refinements (Week 4+)

1. **Add pYIN** for smoother continuous tracking (use `pyin-rs` or implement from scratch)
2. **Implement Abesser's 48 features** as an alternative/complement to B-only string ID
3. **Frame aggregation** for string ID: vote over 5 consecutive frames
4. **Evaluate Basic Pitch** for the WASM playground (offline transcription mode)
5. **Consider ESPRIT** if the ~23 ms MPM latency proves too high for the low strings

### Rust Crate Dependencies

| Crate | Purpose | WASM? |
|-------|---------|-------|
| `pitch-detection` | MPM, YIN, autocorrelation | Yes |
| `rustfft` | FFT for pitch detection (transitive dep) | Yes |
| `pyin-rs` | Probabilistic YIN | Needs verification |
| `basic-pitch` (Python/TS) | Offline transcription | TypeScript version for browser |

### Key Implementation Parameters

```rust
// Pitch detection
const SAMPLE_RATE: usize = 44100;
const BUFFER_SIZE: usize = 2048;     // 46.4 ms - covers ~3.8 periods of E2
const HOP_SIZE: usize = 128;         // 2.9 ms - matches WebAudio quantum
const POWER_THRESHOLD: f32 = 0.01;   // Reject silence
const CLARITY_THRESHOLD: f32 = 0.80; // MPM peak confidence threshold
const MIN_FREQ: f32 = 75.0;          // Below E2, ignore
const MAX_FREQ: f32 = 1400.0;        // Above E6, ignore

// String ID
const NUM_HARMONICS: usize = 15;     // First 15 partials for inharmonicity
const AR_ORDER: usize = 60;          // AR model order (2*N/3 where N=90)
const AR_BLOCK_SIZE: usize = 256;    // Block size for AR estimation
const AR_HOP_SIZE: usize = 64;       // Hop size for AR estimation
const AR_DOWNSAMPLE_RATE: usize = 11025; // Downsample for guitar frequency range

// Calibration
const OPEN_STRING_MIDI: [u8; 6] = [40, 45, 50, 55, 59, 64]; // E2 A2 D3 G3 B3 E4
// B coefficients stored per-guitar after calibration
```

---

## Summary of Key Findings

1. **MPM is the best starting point** for Contrapunk's real-time pitch detection. It outperforms YIN on clean DI guitar, has a Rust crate that compiles to WASM, and provides a built-in clarity metric. Expected latency: ~15-23 ms for the lowest guitar notes.

2. **ESPRIT achieves the theoretical minimum latency** (~12 ms for E2) but is complex to implement and has no Rust crate. Worth revisiting in v2 if MPM's latency is insufficient.

3. **Inharmonicity B estimation is the most efficient string identifier.** Only needs 1 calibration sample per string, ~40 ms latency, and 98.5% accuracy (Hjerrild 2019). Abesser's 48-feature approach provides a richer but slower alternative.

4. **The most discriminative features for string ID** are the normalized partial frequency deviations (how much each harmonic deviates from ideal), not MFCCs or spectral centroid. This is directly implementable with an AR spectral model.

5. **Spotify Basic Pitch** (17K params, ONNX, TypeScript) is ideal for offline/non-real-time use (WASM playground transcription) but not a replacement for the real-time DSP pipeline due to latency. Its contour head provides excellent pitch bend detection.

6. **pYIN adds robustness** at the cost of latency (40-80 ms total). Best used for continuous pitch tracking rather than note onset detection.

7. **Bitstream autocorrelation was deprecated by its own creator** in favor of better algorithms. Not recommended as a primary approach.

8. **The plausibility filter** (from Abesser) is a free accuracy boost: simply zero out string ID probabilities for strings that physically cannot play the detected pitch. Trivial to implement with a lookup table.
