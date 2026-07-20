# Contrapunk architecture reconnaissance

_Date: 2026-06-29. Scope: repository architecture + runtime surfaces. Source files were not modified; this file is the requested output artifact._

## High-level map

Contrapunk is a Rust workspace with one shared musical/DSP core and several host surfaces:

```text
Input (MIDI / guitar audio / DAW notes / browser keyboard)
        ↓
contrapunk-harmony::HarmonyEngine
        ↓
Companion lanes (optional: canon + counterpoint)
        ↓
Dispatch to surface-specific outputs:
  - Tauri: external MIDI ports + internal Rust synth/FX chain + CLAP-host blocks
  - Browser: Web MIDI + WebAudio synth/FX + WASM Companion
  - Plugin: DA