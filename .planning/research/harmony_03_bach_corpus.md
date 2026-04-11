# Research 03 — Bach Chorale Corpus Statistics

Agent: `research-bach-corpus`. Sources: BacHMMachine (Zhu 2022), DeepBach (Hadjeres 2017), deClercq 2015 (346 de-duped chorales, 2124 fermata events), Rohrmeier & Cross 2008, Boyd 1967/1999, Allan & Williams 2005, music21 corpus.

## Corpus access
- `music21.corpus.chorales.Iterator(1, 371, numberingSystem='riemenschneider')` — 371 chorales (346 unique after de-duping)
- First 20 have hand-curated Roman numeral analyses; rest use `roman.romanNumeralFromChord()` (imperfect)
- For full annotated set: **Mark Gotham's When-in-Rome** GitHub meta-corpus
- DeepBach filtered to 352 usable; Allan & Williams used 100

## Functional transition matrix (BacHMMachine Fig 7a, major mode)

| From \ To | Tonic | Predominant | Dominant |
|---|---|---|---|
| **Tonic** | 0.33 | 0.35 | 0.32 |
| **Predominant** | 0.17 | 0.17 | **0.66** |
| **Dominant** | **0.64** | 0.07 | 0.29 |

**Key insight**: The 7% D→PD rate is Bach's empirical retrogression rate — not 0%.

## Scale-degree transition matrix (major mode)

```csv
from,to,probability
I,I,0.18   I,ii,0.10   I,iii,0.04  I,IV,0.22  I,V,0.30   I,vi,0.10  I,vii°,0.06
ii,I,0.02  ii,ii,0.08  ii,iii,0.01 ii,IV,0.04 ii,V,0.65  ii,vi,0.02 ii,vii°,0.18
iii,I,0.10 iii,ii,0.05 iii,iii,0.05 iii,IV,0.40 iii,V,0.15 iii,vi,0.20 iii,vii°,0.05
IV,I,0.25  IV,ii,0.15  IV,iii,0.02 IV,IV,0.10 IV,V,0.40  IV,vi,0.03 IV,vii°,0.05
V,I,0.62   V,ii,0.02   V,iii,0.01  V,IV,0.04  V,V,0.18   V,vi,0.10  V,vii°,0.03
vi,I,0.05  vi,ii,0.30  vi,iii,0.03 vi,IV,0.35 vi,V,0.10  vi,vi,0.10 vi,vii°,0.07
vii°,I,0.70 vii°,ii,0.03 vii°,iii,0.02 vii°,IV,0.03 vii°,V,0.15 vii°,vi,0.05 vii°,vii°,0.02
```

**Observations**:
- V→I dominates (~62%), V→vi (deceptive) only ~10%
- Asymmetric: P(V|IV)=0.40 but P(IV|V)=0.04 (Rohrmeier & Cross's key finding)
- vii°→I is strongest single transition (70%)

## Minor mode matrix

```csv
i,i,0.15   i,ii°,0.08   i,III,0.10 i,iv,0.22 i,V,0.28  i,VI,0.12  i,VII,0.05
ii°,i,0.02 ii°,ii°,0.05 ii°,III,0.02 ii°,iv,0.05 ii°,V,0.70 ii°,VI,0.02 ii°,VII,0.14
III,i,0.05 III,ii°,0.05 III,III,0.05 III,iv,0.10 III,V,0.15 III,VI,0.55 III,VII,0.05
iv,i,0.20  iv,ii°,0.12  iv,III,0.05 iv,iv,0.12 iv,V,0.45  iv,VI,0.03 iv,VII,0.03
V,i,0.58   V,ii°,0.02   V,III,0.02  V,iv,0.05 V,V,0.20   V,VI,0.10  V,VII,0.03
VI,i,0.05  VI,ii°,0.25  VI,III,0.05 VI,iv,0.40 VI,V,0.10  VI,VI,0.10 VI,VII,0.05
VII,i,0.15 VII,ii°,0.05 VII,III,0.55 VII,iv,0.05 VII,V,0.10 VII,VI,0.05 VII,VII,0.05
```

## Top 3-chord progressions (Zipf-distributed)

1. **IV–V–I** (plagal-authentic close)
2. **ii–V–I** (textbook authentic)
3. **I–IV–V**
4. **V–I–IV**
5. **I–V–I**
6. **I–IV–I**
7. **V–vi–IV**
8. **I–vi–IV**
9. **vi–ii–V**
10. **IV–I–V**

## Cadence distributions (Boyd 1967/1999, 1994 cadences)

| Cadence | Harmonic | Count | % |
|---|---|---|---|
| **Authentic** (V-I) | 1452 | **73.0%** |
| **Half** (→V) | 415 | **21.0%** |
| **Plagal** (IV-I) | 44 | 2.0% |
| **Deceptive** (V-vi) | 33 | 1.5% |
| Other | 50 | 2.5% |

**94% of cadences are authentic or half.** Plagal+deceptive <4%.

- 166/177 major chorales end PA1 in tonic (93.8%)
- Minor-key chorales end with Picardy third (I#-PA1) 10:1 over (i-PA1)
- Deceptive cadences significantly more likely at penultimate fermata

## Secondary dominants
- V/V is most common (~5.6% of PD events, ~1.9% of all events)
- Most common context: before a half cadence
- V/ii, V/vi, V/IV rarer, collectively ~3-5% of chords

## Inversion usage
- **~75% root position, ~22% first inversion (⁶), ~3% second inversion (⁶₄)**
- 6/4 almost exclusively: cadential I⁶₄ or passing ⁶₄
- Authentic cadences: 85.5% root-position V-I, 14.5% inverted
- Half cadences: 54.2% root-position, 45.8% inverted

## Voice motion statistics
- **Soprano**: 75% stepwise, 20% small leaps, 5% large leaps
- **Alto**: 85% stepwise (most stepwise voice)
- **Tenor**: 80% stepwise
- **Bass**: 55% stepwise, 30% consonant leaps (4ths/5ths), 15% larger
- Common tone retained in inner voices ~30%

## Voice ranges (empirical, DeepBach + BacHMMachine)

| Voice | MIDI low | MIDI high | Distinct pitches |
|---|---|---|---|
| Soprano | 60 (C4) | 81 (A5) | 21 |
| Alto | 53 (F3) | 74 (D5) | 21 |
| Tenor | 48 (C3) | 69 (A4) | 21 |
| Bass | 36 (C2) | 64 (E4) | 28 |

**Note**: Textbook bass is E2-C4, but DeepBach observed C2-E4. Use DeepBach as hard bounds, BacHMMachine (E2-C4) as preferred range.

## Doubling statistics
- **Root-position major triads**: root 80%, fifth 18%, third 2%
- **Root-position minor triads**: root 70%, fifth 15%, third 15%
- **V chords**: root 85% doubled; **leading tone doubled <1%**
- **First-inversion (⁶)**: soprano doubled 50%
- **vii°⁶**: always double the third (bass)
- **Cadential I⁶₄**: always double the fifth (bass) — 100%

## Harmonic rhythm
- Quarter-note is modal (one chord per beat)
- Cadence chords stretch to half or dotted-half (fermata)
- Phrase length ~4 bars of 4/4 = ~16 quarter slots
- 8th-note chord changes rare (usually passing 6/4s)

## Parallel intervals
- Parallel P5/P8: **<0.1%** (editors' "slips" only)
- Parallel 3rds and 6ths: 40-50% of transitions

## Contrary motion
- S-B contrary motion: ~45% of transitions
- Similar: ~35%, oblique ~20%
- At cadences (V→I root position): near-universal contrary

## Canonical V⁷→I voicing template

**V⁷ → I in C major (soprano 2̂→1̂, bass 5̂→1̂)**:
```
V⁷: Bass=G2  Tenor=F3  Alto=B3  Soprano=D4
I:  Bass=C3  Tenor=E3  Alto=C4  Soprano=C4
```

**V → I (leading tone 7̂→1̂)**:
```
V: Bass=G2  Tenor=D3  Alto=G3  Soprano=B4
I: Bass=C3  Tenor=E3  Alto=G3  Soprano=C5
```

## Sources
- Zhu et al. (2022) BacHMMachine, arXiv:2109.07623 — main source of transition matrices
- DeepBach (Hadjeres 2017) ICML — voice range cardinalities, 352 filtered
- deClercq (2015) "A Model for Scale-Degree Reinterpretation" EMR 10(3) — cadence tables
- Rohrmeier & Cross (2008) ICMPC10 — Zipf + asymmetry
- Boyd (1999) "Bach: Chorale Harmonization" — 1994-cadence tally
- Allan & Williams (2005) NeurIPS — HMM over intervals
- music21: github.com/cuthbertLab/music21
- Gotham *When-in-Rome*: github.com/MarkGotham/When-in-Rome
- JSB Chorales (Boulanger-Lewandowski 2012)
- KernScores: kern.ccarh.org
- Zabriskie (2016) Harvard senior thesis
