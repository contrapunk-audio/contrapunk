# ML & Audio Concepts for Contrapunk Guitar Classifier

This document explains every concept used in the guitar classifier pipeline. It's a living reference — updated as we build.

---

## Audio Fundamentals

### What is a DI (Direct Input) Signal?
A DI signal is captured directly from the guitar's pickup into the audio interface, bypassing any microphone. The Audient iD14's instrument input converts the guitar's electrical signal to digital. This means:
- **No room acoustics** — no reverb, no reflections
- **No microphone coloring** — the signal IS the string vibration
- **Very clean** — noise floor is just the preamp's thermal noise (~-127 dBu for the iD14)
- **Consistent** — same guitar + same interface = same signal characteristics every time

This is why many standard audio ML techniques (designed for noisy microphone recordings) are overkill or even harmful for our use case.

### What is RMS?
RMS = Root Mean Square. It's a way to measure the "average loudness" of an audio signal:
```
RMS = sqrt( (1/N) * sum(sample[i]^2) )
```
A silent signal has RMS ≈ 0. A loud pluck might have RMS ≈ 0.1-0.3. We use RMS for:
- **Noise gating** — if RMS < threshold, it's silence/noise
- **Onset detection** — sudden RMS spike = a pluck just happened
- **Dynamic classification** — soft vs hard pluck

### What is a Harmonic Series?
When you pluck a guitar string, it vibrates at multiple frequencies simultaneously:
- **Fundamental (h1)** — the note you hear (e.g., 110 Hz for A2)
- **2nd harmonic (h2)** — 2x the fundamental (220 Hz)
- **3rd harmonic (h3)** — 3x (330 Hz)
- **4th harmonic (h4)** — 4x (440 Hz)
- ...and so on

The **relative amplitudes** of these harmonics (h2/h1, h3/h1, etc.) are what make the same note sound different on different strings. A thick Low E string has stronger higher harmonics than a thin High E string. This is the primary signal we use for string identification.

### What is Inharmonicity?
Real guitar strings aren't perfectly flexible — they have stiffness. This causes the harmonics to be slightly **sharper** than perfect integer multiples. The 2nd harmonic isn't exactly 2x the fundamental — it's slightly higher. This inharmonicity:
- Is greater for thicker, shorter strings
- Differs between wound and plain strings
- Is a reliable fingerprint for identifying which string was plucked
- Can be measured as the deviation of each harmonic from its expected frequency

### What is Spectral Centroid?
The spectral centroid is the "center of mass" of the frequency spectrum:
```
centroid = sum(frequency[i] * magnitude[i]) / sum(magnitude[i])
```
Think of it as the "brightness" of a sound. A bright sound (lots of high frequencies) has a high centroid. A dull sound (mostly low frequencies) has a low centroid.

Guitar plucks start bright (high centroid at the attack) and get duller as harmonics decay (centroid drops over time). This trajectory is different for each string.

---

## Feature Extraction

### What is an FFT?
FFT = Fast Fourier Transform. It takes a chunk of audio samples (time domain) and converts it to a frequency spectrum (frequency domain). You go from "here are 2048 amplitude values over time" to "here is how much energy exists at each frequency."

**Parameters that matter:**
- **FFT size (n_fft)** — how many samples per analysis window. Larger = better frequency resolution but worse time resolution. We use 2048.
- **Frequency resolution** = sample_rate / n_fft. At 44,100 Hz / 2048 = **21.5 Hz per bin**
- **Hop length** — how many samples between consecutive FFTs. Smaller hop = more overlap = more time frames.

### What is a Mel-Spectrogram?
A spectrogram is a 2D image of FFT results over time: x-axis = time, y-axis = frequency, color = magnitude.

A **mel-spectrogram** applies the **mel scale** — a frequency mapping that matches human hearing perception. Humans hear logarithmically: the difference between 100 Hz and 200 Hz sounds the same as 1000 Hz and 2000 Hz. The mel scale compresses high frequencies and expands low frequencies to match this.

**Our parameters:**
- 64 mel bins from 60 Hz to 8000 Hz
- FFT size: 2048
- Hop length: 512
- This produces a (64, ~22) "image" per 500ms sample

### What is a Goertzel Filter?
The Goertzel algorithm computes the FFT magnitude at a **single specific frequency** — much faster than computing the entire FFT when you only care about certain frequencies.

We use a **Goertzel bank** — 49 Goertzel filters tuned to every guitar note from E2 to E6. For each audio frame, we get the energy at each guitar note frequency. This tells us exactly which notes are present and how strong they are.

**Why Goertzel > FFT for our case:**
- We only care about 49 specific frequencies (guitar notes), not all 1024 FFT bins
- Each Goertzel filter is O(N) with just one multiply-add per sample
- The result directly tells us "how much A2 energy is present" — no bin interpolation needed

### What are Harmonic Ratio Features?
Given a detected fundamental frequency (e.g., 110 Hz for A2), we run Goertzel filters at its first 10 harmonics (110, 220, 330, 440, 550, ...) and compute:
```
h2_ratio = magnitude_at_220Hz / magnitude_at_110Hz
h3_ratio = magnitude_at_330Hz / magnitude_at_110Hz
...
```
These ratios are the "harmonic fingerprint" of the string. Different strings produce the same fundamental but with different harmonic ratios because of their physical properties (gauge, tension, material, length).

---

## Data Pipeline Concepts

### What is Onset Detection?
Onset detection identifies the moment a new note begins — the "pluck" instant. Methods we use:
- **RMS spike** — sudden jump in loudness (simplest, fastest)
- **HFC (High-Frequency Content)** — weighted sum of spectral bins emphasizing high frequencies. Pluck attacks are broadband, so HFC spikes sharply.
- **Spectral flux** — frame-to-frame change in the spectrum. A new note creates a big spectral change.

We combine all three: HFC spike AND (flux spike OR RMS slope) = onset detected.

### What is Onset-Forward Capture?
**Wrong way:** Keep a ring buffer of the last 500ms. When onset happens, grab the buffer. Problem: the pluck is at the END — 90% of the capture is irrelevant pre-pluck audio.

**Right way:** When onset is detected, START recording from that moment forward for 500ms. The pluck attack is at the BEGINNING — all 500ms is useful signal.

### Why O(1) Ring Buffers Matter on Audio Threads
Audio callbacks run on a real-time thread with strict timing requirements. If processing takes too long, audio samples get dropped.

`Vec::remove(0)` is O(n) — it shifts ALL elements left by one. At 44,100 Hz with a 22,050-element buffer, that's ~88KB of memory moved per sample = **3.9 GB/s of memory copies.**

`VecDeque::pop_front()` is O(1) — it just moves a pointer. No data is copied. This is why we use VecDeque for audio buffers.

### What is Pitch Validation?
The capture tool tells you "play A string, fret 5." You play it. The pitch detector says it heard E3 (MIDI 52, which IS A fret 5). If the detected pitch matches the expected pitch within ±1 semitone, the pluck is **validated**.

If they don't match (you hit the wrong string, missed the fret, got a dead note), the pluck is **rejected** and you try again. This ensures every training sample has the correct label.

### What is Per-Sample vs Global Normalization?
- **Per-sample normalization:** Each sample is independently scaled to zero mean and unit variance. Problem: a quiet noise sample gets scaled UP to look as "big" as a loud pluck. Destroys loudness information.
- **Global normalization:** Compute the mean and standard deviation across the ENTIRE training set, then apply those same numbers to every sample. A quiet sample stays quiet, a loud sample stays loud. Relative amplitudes are preserved.

We use **global normalization** because the loudness difference between strings and noise is a real, useful signal.

---

## Machine Learning Concepts

### What is a CNN (Convolutional Neural Network)?
A CNN processes 2D data (like images or spectrograms) by sliding small filters across the input. Each filter detects a specific pattern (e.g., "energy peak at this frequency," "this harmonic ratio pattern"). Multiple layers of filters build up from simple patterns to complex ones.

For our spectrogram input (64x22), the CNN learns to recognize frequency patterns that distinguish different strings and frets.

### What is a Random Forest?
A random forest is a collection of decision trees. Each tree asks a series of yes/no questions about the input features:
```
Is h2_ratio > 0.5?
  Yes → Is spectral_centroid > 2000?
    Yes → Probably A string
    No → Probably D string
  No → Is rms > 0.1?
    ...
```
Many trees (500) each see a random subset of features, then they "vote" on the answer. Random forests:
- Don't need much data (work well at 30+ samples per class)
- Are interpretable (you can see which features matter)
- Are fast (just nested if/else)
- Work in pure Rust with zero dependencies

### What is an Ensemble?
An ensemble combines multiple different models to make better predictions than any single model:
```
Audio → Goertzel features → Random Forest → prediction A
Audio → Attack spectrogram → small CNN → prediction B

If A and B agree → high confidence, use that answer
If they disagree → use the one with higher confidence
If both low confidence → reject as noise
```
Different models see different aspects of the data. The RF sees harmonic ratios (physics), the CNN sees attack shape (learned). Together they're more accurate than either alone.

### What is Overfitting?
When a model memorizes the training data instead of learning general patterns. Signs:
- Training accuracy: 99%. Test accuracy: 60%.
- Model works perfectly on data it's seen, fails on new data.

For our case, overfitting to ONE guitar is actually fine — this is a personal model. But we still need a test set to verify the model works on plucks it hasn't seen from that same guitar.

**Mitigations we use:**
- Dropout (randomly disable neurons during training)
- BatchNorm (normalize activations)
- Early stopping (stop training when validation loss starts rising)
- Data augmentation (create varied copies of training data)

### What is Data Augmentation?
Creating modified copies of training data to increase dataset size and diversity. Safe augmentations for guitar:
- **Gain variation** — make plucks louder/quieter (simulates different pick attack strengths)
- **Noise injection** — add real captured noise at different levels (simulates noisy environments)
- **Time jitter** — shift the onset alignment by a few milliseconds (simulates onset detection timing variation)

**Dangerous augmentations for guitar:**
- **Pitch shifting** — shifting even 0.5 semitones can make one fret sound like another, corrupting labels
- **SpecAugment** — masking frequency bands can remove exactly the harmonics that distinguish strings

### What is a Confusion Matrix?
A grid showing which classes get confused with which:
```
              Predicted
              E2  A2  D3  G3  B3  E4
Actual  E2  [ 95   3   2   0   0   0 ]
        A2  [  4  91   5   0   0   0 ]  ← A2 confused with D3 sometimes
        D3  [  1   4  93   2   0   0 ]
        ...
```
The diagonal shows correct predictions. Off-diagonal shows errors. This tells you which strings the model struggles to distinguish — critical for debugging.

### What is Stratified Splitting?
When splitting data into train/test sets, **stratified** means each split has the same proportion of each class. Without stratification, the random split might put all your A-string-fret-7 samples in the test set and none in training — making it impossible to learn that class.

```python
X_train, X_test, y_train, y_test = train_test_split(
    X, y, test_size=0.20, stratify=y, random_state=42
)
```

---

## Deployment Concepts

### ONNX vs Pure Rust Inference
- **ONNX:** Standard format for ML models. Export from PyTorch, load in any language. But ONNX Runtime adds 30-50MB to binary size and has no WASM support.
- **Pure Rust:** Write the forward pass (conv2d, relu, batchnorm, linear) manually in Rust. Zero dependencies, works in WASM, ~80 lines of code for a 25K parameter model. Weights embedded via `include_bytes!`.

For our tiny model, pure Rust is the clear winner.

### Why ML is a Correction Layer, Not a Trigger
The existing onset detection pipeline fires within ~10ms. ML classification needs 300-500ms of audio. If ML were the trigger, you'd hear nothing for 500ms after plucking — unacceptable for a musician.

Instead: onset fires immediately → heuristic makes a quick guess → note sounds → ML classifies in the background → if ML disagrees, it corrects. The correction must happen within 50ms to be musically acceptable (not 500ms).

---

## Physics of Guitar String Identification

### Why Same Note Sounds Different on Different Strings
A2 (110 Hz) can be played on the Low E string (fret 5) or the A string (open). The fundamental frequency is identical. But they sound different because:

1. **String gauge** — Low E is ~0.046" thick, A is ~0.036". Thicker strings have more mass, producing stronger higher harmonics.
2. **Speaking length** — fretted Low E has a shorter vibrating length (25.5" x 17/22) than open A (25.5"). Shorter length = more inharmonicity.
3. **Winding** — Low E is wound (metal wire wrapped around a core). A string may be wound or plain depending on gauge set. Wound strings have a different spectral decay.
4. **Pluck position relative to string length** — the same absolute pluck position is at a different ratio of the vibrating length for a fretted vs open string, changing which harmonics are excited.

These differences show up in the harmonic ratio features (h2/h1, h3/h1...) and the attack transient shape.

### What Makes a Hollow Body Different?
Your Ibanez Artcore is a hollow body guitar. Compared to a solid body:
- **More acoustic resonance** — the hollow chamber amplifies certain frequencies
- **Feedback susceptibility** — at high gain, the body resonates with the speaker
- **Richer harmonic content** — more complex overtone structure
- **More body noise** — tapping the body produces audible sound through the pickup

This means: more diverse noise samples needed, and the harmonic features will be richer (potentially easier for the model to distinguish strings).
