# Research: Chord mini app — standalone chord detection and display tool

**Issue(s):** #12
**Date:** 2026-05-11
**Researcher:** issue-researcher
**Verdict:** in-repo core (new SvelteKit route in `ui/src/routes/chord/`) — option A from the brief, with the option B subdomain reserved as a zero-cost deployment switch later.

## Problem

A musician wants a focused "play notes → see chord" tool: real-time chord name, roman-numeral analysis in a chosen key, voicing visualization, and chord history. The detection engine already exists; this issue is asking us to *package* the existing capability as a standalone surface, decoupled from Contrapunk's full harmony-generation UI.

## Touchpoints

Detection engine (done, untouched):

- `crates/contrapunk-chord/src/lib.rs:441` — `chord_display_with_analysis(notes, key_tonic) -> String`. Three-tier chain: `detect_chord` → `detect_partial_chord` → `notes_as_intervals` fallback. Supports extended chords, slash, add, 6ths; produces strings like `"Fmaj7 (IVmaj7 in C)"`.
- `wasm/src/lib.rs:582-595` — already calls `chord_display_with_analysis` inside `WasmHarmonyEngine::get_note_state()` from the union of `last_input_notes` ∪ `last_harmony_notes`. Browser parity exists.
- `src-tauri/src/commands/engine.rs:826,917` — same call from the Tauri router.

UI layer (where new code lands):

- `ui/svelte.config.js` — SvelteKit static adapter with prerender entries; new `/chord` route just needs an entry.
- `ui/src/routes/+page.svelte:17,80-87` — pattern for wiring `midi.svelte.ts` store + `keyboard-input.ts` + WASM adapter into a single page.
- `ui/src/lib/stores/midi.svelte.ts`, `ui/src/lib/keyboard-input.ts`, `ui/src/lib/adapter/wasm.ts:557-562` — all reusable as-is; chord page subscribes to the same `engine.chordName` derived state.
- `ui/src/lib/components/Piano.svelte:209,294` and `ui/src/lib/embed/Piano.svelte`, `ui/src/lib/embed/Fretboard.svelte` — canonical state-library-agnostic embed components (issue #66). The chord app composes these.

Concerns:

- `.planning/codebase/CONCERNS.md` — no chord-detector entries; `src-tauri/.planning/codebase/CONCERNS.md:92` notes a `HashSet::union` heap-alloc on the hot path. Not a blocker for a mini app (call rate is ≤30 Hz), but worth noting if we ever embed in a guitar-input page.

## Architecture verdict

**Option A: new SvelteKit route at `/chord` inside `ui/`.** This is a UI-only repackage of WASM code that already exists and is already exported to the browser. Entropy delta: one new route folder, zero new crates, zero new build pipelines, zero new release boundaries. The existing `npm run build:wasm` + `vite build` produces the bundle.

Why not the other options:

- **Option B (chord.contrapunk.com subdomain).** Same code, different Cloudflare Pages target. Zero new entropy; defer until A is shipping and we want a clean URL. The SvelteKit static build is already a self-contained `dist/` so flipping subdomains is a deployment-config change, not a code change.
- **Option C (`@contrapunk/chord-detector` npm package).** Only justified if third parties want to embed. Premature: no inbound demand, the existing wasm-pkg already imports cleanly, and publishing a single-export npm package locks us into semver before the API stabilizes.
- **Option D (standalone binary / desktop app).** Pure entropy — duplicates the Tauri shell for no functional gain. Reject.

Entropy lens: the chord engine is shipping on all four surfaces today. A `/chord` route adds ~3 Svelte files; everything else (WASM, MIDI store, Piano embed, Fretboard embed, keyboard input) is reused. This is the cheapest possible delivery.

## Implementation outline

1. **Add `ui/src/routes/chord/+page.svelte`.** Minimal scaffold: import `midi` store, `keyboard-input`, WASM adapter (`createWasmAdapter`). Subscribe to `engine.chordName`. Render: a single big chord readout (already-existing `stripAnalysis` + `formatMusicalString` helpers in `Piano.svelte:27`).
2. **Compose existing embeds.** Drop `lib/embed/Piano.svelte` (already shows highlighted active notes) and `lib/embed/Fretboard.svelte` underneath the chord readout. No new components needed for v1.
3. **Add key selector.** New small `<KeySelect>` component (12 root buttons or a dropdown) bound to a local `$state` that calls the WASM engine's existing `set_key` / equivalent setter. Roman-numeral analysis follows automatically (the engine already passes `key_tonic` into `chord_display_with_analysis`).
4. **Chord history strip.** Local `$state<string[]>` ring buffer of length ~16. Push on `chordName` change (debounce 150 ms to filter transient detections during chord changes). Render as horizontal scrolling list with timestamp. Pure UI; no engine changes.
5. **Add `/chord` to prerender entries** in `ui/svelte.config.js`.
6. **(Optional, deferred)** Audio input path. Issue #12 lists "or guitar audio" — defer to issue #82's guitar pipeline rewrite. v1 ships MIDI-only and keyboard-only.

## Test strategy

- Unit: chord engine is already exhaustively tested in `crates/contrapunk-chord/src/lib.rs:726+`. Nothing new needed at the engine layer.
- UI: Playwright smoke test in `ui/tests/` — load `/chord`, dispatch synthetic note-on events via `keyboard-input` (`a` key triggers C4), assert `chord-display` text content updates. Pattern already used; one new spec file.
- Manual UAT: play a Cmaj7 on attached MIDI keyboard → expect "Cmaj7 (Imaj7 in C)" when key=C. Verify history strip captures last 16 chords. Verify Fretboard + Piano embeds highlight the same notes.
- TDD-first: write the Playwright spec before the route; assert the readout selector exists and updates.

## Dependencies

None. Reuses `vexflow` (already a dep, useful later for voicing notation but not needed for v1), `posthog-js` (analytics), existing WASM bundle. No new npm packages, no new crates.

## Entropy impact

- New surfaces: **one new SvelteKit route**. No new build target.
- New build-time cost: negligible — SvelteKit adds the prerendered page to `dist/` at zero perceptible cost.
- New release boundary: none. Ships with the rest of `ui/` on every `contrapunk-website` Cloudflare Pages deploy.
- Files affected outside `ui/src/routes/chord/`: `ui/svelte.config.js` (one new prerender entry). Optionally a new `KeySelect.svelte` in `ui/src/lib/components/`.
- Regression risk: ~zero. Reuses already-shipping code paths.

## Open questions / blockers

- **Audio input.** #12 mentions guitar audio. Punt to #82's pipeline rewrite — once that lands, the chord page subscribes to the same note stream and gets guitar input for free.
- **Subdomain or path?** App-route is cheaper; subdomain is brand-friendlier ("chord.contrapunk.com"). Recommend ship as `/chord` first, register subdomain only after the page has measurable inbound traffic. This is a deployment-config decision, not blocking.
- **Voicing visualization scope.** Issue #12 lists "Chord voicing visualization" — Piano + Fretboard embeds together cover this for v1. If we later want SATB-style staff notation, `vexflow` is already a dep.

## Estimated effort

**XS (≤1 day).** Three Svelte files, one prerender entry, one Playwright spec. The chord engine is already shipping; this is plumbing.
