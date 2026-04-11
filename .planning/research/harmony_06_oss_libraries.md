# Research 06 — OSS Harmony Libraries + Implementation Recommendation

Agent: `research-oss-harmony-libs`.

## Python / Academic

### music21 (cuthbertLab)
- BSD-3-Clause. MIT Music & Theater. Active v9/v10 (2025-2026)
- Bach chorales built in (326 usable). `corpus.chorales.Iterator()`
- Roman numeral analysis via `roman.RomanNumeral`
- `figuredBass.realizer` can auto-realize figured bass via backtracking
- **Use as**: offline data generator. Not embeddable (Python)

### DeepBach (Hadjeres 2017)
- `Ghadjeres/DeepBach` — 4 parallel LSTM+FF + pseudo-Gibbs sampling
- **20-60 seconds per 16-bar chorale** — NOT real-time
- Beats humans in Turing test
- **Use as**: offline Performance Mode analyzer

### Coconet / Bach Doodle (Magenta)
- Apache 2.0. Conv orderless NADE, ~400KB model, ~2s in TF.js
- **Use as**: offline Performance Mode via WASM ONNX

### Others
- BachBot (2016) — Torch/Lua, deprecated
- Strasheela — Mozart/Oz CSP, dormant, offline only

## Rust

**No Bach harmonizer exists in Rust. Contrapunk would be the first.**

- `rust-music-theory` — primitives only (notes, scales, chords). No harmony analysis.
- `kord` — chord parsing + guessing. No progression generation.
- `staff`, `music-note`, `mumuse` — toy scale.

## Academic papers with code

| Paper | Year | Approach |
|---|---|---|
| Ebcioğlu CHORAL | 1986-88 | Hand-coded expert system, 350 rules |
| Rohrmeier generative syntax | 2011 | CFG, Chomsky-style |
| Tymoczko Geometry of Music | 2011 | Voice-leading orbifolds |
| Pachet "Markov Constraints" | 2011 | Markov + hard constraints |
| DeepBach | 2017 | 4-voice parallel LSTM |
| Coconet | 2017 | Orderless NADE conv |
| Allan & Williams | 2005 | HMM over Bach |
| BacHMMachine | 2022 | Interpretable HMM |

## Ranked algorithms for Contrapunk

Constraints: **Rust, <15ms per note, monophonic input, deterministic, explainable**.

1. **Pure rule-based (CHORAL-style)**: 5/10 quality, <100µs. Predictable.
2. **Markov chain**: 6/10 progressions, 4/10 voice leading. <50µs.
3. **⭐ Hybrid Markov+rules**: 7.5/10. <500µs. Determinism with seeded RNG.
4. Markov Constraints (Pachet): 8/10, unbounded latency for long windows.
5. CSP (Strasheela): 9/10 quality, **unbounded latency**, offline only.
6. Neural (DeepBach/Coconet): 9/10, 200ms-seconds, Performance Mode only.

## 🎯 RECOMMENDATION: Hybrid Markov+Rules (Approach #3)

**Build**:
- **`BachChorale` mode**: 2nd-order HMM over (functional state, bass scale-degree, metric position). Chord-by-chord sampling, then through existing `voice_leading/rules.rs` as hard constraint layer. Markov table trained **offline** from 326 Bach chorales via music21, exported as MessagePack binary
- **`FunctionalHarmony` mode**: Non-Markov variant. Picks functionally-correct RN for incoming melody note. Deterministic, zero training data

**Why**:
1. Fits 15ms latency budget with ~1ms actual
2. Matches Contrapunk's explainability: debug panel can show "chose V7 (p=0.43), rejected vi (parallel fifth)"
3. Reuses existing voice-leading infrastructure
4. Upgrade path: rules → Markov → Markov Constraints → Performance Mode
5. Training is offline one-time: music21 → serialize to MessagePack → Rust `HashMap<State, WeightedChoice<Chord>>` (<500KB binary)

**Crates to add**: `rand = "0.8"`, `rand_distr = "0.4"`, `rmp-serde = "1"`, `phf = "0.11"`

**Do NOT add**: `tract-onnx`, `z3`, `petgraph`-CSP — Performance Mode concerns only.

## Training pipeline (offline Python)

Script: `scripts/train_bach_markov.py`
- Iterate `music21.corpus.chorales.Iterator()`
- Extract Roman numerals via `roman.romanNumeralFromChord(chord, key)`
- Count `(from_state, to_state)` transitions
- Normalize to probabilities
- Serialize via `msgpack` package (NOT pickle — safer)
- Output: `assets/bach_markov.msgpack`

Rust load:
```rust
const BACH_MARKOV: &[u8] = include_bytes!("../assets/bach_markov.msgpack");
static TABLE: Lazy<HashMap<State, WeightedChoice<Chord>>> =
    Lazy::new(|| rmp_serde::from_slice(BACH_MARKOV).unwrap());
```

## Out of scope
- Don't link humlib or music21 at runtime (freeze parsed snapshot at build)
- Don't ship neural harmonizer in main binary
- Don't use `rust-music-theory`/`kord`/`staff` — keep rolling own

## Key Contrapunk files for implementation
- `src/harmony/engine.rs` — add `HarmonyMode::BachChorale`, `FunctionalHarmony` to match
- `src/harmony/modes.rs` — add `bach_chorale()` + `functional_harmony()` functions
- `src/harmony/config.rs` — extend `HarmonyMode` enum
- `src/harmony/voice_leading/rules.rs` — already has parallel detection; reuse
- `src/harmony/stateful.rs` — hold Markov state between notes
- NEW: `scripts/train_bach_markov.py`
- NEW: `assets/bach_markov.msgpack`
