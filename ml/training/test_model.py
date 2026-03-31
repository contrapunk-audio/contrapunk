"""
Quick interactive model tester.

Trains the Pure CNN on 80% of data, then lets you:
  - Test on held-out samples
  - See top-3 predictions with confidence
  - Play the audio sample (macOS)
  - View per-string accuracy

Usage:
    cd <project_root>
    source ml/venv/bin/activate
    python ml/training/test_model.py
"""

import sys
import os
import time
import random
import numpy as np

import torch
import torch.nn as nn
import torch.optim as optim
from torch.utils.data import DataLoader, SubsetRandomSampler
from sklearn.model_selection import train_test_split

import librosa

PROJECT_ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), "../.."))
DATASET_PATH = os.path.join(PROJECT_ROOT, "guitar_training_data.msgpack")
MODEL_PATH = os.path.join(os.path.dirname(__file__), "round_01", "pure_cnn.pt")

sys.path.insert(0, PROJECT_ROOT)
from ml.loader import GuitarDataset

# -- Feature extraction (same as train.py) ---------------------------------

N_MELS = 64
N_FFT = 1024
HOP_LENGTH = 256


def extract_mel(audio, sr):
    mel = librosa.feature.melspectrogram(
        y=audio, sr=sr, n_mels=N_MELS, n_fft=N_FFT, hop_length=HOP_LENGTH,
    )
    return librosa.power_to_db(mel, ref=np.max)


def prepare_features(ds):
    print("Extracting mel-spectrograms...")
    raw = []
    labels = []
    for i, s in enumerate(ds.samples):
        raw.append(extract_mel(s.audio_np, s.sample_rate))
        labels.append(s.class_id - 1)
        if (i + 1) % 500 == 0:
            print(f"  {i+1}/{len(ds.samples)}")

    max_t = max(f.shape[1] for f in raw)
    features = []
    for f in raw:
        if f.shape[1] < max_t:
            f = np.pad(f, ((0, 0), (0, max_t - f.shape[1])),
                       mode="constant", constant_values=f.min())
        features.append(f)

    print(f"  Done: {len(features)} spectrograms, shape {features[0].shape}")
    return features, np.array(labels)


# -- Pure CNN (same as train.py) -------------------------------------------

class PureCNN(nn.Module):
    def __init__(self, n_classes):
        super().__init__()
        self.features = nn.Sequential(
            nn.Conv2d(1, 32, 3, padding=1), nn.BatchNorm2d(32), nn.ReLU(), nn.MaxPool2d(2),
            nn.Conv2d(32, 64, 3, padding=1), nn.BatchNorm2d(64), nn.ReLU(), nn.MaxPool2d(2),
            nn.Conv2d(64, 128, 3, padding=1), nn.BatchNorm2d(128), nn.ReLU(), nn.MaxPool2d(2),
            nn.Conv2d(128, 256, 3, padding=1), nn.BatchNorm2d(256), nn.ReLU(), nn.AdaptiveAvgPool2d((1, 1)),
        )
        self.classifier = nn.Sequential(nn.Flatten(), nn.Dropout(0.4), nn.Linear(256, n_classes))

    def forward(self, x):
        return self.classifier(self.features(x))


# -- Training --------------------------------------------------------------

def train_and_save(features, labels, n_classes):
    print("\nTraining Pure CNN (single 80/20 split)...")

    train_idx, val_idx = train_test_split(
        range(len(labels)), test_size=0.2, stratify=labels, random_state=42
    )

    X = torch.tensor(
        np.array([f[np.newaxis, ...] for f in features]), dtype=torch.float32
    )
    y = torch.tensor(labels, dtype=torch.long)
    dataset = torch.utils.data.TensorDataset(X, y)

    train_loader = DataLoader(dataset, batch_size=32, sampler=SubsetRandomSampler(train_idx))
    model = PureCNN(n_classes)
    criterion = nn.CrossEntropyLoss()
    optimizer = optim.Adam(model.parameters(), lr=8e-4, weight_decay=1e-4)
    scheduler = optim.lr_scheduler.CosineAnnealingLR(optimizer, T_max=60)

    best_acc = 0
    best_state = None

    t0 = time.time()
    for epoch in range(60):
        model.train()
        for xb, yb in train_loader:
            optimizer.zero_grad()
            loss = criterion(model(xb), yb)
            loss.backward()
            optimizer.step()
        scheduler.step()

        model.eval_mode = False
        model.eval()
        with torch.no_grad():
            val_X = X[val_idx]
            val_y = y[val_idx]
            preds = model(val_X).argmax(1)
            acc = (preds == val_y).float().mean().item()

        if acc > best_acc:
            best_acc = acc
            best_state = {k: v.clone() for k, v in model.state_dict().items()}

        if (epoch + 1) % 10 == 0:
            print(f"  Epoch {epoch+1}: val_acc={acc:.4f}")

    elapsed = time.time() - t0
    model.load_state_dict(best_state)
    print(f"  Best: {best_acc:.1%} ({elapsed:.0f}s)")

    torch.save({"state_dict": best_state, "n_classes": n_classes}, MODEL_PATH)
    print(f"  Saved: {MODEL_PATH}")

    return model, val_idx


def load_model(n_classes):
    checkpoint = torch.load(MODEL_PATH, weights_only=True)
    model = PureCNN(checkpoint["n_classes"])
    model.load_state_dict(checkpoint["state_dict"])
    model.eval()
    return model


# -- Interactive testing ---------------------------------------------------

STRING_NAMES = ["E2 (low)", "A2", "D3", "G3", "B3", "E4 (high)"]


def predict_sample(model, features, idx, ds):
    sample = ds.samples[idx]
    x = torch.tensor(features[idx][np.newaxis, np.newaxis, ...], dtype=torch.float32)

    with torch.no_grad():
        logits = model(x)
        probs = torch.softmax(logits, dim=1)[0]
        top3_prob, top3_idx = probs.topk(3)

    true_class = sample.class_id - 1
    pred_class = top3_idx[0].item()

    class_to_label = {}
    for s in ds.samples:
        class_to_label[s.class_id - 1] = s.label

    correct = pred_class == true_class
    mark = "CORRECT" if correct else "WRONG"

    print(f"\n  Sample #{idx}: {sample.label}")
    print(f"  String: {STRING_NAMES[sample.string_idx]}, Fret: {sample.fret}")
    print(f"  RMS: {sample.rms:.4f}, Peak: {sample.peak:.4f}")
    print(f"  ---")
    print(f"  Prediction: {class_to_label.get(pred_class, '?')} ({mark})")
    print(f"  Top 3:")
    for i in range(3):
        ci = top3_idx[i].item()
        p = top3_prob[i].item()
        label = class_to_label.get(ci, f"class_{ci}")
        marker = " <--" if ci == true_class else ""
        print(f"    {i+1}. {label:25s} {p:6.1%}{marker}")

    return correct


def interactive_loop(model, features, ds, val_idx):
    print("\n" + "=" * 50)
    print("  Interactive Model Tester")
    print("=" * 50)
    print("  Commands:")
    print("    <number>  - test specific sample index")
    print("    r         - test random held-out sample")
    print("    s <0-5>   - test random sample from string")
    print("    batch     - test 20 random held-out samples")
    print("    play <n>  - play sample audio (macOS)")
    print("    q         - quit")
    print(f"  Held-out samples: {len(val_idx)}")
    print()

    while True:
        try:
            cmd = input("  > ").strip().lower()
        except (EOFError, KeyboardInterrupt):
            break

        if cmd == "q":
            break
        elif cmd == "r":
            idx = random.choice(val_idx)
            predict_sample(model, features, idx, ds)
        elif cmd.startswith("s "):
            parts = cmd.split()
            if len(parts) == 2:
                try:
                    si = int(parts[1])
                    candidates = [i for i in val_idx if ds.samples[i].string_idx == si]
                    if candidates:
                        idx = random.choice(candidates)
                        predict_sample(model, features, idx, ds)
                    else:
                        print(f"  No held-out samples for string {si}")
                except ValueError:
                    print("  Usage: s <0-5>")
            else:
                print("  Usage: s <0-5>")
        elif cmd == "batch":
            batch = random.sample(list(val_idx), min(20, len(val_idx)))
            correct = sum(predict_sample(model, features, i, ds) for i in batch)
            print(f"\n  Batch: {correct}/{len(batch)} ({correct/len(batch):.1%})")
        elif cmd.startswith("play "):
            parts = cmd.split()
            if len(parts) == 2:
                try:
                    idx = int(parts[1])
                    ds.play(idx)
                except ValueError:
                    print("  Usage: play <index>")
            else:
                print("  Usage: play <index>")
        else:
            try:
                idx = int(cmd)
                if 0 <= idx < len(ds.samples):
                    predict_sample(model, features, idx, ds)
                else:
                    print(f"  Index out of range (0-{len(ds.samples)-1})")
            except ValueError:
                print("  Unknown command. Try: r, s <0-5>, batch, <number>, q")


# -- Main ------------------------------------------------------------------

def main():
    ds = GuitarDataset.load(DATASET_PATH)
    features, labels = prepare_features(ds)
    n_classes = len(np.unique(labels))

    if os.path.exists(MODEL_PATH):
        print(f"\nFound saved model: {MODEL_PATH}")
        model = load_model(n_classes)
        _, val_idx = train_test_split(
            range(len(labels)), test_size=0.2, stratify=labels, random_state=42
        )
        print(f"  Loaded Pure CNN ({n_classes} classes)")
    else:
        model, val_idx = train_and_save(features, labels, n_classes)

    interactive_loop(model, features, ds, val_idx)


if __name__ == "__main__":
    main()
