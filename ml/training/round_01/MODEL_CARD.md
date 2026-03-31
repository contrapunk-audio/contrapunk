# Pure CNN Guitar Classifier -- Round 1

## Model Details
- **Architecture:** Pure CNN (4 conv + GAP + linear)
- **Parameters:** 424,266 total (424,266 trainable)
- **Input:** Log-mel spectrogram (1, 64, 94) -- 1 channel, 64 mel bins, 94 time frames
- **Output:** 138 classes (6 strings x 23 frets)
- **Accuracy:** 97.1% (80/20 split) / 96.2% (5-fold CV)
- **Training:** 60 epochs, Adam, lr=8e-4, cosine annealing, weight_decay=1e-4

## Layer Summary

| Layer | Type | Output Shape | Params |
|-------|------|-------------|--------|
| features | Sequential | [1, 256, 1, 1] | 0 |
| features.0 | Conv2d | [1, 32, 64, 94] | 320 |
| features.1 | BatchNorm2d | [1, 32, 64, 94] | 64 |
| features.2 | ReLU | [1, 32, 64, 94] | 0 |
| features.3 | MaxPool2d | [1, 32, 32, 47] | 0 |
| features.4 | Conv2d | [1, 64, 32, 47] | 18,496 |
| features.5 | BatchNorm2d | [1, 64, 32, 47] | 128 |
| features.6 | ReLU | [1, 64, 32, 47] | 0 |
| features.7 | MaxPool2d | [1, 64, 16, 23] | 0 |
| features.8 | Conv2d | [1, 128, 16, 23] | 73,856 |
| features.9 | BatchNorm2d | [1, 128, 16, 23] | 256 |
| features.10 | ReLU | [1, 128, 16, 23] | 0 |
| features.11 | MaxPool2d | [1, 128, 8, 11] | 0 |
| features.12 | Conv2d | [1, 256, 8, 11] | 295,168 |
| features.13 | BatchNorm2d | [1, 256, 8, 11] | 512 |
| features.14 | ReLU | [1, 256, 8, 11] | 0 |
| features.15 | AdaptiveAvgPool2d | [1, 256, 1, 1] | 0 |
| classifier | Sequential | [1, 138] | 0 |
| classifier.0 | Flatten | [1, 256] | 0 |
| classifier.1 | Dropout | [1, 256] | 0 |
| classifier.2 | Linear | [1, 138] | 35,466 |

**Total parameters:** 424,266

## Files
- `pure_cnn.pt` -- PyTorch checkpoint (state_dict + metadata)
- `pure_cnn.onnx` -- ONNX format (opset 18)
- `weights/` -- Raw numpy arrays for Rust integration

### Weight Files
  - `features.0.weight.npy` shape [32, 1, 3, 3]
  - `features.0.bias.npy` shape [32]
  - `features.1.weight.npy` shape [32]
  - `features.1.bias.npy` shape [32]
  - `features.1.running_mean.npy` shape [32]
  - `features.1.running_var.npy` shape [32]
  - `features.4.weight.npy` shape [64, 32, 3, 3]
  - `features.4.bias.npy` shape [64]
  - `features.5.weight.npy` shape [64]
  - `features.5.bias.npy` shape [64]
  - `features.5.running_mean.npy` shape [64]
  - `features.5.running_var.npy` shape [64]
  - `features.8.weight.npy` shape [128, 64, 3, 3]
  - `features.8.bias.npy` shape [128]
  - `features.9.weight.npy` shape [128]
  - `features.9.bias.npy` shape [128]
  - `features.9.running_mean.npy` shape [128]
  - `features.9.running_var.npy` shape [128]
  - `features.12.weight.npy` shape [256, 128, 3, 3]
  - `features.12.bias.npy` shape [256]
  - `features.13.weight.npy` shape [256]
  - `features.13.bias.npy` shape [256]
  - `features.13.running_mean.npy` shape [256]
  - `features.13.running_var.npy` shape [256]
  - `classifier.2.weight.npy` shape [138, 256]
  - `classifier.2.bias.npy` shape [138]

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
logits = sess.run(None, {"mel_spectrogram": mel})[0]
predicted_class = logits.argmax()
```

### ONNX (Rust with ort crate)
```rust
use ort::{Session, Value};
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
