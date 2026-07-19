# Phase 10.2 — Preset 04 “Mensuration Web”
## Independent Role C: performer / interaction report

**Reference:** Johannes Ockeghem, specifically the proportional-mensuration principle associated with *Missa prolationum*; not a claim to reproduce Ockeghem’s music or his whole style.  
**Retrieval date for all web sources:** 2026-06-18  
**Evidence labels:** **HIST** = historical, score/manuscript, or performance-practice evidence; **HCI** = performer guidance inferred from that evidence plus Contrapunk’s current interaction model.

## Summary

The intelligible live input is one clean, moderately paced, three-to-six-note monophonic cell with distinct attacks, controlled releases, and at least two beats of silence afterward. Start transport before the first attack: current proportional playback is calculated from transport beats relative to a phrase anchor, and a stopped clock cannot mature delayed events. The honest reference is the audible idea “related lines begin from common material and unfold at different proportional speeds,” not a reconstruction of *Missa prolationum*.

## Play it like…

> **Play one exact 3–6-note line, clean and even, then leave a wide silence and listen as the same shape opens at different speeds.**

## Expanded performer guidance

### 1. Best source gesture

- Use **single notes**, not a chord: a compact **3–6-note motif** lasting roughly **3–5 beats**.
- Give it one memorable contour: mostly steps, with at most one clear third or fourth; avoid a scale run that has no identifiable shape.
- Prefer four notes for a first pass. Play the motif once, then stop. Repetition is useful only after the generated proportional versions have become audible.
- This is consistent with the catalog’s “exact three-to-six-note motif with space afterward.” The historical analogy is limited but real: a proportion canon presents one melody simultaneously under different rhythmic interpretations/speeds. All voices beginning together is documented for the repertory studied by Schubert; the Ockeghem mass itself is a four-voice double mensuration canon. **[HIST, high]** [Schubert dissertation](https://doi.org/10.82308/31186) [DIAMM work record](https://www.diamm.ac.uk/compositions/24347/) [Musica Nova Lyon performer note](https://www.musicanova-lyon.fr/en/discography-teaching/ockeghem-missa-prolationum/)
- **Do not copy an Ockeghem melody.** Create an abstract cell in the selected scale, for example scale degrees `1–2–4–3`; rhythm `quarter–quarter–half–quarter`. This is an exercise, not a composition attributed to Ockeghem.

### 2. Register, tempo, duration, articulation, velocity, density, silence

| Dimension | Starting recommendation | Why / confidence |
|---|---|---|
| Register | **C4–A4** on keyboard; approximately **G3–E5** on guitar, favoring the upper four strings | Keeps the source central while leaving generated voices room. Higher guitar pitches generally track faster than low ones. **[HCI, medium-high]** |
| Tempo | **66–84 BPM**, start at **72 BPM**, 4/4 | Slow enough that diminution remains articulated and augmentation does not become an indistinct sustain. This is a product range, **not a historical tempo claim**. **[HCI, medium]** |
| Source note lengths | **0.55–0.8 beat** for moving notes; up to **1.25 beats** for the final note | Gives every NoteOn a deliberate NoteOff while preserving gaps after time scaling. **[HCI, high]** |
| Articulation | Clean non-legato/soft tenuto; tiny audible gaps; one attack per intended pitch | Distinct event boundaries preserve motif identity. Avoid tremolo, trills, grace-note sprays, slides, and smeared pedal. **[HCI, high]** |
| Velocity | Keyboard **65–90**, narrow spread (about ±8); guitar: even medium plucks | Current CanonLane copies the input velocity to every generated pitch in that voice’s stack, so hard accents multiply through the texture. Dynamics here organize foreground/background; they are not historical vocal-dynamic evidence. **[HCI/code, high]** |
| Input density | At most **one source NoteOn per beat** on the first pass; never more than six NoteOns before the listening rest | Every input schedules emissions for every configured proportional voice, potentially with a harmony stack. Sparse input is the primary density control. **[HCI/code, high]** |
| Silence | **At least 2 beats**, preferably **one full 4/4 bar**, after the cell | Lets late/augmented events speak. More than two transport beats before the next input also resets CanonLane’s phrase anchor. **[HCI/code, high]** |

Historical tempo and articulation are intentionally not inferred from modern BPM or MIDI velocity. DeFord documents contradictions and ambiguities in Renaissance tactus theory and treats mensuration/rhythm as form-bearing practice; that supports rhythmic seriousness, not an exact “authentic” BPM for this preset. **[HIST, high for ambiguity; low for any exact modern tempo]** [Cambridge University Press](https://www.cambridge.org/core/books/tactus-mensuration-and-rhythm-in-renaissance-music/F2E3F151365C3CD4D5164B43090086F4)

### 3. Exact transport dependency and rhythmic precision

**Transport is mandatory for this preset’s current proportional behavior.**

- `CanonLane` timestamps input with `transport.total_beats()`. For each voice it schedules:
  - `generated_on = phrase_anchor + delay_beats + (input_on − phrase_anchor) × time_ratio`
  - `generated_duration = input_duration × time_ratio` (subject to the selected HoldMode).
- `tick()` emits pending NoteOns and NoteOffs only when the running transport reaches their scheduled beat.
- The transport is sample-count driven; `total_beats = samples / sample_rate × BPM / 60`. `stop()` freezes its position and `play()` resumes from there.
- Therefore: **press Play before the first motif attack**. If stopped throughout, all source notes receive essentially the same beat coordinate, proportional onset spacing collapses, and any positive-delay event cannot mature. Stopping mid-tail freezes pending events rather than completing them.
- Input timing is not internally motif-quantized. Aim for attacks within approximately **±40 ms** of the intended pulse at 72 BPM and release each key/string deliberately. This tolerance is an HCI test target, not historical evidence. Timing error is multiplied by each voice’s ratio: a late source attack is later still in augmentation.
- Preserve the same rhythm when restating the motif. “Nearly the same” source timing creates audibly different webs.
- Proportional integrity also depends on HoldMode. `Forever` preserves the computed delayed NoteOn/NoteOff schedule; `Cancel`, `NearFuture`, or `PhraseEnd` may cancel future attacks or shorten releases. Acceptance must record the preset’s effective HoldMode rather than silently assuming a complete tail.

**Local code basis:** `crates/contrapunk-companion/src/canon_lane.rs` (`CanonVoice.time_ratio`, `PHRASE_SILENCE_THRESHOLD`, `on_input`, `tick`); `crates/contrapunk-companion/src/lane.rs` (`HoldMode`); `crates/contrapunk-transport/src/clock.rs` (`Transport::total_beats`, `advance`, `play`, `stop`). **[code, high]**

### 4. Phrase and section development

Use a restrained three-stage arc:

1. **State (0–12 s):** play one four-note cell at medium velocity; leave a full bar.
2. **Recognize (12–28 s):** while silent, identify the fastest and slowest generated versions. Do not fill their gaps.
3. **Vary (28–45 s):** after the tail clears, restate once with exactly the same rhythm but change **one** property only: transpose the whole cell, change its last scale degree, or soften velocity. Then withdraw.

Across a longer section, alternate `cell → generated response → longer silence → one controlled variation`. A gap greater than two beats causes the current lane to treat the next input as a fresh phrase anchor; use that boundary deliberately. Do not continuously concatenate motifs: the implementation schedules note events, not semantic motif recognition, so “development” must be supplied by the performer. **[HCI/code, high]**

### 5. Listening and responding

- After release of the last source note, wait until you can point to **two different temporal identities**: one voice compressing the source spacing and one stretching it (where configured ratios provide both).
- Re-enter only after the slowest expected generated NoteOff, or after an intentional panic/cleanup. If uncertain, wait one more bar.
- Listen for contour before harmony: can you still hear the source order despite different clocks? If not, reduce the source to three or four notes, lower velocity, or lengthen the silence.
- Treat an unexpectedly dense simultaneous stack as a warning, not an invitation to play louder. Current `CanonLane::compute_voice_stack` can harmonize each proportional subject through a per-voice `HarmonyEngine`; one input NoteOn may therefore emit several pitches per configured canon voice. **[HCI/code, high]**

### 6. Guitar-specific contract

- Use a **clean, dry, monophonic** signal; one string sounding at a time. Fret and pick plainly, then actively mute the string to define NoteOff.
- Favor upper strings/mid neck. Low notes require longer observation to estimate pitch and can feel late; Contrapunk’s local detector also needs attack evidence and has onset cooldown/hysteresis.
- Mute unused strings with both hands. Avoid open-string sympathetic ringing, fret buzz, pick scrape, palm-muted noise bursts, harmonics, tapping, rapid hammer-on/pull-off figures, bends, vibrato, and slides for the acceptance pass.
- Do not let two strings overlap during a position change. “Arpeggiated chord” is still polyphonic if the prior string rings.
- Pitch-tracking literature explains the mechanism: pick transients are initially noise; harmonics, buzz, and decays can cause wrong/octave notes; guitar NoteOff is less determinate than a keyboard switch; plain picking and active damping improve control. **[HCI supported by engineering practice, high]** [Sound On Sound, *MIDI Guitar Workshop*](https://www.soundonsound.com/techniques/midi-guitar-workshop) [Sound On Sound, *20 Tips on Using MIDI Guitar*](https://www.soundonsound.com/techniques/20-tips-using-midi-guitar)
- Contrapunk-specific caveat: the local audio path may emit pitch bend, pressure, and CC74 in addition to NoteOn/NoteOff and uses per-string channels by default. The canon currently ignores CC input, but wrong attacks still schedule full proportional output. **[code, high]** `crates/contrapunk-audio/src/guitar_input.rs`.

### 7. Keyboard opportunities and chord limits

- A keyboard gives deterministic switches for clean NoteOn/NoteOff, repeatable velocity, exact rhythm, and reliable register changes. Use those advantages to make two rhythmically identical statements or a precise whole-cell transposition.
- Play **one key at a time**. Release before the next attack; avoid sustain pedal in the acceptance pass.
- Chord limit for this preset: **one simultaneously held source pitch**. A dyad is not an expressive exception: each key schedules every proportional voice and possible harmony stack, multiplying density. Broken chords are acceptable only if fully monophonic and treated as the motif itself.
- Avoid very short key taps: after a time ratio below 1.0 they may become clicks or inaudibly short events. Avoid long pedal-held notes: augmentation and generated harmony stacks overlap into a pad rather than an intelligible proportional line.

### 8. Failure gestures and misleading claims

| Failure gesture | Observable failure |
|---|---|
| Chords, sustain pedal, overlapping guitar strings | Several source events × proportional voices × per-voice harmony stacks: mud or a note storm |
| Continuous eighth/sixteenth-note stream with no rest | Pending queues remain populated; no listening window; motif identity disappears |
| Repeated same-pitch tremolo/trill | Many near-identical NoteOns obscure proportional spacing and can collide at one MIDI pitch |
| Low, noisy guitar attacks; scrapes, harmonics, bends, slides | Late, false, octave, or bend-heavy detection; each false NoteOn seeds generated events |
| Missing/late NoteOff, ringing open string, held pedal | Augmented voices overlap far beyond the intended phrase; possible stuck-sounding texture |
| Changing tempo during the web | Current transport computes beat count directly from current BPM and accumulated sample position; the proportional tail can be re-referenced unexpectedly. Keep BPM fixed for acceptance. |
| Playing while transport is stopped | Positive-delay events do not mature; relative onset offsets collapse |
| Calling the output “Ockeghem counterpoint” or an “authentic Missa prolationum” | False stylistic claim: the product supplies generic event scaling and optional harmony stacks, not Ockeghem’s composed four-part double canons, interval plan, mensural notation, text, cadences, or contrapuntal solutions |

The primary-source record matters here: DIAMM identifies the mass, its movements, Ockeghem attribution, and manuscript source; the Chigi source is a notated historical object, while another surviving source resolves the canons into uniform mensurations. The preset should cite the work as a reference to a technique, not reduce the composer to “one motif at arbitrary speeds.” **[HIST, high]** [DIAMM composition](https://www.diamm.ac.uk/compositions/24347/) [DIAMM Chigi Codex](https://www.diamm.ac.uk/sources/943/) [Berger, “The Other *Missa Prolationum*”](https://doi.org/10.1525/jm.2020.37.3.267)

## 45-second observable acceptance exercise

**Setup:** 72 BPM, 4/4; transport reset to beat 0 and running; preset 04 active; clean monophonic input; fixed tempo; event monitor records timestamp/beat, source/generated identity, MIDI channel, pitch, velocity, NoteOn, and NoteOff. Record configured voice delays, time ratios, stack sizes, and effective HoldMode before playing.

1. **0–4 s:** start/reset transport and play nothing for one beat. Confirm transport beat position advances.
2. **4–9 s:** play a new four-note scale-degree cell such as `1–2–4–3` in C4–A4. Attack on consecutive quarter-note pulses; hold first three notes about 0.65 beat and the last about 1 beat; velocity 76–84. Ensure exactly four source NoteOns and four matching source NoteOffs, with no overlap.
3. **9–25 s:** hands off / strings muted. Listen and watch the generated events. For each configured voice with ratio `r` and delay `d`, verify generated onset offsets from the phrase anchor equal `d + r × source_offset` within router/audio tolerance, and generated durations equal `r × source_duration` unless the recorded HoldMode legitimately truncates/cancels them. At least two unequal configured ratios must produce audibly unequal spacing for the preset name to be justified.
4. **25–31 s:** after at least two beats of silence and once the first tail is clear, replay the exact rhythm one scale step higher (or change only the final degree), 8–12 velocity units softer.
5. **31–41 s:** hands off again. Verify the second proportional web has the same timing relationships, the intended pitch variation, and lower copied velocities; no extra source NoteOn is allowed.
6. **41–45 s:** wait for all scheduled NoteOffs. Stop/disable the lane or invoke the product’s normal panic/cleanup. Confirm the event trace ends with a matching NoteOff for every emitted `(target/channel, pitch)` count; active input, harmony, canon/counterpoint note counters and pending lane queues are zero. If the normal stop path emits MIDI CC123 All Notes Off, retain that event as additional cleanup evidence; CC123 does not excuse a reproducible missing paired NoteOff during normal completion.

**Pass:** motif contour remains identifiable; proportional onset/duration relations are observable; four source NoteOns/four source NoteOffs only; silence exposes the web; all generated lifecycle counts balance and final state is empty.  
**Fail:** any chord/overlap at source, spurious guitar NoteOn, ratio collapse, positive-delay event stranded by stopped transport, unexplained duration truncation, unmatched NoteOn, nonzero final active/pending state, or a claim of exact Ockeghem imitation.

The event semantics are externally reviewable: the MIDI Association defines NoteOn as note start, NoteOff as note release/end, and CC123 as All Notes Off. **[technical standard, high]** [MIDI Association message summary](https://midi.org/summary-of-midi-1-0-messages)

## Findings and confidence

1. **A short exact monophonic cell plus silence is the best interaction.** It preserves contour under simultaneous speed ratios and bounds multiplicative event density. **[HCI, high]**
2. **Transport must run before input and through the tail.** This follows directly from the lane and transport implementation, not from historical practice. **[code/HCI, high]**
3. **Common material at different speeds is historically grounded; arbitrary live event scaling is only an analogy.** Simultaneous mensuration/proportion canons are documented; Ockeghem’s mass is a composed four-voice double canon, not an improvisation effect. **[HIST, high]**
4. **No exact historical BPM, velocity, or MIDI articulation is supportable.** Recommendations above are interaction settings chosen for intelligibility. **[HIST gap/HCI, high]**
5. **Guitar needs stricter sparsity than keyboard.** Pitch/onset/release inference, low-note latency, transients, and sympathetic noise can create false events that the generator multiplies. **[HCI/engineering, high]**
6. **Current harmony-stack behavior is a density risk.** Each proportional voice can emit more than its subject, so a nominal four-note motif may create substantially more events than a line-only interpretation suggests. **[code, high]**

## Sources

### Kept

- [DIAMM — *Missa Prolationum*](https://www.diamm.ac.uk/compositions/24347/) — Oxford institutional catalog linking Ockeghem, the mass cycle/movements, and manuscript source.
- [DIAMM — Chigi Codex V-CVbav MS Chigi C.VIII.234](https://www.diamm.ac.uk/sources/943/) — institutional primary-manuscript record.
- [Anna Maria Busse Berger, “The Other *Missa Prolationum*,” *Journal of Musicology* 37/3 (2020)](https://doi.org/10.1525/jm.2020.37.3.267) — peer-reviewed study of the Chigi mensuration-canonic notation and Vienna resolved source.
- [Peter Schubert, *Compositional strategies in mensuration and proportion canons, ca. 1400–ca. 1600*](https://doi.org/10.82308/31186) — scholarly dissertation defining simultaneous-entry proportion/mensuration canons as one melody at different speeds.
- [Ruth I. DeFord, *Tactus, Mensuration and Rhythm in Renaissance Music*](https://www.cambridge.org/core/books/tactus-mensuration-and-rhythm-in-renaissance-music/F2E3F151365C3CD4D5164B43090086F4) — university-press performance/theory context and caution about tactus ambiguity.
- [Musica Nova Lyon — Ockeghem, *Missa prolationum*](https://www.musicanova-lyon.fr/en/discography-teaching/ockeghem-missa-prolationum/) — specialist ensemble’s concise performer-facing explanation of simultaneous starts, differing note values, and double-canon texture.
- [MIDI Association — Summary of MIDI 1.0 Messages](https://midi.org/summary-of-midi-1-0-messages) — authoritative NoteOn, NoteOff, and CC123 lifecycle definitions.
- [Sound On Sound — “MIDI Guitar Workshop”](https://www.soundonsound.com/techniques/midi-guitar-workshop) and [“20 Tips on Using MIDI Guitar”](https://www.soundonsound.com/techniques/20-tips-using-midi-guitar) — long-running professional technical publication; direct practical evidence on pitch-tracking latency, transient/buzz errors, clean attacks, and damping.

### Dropped

- Wikipedia — useful orientation but redundant after institutional and scholarly sources.
- IMSLP/CPDL modern editions — useful for score access, but not needed for the performer claims and potentially edition-dependent.
- Scribd, Academia.edu mirrors, DocsLib, and anonymous analysis mirrors — unstable, derivative, or unclear provenance.
- Commercial/SEO guitar-to-MIDI guides — redundant after detailed engineering-practice sources and local implementation evidence.

## Gaps and residual risks

- The final preset’s exact voice ratios, delays, transpositions, per-voice stack sizes, and HoldMode were not present in the catalog; acceptance cannot name exact expected timestamps until synthesis/configuration fixes them.
- No empirical latency/jitter benchmark was run for Contrapunk’s physical MIDI, Tauri guitar, WASM guitar, and plugin surfaces. The ±40 ms performance target is an HCI proposal, not measured system tolerance.
- Historical sources support the proportional-canon analogy but do not justify the proposed modern register, BPM, velocity, articulation, or phrase duration.
- The current CanonLane’s phrase anchor resets after more than two transport beats between **input events**, while already scheduled long-ratio tails may still sound. Product testing must verify that a fresh phrase cannot collide with an old tail under the chosen ratios/HoldMode.
- A true Ockeghem-informed realization would require designed contrapuntal compatibility, mensural relationships, interval/cadential planning, and likely score-level validation; arbitrary input cannot guarantee those. UI copy should remain “reference”/“inspired by proportional canon.”

## Acceptance report

```acceptance-report
{
  "criteriaSatisfied": [
    {
      "id": "criterion-1",
      "status": "satisfied",
      "evidence": "Produced only the requested standalone Role C performer/interaction report at /tmp/contrapunk-mensuration-web-performance.md; project and source files were not modified."
    },
    {
      "id": "criterion-2",
      "status": "satisfied",
      "evidence": "Report includes claim-labeled historical versus HCI evidence, stable URLs and retrieval date, code-grounded transport/input lifecycle behavior, performer contracts for guitar and keyboard, failure cases, confidence/gaps, and a 45-second event-level acceptance exercise with NoteOn/NoteOff and final cleanup checks."
    }
  ],
  "changedFiles": [
    "/tmp/contrapunk-mensuration-web-performance.md"
  ],
  "testsAddedOrUpdated": [],
  "commandsRun": [
    {
      "command": "Read required catalog/template and minimum relevant local CanonLane, HoldMode, Transport, guitar-input, HarmonyEngine, and Tauri router code via read-only file tools",
      "result": "passed",
      "summary": "Confirmed preset contract, proportional scheduling equations, >2-beat phrase reset, transport freeze behavior, hold-mode lifecycle, generated harmony-stack risk, and guitar event model."
    },
    {
      "command": "Focused web research and source retrieval across manuscript/scholarly, performance-practice, MIDI-standard, and guitar-interaction angles",
      "result": "passed",
      "summary": "Retained institutional, peer-reviewed/university-press, specialist-performance, MIDI Association, and professional engineering-practice sources; rejected derivative mirrors."
    },
    {
      "command": "Manual artifact validation against Role C required sections and user acceptance contract",
      "result": "passed",
      "summary": "All requested interaction dimensions, Play-it-like line, 30–60 second exercise, source quality fields, confidence, gaps, and cleanup evidence are present."
    }
  ],
  "validationOutput": [
    "Standalone Markdown written to the authoritative /tmp path.",
    "No sibling reports were read.",
    "No copied melody is included; the only example is an abstract scale-degree cell.",
    "Acceptance exercise duration is 45 seconds and requires balanced source/generated NoteOn/NoteOff counts plus zero final active/pending state."
  ],
  "residualRisks": [
    "Exact preset ratios, delays, stack sizes, and effective HoldMode remain unspecified until parent synthesis/configuration.",
    "The proposed timing tolerance is not yet benchmarked across all four product surfaces.",
    "Current per-canon-voice harmony stacks can create more density than a line-only mensuration metaphor suggests."
  ],
  "noStagedFiles": true,
  "diffSummary": "Added one temporary standalone research artifact; no repository diff.",
  "reviewFindings": [
    "no blockers",
    "review note: parent synthesis must set and publish exact ratios/delays/HoldMode before automated timing acceptance can use fixed expected values"
  ],
  "manualNotes": "Read-only research task. No tests or project edits were appropriate; no staging operation was performed."
}
```
