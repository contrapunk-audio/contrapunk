# Research 02 — Fux Species Counterpoint (Species 1-5)

Agent: `research-species-counterpoint`. Based on Fux *Gradus ad Parnassum* (1725, Mann 1965 translation), Jeppesen, Salzer, Open Music Theory.

## Current state of Contrapunk's `StrictCounterpoint`

**Contrapunk is doing a *relaxed* Species 1**. It has:
- ✅ Parallel perfect rejection (P5, P8, unison)
- ✅ Stepwise motion preference (+4 for 1-2 semitones)
- ✅ Contrary motion preference (+3)
- ✅ Interval variety penalty (3+ same in last 4)
- ✅ Melody-repeats forces harmony move

But is **missing** (can all be added today, ~40 lines in `score_candidate`):
- ❌ Explicit vertical consonance filter (P4 currently leaks through)
- ❌ Leap recovery (leap then step opposite)
- ❌ Melodic 7th rejection (hard)
- ❌ Augmented/diminished melodic interval rejection
- ❌ Ambitus cap (max 10th)
- ❌ Hidden 5ths/8ves (similar motion into perfect)
- ❌ Tritone outline detection (5-note window)
- ❌ Beginning rule (first interval must be perfect consonance)
- ❌ Ending rule (clausula vera — needs phrase awareness)
- ❌ Climax rule (single apex, touched once — needs phrase awareness)

## Global rules (all species)

### Consonance classification (Fuxian two-voice)
- **Perfect**: P1, P5, P8
- **Imperfect**: m3, M3, m6, M6
- **Dissonant**: 2nds, **P4** (yes, P4 is dissonant here), 7ths, all augmented/diminished, tritone

### Universal motion rules (HARD)
1. No parallel perfect consonances (P5, P8, unison)
2. No reaching perfect consonance by similar motion (hidden/direct 5ths/8ves)
3. No voice crossing (Fux allows briefly; modern editors forbid)
4. No unison except on first and last note

### Universal melodic rules (HARD)
1. No augmented/diminished melodic intervals
2. No melodic leap of 7th or > octave
3. No tritone outline across 2-3 notes with direction reversal
4. Leaps > 3rd must be followed by stepwise motion in opposite direction (recovery)
5. No consecutive leaps same direction (unless outlining a consonant triad)
6. Max single leap: octave; preferred 3rd, 4th, 5th, m6; **M6 forbidden melodically**

### Universal melodic rules (SOFT)
1. **Climax**: exactly one apex, touched once, not coinciding with CF apex, not on leading tone
2. **Ambitus**: ≤ 10th, prefer octave/9th
3. Stepwise predominance
4. More imperfect than perfect consonances
5. Prefer contrary or oblique over similar
6. Avoid repeated notes (species 1-4)
7. Mode unity; accidentals only for *musica ficta* at cadences

## Species 1 — Note against note (1:1)

### Allowed intervals
Only consonances on every beat: P1, m3, M3, P5, m6, M6, P8. **P4 forbidden**.

### Beginning (HARD)
First interval must be perfect consonance.
- CP above CF: P1, P5, P8
- CP below CF: P1 or P8 only (P5 below creates implied 6/4)

### Ending (HARD)
- Last interval: P1 or P8 (octave/unison on final)
- Penultimate interval: M6 (CP above, CF descends 2-1) or m3 (CP below, CF ascends 7-1) — this is the *clausula vera*
- Dorian/Mixolydian/Aeolian: raise 7th for leading tone (musica ficta)
- Contrary stepwise motion into final required

### Motion
- No parallel P1/P5/P8
- No hidden 5ths/8ves
- Prefer contrary motion
- Max 3 parallel imperfect consonances in a row

### Dissonance
**None allowed anywhere**. Species 1 is the only species where every vertical is consonant.

## Species 2 — Two notes against one (2:1)

### Setup
CP: two half notes per CF whole note. **Strong beat** (downbeat) and **weak beat** (upbeat) now exist.

### Intervals
- **Strong beat**: must be consonant
- **Weak beat**: consonant OR dissonant **as passing tone only** — approached by step from strong, resolved by step in same direction to next strong. No neighbor, no suspension, no cambiata.

### Parallels
Checked **between successive strong beats** AND weak-beat to next-strong-beat. Parallels separated by a weak beat are still forbidden ("on-the-beat fifths" rule).

### Start/end
- Start: half rest + weak beat consonance, OR downbeat perfect consonance
- End: clausula vera as species 1, final is whole note

## Species 3 — Four notes against one (4:1)

### Setup
4 quarters per CF whole note. Beat 1 strongest, then 3 > 2 = 4.

### Intervals
- **Beat 1**: consonant (strict)
- **Beats 2, 3, 4**: consonant or dissonant as passing/neighbor, stepwise approach/resolution

### Dissonance figures (the species-3 flavor)
1. **Passing tone**: dissonant beat 2/3/4 approached + left by step same direction
2. **Neighbor tone**: consonant-dissonant-back-to-same-consonant, stepwise both ways
3. **Double neighbor / changing tones**: 4-note figure (e.g., C-D-B-C or C-B-D-C) — beats 2-3 are a step above and below a consonant pitch, the leap between them is a 3rd. **The one place in species 3 where leaping to/from dissonance is allowed.**
4. **Nota cambiata**: 5-note figure across 2 measures: consonant → step down (dissonant) → leap down 3rd (consonant) → step up → step up. Notes 1, 3, 4, 5 consonant; note 2 dissonant exits by leap of 3rd down.

## Species 4 — Suspensions (syncopation)

### Setup
Half notes, **tied across bar line**. CP attacks on upbeat, ties into downbeat where it may now form dissonance.

### The 3-stage event
1. **Preparation (P)**: weak beat, must be consonant
2. **Suspension (S)**: tied to next strong beat; CF changes; CP now may be dissonant
3. **Resolution (R)**: next weak beat, CP moves **down by step** to consonance

### Valid suspensions (HARD set)
**CP above CF**:
- **9-8**: 9th → 8ve. Weak (resolution is perfect).
- **7-6**: 7th → 6th. **Best** (resolves to imperfect consonance).
- **4-3**: 4th → 3rd. **Best** (resolves to imperfect consonance).
- Do NOT chain two 9-8s (parallel octaves on resolution).

**CP below CF**:
- **2-3**: 2nd → 3rd below. **Best for CP-below.**
- **7-8 below**: forbidden (hidden octaves on resolution)

### HARD rules
- Resolution always stepwise **downward**. Never upward, never by leap.
- Resolution pitch must be consonant with CF
- Preparation must be consonant with CF of its beat
- Cannot suspend into a perfect consonance that creates parallels
- If no legal suspension possible at a bar, fall back to species 2 motion for that bar ("breaking the ligature" — Fux permits)

## Species 5 — Florid counterpoint

### Setup
Free mixture of whole, half, quarter, and paired eighth notes, plus species-4 suspensions.

### New rules (beyond inherited species 1-4)
1. **Rhythmic variety**: no more than 2 consecutive bars of identical rhythm
2. **Eighth notes**: only in pairs, only on weak beats (ands of 2 or 4), never on beat 1 or 3. Both eighths stepwise approached/left. Max 1 pair per bar.
3. **Decorated suspensions**: resolution may be delayed/ornamented by 1 interpolated consonant note (anticipation, portamento, échappée). Still resolves by step down.
4. **Tied halves** (syncopation) across bar line encouraged — species 5 signature
5. Cadence is always a species-4 suspension cadence

## State required per species

| Field | Sp1 | Sp2 | Sp3 | Sp4 | Sp5 |
|---|---|---|---|---|---|
| `prev_cf_note`, `prev_cp_note` | ✓ | ✓ | ✓ | ✓ | ✓ |
| `prev_interval_class` | ✓ | ✓ | ✓ | ✓ | ✓ |
| `last_melodic_move` (dir+size) | ✓ | ✓ | ✓ | ✓ | ✓ |
| `consecutive_leap_count` | ✓ | ✓ | ✓ | ✓ | ✓ |
| `was_last_a_leap` | ✓ | ✓ | ✓ | ✓ | ✓ |
| `running_cp_min/max`, `apex_count` | ✓ | ✓ | ✓ | ✓ | ✓ |
| `beat_position` | — | ✓ | ✓ | ✓ | ✓ |
| `prev_beat_strong_interval` | — | ✓ | ✓ | ✓ | ✓ |
| `tied_pitch` | — | — | — | ✓ | ✓ |
| `suspension_phase` (P/S/R) | — | — | — | ✓ | ✓ |
| `current_rhythm_mode` | — | — | — | — | ✓ |
| `bars_in_current_rhythm` | — | — | — | — | ✓ |
| phrase-level: `cf_length`, `cf_apex_bar`, `planned_cp_apex_bar` | ✓ | ✓ | ✓ | ✓ | ✓ |

Species 1: ~6 fields. Species 5: ~14 fields plus 1-bar lookahead.

## Reactive vs lookahead feasibility

| Species | Reactive feasible? | Notes |
|---|---|---|
| 1 | Mostly yes | Cadence + climax + ambitus need phrase awareness |
| 2 | Yes | +beat strong/weak toggle |
| 3 | Mostly yes | +5-note rolling buffer for cambiata/double neighbor |
| 4 | Yes | 3-phase state machine. Fall back to species 2 when no legal suspension. API needs to return 2 notes per CF tick |
| 5 | **Partially** | Needs 2-4 bar buffer for rhythmic variety and ornamented suspensions. Fully reactive species 5 sounds random |

## 7 rules to add TODAY to strict species 1

All local, no API changes, ~40 lines in `CounterpointState::score_candidate`:

1. **Explicit vertical consonance filter**: reject if `semitones ∈ {1, 2, 5, 6, 10, 11}` mod 12 (m2, M2, P4, tritone, m7, M7)
2. **Leap recovery**: if `|last_harmony_move| > 4 semitones`, force next move `≤ 2 semitones` in opposite direction. Add `last_harmony_move: i8` field
3. **No melodic 7th**: hard reject harmony move of 10 or 11 semitones
4. **No aug/dim melodic intervals**: reject 6-semitone moves (tritone melodically)
5. **Ambitus cap**: reject if `max_harmony - min_harmony > 16 semitones` (10th)
6. **Hidden 5ths/8ves**: when harmony and melody move same direction into perfect consonance, reject unless harmony move is stepwise
7. **Tritone outline**: 5-note rolling harmony buffer; penalize if buffer traces 6-semitone span then reverses direction

## API extensions needed for species 2-5

- Current: `process(&mut self, scale: &mut Scale, melody: Note) -> Vec<Note>` (1 melody in, variable harmony out)
- Species 2: OK as-is (returns 2 halves)
- Species 3: OK as-is (returns 4 quarters)
- Species 4: Need tie metadata — `Vec<(Note, TieKind)>`
- Species 5: Need variable-length `Vec<(Note, Duration)>`, OR buffer CF notes and emit in bar chunks

## Phrase-level problem

Contrapunk's reactive infinite-stream model can't handle:
- **Cadence**: last 2 bars must form clausula vera — must plan backward from end of CF
- **Climax**: single apex touched once — requires pre-planned apex bar
- **Ambitus guarantee**: need pre-computed `[cp_low, cp_high]` clamp

Two options:
1. **Phrase-boundary flag**: caller signals `end_of_phrase()`, state machine plans last 2 bars as cadence
2. **Pre-computed CF mode**: when CF known in advance, two-pass — first pass chooses apex+cadence, second pass fills with local rules

Recommend exposing both. Species 1-3 can stream with "best-effort" cadence. Species 4-5 should require known CF.

## Sources
- Fux, *Gradus ad Parnassum*, 1725 — IMSLP scan
- Mann (trans.), *The Study of Counterpoint from Johann Joseph Fux's Gradus ad Parnassum*, Norton 1965
- Jeppesen, *Counterpoint*, 1939/1960 — Palestrina-style relaxations
- Salzer & Schachter, *Counterpoint in Composition*, 1969 — Schenker-influenced
- Piston, *Counterpoint*, 1947
- Schoenberg, *Preliminary Exercises in Counterpoint*, 1963
- Open Music Theory species chapters (viva.pressbooks.pub/openmusictheory/)
- music21 species counterpoint module: github.com/cuthbertLab/music21/blob/master/music21/counterpoint/species.py
- Komosinski & Szachewicz, *Automatic species counterpoint by dominance relation*, Poznań UT
- Donnelly & Sheppard, *Evolving Musical Counterpoint* (GA), arxiv:1207.5560
