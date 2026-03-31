"""
Export representative WAV samples from the Contrapunk guitar training dataset.

Produces three categories of samples for the web diary:
  1. by_class/   — 1 best sample per (string, fret) class (highest RMS)
  2. showcase/    — 3 per string (open, fret 5, fret 12) = 18 samples
  3. confused_pairs/ — same-note pairs on different strings

Output: ui/static/samples/ with an index.json manifest.

Usage:
    source ml/venv/bin/activate
    python ml/training/export_samples.py
"""

import json
import os
import sys
from pathlib import Path

import numpy as np

# ── Project paths ────────────────────────────────────────────────────
PROJECT_ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(PROJECT_ROOT))

from ml.loader import GuitarDataset

DATASET_PATH = PROJECT_ROOT / "guitar_training_data.msgpack"
OUTPUT_DIR = PROJECT_ROOT / "ui" / "static" / "samples"

SAMPLE_RATE = 48000
DURATION_SECS = 0.5
DURATION_SAMPLES = int(SAMPLE_RATE * DURATION_SECS)

STRING_NAMES = ["E2", "A2", "D3", "G3", "B3", "E4"]

# Confused pairs: (note_name, midi_note, [(string_idx, fret), ...])
CONFUSED_PAIRS = [
    ("A2", 45, [(0, 5), (1, 0)]),
    ("D3", 50, [(1, 5), (2, 0)]),
    ("G3", 55, [(2, 5), (3, 0)]),
    ("B3", 59, [(3, 4), (4, 0)]),
    ("E4", 64, [(4, 5), (5, 0)]),
]


def pick_best(samples):
    """Return the sample with the highest RMS (best signal quality)."""
    if not samples:
        return None
    return max(samples, key=lambda s: s.rms)


def trim_or_pad(audio, target_len):
    """Trim to target_len or zero-pad if shorter."""
    if len(audio) >= target_len:
        return audio[:target_len]
    return np.pad(audio, (0, target_len - len(audio)), mode="constant")


def write_wav(path, audio_f32, sr):
    """Write a float32 array as 16-bit PCM WAV using soundfile."""
    import soundfile as sf

    # Clip to [-1, 1] then convert to 16-bit PCM via subtype
    audio_clipped = np.clip(audio_f32, -1.0, 1.0)
    sf.write(str(path), audio_clipped, sr, subtype="PCM_16")


def fret_label(fret):
    """Human-readable fret label for filenames."""
    if fret == 0:
        return "open"
    return f"fret{fret}"


def string_label_for_confused(string_idx):
    """Return the string letter for confused-pair filenames."""
    return STRING_NAMES[string_idx].rstrip("0123456789")


def export_sample(sample, filepath, index_entries):
    """Export a single sample to WAV and record metadata."""
    audio = trim_or_pad(sample.audio_np, DURATION_SAMPLES)
    write_wav(filepath, audio, SAMPLE_RATE)

    entry = {
        "filename": str(filepath.relative_to(OUTPUT_DIR)),
        "string_idx": sample.string_idx,
        "fret": sample.fret,
        "midi_note": sample.expected_midi,
        "rms": round(float(sample.rms), 6),
        "duration": DURATION_SECS,
    }
    index_entries.append(entry)
    return entry


def main():
    # ── Load dataset ─────────────────────────────────────────────────
    ds = GuitarDataset.load(str(DATASET_PATH))
    note_samples = ds.string_samples
    print(f"\n  Note samples available: {len(note_samples)}")
    print(f"  Unique classes: {len(ds.class_ids)}\n")

    # ── Prepare output dirs ──────────────────────────────────────────
    by_class_dir = OUTPUT_DIR / "by_class"
    showcase_dir = OUTPUT_DIR / "showcase"
    confused_dir = OUTPUT_DIR / "confused_pairs"
    for d in (by_class_dir, showcase_dir, confused_dir):
        d.mkdir(parents=True, exist_ok=True)

    index_entries = []
    exported = 0
    skipped_classes = []

    # ── 1. By-class: best sample per (string, fret) ─────────────────
    print("Exporting by_class/ (1 per class, best RMS) ...")
    for string_idx in range(6):
        for fret in range(23):  # 0..22
            candidates = ds.get_by_position(string_idx, fret)
            best = pick_best(candidates)
            if best is None:
                skipped_classes.append((string_idx, fret))
                continue
            fname = f"string_{string_idx}_fret_{fret}.wav"
            export_sample(best, by_class_dir / fname, index_entries)
            exported += 1

    if skipped_classes:
        print(f"  Skipped {len(skipped_classes)} empty classes "
              f"(no samples in dataset)")

    # ── 2. Showcase: open, fret 5, fret 12 per string ───────────────
    print("Exporting showcase/ (open, fret5, fret12 per string) ...")
    showcase_frets = [0, 5, 12]
    for string_idx in range(6):
        name = STRING_NAMES[string_idx]
        for fret in showcase_frets:
            candidates = ds.get_by_position(string_idx, fret)
            best = pick_best(candidates)
            if best is None:
                print(f"  WARNING: No sample for {name} fret {fret}")
                continue
            fl = fret_label(fret)
            fname = f"{name}_{fl}.wav"
            export_sample(best, showcase_dir / fname, index_entries)
            exported += 1

    # ── 3. Confused pairs ────────────────────────────────────────────
    print("Exporting confused_pairs/ ...")
    for note_name, midi, positions in CONFUSED_PAIRS:
        for string_idx, fret in positions:
            candidates = ds.get_by_position(string_idx, fret)
            best = pick_best(candidates)
            if best is None:
                sname = STRING_NAMES[string_idx]
                print(f"  WARNING: No sample for {note_name} on "
                      f"{sname} string fret {fret}")
                continue
            sname = STRING_NAMES[string_idx]
            s_letter = string_label_for_confused(string_idx)
            fl = fret_label(fret)
            fname = f"{note_name}_on_{s_letter}_string_{fl}.wav"
            export_sample(best, confused_dir / fname, index_entries)
            exported += 1

    # ── Write index.json ─────────────────────────────────────────────
    index_path = OUTPUT_DIR / "index.json"
    with open(index_path, "w") as f:
        json.dump(index_entries, f, indent=2)
    print(f"\nWrote {index_path}")

    # ── Report ───────────────────────────────────────────────────────
    total_bytes = sum(
        p.stat().st_size
        for p in OUTPUT_DIR.rglob("*.wav")
    )
    index_bytes = index_path.stat().st_size
    total_bytes += index_bytes

    print(f"\n{'='*50}")
    print(f"  Exported {exported} WAV files")
    print(f"  Total size: {total_bytes / 1024:.1f} KB "
          f"({total_bytes / (1024*1024):.2f} MB)")
    print(f"  index.json: {index_bytes} bytes "
          f"({len(index_entries)} entries)")
    print(f"{'='*50}\n")


if __name__ == "__main__":
    main()
