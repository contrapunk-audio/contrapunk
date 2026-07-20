# Research: Phase 10.2 Preset 48 — “Crystal Chorale”

**Role:** Independent music-theory / temporal-behavior report  
**Reference:** Nobuo Uematsu, bounded Final Fantasy corpus  
**Decision:** **Operational only as a bounded static/reactive approximation.** The present engine can make a four-part minor-key chorale plus one delayed octave answer with deterministic ownership and cleanup. It cannot honestly claim learned motifs, four-/eight-bar form, adaptive growth, orchestration, or game-state evolution.

## Summary

The defensible identity is not “Uematsu equals harmonic minor.” In a bounded corpus—*Final Fantasy IV*’s lyrical/theme writing, the *Final Fantasy VI* opera sequence, and the sectional *Final Fantasy VII Main Theme*—the stronger intersection is a plainly profiled, repeatable melody set against tonal/modal harmony, with contrast and recurrence carrying narrative memory. Scholarship supports Uematsu’s sophisticated linear writing, thematic networking, and large-scale leitmotivic/narrative organization; it does **not** establish octave answers or harmonic minor as universal fingerprints. [Comeaux, *An Analysis of Nobuo Uematsu’s Linear Structures*](https://aquila.usm.edu/masters_theses/822) [Gibbons, “Leitmotivic Strategies”](https://doi.org/10.1093/mts/mtad009) [“Kishōtenketsu as Leitmotif”](https://doi.org/10.1525/jsmg.2023.4.4.15)

Accordingly, “Crystal Chorale” should be described as **Uematsu-informed**: the player supplies a memorable, breath-shaped minor melody; Contrapunk supplies reactive SATB-style chordal support and one delayed octave echo. Harmonic minor is a selectable palette for strong leading-tone cadences, not a career-wide style rule.

## 1. Scope and corpus

### Selected corpus

1. **Final Fantasy IV (1991):** lyrical, songlike thematic practice, especially material commonly represented by “Theme of Love,” plus the franchise “Prelude/Prologue” vocabulary as context.
2. **Final Fantasy VI (1994):** the opera sequence, especially “Aria di Mezzo Carattere,” because it is the best-supported bounded source for tonal voice leading, phrase hierarchy, and theatrical contrast. Comeaux explicitly treats the opera score through linear/contrapuntal analysis. [Comeaux](https://aquila.usm.edu/masters_theses/822)
3. **Final Fantasy VII (1997):** “Main Theme of Final Fantasy VII” as a model of memorable thematic statement embedded in contrasting sections, not as a source to reproduce. The scholarly discussion of FFVII connects leitmotif with a four-part narrative process rather than a static harmonic recipe. [“Kishōtenketsu as Leitmotif”](https://doi.org/10.1525/jsmg.2023.4.4.15)
4. **16-bit Final Fantasy scores generally (IV–VI):** only for thematic-network behavior. Gibbons identifies multiple ways main and character themes are connected—omission, motivic networking, hybridization, associative troping, and double-main-theme technique. [Gibbons](https://doi.org/10.1093/mts/mtad009)

This report does **not** generalize from battle music, *One-Winged Angel*, *Dancing Mad*, later orchestral arrangements, or Uematsu’s non-Final-Fantasy career. Uematsu himself has described technological change as changing how fully he could realize orchestral intentions, making original game assets and later concert orchestration different media rather than interchangeable evidence. [Red Bull Music Academy interview](https://daily.redbullmusicacademy.com/2014/10/nobuo-uematsu-interview/)

### Copyright boundary

No melody, note-for-note rhythm, bass line, or score excerpt from the corpus is reproduced. All acceptance examples below use invented scale-degree cells.

## 2. Theoretical profile

### 2.1 Tonal center, modality, chromaticism, tuning

- **Invariant:** maintain an audible tonal center. Even when a section darkens or destabilizes, the player should be able to hear arrival, departure, and return.
- **Approved default palette:** harmonic minor, abstract degrees `1 2 ♭3 4 5 ♭6 7`, in 12-tone equal temperament. Its raised `7` supports a clear dominant-to-tonic arrival and its `♭6–7` gap provides “fantasy” color without requiring chromatic generation.
- **Allowed alternatives:** Aeolian for softer/modal passages; Dorian for a brighter minor sixth; Ionian for a luminous return. These are legitimate variations, not simultaneous automatic mode rotation.
- **Chromaticism:** player chromatic notes may be accepted, but they must not be advertised as authored secondary dominants, modulation, or narrative reharmonization. The current reactive chord selector decides from each incoming pitch and short internal context; it has no authored harmonic timeline.
- **Tuning:** 12-TET only. No microtonal claim.

**Competing interpretation:** harmonic minor is an effective product color, but the academic sources retained here establish linear/thematic craft more strongly than a corpus-wide harmonic-minor preference. Treat “harmonic-minor chorale” as the preset’s bounded construction, not a finding that Uematsu habitually uses one scale.

### 2.2 Chord vocabulary, spacing, inversion, doubling, tension

- Use functional **triads as the floor**: tonic, predominant, dominant, and relative-major regions.
- Seventh sonorities may occur as contextual color, but persistent stacked diatonic sevenths on every melody note would turn the result into planing rather than chorale support.
- Four parts should favor distinct registers, no voice crossing, limited upper-part spacing, and parsimonious common-tone/step motion.
- Root doubling is acceptable in stable triads; leading-tone doubling near cadence is not a stylistic target.
- Dissonance should read as directed tension: passing/neighbor color or dominant pull, followed by consonant arrival. The preset must not promise suspension grammar unless a beat-aware Species IV lane is actually used.

The FFVI opera thesis supports hearing contrapuntal/linear progressions as structurally differentiated rather than treating every audible surface event as equally important. [Comeaux](https://aquila.usm.edu/masters_theses/822)

### 2.3 Memorable melody, contour, motif, cadence

The **player**, not the preset, must provide melody identity.

A suitable source phrase has:

- a compact 3–5-note cell;
- one contour apex rather than continuous range expansion;
- mostly steps and thirds, with at most one salient leap;
- literal or near-literal rhythmic recurrence;
- a held destination on `5`, `7→1`, or `2→1`;
- a real rest after the cadence.

“Memorable” here means perceptually repeatable, not algorithmically detected. Gibbons’s thematic-network findings show that identity can survive omission, recombination, and association across a score, but current Contrapunk has no motif identity store and cannot implement those transformations. [Gibbons](https://doi.org/10.1093/mts/mtad009)

Cadence targets:

- **Strong:** `… 2–7–1` or `… ♭6–7–1`, with the final `1` held.
- **Half-close:** `… 4–5`, leaving response space.
- **Modal/soft:** `… ♭3–2–1` without forcing a raised leading tone.

The harmony engine may happen to select dominant-to-tonic motion, but a specific cadence cannot be guaranteed from an arbitrary melody because the chord choice is reactive rather than score-authored.

### 2.4 Voice leading and independence

Desired chorale behavior:

1. soprano/player line remains identifiable;
2. inner voices retain common tones or move by step where possible;
3. bass articulates functional change rather than mechanically shadowing the soprano;
4. parallel perfect fifths/octaves are avoided between successive voicings;
5. cadential leading tone resolves to tonic when the input/context permits.

The current `BachChorale` mode is a closer implementation than `DiatonicThirds`: it selects a contextual chord, forces four-note SATB output, stores the previous voicing, and uses a Bach-specific SATB voicer. Its tests explicitly check four outputs and reject parallel fifths/octaves across a test transition (`crates/contrapunk-harmony/src/functional/mod.rs`). That supports **“SATB-style reactive voice leading,” not “Uematsu orchestration” and not a composed chorale progression.**

`DiatonicThirds` is a fallback only. With four voices it chains thirds into a seventh chord above every input and therefore cannot substantiate functional chorale or cadential claims. `FunctionalHarmony` can provide reactive triadic/seventh support with configurable voice count, but its generic near-placement is weaker than the dedicated SATB voicer. `StrictCounterpoint` and `ContraryMotion` create line-to-line motion; they do not select four-part functional chorale harmony.

### 2.5 Octave / call-answer texture and layer interaction

The approved temporal image is:

`player + immediate chorale` → **fixed delay** → `one exact-register octave echo`

This is a restrained antiphonal effect, not evidence that octave answering is a universal Uematsu trait. The delayed answer should be quieter at mix level if a separate Canon group gain is available, and should enter only once per input note. It should not itself be reharmonized.

A useful default is a **2-beat delay in 4/4**, or 4 beats when the performer plays half-note/whole-note phrase tones. Delay is fixed; there is no phrase-end detector selecting an answer time.

### 2.6 Register and orchestration

Abstract target registers:

- bass: approximately MIDI 36–52;
- tenor: 48–60;
- alto: 55–69;
- player/soprano: approximately 60–81;
- octave echo: player pitch +12, provided MIDI range permits.

These are acceptance bands, not hard guarantees of the preset schema. The global `BachChorale` path redistributes its fixed block around `voice_position` and computes arrangement indices from pitch order, but no ArrangementPresetV2 field authors an instrument family, attack envelope, choir section, or dynamic orchestration. A “crystal,” choir, strings, celesta, or synth timbre may be suggested separately; it must not be claimed as arranged by this preset.

### 2.7 Rhythm, meter, articulation, density, silence

- **Meter:** 4/4 default; 3/4 is allowed if the player maintains phrase clarity.
- **Tempo:** approximately 60–92 BPM.
- **Subdivision:** quarter- and half-note-dominant melody, occasional paired eighths; no swing.
- **Attack:** soft/normal, legato but rearticulated enough that NoteOff ownership remains clear.
- **Cadence:** final note 2–4 beats.
- **Density:** one monophonic input at a time; immediate texture is four voices; delayed echo adds one, for a strict maximum of **five logical sounding notes per still-held source note** under the approved mapping.
- **Silence:** at least 2 beats after a four-bar statement and preferably one bar after an eight-bar statement. Silence is performed, not generated.

Fast continuous eighths cause every note to receive a four-note block plus an octave echo and are therefore rejected as “chorale.” Input chords are also rejected: each physical note becomes an independent source owner and can create up to five logical outputs, producing a note storm.

## 3. Temporal behavior

### Within-phrase evolution

The musically desired model is:

`Clear solo attack → supported continuation → one delayed high answer → held cadential chord → silence`

Present implementation can realize only the event-local portions:

1. each input attack receives an immediate reactive four-part voicing;
2. the same source schedules one octave-transposed Canon attack at `t + delay`;
3. the octave answer preserves the input duration and releases at the correspondingly delayed off-time;
4. player-provided long final duration and rest create cadence and thinning.

It cannot autonomously make the opening sparse, increase density near the middle, recognize the cadence, or suppress the Canon during the final rest based on form.

### Across-section / game evolution

The corpus supports thinking in terms of recurrence, thematic relationship, and contrast. Gibbons documents thematic networking across 16-bit scores, while FFVII scholarship reads leitmotif through a four-part narrative shape. [Gibbons](https://doi.org/10.1093/mts/mtad009) [“Kishōtenketsu as Leitmotif”](https://doi.org/10.1525/jsmg.2023.4.4.15)

A desirable game-aware state sketch would be:

`Sparse statement → chorale return → octave-highlighted expansion → contrasting/destabilized section → tonic recall`

**Unsupported now:** no game-state input, motif recognition, named theme memory, density sensing, mode timeline, section trigger, orchestration scene, or automatic return. UI/result text must not imply these behaviors. The player may manually evoke the arc by first playing sparsely, then repeating higher/louder, then withdrawing.

### Career / period boundary

This preset references the melodic, loop-conscious, limited-hardware-to-early-PlayStation period represented by FFIV–VII, not later fully orchestrated concert practice. Uematsu’s interview distinguishes earlier technological limits from later access to fuller orchestral realization. [Red Bull Music Academy](https://daily.redbullmusicacademy.com/2014/10/nobuo-uematsu-interview/)

## 4. Testable stylistic invariants

1. **Tonal center remains legible.** Default scale is harmonic minor only if the functional/Bach chord table supports it on the target build; otherwise use Aeolian and disclose the weaker leading-tone behavior.
2. **Player melody remains topologically intact:** no replacement, quantization, fragmentation, or invented continuation.
3. **Exactly four immediate BachChorale outputs per valid monophonic NoteOn**, including the played note, independent of the nominal global `voice_count` setting.
4. **Exactly one delayed Canon subject per source NoteOn**, configured `PassThrough`, `voice_count = 1`, `transpose_degrees = +7`, `time_ratio = 1.0`.
5. **No reharmonization of the answer.** If Canon inherits `BachChorale` or has `voice_count > 1`, the contract fails.
6. **Strict maximum = five logical notes per source event** (four immediate + one delayed), not “four voices total.”
7. **Every emitted NoteOn has one matching owner-scoped NoteOff**, including delayed notes; disable, Panic, or preset replacement clears pending runtime.
8. **Cadence and silence are performer-controlled.** No automatic formal claim.
9. **No protected melodic material is embedded.**

## 5. Parameters that may vary without losing identity

- tonic: any MIDI key that leaves SATB and +12 echo in range;
- scale: harmonic minor default; Aeolian/Dorian/occasional Ionian as disclosed alternatives;
- tempo: 60–92 BPM;
- Canon delay: 2 or 4 beats;
- voice position: soprano input preferred; alto-position input allowed if redistribution remains uncrossed;
- echo direction: +7 diatonic degrees preferred; −7 is an allowed darker variant;
- phrase length: four or eight bars, supplied by the performer;
- articulation: legato to lightly detached, but not dense staccato streams;
- mix: Canon answer below the immediate chorale in gain.

## 6. Rejection behaviors

Reject or gate the preset if any of the following is observed:

1. Canon inherits global `BachChorale`, producing a delayed four-note block (eight-note combined peak) instead of one octave answer.
2. Canon uses `voice_count = 2` default; even `PassThrough` happens to emit one pitch, but the invariant should be explicit rather than relying on mode behavior.
3. “Octave” is claimed for chromatic/out-of-scale input without verifying the output is exactly `input ± 12`; `harmonize_smart(+7)` is guaranteed as an octave only for a valid heptatonic in-scale degree and sufficient MIDI headroom.
4. Global `voice_count = 4` is presented as controlling `BachChorale`; that mode always returns four voices in the audited implementation.
5. The product copy promises learned themes, leitmotif combination, four-/eight-bar recognition, adaptive intensity, authored modulation, instrument orchestration, or game-state response.
6. Parallel stacked sevenths (`DiatonicThirds`, four voices) are labeled a chorale substitute.
7. Dense chords, rapid repeated notes, sustain-pedal accumulation, or no phrase rests push the texture above the bounded monophonic-source contract.
8. A held input is released but delayed notes remain without matched NoteOff, or a canceled pending NoteOn later receives an orphan NoteOff.
9. “In the style of Nobuo Uematsu” or exact imitation replaces the narrower “Uematsu-informed FFIV–VII melodic/chorale approximation.”

## 7. Engine and ArrangementPresetV2 capability audit

### HarmonyEngine

**Implementable now**

- `ScaleMode::HarmonicMinor` exists with intervals `[0,2,3,5,7,8,11]` (`config.rs`).
- `HarmonyMode::BachChorale` routes through `functional::bach_chorale`, returns a fixed four-note SATB voicing, stores previous voicing, and has a regression checking no parallel perfect fifths/octaves over a test transition (`engine.rs`; `functional/mod.rs`).
- NoteOn/NoteOff mappings are stored in `active_notes`, so the release uses the pitches created at attack time (`engine.rs`).
- Parameter changes route through the engine’s re-harmonization lifecycle rather than simply forgetting sounding outputs.

**Important limits**

- The functional path first checks `HarmonicContext::is_compatible_scale`; on an incompatible scale it passes the input through. Therefore activation must include a concrete HarmonicMinor smoke test on the exact build. If that test fails, the honest baseline is Aeolian or the preset remains unavailable.
- Chord selection is reactive and context-scored, not a fixed four-/eight-bar progression.
- `BachChorale` ignores requested voice count and always returns four notes.
- `apply_octave_mode` is not called in the `harmonize_functional` path, so global `OctaveMode` does not create the desired answer there.
- SATB voice leading is generic/Bach-labeled; it does not encode Uematsu-specific voicings, orchestration, or game form.

### BachChorale versus other modes

| Mode | What it can substantiate | Why it is insufficient alone |
|---|---|---|
| `BachChorale` | Fixed four-part reactive SATB, contextual chord choice, previous-voicing continuity | No authored progression, section form, motif, or octave response |
| `FunctionalHarmony` | Reactive functional triadic support with requested voice count | Generic near-placement; weaker chorale-spacing claim |
| `DiatonicThirds` | Deterministic tertian stack in selected scale | Four voices become a moving seventh stack; no contextual cadence |
| `StrictCounterpoint` | Independent consonant line, parallel-perfect avoidance, species timing | Not block chorale harmony; Species 2–4 add attacks/density |
| `ContraryMotion` | Opposed contour | No functional chord/cadence model |
| `PassThrough` | Exact preservation of Canon subject | Required for the one-note octave answer |

### Canon / Free Imitation

The audited `CanonLane` supports:

- per-voice delay `0..16` beats;
- diatonic transpose `−7..+7`;
- time ratio;
- per-voice harmony mode and voice count;
- owner-scoped delayed NoteOn/NoteOff scheduling;
- `HoldMode` resolution `voice → lane → global`;
- bounded maximum eight Canon voices;
- reset/disable cleanup of pending queues and held ownership.

For this preset, use **one** Canon voice only:

```text
delay_beats = 2.0
transpose_degrees = +7
time_ratio = 1.0
harmony_mode = PassThrough
voice_count = 1
hold_mode = Forever
```

Here `Forever` means “do not cancel the already-scheduled echo when the source key is released”; the code still schedules its matching delayed NoteOff from the source duration. It does not mean an unending MIDI note.

The lane’s phrase anchor resets after more than two beats of input silence, but at `time_ratio = 1.0` this does not amount to phrase recognition. “Free Imitation” can alter time ratio and per-voice harmony, but those features are deliberately excluded: they would weaken the exact octave-answer and five-note peak contract.

### Combining chorale and octave answer

**Yes, with a five-note total contract.** Main HarmonyEngine and CanonLane are separate owners: one source NoteOn yields four immediate main outputs and one delayed Canon output. Main note release uses the engine’s stored attack mapping; Canon release uses its `HeldEntry/HeldVoiceFire` ledger and pending-off queue.

**No, if “strict total voice” means four voices inclusive.** There is no current declarative way to allocate “three immediate members of the chorale plus one delayed octave member” while retaining `BachChorale`’s fixed SATB behavior. Claiming four total would be false. There may also be transient retrigger overlap when the player repeats a pitch before the previous delayed answer releases; acceptance must count active source-event ownership, not merely unique MIDI pitch classes.

### ArrangementPresetV2

The Phase 10.2 contract can statically store the required fields:

- harmonic snapshot (scale, mode, voice position/leading);
- Companion enable/global Hold;
- stable Canon lane config and lane/voice Hold;
- base Input/Harmony/Canon/Counterpoint gains;
- capability requirement and availability;
- result/play guidance and separate suggested sound.

It cannot itself execute musical form. A record is configuration plus metadata, not a motif recognizer, harmonic timeline, intensity model, or orchestrator. The following claims remain unsupported until the planned later lanes/scenes exist:

- learned or memorable-motif detection;
- four-/eight-bar phrase capture and formal answer placement;
- automatic sparse→full density growth;
- modulation or harmonic trajectory;
- game-state/adaptive section changes;
- instrument-group orchestration and articulation morphing;
- named theme recurrence/transformation.

## 8. Abstract acceptance examples

Notation: `HM` = harmonic-minor degree; pitches are relative; `ON(p,t,v)` and `OFF(p,t)` are MIDI lifecycle events. `C(p)` means the immediate four-note chord containing player pitch `p`; its other members are context-selected SATB chord tones. `E(p+12)` is the single Canon echo. These examples are invented.

### Example A — one cadential source tone

Configuration: tonic `T = MIDI 57` (A3 pitch class), player input register A4; HarmonicMinor; global BachChorale; one Canon voice as above; delay `2.0`; transport 4/4.

```text
Input:  ON(HM 7 at MIDI 68, beat 2.0, vel 88)
        OFF(MIDI 68, beat 3.0)

Expected immediate at beat 2.0:
        exactly 4 NoteOns = C(7), one of which is MIDI 68
        arrangement pitch order is noncrossing after redistribution

Expected answer at beat 4.0:
        exactly 1 NoteOn at MIDI 80 (= 68 + 12), same source velocity unless mix changes level downstream

Expected releases:
        exactly 4 matching main NoteOffs at beat 3.0
        exactly 1 Canon NoteOff at beat 5.0
        active/pending owner count = 0 after beat 5.0 and tick drain
```

Acceptance does **not** require naming the three selected chord pitches unless the scorer is frozen in a unit fixture. It does require four outputs, inclusion of input, no parallel-perfect violation relative to the preceding frozen fixture, and exact +12 Canon pitch.

### Example B — invented four-beat cell with answer overlap

```text
Input degrees: 1  –  ♭3  2  | 7 ——
Input beats:   0     1.0 2.0 | 3.0
Durations:     0.75  0.75 0.75 2.0 beats
Canon delay:   2.0 beats
```

For each source event `i`:

```text
at t[i]:       4 immediate NoteOns C(p[i])
at t[i]+dur:   the same 4 pitches receive NoteOff
at t[i]+2:     1 Canon NoteOn at p[i]+12
at t[i]+2+dur: that Canon pitch receives NoteOff
```

Required counts over the complete cell: **16 immediate NoteOns + 4 Canon NoteOns; 20 matched NoteOffs.** The cadence is performer-created by holding degree `7`; the engine must not be said to detect or complete it. If the player adds a final invented tonic at beat 5.0 for 3 beats, the same 5-event ownership rule applies and silence begins only after all delayed releases drain.

### Example C — range and chromatic rejection

```text
Input: ON(MIDI 121, beat 0)
Requested echo: 133
```

Because 133 exceeds MIDI range, this input is outside the approved +12-answer register. The preset must clamp/reject at the performer boundary or fail the octave assertion; it must not silently call a fallback pitch an octave answer.

For an out-of-scale chromatic input, accept the main reactive harmony only as generic chromatic handling. The Canon output passes the octave test **only if** `echo_midi == input_midi + 12`; otherwise reject that event from octave-answer acceptance.

### Lifecycle / replacement acceptance

1. Play a note shorter than the 2-beat delay with Canon `Forever`; verify the delayed ON still fires and its delayed OFF follows by the source duration.
2. Repeat the same pitch before the first echo releases; verify two attack owners produce two releases even if MIDI pitches coincide.
3. Disable Canon before its delayed attack; pending attack and pending release disappear without an orphan event.
4. Panic or apply another preset while main and Canon notes sound; one cleanup transaction leaves zero main active notes, Canon held entries, pending ons, and pending offs.
5. Run the same sequence stopped and with transport running. If stopped transport cannot advance Canon deadlines on a surface, the capability must be gated rather than described as operational.

## 9. Confidence and gaps

| Claim | Confidence | Basis / gap |
|---|---|---|
| Bounded FFIV–VII identity prioritizes memorable thematic writing and recurrence | High | Peer-reviewed leitmotivic scholarship and FFVII narrative analysis |
| FFVI opera supports serious linear/contrapuntal hearing | High | Dedicated graduate thesis |
| Harmonic minor is useful for this preset | High as design choice | Exact scale exists in engine |
| Harmonic minor is a defining Uematsu trait | Low | Retained sources do not support a corpus-wide rule; explicitly rejected |
| Octave answer is an implementable texture | High | Canon +7 degrees, PassThrough, one voice |
| Octave answer is a persistent Uematsu fingerprint | Low | Not established by retained scholarship; use as product imagery only |
| Four-part reactive chorale works now | High for supported scales | Direct code/tests; HarmonicMinor compatibility still needs an exact build smoke test |
| Combined strict peak of five notes per non-overlapping source event | High | Four fixed main outputs + one one-note Canon output |
| Four total voices including answer | High-confidence rejection | Fixed four-note Bach path leaves no slot for delayed answer |
| Motif/form/adaptive/orchestration evolution | High-confidence unsupported | No corresponding current engine/Lane state |

Unresolved implementation gap: this read-only audit could not execute the HarmonicMinor/BachChorale fixture or a surface-specific stopped-transport Canon test. Those are mandatory before activation.

## Findings

1. **The strongest researched Uematsu connection is thematic/linear craft, not a single scale.** Academic work documents contrapuntal structure, thematic networking, and narrative leitmotif across bounded scores. [Comeaux](https://aquila.usm.edu/masters_theses/822) [Gibbons](https://doi.org/10.1093/mts/mtad009) [JSMG](https://doi.org/10.1525/jsmg.2023.4.4.15)
2. **A static harmonic-minor SATB approximation is honest if labeled narrowly.** `BachChorale` supplies four reactive voices; HarmonicMinor is present, but compatibility must be smoke-tested.
3. **A one-note octave answer is implementable now.** One Canon voice at `+7` diatonic degrees, `PassThrough`, and `voice_count=1` yields a delayed +12 echo for valid in-scale notes.
4. **The combined honest contract is five, not four, logical notes.** Four immediate SATB outputs plus one delayed echo preserve separate main/Canon lifecycle ownership.
5. **Temporal style remains performer-led.** Phrase length, cadence holds, density arc, and silence are guidance; motif/form/adaptive/orchestration claims remain unavailable.

## Sources

### Kept

- [Daniel Comeaux, *An Analysis of Nobuo Uematsu’s Linear Structures: The Score of Final Fantasy VI’s Opera*](https://aquila.usm.edu/masters_theses/822) — focused score-based linear/contrapuntal analysis of the most chorale-adjacent bounded corpus.
- [William Gibbons, “Leitmotivic Strategies in Nobuo Uematsu’s Final Fantasy Soundtracks,” *Music Theory Spectrum*](https://doi.org/10.1093/mts/mtad009) — peer-reviewed account of thematic networking in the 16-bit scores.
- [“Kishōtenketsu as Leitmotif,” *Journal of Sound and Music in Games*](https://doi.org/10.1525/jsmg.2023.4.4.15) — supports narrative/formal reading of FFVII leitmotif rather than a static style token.
- [Red Bull Music Academy, “Interview: Final Fantasy’s Nobuo Uematsu”](https://daily.redbullmusicacademy.com/2014/10/nobuo-uematsu-interview/) — direct composer testimony on background and changing technological/orchestral conditions.
- [Ludomusicology, *Uematsu Final Fantasy: Music, Context, Meaning*](https://www.ludomusicology.org/studies-game-sound-music/uematsu-final-fantasy/) — scholarly-book table of contents confirming dedicated work on rhythm, villainy, detail, and thematic cohesion; used for research orientation, not unsupported technical claims.
- Repository primary evidence: `crates/contrapunk-harmony/src/engine.rs`, `config.rs`, `functional/mod.rs`; `crates/contrapunk-companion/src/canon_lane.rs`; Phase 10.2 context/catalog/template — authoritative for implementability and lifecycle claims.

### Dropped

- “Nobuo Uematsu — An Analysis of Three Pieces” blog — informal, unsourced, and too narrow for engine invariants.
- Sector VII Music’s FFVII main-theme theory post — potentially useful pedagogically, but secondary/informal and unnecessary where peer-reviewed sources cover phrase/narrative claims.
- Video Game Music Shrine and DowneLink analyses — fan/SEO-level authority and insufficiently bounded sourcing.
- YouTube “Final Fantasy VI Analysis Series” — not inspected in full and therefore not used as evidence.
- Ffmusic.info liner-note translations — potentially valuable primary-adjacent material, but unofficial translation provenance and no direct support for the specific harmonic/octave claims.
- Generic “Music Theory Examples in Video Game Music” compendium — broad examples, not a bounded Uematsu corpus.
- Battle-rhythm chapter — credible scholarship but outside this lyrical-chorale corpus; battle rhythm would bias the preset toward an irrelevant high-density behavior.

## Gaps

- Run an exact HarmonicMinor + BachChorale unit fixture; gate the preset if the functional chord table rejects that scale.
- Verify `+7` returns exactly `+12` for every approved in-scale input register and reject overflow/chromatic exceptions.
- Execute NoteOn/NoteOff, retrigger, disable, Panic, replacement, running-transport, and stopped-transport lifecycle tests on each enabled surface.
- A future motif/phrase lane, harmonic timeline, intensity scene model, and stable orchestration groups are required before the stronger catalog wording “melody grows” can become autonomous behavior rather than performer guidance.
