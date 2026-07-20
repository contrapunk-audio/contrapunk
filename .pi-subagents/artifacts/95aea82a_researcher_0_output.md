# Pixel Trio — Independent Music-Theory / Temporal-Behavior Research

**Phase:** 10.2, preset 36  
**Research role:** B — music theory and temporal behavior  
**Referenced scope:** Koji Kondo’s early Famicom writing, principally *Super Mario Bros.* (1985) and the original *The Legend of Zelda* (1986); Manami Matsumae’s *Rockman / Mega Man* (1987), principally its stage-music practice.  
**Decision:** **Keep “Pixel Trio” capability-locked in its stated melody–bass–counterline form.** The present shared `HarmonyEngine` can cap a response at three simultaneous pitched voices, but it cannot assign independent melody, bass, and counterline roles or reproduce their rhythms, rests, loops, articulation, register policies, channel stealing, or game-state changes. A synchronous three-note harmony is not three-channel economy.

## 1. Scope, corpus, and claim boundary

This report does not treat “8-bit” as a genre or infer either composer’s whole style from one famous tune. It studies a bounded intersection:

- Kondo’s early Famicom problem: memorable, gameplay-matched music made from very few simultaneous sounds, with *Super Mario Bros.* as the clearest documented case. Kondo says the ground music grew from an emphatic hi-hat basis, an open-voiced chord riff, and multiple melodic variations; he also says square waves made close chords unclear and that wider spacing let three channels suggest more voices. [Kondo interview](https://shmuplations.com/kojikondo/)
- Kondo’s game-state thinking: distinct music by location, a faster last-100-seconds state, and close balancing of music with effects. Nintendo’s retrospective likewise records that he repeatedly played the prototype and rejected music whose rhythm did not fit Mario’s movement. [Nintendo, “To Save Memory”](https://www.nintendo.com/en-gb/Iwata-Asks/Super-Mario-Bros-25th-Anniversary/Vol-5-Original-Super-Mario-Developers/5-To-Save-Memory/5-To-Save-Memory-212974.html) [Kondo interview](https://shmuplations.com/kojikondo/)
- Matsumae’s original *Mega Man*: lively, easily remembered, anime-song-like stage melodies written against stage layouts. She explicitly relates comfort with three channels to intensive study of Bach’s three- and four-part *Well-Tempered Clavier*, while saying she separately had to learn drum-oriented writing from contemporary records. [Matsumae interview](https://bravewave.net/interviews/a-conversation-with-manami-matsumae/)
- The hardware boundary: an NES APU has two pulse generators, triangle, noise, and DMC. “Three-channel” in these composer recollections is therefore best read as **three pitched musical lines**, not a complete claim that only three hardware sources existed; noise/DMC and sound effects remain part of the audible economy. [NESdev APU reference](https://www.nesdev.org/wiki/APU)

Later SNES/N64 orchestration, modern chiptune production, sampled “retro” timbres, and later *Mega Man* composers are out of scope. Matsumae composed the original *Mega Man*; *Mega Man 2* is not evidence for her personal vocabulary except for her disclosed small contribution to Air Man and the transfer of her ending-theme data. [Matsumae interview](https://bravewave.net/interviews/a-conversation-with-manami-matsumae/)

## 2. Documented intersection and meaningful differences

### Shared intersection

1. **Scarcity is arranged, not merely counted.** Both composers describe three pitched channels as enough when each line carries information. Kondo uses wide spacing and rests; Matsumae invokes contrapuntal keyboard study and treats effects as notes that can collide with or steal musical lines. [Kondo interview](https://shmuplations.com/kojikondo/) [Matsumae interview](https://bravewave.net/interviews/a-conversation-with-manami-matsumae/)
2. **A foreground hook is indispensable.** Kondo selected a melody from variations over a rhythmic/chordal basis; Matsumae sought lively, singable, easy-to-remember anime-like melody. The identity is motivic and rhythmic, not a scale preset. [Kondo interview](https://shmuplations.com/kojikondo/) [Matsumae interview](https://bravewave.net/interviews/a-conversation-with-manami-matsumae/)
3. **Lines and effects share finite space.** Kondo composed music and effects together; Matsumae shortened effects and adjusted them not to cut the melody, though she reports some released passages where effects do make melody disappear. [Kondo interview](https://shmuplations.com/kojikondo/) [Matsumae interview](https://bravewave.net/interviews/a-conversation-with-manami-matsumae/)
4. **Short loops must tolerate repetition.** Kondo’s lost B section from the *SMB* ending—removed for memory, leaving repeated A material—is direct evidence that formal economy could be imposed by storage. Matsumae’s stage cues are likewise looped game music, but no reliable primary source found here specifies exact phrase counts for every cue. [Kondo interview](https://shmuplations.com/kojikondo/)

### Differences that must not be averaged away

- **Kondo’s selected evidence is groove/space/open-voicing led.** His account starts from the hi-hat push and chord riff, favors wide vertical spacing for square waves, and explicitly values rests and short bass punctuation. Latin and Japanese fusion influences belong especially to *Mario*, while *Zelda* pursues less conventional chord movement and atmosphere. [Kondo interview](https://shmuplations.com/kojikondo/)
- **Matsumae’s selected evidence is songful/contrapuntal/rock-pop led.** Her explicit models are Bach for multi-line clarity and Propaganda, Phil Collins, and Mezzoforte for drums; her target for *Mega Man* was a heroic, lively, memorable anime-song melody. This supports more continuous energetic stage writing and more contrapuntal redistribution than a generic “Mario-like” bounce. [Matsumae interview](https://bravewave.net/interviews/a-conversation-with-manami-matsumae/)
- **Game differentiation differs.** Kondo describes music selected and revised against character movement and location, including state-triggered tempo change. Matsumae describes composing with stage layouts/designs known and balancing the finished cue against effects during testing. These intersect as audiovisual fit, not as one shared harmonic recipe.

A coherent preset may use the intersection—three clearly separated pitched roles, a compact rhythmic hook, economical repetition, and deliberate holes—but must not claim that one Dorian/Ionian setting reproduces either composer.

## 3. Theoretical and temporal profile

### 3.1 Tonal center, modality, chromaticism, tuning

- **Tuning:** ordinary twelve-tone equal-tempered MIDI is an acceptable pitch abstraction. NES timer quantization and pulse/triangle spectra affect exact tuning and perception, but microtonality is not a defining compositional invariant for this bounded preset.
- **Center:** establish a clear local tonic by recurrence, bass arrival, and phrase ending. A stable center helps a very short loop remain legible.
- **Mode:** Ionian, Mixolydian, Dorian, Aeolian, and chromatically colored tonal writing are all plausible across the bounded corpus. **No single mode is invariant.** Kondo explicitly contrasts bright *Mario* practice with *Zelda*’s less conventional chords/atmosphere; Matsumae describes memorable song construction, not a privileged mode. Dorian is therefore an optional color, not “the NES scale.”
- **Chromaticism:** permit short approaches, altered dominant/color tones, or sectional shifts when they clarify motion. Do not quantize every line into one mode and call that authenticity; equally, do not fill every beat with chromatic passing notes.

**Confidence:** high that no one-scale mapping is supportable; medium on the enumerated modal range, which is score-informed generalization rather than a composer quotation.

### 3.2 Interval and harmonic vocabulary

- Prefer **open vertical spacing**, especially between the lowest and middle parts. Kondo directly reports that wide 1–3–5 spacing read more clearly through harmonically rich square waves. [Kondo interview](https://shmuplations.com/kojikondo/)
- Stable arrivals may expose thirds, sixths, fifths, octaves, or incomplete triads. The economy often comes from implication: bass plus melody can define function while the remaining pitched channel moves.
- Seconds, fourths, tritones, and non-chord tones are useful as **linear events** with preparation, passing function, or prompt release. They must not become a permanently planed three-note cluster.
- Doubling is functional only when it reinforces a cadence, octave hook, or register boundary. Continuous octave doubling spends a scarce line without adding contrapuntal information.
- Avoid close three-note block harmony at every melody onset. Even if pitch classes are plausible, it erases the documented open spacing and independent-line premise.

### 3.3 Melody, motive, repetition, and ornament

- The lead should be a compact, singable cell with a stable rhythmic fingerprint: approximately 3–8 salient notes before recognizable recurrence.
- Repetition should include one controlled change: ending degree, octave, pickup, truncation, sequential transposition, or a one-beat extension. Kondo describes making melodic variations over a groove/chord basis; Matsumae explicitly values melody memorable enough to sing or lyricize. [Kondo interview](https://shmuplations.com/kojikondo/) [Matsumae interview](https://bravewave.net/interviews/a-conversation-with-manami-matsumae/)
- Ornament is economical: neighbor flicks, short approaches, repeated-note attacks, and octave transfers. Long ornamental streams weaken hook recognition and crowd effects.
- Contours may be jaunty and syncopated (bounded Kondo/*Mario* side) or broad/heroic and driving (bounded Matsumae side); this is an allowed composer-specific branch, not a blended average.

### 3.4 Voice motion and interaction

The three pitched roles are behavioral, not merely MIDI voice indices:

1. **Lead:** owns the hook and may rest for pickups, effects, or a reply.
2. **Bass:** owns foundation and pulse; uses roots, fifths, stepwise approaches, short scalar links, and occasional pedal points. It should move less often than the lead and occupy a clearly lower register.
3. **Counterline / inner support:** alternates among offbeat chord punctuations, contrary/oblique response, short imitation, and rests. It must not shadow every lead note at a constant interval.

Preferred motion is mixed: contrary or oblique motion at structurally important changes, limited parallel thirds/sixths for brief emphasis, and held/resting voices to preserve independence. A voice can temporarily inherit accompaniment duty when another is silent. Matsumae’s report that sound effects could make the melody disappear is particularly important: channel ownership was dynamic, not three permanently saturated melodic tracks. [Matsumae interview](https://bravewave.net/interviews/a-conversation-with-manami-matsumae/)

### 3.5 Register, spacing, and texture

- Keep bass at least an octave below the lead’s center where range permits; aim for roughly 7–19 semitones between bass and the nearest upper role rather than a fixed close stack.
- Keep lead and counterline separable by register, articulation, rhythm, or rests. Crossing is allowed as an occasional handoff, not a default.
- Texture ceiling is three **pitched** voices. Percussive/noise behavior is separate and must not be advertised unless a sound/pattern capability supplies it.
- Density target: commonly one or two active pitched roles, reaching three at arrivals, replies, or peaks. “All three on every NoteOn” fails the economy test.

### 3.6 Rhythm, meter, subdivision, accent, and phase

- Straight binary subdivision is the safe common ground, with syncopation, pickups, ties, repeated-note drive, and offbeat punctuation. Kondo specifically describes the *SMB* ground cue as based on an emphatic hi-hat beat and rhythmic fit to movement. [Kondo interview](https://shmuplations.com/kojikondo/)
- Do not encode swing as invariant. Latin/fusion influence supports syncopated groove in the selected *Mario* evidence, while Matsumae’s selected evidence supports rock/pop drum energy.
- Bass should not simply duplicate lead rhythm. Counterline attacks should preferentially occupy lead gaps, anticipations, or selected accents.
- A transport clock is required for authentic pattern placement, loop boundaries, tempo-up behavior, and quantized game-state transitions. A note-on counter is not an adequate substitute.

### 3.7 Articulation and dynamics

- Short, clean releases prevent saturation and leave room for effects. Kondo explicitly endorses rest space and short bass punctuation. [Kondo interview](https://shmuplations.com/kojikondo/)
- Use articulation contrast: legible lead attacks, shorter bass/punctuation notes, and clipped counterline replies. Sustaining all three voices through every change produces organ-like block harmony, not scarce-channel writing.
- MIDI velocity may distinguish foreground from support, but classic hardware amplitude behavior and timbre are not reproduced by velocity alone. Do not advertise pulse duty, triangle bass, noise drums, or channel stealing without synthesis/routing support.

### 3.8 Phrase, loop, density, silence, and cadence

A practical abstract loop is 4 or 8 bars, but phrase length must remain variable because no primary source here establishes one universal count. The behavioral arc is stronger than the number:

`Hook attack (lead + sparse foundation) → continuation (counterline enters in holes) → density peak (brief three-role coincidence) → cadence/turnaround (bass confirms center, one upper voice releases) → seam/pickup → loop`

- **Attack:** expose the hook rather than immediately harmonizing every note.
- **Continuation:** repeat/sequence the hook; counterline adds complementary rhythm.
- **Peak:** three pitched roles may coincide briefly, ideally in open spacing.
- **Cadence:** tonic or strongly directed return; thin before or on the seam so recurrence is audible rather than mechanically glued.
- **Silence:** at least one role should rest during a normal phrase, and the lead may have deliberate gaps. Silence is a positive arrangement event, not missing output.

### 3.9 Across-section and game-state development

For this early-console scope, development is economical and often tied to play state:

- **Section A:** establish hook and center with two-role texture.
- **A′:** preserve rhythmic identity; alter ending, counterline, or register.
- **B/contrast when available:** change harmonic region, contour, or rhythmic emphasis while preserving channel ceiling.
- **Return:** restore hook with a small redistribution or octave change.
- **Urgency state:** tempo or subdivision energy increases without adding pitched channels; Kondo names the last-100-seconds tempo increase as a model of interactive enhancement. [Kondo interview](https://shmuplations.com/kojikondo/)
- **Effect/event state:** one musical role yields briefly to a sound event, then resumes at a safe boundary. Matsumae’s production account directly documents this competition. [Matsumae interview](https://bravewave.net/interviews/a-conversation-with-manami-matsumae/)

The current preset cannot implement this state model; it is an acceptance target for a future pattern/phrase lane plus ownership-aware routing.

## 4. Testable stylistic invariants

1. **Three-pitched-role ceiling:** never more than lead, bass, and one counter/support line; noise/percussion is not silently counted as a fourth pitched role.
2. **Role independence:** across a phrase, bass and counterline must each have at least one onset or rest pattern different from the lead.
3. **Hook priority:** a compact rhythmic-melodic cell recurs recognizably before extensive variation.
4. **Open low spacing:** at full density, the lowest-to-next voice is normally at least a perfect fifth and preferably near an octave; close low clusters are rejected.
5. **Density breathing:** at least one role rests somewhere in each 4–8-bar cycle; three-role simultaneity is a peak, not the default.
6. **Bass ownership:** the lowest role confirms center/function and is not merely “the melody two diatonic thirds down.”
7. **Counterline complement:** the third role uses gaps, contrary/oblique motion, or brief imitation; constant parallel tracking fails.
8. **Loop legibility:** recurrence has a cadence, thinning, pickup, or seam cue and produces no hanging notes.
9. **Audiovisual/state honesty:** no claim of tempo-up, event interruption, percussion, or adaptive music without a capability and deterministic trigger.
10. **Composer-reference honesty:** Kondo’s open, syncopated spacious branch and Matsumae’s driving songful contrapuntal branch remain named alternatives; the preset does not flatten both to “Dorian chiptune.”

## 5. Parameters that may vary without losing identity

- Ionian, Mixolydian, Dorian, Aeolian, or modest chromatic tonal color.
- 4- or 8-bar loop; duple or compound-feeling subdivision if all roles remain clear.
- Bright syncopated Kondo-leaning versus driving heroic Matsumae-leaning hook.
- Bass as root/fifth punctuation, pedal, or stepwise connector.
- Counterline as offbeat chordal stabs, short imitation, contrary line, or strategic silence.
- One or two upper registers, moderate tempo over a broad playable range, and either pickup-led or downbeat-led hook.
- Cadence strength and the amount of A/A′/B contrast.

## 6. Misleading behaviors / caricatures to reject

- Selecting Dorian, square-wave sound, and three voices and calling the result Kondo/Matsumae.
- Three close block-chord notes on every input onset.
- A “bass” generated by fixed interval chaining from every melody note.
- A “counterline” with the lead’s exact rhythm and duration.
- Continuous maximum density, no rests, or no room for effects.
- Random-below harmony presented as channel economy; randomness does not create role ownership.
- Strict species counterpoint presented as *Mega Man* stage writing; it lacks rock/pop patterning and still shares input onsets.
- Pulse/triangle/noise claims made by an arrangement preset that does not control chip synthesis.
- Claiming the NES had only three total audio channels. The APU has five hardware sources; the bounded “trio” is three pitched musical roles. [NESdev](https://www.nesdev.org/wiki/APU)
- Treating all Kondo Famicom work as Latin or all Matsumae work as Bach-like.

## 7. Current implementation audit

### 7.1 What the shared `HarmonyEngine` actually owns

The inspected shared engine is a synchronous input-to-pitches transformer:

- `voice_count` counts the played note plus generated harmony notes.
- `voice_position` places the played input at an arrangement slot.
- ordinary modes generate outward in chains; each generated pitch is derived from the previous pitch, not from an independently scheduled role.
- `DiatonicThirds` moves exactly two scale degrees per chain step; `DiatonicFourths` moves three.
- `ContraryMotion` and `StrictCounterpoint` retain pitch history, but generated notes are still emitted from the same input event. Species beat phase influences pitch/rule dispatch, not an independent bass or counterline rhythm.
- voice-leading revoices pitch classes/registers after generation. It does not create rests, accents, loop structure, role-specific note lengths, or game states.
- octave modes shift or duplicate generated notes. `Mirror` exceeds the three-voice ceiling and is disallowed.
- NoteOn/NoteOff ownership is stored per input note, which is good lifecycle behavior but not independent lane ownership.

Code evidence: [`HarmonyEngine::harmonize`](https://github.com/contrapunk-audio/contrapunk/blob/main/crates/contrapunk-harmony/src/engine.rs), [`modes.rs`](https://github.com/contrapunk-audio/contrapunk/blob/main/crates/contrapunk-harmony/src/modes.rs), and [`config.rs`](https://github.com/contrapunk-audio/contrapunk/blob/main/crates/contrapunk-harmony/src/config.rs).

### 7.2 `ArrangementPresetV2` audit

The Phase 10.2 contract can serialize a harmony snapshot, Companion lane snapshot, Hold, mix gains, guidance, references, and capability requirements. Those fields are configuration/metadata; they do not themselves synthesize missing temporal behavior. In the inspected working tree, the production `contrapunk-preset` crate still exposes the older `StylePreset`, while the Phase 10.2 context describes `ArrangementPresetV2` as the planned contract. No current V2 field supplies an independent bass pattern, motif loop, per-role onset grid, effect-driven voice stealing, or game-state timeline.

Therefore a V2 record may honestly **declare** the missing capabilities and remain unavailable; it may not use metadata labels or mix group names to pretend synchronous engine outputs are independent lane roles.

### 7.3 Exact candidate mappings and why none satisfies the catalog result

Let input scale degree be `d`, `S(d)` its MIDI pitch in a selected seven-note scale with octave continuation, and output order be `[input, generated…]`.

#### Candidate A — three voices, input at top, Diatonic Thirds

Configuration core: `voice_count=3`, `voice_position=0`, `HarmonyMode::DiatonicThirds`, `OctaveMode::None`, interchange off.

Exact pitch vector:

`P(d) = [S(d), S(d−2), S(d−4)]`

For C Ionian inputs C4–B4, exact MIDI/output vectors are:

| input | output MIDI | relative semitones |
|---|---:|---:|
| C4 (60) | `[60,57,53]` = C4 A3 F3 | `[0,−3,−7]` |
| D4 (62) | `[62,59,55]` = D4 B3 G3 | `[0,−3,−7]` |
| E4 (64) | `[64,60,57]` = E4 C4 A3 | `[0,−4,−7]` |
| F4 (65) | `[65,62,59]` = F4 D4 B3 | `[0,−3,−6]` |
| G4 (67) | `[67,64,60]` = G4 E4 C4 | `[0,−3,−7]` |
| A4 (69) | `[69,65,62]` = A4 F4 D4 | `[0,−4,−7]` |
| B4 (71) | `[71,67,64]` = B4 G4 E4 | `[0,−4,−7]` |

This is deterministic three-note diatonic planing. The “bass” has exactly the lead’s onset, release, and contour transformed by fixed degree offset. It fails bass ownership, rhythmic independence, density breathing, and usually low-register spacing. **Rejected as Pixel Trio.**

For tonic Dorian with tonic input `T`, the vector is `[T, T−3, T−7]`; for scale degree 4 it can expose a tritone in the outer chain just as Ionian does on degree 4. Changing Ionian to Dorian changes pitch content, not ownership.

#### Candidate B — three voices, input in middle, Diatonic Thirds

Configuration core: `voice_count=3`, `voice_position=1`, thirds, no octave transform.

Exact pitch vector/output ownership:

`P(d) = [S(d), S(d+2), S(d−2)]`, with arrangement slots `[middle(input), top, bottom]`.

For C4 in C Ionian this is `[60,64,57]` = C4/E4/A3. It improves vertical distribution but both generated voices still attack/release with C4 and follow fixed parallel degree offsets. Calling the lower note bass and upper note counterline would be a label-only fiction. **Rejected.**

#### Candidate C — three voices, input at top, Diatonic Fourths

`P(d) = [S(d), S(d−3), S(d−6)]`; C4 in C Ionian yields `[60,55,50]` = C4/G3/D3, relative `[0,−5,−10]`.

This gives clearer open spacing but creates synchronous quartal planing, not a functional bass plus independent answer. It also substitutes a jazz/quartal signature not supported as the shared Kondo–Matsumae invariant. **Rejected.**

#### Candidate D — Contrary Motion or Strict Counterpoint, three voices

No single exact pitch vector exists because history and scoring affect output. Exact event ownership does exist: every generated NoteOn and NoteOff remains causally owned by the same live input event, and all three roles share its event boundary. Pitch independence cannot repair temporal dependence. **Rejected as the operational preset**, though useful as a generic live contrapuntal effect.

### 7.4 Required ownership acceptance tests

Any future implementation must pass all of these before the preset unlocks:

1. **Voice cap:** for every timestamp, active pitched owners ⊆ `{lead,bass,counter}` and count ≤ 3. Percussion/effects are separately typed.
2. **Distinct source ownership:** each emitted note records `(lane_instance_id, role, source_event_id)`; NoteOff matches that tuple. Bass/counter notes may outlive or avoid a lead note only under explicit Hold policy.
3. **Rhythmic non-identity:** in one accepted 4–8-bar fixture, neither bass onset set nor counter onset set equals the lead onset set; each role also owns at least one rest while another sounds.
4. **Bass semantic test:** transposing one interior lead note without changing the harmony state must not necessarily transpose the bass at that instant. A fixed `S(d−4)` mapping fails.
5. **Counterline semantic test:** at least one reply begins during a lead rest or sustains under a changed lead; fixed NoteOn chaining fails.
6. **Register test:** whenever all three sound, `counter/lead` remain distinguishable and `middle−bass ≥ 7 semitones` except a documented passing crossing.
7. **Density test:** a fixture contains 1-, 2-, and 3-pitched-role states; three is not active throughout.
8. **Loop/lifecycle test:** after two loops, transport stop, preset replacement, and Panic, every owner has zero active notes and empty delayed queues.
9. **State-change test:** urgency can change tempo/pattern at a declared boundary without adding a pitched voice; event interruption releases the displaced owner before reuse.
10. **Surface gate:** if independent pattern/lane roles or ownership telemetry is absent on a surface, Apply remains disabled with a precise explanation.

### 7.5 Capability conclusion

The engine can honestly promise only **“three simultaneous diatonic/counterpoint-derived pitches from one live note.”** It cannot honestly promise **“melody, bass, and one counterline obey three-channel economy.”** The latter requires, at minimum:

- an ownership-safe pattern/phrase lane or equivalent independent scheduler;
- explicit stable role assignment and register bounds;
- per-role rhythm, note length, rests, density, and loop boundaries;
- transport for quantization and state changes;
- optional effect/channel-stealing policy if that behavior is claimed.

Chip synthesis is not required to realize the *arrangement principle*, but without it the result must be described as a modern three-role MIDI arrangement inspired by early-console economy, never as NES sound simulation.

## 8. Abstract acceptance examples (original; no borrowed melodies)

Notation: scale degrees use `^1…^7`; octave displacement is `+8/−8`; beats are in 4/4. `—` sustains; `r` rests. These examples specify relationships, not copyrighted tunes.

### Example 1 — bright syncopated, Kondo-leaning economy

**Input lead, two bars:**

`b1 &:^3(⅛), b2:^5(⅛), b2&:^6(¼), b3&:^5(⅛), b4:^2(¼) | b1:^3(¼), b2:r, b3:^2(⅛), b3&:^1(⅜), b4:r`

**Acceptable arrangement behavior:**

- Bass: `b1:^1−8(⅛), b2:r, b3:^5−8(⅛), b4:r | b1:^4−8(⅛), b3:^5−8(⅛), b4:^1−8(⅛)`.
- Counter: rests through the first hook attack; punctuates `b2&:^1`, answers the lead gap at bar 2 beat 2 with `^6–^5`, then releases before the cadence.
- Full three-role density occurs only at one or two accents; cadence lands on lead `^1` against bass `^1−8` or `^5−8`; loop pickup remains exposed.

**Reject:** outputting `[lead, lead−2 degrees, lead−4 degrees]` at every listed lead onset.

### Example 2 — driving songful, Matsumae-leaning contrapuntal stage loop

**Input lead, four-beat cell repeated with changed ending:**

`b1:^1(⅛) ^1(⅛), b2:^3(¼), b2&:^4(⅛), b3:^5(¼), b4:^3(¼) | repeat, but b4:^2(⅛)→^1(⅛)`

**Acceptable arrangement behavior:**

- Bass: repeated `^1−8` on beats 1 and 3 in bar 1; moves `^6−8 → ^5−8 → ^1−8` across bar 2’s last three beats.
- Counterline: contrary fragment `^5 → ^4 → ^3` beginning in the lead’s first meaningful gap; one held `^6` may create controlled oblique tension before resolving to `^5`.
- The lead’s repeated-note attack remains foreground; the bass does not reproduce it. Bar 2 ending is recognizably varied and loop-safe.

**Reject:** “Bach” species rules alone, continuous close triads, or unbroken sixteenth-note filling.

### Example 3 — game-state transition

**Normal state:** two-role A loop, lead + sparse bass; counter enters only in A′.  
**Trigger:** deterministic urgency event at next bar boundary.  
**Urgency state:** same pitch-role ceiling and motif, tempo increases or bass subdivision becomes more active; counterline shortens values rather than adding a fourth part.  
**Event interruption:** on an effect trigger, counter releases at the boundary, its slot remains unavailable until the effect completes, then counter resumes from a loop-safe point.

**Current-engine expectation:** this example must be reported unsupported, not approximated by increasing harmony voice count.

## 9. Competing interpretations and confidence

- **“Three channels” versus five APU channels:** high confidence. The technical APU has five sources, while both composers discuss three musical sounds/chords. The preset should say “three pitched roles,” not rewrite the hardware specification. [NESdev](https://www.nesdev.org/wiki/APU)
- **Independent counterpoint versus perceptual channel multiplexing:** medium confidence. Matsumae’s Bach statement strongly supports line-aware thinking, and academic work argues that *Mega Man* can blur continuity to suggest denser textures; however, cue-by-cue ownership/transcription was not available here, so no universal hocketing rule is asserted. [Banks, “The Wrong Tool for the Right Job”](https://doi.org/10.51191/issn.2637-1898.2019.2.3.69)
- **Mode choice:** high confidence that neither Dorian nor Ionian alone identifies the style; medium confidence on which modal mix best serves a future implementation because this requires approved score/NSF transcription across the selected cues.
- **Exact phrase lengths:** low-to-medium confidence beyond short-loop practice. Memory-driven repetition is documented, but a universal 4/8-bar rule is not; those lengths are implementation-friendly variables, not historical invariants.
- **Open spacing as shared invariant:** high for Kondo, medium for the combined preset. Matsumae’s own statement supports chordal clarity through multi-part writing but does not independently prescribe Kondo’s exact spacing rule.

## 10. Sources

### Kept

- [Koji Kondo — 2001 composer interview (translated by Shmuplations)](https://shmuplations.com/kojikondo/) — direct, detailed statements on three channels, square-wave/open spacing, the *SMB* hi-hat/chord-riff/melodic-variation process, rests, short bass notes, memory cuts, and game-state music.
- [Nintendo, *Iwata Asks: Super Mario Bros. 25th Anniversary*, “To Save Memory”](https://www.nintendo.com/en-gb/Iwata-Asks/Super-Mario-Bros-25th-Anniversary/Vol-5-Original-Super-Mario-Developers/5-To-Save-Memory/5-To-Save-Memory-212974.html) — first-party retrospective on composing to the moving prototype and severe memory economy.
- [Manami Matsumae, “A Conversation with Manami Matsumae”](https://bravewave.net/interviews/a-conversation-with-manami-matsumae/) — direct statements on three-channel comfort through Bach, learned drum vocabulary, anime-like memorable melody, stage awareness, and music/effect collision.
- [NESdev Wiki, APU](https://www.nesdev.org/wiki/APU) — technical reference establishing two pulse, triangle, noise, and DMC sources; prevents the false “three total hardware channels” claim.
- [Tobias Banks, “The Wrong Tool for the Right Job: Composition on 8-bit Machines”](https://doi.org/10.51191/issn.2637-1898.2019.2.3.69) — academic contextual source on hardware-shaped 8-bit composition and *Mega Man*’s perceptual handling of voices; used cautiously because only abstract-level evidence was accessible.
- [Contrapunk `HarmonyEngine`](https://github.com/contrapunk-audio/contrapunk/blob/main/crates/contrapunk-harmony/src/engine.rs), [`modes.rs`](https://github.com/contrapunk-audio/contrapunk/blob/main/crates/contrapunk-harmony/src/modes.rs), and [`config.rs`](https://github.com/contrapunk-audio/contrapunk/blob/main/crates/contrapunk-harmony/src/config.rs) — direct implementation evidence for chained pitch generation, shared event timing, voice count/position, scales, modes, and octave behavior.

### Dropped or restricted

- Hooktheory pages for *Zelda*, Cut Man, and Elec Man — community-entered reductions are useful leads but insufficiently authoritative for exact ownership/form claims.
- Fan “inside the score” essays and YouTube explainers — secondary interpretations, redundant once direct interviews and implementation evidence were available.
- Video Game Music Preservation Foundation cue pages — useful for credits and loop duration, but not strong enough to establish theoretical invariants.
- Nintendo Life, Nintendo Everything, VG247, Kotaku, and other interview summaries — dropped where direct interview text was available.
- *Mega Man 2* analyses — wrong principal composer for this Matsumae-bounded report.
- Later Kondo/Matsumae work — excluded to avoid turning period-specific Famicom constraints into whole-career traits.

## 11. Gaps and next research steps

1. Obtain legally accessible, independently verified transcriptions or event dumps for a small fixed corpus: *SMB* Ground/Underground/Castle, original *Zelda* Overworld, and *Mega Man* Cut Man/Elec Man/Ending. Measure per-channel onset overlap, rest ratio, register span, interval classes, loop seam, and effect stealing.
2. Verify whether each selected cue uses DMC and how often noise/effects alter pitched-channel availability; do not infer this from the generic APU.
3. Resolve the exact Phase 10.2 capability IDs and final `ArrangementPresetV2` source once implemented. The musical gate should name independent pattern/phrase scheduling, stable lane roles/ownership, register constraints, and transport.
4. If product scope accepts a weaker result—“three synchronous open diatonic voices”—rename and rewrite the Result text. Do not unlock it under “melody, bass, and one counterline.”

## 12. Final recommendation

`Pixel Trio` is musically coherent as a **future three-role arrangement preset**: compact hook, sparse functional bass, complementary counterline, open spacing, role rests, short loop, and bounded state changes. It is **not** coherent as a shared scale plus three-voice harmony snapshot. Keep unsupported channel simulation, chip timbre, percussion, independent bass, looping, and game-state behavior explicitly locked. The smallest honest implementation is not a clever `HarmonyEngine` setting; it is no implementation until stable independent role scheduling and ownership tests exist.