# Guitar Pitch Detection & String Identification — Research

## Context

Contrapunk needs real-time guitar audio → MIDI conversion with string identification. This document collects research on DSP-based and ML-based approaches for monophonic and polyphonic guitar pitch detection, string identification, and fret estimation.

## The Hybrid Architecture

```
Audio → Gain normalization → Two parallel paths:

PATH 1: Fast DSP (< 10ms)
  → YIN/autocorrelation pitch → frequency → MIDI note
  → Note onset detection → MIDI note-on with velocity
  → Continuous pitch tracking → MIDI pitch bend for bends
  → Note-off gate (filter previous resonance)
  → OUTPUT: instant MIDI note (which note, but not which string)

PATH 2: ML String ID (~50ms)
  → Inharmonicity parameter estimation (physical model)
  → OR mel-spectrogram → CNN → string+fret class
  → OUTPUT: which string, which fret

FUSION:
  → DSP gives the NOTE instantly
  → ML gives the STRING within 50ms
  → Combined: full tablature-accurate MIDI with string routing
```

## Key Research Papers

### Pitch Detection (Monophonic)

**CREPE: A Convolutional Representation for Pitch Estimation (2018)**
- Authors: Jong Wook Kim, Justin Salamon, Peter Li, Juan Pablo Bello
- Source: ICASSP 2018
- URL: https://arxiv.org/abs/1802.06182
- Method: 6-layer CNN operating directly on time-domain waveform (1024 samples at 16kHz)
- Output: 360-dimensional vector (pitch bins from 32.7Hz to 1975.5Hz)
- Accuracy: Outperforms YIN, pYIN, SWIPE on multiple datasets
- Variants: Full, Large, Medium, Small, Tiny (for real-time)
- Relevance: Could replace YIN as the fast-path pitch detector. Tiny model runs in real-time.

**YIN: A Fundamental Frequency Estimator (2002)**
- Authors: Alain de Cheveigné, Hideki Kawahara
- Source: Journal of the Acoustical Society of America
- Method: Modified autocorrelation with parabolic interpolation
- Latency: ~2 periods of the fundamental (for E2 at 82Hz: ~24ms)
- Relevance: Standard baseline for monophonic pitch detection. Fast, simple, well-understood.

**Fast and Efficient Pitch Detection (Cycfi Research, 2017)**
- URL: https://www.cycfi.com/2017/10/fast-and-efficient-pitch-detection/
- Method: Bitstream autocorrelation — converts signal to 1-bit, then autocorrelates
- Advantage: O(N) with bit-parallel operations, extremely fast
- Relevance: Potentially the fastest pitch detection for the fast path

### String & Fret Identification

**Estimation of Guitar String, Fret and Plucking Position Using Parametric Pitch Estimation (2019)**
- Authors: Daniel Hjerrild, Mads G. Christensen
- Source: ICASSP 2019
- URL: https://ieeexplore.ieee.org/document/8683408/
- Method: Physical model with inharmonicity parameter B
- Key equation: f_n = n * f_1 * sqrt(1 + B * n^2)
- B depends on string diameter, tension, vibrating length
- Needs only 1 training sample per string (not 10 per class like CNN)
- Works on 40ms segments — real-time capable
- Error rate: 1.5% on string+fret classification
- Also estimates plucking position
- Relevance: **Best approach for string identification.** Physically grounded, minimal training data, real-time.

**Automatic String Detection for Bass Guitar and Electric Guitar (2012)**
- Author: Jakob Abeßer
- Source: CMMR 2012
- URL: http://cmmr2012.eecs.qmul.ac.uk/sites/cmmr2012.eecs.qmul.ac.uk/files/pdf/papers/cmmr2012_submission_70.pdf
- Method: 541 spectral/cepstral/harmonic features → SVM
- F-measure: 0.93 (bass, 4 classes), 0.90 (guitar, 6 classes)
- Key features: inharmonicity, spectral centroid, MFCCs, harmonic ratios
- Relevance: Validates the ML approach. Our CNN with mel-spectrograms is a modern equivalent.

**Automatic Guitar String Detection by String-Inverse Frequency Estimation (2018)**
- URL: https://www.researchgate.net/publication/328602822
- Method: String-Inverse Frequencies (SIFs) — novel feature for string detection
- Uses the relationship between string physical properties and spectral content
- Relevance: Alternative feature to inharmonicity for string identification.

**Inharmonicity-Based Method for Automatic Generation of Guitar Tablature**
- Authors: Barbancho et al.
- URL: https://www.researchgate.net/publication/260691838
- Method: Inharmonicity coefficients mapped to string/fret lookup table
- Simultaneous pitch + string detection
- Relevance: Confirms inharmonicity as the primary cue for tablature generation.

**Physical Models for Fast Estimation of Guitar String, Fret and Plucking Position (2019)**
- URL: https://www.researchgate.net/publication/334593495
- Method: Extended physical model beyond Hjerrild 2019
- Faster estimation suitable for real-time
- Relevance: Improved version of the parametric approach.

**Real-time Guitar String Detection for Music Education Software**
- URL: https://www.researchgate.net/publication/261024961
- Focus: Real-time constraint for educational use
- Relevance: Proves real-time string detection is achievable.

### Polyphonic Pitch Detection

**Polyphonic Pitch Tracking with Neural Networks (2018)**
- URL: https://arxiv.org/pdf/1804.02918
- Method: Neural network on spectrogram input
- Challenge: Dense polyphonic textures (not our primary use case — guitar is mostly monophonic/limited polyphony)

**Real-Time Polyphonic Pitch Detection on Acoustic Musical Signals**
- URL: https://www.researchgate.net/publication/331205992
- Method: "Raking" method over frequency-domain spectra
- Accuracy: 83.20% with 140ms window
- Relevance: If we ever extend to chord detection.

**Harmonic Product Spectrum (HPS)**
- GitHub: https://github.com/joaocarvalhoopen/Polyphonic_note_detector_using_Harmonic_Product_Spectrum
- Method: Multiply downsampled spectra to find fundamentals
- Simple, fast, works for 2-3 simultaneous notes
- Relevance: Could complement single-note detection for chord playing.

### Guitar Physics

**Inharmonicity in Bass Guitar Strings (2020)**
- URL: https://link.springer.com/article/10.1007/s42452-020-2391-2
- Detailed physical model of inharmonicity for wound and plain strings
- B coefficient derivation from string properties
- Relevance: Foundation for the physical model approach.

**Guitar String Theory and Frequency Analysis**
- URL: https://makersportal.com/blog/2018/9/20/audio-processing-in-python-part-iii-guitar-string-theory-and-frequency-analysis
- Practical Python implementation of guitar frequency analysis
- Relevance: Good reference for implementation.

## DSP Approaches Ranked by Efficiency

### For Pitch Detection (Fast Path):

| Method | Latency | Complexity | Accuracy | Best For |
|--------|---------|------------|----------|----------|
| Bitstream autocorrelation | ~5ms | O(N) bit-parallel | Good | Fastest possible pitch |
| YIN | ~2 periods (~24ms for E2) | O(N*tau_max) | Very good | Standard monophonic |
| Autocorrelation (AMDF) | ~2 periods | O(N*tau_max) | Good | Simple implementation |
| CREPE Tiny | ~10ms | CNN inference | Best | When GPU/NPU available |
| Goertzel bank | ~10ms | O(N*K) K=num freqs | Good for known set | When target freqs known |
| HPS | ~20ms | O(N log N) + O(K) | Fair for polyphonic | Multi-note detection |
| Cepstral | ~20ms | O(N log N) | Fair | Robust to harmonics |

### For String Identification (Slow Path):

| Method | Latency | Training Data | Accuracy | Best For |
|--------|---------|---------------|----------|----------|
| Inharmonicity B parameter | ~40ms | 1 sample/string | 98.5% | Physical model, minimal data |
| Harmonic ratios (Goertzel) | ~50ms | 10 samples/class | ~96% | Our current approach |
| 541-feature SVM | ~50ms | Many samples | 90% F1 | Feature-rich ML |
| CNN on mel-spectrogram | ~50ms | 10 samples/class | 97.3% | Our current best |
| String-Inverse Frequencies | ~40ms | Few samples | Good | Novel approach |

## Recommended Implementation Order

1. **Implement YIN pitch detection in Rust** — fast path for instant note output
2. **Add inharmonicity B estimation** — physical model for string ID (paper: Hjerrild 2019)
3. **Combine with existing CNN** — ML as backup/refinement for string ID
4. **Add continuous pitch tracking** — for bend detection (pitch between frets)
5. **Add legato detection** — pitch change without onset
6. **Add note-off gate** — prevent ghost notes from resonance
7. **Per-guitar calibration** — capture B coefficients for the specific guitar

## Additional Key Papers

### Sub-Period Pitch Estimation

**A Very Low Latency Pitch Tracker for Audio to MIDI Conversion (2014)**
- Author: Olivier Derrien
- Source: DAFx-14
- URL: https://hal.science/hal-01089401/document
- Method: ESPRIT algorithm with statistical model for partial frequencies
- Complexity: O(KN log N) where K=6 partials, N<300 samples
- Tested on real guitar recordings, compared to YIN
- Key advantage: can estimate pitch from LESS than one full period
- Relevance: **Potentially the lowest-latency DSP approach.** Sub-period estimation means you don't need to wait for a full cycle of E2 (12ms).

### Neural Pitch Detection

**Spotify Basic Pitch (2022)**
- URL: https://github.com/spotify/basic-pitch
- Source: ICASSP 2022 (Bittner et al.)
- Method: Lightweight neural network for polyphonic AMT
- Size: <17K parameters, <20MB memory, runs faster than real-time
- Features: pitch bend detection, monophonic + polyphonic
- Formats: TensorFlow, CoreML, TFLite, **ONNX**
- TypeScript version: https://github.com/spotify/basic-pitch-ts (runs in browser!)
- Relevance: **Could replace our CNN entirely.** 17K params, ONNX format, runs in browser. Handles polyphony AND pitch bends.

### Probabilistic YIN

**pYIN: A Fundamental Frequency Estimator Using Probabilistic Threshold Distributions (2014)**
- Authors: Matthias Mauch, Simon Dixon
- Source: ICASSP 2014
- URL: https://www.eecs.qmul.ac.uk/~simond/pub/2014/MauchDixon-PYIN-ICASSP2014.pdf
- Method: YIN with multiple pitch candidates + HMM Viterbi decoding
- Advantage: Better handling of uncertain frames (transitions, vibrato)
- Rust implementation: https://github.com/Sytronik/pyin-rs
- Relevance: More robust than plain YIN for guitar (handles vibrato, bends).

### Bitstream Autocorrelation

**Fast and Efficient Pitch Detection (Cycfi Research, 2018)**
- URL: https://www.cycfi.com/2018/03/fast-and-efficient-pitch-detection-bitstream-autocorrelation/
- Method: Convert signal to 1-bit, then autocorrelate with bit-parallel operations
- Complexity: O(N) with POPCNT instructions
- Advantage: Potentially 10-100x faster than standard autocorrelation
- Relevance: When every microsecond of latency matters.

## Existing Rust Crates for Pitch Detection

| Crate | Methods | WASM | Notes |
|-------|---------|------|-------|
| `pitch-detection` | YIN, McLeod MPM | Yes | https://docs.rs/pitch-detection |
| `pyin-rs` | pYIN (probabilistic) | Unclear | https://github.com/Sytronik/pyin-rs |
| `aubio-rs` | YIN, McLeod, FCOMB, Schmitt | No (C bindings) | Wrapper around aubio |

The `pitch-detection` crate is the best option — pure Rust, WASM-compatible, implements both YIN and McLeod MPM.

## The Pure DSP Approach (No ML)

Based on the research, a pure DSP approach can achieve:

**Step 1: Pitch detection (< 10ms)**
- ESPRIT or YIN on short window
- Output: fundamental frequency in Hz

**Step 2: Frequency → MIDI note**
- `midi = 69 + 12 * log2(freq / 440)`
- Quantize to nearest semitone (or output continuous for bends)

**Step 3: String identification via inharmonicity (40ms)**
- Measure B coefficient from first 6 harmonics
- Compare to calibrated B values per string
- Each string has unique B regardless of fret
- Output: string number (0-5)

**Step 4: Fret from pitch + string**
- `fret = midi_note - open_string_midi[string]`
- Trivial once you know the pitch and the string

**Total latency: ~40ms** for full string+fret+MIDI with pure DSP. No ML needed.

**Accuracy:** Hjerrild 2019 reports 98.5% on string+fret with this approach. That's HIGHER than our CNN (97.3%) and needs almost no training data.

## Should We Use DSP Instead of ML?

**For pitch detection: YES, DSP is better.**
- Our CNN classifies the note from the spectrogram (indirect)
- YIN detects the fundamental directly from the waveform (direct)
- DSP is faster, more accurate, and needs no training

**For string identification: MAYBE DSP is better.**
- Inharmonicity B coefficient: 98.5% accuracy, 1 sample per string, 40ms
- Our CNN: 97.3% accuracy, 10 samples per class, 50ms
- DSP approach is more efficient and potentially more accurate

**For playing technique (bends, legato, muting): DSP is necessary.**
- ML can't track continuous pitch changes (it classifies discrete positions)
- DSP tracks frequency frame-by-frame → bends, vibrato, slides

**Recommendation: Use DSP as the primary pipeline, ML as a refinement layer.**
- DSP handles: onset, pitch, string ID, bends, legato, velocity
- ML handles: ambiguous cases where DSP is uncertain, and validates DSP results

## Key Insight

The inharmonicity parameter B is the most efficient single feature for string+fret identification. It requires only ~40ms of audio and 1 calibration sample per string. Combined with fast pitch detection (YIN at ~8ms), this gives us MiGiC-level latency with string identification that MiGiC doesn't have.

Spotify's Basic Pitch (17K params, ONNX, runs in browser) could be a direct replacement for our custom CNN — it's lighter, handles polyphony, detects pitch bends, and has a TypeScript version for the WASM playground.
