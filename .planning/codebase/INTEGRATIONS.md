# External Integrations

**Analysis Date:** 2026-01-28

## APIs & External Services

**None Detected**

No HTTP-based external APIs or cloud services are integrated. Application is entirely self-contained and runs locally.

## Data Storage

**Databases:**
- Not applicable - No persistent data storage

**File Storage:**
- Local filesystem only
  - Templates directory: `/templates/` (unused in current implementation)
  - No file-based data persistence

**Caching:**
- None - All operations are real-time in-memory

## Audio Hardware Integration

**Audio Devices:**
- System audio interface (via sounddevice)
  - Configuration: Device selected at runtime via TUI
  - Default sample rate: 48000 Hz (configurable per `AudioToMidi` class)
  - Buffer size: 512 samples (configurable)
  - Multiple input channels supported via `input_channel` parameter

**MIDI Hardware:**
- Local MIDI input devices
  - Enumerated via mido at startup
  - Selected via TUI menu
  - Supports multiple simultaneous input sources

- Local MIDI output devices
  - Enumerated via mido at startup
  - Multiple outputs selected (2-8 ports) via TUI
  - First port: melody pass-through
  - Ports 2+: Harmony generation output

## Authentication & Identity

**Auth Provider:**
- Not applicable - No authentication required

## Monitoring & Observability

**Error Tracking:**
- None

**Logs:**
- Console output via print statements
  - Audio status messages from sounddevice
  - MIDI message processing errors
  - General exception handling with stderr output

## CI/CD & Deployment

**Hosting:**
- Local/desktop application only

**CI Pipeline:**
- Not detected

## Environment Configuration

**Required env vars:**
- None

**Secrets location:**
- Not applicable

## Webhooks & Callbacks

**Incoming:**
- Not applicable

**Outgoing:**
- Not applicable

## Hardware Communication Details

**MIDI Flow:**
1. User selects MIDI input device from available ports (enumerated via mido)
2. Input device sends note_on/note_off messages
3. Application processes messages based on selected mode (harmony generation)
4. Harmony notes sent to multiple output MIDI ports simultaneously
5. Timing: Real-time event-driven processing via mido.open_input().poll()

**Audio Flow:**
1. User selects audio device from available inputs (enumerated via sounddevice)
2. Continuous audio stream captured at configured sample rate and buffer size
3. Pitch detection performed via autocorrelation in `AudioToMidi.detect_pitch()`
4. Detected pitch converted to MIDI note
5. Note sent to configured output ports via mido

---

*Integration audit: 2026-01-28*
