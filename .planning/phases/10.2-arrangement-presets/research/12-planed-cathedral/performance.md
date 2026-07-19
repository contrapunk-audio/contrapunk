# Research: Phase 10.2 preset 12 — Planed Cathedral (Debussy), Role C performer/HCI

**Independence and scope.** This report used only the required catalog/template, public Contrapunk adapter/config surfaces, and public sources; it did not read sibling preset research or another Planed Cathedral report. The performance reference is narrowly Debussy’s 1910 piano prelude *La cathédrale engloutie*, not Debussy’s whole output and not a claim of imitation. All web sources were retrieved **2026-07-15**.

## Summary

The cleanest live control is one sustained note at a time in the middle register, with soft rounded attacks, complete releases, and audible space; each NoteOn should expose one fixed parallel chord plane while the player—not the preset—creates a low/quiet → higher/louder → low/quiet emergence and submergence. Guitar should use clearly separated, preferably fretted single notes with damping; keyboard should remain one-finger monophonic but can shape register, velocity, and release much more exactly. Transport is optional because the current harmony baseline is stateless per note; a running pulse may orient the performer but must not be required or altered.

## Evidence-led interaction thesis

1. **The source score asks for an arc, not an endless static wash.** Debussy begins “Profondément calme (Dans une brume doucement sonore),” then marks “Peu à peu sortant de la brume,” “Augmentez progressivement (sans presser),” a powerful but non-harsh sonority, later a growing expression, and finally a floating/dull echo and return to the opening sonority. Those directions justify an interaction arc of **mist → emergence → broad peak → echo/submergence**, supplied here by the player’s register, velocity, duration, density, and silence. [Debussy, *Préludes, 1er livre*, Durand, 1910, no. 10 (first-edition scan)](https://vmirror.imslp.org/files/imglnks/usimg/0/0c/IMSLP521885-PMLP2394-Debussy_Claude-Pr%C3%A9ludes_1er_Livre_Durand_7687_scan.pdf) **Confidence: high.**
2. **Spatial/register change is structurally relevant.** Guigue’s scholarly study specifically treats the use of musical space as a compositional/formal strategy rather than relying only on pictorial metaphor. Therefore register migration should articulate the section arc, not be decorative randomness. [Didier Guigue, “Sonorité, Espace et Forme dans ‘La Cathédrale Engloutie’ de Debussy,” *Revista Música* 5/2 (1994), DOI 10.11606/rm.v5i2.55080](https://doi.org/10.11606/rm.v5i2.55080) **Confidence: high for register/form; medium for the precise live mapping, which is an interaction inference.**
3. **Do not equate Debussy performance with metronomic restraint.** Research on Debussy’s 1912 piano rolls challenges the modern assumption that his music must use minimal rubato and argues that his performance practice retained late-Romantic flexibility. This supports gently breathing attacks and rests, while the score’s “sans presser” still rules out accelerating into the climax. [Anna Scott, “Debussy and Late-Romantic Performing Practices: The Piano Rolls of 1912,” in *Debussy’s Resonance* (University of Rochester Press/Cambridge Core, 2018)](https://www.cambridge.org/core/books/debussys-resonance/debussy-and-lateromantic-performing-practices-the-piano-rolls-of-1912/D2A78D83FF795E3D43FDCDF204D487FE) **Confidence: medium-high; the chapter concerns Debussy’s recorded practice broadly, not a roll of this preset interaction.**
4. **Performance meaning emerges through shaping.** Kaminsky’s performance-analysis work stresses that Debussy’s structure is not exhausted by score symbols and that performer timing, dynamics, articulation, timbre, and shaping contribute to the heard structure. That supports requiring the user to shape sections rather than asking a static mapper to fake them. [Peter Kaminsky, “Listening to Performers’ Writings and Recordings,” *Music Theory Online* 22/3 (2016)](https://mtosmt.org/issues/mto.16.22.3/mto.16.22.3.kaminsky.html) **Confidence: medium-high; the case study is another Debussy work, so it supports method, not exact notes for this prelude.**
5. **Clean single notes are a real technical boundary for guitar audio.** A real-time guitar pitch-tracking comparison defines its target as dry electric-guitar single notes and reports octave jumps/frequency fluctuations on recorded guitar even when synthetic accuracy is good; YIN’s tested low-latency result was 13.4 ms without post-processing and 27.4 ms with it. This supports damping, stable fretted pitch, clear separation, and avoiding bends/vibrato/slides in acceptance tests. [A. von dem Knesebeck and U. Zölzer, “Comparison of Pitch Trackers for Real-Time Guitar Effects,” DAFx-10 (2010)](https://dafx10.iem.at/proceedings/papers/VonDemKnesebeckZoelzer_DAFx10_P102.pdf) **Confidence: high for the boundary; medium for exact behavior of Contrapunk’s different detector.**
6. **The current product surface can make static planes but does not expose an adaptive form engine.** `WholeTone` and pentatonic collections exist; `DiatonicThirds` adds fixed +2 scale degrees per voice and stacks multiple voices, while `DiatonicFourths` adds fixed +3 degrees. The adapters expose key/scale/mode/voice count, per-note velocity injection, NoteOff, panic, routing, transport, and audio state, but no phrase-intensity or automatic emergence/submergence parameter. [Contrapunk harmony config](https://github.com/contrapunk-audio/contrapunk/blob/main/crates/contrapunk-harmony/src/config.rs) and [public adapter contract](https://github.com/contrapunk-audio/contrapunk/blob/main/ui/src/lib/adapter/types.ts) **Confidence: high as of the inspected revision.**

## Performer contract

### 1. Best source gesture

- **Guitar:** one clean, stable, fretted note, held long enough for the generated plane to be heard, then deliberately muted before the next note. Prefer a new pick for each structural plane; use legato only if the detector produces an unambiguous NoteOff/NoteOn pair.
- **Keyboard:** one finger at a time, one sustained key per plane, complete key release before the next attack. Do **not** feed played chords: the preset already supplies the vertical plane.
- Use a small stepwise or gently arching line, not a tune that demands functional cadences. Occasional collection-degree skips are useful only to mark the peak or return.

### 2. Register, tempo, lengths, articulation, velocity, density, silence

- **Input register:** begin around degrees placed in the middle register (roughly C3–C5 equivalent). Guitar’s safest practical band is approximately D3–B4; avoid the lowest string during a checked audio-pitch test because low fundamentals increase observation time and ambiguity. Keyboard may rise one octave for the peak, then return.
- **Tempo/pacing:** slow to moderate, approximately **48–66 quarter-note pulses/minute** if the player wants a reference. Never accelerate toward the peak; breathe locally rather than quantizing every onset.
- **Note lengths:** normally **2–5 seconds**. A transition note may be 1–2 seconds; the peak may last 5–7 seconds. Release fully.
- **Silence:** leave **0.5–2 seconds** between planes and a longer 2–4 second breath before/after the peak. Silence is structural: it prevents accumulated releases from becoming an undifferentiated note storm.
- **Articulation:** soft, rounded, sustained, and connected at the phrase level but **not physically overlapped at the input**. Avoid staccato repeated-note tapping and hard accents.
- **Velocity/dynamic arc:** start about MIDI **35–50**, grow across several attacks to **80–100** at one broad peak, then withdraw through **55–40**. Velocity should change between planes; no claim is made that the baseline engine continuously crescendos a held chord.
- **Density:** one input NoteOn at a time; usually 3–6 planes per 15 seconds. At the climax, increase attack strength and register before shortening inter-plane silence slightly; do not increase input polyphony.

### 3. Transport dependency

**Not transport-dependent.** The baseline planing modes are stateless mappings of each input note. If transport is already running, use its existing pulse loosely; if stopped, play by breath/seconds. Loading or testing this preset must not start, stop, reset, retime, or remeter transport. A future beat-synchronized evolution feature would be a different capability and must be declared explicitly.

### 4. Player-supplied phrase and section evolution

- **Mist (opening):** 1–2 low/middle planes, pp, 3–5 seconds each, generous rests.
- **Emergence:** rise mostly by adjacent collection degrees; add velocity and slightly reduce silence over 2–4 planes.
- **Presence/peak:** reach the upper input register and make one or two longest, strongest planes; remain broad, never rushed or harsh.
- **Echo/submergence:** drop register, velocity, and event rate; use fewer notes and longer gaps; finish with a complete quiet release.

This is the essential distinction: **the generated planing is static** (the same configured scale-degree offsets are remapped at each NoteOn); **emergence/submergence is player-shaped** through register, velocity, time, duration, and silence. The preset must not claim that it listens for or autonomously creates that form.

### 5. Listening/responding instructions

After every attack, listen until the entire vertical plane is perceptible and stable. Do not add the next input merely because the key/string is available; wait for the plane’s attack to settle, decide whether it should feel nearer/brighter or farther/darker, then answer with the next register/velocity choice. At the climax, wait through the longest chord and its release before beginning the descent. After the final NoteOff, listen through at least two seconds of silence and confirm no generated note remains.

### 6. Guitar-specific limits

- Use a clean direct signal; mute all unplayed strings with both hands.
- Prefer fretted middle strings and separate picks. Avoid adjacent-string resonance, double-stops, natural harmonics, pick scrapes, hammer-on/pull-off flurries, slides across semitone boundaries, deep vibrato, bends, and letting the previous string ring.
- Slow release/muting must still produce one normal NoteOff. If an open string sustains sympathetically, stop and retry rather than treating the extra pitch as expressive texture.
- Guitar cannot independently realize the piano source’s very wide low/high spacing or pedaled multi-register resonance under a clean-monophonic contract; generated voicing and downstream sound may suggest space, but must not be advertised as pianistic replication.

### 7. Keyboard-specific opportunities and limits

- Keyboard gives precise velocity, duration, octave placement, clean NoteOff, and easy one-octave register migration; use those to make the formal arc legible.
- Keep chord density at exactly **one physical key down**. Avoid sustain/sostenuto pedal during the checked exercise because it can conceal release errors and overlap successive planes. Aftertouch/mod-wheel color is optional only if routed downstream and must not be required by the preset.
- A keyboardist may make a cleaner emergence by repeating one degree at increasing velocity before moving upward, but should avoid rapid tremolo or rolled chords.

### 8. Failure gestures

- Played triads/double-stops, sustain-pedal overlap, or uncleared ringing strings → multiplied chord density and mud.
- Fast scalar runs, repeated-note tremolo, or gaps shorter than the release → note storms rather than architectural planes.
- Hard, uniformly high velocities → no emergence and a harsh contradiction of “sonore sans dureté.”
- Strong leading-tone-to-tonic cadences or busy functional arpeggios → contradict the suspended, color-plane interaction target.
- Guitar bends/vibrato/slides/harmonics → detector fluctuations or octave errors; lowest-register notes may add latency.
- Expecting the preset to crescendo, orchestrate, pedal, change register, or submerge on its own → a capability mismatch.

### 9. Plain-language Play prompt

**Play it like:** “Hold one soft note at a time. Rise and grow until one broad, bright peak, then fall away into longer silences; release every note cleanly before the next.”

Expanded guidance: Start in the middle-low register with quiet 2–5 second tones. Let each generated chord settle before answering it. Over several attacks move upward and louder without speeding up, sustain one strong high point, then use fewer, softer, lower notes and finish in silence. Guitarists should damp every unused string; keyboardists should use one key and no sustain pedal.

## Deterministic 49-second acceptance exercise

### Preconditions

1. Snapshot **tonic/key, BPM, meter, selected input/output devices, per-voice routing, sound/synth, mix gains, plugin chain and plugin state, and complete transport state (running/stopped and position)** before loading/testing.
2. Use the preset’s declared collection. For a concrete baseline acceptance, the synthesis should choose and publish one collection; the recommended test oracle is **whole-tone with three-note planes at fixed offsets `{d, d+2, d+4}` in collection degrees** (octave-normalized). If synthesis instead chooses pentatonic or fourth-stack offsets, substitute its published fixed offset vector but do not weaken the parallel-motion assertions.
3. Use one monophonic source note at a time. Disable keyboard sustain. Guitar uses a clean, damped fretted tone.
4. Do not alter or start transport. Timing below is wall-clock timing from the first NoteOn.

### Input (abstract degrees only)

Each event is `time: degree/register, velocity; NoteOff time`:

- `00s: 1(mid-low), v38; off 05s`
- `07s: 2(mid-low), v48; off 12s`
- `14s: 3(mid), v62; off 19s`
- `21s: 5(mid-high), v82; off 27s`
- `29s: 6(high), v96; off 35s`
- `37s: 2(mid), v56; off 41s`
- `43s: 1(mid-low), v36; off 47s`
- `47–49s: silence`

No melody is quoted; degrees are relative to the preset collection. The exact wall-clock schedule makes the exercise deterministic even when transport is stopped or its preserved BPM/meter varies.

### Expected relations and evolution

- At every NoteOn of degree `d`, exactly one configured plane appears with the same ordered offset vector; for the recommended oracle it is `{d, d+2, d+4}` modulo the six-degree whole-tone collection, with only declared octave placement differences.
- When input moves by `+1`, every generated stratum also moves by `+1` collection degree; the move `3→5` translates every stratum by `+2`; no voice switches to contrary or functional resolution. Thus the relationship is **parallel/planed**, not merely “three notes that fit the scale.”
- Generated density remains constant per held input. No previous plane survives its corresponding normal NoteOff beyond the configured synth/release tail, and no autonomous planes appear during rests.
- The heard form is quiet/low/sparse at 0–14s, rises and intensifies through 14–35s, then retreats in register/velocity/density through 49s. This evolution comes from the listed player events; the chord generator itself remains static.
- Velocity should be passed through or audibly reflected where the selected surface/sound supports it. A surface that normalizes harmony velocity must disclose that limitation rather than claim a generated dynamic arc.

### Lifecycle and non-destructive checks

1. Verify every ordinary input NoteOff completes normally and clears its owned generated notes; after 49s the active input/harmony note sets are empty.
2. Invoke **Panic/All Notes Off** once after the natural-clear assertion; it must be idempotent and leave no note sounding.
3. Invoke **Stop routing** (not transport stop) and verify cleanup again. If the UI’s generic Stop also controls transport on a surface, first record the transport state and restore it exactly; such coupling is a surfaced limitation.
4. Compare the post-test snapshot with the pre-test snapshot: **tonic, BPM, meter, devices, routing, sound, mix, plugins, and transport running/stopped state and position policy must be preserved**. Loading the preset may change only its documented harmony/scale/voice parameters; it must not silently overwrite the protected context above.

## Honesty blockers

- **Blocker until synthesis chooses one exact collection and fixed plane vector.** Catalog wording “whole-tone or pentatonic” is not a deterministic operational definition; acceptance cannot assert exact relations until one is selected and disclosed.
- **Blocker if UI copy implies adaptive emergence/submergence.** Current public surfaces expose static note mapping, not phrase-intensity sensing, automatic gradual entries, orchestration, or section morphing. The arc is honest only when the Play prompt makes the player responsible.
- **Not a blocker if clearly bounded:** the preset can honestly offer Debussy-referenced parallel color planes and a performance prompt, but cannot claim to reproduce the original piano’s pedaling, layered registers, tempo/meter ambiguity, timbre, narrative, or full harmonic/formal design.

## Findings

1. **Best HCI mapping** — sparse monophonic sustained notes make each generated plane legible and give the player enough dimensions to shape a formal arc without new engine machinery. [Debussy score](https://vmirror.imslp.org/files/imglnks/usimg/0/0c/IMSLP521885-PMLP2394-Debussy_Claude-Pr%C3%A9ludes_1er_Livre_Durand_7687_scan.pdf)
2. **Static versus emergent behavior must be explicit** — fixed scale-degree stacking is available; phrase/section adaptation is not exposed. [Harmony config](https://github.com/contrapunk-audio/contrapunk/blob/main/crates/contrapunk-harmony/src/config.rs)
3. **Guitar acceptance must be stricter than MIDI-keyboard acceptance** — damped, stable single notes are necessary because real guitar pitch tracking can exhibit octave jumps and fluctuations. [DAFx-10 paper](https://dafx10.iem.at/proceedings/papers/VonDemKnesebeckZoelzer_DAFx10_P102.pdf)
4. **Transport should remain untouched** — the desired relation is immediate planing per NoteOn; time shapes interpretation but no beat-phase algorithm is required. [Adapter transport/config contract](https://github.com/contrapunk-audio/contrapunk/blob/main/ui/src/lib/adapter/types.ts)

## Sources

### Kept

- Claude Debussy, *Préludes pour piano, 1er livre*, Durand, 1910 — primary score and expressive/formal directions. https://vmirror.imslp.org/files/imglnks/usimg/0/0c/IMSLP521885-PMLP2394-Debussy_Claude-Pr%C3%A9ludes_1er_Livre_Durand_7687_scan.pdf
- Didier Guigue, “Sonorité, Espace et Forme…” (1994) — scholarly support for register/space as formal behavior rather than imagery alone. https://doi.org/10.11606/rm.v5i2.55080
- Anna Scott, “Debussy and Late-Romantic Performing Practices” (2018) — scholarly piano-roll evidence challenging rigidly metronomic/over-restrained Debussy performance. https://www.cambridge.org/core/books/debussys-resonance/debussy-and-lateromantic-performing-practices-the-piano-rolls-of-1912/D2A78D83FF795E3D43FDCDF204D487FE
- Peter Kaminsky, “Listening to Performers’ Writings and Recordings” (2016) — peer-reviewed performer/analysis method and evidence for shaping as structural interpretation. https://mtosmt.org/issues/mto.16.22.3/mto.16.22.3.kaminsky.html
- von dem Knesebeck & Zölzer, DAFx-10 (2010) — direct measured evidence on real-time monophonic dry-guitar pitch tracking, latency, and errors. https://dafx10.iem.at/proceedings/papers/VonDemKnesebeckZoelzer_DAFx10_P102.pdf
- Contrapunk public config/adapter sources — authoritative evidence for current collections, fixed interval modes, lifecycle, transport, and state surfaces. https://github.com/contrapunk-audio/contrapunk/blob/main/crates/contrapunk-harmony/src/config.rs ; https://github.com/contrapunk-audio/contrapunk/blob/main/ui/src/lib/adapter/types.ts

### Dropped

- Wikipedia — useful orientation but redundant once the primary score and scholarship were available.
- Commercial/teaching-site summaries and anonymous school PDFs — provenance or editorial authority was weaker than the retained score and journal/book sources.
- LSU thesis mirror — blocked by CAPTCHA during this run, so no claims rely on unseen content.
- Henle product page — fetched content did not expose the relevant editorial/performance note reliably.

## Gaps

- Exact operational choice between whole-tone and pentatonic, plane offset vector, voice count, octave placement, synth envelope, and velocity propagation belongs to parent synthesis; without it, the first honesty blocker remains.
- This was not an empirical usability session with Contrapunk guitar audio. Detector behavior may differ from the DAFx study; run the exercise on each shipping surface and record false onsets, octave errors, and NoteOff latency.
- Historical performance evidence supports flexible timing generally, but does not establish one mandatory BPM for this interaction. The proposed 48–66 pulse is guidance, not a musicological authenticity claim.
- Plugin-host preservation of transport position and third-party plugin state needs surface-specific verification; the public adapter intentionally gives the host authority over plugin transport.
