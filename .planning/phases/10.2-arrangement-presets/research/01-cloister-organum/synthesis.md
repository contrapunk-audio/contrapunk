# Synthesis: Preset 01 — Cloister Organum

**Decision:** `ready_for_implementation`

**Bounded scope:** a two-voice, synchronous open-interval arrangement inspired by the chant-supported shell of selected late-twelfth-/early-thirteenth-century Notre-Dame organum repertory associated retrospectively with Léonin and Pérotin. It is not organum purum, discant, clausula, triplum, quadruplum, chant generation, or composer imitation.

## 1. Referenced scope

The defensible intersection is the layered *Magnus liber organi* tradition: inherited liturgical chant remains structural, one or more voices elaborate it, and perfect consonances carry structural weight. Anonymous IV associates Léonin with a great book of organa and Pérotin with revision, replacement clausulae, and named three-/four-voice works; Florence MS Pluteus 29.1 preserves distinct two-, three-, and four-voice repertories rather than one homogeneous style.

The implementation deliberately centers only the shell shared by that repertory: a player-supplied, slow, chant-like line in the lowest role receives one upper open interval. “Cloister” remains an image-title; the historical reference is urban cathedral repertory.

## 2. Style thesis

The performer supplies a short modal line whose long foundation and arrival notes matter more than attack density. The engine places exactly one upper voice at a degree-specific perfect fourth, fifth, or octave. The tonic/final receives an octave; interior degrees alternate bounded perfect fourth/fifth relationships that remain inside the configured modern modal approximation. The player creates the temporal shape—foundation, measured movement, held return, breath—through duration, contour, velocity, and silence.

This is intentionally less than historical organum. Real organum purum has a florid upper voice over prolonged chant; discant coordinates active measured voices; Pérotin-associated tripla/quadrupla require independent upper parts and all-voice sonority control. A synchronous two-note MIDI response cannot claim any of those behaviors.

## 3. Invariants

1. Exactly two total voices: the exact played source once, plus one generated upper voice.
2. The player occupies the lowest arrangement slot; no symmetric “around the chant” claim.
3. The generated voice uses only configured anchor-relative perfect intervals: P4, P5, or P8.
4. The tonic/final maps to an octave; the remaining Dorian degrees map to in-collection perfect fourths/fifths.
5. Output is deterministic for every scale degree and chromatic fallback.
6. No chained fourths, power-chord fifth-plus-octave block, triadic stack, functional cadence, later species rules, Mirror duplication, or modal interchange.
7. No autonomous notes, phrase capture, rhythmic quantization, or transport dependency.
8. NoteOff releases exactly the generated NoteOn set; Panic, preset replacement, and parameter changes leave no active notes.
9. UI and public copy call this an open-interval arrangement, not reconstructed organum.
10. The reusable interval-map capability is data-driven and editable; Rust contains no Cloister Organum branch.

## 4. Evolution model

The engine does not infer form. The performer supplies:

`Foundation → narrow movement → held return → full release`

- **Foundation:** begin on or near the selected final and hold long enough to hear the open relationship.
- **Movement:** play two to five mostly neighboring notes with shorter interior durations.
- **Peak:** rise modestly in register or velocity once; do not add chord density.
- **Return:** settle on degree 1 so the configured octave reads as terminal simplification.
- **Breath:** release fully; silence produces no continuation.

Across 30–60 seconds, perform three phrases: establish, expand, return. This performer-authored arc is an interaction contract, not an automatic historical form model.

## 5. Caricatures and rejected shortcuts

Rejected:

- permanent parallel fourths or fifths as “the Notre-Dame style”;
- `[source, +7, +12]` power chords on every attack;
- chained `DiatonicFourths`, whose nonadjacent relationships are uncontrolled;
- `StrictCounterpoint`, which imports later no-parallel-perfect priorities;
- calling synchronous attacks organum purum, discant, clausula, or Pérotin polyphony;
- “Gregorian chant generator,” “authentic medieval tuning,” cathedral-acoustic causation, or composer reconstruction;
- automatic functional harmony, chromatic borrowing, rhythmic modes, drone sustain, or reverb claims.

## 6. Contrapunk mapping

### Reusable capability

Add one generic **Explicit Interval Map** harmony strategy:

- unit: semitone offsets relative to the exact source anchor;
- seven scale-degree entries, each containing a bounded stack of signed offsets;
- one fallback stack for input outside the selected collection;
- maximum seven generated offsets, unique nonzero values, bounded to MIDI range;
- direct anchor-relative generation, never recursive chaining;
- arrangement indices derived from final pitch order so routing remains stable;
- full NoteOn/NoteOff ownership through existing `active_notes` tracking.

The UI must expose the strategy in Setup on supporting surfaces: select the mode and edit each degree/fallback stack. Preset Apply must populate the editor; edits update live state; Save As snapshots the map. Unsupported surfaces must capability-gate the preset rather than silently no-op.

### Exact preset configuration

```text
scaleMode: Dorian
mode: ExplicitIntervals
voiceCount: 2
voicePosition: 1  # source is the lower/tenor role
voiceLeadingEnabled: false
octaveMode: None
interchangeEnabled: false
interval unit: semitones
scale-degree map:
  1 -> [+12]
  2 -> [+7]
  3 -> [+7]
  4 -> [+5]
  5 -> [+7]
  6 -> [+5]
  7 -> [+5]
fallback -> [+7]
Companion: disabled
mix: input 1.0, harmony 0.78, canon 1.0, counterpoint 1.0
transportRequired: false
```

For C Dorian this yields:

```text
C  -> C'  (+12)
D  -> A   (+7)
E♭ -> B♭  (+7)
F  -> B♭  (+5)
G  -> D   (+7)
A  -> D   (+5)
B♭ -> E♭  (+5)
```

Every generated pitch remains in the modern Dorian approximation while the interval class remains perfect. The map key is the source's scale degree, but each stored value is an exact **semitone** offset; it must not be reinterpreted as a recursive or diatonic `+3/+4` transposition, which would create tritones on some degrees.

Capabilities: `harmony`, `interval_stacks`, `role_mix`.

## 7. Approximation statement

**Public approximation:**

> A bounded modern open-interval arrangement inspired by selected Notre-Dame organum repertory associated with Léonin and Pérotin. Your slow monophonic line supplies the chant-like contour and phrase shape; the engine adds one degree-mapped upper fourth, fifth, or octave. It does not generate chant or reproduce organum purum’s florid duplum, discant rhythm, Pérotin’s independent upper voices, medieval tuning, liturgy, notation, or historical performance.

## 8. Performer contract

**Short Play prompt:**

> Play one slow, chant-like note at a time. Hold arrivals, move mostly by step, return to the tonic, and leave a complete breath between phrases.

**Expanded guidance:**

- Use C3–C5, approximately 48–66 BPM for pacing only.
- Play two to five clean single notes per gesture.
- Hold foundation/arrival notes for 2–6 seconds; make connecting notes shorter.
- Begin `mp`, rise only to `mf`, then recede.
- Keep sustain pedal off; guitarists mute unused strings and avoid bends, slides, or ringing open strings.
- Do not play chords, tremolo, rapid scales, dense chromatic runs, or overlapping source notes.
- Wait for the open interval to settle, then release completely and confirm silence before the next phrase.

## 9. Acceptance examples

### Exact degree oracle

With C Dorian and MIDI source degrees `60, 62, 63, 65, 67, 69, 70`, NoteOn output must be exactly:

```text
[60,72], [62,69], [63,70], [65,70], [67,74], [69,74], [70,75]
```

The matching NoteOff sequence must return exactly the same pairs and end with empty active state.

### Chromatic fallback

C♯4 (61), outside C Dorian, returns exactly `[61,68]` using the configured +7 fallback and releases both notes.

### Lifecycle

- held input produces no extra autonomous attacks;
- source release emits one matching release per emitted pitch;
- changing interval-map values raises the established panic/reharmonization boundary;
- preset replacement and Panic clear tracked pitches;
- transport stopped or absent does not change pitch output;
- no single input can exceed the configured stack or MIDI bounds.

### UI round-trip

- Applying Cloister Organum displays its seven map entries and fallback in Setup;
- changing one degree updates subsequent output;
- Save As captures the edited map;
- applying another preset restores/clears the map honestly;
- a surface without explicit interval maps lists the missing capability and cannot Apply.

## 10. Research traceability

Reports:

- `history.md` — manuscript/treatise scope, repertory evolution, attribution limits, and primary evidence.
- `theory.md` — organum-purum/discant/triplum distinctions, temporal invariants, engine audit, and rejected mappings.
- `performance.md` — live monophonic contract, register/tempo/duration guidance, failure gestures, UI copy, and 45-second exercise.

Key sources include Anonymous IV in Reckow/Yudkin; Edward H. Roesner on the *Magnus liber*; DIAMM records for F, W1, and W2; Ernest H. Sanders on consonance/rhythm; Asher Yampolsky on modal/dronal organization; Catherine Bradley on clausulae; Jeremy Yudkin and Penelope Turner on organum-purum rhythm and performance; and the University of Basel modal-notation resource.

Confidence is high for chant foundation, texture distinctions, structural perfect consonance, and the warning against constant parallelism. Confidence is low for exact authorship, absolute tempo, tuning, ficta, and any universal cadence formula. The degree map is a disclosed product approximation chosen for deterministic bounded behavior, not a transcription statistic.

## 11. Decision

`ready_for_implementation`

Implement only the generic editable degree-to-semitone interval map and the two-voice bounded preset above. Do not add phrase inference, medieval rhythmic modes, florid duplum generation, or multi-voice Notre-Dame simulation in this slice. Those remain future capabilities and must receive separate research, lifecycle evidence, and UI controls before any corresponding claim is made.
