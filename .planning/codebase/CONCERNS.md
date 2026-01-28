# Codebase Concerns

**Analysis Date:** 2026-01-28

## Tech Debt

**Monolithic Main File:**
- Issue: Main application logic (1,686 lines) is contained in a single file `main.py`, mixing UI, MIDI processing, audio analysis, music theory, and threading code
- Files: `main.py` (entire file)
- Impact: Difficult to test individual components, high cognitive load for modifications, poor code reuse across UI implementations
- Fix approach: Extract into modules: `midi_processor.py`, `harmony_generator.py`, `music_theory.py`, `tui.py` with clear separation of concerns

**Code Duplication Between UIs:**
- Issue: Identical MIDI processing logic duplicated between `main.py` (curses TUI) and `contrapunk_ui.py` (tkinter UI)
- Files: `main.py` (lines 1084-1210), `contrapunk_ui.py` (lines 195-290)
- Impact: Bug fixes must be applied in two places; feature additions require code duplication
- Fix approach: Extract shared MIDI processing logic to `midi_processor.py`, import and reuse in both UIs

**Duplicate Harmony Generation Functions:**
- Issue: Seven different harmony mode implementations scattered throughout with copy-pasted pattern
- Files: `main.py` (lines 414-458, 810-929, 931-1028)
- Impact: Inconsistent behavior across different contexts; changes to harmony algorithms require updating multiple functions
- Fix approach: Create unified `HarmonyGenerator` class with configurable modes instead of separate functions

**No Configuration Management:**
- Issue: Magic numbers and thresholds hardcoded throughout: tempo settings (line 533), audio thresholds (audio_to_midi.py lines 20-23), buffer sizes (main.py line 185, 512), MIDI note ranges (audio_to_midi.py lines 54-55)
- Files: `main.py`, `audio_to_midi.py`, `contrapunk_ui.py`
- Impact: Users cannot tune parameters without code edits; inconsistent defaults across modules
- Fix approach: Create `config.py` with configurable settings; load from config file

## Known Bugs

**Audio Monitor Display Corruption:**
- Symptoms: Character encoding and buffer management issues in audio monitor display (main.py lines 178-239)
- Files: `main.py` (lines 178-239), specifically line 215 uses Unicode characters ("█", "░") that may not render correctly in all terminals
- Trigger: Running audio monitor on non-UTF-8 terminals
- Workaround: Run only on UTF-8-enabled terminals; no fallback for ASCII-only environments

**Incomplete Cleanup on Exception:**
- Symptoms: If exception occurs during MIDI port setup, ports may remain open and leak resources
- Files: `main.py` (lines 1255-1257, 1372-1476), `contrapunk_ui.py` (lines 144-159)
- Trigger: Selecting invalid MIDI port or port becoming unavailable mid-initialization
- Workaround: Manual restart of application required
- Fix approach: Use try-finally blocks or context managers consistently for all resource allocation

**Mode Tracking State Not Reset Properly:**
- Symptoms: When switching between harmony modes (especially modes 5-7 which track motion), previous motion state persists incorrectly
- Files: `main.py` (lines 1117-1119, 1651-1653)
- Trigger: Rapid mode switching during playback
- Impact: Harmony output may violate intended counterpoint rules temporarily
- Fix approach: Ensure `prev_input_notes` and `prev_output_notes` are cleared atomically when mode changes

**Keyboard Input Thread Terminal State Not Restored:**
- Symptoms: Terminal may be left in raw mode if exception occurs in keyboard thread
- Files: `main.py` (lines 1050-1067)
- Trigger: Keyboard input thread exception (lines 1061)
- Impact: Terminal becomes unusable; user must manually run `reset` command
- Workaround: Run `reset` command in terminal
- Fix approach: Use signal handlers to ensure terminal restoration on all exit paths

**Audio Queue Unbounded:**
- Symptoms: If audio processing thread falls behind audio input, queue grows unbounded, consuming all memory
- Files: `audio_to_midi.py` (line 14, 31)
- Trigger: CPU-heavy operations or high sample rate with small buffer
- Impact: Memory leak leading to application crash
- Fix approach: Use bounded queue with maxsize parameter (e.g., `queue.Queue(maxsize=100)`)

## Error Handling Issues

**Bare Except Clauses:**
- Problem: `except: pass` statements ignore all exceptions including SystemExit and KeyboardInterrupt
- Files: `main.py` (lines 587, 1067, 1683)
- Impact: Difficult to debug; suppresses critical errors
- Recommendation: Use specific exception types (e.g., `except ValueError:`, `except OSError:`)

**Print-Based Error Reporting:**
- Problem: Errors printed to stdout mixed with functional output; no consistent error logging
- Files: `audio_to_midi.py` (line 28), `main.py` (lines 510-512, 528-530, 546-548, 602, 756-757)
- Impact: Users cannot capture or filter error logs; hard to debug in production
- Recommendation: Use `logging` module with configurable log levels

**Unhandled Queue Exceptions:**
- Problem: `queue.Empty` caught only in `audio_to_midi.py` (line 117), but not in main application
- Files: `main.py` (lines 480-481, 1387-1388, 1550-1551)
- Impact: Queue timeout could cause unexpected behavior; no graceful handling
- Fix approach: Add explicit timeout handling with appropriate user feedback

**Unchecked Index Access:**
- Problem: Direct index access without bounds checking in several places
- Files: `main.py` (lines 395-397, 825-826, 850-851)
- Impact: IndexError possible if scale notes accessed with invalid index
- Fix approach: Add bounds checking or use safer list operations

## Performance Bottlenecks

**Inefficient Pitch Detection:**
- Problem: Autocorrelation in `audio_to_midi.py` (lines 34-66) uses naive peak finding (O(n))
- Files: `audio_to_midi.py` (lines 34-66)
- Cause: Full autocorrelation computed for every audio frame without optimization
- Improvement path: Use FFT-based autocorrelation or YIN algorithm; implement caching of spectral data

**Repeated Scale Calculation:**
- Problem: `get_scale_notes()` recalculated for every note in harmony generation
- Files: `main.py` (called from lines 414, 810, 835, 860, 876, 894, 931, 1102, 1150, 1569)
- Cause: Function not memoized despite deterministic output
- Improvement path: Cache scale notes by key; memoize with `functools.lru_cache`

**Linear Search in Harmony Modes:**
- Problem: Finding diatonic intervals uses sequential list searches
- Files: `main.py` (lines 817-823, 843-848, 909-916)
- Cause: Unoptimized search for scale position
- Improvement path: Use dictionary lookup for note-to-scale-position mapping

**Full MIDI Reset on Shutdown:**
- Problem: Sending note_off for all 128 notes on 16 channels (~2,048 messages) on every shutdown
- Files: `main.py` (lines 1470-1472, 1659-1661)
- Impact: Audible clicks/pops on audio hardware; unnecessary latency on shutdown
- Improvement path: Track only active notes; send note_off only for those

**Blocking Terminal Operations:**
- Problem: Audio monitoring runs synchronous curses operations in audio callback thread
- Files: `main.py` (lines 187-221)
- Cause: All terminal updates happen in audio thread instead of being queued
- Impact: Real-time audio processing can be interrupted by terminal rendering
- Improvement path: Queue display updates; render in separate thread

## Fragile Areas

**Audio Device Selection Logic:**
- Files: `main.py` (lines 720-777)
- Why fragile: Tries to parse device info string with fragile string splitting (line 757)
- Safe modification: Parse device info at selection time, store as tuple instead of string; validate device exists before use
- Test coverage: No validation that selected device still exists when referenced
- Risk: Device id "12: Microphone" split by ":" could fail with unexpected format

**Harmony Mode State Machine:**
- Files: `main.py` (lines 241-282, 1084-1210, 1549-1654), `contrapunk_ui.py` (lines 195-290)
- Why fragile: Complex state tracking (`prev_input_notes`, `prev_output_notes`, `active_notes`) must be synchronized across three different code paths
- Safe modification: Create explicit `HarmonyState` class to encapsulate state; test all mode transitions
- Test coverage: No tests for mode transition edge cases (rapid switches, incomplete note releases)
- Risk: State inconsistency could produce invalid MIDI output (missing note-offs, duplicate note-ons)

**Terminal State Management:**
- Files: `main.py` (lines 1030-1068, 1680-1683)
- Why fragile: Curses and raw terminal mode can be left active if exceptions occur
- Safe modification: Use context managers for terminal mode; add signal handlers
- Test coverage: No recovery testing for interrupted terminal operations
- Risk: Terminal becomes unusable, requires manual `reset`

**MIDI Port Lifecycle:**
- Files: `main.py` (lines 464-565, 1255-1300, 1372-1476, 1542-1668), `contrapunk_ui.py` (lines 144-150, 175-185)
- Why fragile: Multiple exit paths (normal completion, exceptions, user quit) must close ports
- Safe modification: Use context managers consistently; implement `__enter__`/`__exit__` for port groups
- Test coverage: No tests for port closing with pending MIDI messages
- Risk: Ports left open consume system resources; prevents port use in other applications

**Curses-based TUI State:**
- Files: `main.py` (lines 56-77, 113-119, 178-239)
- Why fragile: TUI state stored in instance variables; refresh logic scattered
- Safe modification: Consolidate refresh logic; implement dirty flag pattern
- Test coverage: No unit tests for TUI (requires manual terminal testing)
- Risk: Screen corruption under high frame rate or terminal resize

## Scaling Limits

**Single-threaded Audio Processing:**
- Current capacity: ~2,000 samples/second (48kHz / 512 buffer) with current autocorrelation
- Limit: Cannot process multiple audio channels simultaneously
- Scaling path: Implement per-channel processing threads; use vectorized NumPy operations for pitch detection

**Memory with Many Notes:**
- Current capacity: Raw buffer limited to 100 samples (audio_to_midi.py line 636)
- Limit: Very limited history for motion detection
- Scaling path: Configurable buffer size; implement circular buffer to avoid reallocation

**Terminal Rendering:**
- Current capacity: ~30-50 FPS practical limit for curses
- Limit: Cannot update faster without flickering
- Scaling path: Use triple buffering; implement dirty rectangle updates

**MIDI Port Limit:**
- Current capacity: Can open up to ~8 output ports (hardcoded in UI spinbox)
- Limit: Some systems support 16-32 ports; arbitrary limit prevents use with large orchestrations
- Scaling path: Remove hardcoded limits; validate against system capabilities

## Dependencies at Risk

**librosa (machine learning library):**
- Risk: Heavy dependency (~300MB) for single FFT function that NumPy provides
- Impact: Slow installation; version conflicts with other audio libraries
- Migration plan: Replace `librosa` with `numpy.fft` equivalent; already imported separately

**mido (MIDI library):**
- Risk: Python-rtmidi backend may not work on all platforms (especially Windows with non-standard MIDI setups)
- Impact: Application requires python-rtmidi system dependency
- Current status: Stable and actively maintained; no immediate migration needed
- Monitoring: Keep updated; test on multiple platforms

**sounddevice:**
- Risk: Depends on libsndfile system library which may not be installed
- Impact: Audio input fails without clear error message on some systems
- Recommendation: Add installation documentation for required system libraries (libsndfile, PortAudio)

## Security Considerations

**No Input Validation on MIDI Messages:**
- Risk: Malformed MIDI data not validated before routing
- Files: `main.py` (lines 290-306, 1125-1200, 1550-1640)
- Current mitigation: mido library does basic validation
- Recommendations: Add explicit validation of note range (0-127) and velocity (0-127); add message type whitelist

**Command Queue for Keyboard Input:**
- Risk: Keyboard input directly affects mode/key without validation
- Files: `main.py` (lines 1050-1067, 1113-1122)
- Current mitigation: Input limited to specific characters ('1'-'7', 'q', 'n', 'k')
- Recommendations: Already good; maintain whitelist approach

**No Resource Limits:**
- Risk: Unbounded queue growth in audio processing could exhaust memory
- Files: `audio_to_midi.py` (line 14)
- Current mitigation: None
- Recommendations: Add maxsize to queue; implement backpressure

## Test Coverage Gaps

**No Unit Tests:**
- What's not tested: Harmony generation functions, pitch detection accuracy, music theory calculations
- Files: `main.py` (lines 376-412, 414-458, 810-1028), `audio_to_midi.py` (lines 33-78)
- Risk: Bugs in harmony algorithms not caught until runtime; changes may break music theory assumptions
- Priority: **High** - Harmony generation is core functionality

**No Integration Tests:**
- What's not tested: Full MIDI loop (input → processing → output), mode switching during playback
- Files: `main.py` (lines 283-311, 1084-1210), `contrapunk_ui.py` (lines 195-290)
- Risk: State synchronization bugs between components not detected
- Priority: **High** - User-visible behavior

**No Terminal/UI Tests:**
- What's not tested: Curses TUI rendering, error display, menu navigation
- Files: `main.py` (lines 56-140)
- Risk: UI bugs only found through manual testing; difficult to regression test
- Priority: **Medium** - Less critical but improves user experience

**No Performance Tests:**
- What's not tested: Pitch detection latency, real-time responsiveness under load
- Files: `audio_to_midi.py` (lines 33-99)
- Risk: Performance regressions not detected; audio sync issues
- Priority: **Medium** - Important for real-time audio quality

**No Exception Handling Tests:**
- What's not tested: Port errors, device disconnection, corrupted MIDI messages
- Files: `main.py` (lines 510-548, 756-757), `contrapunk_ui.py` (lines 172-173)
- Risk: Application stability under error conditions unknown
- Priority: **Medium** - Affects reliability

## Missing Critical Features

**No Undo/Redo:**
- Problem: Once a note is played, cannot recall previous harmony generation
- Blocks: Composing and editing with the tool; users cannot experiment

**No MIDI File Input:**
- Problem: Must play melodies in real-time; cannot load pre-recorded sequences
- Blocks: Using application with DAW workflows; batch processing
- Note: Listed in ideas (line 72) but not implemented

**No Persistent State:**
- Problem: All settings (key, mode, port selections) lost on exit
- Blocks: Reproducible workflows; users must reconfigure on every launch

**No Audio Output:**
- Problem: Generated harmony only sends MIDI; no audio playback without external synthesizer
- Blocks: Users without MIDI hardware; demo/preview functionality

**No Harmony Editing:**
- Problem: Cannot adjust generated harmony after it's created
- Blocks: Fine-tuning compositions; fixing errors in generation

**No Preset System:**
- Problem: Cannot save/load favorite key+mode combinations
- Blocks: Fast switching between musical styles

---

*Concerns audit: 2026-01-28*
