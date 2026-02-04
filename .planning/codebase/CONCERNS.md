# Codebase Concerns

**Analysis Date:** 2026-02-04

## Tech Debt

**Note Generator Module (Phase 6.5 - Deferred):**
- Issue: Note generator feature is non-functional. Module exists with config and engine types but doesn't work end-to-end.
- Files: `src/generator/mod.rs`, `src/generator/config.rs`, `src/generator/engine.rs`
- Impact: Feature is advertised in roadmap but cannot be used by end users. Dead code remains in codebase. Unused exports trigger compiler warnings.
- Fix approach: Either complete implementation per Phase 6.5 plans or remove module entirely and update roadmap. Current state is confusing limbo.

**Voice Leading Distinctness (Phase 6.2 Feedback):**
- Issue: Different voice leading styles don't produce audibly distinct results. VoiceLeadingStyle enum has Palestrina, Bach, Jazz, Free variants but user reports they sound too similar.
- Files: `src/harmony/voice_leading/styles.rs`, `src/harmony/voice_leading/voicer.rs`, `src/harmony/engine.rs` (lines 1-1523)
- Impact: Feature complexity doesn't deliver proportional value. Users can't meaningfully choose between styles.
- Fix approach: Research and implement more aggressive style differentiation (Palestrina = strict dissonance rules, Jazz = extended voicings/chromaticism, Bach = specific SATB spacing). Alternatively, remove style variants and keep single "smooth voice leading" mode.

**WASM Borrowed Notes Tracking (TODO):**
- Issue: Borrowed notes from modal interchange are not tracked in WASM build, only in native.
- Files: `src/app.rs` (line 538-539)
- Impact: WASM users don't see amber highlights for borrowed notes on piano keyboard. Visual feedback is incomplete compared to native.
- Fix approach: Add `wasm_borrowed_notes: HashSet<u8>` field to ContrapunkApp, populate it during WASM MIDI processing path similar to native router state. Update `get_router_notes()` to return non-empty third tuple element.

**Unused Imports and Dead Code:**
- Issue: Multiple unused imports trigger compiler warnings. Generator module exports are unused. Voice leading rule functions unused. ScaleFamily/ScaleMode unused in harmony mod. Stateful types unused.
- Files: `src/generator/mod.rs` (lines 4-5), `src/harmony/voice_leading/mod.rs` (lines 9-10), `src/harmony/mod.rs` (lines 13, 16)
- Impact: Code hygiene degrades. Warnings noise makes real issues harder to spot. Suggests incomplete refactoring or feature removal.
- Fix approach: Remove unused exports and imports. If generator module is truly deferred, gate with `#[cfg(feature = "generator")]` or remove entirely.

**Stuck MIDI Notes on Settings Change:**
- Issue: Changing settings (voice leading, key, mode) mid-play clears active_notes tracking without sending Note-Off messages, causing stuck notes.
- Files: State tracking in `src/router.rs`, `src/app.rs` (harmony engine resets)
- Impact: User must manually silence stuck notes or restart application. Unprofessional behavior. Workarounds exist but proper fix not implemented.
- Fix approach: Before clearing active_notes, send Note-Off for all tracked notes across all outputs. Requires port routing awareness and careful WASM/native handling.

**Dual MIDI Code Paths (Native vs WASM):**
- Issue: Significant duplication between native (midir, Arc<Mutex>, background threads) and WASM (Web MIDI API, Rc<RefCell>, frame polling). 45+ `cfg(target_arch = "wasm32")` blocks across 5 files.
- Files: `src/midi/mod.rs`, `src/app.rs` (39 occurrences), `src/main.rs` (2), `src/ui.rs` (2), `src/lib.rs` (1)
- Impact: Maintenance burden. Bug fixes must be applied twice. Feature parity hard to verify. Refactoring risky.
- Fix approach: Extract common MIDI logic into trait-based abstraction (MidiBackend trait). Implement for native and WASM separately. Reduces duplication but increases abstraction complexity. Evaluate cost/benefit.

**Large Complex Files:**
- Issue: Several files exceed 700+ lines with complex logic, making them hard to navigate and test.
- Files: `src/harmony/engine.rs` (1523 lines), `src/app.rs` (1278 lines), `src/harmony/stateful.rs` (1070 lines), `src/router.rs` (791 lines), `src/harmony/voice_leading/voicer.rs` (704 lines)
- Impact: Cognitive load for contributors. Testing requires understanding entire file context. Refactoring risk increases.
- Fix approach: Split harmony/engine.rs into separate modules (engine.rs, voice_leading_processor.rs, note_tracker.rs). Extract app.rs tab rendering into separate files (already started with ui.rs). Break stateful.rs into per-mode files. Requires careful module boundary design.

**Extensive Clone Usage (48 occurrences):**
- Issue: Heavy reliance on `.clone()` across 14 files suggests value-passing inefficiency, particularly for HashSets and Vecs.
- Files: `src/app.rs` (17), `src/ui.rs` (10), others
- Impact: Potential performance overhead, especially in WASM where frame-based polling clones entire note sets every update. Real-time audio requires <5ms latency.
- Fix approach: Profile hotspots first. Consider borrowing where possible, or use Rc/Arc for shared state instead of cloning. WASM MIDI queue could use slice drain instead of clone-and-clear.

## Known Bugs

**No Critical Bugs Identified:**
- No open bugs beyond the stuck MIDI notes issue documented in tech debt.
- Stuck notes workaround: avoid changing settings mid-play.

## Security Considerations

**No Secrets in Codebase:**
- No hardcoded credentials, API keys, or secrets detected in source.
- Fly.io deployment uses `FLY_API_TOKEN` secret via GitHub Actions (secure).
- Web MIDI API requires user permission grant (browser-enforced).

**MIDI Device Access:**
- Risk: MIDI device enumeration exposes system device names (potential privacy leak).
- Files: `src/midi/ports.rs`, Web MIDI integration
- Current mitigation: Standard practice for MIDI applications. Users grant permission explicitly in WASM.
- Recommendations: Document that device names are visible to application. No further action needed.

**Dependency Trust:**
- Risk: Third-party crates (midir, wmidi, eframe, web-sys) could contain vulnerabilities.
- Current mitigation: Using established crates with active maintenance. Locked versions in Cargo.toml.
- Recommendations: Run `cargo audit` periodically. Update dependencies regularly. Consider adding to CI pipeline.

## Performance Bottlenecks

**WASM Frame Polling Overhead:**
- Problem: WASM build polls MIDI queue every frame (~60Hz), cloning entire Vec<Vec<u8>> each time.
- Files: `src/app.rs` (WASM update path), `src/midi/web.rs`
- Cause: Rc<RefCell<Vec>> requires clone to safely iterate. Frame rate tied to MIDI processing rate.
- Improvement path: Use slice drain pattern to move messages out of queue without cloning. Benchmark if 60Hz polling is bottleneck for latency.

**Voice Leading Algorithm Complexity:**
- Problem: Voice leading revoice_chord generates all candidate voicings combinatorially, evaluates each against style rules.
- Files: `src/harmony/voice_leading/voicer.rs` (704 lines)
- Cause: Exhaustive search for optimal voicing. N voices with M octave possibilities = O(M^N) candidates.
- Improvement path: Real-time constraint is <5ms per note change. Profile if voice leading exceeds this. Consider pruning strategies (limit octave search range, early termination on good-enough candidate). May be premature optimization if current performance acceptable.

**No Profiling Data Available:**
- Problem: Performance assumptions not verified with measurements.
- Impact: Optimizing wrong areas, or over-engineering solutions to non-problems.
- Recommendation: Add instrumentation (e.g., instant::Instant timestamps) to measure harmony processing, voice leading, WASM frame overhead. Profile under load (rapid note input).

## Fragile Areas

**Generator Module (Non-Functional):**
- Files: `src/generator/mod.rs`, `src/generator/config.rs`, `src/generator/engine.rs`
- Why fragile: Partially implemented, not tested, deferred indefinitely. Unknown what works and what doesn't.
- Safe modification: Don't touch until Phase 6.5 is resumed or module is removed.
- Test coverage: Zero. No tests exist for generator module.

**Modal Interchange Borrowing Logic:**
- Files: `src/harmony/scale.rs` (harmonize_smart), `src/harmony/engine.rs` (modal interchange integration)
- Why fragile: Complex logic with borrowing_sources mapping, range-dependent mode selection, last_borrowed_from state tracking. Easy to introduce edge cases.
- Safe modification: Verify with all scale modes (Ionian, Dorian, Phrygian, etc.) and all borrowing ranges (1-5). Test out-of-key notes extensively.
- Test coverage: No dedicated tests for modal interchange found. Manual verification only.

**MIDI Note Tracking (Active Notes):**
- Files: `src/router.rs` (HashMap<u8, Vec<Note>>), `src/app.rs` (WASM tracking)
- Why fragile: Note-On/Note-Off pairing must be perfect or stuck notes occur. Port routing complicates Note-Off delivery. Settings changes can orphan tracking.
- Safe modification: Always test Note-Off delivery. Never clear tracking without sending offs. Use exhaustive manual testing with physical MIDI hardware.
- Test coverage: No automated tests for note tracking lifecycle. Hardware verification only.

**Web MIDI Integration (WASM):**
- Files: `src/midi/web.rs`, Web MIDI callbacks
- Why fragile: JavaScript interop via wasm-bindgen, async promises, closure lifetimes. Browser API differences (Chrome vs Firefox). User permission flows.
- Safe modification: Test in multiple browsers (Chrome, Firefox, Safari). Verify permission denial handling. Check async callback ordering.
- Test coverage: Manual browser testing only. No automated WASM tests.

## Scaling Limits

**Voice Count Limit (8 Outputs):**
- Current capacity: 8 MIDI output ports maximum.
- Limit: Hardcoded slot count in UI and router logic.
- Scaling path: Change output_slots Vec size in `src/app.rs`. Update UI slot rendering loop. Voice leading registers limited to 4 (Soprano/Alto/Tenor/Bass) — would need redesign for >4 harmony voices.

**Preset Storage (JSON Files):**
- Current capacity: File-based JSON per preset, no database.
- Limit: Filesystem I/O on every load/save. No multi-user support. WASM uses localStorage (5-10MB browser limit).
- Scaling path: For large preset libraries, consider SQLite or indexed storage. For multi-user, add server-side storage with sync.

**Single-Threaded Harmony Processing:**
- Current capacity: All harmony generation on single thread (main thread in WASM, dedicated router thread in native).
- Limit: Processing latency scales with note count and harmony mode complexity. Voice leading adds overhead.
- Scaling path: Unlikely bottleneck for typical use (1-4 input notes at a time). If needed, parallelize per-note harmony generation or voice leading per output slot.

## Dependencies at Risk

**eframe 0.33 (GUI Framework):**
- Risk: Major version changes in egui/eframe can break UI code. Breaking changes require manual migration.
- Impact: Entire GUI layer. 1200+ line app.rs relies heavily on eframe APIs.
- Migration plan: Pin to 0.33.x for stability. Monitor egui changelog for 0.34/1.0 breaking changes. Budget significant refactor time for major version upgrades.

**midir 0.10 (Native MIDI):**
- Risk: Low. midir is mature and stable. API changes rare.
- Impact: MIDI I/O on native platforms (macOS, Linux, Windows).
- Migration plan: Stay current with minor versions. No urgent concerns.

**Web MIDI API (WASM):**
- Risk: Browser API evolution or deprecation. Not a crate, so no version control.
- Impact: Entire WASM MIDI functionality depends on browser support. Safari historically spotty.
- Migration plan: Monitor web standards. Test regularly across browsers. No alternative web MIDI solution available.

**Rust Edition 2021:**
- Risk: Edition 2024 will eventually be required for new language features.
- Impact: Minimal. Edition migrations usually straightforward.
- Migration plan: Update `edition = "2021"` in Cargo.toml when ready. Test thoroughly.

## Missing Critical Features

**Stuck Note Emergency Stop:**
- Problem: No global "panic" button to send Note-Off for all active notes across all outputs.
- Blocks: Graceful recovery from stuck note bugs or user errors.
- Priority: High. Common pain point in MIDI applications.

**MIDI Routing Visualization:**
- Problem: No visual indication of which input note routes to which output port, especially in Mirror Octaves or multi-voice modes.
- Blocks: Debugging routing issues, understanding complex voice configurations.
- Priority: Medium. Advanced users only.

**Preset Sharing/Import:**
- Problem: No way to share presets between users or machines except manual JSON file copy.
- Blocks: Community preset libraries, collaborative workflows.
- Priority: Low. Phase 6.3 implemented local preset save/load/export.

**Audio Output (Vocoder):**
- Problem: Phase 8 (Vocoder) not implemented. No audio synthesis or vocoding capability.
- Blocks: Self-contained musical instrument use without external MIDI synths.
- Priority: Deferred. Phase 8 planned but not started.

**Error Reporting/Logging:**
- Problem: No structured logging or user-facing error messages. Errors silently swallowed in many paths.
- Blocks: Debugging user issues, understanding failures in production (Fly.io deployment).
- Priority: Medium. Add tracing/logging crate, emit to console (WASM) or file (native).

## Test Coverage Gaps

**No Unit Tests:**
- What's not tested: Zero test functions in codebase. `cargo test` reports 0 tests.
- Files: All `src/**/*.rs` files lack `#[cfg(test)]` modules.
- Risk: Regressions go undetected. Refactoring is dangerous. No confidence in correctness beyond manual verification.
- Priority: Critical. Add tests for harmony modes, voice leading, chord detection, scale transposition, modal interchange, note routing.

**No Integration Tests:**
- What's not tested: End-to-end MIDI flow, router behavior, GUI interactions, preset loading.
- Files: No `tests/` directory exists.
- Risk: Breaking changes to public APIs or cross-module contracts go unnoticed.
- Priority: High. Add integration tests for router startup, MIDI message processing, harmony engine integration.

**No WASM Tests:**
- What's not tested: WASM build correctness, Web MIDI API integration, localStorage persistence.
- Files: No wasm-bindgen-test usage.
- Risk: WASM-specific bugs only found through manual browser testing. CI doesn't catch WASM regressions beyond compilation.
- Priority: Medium. Add wasm-bindgen-test for basic WASM functionality. CI already runs WASM build check.

**Manual Hardware Verification Only:**
- What's not tested: Physical MIDI device interaction, latency, stuck notes, Note-Off delivery.
- Files: All MIDI I/O code (`src/midi/*`, `src/router.rs`).
- Risk: Hardware-specific issues (buffer sizes, timing, device quirks) not caught until user reports.
- Priority: Medium. Difficult to automate (requires MIDI loopback devices). Consider virtual MIDI port testing.

**Voice Leading Correctness:**
- What's not tested: Parallel fifths/octaves detection, voice crossing prevention, register constraints, suspension resolution.
- Files: `src/harmony/voice_leading/*.rs`
- Risk: Counterpoint rules violated, producing musically incorrect output. Users may not notice subtle voice leading errors.
- Priority: High. Add unit tests with known-good voicing examples from music theory literature (Bach chorales, Palestrina exercises).

**Chord Detection Accuracy:**
- What's not tested: Extended chord recognition (9th, 11th, 13th), slash chords, roman numeral analysis, edge cases (3+ octaves spanning).
- Files: `src/chord.rs` (480 lines)
- Risk: Misidentified chords confuse users or display wrong names.
- Priority: Medium. Add test suite with comprehensive chord examples (triads, sevenths, extended, altered, slash).

**Modal Interchange Edge Cases:**
- What's not tested: Borrowing from all 28 scale modes, borrowing range 1-5 variations, scale mode + harmony mode combinations.
- Files: `src/harmony/scale.rs` (harmonize_smart function), `src/harmony/engine.rs`
- Risk: Certain scale mode + borrowing range combinations produce unexpected or wrong harmonies.
- Priority: High. Complex feature with many permutations. Add parameterized tests covering all modes and ranges.

---

*Concerns audit: 2026-02-04*
