# Context Intel

Topic-keyed running notes appended verbatim with source attribution. Sourced from DOC-type inputs and from prose blocks in SPECs that don't fit `decisions.md` / `requirements.md` / `constraints.md`.

The Elixir ingest contained no DOC-type inputs; the entries below are SPEC prose extracted as background context.

---

## Topic: Why Elixir is a major new initiative

> **Source:** `ELIXIR-PLAN.md` §0 TL;DR (entire framing)

Three parallel tracks. Each phase is a ship gate — nothing merges that doesn't pass the gate's audible/observable test. The three tracks share one engineering substrate: the `Chain` / `AudioBlock` abstraction in `src/chain/`, which is already in place and has a lock-free SPSC command queue (`ChainCommand::PushBlock` etc.). All three tracks plug into it.

Total calendar time if one developer works strictly serially: ~40 weeks. With Track B running parallel to A (most of B reuses A's core), and Track C interleaved on different files, **realistic shipping window is 24–28 weeks**.

This is a milestone-sized addition spanning three product surfaces:
1. **Internal:** Replacing Contrapunk's built-in synth (Track A)
2. **External plugin:** Hosting third-party plugins inside Contrapunk (Track C)
3. **New product:** Public Elixir standalone + plugin released from this repo (Track B)

## Topic: Elixir's distinguishing trait

> **Source:** `ELIXIR-DESIGN.md` §1

Elixir is a polyphonic wavetable synthesizer whose distinguishing trait is **frequency-domain warping**: each wavetable frame is editable in the spectral domain, and an oscillator can morph the spectrum at audio rate through one of twelve operations (vocode, smear, harmonic-scale, phase-disperse, shepard, skew, etc.) before the IFFT writes time-domain samples for playback. The result feels like a wavetable synth and a spectral-modeling synth fused into one.

## Topic: Why workspace location is locked to inside contrapunk

> **Source:** `ELIXIR-PLAN.md` §1 "Why inside the workspace"

- One `cargo check` covers everything.
- Existing CI (clippy, wasm build, tauri build, plugin build) covers Elixir for free.
- Release infrastructure (code-signing, notarization, `.github/workflows/macos-build.yml`) is reused — Elixir gets DMG + plugin bundle builds out of the same pipeline.
- `elixir-standalone` and `elixir-plugin` are their own crates with no Contrapunk dependencies — they can be extracted to a separate repo later if desired with zero refactoring (a `git filter-repo` away).

## Topic: Why cutover ambition is full feature parity (locked)

> **Source:** `ELIXIR-PLAN.md` §2

A-Cut happens **after A6 lands**, not after A5. Rationale: shipping a partial Elixir as the default would mean Contrapunk users lose features that the design doc promises (spectral morphs, unison, chorus, phaser, full filter set) on a temporary basis. The risk profile of "ship the new synth missing features" is worse than "ship the old synth for 4 more weeks while we finish A6". Track B isn't blocked — `elixir-plugin` ships earlier and gives users the new sound through their DAW; A-Cut is purely about Contrapunk's built-in synth.

## Topic: Why egui is locked for Elixir's UI (and Svelte for Contrapunk's)

> **Source:** `ELIXIR-PLAN.md` §4 UI choice

`egui` is chosen for these reasons:
- Heavy custom-paint surfaces (mod-matrix, wavetable editor) where browser DOM offers no advantage.
- Single widget set shared between `elixir-standalone` and `elixir-plugin`'s in-DAW window — no double UI maintenance.
- Elixir's standalone app launches as a **separate window/process** from Contrapunk. By design, not a regression — keeps the two products independent (you can quit Contrapunk and keep Elixir open, run Elixir inside a DAW with Contrapunk closed, etc.).

Tauri+Svelte is rejected for B6 specifically. (It remains the right call for Contrapunk's main app; the two surfaces serve different users.)

## Topic: Public release from this repo means decoupled release cadence

> **Source:** `ELIXIR-PLAN.md` §4 public release

Tag `elixir-v0.1.0` triggers a release workflow that:
1. `cargo test -p elixir-core` + `cargo test -p elixir-plugin` + `cargo test -p elixir-standalone`.
2. Build matrix: macOS arm64 + x86_64 universal, Windows x86_64, Linux x86_64.
3. Sign + notarize macOS artifacts using existing Contrapunk certs.
4. Bundle: `elixir-standalone.dmg`, `Elixir.vst3`, `Elixir.clap`, `Elixir.component` (AU), `elixir-headless` binary.
5. Attach all artifacts to a new GitHub Release on this repo with hand-written release notes (use the `release-patch` skill, extended to recognise the `elixir-` tag prefix).

This means Elixir's release cadence is decoupled from Contrapunk's. v1.4 of Contrapunk can ship before `elixir-v0.1.0`. They're independent products from one monorepo.

## Topic: Plugin discovery and the user library (Track C)

> **Source:** `ELIXIR-PLAN.md` §5

Standard OS paths per format:
- **CLAP:** `~/Library/Audio/Plug-Ins/CLAP/` (macOS), `%CommonProgramFiles%\CLAP\` (Windows), `~/.clap/` and `/usr/lib/clap/` (Linux)
- **VST3:** `~/Library/Audio/Plug-Ins/VST3/` (macOS), `%CommonProgramFiles%\VST3\` (Windows), `~/.vst3/` and `/usr/lib/vst3/` (Linux)
- **AU:** `~/Library/Audio/Plug-Ins/Components/` (macOS only)

Scan is async, off the audio thread, cached to `~/.config/contrapunk/plugins.json` with mtime invalidation. Existing `discovery.rs` already does this for CLAP — generalise the pattern.

## Topic: Multi-plugin chain UI (Track C)

> **Source:** `ELIXIR-PLAN.md` §5

When ≥ 2 plugins are loaded, Contrapunk needs a per-slot UI strip: name, format badge (CLAP/VST3/AU), bypass, latency report, "open GUI", parameter expander, and "remove". This is ~1 week of UI work and folds into Contrapunk's existing Svelte UI (not Track B's egui — different product, different users).

## Topic: WASM caveat

> **Source:** `ELIXIR-PLAN.md` §7

Contrapunk's WASM surface doesn't render Rust audio today — the browser drives Web Audio from JS. Compiling `elixir-core` to wasm32 is enforced for the *standalone* Elixir web product (if Track B ever ships one), not for replacing Contrapunk's WASM synth (which doesn't exist).

## Topic: Surface matrix — what ships where

> **Source:** `ELIXIR-PLAN.md` §7

| Crate / module | CLI bin | Tauri desktop | WASM browser | contrapunk_plugin (VST3/CLAP) | elixir-standalone | elixir-plugin |
|---|---|---|---|---|---|---|
| `elixir-core` | yes (post-A1) | yes (post-A1) | yes (post-A1, smoke-tested in CI; not wired to audio output until/unless Contrapunk's WASM gets a Rust audio path) | yes (post-A1) | yes | yes |
| `src/synth/` (legacy) | until A7 | until A7 | n/a | until A7 | no | no |
| `src/plugin_host/clap/` | yes | yes | no | yes | no | no |
| `src/plugin_host/vst3/` (Track C) | yes | yes | no | yes | no | no |
| `src/plugin_host/au/` (Track C) | macOS only | macOS only | no | macOS only | no | no |
| `egui` UI widgets (`elixir-ui` if extracted) | no | no | no | no | yes | yes |

## Topic: Recommended start sequence (next 2 weeks)

> **Source:** `ELIXIR-PLAN.md` §11

**Week 1**
1. (1 d) Run `/gsd-ingest-docs ELIXIR-DESIGN.md ELIXIR-PLAN.md` so the planning artifacts land in `.planning/` and the GSD workflow picks them up.
2. (1 d) A0: add `crates/elixir-core` with empty `Engine`, wire it as a feature-flagged `AudioBlock` in `src/chain/`. Confirm CI green on all 4 surfaces.
3. (2 d) B0: skeleton `elixir-standalone` binary that opens cpal + midir and plays silence; egui window with a single "Hello Elixir" label.
4. (1 d) Reserve `com.contrapunk.elixir` and `com.contrapunk.elixir.plugin` bundle IDs in App Store Connect; extend `release-patch` skill to recognise the `elixir-` tag prefix.

**Week 2**
1. (5 d) A1 starts: voice handler + wavetable oscillator + DAHDSR. Daily commits; daily audible-progress demo against the previous day's build.
2. (in parallel, 2 d) C0 prep: read `src/plugin_host/clap/block.rs` and `controller.rs` stubs end-to-end; write the design note for finishing CLAP activation; confirm the 2-week budget for C0.

End of week 2 you have:
- Elixir crate compiling on every surface and ingested into `.planning/` as a GSD milestone
- Standalone binary opening audio + window (no notes yet)
- A clear go/no-go on C0's 2-week budget
- Bundle IDs reserved so signing isn't a last-minute blocker

## Topic: Implementation notes that don't fit a single constraint

> **Source:** `ELIXIR-DESIGN.md` §9 open questions

- **SIMD width.** Portable-simd lets you write width-agnostic code but is nightly. Stable + `wide::f32x4` / `f32x8` works but requires picking a width. Recommend `f32x8` (256-bit) for desktop, `f32x4` (128-bit) for any WASM build, gated by `cfg(target_feature)`.
- **GUI choice.** `egui` is the fastest path but limits visual polish. Web-based (Tauri / WebView) gives unlimited polish at the cost of an IPC boundary. Pick early; the mod-matrix and wavetable editor are large UI surface area. (Locked in plan §10 #3: egui.)
- **WASM target.** Every DSP component compiles to WASM today, but oversampling and the reverb are expensive; consider FDN-8 + 1x oversampling for WASM gated behind a `cfg` flag.
- **Tuning.** Ship with `.scl` / `.kbm` from v1; defer MTS-ESP until a user asks.
- **Plugin distribution.** `nih-plug`'s CLAP support is solid; VST3 needs signing on Windows and notarization on macOS. Budget a week for code-signing logistics before v1.

## Topic: Relationship to existing Contrapunk planning

> **Source:** synthesizer's note, comparing ELIXIR docs to existing `.planning/`

The ingested docs reference (and replace) several pieces of existing planning surface:
- **`src/synth/`** (current `Synth`, `voice.rs`, `params.rs`) — replaced at A-Cut per ELIX-DEC-02.
- **`src/plugin_host/clap/`** (stubbed `block.rs` / `controller.rs`) — finished by Track C Phase C0 per REQ-elixir-c0-clap-activation. This finishes work that ROADMAP.md Phase 16 ("VST3/CLAP/AU Plugin") started and that STATE.md "v1.3.x" describes as "Companion lanes deferred to v1.4".
- **`ui/src/lib/adapter/`** — picks up new param keys post-A-Cut. No surface change.
- **`.github/workflows/macos-build.yml`** — extended (not replaced) with `elixir-standalone` + `elixir-plugin` flows.

The existing Contrapunk UI (Tauri + Svelte) per Phase 06.10.1 is **NOT** affected by Elixir's egui choice. Two products, two UIs, by design.

ROADMAP.md Phase 16 ("VST3/CLAP/AU Plugin") refers to **packaging Contrapunk itself as a plugin** (existing work, ~4/7 plans complete). Track C of Elixir is a different effort: **enabling Contrapunk to host other plugins**. These are complementary, not duplicative.

---

*Synthesized: 2026-05-18 from `/gsd-ingest-docs` run on ELIXIR-DESIGN.md + ELIXIR-PLAN.md.*
