# Synthesis: Preset 14 — Color-Mode Windows

**Decision:** `ready_for_implementation`

**Reference scope:** Messiaen's modal-harmonic language as codified in 1944, with selected works from approximately 1935–44

**Operational claim:** one static, fixed-transposition Mode-2 diminished-seventh window whose phrase contrasts are shaped by the performer

## 1. Referenced scope

The bounded source is not Messiaen's whole career. It centers on the language documented in *Technique de mon langage musical* (1944) and heard in selected works from *La Nativité du Seigneur* (1935) through *Quatuor pour la fin du Temps* (1940–41), *Visions de l'Amen* (1943), and *Vingt Regards sur l'Enfant-Jésus* (1944).

That period joins modes of limited transposition to tonal poles, characteristic and added-note chords, pedals, resonance, compact cyclic motives, register, contrasting dynamics/articulation, added values, non-retrogradable rhythms, and sharply differentiated blocks or layers. The operational preset represents only one pitch-and-voicing subset of that conjunction.

Later birdsong-centered, experimental, operatic, orchestral, and late works establish the boundary rather than expand the claim. *Catalogue d'oiseaux*, *Chronochromie*, *Couleurs de la cité céleste*, *Saint François d'Assise*, and *Éclairs sur l'Au-Delà…* use retained modal resources inside languages that a static live harmonizer does not reproduce.

The catalog's original whole-tone/octatonic/augmented rotation is rejected:

- exposed whole tone was a predecessor-saturated resource Messiaen normally treated cautiously;
- Mode 2 is an eight-note octatonic collection with three distinct transpositions;
- Messiaen's Mode 3 is a nine-note collection with four distinct transpositions, not Contrapunk's six-note `AugmentedHex`;
- current arrangement state selects one scale and has no harmonic timeline that rotates mode families or transpositions.

“Windows” therefore means **player-framed repetitions and contrasts inside one fixed Mode-2 collection**, not automatic scale rotation.

## 2. Style thesis

Each clean in-collection source note becomes a close four-note diminished-seventh sonority drawn entirely from one preserved transposition of Messiaen's Mode 2. When the player moves to the next collection degree, every generated stratum moves with it by the same alternating semitone/whole-tone step and the chord shape stays fixed. The engine supplies this narrow static color field; the player creates form by stating a short cell, leaving silence, repeating it in another register or dynamic, changing one terminal degree, and finally contracting the cell into a quiet residue.

```text
Present
  2–4 clear middle-register notes in one fixed Mode-2 collection
    ↓ full release and a long gap
Refract
  recognizable return, one controlled change in register/velocity/last degree
    ↓ another long gap
Compare
  return of the original cell and voicing relation
    ↓ fewer, softer, lower attacks
Withdraw
  contracted cell, complete release, silence
```

No automatic phrase detector, collection/transposition rotation, tonal-center inference, non-retrogradable rhythm, added-value generator, birdsong, orchestration, chord-color naming, or synesthetic color reproduction is claimed.

## 3. Research convergence and disagreements

All three independent reports agree that:

- modes of limited transposition are one component of a larger harmonic, rhythmic, timbral, theological, and formal language;
- Mode 1 is whole tone, Mode 2 is octatonic, and Mode 3 has nine notes;
- the current six-note `AugmentedHex` must never be called Messiaen Mode 3;
- collection membership alone is shared modernist vocabulary, not stylistic ownership;
- chord, spacing, register, attack, contrast, and silence matter more than an isolated scale label;
- current Contrapunk can select one fixed Mode-2 collection but cannot rotate collections or transpositions over time;
- a four-voice `DiatonicThirds` chain over Mode 2 deterministically produces `[0,3,6,9]` diminished-seventh planes;
- that mapping is honest only as a narrow static study, not Messiaen's general chord vocabulary;
- the performer must supply the short cell, contrast, phrase boundaries, and withdrawal;
- transport is unnecessary and must remain untouched.

Role A recommends the historically broader target of characteristic chords, common tones/pedals, modal changes, rhythm, and contrast. Role B identifies the exact current-engine subset but rates its stylistic representativeness only medium-low because diminished planing can sound generic. Role C confirms that short repeated cells and long rests make that narrow relation audible and playable.

Resolution: activate the static Mode-2 diminished-seventh window because it is deterministic, current, differentiated from Planed Cathedral, and lifecycle-testable. Keep the catalog's plural rotating result unavailable until a shared harmonic timeline exists. Keep genuine Mode 3 unavailable until the nine-note collection is represented correctly. Public copy states the narrowness prominently.

## 4. Stylistic invariants

1. The active pitch field is one fixed transposition of Mode 2, represented by `DiminishedHalfWhole` relative to the user's preserved tonic.
2. Applying the preset does not change tonic. Because Mode 2 has only three distinct transpositions, several tonic labels select pitch-class-equivalent collections; the UI must not imply twelve unique Mode-2 colors.
3. Accepted source notes belong to the displayed collection.
4. Exactly four pitches sound per accepted source attack: input plus three generated voices.
5. With source as the lowest arrangement voice, the ordered close-stack vector is `[0,3,6,9]` semitones for all eight collection degrees in the tested middle register.
6. Moving the source by one Mode-2 degree translates every voice by the same `+1` or `+2` semitone step; chord quality and inversion remain fixed.
7. Voice leading is disabled so common-practice rules do not remove parallel blocks, tritones, or semitone relations.
8. Octave transformation is `None`; no automatic Spread, Mirror, or bass/treble split alters the tested close stack.
9. Modal interchange is disabled. Out-of-collection input is outside the exact-membership acceptance contract.
10. Companion, Canon, and Counterpoint lanes are disabled. No delayed, held, rhythmic, or autonomous event is generated.
11. Source NoteOff owns and releases the exact four-note plane emitted for its NoteOn.
12. Silence remains silence. No prior plane survives into the long rests of the acceptance gesture.
13. The same abstract source cell produces the same pitch relation when repeated in the same register; register/velocity changes come from the player.
14. The performer states only one physical note at a time; played chords, sustain-pedal overlap, or ringing guitar strings violate the density contract.
15. The audible phrase consists of two-to-four-note cells separated by 1.5–3 seconds of silence.
16. A contrasting window changes one player-controlled dimension at a time: register, velocity, final degree, duration, or spacing.
17. The engine never claims to infer or reproduce Messiaen's personal color perceptions.
18. Mode 1, Mode 3, added/resonance chords, pedals, tonal anchoring, added-value/non-retrogradable rhythm, independent cycles, birdsong, orchestration, and theology remain outside the operational behavior.
19. Loading preserves tonic, BPM, meter, devices, guitar state, routing, sound, master, mute/solo, plugins, role mix, and transport running/position state.
20. Normal NoteOff, Panic, Stop routing, preset replacement, and parameter change leave no active generated note.

## 5. Variables that preserve the bounded identity

- Any user-selected tonic, understood as selecting one of only three distinct Mode-2 pitch-class collections.
- Middle source registers that leave room for the three upper minor-third strata.
- A repeated two-, three-, or four-note in-collection source cell.
- Source duration around 0.6–2 seconds, with one 2–3 second destination.
- Moderate pacing around 54–76 BPM if the player wants a reference; transport remains optional.
- Player-controlled register, velocity, articulation, duration, and silence.
- Transposition of the entire exercise.

Not variable in the first operational record:

- `DiminishedHalfWhole` collection family;
- four total voices;
- source at voice position 3, the lowest of four;
- `[0,3,6,9]` ordered close stack;
- `DiatonicThirds`;
- voice leading off;
- octave mode `None`;
- interchange off;
- no Companion lane.

## 6. Evolution model

### Within one cell

`Clear attack → stable four-note color → one controlled source move → destination/release → silence`

- **Attack:** one firm, non-percussive middle-register note makes the full plane audible.
- **Continuation:** two or three more in-collection notes retain the same voicing relation.
- **Destination:** a slightly longer or stronger final note supplies an apex without functional cadence.
- **Release:** ordinary NoteOff clears all four pitches.
- **Breath:** a long rest frames the cell and proves no autonomous continuation.

### Across windows

`Present → Refract → Compare → Withdraw`

- **Present:** original three-note cell at medium-soft velocity.
- **Refract:** recognizable return one octave higher or with one changed final degree and stronger attack.
- **Compare:** original cell returns in its first register so the stable mapping is audible.
- **Withdraw:** first two degrees only, lower/softer/longer, then silence.

These transitions are entirely player-triggered. A future harmonic timeline could introduce another Mode-2 transposition or genuine Mode 3 at an explicit phrase boundary, but that capability is not present and is not required by this record.

## 7. Rejected shortcuts and caricatures

Reject:

- calling the preset “Messiaen's harmony,” “Messiaen mode,” or a reconstruction of any work;
- equating Messiaen with symmetric scales alone;
- rotating whole-tone, octatonic, and augmented scales mechanically;
- treating degree rotation, transposition, and mode-family change as the same operation;
- calling `AugmentedHex` Mode 3;
- treating any octatonic line or diminished chord as uniquely Messiaen;
- promising named visual/RGB colors from single notes or generated chords;
- using whole tone as the default Messiaen identifier;
- random chord membership or random transposition per note;
- modal interchange as a fake collection timeline;
- Palestrina/Bach/Common-Practice voice leading that disturbs intentional parallels;
- Mirror or Spread as a fake resonant/orchestral peak;
- random jitter as “added-value rhythm”;
- delayed playback as “non-retrogradable rhythm”;
- constant high density, low-register chord storms, or sustain overlap;
- claiming autonomous motif, phrase, birdsong, orchestration, tonal plan, theology, or color-form behavior.

## 8. Exact Contrapunk mapping

```ts
{
  requirements: ['harmony'],
  config: {
    harmony: {
      scaleMode: 'DiminishedHalfWhole',
      mode: 'DiatonicThirds',
      voiceCount: 4,
      voicePosition: 3,
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

Why this works for in-collection middle-register input:

- `DiminishedHalfWhole` contains semitone classes `[0,1,3,4,6,7,9,10]` relative to preserved tonic;
- Mode 2 has three distinct transpositions, and tonic transposes the selected collection without creating a new scale family;
- `DiatonicThirds` advances two scale degrees for each chained voice;
- four voices with source at position 3 produce source, +2, +4, and +6 degrees;
- over Mode 2 those offsets are `[0,3,6,9]` semitones for every collection degree;
- voice-leading, octave, and interchange transformations are off;
- no temporal lane is needed for the static mapping.

No engine branch, schema field, or new capability is required.

## 9. Approximation statement

A static, fixed-transposition Mode-2 diminished-seventh window inspired by one pitch-and-voicing property of Messiaen's modal-harmonic language as codified in 1944. It does not reproduce his characteristic/added/resonance chords, pedals, tonal poles, Mode-1 concealment, nine-note Mode 3, changes among modal transpositions, added-value or non-retrogradable rhythms, independent cycles, birdsong, register/orchestration plans, theology, or personal chord-color perceptions. It never rotates collections automatically. The player supplies the cells, contrasts, register, dynamics, articulation, duration, and silence.

## 10. Performer contract

**Play it like:** Play a crisp two-to-four-note cell from the shown collection, then leave a long gap. Repeat it once higher or stronger, change only one note, and listen to each generated chord as a separate color window.

Expanded guidance:

- stay in the displayed octatonic collection;
- begin around C3–C5 equivalent; guitar prefers stable fretted middle-string notes around D3–B4;
- use one physical note at a time and release it before the next attack;
- make attacks clear and firm rather than harsh;
- sustain most notes for 0.6–2 seconds and one destination for 2–3 seconds;
- use two to four notes per cell and no more than 6–10 attacks per 15 seconds;
- leave 0.3–0.8 seconds between notes and 1.5–3 seconds after each cell;
- repeat the cell with one controlled change in octave, final collection degree, velocity, or spacing;
- contract the final cell to two softer/lower notes and stop;
- guitarists mute every unused string and avoid double-stops, ringing open strings, harmonics, bends, slides, wide vibrato, and legato flurries;
- keyboardists use one key at a time and no sustain pedal during acceptance;
- after each attack, wait until the complete plane is audible; after each cell, listen through the release before continuing.

Transport is optional. Applying the preset must not start, stop, reset, retime, or remeter it.

## 11. Abstract acceptance examples

These examples are synthetic and quote no melody.

### Exact Mode-2 plane table

With preserved tonic C and middle-register inputs:

| Mode-2 source | Expected MIDI plane |
|---|---|
| degree 1 / 60 | `[60,63,66,69]` |
| degree 2 / 61 | `[61,64,67,70]` |
| degree 3 / 63 | `[63,66,69,72]` |
| degree 4 / 64 | `[64,67,70,73]` |
| degree 5 / 66 | `[66,69,72,75]` |
| degree 6 / 67 | `[67,70,73,76]` |
| degree 7 / 69 | `[69,72,75,78]` |
| degree 8 / 70 | `[70,73,76,79]` |

For every row:

- four distinct pitches;
- ordered pitch vector `[0,3,6,9]` relative to input;
- all pitch classes remain in the selected Mode-2 collection;
- ordinary NoteOff returns the identical owned pitch set;
- active input and harmony state returns to empty.

### Short cell

```text
window A: degree 1 → degree 3 → degree 2 → silence
window B: same contour one octave higher, final degree 4 → silence
window C: degree 1 → degree 3 → degree 2 → silence
window D: degree 1 → degree 2, softer/lower → final silence
```

Expected:

- A and C produce the same abstract four-note relation;
- B changes only player register/velocity and terminal degree;
- D contracts source density without changing generated density per attack;
- whole planes move by the same alternating one- or two-semitone collection step;
- no old plane or autonomous event crosses a window's long rest.

### 42-second user exercise

Use the Role C schedule: present `1–3–2` at 0, 2, and 4 seconds; rest to 9; refract `1–3–4` one octave higher at 9, 10.7, and 12.4 seconds; rest to 18; compare `1–3–2` at 18, 20, and 22 seconds; rest to 28; withdraw `1–2` lower/softer at 28 and 31 seconds; release by 34 and remain silent through 42 seconds.

Expected: four audible player-framed windows, constant four-note generated density per attack, no automatic collection change, no event during long rests, and empty active/pending state at 42 seconds.

## 12. Lifecycle and environment acceptance

1. Preset application validates, calls Panic once, and rolls back on failure.
2. All eight in-collection degree inputs emit the exact four-note planes above.
3. Matching NoteOff returns the identical owned plane.
4. Sequential non-overlapping notes never retain the previous plane.
5. Silence emits nothing.
6. Panic after natural release is idempotent.
7. Stop routing releases external/synth output and clears tracked state.
8. The config contains no tonic, BPM, meter, devices, guitar state, routing, sound, master, mute/solo, plugin, or transport fields.
9. The preserved tonic selects the Mode-2 transposition; the preset does not overwrite it.
10. Any suggested sound remains metadata and is never silently applied.

## 13. Research traceability

- `history.md`: Messiaen's *Technique*, institutional publication/work records, early/late score-based scholarship, Benitez, Pople, Dingle, Fallon, and Taruskin. High confidence in Mode 1/2/3 facts, the 1935–44 scope, Mode 1 caution, shared octatonic lineage, chord-centered color claims, and career expansion beyond the modes.
- `theory.md`: exact M1/M2/M3 pitch-class/transposition math, four-voice Mode-2 chain, all required musical dimensions, temporal states, current code-capability audit, abstract examples, and explicit blockers. High confidence in the exact fixed mapping; medium-low confidence in diminished planing as representative without the bounded label.
- `performance.md`: treatise/Pople/Benitez scope, guitar-tracking evidence, current adapter/config inspection, a monophonic short-cell interaction contract, and a deterministic 42-second exercise. High confidence in the short-cell/rest design and player-framed contrast; cross-surface live usability remains manual.

Unresolved/deferred:

- genuine nine-note Mode 3;
- automatic changes among Mode-2 transpositions or mode families;
- characteristic, added-note, resonance, pedal, and tonal-anchor voicings;
- non-retrogradable/additive rhythm and unequal cycles;
- stable register/orchestration groups;
- birdsong and motif recognition;
- adaptive phrase/intensity scenes;
- synesthetic color descriptions;
- direct cross-surface performance evidence;
- off-collection exact-mapping policy.

## 14. Decision

`ready_for_implementation`

Reuse `DiminishedHalfWhole + DiatonicThirds` with four voices/source-lowest and disable every transformation that would disturb the exact Mode-2 diminished-seventh plane. Add an eight-degree NoteOn/NoteOff regression, approve one immutable `ArrangementPresetV2` record with the bounded copy above, and keep plural mode rotation and genuine Mode 3 unavailable rather than faking them.
