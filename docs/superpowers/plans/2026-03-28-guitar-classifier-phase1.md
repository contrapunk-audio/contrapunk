# Guitar Classifier Phase 1: Data Capture + Initial Exploration

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Capture a labeled dataset of 13,800+ pluck samples + 300+ noise samples from the Ibanez Artcore AG85, then build initial exploration tooling (Python + SvelteKit) to look at the raw data before deciding on processing steps.

**Architecture:** Rust capture tool (already built) → MessagePack dataset → Python loader + analysis scripts → SvelteKit visualization app. Data-first approach: capture, look, then decide.

**Tech Stack:** Rust (capture), Python (analysis: librosa, numpy, matplotlib, msgpack), SvelteKit + Tailwind (visualization app), MessagePack (storage)

---

### Task 1: Add Dataset to .gitignore

**Files:**
- Modify: `/.gitignore`

- [ ] **Step 1: Add ML dataset and large files to .gitignore**

Add these lines to the end of `.gitignore`:

```
# ML training data (too large for git)
guitar_training_data.msgpack
ml/capture/*.msgpack
ml/processing/**/*.npy
ml/training/checkpoints/
ml/app/node_modules/
ml/app/.svelte-kit/
```

- [ ] **Step 2: Commit**

```bash
git add .gitignore
git commit -m "chore: add ML dataset files to gitignore"
```

---

### Task 2: Add Session Resume to Capture Tool

The current capture tool overwrites the dataset file. For 11.5 hours of capture across multiple sessions, it must **load existing data and continue from where we left off**.

**Files:**
- Modify: `examples/guitar_capture.rs`

- [ ] **Step 1: Add session resume logic after dataset initialization**

In `guitar_capture.rs`, after the `dataset` variable is created (~line 175), add:

```rust
    // ── Resume from existing dataset if present ─────────
    let mut start_string = 0usize;
    let mut start_fret = 0u8;

    if std::path::Path::new(DATASET_PATH).exists() {
        print!("  Existing dataset found. Resume? [Y/n]: ");
        std::io::stdout().flush().unwrap();
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();

        if input.trim().to_lowercase() != "n" {
            let data = std::fs::read(DATASET_PATH).expect("Failed to read existing dataset");
            let existing: TrainingDataset = rmp_serde::from_slice(&data)
                .expect("Failed to deserialize existing dataset");

            println!("  Loaded {} existing samples from '{}'",
                existing.samples.len(), existing.metadata.guitar_name);

            // Find last captured position to resume from
            let mut max_string = 0u8;
            let mut max_fret = 0u8;
            for s in &existing.samples {
                if s.string_idx < 6 {
                    if s.string_idx > max_string
                        || (s.string_idx == max_string && s.fret > max_fret)
                    {
                        max_string = s.string_idx;
                        max_fret = s.fret;
                    }
                }
            }

            // Resume from next position
            if max_fret < FRETS as u8 {
                start_string = max_string as usize;
                start_fret = max_fret + 1;
            } else if (max_string as usize) < 5 {
                start_string = max_string as usize + 1;
                start_fret = 0;
            }

            dataset.samples = existing.samples;
            total_captured = dataset.samples.len();
            println!("  Resuming from string {} fret {}\n", start_string, start_fret);
        }
    }
```

- [ ] **Step 2: Update the fretboard walk loop to use start_string/start_fret**

Change the outer loop from `for string_idx in 0..6` to:

```rust
    for string_idx in start_string..6 {
        if quit { break; }
        let string_name = STRING_NAMES[string_idx];
        let base_midi = STRING_BASE_PITCH[string_idx];

        // ... existing header code ...

        let fret_start = if string_idx == start_string { start_fret } else { 0 };
        for fret in fret_start..=(FRETS as u8) {
```

- [ ] **Step 3: Build and verify**

```bash
cargo build --release --example guitar_capture
```

Expected: builds clean.

- [ ] **Step 4: Commit**

```bash
git add examples/guitar_capture.rs
git commit -m "feat(capture): add session resume - load existing dataset and continue"
```

---

### Task 3: Add Goertzel Feature Extraction to Capture Tool

The spec requires Goertzel features computed in Rust during capture (for feature extraction parity). Store them alongside raw audio so Python never recomputes them.

**Files:**
- Modify: `examples/guitar_capture.rs`

- [ ] **Step 1: Add Goertzel features to TrainingSample struct**

Add after the `pitch_validated` field:

```rust
    /// Pre-computed Goertzel harmonic features (computed in Rust for parity)
    goertzel_harmonics: Vec<f32>,
    /// Harmonic ratios (h2/h1, h3/h1, ..., h10/h1)
    harmonic_ratios: Vec<f32>,
    /// Spectral centroid at onset
    spectral_centroid: f32,
```

- [ ] **Step 2: Add Goertzel computation function**

Add before the `audio_process` function:

```rust
use contrapunk::audio::detectors::GoertzelBank;

/// Compute Goertzel harmonic features for a pluck.
/// Given audio and detected fundamental, extract first 10 harmonic amplitudes.
fn compute_goertzel_features(
    audio: &[f32],
    fundamental_freq: f32,
    sample_rate: usize,
) -> (Vec<f32>, Vec<f32>, f32) {
    if fundamental_freq <= 0.0 || audio.len() < 512 {
        return (vec![0.0; 10], vec![0.0; 9], 0.0);
    }

    // Compute magnitude at first 10 harmonics using Goertzel
    let mut harmonics = Vec::with_capacity(10);
    for h in 1..=10 {
        let target_freq = fundamental_freq * h as f32;
        if target_freq > sample_rate as f32 / 2.0 {
            harmonics.push(0.0);
            continue;
        }
        let k = (target_freq * audio.len() as f32 / sample_rate as f32).round() as usize;
        let coeff = 2.0 * (2.0 * std::f32::consts::PI * k as f32 / audio.len() as f32).cos();

        let mut s0 = 0.0f32;
        let mut s1 = 0.0f32;
        let mut s2;
        for &sample in audio {
            s2 = s1;
            s1 = s0;
            s0 = sample + coeff * s1 - s2;
        }
        let real = s0 - s1 * (2.0 * std::f32::consts::PI * k as f32 / audio.len() as f32).cos();
        let imag = s1 * (2.0 * std::f32::consts::PI * k as f32 / audio.len() as f32).sin();
        harmonics.push((real * real + imag * imag).sqrt());
    }

    // Compute ratios (h2/h1, h3/h1, ..., h10/h1)
    let h1 = harmonics[0].max(1e-10);
    let ratios: Vec<f32> = harmonics[1..].iter().map(|&h| h / h1).collect();

    // Spectral centroid
    let total_mag: f32 = harmonics.iter().sum();
    let centroid = if total_mag > 1e-10 {
        harmonics.iter().enumerate()
            .map(|(i, &m)| (i + 1) as f32 * fundamental_freq * m)
            .sum::<f32>() / total_mag
    } else {
        0.0
    };

    (harmonics, ratios, centroid)
}
```

- [ ] **Step 3: Call Goertzel computation when capturing plucks**

In `capture_position_validated`, after the pitch validation check, compute and store features:

```rust
            let (goertzel_harmonics, harmonic_ratios, spectral_centroid) =
                compute_goertzel_features(&audio, s.frequency, sample_rate);

            plucks.push((audio, detected_midi, conf, rms, peak, validated,
                         goertzel_harmonics, harmonic_ratios, spectral_centroid));
```

Update the `ValidatedPluck` type and all places where plucks are pushed to `dataset.samples` to include the new fields.

- [ ] **Step 4: Build and verify**

```bash
cargo build --release --example guitar_capture
```

- [ ] **Step 5: Commit**

```bash
git add examples/guitar_capture.rs
git commit -m "feat(capture): compute Goertzel harmonic features during capture"
```

---

### Task 4: Create Python Dataset Loader

**Files:**
- Create: `ml/loader.py`
- Create: `ml/requirements.txt`

- [ ] **Step 1: Create requirements.txt**

```
msgpack>=1.0
numpy>=1.24
librosa>=0.10
soundfile>=0.12
matplotlib>=3.7
seaborn>=0.12
scikit-learn>=1.3
```

- [ ] **Step 2: Create loader.py**

```python
"""
Guitar training dataset loader.

Loads MessagePack datasets captured by guitar_capture.rs.
Provides access to raw audio, labels, metadata, and pre-computed features.

Usage:
    from loader import GuitarDataset
    ds = GuitarDataset("guitar_training_data.msgpack")
    print(ds.summary())
    sample = ds[0]
    ds.play(0)
"""

import msgpack
import numpy as np
import struct
from pathlib import Path
from typing import Optional
import subprocess
import tempfile
import wave


class GuitarSample:
    """A single training sample."""
    def __init__(self, data: dict):
        self.audio = np.array(data["audio"], dtype=np.float32)
        self.label = data["label"]
        self.class_id = data["class_id"]
        self.string_idx = data["string_idx"]
        self.fret = data["fret"]
        self.expected_midi = data["expected_midi"]
        self.detected_midi = data["detected_midi"]
        self.confidence = data["confidence"]
        self.rms = data["rms"]
        self.peak = data["peak"]
        self.sample_rate = data["sample_rate"]
        self.pitch_validated = data["pitch_validated"]
        self.goertzel_harmonics = np.array(
            data.get("goertzel_harmonics", []), dtype=np.float32
        )
        self.harmonic_ratios = np.array(
            data.get("harmonic_ratios", []), dtype=np.float32
        )
        self.spectral_centroid = data.get("spectral_centroid", 0.0)

    @property
    def duration_secs(self) -> float:
        return len(self.audio) / self.sample_rate

    @property
    def is_noise(self) -> bool:
        return self.string_idx == 255

    @property
    def note_name(self) -> str:
        if self.is_noise:
            return "noise"
        names = ["C","C#","D","D#","E","F","F#","G","G#","A","A#","B"]
        return f"{names[self.expected_midi % 12]}{self.expected_midi // 12 - 1}"


class GuitarDataset:
    """Full training dataset."""
    def __init__(self, path: str):
        self.path = Path(path)
        with open(path, "rb") as f:
            raw = msgpack.unpackb(f.read(), raw=False)

        self.version = raw["version"]
        self.metadata = raw["metadata"]
        self.samples = [GuitarSample(s) for s in raw["samples"]]

    def __len__(self) -> int:
        return len(self.samples)

    def __getitem__(self, idx) -> GuitarSample:
        return self.samples[idx]

    @property
    def guitar_name(self) -> str:
        return self.metadata["guitar_name"]

    @property
    def sample_rate(self) -> int:
        return self.metadata["sample_rate"]

    def summary(self) -> str:
        """Print a summary of the dataset."""
        lines = [
            f"Dataset: {self.path.name}",
            f"Guitar: {self.guitar_name}",
            f"Samples: {len(self.samples)}",
            f"Sample rate: {self.sample_rate} Hz",
            f"Version: {self.version}",
            "",
        ]

        # Per-string counts
        from collections import Counter
        string_counts = Counter()
        noise_count = 0
        validated_count = 0
        for s in self.samples:
            if s.is_noise:
                noise_count += 1
            else:
                string_counts[s.string_idx] += 1
                if s.pitch_validated:
                    validated_count += 1

        string_names = ["Low E", "A", "D", "G", "B", "High E"]
        for i in range(6):
            lines.append(f"  String {string_names[i]}: {string_counts.get(i, 0)} samples")
        lines.append(f"  Noise: {noise_count} samples")
        lines.append(f"  Pitch validated: {validated_count}/{len(self.samples) - noise_count}")

        return "\n".join(lines)

    def get_by_position(self, string_idx: int, fret: int) -> list:
        """Get all samples for a specific string+fret position."""
        return [s for s in self.samples
                if s.string_idx == string_idx and s.fret == fret]

    def get_noise(self, category: Optional[str] = None) -> list:
        """Get noise samples, optionally filtered by category."""
        if category:
            return [s for s in self.samples if s.label == f"noise_{category}"]
        return [s for s in self.samples if s.is_noise]

    def get_by_string(self, string_idx: int) -> list:
        """Get all samples for a string (all frets)."""
        return [s for s in self.samples if s.string_idx == string_idx]

    def play(self, idx: int):
        """Play a sample using system audio (macOS afplay)."""
        s = self.samples[idx]
        # Write to temp WAV file
        with tempfile.NamedTemporaryFile(suffix=".wav", delete=False) as f:
            with wave.open(f.name, "w") as w:
                w.setnchannels(1)
                w.setsampwidth(2)  # 16-bit
                w.setframerate(s.sample_rate)
                # Convert float32 to int16
                audio_int16 = (s.audio * 32767).clip(-32768, 32767).astype(np.int16)
                w.writeframes(audio_int16.tobytes())
            subprocess.run(["afplay", f.name], check=True)


if __name__ == "__main__":
    import sys
    if len(sys.argv) < 2:
        print("Usage: python loader.py <dataset.msgpack>")
        sys.exit(1)

    ds = GuitarDataset(sys.argv[1])
    print(ds.summary())
```

- [ ] **Step 3: Test the loader (requires a captured dataset)**

```bash
cd ml && python loader.py ../guitar_training_data.msgpack
```

Expected: prints dataset summary with per-string sample counts.

- [ ] **Step 4: Commit**

```bash
git add ml/loader.py ml/requirements.txt
git commit -m "feat(ml): add Python dataset loader for MessagePack guitar data"
```

---

### Task 5: Create Raw Data Analysis Script

Generates the visualizations for `ml/processing/01_raw_analysis/`.

**Files:**
- Create: `ml/processing/01_raw_analysis/analyze.py`
- Create: `ml/processing/01_raw_analysis/WHAT_IS_HAPPENING.md`

- [ ] **Step 1: Create WHAT_IS_HAPPENING.md**

```markdown
# Step 1: Raw Data Analysis

## What This Step Does

Before processing the data, we look at it raw. This catches problems early:
- Are all samples the right length?
- Are any samples clipped (too loud, hitting the digital ceiling)?
- Are any samples silent (pluck missed, onset detection failed)?
- Is the amplitude distribution consistent across strings?
- Do the pitch detector's results match the expected notes?

## What to Look For

### Waveform Grid
- Each row is a string (Low E through High E)
- Each column is a random sample from that string
- **Good:** Clear attack at the start, visible decay, consistent amplitude
- **Bad:** Attack in the middle (onset alignment issue), flat line (silence), square clipping

### Amplitude Histogram
- Shows the distribution of RMS values per string
- **Good:** Each string has a distinct but overlapping range
- **Bad:** One string is much quieter (gain issue) or all identical (normalization ran too early)

### Pitch Accuracy
- Compares what the pitch detector heard vs what you played
- **Good:** >95% match within 1 semitone
- **Bad:** Systematic errors (always off by octave) indicate detector issues

### Clipping Report
- Lists any samples where |sample| > 0.99 for >1% of frames
- **Good:** Zero clipped samples
- **Bad:** Clipped samples need to be re-recorded (distorted harmonics corrupt features)
```

- [ ] **Step 2: Create analyze.py**

```python
"""
Raw data analysis — Step 1 of the processing pipeline.

Generates:
- waveform_grid.png — random samples per string
- amplitude_histogram.png — RMS distribution per string
- pitch_accuracy.png — detected vs expected MIDI
- duration_histogram.png — sample lengths
- clipping_report.txt — flagged clipped/silent samples
- sample_counts.png — per-class sample count bar chart

Usage:
    python analyze.py ../../guitar_training_data.msgpack
"""

import sys
import os
sys.path.insert(0, os.path.join(os.path.dirname(__file__), "../.."))
from ml.loader import GuitarDataset

import numpy as np
import matplotlib.pyplot as plt
import matplotlib
matplotlib.use("Agg")  # Non-interactive backend

OUTPUT_DIR = os.path.dirname(__file__)
STRING_NAMES = ["Low E", "A", "D", "G", "B", "High E"]


def plot_waveform_grid(ds, n_cols=4):
    """Plot random waveforms per string."""
    fig, axes = plt.subplots(6, n_cols, figsize=(16, 12))
    fig.suptitle(f"Raw Waveforms — {ds.guitar_name}", fontsize=14)

    for si in range(6):
        samples = ds.get_by_string(si)
        if not samples:
            continue
        chosen = np.random.choice(len(samples), min(n_cols, len(samples)), replace=False)
        for col, idx in enumerate(chosen):
            s = samples[idx]
            t = np.arange(len(s.audio)) / s.sample_rate * 1000  # ms
            axes[si][col].plot(t, s.audio, linewidth=0.3, color="steelblue")
            axes[si][col].set_ylim(-1, 1)
            axes[si][col].set_title(f"{STRING_NAMES[si]} f{s.fret}", fontsize=8)
            if col == 0:
                axes[si][col].set_ylabel(STRING_NAMES[si], fontsize=9)
            axes[si][col].tick_params(labelsize=6)

    plt.tight_layout()
    plt.savefig(os.path.join(OUTPUT_DIR, "waveform_grid.png"), dpi=150)
    plt.close()
    print("  Saved waveform_grid.png")


def plot_amplitude_histogram(ds):
    """RMS distribution per string."""
    fig, ax = plt.subplots(figsize=(10, 6))
    for si in range(6):
        samples = ds.get_by_string(si)
        rms_vals = [s.rms for s in samples]
        if rms_vals:
            ax.hist(rms_vals, bins=30, alpha=0.5, label=STRING_NAMES[si])

    noise_rms = [s.rms for s in ds.get_noise()]
    if noise_rms:
        ax.hist(noise_rms, bins=30, alpha=0.5, label="Noise", color="gray")

    ax.set_xlabel("RMS")
    ax.set_ylabel("Count")
    ax.set_title(f"Amplitude Distribution — {ds.guitar_name}")
    ax.legend()
    plt.tight_layout()
    plt.savefig(os.path.join(OUTPUT_DIR, "amplitude_histogram.png"), dpi=150)
    plt.close()
    print("  Saved amplitude_histogram.png")


def plot_pitch_accuracy(ds):
    """Detected vs expected MIDI note."""
    expected = []
    detected = []
    correct = 0
    total = 0

    for s in ds.samples:
        if s.is_noise:
            continue
        total += 1
        expected.append(s.expected_midi)
        detected.append(s.detected_midi)
        if abs(int(s.detected_midi) - int(s.expected_midi)) <= 1:
            correct += 1

    fig, ax = plt.subplots(figsize=(8, 8))
    ax.scatter(expected, detected, alpha=0.1, s=2)
    ax.plot([30, 90], [30, 90], "r--", linewidth=1, label="Perfect")
    ax.set_xlabel("Expected MIDI")
    ax.set_ylabel("Detected MIDI")
    ax.set_title(f"Pitch Accuracy: {correct}/{total} ({100*correct/max(total,1):.1f}%)")
    ax.legend()
    plt.tight_layout()
    plt.savefig(os.path.join(OUTPUT_DIR, "pitch_accuracy.png"), dpi=150)
    plt.close()
    print(f"  Saved pitch_accuracy.png ({correct}/{total} = {100*correct/max(total,1):.1f}%)")


def plot_duration_histogram(ds):
    """Sample duration distribution."""
    durations = [s.duration_secs * 1000 for s in ds.samples]  # ms
    fig, ax = plt.subplots(figsize=(8, 4))
    ax.hist(durations, bins=50, color="steelblue")
    ax.set_xlabel("Duration (ms)")
    ax.set_ylabel("Count")
    ax.set_title("Sample Duration Distribution")
    ax.axvline(500, color="red", linestyle="--", label="Target 500ms")
    ax.legend()
    plt.tight_layout()
    plt.savefig(os.path.join(OUTPUT_DIR, "duration_histogram.png"), dpi=150)
    plt.close()
    print("  Saved duration_histogram.png")


def plot_sample_counts(ds):
    """Bar chart of samples per class (grouped by string)."""
    from collections import Counter
    counts = Counter(s.label for s in ds.samples)

    fig, ax = plt.subplots(figsize=(20, 6))
    labels = sorted(counts.keys())
    values = [counts[l] for l in labels]

    # Color by string
    colors = []
    string_colors = ["#e74c3c", "#e67e22", "#f1c40f", "#2ecc71", "#3498db", "#9b59b6"]
    for l in labels:
        if l.startswith("noise"):
            colors.append("gray")
        else:
            si = int(l.split("_")[1])
            colors.append(string_colors[si])

    ax.bar(range(len(labels)), values, color=colors, width=0.8)
    ax.set_xlabel("Class")
    ax.set_ylabel("Sample Count")
    ax.set_title(f"Samples per Class — {ds.guitar_name}")
    ax.set_xticks(range(0, len(labels), 5))
    ax.set_xticklabels([labels[i] for i in range(0, len(labels), 5)],
                       rotation=90, fontsize=5)
    plt.tight_layout()
    plt.savefig(os.path.join(OUTPUT_DIR, "sample_counts.png"), dpi=150)
    plt.close()
    print("  Saved sample_counts.png")


def clipping_report(ds):
    """Check for clipped or silent samples."""
    clipped = []
    silent = []

    for i, s in enumerate(ds.samples):
        clip_ratio = np.mean(np.abs(s.audio) > 0.99)
        if clip_ratio > 0.01:
            clipped.append((i, s.label, clip_ratio))
        if np.max(np.abs(s.audio)) < 0.005:
            silent.append((i, s.label, np.max(np.abs(s.audio))))

    with open(os.path.join(OUTPUT_DIR, "clipping_report.txt"), "w") as f:
        f.write(f"Clipping Report — {ds.guitar_name}\n")
        f.write(f"Total samples: {len(ds.samples)}\n\n")
        f.write(f"Clipped samples (>1% of frames above 0.99): {len(clipped)}\n")
        for idx, label, ratio in clipped:
            f.write(f"  [{idx}] {label}: {ratio*100:.1f}% clipped\n")
        f.write(f"\nSilent samples (max amplitude < 0.005): {len(silent)}\n")
        for idx, label, maxval in silent:
            f.write(f"  [{idx}] {label}: max={maxval:.4f}\n")

    print(f"  Saved clipping_report.txt ({len(clipped)} clipped, {len(silent)} silent)")


def main():
    if len(sys.argv) < 2:
        print("Usage: python analyze.py <dataset.msgpack>")
        sys.exit(1)

    ds = GuitarDataset(sys.argv[1])
    print(f"\n=== Raw Data Analysis: {ds.guitar_name} ===\n")
    print(ds.summary())
    print()

    np.random.seed(42)
    plot_waveform_grid(ds)
    plot_amplitude_histogram(ds)
    plot_pitch_accuracy(ds)
    plot_duration_histogram(ds)
    plot_sample_counts(ds)
    clipping_report(ds)

    print(f"\nAll outputs in: {OUTPUT_DIR}/")


if __name__ == "__main__":
    main()
```

- [ ] **Step 3: Run analysis (requires captured data)**

```bash
cd ml/processing/01_raw_analysis
python analyze.py ../../../guitar_training_data.msgpack
```

Expected: generates 5 PNGs + 1 TXT in `ml/processing/01_raw_analysis/`.

- [ ] **Step 4: Commit**

```bash
git add ml/processing/01_raw_analysis/
git commit -m "feat(ml): add raw data analysis script with visualizations"
```

---

### Task 6: Scaffold SvelteKit Visual Learning App

**Files:**
- Create: `ml/app/` — SvelteKit project
- Create: `ml/app/src/routes/+page.svelte` — landing page
- Create: `ml/app/src/routes/raw-data/+page.svelte` — raw data explorer

- [ ] **Step 1: Initialize SvelteKit project**

```bash
cd ml
npx sv create app --template minimal --types ts
cd app
npm install
npm install -D @tailwindcss/vite tailwindcss
```

- [ ] **Step 2: Create landing page**

Create `ml/app/src/routes/+page.svelte`:

```svelte
<script lang="ts">
    const pages = [
        { href: '/raw-data', title: '1. Raw Data', desc: 'Browse samples, play audio, check quality' },
        { href: '/onset', title: '2. Onset Alignment', desc: 'Before/after onset alignment' },
        { href: '/features', title: '3. Features', desc: 'Mel-spectrograms, Goertzel harmonics' },
        { href: '/normalization', title: '4. Normalization', desc: 'Before/after global normalization' },
        { href: '/augmentation', title: '5. Augmentation', desc: 'Noise injection, gain variation' },
        { href: '/validation', title: '6. Validation', desc: 't-SNE, class balance, outliers' },
        { href: '/training', title: '7. Training', desc: 'Live training curves, confusion matrix' },
        { href: '/comparison', title: '8. Comparison', desc: 'Side-by-side model comparison' },
        { href: '/ensemble', title: '9. Ensemble', desc: 'Combined model behavior' },
        { href: '/live', title: '10. Live Demo', desc: 'Real-time guitar classification' },
    ];
</script>

<div class="min-h-screen bg-gray-950 text-gray-100 p-8">
    <h1 class="text-3xl font-bold mb-2">Contrapunk ML Pipeline</h1>
    <p class="text-gray-400 mb-8">Guitar String+Fret Classifier — Visual Learning Environment</p>

    <div class="grid grid-cols-2 gap-4 max-w-4xl">
        {#each pages as page}
            <a href={page.href}
               class="block p-4 bg-gray-900 border border-gray-800 rounded hover:border-cyan-500 transition-colors">
                <h2 class="text-lg font-semibold text-cyan-400">{page.title}</h2>
                <p class="text-sm text-gray-400 mt-1">{page.desc}</p>
            </a>
        {/each}
    </div>
</div>
```

- [ ] **Step 3: Create API endpoint to serve dataset summary**

Create `ml/app/src/routes/api/dataset/+server.ts`:

```typescript
import { json } from '@sveltejs/kit';
import { readFileSync, existsSync } from 'fs';
import { join } from 'path';

export async function GET() {
    // Check for analysis outputs
    const analysisDir = join(process.cwd(), '..', 'processing', '01_raw_analysis');
    const images = ['waveform_grid.png', 'amplitude_histogram.png',
                    'pitch_accuracy.png', 'sample_counts.png', 'duration_histogram.png'];

    const available = images.filter(img => existsSync(join(analysisDir, img)));

    return json({
        analysisAvailable: available.length > 0,
        images: available,
        analysisDir,
    });
}
```

- [ ] **Step 4: Create raw data page placeholder**

Create `ml/app/src/routes/raw-data/+page.svelte`:

```svelte
<script lang="ts">
    import { onMount } from 'svelte';

    let images: string[] = [];

    onMount(async () => {
        const res = await fetch('/api/dataset');
        const data = await res.json();
        images = data.images;
    });
</script>

<div class="min-h-screen bg-gray-950 text-gray-100 p-8">
    <a href="/" class="text-cyan-400 text-sm hover:underline">← Back</a>
    <h1 class="text-2xl font-bold mt-4 mb-6">1. Raw Data Explorer</h1>

    {#if images.length === 0}
        <div class="bg-gray-900 border border-gray-800 rounded p-6">
            <p class="text-gray-400">No analysis data yet. Run:</p>
            <code class="block mt-2 text-cyan-400 bg-gray-800 p-3 rounded text-sm">
                cd ml/processing/01_raw_analysis && python analyze.py ../../../guitar_training_data.msgpack
            </code>
        </div>
    {:else}
        <div class="space-y-8">
            {#each images as img}
                <div class="bg-gray-900 border border-gray-800 rounded p-4">
                    <h2 class="text-lg font-semibold mb-3 text-gray-300">
                        {img.replace('.png', '').replace(/_/g, ' ')}
                    </h2>
                    <img src="/analysis/{img}" alt={img} class="w-full rounded" />
                </div>
            {/each}
        </div>
    {/if}
</div>
```

- [ ] **Step 5: Verify dev server starts**

```bash
cd ml/app && npm run dev
```

Expected: SvelteKit dev server starts, landing page shows 10 pipeline steps.

- [ ] **Step 6: Commit**

```bash
git add ml/app/
git commit -m "feat(ml): scaffold SvelteKit visual learning app with landing page"
```

---

### Task 7: First Capture Session

This is the human task — actually recording guitar data. Not code, but documented as a task for tracking.

- [ ] **Step 1: Set up the capture environment**

```bash
# Build the capture tool
cargo run --release --example guitar_capture
```

- Select Audient iD14
- Select the correct channel (your guitar input)
- Enter guitar name: `Ibanez Artcore AG85`
- Verify noise floor measurement looks reasonable (<0.01)

- [ ] **Step 2: Capture first string (Low E — all 23 positions x 100 plucks)**

This takes ~40 minutes. Take breaks between fret positions if needed.

After Low E is complete, quit with `Q` to save progress.

- [ ] **Step 3: Verify the saved data**

```bash
cd ml && python loader.py ../guitar_training_data.msgpack
```

Expected: shows ~2,300 samples for Low E (23 positions x 100).

- [ ] **Step 4: Run raw analysis on the first session**

```bash
cd ml/processing/01_raw_analysis
python analyze.py ../../../guitar_training_data.msgpack
```

Look at the outputs:
- `waveform_grid.png` — do the waveforms look like guitar plucks?
- `amplitude_histogram.png` — is the amplitude range reasonable?
- `pitch_accuracy.png` — are detected pitches matching expected?
- `clipping_report.txt` — any clipped or silent samples?

**CHECKPOINT: Look at the data before continuing capture.** If something looks wrong (bad onset alignment, clipping, pitch detection failures), we fix it before spending 10 more hours capturing.

- [ ] **Step 5: Resume and capture remaining strings**

```bash
cargo run --release --example guitar_capture
# Select "Resume" when prompted
```

Continue across multiple sessions until all 6 strings + noise categories are complete.

---

### Task 8: Post-Capture Data Review

After all data is captured, do a comprehensive review before processing.

- [ ] **Step 1: Run full raw analysis**

```bash
cd ml/processing/01_raw_analysis
python analyze.py ../../../guitar_training_data.msgpack
```

- [ ] **Step 2: Review all outputs and document findings**

Create `ml/processing/01_raw_analysis/FINDINGS.md`:

```markdown
# Raw Data Findings

**Date:** YYYY-MM-DD
**Guitar:** Ibanez Artcore AG85
**Total Samples:** [number]

## Observations
- [What does the waveform grid show?]
- [Is amplitude consistent across strings?]
- [What's the pitch accuracy?]
- [Any clipped or silent samples?]
- [Any surprising patterns?]

## Issues Found
- [List any problems]

## Decisions for Next Steps
- [Based on what we see, what processing steps do we need?]
- [Any parameters to adjust?]
```

- [ ] **Step 3: Open the SvelteKit app and view the data there**

```bash
cd ml/app && npm run dev
```

Navigate to the Raw Data page, verify the visualizations display correctly.

- [ ] **Step 4: Commit findings**

```bash
git add ml/processing/01_raw_analysis/FINDINGS.md
git commit -m "docs(ml): document raw data findings from capture session"
```

---

**STOP HERE.** After Task 8, review the data and decide on the next phase. The processing pipeline (onset alignment, feature extraction, normalization, augmentation) should be designed based on what the raw data actually looks like — not planned in advance.

The next plan (Phase 2: Data Processing) will be created after reviewing the raw data findings.

---

## What Comes After Phase 1

These are NOT part of this plan — they are separate plans created after data review:

- **Phase 2:** Data Processing (onset alignment, feature extraction, normalization, augmentation)
- **Phase 3:** Training (3 models, comparison, ensemble)
- **Phase 4:** Deployment (pure Rust inference, Contrapunk integration, mode selection)
- **Phase 5:** Visual Learning App (remaining 9 pages)
