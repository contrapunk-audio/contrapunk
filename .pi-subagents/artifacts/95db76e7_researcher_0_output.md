# Performer / HCI Research Report — Preset 43 “Hollow Choir”

**Role:** Independent performer and interaction researcher  
**Preset status assumed:** current shared `HarmonyEngine` minor-chorale mapping; clean monophonic input; no motif lane, adaptive scene system, independent counterline, or sound-design substitution  
**Research date:** 2026-07-19

## Summary

“Hollow Choir” should be played as a restrained, breath-shaped minor melody, not as a request for the engine to compose a Christopher Larkin cue. The player supplies the singable contour, phrase timing, held arrivals, restraint, and silence; the current engine can supply note-coupled minor chorale voices. It cannot supply the live soprano, acoustic/orchestral color, distant layers, independent counterline, interactive game scoring, reverberant world, or Christopher Larkin’s authorship and identity documented in the bounded *Hollow Knight* corpus.

The honest interaction is therefore: **one clean note at a time, mostly stepwise, softly attacked, with long destination notes and audible rests; listen to each generated sonority settle before beginning the next thought.** Larkin describes the project direction as “melancholic elegance,” while contemporary reporting gives the fuller brief as dark elegance, minimal instrumentation, classical character, and melancholy. The original score uses soft piano and live viola prominently, and “City of Tears” specifically uses Amelia Jones’s soprano for an ethereal quality—evidence that the reference sound is an authored orchestral/vocal production, not merely a minor scale or a generic choir patch. [Polygon](https://www.polygon.com/2024/8/8/24215602/christopher-larkin-hollow-knight-interview/) [Game Developer](https://www.gamedeveloper.com/audio/crafting-an-evocative-score-for-i-hollow-knight-i-) [Official “City of Tears” credit](https://christopherlarkin.bandcamp.com/track/city-of-tears)

## 1. Scope and performer-facing corpus

This report is deliberately bounded to the 2017 *Hollow Knight* base-game score, especially:

- **“Dirtmouth” / opening-menu language:** soft piano and viola, identified by Larkin as the project’s starting palette. [Game Developer](https://www.gamedeveloper.com/audio/crafting-an-evocative-score-for-i-hollow-knight-i-)
- **“Crossroads”:** described as slow, sweeping string melody with distant harp and ghostly effects, expressing a once-prosperous kingdom now absent. [Bandcamp Daily interview](https://daily.bandcamp.com/features/christopher-larkin-review)
- **“City of Tears”:** the bounded vocal reference; official credits name soprano Amelia Jones, and Larkin/reporting connects that live voice to the track’s ethereal quality. [Official track page](https://christopherlarkin.bandcamp.com/track/city-of-tears) [Game Developer](https://www.gamedeveloper.com/audio/crafting-an-evocative-score-for-i-hollow-knight-i-)
- **The base-score arc:** the official soundtrack description says it begins with soft piano notes echoing the Hollow Knight theme and later swells to orchestral scale as the kingdom’s fate unfolds. [Steam official soundtrack](https://store.steampowered.com/app/598190/Hollow_Knight__Official_Soundtrack)

This is **not** a whole-career claim, a study of every base-game cue, the DLC album, or *Silksong*. Fast boss writing such as “Mantis Lords” is outside the performance target: Larkin explicitly associates it with Vivaldi, tremolo strings, harpsichord, speed, virtuosity, and precision. [Polygon](https://www.polygon.com/2024/8/8/24215602/christopher-larkin-hollow-knight-interview/)

## 2. Evidence-to-interaction translation

1. **Restraint is more defensible than spectacle.** The initial direction was dark/melancholic elegance with minimal instrumentation; Larkin says the score is not continuously heavy-handed with full epic orchestration. The preset should invite sparse melody rather than imply that more input will create a richer imitation. [Game Developer](https://www.gamedeveloper.com/audio/crafting-an-evocative-score-for-i-hollow-knight-i-) [Polygon](https://www.polygon.com/2024/8/8/24215602/christopher-larkin-hollow-knight-interview/)
2. **Register and color carry narrative meaning in the source, but the player only controls part of that.** Larkin chose instruments in response to each location—harp and marimba for Greenpath, organ for Soul Sanctum, and other distinct colors elsewhere. A single universal “Larkin scale” would therefore be a caricature. The performer can contribute a narrow, vocal register and soft dynamics; instrumentation remains sound design. [Game Developer](https://www.gamedeveloper.com/audio/crafting-an-evocative-score-for-i-hollow-knight-i-) [Polygon](https://www.polygon.com/2024/8/8/24215602/christopher-larkin-hollow-knight-interview/)
3. **Phrase boundaries must be performed.** Long notes and temporal gaps are established cues in phrase perception, and controlled expressive timing helps listeners parse phrase boundaries. That supports held destinations and genuine rests instead of continuous eighth-note input. [Bod, *What contributes to the perception of musical phrases*](https://eprints.illc.uva.nl/id/eprint/2057/1/DS-2007-02.text.pdf) [PLOS ONE](https://journals.plos.org/plosone/article?id=10.1371%2Fjournal.pone.0055150)
4. **The world-like distance in the reference is production, not harmony.** Larkin describes room drips, echoes, and very low-mixed environmental sounds used to imply space and memory. That evidence supports honest limits: a chorale mapping does not create distance, ambience, or a remembered crowd. [Bandcamp Daily interview](https://daily.bandcamp.com/features/christopher-larkin-review)

## 3. Best source gesture

**Best gesture:** a **single-note, two-to-five-note vocal motif** expanded into a short phrase, ending on one held destination.

- Begin with one or two notes rather than a complete busy theme.
- Prefer stepwise motion and small thirds; reserve one fourth or fifth for the phrase’s emotional crest.
- Treat a repeated pitch as insistence, not motor rhythm: repeat it once, then change direction or rest.
- End each phrase on a stable-feeling scale degree and hold it long enough to hear the complete mapped chorale.
- Do not enter chords. The product’s clean-monophonic contract means **one source NoteOn remains active at a time**.

Why this works: it gives the current note-coupled harmony mapping legible material and lets the performer author the pacing. It does not pretend the engine recognizes leitmotifs, since distinguishable leitmotifs were an explicit compositional challenge in the source and require authored association and development. [Game Developer](https://www.gamedeveloper.com/audio/crafting-an-evocative-score-for-i-hollow-knight-i-)

## 4. Concrete performance contract

| Dimension | Contract | Why / audible consequence |
|---|---|---|
| **Tonal material** | Stay in the preset’s displayed minor scale. Prefer `1, 2, ♭3, 4, 5, ♭6, ♭7`; emphasize `1`, `♭3`, and `5`. | Keeps the present diatonic minor mapping predictable; chromaticism is not the player’s route to “mystery.” |
| **Register** | **Guitar:** written/sounding E3–E5, preferably A3–D5. **Keyboard:** C4–E5 as the default melody band; move at most an octave higher for one crest. | A compact human-voice-like band remains singable and leaves room for mapped voices. Extremely low input crowds lower harmonies; extreme high input becomes brittle. |
| **Tempo** | 56–76 BPM; acceptance at 64 BPM, 4/4. | Slow enough to hear attacks, continuations, and releases without claiming a fixed tempo from the corpus. |
| **Note lengths** | Interior notes: 1–2 beats. Phrase destination: 3–4 beats. Release every note cleanly before or exactly as the next begins. | The engine follows note lifecycle; long arrivals reveal the chorale while rests expose cleanup. |
| **Articulation** | Soft, connected **non-overlapping legato**. Tiny separation (roughly 30–100 ms) is safer than overlap. No tremolo, rake, trill, or repeated-note machine-gun attack. | Maintains monophony and avoids stacked mappings or tracker ghosts. |
| **Velocity** | MIDI 42–72; start 48–56, crest 64–72, cadence 44–56. Avoid alternating extremes. | The player supplies a shallow expressive arc. This does not guarantee acoustic crescendo if the receiving sound ignores velocity. |
| **Density** | 2–5 notes per phrase; normally no more than one new note per beat; one 6-note phrase is the upper bound in the exercise. | Sparse input honors the minimal, melancholic reference and keeps every generated sonority intelligible. |
| **Silence** | 2 beats minimum between short clauses; one full bar after each completed phrase; two bars before the final return. | Silence is part of the material, not idle time. It also verifies that notes actually release. |
| **Repetition** | Repeat one contour once, changing exactly one of: final scale degree, one interior note, register at the crest, or velocity arc. | Creates recognizability without requiring motif memory or automatic variation. |
| **Pitch bend / aftertouch** | Keep pitch bend centered for this baseline. Aftertouch may color an external instrument but is not part of the harmony promise. | Avoids implying that continuous expressive controls are harmonically interpreted. |

These are interaction recommendations, not claims that Larkin uses one fixed scale, tempo, register, or phrase length throughout the score.

## 5. Transport dependency and rhythmic precision

- **Musical dependency:** transport is **recommended, not required**, for the current note-to-chorale mapping. Start it when the UI provides it so bar-counted practice and acceptance are repeatable.
- **Engine dependency:** do not claim tempo-synchronized evolution, adaptive entries, or delayed responses. A shared note-on/note-off harmony mapping can respond without knowing a game state or a formal section.
- **Precision:** downbeat-perfect input is unnecessary. Clean releases and intentional gaps matter more than quantization. At acceptance, allow normal human timing but keep each event in its named beat/bar window.
- **Stopped transport:** the mapping must still release all generated notes. Stopping transport must never be the only way to end a chord; NoteOff, Panic, and preset replacement must clean up deterministically.

## 6. How the player develops the material

Use a four-state human-performed arc:

`Call → Settle → One variation/crest → Withdrawal`

1. **Call:** play a 3-note rising or arching idea softly in the middle register.
2. **Settle:** hold its destination for 3–4 beats. Do not decorate it; hear the harmony’s sustain.
3. **Breathe:** release fully and leave a bar empty.
4. **Variation:** restate the contour once, altering one note or lifting only its crest. Increase velocity slightly, not density.
5. **Withdrawal:** after a longer rest, return a shortened fragment at lower velocity and hold tonic or another stable destination.

The performer, not the preset, makes this an evolving phrase. The current mapping can change sonority when input pitch changes, but it cannot remember that a motif “means” a place, decide when the story intensifies, add an independently composed countermelody, or orchestrate a scene.

## 7. Listening and responding

After each note:

1. Listen for the **attack** of all mapped voices. If attacks smear or double, simplify the source and check tracking.
2. On a destination note, wait until the sonority feels stationary—at least 2 beats—before deciding whether the phrase needs another pitch.
3. Release, then listen for **complete silence**. Do not cover a stuck note with the next phrase.
4. Begin the next clause only after the generated harmony has ended and at least 2 beats of silence have passed.
5. If a generated inner voice moves unexpectedly, answer by **holding or resting**, not by adding corrective notes. More input multiplies the ambiguity.

The listening loop is deliberately local: source note → mapped chorale → full release. It does not advertise an autonomous ensemble that listens back.

## 8. Guitar-specific contract

“Clean monophonic” describes both signal and technique:

- Use a clean DI/bridge-or-neck pickup signal with distortion, delay, chorus, and reverb bypassed **before** pitch-to-MIDI conversion. Recorded reverb, delay, modulation, or distortion can compromise translation. [Sound on Sound, *Jam Origin MIDI Guitar 2*](https://www.soundonsound.com/reviews/jam-origin-midi-guitar-2)
- Tune first. Jam Origin’s documentation explicitly recommends staying in tune for best tracking. [Jam Origin documentation](https://www.jamorigin.com/docs/midi-guitar-for-ios/)
- Pick one string at a time with moderate, even attacks. Fret cleanly; mute the five unused strings with both hands.
- Prefer the middle strings and A3–D5 for the exercise. Low pitches inherently take longer to identify, while the pick transient begins as noisy/chaotic rather than stable pitch. [Sound on Sound, *MIDI Guitar Workshop*](https://www.soundonsound.com/techniques/midi-guitar-workshop)
- Use small position shifts rather than open strings ringing under fretted notes.
- Avoid bends, wide vibrato, slides across semitone boundaries, hammer-on/pull-off flurries, harmonics, pick scrapes, and sympathetic ringing during acceptance.
- Let the old note mute before the new attack. “Legato” here means connected phrase shape, **not overlapping strings**.

A guitarist may add subtle vibrato only in free play after tracking is proven stable; the preset does not promise to harmonize continuous pitch expression coherently.

## 9. MIDI-keyboard opportunities and limits

- Use one hand and one key at a time. The free hand may ride an external expression control, but it must not add notes.
- Velocity gives the keyboard player the cleanest reproducible arc: soft call, modest crest, soft return. Expressive performance ordinarily includes dynamics, timing, and articulation; the preset should preserve rather than manufacture those choices. [Frontiers review](https://www.frontiersin.org/journals/digital-humanities/articles/10.3389/fdigh.2018.00025/full)
- Use deliberate key release. Sustain pedal stays **up** during acceptance because it can extend note lifecycle and obscure whether the mapping cleans up.
- Key overlap should be zero or negligible. If the device scans quick overlaps, insert a visibly small gap.
- **Chord-density limit:** one active source note. Dyads, grace-note crushes, octave doubling, and pedal-held notes violate the contract even if the hardware can transmit them.
- A keyboard may safely make one octave displacement at the variation crest, but repeated octave toggling falsely suggests separate ensemble roles.

## 10. Failure gestures and recovery

| Failure gesture | Likely result | Recovery |
|---|---|---|
| Chord, dyad, overlapping key/string, or sustain pedal | Several complete chorales at once; mud or note storm | Release all input, pedal up, wait for silence; Panic if any voice remains. Resume with one note. |
| Fast repeated notes, tremolo picking, trill | Re-triggered attacks dominate; no vocal phrase | Stop, leave one full bar, restart at one note per beat or slower. |
| Low guitar notes with hard pick attack | Latency, octave/pitch mistracking, ghost NoteOns | Move up an octave, soften attack, mute unused strings, verify tuning/gate. |
| Slide/bend across notes | Ambiguous or stepped MIDI pitches; harmony churn | Re-center pitch, release completely, restart from a discrete fretted/keyed pitch. |
| Continuous scale run with no destination | Generic parallel harmonizer demonstration, not a mournful phrase | Select 3–5 notes, decide the arrival in advance, hold it, then rest. |
| Dense chromatic input | Mapping may borrow or force consonances unpredictably; stylistic contradiction | Return to displayed minor-scale tones. Use silence, register, and dynamics for tension. |
| Trying to “correct” an odd generated note by adding notes | Escalating density and unclear ownership | Hold or release; do not negotiate with the mapping polyphonically. |
| Preset change or transport stop while notes are held | Risk of stale notes if lifecycle is defective | Release first when possible; product must issue deterministic cleanup/Panic on replacement regardless. |

## 11. Lifecycle contract

1. **Before playing:** select the preset, confirm tonic/minor mode, verify no active notes, tune guitar if applicable, pedal up, and optionally start 64 BPM / 4/4 transport.
2. **Note ownership:** every clean source NoteOn may create a bounded chorale; its matching source NoteOff must release every note generated from that event.
3. **Phrase rest:** rests must contain no active or pending voices. Silence is both musical material and the performer-visible cleanup check.
4. **Held destination:** Hold may lengthen the perceived destination only if its UI semantics are explicit. The baseline exercise assumes ordinary source-held notes, not latch/Forever behavior.
5. **Panic:** Panic must immediately clear source-derived harmony and any pending lane events.
6. **Preset replacement / disable:** apply must perform one cleanup transaction before new configuration becomes audible; no old chorale may survive.
7. **Transport stop:** must not strand notes. Restart must begin from an empty runtime state rather than replaying the previous held destination.
8. **End:** key/string released, sustain off, zero sounding harmony, zero pending events.

## 12. Product copy

### Result

**Your minor melody opens into a restrained chorale. Long notes let the harmony settle; rests keep it distant and clear.**

### Play it like — short prompt

**Play one soft, singable minor phrase at a time. Hold the destination, release completely, then leave a long breath.**

### Expanded guidance

Use 2–5 clean single notes in a comfortable vocal register. Move mostly by step or small third, save one wider leap for a crest, and keep velocities gentle. Hold the last note for 3–4 beats, listen to the full chorale settle, then release and leave at least one bar of silence. Repeat the contour once with only one small change. Guitarists: use a dry clean signal and mute every unused string. Keyboardists: use one key at a time, no sustain pedal and no overlapping notes.

### Required approximation disclosure

**Inspired by the sparse, melancholic vocal/orchestral atmosphere of a bounded *Hollow Knight* reference—not a Christopher Larkin style emulator. This preset maps your notes to minor chorale voices. It does not generate a choir, acoustic orchestration, distant ensemble layers, an independent counterline, adaptive game scenes, narrative response, ambience, or reverb. Sound choice and performance create the color.**

## 13. What the player creates vs. what the system cannot manufacture

| The player creates | The current system cannot honestly manufacture |
|---|---|
| The actual melody, its singability, contour, tonal focus, repetition, and one-note variation | A Christopher Larkin melody, leitmotif, compositional judgment, identity, or authorship |
| Phrase attack, continuation, crest, held destination, release, and silence | Formal development, motif memory, an independently composed counterline, or an autonomous ensemble response |
| Register, articulation, source velocity, density, restraint, and timing | Live soprano/choir technique, live viola, piano touch, harp, organ, strings, brass, or acoustic orchestration |
| A human decision to repeat, vary, intensify, or withdraw | Game-state sensing, location-specific instrumentation, adaptive scenes, narrative or character response |
| Clean note lifecycle and the opportunity for mapped voices to sound clearly | “Distance,” room size, echo, drips, low-mixed memories, ambience, reverb, or spatial ensemble layers |
| A minor chorale approximation using the shared HarmonyEngine | The documented contrast across the full soundtrack, including virtuosic boss cues and orchestral swells |

The distinction matters because the official score credits a real soprano for “City of Tears,” identifies live viola elsewhere, and describes an arc from soft piano to orchestral scale. Those production and composition choices cannot be collapsed into a harmony preset. [Official “City of Tears” credit](https://christopherlarkin.bandcamp.com/track/city-of-tears) [Game Developer](https://www.gamedeveloper.com/audio/crafting-an-evocative-score-for-i-hollow-knight-i-) [Steam official soundtrack](https://store.steampowered.com/app/598190/Hollow_Knight__Official_Soundtrack)

## 14. Deterministic 45-second acceptance exercise

**Setup:** tonic A; displayed minor mode (A Aeolian for this abstract test); 4/4; 64 BPM; transport running; one active input note maximum; sustain/latched Hold off; dry source; velocity as listed. At 64 BPM, 12 bars last exactly **45 seconds**.

Scale degrees are relative to A minor. Notes are examples only, not copied melodies.

| Bar(s) | Input events | Expected performer/system behavior |
|---|---|---|
| 1 | Beat 1: `1` (A3), velocity 50, hold 2 beats. Beat 3: `♭3` (C4), velocity 54, hold 1 beat. Beat 4: `4` (D4), velocity 56, hold 1 beat. | Three clean monophonic attacks; each source release ends its mapped sonority before/at the next attack. |
| 2 | Beat 1: `5` (E4), velocity 58, hold all 4 beats; NoteOff exactly at bar end. | Complete chorale is stable for the destination; no autonomous counterline is expected. |
| 3 | Rest, 4 beats. | Absolute silence by bar 3 beat 1; zero active notes throughout rest. |
| 4 | Beat 1: `1` (A3), v50, 2 beats. Beat 3: `♭3` (C4), v56, 1 beat. Beat 4: `4` (D4), v60, 1 beat. | Recognizable restatement; same contour and slightly stronger arc. |
| 5 | Beat 1: **variation** `♭6` (F4), v68, hold 2 beats. Beat 3: `5` (E4), v58, hold 2 beats; release at bar end. | Exactly one pitch-level variation/crest, then a held settling note; no added player chord. |
| 6 | Rest, 4 beats. | Full release; no stale destination. |
| 7 | Beat 1: `2` (B3), v48, 1 beat. Beat 2: `♭3` (C4), v52, 1 beat. Beat 3: `5` (E4), v60, hold 2 beats. | Short new clause, still sparse and singable. |
| 8 | Beat 1: `4` (D4), v54, 1 beat. Beat 2: `♭3` (C4), v50, hold 3 beats; release at bar end. | Descending withdrawal and long destination. |
| 9–10 | Rest, 8 beats. | Two-bar breath; zero active and pending notes. If any voice sounds, fail and use Panic. |
| 11 | Beat 1: `♭3` (C4), v46, hold 2 beats. Beat 3: `2` (B3), v44, hold 2 beats. | Soft shortened return, no density increase. |
| 12 | Beat 1: `1` (A3), v42, hold 3 beats; release on beat 4. Beat 4: rest. | Final mapped chorale releases completely within the last beat. End state: zero active/pending notes. |

**Pass criteria**

- Input remains monophonic; no event outside the displayed minor scale.
- No more than one attack per beat; all three destination notes are audibly longer than their approaches.
- Bars 3, 6, 9, and 10 are silent.
- The second phrase differs from the first at one musically legible point rather than becoming denser.
- Every source NoteOff releases all voices attributed to it.
- At 45 seconds, active notes = 0 and pending events = 0.
- Repeat once with transport stopped: pitches and cleanup still work; only bar-count convenience is lost.
- Trigger Panic during the bar-5 destination in a separate lifecycle check: immediate silence and empty runtime state are required.

## 15. Confidence and gaps

- **High confidence:** the source brief emphasizes melancholic/dark elegance and minimal instrumentation; soft piano and viola are central starting colors; “City of Tears” credits Amelia Jones; instrumentation is location-sensitive; the score’s large arc reaches orchestral scale. These are directly supported by Larkin interviews and official credits.
- **High confidence:** the current baseline must not claim choir synthesis, acoustic orchestration, ambience, narrative adaptation, or Christopher Larkin’s identity. Those require capabilities or authored material beyond note-to-harmony mapping.
- **Medium confidence:** the proposed register, 56–76 BPM range, velocity band, 2–5-note phrase size, and exact rest lengths. They are ergonomic HCI constraints derived from the bounded aesthetic and monophonic engine behavior, not transcriptions or universal properties of the score.
- **Medium confidence:** A Aeolian in acceptance is a deterministic test choice, not a claim about the key or complete pitch grammar of the cited tracks.
- **Gap:** no licensed full score or stem session was examined, so this report does not assert exact voicings, meters, keys, MIDI velocities, mix distances, or bar-level orchestration.
- **Gap:** the final preset’s exact harmony mode, voice count/position, leading, octave placement, Hold resolution, and gains were not supplied to this role. The parent synthesis must confirm that they preserve bounded density and deterministic ownership.
- **Gap:** surface-specific pitch-to-MIDI behavior varies. The guitar limits should be validated on each supported tracker/device rather than marketed as universal latency or accuracy guarantees.

## 16. Sources

### Kept

- [Christopher Larkin / Bandcamp — “City of Tears”](https://christopherlarkin.bandcamp.com/track/city-of-tears) — primary artist page; confirms the 2017 release context and soprano Amelia Jones credit.
- [Steam — *Hollow Knight Official Soundtrack*](https://store.steampowered.com/app/598190/Hollow_Knight__Official_Soundtrack) — official product description and track list; supports the soft-piano-to-orchestral-scale arc.
- [Polygon — “Hollow Knight’s composer talks musical inspirations”](https://www.polygon.com/2024/8/8/24215602/christopher-larkin-hollow-knight-interview/) — direct Larkin answers on melancholic elegance, instrumentation, area-specific choices, and “Mantis Lords.”
- [Game Developer — “Crafting an evocative score for Hollow Knight”](https://www.gamedeveloper.com/audio/crafting-an-evocative-score-for-i-hollow-knight-i-) — direct quotations on the initial brief, piano/viola, location-specific instrumentation, live musicians, and “City of Tears.”
- [Bandcamp Daily — “Inside Christopher Larkin’s Darkly Elegant Hollow Knight Score”](https://daily.bandcamp.com/features/christopher-larkin-review) — direct interview material on “Crossroads,” environmental echo, memory, exploration, and the game’s world-building context.
- [Team Cherry — “Composers! Sound Designers! Hollow Knight’s Getting Aural!”](https://www.teamcherry.com.au/blog/composers-sound-designers-hollow-knights-getting-aural) — primary developer context for Larkin’s close collaboration with the team.
- [Sound on Sound — “MIDI Guitar Workshop”](https://www.soundonsound.com/techniques/midi-guitar-workshop) — technical explanation of pitch tracking, noisy pick transients, low-note latency, and careful guitar-synth technique.
- [Jam Origin — MIDI Guitar documentation](https://www.jamorigin.com/docs/midi-guitar-for-ios/) — vendor documentation supporting tuning and tracking setup cautions.
- [Sound on Sound — “Jam Origin MIDI Guitar 2”](https://www.soundonsound.com/reviews/jam-origin-midi-guitar-2) — practical evidence that time/modulation/distortion processing can compromise guitar-to-MIDI translation.
- [PLOS ONE — “Expressive Timing Facilitates the Neural Processing of Phrase Boundaries in Music”](https://journals.plos.org/plosone/article?id=10.1371%2Fjournal.pone.0055150) — peer-reviewed support for performed timing as a phrase-boundary cue.
- [Bod — *What contributes to the perception of musical phrases in western classical music*](https://eprints.illc.uva.nl/id/eprint/2057/1/DS-2007-02.text.pdf) — synthesis of long notes, rests, interval changes, and repetition as phrase cues.
- [Frontiers — “Computational Models of Expressive Music Performance”](https://www.frontiersin.org/journals/digital-humanities/articles/10.3389/fdigh.2018.00025/full) — scholarly review supporting timing, dynamics, and articulation as performer-controlled expressive dimensions.

### Dropped

- **Wikipedia, “Music of Hollow Knight”** — useful discovery index but redundant with direct interviews and official credits.
- **Fan analyses and YouTube theory essays** — potentially perceptive, but unnecessary where direct Larkin testimony and official credits answer the bounded HCI question; unverified harmonic claims were excluded.
- **Official *Hollow Knight Piano Collections*** — licensed but arranged by David Peacock rather than the original orchestration; inappropriate evidence for exact base-score voicing or keyboard technique.
- **AllMusic and retailer listings** — metadata duplicated stronger official sources and offered little performer evidence.
- **Jam Origin user-forum anecdotes** — device- and setup-specific, with weaker provenance than vendor documentation and technical reporting.
- ***Hollow Knight: Silksong* sources and “The Choir”** — later project outside the explicitly bounded 2017 base-game corpus; using the title alone would falsely turn a different score into evidence for this preset.

## 17. Recommendation

**Ready for parent synthesis as an honest baseline performer contract**, provided the final product copy retains the approximation disclosure and the implementation proves note ownership, all-rest silence, Panic, preset replacement, and stopped-transport cleanup. If the shipped preset advertises a “counterline,” “choir,” “distance,” or adaptive scene behavior beyond note-coupled minor chorale voices, it should instead be capability-gated until those features actually exist.