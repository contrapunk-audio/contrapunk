# Research: VST/AU plugin version for DAW integration

**Issue(s):** #9
**Date:** 2026-05-11
**Researcher:** issue-researcher
**Verdict:** **in-repo plugin (workspace member) — Architecture B (native UI via VIZIA), pivoting away from webview.** Spike required before commit.

---

## Problem

HN-surfaced request: users want Contrapunk inside their DAW (Logic, Ableton, Bitwig, REAPER) instead of routing IAC + BlackHole between the standalone Tauri app and the DAW. A skeleton `plugin/` workspace member already exists with full nih-plug parameter wiring and a webview-editor scaffold. The decision now is what the plugin should *become*, not whether to start one.

## Touchpoints

- `plugin/Cargo.toml:16-18` — depends on **forked** `nih-plug`, `nih_plug_webview` (contrapunk-audio org), and the workspace root crate `contrapunk` (path = "..").
- `plugin/src/lib.rs:1-523` — full plugin: 8 parameters, MIDI mode + Audio (guitar pitch-detect) mode, MPE channel fan-out for harmonized voices, VST3 + CLAP exports.
- `plugin/src/editor.rs:1-284` — `ContrapunkEditorHandler`, JSON message bridge, custom-protocol asset serving (gated behind `embed-ui` feature flag, not on by default).
- `plugin/src/editor_fallback.html` — minimal placeholder UI.
- `au-wrapper/CMakeLists.txt` + `au-wrapper/build.sh` — clap-wrapper based AUv2 build, universal binary, target macOS 10.13+. Produces `Contrapunk.component`.
- `ui/src/lib/adapter/plugin.ts:1-221` — JS-side `PluginAdapter` mapping the Svelte UI to `window.plugin.send/listen`.
- `ui/src/lib/adapter/index.ts` — platform detection: plugin > tauri > browser.
- `.planning/phases/plugin-webview-gui/.continue-here.md` — paused at 4/7 (Apr 7 2026), blockers documented.
- Git history: last functional plugin commit was `ecd12c6` (Apr 7 2026). Since then only drive-by edits for new harmony modes (`550137b`).

## Current state

**Working:** parameter exposure, MIDI-in→MIDI-out fan-out on MPE channels, audio-in (guitar pitch detection) → MIDI, VST3 + CLAP bundling via `cargo xtask bundle`, AUv2 build via clap-wrapper. Generic parameter UI loads correctly in Logic Pro.

**Broken / unverified:**
1. Webview GUI **never confirmed loading** in any DAW (last user check: Logic showed generic sliders).
2. `embed-ui` feature flag references `OUT_DIR`/`ui_assets.rs` build-script output that doesn't exist — Svelte→plugin embedding pipeline (Task 6 of the paused phase) was never built.
3. macOS 26 (Tahoe beta) `auval` cannot list third-party AUs, so we can't validate end-to-end automation.
4. Three external forks (`nih-plug`, `nih-plug-webview`, `wry`) require ongoing maintenance.

**Unmodified since pause:** plugin/ is ~6 weeks stale; no usable user-facing release.

## External-library state (verified 2026-05-11)

- **nih-plug upstream is dead.** `robbert-vdh/nih-plug` issue #265 (Mar 29 2026): "no longer maintained." Canonical successor is **`BillyDM/nih-plug`** — framework-only, no example plugins. Source: [issue #265](https://github.com/robbert-vdh/nih-plug/issues/265).
- We currently track our own `contrapunk-audio/nih-plug` fork, which forks the dead upstream. We need to **rebase onto BillyDM/nih-plug** or accept losing security fixes / CLAP-host updates.
- **nih_plug_webview** (httnn/toiglak ecosystem) is self-described "work in progress, not production-ready." Tested only on macOS. Windows "needs verification." Known crash on Escape key in Ableton Live. AU forwarding through clap-wrapper is **not part of its test matrix** — the JS-host bridge talks to CLAP's GUI extension; whether clap-wrapper's `wrappedview.mm` propagates that through `AUCocoaView` correctly is exactly the blocker the paused phase couldn't resolve.
- **VIZIA / iced / egui** are all supported first-class by nih-plug, mature, in-tree examples exist (Diopser, Crisp, STFT all ship with VIZIA UIs). Binary footprint is much smaller than a bundled webview + WASM.

## Three architectures evaluated

### A. nih-plug + nih_plug_webview (current paused direction)
- **Pros:** UI parity with desktop/browser; one Svelte codebase, three targets.
- **Cons:** Per-plugin bundle is *very* large (Svelte build + wasm-pack output + webview runtime); on macOS the webview uses WKWebView and shares process risks; AU support is the open question (clap-wrapper GUI forwarding unverified); cross-platform: Windows path unverified, Linux unsupported; depends on **4 forked repos** we maintain.
- **Failure mode:** If the webview can't be forwarded to AU, Logic users get only the generic UI — which is the user's *primary* target DAW.

### B. nih-plug + native VIZIA UI (recommended)
- **Pros:** Battle-tested across the nih-plug ecosystem; supports CLAP/VST3/AU GUI hosting natively without webview wrappers; ~1-2 MB UI overhead vs ~20-40 MB for webview; faster startup; no JS runtime in audio process; cross-platform parity (mac/win/linux); same UX *quality* on desktop and plugin even if not pixel-identical.
- **Cons:** UI is a second implementation. Harmony-engine parameter surface is small (~10 params), so the duplication is *small in scope* — but lays-out, knob aesthetics, the chord-name display, etc. must be re-built. Drops the "one UI for all surfaces" dream — desktop Svelte + plugin VIZIA become two interfaces.
- **Failure mode:** Visual drift over time as desktop UI evolves; mitigated by treating plugin UI as a *focused subset* (key, mode, voices, voice-position, octave, auto-key, voice-leading toggle). No need to replicate Performance view, MIDI device pickers, etc. — none of those make sense in-plugin anyway (per the `PluginAdapter.ts` stubs).

### C. Plugin-as-parameters-only (MIDI-FX with no GUI)
- **Pros:** Smallest possible binary; current generic UI is already produced by nih-plug for free; zero GUI maintenance.
- **Cons:** Bad UX for the chord-name display, key visualization, suggestions panel; users coming from the standalone app will feel downgraded; no path to expose advanced features (Performance knobs, mode-specific controls).
- **Failure mode:** Loses to free competing plugins on UX, even if engine is better.

## Cross-platform plugin formats

Confirmed: **nih-plug exports VST3 and CLAP natively** (`plugin/src/lib.rs:521-522` already does this). **AU is wrapped via clap-wrapper** in `au-wrapper/` (macOS-only). All three formats reachable from a single `cdylib`.

- **VST3** — Steinberg, ubiquitous on Win/mac/linux, license is GPLv3-compatible.
- **CLAP** — open standard, modern, supported by Bitwig, REAPER, Studio One, increasingly by FL Studio. Recommended primary target.
- **AU** — Apple-only, mandatory for Logic / GarageBand. Wrapped from CLAP via clap-wrapper.

## Code-sharing strategy

The core engine already lives in `crates/contrapunk-harmony` + workspace `contrapunk` and is reused by `plugin/` via `contrapunk = { path = ".." }`. What *cannot* port from `src-tauri/`:

- `audio_clock.rs` — DAW provides the clock via transport.
- `guitar_bridge.rs` — overlaps with `plugin/src/lib.rs` audio mode but uses cpal; plugin uses DAW audio buffers.
- `commands/midi.rs`, `commands/plugins.rs`, `commands/routing.rs` — DAW manages MIDI I/O and FX chains; plugin shouldn't.
- `companion/` — desktop-only orchestration.

What **should** lift across: chord-name derivation, key-display logic, suggestion scoring snapshots (`SuggestionSnap`). These are pure-Rust and already in the core lib — the plugin needs a thin presentation layer over them, not a re-port.

## Architecture verdict (entropy framing)

**In-repo plugin (workspace member, kept where it is), Architecture B.**

Entropy math:
- Path A (webview) has *highest* entropy: 4 maintained forks, an unfinished build pipeline, a JS↔Rust bridge that duplicates the Tauri IPC adapter, and an unverified AU path. Sunk cost is real but the bill keeps growing.
- Path B (VIZIA) has *medium* entropy: one second UI implementation, but contained in `plugin/src/editor/` with no cross-surface coupling. No forks of GUI deps (VIZIA is upstream-supported in nih-plug). Drops 3 of 4 forks (`nih_plug_webview`, `wry`, and reduces `clap-wrapper` to AU-wrapping only).
- Path C (no UI) has *lowest* entropy but loses the product.

**Strategic compounding:** Path B aligns with industry norms (every major nih-plug plugin uses native UI), so external help / examples / Stack-Overflow signal are higher. Path A makes us the lone webview-plugin shipper, paying for the privilege.

## Recommended next step: 1-day spike

**Question:** Can a minimal VIZIA UI render Contrapunk's 8 parameters with chord-name readout in Logic Pro via the clap-wrapper AU path, and what's the resulting `.component` bundle size?

**Shape:**
1. Branch off `main`. Add `nih_plug_vizia` dep, write a single-screen VIZIA editor (key dropdown, mode dropdown, voice count, voice position, octave mode, auto-key toggle, voice-leading toggle, input-mode toggle).
2. Wire `params_changed` notification from engine to UI for chord-name update (existing `params_json()` logic in `editor.rs` is the model).
3. `cargo xtask bundle contrapunk_plugin --release` → `au-wrapper/build.sh` → load in Logic.
4. Measure: bundle size, UI render correctness, parameter automation round-trip, startup time.

**Exit criteria:** if it works and bundle is under ~10 MB, kill the webview branch and commit to Path B. If VIZIA cannot render through clap-wrapper AU bridge either, we have a deeper clap-wrapper problem (independent of UI choice) and *that* becomes the blocker.

## Implementation outline (post-spike, if Path B confirmed)

1. Rebase `contrapunk-audio/nih-plug` fork onto `BillyDM/nih-plug` (upstream changed).
2. Drop `nih_plug_webview`, `wry`, and `embed-ui` feature flag from `plugin/Cargo.toml`. Remove `plugin/src/editor.rs` and `ui/src/lib/adapter/plugin.ts` + the `plugin` branch in `ui/src/lib/adapter/index.ts`. Archive `plugin-webview-gui` phase.
3. Add `nih_plug_vizia` dep; rewrite `plugin/src/editor.rs` as a VIZIA editor with the 8 params + chord-name display + voice activity LEDs.
4. Build `release.sh` for `.vst3` / `.clap` / `.component` bundling + macOS codesign + notarize + Windows codesign. Use `melatonin.dev` CI pattern as reference.
5. Wire CI to produce signed installers (.pkg on mac, .exe NSIS on win) per release tag.
6. Update `STACK.md`, `CONCERNS.md`, README plugin section.

## Test strategy

- **Unit:** engine already covered. Plugin-specific: parameter→engine sync (`sync_params` test fixture), MPE channel routing (golden MIDI trace per harmony mode).
- **Integration:** snapshot a 30-second MIDI input file → run through plugin in offline render mode → diff against golden MIDI output. nih-plug has an offline-host crate for this.
- **Manual UAT (per host):** Logic (AU), REAPER (VST3 + CLAP), Bitwig (CLAP), Ableton (VST3), Cubase (VST3). Matrix of: GUI loads, params automate, MIDI fans out on correct channels, project save/load round-trips state.
- **TDD-first:** chord-name derivation + param-sync helpers before UI work.

## Dependencies

- Keep: `contrapunk-audio/nih-plug` fork (rebased onto BillyDM), `wmidi`, `serde_json`.
- Add: `nih_plug_vizia` (in-tree with nih-plug, MIT/ISC), `vizia` (transitive, MIT).
- Drop: `nih_plug_webview`, `wry`, `contrapunk-audio/clap-wrapper` GUI-forwarding code paths (still need clap-wrapper for AU bundle generation, but stop chasing its `wrappedview.mm`).
- Bundle size estimate: VST3 single-arch ~3-5 MB, universal mac ~8-10 MB (vs. webview path ~25-40 MB with WebKit).
- Maintenance: BillyDM/nih-plug is the active fork (sourced from issue #265, Mar 2026).

## Distribution / signing

- **macOS:** Developer ID Application cert; codesign with `--options=runtime`; wrap `.vst3` / `.component` in a `.pkg` (productbuild) + `.dmg`; submit pkg/dmg to `xcrun notarytool`; staple. Pattern documented at melatonin.dev. **Cost:** $99/yr Apple Developer account.
- **Windows:** EV or OV code-signing cert (DigiCert / SSL.com); sign `.dll` (VST3) and `.exe` installer (NSIS or WiX); cost $200-500/yr. SmartScreen reputation builds over time.
- **Linux:** no signing — distribute `.tar.gz` of `.so` (VST3) and `.clap`.
- Plugin distribution is **disjoint from the Tauri `.dmg` path** today; reuse the GitHub Actions workflow shape but add a separate `release-plugin.yml`.

## Entropy impact

- New surfaces: zero net. `plugin/` already exists in the workspace; we are simplifying it.
- Build-time cost: VIZIA build is fast (no webpack/vite); CI gains a plugin lane (~5 min on mac runner).
- Release boundary: plugin will eventually want **independent versioning** from desktop (DAW users don't reinstall every 2 weeks) — but for v1 ship in lockstep with Tauri release.
- Drops 3 fork repos from maintenance burden.
- Risk of regression in unrelated areas: low — the plugin is structurally isolated.

## Open questions / blockers

1. **clap-wrapper AU GUI forwarding still unverified.** Spike will surface this.
2. **macOS 26 auval behavior** for third-party AUs — wait-and-see; doesn't block development since Logic itself loads AUs fine.
3. **Versioning policy** — bound to or independent of Tauri releases? Defer until after v1 plugin ship.
4. **Windows signing budget** — out of scope for this research; product decision.

## Estimated effort

- Spike (Path B confirmation): **XS (1 day).**
- Full Path B implementation post-spike: **M (5-7 days)** — VIZIA UI rewrite, CI for signed bundles, manual host matrix.
- If spike fails and we must keep webview path: **L (1.5-2 weeks)** to finish the paused phase + verify all four DAWs + cover fork maintenance.
