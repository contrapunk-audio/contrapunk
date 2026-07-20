# Music-Theory / Temporal-Behavior Research — Preset 43 “Hollow Choir”

**Role:** independent music-theory and temporal-behavior report
**Phase:** 10.2 Composer-Informed Arrangement Presets
**Reference:** Christopher Larkin, bounded to selected music from *Hollow Knight* (2017)
**Decision:** **bounded static mapping is defensible; the full catalog promise is not**
**Confidence:** medium overall; high for documented instrumentation and engine behavior, medium for listening-derived theory, low for claims requiring an unavailable authorized score

## 1. Scope, corpus, method, and claim boundary

This report does **not** characterize Christopher Larkin’s whole output and does not use *Silksong* to backfill claims about *Hollow Knight*. The bounded corpus is the official 2017 soundtrack, with close analytical listening centered on **“City of Tears,” “Soul Sanctum,” “Resting Grounds,” “White Palace,” “Hollow Knight,” and “Sealed Vessel.”** The official album identifies Larkin as composer and performer, Timothy Cheel on viola, and Amelia Jones as soprano specifically on “City of Tears.” [Official soundtrack and credits](https://christopherlarkin.bandcamp.com/album/hollow-knight-original-soundtrack)

The corpus is chosen to test the preset brief’s intersection—dark minor/modal melody, sustained vocal/chorale color, distant registers, counterline, and changing game texture—not to imply that all six tracks use choir or the same harmonic system. Larkin has described the project brief as “dark elegance and melancholy,” while interviews identify piano and viola as central colors and discuss themes/leitmotifs and gameplay-sensitive scoring. [Bandcamp Daily interview](https://daily.bandcamp.com/features/christopher-larkin-review) [2017 soundtrack interview](https://www.youtube.com/watch?v=cB4zw0DgVbk) [Game Developer interview](https://www.gamedeveloper.com/audio/crafting-an-evocative-score-for-i-hollow-knight-i-)

**Method boundary.** Tonal, textural, registral, and formal observations below are listening-derived comparative analysis of the official recordings. No protected melody is transcribed. Because no composer-authorized full score was found, exact chord labels, bar numbers, and claims of systematic Aeolian usage remain interpretations rather than primary-source facts.

## 2. Summary thesis

The bounded corpus supports an arrangement identity built from **minor/modal gravity, predominantly diatonic consonant sonorities, restrained stepwise voice motion, sustained attacks, wide registral air, and consequential silence**. Identity lies less in an exotic scale than in temporal restraint: a lone or thin line establishes space, another layer enters without crowding it, intensity grows through register/density/dynamics, and cadential or scene boundaries thin back toward resonance and silence.

`Aeolian + BachChorale + four voices + fixed input position` can therefore make one honest claim: **a reactive, four-note, minor-diatonic SATB-style chordal shadow for a monophonic line**. It cannot honestly claim a choir timbre, a distant choir, an independent counterline, adaptive game scenes, Larkin’s orchestration, phrase-aware entrances, or reproduction of the corpus’s actual harmonic trajectory.

## 3. Theoretical and temporal findings

### 3.1 Tonal center, modality, chromaticism, and tuning

1. **Darkness is center-and-context, not “Aeolian everywhere.”** The selected cues repeatedly sustain a minor or modal center and avoid constant bright dominant closure, but the corpus also admits functional pull, modal ambiguity, pedal tones, and local chromatic color. Aeolian `[1, 2, ♭3, 4, 5, ♭6, ♭7]` is a useful preset boundary, not a finding that every cue or passage uses natural minor.
2. **Center is reinforced by recurrence and pedal-like stability.** Long returns to a pitch or harmony matter more than rapid progressions. Avoid changing harmony on every ornamental note.
3. **Chromaticism is color or directed tension, not saturation.** The static baseline should leave modal interchange off. A later timeline may admit occasional raised leading tones or modal shifts, but indiscriminate chromatic clusters would misstate this corpus.
4. **Tuning is ordinary 12-tone equal-tempered production for this implementation claim.** No source supports microtonality as an invariant here.

**Confidence:** medium for minor/modal gravity; low-to-medium for cue-specific modal labels without scores.

### 3.2 Interval and chord vocabulary

- Favor minor/major thirds, sixths, perfect fourths/fifths, octaves, and complete or nearly complete triadic/seventh-note collections.
- Allow seconds and sevenths as controlled inner tension, suspensive color, or melody-against-chord nonchord tones; do not make seconds the default vertical identity.
- Prefer mixed spacing: compact upper voices over a more separated low voice, or a remote upper line over a middle bed. Constant close-position block chords are too dense; mechanically expanding every voice by another octave is too synthetic.
- Doubling should reinforce stable chord tones, ordinarily the root, and avoid multiplying the leading tone. Tension should tend toward stepwise resolution or simply dissipate into a held resonance/rest.
- Aeolian permits i, ii°, ♭III, iv, v, ♭VI, and ♭VII collections. That palette is defensible as a **dark diatonic approximation**, but a fixed natural-minor palette cannot assert the recordings’ exact chord vocabulary.

The engine’s Bach voicer is unusually relevant at this narrow level: it fixes four SATB outputs, avoids doubled leading tone and parallel perfect fifths/octaves when a valid candidate exists, limits soprano–alto and alto–tenor spacing to an octave in the strict search, rewards common tones, stepwise motion, and soprano/bass contrary motion, and uses a wider possible bass–tenor gap. [Current Bach-chorale implementation](https://github.com/contrapunk-audio/contrapunk/blob/main/crates/contrapunk-harmony/src/functional/voicer_bach.rs)

### 3.3 Melodic contour, motive, repetition, and ornament

- Source melodies suitable for the approximation should be singable: short arcs, stepwise spans, occasional expressive thirds/fourths, and held destinations.
- Repetition should preserve a recognizable contour while varying endpoint, register, or duration. Exact copied themes are neither needed nor allowed.
- Ornament is sparse. Passing or neighbor notes work only when the harmony does not lurch after each one.
- A large leap should be treated as an expressive event and followed by stabilization, continuation in the new register, or a rest—not a trigger for three more competing leaps.

Larkin explicitly discusses distinguishable themes/leitmotifs as part of the scoring challenge; that supports recurrence as a compositional concern but does **not** grant the current preset motif memory. [Game Developer](https://www.gamedeveloper.com/audio/crafting-an-evocative-score-for-i-hollow-knight-i-)

### 3.4 Voice motion and independence

- Within chordal passages, retain common tones and favor semitone/whole-tone motion in inner parts.
- Contrary and oblique motion should outweigh continuous same-direction planing; parallel thirds/sixths can occur locally but should not be the only behavior.
- Outer voices define spaciousness: stable/slow bass against an arcing upper voice is preferable to four equally busy lines.
- An actual **counterline** must have a distinct contour and, at times, a distinct onset/rhythm. Four pitches recomputed simultaneously from every input onset are vertical voice leading, not independent counterpoint.
- Imitation is not established as a universal invariant of this corpus and should not be fabricated merely because the product is named Contrapunk.

### 3.5 Rhythm, meter, subdivision, and articulation

The bounded identity is not tied to swing, polymeter, or a specific notated meter. Useful behavior is slow-to-moderate harmonic rhythm, long values, few attacks, and phrase-shaped rubato or broad pulse. Attacks should be soft or rounded; releases should be long enough to imply resonance but must still obey deterministic NoteOff ownership. Hard repeated staccato, constant sixteenth-note subdivision, and accent grids contradict this mapping.

### 3.6 Register, texture, orchestration, and layer interaction

The corpus’s documented palette is not “full choir”: piano and viola are central, while the official credits name **one soprano on one track, “City of Tears.”** [Bandcamp credits](https://christopherlarkin.bandcamp.com/album/hollow-knight-original-soundtrack) Larkin’s own interview describes piano and viola as centerpiece colors. [Soundtrack interview](https://www.youtube.com/watch?v=cB4zw0DgVbk)

Accordingly:

- **Texture:** solo/duo foreground, sustained bed, selectively enlarged ensemble; not four continuously equal singers.
- **Register:** preserve air between low foundation, middle support, and high/vocal color. “Distant” also implies level, envelope, filtering, and reverberant depth—not pitch height alone.
- **Interaction:** accompaniment should wait, sustain, or thin around the lead. It should not answer every note with a new attack.
- **Orchestration:** choir, solo soprano, piano, viola, synthetic pad, and orchestral strings are distinct claims. Arrangement metadata cannot silently turn a generic synth into any of them.

### 3.7 Dynamics, density, silence, and cadence

- Begin around `p–mp`; build mainly by adding a layer, widening register, increasing sustain, or modestly raising intensity.
- Keep ordinary sounding density at one lead plus one-to-three supporting notes. A peak may use four simultaneous parts; it should not be the unvarying floor.
- Silence is structural. A phrase-end rest of roughly one to two slow beats, and longer gaps between larger statements, is more characteristic of this approximation than continuous generated fill.
- Cadences may settle by step, common-tone retention, return to center, a held unresolved modal sonority, or disappearance into reverb. Do not require a V–i cadence at every phrase.

### 3.8 Within-phrase, across-section, and game evolution

**Researched target state sketch:**

`Empty/resonance → solitary line → soft harmonic shadow → upper/inner counterline → registral/dynamic peak → held destination → thinning tail → silence`

Suggested triggers for a future implementation:

- first clean note after ≥2 beats silence: solitary or very thin state;
- second coherent gesture / longer held destination: admit soft chorale;
- rising velocity, register, repetition, or phrase duration: add counterline and widen register;
- sustained high/long destination: density peak, never a note storm;
- release plus phrase gap: remove counterline first, then harmony, leaving tail;
- prolonged inactivity: return to Empty.

**Across sections/game:** interviews support that the soundtrack was written to guide emotion, establish areas/characters, and respond to gameplay context; this is evidence for distinct cue/layer states, not evidence that a note-reactive harmonizer already implements them. [Game Developer](https://www.gamedeveloper.com/audio/crafting-an-evocative-score-for-i-hollow-knight-i-) [Polygon interview](https://www.polygon.com/2024/8/8/24215602/christopher-larkin-hollow-knight-interview/) The current preset has no scene/intensity model, no knowledge of location/combat, and no phrase detector. Any adaptive claim must remain gated.

**Career/period boundary:** these rules belong only to the 2017 *Hollow Knight* selection. They must not be generalized to Larkin’s other projects or later *Silksong* writing.

## 4. Testable stylistic invariants

1. A stable minor/modal center is audible; the default pitch set is Aeolian and modal interchange is off.
2. The input remains the perceptual lead and is never replaced by a generated melody.
3. Vertical support is consonance-led and voice-led: common tones and stepwise/oblique/contrary motion are preferred over continuous parallel blocks.
4. Ordinary density is sparse; four-part fullness is a destination, not evidence of an actual choir.
5. Registral air and slow attack/release matter; low-mid congestion is rejected.
6. Phrase-end held tones and rests are allowed to remain empty; the system must not auto-fill silence.
7. Every generated NoteOn has a matching owned NoteOff; Panic/replacement leaves zero active or pending notes.
8. No preset text claims to reproduce a protected melody, a literal choir, or gameplay adaptation.

## 5. Parameters that may vary without losing identity

- tonic: any of 12 pitch classes;
- tempo: broadly slow to moderate (performer-led; approximately 45–90 BPM is practical guidance, not a corpus statistic);
- input position: **Soprano is the approved static default**; Alto may be a user variation if tested, but it weakens the literal current Bach-voicer assumption that input is initially soprano;
- phrase length: roughly 2–8 notes, provided held endpoints and rests remain;
- chord degree/inversion selected by the shared reactive scorer;
- occasional local thirds/sixths, fourths/fifths, or unresolved added-seventh color;
- intensity from very sparse `p` statements to a restrained four-part `mf` peak;
- sound preset chosen separately, provided it has rounded attacks and a long but controlled tail.

## 6. Misleading/rejection behaviors

Reject or gate the result if any of the following is presented as part of “Hollow Choir”:

- “authentic Christopher Larkin harmony,” “the Hollow Knight scale,” or exact soundtrack emulation;
- treating the solo soprano credit on “City of Tears” as evidence of a four-part choir throughout the score;
- bright, automatic common-practice cadences on every phrase;
- block-chord retriggering on every fast passing note;
- constant full density, octave-mirror multiplication, bass mud, voice crossing, or a high-register note spray;
- random-below harmony, parallel-third stacking, or free chromatic clusters as the default;
- calling simultaneous reactive chord members an “independent counterline”;
- calling octave displacement “distance” without timbral/depth controls;
- claiming adaptive scene or combat/exploration behavior without a Sense-phase intensity model and scene state;
- generating notes during performer silence, retaining notes after source release, or leaving queued events after Panic/preset replacement.

## 7. Mapping audit: HarmonyEngine and ArrangementPresetV2

### 7.1 Approved bounded static mapping

| Field | Approved value | What it can honestly claim |
|---|---:|---|
| `scale_mode` | `Aeolian` | natural-minor/modal-dark pitch boundary only |
| `harmony_mode` | `BachChorale` | reactive four-note SATB-style chordal voicing |
| `voice_count` | `4` | requested fixed density; Bach mode itself returns four notes |
| `voice_position` | `0` / Soprano | player is the upper lead; matches the Bach voicer’s native assumption |
| voice leading | Bach mode’s internal SATB voicer; external VL should not be advertised as extra independence | common-tone/stepwise preferences, no parallel P5/P8 in successful strict/relaxed search |
| `octave_mode` | `None` | preserves the voicer’s register logic; avoids synthetic mirror/spread multiplication |
| `octave_intensity` | `0` or irrelevant with `None` | no unsupported “distant” claim |
| interchange | `false` | bounded diatonic Aeolian claim |
| species | irrelevant to `BachChorale` | no species-counterpoint claim |
| Companion | disabled for the baseline | no generated counterline or adaptive timing claim |
| Hold | source-owned/default release, not Forever | deterministic cleanup |
| base mix | harmony below input, conservatively | supports “shadow,” not literal orchestration |
| suggested sound | optional and separately disclosed | may recommend vocal/pad depth; must not silently load it |

The current engine’s `BachChorale` path always calls the four-voice SATB allocator, then redistributes around `voice_position`; it is reactive per input onset and its harmonic-rhythm governor uses **note count**, not transport time. The first note chooses a chord, then at least two processed notes are required before another change is permitted. [Functional harmony context](https://github.com/contrapunk-audio/contrapunk/blob/main/crates/contrapunk-harmony/src/functional/context.rs) [Functional modes](https://github.com/contrapunk-audio/contrapunk/blob/main/crates/contrapunk-harmony/src/functional/mod.rs)

`ArrangementPresetV2` can store the scale/harmony/voice/position/octave/interchange/species snapshot, Companion lanes and Hold, base gains, guidance, capabilities, and a suggested sound ID. That contract is sufficient to serialize the bounded static mapping and sufficient to **declare missing capabilities**. Metadata does not create choir synthesis, depth, phrase analysis, counterline generation, or scenes.

### 7.2 Why alternatives are weaker

- `DiatonicThirds`: too mechanically parallel and can stack seventh chords from each input; weak voice independence.
- `DiatonicFourths`: useful for quartal presets, not the primarily chorale-led bounded claim here.
- `StrictCounterpoint`: produces a note-against-note line rather than the requested chordal expansion and does not create choir/register groups.
- `FunctionalHarmony`: permits variable voice count and generic near-placement; it is less defensible than the constrained Bach voicer for a chorale shadow.
- `RandomBelow*` / `ContraryMotion`: either stochastic or only line-directional; neither establishes the intended sustained four-part field.
- `Mirror` or full `Spread`: can multiply or displace voices, but does not make them distant in depth and may destroy the SATB spacing rationale.

### 7.3 Unsupported claims and exact capability gaps

| Desired catalog claim | Status | Missing capability |
|---|---|---|
| “dark minor melody” | supported, bounded | Aeolian plus player contract |
| “expands into chorale” | partially supported | four-part reactive chord exists; no gradual/phrase-aware expansion |
| literal choir | unsupported | vocal/choir sound source and disclosed sound-preset application |
| distant choir/register groups | unsupported | stable mix/register groups, per-group output/depth/envelope/reverb control |
| independent counterline | unsupported | phrase-aware counterline Lane with separate rhythm, contour, ownership, and mix role |
| adaptive exploration/build/combat evolution | unsupported | density/velocity/register/silence sensing plus Sparse/Building/Full scenes |
| Larkin orchestration | unsupported | role-aware piano/viola/soprano instrumentation and authored orchestration trajectory |
| cue-specific formal evolution | unsupported | phrase/section memory, event triggers, layer entrances/exits, harmonic timeline |
| authentic soundtrack harmony | unsupported and inappropriate | score-grounded cue model; even then UI must say approximation |

## 8. Abstract acceptance examples (new material only)

Notation: scale degrees are relative to an arbitrary tonic `1`; octave marks indicate register. For a concrete engine oracle, set tonic C only to turn the abstract relations into MIDI numbers. These examples test the **approved static mapping**, not a melody from the game.

### Example A — first sustained tonic, exact pitch/lifecycle oracle

**Config:** C Aeolian, `BachChorale`, 4 voices, player=Soprano (`voice_position=0`), `OctaveMode::None`, interchange off, fresh engine state.
**Input:** beat 1 `NoteOn C5(72), velocity 54`; hold through beat 4; then `NoteOff C5(72)`.

**Expected NoteOn:** exactly four notes, input first: **`[72, 63, 55, 46]` = `[C5, E♭4, G3, B♭2]`**, an abstract `1′ / ♭3 / 5 / ♭7` i7 field. This is a test oracle for the current fixed-PC assignment and first-voicing register-center scoring, not a required artistic voicing for all implementations.

**Lifecycle:** no additional note may be emitted merely because beats 2–4 elapse. On source NoteOff, expected releases are the same four note identities (order may remain input first): **`[72, 63, 55, 46]`**. Afterwards `active_notes=0`, no pending releases, and a second unmatched NoteOff returns/releases only the source note without inventing harmony.

### Example B — nonchord continuation tests harmonic inertia, exact oracle

**Config:** continue directly from Example A without reset and before its source is released only if the surface supports held-source reharmonization safely; otherwise replay A, release it, and preserve harmonic context.
**Input:** next onset `NoteOn D5(74)` (abstract degree `2′`) while the note-count governor has not yet permitted a chord change.

**Expected NoteOn:** current i harmony is retained; because degree 2 is not a chord member, it remains the soprano color above the first three i chord tones. Exact current oracle: **`[74, 60, 51, 43]` = `[D5, C4, E♭3, G2]`**. This verifies “passing color without a new chord on every note.” On `NoteOff D5`, release exactly those four identities and leave no D-owned notes active.

### Example C — abstract phrase behavior the baseline may require from the player, not manufacture

**Input over 8 slow beats:** `1′(beats 1–2) → ♭3′(3) → 2′(4) → 5′(5–7) → rest(8–10)` with soft velocity and legato overlap no greater than one transition.

**Accepted relationships:** each onset returns exactly four notes; input is first and perceptually highest; generated PCs belong to the selected Aeolian diatonic chord except the preserved input may act as a nonchord color; successive generated parts prefer retained tones or small moves and must not produce detected parallel P5/P8 where the voicer found a constrained candidate. The held degree 5 creates no autonomous counter-melody. During the final rest all source-owned chord notes receive NoteOff and the engine reaches zero active/pending notes.

### Example D — rejection/lifecycle stress

**Input:** eight 1/16-note repetitions alternating `1′` and `2′`, then Panic during the fourth onset.
**Expected:** this gesture is **stylistically rejected** even if technically processable; it must not be described as choir-like. Runtime acceptance still requires one owned chord per accepted source NoteOn, no unbounded queue growth, one transactional Panic, immediate clearing of all four-note ownership sets, and no later delayed NoteOn.

**Oracle caveat:** exact MIDI arrays A/B depend on the current implementation’s chord table, fixed remaining-PC ordering, scorer defaults, and relaxation cascade. They should be pinned as regression tests only if the baseline adopts this precise mapping; a future improved voicer may intentionally change them while retaining the relational invariants.

## 9. Competing interpretations

1. **Aeolian vs broader minor/modal mixture.** Aeolian is the smallest truthful baseline. Harmonic minor could intensify cadences, but asserting it as the default overstates raised-leading-tone behavior; Dorian/Phrygian would similarly make one modal color universal.
2. **BachChorale vs StrictCounterpoint.** “Chorale” favors the former; “counterline” favors the latter. One shared mode cannot honestly do both simultaneously. For baseline activation, retain the chorale and explicitly drop the independent-counterline clause.
3. **Four voices vs sparse identity.** Four outputs satisfy the vertical chorale claim but risk making peak density constant. Lower harmony gain and performer rests mitigate this; true staged density requires a later scene/lane capability.
4. **“Choir” as imagery vs instrumentation.** The title can remain imagery if Result text says “minor SATB-style harmonic shadow” and the UI discloses that sound design is separate. Literal choir language is unsupported.

## 10. Recommendation

**Approve for baseline implementation only with narrowed copy**, for example:

> **Result:** A singable Aeolian line receives a restrained four-part, SATB-style minor harmonic shadow. Sound design is separate; this does not supply a literal choir, independent counterline, or adaptive game scene.

The original catalog sentence—“dark minor melody expands into distant chorale and counterline”—is too broad for current capabilities. “Expands” is only onset-reactive chord generation; “distant” needs stable register/depth groups; “counterline” needs independent temporal behavior. Keep those clauses capability-gated for the stable-groups/adaptive-scenes work.

## 11. Sources

### Kept

- [Christopher Larkin — *Hollow Knight (Original Soundtrack)*](https://christopherlarkin.bandcamp.com/album/hollow-knight-original-soundtrack) — official corpus, track durations, release and performer credits; primary evidence that the credited voice is soprano on “City of Tears,” not a blanket choir credit.
- [Christopher Larkin, “Hollow Knight Soundtrack Interview”](https://www.youtube.com/watch?v=cB4zw0DgVbk) — composer interview; primary evidence for piano/viola focus, themes, and gameplay relationship.
- [Game Developer, “Crafting an evocative score for Hollow Knight”](https://www.gamedeveloper.com/audio/crafting-an-evocative-score-for-i-hollow-knight-i-) — direct composer discussion of evocative scoring, leitmotifs, world/character functions, and interactive context.
- [Bandcamp Daily, “Inside Christopher Larkin’s Darkly Elegant ‘Hollow Knight’ Score”](https://daily.bandcamp.com/features/christopher-larkin-review) — direct interview and project brief (“dark elegance and melancholy”), plus bounded aesthetic context.
- [Polygon, “Hollow Knight’s composer talks musical inspirations”](https://www.polygon.com/2024/8/8/24215602/christopher-larkin-hollow-knight-interview/) — later direct interview useful for stated influences and cue/world intent; used cautiously, not as score-level proof.
- [Contrapunk `functional/mod.rs`](https://github.com/contrapunk-audio/contrapunk/blob/main/crates/contrapunk-harmony/src/functional/mod.rs) — shared reactive FunctionalHarmony/BachChorale behavior and fixed four-voice return.
- [Contrapunk `functional/context.rs`](https://github.com/contrapunk-audio/contrapunk/blob/main/crates/contrapunk-harmony/src/functional/context.rs) — note-count harmonic rhythm, cadence state, and reset behavior.
- [Contrapunk `functional/voicer_bach.rs`](https://github.com/contrapunk-audio/contrapunk/blob/main/crates/contrapunk-harmony/src/functional/voicer_bach.rs) — SATB ranges, spacing, leading-tone, parallel-perfect, movement, and relaxation rules.
- [Contrapunk `engine.rs`](https://github.com/contrapunk-audio/contrapunk/blob/main/crates/contrapunk-harmony/src/engine.rs) — voice-position redistribution, octave processing, active-note ownership, matching NoteOff, reharmonization, and Panic cleanup behavior.

### Dropped or not used as evidence

- **Hollow Knight Wiki soundtrack page** — useful index but secondary/fan-maintained; official Bandcamp supplies the needed corpus and credits.
- **VGMdb release listing** — redundant third-party metadata once official credits were available.
- **COGconnected “Hollow Sounds”** — commentary with insufficient score-level or primary evidence for the exact implementation claims.
- **Tumblr leitmotif analysis** — informal fan analysis; not needed and too weak for operational invariants.
- **Generic piano tutorials, MIDI transcriptions, fan scores, YouTube harmonic reductions, and SEO “music theory” pages** — provenance/authorization and analytical reliability unclear; excluded to avoid melody copying and false precision.
- ***Hollow Knight: Silksong* soundtrack/“The Choir”** — outside the bounded 2017 corpus and a later project; excluded from style evidence.

## 12. Gaps and confidence ledger

| Claim | Confidence | Reason / next step |
|---|---|---|
| dark elegance/melancholy was the project brief | high | direct composer quotation in Bandcamp Daily |
| piano/viola central; one credited soprano on “City of Tears” | high | composer interview and official credits |
| bounded corpus favors minor/modal gravity, space, sustain, and restrained layering | medium | repeatable listening analysis, but no authorized score |
| Aeolian is the best single baseline scale | medium | defensible reduction, not universal corpus fact |
| current BachChorale makes four reactive SATB-style notes | high | code audit |
| exact example arrays under the stated fresh-state config | medium-high | directly derived from current algorithm; should be confirmed by one executable regression before synthesis freezes them |
| independent counterline, distant choir, and adaptive scenes are absent | high | current shared engine/preset contract has no such temporal/depth machinery |
| cue-level chord labels, meter, and formal bar counts | low/not claimed | require authorized scores or a documented cue-by-cue transcription protocol |

**Next evidence step:** if exact theoretical fidelity rather than a bounded approximation is required, obtain composer-authorized scores/stems or a direct technical interview addressing modality, harmonic rhythm, layer switching, and vocal orchestration. Otherwise the narrowed static mapping is sufficient and avoids speculative extra machinery.
