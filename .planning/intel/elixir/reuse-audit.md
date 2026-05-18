# Code Context

## Files Retrieved
1. `Cargo.toml` (lines 1-86) - workspace layout, `elixir-core` optional dependency, root dependencies/features that constrain direct reuse.
2. `src/chain/block.rs` (lines 1-78) - reusable `AudioBlock` and `MidiBlockEvent` contract.
3. `src/chain/chain.rs` (lines 1-120) - existing linear audio chain and command-drain pattern.
4. `src/chain/command.rs` (lines 1-22) - command enum for lock-free chain mutation.
5. `src/chain/commander.rs` (lines 1-130) - SPSC producer/mirror pattern for UI→audio chain mutation.
6. `src/chain/elixir_block.rs` (lines 1-135) - existing Elixir-to-Contrapunk adapter.
7. `src/fx/delay.rs` (lines 1-520) - mature built-in delay with atomic params, tempo sync, `Transport`, tests.
8. `src/fx/reverb.rs` (lines 1-360) - mature built-in Freeverb-style reverb block with atomic params/tests.
9. `src/synth/params.rs` (lines 1-150) - lock-free atomic parameter surface for legacy synth.
10. `plugin/src/lib.rs` (lines 1-220) - nih-plug parameter/export patterns for the existing Contrapunk plugin.
11. `src/plugin_host/clap/audio_block.rs` (lines 1-334) - live CLAP plugin `AudioBlock`, scratch buffers, MIDI ring, event translation.
12. `src-tauri/src/commands/plugins.rs` (lines 1-190) - Tauri command path for CLAP discovery/activation/chain insertion.
13. `crates/contrapunk-transport/src/clock.rs` (lines 1-180) - reusable atomic transport/BPM/beat model.
14. `crates/contrapunk-midi/src/lib.rs` (lines 1-9) - MIDI crate module gates for native/Web MIDI.
15. `crates/contrapunk-audio/src/buffer.rs` (lines 1-240) - ring/overlap buffers; useful pattern but guitar-analysis-specific.
16. `crates/contrapunk-preset/src/lib.rs` (lines 1-86) and `crates/contrapunk-preset/src/storage.rs` (lines 1-12) - existing serde JSON preset style for harmony presets.
17. `crates/elixir-standalone/src/main.rs` (lines 1-220) - current standalone threading/MIDI/audio path and known demo-grade `Arc<Mutex<Engine>>` use.

## Reuse Audit Table

| Component | Existing implementation path | Can Elixir reuse directly? | Why / why not | Recommended action |
|---|---|---:|---|---|
| Audio block interface | `src/chain/block.rs` | Yes, for Contrapunk integration | `AudioBlock` is already the stable in-process contract for source synths, FX, and hosted plugins. `MidiBlockEvent` already covers NoteOn/Off, AllNotesOff, sustain. | **Reuse directly** for A-Cut and any Contrapunk-facing Elixir block. Do not invent another chain trait. |
| Elixir adapter into Contrapunk | `src/chain/elixir_block.rs` | Yes | Existing feature-gated adapter owns `elixir_core::Engine`, forwards `process`, and maps `MidiBlockEvent` to engine calls. It is exactly the A-Cut seam. | **Reuse/adapt directly**. Extend this adapter with A6 params/preset mapping instead of creating a parallel adapter. |
| Linear audio chain | `src/chain/chain.rs` | Yes for Contrapunk, not inside `elixir-core` | Chain already handles ordered `AudioBlock`s and drains mutation commands on the audio thread. `elixir-core` is `no_std` and should not depend on root crate/std chain internals. | **Reuse directly at host layer**. Keep Elixir’s internal FX chain separate unless/until extracted to a shared DSP crate. |
| Chain mutation queue | `src/chain/command.rs`, `src/chain/commander.rs` | Yes for Track C / host UI | SPSC command pattern already prevents locks in audio callback and maintains UI mirror. | **Reuse directly** for plugin-host chain operations; **adapt pattern** for future standalone/plugin param queues. |
| Existing built-in delay DSP | `src/fx/delay.rs` | Not directly in `elixir-core` | Mature tempo-sync delay, atomics, `Transport`, `AudioBlock`, tests. But it depends on `std`, `Arc`, root `AudioBlock`, and `contrapunk_transport`; `elixir-core` is currently `no_std` by default. | **Adapt/extract later**. Reuse semantics/tests/subdivision model. If sharing is desired, extract a `contrapunk-fx`/`elixir-fx` crate after A-Cut; do not copy root `AudioBlock` version into no_std core. |
| Existing built-in reverb DSP | `src/fx/reverb.rs` | Not directly in `elixir-core` | Mature Freeverb-style implementation with atomics and `AudioBlock`, but not FDN-16 and std/root-coupled. Elixir A6 needs self-contained FDN-16 plus WASM fallback. | **Adapt tests/algorithm ideas only**. Do not reuse as A6 primary reverb; keep Elixir FDN self-contained. Consider shared crate later for Schroeder/Freeverb fallback. |
| Transport/BPM clock | `crates/contrapunk-transport/src/clock.rs` | Yes outside `elixir-core`; maybe optional in standalone/plugin host | Atomic `Transport` is already used by plugin/root FX for tempo sync and beat crossings. It is std/Arc-based, not no_std. | **Reuse directly in host layers** (`elixir-standalone`, `elixir-plugin`, Contrapunk A-Cut) for tempo-synced LFO/delay. Keep `elixir-core` transport-agnostic or accept transport values via simple params. |
| MIDI native/Web module | `crates/contrapunk-midi/src/lib.rs`, `crates/contrapunk-midi/src/input.rs`, `output.rs`, `ports.rs`, `web.rs` | Partially | The standalone currently uses `midir` directly. The crate already centralizes native/Web MIDI gates. But Elixir plugin MIDI comes from nih-plug events, not this crate. | **Adapt/reuse for standalone and future browser surface**. Do not use for plugin audio callback; translate host note events directly. |
| Legacy synth params | `src/synth/params.rs` | Yes as A-Cut compatibility model | Provides atomic, lock-free parameter pattern and exact legacy defaults/waveform enum that A-Cut must preserve. | **Reuse/adapt**. Map `SynthParams` to Elixir `ParamId`/preset defaults; copy the atomic scaled-storage pattern for plugin-grade params. |
| Legacy synth DSP | `src/synth/voice.rs`, `src/synth/mod.rs` | No as implementation, yes as parity reference | Elixir is meant to replace it. Reusing internals would undermine A6 and still lacks spectral/unison/FDN features. | **Do not reuse** for new DSP. Use for A/B parity tests and default-preset mapping only. |
| Root FX atomic param pattern | `src/fx/delay.rs`, `src/fx/reverb.rs`, `src/synth/params.rs` | Yes as pattern | Parts-per-thousand atomics, bool flags, clamp/round semantics, no locks in hot path are already proven. | **Adapt pattern** for `elixir-core::params` or `elixir-plugin` params; avoid `Arc<Mutex<Engine>>` in B4/plugin. |
| nih-plug plugin patterns | `plugin/src/lib.rs` | Yes as pattern, no as code | Existing plugin is a Contrapunk harmony/MIDI plugin, not Elixir synth. It demonstrates `Params`, `EnumParam`, plugin constants, transport sync, MIDI event processing, CLAP/VST3 exports. | **Adapt** into a new `crates/elixir-plugin`; do not merge with or rename `plugin/`. |
| Plugin note/UI state pattern | `plugin/src/lib.rs`, `plugin/src/editor.rs` | Limited | Useful for WebView/editor patterns, but Elixir Track B locked egui/separate UI, not Contrapunk Svelte webview. | **Use as reference only** for DAW event/editor timing. Do not reuse WebView editor for Elixir unless product direction changes. |
| CLAP hosted plugin audio block | `src/plugin_host/clap/audio_block.rs` | Yes for Track C | Already solves port layout, scratch buffers, MIDI event ring, CLAP event translation, `AudioBlock` integration. | **Reuse directly** for C0. Improve/validate rather than rebuilding CLAP host audio path. |
| CLAP Tauri command path | `src-tauri/src/commands/plugins.rs` | Yes for Track C desktop | Already runs activation/GUI on main thread, registers controller, pushes `ClapAudioBlock` into chain. | **Reuse/adapt** for plugin-chain UI, bypass/remove/open GUI. Do not create a separate CLAP host command stack. |
| VST3/AU hosting | `src/plugin_host/mod.rs` currently only exports CLAP | No implementation to reuse | No VST3/AU host modules exist. macOS objc deps exist in root for windowing, but no AU host. | **Build new modules**, but mirror CLAP controller/block API so Track C formats share a host abstraction. |
| Preset JSON style | `crates/contrapunk-preset/src/lib.rs`, `storage.rs` | Partially | Existing preset crate is harmony/style-specific (`StylePreset` depends on harmony types), but serde JSON import/export pattern is useful. | **Adapt/extract schema pattern**. Create Elixir-specific preset type; do not force synth preset fields into `StylePreset`. Consider extending crate with separate `ElixirPreset` only if dependency direction stays clean. |
| Audio buffer/ring helpers | `crates/contrapunk-audio/src/buffer.rs` | Mostly no | `RingBuffer` uses `Vec`, `Clone+Default`, and `rotate_left` on overflow; good for analysis, not ideal for audio delay hot paths or no_std Elixir core. `OverlapManager`/`DualBufferAnalyzer` are guitar-pitch-specific and allocate/clone frames. | **Do not reuse directly** in DSP hot paths. Reuse conceptual fixed-capacity buffer idea only; keep Elixir delay/FDN lines custom. |
| Guitar/audio detection modules | `crates/contrapunk-audio/src/*` | No | Project direction excludes pitch/guitar work; modules are analysis/detection not synth. | **Do not reuse** for Elixir implementation except possible offline test signal helpers. |
| Harmony/chord/companion crates | `crates/contrapunk-harmony`, `contrapunk-chord`, `contrapunk-companion` | No for synth DSP | Musical logic is upstream of synth. Elixir is an audio engine; these crates should feed MIDI notes into it, not become dependencies of `elixir-core`. | **Do not reuse in core**. Use only at host/Contrapunk integration boundary. |
| Standalone cpal/midir scaffolding | `crates/elixir-standalone/src/main.rs` | Yes, with caveat | Already opens audio/MIDI/UI. It intentionally uses `Arc<Mutex<Engine>>`, which is demo-grade and not plugin safe. | **Reuse for B6/B7 standalone**, but **replace/adapt** to lock-free command/param queue before B4/plugin-grade path. |
| Standalone egui widgets | `crates/elixir-standalone/src/ui.rs` | Yes for Track B egui UI | Existing knob/card/curve/keyboard patterns are already Elixir-specific and align with locked egui UI decision. | **Reuse directly** and extend for A6 controls. Later extract shared egui widgets for `elixir-plugin` editor if needed. |
| Tauri/Svelte UI adapter patterns | `ui/src/lib/adapter/*`, `src-tauri/src/commands/*` | Not for Elixir standalone/plugin UI; yes for Track C | Contrapunk UI is Svelte/Tauri and locked separate from Elixir egui UI. However Track C plugin-host controls belong in Contrapunk UI. | **Reuse for Track C only**. Do not port Elixir synth UI to Svelte unless product decision changes. |
| Root `src/fx` as shared FX library | `src/fx/mod.rs`, `delay.rs`, `reverb.rs` | Not directly today | It is root-crate, std, `AudioBlock`-oriented. Elixir A6 is no_std core and needs different DSP/API. | **Do not reuse directly now**. Recommended longer-term action: extract root FX and Elixir FX common pieces into a workspace crate after A-Cut if duplication becomes painful. |

## Key Code

### `AudioBlock` is the canonical host boundary

`src/chain/block.rs` (lines 1-78) defines `MidiBlockEvent` and `AudioBlock` with the realtime contract: `process(&mut [f32], channels)`, `midi_event`, `reset`, `set_sample_rate`, and `enabled`. Elixir already has `src/chain/elixir_block.rs` wrapping `elixir_core::Engine`; A-Cut should extend this rather than invent a new bridge.

### Existing root FX are useful but root-coupled

`src/fx/delay.rs` (lines 1-520) includes a mature delay with:

- atomic `DelayParams`
- tempo sync via `contrapunk_transport::Transport`
- stable `Subdivision` encoding/string IDs
- `AudioBlock` implementation
- tests for bypass, echo, feedback, reset, mix, subdivision, BPM sync

`src/fx/reverb.rs` (lines 1-360) includes a Freeverb-style block with atomic params and tests. Both are good sources for parameter semantics and tests, but direct reuse in `elixir-core` would pull in `std`, root `AudioBlock`, and transport assumptions.

### Chain mutation and plugin-host code should not be rebuilt

`src/chain/chain.rs`, `command.rs`, and `commander.rs` already implement the SPSC command and UI mirror pattern. `src/plugin_host/clap/audio_block.rs` already solves CLAP processing as an `AudioBlock`, including scratch buffers and MIDI event conversion. `src-tauri/src/commands/plugins.rs` already performs main-thread CLAP activation and pushes the live audio block into the chain.

## Architecture

Elixir has two different reuse zones:

1. **Core DSP (`crates/elixir-core`)**: keep small/no_std/host-agnostic. Avoid direct dependencies on root `contrapunk`, root `src/fx`, Tauri, nih-plug, or `contrapunk_transport`. Reuse algorithms/patterns, not root `AudioBlock` code.
2. **Host/adapters (`src/chain`, `crates/elixir-standalone`, future `crates/elixir-plugin`, Tauri/plugin-host)**: reuse existing `AudioBlock`, `Chain`, `ChainCommander`, `Transport`, MIDI, plugin-host, and nih-plug patterns directly.

That means “reuse wherever possible” should mostly mean:

- Reuse **interfaces and host plumbing directly**.
- Adapt **atomic param patterns and tests**.
- Extract shared DSP later if needed.
- Avoid forcing std/root modules into `elixir-core` while it is intentionally no_std/WASM-friendly.

## Start Here

Open `src/chain/elixir_block.rs` first for A-Cut reuse: it is already the Elixir adapter into the existing audio chain. For avoiding DSP reinvention, open `src/fx/delay.rs` next and reuse its parameter semantics/tests/subdivision model rather than duplicating ad-hoc tempo-sync behavior in Elixir.

## Supervisor coordination

No blocker requiring a decision. The only product-level choice to keep visible: direct reuse of root `src/fx` inside `elixir-core` conflicts with Elixir's current no_std/core independence. Recommended path is host-level reuse now, shared-FX extraction after A-Cut if duplication remains costly.
