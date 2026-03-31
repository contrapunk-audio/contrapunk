"""
Export pre-computed mel-spectrogram data as JSON for interactive web visualization.

Generates:
  - 6 showcase samples (one per string, open position, best RMS)
  - 3 confused pairs (same note on different strings)
  - 1 before/after quality pair (normal vs clipped)

Output: ui/static/spectrograms/ with index.json manifest.

Usage:
    cd <project_root>
    source ml/venv/bin/activate
    python ml/training/export_spectrograms.py
"""

import sys
import os
import json

import numpy as np
import librosa

# -- Paths -----------------------------------------------------------------

PROJECT_ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), "../.."))
DATASET_PATH = os.path.join(PROJECT_ROOT, "guitar_training_data.msgpack")
OUTPUT_ROOT = os.path.join(PROJECT_ROOT, "ui", "static", "spectrograms")

sys.path.insert(0, PROJECT_ROOT)
from ml.loader import GuitarDataset

# -- Mel-spectrogram params (same as training) -----------------------------

N_MELS = 64
N_FFT = 1024
HOP_LENGTH = 256

# -- Guitar tuning constants -----------------------------------------------

STRING_NAMES = ["E2", "A2", "D3", "G3", "B3", "E4"]
OPEN_MIDI = [40, 45, 50, 55, 59, 64]  # standard tuning MIDI note numbers


# -- Helpers ---------------------------------------------------------------

def compute_spectrogram(audio, sr):
    """Compute log-mel spectrogram matching training pipeline."""
    mel = librosa.feature.melspectrogram(
        y=audio, sr=sr,
        n_mels=N_MELS, n_fft=N_FFT, hop_length=HOP_LENGTH,
    )
    log_mel = librosa.power_to_db(mel, ref=np.max)
    return log_mel


def downsample_waveform(audio, target_points=500):
    """Downsample waveform to target number of points."""
    n = len(audio)
    if n <= target_points:
        return audio.tolist()
    indices = np.linspace(0, n - 1, target_points, dtype=int)
    return audio[indices].tolist()


def compute_rms_envelope(audio, sr, window_ms=10):
    """Compute RMS envelope in fixed-size windows."""
    window_samples = int(sr * window_ms / 1000)
    n_windows = len(audio) // window_samples
    if n_windows == 0:
        return [float(np.sqrt(np.mean(audio ** 2)))]
    rms_vals = []
    for i in range(n_windows):
        chunk = audio[i * window_samples : (i + 1) * window_samples]
        rms_vals.append(float(np.sqrt(np.mean(chunk ** 2))))
    return rms_vals


def round_nested(obj, decimals=2):
    """Round all float values in nested lists/dicts to given precision."""
    if isinstance(obj, float):
        return round(obj, decimals)
    if isinstance(obj, (list, tuple)):
        return [round_nested(v, decimals) for v in obj]
    if isinstance(obj, dict):
        return {k: round_nested(v, decimals) for k, v in obj.items()}
    if isinstance(obj, np.floating):
        return round(float(obj), decimals)
    if isinstance(obj, np.integer):
        return int(obj)
    return obj


def sample_to_json(sample, label_override=None):
    """Convert a GuitarSample to the JSON export format."""
    audio = sample.audio_np
    sr = sample.sample_rate
    mel = compute_spectrogram(audio, sr)

    midi_note = OPEN_MIDI[sample.string_idx] + sample.fret

    result = {
        "label": label_override or sample.label,
        "string_idx": int(sample.string_idx),
        "fret": int(sample.fret),
        "string_name": STRING_NAMES[sample.string_idx],
        "midi_note": midi_note,
        "sample_rate": int(sr),
        "duration": round(len(audio) / sr, 3),
        "rms": round(float(sample.rms), 3),
        "peak": round(float(sample.peak), 3),
        "spectrogram": {
            "n_mels": N_MELS,
            "n_fft": N_FFT,
            "hop_length": HOP_LENGTH,
            "time_frames": mel.shape[1],
            "data": mel.tolist(),
        },
        "waveform": downsample_waveform(audio),
        "rms_envelope": compute_rms_envelope(audio, sr),
    }

    return round_nested(result)


def write_json(data, path):
    """Write JSON data to file, creating parent dirs as needed."""
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, "w") as f:
        json.dump(data, f, separators=(",", ":"))
    size_kb = os.path.getsize(path) / 1024
    print(f"  Wrote {path} ({size_kb:.1f} KB)")
    return os.path.getsize(path)


# -- Export Functions ------------------------------------------------------

def export_showcase(ds):
    """Export one sample per string (open position, best RMS)."""
    print("\n--- Showcase: 6 open-string samples (best RMS) ---")
    files = []
    total_bytes = 0

    for si in range(6):
        candidates = ds.get_by_position(si, 0)  # open string
        if not candidates:
            print(f"  WARNING: No samples for string {si} fret 0, skipping")
            continue

        # Pick sample with highest RMS (best quality)
        best = max(candidates, key=lambda s: s.rms)
        filename = f"{STRING_NAMES[si]}_open.json"
        path = os.path.join(OUTPUT_ROOT, "showcase", filename)
        data = sample_to_json(best)
        total_bytes += write_json(data, path)
        files.append({
            "path": f"showcase/{filename}",
            "label": data["label"],
            "string_name": data["string_name"],
            "fret": 0,
            "rms": data["rms"],
            "category": "showcase",
        })

    return files, total_bytes


def export_confused_pairs(ds):
    """Export 3 confused pairs: same note on different strings."""
    print("\n--- Confused pairs: same note, different strings ---")

    # Confused pairs:
    #   A2 (MIDI 45): string 0 fret 5  vs  string 1 fret 0
    #   D3 (MIDI 50): string 1 fret 5  vs  string 2 fret 0
    #   G3 (MIDI 55): string 2 fret 5  vs  string 3 fret 0
    pairs = [
        ("A2", 45, (0, 5), (1, 0)),  # A2 on E string fret 5 vs A string open
        ("D3", 50, (1, 5), (2, 0)),  # D3 on A string fret 5 vs D string open
        ("G3", 55, (2, 5), (3, 0)),  # G3 on D string fret 5 vs G string open
    ]

    files = []
    total_bytes = 0

    for note_name, midi, (s1, f1), (s2, f2) in pairs:
        # First of pair
        candidates_a = ds.get_by_position(s1, f1)
        if candidates_a:
            best_a = max(candidates_a, key=lambda s: s.rms)
            fname_a = f"{note_name}_{STRING_NAMES[s1]}_string_fret{f1}.json"
            path_a = os.path.join(OUTPUT_ROOT, "confused_pairs", fname_a)
            data_a = sample_to_json(best_a)
            total_bytes += write_json(data_a, path_a)
            files.append({
                "path": f"confused_pairs/{fname_a}",
                "label": data_a["label"],
                "string_name": data_a["string_name"],
                "fret": f1,
                "note_name": note_name,
                "rms": data_a["rms"],
                "category": "confused_pair",
            })
        else:
            print(f"  WARNING: No samples for string {s1} fret {f1}")

        # Second of pair
        candidates_b = ds.get_by_position(s2, f2)
        if candidates_b:
            best_b = max(candidates_b, key=lambda s: s.rms)
            fname_b = f"{note_name}_{STRING_NAMES[s2]}_string_fret{f2}.json"
            # Use "open" naming if fret is 0
            if f2 == 0:
                fname_b = f"{note_name}_{STRING_NAMES[s2]}_string_open.json"
            path_b = os.path.join(OUTPUT_ROOT, "confused_pairs", fname_b)
            data_b = sample_to_json(best_b)
            total_bytes += write_json(data_b, path_b)
            files.append({
                "path": f"confused_pairs/{fname_b}",
                "label": data_b["label"],
                "string_name": data_b["string_name"],
                "fret": f2,
                "note_name": note_name,
                "rms": data_b["rms"],
                "category": "confused_pair",
            })
        else:
            print(f"  WARNING: No samples for string {s2} fret {f2}")

    return files, total_bytes


def export_quality(ds):
    """Export a normal sample and a clipped sample (peak > 0.99)."""
    print("\n--- Quality: normal vs clipped ---")

    files = []
    total_bytes = 0

    # Find a clipped sample (peak > 0.99)
    clipped_candidates = [
        s for s in ds.string_samples if s.peak > 0.99
    ]

    if not clipped_candidates:
        print("  WARNING: No clipped samples found (peak > 0.99)")
        print("  Falling back to highest-peak sample in dataset")
        clipped_candidates = sorted(ds.string_samples, key=lambda s: s.peak, reverse=True)

    clipped = clipped_candidates[0]
    print(f"  Clipped sample: {clipped.label} peak={clipped.peak:.4f} rms={clipped.rms:.4f}")

    # Find a normal sample: same string/fret position if possible, or
    # just a clean sample with moderate RMS and peak well below 0.99
    normal_candidates = [
        s for s in ds.get_by_position(clipped.string_idx, clipped.fret)
        if s.peak < 0.95 and s is not clipped
    ]
    if not normal_candidates:
        # Broaden search: any sample on same string
        normal_candidates = [
            s for s in ds.get_by_string(clipped.string_idx)
            if 0.3 < s.rms < 0.8 and s.peak < 0.95
        ]
    if not normal_candidates:
        # Broadest: any clean sample
        normal_candidates = [
            s for s in ds.string_samples
            if 0.3 < s.rms < 0.8 and s.peak < 0.95
        ]

    if normal_candidates:
        # Pick the one closest to median RMS
        median_rms = np.median([s.rms for s in normal_candidates])
        normal = min(normal_candidates, key=lambda s: abs(s.rms - median_rms))
    else:
        print("  WARNING: Could not find a good normal sample, using first sample")
        normal = ds.string_samples[0]

    print(f"  Normal sample: {normal.label} peak={normal.peak:.4f} rms={normal.rms:.4f}")

    # Export normal
    fname_normal = "normal_sample.json"
    path_normal = os.path.join(OUTPUT_ROOT, "quality", fname_normal)
    data_normal = sample_to_json(normal)
    total_bytes += write_json(data_normal, path_normal)
    files.append({
        "path": f"quality/{fname_normal}",
        "label": data_normal["label"],
        "string_name": data_normal["string_name"],
        "fret": data_normal["fret"],
        "rms": data_normal["rms"],
        "peak": data_normal["peak"],
        "category": "quality_normal",
    })

    # Export clipped
    fname_clipped = "clipped_sample.json"
    path_clipped = os.path.join(OUTPUT_ROOT, "quality", fname_clipped)
    data_clipped = sample_to_json(clipped)
    total_bytes += write_json(data_clipped, path_clipped)
    files.append({
        "path": f"quality/{fname_clipped}",
        "label": data_clipped["label"],
        "string_name": data_clipped["string_name"],
        "fret": data_clipped["fret"],
        "rms": data_clipped["rms"],
        "peak": data_clipped["peak"],
        "category": "quality_clipped",
    })

    return files, total_bytes


# -- Main ------------------------------------------------------------------

def main():
    print("=" * 60)
    print("  Contrapunk: Export Spectrograms for Web Visualization")
    print("=" * 60)

    ds = GuitarDataset.load(DATASET_PATH)
    ds.summary()

    all_files = []
    total_bytes = 0

    # 1. Showcase
    files, nbytes = export_showcase(ds)
    all_files.extend(files)
    total_bytes += nbytes

    # 2. Confused pairs
    files, nbytes = export_confused_pairs(ds)
    all_files.extend(files)
    total_bytes += nbytes

    # 3. Quality comparison
    files, nbytes = export_quality(ds)
    all_files.extend(files)
    total_bytes += nbytes

    # Write index.json
    index = {
        "generated": __import__("datetime").datetime.now().isoformat(),
        "dataset_version": ds.version,
        "spectrogram_params": {
            "n_mels": N_MELS,
            "n_fft": N_FFT,
            "hop_length": HOP_LENGTH,
        },
        "total_files": len(all_files),
        "total_size_kb": round(total_bytes / 1024, 1),
        "files": all_files,
    }
    index_path = os.path.join(OUTPUT_ROOT, "index.json")
    os.makedirs(OUTPUT_ROOT, exist_ok=True)
    with open(index_path, "w") as f:
        json.dump(index, f, indent=2)
    index_size = os.path.getsize(index_path)
    total_bytes += index_size

    print(f"\n{'=' * 60}")
    print(f"  EXPORT COMPLETE")
    print(f"  Files: {len(all_files) + 1} (including index.json)")
    print(f"  Total size: {total_bytes / 1024:.1f} KB")
    print(f"  Output: {OUTPUT_ROOT}")
    print(f"{'=' * 60}")


if __name__ == "__main__":
    main()
