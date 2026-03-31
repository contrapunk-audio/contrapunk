# Codebase Concerns

**Analysis Date:** 2026-03-31

## Tech Debt

**Humanize and Generator are desktop-only stubs in WASM:**
- Issue: `WasmAdapter.getHumanizeState()` returns hardcoded defaults and `setHumanizeConfig()` silently discards all calls. The `GeneratorPanel.svelte` entire UI is wrapped in `{#if false}` and shows "Desktop only".
- Files: `ui/src/lib/adapter/wasm.ts` lines 157-179, `ui/src/lib/components/GeneratorPanel.svelte` line 78, `ui/src/lib/components/HumanizePanel.svelte` lines 99-100
- Impact: Humanize and Generator features are entirely non-functional in the browser/WASM build. The NoteGenerator engine (`src/generator/engine.rs`) is fully implemented in Rust but has no WASM binding in `wasm/src/lib.rs`.
- Fix approach: Add `NoteGenerator` and humanize bindings to `wasm/src/lib.rs` analogous to how `HarmonyEngine` is wrapped. The JS adapter's `setHumanizeConfig` and `getHumanizeState` already forward to the right method signatures on `ContrapunkAdapter`.

**Browser-mode detection in HumanizePanel uses a fragile heuristic:**
- Issue: `isBrowserMode` is set by checking if ALL of `jitterEnabled`, `velocityEnabled`, `durationEnabled`, `swingEnabled` are false AND `bpm === 120`. Any default Tauri state that matches these defaults would also be flagged as browser mode, hiding the UI.
- Files: `ui/src/lib/components/HumanizePanel.svelte` line 26
- Impact: In Tauri mode with all humanize settings at factory defaults, the panel incorrectly shows "Desktop only".
- Fix approach: Pass `platformName` from `$lib/adapter` directly into the component rather than inferring platform from state values. `ui.svelte.ts` already exposes `platform` state.

**"Note Generator" input option has no wiring:**
- Issue: `MidiDevices.svelte` lists "Note Generator" as a valid input option (`VIRTUAL_NOTE_GENERATOR = Number.MAX_SAFE_INTEGER`) and stores it in `midi.selectedInput`, but `+page.svelte` has no handler for `VIRTUAL_NOTE_GENERATOR`. Only `VIRTUAL_COMPUTER_KEYBOARD` is handled. Selecting "Note Generator" then pressing Start would call `adapter.startRouting(Number.MAX_SAFE_INTEGER, [...])`, which will silently fail or enumerate a non-existent port.
- Files: `ui/src/lib/components/MidiDevices.svelte` lines 6-16, `ui/src/routes/+page.svelte`
- Impact: "Note Generator" input is selectable but completely non-functional. Will confuse users.
- Fix approach: Either remove "Note Generator" from the input dropdown until wired, or add a handler in `+page.svelte` that drives `GeneratorPanel.svelte` state when that sentinel is selected.

**`toggle()` on `EngineStore` is a half-implementation:**
- Issue: `engine.toggle()` only stops, never starts. The comment says "Start requires device indices; callers should use start() directly" but the method is exported publicly and appears to be a full toggle.
- Files: `ui/src/lib/stores/engine.svelte.ts` lines 613-619
- Impact: Any future code that calls `engine.toggle()` to start will silently do nothing.
- Fix approach: Either remove the method, make it private, or accept device indices as parameters.

**Tauri detune does not send pitch bend:**
- Issue: `TauriAdapter.setDetune()` stores the cents value locally but has no backend call and no pitch bend MIDI output. The comment says "Tauri: pitch bend would be handled by the Rust backend" but no Tauri command or Rust implementation exists for this.
- Files: `ui/src/lib/adapter/tauri.ts` lines 316-323
- Impact: Detune is a saved setting that silently has no effect in Tauri mode.
- Fix approach: Either implement a `set_detune` Tauri command in `src-tauri/src/commands/engine.rs` that sends pitch bend CC on routing start/change, or use the same pitch bend approach as WasmAdapter.

**Tauri router snapshots engine config at `start_routing` time:**
- Issue: `start_routing` in `src-tauri/src/commands/engine.rs` captures a snapshot of the engine config tuple at call time. If the user changes key/mode/scale while routing is active (via the UI controls), those changes call `adapter.setKey()` etc. which update the `AppState.engine` mutex but the running router thread has its own local `HarmonyEngine` instance that never sees the changes.
- Files: `src-tauri/src/commands/engine.rs` lines 73-86, 210-219
- Impact: Live parameter changes during Tauri routing have no effect until the user stops and restarts. The WASM adapter does not have this problem because it calls engine methods directly on the same in-memory instance.
- Fix approach: Replace the config snapshot with an `Arc<Mutex<HarmonyEngine>>` shared between the command thread and router thread, similar to how `input_notes` / `harmony_notes` are already shared.

**Preset storage is in-memory only (WASM):**
- Issue: `WasmAdapter.savePreset()` calls `engine.save_preset(name)` on the WASM Engine. The `PresetManager` in WASM holds state only in the current JS heap. Browser page reload destroys all custom presets.
- Files: `ui/src/lib/adapter/wasm.ts` lines 501-508, `wasm/src/lib.rs` lines 236-241
- Impact: User-created presets in the browser are lost on refresh.
- Fix approach: Serialize `PresetManager` state to `localStorage` on each save/delete, and restore it on `Engine` construction. Alternatively, expose JSON import/export from the preset panel.

**`send_humanized_note` is duplicated between router crates:**
- Issue: An identical function `send_humanized_note` exists in both `src/router.rs` (line 18) and `src-tauri/src/commands/engine.rs` (line 517). Both build a `MidiMessage`, size a buffer, and call `output.send_to_port`.
- Files: `src/router.rs` lines 17-27, `src-tauri/src/commands/engine.rs` lines 517-526
- Impact: Any future bug fix or optimization must be applied in two places.
- Fix approach: Move `send_humanized_note` into a shared location, e.g., `src/midi/output.rs` as a method or free function.

**Scale interval table duplicated between Rust and TypeScript:**
- Issue: `SCALE_INTERVALS` in `ui/src/lib/stores/engine.svelte.ts` (lines 211-245) manually mirrors the interval arrays from `src/harmony/config.rs`. There is no compile-time check that they stay in sync.
- Files: `ui/src/lib/stores/engine.svelte.ts` lines 211-245, `src/harmony/config.rs`
- Impact: Adding or correcting a scale in Rust requires a separate manual update in TypeScript. A mismatch would cause the piano "in-scale" highlight to be wrong.
- Fix approach: Generate the TypeScript constants from the Rust source (e.g., via a build script or wasm-exported function), or accept the duplication and add a comment documenting the coupling.

**Debug `println!` left in production MIDI input:**
- Issue: `src/midi/input.rs` (line 62) logs every incoming MIDI message timestamp and bytes to stdout in all build modes including release.
- Files: `src/midi/input.rs` lines 62-67
- Impact: Noise in production CLI use. Minor performance overhead for high-density MIDI streams (e.g., pitch bend, CC automation).
- Fix approach: Wrap behind `#[cfg(debug_assertions)]` or use `log::trace!`.

## Known Bugs

**`wasm-types.d.ts` missing `set_voice_count` and `clear_notes`:**
- Symptoms: TypeScript can only validate against the manual type stub at `ui/src/lib/adapter/wasm-types.d.ts`. The stub is missing `set_voice_count`, `clear_notes`, `note_off`, and `note_on` which are all called at runtime.
- Files: `ui/src/lib/adapter/wasm-types.d.ts`, `ui/src/lib/adapter/wasm.ts` lines 148-155, 333-360
- Trigger: Always present; only caught at runtime if the real WASM build diverges from the stub.
- Workaround: The production build uses actual wasm-pack-generated `.d.ts` from `ui/src/lib/wasm-pkg/contrapunk_wasm.d.ts`, so this only affects stub-based development.

**`NoteState.borrowedNotes` never cleared on Note-Off in Tauri:**
- Symptoms: `handle_note_off` in `src-tauri/src/commands/engine.rs` removes harmony notes from `borrowed_notes` using the same notes returned by `harmonize_note_off`. However if notes were borrowed from a different mode interval on the original Note-On (tracked by `engine.last_borrowed_from()`), the removal set may not match the insertion set perfectly under mode-change edge cases.
- Files: `src-tauri/src/commands/engine.rs` lines 480-489
- Trigger: Change key/mode while notes are held in Tauri routing mode.

**`stop_routing` in Tauri returns an error if called when not running, but UI always calls it:**
- Symptoms: `TauriAdapter.stopRouting()` calls `invoke('stop_routing')`. The Tauri command returns `Err("Routing is not active")` if called redundantly. The adapter wraps this in `throw new Error(...)`, which propagates to `engine.stop()` and would display an error.
- Files: `src-tauri/src/commands/engine.rs` lines 145-147, `ui/src/lib/adapter/tauri.ts` lines 219-224
- Trigger: Double-clicking Stop, or navigating/refreshing while stopped.

## Security Considerations

**TCP server binds to `0.0.0.0` with no authentication:**
- Risk: `src/server/mod.rs` binds to `0.0.0.0:{port}` which exposes the harmony server on all network interfaces. There is no authentication in `src/server/session.rs`.
- Files: `src/server/mod.rs` line 18, `src/server/session.rs`
- Current mitigation: The server is only used by the CLI binary, not the Tauri/WASM builds.
- Recommendations: Bind to `127.0.0.1` by default. Add `ServerConfig` option to specify bind address.

**Web MIDI permission has no error UI beyond a null return:**
- Risk: If the user denies MIDI access, `WasmAdapter.ensureMidiAccess()` returns `null` silently. `listMidiInputs` / `listMidiOutputs` return empty arrays. No error is surfaced to the user.
- Files: `ui/src/lib/adapter/wasm.ts` lines 184-195
- Current mitigation: None — user sees empty device lists with no explanation.
- Recommendations: Return and display a permission-denied message in the MIDI device panel.

## Performance Bottlenecks

**WASM note state polling via `requestAnimationFrame` at 60fps:**
- Problem: `WasmAdapter.startNotePolling()` calls `engine.get_note_state()` every animation frame (~16ms). `get_note_state` serializes a full state object through wasm-bindgen on every frame even when no notes have changed.
- Files: `ui/src/lib/adapter/wasm.ts` lines 399-420
- Cause: WASM has no push-event mechanism; polling is necessary, but serialization overhead is paid unconditionally.
- Improvement path: Cache last serialized state and skip the callback if the result is identical (compare `input_notes` + `harmony_notes` arrays by reference or checksum). Also stop polling when `_isRunning` is false — the check is there but only applies after the first tick.

**Multiple Mutex locks per MIDI message in Tauri router:**
- Problem: Each MIDI Note-On in `handle_note_on` acquires 4 `Mutex::lock().unwrap()` calls sequentially: `input_notes`, `harmony_notes`, `borrowed_notes`, and `chord_name`. Chord name computation (`chord_display_with_analysis`) also acquires `input_notes` and `harmony_notes` again.
- Files: `src-tauri/src/commands/engine.rs` lines 403-433
- Cause: Note state is split into four separate `Mutex<HashSet>` fields rather than one `Mutex<NoteState>`.
- Improvement path: Consolidate into `Mutex<NoteState>` with one lock per message. Low priority given typical MIDI note rates, but becomes relevant under busy arpeggio or drum MIDI streams.

## Fragile Areas

**Engine config snapshot in Tauri start_routing (live-change blindspot):**
- Files: `src-tauri/src/commands/engine.rs` lines 73-86, 210-219
- Why fragile: All UI setting changes during routing (key, mode, scale, voice leading, interchange, voice position) take effect in `AppState.engine` but not in the thread-local engine copy. The divergence is silent — no error, no UI indicator.
- Safe modification: Any changes to which fields are captured in the `EngineConfig` tuple require updating both the capture site (line 73) and the reconstruction site (line 197).
- Test coverage: No tests for live-change behavior during routing.

**`isBrowserMode` heuristic in HumanizePanel:**
- Files: `ui/src/lib/components/HumanizePanel.svelte` lines 22-27
- Why fragile: The condition `!jitterEnabled && !velocityEnabled && !durationEnabled && !swingEnabled && bpm === 120` will incorrectly hide the Tauri UI panel for any user who has not modified humanize settings from defaults.
- Safe modification: Replace with a direct `ui.platform === 'browser'` check.
- Test coverage: No automated component tests.

**`wasm-pkg` directory is gitignored but required for build:**
- Files: `ui/src/lib/wasm-pkg/.gitignore`
- Why fragile: The `.gitignore` inside `wasm-pkg/` excludes the compiled `.wasm` binary from git. CI/CD or a fresh clone will not have the binary. The build script (`scripts/build-wasm.sh`) creates a JS stub if `wasm-pack` is absent, but the stub has no real harmony logic.
- Safe modification: Document required `wasm-pack` installation in project README or a Makefile.
- Test coverage: No CI pipeline detected.

**`VIRTUAL_NOTE_GENERATOR` selected input reaches `startRouting` with sentinel value:**
- Files: `ui/src/lib/components/MidiDevices.svelte` lines 6-29, `ui/src/routes/+page.svelte`
- Why fragile: If `Note Generator` is selected and Start is pressed, `engine.start(Number.MAX_SAFE_INTEGER, outputs)` is called. In WASM mode this reaches `WasmAdapter.startRouting` which does `inputs[Number.MAX_SAFE_INTEGER]` on the Web MIDI inputs array — always `undefined`, so `activeInput` is null and no MIDI handler is registered. In Tauri mode it invokes `start_routing` with an absurdly large port index, which will fail with a port-not-found error.
- Safe modification: Add a guard in `+page.svelte` that detects `VIRTUAL_NOTE_GENERATOR` before calling `engine.start()`.

## Scaling Limits

**`PresetManager` is entirely in-memory:**
- Current capacity: Presets are stored as `Vec<StylePreset>` in the process heap. Custom presets added via WASM are lost on page reload (see Tech Debt above). Tauri builds persist presets only for the session duration — no disk serialization detected.
- Limit: Not a hard limit, but meaningful for user workflow.
- Scaling path: Add `localStorage` persistence in WASM, and file-system persistence (via Tauri FS plugin) in desktop mode.

## Dependencies at Risk

**`@tauri-apps/api` v2 in `dependencies` (not `devDependencies`):**
- Risk: The Tauri API is listed as a runtime dependency in `ui/package.json`. In the browser/WASM build, the Tauri module is dynamically imported only when `isTauri()` returns true (via `adapter/index.ts`), but the package is bundled into every build.
- Files: `ui/package.json` line 28
- Impact: Adds dead code and bundle weight to the WASM/browser distribution.
- Migration plan: Use a dynamic `import()` for the Tauri adapter or move it to an optional import that vite tree-shakes.

**`wasm-pack` is an undeclared external build dependency:**
- Risk: `build-wasm.sh` silently falls back to a JS stub if `wasm-pack` is not installed. This means a developer without `wasm-pack` will get a non-functional WASM build with no error, only a console warning.
- Files: `ui/scripts/build-wasm.sh` lines 9-17
- Impact: Silent degraded builds in dev and CI.
- Migration plan: Fail hard (`exit 1`) if `wasm-pack` is missing, or add it as a dev dependency via npm.

## Missing Critical Features

**No WASM bindings for Generator or Humanize:**
- Problem: `src/generator/engine.rs` (fully implemented `NoteGenerator`) and `src/humanize/engine.rs` (fully implemented `Humanizer`) have no exported functions in `wasm/src/lib.rs`.
- Blocks: Any use of humanize or generator features in the browser build.

**No tests for the UI adapter layer:**
- Problem: Zero test files exist under `ui/src/`. The Svelte adapter layer (`wasm.ts`, `tauri.ts`), stores (`engine.svelte.ts`, `midi.svelte.ts`), and all components have no unit or integration tests.
- Files: All of `ui/src/`
- Risk: Adapter contract divergence between WASM and Tauri builds, regression in store logic, and UI interaction bugs go undetected.
- Priority: High

## Test Coverage Gaps

**No tests for platform-specific adapter behavior:**
- What's not tested: `WasmAdapter` polling lifecycle, `TauriAdapter` event listener cleanup on `stopRouting`, the sentinel-value handling (`VIRTUAL_COMPUTER_KEYBOARD`), and adapter selection in `adapter/index.ts`.
- Files: `ui/src/lib/adapter/wasm.ts`, `ui/src/lib/adapter/tauri.ts`, `ui/src/lib/adapter/index.ts`
- Risk: Regressions in MIDI routing lifecycle (stuck notes, double-listen, leaked `requestAnimationFrame` handles) are not caught.
- Priority: High

**No integration tests for the Tauri router thread:**
- What's not tested: The live routing loop in `src-tauri/src/commands/engine.rs` (`run_tauri_router`), config snapshot capture, and stop-signal behavior.
- Files: `src-tauri/src/commands/engine.rs` lines 185-307
- Risk: The threading and shared-mutex logic is the highest-complexity area and relies entirely on manual testing.
- Priority: High

---

*Concerns audit: 2026-03-31*
