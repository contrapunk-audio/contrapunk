---
name: contrapunk-nih-plug-skeleton
description: Create or review nih-plug CLAP/VST3 plugin crates in Contrapunk, especially Elixir plugin work. Use when adding plugin/Cargo.toml crates, implementing Plugin/ClapPlugin/Vst3Plugin, handling MIDI events, or creating DAW-facing synth/effect skeletons.
---

# Contrapunk nih-plug Skeleton

## Pattern

For new DAW plugin surfaces, start with a small compiling `nih_plug` crate that exports both CLAP and VST3, then add full parameter/editor/preset surfaces in later commits.

## Minimum Skeleton

1. Add a workspace crate with:
   - `[lib] crate-type = ["cdylib", "rlib"]`
   - `nih_plug` dependency pinned to the repo's existing git fork
   - product/core crate dependency (`elixir-core`, etc.)
2. Implement:
   - `#[derive(Params)]` parameter struct
   - plugin struct with `Arc<Params>` and DSP engine state
   - `impl Plugin` with `AUDIO_IO_LAYOUTS`, MIDI config, `initialize`, `reset`, `process`
   - `impl ClapPlugin`
   - `impl Vst3Plugin`
   - `nih_export_clap!` and `nih_export_vst3!`
3. For instrument plugins:
   - `main_input_channels: None`
   - mono + stereo output layouts
   - `MIDI_INPUT: MidiConfig::Basic`
   - `ClapFeature::Instrument` + `ClapFeature::Synthesizer`
   - VST3 subcategories `Instrument` + `Synth`

## Realtime Rules

- Do not allocate in `process()`.
- Use `BufferConfig::max_buffer_size` to preallocate scratch in `initialize()` when scratch is needed.
- For tiny mono/stereo frame rendering, prefer stack arrays (`[f32; 2]`) over heap scratch.
- Handle sample-accurate MIDI by iterating `buffer.iter_samples().enumerate()` and applying events whose `event.timing() <= sample_id`.
- Keep B3 small: no editor, no giant parameter matrix, no preset format. Those belong to B4/B5.

## Validation

- `cargo check -p <plugin-crate>`
- If the crate touches shared DSP, also run the source core tests (e.g. `cargo test -p elixir-core --lib`).
- Check both CLAP and VST3 metadata for stable IDs before release.
