# Research: Preset 25 — Bebop Chase (music theory and temporal behavior)

## Summary

**Decision: the documented 1940s Parker/Gillespie language supports fast, harmonically directed, rhythmically displaced lines and real, form-aware alternation, but the present Free Imitation/Canon lane cannot honestly produce “shortened delayed answers.”** It can emit a fixed-delay, optionally diatonically transposed, note-for-note copy (and can scale timestamps/durations), but it has no completed-phrase capture, truncation, motif/fragment recognition, response selection, accent model, or swing model. Therefore the catalog’s current Result text must either be narrowed to **“fast phrases receive delayed transposed echoes”** or preset 25 must remain capability-gated until a phrase/motif response lane exists.

The scoped reference is the **shared small-group language and interaction of Parker and Gillespie in 1945–47**, especially the 1945 recordings *Shaw ’Nuff* and *Ko Ko*, not either musician’s whole career. Their intersection is a tonal-functional bebop vocabulary, fast eighth-note continuity, chromatic approach/enclosure, altered dominants, formulaic cells flexibly recombined, sharp accents, and formal awareness. Their differences must remain audible: Parker’s line is unusually fluid, asymmetrically segmented, varied in accent and phrase length; Gillespie’s trumpet vocabulary is more visibly tied to pianistic harmonic understanding, triplet runs, high-register projection, repeated-note/brassy punctuation, and explicit arranging/ensemble organization.

## Scope, corpus, and period boundary

- **Core joint corpus:** Dizzy Gillespie All Star Quintet, *Shaw ’Nuff* (11 May 1945; Parker alto, Gillespie trumpet); Charlie Parker’s Reboppers, *Ko Ko* (26 November 1945; Parker alto, Gillespie trumpet/piano); the 1946 Parker recording of *Ornithology* as evidence of actual call-and-response construction, not a generic model for every solo.
- **Analytical comparison:** Parker blues performances from 1944–53 in Stefan Love’s transcription-based corpus; Henry Martin’s score/recording analysis of Parker’s compositional and motivic procedures; Gillespie’s own retrospective interview on harmonic thinking and his meeting with Parker; Nicholas Schroeder’s dissertation record is retained only as evidence that detailed Gillespie transcription research exists, because its full text is access-restricted and its analyzed solos are 1956–59, outside the preset period.
- **Period boundary:** target **1945–47**. Do not import Gillespie’s later mature Afro-Cuban language wholesale: his Chano Pozo collaboration and *Manteca* phase begins in 1947 and becomes a distinct rhythmic/orchestral project. Do not import Parker’s late strings projects or treat post-1949 formula use as proof of every 1945 interaction. Gillespie himself describes Parker and himself as already “thinking almost in the same line” before their Earl Hines (1942) and Billy Eckstine (1943) work, while distinguishing the rhythmic laboratory of Minton’s from the harmonic one at Monroe’s. [National Jazz Archive interview](https://jazzpro.nationaljazzarchive.org.uk/interviews/Dizzy%20Gillespie.htm)

## Findings

### 1. Tonal center, harmony, chromaticism, and tuning

1. **Functional form remains the floor; chromaticism intensifies direction rather than abolishing it.** Love’s Parker corpus describes fixed meter, elastic surface harmony, and free melody over preserved harmonic middleground; bebop blues retains tonic establishment, IV around bar 5, and a ii–V–I cadence around bars 9–11 even under substitutions. Parker can delay or suppress a tonic arrival, but that is tension against an understood form, not atonality. [Love, “Possible Paths” §§1.3, 2.1–2.3](https://mtosmt.org/issues/mto.12.18.3/mto.12.18.3.love.php)
2. **The shared vocabulary is chord-progressional, not one “bebop scale.”** *Shaw ’Nuff* is in B-flat, 4/4, with A sections cycling through I–vi–ii–V and secondary dominants/diminished connectors; its bridge uses altered dominant regions including b9/#11 collections before ii–V. The audio-aligned harmonic annotation places the head at 20.83–47.86 s, Parker’s alto chorus at 48.08–75.26 s, and Gillespie’s trumpet chorus at 75.46–103.20 s, providing a clean same-form comparison. [JAAH/MTG *Shaw ’Nuff* annotation](https://mtg.github.io/JAAH/data/shaw_nuff.html)
3. **Parker’s chromatic notes are voice-leading events.** Love identifies chromatic descent into ii, altered dominants, blue b3/b7, enclosures, and schema inflection, with stepwise destination tones emphasized by register, meter, duration, dynamics, or phrase position. Martin likewise concludes that Parker is tonally conservative at the deeper level: upper extensions and non-chord tones are generally resolved and “loose ends” tied up. [Love §§4.1–4.15](https://mtosmt.org/issues/mto.12.18.3/mto.12.18.3.love.php) [Martin §§5.1–5.7](https://mtosmt.org/issues/mto.18.24.2/mto.18.24.2.martin.html)
4. **Gillespie’s difference is explicit harmonic intentionality.** Gillespie says the piano was his “greatest source of inspiration,” that his conception became harmonically deeper than Roy Eldridge’s, and that a lick is not transferable merely by pitch: its meaning depends on the chord and where that chord is going. This supports response transformations that retarget chord tones, but not blind scale transposition. [National Jazz Archive interview](https://jazzpro.nationaljazzarchive.org.uk/interviews/Dizzy%20Gillespie.htm)
5. **Tuning:** no evidence supports alternate temperament. Use ordinary 12-TET MIDI. Expressive intonation, scoops, bends, growls, and trumpet/sax timbre are performance/orchestration matters that the current note mapper does not model.

**Implication:** `ScaleMode::BebopDominant` may be a useful pitch safety net over a dominant area, but is not a complete Parker/Gillespie harmonic model and is actively misleading as a global scale over all functional changes. `Ionian` plus chromatic player input preserves notes, but the current diatonic transpose can map chromatic notes through `harmonize_smart`/modal interchange rather than understanding their approach-tone destination.

### 2. Interval and line vocabulary

- Predominantly **stepwise linear continuity**, broken by arpeggiated chord members, octave/register resets, sevenths or other leaps that launch/continue a line, and chromatic approach/enclosure around structural chord tones.
- Tensions include 7ths, 9ths, altered 5ths/9ths and #11 over dominants; blue b3/b7 can be reinterpreted by changing harmony. Martin’s *Red Cross* analysis specifically identifies a bebop enclosing-note figure made more convincing when its third lands on the beat and the sequence follows circle-of-fifths voice leading. [Martin §§4.5–4.10](https://mtosmt.org/issues/mto.18.24.2/mto.18.24.2.martin.html)
- Parker’s recurring medium-range paths include descending scale-degree spans, often interrupted, divided, truncated, extended, octave-transferred, or chromatically inflected. The point is the **path and target**, not literal interval replication. [Love §§4.8–4.16](https://mtosmt.org/issues/mto.12.18.3/mto.12.18.3.love.php)
- Gillespie-specific difference: triplet-based runs are important in his solos even though the repeated traded triplets in the first *Ornithology* version are judged unidiomatic as a generic bop device. Therefore triplets may color a Gillespie-like answer but must not become the entire shared preset. [Martin §§2.15–2.16 and n.36](https://mtosmt.org/issues/mto.18.24.2/mto.18.24.2.martin.html)

### 3. Motif, fragment, repetition, and transformation

- Parker does reuse formulas, but not as rigid MIDI clips. Love summarizes Owens’s roughly 100 formulas and transformations: **metric displacement, augmentation/diminution, addition/subtraction, altered phrasing and articulation**. Formulae are integrated into longer voice-led paths. [Love §§4.3–4.7](https://mtosmt.org/issues/mto.12.18.3/mto.12.18.3.love.php)
- Martin’s *Blues (Fast)* take sequence demonstrates actual development: a three-note turn is moved on the metric grid, expanded, contracted, syncopated, embedded in longer phrases, and ultimately subordinated because literal repetition was insufficient. [Martin §§4.12–4.30](https://mtosmt.org/issues/mto.18.24.2/mto.18.24.2.martin.html)
- A credible chase answer should therefore preserve **one recognisable invariant** (contour kernel, terminal target, rhythm cell, or opening interval) while changing at least one other dimension (length, starting beat, octave/register, approach notes, or cadence). Literal full copying is allowed only as an occasional echo, not the defining behavior.

### 4. Voice interaction and imitation

- The strongest evidence for actual Parker/Gillespie alternation is *Ko Ko*: unison opening, **traded eight-bar phrases**, a quick unison bridge, then Parker’s solo. This is sectional/formal alternation; it is not one horn shadowing every note of the other at a fixed delay. [NPR/Weekend Edition account with Gary Giddins and Phil Schaap](https://text.npr.org/1081208)
- *Ornithology* (1946) contains a composed call-and-response triplet figure answered by piano and later passed among players without alteration. Martin stresses that this device works because multiple players trade it, and that its unchanged repetition is not broadly idiomatic to bebop. [Martin §§2.15–2.16](https://mtosmt.org/issues/mto.18.24.2/mto.18.24.2.martin.html)
- *Shaw ’Nuff* demonstrates a different norm: tightly coordinated ensemble head, then distinct full AABA solo choruses (alto followed by trumpet), then return. This is contrast of individual voices inside a shared harmonic/formal language, not simultaneous canon. [JAAH annotation](https://mtg.github.io/JAAH/data/shaw_nuff.html)
- Preserve differences: a Parker-coded call favors fluid continuity, variable phrase length and shifting accents; a Gillespie-coded answer may be more declamatory, registrally projecting, triplet-inflected, and harmonically explicit. Do not average both into an anonymous scale run.

### 5. Rhythm, meter, swing, accent, and displacement

- **4/4 and a stable beat grid are invariant.** Metrical dissonance is superimposed on the form; added/subtracted beats or bars are errors in the scoped idiom. [Love §2.1](https://mtosmt.org/issues/mto.12.18.3/mto.12.18.3.love.php)
- Fast bop typically uses streams of eighth notes, but phrase beginnings/endings, rests, accents and occasional triplets prevent a mechanical grid. Giddins describes *Ko Ko* as changing accents measure by measure and phrase by phrase, mixing very long lines with very short riffs. [NPR *Ko Ko*](https://text.npr.org/1081208)
- Phrase/meter relationships may align at four-bar boundaries or cut across them (4/4/4, 8/4, 4/8, 6/6, through-composed in Parker’s blues corpus). Phrase identity depends on inter-onset gap, melodic discontinuity, strong-beat proximity and parallelism—not merely NoteOff. [Love §§3.2–3.15](https://mtosmt.org/issues/mto.12.18.3/mto.12.18.3.love.php)
- “Swing” must not be reduced to a fixed triplet ratio at these tempos. The sources support off-beat placement and accent mobility, but the current engine records raw input event times and velocities; it neither estimates swing ratio nor creates an accent pattern.

### 6. Phrase length, density, silence, cadence, and formal pacing

- For this **performable preset approximation**, accept user calls of 4–8 notes over roughly 1–2 bars followed by at least 0.5–1 beat of silence. This is a product constraint inspired by fast short riff exchange, not a claim that Parker/Gillespie phrases were always 4–8 notes.
- Scholarly evidence explicitly ranges from tiny riffs to very long lines. Parker’s blues phrase schemata can span 4, 6, 8, or 12 bars and can spill into the next chorus. The preset’s “burst” input is therefore a constrained interaction idiom, not a corpus-wide phrase rule.
- Within a phrase: **pickup/attack → continuity/extension → compressed peak or registral crest → target/release → gap**. Chromatic density should increase toward a dominant/approach target and decrease on cadence; silence is structural and opens the response slot.
- Across a section: **sparse exchange → closer/shorter alternation → density/registral peak → thinning → cadence/unison-like closure**. This is supported as a general arrangement arc by traded formal units and Parker’s tension/climax/resolution analyses, but no evidence supports an automatic universal chase algorithm.

### 7. Register, texture, articulation, and dynamics

- Core texture: two differentiated monophonic horn roles over rhythm section; unison/tight coordinated heads alternate with one-at-a-time solo space. Simultaneous dense chord stacks on every delayed note contradict that model.
- Use adjacent but distinguishable middle/high registers; answers may jump an octave or project above the call, but must avoid accumulating 2–4 harmony notes per copied input.
- Attacks should be clean and precise, with mixed legato groups and sharp articulated punctuation. Parker’s own interview language emphasizes “clean” and “precise.” [NPR *Ko Ko*](https://text.npr.org/1081208)
- Preserve input velocity where possible, but a true answer should derive accents from phrase structure, not copy every velocity mechanically. Current Canon preserves input velocity exactly and sends all pitches in a voice stack with the same velocity.

## Testable stylistic invariants

1. **Stable 4/4/form clock:** no inserted/deleted beats; response entry quantized or intentionally offset against an intact bar/phrase grid.
2. **Tonal destination:** each chromatic answer tone is either an approach/enclosure, an altered dominant color, a blue inflection, or resolves by the response/cadence boundary; no unbounded chromatic random walk.
3. **Response is phrase-level, not per-note shadowing:** do not attack while the call is still actively unfolding unless configured as a deliberate overlap/stretto; normal chase begins after a detected gap.
4. **Recognisable but transformed:** answer retains contour/rhythm/target identity while changing at least one of note count, onset placement, register, or approach detail.
5. **Shortened means fewer events:** for an input burst of `N >= 4`, the normal answer contains `2..N-1` note attacks. Shorter durations or a smaller timestamp ratio alone do **not** satisfy this invariant.
6. **One response role:** default answer is monophonic. No chord stack per copied source note.
7. **Space and bounded density:** after an answer, leave a gap; cap queued events and permit a maximum one answer per detected call.
8. **Difference without caricature:** shared harmonic grammar, but do not force Parker and Gillespie into identical articulation/register; do not make all answers triplets or all calls uninterrupted eighth-note torrents.
9. **Lifecycle:** NoteOn ownership has exactly one matching NoteOff; disable, Panic, reconfigure, transport stop, and preset replacement leave zero sounding and pending notes.

## Allowed variation

- Tempo approximately **180–300 quarter-note BPM**, with lower values acceptable for practice. Source evidence includes Parker blues >=160 BPM and *Shaw ’Nuff* near 286 BPM from the JAAH beat timestamps.
- Answer delay 0.5–2 bars; answer may begin on an offbeat or pickup, provided meter remains clear.
- Transposition may be unison, octave, diatonic third/fourth, or chord-targeted reharmonization. Fixed diatonic transpose is an approximation, not an invariant.
- Input/answer burst length, contour direction, register, chromatic approach count, and degree of overlap may vary within density bounds.
- Phrase evolution may use aligned four-bar exchange, displaced entries, or brief overlap at a later density peak.

## Rejection behaviors

Reject or capability-gate configurations that:

- label every note-for-note delayed transposition “Parker/Gillespie call-and-response”;
- claim “shortened answer” when only note durations/timestamps are scaled;
- use global `BebopDominant` as if it represented all ii–V–I, minor, diminished, altered-dominant and blues contexts;
- create a multi-note harmony stack for every delayed note by inheriting global `voice_count = 2+`;
- answer chords/polyphonic input as an exploding canon;
- let `HoldMode::Forever` emit stale responses long after the performance has moved to another phrase;
- quantize every event to straight equal eighths, erase swing/accent, or force every line into triplets;
- transpose chromatic approaches diatonically without preserving their target relationship;
- continue pending NoteOns after disable/Panic/stop/reconfigure or emit orphan NoteOffs.

## Temporal state sketch

### Required honest behavior (not currently implementable)

`Idle`  
→ **CaptureCall** on first NoteOn after >=0.5–1 beat silence  
→ **TrackBurst** while events continue; retain onset, duration, velocity/accent, contour, harmonic target  
→ **CloseCall** on gap threshold (tempo-relative, e.g. >=0.5 beat) or max 2 bars / 8 notes  
→ **SelectFragment** choose 2–`N-1` salient attacks (opening kernel, crest, and/or cadence target); reject/trim overflow  
→ **ScheduleAnswer** after 0.5–2 beats or next formal slot; chord-target transpose and optional rhythmic displacement  
→ **Answering** monophonic, bounded, recognisably transformed; suppress recapture of generated notes  
→ **Gap** release all owned notes and wait 0.5 beat  
→ repeat, with section counters causing `Sparse → Building → Peak/overlap → Cadence/thin → Idle`.

**Evolution triggers:**
- 2–3 successful call gaps: reduce delay or allow one-beat overlap (`Building`).
- Higher user register/velocity/repetition: shorten answer further and raise answer register (`Peak`), never increase voices beyond density cap.
- Long input silence, transport boundary, or cadence target on tonic/third: thin to 2–3 note answer and return `Idle`.
- Panic/disable/stop/reconfigure: immediate `Idle`, clear captured phrase, queues, ownership, and sounding notes.

### What current CanonLane actually does

`Idle` → on **each NoteOn**, immediately compute transposed subject/full harmony stack and enqueue every resulting pitch at `anchor + delay + relative_on * time_ratio` → on each NoteOff, schedule releases and filter pending events by effective Hold (`voice > lane > global`) → `tick` drains matured events.

It has only a two-beat-silence **sequence-anchor reset** for time scaling. That is not phrase capture or motif recognition. `PhraseEnd` means `sequence_anchor + beats_per_bar`, not a detected musical phrase. See `crates/contrapunk-companion/src/canon_lane.rs` and `lane.rs`.

## Abstract, non-copyrighted acceptance examples

### Example A — shortened, target-preserving answer (required future behavior)

Context: C major, 4/4, ii–V area; degrees relative to C. Input call is invented.

- **Input:** beat `1&: 2`, `2: b3` (chromatic lower neighbor to 3), `2&: 3`, `3: 5`, `3&: 6`, `4: b6`, `4&: 5`; then silence.
- **Expected answer:** begins next bar at `2&`; attacks `3, 5, #4, 4` at `2&, 3, 3&, 4&` (four attacks versus seven), preserving rise-to-crest then stepwise resolution while retargeting #4→4 into the local harmony.
- **Reject:** seven delayed attacks with identical inter-onset pattern; a four-note chord on every attack; or b6 transposed as an unrelated stable scale degree.

### Example B — displaced fragment with register contrast (required future behavior)

Context: B-flat major, 4/4. Input is invented.

- **Input:** `beat 1: 1`, `1&: 3`, `2: 5`, `2&: 7`, `3: 6`, `3&: 5`; rest beats 4–4&.
- **Expected answer:** one octave higher, entering on next bar `1&`: `5, 7, b7, 6` on `1&, 2, 2&, 3&`; four attacks versus six, delayed and displaced, retaining upper-neighbor tension and descending release. Final gap >=0.5 beat.
- **Reject:** response begins before the input gap in normal sparse state; a global dominant-bebop scale silently changes the call; or answer continues beyond Panic/stop.

### Example C — current-engine truth test

Configure one CanonVoice: `PassThrough`, `voice_count=1`, `delay_beats=1`, `transpose_degrees=2`, `time_ratio=1`, `HoldMode::Forever`. Input four attacks `1,2,3,5` at beats `0,.5,1,1.5`.

- **Actual valid expectation:** four attacks, each +2 diatonic degrees, at beats `1,1.5,2,2.5`, with correspondingly delayed durations.
- **Wording allowed:** “delayed diatonic echo.”
- **Wording rejected:** “shortened answer,” “motivic chase,” or “Parker/Gillespie call-and-response.”

## Exact mapping to current Contrapunk capabilities

| Requirement | Current capability | Honest mapping / limitation |
|---|---|---|
| Stable tonic/scale | `HarmonyEngine` key + 57 `ScaleMode`s | Available; no chord-timeline-aware scale switching in this preset wave. |
| Functional/chromatic bebop line | User input passes chromatic MIDI; modes include `BebopDominant`, `Diminished*`, `SuperLocrian`, `BarryHarris` | Static scale selection cannot represent progression-specific chromatic function. HarmonyEngine transforms pitches; it does not compose bebop lines. |
| Delayed response | `CanonVoice.delay_beats` (0–16), transport-driven `tick` | Available as fixed-delay echo. Transport should be required for predictable chase timing. |
| Transposed response | `transpose_degrees` (-7..7) through `Scale::harmonize_smart` | Available, but diatonic/chromatic fallback is not harmonic-target recognition. |
| Time scaling | `time_ratio` (0.125..8), anchor-relative onset and duration scaling | Available mathematically. For ratio <1, later source events may compute fire times already in the past and then emit only when received/ticked; without phrase capture it cannot causally replay an arbitrarily long phrase at true diminution unless delay is sufficiently large. Existing diminution test rewinds transport after scheduling, so it does not prove live causal compressed replay. |
| Monophonic answer | Per-voice `harmony_mode=PassThrough`, `voice_count=1`, `voice_position=0`, VL off, octave none | Available and should be mandatory. Default/inherited mini-engine is 2 voices and can emit a harmony stack, which is wrong for this preset. |
| Preserve note lengths | Delayed NoteOff scheduling from source duration × `time_ratio` | Available. Shorter duration is not fewer-note truncation. |
| Hold/release | `Cancel`, `NearFuture`, `PhraseEnd`, `Forever`; precedence voice > lane > global | Available. Prefer `NearFuture` for bounded tails; `PhraseEnd` is only one bar from anchor and not detected phrase completion. |
| Phrase boundary | Two-beat silence resets `sequence_anchor` | Only anchor housekeeping. No buffer closure, phrase object, or response trigger after gap. |
| Shortened answer | None | **Named gap: phrase capture + fragment/truncation selection.** Current lane always schedules every input attack (and possibly extra harmony pitches). |
| Motif recognition/transformation | None | **Named gap: bounded phrase/motif Lane** (planned Wave 4). |
| Accent/swing/displacement model | Raw input timing/velocity copied; fixed delay/time ratio only | **Named gap: accent/swing and advanced timing lane** (planned Wave 5). No accent derivation, swing estimation, or groove-aware answer placement. |
| Form/harmonic timeline | Static engine settings | **Named gap: formal/harmonic timeline** (planned Wave 6). |
| Adaptive density/section arc | No phrase/density sensing in CanonLane | **Named gap: intensity model / Sparse-Building-Full scenes** (planned Wave 6). |
| Lifecycle | `reset_runtime`; disable clears queues/held state; Hold tests cover pending/on/off pairing; orchestrator resets all lanes | Strong baseline, but preset acceptance must explicitly test Panic, stop, disable, reconfigure, and replacement with zero active/pending notes on every surface. |

### Explicit verification: can Free Imitation create shortened delayed answers now?

**No.** Inspection of `CanonLane::on_input` shows that every source `NoteOn` immediately produces one pending event for every pitch in `compute_voice_stack`; there is no phrase buffer from which to choose a shorter subset. `time_ratio < 1` compresses scheduled relative times and NoteOff durations but never removes attacks. `HoldMode::Cancel/NearFuture/PhraseEnd` can cancel future emissions based on source release and horizon, but cancellation is incidental per-note queue filtering, not motif-aware truncation and cannot guarantee a coherent shortened answer. The lane does not analyze accents, swing, motifs, cadence targets, or input gaps as response triggers. Consequently, **Free Imitation is a bounded delayed imitation engine, not a truthful implementation of the catalog Result “fast phrases are pursued by shortened delayed answers.”**

Relevant inspected tests confirm fixed delayed emission, per-voice delay, duration preservation, diminution/augmentation timing, phrase-anchor reset, Hold behavior, and reset lifecycle. None asserts `answer_note_count < call_note_count`, response-after-gap, motif preservation, swing/accent, or chord-targeted chromatic resolution. The existing `diminution_voice_plays_at_double_speed` test schedules the second note at beat 1 and then resets/advances transport backward to beat 0.6 before tick; this is not evidence that live real-time diminution can emit that note at beat 0.5, because the source note did not exist then.

## Confidence and competing interpretations

- **High confidence:** 1940s bop is tonal-functional beneath chromatic surface; Parker uses flexible recurring paths/formulas; *Ko Ko* trades eight-bar units; current Canon copies every input NoteOn and lacks phrase/motif/accent/swing logic.
- **Medium-high confidence:** monophonic, transformed, shortened response is the least misleading product translation of “Bebop Chase.” It is a design synthesis, not a claim that Parker and Gillespie habitually answered every phrase this way.
- **Medium confidence:** Parker/Gillespie contrast as fluid/asymmetric versus declarative/triplet/high-register. Gillespie’s own interview and Martin’s triplet observation support it, but the most detailed accessible Gillespie dissertation analyzes 1956–59, not 1945.
- **Competing interpretation 1 — formula vs motive:** Owens/Love treat recurrent material as formulas/schemata; Martin argues thematic/motivic influence can be subtler and stronger than a purely formulaic account. Implementation should preserve resemblance without claiming to model either musician’s cognition.
- **Competing interpretation 2 — “call-and-response”:** one may use the term broadly for traded solos/sections, or narrowly for a direct musical answer. *Ko Ko* supports alternation in eight-bar units; *Ornithology* supports an explicit traded figure. Neither supports calling arbitrary fixed-delay note copying a Parker/Gillespie chase.
- **Competing interpretation 3 — swing:** swung eighth inequality versus accent/onset microtiming varies with tempo and player. Evidence here supports mobile accent and offbeat phrasing, not one fixed ratio; no exact swing ratio should be encoded without dedicated corpus measurement.

## Sources

### Kept

- Stefan C. Love, [“Possible Paths”: Schemata of Phrasing and Melody in Charlie Parker’s Blues](https://mtosmt.org/issues/mto.12.18.3/mto.12.18.3.love.php) — peer-reviewed, transcription-based corpus; phrase, schema, harmony, chromaticism, cadence and competing interpretations.
- Henry Martin, [“Four Studies of Charlie Parker’s Compositional Processes”](https://mtosmt.org/issues/mto.18.24.2/mto.18.24.2.martin.html) — peer-reviewed score/recording analysis; motivic transformation, tonal voice leading, *Ornithology* call-response, Parker/Gillespie triplet distinction.
- Library of Congress, [Ed Komara, “Ko Ko” National Recording Registry essay](https://www.loc.gov/static/programs/national-recording-preservation-board/documents/ko-ko.pdf) — institutional recording history and 1945 context.
- NPR Weekend Edition, [“The Story of Charlie Parker’s ‘Ko Ko’”](https://text.npr.org/1081208) — reputable institutional account quoting Gary Giddins, Phil Schaap, and Parker; exact trading structure, accent/phrase description, primary interview excerpts.
- Music Technology Group, Universitat Pompeu Fabra, [JAAH *Shaw ’Nuff* annotation](https://mtg.github.io/JAAH/data/shaw_nuff.html) — audio-aligned key, meter, chord/form and solo-section timings for the core joint recording.
- National Jazz Archive, [Les Tomkins interview with Dizzy Gillespie (1973)](https://jazzpro.nationaljazzarchive.org.uk/interviews/Dizzy%20Gillespie.htm) — primary testimony on piano/harmonic thinking, chord destination, relationship with Parker, Minton’s/Monroe’s, and later Afro-Cuban development.
- NEA, [John Birks “Dizzy” Gillespie](https://www.arts.gov/honors/jazz/john-birks-dizzy-gillespie) — institutional career boundary and Gillespie’s co-leadership of bebop / later big-band and Afro-Cuban work.
- Nicholas Walter Schroeder, [*The Musical Language of John Birks “Dizzy” Gillespie*](https://hdl.handle.net/2142/90991) — doctoral research metadata/abstract; useful caution that Gillespie rhythmic notation is complex, but period mismatch and access restriction limit use.

### Dropped or limited

- Scribd transcription uploads — provenance/access problems; peer-reviewed MTO analyses supersede them.
- Generic jazz-chord and SEO pages — insufficient authority and tend to reduce bebop to scales/licks.
- “Bird Over Time” Zenodo item dated after the project’s current context and not necessary for the established claims.
- Schroeder dissertation details beyond its abstract — full text institution-restricted and corpus is 1956–59, so it cannot establish exact 1945 behavior.
- Howard University Gillespie transcript — promising primary institutional source, but automated fetch returned no usable content in this run.

## Gaps and next steps

1. Obtain a legally accessible, measure-level transcription comparison of Parker’s and Gillespie’s respective *Shaw ’Nuff* choruses; use it to quantify attack density, rest lengths, triplet proportion, register and accent placement without copying melody into the product report.
2. Implement or wait for the planned bounded phrase/motif Lane: capture on gap, maximum notes/bars, salience selection, coherent truncation, ownership, and lifecycle tests.
3. Add a groove/accent representation before claiming swing-aware chase behavior.
4. Until those gaps close, either change preset 25’s Result to **“fast bursts receive delayed, monophonic diatonic echoes”** and describe Parker/Gillespie only as contextual inspiration, or keep the preset unavailable with missing capabilities `phrase_capture`, `fragment_truncation`, `motif_transform`, and `accent_swing_timing`.
