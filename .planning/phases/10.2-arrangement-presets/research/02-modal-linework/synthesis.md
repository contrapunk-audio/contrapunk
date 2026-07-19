# Synthesis: Preset 02 — Modal Linework

**Decision:** `ready_for_implementation` as a bounded note-against-note arrangement, not an exact Palestrina simulation  
**Reference:** Giovanni Pierluigi da Palestrina  
**Research pack:** three independent reports plus this parent synthesis

## 1. Referenced scope

Modal Linework references Palestrina's mature, late-**published** four-voice Latin sacred polyphony, principally the 1584 *Motettorum quatuor vocibus liber secundus* and *Sicut cervus / Sitivit anima mea*, corroborated by the four-voice *Missa Aeterna Christi munera* in *Missarum liber quintus* (1590). The 1563/64 first four-voice motet book is comparative evidence for persistent imitative organization, not the preset's primary period.

Publication date is not silently treated as composition date. The preset does not claim to represent Palestrina's whole career, larger-voice Masses, Song of Songs motets, secular repertory, Latin text setting, historical tuning, or one universal late-Renaissance performance style.

## 2. Style thesis

The reference sound comes from singable lines coordinated contrapuntally: mostly conjunct contours, balanced leaps, staggered or changing participation, imperfect consonances that keep motion alive, contextual passing/neighbor tones and prepared suspensions, and contrapuntal cadences that thin and settle the texture. Four voices name available parts, not four notes sounding continuously.

Contrapunk's first operational version deliberately captures only the safest synchronous slice: every clean monophonic input receives a close four-part, scale-bounded, note-against-note response with strict counterpoint pitch selection and Palestrina-weighted smooth voicing. The performer supplies phrase contour, repetition, density change, and breath. The preset does not claim delayed imitation, independent generated rhythms, historical modal syntax, contextual *musica ficta*, or authentic dissonance scheduling.

## 3. Convergent findings and disagreements

All three reports converge on:

- one clean, singable source line rather than chords;
- predominantly stepwise motion with sparse, balanced leaps;
- changing density, rests, and phrase breaths rather than continuous four-part blocks;
- dissonance controlled by temporal context, not a color-chord switch;
- imitation that becomes freer counterpoint, not endless strict canon;
- modality richer than choosing a modern seven-note scale;
- Palestrina as a bounded reference, not an exact-imitation claim;
- normal NoteOff parity and empty final active-note state as musical acceptance criteria.

The theory report requires transport for genuine scheduled imitation, passing tones, suspensions, and cadential behavior. The performer report says transport is helpful but not required for the current synchronous baseline. These are compatible: **this baseline does not schedule those temporal behaviors**, so `transportRequired` is false. A future phrase-aware Modal Linework Lane would require transport and new capability gating.

The reports also distinguish later species pedagogy from source repertory. `Species1` here is an implementation safety boundary—one response per input—not a claim that Palestrina composed in Fux's first species.

## 4. Implementation invariants

1. The source contract is clean monophonic single notes.
2. `voiceCount` is exactly four including the player; no octave Mirror or duplicated parts.
3. Pitch material is diatonic and modal; generic modal interchange is disabled.
4. Generated motion prefers step/repetition, contrary or oblique motion, common tones, close vocal-style spacing, and stable register identity.
5. Strict counterpoint scoring and strict Palestrina voicing constraints remain deterministic.
6. Automatic call-count suspension is forbidden. A voice-leading style must not invent an unprepared held dissonance merely because several NoteOns occurred.
7. This baseline emits no delayed Canon, Counterpoint Lane, passing subdivision, or cadence queue.
8. Every source NoteOn has matching source-owned NoteOff cleanup for all generated notes.
9. Apply performs the existing transactional Panic/validate/apply-or-rollback path.
10. Applying preserves tonic, BPM, time signature, devices, guitar state, external routing, sound, master level, mute/solo, hosted plugins, and transport running state.
11. UI copy says “bounded arrangement reference” and “note-against-note”; it does not promise independent generated rhythms or historical authenticity.
12. Without new phrase/lane capability, temporal evolution is performer-supplied and explicitly described as such.

## 5. Evolution model

### Operational baseline

```text
BREATH
  first clean source NoteOn
    ↓
OFFER — 4–5 mostly stepwise notes
  synchronous four-part responses; moderate density
  source releases each note cleanly
    ↓ 1–2 beat rest
DEVELOP — related contour from another degree
  one balanced leap; modest register high point
  listen for close inner-line motion
    ↓ full release / one-bar listening gap
SETTLE — 3–4 slower notes toward a stable degree
  lengthen final source note; reduce velocity
    ↓ NoteOff
BREATH — zero active/pending notes
```

The player's rests and contour changes create the phrase/section arc. No code claims generated exposition, imitation, cadence detection, or text-sensitive form.

### Future non-baseline extension

A genuine phrase-aware implementation would need a reusable Companion Lane with stable voice IDs, motif capture, staggered entries, independent onsets/rests, checked dissonance contracts, cadence state, bounded scheduling, and transport-gated cleanup. That work is not smuggled into this preset record.

## 6. Caricatures and rejected shortcuts

Reject:

- four parallel thirds or chord pads on every input;
- continuous full density;
- “choose Dorian = Palestrina” copy;
- generic random modal borrowing;
- endless strict canon;
- Fux species controls marketed as Palestrina's compositional method;
- automatic suspensions based on process-call count rather than preparation, beat, vertical legality, resolution, and perfection;
- all voices moving in the same direction as a global transform;
- wide cinematic Spread or octave Mirror duplication;
- triadic arpeggios, rapid scalar note storms, sustain-pedal smear, or overlapping guitar strings as the expected source;
- the Council of Trent “saved polyphony” myth;
- “serene, impersonal, uniformly soft” as a universal performance prescription;
- claims that a four-part algorithm imitates Palestrina.

## 7. Exact Contrapunk mapping

### Metadata

```ts
{
  id: '02-modal-linework',
  name: 'Modal Linework',
  family: 'classical',
  tags: ['late-renaissance', 'modal', 'four-voice', 'counterpoint'],
  builtIn: true,
  result: 'Four close modal lines follow each note with conservative counterpoint and smooth voice leading.',
  references: [{
    name: 'Giovanni Pierluigi da Palestrina',
    context: 'Bounded reference to mature, late-published four-voice sacred polyphony (1584–90).'
  }],
  researchStatus: 'approved',
  requirements: ['harmony', 'voice_leading']
}
```

### Performer guidance

```ts
{
  prompt: 'Play like one calm singer in a four-part choir: shape a mostly stepwise modal arc, breathe after each short phrase, and leave space to hear the other lines settle.',
  input: 'single_notes',
  articulation: 'Connected one- and two-beat notes with clean releases; one longer destination note.',
  density: 'Four to seven attacks per phrase; avoid chords and rapid runs.',
  space: 'Rest one to two beats between phrases and one full bar before the final phrase.',
  tempo: '60–84 BPM; 72 BPM is the test default.',
  transportRequired: false
}
```

### Arrangement config

```ts
{
  harmony: {
    scaleMode: 'Dorian',
    mode: 'StrictCounterpoint',
    voiceCount: 4,
    voicePosition: 1,
    voiceLeadingEnabled: true,
    voiceLeadingStyle: 'Palestrina',
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
    canon: {
      enabled: false,
      form: 'free_imitation',
      holdMode: null,
      voices: []
    },
    counterpoint: {
      enabled: false,
      species: 'Species1',
      transposeDegrees: 2,
      preferAbove: true,
      holdMode: null
    }
  },
  mix: {
    input: 1,
    harmony: 1,
    canon: 1,
    counterpoint: 1
  }
}
```

`Dorian` is a neutral product default whose tonic remains the user's current tonic. It is not a claim that the selected corpus has one settled modern Dorian identity. Base role gains remain unity, so this preset does not require the `role_mix` capability or silently change master/mute/solo state.

### Implementation prerequisite

`VoiceLeadingStyle::Palestrina` currently activates `SuspensionState`, which holds a changed harmony voice and resolves it after later process calls without proving beat placement, consonant preparation, dissonance legality, or a cadential perfection phase. Remove that automatic call-count behavior from the voice-leading style before activating this preset. Explicit suspension behavior belongs to a beat-aware Counterpoint/Companion capability and preset 08, not a pitch-voicing style flag.

## 8. Approximation statement

> Modal Linework is a bounded note-against-note arrangement inspired by mature four-voice Renaissance vocal counterpoint. It favors close, smooth modal lines and conservative motion. It does not reproduce Palestrina's text setting, historical tuning, contextual ficta, independent vocal rhythms, imitative form, or cadence planning; your contour, pacing, and rests supply the phrase shape.

This statement must remain visible in preset detail after Apply.

## 9. Performer contract

### Short prompt

**Play like one calm singer in a four-part choir: shape a mostly stepwise modal arc, breathe after each short phrase, and leave space to hear the other lines settle.**

### Expanded guidance

- Use one note at a time around MIDI 55–72, preferably within one comfortable octave.
- Play mostly adjacent modal degrees, with at most one third/fourth that reverses direction.
- Use one- and two-beat notes at 60–84 BPM; lengthen the phrase destination.
- Keep velocity moderate and gently arched; do not hard-accent every beat.
- Rest one to two beats after 4–7 attacks.
- Keyboard: no sustain pedal or chords for acceptance.
- Guitar: clean fretted attacks, prompt muting, no double-stops, sympathetic open strings, wide bends, or uncontrolled overlapping sustain.
- During rests, wait for both audible release and empty active-note indicators before continuing.

## 10. Acceptance examples

All examples use abstract scale degrees and relationship assertions, not copied melody.

### A. Offer and cleanup

Input at 72 BPM: `1(2 beats) – 2(1) – 3(2) – 4(1) – 3(2)`, then two beats of rest.

Expected:

- exactly one source note is active at a time;
- each attack produces at most four total arrangement voices including the player;
- no Canon or Counterpoint Lane event is queued;
- outputs remain within four stable voice roles with no Mirror duplicates;
- every generated note from an input is released by that input's NoteOff;
- active and pending states are empty during the rest.

### B. Related development

Input: `2 – 3 – 5 – 4 – 3 – 2`, mostly quarter/half notes. The largest leap immediately reverses direction.

Expected:

- deterministic output for repeated engine state/input;
- strict counterpoint and voicing do not produce a call-count-held suspension;
- no generic interchange note appears;
- copy and UI identify the synchronous result as an approximation, not generated imitation.

### C. Settle and hard lifecycle boundary

Input: `3(1) – 2(1) – 1(4)`, release, then five seconds untouched.

Expected:

- source and generated NoteOff parity;
- zero active notes at the end without requiring Panic;
- Panic, preset replacement, transport Stop, and transport Reset remain independent hard cleanup boundaries and also end at zero active/pending notes.

### D. Environment preservation

Set a non-default tonic, BPM, time signature, input/output devices, sound, master level, mute/solo, plugin state, and transport state. Apply Modal Linework.

Expected: all listed environment values remain unchanged; only the `ArrangementConfig` fields above change.

## 11. Research traceability

- `history.md` — independent historian/primary-source report. Principal evidence: 1584 four-voice motet book and source-linked *Sicut cervus* score; 1590 *Missa Aeterna Christi munera*; Zarlino; Schubert; Monson; Hanson; Anders/Inden; O'Regan. High confidence in scoped publication/corpus and myths rejected; lower confidence in equating late publication with one homogeneous late compositional style.
- `theory.md` — independent theory/temporal report. Principal evidence: selected scores; Anders/Inden; Sigler/Wild; Schubert; Powers; Morgan; DeFord; CRIM. High confidence in controlled dissonance categories, pairwise perfect-motion concern, line independence, phrase-density/cadence trajectory, and current-engine gaps.
- `performance.md` — independent performer/HCI report. Principal evidence: CPDL score record, Open Music Theory, Zarlino/TMI, Cambridge performance-history survey, and local lifecycle code. High confidence in monophonic source, clean releases, rests, failure gestures, and acceptance exercise; numeric BPM/velocity/timing values are interaction defaults rather than historical facts.

Unresolved but non-blocking for this bounded baseline:

- exact composition chronology within the late-published corpus;
- exact encoded-corpus distributions for ambitus, entry lag, density, and phrase length;
- one definitive mode/ficta/tuning mapping;
- full independent-rhythm and cadence generation;
- all-pair perfect-motion guarantees beyond the engine's existing chained counterpoint and voicing checks.

These gaps block an “authentic Palestrina” claim, not the explicitly limited operational preset.

## 12. Decision

`ready_for_implementation`

Activation is conditional on removing automatic process-call suspension from Palestrina voice leading and passing the data/config, environment-preservation, deterministic lifecycle, and zero-active-note checks above. Genuine temporal imitation or controlled suspension remains a separately gated future Lane capability; it is not required or claimed by this baseline.
