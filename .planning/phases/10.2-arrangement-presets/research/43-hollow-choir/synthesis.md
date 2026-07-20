# Synthesis: Preset 43 — Hollow Choir

**Decision:** `ready_for_implementation`

**Reference scope:** selected sparse-to-climactic area and narrative scoring from Christopher Larkin’s 2014–18 *Hollow Knight* project, bounded especially by the intimate piano/viola language, Amelia Jones’s credited solo soprano on “City of Tears,” Gothic organ color, and the game-scale movement from restraint toward orchestral intensity

**Operational claim:** a clean monophonic Aeolian line receives one reactive four-note SATB-style minor harmonic shadow; the player supplies melody, phrase timing, held destinations, dynamics, register, development, restraint, and silence

## 1. Historical boundary

The selected project supports dark elegance, melancholy, simple recurring melodic identities, area-specific instrumental color, sparse/full layer contrast, reverberant space, and reserved orchestral or operatic escalation. It does not support “choir everywhere.” Official evidence identifies one solo soprano on “City of Tears”; the original score was otherwise mostly sample-based apart from documented viola and voice recordings.

The title **Hollow Choir** remains imagery. It must not be presented as a literal live choir, Amelia Jones vocal, Christopher Larkin style model, *Hollow Knight* cue reconstruction, or universal account of Larkin’s career.

## 2. Report agreement and parent resolution

All reports agree that:

- minor/modal gravity, consonance-led voicing, slow movement, wide air, held arrivals, and consequential silence are defensible;
- the current engine can provide a reactive minor chorale, not a choir timbre or authored orchestration;
- a real counterline requires distinct rhythm/contour and is not present in simultaneous chord members;
- “distance” requires level, envelope, filtering, reverb, and spatial production rather than pitch placement alone;
- adaptive area, exploration, combat, narrative, sparse/full stem, and game-state behavior are unavailable;
- the performer must author every phrase and larger dynamic arc.

The history report’s preferred `sparse → full → sparse` scene arc is historically stronger than a static mapping but requires adaptive-scene, stable-group, sound-role, and phrase capabilities. The parent does not fake it. The current preset narrows the result to one restrained SATB-style harmonic shadow at each source onset.

## 3. Exact current mapping

Use the shared HarmonyEngine only:

```json
{
  "harmony": {
    "scaleMode": "Aeolian",
    "mode": "BachChorale",
    "voiceCount": 4,
    "voicePosition": 0,
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

**Requirement:** `['harmony']`. Transport is optional and preserved.

`BachChorale` supplies its existing four-voice SATB allocator with common-tone, stepwise-motion, spacing, leading-tone, and parallel-perfect constraints plus its documented relaxation cascade. It remains reactive per source onset; harmonic inertia is note-count-based, not transport- or phrase-aware.

With fresh C Aeolian state and C5 input, the current exact oracle is `[C5, E♭4, G3, B♭2]` / MIDI `[72,63,55,46]`. The exact source occurs once as soprano. NoteOff must return that same set and empty active state. A following D5 nonchord color before the harmonic governor allows change currently yields `[74,60,51,43]`; this pins harmonic inertia without claiming that either array reproduces the reference score.

## 4. Performer contract

- Play one clean note at a time from the displayed Aeolian collection.
- Use a two-to-five-note singable phrase, mostly steps and small thirds, with at most one expressive fourth or fifth.
- Use soft, rounded, non-overlapping attacks around 56–76 BPM; transport/metronome is optional.
- Hold the destination for three to four beats, release completely, then leave at least one full bar silent.
- Repeat the contour once with exactly one changed note, crest octave, ending, or velocity arc.
- Guitar: use dry clean input, tune first, mute unused strings, and avoid bends, slides, harmonics, wide vibrato, double-stops, and ringing releases.
- Keyboard: one key at a time, sustain pedal up, no arpeggiator, chord mode, pitch bend, or overlapping keys.
- Keep the source in a comfortable vocal middle register; choose sound and reverb separately.

The player creates `call → settle → one variation/crest → withdrawal`. The engine does not infer those states.

## 5. Honest public copy

Use copy equivalent to:

- **Name:** Hollow Choir
- **Result:** A singable Aeolian line receives a restrained four-part, SATB-style minor harmonic shadow.
- **Play:** Play one soft two-to-five-note minor phrase, hold the destination, release completely, then leave a full bar of silence before one small variation.
- **Approximation:** A static harmony study informed by sparse, melancholic vocal/orchestral atmosphere in a bounded 2014–18 *Hollow Knight* project corpus. It does not generate a literal choir, acoustic orchestration, distant layers, independent counterline, adaptive game scenes, ambience, reverb, narrative response, protected cue material, or Christopher Larkin’s identity.

Sound design is separate. Do not say “Hollow Knight preset,” “Christopher Larkin preset,” “live choir,” “Amelia Jones vocal,” “authentic soundtrack harmony,” “Hallownest,” or name any cue/location/character in public preset copy.

## 6. Explicitly out of scope

The current preset does not provide:

- gradual sparse-to-full expansion or automatic thinning;
- independent vocal, counterline, piano, viola, organ, string, brass, or harp roles;
- stable distant register/mix groups, envelopes, filtering, reverb, environmental acoustics, or spatial depth;
- phrase recognition, leitmotif memory, density/intensity sensing, scene changes, location/combat/narrative response, or authored stem transitions;
- exact score harmony, orchestration, form, melody, samples, recordings, or copyrighted game assets;
- artist identity, endorsement, or a whole-career style model.

Future upgrades must use reusable shared stable-group/adaptive-scene/sound-role capabilities, never preset-specific branches.

## 7. Implementation acceptance checks

1. Catalog validation accepts the narrowed approved preset with only `harmony` required and `transportRequired: false`.
2. Applying it preserves tonic, BPM, transport, devices, routing, sound, master, mute/solo, and plugins.
3. Fresh C Aeolian C5 NoteOn returns exactly `[72,63,55,46]`, source once as soprano; NoteOff returns the same identities and empties active state.
4. The documented D5 continuation pins note-count harmonic inertia and releases its exact set.
5. A short newly invented Aeolian phrase produces four total notes per accepted onset, matching NoteOff ownership, and no output during rests.
6. Companion remains disabled; no independent counterline or delayed event is emitted.
7. Stop/panic/preset-change contracts leave no active or pending notes through existing surface cleanup.
8. UI copy says SATB-style harmonic shadow, not literal choir, distance, adaptive expansion, counterline, artist imitation, or soundtrack reproduction.
9. Shared harmony, catalog/persistence, and UI checks pass with no new warnings.

## 8. Evidence trail

This synthesis depends on the three independent cited reports in this directory:

- `history.md` for the project scope, production/collaborator evidence, solo-soprano correction, area/stem behavior, game-scale evolution, attribution, and copyright boundaries;
- `theory.md` for minor/modal invariants, BachChorale code audit, exact current pitch oracles, harmonic inertia, and unsupported capability matrix;
- `performance.md` for the phrase contract, guitar/keyboard constraints, dynamics, silence, failure gestures, lifecycle expectations, and 45-second acceptance exercise.

Where research describes the richer sparse-to-climactic source trajectory, this synthesis assigns the arc to the performer and keeps software behavior to the smaller static mapping the current shared engine can prove.
