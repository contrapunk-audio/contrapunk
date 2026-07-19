# Research: Phase 10.2 Preset 02 “Modal Linework” — Performer/Interaction Report

**Role:** C — independent performer and interaction research  
**Reference boundary:** Palestrina and late-sixteenth-century four-part vocal polyphony are references, not an exact-imitation claim. This report gives a bounded way to drive Contrapunk; it does not claim to recreate historical singers, tuning, text underlay, acoustics, or a particular composition.  
**Retrieved:** 2026-07-18

## Summary

A player should contribute one singable modal line at a time: mostly stepwise, gently arched, connected but with unambiguous note releases, and separated into short breath-shaped phrases. A moderate, steady pulse and deliberate rests let Contrapunk’s four generated lines remain audible as lines rather than a succession of keyboard blocks; chords, rapid arpeggios, hard accents, and uncleared guitar resonance work against both the product’s monophonic contract and the referenced texture.

## Play it like…

> **Play it like one calm singer in a four-part choir: trace a mostly stepwise modal arc, breathe after each short phrase, and leave enough space to hear the other lines settle.**

## Evidence boundary

Labels used below:

- **Historical/score evidence** means the claim is supported by a sixteenth-century source, a score, or modern scholarship/corpus pedagogy about the repertory.
- **Interaction-design inference** means the recommendation translates that evidence into a reliable real-time MIDI gesture. Numbers such as BPM, velocity, milliseconds, and phrase duration are product-test values, not historical facts.

The CPDL record identifies *Sicut cervus* as a four-voice, a-cappella motet first published in 1587; its score resources include original-note-value and barless editions. The score is useful as evidence of independently entering and resting parts, not as material to copy. [CPDL work record and editions](https://www.cpdl.org/wiki/index.php/Sicut_cervus_(Giovanni_Pierluigi_da_Palestrina)) (retrieved 2026-07-18; **high** confidence for publication, scoring, and visible score features).

Open Music Theory’s corpus-oriented summary distinguishes sixteenth-century contrapuntal practice from later species exercises. It documents predominantly conjunct lines, compensating motion after larger leaps, arch-like contours, slower motion at phrase boundaries, metrical placement of imitation, overlap between phrases, and controlled rather than free dissonance. [Open Music Theory, “16th-Century Contrapuntal Style”](https://pressbooks.nebraska.edu/openmusictheory/chapter/16th-century-contrapuntal-style/) (retrieved 2026-07-18; **medium-high** confidence: scholarly teaching synthesis rather than a primary source).

Zarlino’s 1558 *Le istitutioni harmoniche*, Part III, is contemporary evidence that counterpoint was conceived through the relationships and conduct of parts, not modern chord-symbol thinking. It is contextual evidence, not proof of one mandatory Palestrina performance. [Thesaurus Musicarum Italicarum critical text](https://tmiweb.science.uu.nl/text/critical-edition/zarins58.html) (retrieved 2026-07-18; **high** confidence as primary-source text, **medium** for direct transfer to this preset).

Cambridge’s performance-history chapter warns against treating Renaissance vocal practice as one uniform modern sound: late-Renaissance church and chamber practices differed, including loud versus modulated delivery. Therefore this report does not prescribe “emotionless,” uniformly soft singing as historically authentic. [McGee, “Vocal performance in the Renaissance,” Cambridge History of Musical Performance, DOI](https://doi.org/10.1017/CHOL9780521896115.014) (retrieved 2026-07-18; **high** confidence for the stated survey conclusion; full chapter was access-restricted).

## Performer contract

### 1. Best source gesture

**Use single notes forming a short, singable line.** Start with a 4–7 note modal gesture rather than a chord, ostinato, lick, or long featureless drone. Favor adjacent scale degrees; one third or fourth can provide shape, but turn back toward the middle after a larger leap. Avoid arpeggiating a triad as the basic identity of the phrase.

- **Historical/score evidence — medium-high:** conjunct motion predominates; large leaps are normally balanced by contrary motion; arch contours and slower phrase boundaries are common in the cited repertory synthesis.
- **Interaction-design inference — high:** one clean NoteOn at a time is the catalog’s default, and each input note already fans out through the engine. A chord would ask several four-voice responses to coexist.

### 2. Register

Use the **middle of the instrument**, approximately MIDI **55–72 (G3–C5)**, narrowing further to a comfortable octave when possible. Begin near the center, let one modest high point occur, then return. Treat MIDI 55–67 as the safest shared guitar/keyboard zone if the resulting bass or soprano reaches an uncomfortable output extreme.

- **Historical/score evidence — medium:** individual sixteenth-century lines commonly occupy about an octave to a ninth/major ninth; the cited synthesis gives roughly 14 semitones as common and a twelfth as an unusual ceiling.
- **Interaction-design inference — medium-high:** a centered input gives the four-voice arrangement room above and below. Exact safe bounds depend on configured voice position and octave mode.

### 3. Tempo and rhythmic precision

Start at **60–84 BPM in 4/4**, with **72 BPM** as the acceptance-test default. Think in a broad, even pulse: most attacks on beats, occasional half-beat motion only as a light continuation, and no machine-gun subdivision.

Transport is **helpful but not intrinsically required** for this baseline gesture. Start it when available so phrase lengths and rests are reproducible. Do not quantize every note so aggressively that the line becomes detached; aim for attacks within roughly **±60 ms** of the intended beat in the acceptance exercise. If a configuration uses beat-aware Counterpoint Species 2–4, transport phase changes the behavior and a running transport becomes operationally important.

- **Historical evidence — medium-high:** the original repertory may lack modern barlines but retains metrical organization; imitation commonly enters at comparable metrical positions, and quicker motion normally begins in weak positions.
- **Interaction-design inference — high:** the local Tauri router forwards running transport beat position to the harmony engine; when stopped it sends `None`. The engine has a synthetic fallback for Species 2–4, but external phase is the reproducible test clock (`src-tauri/src/commands/engine.rs`, `run_tauri_router`; `crates/contrapunk-harmony/src/engine.rs`, `set_counterpoint_beat_phase` and `effective_counterpoint_beat_phase`). BPM and tolerance are test choices, not historical tempo claims.

### 4. Note lengths and articulation

Use a mixture of **one-beat and two-beat notes**, with an occasional three- or four-beat destination. Make the line sound legato without creating overlapping inputs: on keyboard, release the old key just before the next attack (about **20–60 ms** is a useful target); on guitar, mute the old string as the new pitch speaks. End every phrase with a full release, then rest for **1–2 beats**.

Do not use continuous staccato. A few lightly separated notes can clarify a new phrase, but repeated clipped eighth notes turn four generated voices into a note storm.

- **Historical/score evidence — medium-high:** slower motion often frames phrase openings/endings, and score evidence shows rests and staggered entries. “Legato with a 20–60 ms release gap” is not a historical prescription.
- **Interaction-design inference — high:** the engine maps every input NoteOn to a stored set of generated notes and relies on the corresponding NoteOff to release that set (`HarmonyEngine::harmonize_note_on/off`). Clean releases are therefore part of the musical and lifecycle contract.

### 5. Dynamics and velocity

Use **moderate, even velocity, roughly 55–80**, with a small arc of at most 10–15 velocity points across a phrase. Let phrase high points become slightly more present, then withdraw. Avoid alternating whisper/hard-strike attacks and avoid treating every tactus as an accent.

- **Historical evidence — low-medium:** surviving notation does not encode modern MIDI velocity, and the Cambridge survey explicitly cautions against one uniform Renaissance delivery.
- **Interaction-design inference — medium-high:** restrained velocity keeps the input from masking three generated parts. The numeric range is a practical default only; instrument patches differ.

### 6. Density and silence

Target **4–7 attacks over 6–10 seconds**, then rest **1–2 beats**. Across a 30–60 second passage, keep the input silent for roughly **20–30%** of the time. One longer held destination per phrase is enough; constant sustain obscures inner movement, while nonstop short notes multiply the output density.

- **Historical/score evidence — medium-high:** four-part motet scores show staggered entries, rests, overlapping points, and differentiated part activity rather than four parts attacking every syllable together.
- **Interaction-design inference — high:** each attack can produce up to four simultaneous arrangement voices. Sparse input is the simplest density control already available.

### 7. Phrase and section development

Use a small three-stage arc rather than arbitrary noodling:

1. **Offer (0–12 s):** 4–5 mostly adjacent notes; establish a center and stop.
2. **Answer/develop (12–30 s):** reuse the contour, beginning on a different scale degree or changing one interval/rhythm; permit one balanced fourth; reach the register high point.
3. **Settle (30–45 s):** reduce to 3–4 notes, lengthen the last note, return toward the central/final degree, and leave a longer rest.

This is not literal Renaissance imitation: the present baseline reacts note-by-note and does not necessarily capture a motif for delayed entry. It is an interaction analogue of offering related material, overlapping activity, and then thinning toward a cadence.

- **Historical evidence — medium-high:** points of imitation often begin movements/sections, may be rhythmically or melodically varied, overlap a following point, and become freer after their recognizable opening.
- **Interaction-design inference — high:** reuse contour rather than a copyrighted melody; the performer supplies temporal development the preset cannot infer from arbitrary input.

### 8. Listening and responding

After the first 2–3 attacks, listen specifically for the **nearest generated inner voice**: is it moving smoothly or bunching against the input? At each phrase rest, wait until the release is audible and visually clear before beginning the next phrase. If the texture sounds crowded, do not “correct” it by adding notes; lengthen the next note or leave another beat of rest. If a generated voice jumps register, move the next input by step toward the middle rather than chasing the jump.

- **Historical evidence — medium:** independent parts are understood relationally, and Zarlino’s framework treats counterpoint as inter-part conduct.
- **Interaction-design inference — high:** silence creates the only reliable real-time inspection window for a synchronous four-voice response.

## Instrument-specific guidance

### Clean-monophonic guitar

- Use a neck position where a 7–10 note range can be played with minimal string crossing.
- Prefer fretted notes with firm attacks and prompt left- or right-hand muting. Let only one pitch ring.
- Slides, hammer-ons, pull-offs, wide bends, and deep vibrato can generate pitch-bend or new-note behavior in the local detector; use them only if calibrated and verified. For this preset’s acceptance test, disable expressive pitch changes or keep them too small to cross a semitone.
- Avoid open strings when they will sympathetically ring across the next note. Avoid double-stops, rake attacks, pick noise that causes false onsets, and overlapping sustain between strings.
- Leave slightly more air around repeated notes so the detector receives a clear decay/re-onset.

**Operational basis — high confidence:** the local guitar pipeline uses onset detection, attack confirmation, a sustain/decay state machine, McLeod pitch detection, and optional legato/slide/vibrato and bend tracking; it emits MIDI NoteOn/NoteOff/PitchBend events (`crates/contrapunk-audio/src/guitar_input.rs`). The bridge sends All Notes Off on overflow/teardown (`src-tauri/src/guitar_bridge.rs`). The specific playing remedies above are interaction-design inference, not historical guitar practice.

### Keyboard

- Play one finger at a time. Use finger substitution or a tiny release gap for connected sound; **do not use sustain pedal** in the acceptance test.
- Velocity can shape a gentle phrase arc more repeatably than on detected guitar input.
- Keyboard range makes it easy to test the same contour in alto/tenor versus soprano positions, but change register only between phrases.
- **Chord-density limit: one input note.** This preset does not opt into chord input. Even a dyad temporarily doubles the engine requests and undermines the “one singer” role; a 3–4 note chord is a failure gesture, not an opportunity.

**Operational basis — high confidence:** the engine generates a configured multi-voice arrangement outward from the user’s voice position and stores active output per input pitch (`crates/contrapunk-harmony/src/engine.rs`, `harmonize`, `harmonize_note_on/off`). The no-chord rule follows the catalog’s explicit clean-monophonic default.

## Failure gestures

| Gesture | Observable failure | Why it fails |
|---|---|---|
| Held dyad/chord or sustain-pedal smear | Eight or more requested/generated note events may coexist; block-like or muddy result | Violates the preset’s monophonic source role; each input is independently harmonized |
| Fast scalar run or repeated 16ths | Note storm, masked voice leading, unreliable guitar attacks | Four-part output multiplies input density |
| Unmuted guitar string crossing/double-stop | False or overlapping NoteOns; stale-sounding harmony until release | Pitch/onset detector and active-note map need a clean lifecycle |
| Repeated large leaps in one direction | Disconnected registral zigzag; generated voices may bunch or jump | Contradicts the predominantly conjunct, balanced contour evidence |
| Triad-outline arpeggio as every phrase | Sounds chord-driven rather than like independent modal lines | Referenced practice is organized as interacting lines, not modern chord-symbol accompaniment |
| Hard accent on every beat | March-like vertical emphasis; inner lines become accompaniment | Metrical clarity does not imply identical attack stress |
| No phrase rests | No listening window, no audible breath, accumulating density | Removes the performer’s simplest interaction feedback mechanism |
| Changing key/mode/voice count while notes are held | Reharmonization event complicates evaluation even though cleanup exists | Acceptance should isolate playing behavior from parameter mutation |

## 45-second acceptance exercise

**Setup:** Modal Linework, four voices, a diatonic mode and tonic that place all test degrees in range, input in an inner voice position if configurable, no sustain pedal, no octave Mirror mode, transport at **72 BPM, 4/4**. Guitarist uses a calibrated clean input and mutes unused strings. Use abstract scale degrees, not a Palestrina melody.

1. **0–4 s — lifecycle check:** play degree **1** in the middle register for two beats; release; rest two beats.
2. **4–16 s — offer:** play **1–2–3–4–3** as quarter, quarter, half, quarter, half; make a small velocity swell; release; rest two beats.
3. **16–30 s — developed answer:** begin on **2** and use **2–3–5–4–3–2**; the fourth (`3→5` in scale-degree distance is a third, or substitute one diatonic fourth if comfortable) must reverse direction immediately; use mostly quarter/half notes; release; rest one full bar while listening to the generated inner voice and confirming silence.
4. **30–40 s — settle:** play **3–2–1**, one beat, one beat, four beats, reducing velocity slightly into the last note.
5. **40–45 s — cleanup:** release everything and touch nothing. Observe the piano/active-note display and listen.

### Pass evidence

All of the following must be observable:

- At no point are two source/input notes shown as held simultaneously.
- Each phrase is recognizably connected, mostly stepwise, and followed by its specified silence.
- The generated texture remains bounded to the configured four arrangement voices for a single input event; no burst attributable to a chord, false guitar onset, or pedal overlap occurs.
- Attacks intended on beats are within approximately ±60 ms of the 72-BPM grid (manual DAW/MIDI monitor inspection is sufficient).
- The second phrase is related but not identical to the first, reaches the session’s modest high point, and reverses after its largest leap.
- During the full-bar listening rest and final five seconds, all source and generated notes release.
- **Cleanup gate:** by the end, active input and harmony indicators are empty and there is no audible stuck note. A MIDI monitor should show a NoteOff corresponding to every source NoteOn (or an equivalent All Notes Off during an explicit stop).

### Fail evidence

Fail if any source overlap/chord appears, more than one unplanned onset occurs on guitar, rests are filled by new attacks, the line repeatedly leaps without compensation, a generated note remains active after the final five-second cleanup window, or the test requires panic/transport stop to silence an ordinarily released phrase. Panic/All Notes Off proves emergency cleanup, not normal lifecycle correctness.

## Findings

1. **The player must behave as one voice, not as a chord source.** This follows both the accepted catalog contract and the engine’s per-note fan-out. **Interaction inference; high confidence.**
2. **Mostly conjunct, balanced contours are the strongest historically grounded source gesture.** They preserve line identity while leaving generated voices room to move. [Open Music Theory](https://pressbooks.nebraska.edu/openmusictheory/chapter/16th-century-contrapuntal-style/) **Historical synthesis; medium-high confidence.**
3. **Rests are structural and operational.** Scores expose staggered entries/rests; in the product, rests also reveal whether NoteOff cleanup and texture release work. [CPDL score record](https://www.cpdl.org/wiki/index.php/Sicut_cervus_(Giovanni_Pierluigi_da_Palestrina)) **Score evidence medium-high; interaction inference high.**
4. **A pulse should organize entries without turning the line into rigid block chords.** Metrical correspondence is documented, while the exact 60–84 BPM recommendation is only a reproducible product default. [Open Music Theory](https://pressbooks.nebraska.edu/openmusictheory/chapter/16th-century-contrapuntal-style/) **Historical medium-high; numeric inference medium.**
5. **There is no defensible single “authentic” MIDI dynamic.** Late-Renaissance performance differed by setting; use restrained velocity as a mix strategy, not a historical claim. [Cambridge DOI](https://doi.org/10.1017/CHOL9780521896115.014) **Historical high for variability; velocity inference medium-high.**
6. **Normal NoteOff behavior is part of musical acceptance.** The engine stores generated notes against source pitches, the router dispatches their releases, and teardown additionally clears notes. **Local-code evidence; high confidence.**

## Sources

### Kept

- [Open Music Theory — “16th-Century Contrapuntal Style”](https://pressbooks.nebraska.edu/openmusictheory/chapter/16th-century-contrapuntal-style/) — corpus-oriented, testable guidance on contour, range, rhythm, imitation, phrase boundaries, and dissonance; retrieved 2026-07-18.
- [CPDL — *Sicut cervus* work record and score editions](https://www.cpdl.org/wiki/index.php/Sicut_cervus_(Giovanni_Pierluigi_da_Palestrina)) — stable public score index confirming four voices, a-cappella medium, 1587 publication, original-value/barless editions, and visible rests/entries; retrieved 2026-07-18.
- [Thesaurus Musicarum Italicarum — Zarlino, *Le istitutioni harmoniche* (1558), critical text](https://tmiweb.science.uu.nl/text/critical-edition/zarins58.html) — contemporary primary context for part-based counterpoint; retrieved 2026-07-18.
- [Timothy McGee, “Vocal performance in the Renaissance,” Cambridge History of Musical Performance](https://doi.org/10.1017/CHOL9780521896115.014) — authoritative survey summary supporting variability of vocal practice and caution against one modernized “authentic” dynamic; retrieved 2026-07-18.
- Local code: `crates/contrapunk-harmony/src/engine.rs`, `crates/contrapunk-audio/src/guitar_input.rs`, `src-tauri/src/guitar_bridge.rs`, and the minimum router sections of `src-tauri/src/commands/engine.rs` — operational evidence for per-note fan-out, active-note release, transport phase, guitar onset/pitch lifecycle, and emergency cleanup.

### Dropped

- Academia.edu/Scribd reposts of counterpoint books — unstable or access-mediated copies; stronger institutional/critical links were available.
- General choir blogs and unsourced “Palestrina style” summaries — insufficient authority and prone to prescribing a single modern choral sound.
- Modern performances/recordings — interpretation is useful for auditioning but cannot by itself prove historical articulation, dynamics, tempo, or ensemble size.
- Existing project theory report — deliberately not read, preserving the requested independent perspective.

## Gaps and residual uncertainty

- The exact preset configuration was not part of the allowed reading set, so safe register still depends on final voice position, octave mode, synth patch, and output transposition.
- The Cambridge chapter’s full text was access-restricted; only its publisher-supplied summary was used.
- Historical tactus does not map to one modern BPM. The proposed range is deliberately a usability/test range, not an authenticity claim.
- MIDI velocity, millisecond legato gaps, and a ±60 ms timing tolerance have no direct historical equivalent.
- The current baseline is synchronous note harmonization; true delayed imitation, text-driven phrasing, historical tuning, singer breathing, and room acoustics are outside this performer contract.
- Guitar detector results vary with instrument, pickup, gain, muting, calibration, and room noise. The acceptance exercise must be run on both clean MIDI keyboard and calibrated guitar before implementation can be called broadly performer-safe.

```acceptance-report
{
  "criteriaSatisfied": [
    {
      "id": "criterion-1",
      "status": "satisfied",
      "evidence": "Created only the requested standalone Role C report at /tmp/contrapunk-modal-linework-performance.md; no project or planning files were modified, and the existing theory report was not read."
    },
    {
      "id": "criterion-2",
      "status": "satisfied",
      "evidence": "Report includes source gesture, register, tempo, lengths, articulation, dynamics, density, rests, transport, development, listening, guitar/keyboard limits, failure gestures, confidence labels, evidence/inference separation, sources/gaps, and a 45-second observable pass/fail exercise with NoteOff cleanup."
    }
  ],
  "changedFiles": [
    "/tmp/contrapunk-modal-linework-performance.md"
  ],
  "testsAddedOrUpdated": [],
  "commandsRun": [
    {
      "command": "read allowed preset catalog and Role C research template",
      "result": "passed",
      "summary": "Confirmed preset 02 scope, monophonic default, current-engine baseline, and all Role C deliverables."
    },
    {
      "command": "read minimum local harmony, router, transport-phase, and guitar-input lifecycle code",
      "result": "passed",
      "summary": "Confirmed per-note four-voice fan-out, active NoteOn/NoteOff mapping, transport phase forwarding, guitar detection lifecycle, and teardown cleanup."
    },
    {
      "command": "focused web research and full-content review of institutional, score, primary, and scholarly sources",
      "result": "passed",
      "summary": "Retained Open Music Theory, CPDL, TMI/Zarlino, and Cambridge DOI sources; excluded unstable reposts and unsourced summaries."
    },
    {
      "command": "manual acceptance-contract checklist against completed artifact",
      "result": "passed",
      "summary": "All requested performer dimensions and structured acceptance evidence are present."
    }
  ],
  "validationOutput": [
    "Standalone Markdown artifact written to the authoritative /tmp path.",
    "No copyrighted melody was reproduced; exercise uses abstract scale degrees.",
    "Historical claims and interaction-design inferences are explicitly distinguished with claim-level confidence.",
    "Acceptance exercise includes normal NoteOff parity, empty active indicators, and no-audible-stuck-note gates."
  ],
  "residualRisks": [
    "Final safe register depends on preset voice position, octave mode, transposition, and patch.",
    "Numeric tempo, velocity, timing, and release-gap values are interaction defaults rather than historical facts.",
    "Calibrated guitar behavior requires device-level performance testing."
  ],
  "noStagedFiles": true,
  "diffSummary": "Added one non-repository Markdown research artifact; project/source/planning tree unchanged.",
  "reviewFindings": [
    "no blockers"
  ],
  "manualNotes": "The existing theory report was intentionally not opened. No automated tests were appropriate for a read-only research deliverable. noStagedFiles means this task performed no git staging or repository writes."
}
```
