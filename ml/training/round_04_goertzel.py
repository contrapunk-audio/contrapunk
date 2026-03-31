"""
Round 4: Goertzel Harmonic Feature Fusion

Fuses physics-based Goertzel harmonic features with mel-spectrogram
features to improve string identification accuracy. The hypothesis:
mel-spectrograms capture pitch (fret) well but struggle with string
identity for shared pitches; harmonic ratios encode the string's
physical timbre signature.

Strategy:
  - RF: concatenate harmonic ratios to flattened mel-spectrogram vector
  - Hybrid CNN: two-branch architecture -- CNN on spectrogram + FC on
    harmonics, fused before final classification
  - Pure CNN: same two-branch approach

Usage:
    cd <project_root>
    source ml/venv/bin/activate
    python ml/training/round_04_goertzel.py

Prerequisites:
    - goertzel_features.json must exist (run goertzel.py first)
    - Round 1 & 2 results.json must exist for comparison
"""

import sys
import os
import json
import time
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
GOERTZEL_JSON = os.path.join(os.path.dirname(__file__), "goertzel_features.json")
OUTPUT_DIR = os.path.join(os.path.dirname(__file__), "round_04")
ROUND_01_DIR = os.path.join(os.path.dirname(__file__), "round_01")
ROUND_02_DIR = os.path.join(os.path.dirname(__file__), "round_02")

sys.path.insert(0, PROJECT_ROOT)
from ml.loader import GuitarDataset

N_MELS = 64
N_FFT = 1024
HOP_LENGTH = 256
STRING_NAMES = ["E2 (low)", "A2", "D3", "G3", "B3", "E4 (high)"]


# == Data Loading ==========================================================

def load_goertzel_features(path):
    """Load Goertzel features from JSON, indexed by label.

    Returns the raw data dict and a label-indexed lookup dict.
    """
    with open(path) as f:
        data = json.load(f)

    by_label = {}
    for feat in data["features"]:
        label = feat["label"]
        if label not in by_label:
            by_label[label] = []
        by_label[label].append(feat)

    print("  Goertzel features loaded: %d samples, %d harmonics" %
          (len(data["features"]), data["params"]["n_harmonics"]))
    return data, by_label


def extract_mel(audio, sr):
    """Extract log-mel spectrogram."""
    mel = librosa.feature.melspectrogram(
        y=audio, sr=sr, n_mels=N_MELS, n_fft=N_FFT, hop_length=HOP_LENGTH
    )
    return librosa.power_to_db(mel, ref=np.max)


def prepare_dataset(ds, goertzel_by_label):
    """Extract mel-spectrograms + Goertzel features for all samples.

    Returns:
        mel_features: list of (N_MELS, TIME) arrays
        harmonic_features: (N, n_harmonic_dims) array, standardized
        labels: (N,) int array (zero-indexed class IDs)
        metadata: list of (string_idx, fret) tuples
    """
    print("Extracting mel-spectrograms + Goertzel features...")

    mel_features = []
    harmonic_features = []
    labels = []
    metadata = []

    for i, sample in enumerate(ds.samples):
        mel = extract_mel(sample.audio_np, sample.sample_rate)
        mel_features.append(mel)

        label = sample.label
        if label in goertzel_by_label and len(goertzel_by_label[label]) > 0:
            group = goertzel_by_label[label]
            feat = group[i % len(group)]
            ratios = feat["harmonic_ratios"]
            centroid = feat["spectral_centroid"]
            inharm = feat["inharmonicity"]
            harm_vec = ratios + [centroid, inharm]
        else:
            harm_vec = [0.0] * 11  # 9 ratios + centroid + inharmonicity

        harmonic_features.append(harm_vec)
        labels.append(sample.class_id - 1)
        metadata.append((sample.string_idx, sample.fret))

        if (i + 1) % 200 == 0:
            print("  %d/%d" % (i + 1, len(ds.samples)))

    max_time = max(f.shape[1] for f in mel_features)
    padded = []
    for f in mel_features:
        if f.shape[1] < max_time:
            f = np.pad(f, ((0, 0), (0, max_time - f.shape[1])),
                       mode="constant", constant_values=f.min())
        padded.append(f)

    labels = np.array(labels)
    harmonic_features = np.array(harmonic_features, dtype=np.float32)

    # Standardize harmonic features (zero-mean, unit-variance)
    harm_mean = harmonic_features.mean(axis=0)
    harm_std = harmonic_features.std(axis=0)
    harm_std[harm_std < 1e-8] = 1.0
    harmonic_features = (harmonic_features - harm_mean) / harm_std

    print("  Mel shape: %s, Harmonic shape: %s" %
          (padded[0].shape, harmonic_features.shape))
    return padded, harmonic_features, labels, metadata


# == Models ================================================================

class GuitarFusionDataset(Dataset):
    """PyTorch dataset with both mel-spectrogram and harmonic features."""

    def __init__(self, mel_features, harmonic_features, labels):
        self.mel = torch.tensor(
            np.array([f[np.newaxis, ...] for f in mel_features]),
            dtype=torch.float32,
        )
        self.harm = torch.tensor(harmonic_features, dtype=torch.float32)
        self.y = torch.tensor(labels, dtype=torch.long)

    def __len__(self):
        return len(self.y)

    def __getitem__(self, idx):
        return self.mel[idx], self.harm[idx], self.y[idx]


class HybridCNNFusion(nn.Module):
    """Two-branch architecture: CNN on spectrogram + FC on harmonics.

    Branch 1 (spectrogram): Conv2d layers -> AdaptiveAvgPool -> flatten
    Branch 2 (harmonics):   FC layer -> ReLU

    Fusion: concatenate both branch outputs, then classify.
    """

    def __init__(self, n_classes, input_shape, n_harmonic_features):
        super().__init__()

        self.spec_branch = nn.Sequential(
            nn.Conv2d(1, 16, 3, padding=1), nn.BatchNorm2d(16), nn.ReLU(), nn.MaxPool2d(2),
            nn.Conv2d(16, 32, 3, padding=1), nn.BatchNorm2d(32), nn.ReLU(), nn.MaxPool2d(2),
            nn.Conv2d(32, 64, 3, padding=1), nn.BatchNorm2d(64), nn.ReLU(), nn.AdaptiveAvgPool2d((4, 4)),
            nn.Flatten(),
        )
        spec_out = 64 * 4 * 4  # 1024

        self.harm_branch = nn.Sequential(
            nn.Linear(n_harmonic_features, 32),
            nn.ReLU(),
            nn.Dropout(0.2),
        )
        harm_out = 32

        self.classifier = nn.Sequential(
            nn.Linear(spec_out + harm_out, 256),
            nn.ReLU(),
            nn.Dropout(0.3),
            nn.Linear(256, n_classes),
        )

    def forward(self, mel, harm):
        spec_feat = self.spec_branch(mel)
        harm_feat = self.harm_branch(harm)
        fused = torch.cat([spec_feat, harm_feat], dim=1)
        return self.classifier(fused)


class PureCNNFusion(nn.Module):
    """Deeper CNN spectrogram branch + harmonic FC branch, fused at end."""

    def __init__(self, n_classes, input_shape, n_harmonic_features):
        super().__init__()

        self.spec_branch = nn.Sequential(
            nn.Conv2d(1, 32, 3, padding=1), nn.BatchNorm2d(32), nn.ReLU(), nn.MaxPool2d(2),
            nn.Conv2d(32, 64, 3, padding=1), nn.BatchNorm2d(64), nn.ReLU(), nn.MaxPool2d(2),
            nn.Conv2d(64, 128, 3, padding=1), nn.BatchNorm2d(128), nn.ReLU(), nn.MaxPool2d(2),
            nn.Conv2d(128, 256, 3, padding=1), nn.BatchNorm2d(256), nn.ReLU(), nn.AdaptiveAvgPool2d((1, 1)),
            nn.Flatten(),
        )
        spec_out = 256

        self.harm_branch = nn.Sequential(
            nn.Linear(n_harmonic_features, 32),
            nn.ReLU(),
            nn.Dropout(0.2),
        )
        harm_out = 32

        self.classifier = nn.Sequential(
            nn.Dropout(0.4),
            nn.Linear(spec_out + harm_out, n_classes),
        )

    def forward(self, mel, harm):
        spec_feat = self.spec_branch(mel)
        harm_feat = self.harm_branch(harm)
        fused = torch.cat([spec_feat, harm_feat], dim=1)
        return self.classifier(fused)


# == Training ==============================================================

def train_rf_fused(mel_features, harmonic_features, labels, train_idx, val_idx):
    """Train Random Forest on concatenated mel + harmonic features."""
    X_mel = np.array([f.flatten() for f in mel_features])
    X_fused = np.concatenate([X_mel, harmonic_features], axis=1)

    rf = RandomForestClassifier(n_estimators=200, n_jobs=-1, random_state=42)
    rf.fit(X_fused[train_idx], labels[train_idx])
    y_pred = rf.predict(X_fused[val_idx])
    acc = accuracy_score(labels[val_idx], y_pred)
    return y_pred, labels[val_idx], acc


def train_cnn_fused(model, dataset, train_idx, val_idx, epochs, lr):
    """Train a two-branch CNN model with mel + harmonic inputs."""
    device = torch.device("cpu")
    model = model.to(device)
    loader = DataLoader(dataset, batch_size=32, sampler=SubsetRandomSampler(train_idx))
    criterion = nn.CrossEntropyLoss()
    optimizer = optim.Adam(model.parameters(), lr=lr, weight_decay=1e-4)
    scheduler = optim.lr_scheduler.CosineAnnealingLR(optimizer, T_max=epochs)

    best_acc, best_state = 0.0, None
    t0 = time.time()

    for epoch in range(epochs):
        model.train()
        for mel_batch, harm_batch, y_batch in loader:
            optimizer.zero_grad()
            outputs = model(mel_batch, harm_batch)
            loss = criterion(outputs, y_batch)
            loss.backward()
            optimizer.step()
        scheduler.step()

        model.eval()
        correct, total = 0, 0
        with torch.no_grad():
            for i in val_idx:
                mel_i, harm_i, y_i = dataset[i]
                pred = model(mel_i.unsqueeze(0), harm_i.unsqueeze(0)).argmax(1).item()
                correct += (pred == y_i.item())
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
    true_labels = np.zeros(len(val_idx), dtype=int)
    with torch.no_grad():
        for i, idx in enumerate(val_idx):
            mel_i, harm_i, y_i = dataset[idx]
            preds[i] = model(mel_i.unsqueeze(0), harm_i.unsqueeze(0)).argmax(1).item()
            true_labels[i] = y_i.item()
    return preds, true_labels, best_acc, time.time() - t0


# == Report (TODO: fill in after training runs) ============================

def plot_comparison(r4_results, previous, output_dir):
    """Bar chart comparing Round 4 vs Round 1 accuracy, plus per-string breakdown."""
    model_names = [r["name"] for r in r4_results]

    # -- Overall accuracy comparison --
    fig, axes = plt.subplots(1, 2, figsize=(16, 6))

    r1 = previous.get("round_01", [])
    r1_acc = {r["name"]: r["accuracy"] for r in r1} if r1 else {}

    x = np.arange(len(model_names))
    width = 0.35

    ax = axes[0]
    r4_accs = [r["accuracy"] * 100 for r in r4_results]
    r1_accs = [r1_acc.get(n, 0) * 100 for n in model_names]

    bars1 = ax.bar(x - width / 2, r1_accs, width, label="Round 1 (baseline)", color="#5c6bc0")
    bars2 = ax.bar(x + width / 2, r4_accs, width, label="Round 4 (Goertzel fusion)", color="#26a69a")

    for bar, val in zip(bars1, r1_accs):
        ax.text(bar.get_x() + bar.get_width() / 2, bar.get_height() + 0.3,
                "%.1f%%" % val, ha="center", va="bottom", fontsize=9, color="white")
    for bar, val in zip(bars2, r4_accs):
        ax.text(bar.get_x() + bar.get_width() / 2, bar.get_height() + 0.3,
                "%.1f%%" % val, ha="center", va="bottom", fontsize=9, color="white")

    ax.set_ylabel("Accuracy (%)")
    ax.set_title("Overall Accuracy: Round 1 vs Round 4")
    ax.set_xticks(x)
    ax.set_xticklabels(model_names)
    ax.legend()
    ax.set_ylim(80, 100)
    ax.set_facecolor("#1a1a2e")
    ax.grid(True, alpha=0.2, axis="y")

    # -- Per-string accuracy (Round 4 only) --
    ax2 = axes[1]
    x2 = np.arange(6)
    bar_w = 0.25

    for i, r in enumerate(r4_results):
        ps = [v * 100 for v in r["per_string"]]
        ax2.bar(x2 + i * bar_w, ps, bar_w, label=r["name"], alpha=0.85)

    ax2.set_ylabel("Accuracy (%)")
    ax2.set_title("Per-String Accuracy (Round 4)")
    ax2.set_xticks(x2 + bar_w)
    ax2.set_xticklabels(STRING_NAMES, rotation=30, ha="right", fontsize=9)
    ax2.legend(fontsize=8)
    ax2.set_ylim(70, 100)
    ax2.set_facecolor("#1a1a2e")
    ax2.grid(True, alpha=0.2, axis="y")

    fig.tight_layout()
    path = os.path.join(output_dir, "round_04_comparison.png")
    fig.savefig(path, dpi=150, facecolor="#0a0a1a")
    plt.close(fig)
    print("  Saved: %s" % path)

    # -- Per-string delta chart (R4 - R1) --
    if r1:
        fig2, ax3 = plt.subplots(figsize=(12, 6))
        r1_ps = {r["name"]: r["per_string"] for r in r1}
        x3 = np.arange(6)
        bar_w2 = 0.25

        for i, r in enumerate(r4_results):
            if r["name"] in r1_ps:
                deltas = [(r["per_string"][s] - r1_ps[r["name"]][s]) * 100 for s in range(6)]
                bars = ax3.bar(x3 + i * bar_w2, deltas, bar_w2, label=r["name"], alpha=0.85)
                for bar, d in zip(bars, deltas):
                    color = "#4caf50" if d >= 0 else "#f44336"
                    ax3.text(bar.get_x() + bar.get_width() / 2,
                             bar.get_height() + (0.2 if d >= 0 else -0.8),
                             "%+.1f%%" % d, ha="center", va="bottom", fontsize=8, color=color)

        ax3.axhline(y=0, color="white", linewidth=0.5, alpha=0.5)
        ax3.set_ylabel("Accuracy Delta (%)")
        ax3.set_title("Per-String Accuracy Change: Round 4 - Round 1")
        ax3.set_xticks(x3 + bar_w2)
        ax3.set_xticklabels(STRING_NAMES, rotation=30, ha="right", fontsize=9)
        ax3.legend(fontsize=8)
        ax3.set_facecolor("#1a1a2e")
        ax3.grid(True, alpha=0.2, axis="y")

        fig2.tight_layout()
        path2 = os.path.join(output_dir, "round_04_delta.png")
        fig2.savefig(path2, dpi=150, facecolor="#0a0a1a")
        plt.close(fig2)
        print("  Saved: %s" % path2)


def generate_report(r4_results, previous, output_dir):
    """Generate ROUND_04_GOERTZEL.md report."""
    r1 = previous.get("round_01", [])
    r1_acc = {r["name"]: r for r in r1} if r1 else {}

    lines = []
    lines.append("# Round 4: Goertzel Harmonic Feature Fusion")
    lines.append("")
    lines.append("## Hypothesis")
    lines.append("")
    lines.append("Mel-spectrograms capture pitch (fret) well but struggle with string identity")
    lines.append("for shared pitches. Goertzel harmonic ratios encode the string's physical")
    lines.append("timbre signature (wound vs plain, thick vs thin). Fusing both feature types")
    lines.append("should improve string classification accuracy, especially for E2 (low).")
    lines.append("")
    lines.append("## Features")
    lines.append("")
    lines.append("- **Mel-spectrogram**: 64 mel bins, 1024 FFT, 256 hop -> flattened to ~6016 features (RF) or 2D input (CNN)")
    lines.append("- **Goertzel harmonics**: 9 harmonic ratios (H2/H1 ... H10/H1) + spectral centroid + inharmonicity = 11 features")
    lines.append("- **Fusion**: concatenation (RF) or two-branch architecture (CNN)")
    lines.append("")
    lines.append("## Results")
    lines.append("")
    lines.append("| Model | Round 1 | Round 4 | Delta |")
    lines.append("|-------|---------|---------|-------|")

    for r in r4_results:
        r4_pct = r["accuracy"] * 100
        r1_entry = r1_acc.get(r["name"])
        if r1_entry:
            r1_pct = r1_entry["accuracy"] * 100
            delta = r4_pct - r1_pct
            lines.append("| %s | %.1f%% | %.1f%% | %+.1f%% |" %
                         (r["name"], r1_pct, r4_pct, delta))
        else:
            lines.append("| %s | -- | %.1f%% | -- |" % (r["name"], r4_pct))

    lines.append("")
    lines.append("## Per-String Accuracy")
    lines.append("")
    lines.append("| String | " + " | ".join(r["name"] + " R4" for r in r4_results) + " | " +
                 " | ".join(r["name"] + " R1" for r in r4_results if r["name"] in r1_acc) + " |")
    lines.append("|--------|" + "--------|" * (len(r4_results) + len([r for r in r4_results if r["name"] in r1_acc])))

    for si in range(6):
        row = "| %s" % STRING_NAMES[si]
        for r in r4_results:
            row += " | %.1f%%" % (r["per_string"][si] * 100)
        for r in r4_results:
            if r["name"] in r1_acc:
                row += " | %.1f%%" % (r1_acc[r["name"]]["per_string"][si] * 100)
        row += " |"
        lines.append(row)

    lines.append("")
    lines.append("## Key Findings")
    lines.append("")

    # Compute E2 deltas
    for r in r4_results:
        r1_entry = r1_acc.get(r["name"])
        if r1_entry:
            e2_delta = (r["per_string"][0] - r1_entry["per_string"][0]) * 100
            lines.append("- **%s E2 (low)**: %+.1f%% (R1: %.1f%% -> R4: %.1f%%)" %
                         (r["name"], e2_delta,
                          r1_entry["per_string"][0] * 100, r["per_string"][0] * 100))

    lines.append("")
    lines.append("## Visualizations")
    lines.append("")
    lines.append("- `round_04_comparison.png` -- overall + per-string accuracy bar charts")
    lines.append("- `round_04_delta.png` -- per-string accuracy deltas (R4 - R1)")
    lines.append("")

    path = os.path.join(output_dir, "ROUND_04_GOERTZEL.md")
    with open(path, "w") as f:
        f.write("\n".join(lines) + "\n")
    print("  Saved: %s" % path)


# == Main ==================================================================

def main():
    print("=" * 60)
    print("  Round 4: Goertzel Harmonic Feature Fusion")
    print("=" * 60)
    os.makedirs(OUTPUT_DIR, exist_ok=True)

    if not os.path.exists(GOERTZEL_JSON):
        print("ERROR: %s not found." % GOERTZEL_JSON)
        print("Run: python ml/training/goertzel.py")
        sys.exit(1)

    # Load previous results for comparison
    previous = {}
    for name, dirpath in [("round_01", ROUND_01_DIR), ("round_02", ROUND_02_DIR)]:
        rpath = os.path.join(dirpath, "results.json")
        if os.path.exists(rpath):
            with open(rpath) as f:
                previous[name] = json.load(f)
            print("  %s loaded" % name)

    # Load data
    goertzel_data, goertzel_by_label = load_goertzel_features(GOERTZEL_JSON)
    ds = GuitarDataset.load(DATASET_PATH)
    mel_features, harmonic_features, labels, metadata = prepare_dataset(
        ds, goertzel_by_label
    )

    n_classes = len(np.unique(labels))
    n_harm_features = harmonic_features.shape[1]
    input_shape = mel_features[0].shape

    print("\n  Classes: %d, Samples: %d, Harmonic features: %d" %
          (n_classes, len(labels), n_harm_features))

    fusion_ds = GuitarFusionDataset(mel_features, harmonic_features, labels)

    skf = StratifiedKFold(n_splits=5, shuffle=True, random_state=42)
    folds = list(skf.split(np.zeros(len(labels)), labels))

    r4_results = []

    # -- Random Forest (fused) ------------------------------------------------
    print("\n" + "-" * 60)
    print("  Random Forest (mel + Goertzel harmonics)")
    print("-" * 60)

    rf_accs, rf_ps = [], np.zeros((5, 6))
    for fold, (ti, vi) in enumerate(folds):
        yp, yv, acc = train_rf_fused(mel_features, harmonic_features, labels, ti, vi)
        rf_accs.append(acc)
        print("  Fold %d: %.4f" % (fold + 1, acc))
        for si in range(6):
            m = [metadata[i][0] == si for i in vi]
            if any(m):
                rf_ps[fold, si] = accuracy_score(yv[m], yp[m])

    r4_results.append({
        "name": "Random Forest",
        "accuracy": float(np.mean(rf_accs)),
        "per_string": rf_ps.mean(0).tolist(),
        "notes": "mel + Goertzel harmonic ratios, centroid, inharmonicity",
    })

    # -- Hybrid CNN Fusion ----------------------------------------------------
    print("\n" + "-" * 60)
    print("  Hybrid CNN Fusion (two-branch: spec + harmonics)")
    print("-" * 60)

    hc_accs, hc_ps = [], np.zeros((5, 6))
    for fold, (ti, vi) in enumerate(folds):
        print("\n  Fold %d:" % (fold + 1))
        model = HybridCNNFusion(n_classes, input_shape, n_harm_features)
        yp, yv, acc, _ = train_cnn_fused(model, fusion_ds, ti, vi, 50, 1e-3)
        hc_accs.append(acc)
        for si in range(6):
            mask = [metadata[i][0] == si for i in vi]
            if any(mask):
                hc_ps[fold, si] = accuracy_score(yv[mask], yp[mask])

    r4_results.append({
        "name": "Hybrid CNN",
        "accuracy": float(np.mean(hc_accs)),
        "per_string": hc_ps.mean(0).tolist(),
        "notes": "two-branch fusion, 50 epochs",
        "training_curves": True,
    })

    # -- Pure CNN Fusion ------------------------------------------------------
    print("\n" + "-" * 60)
    print("  Pure CNN Fusion (two-branch: spec + harmonics)")
    print("-" * 60)

    pc_accs, pc_ps = [], np.zeros((5, 6))
    for fold, (ti, vi) in enumerate(folds):
        print("\n  Fold %d:" % (fold + 1))
        model = PureCNNFusion(n_classes, input_shape, n_harm_features)
        yp, yv, acc, _ = train_cnn_fused(model, fusion_ds, ti, vi, 60, 8e-4)
        pc_accs.append(acc)
        for si in range(6):
            mask = [metadata[i][0] == si for i in vi]
            if any(mask):
                pc_ps[fold, si] = accuracy_score(yv[mask], yp[mask])

    r4_results.append({
        "name": "Pure CNN",
        "accuracy": float(np.mean(pc_accs)),
        "per_string": pc_ps.mean(0).tolist(),
        "notes": "two-branch fusion, 60 epochs",
        "training_curves": True,
    })

    # -- Results --------------------------------------------------------------
    print("\n" + "-" * 60)
    print("  RESULTS")
    print("-" * 60)

    with open(os.path.join(OUTPUT_DIR, "results.json"), "w") as f:
        json.dump(r4_results, f, indent=2)
    print("  Saved: %s" % os.path.join(OUTPUT_DIR, "results.json"))

    print("\n" + "=" * 60)
    print("  ROUND 4 vs PREVIOUS ROUNDS")
    print("=" * 60)

    for r4 in r4_results:
        line = "  %-20s  R4=%.1f%%" % (r4["name"], r4["accuracy"] * 100)
        for rnd, prev in previous.items():
            match = next((p for p in prev if p["name"] == r4["name"]), None)
            if match:
                delta = r4["accuracy"] - match["accuracy"]
                line += "  %s=%.1f%% (%+.1f%%)" % (rnd, match["accuracy"] * 100, delta * 100)
        print(line)

    # Per-string comparison with Round 1
    if "round_01" in previous:
        print("\n  Per-String Accuracy (Round 4 vs Round 1):")
        r1_data = {r["name"]: r for r in previous["round_01"]}
        for r4 in r4_results:
            r1_entry = r1_data.get(r4["name"])
            if r1_entry:
                print("\n    %s:" % r4["name"])
                for si in range(6):
                    r4_ps = r4["per_string"][si] * 100
                    r1_ps = r1_entry["per_string"][si] * 100
                    delta = r4_ps - r1_ps
                    marker = " ***" if abs(delta) > 2.0 else ""
                    print("      %-12s  R1=%.1f%%  R4=%.1f%%  (%+.1f%%)%s" %
                          (STRING_NAMES[si], r1_ps, r4_ps, delta, marker))

    # Generate visualizations and report
    print("\nGenerating visualizations...")
    plot_comparison(r4_results, previous, OUTPUT_DIR)

    print("\nGenerating report...")
    generate_report(r4_results, previous, OUTPUT_DIR)

    print("\n" + "=" * 60)
    print("  ROUND 4 COMPLETE")
    print("=" * 60)


if __name__ == "__main__":
    main()
