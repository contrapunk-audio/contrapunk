"""
Contrapunk Guitar Dataset Loader

Loads the MessagePack dataset captured by guitar_capture.rs.
The file contains a TrainingDataset struct serialized with rmp-serde.

Usage:
    python loader.py path/to/guitar_training_data.msgpack

As a library:
    from ml.loader import GuitarDataset
    ds = GuitarDataset.load("guitar_training_data.msgpack")
    ds.summary()
"""

import sys
import struct
import tempfile
import subprocess
import numpy as np

try:
    import msgpack
except ImportError:
    print("ERROR: msgpack not installed. Run: pip install msgpack")
    sys.exit(1)


# ── rmp-serde field order (Rust struct field declaration order) ───────
# rmp_serde serializes named structs as MessagePack arrays by default,
# so we must unpack by position, not by key name.

_METADATA_FIELDS = [
    "guitar_name", "strings", "frets", "plucks_per_position",
    "sample_rate", "sample_duration_secs", "device_name", "channel",
    "capture_date", "noise_categories",
]

_SAMPLE_FIELDS = [
    "audio", "label", "class_id", "string_idx", "fret",
    "expected_midi", "detected_midi", "confidence", "rms", "peak",
    "sample_rate", "pitch_validated",
    # Optional fields (may not exist in older datasets):
    "goertzel_harmonics", "harmonic_ratios", "spectral_centroid",
]


# ── GuitarSample ─────────────────────────────────────────────────────

class GuitarSample:
    """Wraps a single training sample with convenient properties."""

    __slots__ = [
        "audio", "label", "class_id", "string_idx", "fret",
        "expected_midi", "detected_midi", "confidence", "rms", "peak",
        "sample_rate", "pitch_validated",
        "goertzel_harmonics", "harmonic_ratios", "spectral_centroid",
    ]

    def __init__(self, raw):
        """
        Accepts either a dict (keyed by name) or a list/tuple (positional,
        matching rmp-serde array serialization).
        """
        if isinstance(raw, (list, tuple)):
            for i, field in enumerate(_SAMPLE_FIELDS):
                if i < len(raw):
                    value = raw[i]
                    # Decode bytes to str for label
                    if field == "label" and isinstance(value, bytes):
                        value = value.decode("utf-8", errors="replace")
                    setattr(self, field, value)
                else:
                    # Older dataset without this field
                    setattr(self, field, self._default(field))
        elif isinstance(raw, dict):
            for field in _SAMPLE_FIELDS:
                # Try both str and bytes keys (msgpack may return bytes keys)
                value = raw.get(field, raw.get(field.encode(), self._default(field)))
                if field == "label" and isinstance(value, bytes):
                    value = value.decode("utf-8", errors="replace")
                setattr(self, field, value)
        else:
            raise TypeError(f"Expected list, tuple, or dict; got {type(raw)}")

    @staticmethod
    def _default(field):
        if field in ("goertzel_harmonics", "harmonic_ratios"):
            return []
        if field == "spectral_centroid":
            return 0.0
        return None

    @property
    def is_noise(self):
        return self.string_idx == 255 or (
            isinstance(self.label, str) and self.label.startswith("noise")
        )

    @property
    def duration_secs(self):
        if self.audio and self.sample_rate:
            return len(self.audio) / self.sample_rate
        return 0.0

    @property
    def audio_np(self):
        """Return audio as a numpy float32 array."""
        return np.array(self.audio, dtype=np.float32)

    @property
    def pitch_error(self):
        """Absolute difference between expected and detected MIDI note."""
        if self.expected_midi is not None and self.detected_midi is not None:
            return abs(int(self.detected_midi) - int(self.expected_midi))
        return None

    def __repr__(self):
        dur = f"{self.duration_secs:.2f}s" if self.audio else "no audio"
        return (
            f"GuitarSample(label={self.label!r}, class_id={self.class_id}, "
            f"rms={self.rms:.4f}, peak={self.peak:.4f}, {dur})"
        )


# ── GuitarDataset ────────────────────────────────────────────────────

class GuitarDataset:
    """Loads and provides access to the full guitar training dataset."""

    def __init__(self, version, metadata, samples):
        self.version = version
        self.metadata = metadata
        self.samples = samples

    @classmethod
    def load(cls, path):
        """Load a MessagePack dataset file produced by guitar_capture.rs."""
        print(f"Loading dataset from {path} ...")
        with open(path, "rb") as f:
            raw = msgpack.unpack(f, raw=False, strict_map_key=False)

        # rmp_serde serializes structs as arrays by default
        if isinstance(raw, (list, tuple)):
            version = raw[0]
            meta_raw = raw[1]
            samples_raw = raw[2]
        elif isinstance(raw, dict):
            version = raw.get("version", raw.get(b"version", 0))
            meta_raw = raw.get("metadata", raw.get(b"metadata", {}))
            samples_raw = raw.get("samples", raw.get(b"samples", []))
        else:
            raise ValueError(f"Unexpected top-level type: {type(raw)}")

        metadata = cls._parse_metadata(meta_raw)
        samples = [GuitarSample(s) for s in samples_raw]

        print(f"  Loaded v{version}: {len(samples)} samples")
        return cls(version, metadata, samples)

    @staticmethod
    def _parse_metadata(raw):
        """Parse metadata from either array or dict format."""
        meta = {}
        if isinstance(raw, (list, tuple)):
            for i, field in enumerate(_METADATA_FIELDS):
                if i < len(raw):
                    value = raw[i]
                    if isinstance(value, bytes):
                        value = value.decode("utf-8", errors="replace")
                    if field == "noise_categories" and isinstance(value, list):
                        value = [
                            v.decode("utf-8", errors="replace") if isinstance(v, bytes) else v
                            for v in value
                        ]
                    meta[field] = value
        elif isinstance(raw, dict):
            for field in _METADATA_FIELDS:
                value = raw.get(field, raw.get(field.encode(), None))
                if isinstance(value, bytes):
                    value = value.decode("utf-8", errors="replace")
                if field == "noise_categories" and isinstance(value, list):
                    value = [
                        v.decode("utf-8", errors="replace") if isinstance(v, bytes) else v
                        for v in value
                    ]
                meta[field] = value
        return meta

    # ── Query Methods ─────────────────────────────────────────────

    def get_by_position(self, string_idx, fret):
        """Get all samples for a specific string/fret position."""
        return [
            s for s in self.samples
            if s.string_idx == string_idx and s.fret == fret
        ]

    def get_by_string(self, string_idx):
        """Get all samples for a given string (0-5)."""
        return [s for s in self.samples if s.string_idx == string_idx]

    def get_noise(self, category=None):
        """Get noise samples, optionally filtered by category."""
        noise = [s for s in self.samples if s.is_noise]
        if category:
            noise = [
                s for s in noise
                if isinstance(s.label, str) and category in s.label
            ]
        return noise

    def get_by_label(self, label):
        """Get all samples matching a label string."""
        return [s for s in self.samples if s.label == label]

    @property
    def labels(self):
        """Sorted list of unique labels."""
        return sorted(set(s.label for s in self.samples))

    @property
    def class_ids(self):
        """Sorted list of unique class IDs."""
        return sorted(set(s.class_id for s in self.samples))

    @property
    def string_samples(self):
        """All non-noise samples."""
        return [s for s in self.samples if not s.is_noise]

    @property
    def noise_samples(self):
        """All noise samples."""
        return [s for s in self.samples if s.is_noise]

    # ── Playback ──────────────────────────────────────────────────

    def play(self, idx):
        """
        Play a sample by index using afplay (macOS) or aplay (Linux).
        Writes a temporary WAV file and plays it.
        """
        if idx < 0 or idx >= len(self.samples):
            print(f"Index {idx} out of range (0-{len(self.samples) - 1})")
            return

        sample = self.samples[idx]
        if not sample.audio:
            print("Sample has no audio data.")
            return

        sr = sample.sample_rate or self.metadata.get("sample_rate", 44100)
        audio = np.array(sample.audio, dtype=np.float32)

        # Write temporary WAV
        try:
            import soundfile as sf
        except ImportError:
            print("ERROR: soundfile not installed. Run: pip install soundfile")
            return

        with tempfile.NamedTemporaryFile(suffix=".wav", delete=False) as tmp:
            tmp_path = tmp.name
            sf.write(tmp_path, audio, sr)

        print(f"Playing sample {idx}: {sample.label} "
              f"(rms={sample.rms:.4f}, {sample.duration_secs:.2f}s)")

        # Platform-specific playback
        if sys.platform == "darwin":
            subprocess.run(["afplay", tmp_path], check=False)
        elif sys.platform.startswith("linux"):
            subprocess.run(["aplay", tmp_path], check=False)
        else:
            print(f"Playback not supported on {sys.platform}. WAV at: {tmp_path}")

    # ── Summary ───────────────────────────────────────────────────

    def summary(self):
        """Print a comprehensive dataset summary."""
        m = self.metadata
        print()
        print("=" * 60)
        print(f"  Contrapunk Guitar Dataset v{self.version}")
        print("=" * 60)
        print(f"  Guitar:       {m.get('guitar_name', 'unknown')}")
        print(f"  Strings:      {m.get('strings', '?')}")
        print(f"  Frets:        {m.get('frets', '?')}")
        print(f"  Plucks/pos:   {m.get('plucks_per_position', '?')}")
        print(f"  Sample rate:  {m.get('sample_rate', '?')} Hz")
        print(f"  Duration:     {m.get('sample_duration_secs', '?')}s per sample")
        print(f"  Device:       {m.get('device_name', 'unknown')}")
        print(f"  Channel:      {m.get('channel', '?')}")
        print(f"  Captured:     {m.get('capture_date', 'unknown')}")
        print(f"  Noise cats:   {m.get('noise_categories', [])}")
        print()

        total = len(self.samples)
        note_samples = self.string_samples
        noise = self.noise_samples
        print(f"  Total samples:   {total}")
        print(f"  Note samples:    {len(note_samples)}")
        print(f"  Noise samples:   {len(noise)}")
        print(f"  Unique labels:   {len(self.labels)}")
        print(f"  Unique classes:  {len(self.class_ids)}")
        print()

        # Per-string stats
        string_names = ["E2 (low)", "A2", "D3", "G3", "B3", "E4 (high)"]
        for si in range(6):
            ss = self.get_by_string(si)
            if not ss:
                continue
            rms_vals = [s.rms for s in ss]
            conf_vals = [s.confidence for s in ss]
            validated = sum(1 for s in ss if s.pitch_validated)
            correct = sum(1 for s in ss if s.pitch_error is not None and s.pitch_error <= 1)
            name = string_names[si] if si < len(string_names) else f"String {si}"
            print(f"  {name:12s}  {len(ss):5d} samples  "
                  f"rms={np.mean(rms_vals):.4f}  "
                  f"conf={np.mean(conf_vals):.0%}  "
                  f"validated={validated}  "
                  f"pitch_acc={correct/len(ss):.0%}")

        if noise:
            rms_vals = [s.rms for s in noise]
            print(f"  {'Noise':12s}  {len(noise):5d} samples  "
                  f"rms={np.mean(rms_vals):.4f}")

        # Sample duration stats
        durations = [s.duration_secs for s in self.samples if s.duration_secs > 0]
        if durations:
            print(f"\n  Duration stats: "
                  f"min={min(durations):.3f}s  "
                  f"max={max(durations):.3f}s  "
                  f"mean={np.mean(durations):.3f}s")

        # Amplitude stats
        rms_all = [s.rms for s in self.samples]
        peak_all = [s.peak for s in self.samples]
        print(f"  RMS range:    [{min(rms_all):.4f}, {max(rms_all):.4f}]  "
              f"mean={np.mean(rms_all):.4f}")
        print(f"  Peak range:   [{min(peak_all):.4f}, {max(peak_all):.4f}]  "
              f"mean={np.mean(peak_all):.4f}")

        # Clipping / silence warnings
        clipped = sum(1 for s in self.samples if s.peak > 0.99)
        silent = sum(1 for s in self.samples if s.rms < 0.005)
        if clipped:
            print(f"\n  WARNING: {clipped} samples may be clipped (peak > 0.99)")
        if silent:
            print(f"  WARNING: {silent} samples may be silent (rms < 0.005)")

        print()
        print("=" * 60)


# ── Main ──────────────────────────────────────────────────────────────

if __name__ == "__main__":
    if len(sys.argv) < 2:
        print("Usage: python loader.py <path_to_msgpack>")
        print("  e.g.: python loader.py guitar_training_data.msgpack")
        sys.exit(1)

    dataset = GuitarDataset.load(sys.argv[1])
    dataset.summary()
