# Clean-room re-engineering candidates

**Date:** 2026-05-11
**Trigger:** Research wave + critique identified four features blocked by AGPL / GPL / unclear-license upstreams. Rather than defer them indefinitely or contaminate Contrapunk's MIT posture, this doc captures them as **clean-room candidates** — MIT-licensed reimplementations under separate repos.

This is a fourth architectural verdict beyond the three in `issue-researcher.md`:

1. ~~in-repo core~~
2. ~~in-repo plugin~~
3. ~~external sub-project~~ (using existing OSS, license-compatible)
4. **clean-room external sub-project** (functionality reimplemented under MIT to avoid an incompatible license on the only available upstream)

Clean-room is expensive — each entry below is a multi-month effort. It's the right move only when (a) the functionality is strategic enough to be worth a dedicated repo and (b) the upstream's license would taint Contrapunk's MIT posture. For one-off integrations that fail those tests, defer indefinitely instead.

## Stack: Rust-first, even for browser targets

All four candidates default to **Rust core + WASM bindings + thin JS host surface**, not JS-native rebuilds. Reasoning:

- Contrapunk's harmony engine is already Rust → WASM (via `wasm-bindgen`). The same toolchain, the same release pattern, the same testing posture, the same author skill set.
- Performance: GLSL chain generation, pattern scheduling, jitter buffers, NTP-style sync — all CPU-bound work that Rust does better than JS even compiled to WASM.
- Type safety carries through. A pattern combinator written in Rust gets compile-time guarantees the equivalent TypeScript would shrug at.
- Shared primitives. A future `contrapunk-livecode` pattern combinator that takes a `HarmonyMode` doesn't need to redefine that enum — it depends on `contrapunk-harmony`, same as the main app.
- Native+web parity for free. A Rust core can power a Tauri-embedded visualizer (no browser at all) AND the `app.contrapunk.com` browser bundle from one source.

The JS surface is just the user-facing API for livecoders — a thin layer that calls into WASM-exposed Rust functions. The user writes `osc(20).modulate(noise(3)).out()` in JS; Rust generates the GLSL, manages WebGL state, and runs the scheduler.

For native-only projects (`contrapunk-net`, `contrapunk-link`), no JS surface at all — pure Rust crates / binaries.

---

## The four candidates

### 1. `contrapunk-viz` — clean-room Hydra equivalent (replaces #101)

**Replaces:** `hydra-synth` (AGPL-3.0, 1.79 MB).

**Functionality:** Functional WebGL visualizer driven by audio FFT. Live-coded chains like `osc(20).modulate(noise(3)).out()`.

**What we build (Rust-first):**
- **Rust crate** (`contrapunk-viz`): GLSL shader generator (the chain → single-fragment-shader compiler), uniform binding logic, AnalyserNode FFT processing. Compiles to WASM via `wasm-bindgen`.
- **WebGL2 renderer**: Rust calls into `web-sys::WebGl2RenderingContext`, OR a `wgpu` backend that targets WebGL2/WebGPU/native (one renderer, three targets).
- **Thin JS surface** (`@contrapunk/viz`): user-facing chain API (`osc(20).modulate(noise(3)).out()`) that calls into the WASM module. ~200-400 LOC of JS, the rest is Rust.
- **Live-reload code editor pane**: SvelteKit-side, integrated into the Contrapunk UI or hosted standalone at `viz.contrapunk.com`.
- **Native variant**: same Rust crate, plus a `wgpu` native renderer, runs inside Tauri without a browser at all. Bonus surface that JS-based Hydra physically can't do.

**Effort:** L (3-6 weeks for v1; the Rust GLSL-generator is more work upfront than a JS one, but the shared rendering across web/native pays it back).
**License:** MIT.
**Repo:** `contrapunk-audio/viz` (new, public). Cargo crate + npm wrapper.
**Integration with Contrapunk:** consumed by the main Contrapunk web app via `import * as viz from '@contrapunk/viz'`, OR as a Cargo dependency in `src-tauri/` for the native-embedded variant. The Rust core can also live as a workspace member if release cadence stays in lockstep.

**Why worth doing:** visualizer is a genuine product moat for an audio tool — users screenshot/share the visuals. Hydra-style livecoding-of-visuals fits Contrapunk's livecoder energy. The AGPL alternative would force `app.contrapunk.com`'s entire shell to AGPL.

**Don't-bother trigger:** if Butterchurn (MIT, 728 KB) ships acceptably, kill this. Butterchurn is a Milkdrop-2 port — different aesthetic from Hydra but legally clean today.

---

### 2. `contrapunk-livecode` — clean-room Strudel equivalent (replaces #15)

**Replaces:** `@strudel/web` + `@strudel/transpiler` (AGPL-3.0).

**Functionality:** Browser-based livecoding pattern language. TidalCycles-derived mini-notation. Generates MIDI events at musical time.

**What we build (Rust-first):**
- **Rust crate** (`contrapunk-livecode`): mini-notation parser (using `nom` or `chumsky`), pattern combinator core (`Pattern<T>` as a function `f: Time -> Vec<Event<T>>`), lookahead scheduler. Compiles to WASM via `wasm-bindgen`. Also runs native for non-browser livecoding hosts (a CLI player, plugin integration, etc.).
- **Pattern combinator API**: `stack`, `cat`, `fast`, `slow`, `every`, `chunk`, `degradeBy`, etc. as Rust functions. Cleanly typed (`Pattern<Note>`, `Pattern<Velocity>`, `Pattern<HarmonyMode>` — leveraging Contrapunk's harmony types).
- **Thin JS surface** (`@contrapunk/livecode`): the user-facing DSL host. Users write JS-flavored code that calls into WASM. Reuses the existing Web MIDI adapter from Contrapunk.
- **CodeMirror editor pane**: SvelteKit-side, deployed at `livecode.contrapunk.com` (separate origin keeps any future legal questions clean, but is no longer required since we're not bundling AGPL code).
- **Harmony-aware primitives** (the strategic moat): `chord("Cmaj7")`, `harmonize(degree)`, `modeBorrow(scale)` — these only work because the pattern engine has direct compile-time access to `contrapunk-harmony` types. JS-based Strudel cannot do this without bridging.

**Effort:** XL (3-6 months for a v1 that's usable; Strudel itself is ~50k LOC of TS). Rust mini-notation parser is comparable effort to TS; the win is on combinators (where Rust's type system actually carries value) and integration (where shared types with `contrapunk-harmony` save the bridging tax forever).
**License:** MIT.
**Repo:** `contrapunk-audio/livecode` (new, public). Cargo crate + npm wrapper.
**Integration with Contrapunk:** depends on `contrapunk-harmony` as a Cargo dep. JS host consumed by the Contrapunk web app as `@contrapunk/livecode` OR hosted standalone.

**Why worth doing:** the livecoding audience overlaps with Contrapunk's target users. Building our own DSL puts us in a strong position long-term — we can extend the pattern language with harmony-aware primitives (`chord("Cmaj7")`, `harmonize(degree)`) that Strudel can't because it doesn't know about Contrapunk's harmony engine.

**Don't-bother trigger:** if user demand stays niche, ship the OSC bridge (`rosc`-based) for TidalCycles + Sonic Pi instead. That's S/M effort and serves the same audience without inventing a new DSL.

**Risk:** XL projects starve other work. Don't start until v1.2.x ships.

---

### 3. `contrapunk-net` — clean-room SonoBus equivalent (replaces #11)

**Replaces:** SonoBus (GPLv3, JUCE-based).

**Functionality:** Low-latency P2P networked audio for live jamming. Mesh of 2-8 peers, Opus-encoded audio over UDP, drift compensation.

**What we build:**
- UDP transport in Rust (std + tokio).
- Opus codec (use `audiopus` crate — BSD-3, MIT-compatible).
- Peer discovery (mDNS via `mdns-sd` or signaling via WebRTC).
- Jitter buffer with drift estimation.
- `cpal`-backed audio I/O.
- Tauri or Electron-style desktop shell for end users.

**Effort:** XL (6+ months for a usable product). This is a small SaaS by itself.
**License:** MIT.
**Repo:** `contrapunk-audio/net` (new, public).
**Integration with Contrapunk:** loopback audio + MIDI between contrapunk-net and Contrapunk on the same machine. No code dependency.

**Why worth doing:** networked jamming is a category Contrapunk could own. Existing tools (SonoBus, JackTrip, Audiomovers, Sessionwire) are all GPL or proprietary.

**Don't-bother trigger:** if the user's audience is solo-musician-with-DAW rather than band-jamming, this is wasted. Validate demand before starting. Practically: don't start until 50+ users have explicitly asked for it.

**Risk:** very high — networked low-latency audio is hard, and the existing tools have ~10 years of head start. A v1 that's worse than SonoBus is worth nothing.

---

### 4. `contrapunk-link` — clean-room Ableton Link equivalent (replaces #98 OR architectural alternative)

**Replaces:** `rusty_link` (GPLv2+, wraps Ableton's GPLv2 C reference impl).

**Functionality:** Distributed tempo sync over LAN. Peers agree on a shared beat clock with sub-millisecond drift correction.

**Important license nuance:** `rusty_link` is GPLv2+ which means linking it into Contrapunk's Tauri binary makes that binary GPLv2+. The "feature flag" mitigation in the v1.2.x roadmap doesn't actually quarantine the license — if the feature is *enabled at build time*, the whole binary is GPLv2+. If we want #98 without contaminating Contrapunk's MIT, we need either:

- **Option A: clean-room Link protocol implementation.** Ableton Link's wire protocol is documented in Ableton's tech notes + the published paper (Goltz 2018). The protocol is essentially NTP-like with peer-to-peer sync and tempo agreement. Reimplement from spec under MIT. Effort: L (3-6 weeks).
- **Option B: external Link bridge process.** A separate GPLv2+ binary that talks to Contrapunk via OSC. Contrapunk stays MIT; the bridge can use `rusty_link`. Effort: S/M (1-2 weeks for the bridge, but adds an external dep for users).

**Recommendation:** Option A. The Link protocol is small enough that clean-room is feasible, and the resulting MIT crate (`contrapunk-link`) is reusable by every Rust audio project that wants Link without GPL contamination — it'd be a meaningful contribution to the Rust audio ecosystem.

**Effort:** L (3-6 weeks).
**License:** MIT.
**Repo:** `contrapunk-audio/link` (new, public) — published as a Rust crate.

**Don't-bother trigger:** if no user has actually asked for Link sync. The v1.2.x roadmap already lists this as Phase 3 / S effort assuming `rusty_link` — that estimate is wrong because of the licensing. Drop #98 from v1.2.x entirely. Either commit to Option A as a separate side project, or drop the feature.

---

## Items that are NOT clean-room candidates

These were considered for this bucket but rejected:

| Item | Why not clean-room |
|---|---|
| **#31 TouchDesigner via td-rs** | TouchDesigner itself is closed-source; td-rs is just a wrapper. The "clean-room" alternative is writing our own TD plugin in C++ against TD's published SDK — feasible but a separate ecosystem play that has nothing to do with Contrapunk's positioning. Drop. |
| **basic-pitch / Demucs ONNX models** | The *models* are MIT/Apache via published ONNX exports (`sevagh/demucs.onnx`, Spotify's basic-pitch). Only the original Python training pipelines have license concerns, which don't affect our runtime use. No clean-room needed. |
| **nih-plug fork** | `BillyDM/nih-plug` is the live MIT/ISC community fork. Rebase, don't clean-room. |
| **JUCE-based libraries generally** | Anything reaching for JUCE in 2026 is a sign to look for a Rust-native alternative first (cpal + symphonia + iced/vizia). JUCE is GPL-or-commercial; cost makes it a non-starter for an indie product. |

---

## Sequencing inside the clean-room bucket

These are mutually competing for the user's time. Recommended order:

1. **`contrapunk-link`** (L, MIT) — first. Smallest, most reusable, useful even if Contrapunk itself never adopts it (the broader Rust audio community benefits). Ships as a crate; doesn't need its own GUI / brand.
2. **`contrapunk-viz`** (L) — second. Visual moat for the product. Higher visibility, faster user impact than Link.
3. **`contrapunk-livecode`** (XL) — third. Only after the OSC bridge proves there's actually a livecoding audience for Contrapunk that wants more than virtual MIDI.
4. **`contrapunk-net`** (XL+) — last. Only after 50+ users have explicitly asked. Don't start because it sounds cool.

None of these belong in v1.2.x. They're tracked here so they don't get re-researched every backlog review.

---

## How clean-room work differs from in-repo work

Workflow differences once one of these starts:

- **Separate repo, separate `.planning/`**, separate harness. The `.claude/agents/issue-researcher.md` and `.claude/skills/*` from this repo are starting points, not requirements — each new repo gets its own scoped harness.
- **Paper trail of derivation.** Every implementation decision references the published spec / paper / patent, NOT the AGPL/GPL source. The `.planning/elixir/oss-prior-art.md` pattern from this repo is the model. Without a paper trail, license cleanliness can be challenged.
- **No copy-paste from the original.** Read the spec, then write fresh code from your understanding. Two-person teams traditionally separate the "studies the GPL source" person from the "writes the MIT replacement" person — but a single developer can do this credibly with discipline and documentation.
- **Release on independent cadence.** The new repo has its own SemVer line, its own changelog. Doesn't follow Contrapunk's release codenames.

---

## v1.2.x roadmap impact

The clean-room verdicts change the v1.2.x roadmap on three items:

- **#98 Ableton Link** — drop from Phase 3. License-contaminating; needs clean-room (separate side project) or skip.
- **#101 visualizer** — clarifies the deferral. Butterchurn is the only legally-clean alternative if we don't build `contrapunk-viz`. Decision: either accept Butterchurn for v1.2.x, or defer entirely until `contrapunk-viz` v1 ships.
- **#15 livecoding** — confirms the deferral. The OSC bridge (`crates/contrapunk-osc/`) remains the cheap path for Sonic Pi + TidalCycles; the Strudel-style in-browser route only happens if/when `contrapunk-livecode` ships.

The v1.2.x roadmap (`ROADMAP-v1.2.x.md`) should be edited to drop #98 and to reference this doc for the visualizer + livecoding decisions.

---

## Index of license verdicts (for future-proofing)

| Library | License | Contrapunk verdict |
|---|---|---|
| `hydra-synth` | AGPL-3.0 | Reject. Clean-room → `contrapunk-viz`. |
| `@strudel/web` | AGPL-3.0 | Reject. Clean-room → `contrapunk-livecode`. |
| SonoBus | GPLv3 (JUCE) | Reject embed. Clean-room → `contrapunk-net`. |
| `rusty_link` (Ableton Link) | GPLv2+ | Reject embed. Clean-room → `contrapunk-link` OR external bridge. |
| `td-rs` | no LICENSE | Block. Not a clean-room candidate (TD is proprietary). |
| `nih-plug` (`robbert-vdh`) | ISC | Upstream dead; rebase onto `BillyDM/nih-plug` fork (also ISC). Not clean-room. |
| `tract-onnx` | MIT/Apache-2.0 | Adopt for #28. |
| `audiopus` | BSD-3 | Adopt if `contrapunk-net` ships. |
| `rosc` | MIT/Apache-2.0 | Adopt if OSC bridge ships. |
| Butterchurn | MIT | Adopt for v1.2.x visualizer IF we don't build `contrapunk-viz`. |
| `hound` | ISC | Adopt for sampler work. |

Keep this table current. New backlog research that proposes a library should check this table first.
