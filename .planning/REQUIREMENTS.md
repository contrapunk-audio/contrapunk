# Requirements: Contrapunk Rust

**Defined:** 2026-01-28
**Core Value:** Real-time harmony generation with minimal latency

## v1 Requirements

### MIDI I/O

- [x] **MIDI-01**: User can select MIDI input device from available ports
- [x] **MIDI-02**: User can select 2-8 MIDI output ports for harmony voices
- [x] **MIDI-03**: Original note passes through to first output port
- [x] **MIDI-04**: Harmony notes route to additional output ports

### Configuration

- [x] **CONF-01**: User can select musical key (C through B)
- [x] **CONF-02**: User can select harmony mode (1-7)
- [x] **CONF-03**: User can change key/mode during playback without stopping

### Harmony Modes

- [x] **HARM-01**: Mode 1 - Forward MIDI as-is (pass-through)
- [x] **HARM-02**: Mode 2 - Diatonic thirds above input note
- [x] **HARM-03**: Mode 3 - Diatonic fourths above input note
- [x] **HARM-04**: Mode 4 - Random diatonic interval below input
- [x] **HARM-05**: Mode 5 - Random diatonic below (excluding seconds)
- [x] **HARM-06**: Mode 6 - Contrary motion (harmony moves opposite to melody)
- [x] **HARM-07**: Mode 7 - Strict counterpoint (traditional voice leading rules)

### GUI

- [x] **GUI-01**: Native window renders with egui/eframe
- [x] **GUI-02**: Display active notes and current configuration
- [x] **GUI-03**: Controls for device selection, key selection, mode selection
- [x] **GUI-04**: Virtual piano keyboard showing input and harmony notes
- [x] **GUI-05**: Chord detection displaying what chord the combined notes form

### Distribution

- [x] **DIST-01**: Compiles to single binary with no runtime dependencies

### Octave Variations

- [ ] **OCT-01**: Octave Spread - each harmony voice in progressively different octaves
- [ ] **OCT-02**: Bass/Treble Split - harmonies below melody go low, above go high
- [ ] **OCT-03**: Mirror Octaves - harmonies duplicate across multiple octaves simultaneously

### Humanization

- [ ] **HUM-01**: Timing jitter - random delays (5-30ms) on harmony note onsets
- [ ] **HUM-02**: Velocity variation - randomize note velocity within ±10-20 range
- [ ] **HUM-03**: Note duration variation - slight sustain changes on harmony notes
- [ ] **HUM-04**: Swing/groove - shift off-beat notes for rhythmic feel

## v2 Requirements

### Extended Features

- **EXT-01**: MIDI file input for offline processing
- **EXT-02**: Preset save/load for configurations
- **EXT-03**: Audio-to-MIDI conversion (pitch detection)
- **EXT-04**: Algorithmic melody generation

## Out of Scope

| Feature | Reason |
|---------|--------|
| Audio-to-MIDI conversion | Reduces complexity, removes heavy dependencies |
| Algorithmic melody generation | Scope reduction for v1, focus on live performance |
| TUI/curses interface | Replaced by native GUI |
| Tkinter GUI | Legacy Python UI not being ported |
| MIDI file output | Real-time focus for v1 |

## Traceability

| Requirement | Phase | Status |
|-------------|-------|--------|
| MIDI-01 | Phase 1 | Complete |
| MIDI-02 | Phase 1 | Complete |
| MIDI-03 | Phase 1 | Complete |
| MIDI-04 | Phase 1 | Complete |
| CONF-01 | Phase 2 | Complete |
| CONF-02 | Phase 2 | Complete |
| CONF-03 | Phase 2 | Complete |
| HARM-01 | Phase 2 | Complete |
| HARM-02 | Phase 2 | Complete |
| HARM-03 | Phase 2 | Complete |
| HARM-04 | Phase 2 | Complete |
| HARM-05 | Phase 2 | Complete |
| HARM-06 | Phase 2 | Complete |
| HARM-07 | Phase 2 | Complete |
| GUI-01 | Phase 3 | Complete |
| GUI-02 | Phase 3 | Complete |
| GUI-03 | Phase 3 | Complete |
| GUI-04 | Phase 3 | Complete |
| GUI-05 | Phase 3 | Complete |
| DIST-01 | Phase 3 | Complete |
| OCT-01 | Phase 5 | Pending |
| OCT-02 | Phase 5 | Pending |
| OCT-03 | Phase 5 | Pending |
| HUM-01 | Phase 6 | Pending |
| HUM-02 | Phase 6 | Pending |
| HUM-03 | Phase 6 | Pending |
| HUM-04 | Phase 6 | Pending |

**Coverage:**
- v1 requirements: 26 total
- Mapped to phases: 26
- Unmapped: 0

---
*Requirements defined: 2026-01-28*
*Last updated: 2026-01-28 after roadmap creation*
