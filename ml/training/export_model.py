"""
Export Pure CNN model weights for Rust integration.

Produces three outputs:
  1. ONNX format  — ml/training/round_01/pure_cnn.onnx
  2. Raw numpy arrays — ml/training/round_01/weights/*.npy + manifest.json
  3. Model card    — ml/training/round_01/MODEL_CARD.md

Usage:
    cd <project_root>
    source ml/venv/bin/activate
    pip install onnx          # if not already installed
    python ml/training/export_model.py
"""

import json
import os
import sys
import textwrap

import numpy as np
import torch
import torch.nn as nn

# ---------------------------------------------------------------------------
# Paths
# ---------------------------------------------------------------------------

SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))
PROJECT_ROOT = os.path.abspath(os.path.join(SCRIPT_DIR, "../.."))
ROUND_DIR = os.path.join(SCRIPT_DIR, "round_01")
CHECKPOINT = os.path.join(ROUND_DIR, "pure_cnn.pt")
ONNX_PATH = os.path.join(ROUND_DIR, "pure_cnn.onnx")
WEIGHTS_DIR = os.path.join(ROUND_DIR, "weights")
MODEL_CARD_PATH = os.path.join(ROUND_DIR, "MODEL_CARD.md")

# Model input shape
BATCH = 1
CHANNELS = 1
N_MELS = 64
TIME_FRAMES = 94
INPUT_SHAPE = (BATCH, CHANNELS, N_MELS, TIME_FRAMES)

# ---------------------------------------------------------------------------
# Model Architecture — must match train.py exactly
# ---------------------------------------------------------------------------


class PureCNN(nn.Module):
    """Deeper ConvNet with global average pooling, no dense bottleneck."""

    def __init__(self, n_classes, input_shape=None):
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


# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------


def count_parameters(model):
    """Return total and trainable parameter counts."""
    total = sum(p.numel() for p in model.parameters())
    trainable = sum(p.numel() for p in model.parameters() if p.requires_grad)
    return total, trainable


def layer_summary(model):
    """Generate a list of (name, module_type, output_shape_str, param_count) tuples."""
    summary = []
    dummy = torch.randn(INPUT_SHAPE)
    hooks = []
    shapes = {}

    def hook_fn(name):
        def hook(module, inp, out):
            if isinstance(out, torch.Tensor):
                shapes[name] = list(out.shape)
        return hook

    for name, mod in model.named_modules():
        if name == "":
            continue
        h = mod.register_forward_hook(hook_fn(name))
        hooks.append(h)

    with torch.no_grad():
        model(dummy)

    for h in hooks:
        h.remove()

    for name, mod in model.named_modules():
        if name == "":
            continue
        n_params = sum(p.numel() for p in mod.parameters(recurse=False))
        shape_str = str(shapes.get(name, ""))
        summary.append((name, mod.__class__.__name__, shape_str, n_params))

    return summary


# ---------------------------------------------------------------------------
# 1. ONNX Export
# ---------------------------------------------------------------------------


def export_onnx(model):
    """Export model to ONNX format."""
    print("\n--- ONNX Export ---")

    dummy_input = torch.randn(INPUT_SHAPE)

    torch.onnx.export(
        model,
        dummy_input,
        ONNX_PATH,
        opset_version=18,
        input_names=["mel_spectrogram"],
        output_names=["class_logits"],
        dynamic_axes=None,  # fixed input size
    )
    print(f"  Saved: {ONNX_PATH}")
    file_size = os.path.getsize(ONNX_PATH)
    print(f"  Size:  {file_size / 1024:.1f} KB")

    # Verify with onnx library
    try:
        import onnx
        onnx_model = onnx.load(ONNX_PATH)
        onnx.checker.check_model(onnx_model)
        print("  ONNX model validation: PASSED")
    except ImportError:
        print("  WARNING: onnx package not installed, skipping validation")
        print("  Install with: pip install onnx")

    # Verify output shape with onnxruntime if available
    try:
        import onnxruntime as ort
        sess = ort.InferenceSession(ONNX_PATH)
        result = sess.run(None, {"mel_spectrogram": dummy_input.numpy()})
        print(f"  Output shape: {result[0].shape}")

        # Cross-check with PyTorch output
        with torch.no_grad():
            pt_out = model(dummy_input).numpy()
        max_diff = np.max(np.abs(result[0] - pt_out))
        print(f"  Max diff vs PyTorch: {max_diff:.2e}")
    except ImportError:
        print("  NOTE: onnxruntime not installed, skipping inference verification")
        print("  Install with: pip install onnxruntime")

    return True


# ---------------------------------------------------------------------------
# 2. Raw Numpy Weights
# ---------------------------------------------------------------------------


def export_numpy_weights(model):
    """Export all model parameters as individual .npy files with a manifest."""
    print("\n--- Numpy Weights Export ---")

    os.makedirs(WEIGHTS_DIR, exist_ok=True)

    state_dict = model.state_dict()
    manifest = {
        "model": "PureCNN",
        "n_classes": 138,
        "input_shape": list(INPUT_SHAPE),
        "layers": [],
    }

    for name, tensor in state_dict.items():
        # Skip num_batches_tracked — not needed for inference
        if "num_batches_tracked" in name:
            continue

        filename = f"{name}.npy"
        filepath = os.path.join(WEIGHTS_DIR, filename)
        arr = tensor.cpu().numpy()
        np.save(filepath, arr)

        manifest["layers"].append({
            "name": name,
            "file": filename,
            "shape": list(arr.shape),
            "dtype": str(arr.dtype),
        })
        print(f"  {filename:45s} shape={list(arr.shape)}")

    # Write manifest
    manifest_path = os.path.join(WEIGHTS_DIR, "manifest.json")
    with open(manifest_path, "w") as f:
        json.dump(manifest, f, indent=2)
    print(f"\n  Manifest: {manifest_path}")
    print(f"  Total files: {len(manifest['layers'])} weight arrays + manifest")

    return manifest


# ---------------------------------------------------------------------------
# 3. Model Card
# ---------------------------------------------------------------------------


def generate_model_card(model, manifest):
    """Generate MODEL_CARD.md."""
    print("\n--- Model Card ---")

    total_params, trainable_params = count_parameters(model)
    summary = layer_summary(model)

    # Build layer summary table
    layer_lines = []
    layer_lines.append("| Layer | Type | Output Shape | Params |")
    layer_lines.append("|-------|------|-------------|--------|")
    for name, mod_type, shape_str, n_params in summary:
        param_str = f"{n_params:,}" if n_params > 0 else "0"
        layer_lines.append(f"| {name} | {mod_type} | {shape_str} | {param_str} |")
    layer_table = "\n".join(layer_lines)

    # Build weight file listing
    weight_lines = []
    for entry in manifest["layers"]:
        weight_lines.append(f"  - `{entry['file']}` shape {entry['shape']}")
    weight_listing = "\n".join(weight_lines)

    card = f"""# Pure CNN Guitar Classifier -- Round 1

## Model Details
- **Architecture:** Pure CNN (4 conv + GAP + linear)
- **Parameters:** {total_params:,} total ({trainable_params:,} trainable)
- **Input:** Log-mel spectrogram (1, {N_MELS}, {TIME_FRAMES}) -- 1 channel, {N_MELS} mel bins, {TIME_FRAMES} time frames
- **Output:** 138 classes (6 strings x 23 frets)
- **Accuracy:** 97.1% (80/20 split) / 96.2% (5-fold CV)
- **Training:** 60 epochs, Adam, lr=8e-4, cosine annealing, weight_decay=1e-4

## Layer Summary

{layer_table}

**Total parameters:** {total_params:,}

## Files
- `pure_cnn.pt` -- PyTorch checkpoint (state_dict + metadata)
- `pure_cnn.onnx` -- ONNX format (opset 18)
- `weights/` -- Raw numpy arrays for Rust integration

### Weight Files
{weight_listing}

## Usage

### PyTorch
```python
import torch
import torch.nn as nn

# Define the model (must match architecture)
model = PureCNN(n_classes=138)

# Load checkpoint
ckpt = torch.load("pure_cnn.pt", map_location="cpu")
model.load_state_dict(ckpt["state_dict"])
model.eval()

# Inference
mel = torch.randn(1, 1, 64, 94)  # (batch, channels, n_mels, time)
logits = model(mel)
predicted_class = logits.argmax(dim=1).item()
```

### ONNX (Python)
```python
import onnxruntime as ort
import numpy as np

sess = ort.InferenceSession("pure_cnn.onnx")
mel = np.random.randn(1, 1, 64, 94).astype(np.float32)
logits = sess.run(None, {{"mel_spectrogram": mel}})[0]
predicted_class = logits.argmax()
```

### ONNX (Rust with ort crate)
```rust
use ort::{{Session, Value}};
use ndarray::Array4;

let session = Session::builder()?.commit_from_file("pure_cnn.onnx")?;
let mel = Array4::<f32>::zeros((1, 1, 64, 94));
let outputs = session.run(ort::inputs![mel]?)?;
let logits = outputs[0].extract_tensor::<f32>()?;
```

### Raw Numpy (Rust with include_bytes!)
```rust
// Load weights at compile time
const CONV1_W: &[u8] = include_bytes!("weights/features.0.weight.npy");

// Parse .npy format and reconstruct forward pass
// See: https://docs.rs/ndarray-npy for .npy parsing
```

## Spectrogram Parameters
- Sample rate: 44100 Hz
- n_fft: 1024
- hop_length: 256
- n_mels: 64
- Power-to-dB: librosa.power_to_db(mel, ref=np.max)
- Duration: 0.5s per sample
"""

    with open(MODEL_CARD_PATH, "w") as f:
        f.write(card)
    print(f"  Saved: {MODEL_CARD_PATH}")

    return True


# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------


def main():
    print("=" * 60)
    print("  Pure CNN Model Export")
    print("=" * 60)

    # Load checkpoint
    print(f"\nLoading checkpoint: {CHECKPOINT}")
    ckpt = torch.load(CHECKPOINT, map_location="cpu", weights_only=False)
    n_classes = ckpt["n_classes"]
    accuracy = ckpt["accuracy"]
    print(f"  n_classes: {n_classes}")
    print(f"  accuracy:  {accuracy:.4f}")
    print(f"  round:     {ckpt['round']}")

    # Build model and load weights
    model = PureCNN(n_classes=n_classes)
    model.load_state_dict(ckpt["state_dict"])
    model.eval()

    total_params, _ = count_parameters(model)
    print(f"  parameters: {total_params:,}")

    # Verify forward pass
    dummy = torch.randn(INPUT_SHAPE)
    with torch.no_grad():
        out = model(dummy)
    print(f"  forward pass: input={list(dummy.shape)} -> output={list(out.shape)}")
    assert out.shape == (1, n_classes), f"Expected (1, {n_classes}), got {out.shape}"

    # Export
    export_onnx(model)
    manifest = export_numpy_weights(model)
    generate_model_card(model, manifest)

    print("\n" + "=" * 60)
    print("  Export complete!")
    print("=" * 60)
    print(f"  ONNX:    {ONNX_PATH}")
    print(f"  Weights: {WEIGHTS_DIR}/")
    print(f"  Card:    {MODEL_CARD_PATH}")
    print("=" * 60)


if __name__ == "__main__":
    main()
