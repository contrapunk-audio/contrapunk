# Wavetable-synthesis chapter exercises

Runnable Rust companions for the first four chapters of the local wavetable-synthesis workbook and the research umbrella in [issue #188](https://github.com/contrapunk-audio/contrapunk/issues/188).

The crate is deliberately small: one phase accumulator, visible formulas, offline rendering, and the already-used `hound` WAV writer. It is a teaching model, not Elixir's production oscillator.

## Run

```bash
cargo test -p wavetable-synthesis-exercises
cargo run -p wavetable-synthesis-exercises --bin ch01_harmonics -- /tmp/ch01.wav
cargo run -p wavetable-synthesis-exercises --bin ch02_gesture -- /tmp/ch02.wav
cargo run -p wavetable-synthesis-exercises --bin ch03_studio -- /tmp/ch03.wav
cargo run -p wavetable-synthesis-exercises --bin ch04_modular -- /tmp/ch04.wav
```

Listen at a fixed safe level. Each WAV is generated locally; no recording is redistributed.

## Cumulative ladder

### Chapter 1: cycles, harmonics, and additive tone

1. Verify `period_seconds` and `harmonic_frequency` by hand and in the tests.
2. Trace `SineOscillator::tick`: phase advances by $2\pi f/f_s$ per sample.
3. Change the coefficient list passed to `additive_sample` and predict the spectrum first.
4. Render the same melody with a sine and a four-harmonic recipe.
5. Extend the eight-note A/B ostinato without changing pitch or rhythm.
6. Add a Nyquist test before trying higher notes or longer coefficient lists.

`ch01_harmonics` uses the opening of the traditional French tune *Ah! vous dirai-je, maman*. The melody predates Mozart; his public-domain variations provide a historical score reference ([IMSLP K.265/300e](https://imslp.org/wiki/12_Variations_on_%27Ah%2C_vous_dirai-je_maman%27%2C_K.265%2F300e_(Mozart%2C_Wolfgang_Amadeus))). The binary contains a new monophonic synthesis, not a copied engraving, arrangement, or recording.

### Chapter 2: difference frequency and continuous gesture

1. Use `heterodyne_components` for 300.000 and 299.560 kHz.
2. Compare linear addition with the audible scaled multiplication at the start of `ch02_gesture`.
3. Verify `cents_ratio(1200) == 2` and the logarithmic midpoint from 220 to 880 Hz is 440 Hz.
4. Compare detached and gliding renderings of the same note centers.
5. Move the accent without changing pitch; then change vibrato without changing amplitude.
6. Explain why `SineOscillator::tick(frequency_hz, ...)` integrates a trajectory correctly while `sin(2π f(t)t)` generally does not.

`ch02_gesture` uses the public-domain NEW BRITAIN pitch incipit `5-1-3-1-3-2-1-6-5-5`, with simplified durations. The Library of Congress documents the tune's 1835 pairing with *Amazing Grace* ([timeline](https://www.loc.gov/collections/amazing-grace/articles-and-essays/timeline/)); [Hymnary tune record](https://hymnary.org/tune/new_britain)). No lyrics or modern arrangement are included.

### Chapter 3: tape operations and stored control

1. Verify that changing playback rate couples duration and pitch.
2. Reverse and splice clips while predicting the exact frame count.
3. Draw the attack and release envelope before applying it.
4. Verify the tiny FIR from its impulse response.
5. Multiply a 440 Hz source by 110 Hz and check the 330/550 Hz components.
6. Read `data/ch03-paper-roll.csv`, then compare its integer event lanes with the rendered study.

The paper-roll study is project-authored and deterministic. It is a teaching surrogate, not a recording of a historical studio or RCA machine.

### Chapter 4: voltage control and a modular patch

1. Verify that one octave of control doubles oscillator frequency.
2. Follow the ADSR state changes, including retrigger and release from the current level.
3. Compare the VCO before and after the one-pole teaching filter.
4. Trace one sequence step across pitch, gate, envelope, VCF, and VCA.
5. Find the explicit previous-sample variable that makes feedback causal and bounded.
6. Render the 16-step study and verify its four-second length and safe peak.

The Chapter 4 motif is project-authored and released with the workbook under CC0-1.0. The one-pole filter is an audible teaching VCF, not a Moog ladder emulation.

## Boundaries

- A 48 kHz renderer cannot represent historical radio-frequency oscillators directly; Chapter 2 labels its 1000/560 Hz multiplication as an audible scaled model.
- Harmonics at or above Nyquist are skipped. This is not a production band-limited wavetable system.
- Generated files stay outside Git unless a review explicitly needs a small licensed fixture.
