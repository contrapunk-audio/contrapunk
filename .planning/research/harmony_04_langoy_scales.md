# Research 04 — Langøy.net Scale Database Extraction

Agent: `research-langoy-scales`. From https://langøy.net/skala — confirmed Lycomedes1814 (HN) is site owner via comment 47663649 ("my scale builder"). Site tech: vanilla HTML + inline `SCALE_FAMILIES` JS. No license (all-rights-reserved by default — needs explicit permission to port).

## Total: 57 scales across 9 families

### 1. Diatonic (7) — already in Contrapunk
Ionian, Dorian, Phrygian, Lydian, Mixolydian, Aeolian, Locrian

### 2. Harmonic minor modes (7) — already in Contrapunk
Harmonic minor, Locrian ♮6, Ionian #5, Dorian #4, Phrygian dominant, Lydian #2, Super Locrian 𝄫7

### 3. Melodic minor modes (7) — already in Contrapunk
Melodic minor, Dorian ♭2, Lydian augmented, Lydian dominant, Mixolydian ♭6, Locrian ♮2, Super Locrian/Altered

### 4. Double harmonic modes (7) — 1 already in Contrapunk, **6 new**
| # | English | Semitones | Notes |
|---|---|---|---|
| 1 | Double harmonic (Byzantine/Arabic) | 0 1 4 5 7 8 11 | **ALREADY** (DoubleHarmonic) |
| 2 | **Lydian ♯2 ♯6** | 0 3 4 6 7 10 11 | NEW |
| 3 | **Ultraphrygian** | 0 1 3 4 7 8 9 | NEW — two aug2, very dark |
| 4 | Hungarian minor (Gypsy minor) | 0 2 3 6 7 8 11 | **ALREADY** (HungarianMinor) |
| 5 | **Oriental** | 0 1 4 5 6 9 10 | NEW |
| 6 | **Ionian ♯2 ♯5** | 0 3 4 5 8 9 11 | NEW |
| 7 | **Locrian 𝄫3 𝄫7** | 0 1 2 5 6 8 9 | NEW — extreme |

### 5. Harmonic major modes (7) — **ALL 7 NEW**
| # | English | Semitones |
|---|---|---|
| 1 | Harmonic major | 0 2 4 5 7 8 11 |
| 2 | Dorian ♭5 (Locrian ♮2♮6) | 0 2 3 5 6 9 10 |
| 3 | Phrygian ♭4 | 0 1 3 4 7 9 10 |
| 4 | Lydian ♭3 (Lydian diminished) | 0 2 3 6 7 9 11 |
| 5 | Mixolydian ♭2 | 0 1 4 5 7 9 10 |
| 6 | Lydian augmented ♯2 | 0 3 4 6 8 9 11 |
| 7 | Locrian 𝄫7 | 0 1 3 5 6 8 9 |

### 6. Pentatonic (8) — **ALL NEW**
| # | Name | Semitones |
|---|---|---|
| 1 | Major pentatonic | 0 2 4 7 9 |
| 2 | Minor pentatonic | 0 3 5 7 10 |
| 3 | Hirajoshi | 0 2 3 7 8 |
| 4 | In sen | 0 1 5 7 8 |
| 5 | Iwato | 0 1 5 6 10 |
| 6 | Yo | 0 2 5 7 9 |
| 7 | Kumoi | 0 2 3 7 9 |
| 8 | Pelog | 0 1 3 7 8 |

### 7. Blues & Bebop (5) — **ALL NEW**
| # | Name | Semitones |
|---|---|---|
| 1 | Minor blues (hex) | 0 3 5 6 7 10 |
| 2 | Major blues (hex) | 0 2 3 4 7 9 |
| 3 | Bebop dominant (8 notes) | 0 2 4 5 7 9 10 11 |
| 4 | Bebop major (8 notes) | 0 2 4 5 7 8 9 11 |
| 5 | Bebop minor (8 notes) | 0 2 3 5 7 8 9 11 |

**⚠️ Note**: Bebop major (0 2 4 5 7 8 9 11) has same pitches as BHMajor6thDim. Bebop minor has same pitches as BHMinor6thDim. Decide: add as distinct pedagogy labels or skip duplicates.

### 8. Symmetric (4) — **ALL NEW**
| # | Name | Semitones |
|---|---|---|
| 1 | Whole tone | 0 2 4 6 8 10 |
| 2 | Diminished (whole-half) | 0 2 3 5 6 8 9 11 |
| 3 | Diminished (half-whole) | 0 1 3 4 6 7 9 10 |
| 4 | Augmented scale (hex) | 0 3 4 7 8 11 |

### 9. Exotic (6) — **2 NEW, 4 already in Contrapunk**
| # | Name | Semitones | Status |
|---|---|---|---|
| 1 | **Persian** | 0 1 4 5 6 8 11 | NEW |
| 2 | Neapolitan minor | 0 1 3 5 7 8 11 | ALREADY |
| 3 | Neapolitan major | 0 1 3 5 7 9 11 | ALREADY |
| 4 | Romanian minor | 0 2 3 6 7 9 10 | **DUPLICATE** of DorianSharp4 — skip |
| 5 | Enigmatic | 0 1 4 6 8 9 11 | ALREADY (verify matches) |
| 6 | **Hungarian major** | 0 3 4 6 7 9 10 | NEW |

## Net new variants to add

**26 unique new scales** (after removing Romanian minor duplicate and the bebop-vs-BH pitch overlaps):

6 double harmonic modes + 7 harmonic major modes + 8 pentatonic + 3 blues (minor blues, major blues, bebop dominant — skipping bebop major/minor as BH duplicates) + 4 symmetric + 2 exotic (Persian, Hungarian major) = **30 if you include the bebop duplicates; 26 if you skip them.**

## Site metadata
- Domain: `langøy.net` / `xn--langy-yua.net`
- Footer: "© 2026 Langøya" (Norwegian island name, not person)
- Site owner = HN `Lycomedes1814` (self-identified in comment 47663649)
- GitHub: https://github.com/Lycomedes1814 (Emacs/C/Zig hacker, not music-specific)
- No license, no about page, no contact email
- **Permission path**: reply to HN comment. He volunteered the link so likely yes, but paper trail matters.

## Ready-to-paste Rust additions

See full code snippet in original research. Variants to add in `pub enum ScaleMode`:
```
LydianSharp2Sharp6, Ultraphrygian, Oriental, IonianSharp2Sharp5, LocrianDoubleFlat3DoubleFlat7,
HarmonicMajor, DorianFlat5, PhrygianFlat4, LydianFlat3, MixolydianFlat2, LydianAugSharp2, LocrianDoubleFlat7,
MajorPentatonic, MinorPentatonic, Hirajoshi, InSen, Iwato, Yo, Kumoi, Pelog,
MinorBlues, MajorBlues, BebopDominant,
WholeTone, DiminishedWholeHalf, DiminishedHalfWhole, AugmentedHex,
Persian, HungarianMajor,
```

Plus optional BebopMajor/BebopMinor if you want them as distinct labels from BH 6th-dim variants.
