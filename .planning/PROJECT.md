# Contrapunk Rust

## What This Is

A native Rust application for real-time MIDI harmony generation. Takes live MIDI input, applies one of 7 harmony algorithms, and outputs harmonized notes to multiple MIDI ports simultaneously. Built with egui/eframe for a responsive native GUI. Replaces the Python implementation for better performance and single-binary distribution.

## Core Value

Real-time harmony generation with minimal latency — when you play a note, the harmony appears instantly on the output ports.

## Requirements

### Validated

(None yet — ship to validate)

### Active

- [ ] User can select MIDI input device from available ports
- [ ] User can select multiple MIDI output ports (2-8 ports for harmony voices)
- [ ] User can select musical key (C through B, 12 options)
- [ ] User can select harmony mode (7 modes)
- [ ] Mode 1: Forward MIDI as-is (pass-through)
- [ ] Mode 2: Add diatonic thirds above input note
- [ ] Mode 3: Add diatonic fourths above input note
- [ ] Mode 4: Add random diatonic interval below input
- [ ] Mode 5: Add random diatonic below (excluding seconds)
- [ ] Mode 6: Contrary motion — harmony moves opposite to melody
- [ ] Mode 7: Strict counterpoint — follows traditional voice leading rules
- [ ] Original note passes through to first output port
- [ ] Harmony notes route to additional output ports (one voice per port)
- [ ] User can change key and mode during playback without stopping
- [ ] GUI displays active notes and current configuration
- [ ] Application compiles to single binary with no runtime dependencies

### Out of Scope

- Audio-to-MIDI conversion — removed to reduce complexity and dependencies
- Algorithmic melody generation — focus is live performance, not composition
- TUI/curses interface — replaced by native GUI
- Tkinter GUI — legacy Python UI not being ported
- MIDI file input/output — real-time only for v1
- Preset save/load — configure at runtime for now

## Context

**Reference implementation:** Existing Python codebase in this repo provides working harmony algorithms and architecture patterns. Key files:
- `main.py` lines 804-1028: Music theory functions (scale generation, interval calculation)
- `main.py` lines 1084-1199: Mode-based MIDI processing loop
- Harmony modes are pure functions that can be translated directly to Rust

**Platform:** macOS primary development target (CoreMIDI), with cross-platform support for Linux (ALSA) and Windows (WinMM) as secondary goals.

**MIDI architecture:** Input device → processing thread → multiple output ports. State tracking needed for motion-based modes (6, 7) which consider previous notes.

## Constraints

- **Language**: Rust — performance and single binary distribution
- **GUI**: egui/eframe — immediate mode, good for real-time apps, cross-platform
- **MIDI**: midir crate — cross-platform MIDI I/O, well-maintained
- **No audio dependencies**: Removing librosa/sounddevice/numpy eliminates heavy dependencies
- **Single binary**: Must compile to standalone executable with no runtime requirements

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Port to Rust instead of optimizing Python | Performance + distribution (no Python runtime needed) | — Pending |
| Use egui/eframe for GUI | Immediate mode suits real-time display, simple API, cross-platform | — Pending |
| Drop audio-to-MIDI feature | Reduces complexity, removes heavy dependencies, focus on core value | — Pending |
| Drop algorithmic generation | Scope reduction for v1, can add later if needed | — Pending |

---
*Last updated: 2026-01-28 after initialization*
