# Synthesis: Preset 07 — Stretto Engine

**Decision:** `ready_for_implementation` as a fixed overlapping-answer Strict Canon approximation  
**Reference:** selected J. S. Bach fugues, principally WTC I BWV 846/2 and *The Art of Fugue* Contrapunctus 5  
**Research pack:** three independent reports plus this parent synthesis

## 1. Referenced scope

Stretto Engine references selected procedures in Bach fugues rather than a generic “Bach style.” Its primary historical anchors are:

- WTC I, Fugue in C major, BWV 846/2, in Bach's 1722 autograph fair copy D-B Mus.ms. Bach P 415: an unusually subject-saturated fugue with many close entries at varied temporal and pitch intervals;
- *The Art of Fugue*, especially Contrapunctus 5, represented by the autograph project state D-B Mus.ms. Bach P 200 (1742–49) and the separately ordered posthumous original-print state: a systematic late exploration of entry distance, rectus/inversus, and dux/comes relationships;
- WTC corpus research as negative evidence against claiming that Bach fugues universally add more, denser, progressively closer stretti near their endings.

BWV 849/2 and other works remain contrasting evidence only. The preset does not turn one five-voice or multi-subject fugue into a default live configuration.

## 2. Style thesis

Fugal stretto is overlap between recognizable subject/answer entries whose onset distance is shorter than the opening reference distance. It is not merely echo, tempo acceleration, a chord stack, or “more notes.” In the selected works, subject identity survives varied pitch levels, register, fragments, inversion, and sometimes rhythmic adjustment because the combinations were compositionally tested for counterpoint, dissonance, and cadence.

Contrapunk's baseline provides a transparent fixed arrangement: the player states one exact two-to-four-note cell; three single-note Strict Canon followers replay every source event with unison-, dominant-, and octave-related transpositions. Their absolute delays are `2.0`, `3.25`, and `4.0` beats, so successive entry gaps contract from `2.0` to `1.25` to `0.75` beats. This creates one exposition-to-overlap gesture. It does not recognize a motif, construct a tonal answer, validate full-subject counterpoint, evolve across sections, or generate a Bach fugue.

## 3. Convergent findings and disagreements

All three reports converge on:

- a short, rhythmically distinctive monophonic cell;
- transport before the first note and stable BPM through the tail;
- subject recognizability and actual overlap as the minimum stretto identity;
- tonic/dominant register relationships as bounded entry levels, not arbitrary semitone offsets;
- one independent note per entry, not harmony stacks;
- sparse player input and deliberate silence as density control;
- exact source and generated NoteOn/NoteOff ownership;
- visible copy stating that progressive compression is a product arc, not a universal Bach rule.

The history and theory reports explicitly reject a global “stretto always tightens toward the final climax” claim. Contrapunctus 5 contains non-monotonic entry-distance groups, and WTC corpus research does not support the common conclusion/density hypotheses. The preset may still choose one short, progressively compressed group as an audible technique—provided it says so.

The theory report distinguishes real and tonal answers. Current `transposeDegrees: 4` is only a diatonic real-answer approximation; it cannot rewrite the subject's 1↔5 skeleton as a tonal answer. The preset copy must not use “tonal answer” or claim source-specific dominant treatment.

## 4. Implementation invariants

1. The performer supplies one sequential two-to-four-note monophonic cell.
2. Transport is running before input and remains running through all delayed releases.
3. Harmony is PassThrough with one player voice; each Canon follower is also PassThrough with `voiceCount: 1`.
4. Strict Canon uses `timeRatio: 1` for every follower; source onset spacing and durations remain unchanged.
5. Followers use absolute delays `2.0`, `3.25`, and `4.0` beats. Entry gaps are therefore `2.0`, `1.25`, and `0.75` beats.
6. The instructed source duration is approximately two beats, so the first follower establishes a reference distance and the later followers overlap earlier entries.
7. Transpositions are diatonic unison `0`, dominant-related `+4`, and octave `+7`; the fifth entry is explicitly a real/diatonic answer approximation.
8. No follower emits a chord stack, countersubject, random harmony, octave Mirror, or additional voice-leading transform.
9. Generic modal interchange is disabled.
10. Canon group Hold is explicitly `Forever` so the whole fixed answer group survives normal source release; every hard boundary still cancels it.
11. The performer does not start a new cell until the full four-entry group ends and a fresh phrase boundary is established.
12. The fixed delays are immutable during active/pending notes.
13. Every emitted NoteOn receives exactly one effective NoteOff. Stop, Reset, Panic, disable, voice replacement, preset replacement, and teardown clear sounding and pending state idempotently.
14. Applying the preset preserves tonic, BPM, time signature, devices, guitar state, routing, sound, master level, mute/solo, hosted plugins, and transport running state.
15. UI copy says “overlapping delayed-answer approximation” and never claims motif recognition, automatic fugue, tonal-answer generation, invertible counterpoint, or a Bach reconstruction.

## 5. Evolution model

```text
ARMED / SILENT
  transport running, queues empty
    ↓ player performs one exact 2–4-note cell
DECLARE
  player states source at delay 0
  no additional source material admitted
    ↓ +2.0 beats
REFERENCE ANSWER
  unison-related follower states the cell
  establishes the wide entry distance
    ↓ +1.25 beats
STRETTO BUILD
  dominant-related follower enters before reference answer completes
    ↓ +0.75 beats
PEAK OVERLAP
  octave-related follower enters at closest spacing
  player remains silent
    ↓ longest scheduled NoteOff
RELEASE
  all followers finish under group Forever Hold
  active/pending/held state becomes zero
    ↓ >2 transport beats silence
ARMED for a new cell
```

Across a 30–60 second exercise, the player repeats the same cell only after the preceding group clears, then changes one performer variable (register or velocity). The engine does not dynamically decrease delay after every repetition; one fixed group carries the progressive-closeness gesture.

## 6. Caricatures and rejected shortcuts

Reject:

- “stretto means faster” or `stringendo` tempo acceleration;
- generic delay/echo with no instructed subject identity;
- every incoming improvisatory note being marketed as a fugue subject;
- tonic/fifth/octave described as Bach's fixed recipe;
- `+4` diatonic degrees labeled a tonal answer;
- “octave answer” treated as a third harmonic function rather than register redistribution;
- automatically reducing delay after every NoteOn;
- every Canon voice emitting a harmony stack;
- parallel third/sixth pads presented as a countersubject;
- five voices because BWV 849 is five-voice;
- continuously dense input, chords, sustain pedal, or overlapping guitar strings;
- claims that closer always means more Bachian or that stretto always appears at a final climax;
- “24 entries in BWV 846/2” as uncontested fact;
- one fixed final source/order for *The Art of Fugue*;
- exact Bach counterpoint, episodes, cadence, inversion, tonal answers, or source-specific voice leading.

## 7. Exact Contrapunk mapping

### Metadata

```ts
{
  id: '07-stretto-engine',
  name: 'Stretto Engine',
  family: 'classical',
  tags: ['baroque', 'stretto', 'strict-canon', 'transport'],
  builtIn: true,
  result: 'One short subject returns at unison-, dominant-, and octave-related levels with entry gaps contracting from 2 to 1.25 to 0.75 beats.',
  references: [{
    name: 'J. S. Bach',
    context: 'Bounded reference to varied stretto procedures in BWV 846/2 and Contrapunctus 5.'
  }],
  researchStatus: 'approved',
  requirements: ['strict_canon']
}
```

### Performer guidance

```ts
{
  prompt: 'Play two to four clear single notes with one memorable rhythm, then stop and listen as unison-, fifth-, and octave-related answers enter progressively closer.',
  input: 'motif',
  articulation: 'Lightly detached, even attacks; gate each note to about 50–75% of its inter-onset interval.',
  density: 'One monophonic cell only; play less as the answers crowd in.',
  space: 'Stay silent through the full delayed group, then leave more than two additional transport beats.',
  tempo: '72–100 BPM; 90 BPM is the test default.',
  transportRequired: true
}
```

### Arrangement config

```ts
{
  harmony: {
    scaleMode: 'Ionian',
    mode: 'PassThrough',
    voiceCount: 1,
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
    enabled: true,
    globalHoldMode: { kind: 'cancel' },
    canon: {
      enabled: true,
      form: 'strict_canon',
      holdMode: { kind: 'forever' },
      voices: [
        {
          delayBeats: 2,
          transposeDegrees: 0,
          timeRatio: 1,
          harmonyMode: 'PassThrough',
          referenceVoice: null,
          voiceCount: 1,
          voicePosition: 0,
          voiceLeadingEnabled: false,
          voiceLeadingStyle: 'Free',
          octaveMode: 'None',
          counterpointSpecies: 'Species1',
          counterpointStrictness: 'Strict',
          holdMode: null
        },
        {
          delayBeats: 3.25,
          transposeDegrees: 4,
          timeRatio: 1,
          harmonyMode: 'PassThrough',
          referenceVoice: null,
          voiceCount: 1,
          voicePosition: 0,
          voiceLeadingEnabled: false,
          voiceLeadingStyle: 'Free',
          octaveMode: 'None',
          counterpointSpecies: 'Species1',
          counterpointStrictness: 'Strict',
          holdMode: null
        },
        {
          delayBeats: 4,
          transposeDegrees: 7,
          timeRatio: 1,
          harmonyMode: 'PassThrough',
          referenceVoice: null,
          voiceCount: 1,
          voicePosition: 0,
          voiceLeadingEnabled: false,
          voiceLeadingStyle: 'Free',
          octaveMode: 'None',
          counterpointSpecies: 'Species1',
          counterpointStrictness: 'Strict',
          holdMode: null
        }
      ]
    },
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
```

The user's tonic remains authoritative. Ionian is a safe product default for the fixed answer levels, not a claim that the selected fugues or Bach's output share one mode. Unity role gains avoid silently requiring role-mix capability.

## 8. Approximation statement

> Stretto Engine is a fixed Strict Canon arrangement inspired by selected Bach stretto procedures. It repeats every note of your short live cell at three preset pitch levels and contracting entry gaps. It does not recognize a subject, generate real versus tonal answers, test invertible counterpoint, compose episodes or cadences, or reconstruct a Bach fugue.

This statement remains visible after Apply.

## 9. Performer contract

### Short prompt

**Play two to four clear single notes with one memorable rhythm, then stop and listen as unison-, fifth-, and octave-related answers enter progressively closer.**

### Expanded guidance

- Start transport before input and keep BPM fixed through the tail.
- Use one note at a time around C4–C5 on keyboard or A3–E5 on clean guitar.
- Use a 2–4-note cell lasting about two beats, with one short and one longer onset gap.
- Play at 72–100 BPM, beginning with 90 BPM.
- Gate notes to roughly 50–75% of their inter-onset spacing; preserve the exact rhythm and release pattern.
- Use moderate, even velocity around 72–92.
- After the cell, play nothing through the delayed group; listen for the rhythmic fingerprint before the pitch level.
- Do not add a cadence chord. Let the fixed answer group release naturally.
- Keyboard: sustain pedal off, one key sounding at a time.
- Guitar: clean low-gain sound, active muting, no double-stops, open-string overlap, bends, wide vibrato, slides, hammer-ons/pull-offs, or noisy attacks during acceptance.
- If a source note mistracks, stop and Panic rather than feeding corrections into the delayed group.

## 10. Acceptance examples

Examples use novel abstract degrees and beats; they do not copy Bach themes.

### A. Entry-distance contraction

Source cell: degrees `1, 2, 5` at relative beats `0, 0.5, 1.5`, each held `0.35` beat. Approximate subject span: `1.85` beats including the last gate.

Expected first-note entry onsets:

- player/source: beat `0`;
- unison follower: beat `2.0`;
- dominant-related follower: beat `3.25`;
- octave follower: beat `4.0`.

Expected gaps: `2.0`, `1.25`, `0.75`. The final two gaps are shorter than the source span, so those entries overlap their predecessors. Every follower preserves source relative onsets `0, 0.5, 1.5` and duration `0.35`.

Assertions:

- exactly one player line plus three single-note follower lines;
- no harmony stack or time-ratio change;
- delays sort to `[2, 3.25, 4]` and successive gaps contract;
- all generated events release naturally under group Forever Hold;
- after the final delayed NoteOff and >2 beats silence, pending/held/active state is zero.

### B. Real-answer approximation disclosure

Input abstract subject: `1, 5, 4, 3`.

Expected dominant follower: a constant diatonic `+4` transform. UI and tests label it `real-answer approximation`; they do not claim the minimal interval rewrites a tonal answer would require.

### C. Normal and hard lifecycle

Play the three-note cell and release every source note before the first follower enters.

Expected normal path: group Forever preserves all three answer statements and paired delayed NoteOffs. Then replay and invoke transport Stop while the second group is pending.

Expected hard path: sounding notes release; pending NoteOns never appear after restart; active, pending_on, pending_off, and held state become zero; repeated Stop/Panic is idempotent.

### D. Environment preservation

Apply Stretto Engine with non-default tonic, BPM, meter, devices, routing, sound, master, mute/solo, plugin, and transport state.

Expected: one transactional Panic/reset; only the arrangement config above changes; the performance environment and whether transport is running remain unchanged.

## 11. Research traceability

- `history.md` — independent history/primary-source report. Principal evidence: Bach Digital P 415 and BWV 1080 version records; DDB; Kerman; McDonald WTC corpus; Alevizos Contrapunctus 5 analysis. High confidence in source chronology, negative evidence against universal progressive compression, and the bounded catalog judgment.
- `theory.md` — independent theory/temporal report. Principal evidence: Bach Digital, Alevizos, score-linked BWV 846/Contrapunctus 5 analyses, university fugue/answer resources, and local CanonLane/HarmonyEngine/transport code. High confidence in overlap, answer distinctions, current capability gaps, and lifecycle requirements.
- `performance.md` — independent performer/HCI report. Principal evidence: WTC corpus research, BWV 878/2 performance analysis, Bach Digital, organist timing study, guitar note-tracking research, MIDI Association, and local scheduling code. High confidence in sparse repeatable source, transport, silence, monophonic constraints, and cleanup; numeric tempo/velocity/gate ranges are HCI defaults.

Unresolved but non-blocking for this explicitly bounded baseline:

- analytical disagreement over complete/fragmentary subject counts;
- full NBA variant collation;
- motif identity detection and head-fragment classification;
- tonal-answer generation;
- prospective vertical/invertible-counterpoint validation;
- dynamic entry groups, episodes, cadence, and non-monotonic section plans;
- live performer evaluation across all shipping surfaces.

These gaps block automatic Bach fugue or adaptive stretto claims, not the fixed delayed-answer approximation.

## 12. Decision

`ready_for_implementation`

Activation requires the exact single-note Strict Canon config, fixed entry-delay sequence, explicit group Forever Hold, transport-required guidance, deterministic delay/NoteOff tests, and visible approximation statement. Surfaces without Strict Canon/Companion capability keep the record visible but locked.
