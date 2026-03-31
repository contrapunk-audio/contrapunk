"""
Contrapunk Pure CNN — Inference Benchmark

Measures end-to-end latency of the guitar classification pipeline:
  1. Model inference (Pure CNN on mel-spectrogram tensor)
  2. Feature extraction (librosa mel-spectrogram on raw audio)
  3. Total pipeline latency (feature extraction + inference)

Also reports model parameter counts per layer.

Usage:
    cd <project_root>
    source ml/venv/bin/activate
    python ml/training/benchmark_inference.py
"""

import os
import time
import statistics

import numpy as np
import torch
import torch.nn as nn
import librosa


# -- Paths -----------------------------------------------------------------

SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))
MODEL_PATH = os.path.join(SCRIPT_DIR, "round_01", "pure_cnn.pt")
OUTPUT_PATH = os.path.join(SCRIPT_DIR, "round_01", "BENCHMARK.md")

# -- Feature extraction constants (must match train.py) --------------------

N_MELS = 64
N_FFT = 1024
HOP_LENGTH = 256
SAMPLE_RATE = 48000
AUDIO_DURATION = 0.5  # seconds

# -- Model Definition (must match train.py) --------------------------------


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


# -- Helpers ---------------------------------------------------------------


def load_model(path):
    """Load Pure CNN from checkpoint."""
    checkpoint = torch.load(path, map_location="cpu", weights_only=True)
    n_classes = checkpoint["n_classes"]
    model = PureCNN(n_classes)
    model.load_state_dict(checkpoint["state_dict"])
    model.eval()
    return model, n_classes


def count_parameters(model):
    """Count parameters per layer and total."""
    layer_info = []
    total_params = 0
    total_trainable = 0

    for name, param in model.named_parameters():
        n = param.numel()
        total_params += n
        if param.requires_grad:
            total_trainable += n
        layer_info.append({
            "name": name,
            "shape": str(list(param.shape)),
            "params": n,
            "trainable": param.requires_grad,
        })

    return layer_info, total_params, total_trainable


def benchmark_inference(model, input_tensor, warmup=10, iterations=1000):
    """Benchmark model inference with nanosecond precision."""
    latencies_ns = []

    # Warm up
    with torch.no_grad():
        for _ in range(warmup):
            _ = model(input_tensor)

    # Timed runs
    with torch.no_grad():
        for _ in range(iterations):
            t0 = time.perf_counter_ns()
            _ = model(input_tensor)
            t1 = time.perf_counter_ns()
            latencies_ns.append(t1 - t0)

    return latencies_ns


def benchmark_feature_extraction(iterations=200):
    """Benchmark librosa mel-spectrogram extraction on a 0.5s audio buffer."""
    # Simulate a 0.5s audio buffer at 48kHz
    audio = np.random.randn(int(SAMPLE_RATE * AUDIO_DURATION)).astype(np.float32)
    latencies_ns = []

    # Warm up (librosa has some one-time overhead)
    for _ in range(5):
        mel = librosa.feature.melspectrogram(
            y=audio, sr=SAMPLE_RATE,
            n_mels=N_MELS, n_fft=N_FFT, hop_length=HOP_LENGTH,
        )
        _ = librosa.power_to_db(mel, ref=np.max)

    # Timed runs
    for _ in range(iterations):
        t0 = time.perf_counter_ns()
        mel = librosa.feature.melspectrogram(
            y=audio, sr=SAMPLE_RATE,
            n_mels=N_MELS, n_fft=N_FFT, hop_length=HOP_LENGTH,
        )
        _ = librosa.power_to_db(mel, ref=np.max)
        t1 = time.perf_counter_ns()
        latencies_ns.append(t1 - t0)

    return latencies_ns


def ns_to_ms(ns):
    """Convert nanoseconds to milliseconds."""
    return ns / 1_000_000


def compute_stats(latencies_ns):
    """Compute statistics from a list of nanosecond latencies."""
    sorted_lat = sorted(latencies_ns)
    n = len(sorted_lat)
    return {
        "mean_ms": ns_to_ms(statistics.mean(sorted_lat)),
        "median_ms": ns_to_ms(statistics.median(sorted_lat)),
        "min_ms": ns_to_ms(sorted_lat[0]),
        "max_ms": ns_to_ms(sorted_lat[-1]),
        "p95_ms": ns_to_ms(sorted_lat[int(n * 0.95)]),
        "p99_ms": ns_to_ms(sorted_lat[int(n * 0.99)]),
        "stdev_ms": ns_to_ms(statistics.stdev(sorted_lat)) if n > 1 else 0.0,
    }


def format_stat(val):
    """Format a millisecond value with appropriate precision."""
    if val < 0.01:
        return f"{val:.4f}"
    elif val < 1.0:
        return f"{val:.3f}"
    else:
        return f"{val:.2f}"


# -- Main ------------------------------------------------------------------


def main():
    print("=" * 60)
    print("  Pure CNN Inference Benchmark")
    print("=" * 60)

    # Load model
    print(f"\nLoading model from {MODEL_PATH}...")
    model, n_classes = load_model(MODEL_PATH)
    print(f"  Classes: {n_classes}")

    # Count parameters
    print("\n--- Parameter Count ---")
    layer_info, total_params, total_trainable = count_parameters(model)
    for layer in layer_info:
        print(f"  {layer['name']:40s}  {layer['shape']:20s}  {layer['params']:>8,}")
    print(f"  {'TOTAL':40s}  {'':20s}  {total_params:>8,}")
    print(f"  Trainable: {total_trainable:,}")

    # Prepare input
    input_tensor = torch.randn(1, 1, 64, 94)
    print(f"\nInput shape: {list(input_tensor.shape)}")

    # Benchmark inference
    print("\n--- Model Inference Benchmark ---")
    print("  Warming up (10 passes)...")
    print("  Benchmarking (1000 passes)...")
    inference_latencies = benchmark_inference(
        model, input_tensor, warmup=10, iterations=1000,
    )
    inf_stats = compute_stats(inference_latencies)

    throughput = 1000.0 / inf_stats["mean_ms"]  # inferences per second

    print(f"  Mean:    {format_stat(inf_stats['mean_ms'])} ms")
    print(f"  Median:  {format_stat(inf_stats['median_ms'])} ms")
    print(f"  Min:     {format_stat(inf_stats['min_ms'])} ms")
    print(f"  Max:     {format_stat(inf_stats['max_ms'])} ms")
    print(f"  P95:     {format_stat(inf_stats['p95_ms'])} ms")
    print(f"  P99:     {format_stat(inf_stats['p99_ms'])} ms")
    print(f"  Throughput: {throughput:,.0f} inferences/sec")

    # Benchmark feature extraction
    print("\n--- Feature Extraction Benchmark ---")
    print("  Warming up (5 passes)...")
    print("  Benchmarking (200 passes)...")
    feat_latencies = benchmark_feature_extraction(iterations=200)
    feat_stats = compute_stats(feat_latencies)

    print(f"  Mean:    {format_stat(feat_stats['mean_ms'])} ms")
    print(f"  Median:  {format_stat(feat_stats['median_ms'])} ms")
    print(f"  Min:     {format_stat(feat_stats['min_ms'])} ms")
    print(f"  Max:     {format_stat(feat_stats['max_ms'])} ms")
    print(f"  P95:     {format_stat(feat_stats['p95_ms'])} ms")
    print(f"  P99:     {format_stat(feat_stats['p99_ms'])} ms")

    # Total pipeline
    total_mean = inf_stats["mean_ms"] + feat_stats["mean_ms"]
    total_p95 = inf_stats["p95_ms"] + feat_stats["p95_ms"]
    total_p99 = inf_stats["p99_ms"] + feat_stats["p99_ms"]

    print("\n--- Total Pipeline Latency ---")
    print(
        f"  Mean:  {format_stat(total_mean)} ms"
        f"  (feature: {format_stat(feat_stats['mean_ms'])}"
        f" + inference: {format_stat(inf_stats['mean_ms'])})"
    )
    print(f"  P95:   {format_stat(total_p95)} ms")
    print(f"  P99:   {format_stat(total_p99)} ms")

    # Verdict
    target_ms = 10.0
    ideal_ms = 5.0
    if total_p99 < ideal_ms:
        verdict = "PASS -- well under 5ms ideal target"
    elif total_p99 < target_ms:
        verdict = "PASS -- under 10ms target (but above 5ms ideal)"
    else:
        verdict = (
            f"FAIL -- P99 total {format_stat(total_p99)}ms exceeds 10ms target"
        )

    print(f"\n  Verdict: {verdict}")

    # -- Generate BENCHMARK.md -----------------------------------------------
    print(f"\nWriting results to {OUTPUT_PATH}...")

    lines = [
        "# Pure CNN Inference Benchmark",
        "",
        f"**Date:** {time.strftime('%Y-%m-%d %H:%M')}",
        "**Model:** Pure CNN (4 conv layers + GAP + linear)",
        f"**Classes:** {n_classes}",
        "**Input shape:** (1, 1, 64, 94) -- single mel-spectrogram",
        "**Device:** CPU",
        "",
        "## Model Inference (1000 iterations, 10 warmup)",
        "",
        "| Metric | Value |",
        "|--------|-------|",
        f"| Mean | {format_stat(inf_stats['mean_ms'])} ms |",
        f"| Median | {format_stat(inf_stats['median_ms'])} ms |",
        f"| Min | {format_stat(inf_stats['min_ms'])} ms |",
        f"| Max | {format_stat(inf_stats['max_ms'])} ms |",
        f"| P95 | {format_stat(inf_stats['p95_ms'])} ms |",
        f"| P99 | {format_stat(inf_stats['p99_ms'])} ms |",
        f"| Stdev | {format_stat(inf_stats['stdev_ms'])} ms |",
        f"| Throughput | {throughput:,.0f} inferences/sec |",
        "",
        "## Feature Extraction (200 iterations, 5 warmup)",
        "",
        f"librosa mel-spectrogram on 0.5s buffer @ {SAMPLE_RATE} Hz"
        f" (n_mels={N_MELS}, n_fft={N_FFT}, hop={HOP_LENGTH})",
        "",
        "| Metric | Value |",
        "|--------|-------|",
        f"| Mean | {format_stat(feat_stats['mean_ms'])} ms |",
        f"| Median | {format_stat(feat_stats['median_ms'])} ms |",
        f"| Min | {format_stat(feat_stats['min_ms'])} ms |",
        f"| Max | {format_stat(feat_stats['max_ms'])} ms |",
        f"| P95 | {format_stat(feat_stats['p95_ms'])} ms |",
        f"| P99 | {format_stat(feat_stats['p99_ms'])} ms |",
        "",
        "## Total Pipeline Latency",
        "",
        "| Stage | Mean | P95 | P99 |",
        "|-------|------|-----|-----|",
        f"| Feature extraction | {format_stat(feat_stats['mean_ms'])} ms"
        f" | {format_stat(feat_stats['p95_ms'])} ms"
        f" | {format_stat(feat_stats['p99_ms'])} ms |",
        f"| Model inference | {format_stat(inf_stats['mean_ms'])} ms"
        f" | {format_stat(inf_stats['p95_ms'])} ms"
        f" | {format_stat(inf_stats['p99_ms'])} ms |",
        f"| **Total** | **{format_stat(total_mean)} ms**"
        f" | **{format_stat(total_p95)} ms**"
        f" | **{format_stat(total_p99)} ms** |",
        "",
        f"**Target:** <10ms (ideal <5ms)",
        f"**Verdict:** {verdict}",
        "",
        "## Model Parameters",
        "",
        "| Layer | Shape | Parameters |",
        "|-------|-------|------------|",
    ]

    for layer in layer_info:
        lines.append(
            f"| {layer['name']} | {layer['shape']} | {layer['params']:,} |"
        )

    lines.extend([
        f"| **Total** | | **{total_params:,}** |",
        f"| Trainable | | {total_trainable:,} |",
        "",
        "*Generated by `ml/training/benchmark_inference.py`*",
    ])

    with open(OUTPUT_PATH, "w") as f:
        f.write("\n".join(lines) + "\n")

    print(f"  Saved: {OUTPUT_PATH}")
    print("\nDone.")


if __name__ == "__main__":
    main()
