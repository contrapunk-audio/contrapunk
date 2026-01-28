# Technology Stack

**Analysis Date:** 2026-01-28

## Languages

**Primary:**
- Python 3.12.4 - Entire application codebase for MIDI processing, audio analysis, and UI

## Runtime

**Environment:**
- Python 3.12.4 (CPython)

**Package Manager:**
- pip (Python package manager)
- Lockfile: requirements.txt (present, minimal)

## Frameworks

**Core Audio/MIDI:**
- mido 1.3.3 - MIDI input/output handling and message processing
- python-rtmidi 1.5.8 - Real-time MIDI interface for device communication
- sounddevice 0.5.1 - Audio input capturing and streaming from audio interfaces
- librosa 0.10.2.post1 - Audio signal processing and pitch detection

**UI Framework:**
- tkinter (standard library) - GUI for `contrapunk_ui.py` (legacy)
- curses (standard library) - Terminal UI (TUI) for `main.py` current implementation

**Utilities:**
- numpy 1.26.4 - Numerical computations for audio processing and pitch detection

## Key Dependencies

**Critical:**
- mido 1.3.3 - Why it matters: Core dependency for all MIDI communication between input devices and output synthesizers. Handles note_on/note_off messages and channel management
- python-rtmidi 1.5.8 - Why it matters: Provides low-level MIDI port enumeration and real-time I/O access
- sounddevice 0.5.1 - Why it matters: Enables audio stream capture from audio interfaces for real-time audio input mode
- librosa 0.10.2.post1 - Why it matters: Provides pitch detection and onset detection algorithms for converting audio to MIDI

**Audio Processing:**
- numpy 1.26.4 - Arrays, signal processing, correlation-based pitch detection

## Configuration

**Environment:**
- No environment variables required
- Configuration is done through interactive TUI menu selection at runtime
- Device selection, key/mode selection, and output port configuration happen via command-line interface

**Build:**
- No build system (interpreted Python)
- Direct execution via `python main.py`

## Platform Requirements

**Development:**
- Python 3.12+
- MIDI drivers installed (varies by OS)
- Audio drivers for sounddevice support
- macOS/Linux/Windows compatibility (uses standard cross-platform libraries)

**Production:**
- macOS with CoreMIDI support (primary development platform)
- Linux with ALSA MIDI support
- Windows with Windows Multimedia MIDI support
- Python 3.12+ runtime
- Audio interface drivers for the target OS

---

*Stack analysis: 2026-01-28*
