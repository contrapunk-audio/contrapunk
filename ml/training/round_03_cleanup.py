"""
Round 3: Quality Cleanup Training

Removes low-quality samples from the dataset before training:
  - Clipped samples:     peak > 0.99
  - Near-silent samples: rms  < 0.005

Then re-trains all 3 models with 5-fold stratified CV and compares
against Round 1 (raw baseline) and Round 2 (onset-aligned).

Usage:
    cd <project_root>
    source ml/venv/bin/activate
    python ml/training/round_03_cleanup.py
"""

import sys
import os
import time
import json
import warnings
import numpy as np

import matplotlib
matplotlib.use("Agg")
import matplotlib.pyplot as plt
import seaborn as sns

from sklearn.ensemble import RandomForestClassifier
from sklearn.metrics import accuracy_score
from sklearn.model_selection import StratifiedKFold

import torch
import torch.nn as nn
import torch.optim as optim
from torch.utils.data import Dataset, DataLoader, SubsetRandomSampler

import librosa

warnings.filterwarnings("ignore", category=FutureWarning)

PROJECT_ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), "../.."))
DATASET_PATH = os.path.join(PROJECT_ROOT, "guitar_training_data.msgpack")
OUTPUT_DIR = os.path.join(os.path.dirname(__file__), "round_03")
ROUND_01_DIR = os.path.join(os.path.dirname(__file__), "round_01")
ROUND_02_DIR = os.path.join(os.path.dirname(__file__), "round_02")

sys.path.insert(0, PROJECT_ROOT)
from ml.loader import GuitarDataset

N_MELS = 64
N_FFT = 1024
HOP_LENGTH = 256
STRING_NAMES = ["E2 (low)", "A2", "D3", "G3", "B3", "E4 (high)"]

CLIP_THRESHOLD = 0.99
SILENCE_THRESHOLD = 0.005


# -- Quality Filtering ----------------------------------------------------

def filter_samples(ds):
    """Remove clipped and near-silent samples. Returns filtered list + stats."""
    clipped_per_string = [0] * 6
    silent_per_string = [0] * 6
    clipped_indices = []
    silent_indices = []
    kept = []

    for i, sample in enumerate(ds.samples):
        si = sample.string_idx
        if sample.peak > CLIP_THRESHOLD:
            clipped_per_string[si] += 1
            clipped_indices.append(i)
        elif sample.rms < SILENCE_THRESHOLD:
            silent_per_string[si] += 1
            silent_indices.append(i)
        else:
            kept.append(sample)

    total_clipped = len(clipped_indices)
    total_silent = len(silent_indices)
    total_removed = total_clipped + total_silent

    print("\n  Quality filter results:")
    print("  %-12s %8s %8s %8s" % ("String", "Clipped", "Silent", "Kept"))
    print("  " + "-" * 40)
    for si, sn in enumerate(STRING_NAMES):
        orig = len([s for s in ds.samples if s.string_idx == si])
        removed = clipped_per_string[si] + silent_per_string[si]
        print("  %-12s %8d %8d %8d" % (sn, clipped_per_string[si], silent_per_string[si], orig - removed))
    print("  " + "-" * 40)
    print("  %-12s %8d %8d %8d" % ("TOTAL", total_clipped, total_silent, len(kept)))
    print()
    print("  Before: %d samples" % len(ds.samples))
    print("  After:  %d samples (removed %d)" % (len(kept), total_removed))

    stats = {
        "total_before": len(ds.samples),
        "total_after": len(kept),
        "total_clipped": total_clipped,
        "total_silent": total_silent,
        "clipped_per_string": clipped_per_string,
        "silent_per_string": silent_per_string,
        "clipped_indices": clipped_indices,
        "silent_indices": silent_indices,
    }
    return kept, stats


# -- Feature Extraction ----------------------------------------------------

def extract_mel(audio, sr):
    mel = librosa.feature.melspectrogram(y=audio, sr=sr, n_mels=N_MELS, n_fft=N_FFT, hop_length=HOP_LENGTH)
    return librosa.power_to_db(mel, ref=np.max)


def prepare_dataset(samples):
    """Extract mel-spectrograms from filtered samples."""
    print("Extracting mel-spectrograms from %d samples..." % len(samples))
    raw_features, labels, metadata = [], [], []

    for i, sample in enumerate(samples):
        mel = extract_mel(sample.audio_np, sample.sample_rate)
        raw_features.append(mel)
        labels.append(sample.class_id - 1)
        metadata.append((sample.string_idx, sample.fret))

        if (i + 1) % 200 == 0:
            print("  %d/%d" % (i + 1, len(samples)))

    # Pad to uniform time dimension
    max_time = max(f.shape[1] for f in raw_features)
    features = []
    for f in raw_features:
        if f.shape[1] < max_time:
            f = np.pad(f, ((0, 0), (0, max_time - f.shape[1])), mode="constant", constant_values=f.min())
        features.append(f)

    labels = np.array(labels)
    print("  All %d samples processed, shape: %s" % (len(features), features[0].shape))
    return features, labels, metadata


# -- Models ----------------------------------------------------------------

class GuitarMelDataset(Dataset):
    def __init__(self, features, labels):
        self.X = torch.tensor(np.array([f[np.newaxis, ...] for f in features]), dtype=torch.float32)
        self.y = torch.tensor(labels, dtype=torch.long)
    def __len__(self): return len(self.y)
    def __getitem__(self, idx): return self.X[idx], self.y[idx]


class HybridCNN(nn.Module):
    def __init__(self, n_classes, input_shape):
        super().__init__()
        self.conv = nn.Sequential(
            nn.Conv2d(1, 16, 3, padding=1), nn.BatchNorm2d(16), nn.ReLU(), nn.MaxPool2d(2),
            nn.Conv2d(16, 32, 3, padding=1), nn.BatchNorm2d(32), nn.ReLU(), nn.MaxPool2d(2),
            nn.Conv2d(32, 64, 3, padding=1), nn.BatchNorm2d(64), nn.ReLU(), nn.AdaptiveAvgPool2d((4, 4)),
        )
        self.fc = nn.Sequential(nn.Flatten(), nn.Linear(1024, 256), nn.ReLU(), nn.Dropout(0.3), nn.Linear(256, n_classes))
    def forward(self, x): return self.fc(self.conv(x))


class PureCNN(nn.Module):
    def __init__(self, n_classes, input_shape):
        super().__init__()
        self.features = nn.Sequential(
            nn.Conv2d(1, 32, 3, padding=1), nn.BatchNorm2d(32), nn.ReLU(), nn.MaxPool2d(2),
            nn.Conv2d(32, 64, 3, padding=1), nn.BatchNorm2d(64), nn.ReLU(), nn.MaxPool2d(2),
            nn.Conv2d(64, 128, 3, padding=1), nn.BatchNorm2d(128), nn.ReLU(), nn.MaxPool2d(2),
            nn.Conv2d(128, 256, 3, padding=1), nn.BatchNorm2d(256), nn.ReLU(), nn.AdaptiveAvgPool2d((1, 1)),
        )
        self.classifier = nn.Sequential(nn.Flatten(), nn.Dropout(0.4), nn.Linear(256, n_classes))
    def forward(self, x): return self.classifier(self.features(x))


# -- Training --------------------------------------------------------------

def train_rf(features, labels, train_idx, val_idx):
    X_flat = np.array([f.flatten() for f in features])
    rf = RandomForestClassifier(n_estimators=200, n_jobs=-1, random_state=42)
    rf.fit(X_flat[train_idx], labels[train_idx])
    y_pred = rf.predict(X_flat[val_idx])
    return y_pred, labels[val_idx], accuracy_score(labels[val_idx], y_pred)


def train_cnn(model, dataset, train_idx, val_idx, epochs, lr):
    """Train CNN with ordered val_idx iteration (not SubsetRandomSampler) for aligned predictions."""
    model = model.to(torch.device("cpu"))
    loader = DataLoader(dataset, batch_size=32, sampler=SubsetRandomSampler(train_idx))
    criterion = nn.CrossEntropyLoss()
    optimizer = optim.Adam(model.parameters(), lr=lr, weight_decay=1e-4)
    scheduler = optim.lr_scheduler.CosineAnnealingLR(optimizer, T_max=epochs)

    best_acc, best_state = 0.0, None
    t0 = time.time()
    for epoch in range(epochs):
        model.train()
        for xb, yb in loader:
            optimizer.zero_grad(); criterion(model(xb), yb).backward(); optimizer.step()
        scheduler.step()

        # Validate by iterating val_idx in order (not via sampler)
        model.eval()
        correct, total = 0, 0
        with torch.no_grad():
            for i in val_idx:
                x, y = dataset[i]
                pred = model(x.unsqueeze(0)).argmax(1).item()
                correct += (pred == y.item())
                total += 1
        val_acc = correct / total
        if val_acc > best_acc:
            best_acc = val_acc
            best_state = {k: v.cpu().clone() for k, v in model.state_dict().items()}
        if (epoch + 1) % 10 == 0 or epoch == 0:
            print("    Epoch %3d: val_acc=%.4f" % (epoch + 1, val_acc))

    print("  Best: %.4f (%.1fs)" % (best_acc, time.time() - t0))
    model.load_state_dict(best_state)
    model.eval()

    # Collect predictions in val_idx order (aligned with indices)
    preds = np.zeros(len(val_idx), dtype=int)
    true = np.zeros(len(val_idx), dtype=int)
    with torch.no_grad():
        for i, idx in enumerate(val_idx):
            preds[i] = model(dataset[idx][0].unsqueeze(0)).argmax(1).item()
            true[i] = dataset[idx][1].item()
    return preds, true, best_acc, time.time() - t0


# -- Visualization ---------------------------------------------------------

def plot_removed_samples(filter_stats, path):
    """Bar chart showing removed samples per string (clipped vs silent)."""
    fig, ax = plt.subplots(figsize=(10, 6))
    x = np.arange(6)
    width = 0.35

    clipped = filter_stats["clipped_per_string"]
    silent = filter_stats["silent_per_string"]

    bars1 = ax.bar(x - width/2, clipped, width, label="Clipped (peak > 0.99)", color="#ff4444", alpha=0.85)
    bars2 = ax.bar(x + width/2, silent, width, label="Near-silent (rms < 0.005)", color="#4488ff", alpha=0.85)

    ax.set_xlabel("String")
    ax.set_ylabel("Samples Removed")
    ax.set_title("Removed Samples per String (Round 3 Quality Cleanup)")
    ax.set_xticks(x)
    ax.set_xticklabels(STRING_NAMES)
    ax.legend()

    # Add count labels on bars
    for bar in bars1:
        h = bar.get_height()
        if h > 0:
            ax.text(bar.get_x() + bar.get_width()/2., h + 0.2, str(int(h)),
                    ha="center", va="bottom", fontsize=9, fontweight="bold")
    for bar in bars2:
        h = bar.get_height()
        if h > 0:
            ax.text(bar.get_x() + bar.get_width()/2., h + 0.2, str(int(h)),
                    ha="center", va="bottom", fontsize=9, fontweight="bold")

    max_val = max(max(clipped), max(silent)) if max(max(clipped), max(silent)) > 0 else 1
    ax.set_ylim(0, max_val * 1.3 + 1)
    fig.tight_layout()
    fig.savefig(path, dpi=150)
    plt.close(fig)
    print("  Saved: %s" % path)


def plot_comparison_3rounds(r1, r2, r3, path):
    """Bar chart comparing all 3 rounds."""
    models = [r["name"] for r in r3]
    x = np.arange(len(models))
    width = 0.25

    fig, ax = plt.subplots(figsize=(12, 6))
    ax.bar(x - width, [r["accuracy"] for r in r1], width, label="Round 1 (raw)", color="#555577")
    ax.bar(x,         [r["accuracy"] for r in r2], width, label="Round 2 (onset)", color="#33ddff")
    ax.bar(x + width, [r["accuracy"] for r in r3], width, label="Round 3 (cleanup)", color="#33ff88")

    ax.set_ylabel("Accuracy")
    ax.set_title("Round 1 vs Round 2 vs Round 3")
    ax.set_xticks(x)
    ax.set_xticklabels(models)
    ax.legend()
    ax.set_ylim(0.85, 1.0)

    for i, (a1, a2, a3) in enumerate(zip(r1, r2, r3)):
        ax.text(i - width, a1["accuracy"] + 0.002, "%.1f%%" % (a1["accuracy"] * 100),
                ha="center", fontsize=8)
        ax.text(i,         a2["accuracy"] + 0.002, "%.1f%%" % (a2["accuracy"] * 100),
                ha="center", fontsize=8, color="#0099aa")
        ax.text(i + width, a3["accuracy"] + 0.002, "%.1f%%" % (a3["accuracy"] * 100),
                ha="center", fontsize=8, color="#009944")

    fig.tight_layout()
    fig.savefig(path, dpi=150)
    plt.close(fig)
    print("  Saved: %s" % path)


# -- Report ----------------------------------------------------------------

def generate_report(r3, r2, r1, filter_stats):
    path = os.path.join(OUTPUT_DIR, "ROUND_03_CLEANUP.md")

    n_clipped = filter_stats["total_clipped"]
    n_silent = filter_stats["total_silent"]
    n_before = filter_stats["total_before"]
    n_after = filter_stats["total_after"]

    lines = [
        "# Round 3: Quality Cleanup Results", "",
        "## What Changed",
        "- **Removed %d clipped samples** (peak > 0.99) -- distorted audio that confuses classifiers" % n_clipped,
        "- **Removed %d near-silent samples** (rms < 0.005) -- too quiet to contain useful signal" % n_silent,
        "- Dataset size: %d -> %d samples (%d removed, %.1f%%)" %
        (n_before, n_after, n_before - n_after, (n_before - n_after) / n_before * 100),
        "",
        "### Removed Samples by String", "",
        "| String | Clipped | Silent | Total Removed |",
        "|--------|---------|--------|---------------|",
    ]
    for si, sn in enumerate(STRING_NAMES):
        c = filter_stats["clipped_per_string"][si]
        s = filter_stats["silent_per_string"][si]
        lines.append("| %s | %d | %d | %d |" % (sn, c, s, c + s))
    lines.append("| **TOTAL** | **%d** | **%d** | **%d** |" % (n_clipped, n_silent, n_clipped + n_silent))

    lines.extend(["", "## Results Comparison", "",
                   "| Model | Round 1 (raw) | Round 2 (onset) | Round 3 (cleanup) | Delta (R3-R1) |",
                   "|-------|---------------|-----------------|-------------------|---------------|"])
    for a, b, c in zip(r1, r2, r3):
        d = c["accuracy"] - a["accuracy"]
        lines.append("| %s | %.1f%% | %.1f%% | **%.1f%%** | %+.1f%% |" %
                      (c["name"], a["accuracy"] * 100, b["accuracy"] * 100, c["accuracy"] * 100, d * 100))

    lines.extend(["", "## Per-String Accuracy (Pure CNN)", "",
                   "| String | Round 1 | Round 2 | Round 3 | Delta (R3-R1) |",
                   "|--------|---------|---------|---------|---------------|"])
    p1 = next(r for r in r1 if r["name"] == "Pure CNN")
    p2 = next(r for r in r2 if r["name"] == "Pure CNN")
    p3 = next(r for r in r3 if r["name"] == "Pure CNN")
    for si, sn in enumerate(STRING_NAMES):
        d = p3["per_string"][si] - p1["per_string"][si]
        lines.append("| %s | %.1f%% | %.1f%% | %.1f%% | %+.1f%% |" %
                      (sn, p1["per_string"][si] * 100, p2["per_string"][si] * 100,
                       p3["per_string"][si] * 100, d * 100))

    lines.extend(["",
                   "![Removed Samples](removed_samples.png)", "",
                   "![Comparison](comparison_bars.png)", "",
                   "*Generated: %s*" % time.strftime("%Y-%m-%d %H:%M")])

    with open(path, "w") as f:
        f.write("\n".join(lines))
    print("  Report: %s" % path)


# -- Main ------------------------------------------------------------------

def main():
    print("=" * 60)
    print("  Round 3: Quality Cleanup Training")
    print("=" * 60)
    os.makedirs(OUTPUT_DIR, exist_ok=True)

    # Load prior round results
    r1_path = os.path.join(ROUND_01_DIR, "results.json")
    r2_path = os.path.join(ROUND_02_DIR, "results.json")
    with open(r1_path) as f:
        r1_results = json.load(f)
    with open(r2_path) as f:
        r2_results = json.load(f)
    print("  Round 1: %s" % ", ".join("%s=%.1f%%" % (r["name"], r["accuracy"] * 100) for r in r1_results))
    print("  Round 2: %s" % ", ".join("%s=%.1f%%" % (r["name"], r["accuracy"] * 100) for r in r2_results))

    # Load and filter dataset
    ds = GuitarDataset.load(DATASET_PATH)
    filtered_samples, filter_stats = filter_samples(ds)

    # Extract features from clean dataset
    features, labels, metadata = prepare_dataset(filtered_samples)
    n_classes = len(np.unique(labels))
    print("  Classes: %d" % n_classes)

    # Plot removed samples
    plot_removed_samples(filter_stats, os.path.join(OUTPUT_DIR, "removed_samples.png"))

    # Prepare torch dataset
    torch_ds = GuitarMelDataset(features, labels)
    input_shape = features[0].shape
    skf = StratifiedKFold(n_splits=5, shuffle=True, random_state=42)
    folds = list(skf.split(np.zeros(len(labels)), labels))
    r3_results = []

    # ---------- Random Forest ----------
    print("\n" + "-" * 60 + "\n  Random Forest (quality-cleaned)\n" + "-" * 60)
    rf_accs, rf_ps = [], np.zeros((5, 6))
    for fold, (ti, vi) in enumerate(folds):
        yp, yv, acc = train_rf(features, labels, ti, vi)
        rf_accs.append(acc)
        print("  Fold %d: %.4f" % (fold + 1, acc))
        for si in range(6):
            m = [metadata[i][0] == si for i in vi]
            if any(m): rf_ps[fold, si] = accuracy_score(yv[m], yp[m])
    rf_mean = float(np.mean(rf_accs))
    r3_results.append({
        "name": "Random Forest", "accuracy": rf_mean,
        "per_string": rf_ps.mean(0).tolist(),
        "notes": "quality-cleaned, %d clipped + %d silent removed" % (filter_stats["total_clipped"], filter_stats["total_silent"]),
    })
    print("  Mean: %.4f" % rf_mean)

    # ---------- Hybrid CNN ----------
    print("\n" + "-" * 60 + "\n  Hybrid CNN (quality-cleaned)\n" + "-" * 60)
    hc_accs, hc_ps = [], np.zeros((5, 6))
    for fold, (ti, vi) in enumerate(folds):
        print("\n  Fold %d:" % (fold + 1))
        m = HybridCNN(n_classes, input_shape)
        yp, yv, acc, _ = train_cnn(m, torch_ds, ti, vi, 50, 1e-3)
        hc_accs.append(acc)
        for si in range(6):
            mask = [metadata[i][0] == si for i in vi]
            if any(mask): hc_ps[fold, si] = accuracy_score(yv[mask], yp[mask])
    hc_mean = float(np.mean(hc_accs))
    r3_results.append({
        "name": "Hybrid CNN", "accuracy": hc_mean,
        "per_string": hc_ps.mean(0).tolist(),
        "notes": "quality-cleaned", "training_curves": True,
    })
    print("  Mean: %.4f" % hc_mean)

    # ---------- Pure CNN ----------
    print("\n" + "-" * 60 + "\n  Pure CNN (quality-cleaned)\n" + "-" * 60)
    pc_accs, pc_ps = [], np.zeros((5, 6))
    for fold, (ti, vi) in enumerate(folds):
        print("\n  Fold %d:" % (fold + 1))
        m = PureCNN(n_classes, input_shape)
        yp, yv, acc, _ = train_cnn(m, torch_ds, ti, vi, 60, 8e-4)
        pc_accs.append(acc)
        for si in range(6):
            mask = [metadata[i][0] == si for i in vi]
            if any(mask): pc_ps[fold, si] = accuracy_score(yv[mask], yp[mask])
    pc_mean = float(np.mean(pc_accs))
    r3_results.append({
        "name": "Pure CNN", "accuracy": pc_mean,
        "per_string": pc_ps.mean(0).tolist(),
        "notes": "quality-cleaned", "training_curves": True,
    })
    print("  Mean: %.4f" % pc_mean)

    # ---------- Report & Save ----------
    print("\n" + "-" * 60 + "\n  GENERATING REPORT\n" + "-" * 60)
    plot_comparison_3rounds(r1_results, r2_results, r3_results,
                            os.path.join(OUTPUT_DIR, "comparison_bars.png"))

    with open(os.path.join(OUTPUT_DIR, "results.json"), "w") as f:
        json.dump(r3_results, f, indent=2)
    print("  Saved: %s" % os.path.join(OUTPUT_DIR, "results.json"))

    generate_report(r3_results, r2_results, r1_results, filter_stats)

    # Final summary
    print("\n" + "=" * 60)
    print("  ROUND 3 vs ROUND 1 vs ROUND 2")
    print("=" * 60)
    print("  %-20s  %10s  %10s  %10s  %8s" % ("Model", "Round 1", "Round 2", "Round 3", "R3-R1"))
    print("  " + "-" * 62)
    for a, b, c in zip(r1_results, r2_results, r3_results):
        d = c["accuracy"] - a["accuracy"]
        print("  %-20s  %9.1f%%  %9.1f%%  %9.1f%%  %+7.1f%%" %
              (c["name"], a["accuracy"] * 100, b["accuracy"] * 100, c["accuracy"] * 100, d * 100))
    print("=" * 60)
    print("  Dataset: %d -> %d (-%d clipped, -%d silent)" %
          (filter_stats["total_before"], filter_stats["total_after"],
           filter_stats["total_clipped"], filter_stats["total_silent"]))
    print("=" * 60)


if __name__ == "__main__":
    main()
