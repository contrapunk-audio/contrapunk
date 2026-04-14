# Plugin Hosting in Contrapunk — Design Spec

**Status:** Design approved 2026-04-14. Awaiting spec review before writing-plans.
**Scope:** ~4 months of engineering across 7 sub-projects. Each sub-project has its own brainstorm → plan → implement cycle.

## What

Make Contrapunk a standalone playable instrument. Eliminate the need for IAC buses + external DAW for users who just want to hear their harmonies through real synth sounds. Load VST3 plugins (instruments + audio effects) directly inside Contrapunk, route each harmony voice through its own Ableton-style device chain, and output audio to the user's audio interface.

Primary motivating user: someone on macOS with Arturia V Collection (VST3) who wants to play Contrapunk harmonies through Analog V / Piano V / Jup-8 V without setting up IAC buses and a separate DAW.

## Why

Today, Contrapunk outputs MIDI to external ports (IAC on macOS, etc.). To hear anything, the user needs:

1. Create IAC buses in Audio MIDI Setup
2. Open a DAW
3. Load plugins in the DAW
4. Route IAC input → DAW MIDI track → plugin
5. Configure DAW audio output

This is friction that excludes casual users and musicians who don't have a DAW. Replacing this flow with "load a plugin in Contrapunk, play" turns Contrapunk from a MIDI processor into a standalone instrument — dramatically expands the user base.

The existing MIDI-out path is preserved — users who want to continue routing to external DAWs or hardware synths lose nothing.

## Scope (in)

- Load VST3 plugins (instruments + audio effects)
- Per-voice device chains: 1 instrument + unlimited audio effects per voice
- Cross-platform from day 1: macOS, Windows, Linux
- Ableton-style horizontal device view UI in a new "Routing" tab
- Plugin custom GUIs in separate native windows (standard DAW pattern)
- Inline device summary panels in the Routing tab (bypass, gain, top parameters)
- Two-way parameter sync between Contrapunk's inline UI and plugin's custom GUI
- Latency compensation across chains
- Parallel MIDI-out + plugin audio-out per voice (both paths available, toggleable)
- Preset save/load including plugin state chunks
- Plugin directory scanning with caching

## Scope (out) — deferred to later

- CLAP format support (waits for Arturia / major vendors to adopt CLAP)
- AU format support (macOS-only; AU is secondary to the cross-platform VST3 path)
- VST2 (deprecated, no new development)
- Automation: no LFO / envelope / recorded-automation in v1
- Sidechaining between chains
- Audio input processing (treating Contrapunk as an insert effect)
- 5.1 / surround audio
- Plugin sandboxing / process isolation
- Networked audio streams (Audinate / RTP-MIDI — separate project)

## Design decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Plugin format | VST3 only, v1 | Covers Arturia + most commercial plugins. CLAP waits. AU is macOS-only. |
| Per-voice topology | Option B: per-voice chains | Not a single shared plugin slot. Each voice has its own chain. |
| Chain contents | Instrument + audio FX (Ableton parity) | Not instrument-only. Full chain, latency-compensated. |
| Platform | Cross-platform day 1 | Mac + Windows + Linux. VST3 ABI is platform-agnostic; only GUI window parenting differs. |
| VST3 host library | Fork `cutoff/vst3-rs` → `contrapunk-audio/vst3-rs`, build safe layer on top | `cutoff-vst` (the commercial safe layer) is not open-source. We own the stack. |
| Safe host repo | Separate repo `contrapunk-audio/contrapunk-vst3-host` | Reusable asset, potentially open-source later, cleaner boundary enforcement. |
| Audio I/O | `cpal` (already a dep) | Cross-platform, proven, already in use for guitar input |
| Audio sample rate | Auto-match device, user override in preferences | Standard DAW behavior |
| Audio buffer size | 256 samples default (~5.3ms @ 48kHz), configurable | Low-latency default, user can increase if CPU-constrained |
| Chain processing | Sequential in v1 | Parallel introduces ordering/memory-sync complexity. Profile first, parallelize if needed (Ableton approach). |
| MIDI-out path | Preserved and runs in parallel with audio path, per-voice toggle | No regression for existing users. Users can put Voice 1 on a hardware synth + Voice 2-4 on plugins. |
| Plugin GUI windows | Separate native windows per open GUI | Every major DAW does this. Embedding inside WebView is painful on all platforms. |
| Inline device summary | Svelte-rendered using `IEditController` metadata | Ableton-style generic parameter view for quick tweaks without opening the custom GUI |
| Latency compensation | Delay lower-latency chains to match max | Voices stay aligned when one chain has a heavy reverb and another has nothing |
| Parameter automation | Not in v1 | Static params only. Automation is a post-MVP feature. |
| Plugin crash handling | Isolate to chain, zero output, log, red UI state, offer reload | Audio continues for other chains. No whole-app crash. |
| Plugin directories | Default OS paths + user-configurable in preferences | `/Library/Audio/Plug-Ins/VST3` on macOS, `C:\Program Files\Common Files\VST3` on Windows, `/usr/lib/vst3` + `~/.vst3` on Linux |

## Section 1: Architecture

```
  MIDI IN                      (new) Routing Tab UI
  (or guitar audio)                       ↑
       ↓                        ┌─────────┴──────────┐
  ┌──────────────────┐          │ Chain config state │
  │ Harmony Engine   │          │ (voice → [devices])│
  │ (unchanged)      │          └─────────┬──────────┘
  └────────┬─────────┘                    │ (serialized state)
           │ per-voice MIDI               ↓
           ↓                    ┌────────────────────┐
  ┌──────────────────────────→  │ Plugin Host Layer  │
  │                             │ - VST3 loader      │
  │    OR MIDI OUT              │ - Instance mgr     │
  │    (existing ports)         │ - Parameter sync   │
  │    (optional)               └────────┬───────────┘
  │                                      │
  │                              per-voice audio chains
  │                                      ↓
  │                    ┌─────────────────────────────┐
  │                    │ Voice 1: VST3 → FX → FX → ─┐│
  │                    │ Voice 2: VST3 → FX → ──────┤│
  │                    │ Voice 3: VST3 ─────────────┤│
  │                    │ Voice 4: VST3 → FX ────────┘│
  │                    │                             │
  │                    │        Master Mixer         │
  │                    └──────────────┬──────────────┘
  │                                   │
  │                                   ↓
  │                         ┌──────────────────┐
  │                         │ cpal audio output│
  │                         │ (CoreAudio/WASAPI│
  │                         │  /ALSA)          │
  │                         └──────────────────┘
  └── MIDI out still available for external routing
      (users who still want DAW/external synths)
```

### Key libraries

| Concern | Choice | Notes |
|---------|--------|-------|
| VST3 bindings | `contrapunk-audio/vst3-rs` (forked from `cutoff/vst3-rs`) | Low-level raw Rust bindings to VST3 API |
| Safe host layer | `contrapunk-audio/contrapunk-vst3-host` (new repo) | We build this on top of the bindings |
| Audio I/O | `cpal` | Cross-platform, already a dep |
| Plugin GUI windows | `winit` + `raw-window-handle` | Cross-platform native window creation and parent-handle passing |
| Threading | `ringbuf` crate for lock-free SPSC queues | Audio thread is real-time safe, no locks |
| Platform GUI embedding | Platform APIs directly: NSView (macOS) / HWND (Windows) / XEmbed (Linux) | `raw-window-handle` gives uniform abstraction |

### Guiding principles

1. **The existing MIDI-out path stays untouched.** Users can still use IAC buses and external synths alongside plugin hosting. Both run in parallel, per-voice toggle.
2. **Audio thread is sacred.** No allocations, no locks, no IO on the audio callback. All setup happens off-thread; runtime communication is lock-free ringbuffers.
3. **Harmony engine doesn't know about plugins.** The harmony engine stays pure MIDI-generating. Audio graph layer bridges harmony MIDI → plugin audio.

## Section 2: Audio graph + threading

### Audio parameters

| Param | Default | Range |
|-------|---------|-------|
| Sample rate | 48 kHz (auto-match device) | 44.1 / 48 / 96 kHz |
| Buffer size | 256 samples (~5.3ms) | 64 / 128 / 256 / 512 / 1024 |
| Channels | Stereo (2) | v1 only |

### Per-frame audio processing flow

```
cpal audio callback (audio thread, real-time safe)
  │
  ├─ Pull pending MIDI events from lock-free ringbuf
  │  (dispatched per-voice from harmony engine)
  │
  ├─ For each voice chain (sequential in v1):
  │    1. Zero out chain's stereo buffer
  │    2. instrument.process(midi_events_for_voice, empty_audio, chain_buffer)
  │    3. For each FX in chain:
  │         fx.process(chain_buffer, chain_buffer)  // in-place
  │    4. Apply chain gain + pan
  │    5. Apply latency compensation delay (align with slowest chain)
  │
  ├─ Sum all chain buffers into master stereo buffer
  ├─ Apply master gain
  ├─ Write master buffer to cpal output
  │
  └─ (no allocations, no locks, no system calls on this path)
```

### Threading model

| Thread | Responsibilities | Can it allocate? |
|--------|------------------|------------------|
| Audio thread (cpal callback) | Process chains, dispatch MIDI, render audio | **No.** Lock-free only. |
| Control thread (Tauri main) | Plugin load/unload, param changes, preset save | Yes |
| UI thread (Svelte) | React to store changes, render UI | Yes |
| Plugin GUI thread(s) | Plugin's own GUI event loop | Varies per plugin |

### Inter-thread communication

- **Control → Audio**: lock-free ringbuffer for commands (`LoadPlugin`, `UnloadPlugin`, `SetParam`, `SetGain`, `Bypass`). Audio thread drains at buffer boundaries.
- **Audio → Control**: lock-free ringbuffer for events (output meters, plugin crash notifications, note-off pings).
- **Harmony → Audio**: per-voice MIDI events via lock-free SPSC channel.
- **UI → Control**: Tauri IPC (standard async, not audio-thread critical).

### Latency compensation

- Each plugin reports `IAudioProcessor::getLatencySamples()`
- Find max latency across all active chains
- Delay every other chain to match (via ring-buffer delay lines before the master mixer)
- Recalculate on any chain change (plugin add/remove/reorder)
- Prevents phase smear when Voice 1 has a 50ms reverb and Voice 2 has none

### MIDI-out path (unchanged)

Harmony engine's output goes to BOTH:
- **External MIDI ports** (existing behavior — IAC buses, hardware synths)
- **Audio thread's per-voice ringbuffer** (new — feeds plugin instruments)

User toggles per-voice: "audio only", "MIDI only", or "both".

### Glitch handling

- Plugin panics → isolate chain, zero output, log error, show red state in UI, offer reload. Audio continues for other chains.
- Audio device unplug → cpal notifies, fall back to default device or pause audio gracefully.

## Section 3: VST3 hosting strategy

### The ownership decision

`cutoff-vst` (Renaud Denis's commercial library) is not open-source. We cannot fork it. We build the safe host layer ourselves on top of the forked bindings.

### Repo structure

| Repo | Type | Purpose |
|------|------|---------|
| `contrapunk-audio/vst3-rs` | Fork of `cutoff/vst3-rs` (MIT/Apache) | Low-level raw bindings to VST3 C++ API, generated from headers via `libclang` |
| `contrapunk-audio/vst3_c_api` | Fork of `cutoff/vst3_c_api` (MIT/Apache) | VST3 C API headers dependency |
| `contrapunk-audio/contrapunk-vst3-host` | New, private | The safe host layer we build |
| `contrapunk-audio/contrapunk` | Existing | Main app, consumes `contrapunk-vst3-host` as a git dependency |

### What the safe host layer must implement

```
contrapunk-audio/contrapunk-vst3-host/
├── src/
│   ├── factory.rs        Plugin bundle loading, factory discovery
│   ├── instance.rs       IComponent + IEditController pair, lifecycle
│   ├── process.rs        Audio processing, MIDI event translation, param queues
│   ├── params.rs         Parameter metadata + normalized [0,1] values + automation
│   ├── view.rs           IPlugView wrapper, platform-specific parent attach
│   ├── host_context.rs   Our IHostApplication implementation
│   ├── preset.rs         State chunks (IBStream) save/restore
│   ├── scanner.rs        Directory scanning + metadata caching
│   ├── bus.rs            Bus configuration (audio I/O, event I/O)
│   └── error.rs          Result types, plugin-crash-safe panic handling
└── examples/
    └── cli_host.rs       vst3-host-cli /path/to/plugin.vst3
                          (loads a plugin, processes scripted MIDI, writes WAV)
```

### Cross-platform bits

| Platform | Shared-lib ext | GUI parent type | Audio backend (cpal) |
|----------|----------------|-----------------|----------------------|
| macOS | `.vst3` (bundle, dylib inside) | `NSView` | CoreAudio |
| Windows | `.vst3` (DLL) | `HWND` | WASAPI |
| Linux | `.vst3` (bundle, .so inside) | `XID` (X11) | ALSA/JACK |

The VST3 ABI itself is identical across platforms — only GUI window parenting differs. `raw-window-handle` gives uniform abstraction.

### Plugin directory scanning

| Platform | Default VST3 path |
|----------|-------------------|
| macOS | `/Library/Audio/Plug-Ins/VST3` + `~/Library/Audio/Plug-Ins/VST3` |
| Windows | `C:\Program Files\Common Files\VST3` |
| Linux | `/usr/lib/vst3` + `~/.vst3` |

User-configurable in preferences.

### What v1 does NOT implement

- `IAutomationState` / automation-aware processing
- `INoteExpressionController` (beyond basic pitch bend + pressure)
- Multi-bus VST3 plugins (Kontakt's multi-output busses) — treated as stereo out only
- VST3's own `IPluginCompatibility` format conversion

All deferred to post-MVP.

## Section 4: Plugin GUI window embedding

### Strategy: separate native windows

Every open plugin GUI gets its own native OS window via `winit` + platform-native view attach. Standard DAW pattern (Logic, Ableton, Reaper, Bitwig).

### Inline summary + separate window — both

Matching Ableton's model:

1. **Plugin's custom GUI** — separate floating window with the plugin vendor's own UI (Arturia's analog knobs, Kontakt's sample browser)
2. **Inline device summary** — Svelte-rendered compact strip in the Routing tab showing: plugin name, bypass toggle, gain fader, top 3-5 auto-selected parameters

VST3's `IEditController` gives us per-parameter metadata (name, range, default, units, steps) so we can render the inline summary without touching the plugin's custom GUI. Two-way lock-free parameter sync keeps them in step.

### Lifecycle

- **Open GUI**: user clicks ⚙ on a device in the Routing tab → host instantiates the window → attaches plugin view → shows window
- **Close GUI**: user clicks X on the window → plugin view detached → window destroyed → device in Routing tab shows ⚙ (closed state)
- **Plugin unload**: all open GUIs for that plugin auto-close first, then plugin instance is destroyed
- **App close**: all plugin GUIs close, plugins unload, host shuts down

### Window state

- Position/size persist per plugin instance (reopening remembers where it was)
- Stored in session/preset file alongside routing state
- Multi-monitor: remember monitor, fall back to primary if it's gone

### Size + DPI

- Plugin tells us via `IPlugView::getSize()`
- Plugin requests resize via `IPlugFrame::resizeView()` → honor or clamp
- DPI: `IPlugViewContentScaleSupport` provides the scale factor; window framework handles rendering

## Section 5: Routing tab UX

### Layout (Ableton-style horizontal chain view)

```
┌─ Routing ─────────────────────────────────────────────────────────────┐
│  Voice 1  ┌──────────┐    ┌──────────┐    ┌──────────┐    ┌───┐       │
│           │ Piano V  │ →  │ Reverb   │ →  │ EQ Eight │ →  │ + │       │
│           │ ▣ bypass │    │ ▣ bypass │    │ ▣ bypass │    └───┘       │
│           │ gain ▓▓░ │    │ decay ▓▓ │    │ lo  ▓▓░  │      add       │
│           │ [⚙ open] │    │ mix ▓▓░  │    │ mid ▓░   │   device       │
│           └──────────┘    └──────────┘    └──────────┘                 │
│                                                                       │
│  Voice 2  ┌──────────┐    ┌───┐                                        │
│           │ Jup-8 V  │ →  │ + │                                        │
│           └──────────┘    └───┘                                        │
│                                                                       │
│  Voice 3  ┌───┐                                                        │
│           │ + │  empty — MIDI-out only, no audio                       │
│           └───┘                                                        │
│                                                                       │
│  Voice 4  ┌──────────┐    ┌──────────┐                                 │
│           │ CS-80 V  │ →  │ Delay    │                                 │
│           └──────────┘    └──────────┘                                 │
│                                                                       │
│  ─────────────────────────────────────────────────────────────────    │
│                                                                       │
│  Master   │ Gain ▓▓▓▓▓▓▓▓░░    Audio: Audient iD14 ▼                  │
│           │ SR: 48 kHz  Buf: 256  CPU: 12%                             │
└───────────────────────────────────────────────────────────────────────┘
```

### Device card anatomy

Each card (Svelte component) shows:
- **Header**: plugin name (truncated), tiny icon for instrument vs FX
- **Bypass toggle**: ▣/☐ checkbox, instant audible effect
- **Gain fader** (instrument only): per-device gain, 0-200% log-scaled
- **Top 3-5 params**: auto-selected from `IEditController` flags
- **Open-GUI button**: opens plugin's custom GUI in a separate native window
- **Hover/right-click**: Remove / Reset / Copy / Save preset / Open GUI

### Interactions

- Click `[+]` → plugin browser modal
- Drag device left/right → reorders within chain (audio thread reorder message + latency recompute)
- Drag device across chains → moves between voices
- Right-click device → context menu
- Right-click chain row → Clear chain / Duplicate / Solo / Mute

### Plugin browser modal

```
┌─ Load Plugin ────────────────────────────────────────────────┐
│  [🔍 search _______________]  Sort: Name ▼   Category ▼      │
│  ─────────────────────────────────────────────────────────── │
│  INSTRUMENTS                                                 │
│  ▸ Arturia                                                    │
│      Analog Lab V   Piano V III   CS-80 V   Jup-8 V          │
│  ▸ Native Instruments                                         │
│      Kontakt 7    Massive X                                   │
│                                                              │
│  EFFECTS                                                     │
│  ▸ FabFilter                                                  │
│      Pro-Q 3    Pro-R                                         │
│  ▸ Valhalla                                                   │
│      VintageVerb    Delay                                     │
│                                                              │
│                             [Cancel]  [Load]                  │
└──────────────────────────────────────────────────────────────┘
```

- Filesystem scan on first launch, cached
- Rescan button in Settings
- Category = `PClassInfo::category` (Fx, Instrument, Analyzer)
- Keyboard nav: ↑/↓/Enter/Esc

### Master section

- Master gain fader + dB meter (post-sum)
- Audio device dropdown (enumerated via cpal)
- Sample rate + buffer size (display, click to override)
- CPU meter (% of buffer time used)

### Visual style

Match existing PICO-8 retro pixel-art theme:
- Press Start 2P font
- Green = active/enabled, magenta = bypassed
- Chunky bordered boxes, no gradients
- Pixel-art arrows between devices

### State persistence

Routing config (plugins loaded, device order, params, gain, bypass, plugin state chunks) serializes to a preset file alongside harmony presets. Save/load routing presets just like harmony modes.

## Section 6: Sub-project decomposition

### The sub-projects

| # | Sub-project | Repo | Est. weeks |
|---|------------|------|-----------|
| 1 | Audio foundation | `contrapunk` (new module `src/audio_out/`) | 2 |
| 2 | VST3 host library MVP | `contrapunk-audio/contrapunk-vst3-host` | 5-6 |
| 3 | Plugin GUI embedding | `contrapunk-vst3-host` | 2 |
| 4 | Routing tab MVP (1 instrument per voice) | `contrapunk` + Svelte UI | 2 |
| 5 | Inline device summary + parameter sync | `contrapunk-vst3-host` + Svelte | 1-2 |
| 6 | FX chain per voice | `contrapunk-vst3-host` + Svelte | 3 |
| 7 | Persistence + polish | `contrapunk` | 2 |

**Total: 17-19 weeks for production quality.** ~4 months.

### Execution order

```
[1. Audio foundation] ──────┐
                             ├──→ [4. Routing tab MVP] ──→ [5. Inline summary]
[2. VST3 host MVP] ──→ [3. GUI embedding] ─┘                           │
                                                                       ↓
                                               [6. FX chain]  ←───── [7. Persistence]
                                                    │
                                                    └──→ done
```

**Hard dependencies:**
- #3 needs #2
- #4 needs #1 + #2
- #5 needs #4
- #6 needs #4
- #7 needs #4 + #6

**Parallelizable:**
- #1 and #2 can run in parallel (different repos, different agent sessions)
- #5, #6, #7 are post-MVP improvements; order mostly flexible

### 🎯 Minimum Viable Milestone

**"Play Arturia Analog V through Contrapunk"** = sub-projects **#1 + #2 + #3 + #4** = 9-11 weeks.

At MVP completion:
- Launch Contrapunk
- Open Routing tab
- Load Arturia Analog V onto Voice 1
- Click ⚙ to open its GUI, tweak the synth
- Play guitar/MIDI → harmony voices drive the plugin → audio out to Audient iD14
- No more IAC buses needed

This is the ship-worthy demo. Everything after (FX chain, inline summary, persistence) transforms "works" into "as-good-as-Ableton."

## Risks & mitigations

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|-----------|
| VST3 spec edge cases (parameter flushing, preset chunks) | High | Medium | Reference `cutoff-vst`'s public checklist (150+ functions) as coverage target. Test against top plugins (Arturia, FabFilter, Valhalla) incrementally. |
| Plugin crashes the whole app | Medium | Critical | Isolate each chain, zero output on crash, UI shows red state, offer reload. Never panic unwind into the audio thread. |
| Cross-platform GUI embedding is more work than budgeted | Medium | High | Ship macOS first in sub-project #3, add Windows + Linux as follow-ups if needed. `raw-window-handle` gives us the abstraction. |
| Real-time audio glitches under plugin load | Medium | High | Profile early. Pre-allocate all buffers. No allocations in audio callback. Sequential chain processing. |
| User plugin library is VST2-only | Low | Medium | Arturia V (primary target) is VST3. Other major vendors support VST3. If user has legacy VST2-only plugins, they can use the existing IAC/DAW flow for those. |
| Renaud Denis's cutoff-vst goes open-source, duplicating our work | Low | Low | Our work remains valuable as the Contrapunk-owned layer. We can evaluate using cutoff-vst at that point with our existing layer as fallback. |
| Latency compensation introduces bugs | Medium | Medium | Ship without compensation in sub-project #6 first, add as a polish pass. Test with known-latency plugins (Pro-Q 3 has reported latency). |

## Licensing

- `contrapunk-audio/vst3-rs` — fork of MIT/Apache project, stays MIT/Apache
- `contrapunk-audio/vst3_c_api` — fork of MIT/Apache project, stays MIT/Apache
- `contrapunk-audio/contrapunk-vst3-host` — MIT/Apache dual-license to match the Rust ecosystem convention. Repository stays private during development; becomes public once mature enough to be used by other Rust audio projects.
- `contrapunk` main app — existing license (check `LICENSE` file in repo)

## Out of scope for this spec — future follow-ups

- CLAP format support (sub-project 8+): reuses audio graph + Routing tab UX, adds a CLAP loader module to the host library. Probably 3-4 weeks after VST3 path is stable.
- AU format support: macOS-only, likely skippable if VST3 covers all macOS plugins users need.
- Audio input processing (Contrapunk as insert effect in a DAW): separate project.
- Networked audio streams (Contrapunk Cloud): separate project, different architecture.
- Plugin sandboxing (subprocess isolation per plugin): separate project, low priority until crashes become a real user pain.
