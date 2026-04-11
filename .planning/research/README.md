# Harmony Rework Research (April 2026)

Triggered by Lycomedes1814's HN critique on comment 47663649 — "Bach mode in C major produces Dm7 Em7 Fmaj7; C G7 Am would be more Bach." Seven parallel research agents investigated every angle.

## Reports

1. **[harmony_01_functional_satb.md](harmony_01_functional_satb.md)** — Classical functional harmony (Rameau/Riemann) + Bach chorale SATB voice-leading rules. Concrete Rust types: `Chord`, `Quality`, `Inversion`, `Function`, `ProgressionState`. Hard/soft constraint split. Corpus-approximate transition weights (T→S=50, S→D=80, D→T=100, D→S=FORBIDDEN).

2. **[harmony_02_fux_species.md](harmony_02_fux_species.md)** — Fux species counterpoint 1-5, all rules enumerated. Current Contrapunk `StrictCounterpoint` is a **relaxed species 1**; 7 rules can be added today without API changes. Species 2-5 need beat-position + suspension-phase state. Species 5 needs phrase lookahead.

3. **[harmony_03_bach_corpus.md](harmony_03_bach_corpus.md)** — Actual transition matrices from BacHMMachine/DeepBach/deClercq. Major + minor scale-degree matrices in CSV. Top 10 three-chord progressions. Cadence distribution (73% authentic, 21% half, 2% plagal, 1.5% deceptive). Voice motion statistics. Doubling rules. Empirical voice ranges.

4. **[harmony_04_langoy_scales.md](harmony_04_langoy_scales.md)** — Complete extraction of 57 scales from langøy.net/skala across 9 families. Author = Lycomedes1814 (confirmed via HN comment). **26-29 new scales to add** to Contrapunk's current 28. Ready-to-paste Rust enum variants.

5. **[harmony_05_jazz_barry_harris.md](harmony_05_jazz_barry_harris.md)** — Jazz/modern functional harmony + **real** Barry Harris theory. **Critical finding: current `BarryHarris` mode captures only ~5% of the method**. BH needs 8-note scale guard, beat-phase input, drop-2 4-voice output (not 2). Chord-scale lookup table. Pop/film progression patterns (Axis, doo-wop, mixolydian rock, etc.).

6. **[harmony_06_oss_libraries.md](harmony_06_oss_libraries.md)** — music21, DeepBach, BachBot, Coconet, Strasheela, Rust crates. **Recommendation: Hybrid Markov+rules (Approach #3)**. 7.5/10 quality, <500µs latency, deterministic with seeded RNG. Training pipeline: offline music21 → MessagePack → Rust `HashMap<State, WeightedChoice<Chord>>` (<500KB binary).

7. **[harmony_07_next_note_formula.md](harmony_07_next_note_formula.md)** — The "what to play next" scoring function. Based on Meyer + Narmour + Huron + Temperley + Bach corpus statistics. **11-term weighted sum**, pure function, O(1) per candidate, ~500 ops for 48-note guitar range, <1ms total. Default weights derived from empirical Bach chorale soprano statistics (79% stepwise, 89% strong-beat chord tone).

## Immediate decisions forced by this research

1. **Rename** current modes for honesty:
   - `DiatonicThirds` → "Stacked Thirds (chained)"
   - `BarryHarris` → "BH Parallel 2nds (requires 8-note BH scale)"

2. **Fix BarryHarris** — require 8-note scale, add beat-phase, return 4-voice drop-2

3. **Add ~20 new modes** (see master list below)

4. **Port langøy scales** — pending permission from Lycomedes1814 via HN reply

5. **Extend `CounterpointState`** with 7 local rules to reach strict Fux species 1

## Master list of new modes to add

### Functional harmony family
- **FunctionalHarmony** — deterministic RN picker (T→PD→D→T skeleton)
- **BachChorale** — Markov progression + existing voice-leading rules
- **JazzReharmonize** — ii-V-I + tritone subs + secondary dominants
- **ModalInterchange** — rate-limited borrowed chords with priority order
- **ChromaticMediant** (film scoring) — neo-Riemannian L/P/R operations

### Species counterpoint family
- **SpeciesOne** — strict Fux 1:1 (replaces relaxed `StrictCounterpoint`)
- **SpeciesTwo** — 2:1 with passing tones
- **SpeciesThree** — 4:1 with passing/neighbor/cambiata
- **SpeciesFour** — suspensions (9-8, 7-6, 4-3, 2-3)
- **SpeciesFive** — florid (needs phrase lookahead)

### Pop/rock progression family
- **AxisProgression** — I-V-vi-IV variants
- **DooWop** — I-vi-IV-V
- **PopMinor** — vi-IV-I-V
- **MixolydianRock** — I-bVII-IV
- **FlatVIPop** — I-bVI-bVII-I

### Barry Harris family
- **BarryHarrisProper** — 8-note scale guard, drop-2 voicings, chord/passing parity
- Replaces current `BarryHarris` as correct implementation

### Melodic
- **NextNoteSuggest** — not a harmony mode but an overlay layer (see research_07)

## Architecture changes needed

1. **`HarmonicContext` struct** on `HarmonyEngine`:
   - `last_chord: Option<ChordSymbol>`
   - `target_chord: Option<ChordSymbol>`
   - `key_center: (PitchClass, Mode)`
   - `beat_position: BeatPhase`
   - `borrowing_budget: f32`
   - `chord_scale_cache: HashMap<ChordSymbol, Scale>`

2. **Replace `interchange_enabled: bool`** with:
   ```rust
   InterchangePolicy {
       enabled: bool,
       priority: Vec<ScaleMode>,
       max_per_bar: f32,
       hold_bars: u8,
   }
   ```

3. **Mode function signature change**: some modes need to return up to 4 notes (not 2). Ripples through `harmonize_note_on` path.

4. **Add beat-phase** parameter to mode dispatcher.

5. **New module `src/harmony/functional/`**:
   - `chord.rs` — Chord/Quality/Inversion types
   - `function.rs` — Function/ScaleDegree mapping
   - `progression.rs` — ProgressionState state machine
   - `rules.rs` — additional constraint checkers
   - `cadence.rs` — cadence detection

6. **New offline Python script** `scripts/train_bach_markov.py` to extract transition matrix from music21 corpus.

## Triggered HN follow-up
Reply to Lycomedes1814 on HN comment 47663649 acknowledging the critique + asking permission to port langøy.net/skala scale database.
