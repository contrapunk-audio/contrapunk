# Testing Patterns

**Analysis Date:** 2026-01-28

## Test Framework

**Runner:** Not detected

**Assertion Library:** Not detected

**Current State:** No automated tests present in codebase

**Testing Infrastructure:** None configured
- No `pytest.ini`, `tox.ini`, `setup.py`, or test configuration files
- No test directories (`tests/`, `test/`)
- No test files (`*_test.py`, `test_*.py`, `*.spec.py`)

**Run Commands:** None established

## Test File Organization

**Location:** Not applicable - no tests exist

**Naming:** Not applicable - no tests exist

**Structure:** Not applicable - no tests exist

## Manual Testing Approach

**Current verification methods:**
- Interactive TUI testing during development (curses-based UI)
- Manual MIDI port selection and audio device verification
- Print statements for debug output during execution

**Observed in code:**
- Debug print statements: `print(f"Device info: {device_info}")`  in `monitor_audio_levels()`
- Status messages via TUI: `self.show_error(f"Status: {status}")` in audio monitoring
- Runtime assertions implicit in error handling

## Error Handling as Testing

**Pattern:** Extensive try-except blocks serve as runtime validation

Examples from `main.py`:
```python
try:
    val = int(value)
    if (min_val is None or val >= min_val) and (max_val is None or val <= max_val):
        return val
    else:
        self.show_error(f"Value must be between {min_val} and {max_val}")
except ValueError:
    self.show_error("Please enter a valid number")
```

Examples from `audio_to_midi.py`:
```python
try:
    audio_data = self.audio_queue.get(timeout=1.0)
    # processing
except queue.Empty:
    continue
```

**Areas with error handling:**
- MIDI message processing: 37 try-except blocks in main.py
- Device initialization and audio streaming
- Port operations and MIDI communication
- Queue operations with timeout handling
- User input validation (integer range checking)

## Components Requiring Testing

**High Priority - Audio Processing:**
- `AudioToMidi.detect_pitch()` - Autocorrelation pitch detection algorithm
  - Location: `audio_to_midi.py:33-66`
  - Inputs: audio data as numpy array
  - Outputs: frequency (float) and confidence (float)
  - Testable with synthetic audio (sine waves at known frequencies)

- `AudioToMidi.detect_onset()` - Spectral flux onset detection
  - Location: `audio_to_midi.py:68-78`
  - Inputs: magnitude spectrum
  - Outputs: boolean
  - Testable with stepped magnitude changes

**High Priority - Music Theory:**
- `generate_harmony()` - Harmony generation with multiple modes
  - Location: `main.py:414-458`
  - 7 different harmony modes to test independently
  - Inputs: melody (list of tuples), key (int), voice number, mode (1-7)
  - Outputs: harmony (list of tuples)
  - Each mode needs validation of interval rules

- Interval functions (find_nearest_diatonic_third, find_random_diatonic_below, etc.)
  - Locations: `main.py:810-1029`
  - 6+ interval calculation functions
  - Inputs: MIDI note number, key
  - Outputs: MIDI note number
  - Should verify correct intervals within key

**Medium Priority - Melody Generation:**
- `generate_melody()` - Generates melodies following progressions
  - Location: `main.py:376-412`
  - Inputs: key, length, tempo, progression, rhythm
  - Outputs: melody (list of note tuples)
  - Should verify output respects chord progressions

**Medium Priority - UI/TUI:**
- `ContrapunkTUI` class - Text UI using curses
  - Location: `main.py:56-347`
  - Harder to unit test (requires terminal context)
  - Could test with curses stubs or manual testing

**Low Priority - Utilities:**
- `get_scale_notes()` - Scale generation
  - Location: `main.py:804-808`
  - Simple calculation, unlikely to break
  - Input validation not critical

- Device listing/selection functions
  - Location: `main.py:590-777`
  - Depends on external MIDI/audio devices
  - Limited automation possible

## Testing Challenges

**Terminal-based UI:**
- `ContrapunkTUI` uses curses which requires terminal environment
- Cannot easily unit test without curses mocking
- Interactive nature makes automated testing difficult
- Recommendation: Either mock curses or use manual integration testing

**External Dependencies:**
- MIDI port availability varies by system
- Audio devices vary per machine
- Cannot guarantee test reproducibility across environments
- Recommendation: Mock `mido` and `sounddevice` for unit tests

**Threading/Async:**
- Audio processing uses threads and queues
- `process_audio()` runs in separate thread
- `keyboard_input_thread()` handles user input
- Recommendation: Use threading.Event and queue.Empty patterns for synchronization in tests

**Real-time Audio:**
- Audio streaming is real-time
- Timing-dependent behavior
- Callback-driven architecture
- Recommendation: Mock audio stream with pre-recorded data or synthetic signals

## Testable Components Without Dependencies

**Best candidates for unit testing:**
1. Scale/note calculation functions (pure functions)
2. Interval calculation functions (pure music theory logic)
3. Harmony generation algorithms (deterministic with seed)
4. MIDI message creation/manipulation

**Test approach:**
```python
# Example test structure for interval functions
def test_find_nearest_diatonic_third():
    # Test in C major
    key = 0
    note = 60  # C
    result = find_nearest_diatonic_third(note, key)
    # Should be 64 (E, a third above C)
    assert result == 64

def test_generate_harmony_mode_2():
    # Test diatonic thirds mode
    melody = [(60, 1.0, 80), (62, 1.0, 80)]  # C, D
    harmony = generate_harmony(melody, key=0, voice_number=1, harmony_mode=2)
    # Should be [E, F#] or similar thirds
    assert len(harmony) == 2
    assert harmony[0][0] == 64  # E is third above C
```

## Coverage Gaps

**Not tested (currently untested areas):**
- `AudioToMidi` pitch detection accuracy
- Onset detection reliability
- Counterpoint rule enforcement (mode 7)
- Complex harmony mode interactions
- MIDI threading and port communication
- UI state management under user input
- Audio monitoring visualization
- Error recovery under extreme conditions

**Risk Assessment:**
- Audio-to-MIDI conversion has high risk of subtle bugs (pitch/confidence thresholds)
- Counterpoint rules (mode 7) complex logic without validation
- Thread safety not verified (shared state between audio/MIDI threads)
- UI state can become inconsistent under rapid input

## Recommendations

**Immediate:**
1. Add unit tests for interval calculation functions (quick wins, pure logic)
2. Add unit tests for scale generation (trivial but validates music theory)
3. Add tests for harmony modes 1-5 (deterministic behavior)

**Short-term:**
1. Mock `mido` and `sounddevice` for integration tests
2. Create test fixtures for MIDI data (preset note sequences)
3. Test audio processing with synthetic signals (pre-generated numpy arrays)

**Long-term:**
1. Consider refactoring TUI to separate logic from curses rendering
2. Add threading tests for AudioToMidi class
3. Create full integration tests with mock devices

## Test Data Needs

**MIDI data fixtures:**
- Standard scale sequences (all 12 keys)
- Chord progressions (pre-generated note sequences)
- Test melodies in various keys and modes

**Audio data fixtures:**
- Sine wave samples at known frequencies (100Hz, 440Hz, 1000Hz)
- Complex audio with overlapping frequencies
- Silent audio for threshold testing
- Frequency sweeps for onset detection

**Sample code for test fixtures:**
```python
# Generate synthetic audio for testing
import numpy as np

def generate_sine_wave(frequency, duration=1.0, sample_rate=48000):
    """Generate test audio: pure sine wave at given frequency."""
    t = np.linspace(0, duration, int(sample_rate * duration))
    return np.sin(2 * np.pi * frequency * t).astype(np.float32)

# Test pitch detection
audio = generate_sine_wave(440)  # A4
frequency, confidence = detector.detect_pitch(audio)
assert abs(frequency - 440) < 5  # Within 5Hz tolerance
```

---

*Testing analysis: 2026-01-28*
