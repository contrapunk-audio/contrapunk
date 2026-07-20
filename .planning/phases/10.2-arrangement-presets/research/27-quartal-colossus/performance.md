# Preset 27 — “Quartal Colossus”: Performer / HCI Research Report

**Role:** Independent performer and interaction research
**Preset status assumed:** existing shared `HarmonyEngine` baseline using scale-derived fourth stacks; no pattern, bass, phrase-memory, scene, or artist-model layer
**Reference scope:** McCoy Tyner’s forceful modal/quartal vocabulary, especially the 1960–65 John Coltrane quartet context, as a bounded reference—not a career-wide simulation or claim of imitation
**Primary users:** clean monophonic guitar and MIDI keyboard performers

## Executive conclusion

The most reliable source gesture is an **assertive, monophonic, one- or two-bar modal riff**, repeated with deliberate accents and then answered by silence. In D Dorian, the player supplies the tonic, rhythm, accent pattern, register, dynamics, phrase development, and rests; the shared engine can supply an immediate scale-derived fourth stack for each accepted note. That is a playable contract, but it is not “McCoy Tyner in a box”: Tyner described an open modal setting as freedom for personal expression, and specifically resisted reducing his practice to named scales or analysis alone ([Jazz Resource Center interview, 2005](http://www.jazzcenter.org/tyner/interview_2005.htm)). Scholarship likewise warns that his early language balanced bebop syntax, pentatonic melody, quartal harmony, and mainstream jazz practice rather than consisting only of fourths and pentatonics ([Satterthwaite, 2020](https://digital.library.unt.edu/ark:/67531/metadc1703303/)).

## 1. Best source gesture

### Recommended gesture

Play **single-note riffs of 3–6 notes**, normally lasting one or two bars, with:

- a clearly repeated rhythmic cell;
- one or two accented attacks;
- mostly Dorian or minor-pentatonic pitch material;
- a release or full rest after the cell;
- one controlled change on repetition (ending pitch, register, rhythm, or intensity).

A riff is better than an arbitrary scale run because repetition lets the ear recognize both the input idea and the changing force of its harmonized attacks. A riff is better than a held tone because this preset’s identity depends partly on attack and rhythmic insistence. A chord is the wrong source gesture: each chord tone can request its own stack, multiplying density without adding a coherent comping role.

Tyner connected his musical language to rhythm, dance, African music, and drums, and described Elvin Jones as a player who was powerful but listened; he also described the fourth-based language as something developed from broad listening and the need for a personal voice ([Dan Tepfer’s transcript of a 1984 Tyner interview](https://dantepfer.com/blog/?p=842)). Those statements support a player-led rhythmic, responsive contract—not a “hold any note for instant style” contract.

### Secondary gestures

- **Sustained tone:** useful only as a 1–2 beat arrival or a test of the stable voicing.
- **Short motif:** useful when repeated exactly once, then varied.
- **Long scalar line:** usable sparingly, but risks a continuous chord conveyor.
- **Chord input:** excluded from the default contract.

## 2. Playable parameter contract

| Dimension | Clean guitar | Keyboard | Contract and rationale |
|---|---|---|---|
| **Input** | One picked/fingered note at a time | One RH note at a time | The accepted catalog defines clean monophonic notes as the default. Contrapunk’s public architecture describes onset/pitch detection feeding a harmony engine; a clean, isolated source is therefore the honest guitar path ([Contrapunk README](https://github.com/contrapunk-audio/contrapunk/blob/main/README.md)). |
| **Register** | Start around E3–D5 (standard guitar: roughly 4th-string 2nd fret through 1st-string 10th fret) | Start around D3–A4; move up if generated voices are placed below | Keep the source out of the low bass so several fourth-related voices remain separable. Transpose an octave when the bottom of the result sounds like one mass rather than distinct attacks. |
| **Tempo** | **88–132 BPM**, preferred **96–116** | Same | Broad enough for weight and repetition without forcing fast tracking or an unbroken note storm. Faster is possible only with shorter, cleaner cells and more rests. |
| **Note length** | 45–75% of the notated subdivision; 1–2 beats for arrival tones | 50–85%; use pedal only outside this preset’s default test | Separate attacks expose rhythmic comping. Very long overlaps turn successive stacks into accumulated harmony. |
| **Articulation** | Firm, dry pick/finger attacks; mute the previous string; no wide bends during detection | Non-legato to lightly detached; no sustain pedal by default | The fourth stack should be heard as repeated punches, not a wash. Legato is acceptable for a brief contrast, not the continuous default. |
| **Velocity / dynamics** | Tracking-equivalent medium-firm attacks; avoid clipping | Unaccented MIDI 78–94; accents 105–118; optional ghost notes 55–68 | Use a **20+ velocity-unit** contrast for the acceptance exercise. MIDI velocity is not a universal loudness unit: identical values can yield different dynamics across instruments and registers ([Ichikawa et al., 2007](https://doi.org/10.1250/ast.28.27)). Tune by audible output, not numbers alone. |
| **Density** | 3–6 onsets per bar normally; brief maximum 8 | Same; never “thicken” by adding simultaneous keys | One source onset already creates a stack. Leave at least one half-bar opening every 2–4 bars. |
| **Silence** | One half-bar or full bar after every 1–2 riff statements | Same | Silence is the response space and the clearest lifecycle test. It also prevents the generated layer from becoming an undifferentiated conveyor. |

These are interaction defaults, not historical claims about a fixed Tyner tempo or MIDI dynamic. Tyner’s recorded output spans many ensembles and decades; the preset is intentionally narrower.

## 3. Dorian pitch guidance

For a tonic of **D**, use D Dorian degrees:

`1  2  ♭3  4  5  6  ♭7` = `D E F G A B C`

Practical pitch hierarchy:

1. Establish `1`, `♭3`, or `5` early.
2. Build compact riffs chiefly from minor-pentatonic degrees `1–♭3–4–5–♭7`.
3. Include natural `6` occasionally—often as `5–6–5` or `♭7–6–5`—so the result reads as Dorian rather than generic natural minor/pentatonic.
4. Use `2` as connective color.
5. Treat chromatic approaches as **player-authored tension** and resolve them immediately; do not assume the baseline preset will infer the intended outside harmony.
6. Avoid mechanically ascending and descending the whole mode. The scale constrains pitch; it does not supply a phrase.

Tyner said that an open modal sound allowed greater freedom and more personal choice than dense changes ([Jazz Resource Center interview](http://www.jazzcenter.org/tyner/interview_2005.htm)). That supports treating Dorian as a field for motifs, superimposition, and rhythmic choice, not as an instruction to run a scale. It is also important not to erase dominant-chord practice: research specifically argues that dominant sevenths were significant in Tyner’s 1960s style and underrepresented by a purely modal/quartal account ([Kivinen, *McCoy Tyner, Modal Jazz, and the Dominant Chord*](https://taju.uniarts.fi/handle/10024/6819)). This baseline preset does not represent that fuller harmonic range.

## 4. Transport and rhythmic precision

- **Transport is not required** for the core note-to-fourth-stack behavior. The performer’s NoteOns provide the rhythm.
- A metronome or running host transport is recommended for repeatable practice and testing, but the preset must remain playable when transport is stopped.
- Aim for recognizable placement rather than quantized perfection: repeat the same cell within roughly a sixteenth-note window at the chosen tempo.
- The engine should not invent a comping pattern between source onsets. Any offbeat jab, anticipation, repeated quarter-note punch, triplet, or cross-accent must be played at the input.
- If an optional global humanizer/groove changes timing, that is a separate system feature and must not be described as part of “Quartal Colossus.”

Interactive latency matters because the performer coordinates future attacks with heard results. Controlled studies find that musicians can be sensitive to action-to-sound delay and that latency degrades live interaction, though thresholds depend on task and setup ([Liebig & Jürgens, 2024](https://dl.acm.org/doi/fullHtml/10.1145/3678299.3678331); [van Vugt & Tillmann, 2014](https://journals.plos.org/plosone/article?id=10.1371/journal.pone.0087176)). The UI should therefore avoid promising a universal millisecond feel; the acceptance test uses rhythmic recognizability and repeatability on the actual surface.

## 5. Phrase and section development

### Within a phrase

Use this four-stage performer state:

`State A: Statement → State B: Reinforcement → State C: Lift/peak → State D: Release`

- **A — Statement (1–2 bars):** play one 3–5-note riff at medium velocity and mid register.
- **B — Reinforcement (1–2 bars):** repeat its rhythm; change only the last degree or add one accent.
- **C — Lift/peak (1–2 bars):** move all or part of the cell up an octave, include Dorian `6`, or increase velocity/density—not all three continuously.
- **D — Release (½–1 bar):** land on `1` or `5`, release, and stop. Let the generated notes end before beginning the next phrase.

The system does not know these states. The performer creates them through onset timing, pitch, register, velocity, and silence; the engine reacts locally to each source note.

### Across a section

A useful 8–16 bar arc is:

1. **Sparse:** one statement, one full response gap.
2. **Grounded:** repeat the cell in the original register.
3. **Building:** shorten one gap or add one repeated note.
4. **Full:** highest register and strongest accent for one phrase only.
5. **Withdrawal:** reduce to 2–3 notes and restore a full bar of silence.

Do not expect automatic accumulation, fragmentation, modulation, scene changes, or return. Those require phrase/pattern/intensity capabilities outside this shared baseline. The engine repeats its mapping; the player authors the form.

## 6. Listening and responding

After the first note of a new register:

1. Listen for the **bottom generated pitch**. If it crowds the bass or masks the source, transpose the source up an octave or lower the harmony gain.
2. Listen for at least **two distinct companion pitches** forming an open, fourth-related sonority rather than a close triad.
3. Let the entire stack speak before the next onset; at 96–116 BPM, quarter-note attacks are a good calibration speed.
4. After a 3–6-note cell, stop for at least two beats and confirm that every generated voice releases.
5. Continue only when the previous stack is either cleanly released or intentionally sustained. Do not play over unexplained lingering notes.

The musical model is call-and-listen, not call-and-autocomplete. Tyner’s recollection of Elvin Jones emphasizes force together with listening, while the quartet context placed this piano language in active ensemble collaboration ([Tepfer interview transcript](https://dantepfer.com/blog/?p=842)).

## 7. Guitar-specific constraints under the clean-monophonic claim

1. Use a clean DI or low-gain tone; minimize compression that raises string noise.
2. Play only one string at a time and mute the previous string with both hands where practical.
3. Prefer picked/fingered attacks over volume swells for reliable onset detection.
4. Avoid open-string sympathetic ringing, raked attacks, pinch harmonics, pick scrapes, and unresolved double-stops.
5. Keep bends narrow and apply them after a stable onset; for deterministic testing, use no bends, slides, vibrato, or harmonics.
6. Leave a small separation between repeated notes when retriggering matters.
7. Do not strum quartal shapes into the detector. Guitar quartal shapes are playable and commonly derived from Dorian, Mixolydian, or Aeolian collections ([Premier Guitar, “Quartal Harmony 101”](https://www.premierguitar.com/jazz-boot-camp-quartal-harmony-101)), but here the engine—not the guitarist’s simultaneous strings—owns the fourth stack.

The public Contrapunk description promises guitar pitch detection with onset detection and string/fret identification, but it does not promise arbitrary polyphonic chord transcription ([Contrapunk README](https://github.com/contrapunk-audio/contrapunk/blob/main/README.md)). The UI must preserve “clean monophonic” wording.

## 8. Keyboard-specific opportunities and limits

### Opportunities

- Velocity makes a repeated one-note or two-note cell clearly dynamic.
- Register shifts can be exact and immediate.
- A player can repeat the same cell deterministically for A/B comparison.
- Aftertouch/mod-wheel expression may shape the selected sound, but is not part of the harmony guarantee.

### Chord-density and left-hand limits

- Default to **one key at a time**. Two simultaneous source keys can yield two overlapping stacks; three or more can become a note storm.
- Do not hold the sustain pedal through several source changes.
- Do not add a conventional left-hand chord on the same harmonized input path.
- A bass/left-hand root can be performed only on a **separate pass-through/split track** if the current surface explicitly supports that routing. Otherwise omit it.
- If the left hand is sent into this preset, it is not “bass”: it becomes another harmonized source and can create low-register mud.

Tyner’s actual piano texture involved left-hand open fifth/fourth sonorities, rhythmic coordination, and ensemble bass context; contemporary quotation of Tyner’s own explanation stresses that his sound was “not just a pile of fourths” and included fifths that opened the texture ([Jazz Journal, citing the *Keyboard* interview](https://jazzjournal.co.uk/2020/04/08/obituary-mccoy-tyner/)). A single shared note-mapper cannot allocate those pianistic hand roles.

## 9. Failure gestures and recovery

| Failure gesture | Likely result | Recovery |
|---|---|---|
| Guitar double-stop, ringing open string, scrape, or noisy release | False/multiple onset or wrong pitch | Mute, pause, replay one dry note; Panic if anything remains. |
| Keyboard triad/chord input | Several simultaneous fourth stacks; note storm | Release all keys, release pedal, Panic, resume monophonically. |
| Low source notes with voices below | Indistinct low cluster/mud | Transpose source up 12 semitones or choose an upper player position. |
| Continuous eighth/sixteenth stream with no gaps | Chord conveyor; no phrase hierarchy | Limit burst to one bar, then rest ½–1 bar. |
| Sustain pedal or overlapping long notes | Accumulated stacks and hidden NoteOffs | Pedal up; use 50–75% gates. |
| Every note at maximum velocity | No accent hierarchy; fatigue/clipping | Return baseline to 78–94 and reserve 105–118 for one or two attacks. |
| Mechanical Dorian scale run | Correct pitch set but weak style implication | Convert 3–5 degrees into a repeated rhythmic cell; change one ending. |
| Chromatic wandering | Harmony may be valid by fallback rules but contradict Dorian claim | Resolve approach notes immediately to a named Dorian target. |
| Expecting accompaniment during a rest | Silence; the baseline has no autonomous comping | Play the comping rhythm yourself or use a separate pattern-capable lane/preset. |

## 10. What the performer creates vs. what the system cannot manufacture

| Musical responsibility | Player can create through input | Baseline system can honestly provide | Baseline cannot manufacture |
|---|---|---|---|
| **Pitch field** | Dorian/pentatonic target notes and chromatic approaches | Scale-derived companion pitches in fourth-related stacks | Knowledge of the tune’s changes or the intended harmonic substitution |
| **Bass / left hand** | Separate routed bass line, if the surface supports it | No independent bass role in this contract | Tyner’s hand allocation, bass-player interaction, pedal point, or root choice |
| **Rhythmic comping** | Repeated onsets, offbeats, anticipation, triplets, rests | Immediate stack attacks following those onsets | An autonomous comping pattern, swing feel, or responsive drummer |
| **Accents** | Velocity and attack contrast | Propagation/rendering of the source dynamic, surface permitting | Meaningful accents from flat input |
| **Phrase evolution** | Repetition, variation, register lift, density change, withdrawal | The same bounded mapping at each event | Motif recognition, development, fragmentation, climax, cadence planning |
| **Section evolution** | Deliberate 8–16 bar energy arc | Stable behavior across the arc | Adaptive scenes, orchestration growth, modulation, or return |
| **Contextual chord choice** | Targeting notes that imply a desired context | A local fourth stack that contains the played note | Reading a lead sheet, hearing ensemble harmony, selecting Tyner’s dominant/tertian alternatives |
| **Artist identity** | Personal touch, listening, repertoire knowledge, ensemble interaction | A narrow quartal/modal color associated with the reference scope | McCoy Tyner’s identity, touch, biography, compositional voice, improvisational judgment, or quartet chemistry |

This boundary is crucial. Tyner’s Smithsonian oral history documents a career shaped by classical study, R&B experience, composition, and long-term ensemble work, not a single voicing device ([Smithsonian Jazz Oral History transcript](https://americanhistory.si.edu/sites/default/files/file-uploader/Tyner-McCoy-Transcription-2020.pdf)). Satterthwaite’s analysis similarly identifies bebop and mainstream syntax alongside pentatonic and quartal materials ([Satterthwaite, 2020](https://digital.library.unt.edu/ark:/67531/metadc1703303/)).

## 11. Plain-language product guidance

### Play it like — short prompt

> **Play a firm one-bar Dorian riff, repeat its rhythm with one stronger accent, then leave half a bar of air. Use single notes only.**

### Expanded guidance

> Start in the middle register with 3–6 clean single notes from Dorian or the minor pentatonic. Let each attack trigger its open fourth stack. Repeat the cell once; change only the ending, accent, or octave. Build for one phrase, then stop long enough to hear every generated voice release. Guitar: mute unused strings and avoid bends or double-stops. Keyboard: use one key at a time, no sustain pedal, and route any left-hand bass separately.

### Honest Result copy

> **Turns each clean modal note into a forceful, scale-derived stack of fourths. Your riff supplies the rhythm, accents, register, development, and silence. Inspired by a documented quartal/modal vocabulary associated with McCoy Tyner; it does not model his touch, complete harmonic language, ensemble interaction, or identity.**

### Copy to reject

- “Sound exactly like McCoy Tyner.”
- “AI performs Tyner-style comping for anything you play.”
- “Automatically adds Tyner’s left hand and bass.”
- “Authentic McCoy chords.”
- “Play any notes or chords.”

## 12. Deterministic 40-second acceptance exercise

### Setup

- Tonic: **D**
- Scale: **D Dorian**
- Meter: **4/4**
- Tempo: **96 BPM**
- Duration: **16 bars = 40 seconds**
- Input: one note at a time; no pedal, bends, slides, vibrato, or external groove
- Gate: ordinary quarter notes held about **0.45 seconds**; half note held **1.0 second**
- Velocity: ordinary **84**, accent **110**, soft ending **64**
- Sound: dry or short-decay sound; input and harmony both audible; no delay
- Before bar 1: Panic/reset, then apply the preset

Scale-degree notation uses D Dorian. `—` means sustain and `rest` means no held source note.

| Bars | Deterministic input (beats 1–4) | Purpose |
|---|---|---|
| 1–2 | Each bar: `1(84), ♭3(84), 4(110), 5(84)` | Establish riff and audible accent. |
| 3 | `1(84), ♭3(84), 4(84), 6(110)` | One-note Dorian variation. |
| 4 | `5(84)` on beat 1, sustain through beat 2; beats 3–4 rest | Test arrival and release opening. |
| 5–6 | Each bar: `5(110), ♭7(84), 6(84), 5(84)` | Confirm natural 6 and descending response. |
| 7 | Same pitches as bar 5, but all velocity 84 | A/B accent contrast. |
| 8 | Full-bar rest | Silence and cleanup checkpoint. |
| 9–10 | Each bar: beat 1 `1(110)`, beat 2 `1(84)`, beat 2-and `♭3(84)`, beat 4 `4(84)`; all other subdivisions rest | Player-authored syncopated comping, not autonomous rhythm. |
| 11 | Transpose bar 9’s contour up an octave, same rhythm/velocities | Register lift. |
| 12 | Full-bar rest | Second cleanup checkpoint. |
| 13 | `1(84), ♭3(84), 4(84), 5(110)` | Return. |
| 14 | `♭7(84), 6(84), 5(110), 4(84)` | Peak then descent. |
| 15 | `1(64)` on beat 1, sustain through beat 2; beats 3–4 rest | Soft cadence/withdrawal. |
| 16 | Full-bar rest; invoke Panic after beat 4 | Final lifecycle check. |

### Audibility and behavioral thresholds

Pass only if all conditions hold on the tested surface:

1. **Harmony audibility:** on at least **90% of valid source attacks**, a listener at normal monitoring level can distinguish the source plus **at least two companion pitches** without soloing. If using a meter, set the generated harmony bus no lower than **9 dB below** the dry input during the ordinary notes; this is a test mix threshold, not a psychoacoustic universal.
2. **Quartal distinction:** during the held notes in bars 4 and 15, at least two companion pitches are separately audible and the result is clearly more open than a close-position triad. Event inspection should confirm scale-derived fourth relationships according to the implemented strategy.
3. **Accent audibility:** the velocity-110 note at bar 5 beat 1 must sound clearly stronger than the same pitch at bar 7 beat 1. For meter-based testing, require at least **3 dB higher short-window peak** with the chosen velocity-sensitive sound. If the sound ignores velocity, mark dynamic acceptance **unsupported**, not passed.
4. **Rhythmic ownership:** no generated attack may occur without a source attack. In bars 8, 12, and 16 there must be no new generated NoteOn.
5. **Silence:** by **250 ms after the final source NoteOff** before each full-bar rest, no preset-owned note may remain audible with the dry/short-decay test sound, and active-note/event inspection must report zero held preset-owned notes. Natural synth tail is excluded from ownership but should be inaudible by the next bar under the specified sound.
6. **Determinism:** after Panic/reset, replaying the exact 16-bar MIDI input must produce the same generated pitch tuple for every corresponding NoteOn and the same NoteOff ownership. Timing may differ only by any explicitly enabled global humanizer, which should be off for this test.
7. **Density bound:** one source note must not fan out beyond the preset’s declared voice count. Chord input is not part of this pass case.
8. **Cleanup:** after the bar-16 Panic, active and pending notes are zero. Reapplying, disabling, or replacing the preset must also emit cleanup once and leave zero owned notes.

## 13. Lifecycle expectations

- **Apply:** Panic/clear existing owned notes once, reset state, then apply atomically; never leave the old stack sounding under the new configuration.
- **NoteOff:** every generated NoteOn is owned and receives a matching NoteOff when its source ends.
- **Stopped transport:** ordinary note mapping and release still work because this baseline is not beat-scheduled.
- **Disable/bypass:** immediately clear preset-owned notes; do not wait for transport.
- **Panic:** clear all active and pending preset events on first invocation; repeated Panic is harmless.
- **Preset replacement:** no overlap between old and new owned voices.
- **Device disconnect / lost source:** the surface must offer Panic and should clear owned notes rather than sustaining indefinitely.
- **Hold:** default acceptance assumes Hold is off. If a user separately enables Hold, the UI must attribute continued sound to Hold and its explicit lifecycle, not to this preset’s style behavior.

## 14. Confidence, competing interpretations, and gaps

### Confidence

- **High:** monophonic riff input, deliberate rhythm/accents, Dorian/pentatonic guidance, response space, no chord input, and explicit lifecycle checks are appropriate for this baseline contract.
- **High:** describing Tyner as “just stacked fourths” is misleading. Both primary interviews and scholarship contradict that caricature.
- **Medium:** the proposed register, velocity, gate, density, and tempo ranges are practical performer defaults; they are not transcriptions of a single Tyner recording.
- **Medium:** “at least two companion pitches” assumes the operational preset declares enough voices to make a stack. If the actual record has only one generated voice, the product result “stack” and this acceptance threshold disagree and implementation must be corrected or copy narrowed.
- **Low until measured:** exact audio-level, latency, and velocity-loudness behavior across built-in synths, hosted instruments, WASM, desktop, and plugin surfaces.

### Competing interpretations

- Some pedagogical accounts center quartal left-hand voicings and pentatonic right-hand lines; Satterthwaite and Kivinen show why that is incomplete. This report uses that vocabulary only because the catalog explicitly defines a bounded fourth-stack baseline.
- Dorian is a useful operational mode, not a claim that all relevant Tyner performances are Dorian. Modal centers may be heard differently depending on bass and ensemble context, which this preset does not infer.
- Force may come from loudness, density, register, repetition, touch, or ensemble interaction. The preset can expose player velocity and generate density, but cannot recreate acoustic piano touch or quartet dynamics.

### Unresolved implementation gaps

1. The exact `voice_count`, player position, octave placement, velocity propagation, and mix gain of the final ArrangementPresetV2 record must be verified against these thresholds.
2. Guitar pitch-detection error rates and safe note-separation times require surface-specific measurement; no universal value is claimed here.
3. Hosted instruments map velocity differently, so a MIDI event assertion and an audible/metered assertion are both needed.
4. Independent bass/left-hand routing, autonomous rhythmic comping, phrase evolution, and contextual harmonic selection are not present in this baseline and must not be implied by UI copy.

## Sources

### Kept

- [Smithsonian Jazz Oral History Program — McCoy Tyner transcript](https://americanhistory.si.edu/sites/default/files/file-uploader/Tyner-McCoy-Transcription-2020.pdf) — primary institutional oral history; establishes the breadth of Tyner’s training, career, composition, and ensemble experience.
- [Dan Tepfer, “McCoy Tyner: dance, drums, beauty salons & fourths”](https://dantepfer.com/blog/?p=842) — transcript of Tepfer’s 1984 interview with Tyner; direct statements about rhythm, African music, fourths, personal voice, freedom, and listening.
- [Jazz Resource Center, McCoy Tyner interview (2005)](http://www.jazzcenter.org/tyner/interview_2005.htm) — direct statements resisting reductive scale labels and favoring the personal freedom of open modal harmony.
- [Gregory Satterthwaite, *Beyond Fourths and Pentatonics* (2020)](https://digital.library.unt.edu/ark:/67531/metadc1703303/) — university-hosted doctoral analysis of selected 1962–63 recordings; the strongest source here against the quartal/pentatonic caricature.
- [Joonas Kivinen, *McCoy Tyner, Modal Jazz, and the Dominant Chord*](https://taju.uniarts.fi/handle/10024/6819) — scholarly corrective showing the importance of dominant harmony in Tyner’s 1960s practice.
- [Contrapunk README](https://github.com/contrapunk-audio/contrapunk/blob/main/README.md) — official product evidence for the current guitar onset/pitch-to-harmony architecture and shared HarmonyEngine framing.
- [Premier Guitar, “Quartal Harmony 101”](https://www.premierguitar.com/jazz-boot-camp-quartal-harmony-101) — practical instrument-specific evidence that quartal guitar voicings can be derived from Dorian and related modes; used only to distinguish playable guitar chords from this preset’s monophonic detector contract.
- [Ichikawa et al., “Sound production of MIDI piano tones in a dB scale on the basis of equal loudness”](https://doi.org/10.1250/ast.28.27) — supports the warning that MIDI velocity does not map uniformly to loudness across pitch/instruments.
- [Liebig & Jürgens, “Measuring the Just Noticeable Difference for Audio Latency” (2024)](https://dl.acm.org/doi/fullHtml/10.1145/3678299.3678331) and [van Vugt & Tillmann, “Thresholds of Auditory-Motor Coupling” (2014)](https://journals.plos.org/plosone/article?id=10.1371/journal.pone.0087176) — HCI evidence that action-sound delay is perceptually relevant and task-dependent.
- [Jazz Journal obituary/retrospective](https://jazzjournal.co.uk/2020/04/08/obituary-mccoy-tyner/) — secondary source retaining Tyner’s useful quoted warning that his practice was not merely piled fourths and included open fifths.

### Dropped

- Jazz-advice and “sound like McCoy” lesson pages — useful pedagogy but too reductive and commercially framed for core historical claims.
- Music Stack Exchange discussion of a transcribed comping passage — anonymous interpretation and no stable scholarly review.
- PDFCoffee copy of “Apart Playing” — unauthorized/unclear hosting; the Cambridge abstract was discoverable, but full text was not needed for this performer contract.
- Sheet-music-library repost dated 2025 — uncertain provenance and a publication date outside the stable scholarly record.
- Generic quartal-guitar SEO lessons — redundant once the more focused Premier Guitar source was retained.
- Commercial polyphonic guitar-to-MIDI product claims — not evidence of Contrapunk’s own clean-monophonic contract.
- Newspaper descriptions of Tyner “pounding” the piano — colorful but insufficient for operational dynamics and liable to reinforce caricature.

## Final performer contract

**Single notes in; scale-derived fourth stacks out.** Play a compact Dorian/pentatonic riff at 88–132 BPM, repeat its rhythm with one meaningful change, reserve strong velocity for accents, and leave response space. Keep guitar attacks clean and monophonic; keep keyboard input one-handed and one key at a time unless bass is explicitly routed elsewhere. The player must supply bass intent, rhythmic comping, accents, phrase and section growth, harmonic context, listening, and identity. The baseline system supplies only the bounded fourth-stack color and must cleanly release every note it creates.
