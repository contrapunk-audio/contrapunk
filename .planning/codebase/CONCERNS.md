# Codebase Concerns

> **⚠️ STALE — pre-crate-split + pre-I/O-tab-restructure (2026-04-15)**
>
> Brutal-critic confirmed ~95% of this file references code that has been moved
> or deleted. Specifically the following items below are PHANTOM (do not waste
> time investigating them):
>
> - "Generator Engine never wired" — `src/generator/` deleted; `GeneratorPanel.svelte` gone too
> - "CNN Classifier 949-line inference" — `src/audio/inference.rs` deleted
> - "PolySynth allocates in callback" — `src/audio_out/sine_synth.rs:192` deleted (see CLAUDE.md footgun #3 for current synth path)
> - "MIDI queue 1024 cap" — `src/audio_out/midi_queue.rs` moved; check `src/synth/` instead
> - Most `src/X` file:line citations — `X` is now under `crates/contrapunk-X/`
>
> What IS true after the I/O tab restructure (2026-05-14/15):
>
> - `AudioNormalizer::from_profile` now has a live caller chain (GuitarBridge → GuitarInput.set_calibration_profile), but the consumer wiring inside `process_block`/`analyze_window` is still incomplete (per-string thresholds + brightness rejection unused; only noise_floor_ema seeded). Tracked separately.
> - Calibration startup auto-load is missing — saved profile silently ignored until UI Reload click.
> - VGC slot-mapping uses `voicePosition` directly instead of the engine's `port_map[]`, lying on non-default voicings.
>
> **To regenerate this doc:** `/gsd-map-codebase` (project skill).

**Analysis Date:** 2026-04-15 _(stamped STALE 2026-05-15 — see banner above)_

---

## Tech Debt

**Generator Engine: Exists in Rust, Never Wired to Any Frontend**
- Issue: `src/generator/` (engine.rs, config.rs) implements `GeneratorEngine` with arpeggio, ScaleRunner, and chord modes but is not imported by `wasm/src/lib.rs`, `src-tauri/src/`, or any adapter.
- Files: `src/generator/engine.rs`, `src/generator/config.rs`
- Impact: `GeneratorPanel.svelte` shows "Desktop only" and hides the full UI behind `{#if false}` — the feature is entirely non-functional on all platforms, not just WASM. STATE.md records this as "Phase 6.5 Outcome: DEFERRED — Note Generator just doesn't work."
- Fix approach: Wire `GeneratorEngine` into the WASM `Engine` struct in `wasm/src/lib.rs`. Add `tick()` calls to advance the generator and expose `set_generator_mode`, `set_generator_notes` bindings. Mirror in Tauri commands. Remove the `{#if false}` gate in `GeneratorPanel.svelte`.

**CNN Classifier: 949-line Inference Module with No Live Call Site**
- Issue: `src/audio/inference.rs` is a complete Pure-Rust CNN forward pass (`GuitarClassifier`, `ClassifierWeights`, `decode_class`) compiled on every build but never called from `guitar_input.rs`, the WASM bridge, or any Tauri command. The trained weights file `guitar_training_data.msgpack` sits at the repo root but is only referenced from `examples/guitar_capture.rs:45`.
- Files: `src/audio/inference.rs`, `src/audio/mod.rs:20`, `guitar_training_data.msgpack`
- Impact: ~949 lines compiled for nothing; hybrid DSP+CNN architecture is documented in user memory but not implemented. Confusion risk for anyone reading the codebase — inference looks like it should be part of the pipeline.
- Fix approach: Either integrate `GuitarClassifier` into `GuitarInput` as an optional string-ID fallback (after DSP inharmonicity), or move `inference.rs` to an `examples/` helper and note its experimental status clearly in the module doc.

**Chord Quality Derivation Always Stubbed to 0 (major)**
- Issue: The `SuggestionSnap` struct has a `current_chord_quality: u8` field. The engine snapshot function hardcodes `snap.current_chord_quality = 0; // TODO: derive quality from intervals` when a chord degree is detected, meaning the suggestion scorer always treats the active chord as major regardless of the actual scale degree.
- Files: `src/harmony/engine.rs:665`
- Impact: Suggestion scoring terms `f_chord_tone` and `f_dissonance` receive inaccurate chord membership data for minor/diminished/dominant degrees, degrading note suggestion quality in non-major modes.
- Fix approach: Derive quality from the degree's triad built on `chord_pcs`: compare the interval between `chord_pcs[0]` and `chord_pcs[1]` — minor 3rd (3 semitones) → minor, diminished 5th → diminished, augmented → augmented.

**Next-Chord Prediction Permanently Stubbed**
- Issue: `SuggestionSnap.next_chord_root` and `next_chord_confidence` are always `255` and `0.0`. The comment in `engine.rs:679` says "Next chord not yet implemented -- leave as default."
- Files: `src/harmony/engine.rs:679-680`
- Impact: Next-chord prediction scoring term in suggestion scorer has no signal to work with. The Markov chain in `src/harmony/functional/markov.rs` exists but is only used for current-chord selection, not for look-ahead.
- Fix approach: After `select_chord` picks the current degree, run it again with the winning degree as the "previous" state to get the most likely next degree. Write result into `snap.next_chord_root`.

**Voice Register for Melody is Placeholder Throughout Voice Leading**
- Issue: Three locations in voice leading assign `VoiceRegister::Soprano` as a "melody placeholder" rather than deriving register from the actual note range.
- Files: `src/harmony/voice_leading/rules.rs:166`, `src/harmony/voice_leading/voicer.rs:593,602,610`
- Impact: Voice-crossing rules and register-specific scoring are mildly incorrect for low-pitched melody lines. Practical impact is minor since register rules are permissive.
- Fix approach: Derive register from MIDI note range at assignment time instead of hardcoding Soprano.

**`sub-project-2` Voice Index Substitution**
- Issue: In the delay-queue drain in both `src/router.rs:134` and `src-tauri/src/commands/engine.rs:384`, `hn.port` is used as the `voice_index` argument to `send_humanized_note` / the audio synth fanout. `HumanizedNote` does not carry the original harmony voice index so port is used as a proxy. Per the `FIXME` comment this will break per-voice plugin routing when VST3 plugin hosting lands.
- Files: `src/router.rs:134-139`, `src-tauri/src/commands/engine.rs:384-389`
- Impact: Benign today (PolySynth ignores the `voice` field, outputs one channel), but will cause incorrect per-plugin routing when Audio Foundation sub-project 2 is implemented. GitHub issue #33 tracks this.
- Fix approach: Add `voice_index: u8` field to `HumanizedNote` struct; populate it in `humanize_note_on` from the harmony voice index.

**`audio_out` TODO Comment on Commented-Out Type Imports**
- Issue: `src/audio_out/mod.rs:18` has `// TODO: uncomment as types land in Tasks 2-6`. This is a stub note from the Audio Foundation phase; Tasks 2-6 are paused (Windows pivot).
- Files: `src/audio_out/mod.rs:18`
- Impact: No functional impact now. Becomes misleading if Audio Foundation work resumes without noticing it.

---

## Known Bugs

**Stuck MIDI Notes on Settings Change Mid-Play** — **MOSTLY RESOLVED (v1.3.0)**
- All 14 setter Tauri commands in `src-tauri/src/commands/harmony.rs` now pair with `raise_panic(&state)` after the engine mutation. The engine's `clear_active_for_reharm` (`crates/contrapunk-harmony/src/engine.rs:1417`) stashes held inputs into `pending_reharm_inputs` before clearing `active_notes`; the router-loop drain at `src-tauri/src/commands/engine.rs:463-558` replays each input, diffs old vs new harmonies, and broadcasts NoteOff for `to_release`. The codebase-mapper audit's claim of "12+ broken `.clear()` sites" was stale (only 2 `.clear()` hits in current engine.rs, both inside the safe helper).
- One remaining gap shipped in v1.3.0 (`b065eb5`): `commands/presets.rs::load_preset` previously called 8 engine setters in sequence without raising `panic_pending`, so preset switches stranded the prior preset's harmony on external synths. Fixed by adding `state.panic_pending.store(true, Ordering::SeqCst)` after the setters.
- **Test coverage gap (still open)**: no integration test exercises the full "change setting mid-play, verify NoteOff dispatched" path through the router thread. If a new setter is added to `harmony.rs` or `presets.rs` without raising panic, nothing catches it. Add a test that holds a note, calls a setter, asserts NoteOffs were dispatched.

---

## Security Considerations

**Guitar Calibration File Written to CWD, Not App Data Dir**
- Risk: `guitar_calibration_profile.json` is written with a bare relative path (`std::fs::write("guitar_calibration_profile.json", json)`), which resolves to the process working directory — not the Tauri app data directory. On macOS this is typically `~/` or the bundle dir. File lands in an unpredictable, user-visible location.
- Files: `src-tauri/src/commands/guitar.rs:244`
- Current mitigation: None. The `let _ =` discards write errors silently.
- Recommendations: Use `tauri::path::app_data_dir()` (Tauri v2 API) to resolve a deterministic, platform-appropriate path. Surface write errors to the user.

**No Tauri Command Allowlist Audit Documented**
- Risk: Tauri IPC commands are registered in `src-tauri/src/commands/` and exposed to the webview. Any XSS or script-injection reaching the webview can call these commands without further authentication.
- Files: `src-tauri/src/main.rs`, `src-tauri/src/commands/`
- Current mitigation: Tauri v2's default CSP applies; no `withGlobalTauri: true` is set (confirmed in `index.ts:29`).
- Recommendations: Audit `tauri.conf.json` capabilities to restrict commands to only what the frontend needs. Consider signing Tauri commands for sensitive operations.

---

## Performance Bottlenecks

**`PolySynth::process_stereo` Allocates on Every Audio Callback**
- Problem: `src/audio_out/sine_synth.rs:192` calls `let mut mono = vec![0.0_f32; frames];` inside `process_stereo`, which is called from the real-time audio callback `process_callback` in `engine.rs:173`. This heap-allocates every buffer period.
- Files: `src/audio_out/sine_synth.rs:192`, `src/audio_out/engine.rs:161-174`
- Cause: Convenience over real-time safety. The module doc in `mod.rs:5` claims "no allocations" but this path allocates.
- Improvement path: Pre-allocate a fixed-size scratch buffer in `PolySynth` at construction time (sized to `max_frames`) and reuse it each callback.

**`process_callback` Uses `try_lock` on `Arc<Mutex<AudioState>>`**
- Problem: The audio callback at `src/audio_out/engine.rs:166` calls `state.try_lock()`. A contended lock silently drops the audio frame (outputs silence), and any lock-holding path on the main thread will cause an audio glitch.
- Files: `src/audio_out/engine.rs:161-174`
- Cause: The SPSC ringbuffer (correct for MIDI events) is wrapped in a `Mutex` together with the `PolySynth` so the producer can share the synth safely — but this introduces a lock on the hot path.
- Improvement path: Separate MIDI event routing (already lock-free via `MidiConsumer`) from synth state. Own `PolySynth` exclusively by the audio thread; remove the `Mutex<AudioState>` wrapper.

**Per-Frame `console.log` in Guitar Capture Hot Path**
- Problem: `ui/src/lib/audio/guitarCapture.ts:153-155` logs RMS and frame count every 25 frames (`_frameCount % 25 === 0`), plus logs every detected note event and every WASM event batch at `guitarCapture.ts:169,195,199`. At 44100/1024 sample rate/buffer this is ~43 frames/sec, triggering console output ~1.7 times/second in production.
- Files: `ui/src/lib/audio/guitarCapture.ts:153-155,169,195,199`
- Impact: Measurable in Chrome DevTools timeline; may introduce jank when DevTools is open.
- Improvement path: Gate all `console.log` calls in the `onaudioprocess` handler behind a `DEBUG` constant or remove them.

**`TauriAdapter` Beat Clock is a JS Approximation, Not Rust-Driven**
- Problem: `ui/src/lib/adapter/tauri.ts:500-526` (`startBeatTicker`) runs a `setInterval` at the configured BPM to simulate beat events. This drifts from the Rust `BeatClock` in `src/humanize/beat_clock.rs` over time and does not reflect actual swing/jitter applied to notes.
- Files: `ui/src/lib/adapter/tauri.ts:500-526`
- Impact: Beat indicator UI (4-pip display) may be visually off-sync with actual humanized note timing in Tauri desktop mode.
- Improvement path: Emit a real `beat-update` Tauri event from the router thread at each `BeatClock` tick boundary and subscribe to it in `TauriAdapter`. Remove `_beatInterval` approximation.

---

## Fragile Areas

**WASM `stopTickLoop` Is Never Called — RAF Loop Leaks**
- Files: `ui/src/lib/adapter/wasm.ts:153-158`
- Why fragile: `startTickLoop` fires `requestAnimationFrame` permanently on init. `stopTickLoop` exists as a private method but the comment at line 153 reads "not currently called." Page navigation or hot-module reload in dev will accumulate leak loops.
- Safe modification: Call `stopTickLoop` from a public `destroy()` method and call it from the SvelteKit `onDestroy` lifecycle hook in the root layout.
- Test coverage: No test for adapter lifecycle/teardown.

**Tauri `guitar-signal` Listener Unsub is Only Attempted on `stopRouting`**
- Files: `ui/src/lib/adapter/tauri.ts:258-285`
- Why fragile: `this._guitarSignalUnsub` is only cleaned up inside `stopRouting`. If the component unmounts without calling `stopRouting` (e.g., error path, navigation), the listener keeps firing and updating the (now-stale) `guitar` store.
- Safe modification: Add a guard that calls `_guitarSignalUnsub?.()` in `init()` before re-registering.

**`HumanizePanel.svelte` Uses `platformName === 'plugin'` Not `isTauri`**
- Files: `ui/src/lib/components/HumanizePanel.svelte:9`, `ui/src/lib/components/GeneratorPanel.svelte:74`
- Why fragile: Humanize is correctly functional in WASM (wired via `wasm.ts` `getHumanizeState`/`setHumanizeConfig`), yet `GeneratorPanel.svelte:74` shows "Desktop only" for WASM users. The `platformName` check for generator is correct (feature truly absent), but the `AudioOutputPanel.svelte:6` check (`platformName === 'tauri'`) correctly gates that panel. The inconsistency between panels creates a confusing pattern when adding new features.
- Safe modification: Define a typed `FEATURE_FLAGS` object in `adapter/index.ts` that maps capabilities to platform, rather than scattering `platformName === 'X'` checks across components.

**Mutex Unwrap Cascade in Note-Update Router Thread**
- Files: `src-tauri/src/commands/engine.rs:422-425,605-632`
- Why fragile: The note-update emit loop unwraps 5+ `Mutex::lock()` calls in sequence. If any lock is poisoned (thread panic while holding it), the emit loop panics and the router thread crashes silently — routing stops and the UI stops receiving note-update events with no user-visible error.
- Safe modification: Replace `.unwrap()` with `.unwrap_or_default()` or a `?`-propagating helper that logs and continues.

---

## Scaling Limits

**MIDI Queue Capacity is Fixed at 1024 — Silent Drop on Overflow**
- Files: `src/audio_out/midi_queue.rs:36-38`, `src/audio_out/mod.rs:55` (default caller uses 1024)
- Current capacity: 1024 events
- Limit: On overflow `MidiProducer::push` returns `Err(QueueFull)`. All callers in the router use `.unwrap()` on the result (`router.rs:452,459`), which panics and crashes the router thread on queue full.
- Scaling path: Change router callers to handle `QueueFull` gracefully (drop oldest event or log+skip). Separate concern from panic-on-overflow.

---

## Dependencies at Risk

**`ScriptProcessorNode` is Deprecated Web API**
- Risk: `ui/src/lib/audio/guitarCapture.ts:121,287` uses `createScriptProcessor`, which is deprecated in the Web Audio API spec and may be removed in future browser versions. Chrome has flagged it since 2019.
- Impact: Guitar audio capture in WASM/browser will break silently when removed.
- Migration plan: Refactor to `AudioWorkletNode` with a worklet script that passes `Float32Array` blocks to the WASM DSP. This is a non-trivial refactor but required for long-term browser support.

---

## Missing Critical Features

**Audio Output Not Available in Browser (WASM)**
- Problem: `WasmAdapter.startAudioOutput` (`ui/src/lib/adapter/wasm.ts:743`) is a no-op that prints a console warning. `AudioOutputPanel.svelte:6` hides the toggle for non-Tauri platforms. There is no Web Audio API synth wired up for browser users.
- Blocks: Browser users cannot hear harmony output without a separate hardware MIDI connection.

**Generator Has No WASM Binding or Tauri Command**
- Problem: `src/generator/engine.rs` (`GeneratorEngine`, `GeneratorMode`) is compiled into the library but never exported through WASM bindings or Tauri IPC. `GeneratorPanel.svelte` is dead UI code gated with `{#if false}`.
- Blocks: Feature parity target in `.planning/ROADMAP.md:265` ("fix WASM feature parity so humanize and generator work in the browser").

**Tauri Detune Has No Backend Command**
- Problem: `TauriAdapter.setDetune` (`ui/src/lib/adapter/tauri.ts:468-471`) stores `_detuneCents` locally but never calls `invoke()` — there is no matching `set_detune` Tauri command. Detune works in WASM (pitch bend sent to Web MIDI outputs) but is a no-op in Tauri desktop mode.
- Files: `ui/src/lib/adapter/tauri.ts:468-471`, `src-tauri/src/commands/`

---

## Test Coverage Gaps

**Audio Output Callback Path — No Real-Time Safety Test**
- What's not tested: `process_callback` + `PolySynth::process_stereo` under contention (lock held by main thread).
- Files: `src/audio_out/engine.rs:161-174`, `src/audio_out/sine_synth.rs:183-201`
- Risk: Silent audio glitch (dropped callback) under main-thread lock pressure would not be caught.
- Priority: Medium

**Generator Engine — Integration Test Missing**
- What's not tested: `GeneratorEngine` advancing through multiple beats and routing output through the harmony engine.
- Files: `src/generator/engine.rs`
- Risk: Changes to beat clock or scale mappings could silently break generator note output.
- Priority: Medium (feature deferred but unit tests present; integration test missing)

**WASM Adapter Lifecycle — No Teardown/Reinit Test**
- What's not tested: `WasmAdapter.stopTickLoop` path, RAF cleanup, adapter reinitialization.
- Files: `ui/src/lib/adapter/wasm.ts:153-158`
- Risk: RAF leak in long-running sessions or after hot reload.
- Priority: Low

**Tauri Beat Ticker vs Rust BeatClock Drift**
- What's not tested: The approximated `setInterval` beat ticker in `TauriAdapter` vs actual `BeatClock` timing from Rust.
- Files: `ui/src/lib/adapter/tauri.ts:500-526`
- Risk: UI beat indicator visual drift is not caught. Affects metronome display accuracy.
- Priority: Low

---

## DSP vs ML Status (Hybrid Architecture)

The `GuitarClassifier` CNN (97.3% accuracy, `src/audio/inference.rs`) is not connected to the live pipeline. The primary path is `GuitarInput` in `src/audio/guitar_input.rs` which uses McLeod pitch detection (pyin-style) + inharmonicity B-coefficient string identification (98.5% accuracy per user memory). The hybrid plan (DSP first, CNN as fallback) is architectural intent only — no integration exists in the current codebase. The Python training pipeline (`ml/`) and `guitar_training_data.msgpack` remain in the repo but are not referenced by any live Rust build target. If the hybrid path is pursued, the integration point is `guitar_input.rs` step 6 (string identification), replacing or supplementing the inharmonicity match with a CNN call.

---

## Paused Work

**Audio Foundation Task 5 (Plugin Hosting) — In-Flight, Paused**
- Context: `src/audio_out/mod.rs:10` documents "Sub-project 1 of plugin hosting" (sine synth) as shipped. VST3 plugin loading ("sub-project 2") is not started. The `au-wrapper/` directory contains CMake build infrastructure for a CLAP/AU wrapper but it is a standalone C++ project not integrated with the Rust build. The `FIXME(sub-project-2)` at `src/router.rs:134` and `src-tauri/src/commands/engine.rs:384` mark code that will break when per-voice plugin routing is needed.
- Files: `src/audio_out/mod.rs:10`, `src/router.rs:134`, `src-tauri/src/commands/engine.rs:384`, `au-wrapper/`

---

*Concerns audit: 2026-04-15*
