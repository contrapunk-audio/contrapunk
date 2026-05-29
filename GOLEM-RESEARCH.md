# Golem Research Notes

Golem is a proposed standalone drum-player / drum-sound engine for the Contrapunk workspace. It should be a separate crate like Elixir, not a MIDI feature: it listens to musical context and produces audio through its own sampler/synthesis engine. Contrapunk can host it later as an audio block.

## Product references

### Logic Pro Drummer / Session Players

Apple's Session Players are the closest product-level reference. The useful ideas for Golem are:

- a player/performance abstraction, not just a pattern sequencer
- controls for complexity, intensity, fills, swing, and pattern selection
- follow behavior: place rhythmic emphasis on chord changes and/or follow the rhythm of another track
- region/section awareness so grooves and fills change across an arrangement

Source: https://support.apple.com/en-gb/guide/logicpro/lgcpca8dca3d/12.2/mac/15.6

**Golem takeaway:** Golem should keep an independent groove running from transport time, then adapt accents/fills/energy from guitar and Contrapunk state. Guitar should influence the drummer; it should not directly trigger every drum hit.

### Toontrack EZdrummer 3

EZdrummer 3 is useful for songwriter workflow references:

- Bandmate: find grooves from a guitar/rhythm idea
- Tap2Find: search grooves from a tapped rhythm
- Song Creator / grid editor / play-style editing
- built-in kits and mix-ready presets

Source: https://www.toontrack.com/product/ezdrummer-3/

**Golem takeaway:** A future Golem can support “find/derive a groove from my guitar part,” but v1 should start with deterministic adaptive grooves before adding ML or search.

### Addictive Drums, UJAM Virtual Drummer, MDrummer, MODO DRUM

These are useful for non-MVP feature vocabulary:

- multisampled acoustic kits
- per-piece mixer and effects
- groove/phrase libraries
- performance transform controls
- multi-output mixing
- physical/modal modeling mixed with samples

References:

- https://www.xlnaudio.com/products/addictive_drums_2
- https://www.ujam.com/drummer/
- https://www.meldaproduction.com/MDrummer
- https://www.ikmultimedia.com/products/mododrum/

**Golem takeaway:** v1 should not chase full commercial-drummer scope. The core product loop is more important: play guitar while Golem keeps a believable adaptive drummer alive.

## Open-source references

### DrumGizmo

DrumGizmo is an open-source, multichannel, multilayered drum sampler. Useful concepts:

- open drum-kit file format
- instrument groups
- velocity layers
- multichannel/mic-style mixing
- humanization
- kit files separate from mapping files

Source: https://www.drumgizmo.org/wiki/doku.php?id=documentation%3Afile_formats

**Golem takeaway:** The Golem sampler should eventually separate `Kit`, `Instrument`, `SampleRegion`, and `MixerBus`. Even if v1 ships a tiny built-in kit, do not hard-code a one-sample-per-drum model.

### SFZ drum mapping

SFZ is a good reference for sample-region semantics:

- velocity ranges: `lovel` / `hivel`
- round robin: `seq_length` / `seq_position`
- exclusive/choke groups
- one-shot sample playback
- per-region pitch, pan, volume, envelope

Source: https://sfzformat.com/tutorials/drum_basics/

**Golem takeaway:** Use a small native subset inspired by SFZ rather than implementing full SFZ initially.

### Hydrogen

Hydrogen is a pattern-based drum machine with sample playback, swing, velocity/timing humanization, and kit import.

Source: https://github.com/hydrogen-music/hydrogen

**Golem takeaway:** For v1, pattern sequencing plus humanization is a proven base. The differentiator is live adaptation, not having the most complex sequencer.

## Research papers and systems

### Real-Time Drum Accompaniment Systems — RT-DR-AC

A broad dissertation/system reference for real-time drum accompaniment, including groove models and live improvisation contexts.

Source: https://rtdrac.github.io/

**Use for:** architecture, real-time constraints, groove model framing, evaluation ideas.

### DeepDrum: Adaptive Conditional Neural Network for Drum Rhythms

Conditions generated drum rhythms on musical parameters and accompanying instruments such as bass/guitar.

Source: http://arxiv.org/pdf/1809.06127

**Use for:** future ML drummer-brain experiments. Not needed in the audio callback.

### Generating Coherent Drum Accompaniment with Fills and Improvisations

Focuses on coherent drum accompaniment with repeated patterns plus fills/improvisations.

Source: https://ar5iv.labs.arxiv.org/html/2209.00291

**Use for:** phrase-level structure: grooves should repeat, but fills should appear at musically meaningful boundaries.

### Conditional Drums Generation using Compound Word Representations

A symbolic drum-generation paper using compound token/event representations.

Source: https://ar5iv.labs.arxiv.org/html/2202.04464

**Use for:** internal event representation ideas. Golem should not emit MIDI, but it still needs typed drum events.

### CycleDRUMS

Audio-domain drum arrangement conditioned on a bass line using CycleGAN.

Source: https://arxiv.org/abs/2104.00353

**Use for:** long-term “follow an audio stem” ideas. It supports the idea that instrument audio can condition drum accompaniment, but it is not a v1 realtime engine design.

### JukeDrummer

Conditional beat-aware audio-domain drum accompaniment generation using Transformer VQ-VAE.

Source: https://arxiv.org/abs/2210.06007

**Use for:** long-term audio-domain accompaniment generation. Relevant because it explicitly uses beat awareness.

### DARC

Drum accompaniment generation with fine-grained rhythm control, conditioning on musical context and explicit rhythm prompts like tapping/beatboxing.

Source: https://arxiv.org/abs/2601.02357

**Use for:** future “play/tap/strum a rhythm and make the drummer follow it” workflows.

### Real-time human-AI co-performance with latent diffusion + MAX/MSP

A modern reference for hybrid realtime systems: realtime frontend + model server communicating over OSC/UDP.

Source: https://arxiv.org/abs/2604.07612

**Use for:** if Golem ever adds heavyweight generative models, keep them out-of-process / off the audio thread.

## Beat/onset/following references

### aubio

A C library for onset detection, pitch detection, beat tracking, and tempo tracking.

Source: https://aubio.org/

**Use for:** algorithms and vocabulary. Contrapunk already has guitar onset/pitch infrastructure, so Golem may consume existing features rather than link aubio.

### madmom

Python MIR library with onset, beat, downbeat, tempo, and related algorithms; some scripts support online mode.

Source: https://madmom.readthedocs.io/

**Use for:** offline experiments and model evaluation, not production Rust audio thread.

### BEATNET / BEAST / online beat tracking

Online beat/downbeat tracking research is useful if Golem needs to infer tempo from live guitar without Contrapunk transport.

References:

- BEATNET: https://arxiv.org/pdf/2108.03576
- BEAST: https://arxiv.org/html/2312.17156v3

**Golem takeaway:** v1 should prefer explicit transport from Contrapunk or Golem standalone tempo. Online beat tracking is useful later, but it adds latency and uncertainty.

## Drum sound synthesis references

### Multisample sampler

This should be the v1 foundation.

Required features:

- preloaded sample buffers
- velocity layers
- round robin
- one-shot playback
- fixed voice pool
- per-piece gain/pan
- choke groups, especially hats
- no allocation in `process`
- sample loading outside the audio callback

Why: it gives the most convincing acoustic drummer sound with the least research risk.

### Procedural percussion synthesis

Useful for placeholders, electronic kits, and hybrid transient/body layers.

Basic models:

- kick: sine oscillator + fast pitch envelope + transient click
- snare: filtered noise + resonant body modes
- hi-hat/cymbal: filtered noise + inharmonic metallic oscillators
- tom: damped resonant oscillator/modal bank
- clap/percussion: clustered noise bursts + short ambience

References:

- https://cim.mcgill.ca/~clark/nordmodularbook/nm_percussion.html
- http://www.sospubs.co.uk/techniques/synthesizing-percussion

### Modal / physical modeling

Useful long-term for tunable drums: shell size, head tension, damping, material, strike position.

Commercial reference: MODO DRUM combines modal synthesis and sampling.

Source: https://www.ikmultimedia.com/products/mododrum/

**Golem takeaway:** interesting for v2+, but v1 should avoid making physical modeling the core dependency.

### Neural drum sound generation

Relevant papers:

- Neural Drum Machine: https://ar5iv.labs.arxiv.org/html/1907.02637
- CRASH: https://ar5iv.labs.arxiv.org/html/2106.07431
- DrumGAN VST: https://ar5iv.labs.arxiv.org/html/2206.14723
- Differentiable Percussive Audio / Drumblender: https://arxiv.org/abs/2309.06649

**Golem takeaway:** neural synthesis is best used offline for kit generation, resynthesis, and sound-design tools. Do not put diffusion/GAN inference on the realtime callback path.

## Recommended Golem architecture from research

```text
Clock / transport
      +
Guitar-follow features
      +
Contrapunk musical state later
      ↓
Golem drummer brain
      ↓
Internal DrumHit events
      ↓
Sampler / hybrid drum engine
      ↓
Stereo audio output
```

### Internal event model

Golem should have internal events even though it does not use MIDI:

```rust
pub struct DrumHit {
    pub piece: DrumPiece,
    pub articulation: Articulation,
    pub velocity: f32,
    pub offset_frames: u32,
}

pub enum DrumPiece {
    Kick,
    Snare,
    ClosedHat,
    OpenHat,
    Ride,
    Crash,
    TomLow,
    TomMid,
    TomHigh,
}
```

### Timing model

Golem should be transport-driven:

- groove pattern is scheduled from beat/bar phase
- swing offsets subdivision timing
- humanization applies small bounded frame offsets and velocity variation
- guitar/onset input changes density, accents, and fill choices
- it should continue playing when the guitarist stops, unless configured otherwise

### MVP decision

Start with:

```text
rule-based adaptive drummer brain
+ internal DrumHit scheduler
+ procedural placeholder kit
+ sampler-ready architecture
```

Then add real samples.

This avoids the two biggest risks:

1. depending on an ML model before the product loop is proven
2. blocking the audio thread with loading/allocation/inference

## Open questions

1. Should the first audible v0 use purely procedural drums, embedded tiny WAVs, or both?
2. Should Golem v1 ship with an open drum-kit format, or hard-code one built-in kit and add kit loading later?
3. Should guitar-follow input come from Contrapunk's existing guitar DSP first, or should Golem standalone include its own onset follower from day one?
4. Should the first Contrapunk integration mix Golem after the synth and before FX, or expose it as a parallel source bus mixed before master FX?

## Suggested next documents

1. `GOLEM-DESIGN.md` — product/architecture design
2. `GOLEM-PLAN.md` — phased implementation plan
3. `crates/golem-core/` — initial engine scaffold
