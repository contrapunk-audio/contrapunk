# Contrapunk — Claude Code project guide

Real-time MIDI harmony generation. Single Rust core compiled to **four** distribution surfaces — get the surface right before you start editing.

## Surfaces and what they ship

| Surface | Path | Built by | Used for |
|---|---|---|---|
| Native CLI | `src/main.rs` | `cargo build --release` | Headless dev / TCP client mode |
| Tauri desktop app | `src-tauri/` | `cd src-tauri && cargo tauri build` | macOS / Windows / Linux desktop |
| WASM browser | `wasm/` | `cd ui && npm run build:wasm` (or `wasm-pack build`) | `app.contrapunk.com` web version |
| nih-plug VST3/CLAP | `plugin/` | `cargo build -p contrapunk_plugin --release` | DAW plugin |

**Same core library** (`crates/contrapunk-harmony`, `crates/contrapunk-audio`, etc.) feeds all four. A harmony-engine fix lands once and propagates everywhere — but each surface has its **own adapter / glue layer** that may need wiring too:

- Tauri: `src-tauri/src/commands/*.rs` — IPC commands, `run_tauri_router` in `engine.rs`
- WASM: `wasm/src/lib.rs` — `wasm-bindgen` exports; `ui/src/lib/adapter/wasm.ts` — JS adapter
- Plugin: `plugin/src/lib.rs` — nih-plug parameters
- UI: `ui/src/lib/adapter/` — three adapters (`tauri.ts`, `wasm.ts`, `plugin.ts`) behind one `ContrapunkAdapter` interface

When you fix a bug, **ask yourself which surface the user is on** before assuming the fix is complete. WASM and Tauri have different threading models, different transport plumbing, and different audio output paths.

## Commands you'll actually run

```bash
# Fast feedback (use these)
cargo check --workspace --message-format=short   # ~30s — type-check everything
cargo test -p contrapunk-harmony                 # ~10s — harmony unit tests, 249 of them
cargo test -p contrapunk-harmony --lib           # same, skips doctests
npm --prefix ui run check                        # ~15s — Svelte / TS type-check

# Slower (CI runs these)
cargo test --workspace                           # ~5min full workspace
cargo clippy --all-targets -- -D warnings        # repo's pre-commit hook runs this
cd ui && npm run build                           # full UI prod build (~2min)
cd src-tauri && cargo tauri build                # full desktop bundle (~10min)
```

**Rule of thumb:** after editing `crates/contrapunk-harmony/`, always run `cargo test -p contrapunk-harmony`. After editing `ui/`, always run `npm --prefix ui run check`. Don't batch — surface failures immediately.

## Planning artifacts (`.planning/`)

This repo uses GSD (Get Shit Done) for planning. **Don't touch `.planning/` files unless explicitly asked** — they're managed by the GSD slash commands (`/gsd-plan-phase`, `/gsd-execute-phase`, etc.).

Read these to orient yourself (don't write):

- `.planning/PROJECT.md` — vision and requirements
- `.planning/STATE.md` — current position / recent activity
- `.planning/ROADMAP.md` — phase breakdown
- `.planning/codebase/ARCHITECTURE.md` — data flow, thread model, IPC boundaries
- `.planning/codebase/STACK.md` — dependencies, runtime
- `.planning/codebase/CONCERNS.md` — known tech debt and bugs (read before "fixing" something)
- `.planning/codebase/CONVENTIONS.md` — coding standards

If `.planning/STATE.md` says a feature is paused / deferred, don't restart it without asking.

## Known footguns

1. **Species 2-4 need a beat phase** — fixed in `fb3e7b9` (synthetic counter inside `HarmonyEngine`). External transport still wins. If you touch `harmonize_single*` or `process_with_beat`, run the regression tests `test_species2_differs_from_species1_without_transport` and `test_external_phase_wins_over_synthetic`.

2. **Parameter changes mid-play can drop notes** — every harmony setter in `src-tauri/src/commands/harmony.rs` must pair with `raise_panic(&state)` so the router's reharm-diff replay (`commands/engine.rs:463-558`) drains stale harmonies. `presets.rs::load_preset` is the analog. New paths that mutate the engine need the same pairing; regression tests in `commands/presets.rs::tests` cover the contract. Don't add `active_notes.clear()` calls outside `clear_active_for_reharm`.

3. **Audio callback must not allocate** — the synth voice processing lives in `src/synth/voice.rs` and the cpal callback runs `Chain::process` from `src-tauri/src/audio_clock.rs`. New code on the audio thread must pre-allocate scratch buffers at construction. (Earlier versions of this footgun cited `src/audio_out/sine_synth.rs:192` — that file was deleted in commit `bb6c691`.)

4. **Tests in `crates/contrapunk-harmony/src/config.rs` doctests reference an external `contrapunk` crate** — they fail on the harmony crate alone (`cargo test -p contrapunk-harmony` shows 9 doctest failures). This is pre-existing; not your fault. Library tests (`--lib`) bypass them.

5. **`.planning/` and `target/` can be huge** — full-tree `grep` / `find` is slow. Always exclude `target/`, `node_modules/`, `ml/venv/`, and `**/wasm-pkg/`. Prefer `cargo` / `npm` scripts over raw greps when you can.

## Release flow

Patch releases (v1.1.0 → v1.1.1) are codified in `.claude/skills/release-patch/SKILL.md`. The macOS universal `.dmg` is built by `.github/workflows/macos-build.yml` on every `v*` tag push and attached to the GitHub Release automatically.

## When in doubt

- Goal first, code second. If the user says "fix species", ask which species, which surface, which symptom — before you start grepping.
- Test after every increment. Don't batch fixes hoping clippy / cargo will catch them.
- Surface failures immediately rather than retrying silently.
- The user is the planner. Pick sensible defaults for ambiguous open questions, surface info once, carry on — don't gate-keep scope or timeline.
