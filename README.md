# Contrapunk

## Description

This is a program that generates counterpoint for a given melody.

## Usage

```bash
python main.py
```

## Modes

1. As is forwarding
2. Forward Diatonic 3rds
3. Forward random diatonic intervals
4. Forward random diatonic intervals (excluding seconds)
5. Forward contrary motion with random intervals (excluding seconds)
6. Forward strict counterpoint (following standard rules)

The modes can be changed in real-time while playing using number keys 1-6.
Press 'q' to quit.

## Counterpoint Rules (Mode 6)

The strict counterpoint mode follows these rules:
1. Prefers contrary motion
2. Uses consonant intervals (3rds, 6ths, perfect 5ths, octaves)
3. Avoids parallel fifths and octaves
4. Properly resolves leading tones
5. Uses step motion after leaps
6. Maintains limited range between voices

