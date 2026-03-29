"""
01_raw_analysis: Raw dataset visualization and quality checks.

Generates diagnostic plots from a captured guitar training dataset.

Usage:
    cd ml/processing/01_raw_analysis
    python analyze.py ../../../guitar_training_data.msgpack

Outputs (in current directory):
    waveform_grid.png      — 6x4 grid of raw waveforms per string
    amplitude_histogram.png — RMS distribution per string + noise
    pitch_accuracy.png     — detected vs expected MIDI scatter
    duration_histogram.png — sample length distribution
    sample_counts.png      — samples per class, colored by string
    clipping_report.txt    — list of clipped and silent samples
"""

import os
import sys
import random

# ── Make ml.loader importable ─────────────────────────────────────────
# When run from ml/processing/01_raw_analysis/, we need the project root
# on sys.path so that `from ml.loader import GuitarDataset` works.
_SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))
_PROJECT_ROOT = os.path.abspath(os.path.join(_SCRIPT_DIR, "..", "..", ".."))
sys.path.insert(0, _PROJECT_ROOT)

from ml.loader import GuitarDataset  # noqa: E402

import numpy as np  # noqa: E402
import matplotlib  # noqa: E402
matplotlib.use("Agg")  # non-interactive backend for saving PNGs
import matplotlib.pyplot as plt  # noqa: E402
import matplotlib.ticker as ticker  # noqa: E402

# ── Constants ─────────────────────────────────────────────────────────

STRING_NAMES = ["E2 (low)", "A2", "D3", "G3", "B3", "E4 (high)"]
STRING_COLORS = ["#e74c3c", "#e67e22", "#f1c40f", "#2ecc71", "#3498db", "#9b59b6"]
NOISE_COLOR = "#7f8c8d"

OUTPUT_DIR = _SCRIPT_DIR  # save outputs next to this script


def load_dataset(path):
    """Load the dataset, bail on error."""
    if not os.path.isfile(path):
        print(f"ERROR: File not found: {path}")
        sys.exit(1)
    return GuitarDataset.load(path)


# ── 1. Waveform Grid ─────────────────────────────────────────────────

def plot_waveform_grid(ds, out_path):
    """
    6 rows (strings) x 4 columns (random samples), showing raw waveforms.
    Each subplot shows the raw audio with amplitude on y-axis and time on x-axis.
    """
    print("  Generating waveform_grid.png ...")
    fig, axes = plt.subplots(6, 4, figsize=(16, 14))
    fig.suptitle("Raw Waveforms — 4 Random Samples per String", fontsize=14, y=0.98)

    for si in range(6):
        samples = ds.get_by_string(si)
        chosen = random.sample(samples, min(4, len(samples))) if samples else []

        for col in range(4):
            ax = axes[si][col]
            if col < len(chosen):
                s = chosen[col]
                audio = s.audio_np
                sr = s.sample_rate or ds.metadata.get("sample_rate", 44100)
                t = np.arange(len(audio)) / sr
                ax.plot(t, audio, linewidth=0.4, color=STRING_COLORS[si])
                ax.set_ylim(-1.05, 1.05)
                ax.set_title(f"{s.label}  rms={s.rms:.3f}", fontsize=7)
            else:
                ax.text(0.5, 0.5, "no data", ha="center", va="center",
                        transform=ax.transAxes, fontsize=8, color="#999")

            if col == 0:
                ax.set_ylabel(STRING_NAMES[si], fontsize=8)
            ax.tick_params(labelsize=6)

            if si == 5:
                ax.set_xlabel("Time (s)", fontsize=7)

    plt.tight_layout(rect=[0, 0, 1, 0.96])
    fig.savefig(out_path, dpi=150, bbox_inches="tight")
    plt.close(fig)
    print(f"    Saved: {out_path}")


# ── 2. Amplitude Histogram ───────────────────────────────────────────

def plot_amplitude_histogram(ds, out_path):
    """
    RMS distribution per string + noise, overlaid as semi-transparent histograms.
    """
    print("  Generating amplitude_histogram.png ...")
    fig, ax = plt.subplots(figsize=(10, 6))
    fig.suptitle("RMS Amplitude Distribution by String", fontsize=13)

    all_rms = [s.rms for s in ds.samples]
    bins = np.linspace(0, max(all_rms) * 1.05, 50)

    for si in range(6):
        rms_vals = [s.rms for s in ds.get_by_string(si)]
        if rms_vals:
            ax.hist(rms_vals, bins=bins, alpha=0.45, label=STRING_NAMES[si],
                    color=STRING_COLORS[si], edgecolor="none")

    noise_rms = [s.rms for s in ds.get_noise()]
    if noise_rms:
        ax.hist(noise_rms, bins=bins, alpha=0.45, label="Noise",
                color=NOISE_COLOR, edgecolor="none")

    ax.set_xlabel("RMS Amplitude")
    ax.set_ylabel("Count")
    ax.legend(fontsize=8)
    ax.grid(True, alpha=0.3)

    plt.tight_layout()
    fig.savefig(out_path, dpi=150, bbox_inches="tight")
    plt.close(fig)
    print(f"    Saved: {out_path}")


# ── 3. Pitch Accuracy ────────────────────────────────────────────────

def plot_pitch_accuracy(ds, out_path):
    """
    Scatter plot of detected vs expected MIDI notes, colored by string.
    Perfect pitch detection falls on the y=x diagonal.
    """
    print("  Generating pitch_accuracy.png ...")
    fig, ax = plt.subplots(figsize=(10, 10))
    fig.suptitle("Pitch Accuracy — Detected vs Expected MIDI Note", fontsize=13)

    for si in range(6):
        samples = ds.get_by_string(si)
        if not samples:
            continue
        expected = [s.expected_midi for s in samples]
        detected = [s.detected_midi for s in samples]
        ax.scatter(expected, detected, s=6, alpha=0.35,
                   color=STRING_COLORS[si], label=STRING_NAMES[si])

    # Perfect line
    midi_range = range(30, 110)
    ax.plot(midi_range, midi_range, "--", color="#333", linewidth=0.8,
            alpha=0.5, label="Perfect")

    ax.set_xlabel("Expected MIDI Note")
    ax.set_ylabel("Detected MIDI Note")
    ax.legend(fontsize=8, loc="upper left")
    ax.grid(True, alpha=0.3)
    ax.set_aspect("equal")

    plt.tight_layout()
    fig.savefig(out_path, dpi=150, bbox_inches="tight")
    plt.close(fig)
    print(f"    Saved: {out_path}")


# ── 4. Duration Histogram ────────────────────────────────────────────

def plot_duration_histogram(ds, out_path):
    """
    Distribution of sample lengths in seconds.
    Ideally all samples should be the same duration (the configured capture length).
    """
    print("  Generating duration_histogram.png ...")
    fig, ax = plt.subplots(figsize=(10, 5))
    fig.suptitle("Sample Duration Distribution", fontsize=13)

    durations = [s.duration_secs for s in ds.samples if s.duration_secs > 0]
    if not durations:
        ax.text(0.5, 0.5, "No audio data", ha="center", va="center",
                transform=ax.transAxes)
        fig.savefig(out_path, dpi=150, bbox_inches="tight")
        plt.close(fig)
        return

    ax.hist(durations, bins=50, color="#3498db", edgecolor="#2c3e50", alpha=0.8)
    ax.axvline(np.mean(durations), color="#e74c3c", linestyle="--",
               label=f"Mean: {np.mean(durations):.4f}s")
    ax.set_xlabel("Duration (seconds)")
    ax.set_ylabel("Count")
    ax.legend(fontsize=9)
    ax.grid(True, alpha=0.3)

    plt.tight_layout()
    fig.savefig(out_path, dpi=150, bbox_inches="tight")
    plt.close(fig)
    print(f"    Saved: {out_path}")


# ── 5. Sample Counts ─────────────────────────────────────────────────

def plot_sample_counts(ds, out_path):
    """
    Bar chart of samples per class, colored by string.
    Each bar represents one label (e.g., string_0_fret_0, noise_ambient).
    """
    print("  Generating sample_counts.png ...")

    # Count samples per label, preserving order
    from collections import Counter
    counts = Counter(s.label for s in ds.samples)

    # Sort: string samples by string_idx then fret, noise at the end
    def sort_key(label):
        if label.startswith("noise"):
            return (1, 0, label)
        parts = label.replace("string_", "").replace("fret_", "").split("_")
        try:
            return (0, int(parts[0]), int(parts[1]))
        except (IndexError, ValueError):
            return (0, 999, label)

    sorted_labels = sorted(counts.keys(), key=sort_key)
    sorted_counts = [counts[l] for l in sorted_labels]

    # Assign colors based on string
    def label_color(label):
        if label.startswith("noise"):
            return NOISE_COLOR
        for si in range(6):
            if label.startswith(f"string_{si}_"):
                return STRING_COLORS[si]
        return "#bdc3c7"

    colors = [label_color(l) for l in sorted_labels]

    fig, ax = plt.subplots(figsize=(max(14, len(sorted_labels) * 0.15), 6))
    fig.suptitle("Samples per Class", fontsize=13)

    x = np.arange(len(sorted_labels))
    ax.bar(x, sorted_counts, color=colors, edgecolor="none", width=0.8)

    # Only show every Nth tick label to avoid crowding
    step = max(1, len(sorted_labels) // 40)
    ax.set_xticks(x[::step])
    ax.set_xticklabels([sorted_labels[i] for i in range(0, len(sorted_labels), step)],
                       rotation=90, fontsize=5)
    ax.set_ylabel("Sample Count")
    ax.grid(True, axis="y", alpha=0.3)

    # Legend
    import matplotlib.patches as mpatches
    handles = []
    for si in range(6):
        handles.append(mpatches.Patch(color=STRING_COLORS[si], label=STRING_NAMES[si]))
    handles.append(mpatches.Patch(color=NOISE_COLOR, label="Noise"))
    ax.legend(handles=handles, fontsize=7, loc="upper right")

    plt.tight_layout()
    fig.savefig(out_path, dpi=150, bbox_inches="tight")
    plt.close(fig)
    print(f"    Saved: {out_path}")


# ── 6. Clipping Report ───────────────────────────────────────────────

def generate_clipping_report(ds, out_path):
    """
    Text report listing clipped (peak > 0.99) and silent (rms < 0.005) samples.
    """
    print("  Generating clipping_report.txt ...")

    clipped = [(i, s) for i, s in enumerate(ds.samples) if s.peak > 0.99]
    silent = [(i, s) for i, s in enumerate(ds.samples) if s.rms < 0.005]

    with open(out_path, "w") as f:
        f.write("=" * 60 + "\n")
        f.write("  Clipping & Silence Report\n")
        f.write("=" * 60 + "\n\n")

        f.write(f"Total samples: {len(ds.samples)}\n")
        f.write(f"Clipped (peak > 0.99): {len(clipped)}\n")
        f.write(f"Silent  (rms < 0.005): {len(silent)}\n\n")

        if clipped:
            f.write("-" * 60 + "\n")
            f.write("CLIPPED SAMPLES (peak > 0.99)\n")
            f.write("-" * 60 + "\n")
            f.write(f"{'idx':>6s}  {'label':30s}  {'peak':>8s}  {'rms':>8s}\n")
            for idx, s in clipped:
                f.write(f"{idx:6d}  {s.label:30s}  {s.peak:8.4f}  {s.rms:8.4f}\n")
            f.write("\n")
        else:
            f.write("No clipped samples found. Good!\n\n")

        if silent:
            f.write("-" * 60 + "\n")
            f.write("SILENT SAMPLES (rms < 0.005)\n")
            f.write("-" * 60 + "\n")
            f.write(f"{'idx':>6s}  {'label':30s}  {'peak':>8s}  {'rms':>8s}\n")
            for idx, s in silent:
                f.write(f"{idx:6d}  {s.label:30s}  {s.peak:8.4f}  {s.rms:8.4f}\n")
            f.write("\n")
        else:
            f.write("No silent samples found. Good!\n\n")

        # Summary assessment
        f.write("-" * 60 + "\n")
        f.write("ASSESSMENT\n")
        f.write("-" * 60 + "\n")
        total = len(ds.samples)
        if not clipped and not silent:
            f.write("Dataset looks clean. No clipping or silence issues.\n")
        else:
            if clipped:
                pct = len(clipped) / total * 100
                f.write(f"Clipping: {len(clipped)} samples ({pct:.1f}%) — "
                        f"reduce input gain or check cable.\n")
            if silent:
                pct = len(silent) / total * 100
                f.write(f"Silent: {len(silent)} samples ({pct:.1f}%) — "
                        f"may include noise floor captures (expected) "
                        f"or missed plucks (check these).\n")

    print(f"    Saved: {out_path}")


# ── Main ──────────────────────────────────────────────────────────────

def main():
    if len(sys.argv) < 2:
        print("Usage: python analyze.py <path_to_msgpack>")
        print("  e.g.: cd ml/processing/01_raw_analysis")
        print("        python analyze.py ../../../guitar_training_data.msgpack")
        sys.exit(1)

    dataset_path = sys.argv[1]
    ds = load_dataset(dataset_path)
    ds.summary()

    print("\nGenerating visualizations...\n")

    # Seed for reproducible random sample selection
    random.seed(42)

    plot_waveform_grid(ds, os.path.join(OUTPUT_DIR, "waveform_grid.png"))
    plot_amplitude_histogram(ds, os.path.join(OUTPUT_DIR, "amplitude_histogram.png"))
    plot_pitch_accuracy(ds, os.path.join(OUTPUT_DIR, "pitch_accuracy.png"))
    plot_duration_histogram(ds, os.path.join(OUTPUT_DIR, "duration_histogram.png"))
    plot_sample_counts(ds, os.path.join(OUTPUT_DIR, "sample_counts.png"))
    generate_clipping_report(ds, os.path.join(OUTPUT_DIR, "clipping_report.txt"))

    print("\nAll visualizations generated successfully.")
    print(f"Output directory: {OUTPUT_DIR}")


if __name__ == "__main__":
    main()
