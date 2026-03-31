#!/usr/bin/env python3
"""Export numpy weight files into a single binary file for Rust inference.

Reads all .npy files listed in manifest.json and concatenates them into
a single binary file with a simple header format:

    [4 bytes: magic "GCNN"]
    [4 bytes: u32 version = 1]
    [4 bytes: u32 n_classes]
    [4 bytes: u32 n_layers]
    For each layer:
        [4 bytes: u32 name_len]
        [name_len bytes: layer name as UTF-8]
        [4 bytes: u32 n_elements]
        [n_elements * 4 bytes: f32 values in row-major order]

Usage:
    python ml/training/export_weights_bin.py
"""

import json
import struct
import numpy as np
from pathlib import Path


def main():
    weights_dir = Path(__file__).parent / "round_01" / "weights"
    manifest_path = weights_dir / "manifest.json"

    with open(manifest_path, "r") as f:
        manifest = json.load(f)

    n_classes = manifest["n_classes"]
    layers = manifest["layers"]
    n_layers = len(layers)

    out_path = weights_dir / "guitar_classifier.bin"

    with open(out_path, "wb") as out:
        # Header
        out.write(b"GCNN")                              # magic
        out.write(struct.pack("<I", 1))                  # version
        out.write(struct.pack("<I", n_classes))          # n_classes
        out.write(struct.pack("<I", n_layers))           # n_layers

        total_params = 0
        for layer in layers:
            name = layer["name"]
            npy_path = weights_dir / layer["file"]
            arr = np.load(npy_path).astype(np.float32).flatten()

            name_bytes = name.encode("utf-8")
            out.write(struct.pack("<I", len(name_bytes)))
            out.write(name_bytes)
            out.write(struct.pack("<I", len(arr)))
            out.write(arr.tobytes())

            total_params += len(arr)
            print(f"  {name:40s}  shape={str(layer['shape']):20s}  elements={len(arr)}")

    file_size = out_path.stat().st_size
    print(f"\nWrote {out_path}")
    print(f"  Total layers:     {n_layers}")
    print(f"  Total parameters: {total_params}")
    print(f"  File size:        {file_size:,} bytes ({file_size / 1024:.1f} KB)")


if __name__ == "__main__":
    main()
