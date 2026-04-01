# MPE & MIDI Guitar 3 — Deep Analysis from Video Transcript

Source: Video explaining MIDI Guitar 3's MPE implementation for guitarists.

## Key Concepts

### MPE (MIDI Polyphonic Expression)
- Each note gets its own MIDI channel (vs traditional MIDI where one channel = whole instrument)
- For guitar: channel 1 = master, channels 2-7 = strings 1-6
- Enables per-note pitch bend, pressure, CC74, strike velocity, and release velocity
- Same MIDI messages as before, just distributed across channels

### The 5 Dimensions of Expression (Roli Seaboard → Guitar)

| Dimension | Seaboard Gesture | Guitar Gesture | MIDI Message |
|-----------|-----------------|----------------|-------------|
| **Strike** | Key velocity | Pluck/pick attack strength | Note On velocity (per channel) |
| **Glide** | Horizontal finger movement | String bending | Pitch Bend 14-bit (per channel) |
| **Slide** | Vertical finger movement | Position on neck (brightness) | CC74 (per channel) |
| **Press** | Aftertouch pressure | String amplitude envelope over time | Channel Pressure (per channel) |
| **Lift** | Release speed | How fast the note dies / is muted | Note Off velocity (per channel) |

### How MIDI Guitar 3 Extracts These from Audio

**Strike (Note On Velocity):**
- Analyzes transients in incoming audio
- Identifies pitch + highest amplitude for each detected note
- Hardness of pick/pluck = higher transient = higher velocity
- Pick vs finger gives different transient shapes
- Can be adjusted with Strike button per instrument in MG3
- Can be modulated

**Pressure (Channel Pressure / Aftertouch):**
- Based on incoming audio amplitude over time (the string envelope)
- As string rings out → energy tapers off → pressure decreases
- NOT influenced by physically pressing harder on the fretboard
- Highly customizable via MG3's modulators/envelopes:
  - Invert min/max handles → pressure increases as string dies
  - Connect expression pedal or breath controller to handles
  - Dynamics module: compressor, hold pressure, tone shift
- Hold Pressure function: slows the taper, good for wind instrument sounds

**CC74 (Brightness / Slide):**
- Based on audio brightness analysis (likely spectral centroid or HF energy)
- Lower on neck = brighter, higher on neck = less bright
- Physical phenomenon from guitar construction, not pitch-related
- Harmonics register as very high brightness
- Palm mutes and pickup selection also affect it
- Great for mapping synth parameters to neck position
- Can be augmented with expression pedal/breath controller

**Glide (Pitch Bend):**
- Per-note pitch bend in 14-bit resolution (0-16383, center 8192)
- Extracted from the audio pitch tracking
- Pitch bend data arrives BEFORE the Note On message
- All guitar pitch acrobatics (bends, slides, vibrato) translate directly
- Master bend available on master channel (affects all notes like a pitch wheel)

**Lift (Release Velocity / Note Off Velocity):**
- Note Off velocity = last sent pressure value before note ends
- Early stop = high value, long ring = low value
- Recently implemented in MG3
- Can trigger completely new sounds (e.g., breath, pluck on release)
- Highly recommended to use modulators to control this
- Can be connected to synth release triggers

### Critical Insight: Direct vs Modulator Control

**Direct control (expression pedal → pressure button on synth):**
- OVERRIDES the polyonic pressure from the tracker
- All notes get the same value = mono, blocky, loses MPE character
- Like going back to mono aftertouch

**Modulator control (expression pedal → envelope handle):**
- MODIFIES the polyonic pressure from the tracker
- Maintains per-note character while adding human control
- Much more musical result

This is the key architectural insight for Contrapunk: don't replace DSP-extracted values with external controls — modulate them.

## What Contrapunk Should Implement

### Already Done
- Strike: velocity from attack RMS ✓ (now with calibrated power curve + attack peak)
- Glide: per-note pitch bend 14-bit ✓
- Per-string channels ✓
- Note Off velocity ✓

### Needs Implementation
1. **Pressure (Channel Pressure):** amplitude envelope over time during Sustain → Channel Pressure CC. We have basic aftertouch but it should follow the string energy profile more naturally.

2. **CC74 (Brightness):** spectral centroid or HF energy analysis per note → CC74 per channel. This maps neck position to synth parameters. We have Goertzel harmonic analysis — spectral centroid is computable from the same data.

3. **Lift refinement:** Note Off velocity should be the last pressure value before note ends, not just the decay RMS. This is a subtle but important difference.

4. **Dynamics module equivalent:** Compressor, hold pressure, tone shift for the pressure envelope. This makes the guitar work with sustaining synth sounds.

5. **Modulator architecture:** Don't just output raw DSP values. Allow envelopes/modulators between the DSP extraction and the MIDI output. This is MG3's key innovation.

6. **Master channel:** Channel 1 for global controls (master pitch bend, program change), channels 2-7 for per-string voices.

### MIDI Message Priority Order
From the transcript, MIDI Guitar 3 sends messages in this order for each note:
1. Pitch bend value (arrives BEFORE note on)
2. Note On with velocity
3. Ongoing: pressure, CC74, pitch bend updates
4. Note Off with release velocity

This ordering matters — the pitch bend must be set before the Note On fires, otherwise the synth plays the note at the wrong pitch for one frame.

## Architecture Implications

The modulator/envelope system in MG3 is essentially a signal processing chain:

```
Raw DSP extraction (amplitude, pitch, brightness)
  → Per-note envelope shaping (attack, sustain, release curves)
  → Optional modulator (LFO, expression pedal, breath controller)
  → Clamp to MIDI range
  → Send as MPE message on appropriate channel
```

For Contrapunk, this means the `GuitarInput` pipeline should output raw extracted values, and a separate `MpeProcessor` or `ExpressionMapper` layer should shape them before they become MIDI messages. This keeps the DSP clean and the expression customizable.
