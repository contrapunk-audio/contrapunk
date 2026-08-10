# CONTRAPUNK HARMONY ENGINE: DEEP DIVE

A comprehensive technical documentation of the Contrapunk harmony generation and voice-leading system.

---

## Table of Contents

1. [Architecture Overview](#architecture-overview)
2. [Scale System](#scale-system)
3. [Harmony Modes](#harmony-modes)
4. [Voice Leading & Voicing](#voice-leading--voicing)
5. [Stateful Processing](#stateful-processing)
6. [Configuration & Enums](#configuration--enums)
7. [Humanization Pipeline](#humanization-pipeline)

---

## Architecture Overview

### System Flow Diagram

```
┌─────────────────────────────────────────────────────────────────┐
│                    MIDI INPUT (NOTE ON/OFF)                     │
└────────────────────────┬────────────────────────────────────────┘
                         │
                         ▼
        ┌────────────────────────────────────┐
        │      HarmonyEngine                  │
        │  ┌──────────────────────────────┐  │
        │  │ 1. harmonize() [stateless]   │  │
        │  │ 2. harmonize_note_on()       │  │
        │  │ 3. harmonize_note_off()      │  │
        │  └──────────────────────────────┘  │
        └──────────┬─────────────────────────┘
                   │
    ┌──────────────┼──────────────┐
    │              │              │
    ▼              ▼              ▼
┌─────────┐  ┌──────────┐  ┌──────────────┐
│ Scale   │  │ Harmony  │  │ Stateful     │
│ Engine  │  │ Modes    │  │ States       │
└─────────┘  └──────────┘  └──────────────┘
    │              │              │
    ▼              ▼              ▼
┌─────────────────────────────────────────┐
│        VoiceLeadingProcessor             │
│  ┌─────────────────────────────────┐    │
│  │ revoice_chord()                 │    │
│  │ + cartesian_product generation  │    │
│  │ + rule checking & scoring       │    │
│  └─────────────────────────────────┘    │
└────────────┬────────────────────────────┘
             │
             ▼
    ┌─────────────────────┐
    │  HumanizeEngine     │
    │  ┌───────────────┐  │
    │  │ jitter        │  │
    │  │ velocity vary │  │
    │  │ swing/groove  │  │
    │  │ duration      │  │
    │  └───────────────┘  │
    └────────┬────────────┘
             │
             ▼
        ┌─────────────┐
        │  MIDI OUT   │
        │  (with FX)  │
        └─────────────┘
```

### Key Structs & Relationships

```
┌──────────────────────────────────────────────────────────┐
│ HarmonyEngine                                            │
├──────────────────────────────────────────────────────────┤
│ scale: Scale                                             │
│ mode: HarmonyMode                                        │
│ contrary_motion_state: ContraryMotionState              │
│ counterpoint_state: CounterpointState                   │
│ suspension_state: SuspensionState                       │
│ voice_leading_processor: VoiceLeadingProcessor          │
│ humanizer: Humanizer                                    │
├──────────────────────────────────────────────────────────┤
│ harmonize(note) → Vec<Note>                             │
│ harmonize_note_on(note) → Vec<Note>                     │
│ harmonize_note_off(note) → Vec<Note>                    │
└──────────────────────────────────────────────────────────┘
       ▲              ▲              ▲
       │              │              │
       │              │              │
    Scale       HarmonyModes    VoiceRules
```

---

## Scale System

### Scale Structure

The Scale represents a musical scale with diatonic operations.

```
┌──────────────────────────────────────────────────┐
│ Scale                                            │
├──────────────────────────────────────────────────┤
│ tonic: u8 (0-11, pitch class)                    │
│ mode: ScaleMode (Ionian, Dorian, etc.)          │
│ offsets: Vec<u8> (semitone offsets per degree)  │
│ interchange_enabled: bool                       │
│ borrowing_range: u8 (1-5)                       │
│ last_borrowed_from: Option<ScaleMode>           │
├──────────────────────────────────────────────────┤
│ Core Operations:                                │
│ • degree_of(note) → Option<usize>              │
│ • transpose_diatonic(note, degrees) → Note     │
│ • is_in_scale(note) → bool                     │
│ • snap_to_scale(note) → Note                   │
│ • harmonize_smart() [in/out-of-key handling]   │
└──────────────────────────────────────────────────┘
```

### In-Scale Harmonization (harmonize_smart)

When a note is in the scale:

```
User plays: D4 (in C major)
             │
             ▼
    is_in_scale() = true
             │
             ▼
transpose_diatonic(D4, +2 degrees)
             │
             ├─ Current degree: 1 (D)
             ├─ New degree: 3 (F)
             ├─ Semitone difference: 5
             │
             ▼
       Result: F4 (diatonic third)
```

### Out-of-Key Harmonization (3-level fallback)

When a note is NOT in the scale:

```
User plays: Db4 (NOT in C major)
             │
             ▼
    is_in_scale() = false
             │
    ┌────────┴────────┐
    │                 │
    ▼                 ▼
interchange_enabled?  consonant_chromatic?
    │                 │
    ▼                 ▼
┌──────────────┐   ┌───────────────────┐
│ Level 1:     │   │ Level 2: Chromatic│
│ Modal        │   │ Consonance        │
│ Interchange  │   │                   │
├──────────────┤   ├───────────────────┤
│ 1. Search    │   │ 1. Try intervals: │
│    borrowing │   │    [4,3,9,8,7,5]  │
│    sources   │   │    semitones      │
│ 2. Find      │   │                   │
│    parallel  │   │ 2. Prefer matches │
│    mode w/   │   │    to scale tones │
│    note      │   │                   │
│ 3. Use that  │   │ 3. Return first   │
│    mode's    │   │    valid interval │
│    diatonic  │   │                   │
│    third     │   │                   │
└──────────────┘   └───────────────────┘
         │                 │
         └────────┬────────┘
                  ▼
          Result: harmony note
```

### Borrowing Sources by Range

```
Range 1: [Aeolian, HarmonicMinor]
Range 2: + Dorian, MelodicMinor
Range 3: + Mixolydian, Phrygian
Range 4: + Lydian, PhrygianDominant
Range 5: + Locrian, Ionian, LydianDominant (all 11 modes)
```

### ScaleMode Intervals (Semitones from Tonic)

57 total scales across 10 families. `intervals()` returns `&'static [u8]` (no heap allocation).

**Diatonic (7 notes)**

| Mode | Intervals |
|------|-----------|
| Ionian (Major) | [0, 2, 4, 5, 7, 9, 11] |
| Dorian | [0, 2, 3, 5, 7, 9, 10] |
| Phrygian | [0, 1, 3, 5, 7, 8, 10] |
| Lydian | [0, 2, 4, 6, 7, 9, 11] |
| Mixolydian | [0, 2, 4, 5, 7, 9, 10] |
| Aeolian (Minor) | [0, 2, 3, 5, 7, 8, 10] |
| Locrian | [0, 1, 3, 5, 6, 8, 10] |

**Variable cardinality scales (5-8 notes)**

| Mode | Notes | Intervals |
|------|-------|-----------|
| Major Pentatonic | 5 | [0, 2, 4, 7, 9] |
| Minor Pentatonic | 5 | [0, 3, 5, 7, 10] |
| Minor Blues | 6 | [0, 3, 5, 6, 7, 10] |
| Whole Tone | 6 | [0, 2, 4, 6, 8, 10] |
| Barry Harris Major 6th Dim | 8 | [0, 2, 4, 5, 7, 8, 9, 11] |
| Bebop Dominant | 8 | [0, 2, 4, 5, 7, 9, 10, 11] |
| Diminished WH | 8 | [0, 2, 3, 5, 6, 8, 9, 11] |

See `ScaleMode::all()` for the complete list of 57 scales.

### Consonant Intervals for Chromatic Notes

```
PREFER ABOVE (intervals in semitones):
  [4, 3, 9, 8, 7, 5]  =  [M3, m3, M6, m6, P5, P4]

PREFER BELOW (intervals in semitones):
  [-3, -4, -8, -9, -5, -7]  =  [m3, M3, m6, M6, P4, P5]

STRATEGY:
  1st Pass: Find interval that lands on a scale tone
  2nd Pass: Use first valid consonant interval
```

---

## Harmony Modes

### Deterministic Harmony Modes Overview

```
┌──────┬──────────────────────┬──────────────┬─────────┐
│ Mode │ Name                 │ Type         │ Stateful│
├──────┼──────────────────────┼──────────────┼─────────┤
│  1   │ PassThrough          │ No harmony   │   No    │
│  2   │ DiatonicThirds       │ +2 degrees   │   No    │ Parallel Thirds
│  3   │ DiatonicFourths      │ +3 degrees   │   No    │ Parallel Fourths
│  6   │ ContraryMotion       │ Opposite dir │  Yes    │
│  7   │ StrictCounterpoint   │ Species 1    │  Yes    │ Counterpoint (basic)
└──────┴──────────────────────┴──────────────┴─────────┘

> **Note:** BarryHarris mode was removed — it was a duplicate of DiatonicThirds.
> Users who want Barry Harris chord/passing-tone parity should select a BH scale
> (BHMajor6thDim or BHMinor6thDim) with DiatonicThirds mode.
```

### Mode 2: Diatonic Thirds Algorithm

```
Input: Note, Scale reference
   │
   ▼
harmonize_smart(note, 2, above=true)
   │
   ├─ if in_scale(note):
   │    transpose_diatonic(note, +2) → third above
   │
   └─ if NOT in_scale:
      ├─ interchange_enabled?
      │  ├─ YES: harmonize_with_interchange(note, above=true)
      │  └─ NO: harmonize_chromatic(note, above=true)
      │
      └─ Result: harmony note or original if out-of-range

Return: vec![note, harmony] or vec![note]
```

### Mode 8: Barry Harris Movement

```
Barry Harris scales have 8 notes with chord/passing tone parity:

Chord Tones (even degrees):  0, 2, 4, 6
Passing Tones (odd degrees): 1, 3, 5, 7

Movement by +2 preserves parity:
  Chord tone (0) + 2 = Chord tone (2)
  Passing tone (1) + 2 = Passing tone (3)

This maintains harmonic clarity while moving smoothly.
```

---

## Voice Leading & Voicing

### Voicing Architecture

```
┌──────────────────────────────────────────────────────────┐
│ VoiceLeadingProcessor                                    │
├──────────────────────────────────────────────────────────┤
│ Input: harmony pitch classes (0-11)                      │
│        previous voicing (optional)                       │
│        registers (soprano/alto/tenor/bass)               │
│        style rules                                       │
│        voice anchor (user's note position)               │
│                                                          │
│ Process:                                                │
│  1. generate_valid_placements() for each voice          │
│  2. cartesian_product() of all placements               │
│  3. check_rules() for each candidate voicing            │
│  4. score() each candidate holistically                 │
│  5. tiebreak() deterministically                        │
│  6. return best candidate                               │
└──────────────────────────────────────────────────────────┘
```

### Voice Registers (MIDI Ranges)

```
┌──────────┬────────┬─────────────────┐
│ Register │ MIDI   │ Octave Range    │
├──────────┼────────┼─────────────────┤
│ Soprano  │ 60-81  │ C4 to C6 (~1.7) │
│ Alto     │ 55-76  │ G3 to E5 (~1.75)│
│ Tenor    │ 48-69  │ C3 to A4 (~1.75)│
│ Bass     │ 40-64  │ E2 to E4 (~1.67)│
└──────────┴────────┴─────────────────┘

Register Center (for first chord preference):
  Soprano: 70.5 ≈ 71
  Alto:    65.5 ≈ 66
  Tenor:   58.5 ≈ 59
  Bass:    52
```

### Valid Placement Generation

```
Input: pitch class (0-11), register (e.g., Alto)

Algorithm:
┌─────────────────────────────────────┐
│ 1. Get register range: (55, 76)     │
│ 2. Find first note with pc in range │
│    note = 55 + ((76 + pc - 55) % 12)│
│ 3. Collect all octaves in range     │
│    [55, 67, 79, ...] until > 76     │
│ 4. Return [64, 76]                  │
└─────────────────────────────────────┘

Result: All MIDI notes where pitch class matches
        and falls within register bounds
```

### Cartesian Product Voicing Generation

```
Input:
  Voice 1 placements: [60, 72]      (E in Soprano)
  Voice 2 placements: [55, 67]      (C in Alto)
  Voice 3 placements: [48, 60, 72]  (G in Tenor)

Process:
  result = [[]]
  for voice_placements:
    for each note in voice_placements:
      for each existing combination:
        new_combination = existing + note
        add to result

Output:
  [
    [60, 55, 48],
    [60, 55, 60],
    [60, 55, 72],
    [60, 67, 48],
    [60, 67, 60],
    [60, 67, 72],
    [72, 55, 48],
    ... 18 total combinations
  ]
```

### Scoring Algorithm (Holistic)

```
BASE SCORE (starts at 0, higher is better):

1. HARD CONSTRAINTS (filter-out, don't score):
   ├─ parallel_fifths (if hard_reject)
   ├─ parallel_octaves (if hard_reject)
   ├─ max_leap_semitones violation
   └─ voice_crossing_anchor violation (user position)

2. SOFT PENALTIES (subtracted from score):
   ├─ spacing_violation: -score * 80-200
   ├─ voice_crossing: -score * 2-150
   ├─ parallel_fifths (soft): -count * 2-100
   ├─ parallel_octaves (soft): -count * 2-100
   ├─ all_parallel_motion: -score * 5-120
   ├─ leap_penalty_per_semitone: -semitones * 0-15
   └─ motion_independence: -all_down/up penalty

3. SOFT BONUSES (added to score):
   ├─ stepwise_motion (1-2 semitones): +1-60
   ├─ common_tone: +1-70
   ├─ contrary_motion: +3-40
   ├─ spread_preference: +(spread) * -4 to +5
   └─ register_center (first chord): -distance from center

4. SORT (deterministic tiebreaking):
   ├─ Score (descending)
   ├─ Common tones (descending)
   ├─ Stepwise-down count (descending)
   ├─ Stepwise-up count (descending)
   ├─ Total movement (ascending)
   └─ Lexicographic MIDI (ascending)
```

### Example Scoring Trace

```
Candidate 1: [64, 60, 55]
  Spacing check:   64-60=4, 60-55=5   → No violation
  Voice crossing:  64>60>55 ✓        → No crossing
  Parallel fifths: No                → Score: 0
  Common tones:    2 (matched)       → Bonus: +70*2 = +140
  Stepwise motion: 64→62, 60→58, 55→53
                   All steps          → Bonus: +60*3 = +180
  ─────────────────────────────────────
  Total Score: 320 (excellent)

Candidate 2: [55, 64, 72]
  Spacing check:   55-64=? (reversed) → Violation
  Voice crossing:  Alto (55) < Soprano (64)? → Yes, violation!
  ─────────────────────────────────────
  Rejected (hard constraint)
```

### Voice Leading Rules

#### Parallel Fifths Detection

```
Check all harmony voice pairs (skip melody at index 0):

for i in 1..n:
  for j in (i+1)..n:
    prev_interval = interval_class(prev[i], prev[j])
    curr_interval = interval_class(curr[i], curr[j])
    i_moved = prev[i] != curr[i]
    j_moved = prev[j] != curr[j]
    
    if prev_interval == 7  (P5)
       and curr_interval == 7
       and i_moved and j_moved:
         → PARALLEL FIFTHS!

interval_class(a, b) = (|a - b|) % 12
```

#### Voice Crossing Detection

```
Higher register should have higher MIDI note:

if soprano (index i) has order 0 and alto (index j) has order 1:
  if soprano_midi < alto_midi:
    → VOICE CROSSING!
```

#### Spacing Rule

```
Maximum gap between adjacent harmony voices: 12 semitones (octave)
Exception: last pair (bass-tenor) can exceed octave

for i in 1..(n-1):
  if i != (n-2):  // Skip last pair
    if |voice[i] - voice[i+1]| > 12:
      → SPACING VIOLATION!
```

### Style Rules Presets

```
┌────────────┬──────────┬──────────┬──────────┬──────────┐
│ Parameter  │Palestrina│BachChoral│  Jazz    │   Free   │
├────────────┼──────────┼──────────┼──────────┼──────────┤
│Hardreject  │ YES      │ YES      │ NO       │ NO       │
│ fifths     │          │          │          │          │
├────────────┼──────────┼──────────┼──────────┼──────────┤
│Hardreject  │ YES      │ YES      │ NO       │ NO       │
│ octaves    │          │          │          │          │
├────────────┼──────────┼──────────┼──────────┼──────────┤
│Max leap    │  5 semi  │ 12 semi  │ 127 semi │ 127 semi │
│            │(perfect4)│(octave)  │ (none)   │ (none)   │
├────────────┼──────────┼──────────┼──────────┼──────────┤
│Stepwise    │   +60    │   +25    │   +3     │   +1     │
│bonus       │          │          │          │          │
├────────────┼──────────┼──────────┼──────────┼──────────┤
│Common tone │   +45    │   +70    │   +2     │   +1     │
│bonus       │          │          │          │          │
├────────────┼──────────┼──────────┼──────────┼──────────┤
│Leap        │   -15/st │   -4/st  │   0/st   │   0/st   │
│penalty     │          │          │          │          │
├────────────┼──────────┼──────────┼──────────┼──────────┤
│Voice cross │  -150    │   -80    │   -10    │   -2     │
│penalty     │          │          │          │          │
├────────────┼──────────┼──────────┼──────────┼──────────┤
│Spread pref │   -4     │   -1     │   +5     │    0     │
│(per st)    │(tight)   │(tight)   │(wide)    │(neutral) │
├────────────┼──────────┼──────────┼──────────┼──────────┤
│Contrary    │   +40    │   +20    │   +3     │    0     │
│motion      │          │          │          │          │
└────────────┴──────────┴──────────┴──────────┴──────────┘
```

---

## Stateful Processing

### Mode 6: Contrary Motion State Machine

```
┌────────────────────────────────────────────────────┐
│ ContraryMotionState                                │
├────────────────────────────────────────────────────┤
│ prev_melody: Option<Note>                          │
│ prev_harmony: Option<Note>                         │
│ direction: MelodicDirection (Up/Down/None)         │
├────────────────────────────────────────────────────┤
│ process(scale, melody) → Vec<Note>                │
└────────────────────────────────────────────────────┘

MelodicDirection enum:
  Up    → melody moved up since last call
  Down  → melody moved down
  None  → first call or stationary

State Flow:
                   ┌─────────────────┐
                   │ First note call │
                   └────────┬────────┘
                            │
                            ▼
              ┌─────────────────────────┐
              │ Store harmony, set dir  │
              │ Return harmony          │
              └────────┬────────────────┘
                       │
                       ▼
       ┌───────────────────────────────┐
       │ Next note arrives             │
       │ melody_delta = new - prev     │
       └───────┬───────────────────────┘
               │
       ┌───────┴────────────┐
       │                    │
       ▼                    ▼
   Melody up         Melody down
       │                    │
       ▼                    ▼
  Move harmony       Move harmony
     DOWN              UP
   (-interval)      (+interval)
       │                    │
       └────────┬───────────┘
                │
                ▼
         Return harmony
         Update state
```

#### Contrary Motion Algorithm

```
Input: melody (new note), scale reference, above parameter
Current: prev_melody, prev_harmony, direction

Logic:
  if first call:
    harmony = harmonize_smart(melody, +2, above)
    store(harmony, melody)
    return [melody, harmony]

  else:
    prev_dir = determine_direction(prev_melody → melody)
    
    if prev_dir == Up:
      # Melody moved up, make harmony go down
      interval = -2 (if above) else +2
    else if prev_dir == Down:
      # Melody moved down, make harmony go up
      interval = +2 (if above) else -2
    else:
      # Stationary, use preference
      interval = +2 (if above) else -2
    
    harmony = harmonize_smart(melody, interval, above)
    store(harmony, melody)
    return [melody, harmony]
```

### Mode 7: Strict Counterpoint State Machine

```
┌────────────────────────────────────────────────────┐
│ CounterpointState                                  │
├────────────────────────────────────────────────────┤
│ history: VecDeque<CounterpointHistoryEntry> (size:3)
│ current_direction: Option<MelodicDirection>        │
│ contour_shape: Vec<i8> (last 3 intervals)          │
├────────────────────────────────────────────────────┤
│ process(scale, melody) → Vec<Note>                │
└────────────────────────────────────────────────────┘

CounterpointHistoryEntry:
  melody: Note
  harmony: Note
  interval: i8 (semitones between them)
  motion: MelodicDirection (last movement)

History Window (sliding 3-note window):
  [n-2, n-1, n(current)]
```

#### Counterpoint Algorithm

```
Input: melody, scale, direction preference

Process:
  1. Determine melody motion direction
  2. Check if any recent interval is a leap (>2 semitones)
  3. If leap, next harmony should step or leap opposite direction
  4. If stepwise, allow stepwise in harmony
  5. Check interval variety (avoid same interval 3x in a row)
  6. Check contour (avoid all ascending or descending)
  7. Score candidates on:
     - Interval variety
     - Stepwise recovery after leap
     - Opposite direction preference
     - Voice leading smoothness
```

---

## Configuration & Enums

### HarmonyMode Enum

```
┌──────────────────────────────────────────────────────┐
│ HarmonyMode                                          │
├──────────────────────────────────────────────────────┤
│ PassThrough            (1) - No harmony             │
│ DiatonicThirds         (2) - Parallel Thirds        │
│ DiatonicFourths        (3) - Parallel Fourths       │
│ ContraryMotion         (6) - Opposite direction     │
│ StrictCounterpoint     (7) - Counterpoint (Sp. 1)   │
├──────────────────────────────────────────────────────┤
│ fn number() → stable u8 identifier                  │
│ fn all() → &[HarmonyMode]                           │
│ fn description() → &str                             │
│ fn tooltip() → &str                                 │
└──────────────────────────────────────────────────────┘
```

### ScaleMode (57 Variants, 10 Families)

```
DIATONIC MODES (7):
  Ionian, Dorian, Phrygian, Lydian, Mixolydian, Aeolian, Locrian

HARMONIC MINOR MODES (7):
  HarmonicMinor, LocrianNat6, IonianAug, DorianSharp4,
  PhrygianDominant, LydianSharp2, SuperLocrianDim

MELODIC MINOR MODES (7):
  MelodicMinor, DorianFlat2, LydianAug, LydianDominant,
  MixolydianFlat6, LocrianNat2, SuperLocrian

HARMONIC MAJOR MODES (7):
  HarmonicMajor, DorianFlat5, PhrygianFlat4, LydianFlat3,
  MixolydianFlat2, LydianAugSharp2, LocrianDoubleFlat7

DOUBLE HARMONIC MODES (7):
  DoubleHarmonic, LydianSharp2Sharp6, Ultraphrygian,
  HungarianMinor, Oriental, IonianSharp2Sharp5,
  LocrianDoubleFlat3DoubleFlat7

PENTATONIC SCALES (8, 5 notes each):
  MajorPentatonic, MinorPentatonic, Hirajoshi, InSen,
  Iwato, Yo, Kumoi, Pelog

BLUES & BEBOP (3, 6-8 notes):
  MinorBlues, MajorBlues, BebopDominant

SYMMETRIC SCALES (4, 6-8 notes):
  WholeTone, DiminishedWholeHalf, DiminishedHalfWhole, AugmentedHex

WORLD SCALES (5):
  Enigmatic, NeapolitanMinor, NeapolitanMajor, Persian, HungarianMajor

BARRY HARRIS 8-NOTE SCALES (2):
  BHMajor6thDim, BHMinor6thDim
```

### Key Enum

```
C, Db, D, Eb, E, F, Gb, G, Ab, A, Bb, B
(12 chromatic notes)

fn semitones_from_c() → u8:
  C=0, Db=1, D=2, Eb=3, E=4, F=5, Gb=6,
  G=7, Ab=8, A=9, Bb=10, B=11
```

### OctaveMode

```
┌─────────────────┬──────────────────────────────────┐
│ Mode            │ Effect                           │
├─────────────────┼──────────────────────────────────┤
│ None            │ Harmony stays at generated pitch │
├─────────────────┼──────────────────────────────────┤
│ Spread          │ Each voice +1 octave higher     │
│                 │ Voice 1: +0 octaves             │
│                 │ Voice 2: +1 octave              │
│                 │ Voice 3: +2 octaves             │
├─────────────────┼──────────────────────────────────┤
│ BassTrebleSplit │ Harmonies below melody: -1 oct  │
│                 │ Harmonies above melody: +1 oct  │
├─────────────────┼──────────────────────────────────┤
│ Mirror          │ Each harmony: ±1 octave (triple)│
│                 │ E.g., E4 → [E3, E4, E5]         │
└─────────────────┴──────────────────────────────────┘
```

---

## Humanization Pipeline

### Humanizer Flow

```
┌──────────────────────────────────────────────────────────┐
│ Humanizer                                                │
├──────────────────────────────────────────────────────────┤
│ config: HumanizeConfig                                   │
│ clock: BeatClock (for swing calculations)                │
│ active_humanization: HashMap<u8, HumanizationRecord>     │
├──────────────────────────────────────────────────────────┤
│                                                          │
│ humanize_note_on(note, channel, vel, port) {            │
│   1. Apply velocity variation (+/- range)               │
│   2. Add jitter (random delay)                          │
│   3. Add swing (off-beat shift)                         │
│   4. Calculate duration delta                           │
│   5. Store record for matching Note-Off                 │
│   6. Return HumanizedNote with offsets                  │
│ }                                                       │
│                                                          │
│ humanize_note_off(note, channel, vel, port) {          │
│   1. Retrieve stored record                            │
│   2. Use same delay + duration extension                │
│   3. Remove from active map                            │
│   4. Return HumanizedNote                              │
│ }                                                       │
└──────────────────────────────────────────────────────────┘
```

### HumanizeConfig Parameters

```
┌─────────────────────────┬──────────┬──────────────────┐
│ Parameter               │ Typical  │ Effect           │
├─────────────────────────┼──────────┼──────────────────┤
│ jitter_min_ms           │ 1-5      │ Minimum delay    │
│ jitter_max_ms           │ 10-30    │ Maximum delay    │
│                         │ subtle:1-│ (loose:10-30)    │
│                         │ 10, loose│                  │
├─────────────────────────┼──────────┼──────────────────┤
│ velocity_variation      │ 5-25     │ +/- velocity    │
│                         │ subtle:  │ (subtle:5-10,    │
│                         │ 5-10,exp │ expressive:15-25)│
│                         │ ressive: │                  │
│                         │ 15-25    │                  │
├─────────────────────────┼──────────┼──────────────────┤
│ duration_variation_ms   │ 10-50    │ Note ring extend │
│                         │          │ (legato feel)    │
├─────────────────────────┼──────────┼──────────────────┤
│ swing_amount            │ 0.0-0.5  │ Shuffle/swing    │
│                         │ 0.0=none │ 0.2=light,0.4+=  │
│                         │ 0.2=light│ jazz             │
│                         │ 0.4+=jazz│                  │
├─────────────────────────┼──────────┼──────────────────┤
│ bpm                     │ 60-180   │ Tempo            │
│                         │ (common: │ (affects swing)  │
│                         │ 120)     │                  │
├─────────────────────────┼──────────┼──────────────────┤
│ beats_per_bar           │ 3, 4, 6  │ Time signature   │
│                         │ (common: │ numerator        │
│                         │ 4)       │                  │
├─────────────────────────┼──────────┼──────────────────┤
│ beat_unit               │ 4, 8     │ Time signature   │
│                         │ (common: │ denominator      │
│                         │ 4)       │                  │
└─────────────────────────┴──────────┴──────────────────┘
```

### BeatClock (Swing Calculation)

```
┌────────────────────────────────────────────────────────┐
│ BeatClock                                              │
├────────────────────────────────────────────────────────┤
│ bpm: f64                                               │
│ beats_per_bar: u8                                      │
│ beat_unit: u8                                          │
│ running: bool                                          │
│ beat_position: f64 (0.0 to beats_per_bar)             │
│ prev_beat_position: f64 (for detecting boundaries)    │
├────────────────────────────────────────────────────────┤
│ start(now_ms) - Start clock at timestamp              │
│ stop() - Pause clock                                  │
│ tick(now_ms) → beat_position - Advance clock          │
│ is_offbeat() → bool - Returns true if 0.4 ≤ frac ≤0.6│
│ beat_crossed() → Option<u8> - Beat boundary crossed  │
│ update_tempo(bpm, beats, unit) - Sync new tempo      │
└────────────────────────────────────────────────────────┘

Beat Calculation:
  elapsed_seconds = (now_ms - start_time_ms) / 1000.0
  total_beats = elapsed_seconds * bpm / 60.0
  beat_position = total_beats % beats_per_bar

Example (4/4 time, 120 BPM):
  start_time = 1000 ms
  now = 1500 ms (elapsed: 500 ms = 0.5 sec)
  total_beats = 0.5 * 120 / 60 = 1.0
  beat_position = 1.0 % 4 = 1.0 (exactly on beat 2)
```

### Swing Detection

```
Off-beat detection (is_offbeat):
  frac = beat_position.fract()
  Returns true if 0.4 ≤ frac ≤ 0.6
  
  Visual timeline (4/4):
  0.0━━━1.0━━━2.0━━━3.0━━━4.0
       ↑ offbeat region (0.4-0.6)
       centered at 0.5 (halfway)

Swing delay calculation:
  if is_offbeat():
    delay_ms = (60_000 / bpm / 2) * swing_amount
    
  Example (120 BPM, swing=0.3):
    delay = (60_000 / 120 / 2) * 0.3
    delay = 250 * 0.3 = 75 ms
```

### Note Humanization Record Tracking

```
HashMap<u8, HumanizationRecord>

Note-On (C4, velocity=100):
  ├─ Generate humanization:
  │  ├─ Velocity: 100 - 8 (random) = 92
  │  ├─ Jitter: 7 ms
  │  ├─ Swing: 4 ms
  │  └─ Duration: 12 ms
  │
  └─ Store in map:
     humanization[60] = HumanizationRecord {
       delay_ms: 11,              // jitter + swing
       velocity: 92,              // stored but not used for off
       duration_delta_ms: 12
     }

Note-Off (C4):
  ├─ Retrieve from map:
  │  record = humanization.remove(60)
  │
  ├─ Compute Note-Off delay:
  │  delay = 11 + 12 = 23 ms
  │
  └─ Send Note-Off 23 ms later
```

### Full Humanization Flow Example

```
INPUT: Note-On C4 (MIDI 60), vel=100, on-beat

Humanizer.humanize_note_on():
  
  1. Velocity Variation (if enabled, range=15):
     delta = random(-15, +15) = +8
     new_vel = 100 + 8 = 108
  
  2. Jitter (if enabled, 5-10 ms):
     jitter = random(5, 10) = 7 ms
  
  3. Swing (if enabled, amount=0.3):
     is_offbeat() = false (on-beat)
     swing = 0 ms
  
  4. Duration (if enabled, max=20 ms):
     duration = random(0, 20) = 12 ms
  
  5. Total Delay:
     delay = jitter + swing = 7 + 0 = 7 ms
  
  6. Store Record:
     active[60] = {delay_ms: 7, duration: 12, vel: 108}

OUTPUT: HumanizedNote {
  note: C4,
  velocity: 108,
  delay_ms: 7,
  duration_delta_ms: 12,
  is_note_off: false
}

LATER: Note-Off C4

Humanizer.humanize_note_off():
  
  1. Retrieve record:
     record = active.remove(60)
       {delay_ms: 7, duration: 12, ...}
  
  2. Compute Note-Off delay:
     delay = 7 + 12 = 19 ms
  
  3. Return with same characteristics

OUTPUT: HumanizedNote {
  note: C4,
  velocity: original,
  delay_ms: 19,
  duration_delta_ms: 12,
  is_note_off: true
}
```

---

## Voice Position Anchor (Arrangement Mapping)

### VoiceAnchor Structure

```
┌────────────────────────────────────────────┐
│ VoiceAnchor                                │
├────────────────────────────────────────────┤
│ midi: u8                         (user's note)
│ arrangement_pos: usize           (0=S, 1=A, 2=T, 3=B)
│ harmony_arrangement_positions    (where each harmony
│   : Vec<usize>                    voice should go)
├────────────────────────────────────────────┤
│                                            │
│ Example:                                  │
│ User plays: F4 (MIDI 65) as Tenor (pos=2)│
│                                            │
│ midi: 65                                  │
│ arrangement_pos: 2 (Tenor)                │
│ harmony_arrangement_positions: [0, 1, 3]  │
│   Voice 1 (index 1): Soprano (0)          │
│   Voice 2 (index 2): Alto (1)             │
│   Voice 3 (index 3): Bass (3)             │
└────────────────────────────────────────────┘

CONSTRAINT:
  Voices with arrangement_pos < 2 (Tenor):
    Must be >= user_midi (above Tenor)
  
  Voices with arrangement_pos > 2 (Tenor):
    Must be <= user_midi (below Tenor)

NEVER VIOLATED: User's voice always respected
```

### Anchor-Constrained Voicing Selection

```
Without Anchor:
  Soprano:  [60, 72]
  Alto:     [55, 67]
  Tenor:    [48, 60, 72]
  
  Cartesian: 2 × 2 × 3 = 12 candidates
  All valid in principle

With Anchor (User = Tenor F4/65):
  Soprano (must be >= 65):
    Original [60, 72] → filtered to [72]
  
  Alto (must be >= 65):
    Original [55, 67] → filtered to [67]
  
  Tenor (not constrained, but matches user):
    Original [48, 60, 72] → keep all
  
  Cartesian: 1 × 1 × 3 = 3 candidates
  All respect user position
```

---

## Summary: Data Flow Through Complete System

```
┌─────────────────────────────────────────────────────────────────┐
│ COMPLETE PIPELINE                                               │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│ 1. MIDI INPUT: Note-On C4, velocity=100                        │
│                │                                                │
│                ▼                                                │
│ 2. HARMONY ENGINE: harmonize()                                 │
│    ├─ Check scale membership                                   │
│    ├─ Select harmony mode                                      │
│    ├─ Stateless or stateful processing                         │
│    └─ Return: [C4, E4] (melody + harmony)                      │
│                │                                                │
│                ▼                                                │
│ 3. VOICE LEADING PROCESSOR: revoice_chord()                    │
│    ├─ Extract pitch classes [0, 4]                            │
│    ├─ Generate valid placements per register                  │
│    ├─ Cartesian product of placements                         │
│    ├─ Check all voice leading rules                           │
│    ├─ Score holistically with style weights                   │
│    ├─ Deterministic tiebreaking                               │
│    └─ Return: MIDI notes [72, 64] for registers               │
│                │                                                │
│                ▼                                                │
│ 4. HUMANIZATION ENGINE: humanize_note_on()                     │
│    ├─ Add velocity variation: 100 + 8 = 108                   │
│    ├─ Add jitter: +7 ms                                       │
│    ├─ Add swing: +0 ms (on-beat)                              │
│    ├─ Duration extension: +12 ms                              │
│    ├─ Store for Note-Off: [7, 108, 12]                        │
│    └─ Return: HumanizedNote [delay=7, vel=108, dur=12]        │
│                │                                                │
│                ▼                                                │
│ 5. DISPATCH: Send MIDI after delay                            │
│    ├─ Wait 7 ms                                               │
│    ├─ Send Note-On [C4@108, E4@108]                           │
│    └─ Store Note-Off for later: 7 + 12 = 19 ms               │
│                                                                 │
│ ... (note rings for its duration) ...                          │
│                                                                 │
│ 6. MIDI INPUT: Note-Off C4                                    │
│    │                                                            │
│    ▼                                                            │
│ 7. HUMANIZATION: humanize_note_off()                          │
│    ├─ Retrieve stored record [delay=7, dur=12]               │
│    ├─ Compute Note-Off delay: 7 + 12 = 19 ms                │
│    └─ Return: HumanizedNote [delay=19, is_off=true]          │
│                │                                                │
│                ▼                                                │
│ 8. DISPATCH: Send Note-Off after 19 ms                        │
│    └─ Send Note-Off [C4, E4]                                  │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

---

END OF DOCUMENT

