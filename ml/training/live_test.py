#!/usr/bin/env python3
"""
Contrapunk Live Guitar Inference

Captures audio from an audio interface (auto-detects Audient iD14),
detects pluck onsets via RMS threshold, extracts mel-spectrograms,
and classifies string + fret using the trained Pure CNN model.

Usage:
    cd <project_root>
    source ml/venv/bin/activate
    python ml/training/live_test.py

Options (env vars):
    RMS_THRESHOLD=0.02      Onset detection threshold (default 0.02, auto-set by calibration)
    DEVICE_NAME=...         Force a specific audio device name substring
    CHANNEL=1               Audio input channel to use (1-indexed, default 1)
    SKIP_CAL=1              Skip calibration and use default threshold

Press Ctrl+C to stop.
"""

import sys
import os
import time
import threading
import numpy as np

import torch
import torch.nn as nn
import librosa
import sounddevice as sd

# ---------------------------------------------------------------------------
# Paths
# ---------------------------------------------------------------------------

SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))
PROJECT_ROOT = os.path.abspath(os.path.join(SCRIPT_DIR, "../.."))
MODEL_PATH = os.path.join(SCRIPT_DIR, "round_01", "pure_cnn.pt")
DATASET_PATH = os.path.join(PROJECT_ROOT, "guitar_training_data.msgpack")

# ---------------------------------------------------------------------------
# Audio / Feature Constants (must match training)
# ---------------------------------------------------------------------------

SAMPLE_RATE = 48000
CAPTURE_DURATION = 0.5          # seconds
CAPTURE_SAMPLES = int(SAMPLE_RATE * CAPTURE_DURATION)  # 24000
N_MELS = 64
N_FFT = 1024
HOP_LENGTH = 256
TARGET_TIME_FRAMES = 94         # training produced 94 time frames

# Onset detection
DEFAULT_RMS_THRESHOLD = 0.02
RMS_WINDOW = 512                # samples per RMS calculation block
PRE_TRIGGER_SAMPLES = 512       # keep a tiny lookback so the attack isn't clipped

# ---------------------------------------------------------------------------
# ANSI colors
# ---------------------------------------------------------------------------

class C:
    RESET   = "\033[0m"
    BOLD    = "\033[1m"
    DIM     = "\033[2m"
    RED     = "\033[91m"
    GREEN   = "\033[92m"
    YELLOW  = "\033[93m"
    BLUE    = "\033[94m"
    MAGENTA = "\033[95m"
    CYAN    = "\033[96m"
    WHITE   = "\033[97m"


def confidence_color(conf):
    if conf >= 0.90:
        return C.GREEN
    elif conf >= 0.70:
        return C.YELLOW
    return C.RED


def rms_bar(rms, width=40, threshold=DEFAULT_RMS_THRESHOLD):
    """ASCII level meter."""
    level = min(rms / 0.15, 1.0)
    filled = int(level * width)
    bar = "#" * filled + "-" * (width - filled)
    thresh_pos = int(min(threshold / 0.15, 1.0) * width)
    if rms < threshold:
        color = C.DIM
    elif rms < threshold * 3:
        color = C.GREEN
    else:
        color = C.YELLOW
    bar_list = list(bar)
    if 0 <= thresh_pos < width:
        bar_list[thresh_pos] = "|"
    return f"{color}[{''.join(bar_list)}]{C.RESET} {rms:.4f}"


# ---------------------------------------------------------------------------
# Model (must match training exactly)
# ---------------------------------------------------------------------------

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


# ---------------------------------------------------------------------------
# Label Map
# ---------------------------------------------------------------------------

STRING_NAMES = ["E2 (low)", "A2", "D3", "G3", "B3", "E4 (high)"]


def build_label_map():
    """Build class_id (0-indexed) -> label string mapping from the dataset."""
    sys.path.insert(0, PROJECT_ROOT)
    from ml.loader import GuitarDataset

    ds = GuitarDataset.load(DATASET_PATH)
    class_to_label = {}
    for s in ds.samples:
        class_to_label[s.class_id - 1] = s.label
    return class_to_label


def parse_label(label):
    """Parse 'string_3_fret_12' -> (string_idx, fret)."""
    parts = label.split("_")
    try:
        si = int(parts[1])
        fret = int(parts[3])
        return si, fret
    except (IndexError, ValueError):
        return None, None


# ---------------------------------------------------------------------------
# Feature Extraction
# ---------------------------------------------------------------------------

def extract_mel_spectrogram(audio):
    """Extract log-mel spectrogram matching training params."""
    mel = librosa.feature.melspectrogram(
        y=audio, sr=SAMPLE_RATE,
        n_mels=N_MELS, n_fft=N_FFT, hop_length=HOP_LENGTH,
    )
    log_mel = librosa.power_to_db(mel, ref=np.max)
    return log_mel


def pad_spectrogram(mel, target_time=TARGET_TIME_FRAMES):
    """Pad or truncate to uniform time dimension (matches training)."""
    if mel.shape[1] < target_time:
        pad_width = target_time - mel.shape[1]
        mel = np.pad(mel, ((0, 0), (0, pad_width)), mode="constant",
                     constant_values=mel.min())
    elif mel.shape[1] > target_time:
        mel = mel[:, :target_time]
    return mel


# ---------------------------------------------------------------------------
# Audio Device Selection
# ---------------------------------------------------------------------------

def list_input_devices():
    """List available audio input devices."""
    devices = sd.query_devices()
    inputs = []
    for i, d in enumerate(devices):
        if d["max_input_channels"] > 0:
            inputs.append((i, d))
    return inputs


def select_device(force_name=None):
    """Auto-detect Audient iD14 or let user choose."""
    inputs = list_input_devices()

    if not inputs:
        print(f"{C.RED}ERROR: No audio input devices found.{C.RESET}")
        sys.exit(1)

    target_name = force_name or os.environ.get("DEVICE_NAME", "Audient iD14")

    print(f"\n{C.BOLD}Available audio input devices:{C.RESET}")
    print(f"{'':>4}{'#':>4}  {'Name':<45} {'Channels':>8}  {'Rate':>7}")
    print(f"{'':>4}{'':->4}  {'':->45} {'':->8}  {'':->7}")

    auto_match = None
    for idx, (dev_id, d) in enumerate(inputs):
        name = d["name"]
        ch = d["max_input_channels"]
        sr = d["default_samplerate"]
        marker = ""
        if target_name.lower() in name.lower():
            marker = f" {C.GREEN}<-- auto-detected{C.RESET}"
            auto_match = dev_id
        print(f"    {idx:>4}  {name:<45} {ch:>8}  {sr:>7.0f}{marker}")

    print()

    if auto_match is not None:
        dev = sd.query_devices(auto_match)
        print(f"{C.GREEN}Auto-selected: {dev['name']} (device #{auto_match}){C.RESET}")
        return auto_match

    # Manual selection
    while True:
        try:
            choice = input(f"{C.CYAN}Select device # (or index from list): {C.RESET}").strip()
            choice_int = int(choice)
            if 0 <= choice_int < len(inputs):
                dev_id = inputs[choice_int][0]
                dev = sd.query_devices(dev_id)
                print(f"{C.GREEN}Selected: {dev['name']} (device #{dev_id}){C.RESET}")
                return dev_id
            else:
                dev = sd.query_devices(choice_int)
                if dev["max_input_channels"] > 0:
                    print(f"{C.GREEN}Selected: {dev['name']} (device #{choice_int}){C.RESET}")
                    return choice_int
                print(f"{C.RED}Device #{choice_int} has no input channels.{C.RESET}")
        except (ValueError, sd.PortAudioError):
            print(f"{C.RED}Invalid selection. Try again.{C.RESET}")
        except (EOFError, KeyboardInterrupt):
            print()
            sys.exit(0)


# ---------------------------------------------------------------------------
# Inference
# ---------------------------------------------------------------------------

def run_inference(model, audio, class_to_label, torch_device):
    """Run model inference on captured audio, return top-3 predictions."""
    mel = extract_mel_spectrogram(audio)
    mel = pad_spectrogram(mel)

    # Shape: (1, 1, N_MELS, TIME)
    x = torch.tensor(mel[np.newaxis, np.newaxis, ...], dtype=torch.float32).to(torch_device)

    with torch.no_grad():
        logits = model(x)
        probs = torch.softmax(logits, dim=1)[0]
        top3_prob, top3_idx = probs.topk(3)

    results = []
    for i in range(3):
        class_id = top3_idx[i].item()
        conf = top3_prob[i].item()
        label = class_to_label.get(class_id, f"class_{class_id}")
        si, fret = parse_label(label)
        string_name = STRING_NAMES[si] if si is not None and 0 <= si <= 5 else "?"
        results.append({
            "class_id": class_id,
            "label": label,
            "confidence": conf,
            "string_idx": si,
            "fret": fret,
            "string_name": string_name,
        })

    return results


CLEAR_SCREEN = "\033[2J\033[H"  # clear screen + cursor home
HIDE_CURSOR = "\033[?25l"
SHOW_CURSOR = "\033[?25h"


def display_dashboard(results, timestamp, detection_count, rms, rms_threshold, history):
    """Render a stable full-screen dashboard."""
    top = results[0]
    conf = top["confidence"]
    color = confidence_color(conf)

    lines = []
    lines.append(f"  {C.BOLD}{C.MAGENTA}{'=' * 56}{C.RESET}")
    lines.append(f"  {C.BOLD}{C.MAGENTA}  Contrapunk Live Guitar Classifier{C.RESET}")
    lines.append(f"  {C.BOLD}{C.MAGENTA}{'=' * 56}{C.RESET}")
    lines.append("")

    # Big prediction display
    fret_str = f"fret {top['fret']}" if top['fret'] > 0 else "open"
    lines.append(f"  {C.BOLD}  DETECTED:{C.RESET}  {color}{C.BOLD}{top['string_name']}  {fret_str}{C.RESET}")
    lines.append(f"  {C.BOLD}  CONFIDENCE:{C.RESET} {color}{conf:.1%}{C.RESET}  "
                 f"{C.DIM}@ {timestamp} (#{detection_count}){C.RESET}")
    lines.append("")

    # Top 3
    lines.append(f"  {C.DIM}  Top 3 predictions:{C.RESET}")
    for i, r in enumerate(results):
        rc = confidence_color(r["confidence"])
        bar_len = int(r["confidence"] * 25)
        bar = "\u2588" * bar_len + "\u2591" * (25 - bar_len)
        fstr = f"fret {r['fret']:>2}" if r['fret'] > 0 else "open  "
        marker = " \u25c0" if i == 0 else ""
        lines.append(f"    {rc}{r['string_name']:12s} {fstr}  {bar} {r['confidence']:5.1%}{C.RESET}{marker}")
    lines.append("")

    # Level meter
    meter = rms_bar(rms, width=40, threshold=rms_threshold)
    lines.append(f"  {C.DIM}  Level:{C.RESET} {meter}")
    lines.append("")

    # Recent history (last 5)
    if history:
        lines.append(f"  {C.DIM}  History (last {min(len(history), 8)}):{C.RESET}")
        for h in history[-8:]:
            hc = confidence_color(h['conf'])
            fstr = f"fret {h['fret']}" if h['fret'] > 0 else "open"
            lines.append(f"    {C.DIM}{h['time']}{C.RESET}  "
                         f"{hc}{h['string']:12s} {fstr:>8s}  {h['conf']:5.1%}{C.RESET}")
        lines.append("")

    lines.append(f"  {C.GREEN}{C.BOLD}  Listening...{C.RESET}  {C.DIM}(Ctrl+C to stop){C.RESET}")
    lines.append("")

    # Render
    sys.stdout.write(CLEAR_SCREEN + "\n".join(lines) + "\n")
    sys.stdout.flush()


# ---------------------------------------------------------------------------
# Pitch Detection (for tuning check)
# ---------------------------------------------------------------------------

# Standard guitar open string frequencies (Hz) and MIDI notes
OPEN_STRINGS = [
    {"name": "E2", "freq": 82.41, "midi": 40},
    {"name": "A2", "freq": 110.00, "midi": 45},
    {"name": "D3", "freq": 146.83, "midi": 50},
    {"name": "G3", "freq": 196.00, "midi": 55},
    {"name": "B3", "freq": 246.94, "midi": 59},
    {"name": "E4", "freq": 329.63, "midi": 64},
]


def detect_pitch_yin(audio, sr, fmin=70, fmax=400):
    """Simple YIN-based pitch detection. Returns (frequency_hz, confidence)."""
    try:
        f0 = librosa.yin(audio, fmin=fmin, fmax=fmax, sr=sr)
        # Take median of non-zero estimates
        valid = f0[f0 > 0]
        if len(valid) == 0:
            return 0.0, 0.0
        freq = float(np.median(valid))
        # Confidence: fraction of frames that agree (within 5%)
        near_median = np.abs(valid - freq) < freq * 0.05
        conf = float(np.sum(near_median)) / max(len(f0), 1)
        return freq, conf
    except Exception:
        return 0.0, 0.0


def freq_to_cents(detected, target):
    """Cents deviation from target frequency."""
    if detected <= 0 or target <= 0:
        return 0
    return int(round(1200 * np.log2(detected / target)))


# ---------------------------------------------------------------------------
# Calibration
# ---------------------------------------------------------------------------

def run_calibration(dev_id, n_channels, channel, default_threshold):
    """3-step calibration: noise floor, signal level, optional tuning check."""
    print(f"\n  {C.BOLD}{C.CYAN}{'=' * 50}{C.RESET}")
    print(f"  {C.BOLD}{C.CYAN}  CALIBRATION{C.RESET}")
    print(f"  {C.BOLD}{C.CYAN}{'=' * 50}{C.RESET}")

    rms_samples = []

    # --- Step 1: Noise floor ---
    print(f"\n  {C.BOLD}Step 1:{C.RESET} Measuring noise floor...")
    print(f"  {C.DIM}Don't touch the guitar for 3 seconds.{C.RESET}")
    time.sleep(0.5)

    noise_rms = []
    try:
        with sd.InputStream(device=dev_id, samplerate=SAMPLE_RATE,
                            channels=n_channels, dtype="float32",
                            blocksize=2048) as stream:
            end_time = time.time() + 3.0
            while time.time() < end_time:
                data, _ = stream.read(2048)
                mono = data[:, channel]
                rms = float(np.sqrt(np.mean(mono ** 2)))
                noise_rms.append(rms)
                remaining = max(0, end_time - time.time())
                bar_len = int((1 - remaining / 3.0) * 30)
                bar = "\u2588" * bar_len + "\u2591" * (30 - bar_len)
                print(f"\r    [{bar}] {remaining:.1f}s  rms={rms:.5f}", end="", flush=True)
    except Exception as e:
        print(f"\n  {C.RED}Audio error: {e}{C.RESET}")
        return default_threshold

    noise_floor = float(np.mean(noise_rms))
    noise_peak = float(np.max(noise_rms))
    print(f"\r    {C.GREEN}Noise floor: {noise_floor:.5f} (peak: {noise_peak:.5f}){C.RESET}          ")

    # --- Step 2: Signal level ---
    print(f"\n  {C.BOLD}Step 2:{C.RESET} Pluck any string {C.BOLD}hard{C.RESET} (strongest pluck)...")

    signal_rms = 0.0
    try:
        with sd.InputStream(device=dev_id, samplerate=SAMPLE_RATE,
                            channels=n_channels, dtype="float32",
                            blocksize=1024) as stream:
            waiting = True
            timeout = time.time() + 10.0
            while waiting and time.time() < timeout:
                data, _ = stream.read(1024)
                mono = data[:, channel]
                rms = float(np.sqrt(np.mean(mono ** 2)))
                print(f"\r    Level: {rms_bar(rms, 40, noise_peak * 3)}  "
                      f"{C.DIM}Waiting for pluck...{C.RESET}  ", end="", flush=True)
                if rms > noise_peak * 5:
                    # Pluck detected — capture peak RMS over next 0.3s
                    peak_rms_vals = [rms]
                    end = time.time() + 0.3
                    while time.time() < end:
                        data, _ = stream.read(1024)
                        mono = data[:, channel]
                        r = float(np.sqrt(np.mean(mono ** 2)))
                        peak_rms_vals.append(r)
                    signal_rms = float(np.max(peak_rms_vals))
                    waiting = False

            if signal_rms == 0:
                print(f"\r    {C.YELLOW}No pluck detected, using default threshold.{C.RESET}          ")
                return default_threshold
    except Exception as e:
        print(f"\n  {C.RED}Audio error: {e}{C.RESET}")
        return default_threshold

    # Set threshold: midpoint between noise peak and signal, biased toward noise
    threshold = noise_peak * 3
    threshold = max(threshold, 0.005)  # minimum sane value
    threshold = min(threshold, signal_rms * 0.3)  # never more than 30% of signal

    print(f"\r    {C.GREEN}Signal level: {signal_rms:.4f}{C.RESET}                                      ")
    print(f"    {C.GREEN}Threshold set to: {threshold:.4f}{C.RESET}")
    print(f"    {C.DIM}(noise={noise_floor:.4f}, signal={signal_rms:.4f}, "
          f"ratio={signal_rms/max(noise_floor,0.0001):.0f}x){C.RESET}")

    # --- Step 3: Quick tuning check (optional) ---
    print(f"\n  {C.BOLD}Step 3:{C.RESET} Quick tuning check")
    try:
        do_tune = input(f"    {C.CYAN}Check tuning? [Y/n]: {C.RESET}").strip().lower()
    except (EOFError, KeyboardInterrupt):
        do_tune = "n"

    if do_tune != "n":
        print(f"    {C.DIM}Pluck each open string when prompted.{C.RESET}\n")

        tuning_results = []
        for si, string in enumerate(OPEN_STRINGS):
            print(f"    {C.BOLD}{string['name']}{C.RESET} ({string['freq']:.1f} Hz) — pluck now...",
                  end="", flush=True)

            # Capture 0.5s after onset
            captured = False
            try:
                with sd.InputStream(device=dev_id, samplerate=SAMPLE_RATE,
                                    channels=n_channels, dtype="float32",
                                    blocksize=1024) as stream:
                    timeout = time.time() + 8.0
                    while not captured and time.time() < timeout:
                        data, _ = stream.read(1024)
                        mono = data[:, channel]
                        rms = float(np.sqrt(np.mean(mono ** 2)))
                        if rms > threshold:
                            # Capture 0.5s
                            frames = [mono]
                            remaining = CAPTURE_SAMPLES - len(mono)
                            while remaining > 0:
                                data, _ = stream.read(min(remaining, 4096))
                                chunk = data[:, channel]
                                frames.append(chunk)
                                remaining -= len(chunk)
                            audio = np.concatenate(frames)[:CAPTURE_SAMPLES]

                            freq, conf = detect_pitch_yin(audio, SAMPLE_RATE,
                                                          fmin=max(50, string['freq'] * 0.7),
                                                          fmax=string['freq'] * 1.5)
                            cents = freq_to_cents(freq, string['freq'])

                            if conf > 0.3 and abs(cents) < 100:
                                if abs(cents) <= 5:
                                    status = f"{C.GREEN}IN TUNE{C.RESET}"
                                elif abs(cents) <= 15:
                                    arrow = "sharp" if cents > 0 else "flat"
                                    status = f"{C.YELLOW}{arrow} ({cents:+d}c){C.RESET}"
                                else:
                                    arrow = "SHARP" if cents > 0 else "FLAT"
                                    status = f"{C.RED}{arrow} ({cents:+d}c){C.RESET}"
                                tuning_results.append((string['name'], cents, True))
                            else:
                                status = f"{C.DIM}unclear (conf={conf:.0%}){C.RESET}"
                                tuning_results.append((string['name'], 0, False))

                            print(f"  {freq:.1f} Hz  {status}")
                            captured = True

                    if not captured:
                        print(f"  {C.DIM}(timeout){C.RESET}")
                        tuning_results.append((string['name'], 0, False))
            except Exception as e:
                print(f"  {C.RED}error: {e}{C.RESET}")
                tuning_results.append((string['name'], 0, False))

        # Summary
        good = sum(1 for _, c, ok in tuning_results if ok and abs(c) <= 10)
        total = sum(1 for _, _, ok in tuning_results if ok)
        if total > 0:
            print(f"\n    {C.BOLD}Tuning:{C.RESET} {good}/{total} strings within 10 cents")
            if good < total:
                print(f"    {C.YELLOW}Tip: out-of-tune strings may reduce classification accuracy{C.RESET}")
        print()

    print(f"  {C.BOLD}{C.GREEN}Calibration complete!{C.RESET}")
    return threshold


# ---------------------------------------------------------------------------
# Main Loop
# ---------------------------------------------------------------------------

def main():
    rms_threshold = float(os.environ.get("RMS_THRESHOLD", DEFAULT_RMS_THRESHOLD))
    force_device = os.environ.get("DEVICE_NAME")

    print()
    print(f"  {C.BOLD}{C.MAGENTA}========================================{C.RESET}")
    print(f"  {C.BOLD}{C.MAGENTA}  Contrapunk Live Guitar Inference{C.RESET}")
    print(f"  {C.BOLD}{C.MAGENTA}========================================{C.RESET}")
    print()
    print(f"  {C.DIM}Model:      Pure CNN (round 01){C.RESET}")
    print(f"  {C.DIM}Threshold:  {rms_threshold}{C.RESET}")
    print(f"  {C.DIM}Capture:    {CAPTURE_DURATION}s @ {SAMPLE_RATE} Hz{C.RESET}")
    print()

    # -- Load model --
    print(f"  {C.BOLD}Loading model...{C.RESET}", end=" ", flush=True)
    if not os.path.exists(MODEL_PATH):
        print(f"\n{C.RED}ERROR: Model not found at {MODEL_PATH}{C.RESET}")
        sys.exit(1)

    checkpoint = torch.load(MODEL_PATH, weights_only=True, map_location="cpu")
    n_classes = checkpoint["n_classes"]
    model = PureCNN(n_classes)
    model.load_state_dict(checkpoint["state_dict"])
    # Set model to inference mode (disables dropout)
    model.train(False)
    torch_device = torch.device("cpu")
    model = model.to(torch_device)

    acc = checkpoint.get("accuracy", None)
    acc_str = f"{acc:.1%}" if acc else "unknown"
    print(f"{C.GREEN}OK{C.RESET} ({n_classes} classes, accuracy: {acc_str})")

    # -- Build label map --
    print(f"  {C.BOLD}Loading label map...{C.RESET}", end=" ", flush=True)
    class_to_label = build_label_map()
    print(f"{C.GREEN}OK{C.RESET} ({len(class_to_label)} labels)")

    # -- Select audio device --
    dev_id = select_device(force_name=force_device)

    # -- Show device info & select channel --
    dev_info = sd.query_devices(dev_id)
    max_ch = dev_info['max_input_channels']
    print(f"\n  {C.DIM}Device: {dev_info['name']}{C.RESET}")
    print(f"  {C.DIM}Channels: {max_ch}, "
          f"Default SR: {dev_info['default_samplerate']:.0f} Hz{C.RESET}")

    # Channel selection (1-indexed for user, 0-indexed internally)
    env_channel = os.environ.get("CHANNEL")
    if env_channel:
        channel = int(env_channel) - 1
    elif max_ch > 1:
        print(f"\n  {C.BOLD}Select input channel (1-{max_ch}):{C.RESET}")
        for ch in range(min(max_ch, 12)):
            marker = " (default)" if ch == 0 else ""
            print(f"    {ch + 1}{marker}")
        try:
            ch_input = input(f"\n  {C.CYAN}Channel [1]: {C.RESET}").strip()
            channel = int(ch_input) - 1 if ch_input else 0
        except (ValueError, EOFError, KeyboardInterrupt):
            channel = 0
    else:
        channel = 0

    channel = max(0, min(channel, max_ch - 1))
    n_stream_channels = channel + 1  # need at least this many channels from the device
    print(f"  {C.GREEN}Using channel {channel + 1} of {max_ch}{C.RESET}")

    # -- Calibration ----------------------------------------------------------
    skip_cal = os.environ.get("SKIP_CAL", "").lower() in ("1", "true", "yes")
    if not skip_cal:
        rms_threshold = run_calibration(dev_id, n_stream_channels, channel, rms_threshold)

    # -- Start listening ------------------------------------------------------
    print(f"\n  {C.BOLD}{C.GREEN}Listening for plucks...{C.RESET} "
          f"(Ctrl+C to stop)\n")

    # Ring buffer for pre-trigger audio
    ring_buf = np.zeros(PRE_TRIGGER_SAMPLES, dtype=np.float32)
    ring_pos = [0]

    # State machine
    STATE_LISTENING = 0
    STATE_CAPTURING = 1
    state = [STATE_LISTENING]
    capture_buf = [np.zeros(0, dtype=np.float32)]
    capture_needed = [CAPTURE_SAMPLES]

    # Cooldown: ignore triggers for a brief period after a capture
    cooldown_until = [0.0]

    # Lock for thread safety
    lock = threading.Lock()

    # Pending results queue
    pending_audio = [None]

    def audio_callback(indata, frames, time_info, status):
        mono = indata[:, channel].copy()
        now = time.time()

        with lock:
            if state[0] == STATE_LISTENING:
                # Update ring buffer
                for sample in mono:
                    ring_buf[ring_pos[0]] = sample
                    ring_pos[0] = (ring_pos[0] + 1) % PRE_TRIGGER_SAMPLES

                # Check RMS
                rms = np.sqrt(np.mean(mono ** 2))

                # Onset detection
                if rms >= rms_threshold and now > cooldown_until[0]:
                    state[0] = STATE_CAPTURING
                    # Start capture buffer with pre-trigger + current block
                    pre = np.roll(ring_buf, -ring_pos[0])
                    capture_buf[0] = np.concatenate([pre, mono])
                    capture_needed[0] = CAPTURE_SAMPLES - len(capture_buf[0])
                    if capture_needed[0] <= 0:
                        pending_audio[0] = capture_buf[0][:CAPTURE_SAMPLES]
                        state[0] = STATE_LISTENING
                        cooldown_until[0] = now + 0.3

            elif state[0] == STATE_CAPTURING:
                capture_buf[0] = np.concatenate([capture_buf[0], mono])
                capture_needed[0] -= len(mono)
                if capture_needed[0] <= 0:
                    pending_audio[0] = capture_buf[0][:CAPTURE_SAMPLES]
                    state[0] = STATE_LISTENING
                    cooldown_until[0] = now + 0.3

    detection_count = 0
    history = []
    last_rms = [0.0]

    # Track RMS for dashboard display (updated from callback)
    orig_callback = audio_callback

    def audio_callback_with_rms(indata, frames, time_info, status):
        mono = indata[:, channel]
        last_rms[0] = float(np.sqrt(np.mean(mono ** 2)))
        orig_callback(indata, frames, time_info, status)

    try:
        sys.stdout.write(HIDE_CURSOR)
        stream = sd.InputStream(
            device=dev_id,
            samplerate=SAMPLE_RATE,
            channels=n_stream_channels,
            dtype="float32",
            blocksize=1024,
            callback=audio_callback_with_rms,
        )
        stream.start()

        # Initial waiting screen
        sys.stdout.write(CLEAR_SCREEN)
        print(f"\n  {C.BOLD}{C.MAGENTA}{'=' * 56}{C.RESET}")
        print(f"  {C.BOLD}{C.MAGENTA}  Contrapunk Live Guitar Classifier{C.RESET}")
        print(f"  {C.BOLD}{C.MAGENTA}{'=' * 56}{C.RESET}")
        print(f"\n  {C.DIM}  Device:  {dev_info['name']}{C.RESET}")
        print(f"  {C.DIM}  Channel: {channel + 1}/{max_ch}{C.RESET}")
        print(f"  {C.DIM}  Model:   Pure CNN Round 1 ({acc_str}){C.RESET}")
        print(f"\n  {C.GREEN}{C.BOLD}  Waiting for first pluck...{C.RESET}")
        print(f"  {C.DIM}  (Ctrl+C to stop){C.RESET}\n")

        while True:
            audio_data = None
            with lock:
                if pending_audio[0] is not None:
                    audio_data = pending_audio[0]
                    pending_audio[0] = None

            if audio_data is not None:
                detection_count += 1
                timestamp = time.strftime("%H:%M:%S")

                results = run_inference(model, audio_data, class_to_label, torch_device)

                top = results[0]
                history.append({
                    "time": timestamp,
                    "string": top["string_name"],
                    "fret": top["fret"],
                    "conf": top["confidence"],
                })

                display_dashboard(
                    results, timestamp, detection_count,
                    last_rms[0], rms_threshold, history
                )

            time.sleep(0.01)

    except KeyboardInterrupt:
        sys.stdout.write(SHOW_CURSOR)
        print(f"\n\n  {C.BOLD}{C.YELLOW}Stopped.{C.RESET} "
              f"{detection_count} plucks detected.")
        if history:
            print(f"\n  {C.DIM}Session summary:{C.RESET}")
            for h in history:
                print(f"    {h['time']}  {h['string']:12s} fret {h['fret']:>2}  {h['conf']:.1%}")
        print()
    except sd.PortAudioError as e:
        print(f"\n{C.RED}Audio error: {e}{C.RESET}")
        print(f"{C.DIM}Tip: Check that your Audient iD14 is connected "
              f"and recognized by macOS.{C.RESET}")
        sys.exit(1)
    finally:
        if "stream" in dir() and stream.active:
            stream.stop()
            stream.close()


if __name__ == "__main__":
    main()
