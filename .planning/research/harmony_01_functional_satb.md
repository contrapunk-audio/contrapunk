# Research 01 — Classical Functional Harmony + Bach SATB Voice-Leading

Agent: `research-functional-harmony`. For harmony mode audit → Phase B (BachChorale + FunctionalHarmony mode implementation).

## Key findings

### Functional categories (Rameau/Riemann/consensus)
- **Tonic (T)**: I, vi, iii (iii is ambiguous/weak)
- **Subdominant (S / PD)**: ii, IV, bVI, N6, Aug6
- **Dominant (D)**: V, vii°, V7, vii°7

### Primary motion graph with Bach corpus-approximate weights
```
T → T:  I→vi (~15%), I→iii (~3%)
T → S:  I→IV (~25%), I→ii (~18%)
T → D:  I→V (~30%, cadential)
S → D:  IV→V (~40%), ii→V (~50%)
S → T:  IV→I (~10% plagal)
D → T:  V→I (~80%), V→vi (~8% deceptive)
D → S:  **FORBIDDEN** (retrogression)
```

### Cadences (ranked by closure strength)
| Type | Formula | Use |
|---|---|---|
| **PAC** | V→I, both root position, 1 in soprano | End of piece/section |
| **IAC** | V→I but inverted or 3/5 in soprano | Mid-phrase |
| **Plagal** | IV→I | Coda/"Amen" |
| **Half** | ends on V (I-V, ii-V, IV-V) | Phrase pause |
| **Deceptive** | V→vi | Avoid closure, extend phrase |
| **Phrygian half** | iv6→V in minor | Characteristic of Bach minor chorales |

### Secondary dominants
- V/V, V/ii, V/vi, V/IV, V/iii — temporarily tonicize target
- **Rule**: leading tone of temporary key must resolve up by step to root of target chord
- Heuristic: given chord X → Y, optionally insert V/Y between them. Score boost when Y is structurally important

## Bach SATB rules (empirical, from ~371 chorales)

### Voice ranges (Luke Dahn 2018 statistical analysis)
| Voice | Absolute | Typical | MIDI (typical) |
|---|---|---|---|
| Soprano | B3–A5 | C4–G5 | 60–79 |
| Alto | F3–D5 | G3–C5 | 55–72 |
| Tenor | C3–A4 | C3–G4 | 48–67 |
| Bass | D2–E4 | F2–E4 | 41–64 |

**Note**: Contrapunk's existing `VoiceRegister::range()` values (60-81, 55-76, 48-69, 40-64) already match textbook/Bach practice. No change needed.

### Doubling rules
**Root position (5/3)** — ~85% of Bach chorales:
- HARD preference: double the root
- ~10% double the 5th
- ~5% double the 3rd (only in minor triads)
- **HARD forbidden**: never double the leading tone

**First inversion (6/3)**:
- HARD preference: double whichever chord tone is in soprano (Piston's "flexible doubling")
- In vii°6: double the third (which is in the soprano, scale-degree 2)

**Second inversion (6/4)** — treated as dissonance:
- HARD: double the bass (the 5th of the chord)
- Only allowed in: cadential 6/4, passing 6/4, pedal 6/4, arpeggiated 6/4

**V7**: no doubling needed; if omitting, omit the 5th

### Parallels
- Parallel P5 / P8 / unison: **HARD forbidden** (Bach essentially never violates)
- Unequal fifths (P5→d5 inward): **allowed** when d5 resolves inward to a third
- Hidden/direct fifths/octaves: HARD only between outer voices when soprano leaps; SOFT between inner voices (Piston's formulation)

### Leading tone resolution
- **HARD**: in an outer voice (soprano/bass) as part of V/vii°, must resolve up by half-step to tonic at cadence
- **HARD**: leading tone never doubled
- **HARD**: leading tone never approached/left by A2 or tritone melodically
- **SOFT**: in inner voices, may resolve down by 3rd to 5 ("frustrated resolution"). Bach does this ~20% in inner voices (Rohrmeier 2011)

### Seventh resolution
- **HARD** (~99% compliance): chordal 7th resolves down by step to the 3rd of the following chord
- V7 → I template in C major:
```
Voice     G7    C
Sop       G  →  G   (5 stays as 5) OR D→C
Alto      F  →  E   (7th resolves down)
Tenor     B  →  C   (leading tone up)
Bass      G  →  C   (5→1)
```
- Non-dominant 7ths (ii7, IV maj7, vi7) resolve down by step at ~95% compliance

### Voice spacing
- **HARD**: no more than an octave between adjacent upper voices (sop-alto, alto-tenor)
- **Tenor-bass may exceed an octave** (Bach often writes tenor at middle C + bass at low E)
- SOFT average: sop-alto ≈ 4-5 semitones, alto-tenor ≈ 5-7 semitones

### Voice crossing / overlap
- Crossing: upper voice drops below lower in same chord. Bach tolerates 2% of chord changes, mostly alto/tenor; outer crossings <0.3%
- Overlap: one voice moves to a pitch just vacated by an adjacent voice. Bach avoids but not absolutely
- SOFT constraint: -30 inner, -100 outer (HARD for outer)

### Melodic rules per voice
- **Soprano**: mostly stepwise; leaps up to a 6th; 7ths HARD forbidden; octave leaps allowed but must be followed by stepwise motion in opposite direction
- **Alto/Tenor**: predominantly stepwise; small leaps (3rd/4th); leap to chord tone only; tritone leaps HARD forbidden
- **Bass**: most freedom. Leaps of 5th, 6th, octave common. 7ths HARD forbidden. Tritone HARD forbidden
- **"Leap then step opposite"** (species counterpoint heritage): after leap >4th, next note should be step in opposite direction (strongest for soprano + inner voices; bass exempted)

### Forbidden melodic intervals (HARD, all voices)
- Augmented 2nd (except harmonic minor raised 7 → 6)
- Augmented 4th (tritone), except as part of dim chord resolution
- Diminished 5th, except as vii° inward resolution
- Intervals > octave
- 7ths (minor or major)

## Rust data structures proposed

```rust
// src/harmony/functional/chord.rs
pub type PitchClass = u8;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Quality {
    Major, Minor, Diminished, Augmented,
    DominantSeventh, MajorSeventh, MinorSeventh,
    HalfDiminished, FullyDiminished,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Inversion { Root, First, Second, Third }

pub struct Chord {
    pub root: PitchClass,
    pub quality: Quality,
    pub inversion: Inversion,
    pub pitch_classes: Vec<PitchClass>,
}

// src/harmony/functional/function.rs
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Function { Tonic, Subdominant, Dominant }

pub struct ScaleDegree(pub u8);

impl ScaleDegree {
    pub fn function_in_major(self) -> (Function, FunctionStrength) {
        match self.0 {
            1 => (Function::Tonic, Primary),
            2 => (Function::Subdominant, Substitute),
            3 => (Function::Tonic, Ambiguous),
            4 => (Function::Subdominant, Primary),
            5 => (Function::Dominant, Primary),
            6 => (Function::Tonic, Substitute),
            7 => (Function::Dominant, Substitute),
            _ => unreachable!(),
        }
    }
}

// src/harmony/functional/progression.rs
pub const FORBIDDEN: TransitionWeight = i32::MIN;

pub struct ProgressionState {
    pub key: PitchClass,
    pub is_major: bool,
    pub current: Option<Chord>,
    pub previous: Option<Chord>,
    pub expects_cadence: bool,
    pub pending_seventh_resolution: Option<PitchClass>,
    pub pending_leading_tone_resolution: Option<PitchClass>,
}

// Transition weight function (corpus-approximate)
fn functional_motion_weight(from: Option<Function>, to: Function) -> i32 {
    match (from, to) {
        (None, _) => 0,
        (Some(T), S) => 50,
        (Some(T), D) => 40,
        (Some(T), T) => 20,
        (Some(S), D) => 80,
        (Some(S), T) => 25, // plagal
        (Some(S), S) => 10,
        (Some(D), T) => 100,
        (Some(D), D) => 15,
        (Some(D), S) => i32::MIN, // forbidden retrogression
    }
}
```

### Hard vs soft constraint map

| Rule | Where |
|---|---|
| Parallel P5/P8 | HARD reject in voicer |
| Hidden 5/8 (outer, soprano leap) | SOFT -80 |
| Leading tone doubled | HARD |
| LT outer-voice resolution at cadence | HARD |
| LT inner-voice frustrated resolution | SOFT -20 |
| Chordal 7th resolves down by step | HARD |
| Voice range exceeded | HARD |
| Spacing > octave (upper pairs) | HARD |
| Voice crossing (inner) | SOFT -30 |
| Voice crossing (outer) | HARD |
| Forbidden melodic interval | HARD |
| Doubled 3rd of major triad | SOFT -40 |
| Double bass in 6/4 | HARD |
| Cadence bias (PAC at phrase end) | SOFT +150 |
| Retrogression D→S | HARD |
| T→S→D→T motion weights | SOFT (corpus) |
| Secondary dominant insertion | SOFT (+30 when target is cadence) |

## Architecture recommendation

**Two-layer split**:
1. **Chord selection layer** (new `ProgressionState`) — picks next chord pitch-class set from functional state machine
2. **Voicing layer** (existing `revoice_chord()`) — places chord in proper MIDI octaves

Communication: `Chord` (PCs) → `Vec<u8>` (MIDI notes). Matches how Aldwell & Schachter teach it: choose chord first, voice second.

**Fallback policy**: if `score_transition` returns FORBIDDEN for all candidates, relax in this order:
1. Retrogression rule
2. Doubling rules
3. Parallels
**Never relax**: voice ranges, chordal seventh resolution

**Bach corpus weight extraction**: one-time offline Python script using `music21.corpus.chorales.ChoraleList()` to dump actual transition frequencies into a Rust `const` table, replacing hand-tuned weights with measured values.

## Sources
- Piston, *Harmony*, 5th ed. (rev. DeVoto), 1987
- Aldwell & Schachter, *Harmony and Voice Leading*, 4th ed., 2011
- Kostka, Payne, Almén, *Tonal Harmony*, 8th ed., 2018
- Rameau, *Traité de l'harmonie*, 1722 — fundamental bass + cadences
- Schenker, *Harmonielehre*, 1906 — prolongation
- Tymoczko, *A Geometry of Music*, 2011
- Rohrmeier, "Towards a generative syntax of tonal harmony," J. of Mathematics and Music 5(1), 2011
- Rohrmeier & Cross, "Statistical properties of tonal harmony in Bach's chorales," ICMPC10, 2008
- Cuthbert & Ariza, "music21: a toolkit for computer-aided musicology," ISMIR 2010
- Luke Dahn, "The Complete Bach Chorales," bach-chorales.com, 2018 — voice range stats
- KernScores 371chorales: kernscores.stanford.edu/browse?l=/371chorales
