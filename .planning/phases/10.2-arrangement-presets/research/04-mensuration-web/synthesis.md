# Synthesis: Preset 04 — Mensuration Web

**Decision:** `ready_for_implementation` as a bounded scalar proportional-imitation approximation  
**Reference:** Johannes Ockeghem, centered on *Missa prolationum*  
**Research pack:** three independent reports plus this parent synthesis

## 1. Referenced scope

Mensuration Web references the generative principle of Johannes Ockeghem's four-voice *Missa prolationum*: shared written material produces paired canonic realizations under contrasting mensural divisions. Primary source anchors are Vatican City, Biblioteca Apostolica Vaticana, MS Chigi C.VIII.234, fols. 106v–114, and Vienna, Österreichische Nationalbibliothek, Cod. 11883 Han, fols. 208–221. The Chigi source presents compressed canonic notation; the Vienna witness resolves the canons into uniform mensurations.

The Mass's composition date is unresolved. The Chigi copy was produced around 1498–1503 and proves transmission, not composition chronology. The preset therefore does not call this an early, late, or culminating work.

The exhaustive double-mensuration-canon design is work-specific. The preset does not reduce Ockeghem's long French royal/ecclesiastical career, Masses, chansons, or broader contrapuntal practice to one puzzle technique.

## 2. Style thesis

The historical reference is a coordinated temporal system, not several unrelated clocks. Two written lines generate two paired canons; binary/ternary mensural readings make durations, alignments, and entries diverge while the voices remain singable, contrapuntally controlled, and cadentially coordinated. The construction normally sounds like continuous vocal polyphony rather than an arithmetic demonstration.

Contrapunk's operational baseline is deliberately narrower. A live performer supplies one exact short motif. The source plus three single-note Canon voices share one transport anchor and preserve event order at fixed `1:1`, `3:2`, and `2:1` scalar timing relationships. This is **proportional Free Imitation inspired by the Mass's procedure**, not mensural reinterpretation, double-canon reconstruction, or exact Ockeghem counterpoint.

## 3. Convergent findings and disagreements

All three reports converge on:

- one clean three-to-six-note monophonic cell;
- one running transport and one shared phrase epoch;
- exact event order, deliberate NoteOffs, and substantial silence;
- rational/named relationships rather than arbitrary continuous rate knobs;
- four melodic lines, not each Canon voice expanded into a chord stack;
- no claim that mensuration signs equal modern time signatures, BPMs, or independent metronomes;
- no claim that all four historical voices derive from one motif—the Mass normally uses two generating lines/two paired canons;
- normal paired NoteOff cleanup and zero final active/pending state;
- explicit bounded-approximation copy.

The history report emphasizes source and terminology: prolation is one level of mensural division, notation date is not composition date, and absolute performed tempo is not uniquely encoded by a sign. The theory report concludes that scalar phrase stretching is usable but lacks note-value hierarchy, two-source pairing, ficta, vertical validation, cadences, and section state. The performer report makes transport and sparse input mandatory for current live operation.

A local code audit adds one critical causality boundary: a live ratio below `1.0` computes later follower attacks before the corresponding source attack has arrived. CanonLane can only emit such an event immediately/late, not predict it. Therefore this baseline uses no reciprocal/diminution ratio. True live diminution requires phrase capture before playback and remains a future capability.

## 4. Implementation invariants

1. Input is a clean monophonic three-to-six-note motif.
2. Transport is running before the first NoteOn and remains running through the tail.
3. The source and all followers use the same transport beat origin.
4. The player plus three Canon followers form exactly four conceptual single-note lines.
5. Every Canon voice uses `PassThrough`, `voiceCount: 1`, no octave Mirror, no voice-leading transform, and no intra-voice harmony stack.
6. Allowed ratios are exactly `1:1`, `3:2`, and `2:1` in this baseline; no arbitrary float controls are presented as historical mensurations.
7. Event order is unchanged. For source offset `x` and duration `d`, follower timing is `anchor + r*x` and `r*d`.
8. No ratio below `1.0` is used with uncaptured live input.
9. Followers derive independently from the player event stream; the preset does not claim a true two-source double canon.
10. Generic modal interchange is disabled so the source contour/transposition remains predictable.
11. Fixed diatonic transpositions place lines at octave/fifth-related registers; they are product-safe spacing, not a transcription of one Mass section.
12. Canon lane Hold is explicitly `Forever` so already computed proportional tails can finish after source release; ratio and motif length bound the tail.
13. `Forever` does not override hard cleanup: Stop, Reset, Panic, disable, preset replacement, and device teardown clear sounding and pending events.
14. A new motif begins only after the prior tail has ended and at least two transport beats of silence have established a fresh anchor.
15. Every emitted NoteOn receives a matching NoteOff; natural completion and every hard boundary end with zero active/pending/held state.
16. Applying the preset preserves tonic, BPM, time signature, devices, guitar state, routing, sound, master level, mute/solo, hosted plugins, and transport running state.
17. UI copy says “scalar proportional Free Imitation approximation,” never “authentic mensuration,” “four prolations,” or “Ockeghem counterpoint.”

## 5. Evolution model

```text
ARMED / IDLE
  requirement: transport running, queues empty
  first clean NoteOn sets one phrase anchor
    ↓
STATE (source + 1:1 octave line)
  capture/play 3–6 exact ordered notes
  3:2 and 2:1 followers progressively separate from source timing
    ↓ final source NoteOff; lane Hold = Forever
UNFOLD
  performer is silent
  3:2 line completes before 2:1 line
  no new motif or config mutation is admitted
    ↓ slowest scheduled NoteOff
CLEAR
  active/pending/held counts all zero
  wait >2 transport beats so next input receives a fresh anchor
    ↓
VARIATION
  replay the same rhythm with one change only:
  whole-cell transposition, final degree, or velocity
    ↓
UNFOLD → CLEAR
```

Across sections, the performer can retain the exact rhythm while changing one contour degree or transposing the whole cell. The baseline does not automatically widen canonic intervals or detect cadences. Those are historical observations reserved for a future phrase/timeline model.

## 6. Caricatures and rejected shortcuts

Reject:

- “one tune at four arbitrary speeds” as a literal account of the Mass;
- four independent timers or metronomes that can drift;
- a ratio below `1.0` pretending to anticipate live notes;
- the existing heterogeneous `1×/2×/0.5×/4×` Mensuration Quartet template with StrictCounterpoint, BachChorale, ContraryMotion, and serial references;
- each follower emitting a chord stack;
- four unrelated keys or random modal borrowing;
- continuously adjustable rates labeled as the four Renaissance prolations;
- mensuration signs described as BPM or modern time signatures;
- continuous dense input, sustain-pedal smear, or overlapping guitar strings;
- changing ratio, transposition, mode, BPM, or voice vector under sounding notes;
- calling the Chigi source an autograph or sole historical realization;
- dismissing the resolved Vienna witness as an error or simplification;
- Ockeghem portrayed only as a mathematical eccentric;
- claims of exact *Missa prolationum* reconstruction, Ockeghem counterpoint, cadence plan, ficta, or historical tactus.

## 7. Exact Contrapunk mapping

### Metadata

```ts
{
  id: '04-mensuration-web',
  name: 'Mensuration Web',
  family: 'classical',
  tags: ['renaissance', 'proportional-canon', 'free-imitation', 'transport'],
  builtIn: true,
  result: 'One short motif unfolds as four single-note lines at locked 1:1, 3:2, and 2:1 timing relationships.',
  references: [{
    name: 'Johannes Ockeghem',
    context: 'Bounded procedural reference to the paired mensuration canons of Missa prolationum.'
  }],
  researchStatus: 'approved',
  requirements: ['free_imitation']
}
```

### Performer guidance

```ts
{
  prompt: 'Play one exact three-to-six-note line, clean and even, then leave a wide silence and listen as the same shape opens at locked proportional rates.',
  input: 'motif',
  articulation: 'Clean non-legato or soft tenuto with deliberate releases; no pedal, slides, trills, or overlapping strings.',
  density: 'One note per beat at first; three to six source attacks total.',
  space: 'Stay silent until the 2:1 tail ends, then wait at least two more transport beats.',
  tempo: '66–84 BPM; 72 BPM is the test default.',
  transportRequired: true
}
```

### Arrangement config

```ts
{
  harmony: {
    scaleMode: 'Dorian',
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
      form: 'free_imitation',
      holdMode: { kind: 'forever' },
      voices: [
        {
          delayBeats: 0,
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
        },
        {
          delayBeats: 0,
          transposeDegrees: 4,
          timeRatio: 1.5,
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
          delayBeats: 0,
          transposeDegrees: -4,
          timeRatio: 2,
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

The user's current tonic remains authoritative. `Dorian` is an implementation default, not a historical analysis of the entire Mass. Unity role gains avoid silently requiring the `role_mix` capability.

## 8. Approximation statement

> Mensuration Web is a scalar proportional Free Imitation effect inspired by the coordinated canonic procedure of Ockeghem's *Missa prolationum*. It stretches one live motif at three locked ratios on a shared transport. It does not reconstruct the Mass's two written lines, mensural notation, binary/ternary note-value rules, ficta, canonic interval plan, cadences, tactus, or historical performance.

This statement remains visible after Apply.

## 9. Performer contract

### Short prompt

**Play one exact three-to-six-note line, clean and even, then leave a wide silence and listen as the same shape opens at locked proportional rates.**

### Expanded guidance

- Start transport before the first note and leave it running through the tail.
- Use one note at a time around C4–A4 on keyboard or G3–E5 on clean guitar.
- Play 3–6 notes, chiefly stepwise, with at most one clear third/fourth.
- Start at 72 BPM with one attack per beat; use roughly 0.55–0.8 beat moving notes and a slightly longer final note.
- Use moderate, even velocity and deliberate NoteOff gaps.
- After the source ends, do not fill the silence. Identify the 1:1, 3:2, and 2:1 line spacing.
- Do not begin a second cell until the slowest generated NoteOff and at least two additional beats of silence.
- Keyboard: no chords or sustain pedal.
- Guitar: clean fretted attacks, active muting, one string sounding, no open-string overlap, bends, slides, harmonics, scrapes, or hammer-on/pull-off flurries during acceptance.
- For the second statement, preserve the exact rhythm and change only one property.

## 10. Acceptance examples

Examples use novel abstract scale degrees, not Ockeghem melodies.

### A. Exact proportional schedule

At 72 BPM, source attacks `1, 2, 4, 3` at transport offsets `0, 1, 2, 3`, each duration `0.75` beat.

Expected attacks:

- source `1:1`: `0, 1, 2, 3`;
- octave follower `1:1`: `0, 1, 2, 3`;
- fifth follower `3:2`: `0, 1.5, 3, 4.5`;
- lower-fifth follower `2:1`: `0, 2, 4, 6`.

Expected durations are `0.75`, `0.75`, `1.125`, and `1.5` beats respectively. The last slow follower releases at beat `7.5`. Timing tolerance is an implementation measurement, not historical evidence.

Assertions:

- exactly four source NoteOns and four source NoteOffs;
- each follower emits exactly one note per source event;
- no follower emits a harmony stack;
- pitch order/contour is preserved after constant diatonic transposition;
- no new source input occurs before beat `9.5`;
- all active/pending/held state is zero after natural completion.

### B. Causality rejection

Configure a hypothetical ratio `0.5` and feed source offsets `0, 1, 2` live.

Expected: preset validation rejects this operational config. The engine must not market immediate late emissions at source beats `1` and `2` as attacks at unavailable past beats `0.5` and `1`.

### C. Hard-boundary cleanup

Play the four-note cell, then Stop transport at beat `3` while 3:2 and 2:1 events remain pending.

Expected:

- transport Stop issues the established All Notes Off/runtime reset path;
- sounding Canon notes release;
- future pending Canon NoteOns never appear after restart;
- pending_on, pending_off, held entries, and active note counts become zero;
- repeated Stop/Panic is idempotent.

### D. Preset replacement and environment preservation

Set non-default tonic, BPM, time signature, devices, routing, sound, master level, mute/solo, plugin state, and running transport. Apply Mensuration Web while no notes are active.

Expected: one transactional Panic/reset occurs; only the arrangement fields above change; every listed performance-environment value and transport running state remains unchanged.

## 11. Research traceability

- `history.md` — independent historian/primary-source report. Principal evidence: DIAMM records and Vatican facsimile for Chigi C.VIII.234; DIAMM Vienna Cod. 11883; Watson/Long's “The Other Missa Prolationum”; Ricercar/CESR documentary biography. High confidence in source locations, double-canon correction, terminology boundaries, and unresolved chronology.
- `theory.md` — independent theory/temporal report. Principal evidence: Blue Heron analytical program; *Journal of Musicology* source study; Music Theory Online mensural explanation; Cambridge fifteenth-century notation history; local CanonLane/template audit. High confidence in shared-clock/rational-relation invariants and current scalar gaps.
- `performance.md` — independent performer/HCI report. Principal evidence: DIAMM, Schubert, DeFord, Musica Nova Lyon, MIDI Association, professional guitar-tracking guidance, and local lifecycle code. High confidence in transport, source density, silence, and cleanup contract; BPM/velocity/timing tolerances are interaction defaults.

Unresolved but non-blocking for this explicitly bounded baseline:

- exact composition date and occasion;
- exact section-by-section mensuration signs, rests, canonic intervals, ficta, and performance equivalences;
- two independent captured source lines;
- rational-time data types and a true mensural event interpreter;
- cadence and across-section interval state;
- measured latency/jitter across all four surfaces.

These gaps block a *Missa prolationum* reconstruction claim, not the approved scalar Free Imitation approximation.

## 12. Decision

`ready_for_implementation`

Activation requires the exact single-note PassThrough config, no live ratio below `1.0`, explicit `Forever` tail copy, running-transport gating, deterministic ratio/lifecycle tests, and the visible approximation statement. If any surface cannot run Companion/Canon lanes, the draft remains visible but Apply stays capability-locked there.
