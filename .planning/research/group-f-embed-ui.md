# Research: Group F — Embed / UI / Visualization

**Issue(s):** #66, #70, #101
**Date:** 2026-05-11
**Researcher:** issue-researcher
**Verdict (group):** mixed — #66 already substantively shipped (close after wave-2 wrap-up); #70 obsoleted by submodule pattern (recommend close); #101 in-repo UI feature **but blocked on AGPL license incompatibility**.

---

## Headline finding (read first)

The premise of #70 is **out of date**. Issue #70 was written assuming the website carries vendored copies of `ui/src/lib/embed/*`, `keyboard-input.ts`, and `wasm-pkg/*` with manual sync. That hasn't been true since website PR #6 ("feat(vendor): consume canonical embed via contrapunk submodule") merged. The website now pulls this repo as a git submodule at `vendor/contrapunk/` and resolves a Vite alias `@cp/*` → `vendor/contrapunk/ui/src/lib/*` (see `website/astro.config.mjs:11-12, 32`). Components import `@cp/embed/Fretboard.svelte` directly. There is exactly **one canonical copy**; updates flow by bumping the submodule SHA (`bump contrapunk submodule to latest main` is a real commit). The duplication problem #70 proposed to solve no longer exists.

That collapses Group F to: finish the last bit of #66 (Piano wrapper + ChordReadout), close #70, and decide what to do about Hydra given AGPL.

---

## Issue #66 — Canonical embed components in `ui/src/lib/embed/`

### Problem

Move state-binding logic out of `Fretboard.svelte` / `Piano.svelte` / `ChordReadout.svelte` so the same Svelte file works in the Tauri app (Svelte 5 runes + `engine.svelte.ts`) and the website (nanostores). Mechanism: props-and-callbacks pattern; each consumer wraps with its own state binding.

### Touchpoints

- `ui/src/lib/embed/Fretboard.svelte` — **exists**, props-driven, zero store imports (verified at file:line 12-19 — only imports from `./music-utils`; file:line 45-64 — accepts `inputNotes`, `harmonyNotes`, `borrowedNotes`, `onNoteOn`, `onNoteOff` via `$props()`).
- `ui/src/lib/embed/Piano.svelte` — **exists**, props-driven (file:line 22-44).
- `ui/src/lib/embed/music-utils.ts` — **exists** with `NOTE_NAMES`, `midiToName`, `COLOR_INPUT/HARMONY/BORROWED`, `STANDARD_TUNING`, `PIANO_START/END`, `isBlackKey`, `formatMusicalString` (file complete, ~108 lines).
- `ui/src/lib/embed/audio.ts` — **exists** (browser-path Web Audio synth, ~state-library-agnostic).
- `ui/src/lib/embed/ChordReadout.svelte` — **does not exist**. Chord-info block currently lives in `ui/src/lib/components/StatusBar.svelte:176` (`.chord-info` style block) and `ui/src/lib/components/Piano.svelte:204-211` (the `<div class="chord-display">` above the keys). Wave 4 not started.
- `ui/src/lib/components/Fretboard.svelte` — **already a thin wrapper** (22 lines, imports `EmbedFretboard from $lib/embed/Fretboard.svelte`, feeds `engine.svelte` + `adapter` + `ui`).
- `ui/src/lib/components/Piano.svelte` — **still the thick implementation** (~470 lines, imports `engine`, `adapter`, `ui`, `getPianoKeyColor`; reimplements `isBlackKey`, `midiToName`, etc. inline). Not yet a wrapper. **Wave-3 work pending.**
- Website wrappers in `website/src/components/embed/` (`ContrapunkFretboard.svelte`, `ContrapunkPiano.svelte`, `ContrapunkChordReadout.svelte`, `ContrapunkEngine.svelte`) import `@cp/embed/...` and feed nanostores. The flow described in #66 is real and live.

### Architecture verdict

**In-repo (UI). Status: ~70% done, finish.** No architectural change needed — this is straight refactor work to swap `components/Piano.svelte` over to a thin wrapper around `embed/Piano.svelte`, and extract `ChordReadout`. Both lift code already written and reduce duplication.

### Implementation outline (remaining work only)

1. **Wave 3 — Piano wrapper swap.** Reduce `ui/src/lib/components/Piano.svelte` to ~25 lines: import `EmbedPiano from $lib/embed/Piano.svelte`, feed engine state, register `onNoteOn`/`onNoteOff`. The existing thick file's chord display, scale overlay, flash overlays, and HLD ghost overlays must already exist in `embed/Piano.svelte` (verify; backport any that don't). Compare the two files side by side — anything in `components/Piano.svelte` that's not in `embed/Piano.svelte` is either (a) state-binding logic that belongs in the wrapper, or (b) presentation logic that needs to migrate into the embed.
2. **Wave 4 — ChordReadout extract.** Create `ui/src/lib/embed/ChordReadout.svelte` with props `{ chordName: string | null; rootName?: string }`. Lift the chord-display markup from `components/Piano.svelte:204-211` and the `.chord-info` block from `components/StatusBar.svelte:176`. Both `components/Piano.svelte`'s chord row and `StatusBar`'s chord-info block then reduce to a `<ChordReadout chordName={engine.chordName} />` line.
3. **Website-side mirror.** None needed if the wrapper-files already point at `@cp/embed/ChordReadout.svelte` once it exists. Bumping the contrapunk submodule SHA on the website will pick it up. If `ContrapunkChordReadout.svelte` exists in the website and currently reimplements logic, swap it to a wrapper-of-`@cp/embed/ChordReadout` analogous to `ContrapunkPiano.svelte`.
4. **Sync mechanism.** Already in place (submodule). Document it once in `ui/src/lib/embed/README.md` (currently no such README) — point at `website/astro.config.mjs` for proof. Not strictly required for the issue.

### Test strategy

- Visual smoke: load Tauri app, ensure Piano + Fretboard + StatusBar look identical pre/post.
- Functional: click-to-play works in both surfaces (Tauri + website embed iframe demo on the marketing site).
- Reduced-motion: HLD ghost trail respects `prefers-reduced-motion` (already in `embed/Piano.svelte` via `@media`).
- Playwright: `ui/scripts/inject-notes-via-cdp.mjs` exists for headless verification; add a `verify-piano-wrapper.mjs` that mounts the new wrapper, fires a note, asserts the embed renders.
- No new unit tests — these are pure presentation files.

### Dependencies

None. All new code is Svelte component restructuring.

### Entropy impact

**Net negative** (good). Removes ~440 duplicated lines from `components/Piano.svelte`. Doesn't add a new release surface or a new feature flag. The four distribution surfaces (CLI/Tauri/WASM/plugin) are unaffected because the embed files are UI-only and Tauri is the only one that currently mounts them.

### Open questions

- The acceptance criteria list `embed/ChordReadout.svelte` (Wave 4) as required. The current StatusBar `.chord-info` and Piano `.chord-display` differ slightly (StatusBar shows root + quality + tonal degree; Piano shows chord name only via `formatMusicalString`). Decision: ChordReadout takes a single optional `format: 'compact' | 'detailed'` prop with two render modes, or accepts a fully pre-formatted string and leaves formatting to the wrapper. Recommend the latter (simpler API; matches the props-and-callbacks pattern's spirit).
- Engine bootstrap component sharing (`embed/Engine.svelte` per the #66 issue Wave 5) is explicitly marked "may stay forked indefinitely". Recommendation: **leave forked.** WASM init + Tauri-IPC bootstrap is genuinely different per consumer; abstracting it would invent a state-library bridge for no gain.

### Estimated effort

**S (1-2 days)** — Wave 3 + Wave 4 only. Mechanical work; no design decisions remain.

---

## Issue #70 — Extract `@contrapunk/embed` npm package

### Problem (stated)

Website carries vendored copies of `embed/*`, `keyboard-input.ts`, `wasm-pkg/*` with no automated sync. Drift goes unnoticed for weeks. Proposed fix: publish `@contrapunk/embed` to npm; both repos install it.

### Touchpoints

- `website/.gitmodules` registers `vendor/contrapunk` → `github.com/contrapunk-audio/contrapunk`.
- `website/astro.config.mjs:11-12,32` defines `cpRoot = vendor/contrapunk/ui/src/lib` and a Vite alias `@cp` → `cpRoot`.
- `website/scripts/setup-vendor.mjs` writes a stub `vendor/contrapunk/ui/.svelte-kit/tsconfig.json` so esbuild's tsconfig walk-up resolves through the submodule.
- `website/src/components/embed/Contrapunk{Fretboard,Piano,Engine,ChordReadout,...}.svelte` all import from `@cp/embed/...` or `@cp/keyboard-input`. No vendored copies of these files exist in `website/src/lib/contrapunk/embed/` — that directory does not even exist (verified).
- Most recent submodule bump: `417f990 chore(vendor): bump contrapunk submodule to latest main` (2026-05). Submodule status: `v1.1.0-8-g08e1ec3f`.

### Architecture verdict

**Close as obsolete.** The submodule + Vite-alias pattern already gives the website a single source of truth for the canonical embed files, with version-pinning via SHA, and a one-line bump to pick up updates. It dodges every cost of npm publishing (org claim, semver dance, AGPL-of-deps audit, WASM packaging vs SvelteKit/Astro asset resolution) while being **strictly easier to debug** because the consumer sees real source files, not a black-boxed `node_modules/@contrapunk/embed/dist/`.

If publishing ever becomes valuable, the trigger conditions are: (a) a third consumer appears (someone else's site / a Notion-style embed-as-iframe service), or (b) the submodule SHA bumps become a real source of friction (they currently aren't — three bumps in two months, all painless). Until then, the implicit cost of "we have to npm publish for every embed change, plus Renovate-PR through to the website, plus an auto-publish workflow" is real overhead that the submodule pattern avoids.

The one residual concern is **drift visibility**: the submodule SHA on the website can lag main indefinitely. Mitigation: add a renovate-style GitHub Action that opens a PR on the website when contrapunk main moves. This is the "Option C" lite version of the #70 issue's own alternatives table, and it's the *only* piece of #70 worth doing.

### Implementation outline

1. Close #70 with a comment explaining the submodule pattern landed and supersedes the proposal. Cite `website/astro.config.mjs`.
2. **Optional follow-up (separate small issue):** add a scheduled GHA on the website that runs `git submodule update --remote vendor/contrapunk` and opens a PR if the SHA changed. ~30 lines of YAML. Catches drift at the *consumer*, no plumbing needed in this repo.
3. If a future third consumer appears, reopen and reconsider. The migration from submodule to npm package is mechanical — `ui/src/lib/embed/` becomes `packages/embed/src/`, add a `package.json`, publish.

### Test strategy

N/A — this is a "close issue" recommendation, not an implementation.

### Dependencies

N/A.

### Entropy impact

Publishing an npm package would **add** entropy: a new `packages/` directory, a new `pnpm-workspace.yaml` (currently no workspace), a new release surface with its own semver, a new auto-publish CI workflow, a Renovate config on the website, and a non-trivial WASM-bundling question (the issue itself flagged this as "finicky"). Closing the issue keeps the four existing surfaces unchanged.

### Open questions

- The single residual concern (drift visibility) is real. Even with the submodule pattern, the website can sit on a stale SHA. The "Option C" auto-PR is the right answer if/when this becomes painful. Today: no evidence it's painful.

### Estimated effort

**Close: XS (1 hour comment + decision).** Optional drift-watch GHA on the website: **XS (2 hours).** Not in this repo.

---

## Issue #101 — Hydra visualiser (WebGL audio FFT)

### Problem

Embed a `hydra-synth` canvas in the Contrapunk UI, driven by real-time bass/mid/treble RMS computed from the audio engine output. Per-genre presets. Togglable; no CPU cost when hidden. Spec includes Rust-side FFT via `spectrum-analyzer` crate (~90µs/4096-sample, no_std) and Tauri event emission to feed `window.__viz.{bass,mid,treble}` for Hydra source lambdas.

### Touchpoints

- `src-tauri/src/audio_clock.rs:264-296` — `device.build_output_stream` callback with `chain.process(data, channels)`. This is where FFT would compute (on the audio thread). Lock-free ringbuffer + non-audio-thread emit recommended; do NOT call `app_handle.emit` inline from the audio callback (allocates, blocks).
- `src-tauri/src/audio_clock.rs:143-200` — existing forwarding-thread pattern (`BeatCrossings` → `handle.emit`). Exact pattern to reuse for `viz` payload.
- `src-tauri/src/commands/engine.rs:519,587,612` — existing `app_handle.emit("note-update", ...)` calls; pattern for adding `set_viz_enabled` / `set_viz_preset` IPC commands.
- `ui/src/lib/components/` — new `VizPanel.svelte` would live here. No existing visualization component.
- `ui/package.json:18-32` — Svelte 5.55 + Vite 6 + `vite-plugin-wasm` already in place. **Vite has a known `global is not defined` incompatibility with hydra-synth** — README workaround: `define: { global: {} }` in `vite.config.ts` (see Hydra README "Known issues / troubleshooting → Vite" section).
- `ui/src/lib/stores/ui.svelte.ts` — feature flag would live here (`vizEnabled: false`, `vizPreset: 'electronic' | 'classical' | ...`).

### Architecture verdict

**BLOCKED by license incompatibility, then in-repo UI feature behind a setting (lazy-imported).**

The license blocker is the deciding fact. Verified via `npm registry`: `hydra-synth@1.4.0` is **AGPL-3.0-licensed**. Contrapunk is **MIT-licensed** (verified `LICENSE:1`). Bundling AGPL code into the Tauri binary triggers AGPL's copyleft obligations on the combined work: any user accessing the app must be able to obtain corresponding source under AGPL. For a desktop app distributed via download that's manageable (publish source, accept the license — but it forces *Contrapunk's frontend code that links to Hydra to be AGPL-compatible*, which conflicts with MIT redistribution of the rest). For a hosted web build (`app.contrapunk.com`) AGPL's "remote network use" clause kicks in even harder.

The user-facing fix is one of:

1. **Reject Hydra; use Butterchurn instead.** `butterchurn@2.6.7` is **MIT-licensed**, ~728 KB unpacked, MilkDrop-style visuals driven by audio. Less livecoding-friendly (no JS DSL, fixed preset format) but MIT and lighter. Loses the "user writes their own Hydra code" appeal.
2. **Reject Hydra; build a small custom GLSL visualizer.** Use `regl` or raw WebGL2 + a fragment shader; bind bass/mid/treble as uniforms. ~50-100 KB. MIT-compatible. Loses the preset ecosystem and the livecoding angle entirely.
3. **Accept AGPL for the visualizer module only via process isolation.** Run Hydra in an `<iframe sandbox>` pointed at a separate page (or a separate Tauri webview window) that loads only the AGPL JS. The host app communicates via `postMessage`. This is a defensible separation under AGPL "mere aggregation" / Section 5, but the legal interpretation is not bulletproof — talking to a lawyer is mandatory before shipping. Iframe also blocks Hydra's direct DOM canvas mounting; `new Hydra({ canvas })` would need to point at the iframe's own canvas, not the host's.
4. **Drop the visualizer.** It's a nice-to-have, not on any milestone. The visual identity already exists via the HLD afterimage on Piano + Fretboard.

**Recommendation: option 1 (Butterchurn) or option 2 (custom GLSL).** Option 3 is too much legal complexity for a side feature; option 4 is fine if no one cares.

Assuming option 1 or 2, the actual code architecture is straightforward and matches the issue spec:

- FFT runs in the Tauri audio thread; results land in a `triple_buffer::Output`-style lock-free slot (we already use that pattern in `audio_clock.rs`).
- A non-audio thread (or a Tauri command poller at ~30 Hz) reads the slot and `handle.emit("viz", payload)`.
- Svelte `VizPanel.svelte` lazy-imports the visualizer library (`const { default: Vis } = await import('butterchurn')`) on mount; tears down on unmount. Hidden = unmounted = zero CPU.
- Browser path (`app.contrapunk.com`) computes the FFT in JS using `AnalyserNode.getByteFrequencyData` from the embed audio chain — no Rust FFT needed.

### Implementation outline

1. **Resolve license decision** (option 1 vs 2 vs 4). 1-day call. Recommend a 30-minute spike comparing a butterchurn preset to a custom-GLSL prototype before committing.
2. **Rust-side FFT.** Add `spectrum-analyzer = "1.x"` to root `Cargo.toml` (`no_std`-compatible, AGPL-free — verify on docs.rs). Compute `bass = RMS(20-200 Hz)`, `mid = RMS(200-2000 Hz)`, `treble = RMS(2000-20000 Hz)` from a 4096-sample window of the chain output, every N audio buffers (every ~30 Hz). Write into a `triple_buffer` slot.
3. **Tauri bridge.** Spawn a forwarding thread (pattern: `audio_clock.rs:143-200`) that reads the slot and emits `"viz"` event every ~33 ms. Add `set_viz_enabled(bool)` command that stops/starts emission.
4. **Svelte panel.** `VizPanel.svelte` mounts a `<canvas>`, lazy-imports the chosen library, listens to `viz` Tauri events (or `AnalyserNode` in the browser build), writes to a local store, and either calls preset functions (Butterchurn) or sets shader uniforms (custom GLSL).
5. **Settings.** Add `ui.vizEnabled` (default false) and `ui.vizPreset` to `ui.svelte.ts`. SettingsModal gains a "Visualizer" section.
6. **Browser path.** Skip the Rust FFT; use Web Audio's `AnalyserNode` (we already create an AudioContext in `embed/audio.ts`). Wire the embed-side analyser into the panel through a callback prop, keeping the embed library-agnostic.

### Test strategy

- **Headless render test.** `ui/scripts/probe-fretboard-mutation.mjs` exists as a CDP harness; clone for VizPanel. Inject fake bass/mid/treble values, snapshot the canvas pixel data, assert it changes on each frame.
- **Audio-thread budget test.** A Rust criterion bench (`benches/viz_fft.rs`) asserts FFT + RMS-band compute stays under 200 µs at 48 kHz / 4096-sample window. Audio callback budget is ~1 ms at 1024-sample buffer; FFT must not eat more than 20% of that. Issue body claims 90 µs — verify, don't trust.
- **Toggle off → zero CPU.** Manual: open Activity Monitor / `top`; toggle viz off; assert UI-process CPU drops back to baseline.
- **License audit.** `cargo deny check licenses` + `pnpm licenses` in CI; reject AGPL / GPL deps.

### Dependencies

| Dep | Version | License | Size | Status |
|---|---|---|---|---|
| `hydra-synth` | 1.4.0 | **AGPL-3.0** | 1.79 MB unpacked | **REJECT** (license) |
| `meyda` | 5.6.3 | MIT | 556 KB unpacked | OK (transitively pulled by Hydra; standalone is fine) |
| `butterchurn` | 2.6.7 | MIT | 728 KB unpacked | OK; v2.x last published recently |
| `regl` | latest | MIT | ~100 KB | OK |
| `three` | 0.184 | MIT | 37 MB unpacked | **Too heavy** for what's needed |
| `spectrum-analyzer` (Rust) | 1.x | MIT/Apache | ~80 KB compiled | OK |

Recommended pick: `butterchurn` (presets) **or** `regl` + a custom fragment shader (livecoding-feel, lighter), plus `spectrum-analyzer` on the Rust side and `meyda` on the browser side.

### Entropy impact

- New `ui/` npm dep (lazy-imported; size only paid when feature enabled).
- New Rust dep `spectrum-analyzer` — pulls into the main binary + plugin + WASM (gate behind `feature = "viz-fft"` if WASM size matters; current WASM bundle is ~2 MB so an extra 80 KB is acceptable).
- New audio-thread compute path. **High risk if implemented carelessly** — any allocation, lock, or syscall in the audio callback corrupts the audio output. Mandatory: use a lock-free triple-buffer; FFT input window must be a `[f32; 4096]` stack array; no heap allocation in the audio path.
- New `app_handle.emit` channel (`"viz"`) at ~30 Hz. Tauri's event channel is fine at that rate.
- New UI surface (VizPanel) and new settings keys. Mid-cost — minor refactor risk in `ui.svelte.ts` and `SettingsModal.svelte`.

### Open questions / blockers

1. **License decision (HARD BLOCKER).** Until this resolves, no implementation work. See "Architecture verdict" options 1-4.
2. **Where does the visualizer mount?** New tab, overlay, dedicated route at `/visualizer`? Affects the SettingsModal UX.
3. **Does the browser build need parity?** Issue body says yes. If yes, the Web Audio FFT path is mandatory (option 1 or 2 both support this; Hydra's audio analysis uses Meyda which works in-browser).
4. **Preset format.** Butterchurn presets are JSON-ish; custom GLSL is a `.glsl` string. Either way, presets live in `ui/src/lib/embed/viz-presets/` or a new top-level dir. Not a blocker.
5. **Hydra's `eval`-driven model is incompatible with the contrapunk-app CSP.** Even setting AGPL aside, Hydra's web editor and most preset code use `eval()` / `new Function()` (the README doesn't expose an `evalCode()` — callers do it). Tauri v2 default CSP forbids `unsafe-eval`. Loosening CSP to enable Hydra widens the XSS attack surface against Tauri IPC commands (see `CONCERNS.md` — "No Tauri Command Allowlist Audit Documented"). Butterchurn + custom GLSL avoid this entirely.

### Estimated effort

**M (3-7 days)** once license decision and library choice are made. Breakdown: ½ day Rust FFT + bench, 1 day Tauri bridge + emission throttling, 1 day Svelte panel + lazy import + settings, 1 day presets (3-4 presets), 1 day cross-surface testing + Playwright. The legal decision could blow this up to L if option 3 (process isolation) is pursued.

---

## Cross-issue summary table

| Issue | Verdict | Effort | Blocker |
|---|---|---|---|
| #66 — canonical embed | finish (waves 3-4) | S | none |
| #70 — npm package | **close as obsolete** | XS | n/a |
| #101 — Hydra | reframe as Butterchurn / custom GLSL; in-repo behind flag | M | **AGPL license incompatibility for Hydra specifically** |

## Recommended action order

1. Comment-and-close #70 with a pointer to `website/astro.config.mjs` and a note that the submodule pattern obsoletes the proposal.
2. Finish #66 wave 3 + wave 4 in a single 1-2 day phase. Pure refactor; no design questions.
3. Reframe #101 in a comment: "Hydra is AGPL; pick Butterchurn (MIT) or roll a regl/GLSL visualizer." Tag for next backlog review; not on the critical path.
