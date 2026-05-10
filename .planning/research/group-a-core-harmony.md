# Research: Group A — Core Harmony Features

**Issue(s):** #4, #81, #3, #42, #65, #100, #8
**Date:** 2026-05-11
**Researcher:** issue-researcher
**Verdict:** All seven are **in-repo core** (`crates/contrapunk-harmony/`, `crates/contrapunk-preset/`, or thin UI). None warrant a separate crate or external sub-project. Per-issue verdicts below.

Test patterns referenced throughout: existing engine.rs mod tests block (`crates/contrapunk-harmony/src/engine.rs:1360+`) — see `test_species_change_alters_harmony_output` (~L2249), `auto_key_change_queues_old_harmonies_for_release` (~L2305), and `test_species2_differs_from_species1_without_transport` (~L2509) as exemplars for "behavior must visibly diverge" tests and for "auto-key wires `pending_releases`" tests.

---

## #4 — Auto-detect key from played notes

**Verdict:** In-repo core. **Status: shipped, awaiting hands-on UAT.** This research is a recommendation to close-as-done after a tuning pass and to fold its remaining lifecycle work into #81 (the algorithm upgrade).

### Problem
First-time user shouldn't have to pick a key from a dropdown before hearing harmony — the engine should infer the tonic from played notes.

### Touchpoints
- `crates/contrapunk-harmony/src/key_detect.rs` — decay-weighted PC histogram (`DECAY=0.85`, `MIN_NOTES=4`, `CONFIDENCE_MARGIN=0.15`).
- `crates/contrapunk-harmony/src/engine.rs:1248-1287` — `harmonize_note_on` feeds detector, populates `pending_releases` on key change.
- `src-tauri/src/commands/harmony.rs:173` — `set_auto_key`.
- `wasm/src/lib.rs:461` — `set_auto_key` binding.
- `ui/src/lib/stores/engine.svelte.ts:662` — `autoKey` toggle.

### Architecture verdict
In-repo core, already there. Detector lives next to the engine, no boundary to add. PR #80 already handled the stuck-notes lifecycle.

### Implementation outline
Remaining work (post-shipping):
1. Real-music tuning pass. Issue #80 explicitly leaves "close-relative key transitions (C↔G)" as laggy. Adjust `CONFIDENCE_MARGIN` / `DECAY` against recorded phrases.
2. Add hysteresis: refuse a key flip within N notes of the previous flip to stop bouncing on shared-PC modulations.
3. Drop `println!("[AUTOKEY] …")` at `engine.rs:1262` once detector is trusted (or move behind a `tracing` macro).
4. Close ticket after #81 lands — #81's `(tonic, mode)` detector subsumes this.

### Test strategy
TDD-first, before any tuning change:
- "C→G modulation locks within 5 notes" — extension of `test_key_detector` pattern.
- "no-flap test": shared-PC content (chromatic passing tones) does not flip the key.
- "hysteresis": after committed flip, next 4 notes can't trigger another flip.

These all follow the `auto_key_change_queues_old_harmonies_for_release` style at `engine.rs:2305` — drive the engine through a melody, assert `engine.key()` at checkpoints.

### Dependencies
None new.

### Entropy impact
Zero new. Already shipped on all four surfaces.

### Open questions / blockers
Does the user want this auto-on for first-time users in DEMO mode? UX call, not engineering. Track separately.

### Estimated effort
**XS.** Tuning + cleanup + close. Real work lives in #81.

---

## #81 — Auto-key: detect mode + tonic via per-mode Krumhansl profiles

**Verdict:** In-repo core. Algorithm rewrite of `key_detect.rs`. The upgrade path for #4.

### Problem
Current detector only finds tonic against a user-locked mode. Cannot distinguish modes that share pitch classes (Ionian vs Aeolian — same 7 PCs) or detect mid-session mode shifts (C Ionian → C harmonic minor).

### Touchpoints
- `crates/contrapunk-harmony/src/key_detect.rs` — `score_tonic` (line 105) is the binary in/out-of-scale scorer to replace.
- `crates/contrapunk-harmony/src/config.rs:315-708` — `ScaleMode` enum (57 variants). Need `profile()` method.
- `crates/contrapunk-harmony/src/engine.rs:1250-1271` — auto-key path. Must call both `set_key` AND `set_scale_mode` on detection; both must funnel through `pending_releases` (currently only `set_key` does).
- `engine.rs:554` — `set_scale_mode` already updates the detector's scale_mode. Reverse-couple: detector now produces the mode.

### Architecture verdict
In-repo core. The detector is already in the harmony crate; this is a function rewrite, not a structural change. Adding `profile()` to `ScaleMode` is a natural extension — same pattern as `intervals()` (config.rs:468). No new dependencies needed.

### Implementation outline
1. **Add `ScaleMode::profile(self) -> [f32; 12]`.** Krumhansl-Kessler published profiles for Ionian (major) and Aeolian (minor): `[6.35, 2.23, 3.48, 2.33, 4.38, 4.09, 2.52, 5.19, 2.39, 3.66, 2.29, 2.88]` and `[6.33, 2.68, 3.52, 5.38, 2.60, 3.53, 2.54, 4.75, 3.98, 2.69, 3.34, 3.17]`. For other modes, derive analytical profiles: assign high weight (≈6.0) to tonic, secondary peak (≈5.0) to dominant (or modal characteristic tone — e.g. ♭2 for Phrygian), moderate (≈3.5) to other scale tones, low (≈2.5) to chromatic. This is sufficient for distinguishing modes; published profiles for non-diatonic modes are scarce in the literature.
2. **Restrict candidate space.** 12 tonics × 57 modes = 684 candidates is too many — most will never appear. Limit to the user's *family* selection (e.g. Diatonic + HarmonicMinor + MelodicMinor when "common modes" is selected) via a `candidate_modes: Vec<ScaleMode>` field on `KeyDetector`. Default = the 21 modes in those three families.
3. **Rewrite `KeyDetector::detect`** to return `Option<(Key, ScaleMode)>`. For each candidate pair, rotate the profile to that tonic and compute dot product against the histogram. Confidence margin against runner-up across the full candidate space.
4. **Wire engine commit.** In `harmonize_note_on` (engine.rs:1250), on confident detection: gather old harmonies into `pending_releases` (existing pattern, line 1259), then `set_key(detected.0)` and `set_scale_mode(detected.1)` only when changed.
5. **Re-tune `DECAY` / `MIN_NOTES`** — published Krumhansl-style work uses ~20 notes for stable detection. Longer minimum (e.g. 8) reduces false flips on mode-ambiguous opening phrases.

### Test strategy
TDD-first, mirroring `auto_key_change_queues_old_harmonies_for_release` (engine.rs:2305):
- **parallel mode shift**: feed C Ionian (C D E F G A B), then C harmonic minor (C D Eb F G Ab B). Assert key stays C, mode switches Ionian → HarmonicMinor.
- **relative mode shift**: C Ionian → A Aeolian. Assert both `key()` and `scale_mode()`.
- **stability under chromatic passing tones**: feed C Ionian with chromatic F# passing tones — must not flip to Lydian.
- **profile sanity**: every `ScaleMode::profile()` returns a 12-element array summing to ~40, tonic bin is the max.
- **dispatch test**: confirm `set_key + set_scale_mode` both feed `pending_releases` (currently only `set_key` does — easy regression).

Write all five before touching `detect()`.

### Dependencies
None. Pure math in existing crate.

### Entropy impact
- One new `ScaleMode` method (low-risk, every variant must implement; compile-time enforced).
- One new `KeyDetector` field (`candidate_modes`).
- 536+ existing tests must stay green — the issue body promises this.

### Open questions / blockers
- Defining analytical profiles for the 50+ non-diatonic/non-Aeolian modes. Recommend: ship with Krumhansl profiles for the 21 modes in `Diatonic + HarmonicMinor + MelodicMinor`, leave the rest as a flat (no detection) profile. User selects which families participate. Avoids fabricating data.
- Modal-interchange interaction is explicitly out-of-scope per the issue.

### Estimated effort
**M** (3-7 days). Profile data + rewrite + 5 tests + tuning pass.

---

## #3 — Canon mode: generated voices perform a canon of played notes

**Verdict:** In-repo core, new `HarmonyMode` variant. Modest scope.

### Problem
Add a harmony mode where generated voices replay the player's melody at a delay, optionally transposed by a diatonic interval — classical canon (round).

### Touchpoints
- `crates/contrapunk-harmony/src/config.rs:823` — `HarmonyMode` enum. Add `Canon` variant + `number()` / `description()` / `tooltip()` arms.
- `crates/contrapunk-harmony/src/engine.rs:1198-1287` — dispatch in `harmonize_single` / `harmonize_note_on`. Canon needs a per-voice delay buffer that the engine must own.
- `crates/contrapunk-harmony/src/stateful.rs` — pattern for per-voice state (see `CounterpointState`). Add `CanonState { delay_beats: f32, transpose_degrees: i8, buffer: VecDeque<(beat_pos, Note)> }`.
- Beat awareness: use `synthetic_beat_counter` (engine.rs:278) and/or `counterpoint_beat_phase` — same pattern used for Species 2-4.
- WASM/Tauri: no new commands; `set_mode("canon")` already routes via existing `set_mode` binding.

### Architecture verdict
In-repo core. Canon is harmony — fits next to counterpoint. Per-voice delay buffer is bounded (few seconds at most) so no memory concern. Beat-aware but already-built infrastructure (synthetic beat counter) carries it.

### Implementation outline
1. Add `CanonState` to `stateful.rs` with config (`delay_beats`, `transpose_degrees`) and a small `VecDeque<(f64, u8)>` of `(beat_position, midi)` pairs.
2. Engine: on `harmonize_note_on`, push `(beat_pos, midi)` to every CanonState. Each state pops entries whose `beat_pos + delay_beats <= current beat_pos` and emits them as harmony notes (diatonically transposed by `transpose_degrees`).
3. **Critical correctness:** Canon emits notes *between* user note-ons. The current engine only fires output on user input. Two options:
   - **a)** Have the router tick the engine on the beat clock and drain pending canon emissions. Requires a new `engine.tick_canon(beat_pos) -> Vec<(Note, NoteState)>` method called from the router thread on each beat crossing (audio_clock.rs:148 emits `beat-update` already).
   - **b)** Emit only when next user input arrives — voices clump on user note-ons rather than playing in time. **Reject (b)**: not a real canon.
4. UI: expose `delay_beats` (0.5–4) and `transpose_degrees` (-7 to +7, default +0/unison) controls. Reuse the species/strictness panel layout.
5. Note-Off: when a canon voice's emitted note expires (after `delay_beats + held_duration`), engine must send Note-Off. Add an `expiry_beat` to each VecDeque entry.

### Test strategy
TDD-first:
- "canon emits user's notes at +N beats" — feed a C scale at beat positions 0, 1, 2, …, set `delay_beats=2`, advance synthetic clock, assert canon voice outputs the same sequence at beats 2, 3, 4, …
- "canon transposes diatonically" — `transpose_degrees=2`, C Ionian, input C → canon output E.
- "canon respects mode change" — set canon, change key mid-stream, assert pending canon notes get into `pending_releases` (parallel to auto-key release path).
- "tick_canon returns empty when buffer is empty" (baseline).

Follows the `test_species_change_alters_harmony_output` (engine.rs:2249) pattern — drive a melody, assert output divergence.

### Dependencies
None.

### Entropy impact
- One new `HarmonyMode` variant ripples to UI mode selectors, mode-name maps (likely 3-4 places: Svelte mode dropdown, Tauri `parse_harmony_mode`, WASM equivalent).
- New beat-clock-driven `tick_canon` path is a structural addition to the router thread. Risk: real-time-safety regression. Mitigation: emit canon events through the same MIDI queue path used by audio_clock.rs.
- 4 surfaces touched: CLI / WASM bindings inherit for free; Tauri router needs a tick; nih-plug needs equivalent (deferred).

### Open questions / blockers
- Does canon work without a transport? Synthetic beat counter advances on user notes only, so canon emissions arrive on the *next user note-on* without a real clock. May be acceptable as a degraded mode; document.
- Multi-voice canons (3-voice canon at +2, +4)? Trivial extension once 1-voice works.

### Estimated effort
**M** (3-7 days). Router tick is the load-bearing piece.

---

## #42 — AUTOPLAY scale walker through the real harmony engine

**Verdict:** In-repo, UI-side only. **Critical finding: `src/generator/` does not exist in the current workspace** — `crates/contrapunk-harmony/src/` contains no `generator` module. The CONCERNS.md note about "dead GeneratorEngine" is stale relative to the current layout. AUTOPLAY should be built fresh in the Svelte/adapter layer, not by wiring up a Rust generator.

### Problem
Demo button that walks a scale through the live engine so first-time users hear harmony without playing.

### Touchpoints
- `ui/src/lib/components/` — new `AutoPlay.svelte` component or inline in DEMO hero.
- `ui/src/lib/adapter/index.ts` (and `wasm.ts` / `tauri.ts`) — already exposes `injectNoteOn(midi, velocity)` / `injectNoteOff(midi)` per the issue. Use existing API.
- `wasm/src/lib.rs:511` — `harmonize_note_on` binding (already exposed via adapter).

### Architecture verdict
In-repo, UI layer. No Rust changes needed. Walker is a JS `setInterval` calling adapter methods. The actual harmony is computed by the real engine — there is no shortcut to take.

### Implementation outline
1. New `AutoPlay.svelte` with a single button toggling state. On start:
   - Pick scale (default: current `engine.scaleMode` + current `engine.key`).
   - Generate a sequence: `degrees = [0, 1, 2, 3, 4, 3, 2, 1, 0]` repeated, mapped via `ScaleMode.intervals()` (mirrored client-side or fetched once). Octave anchored to MIDI 60.
   - `setInterval(500ms)`: call `adapter.injectNoteOn(midi, 100)`, then 400ms later `injectNoteOff(midi)`. Move to next note.
2. On stop / unmount: clear interval, send remaining note-offs.
3. Show in DEMO mode. Don't gate on platform — works on both WASM and Tauri because adapter is unified.
4. Optional: drive from `BeatClock` events for Tauri if present, else fixed timer. Defer this — fixed timer is fine per the issue.

### Test strategy
TDD-first:
- Component test (Vitest): mock adapter, click start, advance fake timers, assert correct sequence of `injectNoteOn` calls with expected MIDI values.
- Component test: click stop mid-sequence, assert all open notes get `injectNoteOff`.
- Integration: in a manual UAT, switch harmony mode mid-autoplay and verify harmony output changes.

No Rust tests needed — the engine is exercised through its existing test suite.

### Dependencies
None.

### Entropy impact
- One new Svelte component, no platform-specific code.
- Risk: interval lifecycle leak on hot reload. Fix in component cleanup. Reference the `wasm.ts stopTickLoop` fragile-area note in CONCERNS.md:109 — same class of bug to avoid.

### Open questions / blockers
None blocking. Scale choice, pattern variety, tempo can be PR-time decisions.

### Estimated effort
**XS** (≤1 day). Issue body estimates 2 hours; matches.

---

## #65 — Presets UI redesign (re-introduce with new style)

**Verdict:** In-repo, UI layer. Backend untouched.

### Problem
The old `PresetManager.svelte` was unmounted in PR #64. Backend (`crates/contrapunk-preset/`, Tauri preset commands at `src-tauri/src/commands/presets.rs`, WASM bindings at `wasm/src/lib.rs:609+`) is intact. Need a new, more discoverable UI.

### Touchpoints
- `ui/src/lib/components/PresetManager.svelte` (legacy, 349 lines, full implementation — reference for the four adapter calls).
- `crates/contrapunk-preset/src/lib.rs:39-104` — `PresetManager` API (no changes needed).
- `src-tauri/src/commands/presets.rs` — `list_presets`, `load_preset`, `save_preset`, `delete_preset` (no changes needed).
- `wasm/src/lib.rs:609-700` — same four operations exposed.

### Architecture verdict
In-repo, UI only. Backend is stable and complete. Single-file rebuild of the component, possibly with a small extraction of a "current preset name" indicator usable in the StatusBar.

### Implementation outline
1. Design pass first (per the issue body — "open design space"). Pick one of:
   - **a)** Top-of-ControlPanel pill row (recommended — visible without click).
   - **b)** StatusBar dropdown.
   - **c)** Cmd-K command palette.
2. Component skeleton: `<PresetBar />` with active preset name + quick-load buttons + "..." opens a panel (modal or popover) listing all presets.
3. Reuse the four adapter methods. The legacy file at `ui/src/lib/components/PresetManager.svelte` shows the full call pattern — clone the logic, replace the layout.
4. Wire `engine.syncFromBackend()` after `loadPreset` (legacy file, line 41) so UI reflects the loaded preset's `key`/`mode`/`scale_mode`.
5. Delete the legacy `PresetManager.svelte` once the new component ships (per AC).
6. Backend gap to watch: `save_preset` in `src-tauri/src/commands/presets.rs:75-89` hardcodes `persona: "Custom"` and `genre: "Custom"` — fine for now, but if the new UI surfaces persona/genre, the command needs new args.

### Test strategy
TDD-first (component tests via Vitest, no Rust tests needed):
- Renders empty when adapter returns `[]`.
- Calls `adapter.loadPreset` with the right name on click; then `engine.syncFromBackend()`.
- Save flow: enter name → calls `adapter.savePreset(name)` → refetches list → highlights the new active preset.
- Delete flow: two-click confirmation (legacy pattern) → calls `adapter.deletePreset`.
- LocalStorage persistence survives reload (UAT, not unit).

### Dependencies
None.

### Entropy impact
Single-component change. Low risk. Already proven backend.

### Open questions / blockers
Design call: where does the bar live? Need a sketch session (`/gsd-sketch`) before building.

### Estimated effort
**S** (1-3 days). Design pass + rebuild + tests.

---

## #100 — Smart bass register suppression

**Verdict:** Mixed. Approach B (velocity + register heuristic) is in-repo core, ≤50 lines. Approach A (DAW IAC feedback) is blocked on #98 — external integration, deferred.

### Problem
Sub-bass notes shouldn't be harmonized; melodic notes in the low register should. Static MIDI cutoff doesn't work because the same note can be either.

### Touchpoints
- `crates/contrapunk-harmony/src/engine.rs:1248` — `harmonize_note_on` is the gate. Add early-return suppression before the auto-key feed and synthetic-beat increment.
- Engine struct: add `bass_register_threshold: u8` (default 40, E2) and `bass_velocity_threshold: u8` (default 90).
- `harmonize_note_on` signature today takes only `Note` — but the router has the velocity. Either:
  - **a)** Plumb `velocity: u8` through `harmonize_note_on` (breaking API change, affects 5+ call sites including WASM at `wasm/src/lib.rs:511`).
  - **b)** Add `last_velocity: u8` field set by a `set_input_velocity` setter called by the router right before each note-on (back-compat, but stateful — risk of mismatched ordering).
  - **Recommend (a)**: cleaner; velocity is intrinsic to a note-on. Compile-time enforcement that all call sites pass it.
- Tauri/WASM bindings: update signatures.
- UI: two new sliders in ControlPanel ("Bass floor" 0-127, "Bass vel" 0-127). Toggle for "off" (threshold=0).

### Architecture verdict
In-repo core. Velocity is harmony-engine concern. The Approach A external-IAC path is deferred until #98 lands; the engine's `suppress_note(midi)` API can be added later when needed.

### Implementation outline
1. Change `harmonize_note_on(&mut self, note: Note, velocity: u8)`. Update call sites in router, Tauri, WASM, examples, tests.
2. At top of `harmonize_note_on`: `if u8::from(note) <= self.bass_register_threshold && velocity >= self.bass_velocity_threshold && self.bass_register_threshold > 0 { return vec![note]; }`. Note: returns just the input — passes through dry, no harmony.
3. **Critical:** Do NOT update `active_notes` for suppressed notes. Then `harmonize_note_off` returns `vec![note]` for free (no tracked harmony).
4. Setters: `set_bass_register_threshold`, `set_bass_velocity_threshold`. Persist via existing preset machinery — add the two u8 fields to `StylePreset` (with `#[serde(default = "default_…")]` for back-compat).
5. UI: ControlPanel sliders + adapter methods + Tauri/WASM bindings.

### Test strategy
TDD-first, in engine.rs mod tests:
- "high velocity below threshold → no harmony": `harmonize_note_on(Note::C2, 110)` returns `vec![Note::C2]`.
- "low velocity below threshold → harmony fires": `harmonize_note_on(Note::C2, 60)` returns multi-note (mode=DiatonicThirds).
- "above register threshold → harmony regardless of velocity": `harmonize_note_on(Note::C5, 110)` harmonizes.
- "threshold=0 disables": with `set_bass_register_threshold(0)`, low+loud notes get harmonized.
- "suppressed note: NoteOff returns just the input" — round-trip the `active_notes` invariant.

Same style as `test_random_mode_tracking` (engine.rs:1479) — exercise the note-on/note-off pair and inspect output.

### Dependencies
None.

### Entropy impact
- API change to `harmonize_note_on` is the biggest risk. Compile-time enforced — every call site fails to build until updated. CI will catch.
- Two new `StylePreset` fields — additive, default-back-compat.

### Open questions / blockers
Issue mentions Approach C (phrase context). Skip in v1 — Approach B handles the headline use case at near-zero cost. Revisit if A/B prove insufficient.

### Estimated effort
**S** (1-3 days). The API plumb is most of it.

---

## #8 — Rhythm-aware triggers (beat-aware harmony responses)

**Verdict:** In-repo core, but **moderate scope and worth narrowing**. Issue body conflates three independent features: (a) detect beat/tempo from input, (b) trigger pattern-recognition responses ("1-3-5 eighths → arpeggio one octave higher"), (c) generate voices that play on the upbeat. Recommend splitting.

### Problem
Make generated harmony respond to *when* the user plays, not just *what*.

### Touchpoints
- `crates/contrapunk-harmony/src/engine.rs:278` — `synthetic_beat_counter` (from fb3e7b9). Already gives the engine "internal beat awareness without a transport." This is the critical lever for #8.
- `crates/contrapunk-harmony/src/engine.rs:1125-1138` — `effective_counterpoint_beat_phase()`. Existing fallback pattern from external transport to synthetic counter.
- `src-tauri/src/audio_clock.rs:115-198` — real `Transport` + `BeatCrossing` already emits `beat-update` events. Use as ground-truth beat for Tauri.
- `src-tauri/src/commands/transport.rs:43` — metronome toggle. Existing tempo state.

### Architecture verdict
In-repo core. The infrastructure mostly already exists (synthetic beat counter + Tauri Transport). #8 is layering *features* on top of that infrastructure, not building new transport.

**Recommendation: split #8 into three sub-issues** and pick which to ship first. Each is independently scoped:
- **#8a — Tempo inference from inter-note intervals.** Pure DSP. Add `TempoEstimator` to harmony crate that tracks inter-onset intervals (IOI) and emits a BPM estimate. Output drives the synthetic clock for non-Tauri surfaces. Effort: S.
- **#8b — Upbeat/offbeat voice placement.** Cheap; uses existing `counterpoint_beat_phase`. Voice 2 fires on `+0.5` (upbeat) rather than `+0.0` (downbeat). Effort: XS — already 80% wired via Species 2-4 work.
- **#8c — Pattern detection ("1-3-5 eighths → arpeggio").** Hard. Needs a phrase buffer + pattern matcher + a response generator. Effort: L. Defer.

### Implementation outline (for #8a + #8b, the realistic first shippable)

**#8a (tempo inference):**
1. New `crates/contrapunk-harmony/src/tempo.rs`. `TempoEstimator { ioi_buffer: VecDeque<f64>, … }`. On each note-on with timestamp (engine clock or external), append the delta-time to the buffer (max 8 entries).
2. Estimate BPM as `60 / median(ioi_buffer)` once buffer has ≥4 entries, with outlier rejection (drop top/bottom).
3. Engine: `tempo_bpm() -> Option<f64>`. Synthetic beat counter increments at `ioi_inferred` rate rather than per-note when tempo is known.
4. Surface to UI as `engine.detectedBpm` reactive.

**#8b (upbeat placement):**
1. Add `voice_offsets: Vec<f32>` to engine — per-voice phase offset in beats, default all 0.
2. In `harmonize_single_directed` (engine.rs:1139), pass `phase + voice_offsets[voice_idx]` instead of bare `phase`.
3. UI: per-voice offset slider (-0.5 to +0.5) for advanced mode.

### Test strategy
TDD-first:
- **#8a:** "feed 8 notes 500ms apart, tempo_bpm returns ~120". "feed 4 outlier-laden notes, median rejection keeps tempo stable". "no input → tempo_bpm returns None".
- **#8b:** "voice 1 with offset 0.5 produces different output from voice 1 with offset 0.0 over a melody". Same shape as `test_species2_differs_from_species1_without_transport` (engine.rs:2509).
- Integration: synthetic counter respects inferred tempo when both #8a and #8b are on.

### Dependencies
None new.

### Entropy impact
- New module `tempo.rs` is small and additive.
- `voice_offsets` is one new field. No API break.
- Risk: 4 surfaces inherit free, but each needs UI work. Defer UI for #8a (just expose detected BPM); UI for #8b is one slider.

### Open questions / blockers
- Is #8c (pattern recognition) worth chasing? Smells like ML or expert-system territory. Recommend a *separate* spike, not in this group.
- Quantize-to-grid vs phase-offset for upbeat: two implementations of similar idea. Phase-offset is simpler.

### Estimated effort
**M** (3-7 days) for #8a + #8b combined. **#8c is L+ and should not be in this milestone.**

---

## Summary table

| Issue | Verdict | Effort | Status |
|---|---|---|---|
| #4 | in-repo core | XS | Shipped; tune+close |
| #81 | in-repo core | M | Greenfield rewrite of detector |
| #3 | in-repo core | M | New HarmonyMode + router tick |
| #42 | in-repo (UI) | XS | UI-only |
| #65 | in-repo (UI) | S | UI-only; backend ready |
| #100 | in-repo core | S | API plumb + early-return |
| #8 | in-repo core (split) | M (8a+8b) / L (8c) | Recommend splitting issue |

All sit inside the existing harmony crate or the UI. No new dependencies. No new release boundaries. The expensive piece across all seven is the API change in #100 (`harmonize_note_on` gains `velocity`) — touched everywhere because velocity is universal. The most interesting risk is #3's router-tick path, which adds a new "engine emits without user input" code path that has to be real-time safe.
