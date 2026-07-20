## Review

- **Blocker — citation metadata is not fully traceable.** `research/01-cloister-organum/theory.md` misattributes:
  - “Who ‘Made’ the *Magnus Liber*?” to Catherine Bradley; the cited DOI is Edward H. Roesner’s article, correctly identified in `history.md`.
  - “Organum at Notre-Dame…” to Mary E. Wolinski; the cited DOI is Guillaume Gross’s article, correctly identified in `history.md`.
  
  Correct these before the research gate passes.

- **Correct — template coverage is strong.** All three reports cover their required sections, temporal development, uncertainty, abstract examples, performer behavior, and implementation gaps. `history.md` §§3–5 clearly distinguishes the Léonin-associated duplum layer, Pérotin-associated revision/multivoice practice, their documented intersection, and deliberately excluded differences.

- **Correct — claims avoid caricature.** The reports consistently reject “medieval music equals parallel fifths,” static power-chord stacks, symmetric voices around chant, exact attribution, authentic tuning, and MIDI reconstruction claims. Manuscript, treatise, institutional, and scholarly sources generally support the bounded Notre-Dame framing.

- **Correct with constraint — the generic Explicit Interval Map is architecturally appropriate.** A reusable, anchor-relative degree-to-semitone map belongs in shared `HarmonyEngine` configuration and can serialize through `ArrangementPresetV2`. This matches `theory.md` §§16–17 and avoids a preset-specific medieval algorithm.

- **Blocker if absent from the synthesis/config — the Cloister payload must not be a permanent `[fourth, fifth, octave]` fanout.** All three reports reject that shortcut. The bounded baseline should normally add one explicitly mapped upper voice, vary the selected relationship by source degree/state, and remain tenor-relative rather than chaining each generated voice from the previous one. Any simultaneous fixed `[+5,+7,+12]` stack on every NoteOn would contradict the pack despite using a generically named facility.

- **Note — the C Dorian oracle must test diatonic conversion, not fixed semitone aliases.** For degree offsets `+3/+4/+7`, the semitone results vary by source degree: the fourth above E♭ is 6 semitones and the fifth above A is 6. The oracle should cover all seven degrees, octave wrapping, tonic transposition, and off-scale-input policy. The Cloister preset should avoid structurally sustaining those tritone cases rather than pretending every diatonic fourth/fifth is perfect.

- **Correct — UI/capability requirements are feasible.** If user-tunable, the map needs a focused Setup editor, typed adapter/store round-trip, Apply reflection, and Save As capture. Unsupported surfaces must capability-gate it. Harmony mutation must also follow existing panic/reharmonization handling, while NoteOff releases every mapped output.

- **Note — copy must retain the bounded claim.** The implementation is synchronous open-interval harmonization controlled temporally by performer duration and rests. It must not claim generated florid duplum, discant, clausulae, or Pérotin-style independent polyphony.

**REVISE**