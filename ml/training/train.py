"""
Contrapunk Guitar Classifier — Round 1 Baseline Training

Trains 3 models on RAW data (no preprocessing, no augmentation):
  1. Random Forest on mel-spectrogram features (flattened)
  2. Hybrid CNN (small ConvNet -> FC layers)
  3. Pure CNN (deeper ConvNet, no dense bottleneck)

Each model classifies guitar samples into 138 string+fret classes.
Results are saved to ml/training/round_01/ with confusion matrices
and per-string/fret accuracy heatmaps.

Usage:
    cd <project_root>
    source ml/venv/bin/activate
    python ml/training/train.py
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
from sklearn.metrics import (
    accuracy_score, classification_report, confusion_matrix
)
from sklearn.model_selection import StratifiedKFold

import torch
import torch.nn as nn
import torch.optim as optim
from torch.utils.data import Dataset, DataLoader, SubsetRandomSampler

import librosa

warnings.filterwarnings("ignore", category=FutureWarning)

# -- Paths -----------------------------------------------------------------

PROJECT_ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), "../.."))
DATASET_PATH = os.path.join(PROJECT_ROOT, "guitar_training_data.msgpack")
OUTPUT_DIR   = os.path.join(os.path.dirname(__file__), "round_01")

sys.path.insert(0, PROJECT_ROOT)
from ml.loader import GuitarDataset


# -- Feature Extraction ----------------------------------------------------

N_MELS = 64
N_FFT = 1024
HOP_LENGTH = 256


def extract_mel_spectrogram(audio, sr):
    """Extract log-mel spectrogram from raw audio."""
    mel = librosa.feature.melspectrogram(
        y=audio, sr=sr,
        n_mels=N_MELS, n_fft=N_FFT, hop_length=HOP_LENGTH,
    )
    log_mel = librosa.power_to_db(mel, ref=np.max)
    return log_mel


def prepare_dataset(ds):
    """Extract features and labels from GuitarDataset."""
    print("Extracting mel-spectrograms...")
    raw_features = []
    labels = []
    metadata = []  # (string_idx, fret) for analysis

    for i, sample in enumerate(ds.samples):
        audio = sample.audio_np
        sr = sample.sample_rate

        mel = extract_mel_spectrogram(audio, sr)
        raw_features.append(mel)
        labels.append(sample.class_id - 1)  # zero-index
        metadata.append((sample.string_idx, sample.fret))

        if (i + 1) % 200 == 0:
            print(f"  {i + 1}/{len(ds.samples)} samples processed")

    # Pad/truncate to uniform time dimension (max across dataset)
    max_time = max(f.shape[1] for f in raw_features)
    features = []
    for f in raw_features:
        if f.shape[1] < max_time:
            pad_width = max_time - f.shape[1]
            # Pad with min value (silence in dB scale)
            f = np.pad(f, ((0, 0), (0, pad_width)), mode="constant",
                       constant_values=f.min())
        elif f.shape[1] > max_time:
            f = f[:, :max_time]
        features.append(f)

    print(f"  All {len(features)} samples processed")
    print(f"  Spectrogram shape: {features[0].shape} (uniform)")

    return features, np.array(labels), metadata


# -- Model 1: Random Forest -----------------------------------------------

def train_random_forest(features, labels, train_idx, val_idx):
    """Train RF on flattened mel-spectrograms."""
    X_flat = np.array([f.flatten() for f in features])

    X_train = X_flat[train_idx]
    y_train = labels[train_idx]
    X_val = X_flat[val_idx]
    y_val = labels[val_idx]

    print(f"\n  RF: Training on {len(X_train)} samples, "
          f"validating on {len(X_val)}, features={X_flat.shape[1]}")

    t0 = time.time()
    rf = RandomForestClassifier(
        n_estimators=200,
        max_depth=None,
        min_samples_leaf=1,
        n_jobs=-1,
        random_state=42,
    )
    rf.fit(X_train, y_train)
    train_time = time.time() - t0

    y_pred = rf.predict(X_val)
    acc = accuracy_score(y_val, y_pred)

    print(f"  RF: val accuracy = {acc:.4f} ({train_time:.1f}s)")
    return rf, y_pred, y_val, acc, train_time


# -- Model 2 & 3: CNN architectures ---------------------------------------

class GuitarMelDataset(Dataset):
    """PyTorch dataset wrapping mel-spectrograms."""

    def __init__(self, features, labels):
        # Add channel dimension: (N_MELS, TIME) -> (1, N_MELS, TIME)
        self.X = torch.tensor(
            np.array([f[np.newaxis, ...] for f in features]),
            dtype=torch.float32,
        )
        self.y = torch.tensor(labels, dtype=torch.long)

    def __len__(self):
        return len(self.y)

    def __getitem__(self, idx):
        return self.X[idx], self.y[idx]


class HybridCNN(nn.Module):
    """Small ConvNet -> flatten -> FC layers."""

    def __init__(self, n_classes, input_shape):
        super().__init__()
        h, w = input_shape  # (N_MELS, TIME_FRAMES)

        self.conv = nn.Sequential(
            nn.Conv2d(1, 16, 3, padding=1),
            nn.BatchNorm2d(16),
            nn.ReLU(),
            nn.MaxPool2d(2),

            nn.Conv2d(16, 32, 3, padding=1),
            nn.BatchNorm2d(32),
            nn.ReLU(),
            nn.MaxPool2d(2),

            nn.Conv2d(32, 64, 3, padding=1),
            nn.BatchNorm2d(64),
            nn.ReLU(),
            nn.AdaptiveAvgPool2d((4, 4)),
        )

        self.fc = nn.Sequential(
            nn.Flatten(),
            nn.Linear(64 * 4 * 4, 256),
            nn.ReLU(),
            nn.Dropout(0.3),
            nn.Linear(256, n_classes),
        )

    def forward(self, x):
        x = self.conv(x)
        x = self.fc(x)
        return x


class PureCNN(nn.Module):
    """Deeper ConvNet with global average pooling, no dense bottleneck."""

    def __init__(self, n_classes, input_shape):
        super().__init__()

        self.features = nn.Sequential(
            nn.Conv2d(1, 32, 3, padding=1),
            nn.BatchNorm2d(32),
            nn.ReLU(),
            nn.MaxPool2d(2),

            nn.Conv2d(32, 64, 3, padding=1),
            nn.BatchNorm2d(64),
            nn.ReLU(),
            nn.MaxPool2d(2),

            nn.Conv2d(64, 128, 3, padding=1),
            nn.BatchNorm2d(128),
            nn.ReLU(),
            nn.MaxPool2d(2),

            nn.Conv2d(128, 256, 3, padding=1),
            nn.BatchNorm2d(256),
            nn.ReLU(),
            nn.AdaptiveAvgPool2d((1, 1)),
        )

        self.classifier = nn.Sequential(
            nn.Flatten(),
            nn.Dropout(0.4),
            nn.Linear(256, n_classes),
        )

    def forward(self, x):
        x = self.features(x)
        x = self.classifier(x)
        return x


def train_cnn(model, name, dataset, train_idx, val_idx, epochs=50, lr=1e-3):
    """Train a PyTorch CNN model with given train/val split."""
    # Use CPU — MPS adaptive_avg_pool2d requires divisible sizes (pytorch#96056)
    device = torch.device("cpu")
    print(f"\n  {name}: device={device}, epochs={epochs}, lr={lr}")

    model = model.to(device)

    train_sampler = SubsetRandomSampler(train_idx)
    val_sampler = SubsetRandomSampler(val_idx)

    train_loader = DataLoader(dataset, batch_size=32, sampler=train_sampler)
    val_loader = DataLoader(dataset, batch_size=64, sampler=val_sampler)

    criterion = nn.CrossEntropyLoss()
    optimizer = optim.Adam(model.parameters(), lr=lr, weight_decay=1e-4)
    scheduler = optim.lr_scheduler.CosineAnnealingLR(optimizer, T_max=epochs)

    best_acc = 0.0
    best_state = None
    history = {"train_loss": [], "val_acc": []}

    t0 = time.time()
    for epoch in range(epochs):
        model.train()
        running_loss = 0.0
        for X_batch, y_batch in train_loader:
            X_batch, y_batch = X_batch.to(device), y_batch.to(device)
            optimizer.zero_grad()
            outputs = model(X_batch)
            loss = criterion(outputs, y_batch)
            loss.backward()
            optimizer.step()
            running_loss += loss.item() * X_batch.size(0)

        scheduler.step()
        epoch_loss = running_loss / len(train_idx)
        history["train_loss"].append(epoch_loss)

        # Validation
        model.eval()
        correct = 0
        total = 0
        with torch.no_grad():
            for X_batch, y_batch in val_loader:
                X_batch, y_batch = X_batch.to(device), y_batch.to(device)
                outputs = model(X_batch)
                _, predicted = outputs.max(1)
                total += y_batch.size(0)
                correct += predicted.eq(y_batch).sum().item()

        val_acc = correct / total
        history["val_acc"].append(val_acc)

        if val_acc > best_acc:
            best_acc = val_acc
            best_state = {k: v.cpu().clone() for k, v in model.state_dict().items()}

        if (epoch + 1) % 10 == 0 or epoch == 0:
            print(f"    Epoch {epoch+1:3d}: loss={epoch_loss:.4f} val_acc={val_acc:.4f}")

    train_time = time.time() - t0
    print(f"  {name}: best val_acc={best_acc:.4f} ({train_time:.1f}s)")

    # Restore best weights for final predictions
    model.load_state_dict(best_state)
    model.eval()

    # Collect val predictions in val_idx order (not shuffled sampler order)
    all_preds = np.zeros(len(val_idx), dtype=int)
    all_labels = np.zeros(len(val_idx), dtype=int)
    with torch.no_grad():
        for i, idx in enumerate(val_idx):
            X_single = dataset[idx][0].unsqueeze(0).to(device)
            output = model(X_single)
            _, predicted = output.max(1)
            all_preds[i] = predicted.cpu().item()
            all_labels[i] = dataset[idx][1].item()

    return model, all_preds, all_labels, best_acc, train_time, history


# -- Visualization ---------------------------------------------------------

def plot_confusion_matrix(y_true, y_pred, n_classes, title, path):
    """Plot and save confusion matrix."""
    cm = confusion_matrix(y_true, y_pred, labels=range(n_classes))
    fig, ax = plt.subplots(figsize=(14, 12))
    sns.heatmap(cm, ax=ax, cmap="viridis", xticklabels=False, yticklabels=False)
    ax.set_title(title, fontsize=14)
    ax.set_xlabel("Predicted")
    ax.set_ylabel("True")
    fig.tight_layout()
    fig.savefig(path, dpi=150)
    plt.close(fig)
    print(f"  Saved: {path}")


def plot_string_fret_heatmap(y_true, y_pred, metadata, val_idx, title, path):
    """Plot per-string x per-fret accuracy heatmap."""
    correct = np.zeros((6, 23))
    total = np.zeros((6, 23))

    for i in val_idx:
        si, fret = metadata[i]
        total[si][fret] += 1
        if y_true[i] == y_pred[i]:
            correct[si][fret] += 1

    with np.errstate(divide="ignore", invalid="ignore"):
        acc_grid = np.where(total > 0, correct / total, np.nan)

    fig, ax = plt.subplots(figsize=(16, 5))
    sns.heatmap(
        acc_grid, ax=ax, cmap="RdYlGn", vmin=0, vmax=1,
        annot=True, fmt=".0%",
        xticklabels=range(23),
        yticklabels=["E2", "A2", "D3", "G3", "B3", "E4"],
    )
    ax.set_title(title, fontsize=14)
    ax.set_xlabel("Fret")
    ax.set_ylabel("String")
    fig.tight_layout()
    fig.savefig(path, dpi=150)
    plt.close(fig)
    print(f"  Saved: {path}")


def plot_per_string_accuracy(y_true, y_pred, metadata, val_idx, title, path):
    """Bar chart of accuracy per string."""
    string_names = ["E2 (low)", "A2", "D3", "G3", "B3", "E4 (high)"]
    correct_per_string = [0] * 6
    total_per_string = [0] * 6

    for i in val_idx:
        si, _ = metadata[i]
        total_per_string[si] += 1
        if y_true[i] == y_pred[i]:
            correct_per_string[si] += 1

    accs = [c / t if t > 0 else 0 for c, t in zip(correct_per_string, total_per_string)]

    fig, ax = plt.subplots(figsize=(8, 5))
    bars = ax.bar(string_names, accs, color=sns.color_palette("viridis", 6))
    ax.set_ylim(0, 1.05)
    ax.set_ylabel("Accuracy")
    ax.set_title(title)
    for bar, acc in zip(bars, accs):
        ax.text(bar.get_x() + bar.get_width() / 2, bar.get_height() + 0.01,
                f"{acc:.1%}", ha="center", fontsize=10)
    fig.tight_layout()
    fig.savefig(path, dpi=150)
    plt.close(fig)
    print(f"  Saved: {path}")


def plot_training_curves(history, name, path):
    """Plot loss and accuracy curves for CNN training."""
    fig, (ax1, ax2) = plt.subplots(1, 2, figsize=(12, 5))
    ax1.plot(history["train_loss"])
    ax1.set_title(f"{name} -- Training Loss")
    ax1.set_xlabel("Epoch")
    ax1.set_ylabel("Loss")

    ax2.plot(history["val_acc"])
    ax2.set_title(f"{name} -- Validation Accuracy")
    ax2.set_xlabel("Epoch")
    ax2.set_ylabel("Accuracy")
    ax2.set_ylim(0, 1)

    fig.tight_layout()
    fig.savefig(path, dpi=150)
    plt.close(fig)
    print(f"  Saved: {path}")


# -- Report Generation -----------------------------------------------------

def generate_report(results, metadata_info):
    """Generate ROUND_01_BASELINE.md report."""
    report_path = os.path.join(OUTPUT_DIR, "ROUND_01_BASELINE.md")

    lines = [
        "# Round 1: Baseline Training Results",
        "",
        "## Dataset",
        f"- **Samples:** {metadata_info['total']}",
        f"- **Classes:** {metadata_info['n_classes']} (6 strings x 23 frets)",
        f"- **Per class:** {metadata_info['per_class']} samples",
        f"- **Duration:** 0.5s per sample @ {metadata_info['sample_rate']} Hz",
        f"- **Features:** Log-mel spectrogram "
        f"(n_mels={N_MELS}, n_fft={N_FFT}, hop={HOP_LENGTH})",
        f"- **Preprocessing:** None (raw audio, no onset alignment, no augmentation)",
        f"- **Quality issues:** 32 clipped, 7 near-silent (left in for baseline)",
        "",
        "## Evaluation",
        "- **Method:** 5-fold stratified cross-validation",
        "- **Metric:** Classification accuracy (macro across folds)",
        "",
        "## Results",
        "",
        "| Model | Accuracy | Train Time | Notes |",
        "|-------|----------|------------|-------|",
    ]

    for r in results:
        lines.append(
            f"| {r['name']} | **{r['accuracy']:.1%}** | {r['train_time']:.1f}s | {r['notes']} |"
        )

    lines.extend([
        "",
        "## Per-String Accuracy",
        "",
        "| String | " + " | ".join(r["name"] for r in results) + " |",
        "|--------|" + "|".join("-------" for _ in results) + "|",
    ])

    for si, sname in enumerate(["E2 (low)", "A2", "D3", "G3", "B3", "E4 (high)"]):
        row = f"| {sname} |"
        for r in results:
            acc = r["per_string"][si]
            row += f" {acc:.1%} |"
        lines.append(row)

    lines.extend([
        "",
        "## Visualizations",
        "",
        "### Confusion Matrices",
    ])
    for r in results:
        safe_name = r["name"].lower().replace(" ", "_")
        lines.append(f"![{r['name']} Confusion Matrix](confusion_{safe_name}.png)")
        lines.append("")

    lines.extend([
        "### Per-String Accuracy",
    ])
    for r in results:
        safe_name = r["name"].lower().replace(" ", "_")
        lines.append(f"![{r['name']} Per-String](per_string_{safe_name}.png)")
        lines.append("")

    lines.extend([
        "### Per-Fret Accuracy Heatmaps",
    ])
    for r in results:
        safe_name = r["name"].lower().replace(" ", "_")
        lines.append(f"![{r['name']} Fret Heatmap](fret_heatmap_{safe_name}.png)")
        lines.append("")

    for r in results:
        if "training_curves" in r:
            safe_name = r["name"].lower().replace(" ", "_")
            lines.extend([
                f"### {r['name']} Training Curves",
                f"![{r['name']} Curves](training_curves_{safe_name}.png)",
                "",
            ])

    lines.extend([
        "## Analysis",
        "",
        "### What this baseline tells us",
        "- How well raw mel-spectrograms distinguish 138 guitar positions",
        "- Which strings/frets are hardest to classify (inform next round)",
        "- Whether the dataset size (10 per class) is sufficient for each model type",
        "",
        "### Planned improvements for Round 2+",
        "- Onset alignment (trim silence before note attack)",
        "- Noise class samples (ambient, muting, scrapes)",
        "- Goertzel harmonic features (fundamental + harmonics strength)",
        "- Data augmentation (pitch shift, time stretch, noise injection)",
        "- More samples per position",
        "",
        f"*Generated: {time.strftime('%Y-%m-%d %H:%M')}*",
    ])

    with open(report_path, "w") as f:
        f.write("\n".join(lines))

    print(f"\n  Report saved: {report_path}")


# -- Main ------------------------------------------------------------------

def main():
    print("=" * 60)
    print("  Round 1: Baseline Training (Raw Data)")
    print("=" * 60)

    os.makedirs(OUTPUT_DIR, exist_ok=True)

    # Load dataset
    ds = GuitarDataset.load(DATASET_PATH)
    features, labels, metadata = prepare_dataset(ds)

    n_classes = len(np.unique(labels))
    print(f"\n  Classes: {n_classes}, Samples: {len(labels)}")

    # Create PyTorch dataset for CNN models
    torch_ds = GuitarMelDataset(features, labels)
    input_shape = features[0].shape  # (N_MELS, TIME_FRAMES)

    # 5-fold stratified CV
    skf = StratifiedKFold(n_splits=5, shuffle=True, random_state=42)
    fold_indices = list(skf.split(np.zeros(len(labels)), labels))

    results = []

    # -- Model 1: Random Forest --------------------------------------------
    print("\n" + "-" * 60)
    print("  MODEL 1: Random Forest")
    print("-" * 60)

    rf_accs = []
    rf_per_string = np.zeros((5, 6))
    all_rf_preds = np.zeros(len(labels), dtype=int)
    all_rf_used = np.zeros(len(labels), dtype=bool)

    for fold, (train_idx, val_idx) in enumerate(fold_indices):
        print(f"\n  Fold {fold + 1}/5:")
        _, y_pred, y_val, acc, _ = train_random_forest(
            features, labels, train_idx, val_idx
        )
        rf_accs.append(acc)
        all_rf_preds[val_idx] = y_pred
        all_rf_used[val_idx] = True

        for si in range(6):
            mask = [metadata[i][0] == si for i in val_idx]
            if any(mask):
                s_true = y_val[mask]
                s_pred = y_pred[mask]
                rf_per_string[fold, si] = accuracy_score(s_true, s_pred)

    rf_mean_acc = np.mean(rf_accs)
    rf_mean_per_string = rf_per_string.mean(axis=0)

    # Last fold for visualizations
    last_train, last_val = fold_indices[-1]
    rf_model, rf_y_pred, rf_y_val, _, rf_time = train_random_forest(
        features, labels, last_train, last_val
    )

    safe = "random_forest"
    plot_confusion_matrix(
        rf_y_val, rf_y_pred, n_classes,
        f"Random Forest -- Val Acc: {rf_mean_acc:.1%}",
        os.path.join(OUTPUT_DIR, f"confusion_{safe}.png"),
    )
    plot_string_fret_heatmap(
        labels, all_rf_preds, metadata,
        np.where(all_rf_used)[0].tolist(),
        f"Random Forest -- Per-Fret Accuracy",
        os.path.join(OUTPUT_DIR, f"fret_heatmap_{safe}.png"),
    )
    plot_per_string_accuracy(
        labels, all_rf_preds, metadata,
        np.where(all_rf_used)[0].tolist(),
        f"Random Forest -- Per-String Accuracy",
        os.path.join(OUTPUT_DIR, f"per_string_{safe}.png"),
    )

    results.append({
        "name": "Random Forest",
        "accuracy": rf_mean_acc,
        "train_time": rf_time,
        "per_string": rf_mean_per_string.tolist(),
        "notes": "200 trees, flattened mel-spectrogram features",
    })

    # -- Model 2: Hybrid CNN -----------------------------------------------
    print("\n" + "-" * 60)
    print("  MODEL 2: Hybrid CNN")
    print("-" * 60)

    cnn_accs = []
    cnn_per_string = np.zeros((5, 6))
    all_cnn_preds = np.zeros(len(labels), dtype=int)
    all_cnn_used = np.zeros(len(labels), dtype=bool)
    cnn_total_time = 0.0
    last_history = None

    for fold, (train_idx, val_idx) in enumerate(fold_indices):
        print(f"\n  Fold {fold + 1}/5:")
        model = HybridCNN(n_classes, input_shape)
        _, y_pred, y_val, acc, fold_time, history = train_cnn(
            model, "Hybrid CNN", torch_ds, train_idx, val_idx,
            epochs=50, lr=1e-3,
        )
        cnn_accs.append(acc)
        cnn_total_time += fold_time
        all_cnn_preds[val_idx] = y_pred
        all_cnn_used[val_idx] = True
        last_history = history

        for si in range(6):
            mask = [metadata[i][0] == si for i in val_idx]
            if any(mask):
                s_true = y_val[mask]
                s_pred = y_pred[mask]
                cnn_per_string[fold, si] = accuracy_score(s_true, s_pred)

    cnn_mean_acc = np.mean(cnn_accs)
    cnn_mean_per_string = cnn_per_string.mean(axis=0)

    safe = "hybrid_cnn"
    plot_confusion_matrix(
        labels[all_cnn_used], all_cnn_preds[all_cnn_used], n_classes,
        f"Hybrid CNN -- Val Acc: {cnn_mean_acc:.1%}",
        os.path.join(OUTPUT_DIR, f"confusion_{safe}.png"),
    )
    plot_string_fret_heatmap(
        labels, all_cnn_preds, metadata,
        np.where(all_cnn_used)[0].tolist(),
        f"Hybrid CNN -- Per-Fret Accuracy",
        os.path.join(OUTPUT_DIR, f"fret_heatmap_{safe}.png"),
    )
    plot_per_string_accuracy(
        labels, all_cnn_preds, metadata,
        np.where(all_cnn_used)[0].tolist(),
        f"Hybrid CNN -- Per-String Accuracy",
        os.path.join(OUTPUT_DIR, f"per_string_{safe}.png"),
    )
    plot_training_curves(
        last_history, "Hybrid CNN",
        os.path.join(OUTPUT_DIR, f"training_curves_{safe}.png"),
    )

    results.append({
        "name": "Hybrid CNN",
        "accuracy": cnn_mean_acc,
        "train_time": cnn_total_time / 5,
        "per_string": cnn_mean_per_string.tolist(),
        "notes": "3 conv layers + FC, 50 epochs, Adam + cosine LR",
        "training_curves": True,
    })

    # -- Model 3: Pure CNN -------------------------------------------------
    print("\n" + "-" * 60)
    print("  MODEL 3: Pure CNN")
    print("-" * 60)

    pcnn_accs = []
    pcnn_per_string = np.zeros((5, 6))
    all_pcnn_preds = np.zeros(len(labels), dtype=int)
    all_pcnn_used = np.zeros(len(labels), dtype=bool)
    pcnn_total_time = 0.0
    last_history = None

    for fold, (train_idx, val_idx) in enumerate(fold_indices):
        print(f"\n  Fold {fold + 1}/5:")
        model = PureCNN(n_classes, input_shape)
        _, y_pred, y_val, acc, fold_time, history = train_cnn(
            model, "Pure CNN", torch_ds, train_idx, val_idx,
            epochs=60, lr=8e-4,
        )
        pcnn_accs.append(acc)
        pcnn_total_time += fold_time
        all_pcnn_preds[val_idx] = y_pred
        all_pcnn_used[val_idx] = True
        last_history = history

        for si in range(6):
            mask = [metadata[i][0] == si for i in val_idx]
            if any(mask):
                s_true = y_val[mask]
                s_pred = y_pred[mask]
                pcnn_per_string[fold, si] = accuracy_score(s_true, s_pred)

    pcnn_mean_acc = np.mean(pcnn_accs)
    pcnn_mean_per_string = pcnn_per_string.mean(axis=0)

    safe = "pure_cnn"
    plot_confusion_matrix(
        labels[all_pcnn_used], all_pcnn_preds[all_pcnn_used], n_classes,
        f"Pure CNN -- Val Acc: {pcnn_mean_acc:.1%}",
        os.path.join(OUTPUT_DIR, f"confusion_{safe}.png"),
    )
    plot_string_fret_heatmap(
        labels, all_pcnn_preds, metadata,
        np.where(all_pcnn_used)[0].tolist(),
        f"Pure CNN -- Per-Fret Accuracy",
        os.path.join(OUTPUT_DIR, f"fret_heatmap_{safe}.png"),
    )
    plot_per_string_accuracy(
        labels, all_pcnn_preds, metadata,
        np.where(all_pcnn_used)[0].tolist(),
        f"Pure CNN -- Per-String Accuracy",
        os.path.join(OUTPUT_DIR, f"per_string_{safe}.png"),
    )
    plot_training_curves(
        last_history, "Pure CNN",
        os.path.join(OUTPUT_DIR, f"training_curves_{safe}.png"),
    )

    results.append({
        "name": "Pure CNN",
        "accuracy": pcnn_mean_acc,
        "train_time": pcnn_total_time / 5,
        "per_string": pcnn_mean_per_string.tolist(),
        "notes": "4 conv layers + GAP, 60 epochs, Adam + cosine LR",
        "training_curves": True,
    })

    # -- Generate Report ---------------------------------------------------
    print("\n" + "-" * 60)
    print("  GENERATING REPORT")
    print("-" * 60)

    metadata_info = {
        "total": len(ds.samples),
        "n_classes": n_classes,
        "per_class": len(ds.samples) // n_classes,
        "sample_rate": ds.samples[0].sample_rate,
    }
    generate_report(results, metadata_info)

    # Save raw results JSON
    json_path = os.path.join(OUTPUT_DIR, "results.json")
    with open(json_path, "w") as f:
        json.dump(results, f, indent=2)
    print(f"  Raw results: {json_path}")

    # Save model weights (retrain on last fold for saved checkpoint)
    import joblib

    # RF: save last-fold model
    rf_path = os.path.join(OUTPUT_DIR, "random_forest.joblib")
    joblib.dump(rf_model, rf_path)
    print(f"  RF model: {rf_path}")

    # CNNs: train one final model on 80/20 split and save best weights
    from sklearn.model_selection import train_test_split as tts
    final_train, final_val = tts(
        range(len(labels)), test_size=0.2, stratify=labels, random_state=42
    )

    print("  Saving Hybrid CNN weights...")
    hcnn = HybridCNN(n_classes, input_shape)
    hcnn_model, _, _, hcnn_acc, _, _ = train_cnn(
        hcnn, "Hybrid CNN (save)", torch_ds, final_train, final_val, epochs=50, lr=1e-3
    )
    torch.save({
        "state_dict": hcnn_model.state_dict(),
        "n_classes": n_classes,
        "accuracy": hcnn_acc,
        "round": 1,
    }, os.path.join(OUTPUT_DIR, "hybrid_cnn.pt"))

    print("  Saving Pure CNN weights...")
    pcnn = PureCNN(n_classes, input_shape)
    pcnn_model, _, _, pcnn_acc, _, _ = train_cnn(
        pcnn, "Pure CNN (save)", torch_ds, final_train, final_val, epochs=60, lr=8e-4
    )
    torch.save({
        "state_dict": pcnn_model.state_dict(),
        "n_classes": n_classes,
        "accuracy": pcnn_acc,
        "round": 1,
    }, os.path.join(OUTPUT_DIR, "pure_cnn.pt"))

    print(f"  All models saved to {OUTPUT_DIR}")

    # -- Final Summary -----------------------------------------------------
    print("\n" + "=" * 60)
    print("  ROUND 1 BASELINE -- SUMMARY")
    print("=" * 60)
    for r in results:
        print(f"  {r['name']:20s}  {r['accuracy']:.1%}")
    print("=" * 60)


if __name__ == "__main__":
    main()
