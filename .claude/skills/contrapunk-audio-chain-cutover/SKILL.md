---
name: contrapunk-audio-chain-cutover
description: Wire or review feature-flagged audio-chain cutovers in Contrapunk, especially replacing the legacy built-in Synth with ElixirSynthBlock behind elixir-synth. Use when touching src-tauri/src/audio_clock.rs, src/chain/*, built-in synth routing, Chain AudioBlock setup, or A-Cut work.
---

# Contrapunk Audio Chain Cutover

## Pattern

When replacing a built-in audio component, keep the chain contract stable and switch only the block instantiation behind a Cargo feature. The router should continue sending the same `SynthEvent`/MIDI events unless a dedicated event bridge is explicitly part of the task.

## Default Approach

1. Preserve public commands and UI behavior first.
2. Add a small constructor/helper that selects the first `AudioBlock`:
   - `#[cfg(feature = "elixir-synth")]` → `ElixirSynthBlock`
   - `#[cfg(not(feature = "elixir-synth"))]` → legacy `Synth`
3. Keep `BlockDescriptor` type IDs honest:
   - legacy: `builtin.synth`
   - Elixir: `builtin.elixir-synth`
4. Do not import root `src/fx` into `elixir-core`; share only neutral primitives through `contrapunk-dsp`.
5. Validate both feature configurations:
   - `cargo check -p contrapunk`
   - `cargo check -p contrapunk --features elixir-synth`
   - relevant Tauri/chain tests when touched

## A-Cut Gotchas

- `src-tauri/src/audio_clock.rs` owns the cpal chain startup; this is where the first synth block is instantiated.
- `src/chain/elixir_block.rs` already adapts `elixir_core::Engine` to the `AudioBlock` trait behind `elixir-synth`.
- `src-tauri/src/commands/engine.rs` still emits `SynthEvent` to the built-in synth channel; if Elixir is selected through the chain, ensure events still reach the chain or add a bridge explicitly.
- Keep non-F32 cpal paths safe; they currently output silence and tick transport only.
- Avoid adding locks or allocation to the cpal callback.

## Success Criteria

- Both feature-on and feature-off builds compile.
- Feature-off behavior remains legacy-compatible.
- Feature-on chain reports/uses `builtin.elixir-synth` and renders notes through Elixir.
- No new audio-thread allocation or blocking beyond existing chain behavior.
