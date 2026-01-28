# Requirements: Contrapunk Rust

**Defined:** 2026-01-28
**Core Value:** Real-time harmony generation with minimal latency

## v1 Requirements

### MIDI I/O

- [ ] **MIDI-01**: User can select MIDI input device from available ports
- [ ] **MIDI-02**: User can select 2-8 MIDI output ports for harmony voices
- [ ] **MIDI-03**: Original note passes through to first output port
- [ ] **MIDI-04**: Harmony notes route to additional output ports

### Configuration

- [ ] **CONF-01**: User can select musical key (C through B)
- [ ] **CONF-02**: User can select harmony mode (1-7)
- [ ] **CONF-03**: User can change key/mode during playback without stopping

### Harmony Modes

- [ ] **HARM-01**: Mode 1 - Forward MIDI as-is (pass-through)
- [ ] **HARM-02**: Mode 2 - Diatonic thirds above input note
- [ ] **HARM-03**: Mode 3 - Diatonic fourths above input note
- [ ] **HARM-04**: Mode 4 - Random diatonic interval below input
- [ ] **HARM-05**: Mode 5 - Random diatonic below (excluding seconds)
- [ ] **HARM-06**: Mode 6 - Contrary motion (harmony moves opposite to melody)
- [ ] **HARM-07**: Mode 7 - Strict counterpoint (traditional voice leading rules)

### GUI

- [ ] **GUI-01**: Native window renders with egui/eframe
- [ ] **GUI-02**: Display active notes and current configuration
- [ ] **GUI-03**: Controls for device selection, key selection, mode selection

### Distribution

- [ ] **DIST-01**: Compiles to single binary with no runtime dependencies

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
| MIDI-01 | Phase 1 | Pending |
| MIDI-02 | Phase 1 | Pending |
| MIDI-03 | Phase 1 | Pending |
| MIDI-04 | Phase 1 | Pending |
| CONF-01 | Phase 2 | Pending |
| CONF-02 | Phase 2 | Pending |
| CONF-03 | Phase 2 | Pending |
| HARM-01 | Phase 2 | Pending |
| HARM-02 | Phase 2 | Pending |
| HARM-03 | Phase 2 | Pending |
| HARM-04 | Phase 2 | Pending |
| HARM-05 | Phase 2 | Pending |
| HARM-06 | Phase 2 | Pending |
| HARM-07 | Phase 2 | Pending |
| GUI-01 | Phase 3 | Pending |
| GUI-02 | Phase 3 | Pending |
| GUI-03 | Phase 3 | Pending |
| DIST-01 | Phase 3 | Pending |

**Coverage:**
- v1 requirements: 17 total
- Mapped to phases: 17
- Unmapped: 0 ✓

---
*Requirements defined: 2026-01-28*
*Last updated: 2026-01-28 after initial definition*
