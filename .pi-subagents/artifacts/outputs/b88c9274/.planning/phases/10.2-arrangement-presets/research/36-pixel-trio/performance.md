# Pixel Trio — Independent Performer / HCI Research Report

**Phase:** 10.2 composer-informed arrangement presets  
**Preset:** 36, “Pixel Trio”  
**Research role:** Performer and interaction  
**Reference boundary:** bounded early-Famicom/NES three-pitched-voice economy, with Koji Kondo’s *Super Mario Bros.* (1985) and Manami Matsumae’s *Mega Man* (1987) as selected points of reference—not either composer’s complete output or identity  
**Decision:** suitable as an honest current-engine performance study if the UI describes three simultaneous mapped voices, not a generated console arrangement

## Summary

Pixel Trio should ask the performer for a short, clean, monophonic hook with a stable pulse, audible gaps, and restrained variation. The current shared `HarmonyEngine` can turn each accepted input note into **three total voices**—the played note plus two pitch-derived notes—but it cannot compose an independent bass part or counterline, assign authentic chip timbres, create percussion or loops, or react to game state.

The useful historical intersection is economy: Kondo recalls making *Super Mario Bros.* distinctive and playable with “three sounds,” while Matsumae says three-channel writing was comfortable because of her intensive three- and four-part Bach study; both also describe writing in response to a game’s movement, image, or stage rather than applying a generic “8-bit” formula. [Nintendo](https://www.nintendo.com/en-gb/Iwata-Asks/Iwata-Asks-Super-Mario-All-Stars/Vol-1-Super-Mario-History-Soundtrack-CD/4-Music-Commentary-by-Koji-Kondo-1-/4-Music-Commentary-by-Koji-Kondo-1--219986.html) [Brave Wave](https://bravewave.net/interviews/a-conversation-with-manami-matsumae/)

## 1. Scope and evidence boundary

This report translates a narrow historical constraint into a live-input contract. It does **not** infer a universal Kondo style, a universal Matsumae style, or a single “NES style.”

- **Kondo scope:** the original *Super Mario Bros.* Famicom score, especially his retrospective account of the Ground Theme. Kondo says he could generate only three sounds at once, sought distinctiveness within that restriction, differentiated music by location, and discarded an easy-going first version because it did not match Mario’s running and jumping. [Nintendo: music commentary](https://www.nintendo.com/en-gb/Iwata-Asks/Iwata-Asks-Super-Mario-All-Stars/Vol-1-Super-Mario-History-Soundtrack-CD/4-Music-Commentary-by-Koji-Kondo-1-/4-Music-Commentary-by-Koji-Kondo-1--219986.html) [Nintendo: prototype and rhythm](https://www.nintendo.com/en-gb/Iwata-Asks/Super-Mario-Bros-25th-Anniversary/Vol-5-Original-Super-Mario-Developers/5-To-Save-Memory/5-To-Save-Memory-212974.html)
- **Matsumae scope:** the original *Mega Man* score and sound work. Matsumae describes three-channel composition as compatible with her Bach training, but says rhythm-based/drum writing required separate listening study. She aimed for a lively, memorable, anime-like melody, knew the stage layouts, balanced sound effects against music, and shortened effects when they threatened to cut into the melody. [Brave Wave](https://bravewave.net/interviews/a-conversation-with-manami-matsumae/)
- **Shared early-console context:** the NES actually has five monophonic audio channels—two pulse, one triangle, noise, and DMC—not literally three channels total. The useful “trio” abstraction concerns the three pitched channels commonly available for musical lines; the noise and DMC resources are excluded from this preset. Peer-reviewed analysis notes that pulse channels often share melodic work while the lower triangle channel often carries bass, and that sound effects can force musical material to drop out. [Cook, *Music Theory Online* §§3.2–3.8](https://www.mtosmt.org/issues/mto.23.29.3/mto.23.29.3.cook.php)

**Documented intersection:** a small number of monophonic parts makes every onset, rest, register, and repeated cell consequential. **Meaningful difference retained:** Kondo’s cited account emphasizes matching movement and location; Matsumae’s emphasizes contrapuntal training, memorable heroic melody, stage awareness, and negotiating music with sound effects. Pixel Trio borrows only the bounded economy and performer discipline common to those accounts.

## 2. Current system model and honest role labels

For this report, the current shared `HarmonyEngine` contract is:

1. one monophonic source Note On enters;
2. the source pitch remains one of the outputs;
3. two harmony pitches are derived from that source under the selected scale/harmony mapping;
4. at most three total voices sound for that accepted input event;
5. the corresponding source Note Off releases the tracked three-note result.

“Melody / bass / counterline” may therefore be used only as **register and listening labels**. The two generated notes are event-coupled shadows: they enter and leave with the player and are pitch-derived from that note. They are not autonomous musicians. This differs materially from early-console channel writing, where individual channels could carry distinct rhythms and could loop or recall material independently. [Cook §§5.1–5.3](https://www.mtosmt.org/issues/mto.23.29.3/mto.23.29.3.cook.php)

The player creates the only actual line, including its rhythm, rests, repetitions, phrase boundaries, and variation. The engine supplies a bounded three-note vertical mapping around each event. That is enough for a playable “three-voice economy” exercise; it is not enough for an arranged console cue.

## 3. Best source gesture

**Best gesture: a compact, single-note hook.** Use a two-bar idea of **four to eight attacks**, a recognizable rhythm, mostly steps and small skips, and at least one quarter-note rest. Repeat it once before changing one detail.

Why this source works:

- Kondo’s account ties musical success to the rhythm of movement, not merely to scenery or timbre. [Nintendo](https://www.nintendo.com/en-gb/Iwata-Asks/Super-Mario-Bros-25th-Anniversary/Vol-5-Original-Super-Mario-Developers/5-To-Save-Memory/5-To-Save-Memory-212974.html)
- Matsumae explicitly describes aiming for lively, easy-to-remember melody and later characterizes an old-game melody as singable and capable of taking lyrics. [Brave Wave](https://bravewave.net/interviews/a-conversation-with-manami-matsumae/)
- Early-console form economized through microloops, section-level loops, recalled material, and variation rather than an endless stream of unrelated notes. [Cook §§3.8–3.12](https://www.mtosmt.org/issues/mto.23.29.3/mto.23.29.3.cook.php)

Do not feed a sustained drone, a chord progression, an improvised note cloud, or a long legato solo and expect the preset to discover a hook.

## 4. Suggested performance envelope

| Dimension | Contract | Reason |
|---|---|---|
| **Input** | Exactly one note at a time | The product claim is clean monophonic input; overlapping notes obscure ownership and make a three-total-voice promise harder to hear. |
| **Register** | Guitar: E3–B4 preferred. Keyboard: C4–C6 preferred. | Keeps the supplied hook clear while leaving useful space for a low-role mapping and avoiding the guitar detector’s most difficult low fundamentals. If generated notes clip/cross, move the hook one octave toward the middle. |
| **Tempo** | 100–150 BPM; acceptance at 120 BPM | Supports a forward-moving hook without requiring tracker-hostile tremolo or a pattern engine. This is a product envelope, not a claim about all referenced tracks. |
| **Note length** | Staccato-to-portato, about 50–75% of the notated value; deliberate release before the next attack | Produces audible separation and exposes whether Note Off ownership is correct. Avoid microscopic taps. |
| **Articulation** | One clean attack per intended note; light palm mute or dry key release is welcome | Three simultaneous pitches already increase density. Clear attacks make the mapping intelligible. |
| **Velocity** | MIDI keyboard: mostly 72–96, hook accent 100–108, phrase ending 64–80. Guitar: medium, even pick attack; do not rely on precise velocity transfer. | MIDI velocity normally encodes relative attack/loudness, but actual audible response depends on the receiving instrument. [MIDI Association](https://midi.org/about-midi-part-3midi-messages) |
| **Density** | 4–8 attacks per two bars; normally no faster than steady eighth notes | Leaves room to hear all three mapped pitches and prevents a vertically multiplied note storm. |
| **Rests** | At least one beat per two-bar hook and one full bar after 6–8 bars | Silence is structural, not failure. NES practice used channel silence and dropout as genuine resources. [Cook §3.3](https://www.mtosmt.org/issues/mto.23.29.3/mto.23.29.3.cook.php) |
| **Repetition** | State A twice exactly before A′ | Lets the listener learn the hook and inspect deterministic mapping. |
| **Variation** | Change one pitch, one ending, or one register—not all at once | Preserves identity while avoiding mechanical sameness; the player, not the preset, performs the variation. |

These bounds are intentionally conservative. They are an interaction design for reliable three-voice audition, not a musicological assertion that Kondo or Matsumae always used these registers, tempi, velocities, or phrase lengths.

## 5. Transport dependency and rhythmic precision

**Transport is optional for the current baseline.** The mapping should respond to Note On/Note Off without needing a running clock because it creates no beat-scheduled bass, percussion, pattern, or loop. If a metronome is available, use it as performer support only.

Rhythmic precision still matters because the player supplies the entire rhythmic identity. Aim for attacks within a plainly audible eighth-note grid, but do not imply sample-accurate quantization. The preset must not advertise that it corrects timing. MIDI defines separate Note On and Note Off events and provides real-time clock messages for synchronized equipment, but receiving note messages alone does not create a sequenced pattern. [MIDI Association](https://midi.org/about-midi-part-3midi-messages)

**Stopped transport expectation:** notes still map and release normally.  
**Running transport expectation:** identical pitch and ownership behavior; transport must not invent extra notes.

## 6. Phrase and section development by the player

Use the following playable state path:

`Present A → repeat A exactly → make one A′ change → full-bar listen/rest → return A → cadence and clear`

1. **Present A:** establish a two-bar hook at medium velocity. Do not fill every subdivision.
2. **Repeat A:** preserve pitches, rhythm, register, and articulation. Listen for the two derived voices to reproduce the same relationships.
3. **Make A′:** alter one pitch near the end, or displace only the last note by an octave if the instrument/input range remains reliable.
4. **Listen:** release fully and wait one bar. Silence verifies cleanup and gives the generated sonority room to register.
5. **Return:** play A again; the listener should recognize the return because the performer preserves it.
6. **Cadence:** lengthen the last source note modestly, release it, and stop. Do not add a flourish after release.

Across a longer section, increase intensity with **one dimension at a time**: slightly firmer attacks, one extra repeated note, or a one-octave register lift. Do not increase speed, register, velocity, and density together; the current engine has no scene model that can interpret such escalation.

## 7. Listening and response instructions

After the first source note, confirm three things before continuing at full pace:

1. **Count:** the input plus only two generated pitches are present—three total, not three added voices.
2. **Separation:** a low-role note and an upper/inner-role note are perceptible around the source. If all three bunch together or cross confusingly, move the hook toward the middle register rather than playing harder.
3. **Release:** when the source note ends, all three voices end. If any voice hangs, stop and invoke Panic; do not continue stacking notes.

During the full-bar rest, listen for literal silence. The system has not “taken a turn”; it has only stopped because the player stopped. Continue only after cleanup is clear.

## 8. Guitar-specific constraints under the clean-monophonic claim

Use a standard-tuned six-string guitar with a clean, isolated signal and play one fretted note at a time.

- Prefer alternate or consistent down-picking with one decisive attack per event.
- Damp unused strings with both hands; sympathetic open-string ringing can look like additional pitch content.
- Release one note before attacking the next. Avoid ringing open strings beneath fretted notes.
- Keep bends, wide vibrato, slides, hammer-ons, pull-offs, harmonics, pick scrapes, and feedback out of the acceptance path. Peer-reviewed guitar-transcription research shows that bends and vibrato can be split into false note events and that slides/hammer-ons/pull-offs complicate onset and pitch segmentation. [Su et al., TENT §§1.1–1.3](https://transactions.ismir.net/articles/10.5334/tismir.23)
- Use medium, even pick force. Guitar audio input should not promise the exact 7-bit velocity control available from a MIDI keyboard.
- Avoid very low, soft attacks for this preset. If low-role output is desired, let the mapping provide it instead of feeding a low bass riff.

A graceful guitar failure gesture is **mute and wait**, not frantic correction: damp all strings, wait until the UI shows no active notes, then restart A. If notes remain active, use Panic.

## 9. MIDI-keyboard opportunities and chord-density limits

A keyboard gives more repeatable onset, release, register, and velocity control than audio pitch tracking.

- Play with one finger linearly; lift before the next key.
- Use velocity to mark only the hook’s first attack or one phrase goal.
- An octave displacement is a useful A′ variation because it is deterministic and clean on MIDI.
- Sustain pedal must remain **off**. Pedal overlap contradicts the sparse three-total-voice contract.
- Pitch bend, aftertouch, arpeggiator, latch, keyboard split layers, and built-in chord mode must remain off for acceptance.
- **Chord-density limit: one input note.** A dyad is not “just a little richer”: if accepted independently it can request up to six simultaneous mapped notes, destroying both the three-total-voice limit and source clarity.

MIDI’s separate Note On/Note Off events make deliberate releases part of the musical and lifecycle contract, not mere cleanup metadata. [MIDI Association](https://midi.org/about-midi-part-3midi-messages)

## 10. Failure gestures and recovery

| Failure gesture | Likely result | Recovery |
|---|---|---|
| Chords, dyads, sustain pedal, or overlapping legato | More than one source owner; vertical multiplication and mud | Release all keys/pedal, wait for zero active notes, restart monophonically. |
| Tremolo sixteenths or repeated attacks without clean releases | Note storm; blurred hook; harder ownership audit | Halve the attack rate and shorten to a two-bar cell. |
| Guitar bends, wide vibrato, slides, hammer-ons, pull-offs | False retriggers, unstable pitch, or merged/split events | Damp, wait, and replay with clean picked fretted notes. |
| Very low hook plus low-role mapping | Register collision, crossing, or output-range fallback | Transpose source up one octave. |
| Continuous unrelated improvisation | No memorable identity; preset sounds like generic harmonization | Stop for one bar; state A and repeat it exactly. |
| No rests | Every source attack becomes a three-note sonority; listener cannot inspect release | Insert at least one beat per two bars and one full-bar checkpoint. |
| Expecting a bass groove, drums, or loop after stopping | Silence mistaken for malfunction | Treat silence as correct; those layers are outside this preset. |
| Any hanging voice after source release | Lifecycle failure, not style | Stop input and invoke Panic before changing preset or transport. |

## 11. Lifecycle expectations

A usable preset must make these expectations visible or testable:

- Every accepted source Note On has one ownership group containing no more than three total output notes.
- Its matching source Note Off releases exactly that tracked group, even if the harmony choice was stateful.
- Rests emit no autonomous material.
- Stopping transport does not strand a note and is not required to release one.
- Applying/replacing the preset, disabling routing, or invoking Panic clears active and pending notes before new configuration takes effect.
- Changing a harmony-affecting setting while a note is held must not orphan the old generated notes; the safe performer action is nevertheless release first, then change.
- Guitar confidence failure should yield silence or a recoverable missed note, never an invented bass pattern.
- The final acceptance state is zero active source notes, zero generated notes, and zero pending events.

## 12. What the player creates; what the system cannot manufacture

### The player creates

- the hook’s pitches and contour;
- every attack, duration, accent, rest, repeat, variation, phrase return, and cadence;
- the sense of forward motion;
- the decision to stay sparse enough that three voices remain intelligible;
- the musical context that makes the mapping feel playful, heroic, tense, or calm.

### The current system can provide

- a bounded **three-total-voice** response to each accepted monophonic note;
- scale/harmony-derived pitch relationships;
- register placement sufficient to suggest source, low, and upper/inner roles;
- deterministic Note On/Note Off ownership when lifecycle behavior is correct.

### The current system cannot honestly claim to provide

- **Independent bass:** no separate bass rhythm, pedal, walking motion, ostinato, rests, or cadence decisions.
- **Independent counterline:** no autonomous contour, imitation, contrary rhythm, delayed entrance, or channel-specific loop.
- **Chip timbre:** the arrangement preset does not select pulse duty cycle, triangle-wave bass, noise, DMC sample playback, envelope behavior, or authentic console synthesis. The historical hardware’s channel timbres are specific affordances, not consequences of using three MIDI pitches. [Cook §§3.2–3.3](https://www.mtosmt.org/issues/mto.23.29.3/mto.23.29.3.cook.php)
- **Patterns and percussion:** no arpeggio scheduler, noise-channel drum part, sampled percussion, accent grid, or sound-effect channel stealing. Kondo’s own account distinguishes three sounds from later delta-modulation percussion possibilities. [Nintendo](https://www.nintendo.com/en-gb/Iwata-Asks/Iwata-Asks-Super-Mario-All-Stars/Vol-1-Super-Mario-History-Soundtrack-CD/4-Music-Commentary-by-Koji-Kondo-1-/4-Music-Commentary-by-Koji-Kondo-1--219986.html)
- **Loop/form logic:** no captured microloop, section recall, intro exclusion, return point, or independently repeating channel. The player manually repeats and varies A.
- **Game-state adaptation:** no knowledge of running, jumping, stage, location, enemy, danger, player success, or screen transition. Both cited composers describe composing against game imagery, movement, stages, or testing; a live harmonizer has none of that context. [Nintendo](https://www.nintendo.com/en-gb/Iwata-Asks/Super-Mario-Bros-25th-Anniversary/Vol-5-Original-Super-Mario-Developers/5-To-Save-Memory/5-To-Save-Memory-212974.html) [Brave Wave](https://bravewave.net/interviews/a-conversation-with-manami-matsumae/)
- **Either composer’s identity:** no preset can manufacture Kondo’s compositional decisions, Matsumae’s melodic judgment and training, either composer’s game-development collaboration, or their broader careers. Composer names remain contextual references only.

## 13. Product copy

### Result

**Turns one clean note into a three-voice stack: your hook plus two tightly bounded harmony roles. Sparse input suggests early-console economy; it does not add drums, loops, chip sound, or independent bass and counterpoint.**

### Play it like — short prompt

**Play a catchy two-bar single-note hook. Repeat it exactly, change one note, then leave a full bar of silence.**

### Expanded guidance

Use clean, separated notes around the middle register at 100–150 BPM. Keep each two-bar idea to four–eight attacks and leave at least one beat empty. Listen after the first note for three voices total—not three extra—and make sure all three release together. Guitarists: pick one fretted note at a time, mute unused strings, and avoid bends, slides, and wide vibrato. Keyboardists: use one finger linearly, keep sustain and arpeggiators off, and reserve a modest velocity accent for the hook’s first note. You supply all rhythm, repetition, variation, and form; the preset supplies only the simultaneous pitch mapping.

### Capability / honesty label

**Available now: three-total-voice harmony mapping. Not included: chip synthesis, percussion, pattern playback, independent parts, loops, or adaptive game music.**

## 14. Deterministic 32-second acceptance exercise

**Setup:** 4/4, 120 BPM, any diatonic key/scale selected by the implementation, transport either stopped or running. Use scale degrees rather than a copyrighted melody. All notes are clean and monophonic. Quarter notes sound for roughly 350–400 ms; eighth notes for roughly 180–220 ms; release before the next onset. Keyboard velocities are given; guitar uses even medium attacks.

### Source script

Define two-bar hook **A**:

- **Bar 1:** `1` quarter (velocity 100), `3` eighth (84), `2` eighth (84), `5` quarter (90), quarter rest.
- **Bar 2:** `3` quarter (88), `2` quarter (82), `1` half (76).

Define **A′** as A with only the first degree of bar 2 changed from `3` to `4`; rhythm and velocities otherwise identical.

Perform 16 bars:

1. Bars 1–2: A  
2. Bars 3–4: A exactly  
3. Bars 5–6: A′  
4. Bar 7: full-bar rest  
5. Bar 8: `5` quarter (88), `2` quarter (80), `1` half (72)  
6. Bars 9–10: A  
7. Bars 11–12: A′  
8. Bar 13: full-bar rest  
9. Bars 14–15: A exactly  
10. Bar 16: `2` quarter (80), `1` dotted half (68), then release and remain silent.

At 120 BPM, 16 bars last exactly **32 seconds**.

### Observable pass conditions

1. Each accepted source onset produces **no more than three total sounding notes**: source plus two mapped notes.
2. No generated entrance occurs without a source onset; the two generated roles do not claim rhythmic independence.
3. The exact A repetition in bars 3–4 yields the same pitch relationships as bars 1–2.
4. A′ changes output only where the one changed source degree occurs; it does not trigger a new pattern or section.
5. Bars 7 and 13 are silent after release: no bass continuation, counterline answer, percussion, or loop.
6. Running versus stopped transport does not change the note-to-note result for this baseline.
7. After the final bar-16 Note Off, active and pending source/generated note counts reach zero.
8. Repeating the complete exercise from reset yields the same event relationships and final zero-note state.

### Immediate fail conditions

- more than three total notes from one valid monophonic onset;
- any note during a scripted full-bar rest;
- a generated note surviving its source release;
- a different output for the exact repeated A without a documented stateful reason;
- a crash, stuck note, or residual pending event after final release or Panic.

## 15. Confidence and gaps

- **High confidence:** Kondo’s stated three-sound constraint, location differentiation, and revision to match movement; Matsumae’s stated three-channel comfort, Bach background, memorable-melody goal, stage knowledge, and sound-effect/music negotiation. These are direct interviews.
- **High confidence:** NES channel inventory, common pulse/triangle roles, monophonic channel behavior, channel dropout for effects, and independent channel-loop concepts. These come from peer-reviewed technical/music-theory analysis.
- **High confidence:** bends, vibrato, slides, hammer-ons, and pull-offs complicate guitar note tracking. This comes from peer-reviewed empirical work.
- **Medium confidence:** the proposed tempo, register, velocity, density, and phrase bounds. They are deliberately conservative HCI recommendations derived from the evidence and the present clean-monophonic product boundary, not historical measurements of the selected scores.
- **Medium confidence:** audibility of low and counterline “roles” depends on the eventual exact harmony/voicing configuration and sound routing. Parent synthesis and implementation testing must verify crossing, range fallback, and balance.
- **Unresolved:** this performer report does not determine the exact scale, harmony strategy, octave placement, gains, or voice-leading configuration; those require reconciliation with the independent theory report and implementation tests.

## Sources

### Kept

- [Nintendo, “Music Commentary by Koji Kondo (1)”](https://www.nintendo.com/en-gb/Iwata-Asks/Iwata-Asks-Super-Mario-All-Stars/Vol-1-Super-Mario-History-Soundtrack-CD/4-Music-Commentary-by-Koji-Kondo-1-/4-Music-Commentary-by-Koji-Kondo-1--219986.html) — first-party interview containing Kondo’s three-sound, location, movement, and percussion recollections.
- [Nintendo, “To Save Memory”](https://www.nintendo.com/en-gb/Iwata-Asks/Super-Mario-Bros-25th-Anniversary/Vol-5-Original-Super-Mario-Developers/5-To-Save-Memory/5-To-Save-Memory-212974.html) — first-party developer interview on prototype testing, matching music to gameplay rhythm, and memory economy.
- [Brave Wave, “A Conversation with Manami Matsumae”](https://bravewave.net/interviews/a-conversation-with-manami-matsumae/) — translated direct interview on three-channel writing, Bach training, memorable melody, stage awareness, sound effects, and later reflection.
- [Karen M. Cook, “8-Bit Affordances,” *Music Theory Online* 29.3](https://www.mtosmt.org/issues/mto.23.29.3/mto.23.29.3.cook.php) — peer-reviewed account of NES channels, roles, silence, sound-effect competition, loops, and channel independence; used for era-wide context, not as evidence about either named composer’s identity.
- [Su et al., “TENT: Technique-Embedded Note Tracking for Real-World Guitar Solo Recordings”](https://transactions.ismir.net/articles/10.5334/tismir.23) — peer-reviewed empirical evidence for guitar-technique tracking hazards.
- [MIDI Association, “About MIDI—Part 3: MIDI Messages”](https://midi.org/about-midi-part-3midi-messages) — authoritative explanation of Note On, Note Off, velocity, pitch bend, modes, and real-time clock.
- [VG247/USgamer, “Manami Matsumae, the Maestro of Mega Man”](https://www.vg247.com/manami-matsumae-the-maestro-of-mega-man?page=2) — direct interview corroborating that she matched music to Mega Man’s movement/energy and worked under a short production schedule; retained as secondary corroboration, not the primary basis.

### Dropped

- **Shmuplations, “Koji Kondo — 2001 Composer Interview”** — useful translated interview but not needed after first-party Nintendo sources covered the operative performer claims; excluded to avoid redundant translation dependence.
- **NintendoEverything summaries of an NHK interview** — secondary summaries without the complete primary transcript; excluded in favor of Nintendo’s direct interviews.
- **NESdev Wiki / technical reference** — technically valuable community documentation, but the fetched page was incomplete and the peer-reviewed Cook article supplied the required channel facts with analytical context.
- **Andrew Schartmann, *Analyzing NES Music*** — promising scholarly book description, but full relevant passages were not available in this research pass; no claim rests on a publisher blurb.
- **Fan transcriptions, soundtrack uploads, “best 8-bit music” lists, and style tutorials** — excluded because authorship, versions, measurements, or analytical methods were insufficiently controlled, and because they encourage melodic copying or generic “chiptune” caricature.

## Final performer finding

Pixel Trio is honest and useful only when its promise is small: **the player authors a memorable, sparse line; the system makes a three-note vertical economy audible and cleans it up deterministically.** Independent voices, chip orchestration, patterns, percussion, looping, game adaptation, and composer identity remain outside the current mapping and must stay outside the product claim.
