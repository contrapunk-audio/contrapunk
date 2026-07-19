# Research: Phase 10.2 preset 14 — Color-Mode Windows (Messiaen), Role C performer/interaction

**Independence and scope.** This is a performer/HCI report, not a final configuration. It scopes the reference to the modes-of-limited-transposition practice codified in Messiaen’s 1944 *Technique de mon langage musical*, with selected later evidence used only to clarify sound-color and contrast. It does **not** treat that technique as Messiaen’s whole career: birdsong, non-retrogradable and added-value rhythms, theology, resonance chords, orchestration, and the dense layered designs of the 1975–83 opera *Saint François d’Assise* remain outside this playable approximation. The catalog’s “rotating whole-tone, octatonic, and augmented colors” is therefore narrowed below to **one fixed Mode-1/Mode-2-derived collection per preset application**, shaped into contrasting windows by the player.

## Summary

A clean monophonic player can evoke this bounded practice with a short, strongly articulated cell drawn from the selected symmetric collection, then silence: let each generated sonority register as a distinct color, repeat the cell with one controlled change, and make contrast through register, attack, duration, and spacing. Contrapunk can presently supply a fixed whole-tone or octatonic pitch environment and fixed scale-degree harmony; it cannot compose modal material, infer a phrase, rotate collections or transpositions over time, reproduce Messiaen’s particular chords/colors, or honestly substitute its augmented hexatonic scale for Messiaen’s nine-note Mode 3.

## Evidence-led interaction thesis

1. **The bounded reference is a symmetric pitch resource, not “the Messiaen style.”** Messiaen’s treatise was published in 1944 and systematizes techniques developed by that point; modern theory describes the modes as pitch collections whose symmetry limits distinct transpositions. Mode 1 is whole-tone, Mode 2 octatonic, and Mode 3 a **nine-note** collection. [Philharmonie de Paris catalog record](https://mediatheque.philharmoniedeparis.fr/mediatheque/doc/ALOES/0077526/technique-de-mon-langage-musical-texte-avec-exemples-musicaux-olivier-messiaen), [Messiaen, *Technique*](https://monoskop.org/images/5/50/Messiaen_Olivier_The_Technique_of_My_Musical_Language.pdf), [Pople, Cambridge appendix](https://doi.org/10.1017/CBO9781139166928.011). **Confidence: high.**
2. **“Color” is relational and chordal, not a magic property of an isolated scale note.** Benitez, citing Messiaen’s lectures and interviews, reports that his colored hearing responded to chords rather than isolated pitches and argues that sonorities gain movement and expression through juxtaposition/contrast. This supports discrete attack–listen–silence windows and rejects claims that one incoming note recreates a named visual color. [Benitez, “Simultaneous Contrast and Additive Designs,” *Music Theory Online* 8/2 (2002), §§1.1–1.7, 3.1–3.5, 7.1–7.4 and n.1](https://www.mtosmt.org/issues/mto.02.8.2/mto.02.8.2.benitez.html). **Confidence: high for the scholarship; medium for the live mapping.**
3. **Selected works combine modal materials with tonal centers, subsets, special chords, register, and timbre.** In the Angel’s discourse from late *Saint François*, Benitez finds Modes 2 and 3 as foreground colors around A major; elsewhere he describes simultaneous modal/timbral layers. That later-work evidence is useful as a warning: merely choosing an octatonic scale is a severe reduction, not a reproduction of the work or of Messiaen’s mature language. [Benitez §§5.2–5.7, 6.3–6.10](https://www.mtosmt.org/issues/mto.02.8.2/mto.02.8.2.benitez.html). **Confidence: high.**
4. **Current mapping is static and narrower than the catalog wording.** Contrapunk defines `WholeTone [0,2,4,6,8,10]`, two diminished/octatonic orderings, and `AugmentedHex [0,3,4,7,8,11]`; its stateless thirds/fourths modes add fixed scale-degree offsets. `setScaleMode` selects one scale; no public adapter field describes automatic collection/transposition rotation or Messiaen-specific chord forms. [Contrapunk `config.rs`](https://github.com/contrapunk-audio/contrapunk/blob/main/crates/contrapunk-harmony/src/config.rs), [`engine.rs`](https://github.com/contrapunk-audio/contrapunk/blob/main/crates/contrapunk-harmony/src/engine.rs), [adapter contract](https://github.com/contrapunk-audio/contrapunk/blob/main/ui/src/lib/adapter/types.ts). **Confidence: high for the inspected revision.**
5. **Guitar needs a stricter source contract than MIDI keyboard.** Comparative real-time tests on dry, single-note electric guitar report pitch fluctuations and octave errors and frame the task explicitly as monophonic. Stable fretted pitches, damping, separated attacks, and conservative low-register use are therefore acceptance safeguards, not claims about Messiaen performance practice. [von dem Knesebeck & Zölzer, “Comparison of Pitch Trackers for Real-Time Guitar Effects,” DAFx-10](https://dafx10.iem.at/proceedings/papers/VonDemKnesebeckZoelzer_DAFx10_P102.pdf). **Confidence: high for the general boundary; medium for Contrapunk’s detector specifically.**

## Performer contract

### 1. Best source gesture

Play a **two-to-four-note monophonic cell**, one note at a time, with a clear attack and a full release after the cell. Use only notes from the currently selected collection. Repeat the cell once or twice, changing only one dimension—register, last note, velocity, or spacing—so the ear compares one color window with the next.

A single sustained tone is useful for setup but too inert as the main gesture; a fast riff hides the generated harmony. Played chords are not suitable: the preset already multiplies each NoteOn.

### 2. Register, tempo, note length, articulation, dynamics, density, and silence

- **Register:** start in the middle band, approximately C3–C5. Guitar: favor D3–B4 and stable fretted notes. Keyboard may answer an established cell one octave higher, but avoid the extreme bass where generated intervals thicken.
- **Tempo/pacing:** transport-free, about **54–76 quarter-note pulses/minute** if a reference pulse helps. This is interaction guidance, not a historically authentic Messiaen tempo.
- **Note length:** normally **0.6–2.0 seconds**; one destination tone may last 2–3 seconds. Release before the next NoteOn in the checked exercise.
- **Articulation:** clean, firm, non-percussive attacks; mostly separated or lightly tenuto. The catalog’s “strongly articulated” should mean unmistakable onset boundaries, not uniformly loud accents.
- **Dynamics:** begin around MIDI velocity **55–70**, make one contrasted window **80–95**, then return to **45–60**. Do not rely on a held note to crescendo unless the chosen sound surface actually supports continuous expression.
- **Density:** one physical note at a time; 2–4 notes per cell; no more than about 6–10 NoteOns per 15 seconds.
- **Silence:** 0.3–0.8 seconds between notes, then **1.5–3 seconds after each cell**. The long gap is the frame around the “window” and prevents release tails from becoming a continuous cluster.

### 3. Transport dependency and precision

**Transport is not required.** The baseline symmetric-scale/harmony mapping is per NoteOn and does not need beat phase. Preserve the existing transport state. Rhythmic precision is local: repeat a cell recognizably, but do not quantize it or claim Messiaen’s added-value/non-retrogradable rhythm. If transport happens to run, use it only as a personal reference; this preset must not start, stop, reset, or retime it.

### 4. Phrase and section development

Use a player-made three-window form:

1. **Present (8–12 s):** state a 3-note middle-register cell at medium-soft velocity; rest.
2. **Refract (10–15 s):** repeat its contour one octave higher or alter one terminal collection degree; slightly stronger attacks; rest longer.
3. **Withdraw (8–12 s):** return to the original register, use only the first two notes or lengthen the last note, then leave silence.

Across a longer section, keep the selected pitch collection fixed and vary one parameter per window. If the user manually chooses another scale/transposition, stop and clear the prior window first; that is an explicit human edit, **not** an automatic form supplied by the preset. Do not cycle whole-tone → octatonic → augmented merely because the title says “windows.”

### 5. Listening and responding

After each NoteOn, wait until the complete generated sonority is audible before deciding on the next note. After each cell, listen through the decay and ask one concrete question: did the repeated cell sound brighter/denser because of register and attack, or did extra notes merely accumulate? Continue only after the previous owned notes have released. At the final rest, confirm that no harmony remains.

### 6. Guitar constraints under clean monophonic input

- Use a dry/clean signal and mute unused strings with both hands.
- Prefer fretted middle-string notes with separate picks; let every note stabilize before moving.
- Avoid double-stops, ringing open strings, harmonics, pick scrapes, hammer-on/pull-off flurries, bends, wide vibrato, and slides across pitch boundaries during acceptance.
- Do not let one string ring under the next. A sympathetic/open-string pitch is an input error here, not a richer color layer.
- If low notes trigger late or octave-displaced responses, move the same abstract cell up an octave rather than increasing density.

### 7. Keyboard opportunities and chord limits

Keyboard can make exact collection choices, velocities, octaves, and NoteOffs, so it is the clearest surface for comparing windows. Use one finger/key down at a time and no sustain pedal during acceptance. A performer may repeat one pitch with changed velocity or answer a cell an octave away. Physical chord limit is **one note** for this preset: two-key dyads and arpeggiated/rolled chords multiply the generated stack and make the source/output relationship hard to audit.

### 8. Failure gestures

- Chromatic runs or tonal scales outside the selected collection → the player contradicts the modal contract; the preset does not compose or police an authentic line.
- Rapid tremolo, scalar bursts, retriggers, or overlapping releases → note storms.
- Guitar double-stops/ringing strings or keyboard sustain pedal → multiplied mud and ambiguous ownership.
- Repeated low-register, high-velocity notes → congested sonority rather than a bounded color window.
- Functional dominant–tonic licks → impose a cadence the approximation does not generate or contextualize.
- Treating `AugmentedHex` as Messiaen Mode 3 → factual error: current hexatonic `[0,3,4,7,8,11]` is not the nine-note Mode 3.
- Expecting the preset to rotate scales/transpositions, choose special chords, add non-retrogradable rhythms, hear phrase boundaries, orchestrate timbres, or map objective visual colors → unsupported capability claims.

### 9. “Play it like” guidance

**Concise Play it like:** “Play a crisp 2–4-note cell from the shown collection, then leave a long gap. Repeat it once higher or stronger, change only one note, and listen to each generated sonority as a separate color window.”

**Expanded guidance:** Stay monophonic and in the middle register. Give each note a definite attack and complete release; after two to four notes, stop for 1.5–3 seconds so the generated stack clears. Restate the cell with one controlled contrast—octave, last collection degree, velocity, or spacing—then thin it to two notes and stop. Guitarists mute every unused string and avoid bends/slides; keyboardists use one key at a time with no sustain pedal. You create the motif, contrast, and form. Contrapunk only maps the current fixed symmetric collection and harmony relation.

## Concrete 42-second acceptance exercise

### Preconditions

1. Record tonic/key, selected scale, harmony mode, voice count/position, routing, sound, devices, and transport running/position state.
2. Load the preset without changing the protected device, sound, routing, tonic, BPM/meter, or transport context required by the phase contract.
3. The synthesis must publish **one** actual collection and one fixed voice relation. For this performer test, use an octatonic mapping if that is the chosen preset baseline and label its ordered degrees `1…8`; otherwise run the same degree pattern in the published whole-tone mapping. Do not use `AugmentedHex` as a Mode-3 oracle.
4. Keyboard: sustain off. Guitar: clean fretted signal, all unused strings damped.

### Input schedule

Wall-clock events from the first NoteOn; every note is released before the next:

- **Window A, present:** `00.0s degree 1 mid v60, off 01.2`; `02.0s degree 3 mid v64, off 03.2`; `04.0s degree 2 mid v68, off 05.8`; silence to `09.0s`.
- **Window B, refract:** `09.0s degree 1 one octave higher v82, off 10.0`; `10.7s degree 3 high v86, off 11.7`; `12.4s degree 4 high v92, off 14.4`; silence to `18.0s`.
- **Window C, compare:** `18.0s degree 1 mid v58, off 19.2`; `20.0s degree 3 mid v62, off 21.2`; `22.0s degree 2 mid v66, off 24.2`; silence to `28.0s`.
- **Window D, withdraw:** `28.0s degree 1 mid-low v52, off 30.0`; `31.0s degree 2 mid-low v46, off 34.0`; silence through `42.0s`.

The degree cell is abstract and quotes no melody.

### Pass criteria

- Each physical NoteOn creates only the declared fixed harmony relation in the selected collection; no collection/transposition changes occur autonomously.
- A and C have the same abstract `1–3–2` source cell and corresponding output relation. B differs only by octave/velocity and terminal degree `4`; D is a two-note contraction.
- The player can hear four separate windows because no prior input/harmony note remains active across the long rests. No autonomous notes appear during silence.
- B is perceptibly higher/stronger without becoming denser in input NoteOns; D is lower, softer, and sparser.
- After each ordinary NoteOff, its generated notes clear normally; at 42 seconds active and pending owned-note sets are empty. Panic after the natural-clear check is idempotent.
- Transport running/stopped state and protected context match the pre-test snapshot. The result passes whether transport began running or stopped.
- A guitar run has no false retriggers/octave jumps; if it does, retry the same cell higher and record the detector limitation rather than relaxing monophony.

## Current capability statement

**Can honestly do now:** select a fixed whole-tone collection (Messiaen Mode 1 equivalent) or octatonic/diminished collection (Mode 2 family/transposition determined by key and variant); generate a consistent scale-degree harmony for each monophonic NoteOn; preserve normal NoteOff ownership; let the player articulate register, velocity, duration, motif, contrast, and silence.

**Cannot honestly do now:** enact an automatic whole-tone/octatonic/other-mode timeline; detect and rotate formal “windows”; guarantee that arbitrary played pitches form idiomatic modal material; generate Messiaen’s specific special chords, inversions, resonance, pitch-timbre layers, color associations, rhythms, or orchestration; claim objective synesthetic colors; or represent Mode 3 with `AugmentedHex`. The two diminished orderings are useful octatonic mappings, not an automatic tour of all forms. Any final UI copy must say **Messiaen-derived symmetric color** or equivalent, never “plays like Messiaen.”

## Findings

1. **Best gesture** — a short, clearly attacked modal cell plus a long rest makes fixed harmony audible and puts form where it belongs: in the player’s actions. [Benitez](https://www.mtosmt.org/issues/mto.02.8.2/mto.02.8.2.benitez.html)
2. **Best temporal shape** — present → contrasted repeat → literal compare → contraction/silence; transport is irrelevant to the baseline mapper.
3. **Honesty boundary** — Mode 1 and Mode 2 families exist, but current `AugmentedHex` is not nine-note Mode 3 and no adaptive scale rotation is exposed. [Contrapunk config](https://github.com/contrapunk-audio/contrapunk/blob/main/crates/contrapunk-harmony/src/config.rs)
4. **Interaction safety** — one clean note at a time, definite releases, and substantial gaps are essential on guitar and useful on keyboard. [DAFx-10 guitar study](https://dafx10.iem.at/proceedings/papers/VonDemKnesebeckZoelzer_DAFx10_P102.pdf)

## Sources

### Kept

- Olivier Messiaen, *Technique de mon langage musical* (Leduc, 1944; English trans. 1956) — primary codification and selected-period scope. https://monoskop.org/images/5/50/Messiaen_Olivier_The_Technique_of_My_Musical_Language.pdf
- Philharmonie de Paris catalog record — authoritative publication metadata and scope. https://mediatheque.philharmoniedeparis.fr/mediatheque/doc/ALOES/0077526/technique-de-mon-langage-musical-texte-avec-exemples-musicaux-olivier-messiaen
- Anthony Pople, “Messiaen’s modes à transpositions limitées,” Cambridge appendix — authoritative explanation of limited transposition. https://doi.org/10.1017/CBO9781139166928.011
- Vincent P. Benitez, *Music Theory Online* 8/2 (2002) — peer-reviewed evidence for modal definitions, chord-centered sound-color, contrast, and the difference between selected early codification and a late opera’s layered practice. https://www.mtosmt.org/issues/mto.02.8.2/mto.02.8.2.benitez.html
- von dem Knesebeck & Zölzer, DAFx-10 (2010) — measured monophonic guitar-tracking limitations. https://dafx10.iem.at/proceedings/papers/VonDemKnesebeckZoelzer_DAFx10_P102.pdf
- Contrapunk `config.rs`, `engine.rs`, and adapter types — primary evidence for actual mappings and lack of adaptive rotation. https://github.com/contrapunk-audio/contrapunk/blob/main/crates/contrapunk-harmony/src/config.rs ; https://github.com/contrapunk-audio/contrapunk/blob/main/crates/contrapunk-harmony/src/engine.rs ; https://github.com/contrapunk-audio/contrapunk/blob/main/ui/src/lib/adapter/types.ts

### Dropped

- Wikipedia and commercial scale-reference pages — redundant and weaker than the treatise/Cambridge/MTO sources.
- Generic “play like Messiaen” teaching pages — they collapse a long, changing career into scale recipes.
- *Catalogue d’oiseaux* commentary — useful for another, birdsong-centered scope but unnecessary and potentially scope-widening here.
- Later-opera evidence as a direct performance template — retained only as an explicit contrast/honesty check, not copied into the preset behavior.

## Gaps

- Parent synthesis still must choose the exact fixed collection, key/transposition, harmony mode, voice count, register layout, and sound-independent velocity expectation before implementation acceptance is deterministic.
- This report did not run an empirical Contrapunk guitar session. Validate the 42-second exercise on every intended surface and record false onsets, octave errors, release latency, and velocity propagation.
- The public adapter does not itself prove how every surface treats off-collection input. Final guidance should display permitted pitch classes rather than promise quantization or rejection until behavior is tested.
