# Synthesis: Preset 27 — Quartal Colossus

**Decision:** `ready_for_implementation`

**Reference scope:** one fourth-rich modal/open-voicing property documented in selected McCoy Tyner performances from the 1960–65 John Coltrane Quartet and Tyner’s independent consolidation through *The Real McCoy* (1967)

**Operational claim:** each clean, non-overlapping in-collection source note selects one deterministic four-voice Dorian fourth-derived block, with the exact source once at the bottom; the player supplies rhythm, accents, register, repetition, development, dynamics, resolution, and silence

## 1. Bounded historical scope

The defensible center is the 1960–67 continuum: selected classic-quartet modal contexts, Tyner’s early leader work, and the contrasting forms on *The Real McCoy*. This supports open fourth-rich sonority, tonal/modal anchoring, forceful but dynamically controlled attacks, repetition, resolution, and space.

The preset does not represent Tyner’s whole career or complete language. The reports agree that his practice also includes bebop syntax, blues, standards, dominant and functional motion, pedals, bass movement, independent hand roles, acoustic-piano touch, formal development, and ensemble listening. Coltrane, Elvin Jones, Jimmy Garrison, and later collaborators remain co-creators of the ensemble evidence.

## 2. Report agreement and parent resolution

All three reports agree on these points:

- fourth-rich modal color is relevant but “Tyner equals stacked fourths and pentatonics” is a caricature;
- a credible result depends on player-authored attacks, accents, density, register, repetition, and silence;
- the current engine can provide a static local harmony mapping, not comping decisions, phrase development, independent bass, adaptive density, or ensemble interaction;
- Dorian is the correct current scale container; `MinorPentatonic + DiatonicFourths` does not preserve the intended fourth relation;
- input must be clean, monophonic, non-overlapping, and safely inside the MIDI range;
- artist imitation and named-recording reconstruction claims are disallowed.

The history report recommends a larger `ANCHOR → PUNCTUATE → INTENSIFY → LAY_OUT → RETURN` behavior. That is historically stronger but unavailable without activity tracking, phrase/section state, density control, and an autonomous layer. The parent decision does not fake it. The performer enacts that arc while the shared HarmonyEngine remains stateless.

## 3. Exact implementable mapping

Use the existing shared HarmonyEngine only:

```json
{
  "harmony": {
    "scaleMode": "Dorian",
    "mode": "DiatonicFourths",
    "voiceCount": 4,
    "voicePosition": 3,
    "voiceLeadingEnabled": false,
    "voiceLeadingStyle": "Free",
    "octaveMode": "None",
    "octaveIntensity": 1,
    "interchangeEnabled": false,
    "interchangeRange": 3,
    "counterpointSpecies": "Species1",
    "counterpointStrictness": "Strict"
  },
  "companion": {
    "enabled": false,
    "globalHoldMode": { "kind": "cancel" },
    "canon": { "enabled": false, "form": "free_imitation", "holdMode": null, "voices": [] },
    "counterpoint": {
      "enabled": false,
      "species": "Species1",
      "transposeDegrees": 2,
      "preferAbove": true,
      "holdMode": null
    }
  },
  "mix": { "input": 1, "harmony": 1, "canon": 1, "counterpoint": 1 }
}
```

**Requirement:** `['harmony']`. Transport is optional and must remain untouched.

For Dorian scale index `d`, the engine chains `+3` scale indices to produce `[d, d+3, d+6, d+9]`. With the source at arrangement slot 3, it is the lowest note and occurs exactly once.

| Source degree | Relative semitones | Public interpretation |
|---|---:|---|
| `1` | `[0,5,10,15]` | exact perfect-fourth stack |
| `2` | `[0,5,10,15]` | exact perfect-fourth stack |
| `♭3` | `[0,6,11,16]` | diatonic fourth-derived variant with one augmented fourth |
| `4` | `[0,5,10,16]` | diatonic fourth-derived variant with one augmented fourth |
| `5` | `[0,5,10,15]` | exact perfect-fourth stack |
| `6` | `[0,5,10,15]` | exact perfect-fourth stack |
| `♭7` | `[0,5,11,16]` | diatonic fourth-derived variant with one augmented fourth |

Product copy must say “Dorian fourth-derived blocks.” It may identify degrees `1, 2, 5, 6` as the exact all-perfect-fourth subset. It must not say every Dorian note creates the same perfect-fourth stack.

## 4. Performer contract

- Play one physical note at a time in the displayed Dorian collection.
- Use a two-to-four-note cell, preferably degrees `1, 2, 5, 6` for exact perfect-fourth blocks.
- Repeat its rhythm once and alter one ending, accent, or octave.
- Use firm, dry attacks at roughly 88–132 BPM; transport/metronome is optional.
- Release each note before the next attack. Keyboard pedal stays up; guitar players mute unused strings and avoid bends, slides, double-stops, and ringing releases.
- Keep the source in a middle register so all four voices remain in range and distinct.
- Leave at least one beat between clauses and one full bar after the phrase.
- Create the larger arc manually: sparse statement, repeated build, one controlled peak, withdrawal, silence.

A chord, sustained pedal, legato overlap, same-pitch retrigger before release, low-register stream, or chromatic run is outside acceptance. Current `harmonize_smart` may process chromatic notes; the preset does not reject them and makes no stylistic claim about that fallback.

## 5. Honest public copy

Use copy equivalent to:

- **Name:** Quartal Colossus
- **Result:** Each in-collection note becomes one four-voice Dorian fourth-derived block above the exact source; degrees 1, 2, 5, and 6 form exact perfect-fourth stacks.
- **Play:** Punch a two-to-four-note cell from degrees 1, 2, 5, and 6, repeat its rhythm with one stronger accent or octave shift, then release and leave a full bar of air.
- **Approximation:** A fixed harmony study inspired by one open fourth-rich feature documented in selected 1960–67 McCoy Tyner performances. It does not generate Tyner’s phrases, rhythm, touch, pedals, bass movement, dominant substitutions, chromatic resolutions, comping decisions, ensemble interaction, or formal development.

Do not use “authentic Tyner voicings,” “play like McCoy Tyner,” “Tyner in a box,” “recreates the John Coltrane Quartet,” “signature scale,” “automatic comping,” or “African rhythm.”

## 6. Explicitly out of scope

The current preset does not provide:

- autonomous chordal punctuation, swing, cross-rhythm, accents, or patterns;
- phrase recognition, motif development, activity-responsive intensity, lay-out, or automatic return;
- an independent pedal, bass line, left-hand role, melody layer, or hand allocation;
- chord-chart awareness, dominant substitutions, contextual chromatic motion, cadence planning, or resolution;
- acoustic-piano touch, pedaling, voicing under the hand, microtiming, dynamics beyond input propagation, or ensemble listening;
- a source-degree mask, out-of-scale rejection policy, or register enforcement;
- Tyner’s identity, complete harmonic language, cultural experience, spirituality, or any named performance.

Do not add preset-specific algorithms for these omissions. Future upgrades must be reusable shared capabilities.

## 7. Implementation acceptance checks

1. Catalog validation accepts the approved preset with only `harmony` required and `transportRequired: false`.
2. Applying it preserves tonic, BPM, transport, devices, routing, sound, master, mute/solo, and plugins.
3. All seven Dorian degrees produce exactly the documented relative vectors with four total notes and the exact source once at the bottom.
4. Each accepted NoteOff returns the exact NoteOn pitch set and leaves HarmonyEngine active state empty.
5. A focused regression documents chromatic input as an outside-corpus fallback rather than rejection.
6. Companion remains disabled; no generated attacks occur during rests.
7. UI copy names the all-perfect-fourth subset and does not promise a universal identical stack.
8. Public text contains no claim of pattern generation, independent bass, phrase/section inference, contextual comping, or artist simulation.
9. Shared harmony, catalog/persistence, and UI checks pass with no new warnings.

The acceptance corpus deliberately excludes overlap, sustain, ringing strings, and same-pitch retrigger because HarmonyEngine ownership is keyed by source MIDI pitch rather than note instance. Existing surface-level panic/reharmonization cleanup remains mandatory.

## 8. Evidence trail

This synthesis depends on the three independent cited reports in this directory:

- `history.md` for corpus, career evolution, ensemble attribution, historical invariants, and caricature risk;
- `theory.md` for exact Dorian vectors, current-engine audit, lifecycle boundary, and minimal mapping;
- `performance.md` for guitar/keyboard input constraints, density, articulation, silence, failure gestures, lifecycle, and the acceptance exercise.

Where the history report asks for adaptive temporal behavior beyond current capabilities, this synthesis adopts the smaller static mapping recommended by theory and performance and assigns the temporal arc to the player.
