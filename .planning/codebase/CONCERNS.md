# Codebase Concerns

**Analysis Date:** 2026-02-05

## Tech Debt

**Extensive use of unwrap() throughout codebase:**
- Issue: Liberal use of `.unwrap()` on MIDI value conversions, velocity creation, note parsing, and mutex locks without proper error handling
- Files: `src/app.rs` (lines 635, 1018, 1054, 1061, 1074, 1082), `src/router.rs` (lines 185, 197, 265, 275, 280, 304, 322, 327, 373, 455, 522), `src/humanize/metronome.rs` (lines 17-32), `src/humanize/scheduler.rs` (line 60), `src/server/protocol.rs` (lines 120-162 in tests), `src/harmony/voice_leading/suspension.rs` (line 127), `src/harmony/stateful.rs` (line 915), `src/harmony/scale.rs` (lines 371, 391, 456), `src/chord.rs` (line 98)
- Impact: Panics possible during MIDI processing if invalid values passed or locks poisoned, causing application crash mid-performance
- Fix approach: Replace `.unwrap()` with proper Result propagation in hot paths, use `.unwrap_or_default()` or `.expect()` with clear messages in initialization code

**Note Generator module marked as non-functional:**
- Issue: Phase 6.5 (Note Generator) deferred due to "note generator just doesn't work" according to user feedback and STATE.md
- Files: `src/generator/engine.rs`, `src/generator/config.rs`, `src/generator/mod.rs`
- Impact: Entire virtual MIDI input feature (arpeggiator, scale runner, beat-synced patterns) is present in codebase but broken
- Fix approach: Debug event generation in `tick()` method, verify beat clock integration, test selected_notes flow from UI to engine

**Voice Leading implementation unverified:**
- Issue: Phase 6.2 planning complete (4 plans written) but execution never started, UAT document shows all 8 tests pending
- Files: `src/harmony/voice_leading/voicer.rs` (704 lines), `src/harmony/voice_leading/rules.rs` (223 lines), `src/harmony/voice_leading/suspension.rs` (241 lines), `src/harmony/voice_leading/styles.rs` (205 lines)
- Impact: ~1400 lines of voice leading code with complex counterpoint rules, suspension state machines, and register assignments may have bugs or not produce expected musical results
- Fix approach: Complete Phase 6.2-04 (Human verification checkpoint), test all 8 UAT criteria, verify Palestrina/Bach/Jazz/Free styles produce distinct results

**Unused imports scattered across modules:**
- Issue: 8+ compiler warnings for unused imports (ScaleFamily, ScaleMode, ContraryMotionState, CounterpointState, VoiceLeadingStyle, BeatClock, Metronome, generator::* functions)
- Files: `src/generator/mod.rs`, `src/harmony/voice_leading/mod.rs`, `src/harmony/mod.rs`, `src/humanize/mod.rs`, `src/router.rs`
- Impact: Code maintenance confusion, potential for stale APIs, compile warnings clutter
- Fix approach: Run `cargo clippy --fix` and audit public API surface, remove dead code or feature-gate conditionally-used imports

**Large monolithic files with high complexity:**
- Issue: Several files exceed 700-1500 lines with deep nesting and multiple responsibilities
- Files: `src/harmony/engine.rs` (1523 lines - HarmonyEngine with voice leading processor, octave modes, stateful mode tracking), `src/app.rs` (1278 lines - GUI state + MIDI routing + humanizer for WASM), `src/harmony/stateful.rs` (1070 lines - 3 stateful mode implementations), `src/router.rs` (791 lines - native routing + GUI state sync), `src/harmony/voice_leading/voicer.rs` (704 lines - complex voicing algorithm)
- Impact: Difficult to understand, test, and modify; high cognitive load for contributors; increased merge conflict risk
- Fix approach: Extract voice leading into separate VoiceLeadingEngine, split app.rs into app_native.rs and app_wasm.rs, refactor stateful modes into trait-based polymorphism

**Clone usage (48 occurrences):**
- Issue: Frequent `.clone()` calls on note collections, state structures, and configuration objects
- Files: 14 files including `src/app.rs`, `src/ui.rs`, `src/harmony/engine.rs`, `src/generator/engine.rs`, `src/harmony/voice_leading/voicer.rs`
- Impact: Memory allocation churn in real-time audio path, potential latency spikes during harmony generation
- Fix approach: Audit hot paths (harmonize, revoice_chord, tick), use references where possible, consider Rc/Arc for immutable shared state

**2.7GB target directory with no .gitignore entry:**
- Issue: Build artifacts consume 2.7GB with multiple incremental compilation caches and WASM targets
- Files: `target/` directory (not in .gitignore but should be)
- Impact: Slow git operations if accidentally staged, excessive disk usage, confusion for contributors
- Fix approach: Verify `target/` is in .gitignore (it is on line 2), run `cargo clean` periodically, document cache management in CONTRIBUTING.md

## Known Bugs

**Stuck MIDI notes on configuration changes:**
- Symptoms: When changing voice leading settings, key, mode, or other harmony parameters mid-play, active notes may not receive Note-Off messages
- Files: Noted in `STATE.md` line 133 as pending todo
- Trigger: User changes key/mode/voice leading style while holding notes on MIDI controller
- Workaround: Release all keys before changing settings; manually send Note-Off on all channels from external MIDI utility

**Mutex lock unwrap in router could poison on panic:**
- Symptoms: If one thread panics while holding GUIRouterState mutex, all subsequent lock attempts will unwrap and panic
- Files: `src/router.rs` (lines 185, 197, 373, 455, 522), `src/app.rs` (lines 497, 528, 551, 593, 1112, 1187)
- Trigger: Panic during harmony processing or MIDI I/O while lock held
- Workaround: None; requires application restart if mutex poisoned

## Security Considerations

**WASM callback.forget() leaks memory:**
- Risk: Web MIDI input callback intentionally leaked via `callback.forget()` to remain active
- Files: `src/midi/web.rs` line 88
- Current mitigation: Single callback per input device connection, not called repeatedly
- Recommendations: Track callbacks in app state for explicit cleanup on device disconnect, implement Drop trait for WebMidiInput wrapper

**No MIDI input validation:**
- Risk: Malformed or malicious MIDI messages from physical devices or network clients processed without validation
- Files: `src/router.rs` (processes raw MIDI bytes), `src/server/session.rs` (reads network MIDI messages)
- Current mitigation: wmidi crate provides some parsing safety, protocol.rs uses length-prefixed messages
- Recommendations: Add bounds checking on velocity/note values, implement rate limiting for network clients, validate message types before processing

**Server mode has no authentication:**
- Risk: Any client can connect to server mode and send MIDI data
- Files: `src/server/session.rs` (line 198 - accepts all connections)
- Current mitigation: Server must be explicitly started with --server flag, not exposed by default
- Recommendations: Add token-based authentication, implement connection allow-list, rate limit incoming messages per client

## Performance Bottlenecks

**Mutex contention on GUIRouterState:**
- Problem: Every frame update and every MIDI message acquisition locks the same Arc<Mutex<GUIRouterState>>
- Files: `src/app.rs` update() method, `src/router.rs` routing loop
- Cause: Shared state between GUI thread and router thread with coarse-grained locking
- Improvement path: Split state into read-only (using RwLock) and write-heavy portions, use lock-free channels for note event streaming, cache GUI-only state locally

**Voice leading evaluates all candidate voicings:**
- Problem: Revoice algorithm generates all valid placements per voice then evaluates cartesian product
- Files: `src/harmony/voice_leading/voicer.rs` lines 100+ (generates candidates, then scores all combinations)
- Cause: Holistic chord evaluation for deterministic output quality
- Improvement path: Implement early pruning of poor candidates, cache scoring results for repeated pitch classes, limit candidate count per register

**BinaryHeap re-sorting on every delay queue push:**
- Problem: Humanizer delay queue uses BinaryHeap which has O(log n) push but reallocates frequently
- Files: `src/humanize/scheduler.rs` (DelayQueue wraps BinaryHeap)
- Cause: Each humanized note inserted individually during burst harmony generation
- Improvement path: Pre-allocate capacity based on typical voice count, batch insertions, consider simpler sorted Vec for small (<10) queues

**Clone-heavy chord voicing generation:**
- Problem: Each harmony voice generation clones previous voicing, registers, and style rules
- Files: `src/harmony/engine.rs` harmonize() method
- Cause: Immutable-style API for safety and clarity
- Improvement path: Pass &mut buffers for reuse, use stack-allocated arrays for small voice counts (<8), profile allocation overhead

## Fragile Areas

**WASM MIDI integration depends on browser API availability:**
- Files: `src/midi/web.rs` (entire module), `src/app.rs` WASM-specific code paths
- Why fragile: Assumes Web MIDI API exists and permissions granted; no graceful degradation
- Safe modification: Always check MidiAccess validity before use, wrap all web_sys calls in Result, add UI feedback for permission denials
- Test coverage: No automated tests for WASM paths (requires browser environment)

**Voice position interleaving logic in voice leading:**
- Files: `src/harmony/engine.rs` build_registers_for_position (lines 52-100), VoiceLeadingProcessor register assignment
- Why fragile: Complex index manipulation to reorder voices from [soprano, alto, tenor, bass] to [user, above, below, above, below...] based on voice_position
- Safe modification: Add unit tests for all voice_position values (0 through voice_count-1), verify register order matches expected arrangement
- Test coverage: No dedicated tests for register ordering edge cases

**Stateful harmony mode state management:**
- Files: `src/harmony/stateful.rs` (ContraryMotionState, CounterpointState, PalatrinaCounterpointState)
- Why fragile: Modes track previous melody/harmony notes across calls; state reset on key/mode change can leave inconsistent history
- Safe modification: Always call reset() before first harmonize after config change, verify state cleared in tests
- Test coverage: Basic state tracking tests exist but edge cases (rapid mode switching, empty note input) untested

**Humanizer beat clock wrap-around detection:**
- Files: `src/generator/engine.rs` tick() line 43 (beat wrap detection: `beat_pos < self.last_beat_position`)
- Why fragile: Relies on floating-point comparison for beat position, assumes monotonically increasing beats within bar
- Safe modification: Use integer beat counters, explicit modulo for bar boundaries, add epsilon tolerance for float comparison
- Test coverage: No tests for beat wrap edge cases or tempo changes

## Scaling Limits

**Single-threaded harmony processing:**
- Current capacity: ~10-20 voices before frame drops (estimated based on complexity)
- Limit: All harmony generation happens synchronously in router thread (native) or GUI frame (WASM)
- Scaling path: Move harmony engine to dedicated thread pool, parallelize per-voice processing, pre-compute scale lookups

**No connection limit in server mode:**
- Current capacity: Unlimited concurrent client connections
- Limit: Each session spawns thread; OS thread limit reached around 1000-10000 clients depending on system
- Scaling path: Use async I/O (tokio), implement connection limit with queue, add load shedding

**DelayQueue unbounded growth:**
- Current capacity: Humanized notes accumulate if processing slower than generation
- Limit: Memory exhaustion if sustained >1000 notes/sec input rate
- Scaling path: Add max queue size with oldest-note eviction, warn user on queue depth threshold

## Dependencies at Risk

**midir (MIDI I/O library):**
- Risk: Depends on platform-specific native MIDI APIs (CoreMIDI, ALSA, Windows MM) which may change
- Impact: MIDI I/O breaks on OS updates or unsupported platforms
- Migration plan: Fork and maintain if upstream stalls, abstract MIDI I/O trait for swappable backends

**eframe/egui (GUI framework):**
- Risk: Rapid API evolution in egui ecosystem, breaking changes in major versions
- Impact: GUI code refactor required on updates, WASM backend compatibility issues
- Migration plan: Pin to stable major version (currently 0.33), evaluate alternatives (Iced, Tauri) for long-term support

**wmidi (MIDI message parsing):**
- Risk: Unmaintained crate (last update check needed), limited to standard MIDI 1.0 spec
- Impact: No MIDI 2.0 support, potential parsing bugs unfixed
- Migration plan: Consider midi-msg or midir's built-in parsing, implement MIDI 2.0 parser if needed

## Missing Critical Features

**No MIDI input merging:**
- Problem: User can select only one physical MIDI input device at a time
- Blocks: Multi-keyboard setups, combining Note Generator with physical input simultaneously
- Priority: Medium (workaround: use virtual MIDI merger utility)

**No preset import/export UI:**
- Problem: Preset persistence exists but no GUI buttons to load external preset JSON files
- Blocks: Sharing presets between users, backup/restore of custom presets
- Priority: Low (presets stored in eframe storage, manually copyable)

**No latency monitoring:**
- Problem: No GUI display of current processing latency or buffer health
- Blocks: Performance tuning, user awareness of system limitations
- Priority: Medium (critical for live performance use)

**No undo/redo for settings changes:**
- Problem: Changing harmony settings is destructive; no way to revert to previous configuration
- Blocks: Experimentation workflow, recovery from accidental changes
- Priority: Low (can manually revert settings)

## Test Coverage Gaps

**WASM-specific code paths:**
- What's not tested: Web MIDI API integration, frame-based polling, Rc<RefCell<>> message queue handling
- Files: `src/midi/web.rs`, `src/app.rs` WASM cfg blocks
- Risk: Browser-specific bugs undetected until deployment, MIDI access permission failures not handled gracefully
- Priority: High

**Voice leading voicer algorithm:**
- What's not tested: Cartesian product candidate generation, deterministic tiebreaking, register constraint filtering
- Files: `src/harmony/voice_leading/voicer.rs` revoice_chord function
- Risk: Incorrect voicings under edge cases (empty registers, all candidates invalid, anchor constraints conflict)
- Priority: High

**Mutex poisoning recovery:**
- What's not tested: Router behavior when GUIRouterState mutex poisoned by panicking thread
- Files: `src/router.rs`, `src/app.rs` mutex lock sites
- Risk: Cascading panics, unrecoverable application state
- Priority: Medium

**Humanizer timing under load:**
- What's not tested: DelayQueue behavior with >100 concurrent delayed notes, beat clock accuracy under sustained load
- Files: `src/humanize/scheduler.rs`, `src/humanize/beat_clock.rs`
- Risk: Note timing drift, queue overflow, memory leak
- Priority: Medium

**Server mode concurrent client stress:**
- What's not tested: Behavior with 10+ simultaneous clients, connection churn, malformed protocol messages
- Files: `src/server/session.rs`, `src/server/protocol.rs`
- Risk: Thread exhaustion, memory leak per dropped connection, protocol desync
- Priority: Low (server mode rarely used per roadmap)

---

*Concerns audit: 2026-02-05*
