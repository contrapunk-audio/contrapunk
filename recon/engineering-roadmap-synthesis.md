# Contrapunk Engineering Roadmap Synthesis

Date: 2026-06-29

Sources read: `README.md`, `recon/github-issues.md`, `recon/planning-roadmap.md`, `.planning/STATE.md`, current `git log`. I also read `.planning/HANDOFF.json` because `STATE.md` and the git log conflict on the freshest Elixir status.

## Current state

- **Public product:** Contrapunk is a real-time counterpoint harmony generator and OSS plugin host. The shared Rust core feeds desktop/Tauri, browser/WASM, native CLI, and plugin surfaces.
- **Publicly described capabilities:** MIDI/guitar input, harmony engine with scales/modes/voice-leading styles, built-in synth/delay/reverb, CLAP plugin hosting, browser app, and macOS desktop release.
- **Planning source of truth is split:** top-level `.planning/STATE.md` still says Phase 06.4, and `ROADMAP.md` says Phase 21 is queued, but later `STATE.md`, `.planning/HANDOFF.json`, and git history show **Phase 21 Elixir is active and progressed through A6, A-Cut, B3, B4, and B5.2**.
- **Current Elixir state:** implementation is paused for manual QA, not more feature work. First resume task is testing `elixir-standalone` and the feature-gated Tauri Elixir synth path.
- **Latest committed work:** `45d309d1 Add Golem standalone drummer prototype` added `crates/golem-core/`, `apps/golem/`, and Golem design/research docs. This is newer than the planning summaries, so treat Golem as a committed prototype that still needs planning/QA.
- **Issue backlog:** 63 pure issues in the recon snapshot: 41 open, 22 closed. GitHub's issue API also showed 8 open PRs mixed into the issue-like count.

## Shipped / committed features

### Core Contrapunk

- MIDI foundation, harmony engine, GUI/distribution, server mode, octave variations, WASM/browser support, humanization, voice leading, extended scale modes, Barry Harris scales, CI cleanup, docs, and UI modernization are marked complete in the planning synthesis.
- Modal harmony/chord detection is code-complete but still wants UX/human verification.
- v1.1 website/demo readiness appears closed: piano/fretboard interactions, Web MIDI permission UX, demo mode, typography, WASM transport/metronome, and output routing.
- v1.3-era shipped work in `STATE.md`: HoldMode end-to-end, tempo-synced delay, Tauri per-lane piano colors, BPM re-anchor for metronome, plugin `noteUpdate`, DAW-managed UI capability hiding, presets UI redesign, Krumhansl auto-key, and embed wave work.

### Elixir

- `elixir-core`: A0-A6 landed through polyphony, modulation, filters, FX bus, spectral/phase/unison controls, filter models, and standalone UI exposure.
- A-Cut: Elixir can be selected in the Tauri built-in synth chain behind the `elixir-synth` feature flag, with existing `SynthEvent` and `SynthParams` bridged.
- `elixir-plugin`: nih-plug CLAP/VST3 skeleton, MIDI input, audio output, core automation params, and FX automation params are committed.
- `elixir-preset`: native preset schema plus conservative `.vital` / `.vitalbank` import is committed.
- `elixir-standalone`: Vital import UI from `~/Downloads` is committed.

### Golem

- A standalone adaptive drummer prototype is committed: `golem-core` plus a Svelte/Tauri app.
- Product direction: audio-native drummer, not MIDI-first; rule-based adaptive groove brain first; procedural drum engine now; sampler/kit format later.

## Planned / deferred features

### Immediate Elixir remainder

- Manual QA for `elixir-standalone`: audio/MIDI, A6 controls, FX controls, Vital scan/apply.
- Manual QA for Tauri with `--features elixir-synth`: confirm built-in synth output is Elixir and existing UI controls still affect sound.
- B5.3: plugin preset state serialization/loading using `elixir-preset`.
- B7: wavetable editor and real Vital `.vitaltable` / spectral parity. Current Vital import is subset-only by design.
- B8: headless renderer.
- B9: public `elixir-v0.1.0` release/signing; bundle IDs still need reserving.
- A7/A-default flip: only after parity/QA; then remove legacy synth path if safe.
- C0-C4: multi-plugin hosting in Contrapunk for CLAP/VST3/AU.

### Contrapunk roadmap backlog

- Phase 6.5 Note Generator + WASM parity remains deferred.
- Logo/Tauri app icons and DMG distribution are incomplete or TBD.
- Performance Mode, Mic Input, Vocoder, openDAW device integration, Cloud, and full plugin distribution remain planned.
- Guitar Input is historically near-complete on a branch, but issue recon says the active path is now a fixture-driven rewrite/rebuild rather than more patching.
- Integration test pipeline still needs real guitar fixtures.
- Release engineering is around 80%, not done.

### Issue-driven roadmap

- FTUX sound path: issue `#116` is the clearest current user pain. Fresh installs should make sound without hidden routing knowledge.
- Companion/performance lanes: `#117` / `#121` point toward Drum/Drone/Looper/Arpeggiator/Sample/Sidechain lanes, but should be sliced down before implementation.
- DAW sync/capture: `#98`, `#99`, `#119`, `#120` are the foundation for Link/IAC/CoreAudio/sidechain workflows.
- Audio intelligence/ML: `#102`, `#104`, `#118`, `#121` are ambitious and dependency-risky; defer heavyweight ML until product loops and audio-thread boundaries are stable.
- Theory depth: `#115` and `#112` push toward historical rule sets and live analysis overlays.
- Docs/marketing proof: `#5` and `#13` remain open for audio samples/walkthroughs.

## Top engineering priorities

### Now

1. **Elixir manual QA before more Elixir surface.** The handoff explicitly says standalone and integrated synth are unverified after fast implementation.
2. **Fix first-run “why is there no sound?” UX (`#116`).** Default all useful voices to an audible internal path and show output target inline. Smallest high-impact product fix.
3. **Triage stale issues and stale planning state.** Close/re-scope obvious stale issues, reconcile Phase 21 status, and stop treating old roadmap counters as truth.
4. **Keep Golem as prototype until validated.** Do not expand to ML/sampler/Contrapunk integration until the existing standalone prototype passes basic timing/audio/follow tests.

### Next

1. Finish Elixir B5.3 plugin preset state, then B7 wavetable parity.
2. Add the smallest Golem v0.1 plan/checklist: stable internal clock, no output-callback allocation/blocking, guitar RMS/onset follow, procedural drums audible, Svelte/Tauri controls functional.
3. Ship the minimal Companion lane slice only after FTUX is fixed: probably one or two lanes, not the full `#121` mega-roadmap.
4. Build the guitar rewrite quality gate: fixture corpus, native/WASM parity, A/B stage tests, latency reporting.
5. Build DAW sync/capture primitives before smart sidechain/listen features.

### Later

1. Full audio intelligence / ML-assisted ListenLane, smart slicing, source separation, DDSP/Demucs/Basic Pitch experiments.
2. Full Golem sampler/kit format, velocity layers, round robin, choke groups, and later Contrapunk `GolemBlock` integration.
3. Performance Mode, Mic Input, Vocoder, openDAW, Cloud, and broader AU/VST3/AAX/plugin-hosting ambitions.
4. Historical theory expansions after the current product/QA debt is under control.

## Validation checks

### Baseline per change

- Rust workspace: `cargo check --workspace --message-format=short`
- Harmony changes: `cargo test -p contrapunk-harmony --lib`
- UI changes: `npm --prefix ui run check`
- WASM/browser changes: `cd ui && npm run build:wasm` or the relevant WASM build path

### Elixir-specific

- `cargo test -p elixir-core --lib`
- `cargo check -p elixir-core --target wasm32-unknown-unknown`
- `cargo check -p elixir-standalone`
- `cargo check -p elixir-plugin`
- `cargo check -p contrapunk --features elixir-synth`
- `cargo check -p contrapunk-tauri --features elixir-synth`
- `cargo test -p contrapunk --features elixir-synth chain::elixir_block --lib`
- Manual: run `elixir-standalone` and Tauri with `--features elixir-synth`; verify actual audio, UI controls, FX, MIDI, and Vital import.

### Golem-specific

- `cargo check -p golem-core`
- `cargo check -p golem-tauri`
- `npm --prefix apps/golem run check`
- Manual: start/stop, BPM stability, procedural drum output, guitar input meters, energy/density/fill response, and silence-free callback behavior.

### Plugin / release

- Contrapunk plugin: `cargo build -p contrapunk_plugin --release`
- Elixir plugin: add `pluginval` before calling it releasable.
- Release/macOS: signing/notarization/DMG checks remain necessary before public desktop claims.

## Stale / open issue triage notes

Close or re-scope first; no code needed until maintainer confirms:

- `#4` auto-key: issue recon says implemented but awaiting user testing.
- `#14` runtime/DMG-ish report: latest local build reportedly worked.
- `#30` Windows: green CI artifacts exist; needs tester handoff, not more engineering by default.
- `#42` autoplay: dropped from v1.1 as non-launch-critical; keep only if tied to FTUX sound path.
- `#79` pitch-bend wobble/debug path: superseded by `#82` guitar rewrite.
- `#106` May 14 jam target: deadline passed; close or rewrite as a durable requirement.

Keep open but decompose:

- `#116` should be promoted to near-term P0/P1 because it maps directly to a real user failing to hear sound.
- `#117` / `#121` should be split into small lane/runtime slices.
- `#82` should become the guitar-input umbrella with fixtures and parity gates.
- `#98` / `#99` / `#119` / `#120` should define the DAW sync/capture foundation before sidechain/intelligence work.
- `#102` / `#104` / `#118` need dependency/license/WASM risk review before implementation.
- `#115` / `#112` are valid theory roadmap items, but should stay test-first.

Hygiene:

- 25/63 issues are unlabeled in the recon snapshot; label/milestone cleanup will make the roadmap less guessy.
- Distinguish old “plugin Companion lanes” work from newer performance Companion lanes. Git history and `STATE.md` disagree enough that this should be verified before opening new plugin-lane tasks.
