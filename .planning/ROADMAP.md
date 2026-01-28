# Roadmap: Contrapunk Rust

## Overview

Port Contrapunk from Python to Rust for real-time MIDI harmony generation. The journey builds incrementally: first establish MIDI connectivity (hear notes pass through), then implement harmony algorithms (hear harmonies), finally wrap in a native GUI and ship as a single binary.

## Phases

**Phase Numbering:**
- Integer phases (1, 2, 3): Planned milestone work
- Decimal phases (2.1, 2.2): Urgent insertions (marked with INSERTED)

Decimal phases appear between their surrounding integers in numeric order.

- [x] **Phase 1: MIDI Foundation** - Establish MIDI input/output connectivity with pass-through
- [x] **Phase 2: Harmony Engine** - Implement music theory core and all 7 harmony modes
- [ ] **Phase 3: GUI and Distribution** - Native egui interface and single-binary packaging
- [ ] **Phase 4: Server Mode** - Network server for remote MIDI harmony processing

## Phase Details

### Phase 1: MIDI Foundation
**Goal**: User can connect MIDI devices and hear notes pass through the application
**Depends on**: Nothing (first phase)
**Requirements**: MIDI-01, MIDI-02, MIDI-03, MIDI-04
**Success Criteria** (what must be TRUE):
  1. User can select a MIDI input device from a list of available ports
  2. User can select 2-8 MIDI output ports from available ports
  3. User can play a note on input device and hear it on the first output port
  4. Application runs without GUI (CLI or headless mode) for testing MIDI flow
**Plans**: 3 plans

Plans:
- [x] 01-01-PLAN.md — Project setup and MIDI port enumeration
- [x] 01-02-PLAN.md — MIDI connections and pass-through routing
- [x] 01-03-PLAN.md — Hardware verification checkpoint

### Phase 2: Harmony Engine
**Goal**: User can play notes and hear harmonies generated in any of the 7 modes
**Depends on**: Phase 1
**Requirements**: CONF-01, CONF-02, CONF-03, HARM-01, HARM-02, HARM-03, HARM-04, HARM-05, HARM-06, HARM-07
**Success Criteria** (what must be TRUE):
  1. User can select musical key (C through B) and it affects harmony output
  2. User can switch between all 7 harmony modes and hear different results
  3. User can change key and mode while playing without stopping or restarting
  4. Mode 1 passes notes through unchanged
  5. Modes 2-7 produce audibly different harmonies following their algorithms
**Plans**: 6 plans

Plans:
- [x] 02-01-PLAN.md — Foundation types (Key, HarmonyMode, Scale with diatonic transposition)
- [x] 02-02-PLAN.md — Stateless modes 1-5 and HarmonyEngine struct
- [x] 02-03-PLAN.md — Stateful modes 6-7 (ContraryMotion, StrictCounterpoint)
- [x] 02-04-PLAN.md — Router integration with harmony processing
- [x] 02-05-PLAN.md — CLI key and mode selection
- [x] 02-06-PLAN.md — Hardware verification checkpoint

### Phase 3: GUI and Distribution
**Goal**: User has a complete native application with visual interface as a single binary
**Depends on**: Phase 2
**Requirements**: GUI-01, GUI-02, GUI-03, DIST-01
**Success Criteria** (what must be TRUE):
  1. Application opens as a native window (not terminal)
  2. User can see current configuration (key, mode, active notes) in the GUI
  3. User can change all settings (input device, output ports, key, mode) via GUI controls
  4. Application compiles to single binary that runs without external dependencies
**Plans**: TBD

Plans:
- [ ] 03-01: TBD

### Phase 4: Server Mode
**Goal**: Binary runs as a server allowing remote users to connect and receive MIDI harmony generations back to their output devices
**Depends on**: Phase 3
**Requirements**: TBD (to be defined during planning)
**Success Criteria** (what must be TRUE):
  1. Application can start in server mode, listening on a configurable network port
  2. Remote clients can connect and send MIDI input to the server
  3. Server processes MIDI through harmony engine and returns harmonized output to clients
  4. Multiple clients can connect simultaneously (server handles concurrent sessions)
  5. Clients receive harmonized MIDI output routable to their local output devices
**Plans**: TBD

Plans:
- [ ] 04-01: TBD (run /gsd:plan-phase 4 to break down)

## Progress

**Execution Order:**
Phases execute in numeric order: 1 -> 2 -> 3 -> 4

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 1. MIDI Foundation | 3/3 | Complete | 2026-01-28 |
| 2. Harmony Engine | 6/6 | Complete | 2026-01-28 |
| 3. GUI and Distribution | 0/? | Not started | - |
| 4. Server Mode | 0/? | Not started | - |

---
*Roadmap created: 2026-01-28*
*Last updated: 2026-01-28 — Phase 2 complete (Harmony Engine)*
