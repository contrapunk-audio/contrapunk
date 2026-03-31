"""
Export training metrics as JSON for the diary's interactive web visualizations.

Produces three JSON files in ui/static/training/round_01/:
  1. confusion_data.json   -- full 138x138 + aggregated 6x6 confusion matrices
  2. fret_accuracy.json    -- 6x23 per-string x per-fret accuracy grid
  3. training_meta.json    -- dataset metadata and evaluation summary

For Random Forest, the script re-runs 5-fold stratified CV (fast, ~25s total)
to produce proper out-of-fold predictions. For the CNN models, it loads the
saved 80/20-split checkpoints and runs inference on all samples. The CNN
matrices therefore include training-set predictions and are optimistic; this
is noted in the metadata.

Usage:
    cd <project_root>
    source ml/venv/bin/activate
    python ml/training/export_training_data.py
"""

import sys
import os
import json
import time
import warnings

import numpy as np

warnings.filterwarnings("ignore", category=FutureWarning)

# -- Paths -----------------------------------------------------------------

PROJECT_ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), "../.."))
DATASET_PATH = os.path.join(PROJECT_ROOT, "guitar_training_data.msgpack")
MODEL_DIR = os.path.join(os.path.dirname(__file__), "round_01")
OUTPUT_DIR = os.path.join(PROJECT_ROOT, "ui", "static", "training", "round_01")

sys.path.insert(0, PROJECT_ROOT)
from ml.loader import GuitarDataset

# -- Feature extraction (identical to train.py) ----------------------------

import librosa

N_MELS = 64
N_FFT = 1024
HOP_LENGTH = 256

STRING_NAMES = ["E2", "A2", "D3", "G3", "B3", "E4"]
N_STRINGS = 6
N_FRETS = 23


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
    metadata = []  # (string_idx, fret) per sample

    for i, sample in enumerate(ds.samples):
        audio = sample.audio_np
        sr = sample.sample_rate
        mel = extract_mel_spectrogram(audio, sr)
        raw_features.append(mel)
        labels.append(sample.class_id - 1)  # zero-index
        metadata.append((sample.string_idx, sample.fret))
        if (i + 1) % 200 == 0:
            print(f"  {i + 1}/{len(ds.samples)} samples processed")

    # Pad/truncate to uniform time dimension
    max_time = max(f.shape[1] for f in raw_features)
    features = []
    for f in raw_features:
        if f.shape[1] < max_time:
            pad_width = max_time - f.shape[1]
            f = np.pad(f, ((0, 0), (0, pad_width)), mode="constant",
                       constant_values=f.min())
        elif f.shape[1] > max_time:
            f = f[:, :max_time]
        features.append(f)

    print(f"  All {len(features)} samples processed")
    print(f"  Spectrogram shape: {features[0].shape}")
    return features, np.array(labels), metadata


# -- Class label generation ------------------------------------------------

def make_class_labels(n_strings, n_frets):
    """Generate ordered class labels matching class_id assignment."""
    labels = []
    for s in range(n_strings):
        for f in range(n_frets):
            labels.append(f"string_{s}_fret_{f}")
    return labels


# -- Random Forest: 5-fold CV predictions ---------------------------------

def rf_cross_val_predictions(features, labels):
    """Re-run 5-fold stratified CV for Random Forest, return out-of-fold preds."""
    from sklearn.ensemble import RandomForestClassifier
    from sklearn.model_selection import StratifiedKFold

    print("\n  RF: Running 5-fold stratified CV...")
    X_flat = np.array([f.flatten() for f in features])
    skf = StratifiedKFold(n_splits=5, shuffle=True, random_state=42)

    all_preds = np.zeros(len(labels), dtype=int)
    t0 = time.time()

    for fold, (train_idx, val_idx) in enumerate(skf.split(X_flat, labels)):
        rf = RandomForestClassifier(
            n_estimators=200, max_depth=None,
            min_samples_leaf=1, n_jobs=-1, random_state=42,
        )
        rf.fit(X_flat[train_idx], labels[train_idx])
        all_preds[val_idx] = rf.predict(X_flat[val_idx])
        print(f"    Fold {fold + 1}/5 done")

    elapsed = time.time() - t0
    print(f"  RF: 5-fold CV done in {elapsed:.1f}s")
    return all_preds


# -- CNN: inference with saved weights -------------------------------------

def cnn_inference_all(features, labels, model_path, model_class_name):
    """Load a saved CNN checkpoint and predict all samples."""
    import torch
    import torch.nn as nn

    n_classes = len(np.unique(labels))

    if model_class_name == "HybridCNN":
        model = _build_hybrid_cnn(n_classes)
    else:
        model = _build_pure_cnn(n_classes)

    checkpoint = torch.load(model_path, map_location="cpu", weights_only=False)
    model.load_state_dict(checkpoint["state_dict"])

    # Set to inference mode
    for module in model.modules():
        if hasattr(module, "training"):
            module.training = False

    # Build tensor dataset
    X = torch.tensor(
        np.array([f[np.newaxis, ...] for f in features]),
        dtype=torch.float32,
    )

    all_preds = np.zeros(len(labels), dtype=int)
    batch_size = 64

    with torch.no_grad():
        for start in range(0, len(labels), batch_size):
            end = min(start + batch_size, len(labels))
            outputs = model(X[start:end])
            _, predicted = outputs.max(1)
            all_preds[start:end] = predicted.cpu().numpy()

    return all_preds


def _build_hybrid_cnn(n_classes):
    import torch.nn as nn

    class HybridCNN(nn.Module):
        def __init__(self):
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

    return HybridCNN()


def _build_pure_cnn(n_classes):
    import torch.nn as nn

    class PureCNN(nn.Module):
        def __init__(self):
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

    return PureCNN()


# -- Build confusion matrix ------------------------------------------------

def build_confusion_matrix(y_true, y_pred, n_classes):
    """Build NxN confusion matrix (integer counts)."""
    cm = np.zeros((n_classes, n_classes), dtype=int)
    for t, p in zip(y_true, y_pred):
        cm[t][p] += 1
    return cm


def aggregate_string_confusion(cm, n_strings, n_frets):
    """Aggregate full confusion matrix into 6x6 string-level matrix."""
    scm = np.zeros((n_strings, n_strings), dtype=int)
    for true_s in range(n_strings):
        for pred_s in range(n_strings):
            block = cm[
                true_s * n_frets : (true_s + 1) * n_frets,
                pred_s * n_frets : (pred_s + 1) * n_frets,
            ]
            scm[true_s][pred_s] = int(block.sum())
    return scm


# -- Build fret accuracy grid ---------------------------------------------

def build_fret_accuracy(y_true, y_pred, metadata, n_strings, n_frets):
    """Build 6x23 accuracy grid and sample counts."""
    correct = np.zeros((n_strings, n_frets))
    total = np.zeros((n_strings, n_frets))

    for i in range(len(y_true)):
        si, fret = metadata[i]
        total[si][fret] += 1
        if y_true[i] == y_pred[i]:
            correct[si][fret] += 1

    with np.errstate(divide="ignore", invalid="ignore"):
        acc_grid = np.where(total > 0, correct / total, 0.0)

    return acc_grid, total.astype(int)


# -- Main ------------------------------------------------------------------

def _load_cv_results():
    """Load the original 5-fold CV results from results.json if available."""
    results_path = os.path.join(MODEL_DIR, "results.json")
    if os.path.exists(results_path):
        with open(results_path) as f:
            results = json.load(f)
        return [
            {
                "name": r["name"],
                "accuracy": r["accuracy"],
                "per_string": r["per_string"],
            }
            for r in results
        ]
    return None


def main():
    print("=" * 60)
    print("  Export Training Data as JSON")
    print("=" * 60)

    os.makedirs(OUTPUT_DIR, exist_ok=True)

    # Load dataset and extract features
    ds = GuitarDataset.load(DATASET_PATH)
    features, labels, metadata = prepare_dataset(ds)

    n_classes = len(np.unique(labels))
    n_samples = len(labels)
    class_labels = make_class_labels(N_STRINGS, N_FRETS)

    assert n_classes <= len(class_labels), (
        f"Found {n_classes} classes but label list has {len(class_labels)}"
    )

    print(f"\n  Classes: {n_classes}, Samples: {n_samples}")

    # -- Collect predictions for each model --------------------------------

    models_preds = {}

    # 1. Random Forest -- proper 5-fold CV out-of-fold predictions
    print("\n" + "-" * 60)
    print("  Random Forest (5-fold CV)")
    print("-" * 60)
    models_preds["random_forest"] = {
        "preds": rf_cross_val_predictions(features, labels),
        "eval_method": "5-fold stratified CV (out-of-fold predictions)",
    }

    # 2. Hybrid CNN -- saved model inference on all samples
    print("\n" + "-" * 60)
    print("  Hybrid CNN (saved model inference)")
    print("-" * 60)
    hybrid_path = os.path.join(MODEL_DIR, "hybrid_cnn.pt")
    if os.path.exists(hybrid_path):
        models_preds["hybrid_cnn"] = {
            "preds": cnn_inference_all(features, labels, hybrid_path, "HybridCNN"),
            "eval_method": "inference on all samples (includes training data)",
        }
        acc = np.mean(models_preds["hybrid_cnn"]["preds"] == labels)
        print(f"  Hybrid CNN all-sample accuracy: {acc:.4f}")
    else:
        print(f"  WARNING: {hybrid_path} not found, skipping Hybrid CNN")

    # 3. Pure CNN -- saved model inference on all samples
    print("\n" + "-" * 60)
    print("  Pure CNN (saved model inference)")
    print("-" * 60)
    pure_path = os.path.join(MODEL_DIR, "pure_cnn.pt")
    if os.path.exists(pure_path):
        models_preds["pure_cnn"] = {
            "preds": cnn_inference_all(features, labels, pure_path, "PureCNN"),
            "eval_method": "inference on all samples (includes training data)",
        }
        acc = np.mean(models_preds["pure_cnn"]["preds"] == labels)
        print(f"  Pure CNN all-sample accuracy: {acc:.4f}")
    else:
        print(f"  WARNING: {pure_path} not found, skipping Pure CNN")

    # -- Build and export confusion_data.json ------------------------------

    print("\n" + "-" * 60)
    print("  Building confusion matrices")
    print("-" * 60)

    confusion_data = {
        "labels": class_labels[:n_classes],
        "string_names": STRING_NAMES,
        "models": {},
    }

    for model_key, data in models_preds.items():
        preds = data["preds"]
        cm = build_confusion_matrix(labels, preds, n_classes)
        scm = aggregate_string_confusion(cm, N_STRINGS, N_FRETS)

        confusion_data["models"][model_key] = {
            "matrix": cm.tolist(),
            "string_confusion": scm.tolist(),
            "eval_method": data["eval_method"],
        }

        acc = np.trace(cm) / cm.sum()
        print(f"  {model_key}: overall accuracy = {acc:.4f}")

    confusion_path = os.path.join(OUTPUT_DIR, "confusion_data.json")
    with open(confusion_path, "w") as f:
        json.dump(confusion_data, f)
    size_kb = os.path.getsize(confusion_path) / 1024
    print(f"\n  Saved: {confusion_path} ({size_kb:.1f} KB)")

    # -- Build and export fret_accuracy.json -------------------------------

    print("\n" + "-" * 60)
    print("  Building fret accuracy grids")
    print("-" * 60)

    fret_accuracy = {
        "models": {},
        "strings": STRING_NAMES,
        "frets": N_FRETS,
    }

    for model_key, data in models_preds.items():
        preds = data["preds"]
        acc_grid, counts = build_fret_accuracy(
            labels, preds, metadata, N_STRINGS, N_FRETS,
        )

        # Round accuracy to 4 decimal places for JSON size
        fret_accuracy["models"][model_key] = {
            "grid": np.round(acc_grid, 4).tolist(),
            "counts": counts.tolist(),
        }

        valid_mask = counts > 0
        if valid_mask.any():
            mean_acc = acc_grid[valid_mask].mean()
            min_acc = acc_grid[valid_mask].min()
            print(f"  {model_key}: mean fret acc = {mean_acc:.4f}, "
                  f"min = {min_acc:.4f}")

    fret_path = os.path.join(OUTPUT_DIR, "fret_accuracy.json")
    with open(fret_path, "w") as f:
        json.dump(fret_accuracy, f)
    size_kb = os.path.getsize(fret_path) / 1024
    print(f"\n  Saved: {fret_path} ({size_kb:.1f} KB)")

    # -- Build and export training_meta.json -------------------------------

    print("\n" + "-" * 60)
    print("  Building training metadata")
    print("-" * 60)

    clipped = sum(1 for s in ds.samples if s.peak > 0.99)
    silent = sum(1 for s in ds.samples if s.rms < 0.005)

    training_meta = {
        "round": 1,
        "dataset": {
            "total_samples": n_samples,
            "classes": n_classes,
            "per_class": n_samples // n_classes if n_classes > 0 else 0,
            "sample_rate": ds.samples[0].sample_rate if ds.samples else 48000,
            "duration": ds.metadata.get("sample_duration_secs", 0.5),
            "quality_issues": {
                "clipped": clipped,
                "silent": silent,
            },
        },
        "evaluation": "5-fold stratified CV",
        "feature": f"log-mel spectrogram ({N_MELS} mels, {N_FFT} FFT, {HOP_LENGTH} hop)",
        "models": {
            model_key: {
                "eval_method": data["eval_method"],
                "accuracy": float(np.mean(data["preds"] == labels)),
            }
            for model_key, data in models_preds.items()
        },
        "cv_results": _load_cv_results(),
    }

    meta_path = os.path.join(OUTPUT_DIR, "training_meta.json")
    with open(meta_path, "w") as f:
        json.dump(training_meta, f, indent=2)
    size_kb = os.path.getsize(meta_path) / 1024
    print(f"\n  Saved: {meta_path} ({size_kb:.1f} KB)")

    # -- Validate JSON files -----------------------------------------------

    print("\n" + "-" * 60)
    print("  Validating JSON files")
    print("-" * 60)

    for path in [confusion_path, fret_path, meta_path]:
        try:
            with open(path) as f:
                data = json.load(f)
            size = os.path.getsize(path)
            print(f"  OK: {os.path.basename(path)} "
                  f"({size / 1024:.1f} KB, valid JSON)")
        except json.JSONDecodeError as e:
            print(f"  FAIL: {os.path.basename(path)} -- {e}")

    print("\n" + "=" * 60)
    print("  Export complete")
    print("=" * 60)


if __name__ == "__main__":
    main()
