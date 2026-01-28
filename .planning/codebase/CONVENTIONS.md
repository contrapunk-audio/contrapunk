# Coding Conventions

**Analysis Date:** 2026-01-28

## Naming Patterns

**Files:**
- Module files use lowercase with underscores: `main.py`, `audio_to_midi.py`, `contrapunk_ui.py`
- No file extension conventions for classes

**Functions:**
- Use snake_case for all function names: `generate_melody()`, `detect_pitch()`, `list_and_choose_input_type()`
- Prefix helper/utility functions with descriptive action verbs: `find_nearest_diatonic_third()`, `get_scale_notes()`, `clear_terminal()`
- Callback functions use descriptive suffix: `audio_callback()`, `keyboard_input_thread()`

**Variables:**
- Use snake_case for variables and instance attributes: `self.samplerate`, `self.active_notes`, `command_queue`, `output_ports`
- Use clear, descriptive names: `amplitude`, `prev_magnitudes`, `note_on_threshold` (not abbreviated)
- Boolean flags use `is_` or action verb prefix: `self.running`, `is_onset`, `show_numbers`

**Classes:**
- Use PascalCase: `ContrapunkTUI`, `AudioToMidi`
- Descriptive class names indicating purpose or responsibility

**Constants:**
- Use UPPERCASE_WITH_UNDERSCORES for module-level constants: `NOTES`, `BASE_NOTE`, `CHORD_PROGRESSIONS`, `RHYTHM_PATTERNS`
- Grouped logically with comments above groups

## Code Style

**Formatting:**
- No explicit formatter configured (no `.prettierrc` or similar)
- Follows standard Python conventions (PEP 8 style observed)
- Consistent indentation with 4 spaces
- One blank line between top-level function definitions
- Two blank lines between class definition and first method

**Linting:**
- No linting configuration files present (no `.pylintrc`, `pyproject.toml`, `setup.cfg`)
- Code follows general PEP 8 style conventions without enforcement
- Line length varies but generally keeps to reasonable limits (under 120 characters)

**Comments:**
- Sparse but meaningful comments: `# Lower threshold for faster response`, `# Note tracking`
- Comments explain WHY, not WHAT: e.g., explaining algorithm choices like onset detection

**Docstrings:**
- All public functions and classes have docstrings using triple-quote format
- Docstrings are concise one-liners for simple functions: `"""Clear the terminal screen in a cross-platform way."""`
- More complex functions have multi-line docstrings with details:
  ```python
  def generate_harmony(melody, key, voice_number=1, harmony_mode=7):
      """Generate harmony for a given melody using selected harmony mode.
      harmony_mode options:
      1: Forward melody as-is
      2: Add diatonic thirds
      ...
      """
  ```

## Import Organization

**Order:**
1. Standard library imports (mido, random, sys, termios, tty, threading, queue, argparse, curses, sounddevice, numpy, time, contextlib, select, os)
2. Third-party library imports (sd for sounddevice, np for numpy)
3. Local imports (from audio_to_midi import AudioToMidi)

**Standard observed in files:**
- `main.py`: Imports at top, organized stdlib then third-party then local
- `audio_to_midi.py`: Imports stdlib first, then third-party (sounddevice, numpy, librosa, mido)
- `contrapunk_ui.py`: Imports stdlib first (tkinter, threading, queue), then local (AudioToMidi)

**No path aliases:** Project uses direct relative imports (`from audio_to_midi import AudioToMidi`)

## Error Handling

**Patterns observed:**
- Try-except blocks are used extensively (37 instances in main.py) for:
  - Queue operations: `except queue.Empty: continue`
  - Device/port operations: `except (ValueError, IndexError) as e:`
  - General exceptions with recovery: `except Exception as e: print(f"Error: {e}")`

**Error reporting:**
- Errors printed to console with descriptive messages: `print(f"Error sending melody note: {str(e)}")`
- UI errors shown via `tui.show_error()` for visual feedback
- Bare except clauses used in cleanup code to ensure resources close

**Exception handling in critical sections:**
- MIDI port operations wrapped individually to prevent cascade failures
- Audio streaming wrapped with comprehensive try-except-finally
- Resource cleanup guaranteed in finally blocks (port closing, stream stopping)

**Graceful degradation:**
- Errors logged but processing continues: `except Exception as e: continue`
- Not all errors are fatal; system attempts recovery

## Logging

**Framework:** No logging framework - uses `print()` and TUI `show_error()` method

**Patterns:**
- Status messages printed to console during initialization/setup
- Error messages use f-strings with error details
- Debug info printed when needed: `print(f"Device info: {device_info}")`
- TUI system has dedicated status display via `self.status_message` and `tui.show_error()`

**When to log:**
- Device selection/initialization
- MIDI port status changes
- Error conditions
- User actions (mode changes, key changes)

## Function Design

**Size:**
- Functions generally 10-50 lines
- Longer functions (100+ lines) are algorithmic generators or UI loops
- Examples: `monitor_audio_levels()` is 120 lines (complex audio monitoring), `generate_melody()` is 40 lines

**Parameters:**
- Functions accept 3-6 parameters typically
- Long parameter lists use defaults: `generate_melody(key, length=16, tempo=120, progression='I-IV-V-I', rhythm_pattern=[1.0, 1.0, 1.0])`
- No *args or **kwargs patterns observed; explicit parameters preferred

**Return values:**
- Single return values or tuples: `return frequency, confidence`
- Some functions return None implicitly: `def setup_colors(self):`
- Generator functions return sequences: `return melody` (list of tuples)
- Choice functions return indices: `return self.selected_index`

## Module Design

**Exports:**
- All public functions and classes available at module level
- No `__all__` declarations observed
- Single class per module where applicable: `AudioToMidi` in `audio_to_midi.py`

**Barrel files:**
- No barrel files or aggregation modules used
- Direct imports from specific modules: `from audio_to_midi import AudioToMidi`

**Class organization:**
- Related methods grouped by responsibility within class
- State tracking variables initialized in `__init__`
- Public methods followed by private helper methods (no leading underscore convention observed)
- Example in `ContrapunkTUI`: color setup, drawing methods, then interactive methods

## Type Hints

**Pattern:** Not used in codebase - no type annotations observed

**Return type inference:** Types can be inferred from docstrings and usage
- MIDI note numbers are int
- Frequencies are float
- Confidences are float (0.0-1.0)
- Harmonies/melodies are lists of tuples

## Class Patterns

**Initialization:**
- Instance attributes set explicitly in `__init__` with comments
- State variables tracked: `self.running`, `self.current_note`, `self.active_notes`
- Queue objects for thread-safe communication: `self.audio_queue`, `self.midi_queue`

**Method patterns:**
- Callbacks prefixed with context: `audio_callback()` vs. generic `callback()`
- Configuration methods: `setup_colors()`
- State update methods: `update_screen()`
- User interaction methods: `show_menu()`, `show_value_input()`, `show_error()`

---

*Convention analysis: 2026-01-28*
