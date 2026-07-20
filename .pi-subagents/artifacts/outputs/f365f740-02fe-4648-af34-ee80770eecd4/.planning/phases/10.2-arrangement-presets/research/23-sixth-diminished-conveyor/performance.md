# Performer/HCI research — Preset 23: Sixth-Diminished Conveyor

**Role:** performer and interaction research  
**Referenced practice:** Barry Harris’s sixth-diminished harmonic pedagogy, bounded to live, single-note control of Contrapunk’s existing four-voice Barry Harris voicer  
**Research date:** 2026-07-19

## Summary and performer thesis

The truthful source gesture is a **clean monophonic line whose notes deliberately alternate between the configured sixth-chord tones and the related diminished passing tones**. Contrapunk can make that alternation immediately audible as fixed four-note drop-2 voicings; the player must supply the line, time, accents, approach-and-resolution logic, phrase shape, and rests. Harris-specific evidence consistently frames chords as movement rather than static objects: the Barry Harris Institute describes sixth-diminished scales as a means of “creating movement,” outlining chord movement, harmonizing melody, borrowing, and resolving; its workshop description says chords are “opportunities for linear movement.” [BHIJ six-part series](https://barryharrisinstituteofjazz.org/spring-2026-a-six-part-zoom-series/) [BHIJ workshops](https://barryharrisinstituteofjazz.org/workshops/)

This preset should therefore not promise that arbitrary input becomes Barry Harris phrasing. It is a bounded, keyed **melody-to-sixth/diminished drop-2 voicing demonstrator**.

## Evidence boundary: Harris-specific versus generic bebop advice

### Harris-specific pedagogy

- Harris’s scale of chords pairs a sixth chord with its related diminished chord in one eight-note collection. Alternating scale-degree parity yields alternating harmonic families, and borrowing a note between the families creates tension that asks to resolve. [Barry Harris Companion, “Scales of Chords”](https://barrycompanion.com/scales-of-chords/)
- The Institute’s current curriculum explicitly separates but relates **harmony** (sixth chords, sixth-diminished scales, movement, borrowing, melody harmonization) and **improvisation** (extra-note rules, related diminished, surrounding, scale maps). The preset implements only a narrow part of the harmony side; it does not implement that larger improvisational curriculum. [BHIJ six-part series](https://barryharrisinstituteofjazz.org/spring-2026-a-six-part-zoom-series/)
- A first-hand report from Harris’s workshop emphasizes small chords in a lower register, melody/song knowledge, and “moving chords through scales,” not isolated chord-scale matching. [JazzTimes, “Inside the Barry Harris Method”](https://www.jazztimes.com/features/profiles/inside-the-barry-harris-method/)
- Howard Rees’s workshop materials, produced from long study with Harris, describe the method as chord movement applied to accompaniment, arranging, composition, and improvisation rather than a stock voicing pack. [Howard Rees, *The Barry Harris Workshop Video*](https://jazzworkshops.com/the-barry-harris-workshop-video/)
- The dedicated guitar adaptation is presented as structural components for personal discovery, explicitly not a set of instant “hip sounding voicings.” [Jamey Aebersold listing, Alan Kingstone, *The Barry Harris Harmonic Method for Guitar*](https://www.jazzbooks.com/jazz/product/BHG)

### Generic bebop performance advice used here, not claimed as uniquely Harris’s

- Use a lightly swung or nearly even eighth-note feel according to tempo, rather than a fixed exaggerated triplet. Empirical jazz-performance research finds that swing ratio varies with tempo; faster playing tends toward straighter subdivision. [Friberg & Sundström, *Music Perception* 19/3, DOI 10.1525/mp.2002.19.3.333](https://doi.org/10.1525/mp.2002.19.3.333) [Butterfield, *Music Theory Spectrum* 33/1, DOI 10.1525/mts.2011.33.1.3](https://doi.org/10.1525/mts.2011.33.1.3)
- Connected line, selective accents, forward-moving offbeats, phrase-end release, and rhythmic variation are generic bebop craft. They help expose the preset’s movement, but they are not evidence that the engine reproduces Harris’s personal touch or time.

## 1. Best source gesture

**Primary gesture:** one connected, single-note, mostly stepwise 4–8-note line through the displayed BH Major 6th Diminished or BH Minor 6th Diminished collection. Include at least one adjacent move from a diminished-family degree to a sixth-chord degree, then stop.

**Best diagnostic gesture:** alternate adjacent degrees of the eight-note collection. In scale-degree notation for the major collection, `1–2–3–4 | 3–2–1` makes the generated layer alternate:

`6th family → diminished family → 6th family → diminished family | 6th → diminished → 6th`.

This is an abstract exercise, not a quotation from a protected melody.

**Secondary gesture:** a 2–4-note chromatic enclosure only when every source pitch is visibly in the configured eight-note collection. In the major collection, the added `♭6` is available between `5` and `6`; many other chromatic pitches are outside the collection and are not a reliable input contract.

**Do not use a source chord.** One source note already requests a four-note voicing. Simultaneous source notes multiply those voicings rather than asking the preset to interpret one chord.

## 2. Register, tempo, length, articulation, dynamics, density, and silence

### Register

- **Keyboard source line:** approximately C4–C6, transposed with the retained tonic; begin around C4–C5. This leaves room for the voicer’s dropped second voice below the source without turning the result into low-register mud.
- **Guitar source line:** approximately D3–B4, favoring one string or a compact two-string area. Avoid the lowest string/register for the exercise because a four-note drop-2 result crowds the bass.
- Judge by the **generated chord’s** range, not the hand position alone. If the result booms or masks the lead, move the source up an octave; if it becomes brittle and detached from the line, move down.

### Tempo

- **Learning/default:** 96–120 BPM, swung eighths.
- **Useful live range:** 88–160 BPM.
- Above roughly 140 BPM, play the eighths less lopsided and make the swing audible through accents and phrase direction rather than a forced long-short ratio. This is generic jazz-performance guidance supported by tempo-dependent swing research, not a Harris-only rule. [Friberg & Sundström](https://doi.org/10.1525/mp.2002.19.3.333)

### Note length and articulation

- Gate notes to roughly **60–80% of the inter-onset interval**: connected enough to hear a line, released enough that adjacent four-note voicings do not smear.
- Use clean, distinct NoteOns with no physical overlap during the diagnostic exercise.
- Accent the intended arrival tone; make intervening diminished-family notes slightly lighter. Do not mechanically accent every other note—the musical target, not parity alone, determines the accent.
- End the phrase with a clean release rather than leaving sustain/pedal/string resonance active.

### Dynamics

- Start mezzo-piano to mezzo-forte, roughly MIDI velocity 60–90 where controllable.
- Make passing/diminished events slightly lighter and the selected sixth-family arrival clearer, while preserving an audible line.
- One modest phrase crescendo followed by a softer answer is enough. The preset does not generate a dynamic arc; the performer must shape it.

### Density and silence

- **4–8 source attacks per phrase**, usually eighth notes, no more than one active source note.
- Leave **1–2 beats after a short clause** and **one full bar after an 8-note phrase**.
- Begin with only one phrase per two bars. Increase density only after the listener can hear tension on diminished events and release on sixth-chord events.
- Silence is diagnostic: after release, no harmony should continue. It also prevents the four-note output from becoming a continuous chord block.

## 3. Transport and rhythmic precision

**Transport dependency: none.** The local implementation’s Barry Harris voicer accepts a beat phase but does not use it; every eligible NoteOn is harmonized immediately. The preset must set `transportRequired: false` and must not start, stop, reset, or retime the user’s transport. See `crates/contrapunk-harmony/src/barry_harris.rs` (`build_voicing`) and the product rule that presets preserve transport state in `.planning/phases/10.2-arrangement-presets/10.2-CONTEXT.md`.

**Precision still matters:** there is no quantizer, swing engine, phrase detector, or accompaniment pulse here. Every source attack creates four generated notes, so uneven attacks become uneven chord attacks. At 96–120 BPM, aim for stable eighth-note placement; use intentional rests, not accidental gaps. External transport/metronome is useful for practice but is not a capability requirement.

## 4. Phrase and section development

### Within one phrase

Use this player-shaped state arc:

`Stable sixth tone → 2–4 connected moves → diminished tension → clear sixth-family arrival → release`.

1. **Attack:** begin on a stable even-indexed collection degree (`1`, `3`, `5`, or `6` in the major sixth chord; analogous minor-sixth members in the minor collection).
2. **Continuation:** move mostly by adjacent collection degrees so the output visibly/audibly flips between sixth and diminished families.
3. **Tension:** let one diminished-family event speak for an eighth note; do not bury it under pedal or another input.
4. **Cadence:** arrive on an intended sixth-family tone, hold it for a quarter note, then release.

### Across a 4–8 bar section

- **Bars 1–2: establish.** One 4-note mid-register line, moderate dynamic, full-bar response gap.
- **Bars 3–4: vary.** Repeat the rhythm with one contour change or begin from another sixth-chord tone; do not merely transpose blindly outside the configured collection.
- **Bars 5–6: intensify.** Extend to 6–8 attacks, rise modestly in register/dynamic, and include one clearly audible diminished-to-sixth resolution.
- **Bars 7–8: withdraw.** Return to 3–4 attacks, lengthen the final stable tone, then leave a full bar silent.

The engine does not know bar or section state. This evolution is entirely performer-supplied.

## 5. Listening and responding

After each short clause, listen for three things before continuing:

1. **Family contrast:** stable source degrees should produce one inversion of the configured major-6/minor-6 sonority; intervening collection degrees should produce the related diminished-seventh family.
2. **Voice motion:** adjacent source degrees should create compact motion among the generated voices, not unrelated chord jumps. The exact implementation makes a parity-preserving four-note voicing and applies drop-2 on every event. See `crates/contrapunk-harmony/src/barry_harris.rs`.
3. **Resolution cue:** the diminished sonority should sound more mobile/tense; the following sixth sonority should sound comparatively settled. The Barry Harris Companion explicitly describes borrowed/diminished notes as asking to move and resolve. [Scales of Chords](https://barrycompanion.com/scales-of-chords/)

**Wait rule:** do not launch the next phrase until the previous source NoteOff has silenced its four generated notes and you have heard at least one beat of empty space. If a harmony remains, stop and Panic rather than layering over a lifecycle fault.

## 6. Clean-monophonic guitar constraints

- Use one picked/plucked note at a time with active left- and right-hand muting.
- Prefer one string for the first exercise; if crossing strings, mute the departed string before the next attack.
- No ringing open strings, double-stops, dyads, sustain devices, bends crossing semitone boundaries, slides that emit intermediate MIDI pitches, tremolo retriggers, or pitch-detection ambiguity.
- Pick/slur balance may shape generic bebop articulation, but each intended pitch needs one clean MIDI NoteOn and one NoteOff.
- Keep the audio input clean enough for stable pitch tracking; distortion, heavy compression, octave effects, chorus, and long reverb may cause false or overlapping note events even though the musical line sounds monophonic acoustically.
- Guitar cannot literally reproduce all piano-centered close voicings under the player’s fingers. That is acceptable here because guitar supplies only the monophonic controller line; Contrapunk supplies the fixed drop-2 harmony. Kingstone’s guitar adaptation is evidence that the structural system can be translated to guitar, not that arbitrary guitar gestures become Harris harmony. [Aebersold/Kingstone](https://www.jazzbooks.com/jazz/product/BHG)

## 7. Keyboard opportunities and chord-density limits

### Opportunities

- Keyboard gives exact attacks, reliable velocity, easy register testing, and clean legato without pitch-tracking ambiguity.
- Play the source line with one finger at a time and pedal up. Use touch and velocity to make the arrival note clearer than the diminished passing event.
- Try the same 4-note line one octave higher after a full rest to compare voicing clarity.
- Once the alternation is understood, a player may improvise a monophonic line through the displayed eight-note collection while preserving phrase gaps.

### Hard density limits

- **Exactly one physical source note at a time** for the built-in guidance.
- One source note produces four output notes. Two overlapping source notes can produce eight; a four-note keyboard chord can request sixteen. That is not a sophisticated block-chord interpretation; it is multiplied independent voicing.
- Sustain pedal must remain up for acceptance. Pedal obscures source ownership/release and makes diminished-to-sixth changes accumulate.
- Keep the source melody primarily C4–C6-equivalent. Avoid bass-octave source notes and octave doubling.

## 8. Failure gestures and recovery

| Failure gesture | Audible/result failure | Recovery |
|---|---|---|
| Holding sustain pedal or overlapping guitar strings | Chords accumulate into mud; parity change is obscured | Pedal up/mute; Panic once; retry one note at a time |
| Playing keyboard chords/double-stops | 8–16+ generated notes per attack; “note storm” | Return to one physical source note |
| Fast continuous scalar run with no rests | Mechanically alternating chord blocks, no phrase or resolution | Limit to 4–8 attacks, hold one target, leave a bar |
| Notes outside the selected BH collection | The Barry Harris builder returns no voicing for that note; harmonic layer may disappear rather than “fix” it | Use the displayed collection; treat unsupported chromaticism as outside the contract |
| Repeating only sixth-family degrees | Only inversions of the sixth chord; no conveyor contrast | Include an adjacent diminished-family degree and resolve |
| Repeating only diminished-family degrees | Continuous unresolved diminished color, stylistically one-sided | Move to an adjacent sixth-family tone and hold it |
| Very low source register | Dense low drop-2 voicings mask pitch and rhythm | Move source up one octave |
| Exaggerated shuffle at fast tempo | Generic caricature; lurching rather than flowing bebop line | Straighten eighths as tempo rises; retain accents/forward motion |
| Random chromatic input marketed as Harris language | Engine output is absent or incoherent; pedagogical claim becomes false | Choose tonic/major-or-minor collection deliberately and phrase inside it |

## 9. What the preset can and cannot manufacture

### It can

- Select BH Major 6th Diminished or BH Minor 6th Diminished under the retained tonic.
- Classify eligible collection notes by even/odd scale degree.
- Build a four-note same-parity sonority: sixth-chord family on even degrees, diminished-seventh family on odd degrees.
- Apply a fixed drop-2 transformation per note.
- Respond immediately and statelessly to a clean NoteOn/NoteOff stream.

Local evidence: `crates/contrapunk-harmony/src/config.rs` describes `HarmonyMode::BarryHarris` as “4-voice drop-2 voicings using 8-note BH scale”; `crates/contrapunk-harmony/src/barry_harris.rs` implements the parity rule, returns `None` for notes outside the scale, and contains no phrase, transport, dynamics, or harmonic-timeline logic.

### It cannot

- Infer the song, melody, words, chord changes, bass roots, cadence, or which sixth-chord scale belongs to each harmonic region.
- Generate Harris’s “borrowed notes,” “Family” dominants, “surrounding,” extra-note rules, scale maps, or melodic improvisation from arbitrary notes.
- Decide where tension should resolve, create a phrase/section arc, swing or quantize timing, vary articulation/dynamics, or listen/respond like an ensemble.
- Recreate Harris’s piano touch, voicing choices in a recorded performance, workshop improvisation, or whole bebop style.
- Turn every chromatic pitch into valid harmony. An out-of-collection pitch is explicitly not voiced by the implementation.
- Interpret a source chord as one harmonic object, perform automatic harmonic modulation, or choose major versus minor from performance context.

Harris’s own workshop tradition centers melody and song as well as movement; reducing it to an eight-note scale would be a caricature. [JazzTimes](https://www.jazztimes.com/features/profiles/inside-the-barry-harris-method/)

## 10. Proposed plain-language UI copy

### Result — concrete wording

> **Result:** Each single note in the selected sixth-diminished collection becomes a four-note drop-2 voicing: sixth-chord tones sound stable, and the notes between them become related diminished passing chords. Your line makes the movement and resolution audible.

### Short “Play it like” prompt

> **Play it like:** Play one clean, connected note at a time through the shown eight-note collection. Move through a diminished passing tone into a clear sixth-chord arrival, hold it briefly, then leave a full-bar rest.

### Expanded guidance

> Start around the middle of your instrument at 96–120 BPM. Play four to eight lightly swung eighth notes, one physical note at a time, with pedal up or unused strings muted. Mostly move to adjacent notes in the shown collection so you can hear the generated harmony alternate between the sixth chord and its related diminished chord. Make the diminished event lighter, hold the sixth-chord destination for a quarter note, release cleanly, and wait one full bar. The preset does not supply swing, phrases, chord changes, or Barry Harris improvisation rules; it only voices eligible notes in the current tonic and major/minor sixth-diminished collection.

### Structured `PresetPlayGuide` recommendation

- `input`: `single_notes`
- `articulation`: “Connected, lightly swung eighth notes with distinct attacks and clean releases; pedal up, no overlapping strings. Hold one sixth-chord destination for a quarter note.”
- `density`: “Four to eight source attacks per phrase; exactly one physical source note at a time.”
- `space`: “Rest one to two beats after a clause and one full bar after each phrase; wait for all generated notes to release.”
- `tempo`: “88–160 BPM; 108 BPM test default. Straighten the eighths as tempo rises.”
- `transportRequired`: `false`

### Approximation disclosure

> A fixed live drop-2 harmonization of the current Barry Harris major- or minor-sixth-diminished collection. It alternates sixth and diminished families from your eligible source notes; it does not infer a song, chord progression, bass, borrowed-note movement, improvisational grammar, swing, phrase form, or Barry Harris’s touch. Notes outside the selected collection are not guaranteed a generated voicing.

## 11. Lifecycle and surface expectations

- Apply must preserve tonic, BPM, time signature, input device/configuration, routing, sound, master/mix environment, and whether transport is running, per `.planning/phases/10.2-arrangement-presets/10.2-CONTEXT.md`.
- This preset requires only the existing `harmony` capability and must be capability-gated on any surface that does not expose the Barry Harris harmony mode and BH scale choices truthfully.
- No Companion lane, queue, delay, phrase capture, Hold tail, or transport scheduling is needed.
- Each eligible source NoteOn should create exactly one four-note generated voicing; its matching source NoteOff should release that voicing. After the prescribed silent bar, active and pending generated-note counts must be zero.
- An out-of-collection NoteOn must not leave orphan notes when no voicing is built.
- Preset replacement and Panic must clear every active generated note exactly once. Reapplication must not duplicate voices or retain prior major/minor collection state unintentionally.
- Guidance and approximation text should remain visible after Apply. Built-in remains immutable; Save As/Duplicate is the customization route.

## 12. 30–60 second acceptance exercise

**Duration:** approximately 45 seconds. **Setup:** select a comfortable tonic, choose BH Major 6th Diminished, 108 BPM metronome if desired, pedal up, one source note at a time. Transport may remain stopped.

1. **0–8 s — establish:** play scale degrees `1–2–3–2–1` as lightly swung eighths; hold final `1` for one beat; release. Listen for `sixth → diminished → sixth → diminished → sixth`. Rest one bar.
2. **8–18 s — expose the added color:** play `5–♭6–6`, one eighth each, then hold `6` for a quarter note. Listen for the middle diminished-family voicing to press forward into the stable sixth-family arrival. Rest one bar.
3. **18–30 s — phrase:** play any original 6–8-note monophonic line using only displayed collection notes, mostly adjacent, ending on a sixth-family degree. Increase velocity slightly toward the last two notes. Rest one full bar.
4. **30–38 s — register check:** repeat the first five-note gesture one octave higher. Confirm the family alternation remains clear and the low dropped voice no longer muddies the line.
5. **38–45 s — lifecycle check:** release completely and remain silent. Confirm no harmony continues. If any note remains, use Panic and record a lifecycle failure.

**Pass by ear:**

- every eligible source attack produces one coherent four-note chord, not multiple retriggers;
- adjacent collection degrees audibly flip between sixth and diminished sonorities;
- the held destination sounds more settled than the preceding diminished event;
- source rhythm, accents, and rests remain audible through the generated layer;
- no generated note survives the final release/silent bar.

**Fail:** no family contrast, low-register mud, more than one voicing per intended attack, harmony on unsupported chromatic notes being presented as guaranteed behavior, or any stuck/pending note after silence.

## Confidence and gaps

- **High confidence:** Harris pedagogy treats sixth-diminished material as harmonic/linear movement; pairing of sixth and diminished families; performer must supply melody and resolution. Supported by the Barry Harris Institute, Howard Rees materials, first-hand workshop reporting, and the local parity implementation.
- **High confidence:** current implementation is fixed four-note drop-2, stateless, transport-independent, and silent on out-of-collection pitches. Directly supported by `crates/contrapunk-harmony/src/config.rs` and `crates/contrapunk-harmony/src/barry_harris.rs`.
- **Medium confidence:** proposed 88–160 BPM range, 60–80% gates, and exact phrase lengths. These are practical HCI defaults informed by generic bebop practice and swing research, not prescriptive Harris rules.
- **Medium confidence:** register bands because the resulting MIDI destination/timbre varies by surface and user routing. They require hands-on tuning with the shipped sound.
- **Gap:** accessible primary footage with stable timestamps demonstrating Harris’s exact articulation and sixth-diminished keyboard movement was not available in this pass. Do not claim a precise Harris touch model from the cited curriculum descriptions.
- **Gap:** the final preset’s exact scale choice (major or minor), retained-tonic display, input-note passthrough/mix level, and out-of-scale UX need synthesis/implementation confirmation.

## Review findings and residual risks

1. **High — `ui/src/lib/arrangement/catalog.ts`, preset 23 draft:** current prompt says “chromatic approaches” without warning that `crates/contrapunk-harmony/src/barry_harris.rs` returns no voicing for notes outside the selected eight-note collection. Replace with the bounded wording above or explicitly say “approaches within the shown collection.”
2. **High — activation contract:** a truthful operational config must use `HarmonyMode::BarryHarris` with `BHMajor6thDim` or `BHMinor6thDim`; generic `DiatonicThirds` would not produce the promised alternating sixth/diminished families.
3. **Medium — HCI density:** keyboard chord input and overlapping guitar tracking can multiply four-note outputs into note storms. Built-in guidance should remain `single_notes`, pedal-up, clean-monophonic.
4. **Medium — stylistic overclaim:** a fixed parity/drop-2 voicer is only one harmonic mechanism, not Harris’s improvisational method or performance style. Keep the approximation disclosure visible after Apply.
5. **Medium — lifecycle:** because an unsupported input may build no voicing, tests should pair eligible and ineligible NoteOn/NoteOff events and assert zero active/pending notes after release, Panic, and replacement.

Residual risk remains that a technically correct fixed voicer will sound mechanical without player-shaped rhythm, targets, dynamics, register, and silence. A user play-test on both clean guitar tracking and velocity-sensitive keyboard is required before approval.

## Sources

### Kept

- [Barry Harris Institute of Jazz — Spring 2026 Six-Part Zoom Series](https://barryharrisinstituteofjazz.org/spring-2026-a-six-part-zoom-series/) — official custodian curriculum; directly names sixth chords, sixth-diminished scales, movement, borrowing, melody harmonization, and separate improvisation concepts.
- [Barry Harris Institute of Jazz — Workshops](https://barryharrisinstituteofjazz.org/workshops/) — official description of harmony as linear movement and workshop instrument prerequisites.
- [Howard Rees’ Jazz Workshops — *The Barry Harris Workshop Video*](https://jazzworkshops.com/the-barry-harris-workshop-video/) — materials developed by Harris with his long-term student/producer; credible evidence of movement-oriented pedagogy and theory-to-performance intent.
- [JazzTimes — “Inside the Barry Harris Method”](https://www.jazztimes.com/features/profiles/inside-the-barry-harris-method/) — first-hand workshop report and direct Harris remarks; supports melody/song/register/movement framing.
- [Barry Harris Companion — “Scales of Chords”](https://barrycompanion.com/scales-of-chords/) — secondary but focused pedagogical explanation of sixth/diminished pairing, borrowing, tension, and resolution.
- [Jamey Aebersold — Alan Kingstone, *The Barry Harris Harmonic Method for Guitar*](https://www.jazzbooks.com/jazz/product/BHG) — reputable published guitar adaptation; useful for the structural-not-instant-style caution.
- [Friberg & Sundström 2002](https://doi.org/10.1525/mp.2002.19.3.333) and [Butterfield 2011](https://doi.org/10.1525/mts.2011.33.1.3) — peer-reviewed support for treating swing as tempo- and performer-dependent rather than a fixed triplet ratio.
- `crates/contrapunk-harmony/src/config.rs` and `crates/contrapunk-harmony/src/barry_harris.rs` — primary implementation evidence for the actual product behavior and limits.

### Dropped

- Reddit, Stack Exchange, SEO guitar blogs, and unsourced lesson summaries — excluded where stronger official, published, first-hand, or direct-code evidence was available.
- Smithsonian oral-history transcript — authoritative for biography, but not needed because accessible evidence in this pass did not yield a sufficiently precise sixth-diminished performance passage.
- Commercial testimonials — not used as proof of musical details; only the Rees/Harris material’s documented pedagogical scope was retained.
