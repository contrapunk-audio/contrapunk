# Research: Livecoding language integration (Strudel, Sonic Pi, TidalCycles)

**Issue(s):** #15
**Date:** 2026-05-11
**Researcher:** issue-researcher
**Verdict:** **mixed** — Strudel: in-repo UI (Svelte route + in-page bridge); Sonic Pi + TidalCycles + FoxDot: one in-repo crate `crates/contrapunk-osc/` (feature-flagged, native-only) plus shipped preset mappings as docs.

## Problem

Livecoders write algorithmic patterns in Strudel / Sonic Pi / TidalCycles / FoxDot and want Contrapunk to harmonise the resulting note stream in real time — turning a single-voice pattern into a 4-voice counterpoint. Today there is no bridge: livecoder notes can reach Contrapunk only by manually routing through a virtual MIDI port (IAC on macOS, loopMIDI on Windows), and the four environments differ enough that one bridge does not fit all.

## Touchpoints

Bridge insertion points already exist; this work plugs *into* them rather than replacing them.

- **Native MIDI input router**: `crates/contrapunk-midi/src/input.rs:32-81` (`connect_input`) — `mpsc::Sender<Vec<u8>>` of raw MIDI bytes. Anything that can emit `wmidi`-compatible byte vectors flows through here.
- **Native router thread**: `src/router.rs:134` (delay-queue drain), `src-tauri/src/commands/engine.rs:384` (Tauri lift) — where harmonised notes get fanned out. OSC-driven note-on must enter here as a `NoteOn { pitch, velocity, channel }` event, not as raw bytes.
- **WASM bridge** (Strudel path): `wasm/src/lib.rs` exposes `Engine::note_on(u8) -> Vec<u8>` and `Engine::note_off(u8) -> Vec<u8>`. Confirmed in `.planning/research/opendaw-integration.md:73-80` — same callable surface the openDAW integration uses.
- **WASM adapter**: `ui/src/lib/adapter/wasm.ts:241-246` — `ensureMidiAccess()` already manages `navigator.requestMIDIAccess()`. Strudel runs in the same window, so it can share this `MIDIAccess` directly. No new permission prompt.
- **UI lane shell**: `ui/src/lib/components/` — companion architecture (commit `2c796ab`) gives us lanes/orchestrator; a `StrudelLane` slots in beside `MidiDevices.svelte`, `Piano.svelte`, etc.
- **Cargo workspace**: `Cargo.toml:2-14` — `crates/contrapunk-osc` becomes a new workspace member, mirroring `crates/contrapunk-midi`.
- **Preset surface**: `crates/contrapunk-preset/` — OSC address mapping ships as a `.json` preset, not as code.

## Architecture verdict

Three integrations, **two artefacts**.

1. **Strudel — in-repo UI (Svelte route + in-page bridge).** Strudel ships as `@strudel/web` on npm (AGPL-3.0). Its scheduler exposes `onTrigger(hap, deadline, duration)` per cycle; `hap.value.note` is the MIDI pitch. We intercept at this callback, call `Engine.note_on(pitch)` synchronously, then re-emit the originals **plus** harmony voices to either the existing WebMIDI output or a `@strudel/webaudio` synth. AGPL-3.0 is the licence sting: Strudel must live behind a route boundary (a separate `/strudel` SvelteKit route that can be a separate AGPL build target, or a dynamic-imported lazy chunk) so Contrapunk's MIT core stays untainted by linkage. Verdict justification: this is a one-surface feature (browser only) that gets first-class value from sharing the AudioContext + WebMIDI permission and the already-loaded WASM engine — making it external would mean redundantly bundling the WASM engine and re-prompting for MIDI. Entropy cost: one new lazy route, one peer dep `@strudel/web`, ~250-500KB extra chunk gated to that route.

2. **OSC bridge for Sonic Pi / TidalCycles / FoxDot — `crates/contrapunk-osc/` (feature-flagged, native-only).** All three speak OSC over UDP. Sonic Pi sends on port 4560 by default; TidalCycles emits `/dirt/play` with format `,sfsfsfsisssf` to port 57120; FoxDot is configurable. One crate, one UDP listener, one OSC→`NoteOn`/`NoteOff` adapter, three address-pattern mappings shipped as presets. Verdict justification: OSC is one wire format; per-language bridges would triplicate the same UDP socket loop. The interesting code is the address-pattern translator, not the protocol — and that is config, not code. Cost: one new ~500-line crate, `rosc 0.11.4` (MIT/Apache-2.0, no transport, last release March 2025), native-only (`cfg(not(target_arch = "wasm32"))`), behind feature `osc-bridge` so the WASM/plugin builds carry zero overhead.

3. **Per-language docs.** Sonic Pi, TidalCycles, FoxDot each get a short `docs/integrations/<tool>.md` page explaining how to point them at Contrapunk's OSC listener, plus a `presets/livecoding-<tool>.json` mapping shipped in `crates/contrapunk-preset/`. No code per language.

Unidirectional first (livecoder → Contrapunk → MIDI out). Bidirectional (Contrapunk → OSC out to e.g. drive Sonic Pi's `live_loop :harmonies`) is interesting but not in scope for v1 — list as an open question.

## Implementation outline

### Strudel (browser path)

1. Add `@strudel/web` and `@strudel/transpiler` as dev dependencies in `ui/package.json`. Gate via `npm run check:licenses` — flag AGPL-3.0 explicitly. Document the licence boundary in `ui/src/routes/strudel/README.md`.
2. New route `ui/src/routes/strudel/+page.svelte`. Lazy-loaded so the AGPL chunk is opt-in; the rest of `app.contrapunk.com` stays MIT.
3. In the new route, `await initStrudel()` then register a custom output module that wraps the default `webaudio` output:
   ```js
   const harmonised = (hap, deadline, duration) => {
     const note = hap.value.note;
     const harmonies = engine.note_on(note); // existing WASM API
     // schedule original + harmonies through @strudel/webaudio or WebMIDI
   };
   ```
4. Reuse `WasmAdapter`'s `ensureMidiAccess()` for the WebMIDI output path. No second permission prompt.
5. Preset: `Strudel — Counterpoint` ships with `mode: StrictCounterpoint`, `voices: 3`, default scale Ionian.
6. UAT: the canonical Strudel example `note("c a f e")` plays as a 4-voice chorale.

### OSC bridge (native path)

1. New workspace member `crates/contrapunk-osc/` with one file `src/lib.rs` and `src/listener.rs`. Cargo:
   ```toml
   [dependencies]
   rosc = "0.11"      # encode/decode only
   tokio = { version = "1", features = ["net", "rt", "macros", "sync"] }
   wmidi = "4.0"
   thiserror = "2.0"
   ```
   Or: synchronous `std::net::UdpSocket` + a dedicated thread, matching the existing `src/server/` style (`server/session.rs` already runs one thread per client). The synchronous path keeps deps minimal — **prefer it** unless a perf bench shows otherwise.
2. Add feature flag `osc-bridge` in root `Cargo.toml`. Default off. Gate via `[target.'cfg(not(target_arch = "wasm32"))'.dependencies]`.
3. `OscBridge::new(bind_addr) -> Self` opens a `UdpSocket`. Spawn one thread that loops `recv_from` → `rosc::decoder::decode_udp(&buf)` → `OscMessage` → translate via `AddressMap` → `NoteOn { pitch, velocity, channel }` → push onto the same `mpsc::Sender<MidiEvent>` the MIDI input router uses today (`crates/contrapunk-midi/src/input.rs:34`).
4. `AddressMap` is a `HashMap<String, NoteExtractor>`. Three built-in extractors ship as JSON presets:
   - **Sonic Pi**: `/midi/*` ← user-defined; or use Sonic Pi's `midi_note_on n, vel` over the IAC bus and skip OSC altogether. Document both. Sonic Pi's authoring style favours MIDI output, so the canonical "Sonic Pi → Contrapunk" doc points at MIDI; OSC is fallback.
   - **TidalCycles**: `/dirt/play` (port 57120) — extract `n` (note) and `s` (sound) fields from the tagged blob; treat `n` as semitones from middle C.
   - **FoxDot**: `/play_synth` (port 57110-range) — semantic similar to TidalCycles; address pattern differs.
5. Surface in Tauri commands: `start_osc_bridge(port)`, `stop_osc_bridge`. Settings UI exposes the bind port (default 57121 — not 4560 / 57120 so we don't collide with Sonic Pi / SuperDirt running locally) and the active address map.
6. Settings UI flag: `live_coding_osc.enabled` (default off). Status row in `StatusBar.svelte` shows "OSC: 57121" when active.
7. CLI flag: `contrapunk --osc-port 57121` for the headless use case.

### Per-language docs

Three short files in `docs/integrations/`:
- `strudel.md` — open `/strudel` in the browser, paste pattern, hit play.
- `sonic-pi.md` — preferred: virtual MIDI port via IAC/loopMIDI, route to Contrapunk MIDI input. Alternative: `use_osc "localhost", 57121` + `osc "/note_on", 60, 100`.
- `tidalcycles.md` — point Tidal's superdirt target at port 57121, or run alongside SuperDirt and OSC-fork via `osc_send`.
- `foxdot.md` — `Server.set_addr("localhost", 57121)`.

## Test strategy

- **TDD order (OSC crate)**:
  1. Unit test: `OscMessage` with `/note_on` + `[60i32, 100i32]` decodes via `rosc::decoder::decode_udp` and translates to `NoteOn { pitch: 60, vel: 100, ch: 0 }`. (Pure function — no socket.)
  2. Unit test: TidalCycles `/dirt/play` `,sfsfsfsisssf` blob (golden bytes captured from a real `tidal-listener` capture, committed to `crates/contrapunk-osc/testdata/`) decodes to the right `n` value.
  3. Unit test: malformed packet → `Err(OscError::Decode)`, no panic, listener thread survives.
  4. Integration test: open a `UdpSocket` on `127.0.0.1:0`, send a packet, assert the `mpsc::Receiver` yields the expected `MidiEvent` within 50ms.
  5. Property test (proptest): random valid OSC bytes never panic the decoder; bounded note range (0-127) always; out-of-range silently drops.

- **Strudel side**:
  1. Vitest unit test on the `harmonised` output: given a fake `hap` with `note: 60`, the output emits 1 original + N harmony note events to a mock WebMIDI output.
  2. Playwright e2e (lives in `ui/tests/`): navigate to `/strudel`, paste `note("c").s("sine")`, click play, assert 4 simultaneous WebAudio nodes are alive.

- **Manual UAT**:
  - Sonic Pi tutorial example "Tetris theme" → Contrapunk renders 3-voice harmony in real time.
  - TidalCycles `d1 $ n "0 ~ 7 3" # s "piano"` → Contrapunk harmonises each note.
  - No stuck notes when the livecoder hits `hush` / panic-stop.

- **Latency budget**: OSC RX → harmony engine → MIDI out target ≤ 5ms p99 on the dev mac. Worse than that and livecoders will route around us.

## Dependencies

| Surface | New dep | Version | Licence | Maintenance | Size impact |
|---|---|---|---|---|---|
| OSC crate | `rosc` | 0.11.4 (Mar 2025) | MIT / Apache-2.0 | active, pure Rust, 215 stars | ~50KB, transport-free (we own the socket) |
| OSC crate | `tokio` (optional) | 1.x | MIT | active | only if we choose async; std socket avoids it |
| Strudel route | `@strudel/web` | latest | **AGPL-3.0** | active (Tidal team) | ~250-500KB gzipped chunk, lazy-loaded |
| Strudel route | `@strudel/transpiler` | latest | AGPL-3.0 | active | bundled with above |

`rosc` is encode/decode only — no UDP listener. We bring transport. That's actually a feature: we wire the listener to our existing `mpsc` event bus directly, no glue.

**AGPL-3.0 boundary management** — Strudel's licence is the real architectural constraint, not the technical integration. The route boundary (lazy chunk, separate page, separate build target in CI if needed) is what keeps Contrapunk MIT. Document the boundary in `LICENSE.md` and in the Strudel route's README. Consider serving the Strudel route from a separate Cloudflare Pages deployment that links back to the main app — this is a clean separation and likely the safest reading of AGPL §13 "remote network interaction".

## Entropy impact

- **+1 workspace member** (`crates/contrapunk-osc/`). Doesn't increase WASM bundle. Doesn't increase plugin binary (feature-flagged off by default).
- **+1 SvelteKit route** (`/strudel`). Doesn't touch the main `+page.svelte`.
- **+1 npm peer dep**, lazy-loaded. First-load size unchanged.
- **+0 lines** in the harmony engine (`crates/contrapunk-harmony/`). The harmony engine stays surgical, as the entropy budget requires.
- **+1 feature flag** (`osc-bridge`) in root `Cargo.toml`. Per `.planning/research/issue-triage.md:140`, "soft cap on new transitive deps in `src-tauri/`" — `rosc` is one transitive, behind a feature flag, native-only. Safe.
- **Release boundary**: Strudel is AGPL — must live behind a build/route boundary. This is a real ongoing cost (license review for every Strudel update, careful chunking) but is unavoidable for browser livecoding integration of any vendor.
- **Affects existing files**: minimal. `Cargo.toml` (one new member), `ui/package.json` (one new dep), maybe `StatusBar.svelte` (one new status row), `tauri.conf.json` (allow listening on UDP for the feature flag).

## Open questions / blockers

- **Sonic Pi: OSC or virtual MIDI?** Sonic Pi's idiomatic path for sending notes to an external synth is `midi_note_on` over a virtual MIDI port. OSC is "send arbitrary control data" territory. Recommendation: docs make MIDI the default, OSC the power-user fallback. No code difference — both already work through the existing MIDI input router.
- **Bidirectional OSC?** Should Contrapunk emit harmony events back to the livecoding tool (e.g. Sonic Pi receives Contrapunk's harmonies as OSC cues and plays them through its own synths)? Useful but not in v1. List as a v2 follow-up.
- **AGPL-3.0 due diligence.** Before merging the Strudel route, get a legal sanity check on AGPL §13 vs Cloudflare Pages hosting (specifically: is serving a page that loads an AGPL bundle from the same origin "remote network interaction" that triggers source disclosure?). Likely yes, in which case we publish the route's source separately from the MIT core. Spike work: minimal — small `STRUDEL-AGPL/` subdir with its own `LICENSE` and a build step.
- **AudioWorklet vs ScriptProcessor for Strudel's audio path.** Strudel uses an AudioWorklet by default. Our existing `GuitarAudioCapture` (`ui/src/lib/audio/guitarCapture.ts:121`) uses the deprecated ScriptProcessor (`.planning/codebase/CONCERNS.md:144`). No conflict for Strudel (different AudioContext lane), but worth noting that the AudioWorklet migration on the guitar capture side (concern in `CONCERNS.md`) and the AudioWorklet usage on Strudel side will touch the same `AudioContext` plumbing. Not a blocker, just a flag.
- **OSC packet rate ceiling.** TidalCycles can fire dozens of OSC packets per cycle at high CPS. With 1024-event MIDI queue cap (`src/audio_out/midi_queue.rs:36`, `.planning/codebase/CONCERNS.md:133`), is there a risk of QueueFull → router panic? Recommendation: at OSC bridge boundary, rate-limit and pre-drop with logging before reaching the existing MIDI queue. Document the cap.
- **FoxDot maintenance.** FoxDot's last major release was 2020. Lower priority than the other three. Defer FoxDot doc page until v1.1 of this work.

## Estimated effort

- **Strudel route**: **M** (3-7 days). Mostly licence boundary + Svelte integration + getting `onTrigger` interception right.
- **OSC bridge crate**: **S** (1-3 days). Pure data transformation + one socket thread + tests.
- **Docs + presets**: **XS** (≤1 day).
- **Total**: **M** (5-9 days, parallelisable).
