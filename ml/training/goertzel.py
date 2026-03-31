"""
Contrapunk Guitar Classifier -- Goertzel Harmonic Feature Extraction

Extracts physics-based harmonic features from guitar samples using the
Goertzel algorithm (targeted single-frequency DFT, O(N) per frequency).

Key insight: the same note played on different strings has identical
fundamental frequency but different harmonic ratios. Thick wound strings
produce stronger higher harmonics than thin plain strings. These ratios
are the physics-based fingerprint for string identification.

Usage:
    cd <project_root>
    source ml/venv/bin/activate
    python ml/training/goertzel.py

As a library:
    from ml.training.goertzel import goertzel, extract_harmonic_features
"""

import sys
import os
import json
import math
import time
import warnings
from collections import defaultdict

import numpy as np

import matplotlib
matplotlib.use("Agg")
import matplotlib.pyplot as plt

warnings.filterwarnings("ignore", category=FutureWarning)

PROJECT_ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), "../.."))
DATASET_PATH = os.path.join(PROJECT_ROOT, "guitar_training_data.msgpack")
OUTPUT_JSON  = os.path.join(os.path.dirname(__file__), "goertzel_features.json")
OUTPUT_PNG   = os.path.join(os.path.dirname(__file__), "goertzel_analysis.png")

sys.path.insert(0, PROJECT_ROOT)

STRING_NAMES = ["E2 (low)", "A2", "D3", "G3", "B3", "E4 (high)"]
N_HARMONICS = 10
SEARCH_RANGE = 0.02  # +/-2% around expected harmonic frequency


# == Goertzel Algorithm ====================================================

def goertzel(samples: np.ndarray, sample_rate: int, target_freq: float) -> float:
    """Compute magnitude at a single frequency using Goertzel algorithm.

    The Goertzel algorithm is a specialized DFT that computes the magnitude
    at exactly one frequency bin in O(N) time with just one multiply-add per
    sample. This is far more efficient than a full FFT when you only need a
    handful of frequencies (e.g., a fundamental + its harmonics).

    How it works:
      - Maps the target frequency to the nearest DFT bin index k
      - Computes a second-order IIR filter coefficient from k
      - Runs a 3-tap recurrence through all N samples
      - Extracts magnitude from the final filter state

    Args:
        samples: Audio samples as float32 array
        sample_rate: Sample rate in Hz
        target_freq: The frequency to measure (Hz)

    Returns:
        Magnitude (not power) at target_freq. Zero if inputs are invalid.
    """
    N = len(samples)
    if N == 0 or sample_rate <= 0 or target_freq <= 0:
        return 0.0

    # Map target frequency to nearest DFT bin
    k = int(0.5 + (N * target_freq / sample_rate))

    # Precompute the Goertzel coefficient
    w = 2.0 * math.pi * k / N
    coeff = 2.0 * math.cos(w)

    # Run the Goertzel recurrence: s0 = coeff*s1 - s2 + sample
    s1 = 0.0
    s2 = 0.0
    for sample in samples:
        s0 = coeff * s1 - s2 + sample
        s2 = s1
        s1 = s0

    # Extract power from final state
    power = s1 * s1 + s2 * s2 - coeff * s1 * s2
    # Guard against floating-point noise producing tiny negative values
    if power < 0.0:
        power = 0.0

    return math.sqrt(power)


def goertzel_batch(samples: np.ndarray, sample_rate: int,
                   frequencies: list[float]) -> list[float]:
    """Compute Goertzel magnitude at multiple frequencies.

    More efficient than calling goertzel() in a loop because it pre-converts
    the sample array once.

    Args:
        samples: Audio samples as float32 array
        sample_rate: Sample rate in Hz
        frequencies: List of target frequencies (Hz)

    Returns:
        List of magnitudes, one per frequency.
    """
    return [goertzel(samples, sample_rate, f) for f in frequencies]


# == MIDI / Frequency Helpers ==============================================

def midi_to_freq(midi_note: int) -> float:
    """Convert MIDI note number to frequency in Hz.

    Uses the standard A440 tuning: freq = 440 * 2^((midi - 69) / 12)

    Examples:
        midi_to_freq(40)  -> 82.41 Hz  (E2, low E string open)
        midi_to_freq(45)  -> 110.0 Hz  (A2, A string open)
        midi_to_freq(69)  -> 440.0 Hz  (A4, concert A)
    """
    return 440.0 * (2.0 ** ((midi_note - 69) / 12.0))


# == Harmonic Feature Extraction ===========================================

def extract_harmonic_features(audio: np.ndarray, sr: int,
                               fundamental_freq: float,
                               n_harmonics: int = N_HARMONICS) -> dict:
    """Extract harmonic ratio features for a guitar sample.

    For each harmonic (1st = fundamental, 2nd = octave, etc.), we:
      1. Compute the expected frequency: fundamental * harmonic_number
      2. Search +/-2% around it for the actual peak (guitars have
         inharmonicity -- higher partials are slightly sharp)
      3. Use the Goertzel algorithm to measure magnitude at the peak
      4. Normalize all magnitudes to the fundamental (harmonic ratios)

    The resulting ratios encode the timbre: wound strings have stronger
    upper harmonics than plain strings, even when playing the same note.

    Also computes:
      - spectral_centroid: center of mass of the harmonic spectrum
        (higher = brighter / more high-frequency content)
      - inharmonicity: how much the actual harmonic frequencies deviate
        from perfect integer multiples of the fundamental

    Args:
        audio: Audio samples as float32 array
        sr: Sample rate in Hz
        fundamental_freq: Expected fundamental frequency (Hz)
        n_harmonics: Number of harmonics to extract (default 10)

    Returns:
        Dict with keys:
          harmonic_magnitudes: [h1, h2, ..., h_n] raw magnitudes
          harmonic_ratios:     [h2/h1, h3/h1, ..., h_n/h1] normalized
          spectral_centroid:   float (Hz)
          inharmonicity:       float (mean fractional deviation)
    """
    magnitudes = []
    actual_freqs = []

    for h in range(1, n_harmonics + 1):
        expected_freq = fundamental_freq * h

        # Don't search above Nyquist
        if expected_freq >= sr / 2:
            magnitudes.append(0.0)
            actual_freqs.append(expected_freq)
            continue

        # Search +/-2% around the expected harmonic for the actual peak.
        # Guitar strings exhibit inharmonicity: higher partials are slightly
        # sharper than perfect integer multiples due to string stiffness.
        # We sample 5 points in the search window and take the max.
        search_low = expected_freq * (1.0 - SEARCH_RANGE)
        search_high = expected_freq * (1.0 + SEARCH_RANGE)
        search_freqs = np.linspace(search_low, search_high, 5)

        best_mag = 0.0
        best_freq = expected_freq
        for f in search_freqs:
            mag = goertzel(audio, sr, f)
            if mag > best_mag:
                best_mag = mag
                best_freq = f

        magnitudes.append(best_mag)
        actual_freqs.append(best_freq)

    # Normalize to fundamental (h1)
    h1 = magnitudes[0] if magnitudes[0] > 1e-10 else 1e-10
    ratios = [m / h1 for m in magnitudes[1:]]

    # Spectral centroid: weighted average frequency
    total_energy = sum(magnitudes)
    if total_energy > 1e-10:
        centroid = sum(f * m for f, m in zip(actual_freqs, magnitudes)) / total_energy
    else:
        centroid = 0.0

    # Inharmonicity: mean fractional deviation from perfect harmonics
    # For each harmonic h, measure |actual_freq - h*f0| / (h*f0)
    deviations = []
    for h in range(1, n_harmonics + 1):
        expected = fundamental_freq * h
        if expected >= sr / 2:
            break
        actual = actual_freqs[h - 1]
        if expected > 0:
            deviations.append(abs(actual - expected) / expected)

    inharmonicity = float(np.mean(deviations)) if deviations else 0.0

    return {
        "harmonic_magnitudes": magnitudes,
        "harmonic_ratios": ratios,
        "spectral_centroid": centroid,
        "inharmonicity": inharmonicity,
    }


# == Dataset-Level Feature Export ==========================================

def extract_all_features(dataset_path: str, output_path: str) -> dict:
    """Extract Goertzel harmonic features for every sample in the dataset.

    For each sample:
      1. Looks up the expected MIDI note -> fundamental frequency
      2. Runs the Goertzel harmonic extraction
      3. Stores the result with full metadata

    Args:
        dataset_path: Path to the .msgpack dataset
        output_path: Where to save the JSON output

    Returns:
        The full feature dict (also saved to output_path).
    """
    from ml.loader import GuitarDataset

    ds = GuitarDataset.load(dataset_path)

    print("Extracting Goertzel harmonic features for %d samples..." % len(ds.samples))
    t0 = time.time()

    all_features = []
    string_ratio_sums = defaultdict(lambda: defaultdict(lambda: np.zeros(N_HARMONICS - 1)))
    string_ratio_counts = defaultdict(lambda: defaultdict(int))

    for i, sample in enumerate(ds.samples):
        midi = sample.expected_midi
        if midi is None or midi <= 0:
            continue

        fundamental = midi_to_freq(midi)
        audio = sample.audio_np
        sr = sample.sample_rate

        feats = extract_harmonic_features(audio, sr, fundamental, N_HARMONICS)

        entry = {
            "label": sample.label,
            "class_id": sample.class_id,
            "string_idx": sample.string_idx,
            "fret": sample.fret,
            "fundamental_freq": round(fundamental, 2),
            "harmonic_ratios": [round(r, 6) for r in feats["harmonic_ratios"]],
            "harmonic_magnitudes": [round(m, 4) for m in feats["harmonic_magnitudes"]],
            "spectral_centroid": round(feats["spectral_centroid"], 2),
            "inharmonicity": round(feats["inharmonicity"], 6),
        }
        all_features.append(entry)

        # Accumulate per-string per-fret ratios for visualization
        si = sample.string_idx
        fret = sample.fret
        string_ratio_sums[si][fret] += np.array(feats["harmonic_ratios"])
        string_ratio_counts[si][fret] += 1

        if (i + 1) % 200 == 0:
            elapsed = time.time() - t0
            rate = (i + 1) / elapsed
            remaining = (len(ds.samples) - i - 1) / rate
            print("  %d/%d (%.0f samples/s, ~%.0fs remaining)" %
                  (i + 1, len(ds.samples), rate, remaining))

    elapsed = time.time() - t0
    print("  Done: %d features extracted in %.1fs (%.0f samples/s)" %
          (len(all_features), elapsed, len(all_features) / elapsed))

    output = {
        "params": {
            "n_harmonics": N_HARMONICS,
            "search_range": SEARCH_RANGE,
        },
        "features": all_features,
    }

    with open(output_path, "w") as f:
        json.dump(output, f, indent=2)
    file_size = os.path.getsize(output_path)
    print("  Saved: %s (%.1f KB)" % (output_path, file_size / 1024))

    return output, string_ratio_sums, string_ratio_counts


# == Visualization =========================================================

def generate_visualization(string_ratio_sums, string_ratio_counts, output_path):
    """Generate per-string harmonic ratio profiles.

    Creates a 2x3 grid of subplots (one per string). Each subplot shows
    the average harmonic ratio profile across all frets for that string.
    If different strings have different harmonic signatures, the curves
    will have visibly different shapes -- confirming that Goertzel
    features can help distinguish strings even for the same pitch.

    Args:
        string_ratio_sums: {string_idx: {fret: sum_of_ratio_arrays}}
        string_ratio_counts: {string_idx: {fret: count}}
        output_path: Where to save the PNG
    """
    fig, axes = plt.subplots(2, 3, figsize=(16, 10))
    fig.suptitle("Average Harmonic Ratio Profile by String\n"
                 "(h2/h1, h3/h1, ..., h10/h1 -- higher = more energy in that harmonic)",
                 fontsize=13)

    harmonic_labels = ["H%d/H1" % h for h in range(2, N_HARMONICS + 1)]
    colors = plt.cm.viridis(np.linspace(0.1, 0.9, 23))

    # Also compute a single grand-average per string for comparison
    grand_averages = {}

    for si in range(6):
        ax = axes[si // 3][si % 3]

        if si not in string_ratio_sums:
            ax.set_title(STRING_NAMES[si] + " (no data)")
            continue

        # Plot individual fret curves (semi-transparent)
        all_ratios_for_string = []
        frets_sorted = sorted(string_ratio_sums[si].keys())
        for fret in frets_sorted:
            count = string_ratio_counts[si][fret]
            if count > 0:
                avg = string_ratio_sums[si][fret] / count
                ax.plot(harmonic_labels, avg, color=colors[fret], alpha=0.3,
                        linewidth=1, label=("fret %d" % fret) if fret % 5 == 0 else None)
                all_ratios_for_string.append(avg)

        # Grand average for this string (bold line)
        if all_ratios_for_string:
            grand_avg = np.mean(all_ratios_for_string, axis=0)
            grand_averages[si] = grand_avg
            ax.plot(harmonic_labels, grand_avg, color="white", linewidth=2.5,
                    zorder=10, label="avg")

        ax.set_title(STRING_NAMES[si], fontsize=11, fontweight="bold")
        ax.set_ylim(bottom=0)
        ax.tick_params(axis="x", rotation=45, labelsize=8)
        ax.set_ylabel("Ratio to fundamental")
        ax.legend(fontsize=7, loc="upper right")
        ax.set_facecolor("#1a1a2e")
        ax.grid(True, alpha=0.2)

    fig.tight_layout()
    fig.savefig(output_path, dpi=150, facecolor="#0a0a1a")
    plt.close(fig)
    print("  Saved: %s" % output_path)

    return grand_averages


def print_analysis(features_data, grand_averages):
    """Print summary analysis of the extracted features."""
    features = features_data["features"]

    print("\n" + "=" * 60)
    print("  HARMONIC FEATURE ANALYSIS")
    print("=" * 60)

    # Per-string statistics
    for si in range(6):
        string_feats = [f for f in features if f["string_idx"] == si]
        if not string_feats:
            continue

        centroids = [f["spectral_centroid"] for f in string_feats]
        inharmonicities = [f["inharmonicity"] for f in string_feats]

        print("\n  %s (%d samples):" % (STRING_NAMES[si], len(string_feats)))
        print("    Spectral centroid: mean=%.1f Hz, std=%.1f Hz" %
              (np.mean(centroids), np.std(centroids)))
        print("    Inharmonicity:     mean=%.4f, std=%.4f" %
              (np.mean(inharmonicities), np.std(inharmonicities)))

        if si in grand_averages:
            top3 = np.argsort(grand_averages[si])[::-1][:3]
            top3_str = ", ".join("H%d=%.2f" % (h + 2, grand_averages[si][h]) for h in top3)
            print("    Strongest harmonics: %s" % top3_str)

    # Cross-string comparison for same notes (shared pitches)
    print("\n  " + "-" * 56)
    print("  SHARED PITCH ANALYSIS (same note, different strings)")
    print("  " + "-" * 56)

    # Group by fundamental frequency
    freq_groups = defaultdict(list)
    for f in features:
        freq_groups[f["fundamental_freq"]].append(f)

    shared_count = 0
    for freq, group in sorted(freq_groups.items()):
        strings_present = set(f["string_idx"] for f in group)
        if len(strings_present) < 2:
            continue
        shared_count += 1
        if shared_count <= 5:  # Show first 5 examples
            print("\n    %.1f Hz (MIDI ~%d):" %
                  (freq, 69 + 12 * math.log2(freq / 440)))
            for si in sorted(strings_present):
                si_feats = [f for f in group if f["string_idx"] == si]
                avg_ratios = np.mean([f["harmonic_ratios"] for f in si_feats], axis=0)
                avg_centroid = np.mean([f["spectral_centroid"] for f in si_feats])
                print("      %-12s centroid=%6.1f Hz  ratios=[%s]" %
                      (STRING_NAMES[si], avg_centroid,
                       ", ".join("%.2f" % r for r in avg_ratios[:5])))

    print("\n    Total shared pitches: %d" % shared_count)
    print("=" * 60)


# == Main ==================================================================

def main():
    print("=" * 60)
    print("  Goertzel Harmonic Feature Extraction")
    print("=" * 60)

    # Extract features from the full dataset
    features_data, ratio_sums, ratio_counts = extract_all_features(
        DATASET_PATH, OUTPUT_JSON
    )

    # Generate per-string harmonic profile visualization
    print("\nGenerating visualization...")
    grand_averages = generate_visualization(ratio_sums, ratio_counts, OUTPUT_PNG)

    # Print analysis
    print_analysis(features_data, grand_averages)

    # Summary
    n_features = len(features_data["features"])
    json_size = os.path.getsize(OUTPUT_JSON) / 1024
    png_size = os.path.getsize(OUTPUT_PNG) / 1024

    print("\n" + "=" * 60)
    print("  OUTPUT FILES")
    print("=" * 60)
    print("  Features JSON: %s (%.1f KB, %d samples)" %
          (OUTPUT_JSON, json_size, n_features))
    print("  Analysis PNG:  %s (%.1f KB)" % (OUTPUT_PNG, png_size))
    print("=" * 60)


if __name__ == "__main__":
    main()
