# Synthesis: Preset 23 — Sixth-Diminished Conveyor

**Decision:** `ready_for_implementation`

**Reference scope:** Barry Harris's mature, explicitly taught sixth-diminished method, approximately 1986–2021, with the strongest technical evidence in the Harris/Howard Rees workshop materials, Harris's 2010 Smithsonian oral history, and documented 1994–2016 teaching

**Operational claim:** one fixed major-sixth/diminished scale-of-chords study whose eligible source notes select a melody-top four-voice drop-2 sixth or related diminished sonority

## 1. Referenced scope

The bounded source is Harris's mature pedagogy, not his whole performing career and not bebop in general. The evidence follows the publicly documented teaching corpus from *Passing It On* (1986), Howard Rees's authorized *Barry Harris Workshop* materials (1994/2005), Harris's Smithsonian oral history (2010), a documented 2016 Bilbao workshop, and the continuity of his teaching through 2021.

That corpus presents sixth and minor-sixth chords, their related diminished sevenths, inversions, borrowed notes, family relationships, melody harmonization, surrounding, phrase rules, and context-dependent scale maps as connected practical tools. Harris explicitly resisted reducing diminished material to a half-step/whole-step formula. The preset therefore cannot claim that an eight-note collection or automatic chord toggle is the whole method.

The first operational version represents only one foundational exercise inside that system:

- one fixed tonic-relative major sixth-diminished collection;
- an unborrowed partition into tonic major-sixth tones and the related diminished-seventh tones;
- one source-controlled block per eligible note;
- a melody-top drop-2 realization;
- ordinary NoteOn/NoteOff ownership.

It does not reconstruct Harris's recorded playing, touch, time, accompaniment choices, improvisational grammar, repertory knowledge, teaching sequence, borrowed-note resolutions, related dominant families, chord-context interpretation, or career.

## 2. Style thesis

A clean monophonic line moves through one preserved major sixth-diminished collection. Notes belonging to the tonic sixth chord produce inversions of that stable family; the four interleaved notes produce inversions of its related diminished seventh. Adjacent collection motion can therefore create an audible `sixth → diminished → sixth` current. Contrapunk supplies only that fixed scale-of-chords relation. The performer supplies the line, rhythmic placement, accents, tension target, phrase arc, destination, and silence.

```text
Establish
  3–5 middle-register notes beginning on a sixth-family tone
    ↓ adjacent collection motion
Move
  one or two related diminished events, lightly articulated
    ↓ chosen sixth-family destination
Resolve
  hold the destination briefly and release cleanly
    ↓ one full bar or equivalent silence
Respond
  vary contour/register once, then thin and stop
```

The word “Conveyor” describes the audible passing motion created by the player's eligible adjacent notes. It does not mean an autonomous sequencer, time-based chord alternator, accompaniment generator, or endless process.

## 3. Research convergence and disagreements

All three independent reports agree that:

- Harris's sixth-diminished teaching is a scale **of chords**, not merely an eight-note bebop scale;
- major-sixth `[1,3,5,6]` and related diminished `[2,4,b6,7]` form the exact major collection partition;
- same-parity four-note sets are a genuine foundational kernel;
- borrowing, related families, inversions, melody harmonization, context, movement, and resolution make the complete method much broader;
- mechanical time alternation is false for repeated notes, leaps, rests, offbeat starts, outside chromatic notes, and changing harmonic regions;
- current `HarmonyMode::BarryHarris` is stateless and ignores beat phase;
- the current engine can honestly implement only a fixed unborrowed block-voicing study;
- outside-collection chromatic approaches currently pass through without generated harmony and must not be promised;
- Companion lanes should remain disabled for the static baseline;
- transport is optional and must remain untouched;
- the performer must use one physical source note at a time, shape phrases, choose arrivals, and leave silence;
- every generated attack must receive a matching release.

The main implementation disagreement is not musicological but contractual. Role A accepts literal alternation only as a basic exercise and insists that copy foreground its narrowness. Role B finds a concrete engine defect: the mode documented as four voices currently prepends the source to a complete four-note voicing, so tests require five outputs and can octave-double the melody pitch class. Role C describes the desired four-note interaction but correctly warns against note storms and unsupported chromatic input.

Resolution: the research is sufficient. Activation requires a small shared `BarryHarris` block-voicing correction so the source is one of exactly four voices, normally the top melody, rather than an undocumented fifth voice. Then approve only the fixed scale-of-chords study. Borrowing, family movement, harmonic timelines, extra-note rules, and phrase-aware resolution stay explicitly unavailable.

## 4. Stylistic invariants

1. Applying the preset preserves the user's tonic.
2. The active collection is `BHMajor6thDim`, tonic-relative pitch classes `[0,2,4,5,7,8,9,11]`.
3. Even collection indices form the tonic major-sixth family `[1,3,5,6]`.
4. Odd collection indices form the related diminished-seventh family `[2,4,b6,7]`.
5. Each accepted source attack sounds exactly four total voices: the physical source as one chord voice plus three generated voices.
6. The source remains perceptually identifiable and is the top melody voice for this built-in.
7. The four voices belong to one parity unless a future explicit borrowing strategy says otherwise.
8. The realization is one deterministic drop-2 spacing, not a claim that fixed drop-2 defines Harris's whole method.
9. Adjacent collection notes cross between sixth and diminished families; repeated same-family notes do not force time-based alternation.
10. Collection parity never masquerades as metrical strong/weak beat phase.
11. `BeatPhase` is not claimed because current Barry Harris voicing is stateless and transport-independent.
12. Outside-collection pitches are outside the generated-harmony contract and pass through without a guaranteed block.
13. Modal interchange remains disabled; it is not a substitute for Harris borrowing.
14. Voice leading remains disabled unless the Barry Harris builder itself owns a tested movement rule. Generic common-practice voice leading must not distort the fixed parity block.
15. Octave transformation remains `None` so the tested drop-2 contract is not silently re-spread or mirrored.
16. Companion, Canon, and Counterpoint lanes remain disabled.
17. The performer plays exactly one physical note at a time; keyboard chords, sustain overlap, double-stops, or ringing guitar strings violate the density contract.
18. A phrase begins on or establishes a sixth-family tone, exposes at least one diminished-family motion, then arrives on a chosen sixth-family tone and releases.
19. Diminished events are passing/mobile rather than repeatedly treated as equally final cadences.
20. Silence after a phrase remains silent; no generated tail survives a complete source release.
21. Every eligible NoteOn owns exactly the four pitches returned by its matching NoteOff.
22. Eligible and ineligible input, Panic, preset replacement, parameter change, and routing Stop end with zero active or pending generated notes.
23. Loading preserves BPM, meter, transport state/position, devices, guitar state, routing, sound, master, mute/solo, plugins, and user mix decisions outside the preset contract.
24. Public copy never says “authentic Barry Harris,” “the Barry Harris sound,” “all bebop harmony,” or “automatic correct chord under every note.”

## 5. Variables that preserve the bounded identity

- Any preserved tonic.
- A future separately researched minor built-in may use `BHMinor6thDim`; this first record stays major so one immutable preset has one audible contract.
- Middle source registers that leave room for the three lower drop-2 voices.
- Four- to eight-note monophonic phrases using eligible collection tones.
- Mostly adjacent motion, repeated notes, small skips, and rests.
- Straight or lightly swung eighth notes, quarter-note practice, or rubato demonstration.
- Player-controlled velocity, articulation, register, contour, and phrase spacing.
- Moderate 88–160 BPM practice reference; 108 BPM is the acceptance default.

Not variable in the first operational record:

- `BHMajor6thDim`;
- `BarryHarris` harmony mode;
- exactly four total voices;
- source in soprano position;
- fixed drop-2 block realization;
- voice leading off;
- octave mode `None`;
- interchange off;
- no Companion lane;
- transport optional.

## 6. Evolution model

### Within one phrase

`Stable sixth → adjacent movement → related diminished tension → sixth-family arrival → release`

- **Attack:** begin on one tonic-sixth member at moderate velocity.
- **Continuation:** move through two to four eligible notes, preferably adjacent collection degrees so family contrast is audible.
- **Tension:** let one related diminished event speak clearly and slightly lighter.
- **Arrival:** land on a chosen sixth-family member, hold it about one beat, and release.
- **Breath:** leave one full bar or equivalent silence; verify all generated pitches are gone.

### Across an eight-bar player-shaped section

`Establish → Vary → Intensify → Withdraw`

- **Establish, bars 1–2:** one four-note middle-register phrase and a full-bar gap.
- **Vary, bars 3–4:** repeat the rhythm with one contour or starting-degree change.
- **Intensify, bars 5–6:** extend to six to eight attacks, rise slightly, and make one diminished-to-sixth arrival unmistakable.
- **Withdraw, bars 7–8:** return to three or four attacks, lengthen the stable destination, release, and leave silence.

These states are performer instructions only. The engine does not detect phrases, sections, cadences, or intensity.

## 7. Rejected shortcuts and caricatures

Reject:

- describing the method as merely an “eight-note bebop scale”;
- mechanically alternating sixth/diminished chords by attack count or beat parity;
- claiming every chromatic approach receives Harris-style treatment;
- promising chord-change, bass-root, cadence, phrase, song, or scale-map inference;
- treating all related-family chords as context-free substitutions;
- presenting fixed drop-2 as the entire Harris system;
- adding the source to a complete four-note block and calling the five-note result “four voices”;
- octave-doubling the melody pitch class accidentally;
- forcing the source into an inner or bass role for this melody-harmonization built-in;
- generic modal interchange as fake borrowed-note movement;
- generic voice-leading motion as fake Harris borrowing/resolution;
- autonomous looping, delayed playback, random revoicing, or transport scheduling;
- continuous fast five-note chord blocks without rests;
- claiming swing, touch, joy, surprise, phrase grammar, or ensemble interaction is generated;
- treating Harris's pedagogical synthesis as a proprietary invention isolated from Parker, Gillespie, Powell, Monk, Dameron, Hawkins, and the broader Black American jazz tradition;
- presenting selected Chopin teaching analogies as proof that bebop harmony was derived from Chopin.

## 8. Exact Contrapunk mapping

After the shared four-total-voice Barry Harris fix:

```ts
{
  requirements: ['harmony'],
  config: {
    harmony: {
      scaleMode: 'BHMajor6thDim',
      mode: 'BarryHarris',
      voiceCount: 4,
      voicePosition: 0,
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

Why this is the smallest honest mapping:

- `BHMajor6thDim` encodes exactly the two interleaved pitch-class families;
- `BarryHarris` already selects same-parity four-note blocks and one drop-2 realization;
- four total voices with source in soprano position make the played melody one actual chord voice;
- no new temporal lane is needed for a stateless source-note mapping;
- no preset-specific algorithm is needed;
- all richer movement remains a named future shared voicing/timeline capability.

Implementation prerequisite:

- correct `harmonize_block_chord`/`build_voicing` so the returned block contains the exact played source once and only three generated voices;
- honor the four-total-voice contract and source-top placement;
- retain melody-only behavior for unsupported pitches without orphaning prior state;
- add exact all-degree and lifecycle regressions before catalog activation.

## 9. Approximation statement

A fixed live drop-2 harmonization of one tonic-relative major sixth-diminished collection from Barry Harris's mature teaching. Eligible sixth-chord tones sound the tonic major-sixth family; the interleaved collection tones sound its related diminished seventh. This foundational scale-of-chords exercise does not infer a song, chord progression, bass, harmonic region, borrowed-note movement, related dominant family, surrounding/extra-note rules, swing, cadence, phrase form, or Harris's touch and improvisational judgment. Notes outside the displayed collection pass through without guaranteed generated harmony. The player supplies movement, target, timing, accents, dynamics, register, resolution, and silence.

## 10. Performer contract

**Play it like:** Play one clean, connected note at a time through the shown eight-note collection. Move through a diminished passing tone into a clear sixth-chord arrival, hold it briefly, release, and leave a full-bar rest.

Expanded guidance:

- begin around C4–C5 equivalent on keyboard or D3–B4 on guitar;
- use one physical source note at a time;
- keyboard pedal stays up; guitarists mute departed and unused strings;
- use four to eight lightly swung or even eighth-note attacks per phrase;
- keep gates around 60–80% of the inter-onset interval so blocks connect without smearing;
- start around 96–120 BPM; use 88–160 BPM as a practical range and straighten eighths as tempo rises;
- begin on a sixth-family degree, include at least one adjacent diminished-family degree, then hold a sixth-family destination for about one beat;
- make the passing/diminished event slightly lighter and the destination clearer;
- rest one to two beats after a clause and one full bar after each phrase;
- after release, listen for complete silence before continuing;
- avoid source chords, sustain pedal, double-stops, ringing open strings, bends/slides through intermediate MIDI notes, low-register blocks, and continuous no-rest runs;
- do not expect outside chromatic notes to receive generated harmony.

Transport is optional. Applying the preset must not start, stop, reset, retime, or remeter it.

## 11. Abstract acceptance examples

These examples are synthetic and quote no melody.

### Exact family table in C major sixth-diminished

| Source collection degree | Expected family |
|---|---|
| `1 / C` | tonic major sixth `[1,3,5,6]` |
| `2 / D` | related diminished `[2,4,b6,7]` |
| `3 / E` | tonic major sixth |
| `4 / F` | related diminished |
| `5 / G` | tonic major sixth |
| `b6 / Ab` | related diminished |
| `6 / A` | tonic major sixth |
| `7 / B` | related diminished |

For every accepted input after the block-voicing fix:

- exactly four total notes;
- exact source pitch appears once;
- source is the top melody voice for the built-in;
- remaining voices share the source's collection parity;
- all pitch classes belong to the selected collection;
- matching NoteOff returns the identical owned set;
- active input/harmony state returns to empty.

### Controlled line

```text
beats:   1   1&  2   2&  3---  | rest
source:  1 → 2 → 3 → 2 → 1    | silence
family:  S → D → S → D → S    | none
```

Expected: audible family contrast, final sixth-family stability, no autonomous continuation, and no surviving note during the rest.

### Repeated-note rejection of mechanical alternation

```text
source: 3 → 3 → 5 → b6 → 6
family: S → S → S → D  → S
```

Expected: repeated/same-family notes remain sixth-family. Attack count does not force alternation.

### Unsupported chromatic approach

```text
source: #1 → 2
```

Expected: `#1` receives no guaranteed block; `2` receives the related diminished block. Both NoteOffs remain balanced and no note survives. Public copy must not call this a Harris extra-note or surrounding implementation.

### 45-second user exercise

1. `1–2–3–2–1`, hold final `1`, release, rest one bar.
2. `5–b6–6`, hold `6`, release, rest one bar.
3. One original six- to eight-note eligible phrase ending on a sixth-family member, then rest.
4. Repeat the first gesture one octave higher after full release.
5. Remain silent through the final bar and verify no generated note remains.

## 12. Lifecycle and environment acceptance

1. Preset application validates, calls Panic once, and rolls back on failure.
2. All eight collection degrees produce exactly four total owned notes with the exact source once.
3. Each source NoteOff returns the exact NoteOn set.
4. Repeated eligible notes do not leak active harmony ownership.
5. One eligible attack followed by one unsupported chromatic attack and both releases ends empty.
6. Rapid retrigger, normal release, Panic, preset replacement, mode/scale change, and routing Stop end with zero active/pending notes.
7. Silence emits nothing.
8. Panic after natural release is idempotent.
9. The config contains no tonic, BPM, meter, devices, guitar state, routing, sound, master, mute/solo, plugin, or transport fields.
10. The retained tonic selects the collection transposition.
11. Applying the record does not change transport state or position.
12. Built-in metadata remains immutable; Save As is the customization path.

## 13. Research traceability

- `history.md`: Harris's mature teaching corpus, Detroit/New York/community-teaching evolution, Smithsonian oral history, Harris/Rees workshop lineage, Barry Harris Institute evidence, shared bebop tradition, and rejected whole-career/performance claims. High confidence in the teaching-method scope and in rejecting scale-only/mechanical caricatures.
- `theory.md`: exact major/minor collections and parity, borrowing/family/context distinctions, all required musical dimensions, temporal model, abstract examples, and direct current-code audit. High confidence in the fixed partition; high confidence that the present five-output contract is defective; medium confidence in product-level phrase/density defaults.
- `performance.md`: monophonic interaction contract, register/tempo/articulation/density/silence guidance, guitar and keyboard constraints, failure recovery, UI wording, and a 45-second acceptance exercise. High confidence in the one-note/family-listening contract; practical tempo/register defaults require hands-on tuning.

Strongest shared sources:

- Smithsonian Jazz Oral History interview with Barry Harris (2010), especially transcript p. 25;
- Howard Rees, *The Barry Harris Workshop*, Parts 1 and 2;
- Barry Harris Institute curriculum and memorial material;
- García-Valdecasas Vaticón, “The Barry Harris harmonic theory in Chopin's work,” *Jazz-hitz* 3 (2020), pp. 109–141;
- Harris/Rees-lineage teaching materials and clearly identified practitioner analyses used only where primary/institutional evidence was unavailable.

Unresolved/deferred:

- exact timestamp audit of the complete commercial Harris/Rees workshop corpus;
- recording-by-recording prevalence in Harris's performance career;
- minor sixth-diminished as a separately tuned built-in;
- borrowed-note/common-tone resolution;
- related dominant/minor-sixth family movement;
- harmonic-region/chord-context selection and stable bass;
- chromatic surrounding and extra-note rules;
- melody harmonization beyond one fixed drop-2 realization;
- phrase/cadence/intensity state;
- transport-aware strong/weak alignment;
- user performance evidence on shipped keyboard and clean guitar surfaces.

## 14. Decision

`ready_for_implementation`

First fix the shared Barry Harris block-voicing contract so one source attack returns exactly four total voices with the exact source once in the top melody role. Add all-degree, repeated-note, unsupported-chromatic, NoteOn/NoteOff, Panic/reconfigure, and environment-preservation regressions. Then approve one immutable `ArrangementPresetV2` record with the bounded copy and config above. Keep borrowing, family movement, chromatic line rules, harmonic timelines, phrase behavior, and claims of Harris's broader style unavailable rather than faking them.
