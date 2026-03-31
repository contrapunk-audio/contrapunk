# Round 4: Goertzel Harmonic Feature Fusion

## Hypothesis

Mel-spectrograms capture pitch (fret) well but struggle with string identity
for shared pitches. Goertzel harmonic ratios encode the string's physical
timbre signature (wound vs plain, thick vs thin). Fusing both feature types
should improve string classification accuracy, especially for E2 (low).

## Features

- **Mel-spectrogram**: 64 mel bins, 1024 FFT, 256 hop -> flattened to ~6016 features (RF) or 2D input (CNN)
- **Goertzel harmonics**: 9 harmonic ratios (H2/H1 ... H10/H1) + spectral centroid + inharmonicity = 11 features
- **Fusion**: concatenation (RF) or two-branch architecture (CNN)

## Results

| Model | Round 1 | Round 4 | Delta |
|-------|---------|---------|-------|
| Random Forest | 93.3% | 93.8% | +0.4% |
| Hybrid CNN | 92.0% | 93.5% | +1.4% |
| Pure CNN | 96.2% | 96.4% | +0.2% |

## Per-String Accuracy

| String | Random Forest R4 | Hybrid CNN R4 | Pure CNN R4 | Random Forest R1 | Hybrid CNN R1 | Pure CNN R1 |
|--------|--------|--------|--------|--------|--------|--------|
| E2 (low) | 88.3% | 86.5% | 94.3% | 86.1% | 81.3% | 94.3% |
| A2 | 91.3% | 90.0% | 94.8% | 91.3% | 88.3% | 93.5% |
| D3 | 97.4% | 95.7% | 97.0% | 97.0% | 93.9% | 97.4% |
| G3 | 95.7% | 97.4% | 98.3% | 95.7% | 96.1% | 98.3% |
| B3 | 93.9% | 94.3% | 96.1% | 93.5% | 96.1% | 96.5% |
| E4 (high) | 96.1% | 97.0% | 98.3% | 96.5% | 96.5% | 97.4% |

## Key Findings

- **Random Forest E2 (low)**: +2.2% (R1: 86.1% -> R4: 88.3%)
- **Hybrid CNN E2 (low)**: +5.2% (R1: 81.3% -> R4: 86.5%)
- **Pure CNN E2 (low)**: +0.0% (R1: 94.3% -> R4: 94.3%)

## Visualizations

- `round_04_comparison.png` -- overall + per-string accuracy bar charts
- `round_04_delta.png` -- per-string accuracy deltas (R4 - R1)

