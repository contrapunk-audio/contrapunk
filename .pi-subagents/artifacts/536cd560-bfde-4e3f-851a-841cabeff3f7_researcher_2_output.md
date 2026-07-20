# Phase 10.2 Preset 01 — Cloister Organum  
## Performer / Interaction Research Report

**Proposed path:** `research/01-cloister-organum/performance.md`  
**Status:** Research only; no files modified.

## Performer-facing thesis

A performer can evoke a **bounded, Notre-Dame-organum-derived atmosphere** by supplying the behavior the preset cannot invent: a slow, chant-like monophonic line, sustained tones, mostly stepwise motion, clear cadential arrivals, and substantial silence. The preset may add a restrained halo of fourths, fifths, and octaves, but that is only an open-interval arrangement technique—not a reconstruction of Léonin’s organum duplum or Pérotin’s rhythmically coordinated three- and four-voice polyphony.

This distinction is essential. In organum purum, chant notes may be prolonged beneath an elaborate, flexibly shaped upper voice; in discant, both tenor and upper voice are rhythmically active and coordinated. Performer Penelope Turner explicitly distinguishes discantus, where both parts are active and rhythmic interpretation is comparatively agreed, from sustained-tenor passages whose rhythm remains debatable and flexible. [Turner, *Notre Dame Organum Duplum: What Does a Performer Need?*](https://www.musicandpractice.org/volume-1/notre-dame-organum-duplum-modern-edition/) Jeremy Yudkin’s reading of medieval theorists likewise argues that organum purum was not governed by the fixed modal rhythm used for discant. [Yudkin, *The Rhythm of Organum Purum*](https://www.examenapium.it/cs/biblio/Yudkin1983.pdf)

---

## 1. Best source gesture

**Best input:** one clean, sustained note at a time, joined into short, chant-like phrases.

Recommended gesture vocabulary:

- Begin with a stable held tone.
- Continue with two to five mostly adjacent notes.
- Give important notes longer duration than connecting notes.
- End the unit on a clearly held arrival, then release completely.
- Reuse a small pitch area rather than continuously introducing new material.

The player should behave like the source of a simplified **cantus/chant line**, not like a soloist expecting the preset to infer a historical chant or compose a Notre-Dame duplum. In historical organum purum, the actual chant belongs in the sustained tenor while a far more elaborate upper melody unfolds over it. [Cambridge repository, *The Creation of Parisian Organum Purum*](https://www.repository.cam.ac.uk/items/492237d0-d1c9-4594-bfc3-d8e65be5bc09) A static interval stack around every live note therefore reverses or compresses important historical voice roles; it is acceptable only when presented as an inspired arrangement.

**Poor source gestures:**

- arbitrary chromatic noodling;
- repeated rapid scales;
- blues bends or riff vocabulary;
- strummed chords;
- dense keyboard harmony;
- continuous playing without note-offs or phrase rests.

---

## 2. Register

### General recommendation

Use a centered, voice-like register:

- **Preferred sounding range:** approximately C3–C5.
- **Optional upper extension:** to G5 for a single phrase peak.
- Avoid spending an entire phrase below C3 if lower generated intervals are enabled.
- Avoid sustained playing above G5, where added upper intervals can become piercing and cease to suggest weight beneath a line.

This is an **interaction-safe range**, not a claim about a single historical absolute pitch or vocal disposition. Historical source clefs and modern pitch realization do not establish one mandatory concert register.

### Register behavior across a phrase

1. Start in the lower or middle part of the range.
2. Rise no more than a fourth or fifth over several gestures.
3. Reserve the highest note for one density or intensity peak.
4. Return toward the opening register before the final rest.

Wide registral spacing may help the open intervals read as separate voices. Close, low stacks are more likely to sound muddy.

---

## 3. Tempo

### Sustained-organum interaction

- **Recommended UI range:** 48–66 BPM.
- Treat the pulse as orientation, not as a stream of required attacks.
- Typical attack rate: one new source note every 1–4 beats.
- Phrase-end notes may last 4–8 beats.

These BPM values are product guidance, not a reconstructed medieval tempo. The historical rhythm of sustained-tenor organum remains disputed; modern performers should not be told that a fixed BPM authenticates it. Turner warns that modern notation can make organum purum falsely rigid and advocates flexibility informed by original ligature groupings. [Turner](https://www.musicandpractice.org/volume-1/notre-dame-organum-duplum-modern-edition/) Yudkin surveys the long-running scholarly disagreement and argues from Garlandia, Anonymous IV, and the St Emmeram Anonymous for non-modal, context-dependent rhythm in organum purum. [Yudkin](https://www.examenapium.it/cs/biblio/Yudkin1983.pdf)

### Discant/polyphonic interaction

A rhythmically coordinated discant mode would require:

- transport;
- an explicit modal-rhythmic pattern;
- independently moving generated voices;
- coordinated phrase/rest boundaries.

A stack that merely attacks and releases with the player is **not discant**. MIT’s early-music outline summarizes the practical distinction as Léonin-associated organum in unknown/free rhythm versus discant in fixed rhythm, and describes Pérotin’s larger textures as fixed/modal rhythmic writing. [MIT OpenCourseWare, *Polyphony at Notre Dame and in the 13th Century*](https://ocw.mit.edu/courses/21m-220-early-music-fall-2010/25c3ebc986e9d14de35660f9606afa9d_MIT21M_220F10_lec07.pdf)

---

## 4. Note duration

Recommended source-note lengths:

- **Opening/foundation note:** 2–4 seconds.
- **Interior connecting notes:** 0.6–1.5 seconds.
- **Phrase arrival:** 3–6 seconds.
- **Silence after a phrase:** 1–3 seconds, longer after the final phrase.

The central expressive contrast should be **long foundation → gently moving continuation → long arrival → silence**.

Do not hold every note for exactly the same duration. Equal mechanical durations erase the distinction between structural tones and motion. Conversely, do not use a rapid ornamental stream: an explicit interval stack will multiply every attack and produce a note storm rather than the elaborate but controlled line of organum.

---

## 5. Articulation

- Use clean, rounded attacks.
- Connect adjacent notes smoothly, but release the previous note decisively before the next harmony accumulates.
- Allow a small breath before a new phrase.
- Avoid heavy accents on every note.
- Avoid palm-muted machine-gun articulation.
- Avoid uncontrolled overlap from sustain pedal, sympathetic strings, or long MIDI releases.

“Legato” here means a connected melodic intention, not indefinite overlap. Generated voices must follow note-off reliably.

For phrase arrivals, a slightly firmer attack followed by a stable sustain is preferable to a percussive accent.

---

## 6. Dynamics

- Begin **mp**.
- Grow only to **mf** at the phrase or section peak.
- Recede to **p–mp** before the closing silence.
- Keep velocity changes gradual and phrase-shaped.
- Avoid alternating extreme low/high velocities note by note.

If the engine does not map velocity musically, dynamics remain performer/timbre guidance rather than a guaranteed arrangement behavior. The UI must not promise historically reconstructed vocal dynamics.

---

## 7. Density and silence

### Input density

- One source note at a time.
- Two to five notes per small gesture.
- Approximately three or four gestures in 30–60 seconds.
- At least one full silence between gestures.
- Prefer fewer than roughly 15 source-note attacks in a 45-second exercise.

### Generated density

The useful ceiling is:

- source line plus **one or two** generated voices for the default;
- at most three generated voices only if registers are deliberately separated and voice release is reliable.

Historical Notre-Dame sources include two-, three-, and four-part repertory; the Florence manuscript records a four-voice *Viderunt omnes*. [DIAMM, Florence Pluteus 29.1](https://www.diamm.ac.uk/sources/924/) That does not justify turning every live note into a dense four-note block. Pérotin’s four-part texture consists of composed, rhythmically related parts, not simply a universal fourth/fifth/octave chord.

### Required silence behavior

When the player releases and waits:

- all generated notes should release;
- no new notes should appear;
- no captured phrase should replay;
- no autonomous “medieval” ornament should be synthesized.

Silence is part of the performer contract, not unused space.

---

## 8. Transport dependency and rhythmic precision

### Honest boundary for the proposed preset

**Default Cloister Organum should not require transport.**

The bounded implementation is a sustained, event-following open-interval halo. It should respond to note-on, note-off, register, and possibly velocity. The player may use a slow metronome for consistency, but the musical result must not depend upon beat phase.

Required precision:

- clean onset;
- correct note-off;
- no simultaneous accidental note;
- approximate phrase-scale pacing rather than grid-perfect subdivision.

### What transport would imply

If the UI offers a future **Discant** or **Pérotin-derived measured polyphony** variation, transport must be required and visible. Such a mode would need modal patterns and independent rhythmic parts; it cannot be represented honestly by synchronous interval stacks.

Do not silently quantize the sustained mode and call that historical modal rhythm. Yudkin documents that medieval theorists differentiated the freer rhythm of organum purum from the regulated measure of discant. [Yudkin](https://www.examenapium.it/cs/biblio/Yudkin1983.pdf)

---

## 9. Phrase and section development

### Within one phrase

Use this performer-controlled state sequence:

`Foundation → measured movement → held arrival → full release`

1. **Foundation:** hold the first note long enough to hear the generated spacing.
2. **Movement:** add two to four mostly stepwise notes; shorten these slightly.
3. **Arrival:** settle on a stable pitch and hold it longer.
4. **Release:** stop and listen to the complete decay.

Turner emphasizes that the tenor should not be treated as an inert drone: it should articulate flexibly and respond to the intervallic activity of the other voice. [Turner](https://www.musicandpractice.org/volume-1/notre-dame-organum-duplum-modern-edition/) For Contrapunk, that means the player should adjust duration and continuation based on what the generated intervals actually sound like.

### Across a 30–60 second section

Use three phrases:

1. **Phrase A — establish:** three notes, narrow range, quiet.
2. **Phrase B — expand:** four or five notes, rise modestly, slightly stronger.
3. **Phrase C — return:** shorten the motion, return near the opening pitch, hold and release.

No automatic accumulation is necessary. The player creates development through register, duration, velocity, repetition, and silence.

### Optional contrast without false discant

A middle phrase may use slightly more regular timing—such as four attacks approximately two beats apart—but it remains an interaction contrast, not historical discant unless the generated parts become independently and measurably coordinated.

---

## 10. Listening and responding cues

After every new long note:

1. Wait until the open interval stack is clearly audible.
2. Check whether its lowest generated note remains clear rather than booming.
3. Listen for beating, register crowding, or a harsh fourth against the current voicing.
4. Continue only after the sonority stabilizes.
5. If the sound becomes thick, release completely instead of adding another note.

At a phrase end:

- wait through the decay;
- confirm no stale harmony remains;
- begin the next phrase only from silence.

The player should treat a generated clash as feedback. Resolve it by moving one scale step, returning to the preceding source pitch, changing register, or releasing—not by adding more notes.

The historical analogy is limited but useful: Turner argues that sustained tenor and upper voice must remain aware of their intervallic play rather than behaving like an unrelated drone and ornament. [Turner](https://www.musicandpractice.org/volume-1/notre-dame-organum-duplum-modern-edition/)

---

## 11. Guitar-specific constraints

The clean-monophonic contract must be literal:

- Pick one note at a time.
- Mute every unused string with both hands as needed.
- Avoid letting an open string ring across a pitch change.
- Release a fretted note cleanly before changing position.
- Prefer picked or firmly articulated fingerstyle attacks over ambiguous swells.
- Use minimal distortion, chorus, delay, and reverb before pitch-to-MIDI conversion.
- Avoid bends, wide vibrato, slides across semitones, hammer-on flurries, and pull-off ornaments during acceptance testing.
- If pitch bend is enabled, use it only after stable triggering has been verified.

Sonuus’s monophonic G2M guidance states that clean, accurate playing and damping of non-sounding strings improve conversion; accidental notes and resonating open strings can become unintended MIDI notes. [Sonuus G2M](https://sonuus.com/products_g2m.html)

### Practical guitar register

A useful starting region is:

- D3–A4 for ordinary guitar-to-MIDI input;
- occasional extension to E5;
- avoid the lowest string register when downward generated intervals are active.

### Guitar failure signatures

- **Open-string residue:** an old generated stack continues beneath the new one.
- **Fret noise or accidental touch:** a brief false note produces a full harmony burst.
- **Slow bend:** chromatic retriggering or unstable mapped pitches.
- **Fast legato technique:** note detection and overlapping releases become ambiguous.
- **Strum:** violates the monophonic contract and may create a note storm.

---

## 12. Keyboard-specific opportunities and chord limit

### Opportunities

A keyboardist can control:

- exact note duration;
- reliable note-off;
- subtle velocity arcs;
- deliberate register migration;
- precise silence;
- optional sustain only at a final cadence, if generated note lifecycle has been tested.

Keyboard makes it easy to compare two approaches:

- free, breath-shaped held notes for the sustained-organum approximation;
- evenly spaced attacks as a clearly labeled modern contrast.

### Chord-density limit

**Operational limit: one depressed pitch at a time.**

The catalog does not explicitly permit chords for Cloister Organum, and the catalog-wide default is clean monophonic notes. A dyad is therefore not an endorsed gesture even if the MIDI adapter happens to accept it.

Sustain pedal should remain off during normal performance. If used at all:

- only on the last source note;
- no new source note while the pedal remains down;
- release pedal and verify complete silence before continuing.

A chord-capable variation would require explicit input filtering, voice limits, duplicate-note handling, and separate testing. It is outside this preset’s current promise.

---

## 13. Failure gestures

| Failure gesture | Likely result | Why it contradicts the preset |
|---|---|---|
| Fast scalar run | Multiplied attack stream | Static stacks cannot reproduce a shaped duplum melisma |
| Repeated-note tremolo | Dense retriggering | Replaces sustained architecture with a pulse effect |
| Strummed guitar chord | Unbounded note burst | Violates clean-monophonic input |
| Keyboard triad plus sustain | Six or more sounding pitches | Obscures voice roles and muddies low intervals |
| Continuous legato overlap | Stale harmony accumulation | Removes phrase articulation and silence |
| Wide chromatic improvisation | Generic chromatic interval planing | Preset does not infer chant, mode, or cadence |
| Blues bend or pitch scoop | Tracking instability or stylistic clash | Not mapped to this bounded source vocabulary |
| Low-register rapid motion | Mud and beating | Fourth/fifth/octave stacks crowd the bass |
| Exact equal-duration notes throughout | Mechanical block progression | Erases structural versus connective tones |
| Expecting autonomous upper melody | Static result mistaken for historical organum | Explicit stacks do not compose organum purum |
| Calling synchronous stacks “discant” | False historical claim | Discant requires coordinated active rhythm in multiple parts |

---

## 14. Plain-language **Play it like** prompt

### Short UI prompt

> **Play one slow, chant-like note at a time. Hold important notes, move mostly by step, and leave a full breath of silence between phrases.**

### Expanded guidance

> Begin with one comfortable middle-register note and let the added open intervals settle. Continue with two to five mostly neighboring notes, making connecting notes shorter and the destination longer. Stop completely after each phrase and listen for every generated voice to release. Start quietly, rise slightly in register and intensity once, then return toward the opening pitch. Do not play chords, rapid runs, continuous tremolo, or overlapping sustain. The preset adds a restrained fourth/fifth/octave halo; it does not turn arbitrary notes into Gregorian chant or reconstruct Léonin or Pérotin.

### Optional inline warning

> **Best with intentional modal or chant-like material. Arbitrary chromatic input remains arbitrary chromatic input.**

---

## 15. 45-second user acceptance exercise

Set a metronome to **56 BPM** for pacing only. Transport should be optional.

### 0–10 seconds — establish

1. Play one middle-register note at **mp**.
2. Hold for approximately four beats.
3. Release and wait two beats.
4. Confirm that all generated voices stop.

**Pass condition:** a clear open sonority appears without extra attacks or stuck notes.

### 10–24 seconds — first phrase

1. Play four single notes: `1–2–3–2` in any comfortable diatonic/modal collection.
2. Hold the first and last for about three beats each.
3. Hold the middle notes for about one beat each.
4. Release and leave approximately three seconds of silence.

**Pass condition:** each source note receives a bounded response; no harmonies accumulate across released notes.

### 24–36 seconds — expansion

1. Play `1–2–4–3–2` one register position higher or with a modest rise in contour.
2. Increase only to **mf**.
3. Keep all notes monophonic.
4. Hold degree `4` slightly longer as the phrase peak.
5. Stop completely.

**Pass condition:** increased activity remains intelligible and does not become a note storm.

### 36–45 seconds — return and cadence

1. Return near the opening pitch.
2. Play `2–1`.
3. Hold `1` for four to six seconds.
4. Release and listen through the decay.

**Pass condition:** the final sonority clears fully; silence produces no replay or autonomous continuation.

### Acceptance failures

Fail the preset if any of the following occurs:

- one clean source note produces an unbounded number of voices;
- a released note remains active into the next phrase;
- transport absence changes pitch behavior or prevents the sustained mode from working;
- simultaneous accidental input causes an uncontrolled burst rather than bounded rejection/voice limiting;
- UI text implies that the exercise reproduces historical organum or supplies chant style automatically.

---

## 16. Sustained organum versus discant/polyphony

| Dimension | Sustained-organum-derived preset | Historical discant / larger Pérotin-derived texture |
|---|---|---|
| Source role | Live line stands in for a slow cantus | Chant tenor participates in measured organization |
| Generated response | Synchronous bounded open intervals | Independently moving, rhythmically coordinated parts |
| Rhythm | Flexible, breath-shaped | Modal/measured patterns |
| Transport | Not required | Required |
| Player precision | Clean onset/release and phrase pacing | Stable pulse, subdivision, pattern alignment |
| Texture | One line plus limited halo | Two to four functionally distinct voices |
| Honest claim | Atmospheric open-interval arrangement | Requires a separate rhythmic/polyphonic engine |
| Current catalog fit | Plausible if wording is narrowed | Not met by explicit static interval stacks alone |

The Florence manuscript evidence confirms that the repertory includes four-voice *Viderunt omnes*, but the number “four” alone does not define Pérotin’s technique. [DIAMM](https://www.diamm.ac.uk/sources/924/) The Cambridge research describes organum purum as sustained chant beneath long, elaborately developed upper-voice melismas, including repeated notes, large intervals, beginnings, endings, and melodic development strategies. [Cambridge repository](https://www.repository.cam.ac.uk/items/492237d0-d1c9-4594-bfc3-d8e65be5bc09) A chord stack has none of that temporal independence.

---

## 17. Catalog promise evaluation

### Current promise

> “Open fourths, fifths, and octaves around a chant line. Play: slow held stepwise notes with clear phrase rests. Needs: explicit interval stacks.”

### What is sound

- “Slow held stepwise notes with clear phrase rests” is useful performer guidance.
- Fourths, fifths, and octaves provide a recognizable open sonority.
- “Needs: explicit interval stacks” identifies a bounded implementation rather than pretending the existing engine already performs the technique.
- Composer names are references rather than a direct imitation claim.

### What overpromises

1. **“Chant line”** may imply that the preset recognizes, retrieves, or generates chant. It does not; the player must provide chant-like contour and pacing.
2. **“Around”** obscures voice roles. Notre-Dame organum is not generically symmetric stacking above and below every note.
3. Léonin/Pérotin together can imply both organum purum and measured multi-voice discant. Static interval stacks reproduce neither in full.
4. Pérotin’s three- and four-voice writing cannot be represented by adding more simultaneous perfect intervals.
5. The catalog does not state that arbitrary chromatic input will remain stylistically arbitrary.

### Catalog verdict

**Conditionally honest only after narrowing.** The performer contract works for an atmospheric, sustained-organum-derived preset. It is misleading if sold as a simulation of Léonin/Pérotin composition or if its static stacks are described as discant.

---

## 18. Proposed honest UI wording

### Title

**Cloister Organum**

### Reference line

> **Open-interval arrangement inspired by selected Notre-Dame organum repertory associated with Léonin and Pérotin.**

### Result

> **Adds a restrained halo of fourths, fifths, and octaves to a slow monophonic line.**

### Play

> **Play one slow, chant-like note at a time. Hold arrivals, move mostly by step, and leave complete rests between phrases.**

### Approximation disclosure

> **This preset does not generate chant or reconstruct historical organum. It does not reproduce the free upper-voice melismas of organum purum or the measured independent parts of discant.**

### Input badge

> **Monophonic input · no chords**

### Transport badge

> **Transport optional — pacing only**

### Availability/requirements

> **Requires bounded explicit interval stacks and reliable note-off handling.**

### Avoid

> **Avoid chords, rapid runs, overlapping sustain, bends, and continuous chromatic improvisation.**

---

## 19. Exact operational boundaries

The preset should be considered operational only if all of these boundaries are true:

1. Accept exactly one intended source pitch at a time.
2. Generate a fixed, documented maximum number of additional notes.
3. Use only configured fourth, fifth, and octave relationships; no hidden motif or chant inference.
4. Release every generated note from the source note’s note-off.
5. Produce no autonomous notes during silence.
6. Require no transport for the default sustained mode.
7. Make no claim to rhythmic modes, discant, clausula, or Pérotin-style independent polyphony.
8. Do not accept chords as a supported performer gesture.
9. Do not claim historical tuning, vocal scoring, acoustics, articulation, or tempo.
10. Describe player-supplied material as **chant-like**, not as chant.
11. Bound low-register output to avoid muddy downward stacks.
12. Treat velocity response as optional unless it is demonstrably mapped.
13. Surface the approximation disclosure in preset details, not only external documentation.
14. If a future measured variation is added, label it separately and require transport.

---

## 20. Confidence and gaps

| Claim | Confidence | Basis / limitation |
|---|---|---|
| Organum purum prolongs chant beneath an elaborate upper voice | High | Cambridge dissertation abstract and performance-practice literature |
| Discant has rhythmically active, coordinated parts | High | Turner; medieval-theory discussion in Yudkin |
| Organum purum rhythm should not be reduced confidently to a fixed modal grid | High | Yudkin’s treatise-based argument; Turner’s performer account |
| Pérotin-associated repertory includes three- and four-voice measured writing | High | MIT overview and manuscript catalog evidence |
| A fourth/fifth/octave stack is not an adequate model of Pérotin polyphony | High | Direct mismatch between manuscript texture/voice behavior and proposed engine behavior |
| 48–66 BPM is historically correct | Low / not claimed | Product pacing recommendation only |
| C3–C5 reflects original absolute performing pitch | Low / not claimed | Interaction-safe modern register only |
| Exact medieval articulation and dynamics | Low | Not recoverable confidently from the consulted sources |
| Guitar tracking guidance | High for monophonic conversion hygiene | Manufacturer documentation; device-specific performance still varies |
| The preset can evoke an open medieval-associated atmosphere with disciplined input | Medium | Practical design inference, not a historical-performance claim |

### Unresolved gaps

- No single historically authoritative absolute tempo can be specified.
- Organum purum rhythm and realization remain subjects of scholarly and performer disagreement.
- Exact intended ensemble size, doubling, pitch standard, vocal timbre, and acoustical realization vary by source and modern interpretation.
- The engine’s actual interval-direction, range-limiting, voice-stealing, and note-off behavior must be validated before final performer copy is approved.
- A true discant variation needs separate research and implementation acceptance; it should not be inferred from this report.

---

## Sources

### Kept

- [Penelope Turner, “Notre Dame Organum Duplum: What Does a Performer Need?”](https://www.musicandpractice.org/volume-1/notre-dame-organum-duplum-modern-edition/) — Direct performer discussion of free sustained-tenor passages, discant, notation, listening between voices, and practical uncertainty.
- [Jeremy Yudkin, “The Rhythm of Organum Purum”](https://www.examenapium.it/cs/biblio/Yudkin1983.pdf) — Scholarly analysis of Garlandia, Anonymous IV, and related evidence distinguishing organum purum from modal discant.
- [Cambridge repository, “The Creation of Parisian Organum Purum”](https://www.repository.cam.ac.uk/items/492237d0-d1c9-4594-bfc3-d8e65be5bc09) — Recent dissertation based on detailed comparison of surviving organa dupla; supports sustained chant, elaborate upper lines, and melodic development.
- [MIT OpenCourseWare, “Polyphony at Notre Dame and in the 13th Century”](https://ocw.mit.edu/courses/21m-220-early-music-fall-2010/25c3ebc986e9d14de35660f9606afa9d_MIT21M_220F10_lec07.pdf) — Institutional teaching source summarizing Léonin/Pérotin, organum versus discant, modal rhythm, and larger voice counts.
- [DIAMM, Florence MS Pluteus 29.1](https://www.diamm.ac.uk/sources/924/) — Institutional manuscript catalog confirming source repertory and four-voice *Viderunt omnes*.
- [Sonuus G2M](https://sonuus.com/products_g2m.html) — First-party monophonic guitar-to-MIDI guidance on clean attacks, string damping, false triggers, pitch bend, and tracking.

### Dropped or used only as discovery leads

- Britannica organum and Notre-Dame-school summaries — useful orientation, but less precise than the retained treatise-based and performance-practice sources.
- IMSLP transcriptions — useful for score access, but not sufficient authority for disputed rhythm or performer interaction claims.
- Academia.edu uploads and informal score videos — provenance or editorial authority was less clear than the Cambridge, DIAMM, and published scholarly sources.
- Blogs, general listening guides, Scribd notes, essay sites, and AI-generated encyclopedia pages — excluded because they were derivative, insufficiently sourced, or not authoritative enough for operational claims.