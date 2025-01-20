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

# Ideas

- start the notes after some time like you are playing a chord naturally
- notes one to four start at different points in time
- make the tracks to follow different rhythms
- counter melodies are always in different rhythms
- separate configurable drum tracks  
- Global tempo
- Dedicated rhythm track annotation
- Uniqueness factor for each track
- Plugin hosting support (VST, AU etc.)
- Plugin build (to use inside DAW)
- Style annotations
- MIDI file input
- Guitar to MIDI using Pitch Detection
- WebRTC support
- Candombe rhythm support, swing support for rhythm
- Is the user playing classical music or jazz music
- How good of a musician is the user 
- Short window music classification for the above
- Tonic identification
- Melodic motif discovery
- Onset detection and prediction
- Meter inference
- Indian art music: a computational perspective 
- WIMAGA Workshop April 7 2025
