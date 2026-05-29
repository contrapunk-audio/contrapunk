# Golem Design

Golem is an audio-native adaptive drummer for the Contrapunk workspace. It is a separate product/crate family like Elixir: it can run standalone first, then later be hosted by Contrapunk as an audio block.

## Product stance

Golem is not a MIDI drummer. It does not depend on Contrapunk's MIDI router and it does not emit MIDI as its primary output.

Golem listens to musical context and produces drum audio:

```text
Guitar audio / clock / future Contrapunk state
                ↓
          Golem drummer brain
                ↓
          internal DrumHit events
                ↓
       sampler / drum synth engine
                ↓
          stereo drum audio
```

The goal is a session-player feel: the drums keep playing in the background while the guitarist plays. Guitar input changes the drummer's energy, accents, density, and fills, but it should not directly trigger every drum hit.

## UI direction

Golem uses a Svelte/Tauri UI from day one.

Rationale:

- The product is musician-facing and visual: meters, groove lanes, energy controls, kit mixer, and follow sensitivity should be fast to iterate.
- Svelte gives us richer interaction and visual design than egui for this kind of instrument.
- Rust still owns all realtime audio. The web UI is only a control/meter layer.

Boundary:

```text
Svelte UI
  start/stop, style, drummer pad, meters, params
        ↓ Tauri commands/events
Rust host
  cpal input/output, thread ownership, device selection
        ↓
golem-core
  realtime drummer + drum engine
```

## Crate/app layout

Initial scaffold:

```text
crates/golem-core/
  audio-safe drummer brain and procedural drum engine

apps/golem/
  Svelte/Tauri standalone app
  cpal input/output host
```

Future additions:

```text
crates/golem-preset/
crates/golem-kit/
crates/golem-plugin/
src/chain/golem_block.rs        # Contrapunk integration later
```

## MVP: Golem Jam v0.1

A single-screen standalone app:

- choose guitar input device
- start/stop drummer
- set BPM
- select style: rock, half-time, four-on-the-floor
- use a 2D drummer pad:
  - x-axis: simple → complex
  - y-axis: soft → loud
- set swing, fill amount, follow amount, master volume
- see live guitar level / onset / density meters
- hear procedural kick/snare/hat/tom/crash drums

Acceptance criteria:

1. Golem runs as a separate Tauri app.
2. It outputs drum audio without MIDI.
3. It keeps stable tempo from an internal sample clock.
4. It listens to guitar input RMS/onsets.
5. Louder/denser guitar playing increases drum energy/density.
6. It keeps playing when guitar stops.
7. No allocation or blocking in the output callback.

## Core API

`golem-core` exposes a transport/follow driven engine:

```rust
pub struct ClockSnapshot {
    pub sample_pos: u64,
    pub sample_rate: u32,
    pub bpm: f64,
    pub playing: bool,
}

pub struct FollowInput {
    pub guitar_rms: f32,
    pub onset_strength: f32,
    pub strum_density: f32,
    pub confidence: f32,
}

pub struct Engine;

impl Engine {
    pub fn prepare(&mut self, sample_rate: u32, max_block: usize);
    pub fn process(
        &mut self,
        clock: ClockSnapshot,
        follow: FollowInput,
        output: &mut [f32],
        channels: usize,
    );
}
```

`process` adds drum audio into the provided buffer. Standalone hosts clear the output buffer first; future Contrapunk integration can mix Golem in parallel before master FX.

## Internal event model

Golem has typed drum hits, not MIDI events:

```rust
pub struct DrumHit {
    pub piece: DrumPiece,
    pub articulation: Articulation,
    pub velocity: f32,
    pub offset_frames: u32,
}
```

These events are generated from groove/transport/follow state and delivered directly to the drum engine.

## Sound engine path

v0.1 uses procedural drums so the app is immediately runnable without sample assets:

- kick: pitch-dropping sine + click
- snare: filtered noise + body tone
- hats: high-passed metallic noise
- toms: damped resonator
- crash: noisy cymbal tail

Next step is a sampler with:

- preloaded sample buffers
- velocity layers
- round robin
- choke groups
- fixed voice pool
- per-piece pan/gain

## Contrapunk integration later

Once Golem standalone feels good, add an optional root feature:

```toml
golem-drums = ["dep:golem-core"]
```

Then create:

```text
src/chain/golem_block.rs
```

`GolemBlock` will implement Contrapunk's `AudioBlock`, share the app transport, read guitar/follow features, and mix Golem audio into the chain before master FX. No MIDI-router changes should be required.
