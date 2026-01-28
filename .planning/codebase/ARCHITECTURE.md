# Architecture

**Analysis Date:** 2026-01-28

## Pattern Overview

**Overall:** Multi-input harmony generation system with layered processing

**Key Characteristics:**
- Multiple input modes (MIDI, audio, generated music, mode-based processing)
- Real-time harmony generation using 7 different algorithmic modes
- TUI-based user interaction via curses library
- Audio-to-MIDI conversion for non-digital instruments
- Polyphonic MIDI output to multiple ports simultaneously

## Layers

**Input Layer:**
- Purpose: Accept user input from various sources (MIDI devices, audio interface, or generated)
- Location: `main.py` lines 566-803 (input selection functions), `audio_to_midi.py` (audio conversion)
- Contains: Device selection, configuration gathering, audio capture
- Depends on: mido, sounddevice, librosa, numpy
- Used by: Processing layer

**TUI/Display Layer:**
- Purpose: Render terminal-based user interface for configuration and runtime monitoring
- Location: `main.py` lines 56-348 (ContrapunkTUI class)
- Contains: Menu rendering, audio level monitoring, status display, active note tracking
- Depends on: curses library
- Used by: Main orchestration function

**Processing Layer:**
- Purpose: Generate and process MIDI note sequences through harmony algorithms
- Location: `main.py` lines 376-1028 (generation and harmony functions)
- Contains: Melody generation, harmony generation (7 modes), mode-based MIDI processing
- Depends on: Input layer, music theory functions
- Used by: Output layer

**Music Theory Layer:**
- Purpose: Provide harmonic calculations and diatonic interval logic
- Location: `main.py` lines 804-1028 (scale/chord/interval functions)
- Contains: Scale generation, diatonic interval finding, consonance checking, counterpoint rules
- Depends on: Python standard library only
- Used by: Processing layer

**Output Layer:**
- Purpose: Send generated MIDI messages to output devices
- Location: `main.py` lines 460-564 (music playback functions), lines 1084-1199 (mode-based processing)
- Contains: MIDI port management, note-on/note-off sequencing, timing control
- Depends on: mido, Processing layer
- Used by: Entry points

## Data Flow

**MIDI Input Mode:**

1. User selects MIDI Input from main menu (TUI)
2. User selects input device and output ports (TUI)
3. `curses_main()` opens MIDI input port
4. Main loop: `inport.poll()` retrieves MIDI messages
5. Messages forwarded to first output port unmodified
6. TUI displays incoming messages
7. User presses 'q' to exit

**Audio Input Mode:**

1. User selects Audio Input from main menu (TUI)
2. User selects audio device and channel via TUI
3. User optionally monitors input levels (curses-based visualization)
4. AudioToMidi instance created with device configuration
5. Audio callback streams samples to AudioToMidi.process_audio()
6. AudioToMidi converts audio to MIDI using pitch detection and onset detection
7. MIDI messages output to selected ports
8. TUI displays current activity

**Generated Music Mode:**

1. User selects Generated Music from main menu (TUI)
2. User configures: key, tempo, output ports, harmony modes, chord progression, rhythm pattern (TUI)
3. Melody generated via `generate_melody()` following chord progression and rhythm
4. For each harmony voice: `generate_harmony()` creates counterpoint using selected mode
5. Both melody and harmonies sent simultaneously to respective output ports
6. Timing controlled by note duration and tempo
7. User commands (1-7 for mode change, k for key change, n for new melody, q to quit) processed from queue
8. New melody generated on user request, same key/progression maintained

**Mode-Based MIDI Processing:**

1. User selects Mode-based MIDI Processing from main menu (TUI)
2. User configures: input port, output ports, initial key, initial mode (TUI)
3. Main loop: `inport.poll()` retrieves MIDI messages
4. Original note sent to first output port
5. For each additional output port, harmony note calculated using current mode
6. Harmony note sent to that output port
7. User can change mode (1-7) and key (k) in real-time via keyboard commands
8. Motion tracking state reset when mode changes

**State Management:**
- Input tracking: `prev_input_notes` dict maps voice number to previous input note
- Output tracking: `prev_output_notes` dict maps voice number to previous output note
- Active notes: `tui.active_notes` dict tracks currently-playing notes for display
- Configuration: Stored in TUI object and passed to processing functions
- Harmony state: Mode number and key stored as module-level or function parameters

## Key Abstractions

**ContrapunkTUI:**
- Purpose: Encapsulates all terminal-based user interaction
- Examples: `main.py` lines 56-348
- Pattern: Class-based stateful wrapper around curses with helper methods for menu, input, display
- Methods: `show_menu()`, `show_value_input()`, `show_error()`, `draw_title()`, `update_screen()`, `run_audio_monitor()`, `run_mode_based_processor()`, `run_midi_processor()`, `run_music_player()`

**AudioToMidi:**
- Purpose: Converts audio samples to MIDI note events
- Examples: `audio_to_midi.py` lines 1-150+
- Pattern: Thread-safe class with audio callback, pitch detection, and MIDI output queue
- Key methods: `audio_callback()` (audio stream handler), `detect_pitch()` (autocorrelation-based), `detect_onset()` (spectral flux), `process_audio()` (main worker thread)

**Harmony Generation Functions:**
- Purpose: Generate counterpoint notes based on algorithmic rules
- Examples: `main.py` lines 810-1028
- Pattern: Pure functions that take input note/key/mode and return harmony note
- Key functions:
  - `find_nearest_diatonic_third()`: Returns note 2 scale positions above input
  - `find_nearest_diatonic_fourth()`: Returns note 3 scale positions above input
  - `find_random_diatonic_below()`: Random diatonic note below input
  - `find_random_diatonic_below_no_seconds()`: Random diatonic excluding seconds
  - `find_contrary_diatonic_below_no_seconds()`: Contrary motion logic
  - `find_strict_counterpoint_below()`: Complex scoring system for traditional rules

**Scale/Chord Helpers:**
- Purpose: Convert between scale degrees, MIDI notes, and pitch values
- Examples: `main.py` lines 804-809, 1069-1082
- Pattern: Pure functions operating on 12-note equal temperament
- Key functions: `get_scale_notes()`, `get_chord_notes()`

## Entry Points

**`curses_main(stdscr)`:**
- Location: `main.py` lines 1212-1631
- Triggers: Called by `curses.wrapper()` in main execution block at end of file
- Responsibilities: Main orchestration function that presents user menus and dispatches to appropriate mode (MIDI input, audio input, generated music, or mode-based processing)

**`main.py` execution block (end of file):**
- Location: `main.py` lines 1632+
- Triggers: When script is run directly
- Responsibilities: Entry point that initializes curses wrapper and calls `curses_main()`

**`AudioToMidi.process_audio()`:**
- Location: `audio_to_midi.py` lines 80-120+
- Triggers: Spawned in separate thread when audio input is selected
- Responsibilities: Continuously processes audio from queue, detects pitch, converts to MIDI notes

## Error Handling

**Strategy:** Try-catch blocks with user-friendly error messages displayed in TUI

**Patterns:**
- MIDI port errors: Caught and displayed via `tui.show_error()`
- Audio stream errors: Status checked in callbacks, error messages shown in display
- Pitch detection failures: Gracefully return None confidence scores
- Thread interruption: KeyboardInterrupt caught, cleanup attempted in finally blocks
- Invalid user input: ValueError/IndexError caught with retry loops in port selection

## Cross-Cutting Concerns

**Logging:** No dedicated logging system; debug output via `print()` statements and `tui.show_error()` for UI-level messages

**Validation:**
- MIDI port validation: Check port exists in mido.get_input/output_names()
- Audio device validation: Check device_id against sounddevice.query_devices()
- User input validation: Integer bounds checking in `show_value_input()`, port number validation in selection loops

**Authentication:** Not applicable; local device/port access only

**Threading:**
- Audio callback runs in sounddevice's internal thread
- AudioToMidi.process_audio() runs in user-spawned thread
- Keyboard input thread spawned for monitoring user commands during playback
- Thread-safe queues used for inter-thread communication

---

*Architecture analysis: 2026-01-28*
