# Performer / HCI Research: Preset 48 — “Crystal Chorale”

**Role:** Independent performer and interaction research
**Reference:** Nobuo Uematsu, bounded to selected *Final Fantasy* music and interviews; this is not an imitation claim
**Operational target:** the current shared harmonic-minor chorale plus one bounded octave mapping; clean monophonic guitar or single-note MIDI keyboard input
**Research date:** 2026-07-19

## Summary

“Crystal Chorale” should be sold as a responsive arrangement of a melody the player authors, not as an Uematsu theme generator. The player supplies a clear four- or eight-bar tune, repeats and varies it, holds cadences, and leaves phrase-end silence; the system can place each note in a shared harmonic-minor chorale and add one register-bounded octave reinforcement, but it cannot invent memorable thematic identity, recognize phrases, orchestrate independent parts, follow game state, or reproduce Nobuo Uematsu’s authorship.

The proposed contract is deliberately simple: 4/4 at **72–88 BPM**, one clean note at a time, mostly quarter and eighth notes, **3–6 attacks per bar**, **MIDI velocity 60–92**, a practical input register of **MIDI 57–74 (A3–D5)**, and a held destination of **2–4 beats** followed by at least **2 beats of silence**. Transport is useful as a player-facing ruler but is not required for the underlying note-to-harmony mapping.

## 1. Scope and bounded reference corpus

This performer contract draws only on the following bounded evidence:

- Uematsu’s original *Final Fantasy* **“Prelude”** (1987), considered here as a recurring series melody, not as material to copy. Uematsu recalled being asked for an opening-screen piece with 30 minutes remaining and said he never expected it to persist across the series. [Red Bull Music Academy interview](https://daily.redbullmusicacademy.com/2014/10/nobuo-uematsu-interview/)
- The series history of differently arranged **Preludes**. Square Enix itself describes *The Preludes since 1987* as a compilation spanning the original and later arrangements. [Square Enix Store](https://gb.store.square-enix-games.com/final-fantasy-the-preludes-since-1987-cd)
- Melody-led, narrative theme practice in the 16-/32-bit period, represented by *Final Fantasy IV* (1991), *VI* (1994), *VII* (1997), and *IX* (2000), plus the intimate melody-led writing of *Final Fantasy X* (2001). These works define the comparison boundary; they do **not** establish one universal “Uematsu scale.”
- Uematsu’s direct statement, “I always begin to compose the melody first.” [archived Eric Steffen interview](https://web.archive.org/web/20160305111131/http:/www.nobuouematsu.com/steffen.html)
- Scholarship on the 16-bit soundtracks’ multiple leitmotivic strategies, rather than a single reusable harmonic recipe. [Miyake, “Leitmotivic Strategies in Nobuo Uematsu’s *Final Fantasy* Soundtracks,” *Music Theory Spectrum* 45/2 (2023)](https://doi.org/10.1093/mts/mtad009)
- Sean Atkinson’s analysis of thematic recurrence, transformation, delayed combination, and narrative association in *Final Fantasy IX*. [“The Musical Narrative of JRPGs”](https://www.firstpersonscholar.com/the-musical-narrative-of-jrpgs/)

The numeric performance ranges below are **interaction design recommendations**, not historical claims that Uematsu used those exact tempi, velocities, registers, or phrase lengths.

## 2. Performer-facing style thesis

The useful, honest intersection is **melody first + clear recurrence + restrained harmonic expansion + cadence space**. Uematsu’s own melody-first description supports asking the player to bring a tune rather than asking the engine to synthesize one. His account of working inventively with few voices also supports a lean texture rather than maximal note generation: he said limited hardware encouraged ingenuity, and described making classical- or rock-like music with three sounds. [Red Bull Music Academy interview](https://daily.redbullmusicacademy.com/2014/10/nobuo-uematsu-interview/)

The preset’s harmonic-minor chorale and octave layer are therefore only an **arrangement frame**. Harmonic minor supplies a dark, goal-directed color; it is not the identity of the original Prelude. Indeed, Masayoshi Soken described Uematsu’s familiar Prelude as bright and major-key before deliberately converting it to minor to fit *Final Fantasy XVI*’s darker world. That evidence directly warns against equating “minor + arpeggio/chorale” with a timeless Final Fantasy essence. [Game Informer interview with Soken](https://gameinformer.com/exclusive-feature/2023/06/02/the-music-of-final-fantasy-16-part-1-creating-the-prelude)

## 3. Best source gesture

**Best source:** a single-note, singable, original four-bar motif-phrase, optionally expanded to eight bars by repetition with one controlled variation.

A good source phrase has:

- one recognizable opening rhythm;
- mostly stepwise motion with one salient leap no larger than a sixth;
- a limited pitch collection of roughly five to seven scale degrees;
- one or two repeated notes used intentionally, not as tremolo;
- a clear destination held for 2–4 beats;
- a silent boundary after the destination.

Do **not** play Uematsu’s copyrighted themes as the acceptance material. The system does not need them and cannot derive their authorship from arbitrary notes.

## 4. Suggested register, tempo, note length, articulation, dynamics, density, and silence

| Dimension | Contract | Why it is useful |
|---|---|---|
| Meter | 4/4 for onboarding and acceptance | Makes four-/eight-bar phrasing and cadence rests audible without implying the corpus only uses 4/4. |
| Tempo | **72–88 BPM**; default test at 80 BPM | Slow enough for the generated chord to settle and for guitar pitch tracking to stabilize, but not so slow that the melody loses continuity. |
| Input register | **MIDI 57–74 (A3–D5)** preferred; hard guidance **55–76 (G3–E5)** | Leaves room for a ±12-semitone octave placement and supporting chorale voices without immediately crowding register extremes. |
| Note lengths | Eighths and quarters at about **65–85% gate**; half/whole-note destinations at **90–100% gate** | Separates attacks while preserving a vocal line; long destinations let the chorale be heard. |
| Articulation | Clean, lightly connected non-legato; one intentional attack per note | Prevents guitar detection debris and keyboard overlaps from multiplying generated notes. |
| Velocity | **60–92** normal; opening 64–76, local peak 82–92, cadence 68–84 | Creates a modest arc without forcing every chord at maximum intensity. MIDI velocity normally represents how hard a key was struck and often controls amplitude. [MIDI Association](https://midi.org/about-midi-part-3midi-messages) |
| Density | **3–6 attacks/bar** typical; **8/bar maximum** for one bar; never continuous sixteenths | Every source attack expands into a chorale plus octave, so source restraint protects clarity. |
| Silence | At least **2 beats** after each four-bar phrase; preferably a whole bar between A and A′ | Silence marks a phrase boundary because the current mapping does not recognize one. |
| Cadence | Hold the phrase destination **2 beats minimum**, ideally 4; release cleanly | Lets harmony register as arrival and provides a deterministic Note Off. |

## 5. Transport dependency and rhythmic precision

### Contract

- **Harmony generation itself:** transport-independent. Each clean Note On produces a related voicing; its matching Note Off releases that voicing.
- **Performance guidance and acceptance:** use running 4/4 transport or an external click so “four bars,” held cadences, and rests mean the same thing to every tester.
- **Required precision:** attacks should be within roughly an eighth note of the intended grid. Expressive microtiming is acceptable; collapsing the phrase boundary is not.
- **No promise of delayed answer timing:** “octave answer” must be understood as a bounded per-note octave relationship/reinforcement under the current baseline, **not** capture-and-replay of the completed phrase. True call-and-response would require phrase memory and scheduling.

MIDI clock can synchronize clocked equipment and supplies 24 timing clocks per quarter note, but ordinary Note On/Note Off events remain distinct performance events. [MIDI Association](https://midi.org/about-midi-part-3midi-messages) The preset should not declare transport a hard capability merely to make a stateless mapping seem more adaptive than it is.

## 6. How the player develops material through a phrase and section

Use the following human-authored state sketch:

`Silence → A stated plainly → held cadence → breath → A′ repeated recognizably → slightly higher/stronger peak → longer final cadence → full release`

### Four-bar A

1. **Bar 1 — establish:** 3–4 medium-soft notes; present the opening rhythm and tonal center.
2. **Bar 2 — continue:** reuse one rhythmic cell; move away from the opening register by step or one leap.
3. **Bar 3 — intensify:** reach the local register/velocity peak; do not increase beyond 6 attacks.
4. **Bar 4 — cadence:** reduce density and hold one destination for 2–4 beats; then release and leave at least 2 beats empty.

### Four-bar A′

1. Preserve the first bar’s rhythm and contour closely enough that a listener—not the software—can hear recurrence.
2. Change only **one or two** middle pitches, or shift one note up an octave if it stays within the safe register.
3. Keep density equal to or lower than A; variation should come from contour, dynamics, or a changed destination, not a note storm.
4. Hold the final destination longer than A and release all input before changing preset or stopping transport.

### Eight-bar option

Treat bars 1–4 as antecedent and 5–8 as consequent. Repeat the opening cell in bar 5, peak in bars 6–7, and reserve the longest hold and clearest rest for bar 8. The system supplies no formal logic; the player must make the eight-bar arc audible.

## 7. Listening and responding instructions

1. Play the first note and listen for three things before accelerating: the source pitch remains intelligible, the chorale does not mask it, and only one octave-related reinforcement appears.
2. During ordinary notes, wait only long enough to hear the chord attack settle; do not wait for a nonexistent memorized phrase response.
3. On the bar-4 destination, hold until the inner voices are perceptible as a stable color rather than a transient stack.
4. Release fully and listen for silence. If any generated note remains, stop the exercise and use Panic; a stuck note is a lifecycle failure, not ambience.
5. Begin A′ only after the phrase gap. Preserve the opening rhythm so the listener can compare A and A′.
6. At the final cadence, listen for the octave voice staying in its bounded register rather than jumping repeatedly across the melody.

## 8. Guitar-specific clean-monophonic contract

“Clean monophonic” is an input safety contract, not a claim that arbitrary guitar audio is unambiguous.

- Use a clean DI signal **before** distortion, delay, chorus, or reverb; tune to concert pitch.
- Play one fretted note at a time, preferably on the middle strings and around 5th–12th position for the preferred A3–D5 range.
- Mute the previous string with both hands before the next attack. No open-string sympathetic ring.
- Pick near-consistent attacks at moderate force. Avoid pick scrapes, fret slap, tapping, harmonics, and palm-muted double attacks.
- Use discrete fretted semitones. Avoid bends, wide vibrato, slides across semitone boundaries, and pre-fretting noise during the acceptance test.
- Leave a small physical separation between repeated notes rather than letting two strings carry the same pitch simultaneously.
- If a false note appears, stop, mute, wait for full release, and restart at the next bar; do not “play through” and compound the generated stack.

This strictness is evidence-based: conventional guitar signals contain shifting harmonics, buzz, and noisy attacks; lower pitches take longer to identify reliably; pitch-to-MIDI velocity is less straightforward than keyboard velocity; and systems designed for conventional pickups often require rigid single-note playing because simultaneous notes can produce an indeterminate result. The same source recommends clean input first in the chain and notes that finger lifts/fretting can trigger short, low-velocity rogue notes. [Sound On Sound, “Guitar To MIDI Explored”](https://www.soundonsound.com/techniques/guitar-midi-explored?amp=)

**Guitar ceiling:** expressive bends and vibrato may be musically attractive, but they are deliberately excluded from the deterministic preset contract. Add them only after verifying the surface’s pitch-bend routing and the receiving instrument’s bend range.

## 9. MIDI-keyboard opportunities and chord-density limits

- Play the melody with one hand and one finger at a time in A3–D5.
- Use velocity intentionally: the keyboard’s switches identify pitch and release unambiguously, while velocity provides a reliable attack-strength dimension compared with audio pitch tracking. [Sound On Sound](https://www.soundonsound.com/techniques/guitar-midi-explored?amp=)
- Do not use sustain pedal in the acceptance test. The source Note Off must visibly and audibly release every generated voice.
- Do not play dyads or chords. A two-note input is not “richer melody”; it asks the harmony engine to expand two simultaneous sources and can double the note count.
- Avoid legato overlaps between successive keys. Release the old key just before or with the next attack.
- Optional aftertouch, mod wheel, and pitch bend are outside this preset contract. MIDI defines them, but the preset does not promise consistent routing or generated-voice behavior. [MIDI Association](https://midi.org/about-midi-part-3midi-messages)
- If testing an 88-key controller, do not exploit extreme bass/treble merely because the keys exist; the bounded octave relation needs headroom.

**Chord-density limit:** exactly **one source note active**. If the implementation accepts polyphonic keyboard input globally, product guidance must still describe chords as unsupported for this preset rather than silently treating them as valid.

## 10. Bounded octave-answer mapping contract

The minimum honest baseline is:

1. For each accepted source note, generate **at most one** octave-related note.
2. Its pitch is exactly source **+12 or −12 semitones**.
3. Prefer +12 while the result stays at or below **MIDI 86 (D6)**; otherwise use −12 if it stays at or above **MIDI 45 (A2)**.
4. Never emit both octave directions for one source note.
5. Do not recursively harmonize the generated octave.
6. The octave Note Off is owned by and paired with the source Note Off.
7. Repeated source notes may repeat the octave, but must not accumulate held duplicates.
8. If no octave candidate fits the output bounds, omit the octave rather than clamp to a non-octave pitch.

This mapping creates register breadth, not an independent answer. It has no memory, contour, articulation plan, orchestral role, or awareness of A versus A′.

## 11. Failure gestures and recovery

| Failure gesture | Likely result | Recovery |
|---|---|---|
| Guitar distortion/delay/reverb before detection | extra or late pitch events | Bypass effects; feed clean DI; Panic and restart. |
| Unmuted adjacent/open strings | transient dyads or rogue low-velocity notes | Mute both hands; wait for silence. |
| Slides, bends, wide vibrato | chromatic chatter or unstable note identity | Use discrete fretted notes for this preset. |
| Keyboard dyads/chords or sustain pedal | multiplied chorales, mud, unclear Note Off ownership | One key at a time; pedal up; release all keys. |
| Continuous sixteenths or repeated-note tremolo | note storm; harmony never settles | Return to ≤8 attacks/bar and add a half-bar rest. |
| Melody below G3 | low chorale congestion and slower guitar tracking | Move source up an octave. |
| Melody above E5 | octave voice competes with or exceeds useful ceiling | Move source down an octave. |
| No phrase-end rest | no audible form; generated texture seems continuous | Insert at least 2 beats, preferably a bar, of silence. |
| Every note velocity >100 | flat, aggressive wall rather than an arc | Reset to 60–92 and reserve the peak. |
| Preset change while holding a cadence | risk of stale notes unless transactional cleanup works | Release first; implementation must still Panic/reset atomically. |

## 12. What the player creates, and what the system cannot manufacture

| Musical/product claim | Player or external author must supply | What the current system can honestly supply | What it cannot claim |
|---|---|---|---|
| Memorable authored theme | Distinct pitches, rhythm, contour, recurrence, expressive timing | Harmonic-minor support under each note | It cannot make arbitrary input memorable or compose “an Uematsu melody.” |
| Repetition and variation | Recognizable A/A′ relationship and intentional changed detail | Consistent remapping of repeated notes | It cannot recognize a motif or decide what should recur. |
| Independent orchestration | Instrument roles, entrances, countermelodies, register dramaturgy, timbral development | A fixed chorale and one octave reinforcement | Octave doubling is not a harp prelude, choir, orchestra, or independent counterline. |
| “Crystal” / Prelude identity | Original material, a suitable separate sound preset, bright/shimmering articulation, or an explicitly licensed quotation | A clear, glass-like register spread if the chosen synth supports it | Harmonic minor and octaves alone do not create the series Prelude. Square Enix documents many distinct arrangements across the Prelude’s history. [Square Enix](https://gb.store.square-enix-games.com/final-fantasy-the-preludes-since-1987-cd) |
| Phrase recognition | The player marks form through cadence and silence | Immediate per-note response | No phrase boundary detection, capture, or delayed playback. |
| Game/narrative state | A game, script, characters, scenes, and authored cue placement | No narrative sensing | It cannot know whether a crystal, character, location, battle, or revelation is present. Uematsu and Sakaguchi explicitly discussed the difficulty of expressing game emotion in real time and the role of programming. [Nintendo, *Iwata Asks*](https://www.nintendo.com/en-gb/Iwata-Asks/Iwata-Asks-The-Last-Story/Vol-2-Hironobu-Sakaguchi-Nobuo-Uematsu/4-The-Curse-of-RPG-Music/4-The-Curse-of-RPG-Music-234585.html) |
| Formal development | Antecedent/consequent design, climax, thinning, reprise, modulation, thematic combination | Stable local voicing | It cannot decide long-range form. Atkinson’s *FFIX* case study depends on recurrence and final combination accumulated across hours of narrative, not a chord rule. [Atkinson](https://www.firstpersonscholar.com/the-musical-narrative-of-jrpgs/) |
| Uematsu’s identity | Uematsu’s choices, historical influences, collaborators, medium constraints, and authorship | A bounded arrangement technique inspired by a small corpus | The preset must not promise “sound like Nobuo Uematsu.” His own interview emphasizes stylistic variety across RPG contexts. [Red Bull Music Academy](https://daily.redbullmusicacademy.com/2014/10/nobuo-uematsu-interview/) |

Daniel Comeaux’s analysis of the *Final Fantasy VI* opera further shows that important linear structures can support tonal identity and align endpoints with programmed game junctures—evidence that local chord color alone is not equivalent to the authored work. [University of Southern Mississippi thesis record](https://aquila.usm.edu/masters_theses/822/)

## 13. Plain-language product copy

### Name

**Crystal Chorale**

### Result

**Your single-note melody opens into a dark harmonic-minor chorale with one octave-spaced reinforcement. Held destinations bloom; clean phrase rests keep the texture clear.**

### Play it like — short prompt

**Play an original four- or eight-bar single-note tune. Repeat its opening, vary one detail, hold the cadence for 2–4 beats, then leave space.**

### Expanded guidance

**Use one clean note at a time around A3–D5 at 72–88 BPM. Favor quarter and eighth notes, 3–6 attacks per bar, and velocities 60–92. State four bars plainly, rest, then repeat the opening rhythm with only one or two changed notes. Hold the final destination longer and release completely. Guitar: clean DI, tuned, muted strings, no chords, bends, slides, delay, or distortion. Keyboard: one key at a time, sustain pedal off. The preset harmonizes what you author; it does not compose a Final Fantasy theme or replay your phrase.**

### Honest capability note

**Octave “answer” means one bounded octave-related note per source note, not a learned call-and-response phrase. Sound design is separate.**

### Reference line

**Reference context: melody-led, limited-voice, thematically recurring writing in a bounded Nobuo Uematsu / *Final Fantasy* corpus. No imitation or identity claim.**

## 14. Lifecycle and interaction safety contract

1. **Apply:** applying the preset issues one Panic, clears active notes and lane/runtime state, validates the complete snapshot, then commits atomically. If validation fails, the previous arrangement remains intact.
2. **Start:** the preset may be played with transport stopped because its baseline mapping is per note. Starting transport must not create notes by itself.
3. **Note ownership:** every chorale and octave Note On is owned by one source Note On; the matching source Note Off releases all of them exactly once.
4. **Held cadence:** “held” means the player continues holding the source note. It does not mean a hidden Forever hold or sustain latch.
5. **Repeated pitch:** releasing and replaying the same pitch must not leave duplicate generated notes active.
6. **Input failure:** on a rogue guitar trigger, the player may mute and restart; Panic must clear the entire generated set immediately.
7. **Stop:** stopping transport must not strand notes. Since transport is not required for mapping, ordinary source Note Off still governs release.
8. **Preset replacement/disable:** one Panic, runtime reset, and no carried cadence or octave state.
9. **End state:** after all source notes are off, active and pending generated-note counts must both be zero.
10. **Surface honesty:** any surface unable to guarantee atomic apply and matching cleanup must keep Apply disabled with a specific capability message.

## 15. Deterministic 36-second acceptance exercise

### Setup

- 4/4, **80 BPM** (3 seconds/bar); total **12 bars = 36 seconds**.
- Harmonic-minor tonic chosen by tester; degree notation below is relative.
- Input register: place degree 1 at **A3 (MIDI 57)**, so all notes remain in range.
- One input note at a time; velocity as written; sustain/hold disabled except physical key/fret duration.
- Guitar uses clean DI and discrete fretted notes; keyboard sustain pedal is up.

### Original abstract input

All unmarked notes are quarter notes with a small separation. `h` = half note, `w` = whole note, `R` = rest. Degrees belong to harmonic minor: `1 2 ♭3 4 5 ♭6 7`.

| Bar | Input | Velocity / intent |
|---:|---|---|
| 1 | `R(w)` | Establish silence and verify no autonomous output. |
| 2 | `1, ♭3, 4, 5` | 68, 72, 74, 76 — state A. |
| 3 | `♭6, 5, 4, ♭3` | 78, 76, 72, 70 — return by step. |
| 4 | `2, 7, 1(h)` | 70, 76, 78 — leading-tone arrival; hold 1 for 2 beats. |
| 5 | `5(h), 1(h)` | 72, 76 — sparse cadence; release exactly at barline. |
| 6 | `R(w)` | Full phrase gap; active/pending count must reach zero. |
| 7 | `1, ♭3, 4, 5` | 70, 74, 78, 80 — recognizable A′ opening. |
| 8 | `♭6, 5, 7, 5` | 82, 80, 86, 78 — change one middle contour detail. |
| 9 | `4, ♭3, 2, 7` | 78, 74, 72, 80 — controlled descent and approach. |
| 10 | `1(w)` | 82 — hold the final destination for 4 beats. |
| 11 | `R(w)` | Verify complete release and silence. |
| 12 | `R(w)` | Stop transport, replace/disable preset, verify cleanup remains zero. |

### Pass criteria

- No sound in bars 1, 6, 11, or 12 after expected releases.
- Every input attack yields one intelligible source relationship, the configured shared harmonic-minor chorale, and **no more than one** octave-related pitch.
- Every octave pitch equals input ±12 semitones and remains within MIDI 45–86; no octave recursion or duplicate accumulation.
- A and A′ sound recognizably related because the **player** repeats bars 2/7; no claim that the engine recognized them.
- The two-beat cadence in bar 4 and whole-note cadence in bar 10 remain stable until source release, then all owned output notes stop.
- The exercise never exceeds one active source note, 4 attacks/bar, or velocity 86.
- At the end of bar 6 and again after bar 10 release: **0 active notes and 0 pending events**.
- Stopping and replacing/disabling in bar 12 emits no new Note On and leaves **0 active / 0 pending**.
- Any stuck note, extra octave, non-octave clamping, autonomous replay, or output during a rest is a failure.

## 16. Confidence, competing interpretations, and gaps

### Confidence

- **High:** the preset must be melody-led and must not claim to generate Uematsu’s identity. This is supported by Uematsu’s direct melody-first statement and by scholarship treating thematic recurrence and transformation as authored/narrative processes.
- **High:** clean, monophonic, muted guitar technique is necessary for deterministic audio-to-MIDI interaction; the guitar-to-MIDI source directly documents attack ambiguity, latency, simultaneous-note difficulty, and rogue triggers.
- **High:** Note On, Note Off, and velocity are distinct MIDI performance data, so deterministic ownership/release is a valid interaction requirement.
- **Medium:** the exact 72–88 BPM, A3–D5, velocity 60–92, and density limits. They are conservative, testable product choices derived from the expansion factor and input technologies, not corpus facts.
- **Medium:** “chorale” as the best arrangement label for this bounded corpus. It communicates sustained multi-voice harmony, but the selected works contain much broader textures.
- **Low as an Uematsu-specific claim:** harmonic minor. It is a usable product color, but neither the original Prelude nor the bounded corpus can be reduced to harmonic minor.

### Competing interpretations

- **Prelude interpretation:** “Crystal” could suggest the recurring bright arpeggiated Prelude. The current harmonic-minor chorale instead points toward a darker, later reinterpretive color. Product copy should preserve “crystal” as imagery and avoid claiming a faithful Prelude mechanism.
- **Octave answer interpretation:** users may hear “answer” as a delayed second phrase. The current safe baseline is simultaneous/per-note octave reinforcement. Rename it “octave reinforcement” in UI copy unless a true phrase-capture lane exists.
- **Chorale interpretation:** a fixed chord stack can sound hymn-like, but independently voiced orchestration and long-range voice leading are stronger claims. Keep “chorale” descriptive, not analytical proof of the referenced corpus.

### Gaps and next steps

- No authoritative published score editions were inspected in this performer pass, so register and phrase recommendations are not measure-by-measure transcriptions.
- The exact current HarmonyEngine voicing, voice count, and collision policy were intentionally not inferred here; the parent synthesis must bind this contract to the verified shared configuration.
- Guitar tracking differs by surface, pickup, buffer size, threshold, and room noise. Run the same deterministic exercise on every enabled guitar surface before making cross-surface latency claims.
- If a future phrase lane adds real octave call-and-response, research and test capture boundaries, quantization, delayed ownership, transport stop, reconfiguration, and cancellation separately. Do not silently upgrade the meaning of the current copy.

## Sources

### Kept

- [Nobuo Uematsu interview — Red Bull Music Academy Daily (2014)](https://daily.redbullmusicacademy.com/2014/10/nobuo-uematsu-interview/) — direct evidence on the Prelude’s origin, hardware constraints, stylistic breadth, harmony, and preferred expressive territory.
- [Eric Steffen interview with Nobuo Uematsu (archived)](https://web.archive.org/web/20160305111131/http:/www.nobuouematsu.com/steffen.html) — direct melody-first statement and evidence of close work with technical limitations.
- [Iwata Asks: “The Curse of RPG Music”](https://www.nintendo.com/en-gb/Iwata-Asks/Iwata-Asks-The-Last-Story/Vol-2-Hironobu-Sakaguchi-Nobuo-Uematsu/4-The-Curse-of-RPG-Music/4-The-Curse-of-RPG-Music-234585.html) — primary discussion with Uematsu and Sakaguchi about melody, game emotion, programming, scene function, and resisting generic RPG assumptions.
- [Square Enix, *Final Fantasy — The Preludes since 1987*](https://gb.store.square-enix-games.com/final-fantasy-the-preludes-since-1987-cd) — official evidence that the Prelude exists through many arrangements across time.
- [Miyake, “Leitmotivic Strategies in Nobuo Uematsu’s *Final Fantasy* Soundtracks”](https://doi.org/10.1093/mts/mtad009) — peer-reviewed evidence that the 16-bit soundtracks use multiple thematic/narrative strategies, resisting a one-trick caricature.
- [Atkinson, “The Musical Narrative of JRPGs”](https://www.firstpersonscholar.com/the-musical-narrative-of-jrpgs/) — scholar-authored, concrete *Final Fantasy IX* case study of recurrence, transformation, thematic family, and narrative culmination.
- [Comeaux, *An Analysis of Nobuo Uematsu’s Linear Structures: The Score of Final Fantasy VI’s Opera*](https://aquila.usm.edu/masters_theses/822/) — university-hosted thesis record supporting the distinction between local sonority and authored linear/game-programmed form.
- [Game Informer, “The Music of Final Fantasy 16: Part 1 — Creating the Prelude”](https://gameinformer.com/exclusive-feature/2023/06/02/the-music-of-final-fantasy-16-part-1-creating-the-prelude) — direct Soken interview clarifying that minor treatment is a later, project-specific transformation of the recognizable bright Prelude.
- [MIDI Association, “About MIDI — Part 3: MIDI Messages”](https://midi.org/about-midi-part-3midi-messages) — authoritative explanation of Note On, Note Off, velocity, mono/poly modes, clock, and stuck-note implications.
- [Sound On Sound, “Guitar To MIDI Explored”](https://www.soundonsound.com/techniques/guitar-midi-explored?amp=) — specialist practical source on pitch-tracking ambiguity, clean input, latency, chords, bends, thresholds, and rogue guitar events.
- [Anatone, ed., *The Music of Nobuo Uematsu in the Final Fantasy Series*](https://www.intellectbooks.com/the-music-of-nobuo-uematsu-in-the-final-fantasy-series) — scholarly book metadata and chapter scope confirming the breadth of relevant analytical approaches; used for corpus restraint, not detailed musical claims.

### Dropped

- **Final Fantasy Wiki / Fandom, “Prelude (theme)”** — useful orientation but fan-edited and unnecessary where Square Enix and direct interviews cover the recurring Prelude.
- **Hooktheory, “Final Fantasy — Prelude”** — crowd/consumer analysis with unclear score authority; excluded from harmonic claims.
- **YouTube score-analysis and soundtrack uploads** — provenance, edition, and copyright context were inconsistent; no melody was transcribed from them.
- **MusicBrainz recording/work pages** — useful catalog metadata but redundant with Square Enix’s official compilation description.
- **Game Informer report on “To Zanarkand” originally being written for flute** — interesting recent anecdote but secondary reporting and not needed for this input contract.
- **Marketing epithets such as “the Beethoven of video game music”** — excluded as caricature rather than operational evidence.
- **Unreviewed fan claims that harmonic minor, arpeggios, or crystals define Uematsu’s style** — rejected because the bounded evidence shows stylistic variety, project-specific arrangement, and narrative thematic work.
