# Codebase Structure

**Analysis Date:** 2026-01-28

## Directory Layout

```
contrapunk/
├── main.py                 # Entry point, orchestration, TUI, processing functions
├── audio_to_midi.py        # Audio capture and pitch-to-MIDI conversion
├── contrapunk_ui.py        # Alternative Tkinter-based GUI (not currently used)
├── requirements.txt        # Python package dependencies
├── README.md               # Project documentation and usage guide
├── .gitignore              # Git ignore rules
├── .planning/              # Planning documents directory
│   └── codebase/           # Architecture/structure analysis documents
└── templates/              # (empty) - reserved for future MIDI templates
```

## Directory Purposes

**Project Root:**
- Purpose: Main application code and documentation
- Contains: Python source files, package manifest, documentation
- Key files: `main.py`, `audio_to_midi.py`, `README.md`

**.planning/codebase/**
- Purpose: Architecture and structure analysis documents
- Contains: ARCHITECTURE.md, STRUCTURE.md, CONVENTIONS.md, TESTING.md (as written)
- Key files: Analysis documents for code navigation and planning

**templates/**
- Purpose: Reserved for future MIDI file templates or configuration files
- Contains: (currently empty)
- Status: Not yet utilized in current implementation

## Key File Locations

**Entry Points:**
- `main.py` (lines 1632+): Python entry point, calls `curses.wrapper(curses_main)`
- `curses_main()` in `main.py` (lines 1212-1631): Main orchestration function presenting menu and dispatching to modes

**Configuration:**
- `requirements.txt`: Package dependencies (mido, python-rtmidi, sounddevice, numpy, librosa)
- Hard-coded configuration in `main.py`: Musical constants (BASE_NOTE, NOTES), chord progressions, rhythm patterns

**Core Logic:**
- **Harmony/Counterpoint:** `main.py` lines 810-1028
  - Scale operations: `get_scale_notes()`
  - Diatonic intervals: `find_nearest_diatonic_third()`, `find_nearest_diatonic_fourth()`, etc.
  - Strict counterpoint: `find_strict_counterpoint_below()`
- **Music Generation:** `main.py` lines 376-459
  - Melody generation: `generate_melody()`
  - Harmony generation: `generate_harmony()`
- **Audio-to-MIDI:** `audio_to_midi.py` lines 1-150+
  - Pitch detection: `detect_pitch()` (autocorrelation)
  - Onset detection: `detect_onset()` (spectral flux)
  - Audio processing: `process_audio()` (worker loop)
- **TUI:** `main.py` lines 56-348
  - Class: `ContrapunkTUI` with menu, display, and input methods

**Testing:**
- No dedicated test files; testing is manual through TUI

**Utilities:**
- Helper functions for MIDI port listing: `list_and_choose_midi_ports()`, `list_and_choose_output_ports()` (lines 590-802)
- Terminal utilities: `clear_terminal()`, `print_status()`, `print_menu()` (lines 349-374)
- Keyboard handling: `get_key_nonblocking()`, `keyboard_input_thread()` (lines 1030-1067)

## Naming Conventions

**Files:**
- Snake_case: `audio_to_midi.py`, `contrapunk_ui.py`
- CamelCase reserved for: `README.md`, class names only

**Functions:**
- Snake_case for all functions: `generate_melody()`, `find_nearest_diatonic_third()`, `process_mode_based_midi()`
- Descriptive names starting with verb: `generate_*`, `find_*`, `list_and_choose_*`, `detect_*`, `process_*`
- Underscore prefix for helper methods: None used; all methods are public

**Classes:**
- PascalCase: `ContrapunkTUI`, `AudioToMidi`
- Descriptive names indicating purpose

**Variables:**
- Local variables: snake_case (note, scale_notes, harmony_note, prev_input_notes)
- Constants: ALL_CAPS (BASE_NOTE, NOTES, CHORD_PROGRESSIONS, RHYTHM_PATTERNS)
- Dictionary/list names: descriptive plural or camelCase (active_notes, prev_input_notes, harmony_modes)

**Types:**
- MIDI note values: Integers 0-127 (type: int)
- Frequencies: Floats in Hz (type: float)
- Musical durations: Floats representing beat fractions (e.g., 1.0 = quarter note at tempo)
- Keys: Integer 0-11 representing chromatic positions from C (type: int)
- Modes: Integer 1-7 representing harmony algorithm selection (type: int)

## Where to Add New Code

**New Harmony Mode:**
1. Add new function in `main.py` similar to `find_*` pattern (e.g., `find_new_mode_harmony()`)
2. Location: `main.py` after line 1028, before `get_key_nonblocking()`
3. Add mode number to `generate_harmony()` elif chain (lines 434-451)
4. Add mode number to `process_mode_based_midi()` elif chain (lines 1151-1164)
5. Update mode selection menus in `curses_main()` (lines 1316-1337, 1516-1524)
6. Update mode names in mode selection displays (lines 246-254, 318-326, 1330-1337)

**New Input Type:**
1. Add new class or functions in separate file if complex (e.g., `midi_file_input.py`)
2. Implement input device selection logic following pattern in `list_and_choose_midi_ports()`
3. Add input type option to `curses_main()` menu (line 1223)
4. Add branch in `curses_main()` to handle new input type
5. Return MIDI messages as mido.Message objects to maintain compatibility with output layer

**New Output Feature:**
1. Add to `play_generated_music()` (lines 460-564) or `process_mode_based_midi()` (lines 1084-1199)
2. Ensure MIDI port compatibility with mido's port.send(msg) interface
3. Track state in TUI object (tui.active_notes) or function-local variables

**Utilities/Helpers:**
- Scale/chord operations: Add near `get_scale_notes()` (line 804)
- Keyboard/IO helpers: Add near `get_key_nonblocking()` (line 1030)
- List/selection helpers: Add near `list_and_choose_midi_ports()` (line 590)

## Special Directories

**`.git/`:**
- Purpose: Git version control metadata
- Generated: Yes (automatically)
- Committed: Yes (included in repo)

**`__pycache__/`:**
- Purpose: Python bytecode cache
- Generated: Yes (automatically by Python)
- Committed: No (listed in .gitignore)

**`.planning/codebase/`:**
- Purpose: Architecture and planning documentation
- Generated: Yes (by GSD tools)
- Committed: Yes (included in repo)

## Import Organization

**main.py imports (lines 1-16):**
```python
# Standard library - system/OS
import mido
import random
import sys
import termios
import tty
import threading
import queue
import argparse
import curses
from audio_to_midi import AudioToMidi
# External packages - audio/MIDI
import sounddevice as sd
import numpy as np
# Standard library - timing
import time
import contextlib
import select
import os
```

**Order:** Standard library → Local modules → External packages (mido, numpy, librosa)

**audio_to_midi.py imports (lines 1-6):**
```python
# External packages
import sounddevice as sd
import numpy as np
import librosa
import mido
# Standard library
import queue
import threading
```

**No import aliases in use except:** `sd` for sounddevice

## Module Dependencies

**main.py:**
- Imports: mido, curses, numpy, sounddevice, audio_to_midi (local)
- Exports: ContrapunkTUI class, multiple functions (no explicit __all__)
- Purpose: Main orchestration and algorithm implementations

**audio_to_midi.py:**
- Imports: mido, numpy, librosa, sounddevice, threading, queue
- Exports: AudioToMidi class
- Purpose: Audio input abstraction and pitch detection

**contrapunk_ui.py:**
- Imports: tkinter, mido, threading, queue, audio_to_midi (local)
- Status: Present but unused (not imported or called from main.py)
- Purpose: Alternative Tkinter GUI (legacy/experimental)

---

*Structure analysis: 2026-01-28*
