# HISTORY / PRIMARY-SOURCE RESEARCH — Preset 36 “Pixel Trio”

## Executive finding

“Pixel Trio” is defensible only as a **bounded arrangement abstraction**: three simultaneous *tonal roles*—lead, bass, and a subordinate counterline/accompaniment—shaped by the economy of selected Famicom/NES scores. It is not a literal model of the console, a generic “8-bit style,” or an imitation of Koji Kondo or Manami Matsumae. The stock NES audio unit has **five hardware channels** (two pulse, triangle, noise, and DMC sample playback), while the composers’ own shorthand of “three sounds/notes” refers to the practical pitched polyphony available for music; noise, samples, sound effects, software drivers, memory, and game-specific allocation complicate that shorthand. [NESdev APU documentation](https://www.nesdev.org/wiki/APU) [Kondo, Nintendo interview](https://www.nintendo.com/en-gb/Iwata-Asks/Iwata-Asks-Super-Mario-All-Stars/Vol-1-Super-Mario-History-Soundtrack-CD/4-Music-Commentary-by-Koji-Kondo-1-/4-Music-Commentary-by-Koji-Kondo-1--219986.html) [Matsumae, Brave Wave interview](https://bravewave.net/interviews/a-conversation-with-manami-matsumae/)

The historically useful intersection is economy, memorable foreground writing, role clarity, short-loop or short-motif construction, and music designed in relation to play. The two composers’ vocabularies should not be averaged: Kondo’s selected early work emphasizes movement-sensitive groove, location contrast, and the border between music and sound effect; Matsumae’s *Mega Man* emphasizes heroic, singable lead writing, chordal implication within three pitched parts, stronger pop/rock drum thinking, and stage/character differentiation.

---

## 1. Scope and corpus

### Referenced hardware and period

- **Base Famicom/NES APU**, especially mid-to-late 1980s cartridge practice: two pulse generators, one triangle generator, one noise generator, and one delta-modulation/DMC sample channel. Register-level documentation shows that the pulse channels have duty/envelope/sweep controls; triangle and noise have different control structures; DMC plays delta-coded samples. These are not five interchangeable pitched “voices.” [NESdev APU](https://www.nesdev.org/wiki/APU) [NESdev APU registers](https://www.nesdev.org/wiki/APU_registers)
- **Famicom Disk System is a qualified exception**, not the baseline. Kondo describes its additional freely definable waveform source and says the Disk System allowed a fourth sound; the overseas NES cartridge version did not have that source. [Nintendo, “Music Commentary by Koji Kondo”](https://www.nintendo.com/en-gb/Iwata-Asks/Iwata-Asks-Super-Mario-All-Stars/Vol-1-Super-Mario-History-Soundtrack-CD/4-Music-Commentary-by-Koji-Kondo-1-/4-Music-Commentary-by-Koji-Kondo-1--219986.html)

### Koji Kondo corpus

The preset may reference:

1. ***Super Mario Bros.* (Famicom, 1985)**—principally its location-specific cue set and Kondo’s documented process of revising the above-ground cue to fit running and jumping rather than scenery alone. Kondo calls it his second game. [Nintendo, “To Save Memory”](https://iwataasks.nintendo.com/interviews/wii/mario25th/4/4/) [Nintendo, *Ocarina of Time 3D* sound interview](https://www.nintendo.com/en-gb/Iwata-Asks/Iwata-Asks-The-Legend-of-Zelda-Ocarina-of-Time-3D/Vol-1-Sound/1-The-Ever-Changing-Music-of-Hyrule-Field/1-The-Ever-Changing-Music-of-Hyrule-Field-231234.html)
2. ***The Legend of Zelda* (Famicom Disk System, 1986)**—only as evidence of rapid project-specific differentiation and of an additional sound source, not as evidence for a stock three-role NES texture. Nintendo’s interview identifies it as Kondo’s third game and records that its development closely followed/overlapped *Super Mario Bros.* [Nintendo Classic Mini interview](https://www.nintendo.com/en-gb/News/2016/November/Nintendo-Classic-Mini-NES-special-interview-Volume-4-The-Legend-of-Zelda-1160048.html) [Nintendo, *Ocarina of Time 3D* sound interview](https://www.nintendo.com/en-gb/Iwata-Asks/Iwata-Asks-The-Legend-of-Zelda-Ocarina-of-Time-3D/Vol-1-Sound/1-The-Ever-Changing-Music-of-Hyrule-Field/1-The-Ever-Changing-Music-of-Hyrule-Field-231234.html)
3. ***Super Mario Bros. 3* and *Yume Kōjō: Doki Doki Panic/Super Mario Bros. 2*** only as later Famicom comparison points: increased cartridge capacity and deliberate DMC percussion changed the available repertoire. They must not be back-projected onto the 1985 score. [Nintendo, “Music Commentary by Koji Kondo”](https://www.nintendo.com/en-gb/Iwata-Asks/Iwata-Asks-Super-Mario-All-Stars/Vol-1-Super-Mario-History-Soundtrack-CD/4-Music-Commentary-by-Koji-Kondo-1-/4-Music-Commentary-by-Koji-Kondo-1--219986.html)

### Manami Matsumae corpus

The preset may reference:

1. ***Rockman/Mega Man* (Famicom/NES, 1987)**—the main Matsumae corpus: stage cues, boss/functional cues, ending, and sound effects. Matsumae says she composed both music and effects; Takashi Tateishi and Yoshihiro Sakaguchi identify Sakaguchi as sound-driver programmer. Original credits used aliases (“Chanchacorin Manami” and “Yuukichan’s Papa”), so role labels in the game are less precise than the later direct testimony. [Mega Man 1 & 2 sound-team interview](https://vgmonline.net/megamaninterview/) [VGMPF archival credit record](https://www.vgmpf.com/Wiki/index.php/Mega_Man_(NES))
2. **Matsumae’s pre-*Mega Man* training and first Capcom contribution**—classical piano/composition study and one classical-type cue for *Ide Yōsuke no Jissen Mahjong*—only to contextualize, not to enlarge the stylistic corpus. [Brave Wave interview](https://bravewave.net/interviews/a-conversation-with-manami-matsumae/)
3. **Her move to Capcom arcade projects and later freelance work** only to establish change and persistence. She names *Area 88* and *Carrier Air Wing* as especially congenial projects and says that expanded voices permitted more explicit melody and chords, while arcade audibility became a new problem. [VGMO career interview](https://vgmonline.net/manamimatsumaeinterview/)

### Explicit exclusions

- Kondo’s entire Mario/Zelda output is not the corpus.
- *Mega Man 2* is not a Matsumae score. Tateishi composed its BGM and effects; Matsumae supplied only a documented portion of the Air Man melody, while material from the first game was reused. [Mega Man 1 & 2 sound-team interview](https://vgmonline.net/megamaninterview/)
- Later chiptune, modern “retro” production, SID/Genesis/arcade hardware, and all early console music are not interchangeable with Famicom/NES practice.
- The report does not transcribe or reproduce protected melodies, game audio, ROM data, or samples.

---

## 2. Hardware truth versus the preset’s three-role abstraction

### What the hardware actually offers

The stock APU’s two pulse, triangle, noise, and DMC channels have unequal affordances. Two pulse channels can carry pitched lines with selectable duty cycles and envelope/sweep behavior. The triangle is pitched but timbrally and dynamically unlike a pulse channel. Noise is suited to unpitched events and percussion-like activity. DMC is sample playback, not a fourth general-purpose melodic oscillator. [NESdev APU](https://www.nesdev.org/wiki/APU) [NESdev APU registers](https://www.nesdev.org/wiki/APU_registers)

Thus, “three-channel music” is useful shorthand for **three ordinary pitched lines**, not a complete hardware inventory. Kondo says he could generate “three sounds at once” on Famicom; Matsumae speaks of “three sound channels plus noise” and of “three notes at a time.” Those statements are consistent once “sounds/notes” is read as musical pitched polyphony rather than a silicon block count. [Nintendo/Kondo](https://www.nintendo.com/en-gb/Iwata-Asks/Iwata-Asks-Super-Mario-All-Stars/Vol-1-Super-Mario-History-Soundtrack-CD/4-Music-Commentary-by-Koji-Kondo-1-/4-Music-Commentary-by-Koji-Kondo-1--219986.html) [CutCommon/Matsumae](https://www.cutcommonmag.com/mega-man-composer-manami-matsumae-talks-us-through-her-creative-process/) [Brave Wave/Matsumae](https://bravewave.net/interviews/a-conversation-with-manami-matsumae/)

### Why channel count does not dictate arrangement

1. **Music and effects compete.** Matsumae says one-note effects played over three-note music; she shortened and adjusted effects to avoid cutting the melody, but some effects still made melody notes disappear. [Brave Wave interview](https://bravewave.net/interviews/a-conversation-with-manami-matsumae/)
2. **Driver and data workflow matter.** Sakaguchi programmed the *Mega Man* sound driver; the composers entered data under severe limits. This means APU capability alone cannot prove what a particular driver, cue, or game used. [Mega Man sound-team interview](https://vgmonline.net/megamaninterview/)
3. **Repertoire changed with memory and peripherals.** Kondo says increased cartridge capacity enabled more songs and DMC percussion in later Mario work, while the Famicom Disk System added another waveform source. [Nintendo/Kondo](https://www.nintendo.com/en-gb/Iwata-Asks/Iwata-Asks-Super-Mario-All-Stars/Vol-1-Super-Mario-History-Soundtrack-CD/4-Music-Commentary-by-Koji-Kondo-1-/4-Music-Commentary-by-Koji-Kondo-1--219986.html)
4. **Individual cue usage varies.** An archival channel breakdown of “Elec Man Stage” reports pulse harmony/response, triangle bass after the opening, noise drums, and unused DMC. This is useful cue-level evidence, not permission to generalize every *Mega Man* cue. [VGMPF, “Elec Man Stage”](https://www.vgmpf.com/Wiki/index.php?title=Elec_Man_Stage)

### Product boundary

“Pixel Trio” should therefore expose **three tonal functions**, not pretend to emulate an APU:

- foreground melody supplied or strongly defined by the player;
- economical low bass/support;
- one bounded upper/middle counterline or accompaniment.

Noise percussion, DMC samples, pulse-duty timbre, hardware sweeps, FDS expansion audio, channel stealing, and authentic driver timing are outside an arrangement preset unless a separate sound engine explicitly implements them. The catalog’s sound-design separation makes this boundary desirable.

---

## 3. Historical and aesthetic context

Early home-console music had to remain intelligible through tiny memory budgets, repeated listening, game sound effects, and limited simultaneous pitched material. The music was not merely a miniature autonomous song: it differentiated places and states, energized repeated action, and coexisted with gameplay feedback.

Kondo’s *Super Mario Bros.* testimony is unusually direct. He first wrote an easy-going cue to match the blue sky and greenery, rejected it because it did not fit Mario’s running and jumping, then composed against the playable prototype. He retained a triplet-like noise/rhythmic figure from the rejected version because it suggested forward motion. He also made above-ground, underwater, underground, and castle areas musically different. [Nintendo, “To Save Memory”](https://iwataasks.nintendo.com/interviews/wii/mario25th/4/4/) [Nintendo, “Music Commentary”](https://www.nintendo.com/en-gb/Iwata-Asks/Iwata-Asks-Super-Mario-All-Stars/Vol-1-Super-Mario-History-Soundtrack-CD/4-Music-Commentary-by-Koji-Kondo-1-/4-Music-Commentary-by-Koji-Kondo-1--219986.html)

Matsumae approached *Mega Man* through a character and animation image: “cool and strong,” heroic like anime, with a lively, memorable melody. She knew stage layouts, tested balance in game, and differentiated upbeat stage material from the slower ending. Her classical keyboard formation helped with three/four-part thinking, but she separately studied then-current rhythm-oriented music because drums were initially unfamiliar to her. [Brave Wave interview](https://bravewave.net/interviews/a-conversation-with-manami-matsumae/) Her later account stresses repeating short motifs, catchy melody, and emotional differentiation—refreshing stage music versus tense boss music—under low memory and “three channels plus noise.” [CutCommon interview](https://www.cutcommonmag.com/mega-man-composer-manami-matsumae-talks-us-through-her-creative-process/)

Both therefore solve a common problem—identity and motion with limited resources—but not with one universal “NES grammar.”

---

## 4. Shared constraints versus distinct vocabulary

### Shared, historically supportable intersection

- **Three-note/pitched-part economy:** both explicitly describe working within three simultaneous sounds or notes. [Nintendo/Kondo](https://www.nintendo.com/en-gb/Iwata-Asks/Iwata-Asks-Super-Mario-All-Stars/Vol-1-Super-Mario-History-Soundtrack-CD/4-Music-Commentary-by-Koji-Kondo-1-/4-Music-Commentary-by-Koji-Kondo-1--219986.html) [VGMO/Matsumae](https://vgmonline.net/megamaninterview/)
- **Memorable foreground:** Kondo sought distinctiveness under restriction; Matsumae sought lively, easily remembered, singable melody. [Nintendo/Kondo](https://www.nintendo.com/en-gb/Iwata-Asks/Iwata-Asks-Super-Mario-All-Stars/Vol-1-Super-Mario-History-Soundtrack-CD/4-Music-Commentary-by-Koji-Kondo-1-/4-Music-Commentary-by-Koji-Kondo-1--219986.html) [Brave Wave/Matsumae](https://bravewave.net/interviews/a-conversation-with-manami-matsumae/)
- **Repetition with purpose:** short reusable data and loops are necessities, but repetition carries forward drive, recognizability, or state identity rather than being mere filler.
- **Game-specific contrast:** cues respond to location, character, stage, boss, ending, or player motion.
- **Effects are part of the composition problem:** they consume memory/channel attention and can function musically or interrupt music.

### Kondo-specific emphasis in the selected period

- **Kinetic fit:** rhythm is validated against movement. The decisive revision of the above-ground cue came from playing the prototype, not applying a genre recipe. [Nintendo, “To Save Memory”](https://iwataasks.nintendo.com/interviews/wii/mario25th/4/4/)
- **Layer friction as groove:** Kondo’s retained triplet-like noise figure does not simply make the melody swing with it; Nintendo colleagues describe the imperfect fit as distinctive. [Nintendo, “Music Commentary”](https://www.nintendo.com/en-gb/Iwata-Asks/Iwata-Asks-Super-Mario-All-Stars/Vol-1-Super-Mario-History-Soundtrack-CD/4-Music-Commentary-by-Koji-Kondo-1-/4-Music-Commentary-by-Koji-Kondo-1--219986.html)
- **Music/effect permeability:** the underground cue was praised as made “with sound effects” yet sounding musical; identical effects were reused in different game contexts to save memory. [Nintendo, “To Save Memory”](https://iwataasks.nintendo.com/interviews/wii/mario25th/4/4/)
- **Strong environmental cue families:** above ground, underwater, underground, and castle were deliberately distinguished rather than normalized into one timbre or harmony.

### Matsumae-specific emphasis in the selected period

- **Singable heroic line:** her brief was self-defined around a cool, strong, anime-like protagonist and a lively, easily retained lead. [Brave Wave interview](https://bravewave.net/interviews/a-conversation-with-manami-matsumae/)
- **Chordal implication:** she recalls continually asking how to create the feeling of chords with only three notes at once. [VGMO career interview](https://vgmonline.net/manamimatsumaeinterview/)
- **Pop/rock rhythmic study:** her rhythm vocabulary was learned separately through listening to contemporary acts; it should not be misdescribed as a simple consequence of classical training. [Brave Wave interview](https://bravewave.net/interviews/a-conversation-with-manami-matsumae/)
- **Cue-level emotional differentiation:** upbeat stage material, tense boss material, and a slower ending; stage layouts and robot imagery inform choices. [CutCommon interview](https://www.cutcommonmag.com/mega-man-composer-manami-matsumae-talks-us-through-her-creative-process/) [Brave Wave interview](https://bravewave.net/interviews/a-conversation-with-manami-matsumae/)

The coherent preset intersection is therefore **economical role clarity plus a memorable hook**. Kondo’s polymetric-feeling groove details and Matsumae’s fuller rock/drum identity are meaningful differences deliberately excluded from a baseline harmony preset.

---

## 5. Evolution through phrase, section, game, and career

### Within a phrase or loop

Primary interviews do not provide measure-by-measure analyses of every cue, so the following is a bounded inference from testimony and channel evidence:

- A memorable lead cell establishes identity.
- Bass/support and one answering or harmonizing part imply motion/chords without sustaining a full block texture.
- A rhythmic/noise layer may reinforce or cross-cut the lead, but it is not represented by the three tonal-role preset.
- Short repetitions remain legible because role, register, rhythm, or cadence changes prevent every beat from carrying the same vertical stack.

Cue-level archival evidence for “Elec Man Stage” supports role changes: the triangle participates in the opening before assuming bass duty, while pulse channels harmonize/respond rather than remaining one fixed chord pad. [VGMPF, “Elec Man Stage”](https://www.vgmpf.com/Wiki/index.php?title=Elec_Man_Stage) Confidence is **medium**, because this is one documented cue and not a corpus-wide score analysis.

### Across sections and a game

- **Kondo:** evolution is strongly spatial/interactive. Distinct area cues change the vocabulary at game boundaries. Within the above-ground cue’s history, an image-matching concept is discarded in favor of movement-matching groove. The underground treatment blurs effect and music. [Nintendo interviews](https://iwataasks.nintendo.com/interviews/wii/mario25th/4/4/)
- **Matsumae:** stage cues maintain energetic identity across repeated action; boss music raises tension; the ending lowers tempo and supplies contrast after an otherwise upbeat set. The score evolves through functional cue replacement, not through a single continuous long-form composition. [Brave Wave interview](https://bravewave.net/interviews/a-conversation-with-manami-matsumae/)

A preset cannot recreate location/state scoring from an arbitrary live monophonic input. It can only preserve the lower-level discipline: concise hook, separated roles, periodic thinning/answering, and a cadence or rest that resets attention.

### Career/style evolution

#### Kondo

The enduring concern is music’s relation to play, not perpetual three-channel writing. On the NES he learned game composition and the importance of effects; he later said improved hardware broadened sound quality while the essence of composing for games remained. [Nintendo Classic Mini Zelda interview](https://www.nintendo.com/en-gb/News/2016/November/Nintendo-Classic-Mini-NES-special-interview-Volume-4-The-Legend-of-Zelda-1160048.html) By *Ocarina of Time* (1998), Kondo used modular, situation-dependent Hyrule Field music that varied among normal, battle, and quiet states—an expansion from fixed short cue economy into dynamic form. [Nintendo, *Ocarina of Time 3D* sound interview](https://www.nintendo.com/en-gb/Iwata-Asks/Iwata-Asks-The-Legend-of-Zelda-Ocarina-of-Time-3D/Vol-1-Sound/1-The-Ever-Changing-Music-of-Hyrule-Field/1-The-Ever-Changing-Music-of-Hyrule-Field-231234.html) By *Super Mario Galaxy* (2007), Kondo served as adviser for the “Essence of Mario” and composed four pieces, while Mahito Yokota composed/arranged most of an orchestral score. This makes sole-author and fixed-timbre narratives untenable. [Nintendo, *Super Mario Galaxy* sound-team interview](https://www.nintendo.com/en-gb/Iwata-Asks/Iwata-Asks-Super-Mario-Galaxy/Volume-3-The-Sound-Team/1-Why-Use-an-Orchestra-/1-Why-Use-an-Orchestra-205026.html)

#### Matsumae

After *Mega Man*, Matsumae moved to arcade projects. More voices let her express melody and chords directly, while noisy arcade presentation required music that remained easy to hear. [VGMO career interview](https://vgmonline.net/manamimatsumaeinterview/) The persistent trait is memorable melody matched to characters/settings, not three-channel texture: she says she continued that melodic focus through later projects. In later retro commissions she sometimes deliberately returned to 8-bit means (*Mega Man 10*) or expanded original material with explicit chords (*Chiptuned Rockman*), showing that “retro” became a chosen project language rather than a permanent personal limitation. [CutCommon interview](https://www.cutcommonmag.com/mega-man-composer-manami-matsumae-talks-us-through-her-creative-process/) [VGMO career interview](https://vgmonline.net/manamimatsumaeinterview/)

**Preset period:** specifically 1985–87 Famicom/NES economy, with the *Zelda* Disk System difference disclosed rather than silently folded in.

---

## 6. Persistent traits versus period/project traits

| Claim | Persistent or bounded? | Evidence strength |
|---|---|---|
| Kondo prioritizes fit between music and gameplay | Persistent concern, demonstrated early and in later interactive work | High |
| Kondo always writes Latin/jazz music | False caricature; he says others described the first Mario cue that way, and later deliberately pursued other genre references | High |
| Kondo uses three tonal parts | Hardware/period condition, not career trait | High |
| Kondo favors location-specific contrast | Strong in selected Mario corpus; not asserted for every work | High |
| Matsumae prioritizes memorable melody | Persistent across her own retrospective accounts | High |
| Matsumae’s style is simply “rock” | Unsupported simplification; anime/heroic image, classical formation, contemporary rhythm study, and project imagery all matter | High |
| Matsumae implies harmony within few voices | Strong *Mega Man*-period problem, not a later-career limit | High |
| Three-channel texture defines either composer | False; it defines selected hardware practice | High |
| Noise/DMC percussion is interchangeable with a third tonal voice | False hardware model | High |

---

## 7. Relationship to peers and tradition

Limited polyphony, looped cues, channel sharing, compact motifs, register-separated roles, and timbral contrast are **shared Famicom/NES craft**, not signatures owned by Kondo or Matsumae. Scholarly work on “8-bit affordances” warns against treating technical limitation as one deterministic style: software and compositional choices create different responses to the same hardware. [James Cook, “8-Bit Affordances,” *Music Theory Online* 29.3](https://www.mtosmt.org/issues/mto.23.29.3/mto.23.29.3.cook.php)

Kondo’s unusual value to this preset is documented prototype-driven rhythmic revision and environmental differentiation. Matsumae’s is documented heroic/singable melody, chord implication, and pop-informed rhythm under the same broad pitched-note ceiling. Bach’s *Well-Tempered Clavier* was Matsumae’s own analogy for feeling comfortable with three/four parts; it is biographical evidence, not proof that *Mega Man* uses Bachian counterpoint. [Brave Wave interview](https://bravewave.net/interviews/a-conversation-with-manami-matsumae/)

---

## 8. Attribution and naming controls

1. Credit **Koji Kondo** for the selected early Nintendo music and sound-design testimony, but do not infer that he alone programmed every driver or created every later Mario/Zelda score.
2. Credit **Manami Matsumae** for *Mega Man*’s music and effects and **Yoshihiro Sakaguchi** for its sound driver, following direct participant testimony. [Mega Man sound-team interview](https://vgmonline.net/megamaninterview/)
3. Do not assign *Mega Man 2* wholesale to Matsumae. Tateishi’s account assigns its BGM/effects to himself, with a limited Matsumae contribution to Air Man and reused first-game material.
4. Preserve “Pixel Trio” as the public technique/image title. Composer names belong only in reference/context metadata, never as a claim of endorsement, authorship, or exact imitation.
5. “Early console writing” must be narrowed in copy to **selected Famicom/NES practice**; otherwise it incorrectly collapses unlike platforms and regional expansion-audio configurations.

---

## 9. Common caricatures and disallowed claims

### Caricatures to avoid

- “The NES had only three channels.” It had five unequal stock APU channels; three is a pitched-musical shorthand.
- “Chiptune means square-wave melody, triangle bass, noise drums.” That is a useful recurring allocation, not a universal law; drivers, cues, effects, DMC, and expansion audio vary.
- “Constraint automatically caused great melodies.” The sources show deliberate revision, testing, data entry, balancing, and project interpretation.
- “Kondo equals happy Latin/jazz Mario.” The selected corpus includes underwater, underground, castle, and Zelda material; Kondo himself treats genre labels cautiously.
- “Matsumae equals aggressive rock.” Her source language includes heroic anime-like memorability, chord implication, stage emotion, a slow ending, classical training, and later broader instrumentation.
- “Three voices should sound like constant triads.” Historical economy often requires independent rhythms, implied harmony, rests, role changes, and effects competition.
- “All old game music is fast, quantized, bright, and looped every few seconds.” The corpus contains slower and darker functional contrasts.

### Claims that product/release copy must not make

- “Authentic NES emulation,” “hardware-accurate,” or “uses the original channels,” unless a separately verified audio engine actually does so.
- “Koji Kondo preset,” “Manami Matsumae style,” “sounds exactly like,” “approved by,” or “the rules used by” either composer.
- Any implication that a generated melody is derived from *Super Mario Bros.*, *The Legend of Zelda*, or *Mega Man* themes.
- Any promise of noise percussion, DPCM drums, pulse-width/duty behavior, FDS wavetable audio, authentic tuning, channel stealing, or game-state adaptivity that the preset does not implement.
- Any statement that three parts alone define Famicom/NES music.
- Any universal claim about either composer’s whole career based on the selected early works.

---

## 10. Copyright-safe boundary

Safe implementation uses **unprotectable high-level constraints and independently generated relationships**: maximum tonal-role count, register separation, concise motif length, subordinate response behavior, periodic thinning, phrase rests, and broad functional contrast. It must not store, quote, transform, or pattern-match protected melodies, bass lines, rhythmic signatures, level cue sequences, samples, ROM/NSF data, or note-for-note channel transcriptions.

Abstract QA material should use neutral scale-degree cells invented for the test (for example, `1–2–5–3`) and generic beat positions. Even when a source offers downloadable audio or sheet music, preservation access is evidence, not a license for product reuse. Sound presets may recommend generic pulse/triangle/noise-inspired timbres, but arrangement metadata should not package copyrighted game audio.

---

## 11. Concrete, testable implications for synthesis and implementation

These are historical constraints for the parent synthesis, not a demand for new hardware emulation.

1. **Three-tonal-role ceiling:** for a monophonic input, output never presents more than lead/input, one low support role, and one counter/accompaniment role as the intended texture. Extra octaves or doublings must not masquerade as additional roles.
2. **Role audibility:** over an eight-beat test phrase, the input hook remains the perceptual foreground; bass stays below it; the counterline does not continuously double the lead at unison/octave.
3. **Economy, not block harmony:** at least one generated role rests, sustains, or answers during part of a normal phrase. Reject all-onset, all-voice triadic padding on every input note.
4. **Rhythmic identity preservation:** repeated input rhythm produces a stable, repeatable generated response. The preset does not randomize away a compact hook.
5. **Bounded answer:** the counterline should answer or fill selected gaps rather than run continuously. Dense legato input must not create an unbounded note storm.
6. **Bass clarity:** bass changes more slowly or more selectively than the lead and avoids crossing the counterline under ordinary register settings.
7. **Cadential thinning:** after a clear phrase-end rest or held destination, generated activity resolves, sustains, or stops; it must not invent a new extended theme.
8. **Contrast without quotation:** two abstract input cells with different contour/rhythm must yield perceptibly different arrangements while producing no recognizable source-game melody.
9. **No false percussion:** if the arrangement engine has no noise/DMC lane, applying Pixel Trio produces no hidden drum pattern and copy states that percussion/timbre are separate.
10. **Lifecycle safety:** NoteOff, Panic, preset replacement, and disabled lanes leave zero active or pending generated notes. Historical channel competition is not an excuse for stuck or dropped notes.
11. **Determinism:** a fixed event sequence and configuration produces the same pitch/role result on every run; game-era economy is represented by clear constraints, not nostalgic randomness.
12. **Acceptance wording:** UI Result should say approximately: “A compact lead is supported by economical bass and one answering tonal line.” It should not say “authentic NES,” “Kondo,” or “Matsumae style.”

### Minimal abstract acceptance cases

- **Case A — hook with gaps:** input scale degrees `1 (beat 1), 2 (&2), 5 (beat 3), rest beat 4`, repeated for two bars. Expect a lower support role with fewer changes and a counterline that occupies no more than selected gaps/held spaces; expect no fourth tonal layer.
- **Case B — held cadence:** input `3–4–2–1`, one beat each, final `1` held two beats. Expect support to settle or thin at the final hold and all generated notes to release with input NoteOff/Panic.
- **Case C — density guard:** eight rapid repeated notes followed by two beats of silence. Expect bounded generated events, lead intelligibility, and complete silence after releases—not continuous arpeggiation.

---

## 12. Confidence and gaps

### High confidence

- Stock APU channel types and their unequal functions.
- Kondo’s “three sounds” account, location differentiation, prototype-driven revision, and later use of DMC/FDS resources.
- Matsumae’s authorship of *Mega Man* music/effects, Sakaguchi’s driver role, three-note working description, melodic priority, and transition to less voice-constrained arcade work.
- The conclusion that “three tonal roles” is an abstraction rather than literal hardware emulation.
- Later-career evidence that neither composer can be reduced to the selected hardware constraint.

### Medium confidence

- The precise musical role allocation across the complete *Super Mario Bros.* and *Mega Man* corpora. Primary interviews describe process and limits, but do not publish full channel-by-channel scores.
- Generalizing the archived “Elec Man Stage” channel behavior beyond that cue.
- Within-loop state descriptions derived from listening/archival breakdown rather than composer-authored scores.

### Low confidence / unresolved

- Exact driver timing, voice-stealing priority, duty-cycle automation, DMC use, and channel allocation for every cue in the corpus.
- Region/version differences in individual releases without ROM-level comparative analysis.
- A complete, official track-by-track technical log for Kondo’s early Famicom scores.

### Recommended next evidence if exact emulation is ever proposed

Obtain legally accessible official scores or licensed multichannel stems; compare verified regional game versions; document each game’s driver and channel-stealing behavior; and commission a cue-by-cue score/channel analysis. None of that is necessary for the present bounded three-role arrangement abstraction.

---

## Sources

### Kept

- [Nintendo, “Music Commentary by Koji Kondo (1),” *Iwata Asks: Super Mario All-Stars*](https://www.nintendo.com/en-gb/Iwata-Asks/Iwata-Asks-Super-Mario-All-Stars/Vol-1-Super-Mario-History-Soundtrack-CD/4-Music-Commentary-by-Koji-Kondo-1-/4-Music-Commentary-by-Koji-Kondo-1--219986.html) — primary composer testimony on three sounds, location contrast, groove, FDS expansion audio, DMC percussion, memory growth, and stylistic change.
- [Nintendo, “To Save Memory,” *Iwata Asks: Super Mario Bros. 25th Anniversary*](https://iwataasks.nintendo.com/interviews/wii/mario25th/4/4/) — primary participant testimony on prototype testing, movement fit, effects/music overlap, and memory reuse.
- [Nintendo, *Nintendo Classic Mini: NES* — *The Legend of Zelda* interview](https://www.nintendo.com/en-gb/News/2016/November/Nintendo-Classic-Mini-NES-special-interview-Volume-4-The-Legend-of-Zelda-1160048.html) — official chronology, project differentiation, copyright anecdote, and Kondo’s retrospective on NES foundations.
- [Nintendo, *Ocarina of Time 3D* sound interview](https://www.nintendo.com/en-gb/Iwata-Asks/Iwata-Asks-The-Legend-of-Zelda-Ocarina-of-Time-3D/Vol-1-Sound/1-The-Ever-Changing-Music-of-Hyrule-Field/1-The-Ever-Changing-Music-of-Hyrule-Field-231234.html) — official evidence for early chronology, FDS fourth source, and later dynamic/modular evolution.
- [Nintendo, *Super Mario Galaxy* sound-team interview](https://www.nintendo.com/en-gb/Iwata-Asks/Iwata-Asks-Super-Mario-Galaxy/Volume-3-The-Sound-Team/1-Why-Use-an-Orchestra-/1-Why-Use-an-Orchestra-205026.html) — official evidence of later collaborative roles, orchestration, and Kondo’s adviser/four-piece contribution.
- [NESdev Wiki, “APU”](https://www.nesdev.org/wiki/APU) and [“APU registers”](https://www.nesdev.org/wiki/APU_registers) — authoritative community technical documentation for the five stock APU channels and their controls.
- [Brave Wave, “A Conversation with Manami Matsumae”](https://bravewave.net/interviews/a-conversation-with-manami-matsumae/) — direct composer testimony on training, first Capcom work, three channels, rhythm study, *Mega Man* concept, cue contrasts, effects, testing, attribution, and later return.
- [VGMO, “Mega Man 1 & 2 Sound Team Interview”](https://vgmonline.net/megamaninterview/) — direct testimony from Matsumae, Tateishi, and Sakaguchi on authorship, sound-driver roles, data limits, and the limited *Mega Man 2* crossover.
- [VGMO, “Manami Matsumae Interview: A Career After Mega Man”](https://vgmonline.net/manamimatsumaeinterview/) — direct testimony on chord implication, transition to arcade hardware, persistent melodic focus, and later retro arrangement choices.
- [CutCommon/Level and Gain, “Mega Man composer Manami Matsumae talks us through her creative process”](https://www.cutcommonmag.com/mega-man-composer-manami-matsumae-talks-us-through-her-creative-process/) — direct later testimony on three channels plus noise, short motifs, emotional cue functions, and modern 8-bit workflow.
- [VGMPF, *Mega Man (NES)*](https://www.vgmpf.com/Wiki/index.php/Mega_Man_(NES)) and [“Elec Man Stage”](https://www.vgmpf.com/Wiki/index.php?title=Elec_Man_Stage) — archival credit, release, NSF/score, and cue-level channel evidence; used cautiously because it is a preservation wiki, not an official technical log.
- [James Cook, “8-Bit Affordances,” *Music Theory Online* 29.3](https://www.mtosmt.org/issues/mto.23.29.3/mto.23.29.3.cook.php) — peer-reviewed scholarship supporting the distinction between shared hardware affordances and composer/game-specific responses.

### Dropped or not relied upon

- General Wikipedia, fan “best soundtrack” lists, YouTube uploads, cover/remix pages, and SEO biographies — insufficient authority and unnecessary when direct testimony/archives exist.
- VGMO’s standalone composer profile and general games-press profiles — mostly redundant with the retained direct interviews.
- Andrew Schartmann’s publisher page for *Analyzing NES Music* — relevant scholarship, but the available page supplies a book description rather than inspectable argument or cue evidence.
- Secondary claims that the NES had “only three channels” — dropped as technically imprecise.
- Similarity claims involving “Elec Man Stage” and a commercial pop song — irrelevant to the preset and risky as a shortcut; Matsumae’s reported denial of influence does not establish a useful compositional rule.
- Unofficial Kondo interview translations where the same claim was available from Nintendo — excluded in favor of official-hosted testimony.