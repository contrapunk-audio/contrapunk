# Research: Preset 02 — Modal Linework (music theory and temporal behavior)

## Summary

An honest Palestrina-referenced preset is not “four notes from a modal scale” and not a live application of Fux’s five species. Its audible identity is a succession of independently singable, mostly stepwise lines: staggered imitative entries, consonant structural beats, brief weak-position passing/neighbor dissonances, metrically controlled prepared suspensions, avoidance of parallel perfect intervals, and cadential convergence followed by a breath. The current HarmonyEngine can approximate smooth four-part pitch choice, but a static four-note response per player attack cannot reproduce rhythmic independence, text-sized points of imitation, phrase-level density, or authentic cadential planning; those require Companion scheduling and phrase/motif state.

## Scope and evidentiary boundary

**Preset target.** Late/mature sacred vocal polyphony, principally four-voice motet texture, with checks against larger Mass textures: the published score of *Sicut cervus* (Second Book of Four-Voice Motets, 1584), *Missa Papae Marcelli* (Second Book of Masses, 1567), Palestrina Mass-corpus studies, and Schubert/Lessoil-Daelman’s analysis of the *Je suys desheritee* Kyrie. These works do not justify claims about every sacred genre, every voice count, or Palestrina’s secular madrigals. [*Sicut cervus* score, CPDL/Opera omnia source](https://www.cpdl.org/wiki/index.php/Sicut_cervus_%28Giovanni_Pierluigi_da_Palestrina%29) [*Missa Papae Marcelli* score and source information](https://www.cpdl.org/wiki/index.php/Missa_Papae_Marcelli_%28Giovanni_Pierluigi_da_Palestrina%29)

**Evidence labels used below.**

- **[P] Palestrina evidence:** score observation or corpus/analysis of Palestrina’s music.
- **[R] Renaissance-practice evidence:** contemporary theory or broad Renaissance corpus; relevant context, not uniquely Palestrina.
- **[J] Jeppesen reconstruction:** influential twentieth-century study derived substantially from Palestrina, but still an analytical/pedagogical model.
- **[F] Fux/species pedagogy:** an eighteenth-century teaching sequence, not direct evidence of Palestrina’s compositional process.

Fux’s 1725 species method deliberately isolates rhythmic situations for learners. Modern historical-theory scholarship warns that eighteenth-century Vienna knew little sixteenth-century Roman repertory and that species should not be equated with Palestrina’s actual mixed, text-driven polyphony. [College Music Symposium, “A New Approach to Species Counterpoint”](https://symposium.music.org/volume-21/articles-1752877059/a-new-approach-to-species-counterpoint) Renaissance pedagogy itself included interval-successions, memorized contrapuntal patterns, improvisation, canon and composition—not simply the later five-species ladder. [Schubert, “Counterpoint pedagogy in the Renaissance,” *Cambridge History of Western Music Theory*](https://resolve.cambridge.org/core/services/aop-cambridge-core/content/view/6EC7E10648302A8BF897F0F22B655B74/9781139053471c16_p503-533_CBO.pdf/counterpoint-pedagogy-in-the-renaissance.pdf)

## Findings

### 1. Mode and tonal organization

1. **Modal identity is distributed, not a modern chord loop. [P/R, high confidence]** Use a diatonic collection and a stable modal final, with cadence tones and characteristic melodic ranges reinforcing it. Do not generate functional I–IV–V–I progressions or a root-position chord on every beat. Powers and later modal scholarship show that mode in polyphony involves final, range, cadence plan, cleffing/transposition and contrapuntal behavior; modal labels can conflict with sounding detail, even in Palestrina. [Powers, “Tonal Types and Modal Categories in Renaissance Polyphony,” JAMS 34/3](https://doi.org/10.2307/831189) [Powers, “Modal Representation in Polyphonic Offertories,” *Early Music History* 2](https://doi.org/10.1017/S0261127900002084)
2. **A single modern scale name is an approximation. [P/R, high confidence]** Dorian, Phrygian, Lydian and Mixolydian settings are usable UI mappings, but accidentals at cadences and counterpoint (“musica ficta”) cannot be inferred from a seven-note whitelist alone. Disable free modal borrowing; permit only cadence-/voice-leading-governed alterations if the engine eventually supports them. The safe first preset is a diatonic mode with explicit approximation copy, not “authentic Renaissance tuning.” [Bent, “Diatonic ficta,” *Early Music History*](https://www.cambridge.org/core/journals/early-music-history/article/abs/diatonic-ficta/756AA78E9E8E65E4F15BC26F7DF56C8A)
3. **Tuning is underdetermined for MIDI. [gap, high confidence]** The sources establish contrapuntal pitch relations, not one universally correct modern temperament for this preset. Twelve-tone MIDI is acceptable as an implementation constraint; do not market equal temperament as historical tuning.

### 2. Melodic contour, interval vocabulary, register and range

1. **Each part must remain vocally plausible. [P/J, high confidence]** Stepwise motion predominates; repeated notes and thirds are ordinary; fourths and fifths are occasional articulating leaps; octave or sixth leaps are exceptional, exposed gestures rather than routine motion. A leap should normally be balanced by subsequent motion in the opposite direction, and consecutive large leaps should not form arpeggiated tonal chords. Jeppesen’s corpus-based account is the source of many familiar “Palestrina style” melodic constraints, but his classroom rules are a reconstruction, not Palestrina’s own rulebook. [Jeppesen, *The Style of Palestrina and the Dissonance* (1946 English trans.)](https://archive.org/details/styleofpalestrin0000jepp_q4n9)
2. **Contour should arc rather than zigzag mechanically. [P/J, medium-high confidence]** Phrase lines generally accumulate motion toward one local high or low region and relax; immediate alternating up/down steps at every event sounds algorithmic. High notes and large upward leaps carry salience and should be sparse.
3. **Vertical vocabulary is contrapuntal. [P/R, high confidence]** Structural sonorities arise from unisons/octaves/fifths and imperfect consonances (thirds/sixths, plus compounds). Perfect consonances articulate starts, goals and cadences; imperfect consonances sustain flow. Fourths depend on the voices/bass relation and cannot be globally labeled “safe.” Do not interpret every slice as a tertian chord or enforce chord roots/inversions.
4. **Keep distinct SATB-like bands but allow overlap, not crossing as a default effect. [P/J, medium confidence]** A practical MIDI ceiling is roughly an octave to a tenth of active ambitus per generated line, with neighboring upper voices usually within an octave and bass–tenor allowed wider. This is an implementation heuristic, not a corpus-derived universal number. Avoid octave spreading and Mirror duplication: they inflate ambitus and destroy four-line identity.

### 3. Dissonance: authentic categories and temporal contracts

1. **Most dissonance is controlled melodic motion, not a color chord. [P, high confidence]** A computational study of 36 Palestrina Agnus movements found five strong clusters: descending passing tones (863), ascending passing tones (643), suspensions on beat 3 (313), suspensions on beat 1 (265), and lower neighbors (230). Passing/neighbor notes were short and weak; suspensions repeated a prepared pitch on a strong position and resolved downward by step. The study explicitly says its conservative detector omitted other real categories. [Anders et al., “Machine learning of symbolic compositional rules…,” *PeerJ Computer Science*](https://pmc.ncbi.nlm.nih.gov/articles/PMC10319261/)
2. **The repertoire exceeds the classroom five. [P, high confidence]** Large-corpus schema work classifies virtually all Palestrina/Victoria dissonance with passing tones, neighbors, suspensions, anticipations, cambiatas, dissonant third-quarters and rarer combinations; fewer than 0.3% remained unclassified in the reported corpus. Therefore “allow only Fux’s five” is too narrow, while “allow any non-chord tone” is far too broad. [Sigler & Wild, “Schematizing the Treatment of Dissonance in 16th-Century Counterpoint,” ISMIR 2015](https://archives.ismir.net/ismir2015/paper/000153.pdf)
3. **Minimum safe real-time subset. [P-derived, high confidence]** Implement only:
   - weak passing: `consonance → diatonic step in same direction (dissonant, shorter) → consonance`;
   - weak lower neighbor: `consonance → step down (dissonant, shorter) → return`;
   - suspension: pitch is consonant before the metric boundary (**preparation**), tied/held while another voice changes so it becomes dissonant (**suspension**), then descends one diatonic step (**resolution**), normally proceeding to a stable perfect goal (**perfection**).
   Reject a candidate unless the complete three-/four-phase contract can be scheduled and checked against all sounding voices. Never create an unprepared accented dissonance merely because a style flag is on.
4. **Renaissance suspension is not merely a tonal 4–3 effect. [R with direct relevance, high confidence]** Morgan demonstrates a fourth “perfection phase” after resolution and shows that contrapuntal rhythm, not the notated meter alone, determines accent placement. The resolution’s third/sixth may still lead onward to a perfect cadential goal. [Morgan, “Renaissance Ternary Suspensions in Theory and Practice,” *Intégral* 33](https://theory.esm.rochester.edu/integral/33-2019/morgan/)
5. **Expressive/exceptional dissonance is out of scope. [J/P, medium confidence]** Jeppesen distinguishes weak connective, accented primary, and rarer expressive dissonance. The preset should model the frequent first two only; rare exceptions require text/form knowledge the live engine lacks.

### 4. Voice independence and motion

1. **Avoid successive perfect fifths, octaves and unisons between every pair, including the player. [P/R/J, high confidence]** Direct/similar approach to an exposed perfect interval should be strongly penalized, especially when the upper part leaps. Contrary and oblique motion are preferred ways to approach structural perfect consonances.
2. **Contrary is a preference, not an always-on transform. [P, high confidence]** Authentic texture mixes contrary, oblique and similar motion. Parallel thirds, sixths and tenths occur locally, but sustained planing of all voices is misleading. In Palestrina’s *Je suys desheritee* Kyrie, analysis finds parallel sixths/tenths as local module components alongside double canon, stretto and recomposed countermelodies—not a global parallel-harmony algorithm. [Schubert & Lessoil-Daelman, “What Modular Analysis Can Tell Us…,” *Music Theory Online* 19/1](https://mtosmt.org/issues/mto.13.19.1/mto.13.19.1.schubert_lessoil-daelman.html)
3. **Independence requires different onsets and note lengths. [P, high confidence]** A pitch-only optimizer that changes all four voices whenever the user attacks yields homorhythm. Genuine independence includes one voice holding while another moves, staggered entries, local rests, and different rates of surface motion over a shared tactus.

### 5. Rhythm, imitation, phrase pacing and evolving texture

1. **Shared tactus, independent surface rhythm. [R/P, high confidence]** Keep a stable pulse but let parts use longer structural values and shorter passing motion. Dissonance legality must consult metric strength/contrapuntal rhythm. Do not quantize every line to identical note-against-note attacks. DeFord treats tactus and contrapuntal rhythm as related but non-identical structural levels. [DeFord, *Tactus, Mensuration and Rhythm in Renaissance Music* (Cambridge, 2015)](https://www.cambridge.org/core/books/tactus-mensuration-and-rhythm-in-renaissance-music/F2E3F151365C3CD4D5164B43090086F4)
2. **A phrase/point begins with a subject, not a full four-note block. [P, high confidence]** Capture a short, singable player cell (practical approximation: 3–6 notes bounded by a breath), then enter generated voices successively at unison/octave or another contrapuntally viable modal relation. Entries may overlap before all voices have stated the cell. Schubert’s study of Palestrina’s First Book of Four-Voice Motets analyzes points of imitation as locally heightening and relaxing structures rather than predictable repetitions. [Schubert, “Hidden Forms in Palestrina’s First Book of Four-Voice Motets,” JAMS 60/3](https://doi.org/10.1525/jams.2007.60.3.483)
3. **Imitation must become free counterpoint. [P, high confidence]** Preserve the subject’s interval/rhythm identity at entry, then let each voice continue independently; exact endless canon is a caricature. Palestrina can compress entry intervals, add stretto, transfer material among all voices, and increase contrapuntal density while varying repetitions. The *Je suys desheritee* Kyrie analysis explicitly limits its broad personal-style claims to that movement, so use “density intensification” as an available trajectory, not a mandatory law for every phrase. [Schubert & Lessoil-Daelman](https://mtosmt.org/issues/mto.13.19.1/mto.13.19.1.schubert_lessoil-daelman.html)
4. **Cadences are two-/multi-voice processes. [R/P, high confidence]** A cadence should be detected/generated as complementary stepwise clausulae (cantizans ascending, often ornamented by suspension; tenorizans descending), with optional bassizans/altizans support, arriving on a perfect consonance. Cadence tone, cadence type and voice functions matter more than a modern V–I chord label. [CRIM Intervals cadence methodology](https://github.com/HCDigitalScholarship/intervals/blob/main/tutorial/11_Cadences.md) [Powers, modal/cadential context](https://doi.org/10.1017/S0261127900002084)
5. **Cadences articulate a hierarchy. [P/R, medium-high confidence]** Interior points may close on modal secondary degrees and overlap with the next entry; section ends should more strongly confirm the final, lengthen arrival values, reduce moving parts, and permit a short collective or near-collective breath. Repeated identical four-bar “perfect cadences” sound tonal and schematic.

## Testable stylistic invariants

1. Output is exactly **four conceptual lines including the player**, with stable voice identity and no routine doubling.
2. On structural/strong positions, every pair is consonant except an explicitly active, correctly prepared suspension.
3. No consecutive parallel P5/P8/P1 between **any** pair of voices across structural events; include player/generated pairs.
4. Every permitted weak dissonance is short and matches a complete passing- or lower-neighbor schema; no stranded dissonance after NoteOff, transport stop or parameter change.
5. Every accented dissonance has recorded consonant preparation and scheduled downward-step resolution; cadence-targeted cases also reach a stable perfection phase.
6. Generated melodic motion is predominantly step/repetition; leaps > P4 are rare, never routine, and followed by contrary recovery. No generated line exceeds its configured recent ambitus ceiling without reset/recovery.
7. At least one voice holds or rests, or moves contrary, whenever 3+ parts change; all-four same-direction motion is rejected except an explicitly whitelisted short imperfect-consonance module.
8. Phrase openings enter progressively (not instant four-part chord); continuation can reach 3–4 active voices; cadence slows/thins and then breathes.
9. An imitative entry retains the captured cell’s abstract interval/rhythm signature, but post-entry continuation is not strict looping canon.
10. Modal final/cadence plan remains stable for a phrase; chromatic notes are not admitted by generic “modal interchange.”
11. Transport is required for scheduled dissonance, imitation and cadence behavior. Without transport, fall back to conservative consonant note-against-note output and state this limitation.

## Variable parameters (identity survives within these bounds)

- Diatonic mode/final; transposition; modern concert pitch/temperament.
- Tempo approximately 54–96 quarter-note beats/minute, provided scheduling is pulse-relative.
- Subject length 3–6 clean notes; entry delay roughly 1–2 tactus units; entry order and octave placement.
- Player voice position (soprano/alto/tenor/bass), with register-aware generated ranges.
- Interior cadence strength and destination; 2–4 active voices during a point; one optional suspension near cadence.
- Passing/neighbor probability (low–moderate), suspension frequency (sparse), and phrase breath threshold.
- Local imperfect-consonance parallel motion and density peak location.

These are implementation bounds, not claims of exact historical frequencies.

## Misleading behaviors / rejection tests

- Four parallel diatonic thirds on every input; chord-pad block attacks; Alberti/arpeggiator figures.
- “Always contrary” mirror line, or random contrary direction with no cadence/contour state.
- Any scale preset marketed as the whole style; major/minor functional chord loops; unrestricted modal interchange.
- Continuous strict canon or subject repetition at every bar; imitation without later free continuation.
- Fux Species 1–4 selector presented as Palestrina emulation.
- Suspensions generated every few note calls regardless of beat, consonant preparation, other voices, or player release.
- Uniform eighth-note note storm; syncopation with no stable tactus; all voices sharing the same duration.
- Mirror/octave duplication, wide cinematic spread, voice crossing as color, or register drift beyond vocal bands.
- Full texture from the first note and unbroken four-voice density through cadence.
- Jazz articulation/dynamics, hard staccato, swing or velocity-accent groove. Dynamics should remain restrained; articulation should be connected but preserve phrase breaths.

## Temporal state sketch

```text
REST / BREATH
  trigger: >= ~1 tactus silence; clear motif and unresolved-dissonance state
  |
  | first 3–6-note, mostly stepwise player cell begins
  v
EXPOSITION (1 → 2 voices)
  capture abstract interval + onset/duration signature
  schedule first imitative entry after 1–2 tactus units
  structural positions consonant; no suspension before preparation exists
  |
  | cell confidence reached OR second entry begins
  v
IMITATIVE BUILD (2 → 3 → 4 voices)
  stagger further entries; vary register/order
  earlier voices continue freely with steps/holds/rests
  allow only weak passing/lower-neighbor schemas
  |
  | all planned entries stated, phrase age threshold, or player sustains destination
  v
FREE CONTINUATION / HEIGHTENING (3–4 voices)
  shorten entry distance or reuse a fragment once; increase active density locally
  preserve independent onset rates; prevent all-parallel motion
  optionally arm one prepared suspension when cadence intent rises
  |
  | player slows/holds, contour settles, silence prediction, or max phrase length
  v
CADENTIAL PREPARATION
  suppress new imitation; lengthen values; choose cadence tone
  assign cantizans/tenorizans (+ optional support functions)
  preparation → suspension (optional) → downward resolution
  |
  | cadence voices reach perfect goal
  v
PERFECTION / RELEASE (4 → 2 → 0/1 voices)
  hold goal ≥ 1 structural unit; release generated voices cleanly
  require a short breath before fresh capture; weak interior cadence may elide to EXPOSITION
```

**Across sections:** start a new point from a new or recognizably transformed cell after a breath; vary entry order/register and density trajectory; reserve the strongest confirmation of the modal final and longest release for the last detected section. Do not fake through-composed form by mechanically transposing the same canon.

## Abstract input/output acceptance examples

Notation: degrees refer to the selected mode; `V0` is player. Beats are tactus-relative. Parentheses mean held through the next event; `r` is rest. These are novel behavioral examples, not melodies from the corpus.

### Example A — staggered imitation into free continuation

**Input (V0, Dorian-like collection, player in upper-middle register):**

```text
beat:  0    1    2    3    4    5    6
V0:    1    2    4    3    5---      r
```

**One acceptable relationship-level output:**

```text
beat:  0    1    2    3    4    5    6    7
V0:    1    2    4    3    5---      r
V1:    r    r    1    2    4    3    2---
V2:    r    r    r    r    1    2    3---
V3:    r    r    r    r    r    6    5---
```

Assertions: no four-part block at beat 0; V1 preserves the input’s opening `+step, +third, -step` identity after a two-beat delay, then diverges; later voices enter rather than chordally stack; at least one oblique/resting voice exists in the dense region; all structural slices are consonant or explicitly labeled; cadence intent at beats 6–7 suppresses further entries and lengthens values. Exact degrees may change if required to avoid parallels/dissonance.

### Example B — legal weak passing tone and cadential suspension

**Input (V0):**

```text
beat:  0    1    2    3    4    5
V0:    5---      4    3---      2--1
```

**One acceptable relationship-level output (generated voices abbreviated):**

```text
beat:       0     1     2     2.5   3     4     5     6
V0:         5-----------4-----------3-----------2-----1---
Cantizans:  3-----------4-----------5(prep)----5(sus)-4(res)-5(goal)
Tenorizans: 1-----------------------2-----------3-----2-----1(goal)
Inner:      r-----6-----5-----------6-----------r-----3-----r
```

Assertions: the inner `6→5` at beats 1–2 may be a weak passing/neighbor event only if pairwise checks label beat 1 as weak and the endpoints consonant; the cantizans pitch at beat 4 is consonant preparation, held into a checked accented dissonance at beat 5, descends one modal step, then participates in the perfect goal at beat 6; no new subject starts during cadential preparation. If the player releases before beat 5, cancel the suspension or safely release it—never leave a prepared pitch sounding without its contract.

### Example C — safe no-transport degradation

**Input:** degrees `1, 3, 2, 4` at irregular wall-clock times, transport stopped.

**Expected:** four smoothly ranged lines may be returned note-against-note, with pairwise consonance and perfect-parallel checks; **no claim of rhythmic imitation, weak passing tone, or suspension**. The output must not use a synthetic per-note counter to pretend irregular attacks are metric beats for this preset.

## Exact Contrapunk implications

### HarmonyEngine: reusable now

- `HarmonyMode::StrictCounterpoint`, four voices, strictness `Strict`, and `VoiceLeadingStyle::Palestrina` are the closest pitch baseline.
- `ScaleMode` supports diatonic modal collections; `voice_count`, `voice_position`, register assignment, recent-range state, interval history, contour preference, and octave `None` are useful.
- `CounterpointState` already rewards stepwise/contrary motion, rejects/penalizes some parallel perfect intervals, tracks recent ambitus, leap recovery and interval variety.
- The voice-leading revoicer has Palestrina weights for max 5-semitone leap, stepwise/common-tone preference, crossing/spacing penalties and contrary motion.

### HarmonyEngine: correctness gaps that block an “authentic” claim

1. **Pairwise checks omit the player.** `voice_leading/rules.rs` starts parallel-fifth/octave, crossing and motion-independence checks at index 1, even though index 0 is the player. Modal Linework requires P5/P8 checks across every pair, especially player–generated. This is a shared root gap, not a preset tweak.
2. **Chord-slice processing cannot create independent rhythms.** `harmonize_note_on` returns one simultaneous vector per attack. Pitch revoicing cannot schedule entries, rests, passing subdivisions or cadence releases.
3. **Current `SuspensionState` is not metrically or harmonically validated.** It picks the highest changed generated voice, holds it, waits by process-call count, then resolves on a later call. It does not prove consonant preparation, dissonance against all voices, beat strength, legal suspension interval, player lifecycle, or perfection goal. Disable this generic automatic suspension for the preset until a beat-scheduled contract checker exists; Species IV is a separate preset (#8), not a shortcut here.
4. **Current Palestrina comments overstate historical rules.** “Extremely tight,” “no leaps beyond a 4th,” and “voices clustered close” are useful product heuristics, not invariant Palestrina facts. Keep them as bounded optimizer weights, not UI scholarship.
5. **Modal model lacks cadence-aware ficta and cadence hierarchy.** `ScaleMode` plus key cannot choose contextual raised/lowered cadence tones or modal secondary cadence plans. Disable interchange rather than substituting random borrowed notes.
6. **Per-event state is too short for form.** Four-interval/three-contour windows cannot detect a subject, phrase breath, entry completion, section, or cadential intent.

### Companion/Lane: reusable now

- `CanonLane` already has beat-clock scheduling, phrase anchor reset after silence, per-voice delay/transposition/time ratio, voice count/position, and Palestrina/StrictCounterpoint mini-engine overrides. This can host a deliberately limited imitative-entry approximation.
- `CounterpointLane` can schedule Species 2/3 subdivisions and route a separate line; `WorldState.transport` supplies actual beats; Hold modes can prevent pending notes from outliving their seed.
- Companion dispatch gives separate NoteOn/NoteOff lifecycles, which HarmonyEngine alone lacks.

### Companion/Lane: required minimum extension

- Prefer **one phrase-aware ModalLinework lane/state**, not stacking current Canon + Counterpoint outputs (which risks 5+ voices and unrelated state). Minimum state: motif signature, phrase anchor/last input beat, stable four voice IDs/registers, per-voice active pitch, scheduled on/off queue, dissonance contract, cadence intent/tone/functions, density stage.
- Capture 3–6 notes only after a rest-defined phrase start; schedule at most three generated conceptual voices, preserving a total of four including the player.
- Reuse `Scale::transpose_diatonic` and CounterpointState candidate scoring, but evaluate the final candidate against **all currently sounding voices and the player**.
- Add explicit event labels (`StructuralConsonance`, `Passing`, `LowerNeighbor`, `Suspension{Prepared,Suspended,Resolving,Perfected}`) so tests can assert lifecycle, not merely MIDI pitches.
- Gate all temporal behavior on live transport. A stopped transport selects conservative fallback, not synthetic attack-count “beats.”
- Add cadence intent from simple observable triggers only: sustained/repeated destination plus phrase age or a real silence; do not invent text-sensitive form.
- Do not use `CounterpointLane` Species 4 as implemented: its own source comments say it is only a half-beat delay and that proper resolution follows later. Species 2/3 also currently schedule predetermined step chains without checking the resulting vertical dissonance against future player notes; unsuitable as-is for this preset.

### Smallest runnable acceptance checks for implementation

1. **All-pair parallel test:** feed two structural slices where player and one generated voice would form successive P5s; candidate must be rejected/revoiced.
2. **Dissonance lifecycle test:** schedule a suspension, release seed before suspension beat, advance transport; no dissonant NoteOn and no stuck note.
3. **Temporal texture test:** input a 4-note cell with transport; assert active conceptual voice counts progress `1 → 2 → 3/4 → <=2 → 0/1`, never exceed four, and entry onsets are not all equal.
4. **No-transport test:** same cell with stopped transport emits no scheduled imitation/passing/suspension.
5. **Cadence test:** after phrase-age + held destination trigger, no fresh imitative entry; assigned clausula voices move by contrary steps to a perfect goal and then release.

## Confidence, disagreements and gaps

- **High:** controlled dissonance categories and metric profiles, prepared downward-resolving suspensions, pervasive stepwise line, pairwise perfect-parallel avoidance, imitative/free-polyphonic texture, and cadence as contrapuntal process. These rest on scores, Palestrina corpora and peer-reviewed historical theory.
- **Medium-high:** progressive-entry → density → cadence temporal model. It is strongly supported as a recurring motet/imitative-Mass strategy, but not a law of every Palestrina section.
- **Medium:** numerical tempo, motif length, ambitus and delay bounds. They are product-safe operational approximations, not exact corpus distributions.
- **Disagreement:** modal analysis can treat mode as sounding organization, compositional category, tonal type or publication-order designation. Do not promise that choosing “Dorian” reconstructs a sixteenth-century modal analysis.
- **Disagreement:** Jeppesen’s elegant normative Palestrina style remains highly useful, but corpus studies find categories and exceptions that simplified species teaching omits. The engine should use probabilities/penalties around the secure frequent behaviors and reserve hard rejection for lifecycle safety and successive perfect parallels.
- **Gap:** no quantitative corpus measurement was obtained here for four-voice ambitus, exact leap percentages, entry delays, phrase lengths or density curves across the scoped late motets. A CRIM/JRP encoded-score query would be the next step before assigning probabilistic defaults.
- **Gap:** contextual musica ficta, text declamation and syntactic phrase boundaries cannot be inferred reliably from monophonic live MIDI. Product copy must call this “modal vocal-counterpoint linework inspired by late-Renaissance practice,” not Palestrina composition.
- **Gap:** tuning and pronunciation/performance-practice questions are outside HarmonyEngine’s 12-TET MIDI arrangement scope.

## Sources

### Kept

- [Palestrina, *Sicut cervus* — CPDL score/source page](https://www.cpdl.org/wiki/index.php/Sicut_cervus_%28Giovanni_Pierluigi_da_Palestrina%29) — primary musical evidence, mature four-voice motet.
- [Palestrina, *Missa Papae Marcelli* — CPDL score/source page](https://www.cpdl.org/wiki/index.php/Missa_Papae_Marcelli_%28Giovanni_Pierluigi_da_Palestrina%29) — primary musical evidence, mature larger-voice Mass.
- [Anders et al., PeerJ CS](https://pmc.ncbi.nlm.nih.gov/articles/PMC10319261/) — open full-text Palestrina Agnus corpus study with counts and learned dissonance profiles.
- [Sigler & Wild, ISMIR 2015](https://archives.ismir.net/ismir2015/paper/000153.pdf) — large Palestrina/Victoria corpus classification beyond textbook categories.
- [Jeppesen, *The Style of Palestrina and the Dissonance*](https://archive.org/details/styleofpalestrin0000jepp_q4n9) — foundational corpus-derived analysis, retained with explicit historiographic caveat.
- [Schubert & Lessoil-Daelman, MTO 19/1](https://mtosmt.org/issues/mto.13.19.1/mto.13.19.1.schubert_lessoil-daelman.html) — detailed, score-referenced analysis of imitation, modular variation and density in a Palestrina Kyrie.
- [Schubert, “Hidden Forms…”](https://doi.org/10.1525/jams.2007.60.3.483) — peer-reviewed Palestrina motet analysis of points of imitation and local intensification/relaxation.
- [Powers, “Modal Representation…”](https://doi.org/10.1017/S0261127900002084) and [“Tonal Types…”](https://doi.org/10.2307/831189) — authoritative corrective to scale-only modality.
- [Morgan, *Intégral* 33](https://theory.esm.rochester.edu/integral/33-2019/morgan/) — corpus-supported Renaissance suspension timing/perfection phase and textbook critique.
- [DeFord, *Tactus, Mensuration and Rhythm*](https://www.cambridge.org/core/books/tactus-mensuration-and-rhythm-in-renaissance-music/F2E3F151365C3CD4D5164B43090086F4) — authoritative rhythmic framework.
- [Schubert, “Counterpoint pedagogy in the Renaissance”](https://resolve.cambridge.org/core/services/aop-cambridge-core/content/view/6EC7E10648302A8BF897F0F22B655B74/9781139053471c16_p503-533_CBO.pdf/counterpoint-pedagogy-in-the-renaissance.pdf) — historical boundary between Renaissance learning and later species reduction.
- [CRIM Intervals cadence tutorial](https://github.com/HCDigitalScholarship/intervals/blob/main/tutorial/11_Cadences.md) — transparent computational definitions for cadence voice functions.

### Dropped

- Wikipedia and unsourced “Palestrina style” rule lists — useful discovery aids but not authoritative enough for implementation invariants.
- Concert program notes on *Sicut cervus* — evocative but not analytical evidence.
- Generic species-counterpoint websites — conflate Fux/modern pedagogy with Palestrina’s mixed practice.
- SEO/bookstore summaries of Jeppesen — replaced by the publication record/full book source.
- Student theses and Academia.edu reposts — omitted where peer-reviewed or primary sources covered the same claim.

## Gaps / recommended next evidence pass

Query encoded late motets directly (CRIM/JRP or downloadable MusicXML) for distributions of per-voice ambitus, melodic interval size, entry lag, active-voice density and pre-cadential note length. That measurement would refine defaults, but it is not needed to reject the current misleading shortcuts or establish the lifecycle invariants above.

```acceptance-report
{
  "criteriaSatisfied": [
    {
      "id": "criterion-1",
      "status": "satisfied",
      "evidence": "Produced only the requested read-only music-theory/temporal report artifact; no project or source files were modified."
    },
    {
      "id": "criterion-2",
      "status": "satisfied",
      "evidence": "Report provides primary scores, peer-reviewed/corpus sources, evidence labels separating Palestrina from Renaissance practice, Jeppesen and Fux, testable invariants, temporal states, three abstract examples, confidence/gaps, and file-specific HarmonyEngine/Companion implications."
    }
  ],
  "changedFiles": [
    ".pi-subagents/artifacts/outputs/42978c4f-4b70-42ee-9350-f36d0aeed0eb/research/02-modal-linework/theory.md"
  ],
  "testsAddedOrUpdated": [],
  "commandsRun": [
    {
      "command": "read 10.2-RESEARCH-TEMPLATE.md and 10.2-PRESET-CATALOG.md",
      "result": "passed",
      "summary": "Confirmed role deliverables, preset scope, baseline status and honesty requirements."
    },
    {
      "command": "read HarmonyEngine, stateful counterpoint, voice-leading, suspension, CounterpointLane and CanonLane source files",
      "result": "passed",
      "summary": "Traced current pitch, metric, scheduling, suspension, register and lane capabilities and gaps."
    },
    {
      "command": "focused web research across primary scores, Palestrina corpora, modality, dissonance, imitation, cadence, rhythm and Fux historiography",
      "result": "passed",
      "summary": "Retained authoritative score, peer-reviewed, institutional and full-text sources; excluded generic/SEO summaries."
    }
  ],
  "validationOutput": [
    "Artifact written at the exact authoritative output path.",
    "Contains all requested theory dimensions, explicit P/J/R/F evidence separation, 11 invariants, variable/rejected behaviors, temporal state sketch, 3 abstract examples, implementation checks, confidence/disagreements/gaps, and cited source audit."
  ],
  "residualRisks": [
    "No direct encoded-corpus query was run for exact ambitus, entry-delay, phrase-length or density distributions; numerical bounds are clearly labeled implementation heuristics.",
    "Repository staging state was not inspected because the available research tools expose no git-status command; this run did not stage files."
  ],
  "noStagedFiles": true,
  "diffSummary": "Added one research report artifact; project/source files unchanged.",
  "reviewFindings": [
    "no blockers",
    "review note: current player-excluding pairwise checks and call-count suspension FSM prevent an authentic-style claim without shared-engine fixes or explicit preset limitations"
  ],
  "manualNotes": "Review gate requested. Treat four-part modal counterpoint as a bounded approximation; do not use current generic Palestrina suspension or Fux species scheduling as proof of historical authenticity."
}
```
