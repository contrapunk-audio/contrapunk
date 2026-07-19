# Synthesis: Preset 12 — Planed Cathedral

**Decision:** `ready_for_implementation`

**Reference scope:** selected mature Debussy piano language, approximately 1903–1910

**Operational claim:** one static, three-note, whole-tone augmented chord plane whose phrase arc is shaped by the performer

## 1. Referenced scope

The bounded source corpus is Debussy's mature color-form period rather than his whole career:

- *Estampes*: “Pagodes” (1903), especially its collection-bound layers, pedals, register separation, and quiet coda;
- *Images* I: “Reflets dans l'eau” (1905), especially retained figures, changed bass meaning, and planed bridges;
- *Préludes* I: “Voiles” and “La cathédrale engloutie” (1910), with “Voiles” treated as an exceptional fixed whole-tone experiment and “La cathédrale” as evidence for a distant → emerged → withdrawn formal/register/dynamic arc;
- *Nuages* and *Faune* as earlier comparators for varied recurrence, orchestration, chromatic connection, and continuity.

The preset does not claim to reconstruct “La cathédrale engloutie.” The title is public imagery. Its actual technical reference is the exact parallel movement of a collection-bound sonority, one process among Debussy's pedals, open intervals, diatonic/modal and pentatonic fields, collection changes, common tones, register, resonance, and formal transformation.

The selected whole-tone realization is closer to the bounded fixed-collection procedure represented exceptionally by “Voiles” than to the complete pitch organization of “La cathédrale.” Public copy must state that limitation.

## 2. Style thesis

Each clean in-collection source note becomes a fixed three-note augmented sonority drawn entirely from one whole-tone collection. When the player moves by one collection degree, all three strata move by the same whole tone, preserving chord quality and inversion. The engine supplies only this static plane; the player supplies the Debussy-referenced temporal image by moving from low/quiet/sparse gestures through a higher/louder broad peak and back toward low/quiet gestures and silence.

```text
Mist
  low/middle register + soft velocity + long gaps
    ↓ 2–4 rising collection degrees, increasing velocity
Emergence
  connected but non-overlapping sustained planes
    ↓ one or two higher/stronger/longer attacks
Presence
  broad static plane at the player-shaped peak
    ↓ lower register + falling velocity + longer gaps
Echo / Submergence
  fewer planes, complete release, silence
```

No automatic state detector, crescendo, collection rotation, pedal, orchestration, cadence, or emergence/submergence engine is implied.

## 3. Research convergence and disagreement

All three independent reports agree that:

- “Debussy equals whole tone” is a caricature;
- parallel chord motion is a bounded foreground color rather than a universal texture;
- pitch collection, fixed contour, register, spacing, resonance, dynamics, and silence matter together;
- the broad phrase trajectory must be player-shaped because current harmony mapping is stateless per NoteOn;
- sparse sustained monophonic input is the safest and clearest control gesture;
- transport must remain optional and untouched;
- the first operational record must choose one collection and one exact plane vector rather than leave “whole-tone or pentatonic” ambiguous.

Role A prefers pentatonic/modal fields as more representative of much of the selected corpus and treats continuous whole-tone material as exceptional. Role B identifies the narrow reason to choose whole tone for v1: `DiatonicThirds` over a six-note whole-tone collection produces a deterministic augmented-triad interval vector `[0, 4, 8]` for every in-scale degree. Role C requires one published vector for observable acceptance.

Resolution: choose the exact whole-tone augmented plane because it is already deterministic and testable, but label it a selected coloristic procedure rather than “Debussy harmony” or a model of “La cathédrale.” Defer a separately labeled pentatonic diatonic-plane variant, collection transitions, and anchored/pedal textures.

## 4. Stylistic invariants

1. Every accepted source note belongs to the selected whole-tone collection rooted at the preserved tonic.
2. Exactly three pitches sound per source attack: input plus two generated voices.
3. With source as the lowest arrangement voice, the semitone vector is exactly `[0, 4, 8]` for all six in-collection degrees in the tested register.
4. Consecutive source moves translate all three strata by the same collection-degree displacement; chord quality and inversion remain fixed.
5. Voice leading is disabled so no common-practice revoicing breaks exact parallel motion.
6. Octave transformation is `None`; the engine does not independently spread or mirror voices.
7. Modal interchange is disabled. Out-of-collection input is outside the exact-plane acceptance contract and must not be advertised as exact planing.
8. Every plane starts and stops with its source ownership. Input silence remains generated silence.
9. No autonomous subdivisions, delayed attacks, pattern lane, Canon, or Counterpoint Lane is active.
10. Default input is one clean physical note at a time; played chords multiply density and violate the contract.
11. The engine does not infer phrase state. Register, velocity, duration, density, and silence from the player create the audible arc.
12. A phrase normally contains 3–6 planes per 15 seconds, one broad peak, then complete thinning and release.
13. Loading preserves tonic, BPM, meter, devices, guitar state, routing, sound, master, mute/solo, plugins, role mix, and transport running/position state.
14. Panic, Stop routing, normal NoteOff, preset replacement, and parameter change leave no active generated note.
15. Public copy distinguishes an exact whole-tone plane from Debussy's broader collection plurality and formal craft.

## 5. Variable parameters that preserve identity

- Either of the two whole-tone collections, selected by the user's preserved tonic.
- Any in-collection source register that leaves room for the two upper major-third strata.
- Slow to moderate unquantized pacing; approximately 48–66 BPM is performance guidance only.
- Source duration about 2–5 seconds, with one 5–7 second peak.
- Velocity arc, register arc, and 0.5–4 seconds of space.
- Transposition of the whole exercise.

Not variable in this first record:

- three total voices;
- source at voice position 2 (lowest of three);
- `[0, 4, 8]` exact interval vector;
- WholeTone collection;
- voice leading off;
- octave mode None;
- interchange off.

## 6. Rejected shortcuts and caricatures

Reject:

- marketing the preset as “La cathédrale engloutie,” “Debussy harmonization,” or Debussy's whole style;
- describing Debussy as permanently whole-tone, atonal, vague, formless, or merely “impressionist wash”;
- random chord shapes moved chromatically;
- claiming pentatonic/gamelan authenticity from equal-tempered scale choice;
- using `DiatonicFourths + WholeTone` as “quartal planing” (three collection degrees form a tritone in this scale);
- enabling Palestrina/Bach/Common-Practice voice-leading rules that undo parallel motion;
- Mirror or Spread as a fake cathedral peak;
- automatic modal interchange as collection evolution;
- a pattern lane or rapid subdivisions that fill the score's structural silence;
- functional leading-tone cadences as the source gesture;
- claiming automatic register, density, dynamic, pedal, resonance, or section evolution;
- sustaining old planes into new ones under normal monophonic acceptance.

## 7. Exact Contrapunk mapping

Role B incorrectly identified the older core `StylePreset` as the operational storage boundary. Phase 10.2 uses typed `ArrangementPresetV2`, and `ArrangementStore.applyUnchecked` already applies `voiceCount` and `voicePosition` transactionally. No shared capability or engine branch is missing for the bounded version.

```ts
{
  requirements: ['harmony'],
  config: {
    harmony: {
      scaleMode: 'WholeTone',
      mode: 'DiatonicThirds',
      voiceCount: 3,
      voicePosition: 2,
      voiceLeadingEnabled: false,
      voiceLeadingStyle: 'Free',
      octaveMode: 'None',
      octaveIntensity: 1,
      interchangeEnabled: false,
      interchangeRange: 3,
      counterpointSpecies: 'Species1',
      counterpointStrictness: 'Strict'
    },
    companion: {
      enabled: false,
      globalHoldMode: { kind: 'cancel' },
      canon: { enabled: false, form: 'free_imitation', holdMode: null, voices: [] },
      counterpoint: {
        enabled: false,
        species: 'Species1',
        transposeDegrees: 2,
        preferAbove: true,
        holdMode: null
      }
    },
    mix: { input: 1, harmony: 1, canon: 1, counterpoint: 1 }
  }
}
```

Why this works:

- `ScaleMode::WholeTone` contains semitone classes `[0, 2, 4, 6, 8, 10]`;
- `HarmonyMode::DiatonicThirds` advances two scale degrees for each chained voice;
- three voices with source at position 2 produce source, +2 degrees, +4 degrees: `[0, 4, 8]` semitones;
- voice-leading/interchange/octave processing remains off, preserving exact shape;
- `ArrangementPresetV2` owns these arrangement fields without owning environment state.

## 8. Approximation statement

A static whole-tone augmented chord plane inspired by one selected coloristic process in Debussy's mature piano writing. It does not reconstruct “La cathédrale engloutie,” model Debussy's pentatonic/diatonic/chromatic collection changes, preserve independent pedals or common tones, create open-fifth cathedral spacing, reproduce piano resonance/pedaling/orchestration, infer a phrase arc, generate cadences, or represent Debussy's whole career. The player supplies emergence and submergence through register, velocity, duration, density, and silence.

## 9. Performer contract

**Play it like:** Hold one soft note at a time. Rise and grow until one broad, bright peak, then fall away into longer silences; release every note cleanly before the next.

Expanded:

- use only notes from the displayed whole-tone collection;
- begin around C3–C5 equivalent; guitar prefers clean fretted middle strings around D3–B4;
- one physical note at a time, no sustain pedal, double-stop, ringing adjacent string, bend, slide, harmonic, or wide vibrato;
- sustain most planes for 2–5 seconds and leave 0.5–2 seconds between them;
- over 3–6 attacks, rise by collection step and velocity from approximately 35–50 toward one 80–100 peak;
- do not accelerate at the peak;
- descend, soften, and lengthen silence; finish with a full release and at least two seconds of silence;
- after every attack, wait until the plane is perceptible before deciding whether the next gesture should feel nearer or farther.

Transport is optional. Applying the preset must not start, stop, reset, retime, or remeter it.

## 10. Abstract acceptance examples

These are synthetic degree examples, not melodies.

### Exact plane table

With preserved tonic represented as MIDI 60 and source register 60–70:

| Whole-tone source | Expected MIDI plane |
|---|---|
| degree 1 / 60 | `[60, 64, 68]` |
| degree 2 / 62 | `[62, 66, 70]` |
| degree 3 / 64 | `[64, 68, 72]` |
| degree 4 / 66 | `[66, 70, 74]` |
| degree 5 / 68 | `[68, 72, 76]` |
| degree 6 / 70 | `[70, 74, 78]` |

For every row:

- three distinct pitches;
- pitch-class vector `[0, 4, 8]` relative to input;
- ordinary NoteOff returns exactly the same owned pitch set;
- active input and harmony state returns to empty.

### Short phrase

```text
beat/time 0: degree 1, soft, long → [1,3,5]
space
beat/time 1: degree 2, medium → [2,4,6]
space
beat/time 2: degree 4, strong/long → [4,6,2′]
longer space
beat/time 3: degree 2, soft → [2,4,6]
release → silence
```

Expected: every stratum translates by the same degree displacement; density remains three; no delayed or autonomous event appears during space.

### 49-second user exercise

Use the Role C schedule: degrees `1, 2, 3, 5, 6, 2, 1` at wall-clock starts `0, 7, 14, 21, 29, 37, 43` seconds with velocities `38, 48, 62, 82, 96, 56, 36`, complete releases at `5, 12, 19, 27, 35, 41, 47`, then two seconds of silence.

Expected arc is player-created low/quiet → rising → high/strong → withdrawing. After the final release, all input/harmony active sets are empty. Panic is idempotent. Stop routing leaves no note sounding. Environment snapshots before/after differ only in documented arrangement config.

## 11. Lifecycle and environment acceptance

1. Apply transaction calls Panic once, validates, and rolls back on failure.
2. For all six collection degrees, NoteOn returns the exact three-note plane above.
3. Matching NoteOff returns the identical three pitches.
4. Sequential non-overlapping notes never retain the previous plane.
5. Silence causes no generated event.
6. Panic after natural release is idempotent.
7. Stop routing releases external/synth output and clears tracked state.
8. The config contains no tonic, BPM, meter, devices, guitar state, routing, sound, master, mute/solo, plugins, or transport fields.
9. Suggested sound, if added later, remains metadata and is never silently applied.

## 12. Research traceability

- `history.md`: primary Durand scores, Debussy correspondence/criticism, BnF/Gallica, Waters, Hepokoski, Parker, Day-O'Connell, and Philharmonie de Paris. High confidence in 1903–10 scope, collection plurality, planing as bounded process, and dynamic/register/formal arc.
- `theory.md`: Waters, Tymoczko, Day-O'Connell, Uchida, primary score access, planing taxonomy, exact whole-tone interval math, abstract examples, and current engine audit. High confidence in the exact plane; its `needs_shared_capability` decision was based on the superseded `StylePreset` path and is corrected by current `ArrangementPresetV2` application code.
- `performance.md`: primary score directions, Guigue, Scott, Kaminsky, guitar pitch-tracking evidence, public adapter/config inspection, and deterministic 49-second exercise. High confidence in monophonic sparse control and player-shaped arc; medium confidence in exact tempo guidance.

Unresolved/deferred:

- pentatonic diatonic-plane variant;
- collection changes and bridges;
- independent pedal/common-tone ownership;
- open-fifth/wide-register cathedral voicing;
- automatic intensity/scene evolution;
- piano pedaling/resonance and orchestration;
- direct cross-surface manual performance evidence;
- guitar detector behavior outside the already accepted clean monophonic scope.

## 13. Decision

`ready_for_implementation`

No new engine or schema capability is needed. Reuse WholeTone + DiatonicThirds with three voices/source-lowest and disable every transformation that would disturb exact planing. Add the six-degree NoteOn/NoteOff regression, approve one immutable ArrangementPresetV2 record with the bounded copy above, and keep broader Debussy evolution unavailable rather than faking it.
