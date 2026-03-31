"""
Round 5: Data Augmentation

With only ~10 samples per class, the CNNs are prone to overfitting.
This round adds safe augmentations that preserve the guitar string+fret
label while increasing training set diversity:

  - Gain variation (different pick attack strengths)
  - Noise injection (ambient / preamp hiss)
  - Time shift (onset detection timing jitter)
  - Slight time stretch (pitch-invariant, <= 5%)

DANGEROUS augmentations deliberately excluded:
  - Pitch shifting  -- even 0.5 semitone makes one fret sound like another
  - Large time stretch  -- changes harmonic content too much
  - SpecAugment freq masking  -- can mask the exact harmonics that
    distinguish strings

Key design: augmented copies of a sample are ALWAYS in the same CV fold
as the original.  Validation sets contain ONLY original (unaugmented)
samples to prevent data leakage.

Usage:
    cd <project_root>
    source ml/venv/bin/activate
    python -u ml/training/round_05_augmentation.py
"""

import sys
import os
import json
import time
import random
import warnings
from collections import defaultdict

import numpy as np

import matplotlib
matplotlib.use("Agg")
import matplotlib.pyplot as plt
import seaborn as sns

from sklearn.ensemble import RandomForestClassifier
from sklearn.metrics import accuracy_score, confusion_matrix
from sklearn.model_selection import StratifiedKFold

import torch
import torch.nn as nn
import torch.optim as optim
from torch.utils.data import Dataset, DataLoader, SubsetRandomSampler

import librosa

warnings.filterwarnings("ignore", category=FutureWarning)

# -- Paths ------------------------------------------------------------------

PROJECT_ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), "../.."))
DATASET_PATH = os.path.join(PROJECT_ROOT, "guitar_training_data.msgpack")
OUTPUT_DIR = os.path.join(os.path.dirname(__file__), "round_05")
ROUND_01_DIR = os.path.join(os.path.dirname(__file__), "round_01")

sys.path.insert(0, PROJECT_ROOT)
from ml.loader import GuitarDataset

# -- Feature params ---------------------------------------------------------

N_MELS = 64
N_FFT = 1024
HOP_LENGTH = 256
SAMPLE_RATE = 48000
STRING_NAMES = ["E2 (low)", "A2", "D3", "G3", "B3", "E4 (high)"]

# Augmentation config
N_AUGMENTED_COPIES = 3  # per original sample (~5,520 total from 1,380)
RNG_SEED = 42


# ==========================================================================
# AUGMENTATIONS
# ==========================================================================

def aug_gain(audio, rng):
    """Scale amplitude by random factor in [0.7, 1.3].

    Simulates different pick attack strengths / dynamics.
    """
    gain = rng.uniform(0.7, 1.3)
    return np.clip(audio * gain, -1.0, 1.0), {"gain": round(gain, 3)}


def aug_noise(audio, rng):
    """Add small Gaussian noise proportional to signal RMS.

    Simulates ambient noise / preamp hiss.  Noise level is 5% of
    the signal RMS so the note is still clearly dominant.
    """
    rms = max(np.sqrt(np.mean(audio ** 2)), 1e-6)
    noise_level = rms * 0.05
    noise = rng.normal(0, noise_level, len(audio)).astype(np.float32)
    return np.clip(audio + noise, -1.0, 1.0), {"noise_level": round(noise_level, 6)}


def aug_time_shift(audio, rng):
    """Shift onset by +/-10ms (480 samples at 48kHz).

    Simulates onset detection timing variation.  Zero-pads at
    start/end to maintain original length.
    """
    max_shift = int(0.010 * SAMPLE_RATE)  # 480 samples
    shift = rng.randint(-max_shift, max_shift + 1)

    shifted = np.zeros_like(audio)
    if shift > 0:
        shifted[shift:] = audio[:-shift] if shift < len(audio) else 0
    elif shift < 0:
        shifted[:shift] = audio[-shift:]
    else:
        shifted[:] = audio

    return shifted, {"shift_samples": shift}


def aug_time_stretch(audio, rng):
    """Pitch-invariant time stretch by factor in [0.95, 1.05].

    5% is small enough that perceived pitch is preserved (well within
    a single fret/semitone boundary), but it changes the temporal
    envelope slightly, adding diversity.
    """
    rate = rng.uniform(0.95, 1.05)
    stretched = librosa.effects.time_stretch(audio, rate=rate)

    # Restore original length: trim or zero-pad
    target_len = len(audio)
    if len(stretched) > target_len:
        stretched = stretched[:target_len]
    elif len(stretched) < target_len:
        stretched = np.pad(stretched, (0, target_len - len(stretched)),
                           mode="constant", constant_values=0.0)

    return stretched.astype(np.float32), {"stretch_rate": round(rate, 4)}


AUGMENTATION_FNS = [
    ("gain", aug_gain),
    ("noise", aug_noise),
    ("time_shift", aug_time_shift),
    ("time_stretch", aug_time_stretch),
]


def augment_sample(audio, rng, n_augs=None):
    """Apply a random combination of 1-2 augmentations.

    Returns (augmented_audio, list_of_augmentation_names, params_dict).
    """
    if n_augs is None:
        n_augs = rng.randint(1, 3)  # 1 or 2

    chosen = rng.sample(AUGMENTATION_FNS, min(n_augs, len(AUGMENTATION_FNS)))
    names = []
    params = {}
    result = audio.copy()

    for name, fn in chosen:
        result, p = fn(result, rng)
        names.append(name)
        params[name] = p

    return result, names, params


# ==========================================================================
# FEATURE EXTRACTION
# ==========================================================================

def extract_mel(audio, sr):
    """Extract log-mel spectrogram."""
    mel = librosa.feature.melspectrogram(
        y=audio, sr=sr, n_mels=N_MELS, n_fft=N_FFT, hop_length=HOP_LENGTH,
    )
    return librosa.power_to_db(mel, ref=np.max)


def prepare_dataset(ds):
    """Extract mel-spectrograms and labels from the original dataset.

    Returns:
        audios: list of float32 audio arrays (raw waveform)
        mel_features: list of (N_MELS, TIME) log-mel arrays
        labels: (N,) int array of zero-indexed class IDs
        metadata: list of (string_idx, fret) tuples
    """
    print("Extracting features from original samples...")
    audios = []
    mel_features = []
    labels = []
    metadata = []

    for i, sample in enumerate(ds.samples):
        audio = sample.audio_np
        sr = sample.sample_rate
        mel = extract_mel(audio, sr)

        audios.append(audio)
        mel_features.append(mel)
        labels.append(sample.class_id - 1)  # zero-index
        metadata.append((sample.string_idx, sample.fret))

        if (i + 1) % 200 == 0:
            print("  %d/%d" % (i + 1, len(ds.samples)))

    labels = np.array(labels)
    print("  %d original samples extracted" % len(labels))
    return audios, mel_features, labels, metadata


def pad_mel_features(mel_list):
    """Pad all mel-spectrograms to the same time dimension."""
    max_time = max(f.shape[1] for f in mel_list)
    padded = []
    for f in mel_list:
        if f.shape[1] < max_time:
            f = np.pad(f, ((0, 0), (0, max_time - f.shape[1])),
                       mode="constant", constant_values=f.min())
        elif f.shape[1] > max_time:
            f = f[:, :max_time]
        padded.append(f)
    return padded, max_time


# ==========================================================================
# AUGMENTATION PIPELINE
# ==========================================================================

def generate_augmented_dataset(audios, mel_features, labels, metadata,
                               sample_rates, n_copies=N_AUGMENTED_COPIES):
    """Create augmented copies of all training samples.

    For each original sample, generates n_copies augmented versions.
    Each augmented copy gets a random 1-2 augmentation combination.

    Returns:
        aug_audios: list of augmented audio arrays
        aug_mels:   list of augmented mel-spectrograms
        aug_labels: (M,) label array for augmented samples
        aug_metadata: list of (string_idx, fret) for augmented samples
        aug_origins: (M,) array mapping each augmented sample to its
                     original sample index (for fold assignment)
        aug_info:    list of dicts describing what augmentations were applied
    """
    rng = random.Random(RNG_SEED)
    np_rng = np.random.RandomState(RNG_SEED)

    # Wrap np_rng in an adapter so augmentation fns can use it
    class RngAdapter:
        def __init__(self, py_rng, np_rng):
            self._py = py_rng
            self._np = np_rng

        def uniform(self, a, b):
            return self._np.uniform(a, b)

        def normal(self, mu, sigma, size):
            return self._np.normal(mu, sigma, size)

        def randint(self, a, b):
            return self._np.randint(a, b + 1)

        def sample(self, population, k):
            return self._py.sample(population, k)

    adapter = RngAdapter(rng, np_rng)

    aug_audios = []
    aug_mels = []
    aug_labels = []
    aug_metadata = []
    aug_origins = []
    aug_info = []

    n_orig = len(audios)
    print("Generating %d augmented copies per sample (%d originals)..." %
          (n_copies, n_orig))

    for i in range(n_orig):
        audio = audios[i]
        sr = sample_rates[i]
        label = labels[i]
        meta = metadata[i]

        for c in range(n_copies):
            aug_audio, aug_names, aug_params = augment_sample(audio, adapter)
            aug_mel = extract_mel(aug_audio, sr)

            aug_audios.append(aug_audio)
            aug_mels.append(aug_mel)
            aug_labels.append(label)
            aug_metadata.append(meta)
            aug_origins.append(i)
            aug_info.append({
                "original_idx": i,
                "copy": c,
                "augmentations": aug_names,
                "params": aug_params,
            })

        if (i + 1) % 200 == 0:
            print("  %d/%d originals processed" % (i + 1, n_orig))

    aug_labels = np.array(aug_labels)
    aug_origins = np.array(aug_origins)

    print("  Generated %d augmented samples" % len(aug_labels))
    print("  Total dataset: %d (original) + %d (augmented) = %d" %
          (n_orig, len(aug_labels), n_orig + len(aug_labels)))

    # Augmentation type distribution
    type_counts = defaultdict(int)
    for info in aug_info:
        for name in info["augmentations"]:
            type_counts[name] += 1
    print("  Augmentation distribution:")
    for name, count in sorted(type_counts.items()):
        print("    %-15s %d" % (name, count))

    return aug_audios, aug_mels, aug_labels, aug_metadata, aug_origins, aug_info


# ==========================================================================
# MODELS (same architectures as Round 1)
# ==========================================================================

class GuitarMelDataset(Dataset):
    """PyTorch dataset wrapping mel-spectrograms."""

    def __init__(self, features, labels):
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
        return self.fc(self.conv(x))


class PureCNN(nn.Module):
    """Deeper ConvNet with global average pooling."""

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
        return self.classifier(self.features(x))


# ==========================================================================
# TRAINING
# ==========================================================================

def train_random_forest(features, labels, train_idx, val_idx):
    """Train Random Forest on flattened mel-spectrograms."""
    X_flat = np.array([f.flatten() for f in features])

    rf = RandomForestClassifier(n_estimators=200, n_jobs=-1, random_state=42)
    rf.fit(X_flat[train_idx], labels[train_idx])
    y_pred = rf.predict(X_flat[val_idx])
    acc = accuracy_score(labels[val_idx], y_pred)

    return y_pred, labels[val_idx], acc


def train_cnn(model, name, dataset, train_idx, val_idx, epochs, lr):
    """Train a PyTorch CNN model with augmented train / original-only val."""
    device = torch.device("cpu")
    model = model.to(device)

    train_loader = DataLoader(dataset, batch_size=32,
                              sampler=SubsetRandomSampler(train_idx))

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
        n_train = 0
        for X_batch, y_batch in train_loader:
            X_batch, y_batch = X_batch.to(device), y_batch.to(device)
            optimizer.zero_grad()
            outputs = model(X_batch)
            loss = criterion(outputs, y_batch)
            loss.backward()
            optimizer.step()
            running_loss += loss.item() * X_batch.size(0)
            n_train += X_batch.size(0)

        scheduler.step()
        epoch_loss = running_loss / max(n_train, 1)
        history["train_loss"].append(epoch_loss)

        # Validation on original samples only
        model.eval()
        correct, total = 0, 0
        with torch.no_grad():
            for i in val_idx:
                X_i, y_i = dataset[i]
                pred = model(X_i.unsqueeze(0).to(device)).argmax(1).item()
                correct += (pred == y_i.item())
                total += 1

        val_acc = correct / total if total > 0 else 0.0
        history["val_acc"].append(val_acc)

        if val_acc > best_acc:
            best_acc = val_acc
            best_state = {k: v.cpu().clone() for k, v in model.state_dict().items()}

        if (epoch + 1) % 10 == 0 or epoch == 0:
            print("    Epoch %3d: loss=%.4f val_acc=%.4f" %
                  (epoch + 1, epoch_loss, val_acc))

    train_time = time.time() - t0
    print("  Best: %.4f (%.1fs)" % (best_acc, train_time))

    # Restore best weights
    model.load_state_dict(best_state)
    model.eval()

    # Collect val predictions (original samples only)
    preds = np.zeros(len(val_idx), dtype=int)
    true_labels = np.zeros(len(val_idx), dtype=int)
    with torch.no_grad():
        for i, idx in enumerate(val_idx):
            X_i, y_i = dataset[idx]
            preds[i] = model(X_i.unsqueeze(0).to(device)).argmax(1).item()
            true_labels[i] = y_i.item()

    return preds, true_labels, best_acc, train_time, history


# ==========================================================================
# FOLD CONSTRUCTION (leakage-safe)
# ==========================================================================

def build_augmented_folds(orig_labels, aug_origins, n_orig, n_aug, n_splits=5):
    """Build CV folds where augmented copies stay with their originals.

    Stratification is on ORIGINAL sample labels.  Each fold's training
    set includes the original training samples PLUS their augmented
    copies.  Each fold's validation set contains ONLY original samples.

    Returns:
        folds: list of (train_idx, val_idx) into the combined dataset
               where indices 0..n_orig-1 are originals and
               n_orig..n_orig+n_aug-1 are augmented.
    """
    skf = StratifiedKFold(n_splits=n_splits, shuffle=True, random_state=42)
    folds = []

    for orig_train_idx, orig_val_idx in skf.split(np.zeros(n_orig), orig_labels):
        # Training: original train samples + their augmented copies
        orig_train_set = set(orig_train_idx.tolist())
        aug_train_idx = []
        for j in range(n_aug):
            if aug_origins[j] in orig_train_set:
                aug_train_idx.append(n_orig + j)

        combined_train = np.concatenate([
            orig_train_idx,
            np.array(aug_train_idx, dtype=int),
        ])

        # Validation: original samples only (no augmented data)
        folds.append((combined_train, orig_val_idx))

    return folds


# ==========================================================================
# VISUALIZATION
# ==========================================================================

def plot_augmentation_examples(audios, sample_rates, output_dir, rng_seed=42):
    """Show original vs augmented waveforms and spectrograms for 4 samples.

    Saves augmentation_examples.png with a 4x4 grid:
      Row per sample, columns: original waveform, augmented waveform,
      original spectrogram, augmented spectrogram.
    """
    rng = random.Random(rng_seed)
    np_rng = np.random.RandomState(rng_seed + 1)

    class RngAdapter:
        def __init__(self, py_rng, np_rng):
            self._py = py_rng
            self._np = np_rng

        def uniform(self, a, b):
            return self._np.uniform(a, b)

        def normal(self, mu, sigma, size):
            return self._np.normal(mu, sigma, size)

        def randint(self, a, b):
            return self._np.randint(a, b + 1)

        def sample(self, population, k):
            return self._py.sample(population, k)

    adapter = RngAdapter(rng, np_rng)

    # Pick 4 samples spread across different strings
    indices = [0, len(audios) // 6, len(audios) // 3, len(audios) // 2]

    fig, axes = plt.subplots(4, 4, figsize=(20, 16))
    fig.suptitle("Round 5: Augmentation Examples\n"
                 "Original vs Augmented Waveforms & Spectrograms",
                 fontsize=14, color="white", y=0.98)

    for row, idx in enumerate(indices):
        audio = audios[idx]
        sr = sample_rates[idx]
        aug_audio, aug_names, aug_params = augment_sample(audio, adapter)

        # Original waveform
        ax = axes[row][0]
        t = np.arange(len(audio)) / sr
        ax.plot(t, audio, color="#5c6bc0", linewidth=0.3, alpha=0.8)
        ax.set_title("Original (sample %d)" % idx, fontsize=10, color="white")
        ax.set_ylabel("Amplitude", fontsize=8, color="white")
        ax.set_ylim(-1, 1)
        ax.set_facecolor("#1a1a2e")
        ax.tick_params(colors="white", labelsize=7)

        # Augmented waveform
        ax = axes[row][1]
        t_aug = np.arange(len(aug_audio)) / sr
        ax.plot(t_aug, aug_audio, color="#26a69a", linewidth=0.3, alpha=0.8)
        aug_desc = " + ".join(aug_names)
        ax.set_title("Augmented: %s" % aug_desc, fontsize=10, color="white")
        ax.set_ylim(-1, 1)
        ax.set_facecolor("#1a1a2e")
        ax.tick_params(colors="white", labelsize=7)

        # Original spectrogram
        ax = axes[row][2]
        mel_orig = extract_mel(audio, sr)
        ax.imshow(mel_orig, aspect="auto", origin="lower", cmap="magma")
        ax.set_title("Original Mel", fontsize=10, color="white")
        ax.set_ylabel("Mel bin", fontsize=8, color="white")
        ax.set_facecolor("#1a1a2e")
        ax.tick_params(colors="white", labelsize=7)

        # Augmented spectrogram
        ax = axes[row][3]
        mel_aug = extract_mel(aug_audio, sr)
        ax.imshow(mel_aug, aspect="auto", origin="lower", cmap="magma")
        ax.set_title("Augmented Mel", fontsize=10, color="white")
        ax.set_facecolor("#1a1a2e")
        ax.tick_params(colors="white", labelsize=7)

    for ax_row in axes:
        for ax in ax_row:
            for spine in ax.spines.values():
                spine.set_color("#444")

    fig.tight_layout(rect=[0, 0, 1, 0.96])
    path = os.path.join(output_dir, "augmentation_examples.png")
    fig.savefig(path, dpi=150, facecolor="#0a0a1a")
    plt.close(fig)
    print("  Saved: %s" % path)


def plot_comparison(r5_results, previous, output_dir):
    """Bar chart comparing Round 5 accuracy vs all previous rounds."""
    model_names = [r["name"] for r in r5_results]
    round_keys = sorted(previous.keys())

    fig, axes = plt.subplots(1, 2, figsize=(18, 7))

    # -- Overall accuracy comparison --
    ax = axes[0]
    n_models = len(model_names)
    n_rounds = len(round_keys) + 1  # previous rounds + round 5
    bar_width = 0.8 / n_rounds
    x = np.arange(n_models)

    colors = ["#5c6bc0", "#7e57c2", "#ab47bc", "#ec407a", "#26a69a"]

    for ri, rk in enumerate(round_keys):
        prev = {r["name"]: r["accuracy"] for r in previous[rk]}
        accs = [prev.get(n, 0) * 100 for n in model_names]
        label = rk.replace("_", " ").title()
        bars = ax.bar(x + ri * bar_width, accs, bar_width,
                      label=label, color=colors[ri % len(colors)], alpha=0.85)
        for bar, val in zip(bars, accs):
            if val > 0:
                ax.text(bar.get_x() + bar.get_width() / 2, bar.get_height() + 0.2,
                        "%.1f%%" % val, ha="center", va="bottom",
                        fontsize=7, color="white")

    # Round 5
    r5_accs = [r["accuracy"] * 100 for r in r5_results]
    bars = ax.bar(x + len(round_keys) * bar_width, r5_accs, bar_width,
                  label="Round 05 (augmentation)", color="#26a69a", alpha=0.95)
    for bar, val in zip(bars, r5_accs):
        ax.text(bar.get_x() + bar.get_width() / 2, bar.get_height() + 0.2,
                "%.1f%%" % val, ha="center", va="bottom",
                fontsize=8, color="white", fontweight="bold")

    ax.set_ylabel("Accuracy (%)")
    ax.set_title("Overall Accuracy: All Rounds")
    ax.set_xticks(x + (n_rounds - 1) * bar_width / 2)
    ax.set_xticklabels(model_names)
    ax.legend(fontsize=8)
    ax.set_ylim(85, 100)
    ax.set_facecolor("#1a1a2e")
    ax.grid(True, alpha=0.2, axis="y")

    # -- Per-string accuracy (Round 5 only) --
    ax2 = axes[1]
    x2 = np.arange(6)
    bar_w = 0.25

    for i, r in enumerate(r5_results):
        ps = [v * 100 for v in r["per_string"]]
        ax2.bar(x2 + i * bar_w, ps, bar_w, label=r["name"], alpha=0.85)

    ax2.set_ylabel("Accuracy (%)")
    ax2.set_title("Per-String Accuracy (Round 5)")
    ax2.set_xticks(x2 + bar_w)
    ax2.set_xticklabels(STRING_NAMES, rotation=30, ha="right", fontsize=9)
    ax2.legend(fontsize=8)
    ax2.set_ylim(70, 100)
    ax2.set_facecolor("#1a1a2e")
    ax2.grid(True, alpha=0.2, axis="y")

    fig.tight_layout()
    path = os.path.join(output_dir, "round_05_comparison.png")
    fig.savefig(path, dpi=150, facecolor="#0a0a1a")
    plt.close(fig)
    print("  Saved: %s" % path)


def plot_training_curves(history, name, path):
    """Plot loss and accuracy curves for CNN training."""
    fig, (ax1, ax2) = plt.subplots(1, 2, figsize=(12, 5))

    ax1.plot(history["train_loss"], color="#5c6bc0")
    ax1.set_title("%s -- Training Loss" % name, color="white")
    ax1.set_xlabel("Epoch")
    ax1.set_ylabel("Loss")
    ax1.set_facecolor("#1a1a2e")
    ax1.tick_params(colors="white")

    ax2.plot(history["val_acc"], color="#26a69a")
    ax2.set_title("%s -- Validation Accuracy (original samples)" % name,
                  color="white")
    ax2.set_xlabel("Epoch")
    ax2.set_ylabel("Accuracy")
    ax2.set_ylim(0, 1)
    ax2.set_facecolor("#1a1a2e")
    ax2.tick_params(colors="white")

    fig.tight_layout()
    fig.savefig(path, dpi=150, facecolor="#0a0a1a")
    plt.close(fig)
    print("  Saved: %s" % path)


# ==========================================================================
# REPORT
# ==========================================================================

def generate_report(r5_results, previous, dataset_info, aug_info, output_dir):
    """Generate ROUND_05_AUGMENTATION.md report."""
    r1 = previous.get("round_01", [])
    r1_acc = {r["name"]: r for r in r1} if r1 else {}

    # Count augmentation types
    type_counts = defaultdict(int)
    combo_counts = defaultdict(int)
    for info in aug_info:
        for name in info["augmentations"]:
            type_counts[name] += 1
        combo_counts[" + ".join(sorted(info["augmentations"]))] += 1

    lines = []
    lines.append("# Round 5: Data Augmentation")
    lines.append("")
    lines.append("## Hypothesis")
    lines.append("")
    lines.append("With only ~10 samples per class, the models (especially CNNs) are prone")
    lines.append("to overfitting. Data augmentation creates modified copies of training")
    lines.append("samples that preserve the guitar string+fret label while increasing")
    lines.append("diversity. This should improve generalization without needing to capture")
    lines.append("more real data.")
    lines.append("")

    lines.append("## Augmentations Applied")
    lines.append("")
    lines.append("### Safe augmentations (label-preserving):")
    lines.append("")
    lines.append("| Augmentation | Description | Parameters |")
    lines.append("|-------------|-------------|------------|")
    lines.append("| Gain variation | Scale amplitude by random factor | [0.7, 1.3] |")
    lines.append("| Noise injection | Add Gaussian noise (5% of signal RMS) | noise_level = rms * 0.05 |")
    lines.append("| Time shift | Shift onset by +/-10ms | +/-480 samples @ 48kHz |")
    lines.append("| Time stretch | Pitch-invariant stretch/compress | [0.95, 1.05] rate |")
    lines.append("")
    lines.append("### Deliberately excluded (DANGEROUS for this task):")
    lines.append("")
    lines.append("- **Pitch shifting** -- even 0.5 semitone makes one fret sound like another")
    lines.append("- **Large time stretch** -- changes harmonic content too much")
    lines.append("- **SpecAugment freq masking** -- can mask exact harmonics that distinguish strings")
    lines.append("")

    lines.append("### Augmentation distribution:")
    lines.append("")
    lines.append("| Type | Count |")
    lines.append("|------|-------|")
    for name, count in sorted(type_counts.items()):
        lines.append("| %s | %d |" % (name, count))
    lines.append("")
    lines.append("Each augmented sample receives 1-2 randomly chosen augmentations.")
    lines.append("")

    lines.append("## Dataset")
    lines.append("")
    lines.append("| Metric | Value |")
    lines.append("|--------|-------|")
    lines.append("| Original samples | %d |" % dataset_info["n_original"])
    lines.append("| Augmented copies per sample | %d |" % dataset_info["n_copies"])
    lines.append("| Total augmented | %d |" % dataset_info["n_augmented"])
    lines.append("| Total training pool | %d |" % dataset_info["n_total"])
    lines.append("| Classes | %d |" % dataset_info["n_classes"])
    lines.append("| Effective samples per class | ~%d |" %
                 (dataset_info["n_total"] // dataset_info["n_classes"]))
    lines.append("")

    lines.append("## Evaluation")
    lines.append("")
    lines.append("- **Method:** 5-fold stratified CV (stratified on ORIGINAL samples)")
    lines.append("- **Leakage prevention:** augmented copies always in same fold as original")
    lines.append("- **Validation:** ONLY original (unaugmented) samples -- never evaluate on augmented data")
    lines.append("- **Models:** same architectures as Round 1 (RF, Hybrid CNN, Pure CNN)")
    lines.append("")

    lines.append("## Results")
    lines.append("")

    # Build round columns dynamically
    round_names = sorted(previous.keys())
    header = "| Model |"
    sep = "|-------|"
    for rn in round_names:
        header += " %s |" % rn.replace("_", " ").title()
        sep += "---------|"
    header += " Round 05 | Delta (R5-R1) |"
    sep += "---------|---------------|"

    lines.append(header)
    lines.append(sep)

    for r in r5_results:
        row = "| %s |" % r["name"]
        for rn in round_names:
            prev_data = previous.get(rn, [])
            match = next((p for p in prev_data if p["name"] == r["name"]), None)
            if match:
                row += " %.1f%% |" % (match["accuracy"] * 100)
            else:
                row += " -- |"
        row += " **%.1f%%** |" % (r["accuracy"] * 100)

        r1_entry = r1_acc.get(r["name"])
        if r1_entry:
            delta = (r["accuracy"] - r1_entry["accuracy"]) * 100
            row += " %+.1f%% |" % delta
        else:
            row += " -- |"
        lines.append(row)

    lines.append("")

    lines.append("## Per-String Accuracy")
    lines.append("")
    ps_header = "| String |"
    ps_sep = "|--------|"
    for r in r5_results:
        ps_header += " %s R5 |" % r["name"]
        ps_sep += "---------|"
    for r in r5_results:
        if r["name"] in r1_acc:
            ps_header += " %s R1 |" % r["name"]
            ps_sep += "---------|"

    lines.append(ps_header)
    lines.append(ps_sep)

    for si in range(6):
        row = "| %s" % STRING_NAMES[si]
        for r in r5_results:
            row += " | %.1f%%" % (r["per_string"][si] * 100)
        for r in r5_results:
            if r["name"] in r1_acc:
                row += " | %.1f%%" % (r1_acc[r["name"]]["per_string"][si] * 100)
        row += " |"
        lines.append(row)

    lines.append("")

    lines.append("## Key Findings")
    lines.append("")
    for r in r5_results:
        r1_entry = r1_acc.get(r["name"])
        if r1_entry:
            delta = (r["accuracy"] - r1_entry["accuracy"]) * 100
            direction = "improved" if delta > 0 else ("unchanged" if delta == 0 else "decreased")
            lines.append("- **%s**: %s by %.1f%% (%.1f%% -> %.1f%%)" %
                         (r["name"], direction, abs(delta),
                          r1_entry["accuracy"] * 100, r["accuracy"] * 100))

            # E2 (low) string comparison
            e2_delta = (r["per_string"][0] - r1_entry["per_string"][0]) * 100
            lines.append("  - E2 (low) string: %+.1f%% (%.1f%% -> %.1f%%)" %
                         (e2_delta,
                          r1_entry["per_string"][0] * 100,
                          r["per_string"][0] * 100))

    lines.append("")

    lines.append("## Visualizations")
    lines.append("")
    lines.append("- `augmentation_examples.png` -- original vs augmented waveforms & spectrograms")
    lines.append("- `round_05_comparison.png` -- accuracy comparison across all rounds")
    lines.append("- `training_curves_*.png` -- CNN training loss and validation accuracy")
    lines.append("")
    lines.append("![Augmentation Examples](augmentation_examples.png)")
    lines.append("")
    lines.append("![Round Comparison](round_05_comparison.png)")
    lines.append("")
    lines.append("*Generated: %s*" % time.strftime("%Y-%m-%d %H:%M"))
    lines.append("")

    path = os.path.join(output_dir, "ROUND_05_AUGMENTATION.md")
    with open(path, "w") as f:
        f.write("\n".join(lines))
    print("  Saved: %s" % path)


# ==========================================================================
# MAIN
# ==========================================================================

def main():
    print("=" * 60)
    print("  Round 5: Data Augmentation")
    print("=" * 60)

    os.makedirs(OUTPUT_DIR, exist_ok=True)

    # -- Load previous results for comparison --
    previous = {}
    for name, dirpath in [("round_01", ROUND_01_DIR)]:
        rpath = os.path.join(dirpath, "results.json")
        if os.path.exists(rpath):
            with open(rpath) as f:
                previous[name] = json.load(f)
            print("  Loaded %s results" % name)

    # Also try round_02, round_03, round_04
    for rnd in ["round_02", "round_03", "round_04"]:
        rpath = os.path.join(os.path.dirname(__file__), rnd, "results.json")
        if os.path.exists(rpath):
            with open(rpath) as f:
                previous[rnd] = json.load(f)
            print("  Loaded %s results" % rnd)

    # -- Load dataset --
    ds = GuitarDataset.load(DATASET_PATH)
    audios, mel_features, labels, metadata = prepare_dataset(ds)
    sample_rates = [s.sample_rate for s in ds.samples]

    n_orig = len(labels)
    n_classes = len(np.unique(labels))
    print("\n  Original: %d samples, %d classes" % (n_orig, n_classes))

    # -- Generate augmented data --
    print("\n" + "-" * 60)
    print("  AUGMENTATION PIPELINE")
    print("-" * 60)

    aug_audios, aug_mels, aug_labels, aug_metadata, aug_origins, aug_info = \
        generate_augmented_dataset(audios, mel_features, labels, metadata,
                                   sample_rates, n_copies=N_AUGMENTED_COPIES)

    n_aug = len(aug_labels)

    # -- Combine original + augmented mel features --
    # Originals are indices 0..n_orig-1, augmented are n_orig..n_orig+n_aug-1
    all_mels = mel_features + aug_mels
    all_labels = np.concatenate([labels, aug_labels])
    all_metadata = metadata + aug_metadata

    # Pad all to uniform time dimension
    all_mels_padded, max_time = pad_mel_features(all_mels)
    input_shape = all_mels_padded[0].shape

    print("\n  Combined dataset: %d samples" % len(all_labels))
    print("  Spectrogram shape: %s" % (input_shape,))

    # -- Visualize augmentation examples --
    print("\nGenerating augmentation examples visualization...")
    plot_augmentation_examples(audios, sample_rates, OUTPUT_DIR)

    # -- Build leakage-safe folds --
    print("\nBuilding leakage-safe CV folds...")
    folds = build_augmented_folds(labels, aug_origins, n_orig, n_aug)
    for i, (ti, vi) in enumerate(folds):
        # Verify no leakage: val indices should all be < n_orig
        assert all(v < n_orig for v in vi), "Leakage detected: augmented sample in val set!"
        # Verify augmented copies of val samples are NOT in training
        val_set = set(vi.tolist())
        for j in range(n_aug):
            if aug_origins[j] in val_set:
                assert (n_orig + j) not in set(ti.tolist()), \
                    "Leakage: augmented copy of val sample found in train!"
        print("  Fold %d: train=%d (orig+aug), val=%d (orig only)" %
              (i + 1, len(ti), len(vi)))

    # -- Create PyTorch dataset --
    torch_ds = GuitarMelDataset(all_mels_padded, all_labels)

    r5_results = []

    # == Model 1: Random Forest =============================================
    print("\n" + "-" * 60)
    print("  MODEL 1: Random Forest (augmented training)")
    print("-" * 60)

    rf_accs = []
    rf_per_string = np.zeros((5, 6))

    for fold, (train_idx, val_idx) in enumerate(folds):
        y_pred, y_val, acc = train_random_forest(
            all_mels_padded, all_labels, train_idx, val_idx,
        )
        rf_accs.append(acc)
        print("  Fold %d: %.4f (train=%d, val=%d)" %
              (fold + 1, acc, len(train_idx), len(val_idx)))

        for si in range(6):
            mask = [all_metadata[i][0] == si for i in val_idx]
            if any(mask):
                rf_per_string[fold, si] = accuracy_score(
                    y_val[mask], y_pred[mask])

    rf_mean_acc = float(np.mean(rf_accs))
    rf_mean_ps = rf_per_string.mean(axis=0).tolist()

    r5_results.append({
        "name": "Random Forest",
        "accuracy": rf_mean_acc,
        "per_string": rf_mean_ps,
        "notes": "200 trees, augmented training (%dx), original-only val" %
                 (N_AUGMENTED_COPIES + 1),
    })
    print("  Mean accuracy: %.4f" % rf_mean_acc)

    # == Model 2: Hybrid CNN ================================================
    print("\n" + "-" * 60)
    print("  MODEL 2: Hybrid CNN (augmented training)")
    print("-" * 60)

    hc_accs = []
    hc_per_string = np.zeros((5, 6))
    last_hc_history = None

    for fold, (train_idx, val_idx) in enumerate(folds):
        print("\n  Fold %d:" % (fold + 1))
        model = HybridCNN(n_classes, input_shape)
        y_pred, y_val, acc, _, history = train_cnn(
            model, "Hybrid CNN", torch_ds, train_idx, val_idx,
            epochs=50, lr=1e-3,
        )
        hc_accs.append(acc)
        last_hc_history = history

        for si in range(6):
            mask = [all_metadata[i][0] == si for i in val_idx]
            if any(mask):
                hc_per_string[fold, si] = accuracy_score(
                    y_val[mask], y_pred[mask])

    hc_mean_acc = float(np.mean(hc_accs))
    hc_mean_ps = hc_per_string.mean(axis=0).tolist()

    r5_results.append({
        "name": "Hybrid CNN",
        "accuracy": hc_mean_acc,
        "per_string": hc_mean_ps,
        "notes": "3 conv layers + FC, 50 epochs, augmented training (%dx)" %
                 (N_AUGMENTED_COPIES + 1),
        "training_curves": True,
    })
    print("  Mean accuracy: %.4f" % hc_mean_acc)

    if last_hc_history:
        plot_training_curves(last_hc_history, "Hybrid CNN",
                             os.path.join(OUTPUT_DIR, "training_curves_hybrid_cnn.png"))

    # == Model 3: Pure CNN ==================================================
    print("\n" + "-" * 60)
    print("  MODEL 3: Pure CNN (augmented training)")
    print("-" * 60)

    pc_accs = []
    pc_per_string = np.zeros((5, 6))
    last_pc_history = None

    for fold, (train_idx, val_idx) in enumerate(folds):
        print("\n  Fold %d:" % (fold + 1))
        model = PureCNN(n_classes, input_shape)
        y_pred, y_val, acc, _, history = train_cnn(
            model, "Pure CNN", torch_ds, train_idx, val_idx,
            epochs=60, lr=8e-4,
        )
        pc_accs.append(acc)
        last_pc_history = history

        for si in range(6):
            mask = [all_metadata[i][0] == si for i in val_idx]
            if any(mask):
                pc_per_string[fold, si] = accuracy_score(
                    y_val[mask], y_pred[mask])

    pc_mean_acc = float(np.mean(pc_accs))
    pc_mean_ps = pc_per_string.mean(axis=0).tolist()

    r5_results.append({
        "name": "Pure CNN",
        "accuracy": pc_mean_acc,
        "per_string": pc_mean_ps,
        "notes": "4 conv layers + GAP, 60 epochs, augmented training (%dx)" %
                 (N_AUGMENTED_COPIES + 1),
        "training_curves": True,
    })
    print("  Mean accuracy: %.4f" % pc_mean_acc)

    if last_pc_history:
        plot_training_curves(last_pc_history, "Pure CNN",
                             os.path.join(OUTPUT_DIR, "training_curves_pure_cnn.png"))

    # == Save results =======================================================
    print("\n" + "-" * 60)
    print("  RESULTS")
    print("-" * 60)

    results_path = os.path.join(OUTPUT_DIR, "results.json")
    with open(results_path, "w") as f:
        json.dump(r5_results, f, indent=2)
    print("  Saved: %s" % results_path)

    # == Print comparison table =============================================
    print("\n" + "=" * 60)
    print("  ROUND 5 vs PREVIOUS ROUNDS")
    print("=" * 60)

    for r5 in r5_results:
        line = "  %-20s  R5=%.1f%%" % (r5["name"], r5["accuracy"] * 100)
        for rnd, prev in sorted(previous.items()):
            match = next((p for p in prev if p["name"] == r5["name"]), None)
            if match:
                delta = r5["accuracy"] - match["accuracy"]
                line += "  %s=%.1f%% (%+.1f%%)" % (
                    rnd, match["accuracy"] * 100, delta * 100)
        print(line)

    # Per-string comparison with Round 1
    if "round_01" in previous:
        print("\n  Per-String Accuracy (Round 5 vs Round 1):")
        r1_data = {r["name"]: r for r in previous["round_01"]}
        for r5 in r5_results:
            r1_entry = r1_data.get(r5["name"])
            if r1_entry:
                print("\n    %s:" % r5["name"])
                for si in range(6):
                    r5_ps = r5["per_string"][si] * 100
                    r1_ps = r1_entry["per_string"][si] * 100
                    delta = r5_ps - r1_ps
                    marker = " ***" if abs(delta) > 2.0 else ""
                    print("      %-12s  R1=%.1f%%  R5=%.1f%%  (%+.1f%%)%s" %
                          (STRING_NAMES[si], r1_ps, r5_ps, delta, marker))

    # == Generate visualizations and report =================================
    print("\nGenerating comparison visualization...")
    plot_comparison(r5_results, previous, OUTPUT_DIR)

    dataset_info = {
        "n_original": n_orig,
        "n_copies": N_AUGMENTED_COPIES,
        "n_augmented": n_aug,
        "n_total": n_orig + n_aug,
        "n_classes": n_classes,
    }

    print("\nGenerating report...")
    generate_report(r5_results, previous, dataset_info, aug_info, OUTPUT_DIR)

    print("\n" + "=" * 60)
    print("  ROUND 5 COMPLETE")
    print("  Dataset: %d original + %d augmented = %d total" %
          (n_orig, n_aug, n_orig + n_aug))
    print("=" * 60)


if __name__ == "__main__":
    main()
