# Codebase Concerns

**Analysis Date:** 2026-04-04

## Tech Debt

**Panics in Hot Audio Loop:**
- Issue: Multiple `unwrap()` calls on Mutex locks in real-time audio callback and routing thread
- Files: `src/guitar_bridge.rs:77`, `src/commands/engine.rs:326-329`, `src/commands/engine.rs:454-481`, `src/commands/engine.rs:531-536`
- Impact: If any Mutex is poisoned (panic in another thread holding the lock), the entire audio pipeline hangs or crashes instead of gracefully degrading. Real-time audio is unforgiving—panics cause clicks, pops, or complete audio dropout.
- Fix approach: Replace `unwrap()` with error handling that logs and either skips the frame or falls back to safe defaults. Use `lock().ok()` patterns with logging.

**Hardcoded Default Audio Configuration:**
- Issue: `set_guitar_config()` in `src/commands/guitar.rs:47` hardcodes `sample_rate = 48000` initially, then overwrites it from the actual device. This is fragile—if the bridge fails to update it, audio processing uses the wrong sample rate.
- Files: `src/commands/guitar.rs:47`
- Impact: Latency, buffer size, and filter calculations all depend on correct sample rate. Mismatch causes distortion, clicks, or frequency aliasing.
- Fix approach: Delay config creation until the audio device is opened, so the sample rate is known upfront. Pass the real sample rate from `GuitarBridge::new()` back to the command handler.

**Manual Port Index Management:**
- Issue: Voice-to-output routing uses raw `Vec<usize>` indices that could exceed `num_outputs`. Bounds checking exists but is scattered and error-prone.
- Files: `src/commands/engine.rs:487-492`, `src/commands/engine.rs:545-550`
- Impact: If a note's port index exceeds available outputs, it silently drops notes or sends to the wrong device. Difficult to debug in a live performance.
- Fix approach: Create a `PortMap` struct that validates bounds at construction and panics/errors clearly if misconfigured.

**Unused `_chord_name` Parameter:**
- Issue: `handle_note_off()` in `src/commands/engine.rs:512-565` takes a `_chord_name` parameter that is never used (name starts with `_` to suppress warning).
- Files: `src/commands/engine.rs:524`
- Impact: Dead code adds cognitive load and suggests incomplete refactoring. Might indicate a feature that was started but not completed.
- Fix approach: Remove the parameter entirely and clean up all call sites.

## Known Issues

**MIDI Routing Not Cancellable (Hard Stop Required):**
- Issue: The router thread runs a blocking `recv_timeout()` loop that checks `stop_signal` with `Ordering::SeqCst`. No graceful shutdown—the thread must finish processing or timeout.
- Files: `src/commands/engine.rs:284-320`
- Impact: When `stop_routing()` is called, the UI thinks it's stopped but the router thread might be stuck processing a message for up to 5ms (recv timeout). Multi-start-stop cycles could spawn orphaned threads.
- Trigger: Call `start_routing()` immediately after `stop_routing()` without waiting.
- Workaround: UI should debounce stop/start with a ~10ms delay, or add a join handle to wait for thread completion before returning success.

**Guitar Bridge Device Selection Fallback Not Signaled to UI:**
- Issue: If the requested device isn't found, `GuitarBridge::new()` silently falls back to default and logs to `eprintln!` (terminal only, not sent to UI).
- Files: `src/guitar_bridge.rs:42-44`
- Impact: User selects "Audient iD14" but it's not found—the app silently uses default device without telling the user. They debug for minutes wondering why it's recording from the wrong input.
- Trigger: Disconnect/rename the selected audio device, then start routing.
- Workaround: UI should periodically check `list_audio_devices()` to detect device changes and warn if selection is stale.

**No Validation of Guitar Channel Index:**
- Issue: `set_guitar_device()` accepts any `channel` value without checking if it exists on the device.
- Files: `src/commands/guitar.rs:20-28`
- Impact: If user sets `channel: 5` on a stereo device (2 channels), `guitar_bridge.rs:72` will silently return 0.0 for that frame. Audio drops.
- Fix approach: Validate channel index against `supported_config.channels()` at config time, or handle gracefully in the audio callback.

**Chord Name Computed from All Sounding Notes Without Voicing Context:**
- Issue: `handle_note_on()` computes chord name from union of input and harmony notes, but doesn't account for which notes are actually routed to which outputs.
- Files: `src/commands/engine.rs:473-483`
- Impact: If voices are missing or sent to muted outputs, the displayed chord is misleading. User sees "C major" but only certain notes are audible.
- Fix approach: Track which notes were successfully sent to outputs, then compute chord from those only.

## Fragile Areas

**Router Thread State Synchronization:**
- Files: `src/commands/engine.rs` (entire router function 214-357)
- Why fragile: Uses multiple `Arc<Mutex<>>` instances updated from both router thread and Tauri command threads without formal synchronization. Race conditions could occur:
  - `input_notes`, `harmony_notes`, `borrowed_notes`, `chord_name` are updated in `handle_note_on/off` (router thread) and read in `get_note_state` (command handler)
  - No explicit ordering guarantees; if router thread is slow, UI shows stale state
  - Mutex poisoning on panic leaves data permanently inaccessible
- Safe modification: Use an event channel (instead of shared Mutex state) to push updates from router to UI, or add explicit versioning/snapshots.
- Test coverage: No unit tests for concurrency or state consistency.

**GuitarBridge-to-Router Communication:**
- Files: `src/guitar_bridge.rs`, `src/commands/engine.rs:241-254`
- Why fragile: `GuitarBridge` sends MIDI bytes through an `mpsc::Sender` that must live for the entire routing session. If the sender is dropped early (e.g., bridge panics), messages silently fail. The router checks for `Disconnected` but only after a 5ms timeout.
- Safe modification: Add a heartbeat or explicit lifecycle signal so router knows if bridge is alive.
- Test coverage: No integration test for audio → MIDI event pipeline.

**Manual Engine Config Capture in start_routing:**
- Files: `src/commands/engine.rs:85-116`
- Why fragile: Copies engine configuration into a tuple before spawning router. If engine config is updated after `start_routing` is called but before the router thread reads from the tuple, the UI and router are out of sync. This is not truly a concern as updates during routing don't apply anyway, but it's not clearly documented.
- Safe modification: Add a comment explaining that config is snapshotted at start and UI updates don't affect active routing.

## Performance Bottlenecks

**Mutex Contention on Note State:**
- Problem: Every MIDI message (potentially 100+ per second) locks 4 Mutexes sequentially to update note state in `handle_note_on/off`
- Files: `src/commands/engine.rs:453-484`, `src/commands/engine.rs:530-541`
- Cause: Separate Mutexes for `input_notes`, `harmony_notes`, `borrowed_notes`, `chord_name` force lock-unlock cycles
- Improvement path: Combine into a single `NotesSnapshot` struct wrapped in one Mutex, or use lock-free data structures (e.g., `parking_lot`).

**Chord Display Computed Every NoteOn:**
- Problem: `handle_note_on` recomputes the full chord display string by iterating union of all sounding notes
- Files: `src/commands/engine.rs:478-482`
- Cause: Union is computed with heap allocation (`HashSet::union`), then passed to `chord_display_with_analysis()`
- Improvement path: Cache the last computed chord and only recalculate when the set of notes actually changes.

**Event Emission at Fixed 30fps Despite Potential Lower Update Rate:**
- Problem: Router emits `note-update` event every 33ms regardless of how many MIDI messages arrived
- Files: `src/commands/engine.rs:322-342`
- Cause: Uses wall-clock `Instant::now().elapsed()` to throttle, which means 1.2x overhead checking on every loop iteration
- Improvement path: Use a tick counter or frame counter instead of timing.

## Scaling Limits

**Maximum Voice Count Not Enforced:**
- Problem: `HarmonyEngine` voice count is set via `set_voice_count()` based on MIDI output count, but there's no upper bound. A system with 16 MIDI outputs could create 16 simultaneous voices, overwhelming some DAWs or synthesis engines.
- Impact: If a user somehow connects 100 outputs (intentionally or by accident), memory and CPU usage spike.
- Scaling path: Document and enforce a reasonable max (e.g., 8 voices), or add a command-line override.

**Single-Threaded Router Loop Bottleneck:**
- Problem: All MIDI processing, humanization, and output routing happens in one thread that blocks on `recv_timeout()`
- Impact: With many simultaneous notes and complex voice leading, jitter increases. No CPU parallelization.
- Scaling path: Decouple input (recv), processing (engine), and output (send) into separate threads with lock-free queues (see `crossbeam` crate).

## Security Considerations

**Device Name Input Not Validated:**
- Risk: `set_guitar_device()` accepts any device name string and passes it to `cpal`. Malformed or extremely long strings could cause issues.
- Files: `src/commands/guitar.rs:20-28`
- Current mitigation: `GuitarBridge::new()` attempts to find the device by substring match; if not found, falls back to default.
- Recommendations: Add length limit (e.g., max 256 chars) and test with pathological inputs. Consider using device index instead of name for robustness.

**No Input Validation on Preset Names:**
- Risk: `save_preset()` accepts a preset name string with no validation. Could contain path traversal characters if the preset manager saves to disk.
- Files: `src/commands/presets.rs:76`
- Current mitigation: Depends on `PresetManager` implementation (not in this file).
- Recommendations: Validate preset name is alphanumeric + underscores, max 64 chars. Check upstream PresetManager for file I/O safety.

**MIDI Bytes Parsed Without Validation:**
- Risk: `process_midi_message()` parses raw `&[u8]` using `wmidi::MidiMessage::try_from()`. Malformed messages fall back to passthrough.
- Files: `src/commands/engine.rs:371-376`
- Current mitigation: `try_from()` returns `Err`, which is caught and the message is sent to first output unchanged (passthrough behavior).
- Recommendations: Log when parse fails or add a metric to track malformed message count. Passthrough is safe but silent failures are hard to debug.

## Test Coverage Gaps

**No Unit Tests for Router Thread Logic:**
- What's not tested: `run_tauri_router()`, `handle_note_on()`, `handle_note_off()`, state synchronization
- Files: `src/commands/engine.rs:214-566`
- Risk: Critical real-time logic is never validated. Changes to humanization or port mapping could break silently.
- Priority: High—this is the core of the application.

**No Integration Tests for Guitar → MIDI Pipeline:**
- What's not tested: `GuitarBridge::new()` → audio capture → DSP → MIDI generation → routing
- Files: `src/guitar_bridge.rs`, entire audio integration
- Risk: A regression in pitch detection or onset detection breaks the main feature without warning.
- Priority: High—this is the unique selling point.

**No Tests for MIDI Device Enumeration:**
- What's not tested: `list_audio_devices()`, device fallback behavior, device disconnection scenarios
- Files: `src/commands/guitar.rs:79-85`
- Risk: Device enumeration could fail on certain systems or configurations without being caught.
- Priority: Medium—mostly edge cases, but affects user setup.

**No Tests for Preset Load/Save:**
- What's not tested: Loading a nonexistent preset, saving with duplicate name, applying preset to engine
- Files: `src/commands/presets.rs:42-98`
- Risk: Preset operations could partially fail or corrupt state without being detected.
- Priority: Medium—less critical than routing, but users rely on presets.

**No Concurrency Tests:**
- What's not tested: Rapid `start_routing()`/`stop_routing()` cycles, simultaneous MIDI input and preset changes, state races
- Files: All of `src/commands/engine.rs` and interaction with `src/state.rs`
- Risk: Race conditions only manifest under load or on high-core-count systems. Could be stable in dev, crash in production.
- Priority: High—critical for stability.

## Untested Error Paths

**Audio Device Failures:**
- What happens: If `build_input_stream()` fails, `GuitarBridge::new()` returns a `String` error. The calling code in `start_routing()` propagates it, but the UI sees a generic error message.
- Files: `src/guitar_bridge.rs:65-92`
- Better: Include device name and specific error type in error message.

**MIDI Connection Failures:**
- What happens: If `connect_input()` fails, `run_tauri_router()` returns `Err`, and the router thread exits silently.
- Files: `src/commands/engine.rs:250-254`
- Better: The error should bubble up and `start_routing()` should mark the session as failed, not partially active.

**Preset Manager Lock Failures:**
- What happens: `preset_manager.lock()` errors are converted to string and returned. No retry or recovery.
- Files: `src/commands/presets.rs:25`, `src/commands/presets.rs:42-98`
- Better: Implement a timeout-based retry or clearer indication that internal state is corrupted.

## Dependencies at Risk

**midir 0.10 (MIDI):**
- Risk: midir is a maintenance-friendly crate but updates are infrequent. No major recent releases suggest it's stable but also potentially unmaintained.
- Impact: If a new OS MIDI API is released (e.g., Windows 11 changes), midir might lag.
- Migration plan: Monitor releases; if no updates in 2+ years, evaluate `alsa-rs` (Linux) or native Cocoa (macOS) backends.

**cpal 0.15 (Audio):**
- Risk: cpal is well-maintained but version jumps can be major breaking changes. Currently pinned to 0.15; 0.17+ may be available.
- Impact: Performance improvements and bug fixes in newer versions are not pulled in.
- Migration plan: Test upgrade to latest cpal before it becomes urgent; plan migration path for major version bumps.

**contrapunk (Local):**
- Risk: Core logic depends on the contrapunk library (path dependency). Breaking changes in that crate break this one.
- Impact: Any changes to `HarmonyEngine` API, `GuitarInput`, or `Humanizer` require careful coordination.
- Migration plan: Keep this crate and contrapunk in sync. Use feature flags or version gates for experimental changes.

## Missing Critical Features

**No Graceful Shutdown for Audio Thread:**
- Problem: `GuitarBridge` has no explicit stop mechanism. The stream is dropped when the struct is dropped, but timing is not guaranteed.
- Impact: If the main app closes before the audio stream is properly cleaned up, system resources might leak or OS audio system might complain.
- Recommendation: Add an explicit `shutdown()` method and call it from `stop_routing()`.

**No Feedback When Device Becomes Unavailable:**
- Problem: If the user unplugs the Audient iD14 while routing is active, the audio thread fails silently. UI still shows "Routing Active."
- Impact: User hears silence and has no way to know the device is gone.
- Recommendation: Add a heartbeat check in the router to detect audio thread death, and emit a `device-disconnected` event.

**No Timeout/Watchdog for Router Thread:**
- Problem: The router thread has no heartbeat or watchdog. If it deadlocks on a Mutex, the app appears frozen but `is_running` remains true.
- Impact: UI becomes unresponsive to stop requests.
- Recommendation: Emit a heartbeat event periodically and implement a watchdog timer in `start_routing()` to force-kill the thread if no heartbeat is received.

