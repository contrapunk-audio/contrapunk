# Research: Group D — DAW Integration & Sync

**Issue(s):** #10 (openDAW device), #11 (SonoBus), #98 (Ableton Link), #99 (DAW side-by-side: IAC + BlackHole), #31 (td-rs / TouchDesigner)
**Date:** 2026-05-11
**Researcher:** issue-researcher
**Verdict:** mixed — see per-issue breakdown.

These are all *bridge* issues. The entropy question is consistent across all five: **at what API boundary do we attach?** Contrapunk already exposes three stable boundaries: MIDI (in + out via the Tauri router at `src-tauri/src/commands/engine.rs::run_tauri_router`, web-midi in `crates/contrapunk-midi/src/web.rs`), the WASM `Engine` API at `wasm/src/lib.rs`, and audio I/O via cpal. Every integration below should reuse one of those boundaries rather than reach into the harmony engine directly.

Per-issue verdict summary:

| # | Title | Verdict | Effort |
|---|-------|---------|--------|
| 98 | Ableton Link | **In-repo, Tauri-only, feature-flagged** | S |
| 99 | DAW side-by-side (IAC + BlackHole) | **Mixed: docs + small in-repo InputRouter+sidechain FX** | M |
| 10 | openDAW device | **External sub-project** (fork of openDAW or standalone SDK app); Contrapunk WASM stays untouched | M |
| 11 | SonoBus | **External sub-project** (run SonoBus as a separate app, connect via IAC + BlackHole); no in-repo code | XS for docs, L if we ever embed |
| 31 | TouchDesigner via td-rs | **External sub-project** (separate workspace member producing a TD plugin binary) | L |

---

## Issue #98 — Ableton Link tempo sync

### Problem
When Contrapunk runs alongside Logic Pro / Ableton, internal BeatClock drifts from the DAW. Looper, arpeggiator, and counterpoint Species 2-4 desynchronize across bar boundaries. Users want one shared beat across all running apps.

### Touchpoints
- `crates/contrapunk-transport/src/clock.rs:65-162` — `Transport` (sample-counter + atomic BPM + run state). `set_bpm()`, `play()`, `stop()`, `beat_position()` already exist and are the natural surface to virtualize behind a Link session.
- `src-tauri/src/commands/transport.rs` — Tauri commands `set_bpm`, `play`, `stop`, `get_transport_state` (need `set_link_enabled`, `get_link_state`).
- `src-tauri/src/commands/engine.rs:347-368` — router thread already calls `transport.is_running()` and `transport.beat_position()` per iteration; no change needed in the router if `Transport` internally consults Link.
- `Cargo.toml` (workspace + `crates/contrapunk-transport/Cargo.toml`) — add optional dep behind feature flag.
- `ui/` — Link toggle + peer count chip in transport panel.

### Architecture verdict
**In-repo, Tauri-only, behind feature flag `ableton-link` (default off in CI, default on in Tauri release builds).** Link is a small, harmony-adjacent timing primitive — exactly the shape that earns its keep in core. But it is *desktop-only* (no WASM path, see sub-agent findings) and pulls a GPLv2+ C dependency, so we **must** keep it behind a feature flag and out of the WASM/plugin builds. The crate to use is `rusty_link` (current best Rust wrapper over Ableton's official `abl_link` C11 shim).

### Implementation outline
1. Add `rusty_link = { version = "<latest>", optional = true }` to `crates/contrapunk-transport/Cargo.toml`; expose feature `ableton-link`. Re-export from workspace root Cargo.toml conditionally on `not(target_arch = "wasm32")` so wasm builds can never accidentally pick it up.
2. New module `crates/contrapunk-transport/src/link.rs` defining a thin `LinkSession` struct wrapping `rusty_link::AblLink` + `SessionState`. Methods: `new(initial_bpm)`, `enable()`, `disable()`, `beat_at_time(host_time_us)`, `bpm()`, `peer_count()`, `quantum()`.
3. Extend `Transport`:
   - Add `link: ArcSwapOption<LinkSession>` (or `Mutex<Option<LinkSession>>`) on the struct.
   - When Link is enabled, `beat_position()` reads from `link.beat_at_time(audio_clock_now)` instead of `sample_pos / sample_rate * (bpm/60)`. Audio clock uses host time captured in the cpal callback (see `src/audio_out/` once that lands — until then, `Instant::now()` + drift correction).
   - `bpm()` returns Link's tempo when enabled (read-only); `set_bpm()` becomes a *proposal* to Link in that mode (`session_state.set_tempo(bpm, host_time)`).
4. Tauri commands `set_link_enabled(bool)`, `get_link_state() -> { enabled, bpm, peer_count }`. UI surfaces a toggle + peer count.
5. Looper / arpeggiator already read `transport.beat_position()`; they get Link sync free.

### Test strategy
- Unit: mock `LinkSession` trait, verify `Transport::beat_position()` switches sources on enable/disable; verify BPM proposal flow.
- Integration: spawn two Contrapunk processes, enable Link in both, verify `beat_position()` converges within 1 beat (use a test harness that polls both transports).
- Manual UAT (acceptance criteria from issue): Enable Link → start Logic with Link → beat lock within 1 beat; tempo change in Logic reflected in Contrapunk display; disable → no audio glitch.

### Dependencies
- `rusty_link` (GPLv2+, wraps Ableton's official `abl_link` C11 shim, last activity Nov 2025, hobby maintainer — acceptable risk because the wrapper is thin and the underlying C SDK is stable).
- Alternative: `ableton-link-rs` (pure-Rust Tokio reimplementation, GPLv2+, early-stage). Listed as fallback only — protocol-compatibility risk vs. the official SDK is higher.
- **License consideration**: GPLv2+ on a feature-flagged optional dependency to an MIT codebase is *fine* for our distribution (we ship combined work under GPL when the flag is on, MIT-only otherwise). Document this in `LICENSE.md`.
- Binary size delta: ~200–400 KB compiled (Link C++ SDK + wrapper).

### Entropy impact
- One new feature flag, one optional dep, one new module (~150–200 LoC). Transport gains an optional indirection but the public API stays identical.
- WASM/plugin builds untouched — `cfg(not(target_arch = "wasm32"))` gating.
- CI: add one job with `--features ableton-link` to catch breakage. Otherwise nothing changes.
- Negligible regression risk; the Link-disabled path is the existing code.

### Open questions
- Audio-clock source when running headless (no cpal stream open). Today Transport advances on the cpal callback; Link wants a *monotonic host clock* anyway, so we may need `Instant::now()` regardless when no audio device is active.
- Whether to expose Link in the nih-plug build. Most DAW hosts already have native Link — exposing Contrapunk-the-plugin's own Link session would conflict. Recommend: **plugin build has Link disabled at the feature-flag level**; the host owns clock.

### Estimated effort
**S (1–3 days)** for the core in-repo work + UI toggle + tests.

---

## Issue #99 — DAW side-by-side: bidirectional IAC MIDI + BlackHole sidechain

### Problem
Contrapunk only sends MIDI *out* today. Users running Logic Pro / Ableton beside it want bidirectional flow: kick triggers from the DAW pump Contrapunk's pads, DAW bass notes suppress Contrapunk's bass-register harmony, DAW transport drives Contrapunk's looper.

### Touchpoints
- `crates/contrapunk-midi/src/input.rs:31` — already connects a MIDI input via `midir`. Currently used for the user's keyboard. We can add **a second input channel** for "DAW return" without touching the existing path.
- `src-tauri/src/commands/engine.rs:100-233` — `start_routing` takes a single `input_idx`. Needs a sibling `set_daw_return_input(port)` that opens an additional input mapped to a new `InputRouter` enum (NoteIn channel-routed → SidechainTrigger / SuppressBassNote / ClockTick).
- `docs/IAC_PLUGIN_SETUP.md` already documents the *outbound* side. Need a companion doc `docs/DAW_SIDE_BY_SIDE.md` for the *return* leg + BlackHole.
- `src/fx/` — currently `delay.rs`, `reverb.rs`, `mod.rs`. **No `sidechain.rs` or `envelope_follower.rs` exist yet** (issue body assumed they would). New files needed.
- `src/` has **no `audio_out/` directory yet** — Audio Foundation phase is paused per `.planning/codebase/CONCERNS.md`. Audio-rate sidechain via BlackHole therefore *blocks on the audio-out subsystem landing*. Until then, **MIDI-only sidechain trigger** is implementable.
- `crates/contrapunk-harmony/src/engine.rs` — needs `suppress_note(midi: u8)` to drop harmony generation for an externally-played pitch.

### Architecture verdict
**Mixed: ~70% docs + small config, ~30% in-repo code.** Splits naturally into three sub-deliverables:

1. **Docs (XS, do first)** — `docs/DAW_SIDE_BY_SIDE.md` covering: macOS IAC setup for a *second* "Contrapunk In" bus, BlackHole 2ch install, Logic Pro routing for kick→IAC ch10 and bass→IAC ch2, kick audio→BlackHole bus. Windows: loopMIDI + VB-CABLE equivalents. This unblocks the user manually today without any code changes.
2. **InputRouter + MIDI-only sidechain trigger (S, in-repo, Tauri-only)** — new `crates/contrapunk-midi/src/input_router.rs` (channel-dispatched event enum), plus Tauri commands. Sidechain at this stage is *trigger-only* (gate pad voices on kick channel 10 NoteOn). No audio-rate envelope follower yet.
3. **Audio-rate sidechain via BlackHole (M, blocks on Audio Foundation)** — `src/fx/envelope_follower.rs` + `src/fx/sidechain.rs`. **Defer until `src/audio_out/` exists and PolySynth output runs.** Document the dependency clearly.

Keeping this in-repo (not external) because: it reuses the existing router architecture, doesn't pull heavy deps, is desktop-only by nature of IAC/BlackHole being OS-specific virtual buses (no WASM concern).

### Implementation outline
1. **Phase 1 — Docs**. Write `docs/DAW_SIDE_BY_SIDE.md` mirroring the style of `docs/IAC_PLUGIN_SETUP.md`. Include screenshots / step list for: IAC "Contrapunk In" bus creation, Logic External Instrument routing, BlackHole audio bus setup. Windows fallback section.
2. **Phase 2 — InputRouter**. New module `crates/contrapunk-midi/src/input_router.rs` exposing `InputEvent::{SidechainTrigger{velocity}, ExternalBassNote{pitch}, ExternalClockTick}`. Subscribes to an extra `midir::MidiInputConnection`. Per-channel dispatch tied to user-configurable channel map (default: ch10=kick, ch2=bass, ch1=clock). New Tauri commands `set_daw_return_input`, `set_input_router_channel_map`.
3. **Phase 3 — Harmony suppression**. Add `HarmonyEngine::suppress_note(midi: u8)` to skip harmony generation for that pitch on the next NoteOn. Wire `ExternalBassNote` → engine.suppress_note. Add `clear_suppressed()` on note-off.
4. **Phase 4 — MIDI-only sidechain pump**. In the router loop, on `SidechainTrigger` push a gain-envelope event into the synth voice pool (attack 5 ms, release 100–300 ms, applied as a VCA modulator on harmony voices). PolySynth gains a `set_sidechain_envelope(gain: f32)` method.
5. **Phase 5 — Audio-rate sidechain (deferred)**. When `src/audio_out/` lands, add `EnvelopeFollower` reading from a BlackHole cpal input device, smoothed RMS → gain reduction signal feeding the same `set_sidechain_envelope` path.
6. UI: sidechain knobs (threshold, attack, release), IAC return-input picker, BlackHole audio-input picker (Phase 5 only).

### Test strategy
- Docs: dry-run the setup on macOS; capture screenshots; check the loop visibly produces sound.
- InputRouter: unit-test channel dispatch and event mapping. Integration test with a `midir` virtual port pumping known sequences.
- Harmony suppression: unit-test that `suppress_note(48)` skips harmony for the next `harmonize_note_on(Note(48))` call but resumes after `clear_suppressed()`.
- Manual UAT (acceptance criteria from issue): kick MIDI triggers pump within 5 ms; bass notes from Logic suppress harmony for those pitches.

### Dependencies
- No new external deps. `midir` already in tree.
- BlackHole installation is **user-side**, not a bundled dep. Document in `DAW_SIDE_BY_SIDE.md`.

### Entropy impact
- One new module in `contrapunk-midi`; two new files in `src/fx/`; engine gains two methods. Modest, but **two new Tauri commands** widen the IPC surface — audit `tauri.conf.json` capabilities accordingly (relevant to `CONCERNS.md` security note about command allowlist).
- Phase 5 (audio-rate) entangles with Audio Foundation — do not start until that lands or risk creating dead code.

### Open questions
- Channel-map UI ergonomics: do we hardcode (ch10=kick, ch2=bass) or fully expose? Hardcoding is simpler; exposing avoids friction for non-standard rigs. **Recommend: hardcode for v1, add settings later.**
- Per-pitch suppression duration: timeout-based (e.g. 200 ms after the external bass note-off) or strictly note-on/off scoped? Lean note-on/off scoped.

### Estimated effort
**M (3–7 days)** for Phases 1-4. Phase 5 is **L (1–3 weeks)** and blocked by Audio Foundation.

---

## Issue #10 — openDAW device integration

### Problem
Run Contrapunk *inside* openDAW (browser-based DAW) as a native MIDI effect device. Both run in the browser; the WASM build of Contrapunk already exists at `wasm/`.

### Touchpoints
- **In Contrapunk**: `wasm/src/lib.rs` already exposes `Engine::new`, `note_on`, `note_off`, `set_key`, `set_mode`, etc. — exactly the API surface openDAW's `MidiEffectProcessor` would call. **No Contrapunk-side code change needed** for the engine itself; the integration is entirely on the openDAW side.
- **In openDAW** (separate repo, AGPL v3 main / LGPL v3 SDK): would add the 5-layer device pattern (schema, adapter, processor, editor, registration) per the prior research at `.planning/research/opendaw-integration.md`.

### Architecture verdict
**External sub-project.** The integration code lives in *openDAW's* repo or a standalone openDAW SDK app, not Contrapunk's. Contrapunk ships `contrapunk.wasm` (it already does); openDAW's `ContrapunkDeviceProcessor` loads it inside an AudioWorklet. The boundary is `Engine`'s WASM API plus the wasm blob URL. Three concrete paths, ordered by realism:

- **Path C (recommended near-term)**: standalone app built on `@opendaw/studio-sdk` (LGPL v3, compatible with our MIT). No upstream PR needed. We control release cadence. This is what the existing research recommends.
- **Path B (medium-term)**: fork openDAW, add the device, submit small PRs after a Calendly with André Michelle.
- **Path A (long-term)**: wait for openDAW's runtime device-loading SDK (post their 1.0, target Q3 2026).

Crucially: **the Contrapunk WASM engine already works in the browser**, and the existing research doc captures the integration pattern exhaustively (138 lines). What this issue actually needs from *this* repo is **zero source changes**; the work is across the boundary.

### Implementation outline
1. **Audit the existing WASM API** for completeness against openDAW's `MidiEffectProcessor` interface. Likely gaps (already flagged): no per-event velocity exposure, no MIDI channel awareness. Patch wasm/src/lib.rs to surface velocity in `note_on` return.
2. **Stand up Path C app** in a new repo `contrapunk-opendaw-bridge` (external). Cargo + npm. Uses `@opendaw/studio-sdk`, vite, wasm-bindgen output from this repo.
3. (Optional, later) **Fork-and-PR Path B** once Calendly with André is booked.
4. Add a CI artifact in *this* repo that publishes `contrapunk.wasm` + glue at a stable URL (already partially done — verify).

### Test strategy
- **In Contrapunk repo**: extend WASM API unit tests for any new exposed methods (velocity, channel). That's the only test surface in *this* repo.
- **In external bridge repo**: openDAW SDK demos already run integration tests; we mirror the NeuralAmpDevice pattern's test setup.

### Dependencies
- No new deps in this repo.
- External repo: `@opendaw/studio-sdk` (LGPL v3), `vite`, `wasm-bindgen-cli`.
- License: LGPL is fine for MIT Contrapunk; AGPL (main openDAW repo) is only a concern for Path B fork work, which we'd handle by keeping the device code in openDAW's repo (AGPL) and Contrapunk's WASM in this repo (MIT). Combined distribution = AGPL.

### Entropy impact
- **Zero entropy increase in this repo** — the integration is across a stable WASM boundary that we already publish.
- Versioning: openDAW bridge needs to pin a specific `contrapunk.wasm` build; we add a `wasm-api-version` semver field to the WASM exports to make compatibility checks explicit.

### Open questions
- Calendly with André Michelle (per existing research). Not blocking technical work, blocks Path B specifically.
- Does the WASM `Engine` need velocity / per-channel state to be a faithful MIDI effect? Probably yes for usability — flag for the bridge work, not for this repo.

### Estimated effort
**M (3–7 days)** for Path C standalone bridge in the external repo. **Zero** for this repo beyond the velocity-exposure WASM tweak (XS).

---

## Issue #11 — SonoBus integration for Contrapunk Cloud

### Problem
Contrapunk Cloud aims for real-time multi-musician jam sessions with shared harmony. SonoBus is a mature P2P low-latency audio collaboration tool. Question: integrate as transport, or run side-by-side?

### Touchpoints
- **Direct touchpoints in this repo: none yet.** Contrapunk Cloud is Phase 15 (per ROADMAP); no networking code exists in `src/` or `crates/`.
- Future: `src/server/` exists as a stub directory but does not contain any P2P / RTP code today.

### Architecture verdict
**External sub-project (as in: SonoBus stays a separate app the user runs alongside Contrapunk).** Specifically:

- SonoBus is **GPLv3** and **JUCE/C++**. There are **no Rust bindings**, no C API for embedding, and the codebase is structured as a JUCE plugin/standalone app, not a library.
- Embedding SonoBus into Contrapunk would mean: writing C++ FFI for JUCE objects, vendoring JUCE itself (~50–100 MB source), and accepting **GPLv3 contamination of a currently-MIT codebase**. That is a one-way door we should not walk through casually.
- The pragmatic alternative is the same pattern as DAW-side-by-side (#99): **users run SonoBus and Contrapunk concurrently**, with audio routed Contrapunk → BlackHole → SonoBus, and MIDI via IAC.
- If Contrapunk Cloud needs *deeper* P2P integration than that, it should build its own thin P2P layer (probably WebRTC for browser compatibility, since Cloud aims at browser) rather than dragging in JUCE/SonoBus. The features Contrapunk Cloud actually needs (shared MIDI + chord state) are much cheaper to ship over WebRTC data channels than over a JUCE plugin's audio pipeline.

So the verdict has two faces: **for the near term (and probably forever), SonoBus is an external app, documented as one of several P2P options for advanced users.** For Contrapunk Cloud proper, the answer is "build a small WebRTC layer", not "embed SonoBus".

### Implementation outline
1. Add a short section to `docs/DAW_SIDE_BY_SIDE.md` (or a new `docs/JAM_WITH_OTHERS.md`) covering: SonoBus install, BlackHole routing to send Contrapunk audio into SonoBus, IAC if MIDI sharing is desired. **This is the entire near-term deliverable.**
2. (Optional, much later) If a user / contributor pushes hard for embedded SonoBus, plan a **spike** to evaluate the JUCE FFI cost and GPLv3 implications. Do not pre-commit to that work.

### Test strategy
- Manual: verify the documented routing actually produces audio on the SonoBus peer's end.
- No code, no automated tests.

### Dependencies
- None. SonoBus install is user-side.
- If we ever did embed: JUCE (ISC + GPLv3), SonoBus (GPLv3), C++ build chain. License-incompatible with MIT distribution.

### Entropy impact
- **Zero entropy in this repo** if we keep it docs-only.
- If we ever embed: massive. Reject default.

### Open questions
- Does Contrapunk Cloud need *audio* sharing at all, or only MIDI/chord state? If MIDI-only, WebRTC data channels are a far better fit than SonoBus and the question becomes moot.
- Should we explicitly endorse JackTrip / Jamulus / SonoBus in docs, or stay neutral? Recommend: neutral, list options, let users pick.

### Estimated effort
**XS (≤1 day)** for the docs-only path. **L+ (>3 weeks)** for an embedded SonoBus integration — strongly disrecommended.

---

## Issue #31 — TouchDesigner via td-rs

### Problem
Ship Contrapunk as a native TouchDesigner Custom Operator (a CHOP), exposing harmony voices / beat phase / chord metadata as streaming signals inside TD's patch graph. Target users: VJs, AV installations.

### Touchpoints
- **Direct touchpoints in this repo: none** beyond optionally adding a new workspace member. The TD plugin would consume Contrapunk's engine via the existing Rust crate API (`contrapunk-harmony`, `contrapunk-transport`), not via WASM or IPC.
- `Cargo.toml` workspace `members = [...]` would gain `crates/contrapunk-td` (or this lives in a separate repo entirely — see verdict).

### Architecture verdict
**External sub-project** (separate repo, separate release cadence), with **the option to start as a workspace member here** if we want shared Cargo.lock / single-PR development for the first few iterations.

Justifications:
- td-rs is **alpha-status** (the author's own README warns of "potential crashes, memory leaks, missing APIs, breaking changes"). Last push 2025-06; only 5 forks, 57 stars; **no LICENSE file in the repo** (legally ambiguous — would need a license clarification before any production use).
- TD plugin output is a **`.dll` / `.plugin` binary**, not something that ships in our existing distribution channels (Tauri app, WASM, nih-plug VST). Whole new release artifact.
- The target audience (VJs + AV installations) is narrow. The issue itself is labeled `help wanted` — explicitly invited as a community contribution.
- td-rs pins TouchDesigner `2023.12000`. Version drift is a real maintenance risk.
- Audio dependency: the issue mentions audio streaming "once sub-project 1 audio foundation lands" — so audio-via-CHOP is gated on the same blocker as #99 Phase 5.

**Concrete shape**: a new repo `contrapunk-td-plugin` that depends on this repo's crates as Git dependencies (or path deps during local dev). Produces `Contrapunk.plugin` (macOS) and `Contrapunk.dll` (Windows) artifacts. Releases out-of-band.

### Implementation outline
1. **Clarify td-rs licensing**. Open issue on tychedelia/td-rs asking for a LICENSE file. Block on that — we cannot ship a derived plugin under ambiguous license.
2. **Hello-world CHOP**: create the external repo, vendor td-rs via Git submodule or Cargo Git dep, write a minimal CHOP that outputs constant zeroes per channel. Verify it loads in TD `2023.12000`.
3. **Wrap `HarmonyEngine`**: instantiate the engine, expose MIDI-in via a TD parameter or input CHOP. Map engine output to channels (voice 1..N pitch, voice 1..N velocity, beat-phase, chord root).
4. **Custom parameters**: TD-side knobs for key, mode, species, density, humanization — bound to engine setters.
5. **DAT operator (stretch)**: chord/scale metadata.
6. **CI artifacts**: GitHub Actions builds macOS `.plugin` and Windows `.dll`. Released per-tag.

### Test strategy
- Unit tests run in the *external* repo against the `contrapunk-harmony` crate API. No tests in this repo.
- Manual UAT: load the plugin in TouchDesigner, verify harmony voices appear on CHOP channels, hook up to a particle system, verify channel timing aligns with TD cook loop.

### Dependencies
- td-rs (alpha, license TBD, last push June 2025) + cxx / autocxx via td-rs build tooling + TouchDesigner C++ Custom Operator SDK (proprietary, distributed by Derivative).
- Build: `cargo-xtask`, MSVC on Windows, Xcode on macOS.

### Entropy impact
- **Zero in this repo** if external. **Two new workspace members + ~20 MB build artifacts in CI** if in-repo.
- Strong recommendation to keep external. td-rs's alpha state + licensing ambiguity + niche audience make a release-coupling boundary expensive.

### Open questions
- td-rs license. Hard blocker.
- Whether TD users want a CHOP (sample stream) or a DAT (chord metadata) primarily. Likely both, but priority informs MVP scope.
- TD version drift. td-rs pins `2023.12000`; TD's at 2024.x now. Will td-rs keep up?

### Estimated effort
**L (1–3 weeks)** for MVP CHOP + parameter mapping in the external repo. **Blocked** on td-rs license clarification.

---

## Cross-cutting findings

1. **Three of five issues (#10, #11, #31) have natural API boundaries already exposed by Contrapunk** (WASM Engine; MIDI IAC + audio BlackHole; Rust crate API). The right architectural move in all three is to *not* pull integration code into this repo. Two of those three (#10, #31) might still land here as optional workspace members if release-cycle convenience wins out — but that's a coordination decision, not an architectural one.

2. **Only #98 unambiguously belongs in this repo**, and even then it must be feature-flagged off WASM. #99 is a hybrid: docs + a small router extension belong here; the audio-rate sidechain piece is blocked on Audio Foundation.

3. **License audit needed** before any of this lands:
   - `rusty_link` is GPLv2+ → feature flag avoids contaminating MIT default build.
   - `td-rs` has **no LICENSE file** → blocker for #31.
   - SonoBus is GPLv3 + JUCE → strong reason to keep external.
   - openDAW main repo is AGPL v3, SDK is LGPL v3 → use the SDK (Path C).

4. **Audio Foundation (paused per CONCERNS.md) is on the critical path** for #99 Phase 5 (audio-rate sidechain) and #31 stretch (audio CHOP). Until `src/audio_out/` exists, both deliverables are MIDI-only.

---

## Sub-agent delegations

Spawned **one** investigation against the Ableton Link Rust ecosystem (combined `WebFetch` + `WebSearch` rather than a separate Agent invocation, given simplicity):

- **Question**: state, license, WASM support of `ableton-link` Rust bindings.
- **Result**:
  - Three candidate crates exist: `ableton-link` (2019, abandoned, v0.1.0), `ableton-link-rs` (anweiss; pure-Rust early-stage Tokio reimplementation, GPLv2+), and **`rusty_link`** (anzbert; thin wrapper over Ableton's official `abl_link` C11 shim, GPLv2+, hobby-maintained, last activity Nov 2025, tested macOS/Windows/Linux).
  - **Recommendation: `rusty_link`** — it's the closest to the official SDK.
  - **No WASM build path for any of them.** Ableton's own SDK has zero WASM/Emscripten support; the C11 shim relies on platform networking. The pure-Rust port could *theoretically* be made WASM-compatible but isn't today.
  - **License: GPLv2+** uniformly (Ableton Link itself is dual GPLv2+/proprietary). Must be feature-flagged in this MIT codebase.
  - **iOS**: Ableton explicitly says use LinkKit (separate SDK), not Link. Future iOS Contrapunk work would need that.

---

## Recommended next phases (in priority order)

1. **#98 Ableton Link** (S, in-repo, feature-flagged). High value, low entropy, clean boundary.
2. **#99 DAW side-by-side Phase 1 + 2** (docs + InputRouter + MIDI-only sidechain, M). High value, leverages existing router.
3. **#10 openDAW velocity-export tweak** (XS in this repo) + stand up external bridge repo (M, external).
4. **#11 SonoBus docs** (XS, docs-only).
5. **#31 td-rs** (L, external) — only after td-rs license clarification.
6. **#99 Phase 5 audio-rate sidechain** — defer until Audio Foundation resumes.
