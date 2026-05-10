# Research: Group H — Bugs & Refactors (#14, #90, #91, #102)

**Issues:** #14, #90, #91, #102
**Date:** 2026-05-11
**Researcher:** issue-researcher
**Verdict:** mixed — see per-issue breakdown
- #14 — bug fix in `src-tauri/src/commands/engine.rs` + small UX heuristic in `MidiDevices.svelte` (XS)
- #90 — bug fix in router companion path (in-repo core) (S)
- #91 — refactor — **partially obsolete**; re-scope to companion `LaneOutput` API (S)
- #102 — in-repo plugin (`crates/contrapunk-ml/`) + external model file shipped out-of-band (L)

---

## #14 — MIDI out not producing messages for some users

### Problem
After Start Routing with a MIDI input and a MIDI output selected, input notes appear in the UI but the connected MIDI watcher/synth receives nothing on the output port. Reported on macOS at v1.0.0 by HN user `dundercoder`. Confirmed reproducible by `platypython` "after pulling latest and rebuilding" — i.e. the bug exists at v1.0.0 but the symptom is not present on a recent HEAD build for at least one user.

### Touchpoints
- `src-tauri/src/commands/engine.rs:763-797` (handle_note_on — fan voices via `voice_outputs[i]`)
- `src-tauri/src/commands/engine.rs:945-984` (`dispatch_voice` — actually emits the bytes)
- `src-tauri/src/state.rs:31-41` — `VoiceOutputTarget` enum; **`Default = Synth`**
- `src-tauri/src/state.rs:138-143,176` — `voice_outputs: Vec<VoiceOutputTarget>`, initialised `vec![Synth; MAX_VOICES]`
- `ui/src/lib/components/MidiDevices.svelte:97-118` — UI calls `setVoiceOutput(slot, { kind: 'midi_port', port })` **only when the user manually opens each per-voice dropdown and picks the device**.
- `crates/contrapunk-midi/src/output.rs:38-75` — `OutputRouter::new` (connection setup, no post-connect delay)
- v1.0.0 reference: `src-tauri/src/commands/engine.rs:559-571` at commit `9eeeff5` — `if port >= num_outputs { continue; }` silently dropped voices whose `port_map[i]` exceeded `num_outputs`.

### Architecture verdict
Bug — multi-root, not a midir platform quirk. Two distinct root causes, one historical, one current:

**Root cause A (v1.0.0 era — the original HN report):** Pre-#59/#60, the router fan-out used `port_map[i]` directly against `num_outputs`. With a single MIDI output selected and a non-trivial harmony (`voice_count > 1`), voices 1..N had `port = i >= 1` and hit the `if port >= num_outputs { continue; }` drop at `engine.rs:562` in commit `9eeeff5`. The melody voice (index 0) *should* have reached the wire, but if `port_map[0]` resolved to a non-zero slot (e.g. `voice_position = soprano-with-arrangement`) every voice could drop. Issue body's own "Possible cause" #1 nails this. **Fixed accidentally** in PR #59/#60 (per-voice routing) — explaining `platypython`'s "works on latest build" comment.

**Root cause B (post-#60, today):** `VoiceOutputTarget::default()` is `Synth` and `voice_outputs` initialises to `[Synth; 8]`. A user who picks a MIDI output device in the top selector but never opens the per-voice dropdowns at `MidiDevices.svelte:97-118` sends every voice to internal synth — external MIDI gets nothing. The behavior is consistent with the comment at `state.rs:30` ("Default is Synth so users get audio out of the box"), but it directly causes the reported symptom whenever the user's mental model is "I picked MIDI out, that should be enough."

Fix is in two places: (1) when the user adds the first device to `selectedOutputs`, **auto-assign Voice 1 (melody) to that port** if it's still `Synth`; (2) make `start_routing` log a warning when `output_indices.len() > 0` but no `voice_outputs` entry is `MidiPort`. Optionally also apply the midir `#167` 50-100 ms post-connect delay as defence-in-depth.

This is **not** a duplicate of #79 — #79 is the guitar-audio pitch-wobble bug, separate code path. The HN reporter did not state guitar input.

### Implementation outline
1. **TDD first test** — `tests/regression/issue_14_default_routing.rs`: instantiate `AppState::new()`, simulate "user picks MIDI device 0" via `midi.toggleOutput(0)`. Assert that `voice_outputs[0]` becomes `MidiPort { port: 0 }` automatically. (UI integration test mirrors this — Playwright clicking the device picker.)
2. UI change in `ui/src/lib/stores/midi.svelte.ts::toggleOutput` (line ~291): when transitioning `selectedOutputs` from empty to non-empty AND `voiceOutputs[0].kind === 'synth'`, call `setVoiceOutput(0, { kind: 'midi_port', port: 0 })`. Guard so it never overwrites a user-set value (compare against a hydrated-from-localStorage flag).
3. Add a startup-log warning in `run_tauri_router` after `OutputRouter::new`: if `num_outputs > 0` and `voice_outputs` snapshot contains zero `MidiPort` entries, `eprintln!("[router] WARNING: {} MIDI port(s) connected but no voice routes to them. Open the Output panel and assign at least one voice.")`.
4. (Optional defence) Apply a `thread::sleep(Duration::from_millis(50))` after `OutputRouter::new` returns and before the first NoteOn can fire. Matches the midir `#167` workaround. Low-cost; eliminates a separate "first NoteOn lost" failure mode reported by midir users.

### Test strategy
- **First test (TDD):** UI store unit test — toggling first output flips Voice 1 from Synth to MidiPort.
- Rust integration test against `AppState`: simulate `set_voice_output(_, MidiPort)` was never called, then run the routing loop with a scripted MIDI input; capture `OutputRouter` send calls via a test-only `OutputRouter::with_capture(Vec<Vec<u8>>)` constructor. Assert no NoteOn appears in capture and an `eprintln!` warning is emitted.
- Manual UAT: macOS, IAC bus, Logic Pro receiver — confirm notes flow on the wire end-to-end with default settings.

### Dependencies
None new.

### Entropy impact
Tiny. One UI hydration heuristic + one Rust log line + one optional sleep. No new surfaces, no build-time cost.

### Open questions
- Should we revisit `VoiceOutputTarget::default() = Synth` for a future major release? Changing the default to "auto-MidiPort if at least one port connected, else Synth" is a per-session decision that doesn't fit a static `Default` impl. The heuristic in (2) above is the cleanest expression.

### Estimated effort
**XS** (≤ 1 day, including test + manual UAT on macOS).

---

## #90 — held_harmonies stale-entry recovery (dropped Note-Off, MPE rotation)

### Problem
The router tracks currently-sounding harmony voices keyed by *input* MIDI note. If the device drops a Note-Off (USB yank, MPE channel rotation that re-keys mid-phrase, host pause/resume), the entry never gets cleared and pattern-tick / future Lane retriggers continue firing ghost voices until the user stops/starts routing.

### Touchpoints
- `src-tauri/src/companion/world.rs:46-50` — `HeldVoice` struct (moved from inline router state into `WorldState` during companion Phase 1.4)
- `src-tauri/src/companion/world.rs:108-113` — `sounding_voices: Arc<Mutex<HashMap<u8, Vec<HeldVoice>>>>` — the actual "held_harmonies" map
- `src-tauri/src/companion/world.rs:107` — `held_inputs: Arc<Mutex<HashMap<u8, HeldInput>>>` — the input-side equivalent (also susceptible to stale entries)
- `crates/contrapunk-harmony/src/engine.rs:253` — `active_notes: HashMap<u8, Vec<Note>>` — engine-side mirror; `clear_active_for_reharm` at `engine.rs:1327-1332` is the only existing reset path
- `src-tauri/src/commands/engine.rs:836-919` — `handle_note_off` (removes from tracking sets on a *received* Note-Off; offers no recovery if the Note-Off never arrives)
- `src-tauri/src/commands/engine.rs:1013-1041` — `broadcast_note_off` (per-channel limitation already documented at line 1006: "voices originally attacked on non-zero channels (MPE) will not be matched")
- `.planning/jam-features-2026/01-companion-architecture.md` § "Architectural debts that remain" — explicitly defers #90 root fix to a later session, suggesting "defensive CC123 ops mitigate."

### Architecture verdict
Bug. Pick the **engine-cross-reference recovery** approach (option 4 in the issue body) — the only candidate without false-release risk. Engine `active_notes` is the authority because it's mutated synchronously inside the same `engine.lock()` scope as `handle_note_on` / `handle_note_off`. Cross-referencing router-side `sounding_voices` against `engine.active_notes()` post-event (not on a TTL timer) drops orphans only when there's no plausible source for them.

The arch doc commits to a `panic_pending → typed EngineMutation` refactor "once Wk 2 ChordSeq Lane lands." Issue #90's root fix slots cleanly into that work — the orchestrator's `Sense` phase already runs every router-loop iteration with an engine snapshot in hand. Adding a "drift reconciliation" Sense step there costs ~30 LOC.

CC 123 (option 3) is a valuable fast-path escape but doesn't replace the cross-reference; ship both.

### Implementation outline
1. **TDD first test** — `companion::world::tests::sounding_voices_reconciles_against_engine`:
   - Set up `WorldState` with a scripted scenario: input 60 on channel 0, harmonize-on emits voices 64/67. Manually insert a stale entry for input 72 → voices 75/79 into `sounding_voices` (simulating the dropped Note-Off). Call a new `WorldState::reconcile_sounding_voices(engine: &HarmonyEngine)`. Assert that the 60 entry survives and the 72 entry is dropped, and the function returns the dropped voices as `Vec<HeldVoice>` so the caller can issue NoteOffs.
2. Add `HarmonyEngine::active_input_notes(&self) -> impl Iterator<Item = u8>` if it doesn't already exist (`active_notes.keys().copied()` is a one-liner).
3. Add `WorldState::reconcile_sounding_voices(engine) -> Vec<HeldVoice>` returning the orphans for the router to NoteOff via existing `broadcast_note_off`.
4. Call the reconcile step at the top of the router-loop iteration (companion Sense phase) at a low frequency — every 500 ms is plenty; orphan ghosts being audible for half a second is preferable to false-releasing real notes.
5. **Add CC 123 handling** in `process_midi_message` (`src-tauri/src/commands/engine.rs:643`): when the CC number is 123 (All-Notes-Off), call `broadcast_note_off` for every entry in `sounding_voices`, clear `held_inputs` and `sounding_voices`, and `engine.clear_active_for_reharm()` (or a new `engine.panic_clear()` without the reharm side effect).
6. **MPE rotation** fix: per the limitation noted at `engine.rs:1003-1006`, broadcast_note_off currently uses channel 0. Capture the original Note-On channel in `HeldVoice` (already there — `channel: u8`) and use it when broadcasting. Already wired; only needs the channel propagation in `broadcast_note_off`'s callers to pass the right value.

### Test strategy
- **First test (TDD):** reconcile drops orphans (unit-test scope, `WorldState` alone).
- Unit test: CC 123 received → all sounding voices released, per-channel respected.
- Integration test (router-thread mock): script a stuck input → reconcile fires → orphan harmony goes silent within 500 ms. Capture via `OutputRouter::with_capture`.
- Manual UAT: hot-plug USB MIDI keyboard mid-phrase (yank cable while holding C4) — verify chord doesn't stick after CC 123 or after 500 ms.

### Dependencies
None new.

### Entropy impact
Low. Touches one new method on `HarmonyEngine`, one new method on `WorldState`, one CC handler. No new threads, no new locks. Aligns with the planned `EngineMutation` refactor — paying down debt while shipping a fix.

### Open questions
- 500 ms reconcile cadence is a guess. Profile if it shows up in router-loop tick budget (unlikely — `HashMap::keys().collect()` over <16 entries is ~ns).
- Should the engine itself perform reconcile on every Note-On (cheap, eager) instead of the router on a timer? Possible alternative; defer to the EngineMutation refactor.

### Estimated effort
**S** (1-3 days).

---

## #91 — Extract router-loop pattern-tick logic into pure function

### Problem (as filed)
Issue claims `run_tauri_router` is ~580 lines with ~465 lines of loop body, owns 3+ mutexes / 4 atomics / 2 channels, embeds pattern-decision logic (F2/F4/F5/M3/H3 invariants) inline, and these invariants have no unit-test coverage.

### Problem (as currently true)
**The pattern-tick code referenced in #91 has been removed.** Commit `1376c4c` ("refactor(pattern): remove pattern programmer, keep metronome") deleted the pattern programmer; `9d50b25` introduced the companion architecture as its replacement. A grep for `pattern_cell` / `last_pattern_cell` / `pattern_dispatch` / `F2|F4|F5|M3|H3` in `src-tauri/src/commands/engine.rs` returns **zero hits**. The TODO marker at engine.rs:569-572 (per issue body) no longer exists. The arch doc explicitly says: **"Subsumes follow-up issue #91 (pure function extraction for testability) — `Lane::tick()` *is* the pure function."**

`run_tauri_router` itself is still long (1041 lines total in engine.rs, ~280 of which is the router loop body) — the *refactor* problem is real, but the *specific invariants* in the issue body do not exist anymore.

### Touchpoints
- `src-tauri/src/companion/lane.rs:1-100` — `Lane` trait already designed as the pure-function abstraction
- `src-tauri/src/companion/orchestrator.rs:75-80` — `Companion::tick()` returns `Vec<DispatchOp>` from Sense+Mutate+Decide phases
- `src-tauri/src/companion/world.rs:94-118` — `WorldState`, the snapshot input
- `src-tauri/src/commands/engine.rs:286-640` — the still-too-large `run_tauri_router` body

### Architecture verdict
Refactor — but **re-scope**. The pure-function extraction the issue asked for is already done by the companion abstraction. What remains is:
(a) the architecture is in place but **no concrete Lanes are registered** at `Companion::new` (orchestrator.rs:67-73). Phase 2's LooperLane will be the first.
(b) the router-loop pre-companion code (panic-replay, detune pitch-bend, knob-cc-raw forwarding, note-update emission) is still inline in `run_tauri_router` and remains hard to unit-test.

The right phase to write is **"wire `Companion::tick()` into `run_tauri_router` and write golden tests for the orchestrator's Sense/Mutate/Decide ordering"** rather than the issue's original "extract a `decide_pattern_dispatch` function." The latter has been pre-empted.

Recommendation: **close #91 with a comment pointing to the companion arch doc**, then file a successor "Wire Companion into router-loop + add golden tests" issue that reflects the new shape.

### Implementation outline (for the successor issue)
1. **TDD first test** — `companion::orchestrator::tests::test_lane_runs_in_phase_order`:
   - Register three `TestLane` instances (one per phase) that each record their tick number to a shared `Vec<u32>`. Call `tick()` three times. Assert order is Sense→Mutate→Decide across each tick, and that the `WorldState` snapshot seen by Mutate reflects writes by Sense.
2. Add a stub `Companion` to `AppState` (default-disabled) so the field exists.
3. Call `Companion::tick()` once per router-loop iteration after the existing detune/CC handling. Convert returned `Vec<DispatchOp>` via the existing `dispatch_voice` helper (already shaped for this).
4. Call `Companion::on_input(InputEvent::NoteOn { .. })` at the top of `handle_note_on` / `handle_note_off`. If `suppress_default == true`, skip the default `engine.harmonize_note_on`.
5. Add a no-op Lane fixture under `#[cfg(test)]` so the orchestrator can be tested in isolation. Confirm `LaneOutput` shape matches `dispatch_voice`'s expectations.

### Proposed pure-function signature (for #91 as filed, if the user insists on the original framing)
```rust
pub fn decide_pattern_dispatch(
    state: &PatternStateSnapshot,
    transport: TransportPhase,
    cfg: &PatternConfig,
) -> Vec<DispatchOp> { ... }
```
**Do not implement.** Pattern code is gone; `Lane::tick(&dyn Lane, &WorldState, &mut HarmonyEngine) -> LaneOutput` is its successor.

### Test strategy
- **First test (TDD):** `Companion::tick` runs lanes in phase order; `WorldState` reads after Sense reflect Sense writes.
- Test: `Companion::on_input` with `suppress_default = true` causes router to skip default harmonize (use a recording engine mock).
- Test: A no-op Lane registered in all three phases produces zero `DispatchOp`s.
- Integration: simulate the F4/F5 scenarios from the original issue with a concrete LooperLane (Wk 1 jam feature, ships Apr 30 — already on calendar).

### Dependencies
None new.

### Entropy impact
Low. The abstraction already exists. Wiring it into `run_tauri_router` reduces engine.rs line count by ~150 lines (panic-replay block lifts cleanly into a Mutate-phase Lane in a future phase). Net entropy: negative.

### Open questions
- Whether to close #91 outright or rebrand it. Recommend the GH issue be re-titled "Wire Companion::tick into router loop + Sense/Mutate/Decide tests" with a comment linking the arch doc.

### Estimated effort
**S** (1-3 days for the wiring + tests; concrete Lanes are separate phases).

---

## #102 — ListenLane: backing track capture + stem separation via Demucs ONNX

### Problem
Add a Sense-phase ListenLane that captures audio from a virtual loopback device (BlackHole on macOS), separates stems via Demucs ONNX, runs the "other" (chords/pads) stem through polyphonic pitch detection, and writes the detected key into `WorldState` / `HarmonyEngine::set_key`. Enables "jam along with a Spotify backing track, auto-locked to its key."

### Touchpoints
- `crates/contrapunk-audio/src/` — sister crate to the new `contrapunk-ml`. Has existing `cpal` capture surface in `src-tauri/src/guitar_bridge.rs:24,95`.
- `src-tauri/src/companion/lane.rs:32-40` — `LanePhase::Sense` is the right home (writes WorldState, doesn't dispatch ops).
- `src-tauri/src/companion/world.rs:117` — `current_chord: Arc<Mutex<DetectedChord>>` — ListenLane writes here.
- `crates/contrapunk-harmony/src/engine.rs` — `HarmonyEngine::set_key` is the Mutate target (called through orchestrator).
- `crates/contrapunk-harmony/src/key_detect.rs` — existing `KeyDetector` — reuse for histogram→key inference.
- `src-tauri/src/audio_clock.rs:209-240` — `cpal::Stream` setup pattern to copy.
- (Wrong upstream dep in issue body): `#98` is *Ableton Link*, not BlackHole-audio-input. The actual BlackHole-input infra has to come from this issue itself.
- (Wrong upstream dep in issue body): `#97` is the Sampler engine; reasonable to share `ort` setup with that crate, but it's not a hard blocker.

### Architecture verdict
**In-repo plugin** (`crates/contrapunk-ml/`) + **external model file**, with the model not shipped in the binary. The crate fits as an optional Cargo workspace member with feature flag `ml-listen`. Build the desktop installer with the feature on; WASM build keeps it off (Demucs ONNX is too heavy for browser delivery — 80 MB model would dwarf the entire WASM bundle).

Two reasons in-repo wins over external sub-project:
1. The Lane abstraction is already in-repo; ListenLane is a tight Sense-phase Lane that reads `WorldState` and writes engine key. The cross-process boundary buys nothing here — the model runs in a background thread inside the same Tauri process, output is a single `set_key` call per 4 bars.
2. `ort` (2.0.0-rc.12, MIT+Apache-2.0, active development through March 2026) has a `download-binaries` feature that fetches the ONNX runtime shared library at build time. Compiled binary stays small; the heavy lifting is the model file.

The model file is the entropy land mine. **Do not commit `htdemucs.onnx` to the repo.** Ship a "Download backing-track model" button in the Listen panel that fetches the model on first use into `app_data_dir/models/htdemucs.onnx` and verifies a SHA-256. Cloudflare Pages / Cloudflare R2 (already hosting `contrapunk.com`) is a free CDN for the file.

### Implementation outline
1. **TDD first test** — `contrapunk_ml::demucs::tests::separate_30s_clip_returns_four_stems`:
   - Load a small fixture WAV (10-30s, MIT-licensed, e.g. one of the freesound CC0 chord progressions). Call `DemucsModel::separate(&audio)`. Assert returned `DemucsStem { vocals, drums, bass, other }` each have the same sample count as input and that `other.iter().map(|s| s.abs()).sum()` > zero (non-trivial output).
2. Create `crates/contrapunk-ml/` workspace member:
   - `src/lib.rs`, `src/demucs.rs`, `src/basic_pitch.rs`, `src/model_loader.rs` (path resolver + checksum verifier).
   - Feature flag in root `Cargo.toml`: `[features] ml-listen = ["dep:ort", "dep:hound"]`.
3. Add `src-tauri/src/companion/lanes/listen_lane.rs` implementing `Lane` with `phase = Sense`, `input_filter = None` (tick-only). Construct `ListenLane::new(model_path, world)` lazily — model only loads when the user enables the lane.
4. Add `src-tauri/src/audio_capture/blackhole.rs` (or extend `guitar_bridge.rs`): cpal input stream on a named device, ring-buffered into 8-second windows with 1-second overlap (per StemRoller pipeline).
5. Decide step: every 4 bars (read from `transport.beat_position()`), gate the `set_key` call on confidence > 0.85.
6. Model download UI: panel button → `tauri::api::http::download` → SHA-256 check → write to `app_data_dir`. Surface progress in the panel.
7. Build matrix: `cargo build --features ml-listen` for desktop; default build skips the crate entirely so WASM and plugin builds stay fast.

### Test strategy
- **First test (TDD):** 30-second clip separation produces 4 non-empty stems.
- Unit: `model_loader` correctly resolves `app_data_dir/models/htdemucs.onnx`, fails fast with a typed error if file is missing or checksum mismatches.
- Integration: 60-second I-IV-V-vi MIDI-rendered backing track → ListenLane → detected key matches the rendered key with > 80% accuracy across 12 keys (acceptance criterion from the issue).
- Manual UAT (macOS): BlackHole 2ch installed, Spotify Web Player → BlackHole → ListenLane → C major track produces `set_key(C)` within ~4 bars.
- Performance: assert separation latency on a 30-second clip < 15 s on an M-series Mac (so 4-bar cadence stays real-time; htdemucs separates ~2× real-time on M1 per `sevagh/demucs.onnx` reports).

### Dependencies
- `ort = "2.0.0-rc.12"` (Apache-2.0 + MIT) — minimal binary overhead per pyke/ort docs; `download-binaries` feature handles ONNX Runtime native library.
- `hound = "3.5"` (Apache-2.0) — also added by #97 SamplerAudioBlock; share.
- `htdemucs.onnx` model file — ~80 MB. **Not vendored**, fetched at runtime. Source: `sevagh/demucs.onnx` MIT-licensed conversion (active through 2024-11-11), or roll our own from `facebookresearch/demucs` (MIT) using the ort-builder strategy referenced in the Mixxx GSoC 2025 write-up.
- `basic-pitch` ONNX model from Spotify — ~17 MB, MIT — fetched same way for polyphonic pitch detection.
- BlackHole 2ch system extension — user-installed, not bundled. Acceptable for a macOS-first feature.

### Entropy impact
Moderate. Adds one new workspace crate (`contrapunk-ml`), one feature flag (`ml-listen`), one new Cargo dependency tree (ort + its native ONNX Runtime download). Build time on the desktop target grows ~30s for the first build (ONNX Runtime native lib download), subsequent builds use the cached lib. WASM and plugin builds unaffected — `ml-listen` is off by default.

Affects:
- `Cargo.toml` (workspace + feature flag)
- `src-tauri/src/companion/` (new `lanes/listen_lane.rs`)
- `src-tauri/src/audio_capture/` (new module for BlackHole input)
- UI: new Listen panel (parity with existing companion lane panels)
- `app_data_dir` shape (new `models/` subdir)

Risk of regression in unrelated areas: low, gated behind the feature flag.

### Open questions / blockers
- **Can htdemucs be exported as a single ONNX file?** The Mixxx GSoC 2025 work suggests yes for v4 but with non-trivial conversion effort. `sevagh/demucs.onnx` already has working artifacts. Spike: 1 day to verify the export loads in `ort` and produces sane output.
- **Model-fetch policy on first run**: opt-in download (preferred) vs auto-download. Opt-in respects user bandwidth; auto-download is friendlier. Default to opt-in with a clear "Enable backing-track listen" button.
- **iPad / mobile**: not a target for this Lane. WASM build flag stays off.
- **Basic Pitch alternative**: if Basic Pitch ONNX bundle adds too much complexity, fall back to a simple FFT pitch-class-histogram on the "other" stem — KeyDetector already works on PC histograms. Probably good enough for key detection; chord-name display can wait.

### Estimated effort
**L** (1-3 weeks). Mostly driven by the ONNX integration work, fixture corpus, accuracy tuning. The Lane wiring itself is days; the ML pipeline is the bulk.

---

## Cross-issue summary

| Issue | Verdict | TDD first test | Effort |
|---|---|---|---|
| #14 | Bug — UI hydration heuristic + log warning | `toggleOutput → voiceOutputs[0] becomes MidiPort` | XS |
| #90 | Bug — engine cross-ref reconcile + CC 123 | `sounding_voices reconciles against engine.active_notes` | S |
| #91 | Refactor — **partially obsolete**; re-scope to companion wiring + tests | `Companion::tick runs lanes in phase order` | S |
| #102 | In-repo plugin (`crates/contrapunk-ml/`) + external model file via CDN | `DemucsModel::separate returns 4 stems for 30s clip` | L |

---

## Sources

- midir Boddlnagg crate, issue #167 (first-message-after-connect lost; 50-100 ms delay workaround): https://github.com/Boddlnagg/midir/issues/167
- midir Windows quirk #122, #156: https://github.com/Boddlnagg/midir/issues
- `ort` Rust ONNX runtime, v2.0.0-rc.12 (2026-03-05), Apache-2.0/MIT: https://github.com/pykeio/ort
- `sevagh/demucs.onnx` C++ ONNX reference (MIT, active through 2024-11-11): https://github.com/sevagh/demucs.onnx
- Mixxx GSoC 2025 Demucs v4 → ONNX writeup (Oct 2025): https://mixxx.org/news/2025-10-27-gsoc2025-demucs-to-onnx-dhunstack/
- Companion architecture: `.planning/jam-features-2026/01-companion-architecture.md`
- Codebase concerns: `.planning/codebase/CONCERNS.md`
