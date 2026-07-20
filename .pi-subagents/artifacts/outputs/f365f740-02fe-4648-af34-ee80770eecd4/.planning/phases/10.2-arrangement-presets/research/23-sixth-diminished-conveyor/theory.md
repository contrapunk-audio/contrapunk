# Preset 23 — Sixth-Diminished Conveyor
## Music-theory and temporal-behavior report

**Research role:** theory and temporal behavior  
**Scope:** Barry Harris’s mature *teaching system* for sixth-diminished harmony and improvisation, as preserved in Harris/Howard Rees workshop materials and continuing institute pedagogy. This is not a claim to model Harris’s entire performing style, all bebop, or all uses of diminished harmony.  
**Decision:** **usable only as a bounded static approximation after honest copy/configuration; the current label and catalog promise overstate the implementation.**

## 1. Executive finding

Harris’s sixth-diminished teaching is not merely an eight-note “bebop scale,” and it is not a rule that every successive melody attack must mechanically flip a whole accompaniment between two blocks. It is a generative system joining a major- or minor-sixth chord with its related diminished seventh chord into a **scale of chords**, then creating directed harmonic motion through inversions, borrowed voices, resolutions, surrounding, related dominant families, melody placement, and context-dependent choice of a scale of chords. The Barry Harris Institute’s current curriculum itself separates and then reconnects “Sixth-Diminished Scales,” chord movement, borrowed notes, “The Family,” surrounding, melody harmonization, extra-note rules, and scale maps; that breadth is direct evidence against reducing the method to one scale or one alternation trick. [Barry Harris Institute of Jazz](https://barryharrisinstituteofjazz.org/spring-2026-a-six-part-zoom-series/)

Contrapunk’s `HarmonyMode::BarryHarris` correctly encodes one genuine kernel: in a major collection `[1,2,3,4,5,b6,6,7]`, even indexed degrees form `[1,3,5,6]` and odd indexed degrees form `[2,4,b6,7]`; its minor form lowers 3. The implementation builds same-parity four-note pitch-class sets and applies a fixed drop-2 transform. That is an **honest bounded approximation of unborrowed scale-of-chords parity for in-collection notes**, not an implementation of the broader Harris harmonic method. It has no temporal harmonic state, ignores beat phase, cannot borrow and resolve individual voices, cannot choose a scale of chords from changing chord context, and passes unsupported chromatic approaches through without harmony. More seriously, the engine inserts the played note in addition to the full four-note voicing, so its tested result is five sounding notes and can octave-double the melody pitch class; that conflicts with both its “4-voice drop-2” documentation and a melody-as-one-voice reading.

## 2. Evidence base and selected-period boundary

### Primary-lineage / authoritative educational evidence

1. **Barry Harris Workshop Video / workbook, produced by Howard Rees.** The product is a filmed Harris master class with Harris’s concepts notated and keyed to the video. Contemporary endorsements describe it specifically as a practical system of chord movement for accompaniment, arranging, and composition, while Paul Berliner characterizes Harris’s teaching as an “expansive generative method” connecting rhythmic, melodic, and harmonic models—not a fixed product. [Howard Rees, *The Barry Harris Workshop Video*](https://jazzworkshops.com/the-barry-harris-workshop-video/)
2. **Barry Harris Institute of Jazz / Howard Rees curriculum.** Rees began studying with Harris in 1978 and has taught the method for more than forty years. The syllabus explicitly includes sixth chords, sixth-diminished scales, key chord movements, borrowed notes, family, surrounding, melody harmonization, extra-note rules, related diminished/family of dominants, and scale maps. [Barry Harris Institute](https://barryharrisinstituteofjazz.org/spring-2026-a-six-part-zoom-series/)
3. **Harris demonstration, “Barry Plays 6th Diminished Scale.”** An academic dissertation cites this direct demonstration while identifying the descending major collection over major-seventh harmony as `1–7–6–b6–5–4–3–2–1`. The video is the primary demonstration; the dissertation provides a stable academic pointer and transcriptional description (p. 15 / Fig. 1.15). [Barry Harris video](https://youtu.be/G1siDXQ92Nw) · [Kenneth D. Gill, University of Michigan DMA dissertation (2024)](https://deepblue.lib.umich.edu/bitstream/handle/2027.42/193224/kgmusic_1.pdf?isAllowed=y&sequence=1)

### Reputable lineage summaries used for operational detail

- **The Barry Harris Companion** describes the scale of chords as the pairing of a sixth chord and its diminished chord; it explicitly warns that this is not merely a “scale in the narrow definition,” and describes borrowing in either direction followed by resolution. [Scales of Chords](https://barrycompanion.com/scales-of-chords/)
- Its **Family** chapter derives four related dominant sevenths by lowering one note at a time from a diminished chord, and a wider family by moving one or two voices by half step; family material is usable in context and eventually resolves back to the harmonic structure. [Family](https://barrycompanion.com/family/)
- Its **Six Chords** and **Chord Context** chapters show why a lead-sheet `maj7`, `m7`, dominant, or `m7b5` symbol does not map mechanically to one identically rooted collection: chords are often re-read as sixth chords or inversions/substitutions, and tonal context decides the applicable scale. [Six Chords](https://barrycompanion.com/six-chords/) · [Chord Context](https://barrycompanion.com/chord-context/)
- Andrew Cochrane’s analytical summary of a Harris workshop excerpt gives the disjoint covers `C6 + Bdim7` and `Cm6 + Bdim7`, and notes the system’s substitutions and chord-scale consequences. It is useful secondary analysis, not a substitute for Harris’s demonstration. [Cochrane Music](https://cochranemusic.com/barry-harris-6th-dim-scale-diminished)
- Matt Otto’s educational analysis describes one Harris chromatic-line procedure: extra half steps are placed so scale degrees fall on downbeats in eighth-note motion, with special handling where the scale already has a half step. This supports beat alignment as a **line-construction rule**, not a universal accompaniment flip-flop. [Matt Otto](https://mattotto.org/barry-harris-chromatic-chord-scales/)

### Boundary

The preset should reference **Harris’s mature, workshop-era pedagogy as taught from the late twentieth century through his final teaching years and transmitted by Howard Rees**, not a selected album, composition, or chronological “Harris style period.” Evidence here supports a pedagogical procedure. It does not justify claims about Harris’s complete pianistic touch, comping on every recording, career evolution, or all bebop practice. Across the selected teaching corpus the stable core is sixth-chord/diminished organization plus movement; later or expanded teaching materials articulate family, borrowing, surrounding, extra notes, and tune-scale mapping as connected tools. The implementation should therefore be marketed as a **sixth-diminished block-voicing study**, not “Barry Harris harmony” without qualification.

## 3. Theory dimensions

### 3.1 Tonal center, modality, chromaticism, and tuning

- **Tonal center is contextual and sectional.** A scale of chords belongs to a harmonic region or lead-sheet function; the Companion explicitly says each section of a tune (for example, a ii within ii–V–I) gets a relevant scale of chords. The root printed in a chord symbol is not always the sixth-chord root used by the method. [Scales of Chords](https://barrycompanion.com/scales-of-chords/) [Chord Context](https://barrycompanion.com/chord-context/)
- **Major sixth-diminished collection:** `[1,2,3,4,5,b6,6,7]`, partitioned into major-sixth `[1,3,5,6]` and related diminished `[2,4,b6,7]`.
- **Minor sixth-diminished collection:** `[1,2,b3,4,5,b6,6,7]`, partitioned into minor-sixth `[1,b3,5,6]` and the same relative-degree diminished set `[2,4,b6,7]`.
- The `b6` is not merely decorative chromaticism: it completes the interlocking four-note collections. Yet chromaticism extends beyond it through borrowed notes, surrounding, family relations, and extra-note line rules.
- **Tuning:** the documented system operates in the twelve pitch classes of equal-tempered jazz instruments. The Companion explicitly frames its family logic within the 12-note octave. Contrapunk’s integer MIDI implementation is adequate for this scoped pitch system; historical or expressive tuning is not a named requirement. [Family](https://barrycompanion.com/family/)

### 3.2 Chord vocabulary, spacing, inversion, doubling, tension/resolution

Core vocabulary:

1. major-sixth and minor-sixth chords and all inversions;
2. the related fully diminished seventh and its inversions;
3. major-seventh understood as major-sixth plus a borrowed diminished note;
4. minor-seventh understood as an inversion of a major-sixth where context supports it;
5. minor-sixth as dominant substitute (“important minor”) in appropriate contexts;
6. diminished-origin families of four dominants and further one-/two-voice half-step derivatives;
7. mixed/borrowed voicings made by taking one or more tones from the partner collection, whose tension calls for resolution.

The invariant is **not a specific closed or drop-2 spacing**. Close position and drop systems are practical realizations; spacing may be close, drop-2, drop-3, inverted, or distributed, provided the scale-of-chords identity and directed resolutions remain audible. Melody harmonization often places the melody in the highest voice of a sixth or diminished voicing. [Scales of Chords](https://barrycompanion.com/scales-of-chords/)

Diminished harmony is mobile tension, not a destination repeated indiscriminately. Borrowing creates mixtures between the two parent sets, and those mixed notes resolve—often by semitone—back into the controlling structure. The Companion’s phrase “move me, resolve the tension” is the clearest operational statement. [Scales of Chords](https://barrycompanion.com/scales-of-chords/)

### 3.3 Scale degree and parity behavior

Parity is a useful computational invariant **within one fixed eight-note collection**:

- indices `0,2,4,6` → sixth-chord family;
- indices `1,3,5,7` → diminished family;
- moving by two collection degrees stays in the same family;
- adjacent collection-degree motion crosses families.

But parity is not sufficient to describe performance:

- repeated or leaping melody degrees need not alternate families;
- a chromatic approach outside the selected eight notes has no parity until interpreted by an extra-note/surrounding rule;
- borrowed voicings deliberately mix the two parities;
- a change of harmonic section changes the relevant tonic/collection;
- strong-beat alignment depends on phrase start, direction, number of notes, rests, ties, triplets, and extra-note rules.

Thus “even = sixth, odd = diminished” is a valid **unborrowed chord-scale classification**, not the whole method.

### 3.4 Borrowing, substitutions, and harmonic movement

Borrowing is central, and absent from the current engine. One or more voices can be taken from the opposite parent chord, creating a mixed sonority that resolves. From a diminished chord, lowering each voice individually generates a four-dominant family; moving voices upward produces minor-sixth relatives, while two-voice moves produce major-sixth relatives. These chords are related by origin but are **not freely interchangeable without context**; the Companion says they have potential to work and should eventually resolve to the harmonic structure. [Family](https://barrycompanion.com/family/)

Movement therefore includes:

- inversion-to-inversion scalar chord motion;
- one-voice or two-voice semitone borrowing and return;
- sixth → diminished → neighboring sixth motion;
- family-derived dominant/minor-sixth substitution under a retained or contextually chosen bass;
- melody harmonization in which top-note demands select an inversion and intermediate movement;
- change of scale map at harmonic-section boundaries.

### 3.5 Voice motion and melodic behavior

- Full unborrowed chord-scale motion can create parallel/similar motion among chord voices, but Harris movement is not reducible to rigid parallel planing.
- Borrowing produces oblique and contrary possibilities: common tones can stay, one voice can move by semitone, or paired voices can move in opposite directions around a diminished position.
- Voice leading should favor small directed motions and retain common tones where the chosen movement permits. A diminished block should normally lead somewhere rather than retrigger unchanged as an isolated color.
- The source melody remains an independent contour and is commonly the upper voice in melody-harmonization practice. Arbitrarily forcing it into alto/bass while adding a duplicate top voice changes the perceptual role.
- Melodic Harris practice uses scales, arpeggiations, thirds, extra notes, surrounding, and characteristic phrase patterns; merely ascending/descending the collection is preparatory material, not the final style. The Institute lists “5–4–3–2” phrases, surrounding, and scale maps separately from the sixth-diminished topic. [Barry Harris Institute](https://barryharrisinstituteofjazz.org/spring-2026-a-six-part-zoom-series/)

### 3.6 Rhythm, meter, subdivision, swing, accent, and phase

The collection’s eight-note organization can align stable tones with strong subdivisions in continuous eighth-note lines. In one documented extra-note procedure, chromatic insertions preserve scale-degree/downbeat alignment, with exceptions around existing semitones. [Matt Otto](https://mattotto.org/barry-harris-chromatic-chord-scales/)

Important limits:

- This is **not** proof that every odd eighth-note onset should trigger diminished accompaniment.
- Alignment reverses if the line starts offbeat or on a different degree; rests, ties, triplets, enclosures, repeated notes, and changing direction alter it.
- Swing feel is essential to a convincing bebop realization but is microtiming/articulation supplied by performer/transport, not encoded in the pitch collection.
- The harmonic scale can be practiced in quarter notes, half notes, rubato, or chordal rhythm. Beat parity and collection parity are distinct variables.

For this preset, transport may remain optional only if copy says the engine reacts to note choice rather than meter. If the product promises strong-beat stable harmony and weak-beat passing harmony, it must require transport and add actual beat-aware state; current `BeatPhase` is ignored.

### 3.7 Phrase length, density, silence, cadence, and formal pacing

No authoritative evidence supports a fixed Harris phrase length or a mandatory density curve for this one technique. A useful bounded arrangement should nevertheless avoid a constant five-note block on every fast eighth note. A musically plausible phrase trajectory is:

1. establish the applicable sixth sonority or a sparse inversion;
2. introduce passing diminished or a borrowed voice during continuation;
3. increase movement selectively, not at every onset;
4. resolve borrowed/diminished tension before a breath or structural arrival;
5. thin or sustain at cadence rather than ending on an arbitrary diminished block.

At a cadence, stable sixth material, a simplified sixth/triad, or a contextually appropriate family resolution is preferable. Silence is structurally useful: it separates scale-of-chords gestures and allows tension/resolution to register. The present engine has no phrase detector, cadence state, density control, or silence-triggered reset beyond ordinary note lifecycle.

### 3.8 Register, texture, articulation, and dynamics

- **Register:** piano/guitar mid-register voicings make semitone inner motion audible; avoid very low close diminished blocks. A melody-harmonization texture normally keeps melody clearly on top.
- **Texture:** three or four actual voices are sufficient to reveal movement; a fifth note may be a deliberate bass or melody doubling, but it should not arise accidentally from inserting the source plus an already complete four-note voicing.
- **Articulation:** connected but clearly re-attacked voicings suit moving chord scales; selective legato/common-tone retention better exposes borrowed voice resolution. Fast per-note five-note blocks produce density rather than clarity.
- **Dynamics:** tension may receive a slight accent and resolution release, but accent is optional; the identity lies in pitch/voice movement, not a fixed dynamic envelope.

### 3.9 Layer interaction

A useful arrangement has distinct roles:

- source line supplies melody, rhythm, approaches, rests, and phrase direction;
- harmony layer chooses an inversion with the source as a real chord voice, introduces bounded movement/borrowing, and resolves;
- an optional bass layer states or retains the contextual root; it should not automatically follow every inversion;
- drums/transport supply swing phase but should not mechanically dictate pitch family unless beat-aware rules are enabled.

Contrapunk currently has only source-plus-immediate-block behavior for this mode. Companion Canon/Counterpoint lanes add unrelated delayed/counterpoint processes, not Harris borrowing or harmonic context, and should remain off for the honest baseline.

## 4. Testable stylistic invariants

1. **Collection invariant:** major mode pitch classes are `[0,2,4,5,7,8,9,11]`; minor mode changes only `3` to `b3` (`[0,2,3,5,7,8,9,11]`).
2. **Unborrowed partition invariant:** `[1,3,5,6]` is the sixth chord; `[2,4,b6,7]` is the related diminished seventh.
3. **Parity invariant:** an unborrowed four-note voicing uses one parity only; a borrowed voicing may mix parity but must have a named source and resolution.
4. **Resolution invariant:** a diminished or borrowed tension is not treated as an equally final static color at every phrase ending.
5. **Context invariant:** a new harmonic region may select a new sixth-chord root/quality; printed chord root alone is insufficient for all symbols.
6. **Melody-role invariant:** in melody-harmonization mode, the source pitch is one of the chord voices and remains perceptually identifiable, normally as top voice—not an extra fifth voice with an unplanned pitch-class duplicate.
7. **Movement invariant:** neighboring chord-scale positions or borrowing should yield bounded semitone/stepwise inner motion; wholesale arbitrary revoicing between every onset is misleading.
8. **Rhythmic invariant:** strong/weak placement is established by actual beat phase and line construction; collection index must never be presented as meter by itself.
9. **Chromatic invariant:** an outside approach must either receive an explicit surrounding/extra-note interpretation or be described as unsupported; silent loss of the harmony layer is not a completed Harris behavior.
10. **Lifecycle invariant:** each NoteOff releases exactly the notes emitted for that NoteOn; preset replacement/Panic leaves no sustained voices.

## 5. Allowed variation

- major versus minor sixth-diminished collection;
- tonic and sectional scale-map changes;
- close, inversional, drop-2/drop-3, guitar- or piano-adapted spacing;
- two to four harmony voices plus an explicitly designed bass/doubling role;
- melody on top or a clearly documented inner-voice exercise;
- straight-eighth practice, swing eighths, quarter-note chord-scale practice, or rubato demonstration;
- ascending, descending, skips, arpeggiation, surrounding, and rests;
- one- or two-note borrowing, provided resolution remains legible;
- sparse movement or denser continuation, provided cadence clears tension;
- root retention versus related-family bass substitution when harmonic context makes it intentional.

## 6. Misleading behaviors / rejection tests

1. Calling the collection simply an “8-note bebop scale” and omitting its two-chord partition, borrowing, family, and movement.
2. Mapping every alternate physical attack to sixth/diminished without considering pitch degree, repeated notes, beat phase, or phrase start.
3. Claiming “chord tones land on every downbeat” for arbitrary input; that only follows under controlled line construction/alignment.
4. Harmonizing arbitrary chromatic approaches by dropping the harmony and still claiming Harris chromatic rules.
5. Producing an unrelated new drop-2 block on every fast eighth note with no common-tone or resolution logic.
6. Ending phrases repeatedly on diminished blocks because the final source degree happened to be odd.
7. Treating every related-family chord as a context-free substitution.
8. Automatically calling all `maj7` symbols same-root major sixth-diminished or all `m7` symbols same-root minor sixth-diminished.
9. Adding source melody plus a complete four-note voicing, accidentally making five voices and doubling its pitch class, while UI says “4-voice drop-2.”
10. Advertising transport-aware alternation: `BeatPhase` currently has no effect.

## 7. Temporal behavior model

```text
SectionContext
  --(select tonic + major/minor sixth interpretation)--> StableSixth

StableSixth
  --(adjacent in-collection melody / chosen passing motion)--> RelatedDiminished
  --(borrow 1–2 partner tones)-----------------------------> BorrowedTension
  --(phrase rest or cadence cue)----------------------------> CadentialStable

RelatedDiminished
  --(step to stable parity / target inversion)--------------> StableSixth
  --(family move justified by bass/chord context)------------> RelatedFamily

BorrowedTension
  --(semitone/common-tone resolution)-----------------------> StableSixth
  --(continued intensification, bounded)---------------------> RelatedFamily

RelatedFamily
  --(harmonic target reached)-------------------------------> StableSixth
  --(section chord/tonic changes)---------------------------> SectionContext

CadentialStable
  --(silence / section boundary)----------------------------> SectionContext
  --(new phrase in same region)-----------------------------> StableSixth
```

**Within phrase:** stable statement → selective passing/borrowing → movement peak → explicit resolution/thinning.  
**Across sections:** retain a scale map while harmony is stable; on a chord-region boundary, reinterpret the symbol as the appropriate sixth chord/important minor/family context, then reset inversion and borrowing state.  
**Current engine’s actual model:** `Input note → fixed collection membership check → even/odd four-note set → fixed drop-2 → octave redistribution → release`. There is no transition memory between attacks, no phrase/section trigger, and no use of beat phase.

## 8. Abstract non-copyrighted input/output examples

Degree names are relative to the currently chosen scale of chords. `S` means a voicing of the sixth set; `D` means the related diminished set; `B(x→y)` means a borrowed voice resolving by semitone.

### Example A — controlled descending eighth-note line (major)

**Input, 4/4, swung eighths:**

| Beat | 1 | 1& | 2 | 2& | 3 | 3& | 4 | 4& |
|---|---|---|---|---|---|---|---|---|
| Degree | 1 | 7 | 6 | b6 | 5 | 4 | 3 | 2 |

**Abstract unborrowed harmonization:**

| Beat | 1 | 1& | 2 | 2& | 3 | 3& | 4 | 4& |
|---|---|---|---|---|---|---|---|---|
| Family | S `[1,3,5,6]` | D `[2,4,b6,7]` | S | D | S | D | S | D |

On the next downbeat, degree `1` returns to `S` and closes the motion. This example demonstrates why alternation can emerge from **adjacent scale-degree motion begun in alignment**. It does not license time-based alternation for arbitrary notes. Current engine can reproduce the pitch-family sequence for every in-collection degree, but not swing, common-tone retention, cadence thinning, or top-melody discipline.

### Example B — borrowing instead of a full block flip (major)

**Input:** degree `3` held from beat 1 through beat 3; short approach `4` at beat `2&`; resolve to `3` on beat 3.

**Desired layer behavior:**

- beat 1: stable `S = [1,3,5,6]`, melody `3` on top;
- beat 2: retain common voices;
- beat `2&`: borrow `b6` from `D` into the voice formerly on `5`, yielding mixed `B(5→b6)` while the melody approaches through `4`;
- beat 3: borrowed `b6` resolves down to `5`, returning to `S`; thin/sustain through the breath.

This is a genuine scale-of-chords movement because only a voice is borrowed and tension resolves. Current engine cannot represent it: it replaces the entire sonority with `D` on degree `4`, has no hold/common-tone ownership, and does not remember a required resolution.

### Example C — minor collection and non-mechanical repetition

**Input, beats:** `1@1, 2@1&, b3@2, b3@2&, 5@3, b6@3&, 6@4, 1@next-1` in minor.

**Expected relationships:** `1→S(minor)`, `2→D`, `b3→S`, repeated `b3→S` again (no forced alternation), `5→S`, `b6→D`, `6→S`, final `1→S` with a cadentially stable release. Current engine gets the parity/repetition relationships for in-collection pitches, but it will emit a dense fresh block for every repeated attack and has no cadence behavior.

### Example D — outside chromatic approach

**Input:** `#1@4& → 2@next-1` against a major-sixth region.  
**Acceptable interpretations:** treat `#1` as an explicit lower surrounding/approach whose harmony is held from or connected into the target, or mark the note unsupported and retain a bounded stable texture.  
**Misleading current outcome:** `#1` is outside `[1,2,3,4,5,b6,6,7]`, so `build_voicing` returns `None` and harmony vanishes for that attack; `2` then brings back a full diminished block. This does not implement Harris’s extra-note rules.

## 9. Exact mapping to current code and ArrangementPresetV2

### 9.1 What exists and is musically defensible

| Requirement | Existing mechanism | Assessment |
|---|---|---|
| Fixed tonic | `HarmonyEngine.key`; presets intentionally preserve tonic | Correct product decision: performer selects tonal center. |
| Major/minor collections | `ScaleMode::BHMajor6thDim`, `BHMinor6thDim` in `crates/contrapunk-harmony/src/config.rs` | Pitch sets are correct for the scoped collections. Documentation calling them merely “8-note bebop scales” is reductive. |
| Same-parity chord sets | `barry_harris::note_parity` and degree stack `[d,d+2,d+4,d+6]` | Correct unborrowed partition kernel. |
| Fixed drop-2 realization | `barry_harris::build_drop2_voicing` | A legitimate bounded voicing option, but not synonymous with the method. |
| Scale guard | `HarmonyEngine::set_mode(BarryHarris)` selects major/minor BH collection based on prior family | Useful safety, though simplistic and order-sensitive. |
| Voice position redistribution | `redistribute_for_voice_position` in `engine.rs` | Register adaptation exists, but can contradict melody-on-top practice and does not solve extra-voice duplication. |
| Exact NoteOn/Off pairing | `active_notes` / `active_port_maps` | Correct lifecycle basis; existing tests verify BH release matching. |
| Preset availability | catalog requirement `['harmony']` | Technically sufficient for the static baseline only. |

### 9.2 Named gaps and findings

1. **HIGH — output voice-count contract is violated.** `HarmonyEngine::harmonize_block_chord` in `crates/contrapunk-harmony/src/engine.rs` prepends the input and then appends the complete four-note voicing, filtering only an exact MIDI-note duplicate. Existing test `test_barry_harris_produces_5_notes` requires five outputs from a mode documented as “4-voice drop-2.” The input pitch class can remain doubled in another octave. `voice_count` is ignored by the block builder except for the global `<=1` bypass. This makes `ArrangementHarmonyConfig.voiceCount` misleading for this mode and can over-densify fast lines.
2. **HIGH — no temporal behavior despite the preset result word “alternate.”** `BeatPhase` is passed into `build_voicing` as `_beat_phase` and unused (`crates/contrapunk-harmony/src/barry_harris.rs`). There is no state linking diminished tension to subsequent resolution, no phrase/cadence state, and no section boundary. The engine classifies each attack solely by scale degree.
3. **HIGH — catalog performance guidance asks for unsupported chromatic approaches.** `build_voicing` returns `None` when `Scale::degree_of` fails; `test_barry_harris_chromatic` explicitly expects a chromatic input to return melody only. The draft at `ui/src/lib/arrangement/catalog.ts` says “chromatic approaches,” implying handled Harris behavior that does not exist.
4. **MEDIUM — fixed drop-2 is presented as the whole named mode.** `HarmonyMode::tooltip` in `config.rs` equates Barry Harris with “4-voice drop-2 voicings.” The fixed spacing is one usable realization, while authoritative curricula center movement, borrowing, family, and melody harmonization.
5. **MEDIUM — melody role can be wrong.** `redistribute_for_voice_position` intentionally permits source melody at any SATB slot. For this preset’s likely melody-harmonization use, the source should normally be one actual top chord voice. Current output begins with source for routing but pitch ordering can place generated tones above it.
6. **MEDIUM — no borrowing/family/context selection.** Neither `barry_harris.rs` nor `ArrangementPresetV2` has per-voice borrowing, related-dominant family selection, chord/bass context, or harmonic timeline. `interchangeEnabled` is unrelated modal interchange and should not be misused as Harris borrowing.
7. **MEDIUM — scale guard is not persistent.** `set_mode` validates the scale, but a later `set_scale_mode` can replace it with any mode while `BarryHarris` remains active; seven-note scales then return melody only. Atomic preset application must ensure final mode/scale pair, and validation should reject incompatible approved config rather than depend on setter order.
8. **LOW — comments/docs conflict.** `modes.rs` still describes “Mode 8” as moving exactly two scale degrees, while `engine.rs` special-cases Barry Harris through the separate block builder. This stale description can mislead future configuration/research.
9. **LOW — tests prove pitch-set parity, not stylistic movement.** Tests in `barry_harris.rs` and `engine.rs` cover valid sets, same parity, scale fallback, five-note output, chromatic passthrough, and NoteOff matching. They do not test top melody, exact voice count, common-tone motion, borrowed resolution, beat effects, phrase endings, or section changes.

### 9.3 Honest baseline ArrangementPresetV2 mapping

A current-capability baseline can use:

```text
scaleMode: BHMajor6thDim          # choose major as one bounded default
mode: BarryHarris
voiceLeadingEnabled: false       # block builder bypasses normal VL processor anyway
voiceLeadingStyle: Free
octaveMode: None
interchangeEnabled: false        # not Harris borrowing
companion.enabled: false
requirements: [harmony]
transportRequired: false         # honest only because the current algorithm is not beat-aware
```

However, activation should wait until the five-note/`voiceCount` behavior is resolved or explicitly surfaced. The best minimal product copy is:

> **Result:** In-scale notes receive a fixed drop-2 voicing from either the tonic major-sixth collection or its related diminished seventh. This is a static scale-of-chords study; it does not perform Harris borrowing, related-family movement, extra-note rules, chord-context mapping, swing, or cadence planning. Outside chromatic notes pass through alone.

Do **not** promise that “sixth chords alternate with diminished passing harmony” for arbitrary input. A safer play prompt is:

> Play a connected ascending or descending line using only the displayed eight-note collection; begin and end on a tonic-sixth tone, use repeated notes and rests, and keep the tempo moderate enough to hear each block resolve. Chromatic approaches currently pass through without generated harmony.

Suggested bounded parameters after the voice-count defect is fixed: source as soprano, four total voices, no extra octave duplication, moderate 72–112 BPM, one short 4–8-note line followed by 1–2 beats of space. Fast bebop eighths should not be the default while every attack emits a full block.

### 9.4 Capabilities required for a fuller model

Named gaps should map to existing planned capabilities rather than preset-specific code:

- **`harmonic_timeline` / chord-context input:** choose scale-of-chords root/quality across sections and cadence targets;
- **new Harris borrowing/movement strategy** (not modal interchange): per-voice partner-tone borrowing, common-tone retention, explicit resolution state, inversion selection with source-on-top;
- **phrase/intensity or pattern lane:** bounded attack/continuation/cadence density and tension scheduling;
- **actual beat phase / transport:** only if strong/weak alignment is promised;
- **stable bass/group role:** retain context root or choose related-family bass independently of upper inversion;
- **chromatic approach/surrounding rules:** explicit extra-note parsing rather than passthrough;
- **articulation/dynamic controls:** optional, for connected inner motion and tension accents.

## 10. Honest-approximation verdict

**Yes, but narrowly:** the current engine is an honest approximation of the **unborrowed pitch-class partition** of one fixed major/minor sixth-diminished collection and one fixed drop-2 realization. This is supported by correct interval sets and parity tests.

**No, under its present broad label/result:** it is not an honest approximation of the temporal Harris method if described as generating chord movement from connected bebop lines. It omits the very features authoritative lineage teaching places beside the collection—movement, borrowing, family, surrounding, melody harmonization, extra-note rules, and scale maps—and it adds an undocumented fifth sounding voice. Preset 23 should remain research-pending/blocked until copy is narrowed and voice-count semantics are fixed. No new lane is needed for the static baseline; a full Harris movement model should wait for a reusable stateful voicing strategy and harmonic timeline rather than a one-off preset branch.

## 11. Confidence and competing interpretations

| Claim | Confidence | Basis / disagreement |
|---|---|---|
| Major/minor collections and sixth/diminished partition | High | Direct-lineage curriculum, Harris video pointer, multiple consistent educational sources, code sets. |
| Borrowing and resolution are central, not optional trivia | High | Institute syllabus and Companion’s scale-of-chords explanation. |
| Family of four dominants derives from lowering each diminished voice | High | Companion and Institute curriculum agree. |
| Fixed drop-2 is only one realization | High | Method sources discuss scales of chords, inversions, borrowing, melody harmonization; none defines the whole method as fixed drop-2. |
| Chord tones on downbeats | Medium, conditional | Supported for controlled eighth-note/extra-note exercises; misleading as a universal rule. Different pedagogues explain this using “bebop scale” terminology, but Harris lineage frames a larger chord/scale method. |
| Melody normally on top | Medium-high for melody harmonization | Companion explicitly describes melody notes in highest position; Harris may also teach inner-voice and accompaniment exercises, so top placement is a preset choice, not universal law. |
| Phrase/density/cadence arc proposed here | Medium-low as historical claim; high as safe product behavior | Operational inference from tension/resolution and live usability, not a fixed Harris form prescription. |
| Exact voicing/articulation/register | Medium-low | Instrument-dependent; preserve variation and avoid claims of unique Harris touch. |

**Competing interpretations:**

1. Some jazz pedagogy calls the major collection a “bebop major” or treats the extra note chiefly as a downbeat-alignment device. That lens is useful for line construction but too narrow for this named preset because Harris’s own teaching lineage joins it to a scale of chords, borrowing, families, and movement.
2. Some implementations harmonize every collection degree with alternating sixth/diminished inversions. That is a valid foundational exercise and exactly the current kernel, but not evidence that live accompaniment should retrigger a complete block on every arbitrary note.
3. Some guitar/piano adaptations prioritize drop-2 because it is playable and clear. This is an acceptable texture choice; the disagreement is only with presenting drop-2 as the definition of Harris harmony.
4. “Minor sixth-diminished” may be explained as melodic minor plus `b6`, harmonic minor plus natural `6`, or minor-sixth plus related diminished. These are pitch-set equivalents; the last is the interpretation most relevant to Harris chord movement.

## 12. Sources kept and dropped

### Kept

- [Barry Harris Institute of Jazz — Howard Rees six-part curriculum](https://barryharrisinstituteofjazz.org/spring-2026-a-six-part-zoom-series/) — authoritative lineage; shows the method’s connected scope.
- [Howard Rees — *The Barry Harris Workshop Video*](https://jazzworkshops.com/the-barry-harris-workshop-video/) — direct Harris master-class provenance and pedagogical framing.
- [Barry Harris — “Barry Plays 6th Diminished Scale”](https://youtu.be/G1siDXQ92Nw) — primary demonstration, corroborated/cited by the Michigan dissertation.
- [Kenneth D. Gill, University of Michigan DMA dissertation (2024)](https://deepblue.lib.umich.edu/bitstream/handle/2027.42/193224/kgmusic_1.pdf?isAllowed=y&sequence=1) — academic transcriptional pointer, Fig. 1.15 and p. 15; limited theoretical depth but useful provenance.
- [Barry Harris Companion — Scales of Chords](https://barrycompanion.com/scales-of-chords/) — concrete collection, borrowing, resolution, and melody-top claims.
- [Barry Harris Companion — Family](https://barrycompanion.com/family/) — operational half-step family derivations and contextual-resolution caveat.
- [Barry Harris Companion — Six Chords](https://barrycompanion.com/six-chords/) and [Chord Context](https://barrycompanion.com/chord-context/) — prevents simplistic chord-symbol/root mapping.
- [Andrew Cochrane — Sixth Diminished Scale](https://cochranemusic.com/barry-harris-6th-dim-scale-diminished) — transparent secondary analysis of the workshop excerpt and major/minor disjoint covers.
- [Matt Otto — Chromatic Chord Scales](https://mattotto.org/barry-harris-chromatic-chord-scales/) — reputable practitioner analysis of conditional downbeat/extra-note behavior.

### Dropped or used only as leads

- Generic “fully explained” YouTube summaries, SEO guitar pages, podcasts, Reddit, Music Stack Exchange, Scribd mirrors, and Slideshare uploads — derivative, weak provenance, inaccessible, or redundant.
- The Jazz-Hitz PDF search hit — fetch returned no usable article text; no claims depend on it.
- Product testimonials on the Howard Rees page — not used as theoretical evidence except the clearly attributed Berliner excerpt and description of the master-class/workbook format.
- General bebop-scale sources — excluded because they encourage the exact caricature the task prohibits.

## 13. Residual research gaps

- Full timestamped access to the commercial Harris/Rees workshop and workbook would improve precision on drop systems, borrowing sequences, and phrase demonstrations.
- The primary YouTube demonstration was identifiable and academically cited, but its transcript could not be extracted in this environment; no timestamp-specific spoken quotation is asserted.
- Evidence does not justify a unique fixed tempo, phrase length, register, articulation, or dynamic curve for the entire method.
- A broader corpus of Harris performances/transcriptions would be required to compare teaching exercises with his recording practice; that is deliberately outside this pedagogical preset boundary.

## 14. Review findings

- **HIGH — `crates/contrapunk-harmony/src/engine.rs` (`harmonize_block_chord`, test `test_barry_harris_produces_5_notes`):** documented four-voice mode emits five notes and ignores configured `voice_count`; accidental octave pitch-class doubling/density risk.
- **HIGH — `crates/contrapunk-harmony/src/barry_harris.rs` (`build_voicing`):** `_beat_phase` is unused; no temporal alternation, tension state, or metrical behavior exists.
- **HIGH — `ui/src/lib/arrangement/catalog.ts` preset 23 draft vs `engine.rs::test_barry_harris_chromatic`:** UI asks for chromatic approaches while engine intentionally returns melody-only for outside-collection notes.
- **MEDIUM — `crates/contrapunk-harmony/src/config.rs` (`HarmonyMode::tooltip`, Barry Harris scale docs):** reductive “8-note bebop scale / 4-voice drop-2” wording overclaims one voicing exercise as the method.
- **MEDIUM — `engine.rs` + `barry_harris.rs`:** no borrowing, family, harmonic context, common-tone voice movement, phrase/cadence, or section evolution; static parity is the only accurate scope.
- **LOW — `crates/contrapunk-harmony/src/modes.rs` module docs:** stale Mode 8 description conflicts with the engine’s special block-voicing dispatch.

**Recommended synthesis decision:** `needs_more_research` only if exact historical workshop examples are required; otherwise research is adequate and implementation decision should be **blocked pending voice-count fix and honest narrowed copy**, then ship only the bounded static study.
