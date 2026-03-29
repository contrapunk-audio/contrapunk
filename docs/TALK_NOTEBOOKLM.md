# Contrapunk: From Palestrina's Rules to Real-Time MIDI Harmony

## Source Document for NotebookLM Slide Generation

**Talk Duration:** 18 minutes (17 min content + 1 min demo)
**Speaker:** Vibhav Bobade
**Project:** Contrapunk — Open source, real-time MIDI harmony generator with classical voice leading rules
**Demo:** Live MIDI keyboard → harmonized output in DAW

---

# PART 1: THE ART OF COUNTERPOINT (4 minutes)

## What Is Counterpoint?

Counterpoint is the art of combining multiple independent melodic lines into a harmonious whole. Not just stacking chords — each voice has its own melody, its own rhythm, its own life. When done well, you can listen to any single voice and hear a complete, beautiful melody. Listen to all of them together and they create something greater.

The word comes from Latin "punctus contra punctum" — point against point, note against note.

## Palestrina (1525-1594): The Lawmaker

Giovanni Pierluigi da Palestrina composed over 100 masses and 300+ motets for the Catholic Church. His music was so perfectly balanced that it became the gold standard for polyphonic composition for the next 500 years.

**Palestrina's Rules (codified by Johann Joseph Fux in "Gradus ad Parnassum", 1725):**

1. **No parallel fifths or octaves** — When two voices are a perfect fifth or octave apart, they must NOT move in the same direction to another fifth or octave. Why? Because parallel perfect intervals make voices fuse together perceptually — they stop sounding like independent melodies and start sounding like one voice. This is the most famous rule in all of music theory.

2. **Stepwise motion preferred** — Melodies should move primarily by step (one note to the adjacent note). Large leaps must be compensated by moving in the opposite direction afterward.

3. **Voice independence** — Each voice must be a self-sufficient melody on its own. Contrary motion (voices moving in opposite directions) is preferred over parallel motion.

4. **Careful dissonance** — Only consonances (thirds, sixths, fifths, octaves) on strong beats. Dissonances only as passing tones or suspensions, always resolved by step.

5. **No voice crossing** — The soprano stays above the alto, the alto above the tenor. Each voice knows its place.

**The Council of Trent (1545-1563)** nearly banned polyphonic music from churches because it had become so complex that congregants couldn't understand the sacred words. Legend holds that Palestrina's Missa Papae Marcelli — with its clean, singable lines that allowed clear text declamation — "saved" polyphonic church music from being banned entirely.

## Bach (1685-1750): The Rule Breaker

Bach knew Palestrina's rules. He studied Fux's treatise. Then he expanded the vocabulary dramatically:

- **Fugues** — The ultimate contrapuntal form. Independent voices enter one by one, each stating the main theme. The Art of Fugue contains fugues where the subject is played upside down, backwards, stretched, compressed, and combined with itself.

- **The Well-Tempered Clavier** — 48 preludes and fugues in all 24 major and minor keys. Many regard it as containing the most compelling counterpoint ever written.

- **Chorale harmonizations** — Bach's 371 chorales are the textbook for four-part harmony. The melody sits in the soprano, the bass provides harmonic foundation with its own melodic interest, and inner voices fill out the harmony while maintaining their own logic.

**How Bach broke Palestrina's rules:**
- Used chromaticism freely (secondary dominants, diminished sevenths, augmented sixths)
- Wrote for instruments, not just voices — allowing wider leaps and faster passages
- Combined multiple fugue subjects simultaneously
- Made harmony serve dual purpose: each line is both melody AND harmonic progression

## Jazz: The Rule Rewriter

Jazz voice leading follows the same principles but applies them differently:

- **Parallelism is OK** — Parallel chords moving together can sound "quite nice and jazzy." Post-bop and modal jazz embrace what Palestrina forbade.

- **Barry Harris (1929-2021)** developed the "sixth diminished scale" — an 8-note scale combining a major sixth chord with a diminished seventh chord. It provides smooth pathways for modulation because of the multiple dominant chords available within the diminished structure. His approach represents dense, chromatic, bebop-derived voice leading.

- **Bill Evans (1929-1980)** revolutionized jazz piano voicings with "rootless voicings" — dropping the root of chords and playing 3rd, 5th, 7th, 9th instead. The famous "So What" voicing (a major third above a stack of perfect fourths) became one of the most iconic sounds in jazz. His work on Miles Davis's Kind of Blue helped define modal jazz.

- **Guide tone lines** — Jazz musicians prioritize smooth "guide tone" lines through chord changes: the 3rds and 7ths of each chord moving by step or common tone. This IS counterpoint — just with different rules.

## Radiohead: Counterpoint in Rock

Jonny Greenwood first heard Olivier Messiaen's Turangalila Symphony at age 15 and became obsessed. A concert of Krzysztof Penderecki's music in the early 1990s was "a conversion experience." He was appointed composer-in-residence to the BBC Concert Orchestra in 2004.

**Counterpoint in Radiohead songs:**
- **"Everything In Its Right Place"** — Layered synthesizer lines in 10/4 time, with digitally manipulated vocal fragments creating additional contrapuntal layers
- **"There There"** — Multiple layered guitar parts in altered tuning (DBDGBE), where sympathetic resonance between strings creates additional harmonic content. Independent guitar lines weave over a driving rhythm.
- **"Nude"** — Two distinct vocal layers: falsetto "oooohs" against Thom Yorke's main vocal. Bass, strings, and guitar create independent contrapuntal lines beneath.
- **"15 Step"** — In 5/4, layers drum loops, vocals, and guitar in independent rhythmic patterns, creating polyrhythmic counterpoint.

Greenwood's film scores for There Will Be Blood (chromatic counterpoint) and Phantom Thread (inspired by Glenn Gould's Bach recordings and 1950s jazz string records) directly bridge classical counterpoint into modern cinema.

## Why This Matters

Counterpoint is everywhere in modern music — from J Dilla's MPC sample chopping creating textural layers, to Kanye West sampling Aphex Twin's contrapuntal piano writing, to electronic producers layering independent melodic lines.

But here's the problem: **no tool exists that generates proper counterpoint in real-time.**

---

# PART 2: THE MARKET GAP (3 minutes)

## What Exists Today

**Hardware pedals:** Boss Harmonist, TC Helicon Harmony Singer, Eventide PitchFactor — all do "diatonic interval transposition." They shift your note up or down by a fixed interval within a chosen key. No voice leading awareness. No counterpoint rules.

**Software plugins:** Antares Harmony Engine, Waves Harmony, iZotope Nectar — same approach, just in software. Choose a key, choose an interval, get a harmony. Up to 8 voices. But each voice just follows the same simple rule: "go N scale degrees up."

**MIDI chord generators:** Cthulhu, Captain Chords — trigger full chord voicings from single notes. Harmony Bloom generates polyrhythmic patterns. None understand voice leading.

**Scaler 3 (Plugin Boutique, March 2025)** — The closest to voice-leading awareness. It has an "Auto Voice Leading" feature that "moves between chords smoothly, the way a trained pianist would." But it works on pre-selected chord progressions, not on a live, note-by-note input stream.

## The Gap

| Feature | Boss/TC/Eventide | Waves/Antares/Nectar | Cthulhu/Captain | Scaler 3 | **Contrapunk** |
|---|---|---|---|---|---|
| Real-time harmony | Yes | Yes | Yes (chords) | Yes (chords) | **Yes** |
| Voice-leading rules | No | No | No | Partial | **Yes** |
| Palestrina/Bach/Jazz styles | No | No | No | No | **Yes** |
| 28+ scale modes | No | No | No | Many | **Yes (28)** |
| Modal interchange | No | No | No | Limited | **Yes** |
| Contrary motion mode | No | No | No | No | **Yes** |
| Strict counterpoint mode | No | No | No | No | **Yes** |

**The gap:** Every existing tool is either a simple "diatonic interval transposer" or a chord-based tool. NONE generate a melodically independent counterpoint line in real-time against live input while following actual voice-leading rules from specific historical traditions.

## Open Source Landscape

- **music21 (Python, MIT)** — Comprehensive music analysis toolkit from MIT. Has a voice leading module and a species counterpoint module. But it's for analysis and offline generation, not real-time MIDI processing.
- **Tonal (JS)** — Music theory primitives. No voice leading.
- **No Rust library exists for counterpoint or voice leading.** This is exactly Contrapunk's niche.
- **No open-source real-time counterpoint-aware harmony generator exists.** Period.

---

# PART 3: WHAT CONTRAPUNK DOES (4 minutes)

## The Core Idea

Play a note on your MIDI keyboard. Contrapunk generates harmony notes that follow actual voice-leading rules — not just "a third above" but a melodically independent line that avoids parallel fifths, prefers stepwise motion, and maintains voice independence. In real-time. With sub-10ms latency.

## 8 Harmony Modes

**Simple (stateless):**
1. **Pass Through** — No harmony, notes pass unchanged
2. **Diatonic Thirds** — Adds 2 scale degrees above
3. **Diatonic Fourths** — Adds 3 scale degrees above
4. **Random Below** — Random diatonic interval below (musical variety)
5. **Random No Seconds** — Random below, excluding dissonant 2nds

**Advanced (stateful):**
6. **Contrary Motion** — Tracks previous melody and harmony, moves harmony OPPOSITE to melody direction. If you go up, the harmony goes down. True independence.
7. **Strict Counterpoint** — Full voice-leading engine with interval history, contour analysis, and scoring. Avoids repeating the same interval, encourages range expansion, prefers contrary motion.
8. **Barry Harris** — Moves by 2 scale degrees in the Barry Harris 6th diminished 8-note scale, preserving chord-tone/passing-tone parity.

## 28 Scale Modes Across 5 Families

**Diatonic Modes (7):** Ionian (Major), Dorian, Phrygian, Lydian, Mixolydian, Aeolian (Minor), Locrian

**Harmonic Minor Modes (7):** Harmonic Minor, Locrian Nat6, Ionian Aug, Dorian #4, Phrygian Dominant, Lydian #2, Super Locrian Dim

**Melodic Minor Modes (7):** Melodic Minor, Dorian b2, Lydian Aug, Lydian Dominant, Mixolydian b6, Locrian Nat2, Super Locrian

**Exotic Scales (5):** Double Harmonic, Hungarian Minor, Enigmatic, Neapolitan Minor, Neapolitan Major

**Barry Harris 6th Diminished (2, 8-note):** BH Major 6th Dim, BH Minor 6th Dim

## 4 Voice Leading Styles

Each style changes what "best voicing" means, creating dramatically different harmonies from the same input:

**Palestrina (Renaissance):**
- Hard reject: parallel fifths and octaves (instant rejection — non-negotiable)
- Stepwise bonus: +60 points
- Leap penalty: -15 per semitone
- Max leap: 5 semitones
- Contrary motion bonus: +40
- Result: Tight, smooth, flowing — like a Renaissance choir

**Bach Chorale (Baroque):**
- Hard reject: parallel fifths and octaves
- Common tone bonus: +70 (strongly prefer retaining notes between chords)
- Stepwise bonus: +25
- Max leap: 12 semitones (allows octave leaps)
- Result: Inner voices held, bass active

**Jazz (Modern):**
- Parallel fifths: ALLOWED (penalty only -2)
- Spread preference: +5 (prefer wide spacing)
- Leap penalty: 0 (leaps are free)
- Result: Open, spacious, drop-2/drop-3 voicings

**Free (Minimal):**
- All penalties = 0
- Any leap allowed
- Result: Whatever the algorithm produces

## Modal Interchange: Handling Out-of-Scale Notes

When a musician plays a note that's NOT in the current scale, Contrapunk doesn't just pick the closest note. It uses **modal interchange** — borrowing from parallel modes:

```
Note arrives → In scale?
  YES → diatonic harmony
  NO → Search parallel modes (Aeolian, Dorian, Mixolydian, etc.)
    FOUND → harmonize using the borrowed mode's rules
    NOT FOUND → use consonant chromatic intervals (thirds, sixths, fifths)
```

**5 borrowing range levels:** From conservative (just Aeolian + Harmonic Minor) to adventurous (all 11 parallel modes). This is how jazz musicians think — "that Db isn't wrong, it's borrowed from Phrygian."

The UI shows which mode was borrowed from in real-time: "from Aeolian", "from Phrygian Dominant" — making the harmonic logic transparent to the musician.

## Determinism: The Live Performance Problem

When a musician plays the same note twice, should they get the same harmony? The answer depends on context.

**Contrapunk's solution: Determinism by exhaustive scoring with 6-tier tiebreaking.**

When multiple voicings score equally, the engine breaks ties with:
1. Total score from style rules
2. Common tone count (prefer retaining notes from previous chord)
3. Stepwise-down count
4. Stepwise-up count
5. Total movement (prefer less overall voice movement)
6. Lexicographic MIDI ordering (last resort: sort by note values)

This is proven by a test that plays the same note with the same context 100 times and verifies the output is identical every time.

**But here's the musical magic:** In stateful modes (6-7), the context changes between notes. Playing C5 three times:
1. First C5: No history → output C3
2. Second C5: History says we just went down a 7th → engine avoids repeating → output E4
3. Third C5: Contour history says Down, Up → engine seeks variety → output G4

All deterministic. But musical variety emerges from accumulated state. The musician gets predictability (same context = same result) AND variation (context changes naturally as they play).

---

# PART 4: HOW IT WORKS (3 minutes)

## Architecture

```
MIDI Input Device
    ↓
HarmonyEngine
    ├── Scale check (in-key? modal interchange?)
    ├── Mode algorithm (8 modes)
    ├── Voice leading post-processing (Palestrina/Bach/Jazz/Free)
    ├── Octave mode (None/Spread/Split/Mirror)
    └── Humanization (timing jitter, velocity, swing)
    ↓
Multiple MIDI Output Ports → DAW
```

## The Voice Leading Scoring System

The engine generates ALL possible voicings, then scores each one:

**Hard constraints (immediate rejection):**
- Parallel fifths/octaves (in Palestrina/Bach styles)
- Voice crossing
- Excessive spacing

**Soft constraints (penalty scoring):**
- Spacing violations: -200 points (Palestrina) to 0 (Free)
- Voice crossing: -150 (Palestrina) to -2 (Free)
- Leap penalty: -15/semitone (Palestrina) to 0 (Jazz)

**Bonuses:**
- Common tones: +45 to +70
- Stepwise motion: +25 to +60
- Contrary motion: +20 to +40

The highest-scoring voicing wins. Same candidates + same scoring = same result. Deterministic.

## Humanization

Not just jitter. The humanizer is tempo-aware with an internal beat clock:
- **Timing jitter:** 1-30ms random delay on note-on
- **Velocity variation:** ±5-25 range
- **Swing:** Off-beat delay based on BPM and swing amount
- **11+ musical style presets** with character personas (The Alchemist, The Architect, The Clockmaker)

## Multi-Platform: One Codebase, Everywhere

Rust library compiles to:
- **Native desktop** (macOS, Linux, Windows via Tauri v2)
- **WebAssembly** (browser via wasm-pack) — deployed on Fly.io
- **Server mode** (TCP network processing for ensemble/studio setups)

Same harmony engine runs everywhere. No ports, no rewrites.

---

# PART 5: OPEN SOURCE (2 minutes)

## Why Open Source?

Counterpoint rules are centuries of accumulated human knowledge about what sounds good and why. They should be accessible to every musician, not locked behind proprietary plugins.

Contrapunk is:
- **MIT licensed** — use it in any project, commercial or otherwise
- **Written in Rust** — fast, safe, compiles to native + WASM
- **Extensively tested** — 280+ unit tests, including the 100x determinism test
- **Well documented** — every harmony mode, scale mode, and voice leading rule explained

## The Journey

The project started as a Python prototype on January 18, 2025 — 7 commits in 4 hours built a working harmony generator.

11 months later, the Rust rewrite began on January 28, 2026. That day: **65 commits** took the project from "map existing codebase" through the complete harmony engine with all 7 modes and a native GUI.

By 2:49 AM on January 29, the app was running in browsers — less than 8 hours after the rewrite began.

**312 total commits.** 28 requirements completed. 8 harmony modes. 28 scale variants. 4 voice leading styles. 40+ chord detection patterns. Native desktop + browser WASM deployment.

Driven by the GSD (Get Stuff Done) planning system: research → plan → execute → verify for every phase. 37 plans completed with an average execution time of 3.4 minutes per plan.

## What's Next

- ML-powered guitar-to-MIDI conversion (training data being captured now)
- Per-guitar calibration profiles
- Visual ML learning app (SvelteKit)
- Polyphonic chord detection from audio
- And the Tekton CI pipeline for automated audio quality testing

---

# PART 6: DEMO (1 minute)

Live demonstration: MIDI keyboard → Contrapunk → harmonized output in DAW

Show:
1. Play a melody in Pass Through mode (no harmony)
2. Switch to Diatonic Thirds — instant harmony
3. Switch to Strict Counterpoint with Palestrina style — watch the voice leading create independent lines
4. Play an out-of-scale note — watch modal interchange kick in ("from Aeolian")
5. Switch voice leading to Jazz style — same melody, completely different harmony character

---

# APPENDIX: Key Technical Details

## The 5 Voice Leading Rules (from code)

1. **check_parallel_fifths** — Detects when two harmony voices both move and maintain a perfect 5th (7 semitones mod 12)
2. **check_parallel_octaves** — Detects perfect octaves/unisons between voices moving together
3. **check_voice_crossing** — Ensures voices maintain their register order
4. **check_spacing** — Prevents gaps >12 semitones between adjacent voices (except bass-tenor)
5. **check_motion_independence** — Penalizes all harmony voices moving in the same direction

## Octave Modes

- **None:** Original pitch
- **Spread:** Each voice +1 octave
- **BassTrebleSplit:** Below melody → -1 octave, above → +1 octave
- **Mirror:** Each voice duplicated at ±1 octave (3x output)

## Chord Detection

Real-time recognition of extended chords, slash chords, add chords with roman numeral analysis. 40+ chord patterns detected.

## Borrowing Range Levels

| Level | Modes Searched |
|-------|----------------|
| 1 | Aeolian, Harmonic Minor |
| 2 | +Dorian, Melodic Minor |
| 3 | +Mixolydian, Phrygian |
| 4 | +Lydian, Phrygian Dominant |
| 5 | +Locrian, Ionian, Lydian Dominant (all 11 parallel modes) |

## Project Stats

- **312 commits** over ~2 months
- **280+ tests** (unit + integration)
- **5,030 lines** of audio processing code
- **28 scale modes** across 5 families
- **8 harmony modes** (5 stateless + 3 stateful)
- **4 voice leading styles** (Palestrina, Bach Chorale, Jazz, Free)
- **11+ humanization presets** with character personas
- **Deployed to:** Fly.io (browser WASM) + native desktop (Tauri v2)
