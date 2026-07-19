"""
Export representative WAV samples from the Contrapunk guitar training dataset.

Produces three categories of samples for the web diary:
  1. by_class/   — 1 clean, independently pitch-validated note per class
  2. showcase/    — 3 per string (open, fret 5, fret 12) = 18 samples
  3. confused_pairs/ — same-note pairs on different strings

Output: ui/static/samples/ with an index.json manifest.

Usage:
    source ml/venv/bin/activate
    python ml/training/export_samples.py
"""

import json
import os
import sys
from pathlib import Path

import librosa
import numpy as np
from scipy.signal import resample_poly

# ── Project paths ────────────────────────────────────────────────────
PROJECT_ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(PROJECT_ROOT))

from ml.loader import GuitarDataset

DATASET_PATH = PROJECT_ROOT / "guitar_training_data.msgpack"
OUTPUT_DIR = PROJECT_ROOT / "ui" / "static" / "samples"
ONSET_ANNOTATIONS_PATH = (
    PROJECT_ROOT
    / "crates"
    / "contrapunk-audio"
    / "tests"
    / "guitar_corpus_onsets.tsv"
)

SAMPLE_RATE = 48000
DURATION_SECS = 0.5
DURATION_SAMPLES = int(SAMPLE_RATE * DURATION_SECS)
NOTE_DURATION_SAMPLES = int(SAMPLE_RATE * 0.25)
FADE_SAMPLES = int(SAMPLE_RATE * 0.005)
PITCH_FRAME_SIZE = 2048
PITCH_HOP_SIZE = 128
PITCH_PREROLL_SAMPLES = int(SAMPLE_RATE * 0.010)
LEGACY_CAPTURE_FRAME_SIZE = 2048
TARGET_PEAK = 0.8
ONSET_FRAME_SIZE = 128
ONSET_PEAK_RATIO = 0.10

STRING_NAMES = ["E2", "A2", "D3", "G3", "B3", "E4"]

# Confused pairs: (note_name, midi_note, [(string_idx, fret), ...])
CONFUSED_PAIRS = [
    ("A2", 45, [(0, 5), (1, 0)]),
    ("D3", 50, [(1, 5), (2, 0)]),
    ("G3", 55, [(2, 5), (3, 0)]),
    ("B3", 59, [(3, 4), (4, 0)]),
    ("E4", 64, [(4, 5), (5, 0)]),
]


def repair_legacy_capture(audio):
    """Remove the onset block duplicated by the legacy capture callback.

    The old callback appended its triggering 2,048-sample block twice. The
    equality guard makes this a no-op for future recordings captured after the
    corresponding Rust fix.
    """
    frame = LEGACY_CAPTURE_FRAME_SIZE
    if len(audio) >= frame * 2 and np.array_equal(audio[:frame], audio[frame:frame * 2]):
        return np.concatenate([audio[:frame], audio[frame * 2:]])
    return audio


def resample_audio(audio, source_rate, target_rate):
    """Resample without changing pitch before writing a new sample-rate header."""
    if source_rate == target_rate:
        return audio.astype(np.float32, copy=False)
    return resample_poly(audio, target_rate, source_rate).astype(np.float32)


def prepare_source_audio(sample):
    """Repair legacy capture data and convert it to the fixture sample rate."""
    repaired = repair_legacy_capture(sample.audio_np)
    return resample_audio(repaired, int(sample.sample_rate), SAMPLE_RATE)


def pitch_evidence_end(audio, expected_midi):
    """Return the end of the first three-frame run at the annotated pitch.

    `librosa.yin(center=False)` timestamps each evidence window at its start;
    stable pitch is only established at the window end. Using that end avoids
    retaining the ~43 ms of non-pitched lead-in that invalidated the old corpus.
    """
    frequencies = librosa.yin(
        audio,
        fmin=75,
        fmax=1400,
        sr=SAMPLE_RATE,
        frame_length=PITCH_FRAME_SIZE,
        hop_length=PITCH_HOP_SIZE,
        center=False,
        trough_threshold=0.1,
    )
    midi = 69 + 12 * np.log2(frequencies / 440)
    matching = np.abs(midi - expected_midi) <= 0.75
    for frame in range(max(0, len(matching) - 2)):
        if matching[frame:frame + 3].all():
            return min(
                len(audio),
                (frame + 2) * PITCH_HOP_SIZE + PITCH_FRAME_SIZE,
            )
    return None


def source_quality(audio, evidence_end, expected_midi):
    """Score periodicity and broadband noise after independent pitch evidence."""
    segment = audio[evidence_end:evidence_end + int(SAMPLE_RATE * 0.1)]
    if len(segment) < 1024:
        return float("-inf")

    expected_freq = 440 * 2 ** ((expected_midi - 69) / 12)
    lag = round(SAMPLE_RATE / expected_freq)
    left = segment[:-lag]
    right = segment[lag:]
    denominator = float(left @ left + right @ right)
    periodicity = 2 * float(left @ right) / denominator if denominator > 1e-12 else -1.0
    flatness = float(np.median(librosa.feature.spectral_flatness(
        y=segment,
        n_fft=1024,
        hop_length=256,
    )))
    return periodicity - 2 * flatness


def pick_cleanest(samples):
    """Select one clean pluck using an independent, uniform quality policy."""
    choices = []
    for source_index, sample in enumerate(samples):
        audio = prepare_source_audio(sample)
        evidence_end = pitch_evidence_end(audio, sample.expected_midi)
        if evidence_end is None:
            continue
        quality = source_quality(audio, evidence_end, sample.expected_midi)
        choices.append((
            -quality,
            source_index,
            sample,
            audio,
            evidence_end,
        ))

    if not choices:
        return None
    _, _, sample, audio, evidence_end = min(choices)
    return sample, audio, evidence_end


def curate_note(audio, evidence_end):
    """Create one clean 250 ms note followed by silence in a 500 ms fixture."""
    start = max(0, evidence_end - PITCH_PREROLL_SAMPLES)
    note = audio[start:start + NOTE_DURATION_SAMPLES].copy()
    if len(note) > FADE_SAMPLES:
        note[-FADE_SAMPLES:] *= np.linspace(
            1.0,
            0.0,
            FADE_SAMPLES,
            endpoint=True,
            dtype=np.float32,
        )
    output = np.pad(
        note,
        (0, DURATION_SAMPLES - len(note)),
        mode="constant",
    )
    peak = float(np.max(np.abs(output)))
    if peak > 1e-6:
        output *= TARGET_PEAK / peak
    return output


def write_wav(path, audio_f32, sr):
    """Write a headroom-safe float32 array as 16-bit PCM WAV."""
    import soundfile as sf

    assert float(np.max(np.abs(audio_f32))) <= 1.0
    sf.write(str(path), audio_f32, sr, subtype="PCM_16")


def fret_label(fret):
    """Human-readable fret label for filenames."""
    if fret == 0:
        return "open"
    return f"fret{fret}"


def string_label_for_confused(string_idx):
    """Return the string letter for confused-pair filenames."""
    return STRING_NAMES[string_idx].rstrip("0123456789")


def fixture_onset_sample(audio):
    """Return the first 128-sample frame reaching 10% of peak frame RMS.

    This preserves the original evaluator heuristic as reproducible fixture
    provenance. These values are not independently hand-reviewed onset truth.
    """
    frame_rms = [
        float(np.sqrt(np.mean(frame * frame)))
        for start in range(0, len(audio), ONSET_FRAME_SIZE)
        if len(frame := audio[start:start + ONSET_FRAME_SIZE])
    ]
    peak = max(frame_rms, default=0.0)
    threshold = peak * ONSET_PEAK_RATIO
    return next(
        (frame * ONSET_FRAME_SIZE for frame, rms in enumerate(frame_rms)
         if rms >= threshold),
        0,
    )


def write_onset_annotations(annotations):
    """Write deterministic corpus onset provenance consumed by Rust tests."""
    header = [
        "# Generated by ml/training/export_samples.py from exported mono 16-bit PCM WAVs.",
        "# Method: first 128-sample frame with RMS >= 10% of that file's peak frame RMS.",
        "# Values are frozen fixture provenance, not independently hand-reviewed ground truth.",
    ]
    rows = [f"{name}\t{annotations[name]}" for name in sorted(annotations)]
    ONSET_ANNOTATIONS_PATH.write_text("\n".join(header + rows) + "\n")


def export_sample(selection, filepath, index_entries, onset_annotations=None):
    """Export one independently curated sample and record its metadata."""
    sample, source_audio, evidence_end = selection
    source_rate = int(sample.sample_rate)
    audio = curate_note(source_audio, evidence_end)
    write_wav(filepath, audio, SAMPLE_RATE)
    if onset_annotations is not None:
        import soundfile as sf
        exported_audio, _ = sf.read(str(filepath), dtype="float32")
        onset_annotations[filepath.name] = fixture_onset_sample(exported_audio)

    entry = {
        "filename": str(filepath.relative_to(OUTPUT_DIR)),
        "string_idx": sample.string_idx,
        "fret": sample.fret,
        "midi_note": sample.expected_midi,
        "rms": round(float(np.sqrt(np.mean(audio * audio))), 6),
        "duration": DURATION_SECS,
        "sample_rate": SAMPLE_RATE,
        "source_sample_rate": source_rate,
    }
    index_entries.append(entry)
    return entry


def main():
    # ── Load dataset ─────────────────────────────────────────────────
    ds = GuitarDataset.load(str(DATASET_PATH))
    note_samples = ds.string_samples
    print(f"\n  Note samples available: {len(note_samples)}")
    print(f"  Unique classes: {len(ds.class_ids)}\n")

    # ── Prepare output dirs ──────────────────────────────────────────
    by_class_dir = OUTPUT_DIR / "by_class"
    showcase_dir = OUTPUT_DIR / "showcase"
    confused_dir = OUTPUT_DIR / "confused_pairs"
    for d in (by_class_dir, showcase_dir, confused_dir):
        d.mkdir(parents=True, exist_ok=True)

    index_entries = []
    onset_annotations = {}
    exported = 0
    skipped_classes = []

    # ── 1. By-class: one clean isolated note per position ────────────
    print("Exporting by_class/ (1 independently curated note per class) ...")
    for string_idx in range(6):
        for fret in range(23):  # 0..22
            candidates = ds.get_by_position(string_idx, fret)
            selection = pick_cleanest(candidates)
            if selection is None:
                skipped_classes.append((string_idx, fret))
                continue
            fname = f"string_{string_idx}_fret_{fret}.wav"
            export_sample(
                selection,
                by_class_dir / fname,
                index_entries,
                onset_annotations,
            )
            exported += 1

    write_onset_annotations(onset_annotations)
    print(f"  Wrote onset provenance: {ONSET_ANNOTATIONS_PATH}")

    if skipped_classes:
        print(f"  Skipped {len(skipped_classes)} empty classes "
              f"(no samples in dataset)")

    # ── 2. Showcase: open, fret 5, fret 12 per string ───────────────
    print("Exporting showcase/ (open, fret5, fret12 per string) ...")
    showcase_frets = [0, 5, 12]
    for string_idx in range(6):
        name = STRING_NAMES[string_idx]
        for fret in showcase_frets:
            candidates = ds.get_by_position(string_idx, fret)
            selection = pick_cleanest(candidates)
            if selection is None:
                print(f"  WARNING: No sample for {name} fret {fret}")
                continue
            fl = fret_label(fret)
            fname = f"{name}_{fl}.wav"
            export_sample(selection, showcase_dir / fname, index_entries)
            exported += 1

    # ── 3. Confused pairs ────────────────────────────────────────────
    print("Exporting confused_pairs/ ...")
    for note_name, midi, positions in CONFUSED_PAIRS:
        for string_idx, fret in positions:
            candidates = ds.get_by_position(string_idx, fret)
            selection = pick_cleanest(candidates)
            if selection is None:
                sname = STRING_NAMES[string_idx]
                print(f"  WARNING: No sample for {note_name} on "
                      f"{sname} string fret {fret}")
                continue
            sname = STRING_NAMES[string_idx]
            s_letter = string_label_for_confused(string_idx)
            fl = fret_label(fret)
            fname = f"{note_name}_on_{s_letter}_string_{fl}.wav"
            export_sample(selection, confused_dir / fname, index_entries)
            exported += 1

    # ── Write index.json ─────────────────────────────────────────────
    index_path = OUTPUT_DIR / "index.json"
    with open(index_path, "w") as f:
        json.dump(index_entries, f, indent=2)
    print(f"\nWrote {index_path}")

    # ── Report ───────────────────────────────────────────────────────
    total_bytes = sum(
        p.stat().st_size
        for p in OUTPUT_DIR.rglob("*.wav")
    )
    index_bytes = index_path.stat().st_size
    total_bytes += index_bytes

    print(f"\n{'='*50}")
    print(f"  Exported {exported} WAV files")
    print(f"  Total size: {total_bytes / 1024:.1f} KB "
          f"({total_bytes / (1024*1024):.2f} MB)")
    print(f"  index.json: {index_bytes} bytes "
          f"({len(index_entries)} entries)")
    print(f"{'='*50}\n")


if __name__ == "__main__":
    main()
