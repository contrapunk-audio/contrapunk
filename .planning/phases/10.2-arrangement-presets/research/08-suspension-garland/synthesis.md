# Synthesis: Preset 08 — Suspension Garland

**Decision:** `operational_bounded_approximation`

**References:** Johann Joseph Fux's fourth-species pedagogy and selected Palestrina four-voice suspension practice

**Activation:** approved after the shared Species IV lifecycle correction in `135042c`

## 1. Referenced scope

Suspension Garland has a coherent but narrow two-reference intersection:

- Fux's *Gradus ad Parnassum* (1725), Part II fourth-species exercises, especially the two-part material represented in the Mann translation at pp. 55–63 and later multi-part ligature discussion from p. 94;
- selected mature four-voice Palestrina sacred sources and corpus evidence, including the 1587 motet book context, Hanson's forty four-voice Mass corpus, and Anders/Inden's smaller 36-Agnus computational sample.

Fux supplies an intentionally isolated pedagogical grid. Palestrina's repertoire mixes suspensions with imitation, passing/neighbor motion, variable density, text pacing, and cadences. Fux wrote roughly 130 years after Palestrina's death; his “Aloysius” is an idealized authority, not documentary access to Palestrina's workshop.

The preset may reference prepared suspension procedure shared by pedagogy and repertoire. It may not claim “Fux equals Palestrina,” continuous historical fourth species, an authentic Palestrina generator, or the myth that Palestrina saved polyphony at Trent.

## 2. Style thesis

The audible identity is causal and temporal:

```text
consonant preparation
→ same logical pitch retained while another voice changes
→ metrically accented contextual dissonance
→ suspended voice descends one diatonic step
→ consonant resolution
→ optional cadential perfection
```

In a strict binary teaching baseline, preparation occupies the weaker phase, the retained dissonance the stronger phase, and resolution the following weaker phase. Repertoire practice is freer and can include chains, consonant syncopation, differing contrapuntal rhythms, multiple suspension classes, and a final perfection phase.

“Across strong beats” must mean that a prepared pitch is retained into an accent and resolves afterward. It must not mean any long note crossing a UI barline, any half-beat delay, or a resolution occurring on the accent itself.

## 3. Research convergence and disagreements

All three reports agree on:

- consonant preparation before the dissonance;
- one logical held pitch across the boundary, not a duplicate attack;
- dissonance caused by oblique motion when another voice changes;
- normally downward diatonic-step resolution to consonance;
- transport and metric phase as essential;
- a sparse two-voice diagnostic baseline before denser chains;
- ordinary NoteOn/NoteOff ownership and hard cleanup independent of Hold;
- current code not being honest enough for activation.

The historical report supports recurrent, not continuous, Palestrina suspension practice. Corpus evidence strongly favors 4–3 over 7–6 in the cited four-voice Mass set, while also documenting 9–8, 2–3/2–1, doubles, and exceptions.

The theory report adds a Renaissance cadence correction: resolution can lead to a later perfect goal; chains may defer that perfection. This supports future phrase behavior but is not required for a narrowly worded first correction.

The performer report exposes a current mismatch: instructions can make NoteOn-driven behavior more audible, but performance technique cannot turn it into a true weak→strong→weak scheduled hold contract.

## 4. Invariants required before activation

1. Preparation is consonant against every structurally relevant sounding voice.
2. Preparation starts before the dissonant accent.
3. Preparation and suspension use the same logical MIDI note ownership; no retrigger occurs at the suspension boundary.
4. Another voice changes while the patient pitch remains, creating approved contextual dissonance by oblique motion.
5. The binary baseline schedules weak preparation → strong suspension → weak resolution from transport, not source-event count.
6. Dissonant suspension resolves down exactly one diatonic step.
7. Resolution is prevalidated as consonant in the sounding texture.
8. A failed candidate falls back to consonant syncopation or no figure; it never emits an unprepared accented dissonance.
9. Initial baseline supports one generated patient line and one player/agent line; density does not hide lifecycle evidence.
10. Meter/subdivision is explicit. If implementation remains 4/4-only, the preset declares and validates 4/4 rather than silently assuming it.
11. Live causality is explicit: preparation is chosen consonantly from the current note; when the future strong-beat note arrives, the Lane validates the dissonance and downward consonant resolution before dispatching that new input. An invalid candidate becomes consonant syncopation or is released before the new attack.
12. A validated normal completion emits one patient NoteOn, no boundary retrigger, then patient NoteOff plus lower-step resolution NoteOn, followed by one resolution NoteOff.
13. Hold either permits the whole atomic figure or cancels it before preparation; it cannot strand a half-figure.
14. Stop, Reset, Panic, disable, preset/species/key change, routing stop, and device loss release sounding notes and clear scheduled phases.
15. No old figure reappears after transport restart or preset replacement.
16. UI copy distinguishes Fux pedagogy, Palestrina corpus practice, and the bounded product study.
17. Tonic, BPM, meter, devices, guitar state, routing, sound, master, mute/solo, plugins, and transport running state remain outside preset ownership. Meter can be a declared requirement without being changed.

## 5. Temporal model required

```text
REST
  valid transport + current source note
    ↓ next weak boundary, choose legal consonant preparation
PREPARED_HELD
  one patient NoteOn; owner and expected strong phase recorded
    ↓ future source NoteOn arrives near that strong boundary
VALIDATE
  approved dissonance + consonant downward step → SUSPENDED
  consonance → CONSONANT_SYNCOPATION
  invalid dissonance → release before dispatching the new source attack
SUSPENDED
  no new patient attack; same pitch remains sounding
    ↓ next weak boundary
RESOLVE
  patient NoteOff + one-step-lower resolution NoteOn
    ↓ no compatible continuation
RESOLVED
  resolution NoteOff
    ↓
REST
```

Any hard boundary transitions to `RELEASE_AND_RESET`, emits required releases through the dispatcher/router, clears queues, and returns to REST.

## 6. Rejected shortcuts

Reject:

- the existing CounterpointLane Species4 half-beat delayed attack;
- treating any delayed or overlapping consonance as a suspension;
- preparing on strong phase and resolving on the next arbitrary NoteOn;
- using an input-event counter as musical time for this transport-required preset;
- returning the same pitch in a new harmony response while calling it a tie;
- pitch-class-only legality without compound/voice relation or full-texture checks;
- hardcoded 4/4 presented as meter-independent;
- timeout by number of player calls rather than transport phase;
- endless uniform suspension saturation;
- block chords, octave Mirror, broad Spread, or reverb as fake suspension texture;
- performer instructions claiming that one held source note automatically advances the FSM;
- Species IV or Fux controls marketed as Palestrina's compositional method.

## 7. Pre-correction code blocker

### `CounterpointState`

The shared state contains `Free → Prepared → Suspended → Resolving` fields and can choose a downward diatonic resolution. However:

- preparation is marked on `is_strong`, not the preceding weak phase;
- suspension is attempted only on a later strong NoteOn call;
- resolution occurs on the next call regardless of weak phase;
- the returned pitch does not encode an actual held-note/tie lifecycle;
- `TieKind`/`CounterpointOutput` do not carry through the active path;
- beat classification hardcodes four beats per bar;
- timeout counts calls, not musical time;
- no cadence/perfection phase exists.

### `HarmonyEngine`

External transport phase is preferred, but the FSM advances only while harmonizing a new NoteOn. The synthetic fallback advances one beat per attack, not elapsed time. A single held input cannot cross a real boundary and trigger a suspension or resolution.

### `CounterpointLane`

The dedicated temporal extension point is the right place to finish the feature, but current Species4 only schedules a consonant pitch at `now + 0.5`. Its own comments describe it as suspension-ready. It does not prepare, retain, create contextual dissonance, resolve, chain, or perfect.

**Pre-correction conclusion:** the delayed-attack behavior could not be activated under the name Suspension Garland. Commit `135042c` replaced that shortcut in the dedicated Lane.

## 8. Completed minimum correction

The correction completed the existing `CounterpointLane` Species4 path rather than adding a preset-specific engine:

1. Use absolute transport beats and a binary half-beat subdivision; do not derive time from player event count.
2. From the current source note, choose and schedule one consonant preparation on the next weak boundary.
3. Retain the same logical note without another NoteOn across the expected strong boundary.
4. When the future source NoteOn arrives, validate the retained dissonance and consonant downward-step result before dispatching that source attack.
5. At the next weak phase, emit preparation NoteOff and downward-step resolution NoteOn only for a validated suspension.
6. Keep consonant motion as consonant syncopation; release invalid dissonance before the new source attack.
7. Emit resolution NoteOff after its bounded duration.
8. Keep one source/figure owner across all phases; make cancellation atomic.
9. Reset/reconfigure through existing Companion/router release paths.
10. Add focused phase-by-phase tests, including cancellation/Stop at PREPARED, SUSPENDED, and RESOLVE.
11. Keep cadence/perfection out of initial Result copy unless implemented; list it as a future extension.

## 9. Approved operational config

```ts
{
  requirements: ['species_counterpoint'],
  play: {
    input: 'single_notes',
    transportRequired: true,
    prompt: 'Play a slow single-note line in 4/4, changing cleanly on the marked beats; leave space to hear a prepared generated note hold into tension and fall by step.',
    tempo: '60–72 BPM; 64 BPM is the test default.'
  },
  config: {
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
      canon: { enabled: false, form: 'free_imitation', holdMode: null, voices: [] },
      counterpoint: {
        enabled: true,
        species: 'Species4',
        transposeDegrees: 2,
        preferAbove: true,
        holdMode: { kind: 'near_future', tail_beats: 2 }
      }
    },
    mix: { input: 1, harmony: 1, canon: 1, counterpoint: 1 }
  }
}
```

The initial correction should use two conceptual lines and no Mirror/Spread. A bounded atomic-figure Hold is preferable; `NearFuture(2)` is only a temporary config if it is proven to preserve the entire scheduled figure.

## 10. Performer contract after correction

**Short prompt:** Play a slow single-note line at 60–72 BPM in 4/4. Change cleanly on the marked beats, leave one full bar of silence every few bars, and listen for a generated note to be prepared, hold into a strong-beat rub, then fall by step.

- one source note at a time in MIDI 55–72;
- 1.5–1.9-beat connected notes with deliberate releases;
- attacks just after displayed target phases, never while stopped;
- keyboard pedal up, no chords;
- clean guitar, one string sounding, active muting, no bends/slides/wide vibrato during acceptance;
- begin with two total lines;
- after 2–4 bars, release and rest a full bar;
- if a figure misfires, stop and Panic rather than adding corrective notes.

## 11. Acceptance required after correction

### Normal figure

In 4/4 at 64 BPM, use an abstract nearby-degree input sequence. Capture event and transport logs.

Expected:

1. weak-phase preparation emits one generated NoteOn at a consonant pitch;
2. next strong phase emits no duplicate patient NoteOn; the same pitch remains and is contextually dissonant after agent movement;
3. next weak phase emits patient NoteOff plus one lower-diatonic-step resolution NoteOn;
4. resolution emits one NoteOff and clears figure ownership;
5. generated NoteOn/NoteOff counts balance by target/channel/pitch;
6. active, pending, and held state returns to zero.

### Cancellation matrix

Repeat with Stop/Panic/disable at PREPARED, SUSPENDED, and RESOLVE.

Expected at every phase: immediate/effective release of every sounded pitch, no future phase attack, empty queues/ownership, no reappearance after the old deadline or transport restart, and idempotent repeated cleanup.

### Environment preservation

Apply only when no notes are active. The preset must not change tonic, BPM, time signature, devices, routing, sound, master, mute/solo, plugins, or transport running state. If the first implementation requires 4/4, non-4/4 surfaces/configurations show a precise unavailable reason rather than changing the meter.

## 12. Research traceability

- `history.md` — Fux primary treatise/facsimile and Mann translation; selected Palestrina score witnesses; Hanson and Anders/Inden corpora; Monson Trent correction. High confidence in the narrow pedagogical/repertoire intersection.
- `theory.md` — Fux-derived fourth-species rules, Palestrina corpus evidence, Morgan's Renaissance cadence/perfection model, and exact code-path audit. High confidence in the correctness blocker.
- `performance.md` — performer contract, metric logging, guitar/keyboard constraints, NoteOn/NoteOff parity, Hold scope, and 45-second exercise. High confidence that current behavior is NoteOn-driven and cannot be repaired by technique alone.

Unresolved:

- edition-stable measure inventory for selected Palestrina motets;
- complete Palestrina suspension distributions by type/phrase/register;
- contextual ficta and text underlay;
- meter-general contrapuntal rhythm;
- cross-surface transport and external MIDI acceptance;
- cadence/perfection implementation.

## 13. Implementation evidence

Commit `135042c` gives Species 4 one explicit monophonic gesture owned by `CounterpointLane`:

- absolute transport schedules preparation at the next weak half-beat;
- the same logical MIDI pitch remains sounding across the expected strong boundary with no retrigger;
- a new same-channel cantus NoteOn within the strong window is checked before the live input is dispatched;
- only approved dissonance classes with a consonant one-step-down result enter `Suspended`;
- the following weak boundary emits ordered `NoteOff(preparation) → NoteOn(resolution)`;
- incompatible early/late/dissonant motion releases the preparation before the new input; compatible consonance remains explicitly a consonant syncopation;
- legato ownership transfers before the old NoteOff;
- Hold deadlines, lane disable, species change, Panic/Stop reset, and backward transport movement cancel future phases and release or externally drain the sounding pitch.

Runnable evidence:

- `cargo test -p contrapunk-companion --lib`: 68 passed;
- `species4_emits_preparation_hold_and_downward_resolution` proves `E on @ 0.5 → no retrigger @ 1.0 → E off/D on @ 1.5 → D off` for the abstract C→F source motion;
- focused tests cover unsounded cancellation, sounded Hold expiry, invalid early motion, disable while suspended, reset after resolution, and transport rewind;
- catalog/persistence tests: 7 passed;
- Svelte check: zero errors and 29 pre-existing warnings.

The live system cannot know the player's future note when preparation begins. Therefore the approved product claim is intentionally opportunistic, not strict guaranteed species generation for arbitrary input. The public approximation text states this boundary. Same-pitch overlapping NoteOns remain outside the accepted clean monophonic contract.

## 14. Decision

`operational_bounded_approximation`

Preset 08 may be approved with the exact config and copy above. It is a transport-scheduled live suspension study: valid source motion yields a real prepared, retained, downward-resolving suspension; other source motion safely yields consonant syncopation or release. It is not an authentic Fux exercise generator or Palestrina model, and cadence/perfection remains a future extension.
