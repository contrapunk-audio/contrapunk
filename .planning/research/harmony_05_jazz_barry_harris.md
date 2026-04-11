# Research 05 — Jazz/Modern Functional Harmony + Real Barry Harris Theory

Agent: `research-jazz-harmony`. Critical finding: **Contrapunk's current `BarryHarris` mode captures ~5% of the real method**.

## Where jazz diverges from Rameau

### Extended chords are the default
- 7ths mandatory on every jazz chord. Cmaj = Cmaj7 or C6.
- 7ths are **structural**, NOT resolving dissonances
- Parallel 7ths/9ths are fine (Bill Evans / Herbie Hancock)
- Build chords as **stacks over root+7th shells**, not triads

### Chord-scale theory (Berklee/Levine)
- Every chord maps to one or more parent scales
- **Avoid notes**: scale tones a half-step ABOVE a chord tone
- Cmaj7 in C Ionian has F as avoid note → use **C Lydian** (no avoid)
- Rule: play anything from the mode **except the half-step-above problem**

### Tritone substitution
- V7 ↔ bII7 share tritone (3+b7)
- **Dm7 – G7 – Cmaj7** → **Dm7 – Db7 – Cmaj7** (chromatic bass descent)
- Bebop weaponizes: **iii – biii7 – ii – bii7 – I**

### Cascaded secondary dominants
- Rhythm changes bridge in Bb: **D7 – G7 – C7 – F7 – Bb** (4 dominants, none diatonic)
- Combined with tritone subs: **D7–Db7–C7–B7–Bb**

### Modal interchange (borrowed chords)
- Common borrowings in C: **Fm (iv), Bb (bVII), Eb (bIII), Ab (bVI), Abmaj7**
- Beatles "Let It Be" uses Fm; Radiohead "Creep" uses Cm in G
- Pop-style modal interchange is **color substitution, not tonicization**
- Everett (MTO 10.4) Beatles corpus analysis

### Line clichés
- Chromatic inner voice: **Cm – Cm(maj7) – Cm7 – Cm6** (C-B-Bb-A)
- Treat as voice-leading decoration on pedal, NOT progression

### Diminished usage
- **Passing**: C – C#dim7 – Dm7 – G7 (C#dim7 = V7/ii)
- **Upper-neighbor**: Cmaj7 – Dbdim7 – Cmaj7
- **Substitute**: Bdim7 substitutes G7b9 (share B-D-F)

### Upper-structure triads
- G7alt voiced as D triad over G7 (= G9 13)
- Ab triad over G7 (= b9, #11, b13)
- Db triad over G7 (= b9, #11, b5 — tritone sub color)

### Rootless/shell voicings
- Drop root+5 because bassist plays root
- A-form Dm7: F-A-C-E (b3, 5, b7, 9)
- **Relevant for Contrapunk**: when guitar feeds piano harmony + bassist plays roots, harmony should NOT double bass

## Barry Harris — what he ACTUALLY taught

**NOT a progression generator. It's an improvisation + reharmonization pedagogy.**

### The Major 6th Diminished Scale (8 notes)
C D E F G **Ab** A B — with b6 (Ab) as chromatic passing tone between 5 and 6.

| Degree | 0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 |
|---|---|---|---|---|---|---|---|---|
| Note | C | D | E | F | G | Ab | A | B |
| Chord | **C6** | Bdim7 | **C6/E** | Bdim7/F | **C6/G** | Bdim7/Ab | **C6/A = Am7** | Bdim7 |
| Function | tonic | passing | tonic | passing | tonic | passing | tonic | passing |

**Even degrees harmonize to C6 inversions. Odd degrees harmonize to Bdim7 inversions.**

The scale is literally "C6 and Bdim7 interlocked" — hence "6th diminished."

### Movement by 2 scale degrees (parity rule)
In 8-note BH scale, +2 degrees preserves parity: chord-tone → chord-tone, passing → passing. **Contrapunk's current code is correct IF paired with 8-note BH scale.** In a 7-note scale it's just diatonic thirds (no-op).

### Drop-2 voicings
Close-voiced 4-note chord → drop second-from-top by octave.
- Cmaj7 close: C-E-G-B
- Drop G: **G-C-E-B** (spread across 10 semitones)

### Line games
Alternate chord tones and passing tones in 8-note scale, always land chord tone on strong beat. The b6 passing tone exists to fix parity.

### Sister chords
- **C6 and Am7** = same notes (C-E-G-A)
- **Cm6 and Am7b5** = same notes
- **Bdim7, Ddim7, Fdim7, Abdim7** = same notes (all diminished 7ths at minor-3rd intervals)
- **Rotate chord by minor 3rd = sister**

## Correct Barry Harris mode spec for Contrapunk

Replace `barry_harris()` in `modes.rs:246-260`:

1. **Guard**: require scale = `BHMajor6thDim` or `BHMinor6thDim` (already exist). Auto-switch or warn if not.
2. **Beat phase input**: mode needs beat position — add `beat_phase: BeatPhase` to dispatcher
3. **Pick harmonization target**:
   - Even BH degree (chord tone) + strong beat → build **C6 block chord** around input
   - Odd BH degree (passing) OR weak beat → build **Bdim7 block chord** around input
4. **Build 4-voice drop-2 voicing**: take 4 chord tones, close-voice with input on top, drop second-from-top by octave, return **4 MIDI notes (not 2)**
5. **Voice-leading continuity**: next note prefers same chord family inversions sharing common tones

**Return shape must change from Vec<Note> length 2 to length 4.** Ripples through engine.

## Jazz functional harmony state needed

- `last_chord: Option<ChordSymbol>`
- `target_chord: Option<ChordSymbol>` — for secondary dominants / tritone subs
- `key_center: (PitchClass, Mode)`
- `beat_position: BeatPhase`
- `borrowing_budget: f32` — rate limiter
- `chord_scale_cache: HashMap<ChordSymbol, Scale>`
- `last_borrowed_mode: Option<ScaleMode>` (already exists)

Replace `HarmonyEngine`'s current `interchange_enabled: bool` with:
```rust
InterchangePolicy {
    enabled: bool,
    priority: Vec<ScaleMode>,  // parallel minor > Dorian > Mixolydian > Phrygian > Lydian > HM > MM
    max_per_bar: f32,          // ~0.15 for pop, max 1 per 4-8 bars
    hold_bars: u8,
}
```

## Chord-scale lookup table (implementation ready)

| Chord | Context | Primary scale | Alternates | Avoid |
|---|---|---|---|---|
| Cmaj7 | I in C | C Lydian | C Ionian, C Lyd Aug | F (if Ionian) |
| Cmaj7#11 | non-functional | C Lydian | — | none |
| Dm7 | ii in C | D Dorian | — | none |
| Dm7 | vi in F | D Aeolian | D Dorian | Bb vs B |
| Dm(maj7) | tonic minor | D Melodic Minor | D Harmonic Minor | none |
| G7 | V in C | G Mixolydian | — | C |
| G7 | V7/ii | G Mixolydian b13 | G HW dim | C |
| G7alt | altered | G Altered (=Ab mm VII) | G HW dim | none |
| G7b9 | | G HW diminished | G Phrygian Dominant | none |
| G7#11 | lydian dom | G Lydian Dominant (=D mm IV) | — | none |
| Dm7b5 | iiø in Cm | D Locrian | D Locrian #2 | Eb often |
| Bdim7 | vii°/passing | B WH diminished | — | none |

**Canonical melodic minor mode mapping**:
- I: m(maj7) → tonic minor
- II (Dorian b2): sus b9
- III (Lydian Aug): maj7#5
- IV (Lydian Dominant): 7#11 / tritone sub
- V (Mixolydian b13): dominant → minor
- VI (Locrian #2): m7b5 in major ii-V-i
- VII (Altered): 7alt

## Pop/film patterns (all addable as new modes)

| Pattern | RN | C major example |
|---|---|---|
| Axis | I–V–vi–IV | C–G–Am–F |
| 50s doo-wop | I–vi–IV–V | C–Am–F–G |
| Pop minor | vi–IV–I–V | Am–F–C–G |
| Mixolydian rock | I–bVII–IV | C–Bb–F |
| Flat-VI pop | I–bVI–bVII–I | C–Ab–Bb–C |
| Minor-iv pop | I–iv–I | C–Fm–C |
| Jazz ii-V-I | ii7–V7–Imaj7 | Dm7–G7–Cmaj7 |
| Turnaround | I–vi–ii–V | Cmaj7–Am7–Dm7–G7 |
| Bird changes | iii–VI7–ii–V7 | Em7–A7–Dm7–G7 |
| Chromatic mediant | I→III or I→bVI | C→E or C→Ab |

Chromatic mediants (Williams, Zimmer film scoring) — neo-Riemannian L/P/R operations.

## Sources
- Berklee Jazz Harmony (Nettles & Graf)
- Mark Levine, *The Jazz Theory Book* (Sher 1995)
- Alan Kingstone, *The Barry Harris Harmonic Method for Guitar*
- Howard Rees, *The Barry Harris Workshop Video*
- Walter Everett, *The Beatles as Musicians* (Oxford 1999/2001)
- Steve Coleman M-Base interviews
- Wikipedia: Chord-scale system, Tritone substitution, I-V-vi-IV
- Cochrane Music BH 6th dim scale articles
- MTO 00.6.1 Levine review; MTO 10.4 Everett
- Open Music Theory 4-chord schemas, substitutions
- Learn Jazz Standards: Rhythm Changes
