"""
Round 2: Onset-Aligned Training

Preprocesses all 1,380 samples by detecting the pluck onset and
re-extracting mel-spectrograms starting from the attack moment.
Then re-trains all 3 models and compares against Round 1 baseline.

Usage:
    cd <project_root>
    source ml/venv/bin/activate
    python ml/training/round_02_onset.py
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
OUTPUT_DIR = os.path.join(os.path.dirname(__file__), "round_02")
ROUND_01_DIR = os.path.join(os.path.dirname(__file__), "round_01")

sys.path.insert(0, PROJECT_ROOT)
from ml.loader import GuitarDataset

N_MELS = 64
N_FFT = 1024
HOP_LENGTH = 256
SAMPLE_RATE = 48000
CAPTURE_SAMPLES = int(SAMPLE_RATE * 0.5)
STRING_NAMES = ["E2 (low)", "A2", "D3", "G3", "B3", "E4 (high)"]


# -- Onset Detection -------------------------------------------------------

def detect_onset(audio, sr):
    """Detect pluck onset. Returns sample index, or 0 if none found."""
    onset_env = librosa.onset.onset_strength(y=audio, sr=sr, hop_length=256)
    onset_frames = librosa.onset.onset_detect(
        onset_envelope=onset_env, sr=sr, hop_length=256,
        backtrack=True, units="frames",
    )
    if len(onset_frames) == 0:
        # Fallback: RMS threshold
        rms = librosa.feature.rms(y=audio, frame_length=512, hop_length=256)[0]
        if len(rms) > 5:
            initial_rms = np.mean(rms[:3])
            threshold = max(initial_rms * 3, 0.005)
            for i, r in enumerate(rms):
                if r > threshold:
                    return max(0, i * 256 - 128)
        return 0

    onset_sample = librosa.frames_to_samples(onset_frames[0], hop_length=256)
    return max(0, onset_sample - 64)  # slight backtrack to catch attack start


def align_sample(audio, sr, target_length=CAPTURE_SAMPLES):
    """Align sample to onset, pad/truncate to target_length."""
    onset = detect_onset(audio, sr)
    aligned = audio[onset:]
    if len(aligned) < target_length:
        aligned = np.concatenate([aligned, np.zeros(target_length - len(aligned), dtype=np.float32)])
    elif len(aligned) > target_length:
        aligned = aligned[:target_length]
    return aligned, onset


# -- Feature Extraction ----------------------------------------------------

def extract_mel(audio, sr):
    mel = librosa.feature.melspectrogram(y=audio, sr=sr, n_mels=N_MELS, n_fft=N_FFT, hop_length=HOP_LENGTH)
    return librosa.power_to_db(mel, ref=np.max)


def prepare_dataset_aligned(ds):
    """Extract onset-aligned mel-spectrograms from all samples."""
    print("Detecting onsets and extracting aligned spectrograms...")
    raw_features, labels, metadata, onset_stats = [], [], [], []

    for i, sample in enumerate(ds.samples):
        aligned, onset_idx = align_sample(sample.audio_np, sample.sample_rate)
        mel = extract_mel(aligned, sample.sample_rate)
        raw_features.append(mel)
        labels.append(sample.class_id - 1)
        metadata.append((sample.string_idx, sample.fret))
        onset_stats.append((onset_idx / sample.sample_rate) * 1000)

        if (i + 1) % 200 == 0:
            print("  %d/%d -- avg onset: %.1fms" % (i + 1, len(ds.samples), np.mean(onset_stats[-200:])))

    max_time = max(f.shape[1] for f in raw_features)
    features = []
    for f in raw_features:
        if f.shape[1] < max_time:
            f = np.pad(f, ((0, 0), (0, max_time - f.shape[1])), mode="constant", constant_values=f.min())
        features.append(f)

    labels = np.array(labels)
    print("  All %d samples aligned, shape: %s" % (len(features), features[0].shape))
    print("  Onset: min=%.1fms, max=%.1fms, mean=%.1fms, median=%.1fms" %
          (min(onset_stats), max(onset_stats), np.mean(onset_stats), np.median(onset_stats)))
    return features, labels, metadata, onset_stats


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

    preds = np.zeros(len(val_idx), dtype=int)
    true = np.zeros(len(val_idx), dtype=int)
    with torch.no_grad():
        for i, idx in enumerate(val_idx):
            preds[i] = model(dataset[idx][0].unsqueeze(0)).argmax(1).item()
            true[i] = dataset[idx][1].item()
    return preds, true, best_acc, time.time() - t0


# -- Visualization ---------------------------------------------------------

def plot_onset_distribution(onset_stats, path):
    fig, ax = plt.subplots(figsize=(10, 5))
    ax.hist(onset_stats, bins=50, color="#33ddff", alpha=0.7, edgecolor="#0a0a1a")
    ax.axvline(np.median(onset_stats), color="#ff3388", linestyle="--",
               label="median: %.1fms" % np.median(onset_stats))
    ax.set_xlabel("Onset position (ms)")
    ax.set_ylabel("Count")
    ax.set_title("Onset Detection Distribution Across 1,380 Samples")
    ax.legend()
    fig.tight_layout(); fig.savefig(path, dpi=150); plt.close(fig)
    print("  Saved: %s" % path)


def plot_before_after(ds, path):
    fig, axes = plt.subplots(6, 2, figsize=(14, 18))
    fig.suptitle("Before (Raw) vs After (Onset-Aligned) Spectrograms", fontsize=14)
    for si in range(6):
        samples = [s for s in ds.samples if s.string_idx == si]
        sample = max(samples[:10], key=lambda s: s.rms)
        audio, sr = sample.audio_np, sample.sample_rate

        axes[si][0].imshow(extract_mel(audio, sr), aspect="auto", origin="lower", cmap="magma")
        axes[si][0].set_title("%s -- Raw" % STRING_NAMES[si], fontsize=10)
        axes[si][0].set_ylabel("Mel bin")

        aligned, onset = align_sample(audio, sr)
        axes[si][1].imshow(extract_mel(aligned, sr), aspect="auto", origin="lower", cmap="magma")
        axes[si][1].set_title("%s -- Aligned (onset: %.0fms)" % (STRING_NAMES[si], onset / sr * 1000), fontsize=10)

    axes[5][0].set_xlabel("Time frame"); axes[5][1].set_xlabel("Time frame")
    fig.tight_layout(); fig.savefig(path, dpi=150); plt.close(fig)
    print("  Saved: %s" % path)


def plot_comparison(r1, r2, path):
    models = [r["name"] for r in r2]
    x = np.arange(len(models))
    fig, ax = plt.subplots(figsize=(10, 6))
    ax.bar(x - 0.175, [r["accuracy"] for r in r1], 0.35, label="Round 1 (raw)", color="#555577")
    ax.bar(x + 0.175, [r["accuracy"] for r in r2], 0.35, label="Round 2 (aligned)", color="#33ddff")
    ax.set_ylabel("Accuracy"); ax.set_title("Round 1 vs Round 2"); ax.set_xticks(x)
    ax.set_xticklabels(models); ax.legend(); ax.set_ylim(0.85, 1.0)
    for i, (a1, a2) in enumerate(zip(r1, r2)):
        ax.text(i - 0.175, a1["accuracy"] + 0.002, "%.1f%%" % (a1["accuracy"] * 100), ha="center", fontsize=9)
        ax.text(i + 0.175, a2["accuracy"] + 0.002, "%.1f%%" % (a2["accuracy"] * 100), ha="center", fontsize=9, color="#33ddff")
    fig.tight_layout(); fig.savefig(path, dpi=150); plt.close(fig)
    print("  Saved: %s" % path)


# -- Report ----------------------------------------------------------------

def generate_report(r2, r1, onset_stats):
    path = os.path.join(OUTPUT_DIR, "ROUND_02_ONSET.md")
    lines = [
        "# Round 2: Onset-Aligned Training Results", "",
        "## Change from Round 1",
        "- **What changed:** Added onset detection to align all samples to the pluck attack",
        "- **Why:** Raw samples had variable silence before the pluck, wasting spectrogram resolution",
        "- **Onset stats:** median=%.1fms, mean=%.1fms, max=%.1fms" %
        (np.median(onset_stats), np.mean(onset_stats), max(onset_stats)), "",
        "## Results Comparison", "",
        "| Model | Round 1 | Round 2 | Delta |",
        "|-------|---------|---------|-------|",
    ]
    for a, b in zip(r1, r2):
        d = b["accuracy"] - a["accuracy"]
        lines.append("| %s | %.1f%% | **%.1f%%** | %+.1f%% |" %
                      (b["name"], a["accuracy"] * 100, b["accuracy"] * 100, d * 100))

    lines.extend(["", "## Per-String (Pure CNN)", "",
                   "| String | Round 1 | Round 2 | Delta |",
                   "|--------|---------|---------|-------|"])
    p1 = next(r for r in r1 if r["name"] == "Pure CNN")
    p2 = next(r for r in r2 if r["name"] == "Pure CNN")
    for si, sn in enumerate(STRING_NAMES):
        d = p2["per_string"][si] - p1["per_string"][si]
        lines.append("| %s | %.1f%% | %.1f%% | %+.1f%% |" %
                      (sn, p1["per_string"][si] * 100, p2["per_string"][si] * 100, d * 100))

    lines.extend(["", "![Onset Distribution](onset_distribution.png)", "",
                   "![Before After](before_after_spectrograms.png)", "",
                   "![Comparison](comparison_bars.png)", "",
                   "*Generated: %s*" % time.strftime("%Y-%m-%d %H:%M")])

    with open(path, "w") as f:
        f.write("\n".join(lines))
    print("  Report: %s" % path)


# -- Main ------------------------------------------------------------------

def main():
    print("=" * 60)
    print("  Round 2: Onset-Aligned Training")
    print("=" * 60)
    os.makedirs(OUTPUT_DIR, exist_ok=True)

    with open(os.path.join(ROUND_01_DIR, "results.json")) as f:
        r1_results = json.load(f)
    print("  Round 1 loaded: %s" % ", ".join("%s=%.1f%%" % (r["name"], r["accuracy"] * 100) for r in r1_results))

    ds = GuitarDataset.load(DATASET_PATH)
    features, labels, metadata, onset_stats = prepare_dataset_aligned(ds)
    n_classes = len(np.unique(labels))

    plot_onset_distribution(onset_stats, os.path.join(OUTPUT_DIR, "onset_distribution.png"))
    plot_before_after(ds, os.path.join(OUTPUT_DIR, "before_after_spectrograms.png"))

    torch_ds = GuitarMelDataset(features, labels)
    input_shape = features[0].shape
    skf = StratifiedKFold(n_splits=5, shuffle=True, random_state=42)
    folds = list(skf.split(np.zeros(len(labels)), labels))
    r2_results = []

    # RF
    print("\n" + "-" * 60 + "\n  Random Forest (onset-aligned)\n" + "-" * 60)
    rf_accs, rf_ps = [], np.zeros((5, 6))
    for fold, (ti, vi) in enumerate(folds):
        yp, yv, acc = train_rf(features, labels, ti, vi)
        rf_accs.append(acc)
        print("  Fold %d: %.4f" % (fold + 1, acc))
        for si in range(6):
            m = [metadata[i][0] == si for i in vi]
            if any(m): rf_ps[fold, si] = accuracy_score(yv[m], yp[m])
    r2_results.append({"name": "Random Forest", "accuracy": float(np.mean(rf_accs)),
                        "per_string": rf_ps.mean(0).tolist(), "notes": "onset-aligned"})

    # Hybrid CNN
    print("\n" + "-" * 60 + "\n  Hybrid CNN (onset-aligned)\n" + "-" * 60)
    hc_accs, hc_ps = [], np.zeros((5, 6))
    for fold, (ti, vi) in enumerate(folds):
        print("\n  Fold %d:" % (fold + 1))
        m = HybridCNN(n_classes, input_shape)
        yp, yv, acc, _ = train_cnn(m, torch_ds, ti, vi, 50, 1e-3)
        hc_accs.append(acc)
        for si in range(6):
            mask = [metadata[i][0] == si for i in vi]
            if any(mask): hc_ps[fold, si] = accuracy_score(yv[mask], yp[mask])
    r2_results.append({"name": "Hybrid CNN", "accuracy": float(np.mean(hc_accs)),
                        "per_string": hc_ps.mean(0).tolist(), "notes": "onset-aligned", "training_curves": True})

    # Pure CNN
    print("\n" + "-" * 60 + "\n  Pure CNN (onset-aligned)\n" + "-" * 60)
    pc_accs, pc_ps = [], np.zeros((5, 6))
    for fold, (ti, vi) in enumerate(folds):
        print("\n  Fold %d:" % (fold + 1))
        m = PureCNN(n_classes, input_shape)
        yp, yv, acc, _ = train_cnn(m, torch_ds, ti, vi, 60, 8e-4)
        pc_accs.append(acc)
        for si in range(6):
            mask = [metadata[i][0] == si for i in vi]
            if any(mask): pc_ps[fold, si] = accuracy_score(yv[mask], yp[mask])
    r2_results.append({"name": "Pure CNN", "accuracy": float(np.mean(pc_accs)),
                        "per_string": pc_ps.mean(0).tolist(), "notes": "onset-aligned", "training_curves": True})

    # Report
    print("\n" + "-" * 60 + "\n  GENERATING REPORT\n" + "-" * 60)
    plot_comparison(r1_results, r2_results, os.path.join(OUTPUT_DIR, "comparison_bars.png"))
    with open(os.path.join(OUTPUT_DIR, "results.json"), "w") as f:
        json.dump(r2_results, f, indent=2)
    generate_report(r2_results, r1_results, onset_stats)

    print("\n" + "=" * 60)
    print("  ROUND 2 vs ROUND 1")
    print("=" * 60)
    for a, b in zip(r1_results, r2_results):
        d = b["accuracy"] - a["accuracy"]
        print("  %20s  %.1f%% -> %.1f%%  (%+.1f%%)" % (b["name"], a["accuracy"] * 100, b["accuracy"] * 100, d * 100))
    print("=" * 60)


if __name__ == "__main__":
    main()
