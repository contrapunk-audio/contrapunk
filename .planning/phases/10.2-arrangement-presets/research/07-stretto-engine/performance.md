# Role C Performer/HCI Report — Preset 07 “Stretto Engine”

**Scope:** live clean-monophonic guitarist or keyboardist; selected reference corpus: J. S. Bach’s *Well-Tempered Clavier* (WTC), especially BWV 878/2, and *The Art of Fugue* BWV 1080, especially Contrapunctus 5. BWV 538 is used only for performer-timing evidence. No theme is copied.  
**Retrieval date:** 2026-06-16.

## Executive recommendation

Use a two-to-four-note, single-line subject whose rhythm is more distinctive than its contour. Play it in the middle register at 72–100 BPM with clean, lightly detached attacks, repeatable onset spacing and releases, medium-even velocity, and a full rest after each statement. As tonic-, fifth-, and octave-related delayed answers enter closer together, play less, not more: preserve the subject and listen for its rhythmic fingerprint returning at another tonal or register level.

This is an HCI contract, not a claim that all Bach fugues grow progressively denser. A WTC corpus study defines stretto as a subject sounding against itself from different starting times, but found no strong general support for increasingly numerous or closer entries near a fugue’s end and directly contradicted its density hypothesis. [McDonald, Western University](https://ir.lib.uwo.ca/etd/6759)

## Evidence boundary: history and HCI inference

### Historical/performance evidence

- **High confidence:** Stretto overlaps subject entries beginning at different times. McDonald studied all 48 WTC fugues and cautions against treating progressive closeness or density as a universal formal law. [Western University](https://ir.lib.uwo.ca/etd/6759)
- **High confidence:** BWV 878/2 is analyzed as a stretto fugue whose overlapping subject/answer entries reveal different aspects of one theme. This supports audible recurrence, not one mandatory touch or tempo. [Royal College of Music Research Online](https://researchonline.rcm.ac.uk/id/eprint/1299/7/CI47-Article%20Charlston-superscript.pdf)
- **High confidence:** Bach Digital’s stable BWV 1080 record identifies original prints, manuscript score sources, and the NBA critical edition. It anchors the selected corpus but does not prescribe this product’s gesture. [Bach Digital](https://www.bach-digital.de/receive/BachDigitalWork_work_00011581)
- **Medium-high confidence:** In a MIDI-console study, sixteen professional organists’ largest tempo variations in BWV 538 coincided with perceived structural boundaries. This supports reserving timing change for phrase/section boundaries rather than distorting every repeated subject. [Gingras, McAdams, and Schubert](https://doi.org/10.31751/p.78)

### HCI/implementation inference

- `CanonLane` schedules delayed NoteOns from transport `total_beats()` and delayed NoteOffs from the captured input duration. Equal source onset intervals and gates are therefore the simplest way to produce identifiable matching entries.
- The lane resets its phrase anchor only after **more than two beats** between input events (`> 2.0`, not `>= 2.0`). Silence is both a musical and computational boundary.
- Each canon voice can have its own delay, diatonic transpose, time ratio, HoldMode, and harmony stack. One false source event can multiply across voices.
- `Transport` is sample-position-based, advances only while running, and applies BPM changes immediately. Start transport first and hold tempo fixed during evaluation.
- MIDI defines NoteOn and NoteOff as note start/end events and CC 123 as All Notes Off. Lifecycle acceptance must inspect event parity and explicit cleanup, not merely perceived quiet. [MIDI Association](https://midi.org/summary-of-midi-1-0-messages)
- Guitar bends, releases, vibrato, hammer-ons, pull-offs, and slides alter pitch contours and can confuse onset, offset, and F0 segmentation. Plain plucks are the safer input for a clean-monophonic tracker. [Su et al., TISMIR](https://transactions.ismir.net/articles/10.5334/tismir.23)

Local code read: `crates/contrapunk-companion/src/canon_lane.rs`, `crates/contrapunk-transport/src/clock.rs`, and minimum relevant `crates/contrapunk-harmony/src/engine.rs`.

## Play it like — concise copy

**Play two to four clear single notes in the middle register. Give them one memorable rhythm, repeat it exactly, then leave a full breath. Keep attacks even; as the answers crowd in, stop adding notes and listen for the same rhythm returning at tonic, fifth, and octave levels.**

## Expanded performer guidance

### 1. Best source gesture

Use one **sequential, monophonic 2–4-note cell**, not a chord. Give it a compact contour and a clear rhythmic asymmetry. Abstract example only: scale degrees `1–2–5`, with onsets at beats `0, 0.5, 1.5`. A rhythm containing one short and one longer gap remains recognizable when pitch levels overlap.

Repeat the whole cell without adding an improvised tail. The preset supplies delayed overlap; it should not be asked to extract identity from arbitrary continuous playing.

### 2. Register

- **Guitar:** roughly A3–E5.
- **Keyboard:** roughly C4–C5.

These are HCI starting ranges, not historical prescriptions. They leave room for fifth and octave displacement and avoid low-register masking. If an answer nears the MIDI range limit or voices bunch together, move the source an octave toward the center.

### 3. Tempo and rhythmic precision

Start at **72–100 BPM in 4/4**; 90 BPM is a useful acceptance tempo. Enter on a downbeat after transport starts.

Keep repeated onsets within about a thirty-second-note of the intended grid. This is a practical target, not a historical rule or hard quantizer threshold. More important than microscopic accuracy is preserving the ratio between the cell’s short and long gaps.

Do not change BPM while judging one overlap arc. Digital-instrument research supports low and consistent action-to-sound latency, but does not justify one universal threshold for all instruments and tasks. [Audio Mostly 2016](https://doi.org/10.1145/2986416.2986428)

### 4. Note lengths and articulation

Gate each note to about **50–75% of its inter-onset interval** on the first pass. Use lightly detached, neutral attacks: long enough to establish pitch, short enough to leave a seam between attacks. Avoid clipped key-click stubs and pedal-length legato.

Release timing matters because the lane delays captured NoteOffs and preserves source duration. Keep the same gate on each repetition. Give the first note only a slight accent; do not strike over generated answers to “help” them.

### 5. Velocity and dynamics

Use stable medium velocity, approximately **MIDI 72–92**, with the first note no more than about eight values stronger. CanonLane forwards source velocity, so large accents recur in every delayed voice.

For section development, use a small dynamic arc—medium, slightly stronger, then withdraw—without changing the subject rhythm. Treat these values as testable defaults, not Bach performance doctrine.

### 6. Density and silence

- One physical input note at a time.
- No more than four source notes per cell.
- As configured entry delays get shorter, reduce source activity rather than play faster.
- After a teaching statement, rest for at least the longest configured delay plus the cell’s sounding duration.
- To force a fresh current-code phrase anchor, leave **more than two transport beats between input events**.

Silence lets the player distinguish source, first answer, and later overlap. It also prevents a continuously growing pending queue.

### 7. Phrase and section development

Use a three-stage interaction arc:

1. **Declare:** play the cell once or twice with long rests so its rhythm and tonal level are learned.
2. **Overlap:** repeat the same cell while the preset’s answer delays become closer. Hold contour, tempo, articulation, and velocity steady so delay is the changing variable.
3. **Release:** omit the next repetition, release the source normally, and listen through the pending tail. Leave more than two beats before a new section.

In a second section, change only one performer variable—such as moving the source up an octave or modestly increasing velocity. Do not change register, rhythm, contour, and articulation together. Progressive closeness is this preset’s interaction design, not a universal WTC formal claim. [McDonald](https://ir.lib.uwo.ca/etd/6759)

### 8. Listening and response cues

- Track the **rhythmic fingerprint** before trying to follow every vertical sonority.
- On the first pass, wait until the first delayed answer has stated at least the cell’s first two onsets before continuing.
- Hear tonic/unison relation as the same level, fifth relation as the same gesture displaced, and octave relation as the same pitch classes in a new register.
- At peak overlap, listen for separate attacks. If attacks fuse into a block, shorten source notes or reduce density.
- If the subject vanishes, add silence rather than velocity.
- Do not chase a mistracked generated pitch with corrective notes; stop, clean up, and restart.
- At the ending, release the source and listen through the answer tail instead of supplying a cadence chord.

### 9. Guitar constraints under clean monophonic input

- Use a clean, low-gain sound and mute unused strings with both hands.
- Use consistent single-note picking with a definite attack.
- Release or mute the previous string before the next one rings.
- Avoid double-stops, open-string drones, sympathetic ringing, feedback, scrape noise, ghost attacks, and heavy compression that hides onset contrast.
- During acceptance, avoid bends, wide vibrato, target-crossing slides, hammer-ons, and pull-offs. Peer-reviewed guitar-transcription work shows these techniques can split or merge note events and confuse pitch tracking. [TISMIR](https://transactions.ismir.net/articles/10.5334/tismir.23)
- If audio-to-MIDI emits a duplicate onset or octave error, stop, mute all strings, invoke hard cleanup, and retry with plain middle-register plucks. Playing through a mistrack seeds more delayed errors.

### 10. Keyboard opportunities and chord limits

- Use one finger/key at a time with sustain pedal off.
- A keyboard offers repeatable attacks, velocities, and releases without audio pitch estimation.
- Finger substitution is acceptable only if notes do not overlap.
- The chord-density limit is **one simultaneous key**. “Two-to-four-note subject” means sequential notes, not a two-to-four-note chord.
- A chord multiplies every input pitch across delayed voices and harmony stacks, producing mud or a note storm.
- Keep aftertouch and expression changes modest; they do not improve subject identity.

### 11. Failure gestures and recovery

| Gesture | Failure | Recovery |
|---|---|---|
| Chord, double-stop, or overlapping keys | Multiple delayed stacks; mud/note storm | Release all; hard cleanup; resume monophonically |
| Continuous sixteenth stream | No phrase boundary; busy pending queue | Stop input, hear the tail, restart with 2–4 notes |
| New rhythm every repetition | Answers no longer identify one subject | Restore one onset grid |
| Sustain pedal or ringing strings | Late NoteOffs and masking | Pedal off/mute; use 50–75% gate |
| Bend, slide, wide vibrato, noisy transient | False onset/pitch/offset multiplied by delays | Plain attack; cleanup before retry |
| Playing while transport is stopped | Pending delays cannot mature | Reset/start transport, count in, then play |
| BPM change mid-tail | Delay spacing cannot be evaluated consistently | Hold tempo until all tails finish |
| Disable/reconfigure while notes sound | Queues may clear while dispatched notes remain sounding | Use explicit panic/All Notes Off |

## 30–60 second acceptance exercise

**Duration:** about 48 seconds at 90 BPM.  
**Setup:** 4/4, transport reset and running; clean monophonic input; velocity near 80; no pedal. Configure three delayed answer voices at tonic/unison-, fifth-, and octave-related levels, strict time ratio `1.0`, and progressively closer delays. Exact voice values remain a synthesis decision. Choose and record a tail policy (`NearFuture`, `PhraseEnd`, `Cancel`, or `Forever`). Make panic/stop hard cleanup available.

1. **0–4 s — Declare.** On beat 1, play abstract cell `1–2–5` at onsets `0, +0.5, +1.5 beats`; gate about 60%; send a normal NoteOff for every source NoteOn.
2. **4–12 s — Listen.** Play nothing. Identify the same rhythm returning at the configured tonal/register levels. Confirm no generated NoteOff occurs without a prior matching generated NoteOn.
3. **12–24 s — Repeat.** Play the identical cell twice on downbeats at velocity 80±8. Confirm delayed answers preserve source onset spacing and gate while closer entries remain recognizable.
4. **24–32 s — Peak and release.** Play the cell once more, then stop. Confirm every physical input NoteOn has one physical NoteOff and every normally emitted generated NoteOn eventually gets an effective NoteOff. There must be no stuck note or orphan release.
5. **32–40 s — Pending-tail behavior.** Play one short final cell and release its last source note before the longest delayed NoteOn matures. Verify the selected policy:
   - `Cancel`: unfired answers are suppressed and their queued offs are removed.
   - `NearFuture`: only answers inside its tail horizon survive.
   - `PhraseEnd`: only answers through the phrase boundary survive.
   - `Forever`: scheduled answers continue to their natural delayed offs.
   Any retained NoteOn must receive a matching NoteOff; any cancelled NoteOn must not later produce an orphan NoteOff.
6. **40–48 s — Hard cleanup.** While a generated note/tail is active, invoke the product panic/stop cleanup. Expect immediate silence, downstream CC 123 All Notes Off or surface-equivalent all-voice release, cleared runtime pending state, and no tail reappearing after the former longest-delay horizon. `CanonLane::reset_runtime` or disabling alone is insufficient evidence for already-dispatched notes because those paths clear queues without themselves emitting releases.

**Required capture:** timestamped input/generated event log; counts grouped by target/channel/note; normal-flow NoteOn/NoteOff parity; pending entries retained/cancelled under the chosen HoldMode; hard-cleanup dispatch; silence after at least the old longest-delay horizon. MIDI event and CC 123 meanings: [MIDI Association](https://midi.org/summary-of-midi-1-0-messages).

## Testable performer invariants

1. Input remains monophonic: at most one physical source note sounding at once.
2. The 2–4-note cell’s onset pattern is unchanged within an overlap section.
3. Transport runs before the first input and BPM stays fixed during the test.
4. Source register leaves headroom for fifth/octave answers.
5. Physical releases are deliberate and repeatable; sustain is off.
6. Density decreases as overlap increases.
7. A phrase-ending rest exceeds two beats when a fresh lane anchor is required.
8. Normal NoteOn/NoteOff parity, pending-tail policy, and hard cleanup all pass independently.

## Sources

### Kept

- Kathryn McDonald, “Exploring Stretto: An Investigation into the Use of Stretto in J. S. Bach’s Well-Tempered Clavier,” Western University, https://ir.lib.uwo.ca/etd/6759 — corpus definition and important negative result on assumed end-directed closeness/density.
- Terence Charlston, “Patterns of Play … Fugue in E Major BWV 878/2, Part II,” Royal College of Music, https://researchonline.rcm.ac.uk/id/eprint/1299/7/CI47-Article%20Charlston-superscript.pdf — selected-fugue stretto analysis.
- Bach Digital, BWV 1080 stable work record, https://www.bach-digital.de/receive/BachDigitalWork_work_00011581 — institutional primary-source catalog and critical-edition record.
- Bruno Gingras, Stephen McAdams, and Peter Schubert, “The Performer as Analyst: … BWV 538,” https://doi.org/10.31751/p.78 — professional-organist MIDI timing evidence.
- Ting-Wei Su et al., “TENT: Technique-Embedded Note Tracking for Real-World Guitar Solo Recordings,” https://transactions.ismir.net/articles/10.5334/tismir.23 — peer-reviewed guitar onset/offset/F0 evidence.
- MIDI Association, “Summary of MIDI 1.0 Messages,” https://midi.org/summary-of-midi-1-0-messages — NoteOn, NoteOff, timing clock, and All Notes Off definitions.
- Jack et al., “Effect of latency on performer interaction and subjective quality assessment of a digital musical instrument,” https://doi.org/10.1145/2986416.2986428 — latency consistency relevance, used without asserting a universal threshold.
- Local implementation files listed above — authoritative for present Contrapunk scheduling and lifecycle behavior.

### Dropped or blocked

- Revue musicale OICRM, “Parameters of Stretto … Contrapunctus 5,” DOI https://doi.org/10.7202/1062432ar — full text blocked by an Anubis proof-of-work page; no detailed claim taken from it.
- Oxford *Music Theory Spectrum*, “Subjects and Phrase Boundaries in Two Keyboard Fugues” — insufficient abstract/paywall access for performer prescriptions.
- Scribd summaries, YouTube recordings, fan and SEO pages — less authoritative or stable than kept sources.
- Generic latency thresholds — evidence is task- and instrument-dependent.

## Confidence and gaps

- **High confidence:** short repeatable rhythmic cell; transport dependence; monophonic density limit; silence requirement in current code; NoteOn/NoteOff and hard-cleanup acceptance; guitar technique risks; warning against universalizing progressive stretto.
- **Medium confidence:** 72–100 BPM, MIDI 72–92, 50–75% gates, and suggested registers. These are HCI defaults inferred from recognizability and implementation, not historical prescriptions.
- **Gaps:** exact preset delays and per-voice transpose/octave settings are parent-synthesis decisions; no live study yet compares guitar and keyboard recognition at each overlap; surface-specific panic routing was outside the minimum code read; blocked Contrapunctus 5 text prevented measure-specific evidence. Recommended next step is the logged acceptance exercise on each shipping surface with 3–5 performers.

```acceptance-report
{
  "criteriaSatisfied": [
    {
      "id": "criterion-1",
      "status": "satisfied",
      "evidence": "Wrote only the requested Role C report to /tmp/contrapunk-stretto-engine-performance.md; project/source files and sibling reports were not modified or consulted."
    },
    {
      "id": "criterion-2",
      "status": "satisfied",
      "evidence": "The artifact contains stable citations, explicit history/HCI separation, all requested performer dimensions, implementation-grounded lifecycle guidance, confidence/gaps, and a 48-second acceptance exercise covering normal parity, pending tails, and hard cleanup."
    }
  ],
  "changedFiles": [
    "/tmp/contrapunk-stretto-engine-performance.md"
  ],
  "testsAddedOrUpdated": [],
  "commandsRun": [
    {
      "command": "Read requested catalog/template and minimum relevant HarmonyEngine, CanonLane, and Transport code with repository read tools",
      "result": "passed",
      "summary": "Verified Role C scope, beat scheduling, phrase reset threshold, duration-preserving NoteOffs, HoldMode behavior, and transport semantics."
    },
    {
      "command": "Use previously collected institutional, scholarly, standards, and performance evidence",
      "result": "passed",
      "summary": "No further search was performed; blocked and insufficient sources are recorded as gaps."
    },
    {
      "command": "Write and read /tmp/contrapunk-stretto-engine-performance.md",
      "result": "passed",
      "summary": "Complete report written to the authoritative path and prepared for independent review."
    }
  ],
  "validationOutput": [
    "Covered gesture, register, tempo, note lengths, articulation, velocity, density, silence, transport precision, development, listening, guitar constraints, keyboard chord limits, and failure recovery.",
    "Acceptance exercise explicitly tests normal NoteOn/NoteOff parity, each pending-tail policy class, and panic/All Notes Off hard cleanup.",
    "Examples use abstract scale degrees and beats; no theme was copied.",
    "Historical claims and implementation/HCI inferences are labeled separately."
  ],
  "residualRisks": [
    "Exact delay and pitch configuration remains a parent-synthesis decision.",
    "No live performer or surface-specific event-log test was possible in this read-only research task.",
    "Git staging state could not be independently inspected because no shell/VCS tool was available; this run performed no staging and changed only /tmp."
  ],
  "noStagedFiles": true,
  "diffSummary": "Added one complete /tmp research report; project tree unchanged.",
  "reviewFindings": [
    "no blockers"
  ],
  "manualNotes": "Reviewer should preserve the distinction between the preset's progressive-closeness interaction and historical claims about Bach; detailed Contrapunctus 5 full text remained blocked."
}
```
