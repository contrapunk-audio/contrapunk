# Phase: MPE Modulator Architecture

## Problem

The current guitar input pipeline outputs raw DSP-extracted values directly as MIDI messages. MIDI Guitar 3's key innovation is a **modulator/envelope layer** between DSP extraction and MIDI output that shapes raw values into musical expression.

Without this layer:
- Pressure follows the natural string decay exactly (always a downward curve)
- CC74 brightness is a fixed function of neck position
- There's no way for the player to influence expression parameters with external controllers while preserving polyphonic character
- Connecting an expression pedal directly overrides per-note values (mono, blocky)

## What the Modulator Architecture Does

```
Raw DSP extraction (per note)
  │ amplitude, pitch, brightness, harmonics
  │
  ▼
Envelope Shaper (per note, per dimension)
  │ Attack/Decay/Sustain/Release curves
  │ Hold factor, compression, tone shift
  │
  ▼
Modulator (per dimension)
  │ LFO, expression pedal, breath controller
  │ Connected to envelope HANDLES, not raw values
  │ Preserves polyphonic character
  │
  ▼
Clamp to MIDI range (0-127 or 0-16383)
  │
  ▼
MPE MIDI output (per channel)
```

## Key Insight: Modulate Handles, Not Values

MIDI Guitar 3 distinction:
- **Direct control** (pedal → pressure value): replaces per-note data, mono result
- **Modulator control** (pedal → envelope handle): scales/shapes per-note data, preserves polyphony

Example: Pressure with expression pedal
- Direct: All notes get pedal value. Polyphonic character destroyed.
- Modulator: Pedal controls the "ceiling" of the pressure envelope. Each note still has its own shape, but the ceiling moves with the pedal. Polyphonic character preserved.

## Proposed Architecture for Contrapunk

### Core Types

```rust
/// A modulation source that provides a value 0.0-1.0
pub enum ModSource {
    /// Fixed value
    Constant(f32),
    /// From the DSP extraction (amplitude, brightness, etc.)
    Dsp(DspParam),
    /// External controller (expression pedal, breath, etc.)
    ExternalCC { controller: u8 },
    /// Internal LFO
    Lfo { rate_hz: f32, shape: LfoShape },
    /// Combination of two sources
    Multiply(Box<ModSource>, Box<ModSource>),
}

/// Parameters extracted by the DSP pipeline
pub enum DspParam {
    Amplitude,     // raw RMS
    Brightness,    // spectral centroid
    PitchDeviation,// cents from base
    AttackStrength,// onset transient
    HarmonicContent,// harmonic richness
}

/// Envelope with adjustable handles
pub struct Envelope {
    pub min: f32,        // 0.0-1.0, can be modulated
    pub max: f32,        // 0.0-1.0, can be modulated
    pub attack_ms: f32,
    pub decay_ms: f32,
    pub sustain: f32,    // 0.0-1.0
    pub release_ms: f32,
    pub hold: f32,       // 0.0-1.0, slows decay
    pub curve: f32,      // gamma for shape
}

/// Maps a DSP parameter through an envelope + modulator to a MIDI output
pub struct ExpressionMapping {
    pub source: DspParam,
    pub envelope: Envelope,
    pub min_mod: Option<ModSource>,  // modulates envelope.min
    pub max_mod: Option<ModSource>,  // modulates envelope.max
    pub output: MpeOutput,
}

pub enum MpeOutput {
    Pressure,      // Channel Pressure
    Brightness,    // CC74
    PitchBend,     // 14-bit pitch bend
    Strike,        // Note On velocity
    Lift,          // Note Off velocity
    CustomCC(u8),  // Any CC number
}
```

### Per-Note Expression State

Each active note tracks its own expression state:

```rust
pub struct NoteExpression {
    pub onset_time: Instant,
    pub onset_amplitude: f32,
    pub current_amplitude: f32,
    pub current_brightness: f32,
    pub current_pitch_cents: f32,
    pub pressure_envelope_state: EnvelopeState,
    pub brightness_envelope_state: EnvelopeState,
}
```

### Processing Flow

```rust
impl MpeProcessor {
    /// Process one frame of expression data for all active notes
    pub fn process(&mut self, notes: &mut [NoteExpression], external_ccs: &[u8; 128]) -> Vec<MidiEvent> {
        let mut events = vec![];

        for note in notes.iter_mut() {
            for mapping in &self.mappings {
                // 1. Get raw DSP value
                let raw = note.get_dsp_param(&mapping.source);

                // 2. Apply envelope (with modulated handles)
                let min = mapping.envelope.min
                    + mapping.min_mod.as_ref().map(|m| m.evaluate(external_ccs)).unwrap_or(0.0);
                let max = mapping.envelope.max
                    + mapping.max_mod.as_ref().map(|m| m.evaluate(external_ccs)).unwrap_or(0.0);

                let shaped = mapping.envelope.process(raw, min, max, &mut note.pressure_envelope_state);

                // 3. Clamp and emit
                let midi_value = (shaped * 127.0).clamp(0.0, 127.0) as u8;
                events.push(mapping.output.to_midi_event(note.channel, midi_value));
            }
        }

        events
    }
}
```

### Default Mappings (Guitar-Optimized)

```rust
fn default_guitar_mappings() -> Vec<ExpressionMapping> {
    vec![
        // Pressure: amplitude envelope with hold
        ExpressionMapping {
            source: DspParam::Amplitude,
            envelope: Envelope {
                min: 0.0, max: 1.0,
                attack_ms: 5.0, decay_ms: 500.0,
                sustain: 0.3, release_ms: 200.0,
                hold: 0.3, curve: 0.7,
            },
            min_mod: None,
            max_mod: None,  // Connect expression pedal here
            output: MpeOutput::Pressure,
        },
        // Brightness: spectral centroid, no envelope shaping
        ExpressionMapping {
            source: DspParam::Brightness,
            envelope: Envelope::passthrough(),  // no shaping
            min_mod: None,
            max_mod: None,
            output: MpeOutput::Brightness,
        },
    ]
}
```

## UI Integration

The modulator architecture needs UI controls:
- Per-instrument expression mapping editor
- Visual envelope editor (drag handles for min/max/ADSR)
- Modulation routing (connect source → handle)
- External CC assignment (which MIDI CC controls which handle)
- Preset system (save/load expression configurations)

This is a significant UI feature — similar to MG3's modulator panel.

## Implementation Plan

### Phase 1: Core Types + Default Processing
- Define `ExpressionMapping`, `Envelope`, `ModSource`, `MpeOutput`
- Implement `Envelope::process()` with ADSR + hold
- Default guitar mappings for pressure + brightness
- Wire into `GuitarInput` pipeline between DSP extraction and MIDI emission

### Phase 2: External Controller Input
- Accept external MIDI CC values (from expression pedal, breath controller)
- Route CCs to modulation handles
- Test with Audient iD14's expression input

### Phase 3: UI Editor
- Visual envelope editor in the SvelteKit UI
- Modulation routing matrix
- Preset save/load

### Phase 4: Advanced Modulators
- LFO sources
- Multiply/combine modulation sources
- Per-string expression configuration

## Dependencies
- Fixes 1-5 (MPE message ordering, CC74, pressure, etc.) must be done first
- Expression pedal/breath controller MIDI input needed for Phase 2
- UI framework changes for Phase 3
