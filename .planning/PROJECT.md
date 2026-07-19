# Contrapunk Rust

## What This Is

A Rust application for real-time MIDI harmony generation across native CLI, Tauri desktop, browser/WASM, and DAW plugin surfaces. It accepts live MIDI or clean monophonic guitar audio, generates harmonies and counterpoint, and routes MIDI and optional built-in synthesis.

## Core Value

Real-time harmony generation with minimal latency — when you play a note, the harmony appears instantly on the output ports.

## Requirements

### Validated

(None yet — ship to validate)

### Active

- [ ] User can select MIDI input device from available ports
- [ ] User can select multiple MIDI output ports (2-8 ports for harmony voices)
- [ ] User can select musical key (C through B, 12 options)
- [ ] User can select harmony mode (7 modes)
- [ ] Mode 1: Forward MIDI as-is (pass-through)
- [ ] Mode 2: Add diatonic thirds above input note
- [ ] Mode 3: Add diatonic fourths above input note
- [ ] Mode 4: Add random diatonic interval below input
- [ ] Mode 5: Add random diatonic below (excluding seconds)
- [ ] Mode 6: Contrary motion — harmony moves opposite to melody
- [ ] Mode 7: Strict counterpoint — follows traditional voice leading rules
- [ ] Original note passes through to first output port
- [ ] Harmony notes route to additional output ports (one voice per port)
- [ ] User can change key and mode during playback without stopping
- [ ] GUI displays active notes and current configuration
- [ ] Application compiles to single binary with no runtime dependencies
- [ ] Clean, monophonic, standard-tuned guitar input produces reliable MIDI notes
- [ ] Guitar processing is deterministic, benchmarked against the checked-in corpus, and real-time safe

### Out of Scope

- Polyphonic guitar, chords, double-stops, alternate tunings, bass, extended-range instruments, and broad microphone support
- Algorithmic melody generation — focus is live performance, not composition
- TUI/curses interface — replaced by native GUI
- Tkinter GUI — legacy Python UI not being ported
- MIDI file input/output — real-time only for v1
- Preset save/load — configure at runtime for now

## Context

**Reference implementation:** Existing Python codebase in this repo provides working harmony algorithms and architecture patterns. Key files:
- `main.py` lines 804-1028: Music theory functions (scale generation, interval calculation)
- `main.py` lines 1084-1199: Mode-based MIDI processing loop
- Harmony modes are pure functions that can be translated directly to Rust

**Platform:** macOS primary development target (CoreMIDI), with cross-platform support for Linux (ALSA) and Windows (WinMM) as secondary goals.

**MIDI architecture:** Input device → processing thread → multiple output ports. State tracking needed for motion-based modes (6, 7) which consider previous notes.

## Constraints

- **Language**: Rust — performance and single binary distribution
- **GUI**: egui/eframe — immediate mode, good for real-time apps, cross-platform
- **MIDI**: midir crate — cross-platform MIDI I/O, well-maintained
- **Guitar claim boundary**: Clean monophonic single-note input on standard-tuned six-string guitar only until broader evidence exists
- **Real-time safety**: Audio callbacks must not allocate, block, or run the heap-oriented guitar detector
- **Single binary**: Must compile to standalone executable with no runtime requirements

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Port to Rust instead of optimizing Python | Performance + distribution (no Python runtime needed) | — Pending |
| Use egui/eframe for GUI | Immediate mode suits real-time display, simple API, cross-platform | — Pending |
| Restore limited guitar-to-MIDI | Existing product surfaces and the repaired 138-file corpus justify a measurable clean-monophonic target | Phase 10.1 automated gates complete; manual release smoke pending |
| Drop algorithmic generation | Scope reduction for v1, can add later if needed | — Pending |

### Elixir Milestone Decisions

Locked decisions from `ELIXIR-PLAN.md` §10 — ingested 2026-05-18 via `/gsd-ingest-docs`. See also: [ELIXIR-DESIGN.md](../ELIXIR-DESIGN.md) and [ELIXIR-PLAN.md](../ELIXIR-PLAN.md).

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| **ELIX-DEC-01** — Workspace location: inside contrapunk workspace | One `cargo check` covers everything; existing CI (clippy, wasm build, tauri build, plugin build) covers Elixir for free; release infrastructure (codesign, notarization) reused. Sibling-repo extraction remains a future option (zero-refactor via `git filter-repo`) but not planned. | LOCKED (ELIXIR-PLAN.md §10 #1) |
| **ELIX-DEC-02** — Cutover ambition: full feature parity at A-Cut (post-A6) | Shipping a partial Elixir as default would mean Contrapunk users lose features the design promises (spectral morphs, unison, chorus, phaser, full filter set) temporarily. The risk profile of "ship the new synth missing features" is worse than "ship old synth for 4 more weeks while we finish A6". Cutover week shifts ~8 → ~12. Track B `elixir-plugin` gives DAW users early access during build-out. | LOCKED (ELIXIR-PLAN.md §10 #2) |
| **ELIX-DEC-03** — Track B UI: egui, separate process/window from Contrapunk | Heavy custom-paint surfaces (mod-matrix, wavetable editor) where browser DOM offers no advantage. Single widget set shared between standalone and plugin in-DAW window — no double UI maintenance. Two products serve different users; keeping them independent (quit one, keep the other open) requires separate UI surfaces. Does NOT affect Contrapunk's own Tauri+Svelte UI (Phase 06.10.1). | LOCKED (ELIXIR-PLAN.md §10 #3) |
| **ELIX-DEC-04** — Track C scope for v1: all three plugin formats (CLAP + VST3 + AU) | Users routing harmony through hosted plugins (e.g., Diva, Pro-Q4) need format coverage that matches what they actually own. AU is mandatory on macOS for Logic users. AU module wrapped in `#[cfg(target_os = "macos")]`. Track C duration revised 8 wk → 9 wk. | LOCKED (ELIXIR-PLAN.md §10 #4) |
| **ELIX-DEC-05** — `elixir-standalone` distribution: public, released from this repo | Shared infrastructure (codesigning, notarization) avoids duplication. Independent product lifecycles via namespaced tags (`elixir-v0.x` vs Contrapunk's `v1.x`). Bundle IDs reserved at `com.contrapunk.elixir` and `com.contrapunk.elixir.plugin`. Elixir tracks its own SemVer independent of Contrapunk's. | LOCKED (ELIXIR-PLAN.md §10 #5) |
| **ELIX-DEC-06** — GSD milestone integration: ingest as real milestone | ELIXIR-PLAN.md and ELIXIR-DESIGN.md are ingested into `.planning/` via `/gsd-ingest-docs`, becoming a real GSD milestone with discuss/plan/execute phases. | LOCKED (ELIXIR-PLAN.md §10 #6) — Completed 2026-05-18 |

---
*Last updated: 2026-05-18 — Added Elixir Milestone Decisions subsection (six ELIX-DEC-0{1..6} locked entries) via `/gsd-ingest-docs` ingest.*
