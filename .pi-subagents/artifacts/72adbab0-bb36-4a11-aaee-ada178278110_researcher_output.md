# Research: Preset 08 — Suspension Garland (Role B: theory and temporal behavior)

**Date:** 2026-03-10  
**Scope:** Fuxian fourth-species pedagogy; selected corpus evidence about Palestrina/Renaissance practice; exact current `CounterpointState`, `HarmonyEngine`, transport, `CounterpointLane`, Hold, and lifecycle behavior. No sibling preset research was read.

## Summary

Suspension Garland should be a bounded suspension texture, not generic “slow Palestrina”: a consonant preparation is sustained into a changed lower/other voice, creating controlled dissonance, then the suspended voice descends by step to consonance; toward a cadence, that imperfect consonance should continue to a perfect interval. Fux supplies the strict two-voice training grid, while corpus studies show Palestrina/Renaissance practice is freer in duration, texture, metric level, chains, and cadential function. Their defensible intersection is **prepared retention/repetition, contextual dissonance, descending-step consonant resolution, smooth independent voices, and phrase-directed cadence**.

**Activation decision: `needs_correctness_fix`.** The shared `CounterpointState` approximates the pitch sequence but does not produce a reliable tie/hold contract, prepares on strong rather than weak position, resolves on the next NoteOn rather than a defined weak phase, hardcodes 4/4, and lacks cadential perfection. More decisively, `CounterpointLane` Species4 is explicitly only a half-beat delayed attack (“suspension-ready”) with no suspension or resolution machinery. Enabling the preset now would mislabel delay as fourth species.

## Findings

### 1. Fux pedagogy and Palestrina corpus practice are related, not identical

**Fux / strict fourth species (pedagogical model).** Two half-note counterpoint events oppose each whole-note cantus event; the weak-half preparation is consonant and tied at the same pitch into the next strong half, where it may become dissonant; a dissonance resolves downward by one diatonic step on the following weak half to consonance. Pure species therefore emphasizes oblique motion at the suspension and uses 7–6 and 4–3 above, 2–3 below, with 9–8 possible but less desirable because it resolves to a perfect consonance. Consecutive weak-beat fifths/octaves remain forbidden. Beginning with a half rest establishes the displacement; strict closure uses a penultimate 7–6 or 2–3 suspension and ends at the final unison/octave. [Open Music Theory, “Fourth-Species Counterpoint”](https://viva.pressbooks.pub/openmusictheory/chapter/fourth-species-counterpoint/) (a modern exposition explicitly grounded in Fux’s examples); [Fux, *Gradus ad Parnassum* scan](https://s9.imslp.org/files/imglnks/usimg/2/22/IMSLP286170-PMLP187246-gradusadparnassu00fuxj_0.pdf).

**Palestrina / repertoire practice.** Species are a later pedagogical partition, not a claim that Palestrina wrote uninterrupted fourth-species exercises. In 36 Palestrina Agnus movements, a corpus study found suspension clusters on both the strongest beat 1 (265 cases) and strong beat 3 (313 cases); their melodic signature was pitch repetition into the dissonance followed by step down, generally at quarter- or half-note duration. The same repertoire also contains passing tones and neighbors, so suspension is one category within mixed florid counterpoint. The authors warn that their sample, detection limits, and omission of ties constrain generalization. [Anders & Inden 2019, PeerJ Computer Science](https://doi.org/10.7717/peerj-cs.244).

**Wider Renaissance corrective.** Renaissance suspensions may be binary or ternary according to the active contrapuntal rhythm, not merely the notated meter. Morgan argues for a fourth, often metrically strong **perfection** phase after the ordinary resolution; chains defer that perfection until the final link. This makes actual Renaissance suspensions strongly cadential and phrase-oriented, whereas pure species creates them nearly continuously. A 1,100-piece/movement early- and mid-Renaissance corpus found 31,710 suspensions, 1,215 ternary (3.83%); this is broader than Palestrina and must not be quoted as a Palestrina rate. [Morgan 2019, *Intégral*](https://theory.esm.rochester.edu/integral/33-2019/morgan/).

**Intersection for this preset.** Preserve the audible causal chain `consonant preparation → same pitch retained while another voice changes → contextual dissonance → suspended voice descends by step → consonance`, smooth ranges, and cadential intensification. Do **not** equate Fux’s fixed two-half-note grid with the whole Palestrina corpus, or claim all Palestrina suspensions are strong-beat binary 4/4 figures.

### 2. Testable stylistic invariants

1. **Preparation validity:** before every intended dissonant suspension, the patient pitch must sound consonantly against all structurally relevant voices; it must exist before the dissonance, not be attacked simultaneously with it.
2. **Identity across the boundary:** preparation and suspension are the same MIDI pitch and same logical note ownership. Prefer an actual held note; if MIDI requires retrigger, suppress duplicate attack and preserve one balanced NoteOff.
3. **Contextual creation:** at the suspension phase the patient stays while an agent voice moves, so the vertical interval changes by oblique motion. A merely delayed consonant attack is not a suspension.
4. **Controlled classes:** for a two-voice strict baseline, use 7–6 and 4–3 above, 2–3 below; permit 9–8 sparingly. Classification must be diatonic/voice-relative, not only pitch-class arithmetic, because 2 vs 9 and compound/register identity matter.
5. **Metric phase:** binary strict baseline is weak preparation → strong suspension → weak resolution. In 4/4, strong locations may be beats 1 and 3, matching the Palestrina subcorpus; time signature and contrapuntal rhythm must remain parameters rather than silently hardcoded assumptions.
6. **Resolution:** a dissonant patient descends exactly one scale step at the scheduled resolution phase and must be consonant in the full texture. No unresolved timeout, arbitrary substitute pitch, upward resolution, or resolution merely because another NoteOn happened.
7. **Perfect-motion safety:** no parallel/uncovered perfect fifths or octaves across structural consonances; avoid consecutive after-beat perfects. Prefer imperfect consonances and contrary/oblique motion; allow similar motion only when it does not expose a perfect interval.
8. **Cadence:** phrase endings intensify through a prepared penultimate suspension; after its imperfect resolution, proceed to unison/octave “perfection” on the target/final. The last event must not remain suspended.
9. **Singability and register:** each generated line is mostly stepwise; leaps restore range and are followed by contrary stepwise recovery. Keep each lane within an approximately octave-to-tenth working ambitus and prevent crossing; outer-voice spacing may be wider than adjacent upper voices.
10. **Lifecycle:** transport stop/reset, panic, lane disable, preset/species/key change, routing stop, and lost input must cancel future attacks and emit balanced releases for every sounding generated note. No delayed attack may survive its owner unless Hold explicitly permits it.

### 3. Variables that preserve identity

- Mode/tonal center: diatonic church-mode/major-scale approximation may vary; modest ficta at cadence is desirable only if explicitly supported. Tuning may remain 12-TET as an honest approximation.
- Patient above or below cantus; prioritize 7–6/4–3 above and 2–3 below.
- Binary suspension unit (e.g. half-beat, one beat, two beats) and tempo, provided P–S–R proportions and phase remain stable.
- Density: 2 voices is the safest strict form; 3–4 voices may stagger patients, but validate preparation/resolution against the full sonority and avoid simultaneous indiscriminate dissonances.
- Chain length: 1–3 links normally; longer chains only near a peak/cadence with explicit final perfection.
- Phrase length and silence: approximately 2–8 bars, with a rest/breath after cadence.
- Articulation/dynamics: legato, restrained attack, modest crescendo into the chain/cadence and release after perfection.
- Occasional consonant syncopation or one brief species break is acceptable when no valid dissonant suspension exists, followed by prompt restoration of the tie pattern.

### 4. Misleading behaviors / rejected shortcuts

- **Rejected: “delay every harmony by 0.5 beat.”** This is the current Lane shortcut; it supplies syncopation but no preparation, contextual dissonance, hold across the boundary, scheduled downward resolution, or cadence.
- **Rejected: repeat a pitch on every strong beat.** An attacked/repeated pitch can stand in for a tie in corpus analysis, but product behavior must still prove prior consonance and logical continuity; repetition alone is not enough.
- **Rejected: any strong-beat dissonance.** Passing/neighbor/cambiata behavior and arbitrary clusters are different categories.
- **Rejected: resolve on next player NoteOn.** Player rhythm is not necessarily the resolution phase.
- **Rejected: pitch-class-only legality.** It cannot distinguish compound suspension type, voice function, spelling, or consonance against every voice.
- **Rejected: endless uniform garland.** Fux’s exercise density is pedagogy; repertoire practice mixes textures and directs suspensions toward cadence/perfection.
- **Rejected: static block chords with a delayed top note, blanket reverb, octave-spread duplication, or many parallel thirds.** None creates independent suspension syntax; octave duplication can introduce forbidden structural perfects and mud.
- **Rejected: claim of “Palestrina imitation.”** The engine lacks text underlay, modal ficta policy, mensural/contrapuntal-rhythm inference, full polyphonic legality, and repertoire-shaped cadence planning.

### 5. Temporal FSM and triggers

```text
REST / FREE
  NoteOn + valid transport phase + enough future duration
  -> PREPARE_CANDIDATE

PREPARE_CANDIDATE (weak phase)
  choose patient consonant against current texture;
  attack once; record pitch, owner, lane, preparation sonority
  -> PREPARED_HELD
  if no legal pitch -> CONSONANT_SYNCOPATION or FREE

PREPARED_HELD
  strong-phase boundary + agent change makes approved dissonance
  -> SUSPENDED
  strong boundary but still consonant -> CONSONANT_SYNCOPATION
  owner release -> Hold policy (Cancel / bounded continuation); never orphan note

SUSPENDED
  keep same logical note sounding; no duplicate attack
  scheduled next weak phase -> RESOLVE
  stop/reset/panic/disable -> RELEASE_AND_RESET

RESOLVE
  patient moves down one diatonic step; old pitch NoteOff and resolution NoteOn
  if full-texture consonant -> RESOLVED
  else choose prevalidated alternative before suspension; never improvise after deadline

RESOLVED
  mid-phrase + chain requested: resolution may also prepare next link -> PREPARED_HELD
  cadence trigger (phrase age, explicit rest/release, bar boundary, target degree) -> PERFECT
  otherwise -> FREE

PERFECT
  move independent voices to legal unison/octave goal on metrically strong point;
  sustain briefly -> RELEASE / REST
```

**Phrase evolution:** `rest/clear attack → one prepared suspension → alternating/staggered links → 2–3-link density peak → downward resolution → strong-beat perfection → thinning/rest`. Across sections, vary register or patient lane and chain length, not the P–S–R law. A phrase-end input rest should close or safely abort the figure, never launch a delayed orphan.

### 6. Three abstract examples (scale degrees and beats)

Assume diatonic mode, one beat = one CR unit, patient above cantus unless stated. These are invented abstractions, not melodies.

1. **7–6 above, binary cadence approach**  
   Cantus/agent: beat 0 `3`, beat 1 `4` (held through beat 2). Patient: beat 0 weak `2` (consonant preparation), beat 1 strong retain `2` (a 7th above `4`, dissonant), beat 2 weak descend to `1` (6th above `4`, consonant). Expected: one patient attack at beat 0, no reattack at beat 1, balanced change at beat 2.

2. **4–3 above into perfection**  
   Agent: beat 0 `5`, beat 1 `1`, beat 2 `1`, beat 3 `1`. Patient: beat 0 weak `4` (consonant), beat 1 strong retain `4` (4th above agent, dissonant), beat 2 weak descend `3` (imperfect consonance), beat 3 strong move to `1`/`8` with contrary agent motion as legal (perfect cadence goal). Expected phrase thinning after beat 3.

3. **2–3 below and a two-link chain**  
   Upper cantus/agent: beats 0–4 `3, 2, 2, 1, 1`. Lower patient: beat 0 weak `2` (prep), beat 1 strong retain `2` (2nd below cantus), beat 2 weak descend to `1` (3rd below and simultaneously next preparation where full texture permits), beat 3 strong retain `1` as next controlled dissonance, beat 4 weak descend to `7`, then a later strong perfection on `1`/octave. Expected: the first link’s perfection is deferred; the final link supplies it.

### 7. Exact current Species IV mapping and gaps

#### Shared `CounterpointState` (`crates/contrapunk-harmony/src/stateful.rs`)

**Mapped now:**
- Has `Free → Prepared → Suspended → Resolving` state plus preparation/suspension pitches and a timeout.
- Uses consonant pitch selection from the Species-1 scorer; strict scorer rejects vertical 2nds/4ths/tritone/7ths, parallel perfects, hidden perfects, and large/tritone melodic leaps, while preferring steps, contrary motion, interval variety, and bounded recent ambitus.
- In Species4, a prepared pitch may be retained when the new melody makes pitch-class 1, 2, 5, 10, or 11; resolution attempts one diatonic step downward and checks consonance.
- State reset clears suspension FSM and rolling history; per-voice FSM is preserved when cascade history is inherited.

**Correctness gaps:**
- `Free` marks a preparation only when `is_strong`; strict Fux preparation should occur on the preceding weak phase.
- `Prepared` only attempts suspension on a later `is_strong` call. There is no explicit weak→strong duration contract, and repeated calls at arbitrary NoteOn times drive the FSM.
- `Suspended` resolves on the very next call regardless of beat strength; it does not require the next weak phase.
- Returning the same held MIDI pitch in a fresh harmonization result is not an actual tie. The per-input `active_notes` map tracks by player pitch, so continuity and balanced ownership across two different player notes are not represented as one patient note.
- No explicit attack/tie output survives: `TieKind` and `CounterpointOutput` are declared but not used by `process_with_beat`, which returns only pitches.
- `beat_strength(bp, 4)` hardcodes 4 beats/bar; engine time-signature changes are not passed here.
- Generic dissonance pitch classes are broader than named suspension grammar and lose compound/above-below identity; no full multi-voice sonority validation occurs after chain generation and octave shifting.
- Timeout counts calls, not musical time, and resets without guaranteeing a resolution.
- No cadence/perfection phase, chain policy, phrase detector, simultaneous-patient coordination, or suspension-density evolution.
- Direction is applied post hoc for Species2–4 by octave-shifting the chosen pitch. This can change interval identity/register after the FSM’s consonance/dissonance decision.

#### `HarmonyEngine` (`crates/contrapunk-harmony/src/engine.rs`)

**Mapped now:** external `counterpoint_beat_phase` is preferred; if absent, Species2–4 use a synthetic counter advanced by `1.0` per `harmonize_note_on`. It maintains independent `CounterpointState`s per chain step, stores generated pitches for matching NoteOff, queues held inputs for reharmonization after parameter changes, and exposes full lifecycle clears.

**Gaps:** the synthetic counter is event count modulo 4, not elapsed beat time, so it cannot make long legato notes cross real beats/barlines. External phase is sampled only when NoteOn processing asks for harmony; no clock tick advances the suspension FSM or emits a resolution by itself. Thus the catalog’s “transport required” must remain true for this preset even though generic engine fallback makes Species4 superficially differ without transport.

#### Dedicated `CounterpointLane` (`crates/contrapunk-companion/src/counterpoint_lane.rs`)

**Mapped now:** schedules Species4’s chosen consonant pitch at `now + 0.5`, tracks pending emissions by player-note owner, routes NoteOn/Off, serializes species and Hold override, and clears queues/state/history on disable/reset.

**Blocker:** its own comment says v1 Species4 is “just the half-beat delay; proper resolution machinery follows.” It calls `process_directed` (Species-1 pitch selection), not `process_with_beat`, and never uses its cantus history to prepare, suspend, resolve, chain, or perfect. The v1.2.0 release calls Species4 “suspension-ready,” which accurately confirms non-completion. [Contrapunk v1.2.0 release](https://github.com/contrapunk-audio/contrapunk/releases/tag/v1.2.0).

### 8. Transport, Hold, and lifecycle contract

- **Transport:** Tauri router pushes `Some(transport.beat_position())` while running and `None` while stopped (`src-tauri/src/commands/engine.rs`). However `CounterpointState` assumes 4/4, while the transport supports configurable time signatures (`src-tauri/src/commands/transport.rs`). For the preset, require running transport and either constrain UI copy/config to 4/4 or pass actual meter and contrapuntal subdivision through the API.
- **Stop/reset:** both call `request_all_notes_off`; this sends synth AllNotesOff, resets Companion runtime, routes CC123 when possible, and otherwise clears engine note tracking. This is the right hard lifecycle boundary. `stop_routing` only signals thread exit in the shown command path; ensure teardown dispatches all external/synth releases before preset acceptance.
- **Hold semantics:** Lane override beats global. `Cancel` drops future owned attacks; `NearFuture` permits only emissions within `tail_beats` and schedules releases at the horizon; `PhraseEnd` permits attacks through the current bar; `Forever` permits all pending attacks. Already-emitted notes are released at input NoteOff (or NearFuture horizon)—so `Forever` does **not** actually sustain emitted notes forever. For Suspension Garland, use a bounded mode (prefer `NearFuture` sized through resolution, or a dedicated atomic-figure hold); `Cancel` will truncate preparations on short input, while `PhraseEnd`/`Forever` can permit a delayed attack after release and then immediately schedule its NoteOff before its NoteOn fires, producing lifecycle ambiguity.
- **Parameter changes:** shared engine setters call `clear_active_for_reharm`; Tauri’s panic/reharm path replays held inputs and diffs sounding harmony. Replaying inputs advances the synthetic beat counter and can restart/corrupt a suspension FSM; preset correctness requires phase-stable reharm or atomic abort/restart at a legal preparation boundary.
- **Lane reset/disable:** clears pending queues, held map, state, and cantus history, but dispatching releases for notes already emitted depends on Companion/router reset. Acceptance must test external MIDI as well as synth.

### 9. Minimum correction before activation

Do not build a new abstraction first. Complete the existing `CounterpointLane` Species4 path and reuse its scheduler/ownership:

1. Schedule a consonant preparation on the weak phase, retain one logical sounding pitch into the next strong phase, then schedule validated downward-step resolution on the next weak phase.
2. Represent note ownership across phases so tie means no duplicate NoteOn and exactly one eventual NoteOff; precompute P–S–R before attacking preparation.
3. Pass actual meter/subdivision; remove event-count timing for this transport-required preset.
4. Add a cadence/perfection terminal event or keep UI claim narrowly “prepared suspensions” until that exists.
5. Add one end-to-end check: long input spanning a bar emits `prep NoteOn → no attack at suspension → prep NoteOff + lower-step resolution NoteOn → resolution NoteOff`, and stop/reset at every phase leaves zero active notes.

Until those are true, preset state should remain unavailable/blocked rather than “Current Species IV baseline.”

## Confidence, disagreements, and gaps

- **High confidence:** P–S–R identity, consonant preparation, downward-step consonant resolution, strict binary placement, common suspension classes, and current code gaps. These are directly documented and visible in code.
- **High confidence:** Palestrina subcorpus suspensions occur on beats 1 and 3 with repetition then step down; numerical counts are limited specifically to the first 36 Agnus movements in music21, not his complete works.
- **Medium-high confidence:** Renaissance cadence/perfection and chain model; Morgan provides treatise/score/corpus support, but the corpus is broader and mainly earlier than Palestrina.
- **Medium confidence:** exact product defaults for chain length, ambitus, and Hold horizon; these are practical bounded choices, not historical constants.
- **Disagreement surfaced:** elementary/Fux teaching calls the suspension an accented strong-beat dissonance. Morgan shows that Renaissance ternary suspensions can place dissonance weakly because the later perfection governs metric orientation. Resolution: implement strict binary Fux as preset baseline but do not market strong-beat placement as universal Palestrina practice.
- **Gap:** direct English extraction of the Fux primary PDF was blocked; the scan was located, while detailed rules were cross-checked through Open Music Theory’s Fux-derived annotated examples. No invented quotation is used.
- **Gap:** no complete Palestrina corpus frequency by suspension type, density, register, or phrase position was established. The Anders/Inden pilot is intentionally narrow and Morgan’s larger figures are not Palestrina-specific.
- **Gap:** no code path currently infers contrapuntal rhythm, modal ficta, textual phrase boundaries, or full-sonority dissonance in 3–4 generated voices.

## Sources

### Kept
- [Open Music Theory — Fourth-Species Counterpoint](https://viva.pressbooks.pub/openmusictheory/chapter/fourth-species-counterpoint/) — clear, inspectable Fux-derived rules, classes, beginnings/endings, parallels, and species breaks.
- [Fux — *Gradus ad Parnassum* (1725 scan)](https://s9.imslp.org/files/imglnks/usimg/2/22/IMSLP286170-PMLP187246-gradusadparnassu00fuxj_0.pdf) — primary treatise anchor; extraction was blocked.
- [Anders & Inden 2019 — “Machine learning … Dissonance treatment in Palestrina”](https://doi.org/10.7717/peerj-cs.244) — peer-reviewed direct corpus evidence with sample and method limitations stated.
- [Morgan 2019 — “Renaissance Ternary Suspensions in Theory and Practice”](https://theory.esm.rochester.edu/integral/33-2019/morgan/) — score/treatise/corpus basis for contrapuntal rhythm, chains, cadence, and perfection.
- [Contrapunk v1.2.0 release](https://github.com/contrapunk-audio/contrapunk/releases/tag/v1.2.0) — public confirmation that Lane Species4 is only “suspension-ready.”
- Local exact inputs and code: `.planning/phases/10.2-arrangement-presets/10.2-PRESET-CATALOG.md`; `.planning/phases/10.2-arrangement-presets/10.2-RESEARCH-TEMPLATE.md`; `crates/contrapunk-harmony/src/stateful.rs`; `crates/contrapunk-harmony/src/engine.rs`; `crates/contrapunk-companion/src/counterpoint_lane.rs`; `src-tauri/src/commands/engine.rs`; `src-tauri/src/commands/transport.rs`.

### Dropped
- SEO/tutorial duplicates of fourth-species rules — redundant after the primary scan and Open Music Theory.
- Informal forum discussion of Fux parallels — not authoritative.
- `Voice-leading in Palestrina’s masses` DOI page — blocked (403) and not needed for a claim beyond accessible sources.
- Search snippets for broader Palestrina cadence and density claims — insufficient direct evidence; recorded as gaps instead.

## Gaps / suggested next steps

Before implementation review, obtain (1) a score-verified Palestrina sample stratified by phrase position and voice count, and (2) an end-to-end event trace for Tauri, WASM, and plugin transport/lifecycle surfaces. These should refine defaults, not weaken the correctness blocker: delayed attack alone is not Species IV.

```acceptance-report
{
  "criteriaSatisfied": [
    {
      "id": "criterion-1",
      "status": "satisfied",
      "evidence": "Created only the requested Role B report at /tmp/contrapunk-suspension-garland-theory.md; project/source and sibling preset files were not modified or read."
    },
    {
      "id": "criterion-2",
      "status": "satisfied",
      "evidence": "Report includes dated citations, Fux/Palestrina separation and intersection, invariants, variables, rejected shortcuts, FSM/triggers, three abstract examples, exact engine/lane/transport/Hold/lifecycle mapping, gaps, confidence, and an explicit activation decision."
    }
  ],
  "changedFiles": [
    "/tmp/contrapunk-suspension-garland-theory.md"
  ],
  "testsAddedOrUpdated": [],
  "commandsRun": [
    {
      "command": "read exact catalog/template and targeted CounterpointState, HarmonyEngine, CounterpointLane, Tauri transport/router lifecycle files",
      "result": "passed",
      "summary": "Targeted read-only inspection completed; no sibling preset reports were read."
    },
    {
      "command": "focused web research and full-content fetch for Fux-derived rules, Palestrina corpus evidence, and Renaissance suspension/cadence research",
      "result": "passed",
      "summary": "Authoritative accessible sources retained; blocked primary PDF extraction and DOI page recorded as gaps."
    },
    {
      "command": "write /tmp/contrapunk-suspension-garland-theory.md",
      "result": "passed",
      "summary": "Requested report written to authoritative runtime path."
    }
  ],
  "validationOutput": [
    "Activation decision: needs_correctness_fix.",
    "Current CounterpointLane Species4 is only a half-beat delayed attack and lacks preparation/retention/resolution/perfection.",
    "No project/source files were edited."
  ],
  "residualRisks": [
    "Fux primary PDF extraction was blocked; primary scan is cited and detailed rules were cross-checked through a Fux-derived scholarly OER.",
    "Available Palestrina corpus evidence is a narrow 36-Agnus pilot, not a complete-work frequency study.",
    "No runtime tests were executed because the task was research-only and prohibited project/source modification."
  ],
  "noStagedFiles": true,
  "diffSummary": "Added one research artifact under /tmp; zero repository changes.",
  "reviewFindings": [
    "blocker: crates/contrapunk-companion/src/counterpoint_lane.rs - Species4 schedules only a delayed attack; it does not implement a suspension lifecycle.",
    "blocker: crates/contrapunk-harmony/src/stateful.rs - Species4 phases are NoteOn-driven, prepare on strong phase, resolve on the next call regardless of weak phase, hardcode 4/4, and do not encode a real tie or cadence/perfection."
  ],
  "manualNotes": "Keep preset unavailable until a clock-scheduled, lifecycle-safe preparation/hold/suspension/down-step-resolution path exists; add cadence/perfection before claiming Renaissance/Palestrina phrase behavior."
}
```
