# The Contrapunk Journey: From First Commit to Real-Time Harmony Engine

*A story told through 312 git commits, 24 planning phases, and one musician's determination to make a computer play counterpoint.*

---

## Prologue: The Python Prototype (January 2025)

The story begins on a Saturday morning in January 2025. At 9:58 AM IST on January 18th, a single commit appears: `first commit`. One file. A README.

But within the next four hours, something extraordinary happens. Seven commits land in rapid succession, each one building on the last:

```
09:58  first commit
10:02  add modes for diatonic thirds, and counterpoint
11:06  forward random diatonic intervals
12:03  add counterpoint mode
12:38  support multiple outputs
13:09  every n harmony is generated based on the n-1 harmony
14:06  add ui
```

By lunchtime, there is a working Python application that takes MIDI input and generates harmonies in real time. Diatonic thirds, random intervals, counterpoint -- the core musical idea is already alive. Multiple MIDI outputs are supported. There is a UI.

Over the next two days, ideas accumulate. WebRTC gets mentioned. Notes get updated. The project goes quiet.

Then, on February 28, 2025 -- about six weeks later -- a late-night burst of activity. At 2:41 AM, audio-to-MIDI conversion gets implemented. By morning, there is a TUI (terminal user interface) with tempo support. The Python prototype is feature-rich but rough around the edges. It works, but it is Python. Performance matters when you are generating harmony in real time. Distribution matters when you want musicians to use your tool.

And then the project goes silent for eleven months.

---

## Act I: The Rewrite Begins (January 28, 2026)

On the evening of January 28, 2026, at 6:57 PM IST, the project wakes up with a purpose. The first commit of the new era is not code -- it is analysis: `docs: map existing codebase`. The Python implementation is studied, its patterns documented, its lessons extracted.

Over the next 8 minutes, a planning system takes shape:

```
18:57  docs: map existing codebase
19:03  docs: initialize project
19:04  chore: add project config
19:05  docs: define v1 requirements
```

This is the moment the GSD (Get Stuff Done) planning framework enters the picture. A `.planning/` directory appears with `PROJECT.md`, `REQUIREMENTS.md`, `ROADMAP.md`, and `STATE.md`. Every phase will be researched before planning, planned before building, and verified after shipping. The config is set to `"mode": "yolo"` with `"depth": "quick"` -- fast execution, but with a research-plan-build-verify discipline baked in.

The requirements are crisp: 28 requirements for v1, covering MIDI I/O, 7 harmony modes, a native GUI, and single-binary distribution. Audio-to-MIDI -- the feature that took a late night in February 2025 -- is explicitly scoped out. Focus.

### Phase 1: MIDI Foundation (Jan 28, ~7:46 PM - 8:28 PM)

The first phase takes 42 minutes from research to completion. Three plans. The Rust project initializes with `midir` for cross-platform MIDI. Port enumeration works. Input connects to output. Notes pass through.

At 8:28 PM, hardware verification confirms: Akai MPK Mini to 4 IAC Driver buses on macOS. Notes go in, notes come out. The foundation is solid.

### Phase 2: Harmony Engine (Jan 28, ~8:36 PM - 10:01 PM)

Music theory arrives in Rust. Six plans in about 85 minutes.

The `Key` type. The `Scale` struct with diatonic transposition. Seven harmony modes, from simple pass-through to strict counterpoint. The stateless modes (thirds, fourths, random intervals) translate almost directly from the Python prototype. The stateful modes (contrary motion, counterpoint) need more care -- they track previous notes, maintain state across events.

By 9:08 PM, the `HarmonyEngine` is integrated into the router. You play a note; you hear harmonies. The core value proposition -- "real-time harmony generation with minimal latency" -- is real.

But counterpoint proves tricky. At 9:26 PM, a fix lands: `improve counterpoint with proper voice-leading rules`. At 9:29 PM, the phase pauses for re-testing. At 10:01 PM, another round of enhancements: interval history, chained harmonies, chromatic handling. Counterpoint, it turns out, is not a problem you solve once.

### Phase 3: GUI and Distribution (Jan 28, ~10:09 PM - 11:42 PM)

`eframe` and `egui` bring a native GUI to life. Six plans, completed in about 90 minutes.

The highlights come fast:
- A `ContrapunkApp` with immediate-mode rendering
- Configuration controls in a side panel
- Active notes displayed with names and colors
- A virtual piano keyboard -- all 88 keys, A0 to C8
- Chord detection that identifies what the combined notes form
- A single binary: 2.9 MB, no external dependencies

At 11:42 PM, Phase 3 is complete. In roughly five hours, the project has gone from "map existing codebase" to a fully functional native application with MIDI I/O, 7 harmony modes, a piano keyboard, and chord detection.

**65 commits on January 28, 2026.** The single most productive day in the project's history.

---

## Act II: Expanding the Architecture (January 29, 2026)

The second day opens with ambition. Four major phases land in a single day -- 50 commits.

### Phase 4: Server Mode (Jan 28 11:46 PM - Jan 29 12:19 AM)

Contrapunk becomes a networked application. A TCP server with a custom wire protocol lets remote clients connect and receive harmonized MIDI. Four plans, a `clap`-based CLI, client mode, server accept loop with connection limiting. A timeout bug surfaces during end-to-end verification and gets fixed.

### Phase 5: Octave Variations (Jan 29, ~12:28 AM - 1:18 AM)

Mirror Octaves: harmony notes duplicate across multiple octaves simultaneously, with port-aware routing. One plan, clean execution. 69 tests pass, including 8 mirror-specific tests.

### Phase 5.1: WASM and Browser Deployment (Jan 29, ~1:38 AM - 2:49 AM)

This is a pivot moment. The project was designed as a native desktop application. Now, in the early hours of the morning, it becomes a web application too.

Three plans transform the codebase:
1. WASM compilation foundation -- `cfg`-gating platform-specific code, Trunk build config
2. Web MIDI backend using `web-sys` -- frame-based polling in the GUI app
3. Deployment to Fly.io

At 2:49 AM: `feat(05.1-03): deploy Contrapunk WASM to Fly.io`. The application is live at `contrapunk.fly.dev`. A musician anywhere in the world can open a browser and generate harmonies. No install required.

A rendering fix follows at 3:12 AM. By 2:18 PM, the phase is formally complete. Human verification: "it works well in the browser."

### Phase 6: Humanization (Jan 29, ~2:29 PM - 3:15 PM)

Generated harmonies sound mechanical. Phase 6 adds humanity. Three plans bring:
- Timing jitter (5-30ms random delays on note onsets)
- Velocity variation
- Note duration variation
- Swing and groove
- An internal beat clock with adjustable BPM
- An optional metronome on a dedicated MIDI channel

The `Humanizer` engine transforms notes, a `DelayQueue` schedules them, and a `Metronome` keeps time. GUI controls appear. WASM compatibility is verified. CI gets a GitHub Actions workflow for Fly.io deployment.

At 3:23 PM, a UI visibility bug surfaces -- humanization controls are hidden below the scroll. A quick fix lands, and the phase pauses.

---

## Act III: Musical Depth (January 30 - 31, 2026)

The project shifts from infrastructure to musical sophistication. 77 commits across two days.

### Phase 6.1: Humanization UI Fix (Jan 29-30)

A focused fix phase: wire the humanizer into the WASM MIDI processing path, redesign the UI with collapsible headers, separate the metronome as an independent feature with its own output port selector.

### Phase 6.2: Voice Leading (Jan 30, ~2:20 PM - 5:44 PM)

This is the phase that earns the name "Contrapunk."

Research into counterpoint theory: Fux's species counterpoint, Bach chorale voice leading, Palestrina style rules, modern jazz voice leading. Then, four plans:

1. **Voice leading rules and style presets** -- Palestrina, Bach, Jazz, Free
2. **A chord-level voicer** with deterministic tiebreaking
3. **A suspension state machine** -- preparation, suspension, resolution, just like in 16th-century counterpoint
4. **Engine integration** and GUI controls

Harmony voices now move by the smallest possible interval. Parallel fifths and octaves are detected and avoided. Voice crossing is minimized. Common tones are held. Dissonance follows species counterpoint rules.

Multiple styles are selectable: strict Palestrina avoids parallel motion entirely; Bach allows some freedoms; Jazz embraces chromatic approach notes; Free mode removes constraints.

### Phase 6.3: Style Update (Jan 30, ~6:17 PM - Jan 31, 5:08 AM)

The GUI gets a complete visual overhaul across 7 plans:

- A steampunk dark theme with gold, copper, and amber (later changed to PICO-8 retro pixel art by user preference)
- Tabbed navigation: Play, Craft, Settings
- 11 built-in style presets with character personas
- Custom steampunk widgets with ornate sliders and decorative frames
- Preset persistence with JSON save/load
- Ambient animations and music-reactive visuals
- Press Start 2P pixel font for a retro aesthetic
- Three-column layout to eliminate scrolling

This is the largest style phase: 20+ commits reshaping the entire user experience.

### Phase 6.4: Modal Harmony and Chord Detection (Jan 31)

The harmony engine grows up:
- All 7 church modes (Ionian through Locrian)
- Harmonic and melodic minor variants
- Modal interchange: when a note is out of key, the engine borrows from parallel modes instead of using generic consonant intervals
- The piano keyboard tints in-scale keys with subtle gold and highlights borrowed notes in amber
- Chord detection expands to 40+ patterns: extended chords, slash chords, add chords, roman numeral analysis
- Roman numeral display: "Fmaj7 (IVmaj7 in C)"

### Phase 6.5: Note Generator (Jan 30-31) -- The First Setback

A virtual MIDI input source: arpeggiator, scale runner, random diatonic, all beat-synced. Four plans get executed. The generator engine works. It appears in the IN dropdown. Piano keys are clickable.

Then human verification fails. The note generator is non-functional. Voice leading styles are not distinct enough. The user defers the feature.

This is the first time a phase is rolled back. The commit message is honest: `docs(06.5): defer phase -- note generator non-functional, voice leading feedback`.

---

## Act IV: Refinement and Infrastructure (February 2026)

With 43 commits on February 2nd alone, the project enters a consolidation phase.

### Phase 6.6: Default MIDI Selection (Feb 2)

A quality-of-life feature: MIDI device choices persist across sessions using port names (not indices, which change between sessions). Works identically on native (file storage) and WASM (localStorage).

### Phase 6.7: Extended Scale Modes and Barry Harris (Feb 2, ~3:35 AM - 4:03 AM)

The scale system expands dramatically:
- 7 modes of harmonic minor
- 7 modes of melodic minor
- 5 exotic scales (Double Harmonic, Hungarian Minor, Enigmatic, Neapolitan Minor/Major)
- 2 Barry Harris 6th diminished 8-note scales
- **28 total scale variants**, grouped by family in the dropdown
- The Scale struct generalizes to support variable-length scales (7 and 8 notes)
- Barry Harris harmony mode implements movement rules: chord tones move to chord tones, passing tones to passing tones

### Phase 6.8: CI Fix (Feb 2)

WASM build breaks are fixed. CI caching improves. Three parallel jobs: check, test, wasm-check.

### Phase 6.9: Repo Cleanup (Feb 2)

The legacy Python files -- the ones that started it all on January 18, 2025 -- are removed. The README is rewritten for the Rust project. CI/CD consolidates into a single workflow. The project formally transitions from "Python prototype with a Rust rewrite" to "Rust application."

### Phase 6.10: Documentation (Feb 5)

Comprehensive rustdoc coverage for the harmony and humanize modules. A CONTRIBUTING.md with architecture overview and a harmony algorithm deep dive. 180 tests pass. `cargo doc --all-features` completes without warnings.

---

## Act V: The Great UI Migration (February 25 - March 2, 2026)

### Phase 6.10.1: UI Modernization -- 9 Plans, One Vision

The biggest architectural change in the project's history. `egui` is abandoned. The new stack: **Tauri v2 + SvelteKit + Svelte 5**, with a visual design inspired by Hyper Light Drifter.

34 commits on February 25th map out the transformation:

1. **Cargo workspace restructuring** -- the core becomes a library crate
2. **SvelteKit scaffolding** with a custom HLD (Hyper Light Drifter) design system: cyan, magenta, teal, gold on dark backgrounds, pixel-art CSS, Press Start 2P font
3. **Platform adapter layer** -- Tauri and WASM implementations behind a common interface, Svelte 5 rune stores for reactive state
4. **88-key piano keyboard** with HLD styling, an Ableton-inspired layout using CSS Grid
5. **MIDI device selection and preset management** with pixel-styled dropdowns
6. **Humanization and generator panels** with HLD health-bar sliders
7. **Atmospheric effects** -- a canvas particle system, neon glow effects, a 4-pip beat indicator
8. **WASM build pipeline** -- `wasm-pack` output into SvelteKit's `$lib`, Docker build for Fly.io
9. **The purge** -- `egui`, `eframe`, CLI mode, Trunk, and the old theme are removed. 6,428 lines deleted in a single commit.

On February 28th, the WASM build is fixed, 4-voice harmony works, and presets are functional. UI settings persist to localStorage.

On March 1st, a sequence of deployment fixes: Dockerfile paths, Rust version bumps (1.85, then 1.88), workspace manifest resolution. The kind of messy, real-world work that makes software actually run in production.

On March 2nd, the audit fixes land: stuck notes resolved, settings validation hardened, dead UI cleaned up, CI strengthened. 36 files changed, 1,118 insertions, 393 deletions.

The application has been reborn. Same Rust core. Entirely new presentation layer.

---

## The GSD Planning System: A Story Within the Story

Contrapunk is not just a music application. It is also a case study in AI-assisted development using the GSD (Get Stuff Done) framework.

Every phase follows the same discipline:

1. **Research** -- `docs(NN): research phase domain` -- understanding the problem space
2. **Plan** -- `docs(NN): create phase plan` -- breaking work into concrete plans with success criteria
3. **Execute** -- `feat(NN-XX): ...` -- implementing each plan
4. **Document** -- `docs(NN-XX): complete [plan name]` -- recording what was built and decided
5. **Verify** -- human verification checkpoints confirm the work is actually done

The `.planning/phases/` directory contains 24 phase directories. Each contains research documents, plan files, and summary documents. The `STATE.md` file tracks velocity metrics:

| Metric | Value |
|--------|-------|
| Total plans completed | 37 |
| Average duration per plan | 3.4 minutes |
| Total execution time | 126.5 minutes |
| Fastest phase average | Repo Cleanup: 1.3 min/plan |
| Slowest phase average | Docs + UI Modernization: 5.5-6 min/plan |

The entire core application -- from MIDI foundation through humanization, voice leading, style presets, modal harmony, 28 scale modes, WASM deployment, CI/CD, documentation, and a complete UI rewrite -- was built in approximately **126 minutes of plan execution time** across **37 completed plans**.

The planning system also tracks decisions. The `STATE.md` file contains 75+ decisions accumulated across all phases, from low-level choices ("Use VecDeque for sliding window history") to architectural ones ("Gate midi module entirely since all files depend on midir").

---

## By the Numbers

| Metric | Value |
|--------|-------|
| Total commits | 312 |
| Time span | Jan 18, 2025 - Mar 2, 2026 (13.5 months) |
| Python prototype era | 16 commits over ~6 weeks |
| Rust rewrite era | 296 commits over ~5 weeks |
| Busiest day | Jan 28, 2026: 65 commits |
| Second busiest | Jan 30, 2026: 57 commits |
| Planning phases defined | 24 (13 completed, 3 in progress/deferred, 8 not started) |
| v1 requirements | 28 defined, 28 completed |
| Harmony modes | 8 (original 7 + Barry Harris) |
| Scale variants | 28 across 5 families |
| Chord detection patterns | 40+ |
| Voice leading styles | 4 (Palestrina, Bach, Jazz, Free) |
| UI frameworks used | 3 (Tkinter in Python, egui in Rust v1, SvelteKit in Rust v2) |
| Deployment targets | Native binary + WASM on Fly.io |
| Tests at last count | 180+ passing |

---

## Timeline Summary

| Date | Milestone |
|------|-----------|
| **2025-01-18** | First commit. Python prototype with counterpoint, diatonic harmony, multiple MIDI outputs |
| **2025-01-19** | Ideas phase: WebRTC, UI experiments |
| **2025-02-28** | Audio-to-MIDI conversion, TUI with tempo support |
| *11-month gap* | |
| **2026-01-28 6:57 PM** | Rust rewrite begins. GSD planning system adopted |
| **2026-01-28 8:28 PM** | Phase 1 complete: MIDI foundation working with hardware |
| **2026-01-28 10:01 PM** | Phase 2 complete: All 7 harmony modes generating real-time harmonies |
| **2026-01-28 11:42 PM** | Phase 3 complete: Native GUI with piano keyboard and chord detection |
| **2026-01-29 12:19 AM** | Phase 4 complete: Networked server mode |
| **2026-01-29 1:18 AM** | Phase 5 complete: Mirror octaves with port-aware routing |
| **2026-01-29 2:49 AM** | Phase 5.1: First WASM deploy to Fly.io -- Contrapunk runs in the browser |
| **2026-01-29 3:15 PM** | Phase 6 complete: Humanization with beat clock, timing jitter, groove |
| **2026-01-30 5:44 PM** | Phase 6.2: Voice leading with Palestrina/Bach/Jazz/Free styles |
| **2026-01-31 5:08 AM** | Phase 6.3: Complete visual overhaul with pixel-art theme and presets |
| **2026-01-31** | Phase 6.4: Modal harmony -- 7 church modes, modal interchange, 40+ chord patterns |
| **2026-01-31** | Phase 6.5: Note generator attempted and deferred -- first setback |
| **2026-02-02** | Phases 6.6-6.9: MIDI persistence, 28 scale modes, Barry Harris, CI fix, Python cleanup |
| **2026-02-05** | Phase 6.10: Full documentation pass, CONTRIBUTING.md |
| **2026-02-25** | Phase 6.10.1: UI rewrite begins -- egui out, Tauri + SvelteKit + HLD design in |
| **2026-03-02** | UI modernization functionally complete, deployed, CI green |

---

## Epilogue: What is Still Ahead

The roadmap extends to Phase 13. Six phases remain unstarted:

- **Phase 7: Performance Mode** -- beat-aware phrase-level harmony, not just note-by-note
- **Phase 8: Mic Input** -- audio capture with pitch detection for voice-to-MIDI
- **Phase 9: Vocoder** -- classic vocoder and real-time vocal harmonization
- **Phase 10: Guitar Input** -- monophonic and polyphonic pitch detection for guitar
- **Phase 11: Trackpad Beat Input** -- computer trackpad as a MIDI beat pad
- **Phase 12: Advanced Voice Leading** -- drop voicings, Neo-Riemannian transforms, negative harmony
- **Phase 13: Voice Leading Test Suite** -- automated regression testing for counterpoint rules

The core is solid. The architecture supports both native and browser. The planning system has proven its discipline. The question is no longer "can this work?" -- it is "how far can this go?"

From a single README on a Saturday morning to a real-time harmony engine running in browsers worldwide, Contrapunk is the story of one musician's vision executed with relentless, systematic momentum.

---

*Document generated from the git history of [Contrapunk](https://github.com/waveywaves/contrapunk). All timestamps are IST (UTC+5:30). Commit data covers 312 commits across 16 unique dates.*
