# Research: Preset 08 “Suspension Garland” — Role C performer/HCI

**Retrieval date:** 2026-07-19  
**Scope:** Independent performer/interaction report. No sibling reports were read. No melody is quoted or copied.

## Summary

Play one clean note at a time in a comfortable middle register, changing pitch near metrically strong points while allowing slight finger/key overlap so the source sounds connected. At 60–72 BPM in 4/4, begin sparsely, listen for a generated pitch that remains while the vertical interval tightens on a strong beat, then allow the next source attack to trigger its downward-step resolution.

Important implementation qualification: the current Species IV state machine advances **only on source NoteOn events**. Merely holding one source note across a beat or barline does not create a new preparation–suspension–resolution event. “Long notes crossing beats and barlines” therefore means a legato sequence of discrete, mostly 1.5–2-beat notes with changes aligned to strong beats—not one uninterrupted note.

## Play it like

> **Play a slow, single-note line at 60–72 BPM. Hold each note almost two beats, change cleanly on beats 1 and 3, and leave a full bar of silence every few bars. Listen for a generated note to be prepared, rub against your next note on a strong beat, then fall by step on the following attack. No chords; use little or no pedal, and mute guitar strings you are not playing.**

## Findings

1. **The audible target is a three-stage event, not a generic held chord.** Historically, fourth species prepares a consonant note on a weak part, carries the same pitch into a strong-beat dissonance, and resolves it downward by step to consonance. Pure fourth species offsets two lines and relies on oblique motion. This is historical/theoretical evidence; translating it into live controls below is HCI inference. [Open Music Theory](https://viva.pressbooks.pub/openmusictheory/chapter/fourth-species-counterpoint/)

2. **Use a steady metric grid, but do not claim one historically exact BPM.** Renaissance tactus, mensuration, and tempo are historically complex and sometimes contradictory; DeFord treats rhythm as form-shaping and devotes separate analysis to Palestrina’s masses. The suggested 60–72 BPM is therefore a modern usability range, not a claim about Palestrina’s performance tempo. [Cambridge University Press, DeFord](https://www.cambridge.org/core/books/tactus-mensuration-and-rhythm-in-renaissance-music/F2E3F151365C3CD4D5164B43090086F4)

3. **Contrapunk’s current Species IV is NoteOn-driven.** In `crates/contrapunk-harmony/src/stateful.rs`, `process_with_beat` records a preparation after a strong-beat NoteOn, may reuse that pitch as a suspension on a later strong-beat NoteOn, and attempts a downward diatonic resolution on the next NoteOn. It recognizes beats 0 and 2 as strong in fixed 4/4, with an on-beat window where fractional phase is `<0.1` or `>0.9`. There is no clock callback that changes an already sounding harmony merely because a barline passes. **HCI consequence:** a performer must supply at least three discrete attacks; a single long note cannot demonstrate the feature. Confidence: high (direct code).

4. **Transport must be running and visible precision should be judged after the click.** The Tauri router repeatedly copies `transport.beat_position()` into the engine; stopped transport supplies `None` (`src-tauri/src/commands/engine.rs`). Although the core now has a synthetic NoteOn counter fallback (`crates/contrapunk-harmony/src/engine.rs`), the catalog explicitly requires transport and the fallback measures attacks, not elapsed musical time. Because a phase just before a barline still floors to the preceding beat, aim source NoteOn **0.00–0.08 beat after** beats 1 or 3, not early. At 60–72 BPM this is roughly 0–80/67 ms after the click. Confidence: high for mechanism, medium for the playable target window.

5. **Source gesture, note length, articulation, dynamics, density, and silence.** Use one scale tone at a time; favor adjacent steps, with an occasional third to refresh register. Suggested source register is MIDI 55–72 (G3–C5) for keyboard and roughly guitar strings 3–1 around frets 2–12, avoiding both the low guitar tracking range and the top MIDI boundary after chained voices. Hold 1.5–1.9 beats, release only 20–60 ms before the next attack or overlap by at most 20–40 ms if the input remains reliably monophonic. Use connected but newly articulated attacks, velocity about 55–85, with variation under ±12 rather than accents that mask the dissonance. Begin at one attack every two beats; never exceed one attack per beat. After 2–4 bars, rest a full bar so every release can be heard and lifecycle state can clear perceptually. These numbers are HCI starting points, not historical facts. Confidence: medium.

6. **Register and texture should preserve audibility and safety.** A middle-register source leaves room for generated voices above or below and makes the held pitch, clash, and stepwise release separable. Start with two total voices for diagnosis; add a third only after the P–S–R contour is consistently audible. Avoid Mirror/large Spread during acceptance because octave duplicates obscure whether the same generated pitch was suspended. The engine chains Species IV state per harmony pair, so more voices mean more simultaneous state machines rather than a clearer single suspension. Confidence: high for engine behavior; medium for perceptual recommendation.

7. **Phrase and section development should increase continuity before density.** Suggested 8-bar arc: bars 1–2, isolated two-beat tones to establish pulse; bars 3–4, connected attacks on beats 1 and 3; bar 5, one full bar rest; bars 6–7, resume one register step higher or lower with the same rhythm; bar 8, stop after a stable generated consonance and release all input. Do not accelerate into eighth-note figuration. This is an interaction design that exposes the state machine, not a claim that it reconstructs a Palestrina phrase.

8. **Listen for relational cues, not fixed notes.** After the first strong-beat attack, identify one generated voice as the prepared pitch. On the next strong-beat source change, listen for that generated pitch to remain while the source moves and creates audible tension. On the following clean attack, listen for the generated voice to move down one scale step and become consonant. If the generated pitch changes immediately instead of remaining, continue at the same tempo for another cycle; candidate selection may not have formed a dissonant suspension. If no resolution arrives within a few attacks, stop, release, and restart rather than adding notes—the Species IV FSM times out after more than four suspended/resolving NoteOn ticks. Confidence: high from code, medium for listening strategy.

9. **Guitar constraints are stricter than the musical prompt.** Use clean DI, one string at a time, conservative picking, and both-hand muting. Let the prior string stop before another string sympathetically rings; prefer same-string adjacent notes when practical. Avoid bends, slides through intermediate frets, wide vibrato, hammer-ons/pull-offs, distortion, and open-string resonance during acceptance. Peer-reviewed guitar note-tracking research shows that bend/release/vibrato can split or merge symbolic events and that slides/legato complicate onset/offset identification; this supports the conservative control gesture, though it did not test Contrapunk itself. [Su et al., TISMIR](https://transactions.ismir.net/articles/10.5334/tismir.23) The local guitar path also uses onset, sustain/decay hysteresis, pitch voting, and optional bend/legato/slide detection (`crates/contrapunk-audio/src/guitar_input.rs`). Confidence: high for constraint direction, medium for exact timing.

10. **Keyboard opportunity: precise attacks; keyboard limit: no chordal or pedal-created polyphony.** Play one finger/key at a time. Finger-legato is useful only if overlap stays short enough that the engine does not receive a meaningful chord. Do not hold two source keys to “make the suspension”; generated harmony already supplies the second line, while every source NoteOn independently runs the state machine. Keep damper pedal up for acceptance; CC64 sustain at the receiver may prolong released pitches without extending the engine’s input ownership, obscuring NoteOn/NoteOff parity. A brief pedal touch may be auditioned later, clearing it on every harmony change, but it is not part of the clean-monophonic contract. MIDI defines NoteOn and NoteOff as separate start/end events and CC123 as All Notes Off. [MIDI Association](https://midi.org/summary-of-midi-1-0-messages) Confidence: high.

11. **Normal NoteOn/NoteOff parity is the safety invariant.** `HarmonyEngine::harmonize_note_on` stores each generated set under the input MIDI note; `harmonize_note_off` returns that same set and removes it (`crates/contrapunk-harmony/src/engine.rs`). Each source NoteOn in the exercise must receive exactly one corresponding source NoteOff. Do not reattack the **same MIDI pitch** before releasing it: the active map is keyed only by pitch, so overlapping same-note ownership can be overwritten. The Tauri router dispatches the stored generated releases with the source release (`src-tauri/src/commands/engine.rs`). Confidence: high.

12. **“Hold” is not sustain for the baseline Species IV engine.** The UI `HoldMode` controls pending **Companion lane** emissions (`cancel`, `near_future`, `phrase_end`, `forever`); it does not change the baseline harmony engine’s NoteOff tracking (`ui/src/lib/adapter/types.ts`). For this preset’s direct StrictCounterpoint/Species4 baseline, acceptance should use `cancel` (or leave Companion disabled), and verify that releasing the source releases its generated voices normally. If a future implementation routes the preset through CounterpointLane, test `cancel` first; `near_future`, `phrase_end`, and `forever` deliberately permit tails and require separate ownership accounting. Confidence: high.

13. **Failure gestures are predictable and should be named in UI help.** Chords, long pedal, overlapping same-pitch retriggers, tremolo/repeated sixteenths, attacks before the target barline, stopped transport, abrupt octave leaps, high-gain guitar, bends/slides/vibrato, palm noise, and no phrase rests can respectively cause source polyphony, release ambiguity, overwritten ownership, note storms, wrong beat classification, Species-I-like fallback, unresolvable candidates, false NoteOns, or perceptual mud. Recovery: stop playing, lift pedal, mute strings, invoke Panic/All Notes Off, wait for silence, restart transport at bar 1, and resume with two voices. MIDI CC123 is the appropriate hard note cleanup primitive, while Contrapunk’s router broadcasts all-notes-off during its cleanup paths. [MIDI Association](https://midi.org/summary-of-midi-1-0-messages)

## Expanded performer guidance

### Setup

- 4/4, 60–72 BPM, metronome on, transport started before the first source attack.
- Strict Counterpoint + Species IV; two total voices initially; one clear, dry sound per voice.
- Source range: G3–C5 (MIDI 55–72). Guitar may center nearer G3–E5 if cleanly tracked.
- Companion off or global Hold=`cancel`; damper pedal up; guitar bends/slides/vibrato off for the test.

### Gesture recipe

1. Wait through one bar and count the clicks.
2. Attack a single scale tone just after beat 1; hold almost two beats.
3. Change to a nearby scale tone just after beat 3, keeping the line connected but emitting a new NoteOn.
4. On the next strong beat, change again by step and listen for generated downward-step release.
5. Repeat no more than twice, then release fully and leave one silent bar.

The literal preparation placement in historical fourth species is weak-to-strong. The current engine, however, records preparation only when its NoteOn is classified strong. The recipe follows current observable behavior and must not be described as a literal realization of Fux’s notation.

### Instrument-specific notes

- **Guitar:** pick softly but decisively enough to cross onset threshold; use fretted middle strings; mute every unused string; wait for an actual NoteOff before same-pitch reuse. If low notes arrive late, move up an octave rather than playing early.
- **Keyboard:** use one finger at a time, no held chord shell, no sustain pedal during validation. Short finger overlap is acceptable only when the monitor still shows one source pitch. Velocity should shape phrase direction gently, not punch every strong beat.

## 45-second observable acceptance exercise

**Prerequisites:** 4/4 at 64 BPM; transport running from bar 1; metronome on; two voices; source MIDI monitor and generated-note monitor/log visible; Hold=`cancel` or Companion disabled; pedal up. Use any five nearby in-scale degrees, e.g. abstract `3–4–2–3–1`; this is not a melody quotation.

| Time / metric phase | Performer action | Observable acceptance |
|---|---|---|
| 0–4 s, bar 1 | Silence; verify phase advances through 0,1,2,3. | Transport phase changes continuously; no notes active. |
| 4–16 s, bars 2–4 | At each displayed phase `0.00–0.08` or `2.00–2.08`, play the next single degree. Hold 1.5–1.9 beats. Emit NoteOff before/at the next different NoteOn; no same-pitch overlap. | Exactly one source pitch active. Within at most two three-attack cycles, monitor/audio shows P: generated pitch present; S: same generated pitch retained on a later strong-beat attack while interval becomes tense; R: generated pitch moves down one scale step on the following attack and returns consonant. Record actual phase for every NoteOn. |
| 16–20 s, one bar | Release and remain silent. | Source and generated active-note sets become empty; NoteOn count equals NoteOff count per source pitch/channel and for every routed generated note. No sound after release tail. |
| 20–32 s, bars 6–8 | Repeat the pattern one scale step higher or lower. Guitar: same-string where possible. Keyboard: pedal remains up. | Same P–S–R relation occurs without extra attacks, pitch flicker, or more than one source note. |
| 32–36 s, one bar | Test Hold: with Companion disabled or Hold=`cancel`, release the final source immediately after its generated response. | Baseline generated notes release with their tracked source NoteOff; no pending lane note survives. This confirms Hold does not replace normal parity. |
| 36–45 s | **Hard cleanup:** release all keys/strings, pedal up; press Panic/All Notes Off; stop transport; wait two seconds. | Input, harmony, borrowed, canon, and counterpoint active-note displays/logs are all empty; CC123/all-notes-off cleanup is emitted on routed outputs where observable; silence remains after transport stop. |

**Pass metrics**

- ≥1 audible/visible P–S–R cycle in two attempts.
- 100% source NoteOn/NoteOff parity by `(channel,pitch)`; 100% generated ownership release by routed `(target,channel,pitch)`.
- All targeted strong-beat NoteOns observed in phase `[0.00,0.08]` or `[2.00,2.08]`; no test NoteOn classified while transport is stopped.
- Maximum simultaneous source-note count = 1.
- After each silence bar and hard cleanup: zero active notes on all displayed roles and no audible stuck note.

## Historical evidence vs HCI inference

- **Historical/scholarly:** preparation is consonant; suspension is a tied strong-beat dissonance; resolution is consonant and down by step; Renaissance tactus is not reducible to a single modern BPM.
- **Direct implementation evidence:** phase classifier, strong beats 0/2, NoteOn-driven FSM, four-tick timeout, transport injection, synthetic fallback, pitch-keyed lifecycle map, HoldMode scope, and cleanup dispatch.
- **HCI inference:** 60–72 BPM, MIDI 55–72, 1.5–1.9-beat notes, velocity 55–85, post-click phase window, two-voice diagnostic setup, phrase-rest schedule, and instrument gestures. These are conservative testable defaults, not historical claims.

## Sources

### Kept

- [Open Music Theory — Fourth-Species Counterpoint](https://viva.pressbooks.pub/openmusictheory/chapter/fourth-species-counterpoint/) — open scholarly textbook; precise P–S–R, metric offset, oblique motion, and resolution rules.
- [Ruth I. DeFord, *Tactus, Mensuration and Rhythm in Renaissance Music*, Cambridge University Press](https://www.cambridge.org/core/books/tactus-mensuration-and-rhythm-in-renaissance-music/F2E3F151365C3CD4D5164B43090086F4) — scholarly framing for tactus/rhythm and a Palestrina-specific chapter; used only to reject false BPM certainty.
- [MIDI Association — Summary of MIDI 1.0 Messages](https://midi.org/summary-of-midi-1-0-messages) — primary standards organization summary for NoteOn, NoteOff, timing, and CC123.
- [Su et al., “TENT,” *Transactions of the International Society for Music Information Retrieval*](https://transactions.ismir.net/articles/10.5334/tismir.23) — peer-reviewed evidence that bends, vibrato, slides, and legato complicate guitar note-event tracking.
- Project code: `crates/contrapunk-harmony/src/stateful.rs`, `crates/contrapunk-harmony/src/engine.rs`, `src-tauri/src/commands/engine.rs`, `crates/contrapunk-audio/src/guitar_input.rs`, `ui/src/lib/adapter/types.ts`, `wasm/src/lib.rs` — authoritative evidence for actual interaction/lifecycle behavior.

### Dropped / blocked

- Fux facsimile at IMSLP — primary and relevant, but Latin/OCR extraction was unreliable for precise performer claims; Open Music Theory’s cited pedagogical account was clearer.
- Internet Archive Mann translation — access/borrow limitations; not needed to support the bounded performer report.
- CCRMA guitar onset paper PDF and DAFx pitch-tracker PDF — fetch returned only minimal extraction; recorded as blocked rather than inferred from snippets.
- Commercial/blog MIDI-guitar advice and generic piano-pedaling blogs — weaker than peer-reviewed guitar tracking and MIDI standards sources.
- Cambridge Palestrina chapter full text — paywalled; only the publisher’s book description/table of contents was used.

## Gaps, confidence, and residual risks

- **High confidence:** current event lifecycle, beat classifier, transport injection, Species IV FSM sequence, and Hold scope, from direct code.
- **Medium confidence:** proposed tempo/register/timing/articulation defaults; they require performer testing on real guitar latency, keyboard scanners, synth envelopes, and output routing.
- **Low/unknown:** historical performance tempo for a generalized “Palestrina” reference; no exact corpus was provided, and a preset should not claim imitation.
- The core’s preparation-on-strong behavior differs from textbook weak-beat preparation. A true time-sustained suspension may require a clock-driven lane or scheduled harmony lifecycle; performance instructions cannot remove this gap.
- The `>0.9` phase branch can classify a pre-boundary attack as belonging to the prior beat because beat index uses `floor`; acceptance deliberately targets just after the click.
- Guitar latency/calibration was not measured. A device-specific offset should be added only after empirical phase logging.
- Same-pitch overlapping NoteOns remain unsafe because baseline ownership is keyed by input pitch, not a counted `(channel,pitch)` identity.
- Plugin transport/Hold parity was not inspected because the requested minimum relevant path was the current Species IV baseline and Tauri/WASM lifecycle; surface-specific review remains necessary before claiming all-surface acceptance.

## Acceptance

This report is the only changed artifact. Project/source files and `.planning/` inputs were read-only; sibling reports were not read.

```acceptance-report
{
  "criteriaSatisfied": [
    {
      "id": "criterion-1",
      "status": "satisfied",
      "evidence": "Created only /tmp/contrapunk-suspension-garland-performance.md; no project or source files were modified. The report covers all requested performer/HCI dimensions and the current Species IV lifecycle qualification."
    },
    {
      "id": "criterion-2",
      "status": "satisfied",
      "evidence": "Included stable citations, direct code-path evidence, history/HCI separation, confidence and gaps, plus a 45-second observable test with phase, parity, Hold, and hard-cleanup metrics."
    }
  ],
  "changedFiles": [
    "/tmp/contrapunk-suspension-garland-performance.md"
  ],
  "testsAddedOrUpdated": [],
  "commandsRun": [
    {
      "command": "read required catalog/template and minimum relevant Species IV, transport, guitar-input, adapter, and lifecycle code",
      "result": "passed",
      "summary": "Confirmed preset scope, NoteOn-driven Species IV FSM, strong-beat phase rules, transport injection, NoteOn/NoteOff ownership, Hold scope, and cleanup behavior."
    },
    {
      "command": "focused web research and full-content fetch for fourth species, Renaissance tactus, MIDI lifecycle, and guitar note tracking",
      "result": "passed",
      "summary": "Kept scholarly/standards sources; blocked or weak sources were dropped and recorded."
    },
    {
      "command": "write /tmp/contrapunk-suspension-garland-performance.md",
      "result": "passed",
      "summary": "Research artifact written to the authoritative output path."
    }
  ],
  "validationOutput": [
    "Report contains concise Play-it-like copy, expanded guidance, all requested instrument and interaction dimensions, and no copied melody.",
    "Acceptance exercise specifies phase windows, monophonic density, normal NoteOn/NoteOff parity, Hold=cancel semantics, CC123/Panic cleanup, and zero-active-note end state.",
    "No project/source files were written and no staging command was used."
  ],
  "residualRisks": [
    "Suggested HCI timing/register defaults need real performer and device testing.",
    "Current Species IV advances only on NoteOn and prepares on a code-classified strong beat, unlike literal textbook weak-to-strong tying.",
    "Same-pitch overlapping source NoteOns can overwrite pitch-keyed lifecycle ownership.",
    "Plugin-surface transport and Hold behavior were not independently inspected."
  ],
  "noStagedFiles": true,
  "diffSummary": "Added one read-only research deliverable under /tmp; zero repository changes.",
  "reviewFindings": [
    "no blockers for the research deliverable",
    "warning: catalog wording about simply holding notes across barlines is mechanically insufficient for the current NoteOn-driven Species IV engine"
  ],
  "manualNotes": "Retrieval date 2026-07-19. No sibling reports read. No tests were added because this task requested research only and prohibited project/source modifications. noStagedFiles reflects that this run made no repository writes or staging operations."
}
```
