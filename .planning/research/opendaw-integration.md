# openDAW Integration Research

**Date:** 2026-04-05
**GitHub Issue:** #10
**Status:** Research complete, ready for planning

## Summary

openDAW is a next-generation web-based DAW by André Michelle (open source, AGPL v3). Contrapunk can integrate as a **MIDI effect device** that takes MIDI input and outputs original notes + real-time counterpoint harmony voices.

## Architecture: 5-Layer Device Pattern

Every openDAW device follows this pattern:

| Layer | Package | Purpose |
|-------|---------|---------|
| **Schema** | `studio/forge-boxes` | Data model — declares parameter fields via `DeviceFactory.createMidiEffect()` |
| **Adapter** | `studio/adapters` | Runtime bridge — wraps fields with `ValueMapping` + `StringMapping` for UI/automation |
| **Processor** | `studio/core-processors` | DSP/MIDI logic — runs in AudioWorklet thread |
| **Editor** | `app/studio` | UI — Knobs, RadioGroups, toggles |
| **Registration** | `EffectFactories` + visitor pattern | Wiring into the device system |

## Contrapunk as a MIDI Effect

Contrapunk fits as a `MidiEffectProcessor` (like Arpeggio, Pitch, Velocity):

- Implements `NoteEventSource` + `NoteEventTarget`
- Pull-based generator pipeline: `processNotes(from, to, flags)` yields events
- Pattern: yield original notes + yield harmony voices for each note-on
- Track source→harmony mapping for proper note-off handling

### processNotes pseudocode:
```
for event in upstream.processNotes(from, to, flags):
    yield event                          // pass through original
    if event is note-on:
        harmonies = wasm_engine.note_on(event.pitch)
        for h in harmonies:
            yield NoteLifecycleEvent.start(h)  // add harmony voice
    if event is note-off:
        for h in tracked_harmonies[event.pitch]:
            yield NoteLifecycleEvent.stop(h)   // release harmony voices
```

## Parameter Mapping

| Contrapunk Parameter | openDAW Pattern |
|---------------------|-----------------|
| Key (C-B) | `ValueMapping.linearInteger(0, 11)` + `StringMapping.indices(["C","C#","D",...])` |
| Scale (28 modes) | `ValueMapping.linearInteger(0, 27)` + `StringMapping.indices([...])` |
| Harmony Mode (8) | `ValueMapping.linearInteger(0, 7)` + `StringMapping.indices([...])` |
| Voice Count (1-8) | `ValueMapping.linearInteger(1, 8)` + `StringMapping.numeric()` |
| Voice Position | `ValueMapping.values(["soprano","alto","tenor","bass"])` |
| Voice Leading Style | `ValueMapping.values(["Free","Palestrina","BachChorale","Jazz"])` |
| Voice Leading On/Off | `ValueMapping.bool` |
| Modal Interchange | `ValueMapping.bool` + `ValueMapping.linearInteger(1, 5)` for range |

## WASM Integration

Follows the proven NAM (Neural Amp Modeler) pattern:

1. Main thread fetches `contrapunk.wasm` binary via `fetchContrapunkWasm()` protocol method
2. Binary transferred to AudioWorklet via MessagePort
3. Compiled inside worklet using wasm-bindgen glue
4. Processor calls `engine.note_on()`/`note_off()` synchronously per 128-sample block

### Key constraint:
- `AudioWorkletGlobalScope` has no `fetch()` — binary must come from main thread
- wasm-bindgen output needs `--target no-modules` or manual `WebAssembly.Module` instantiation
- Processing must complete within ~2.67ms audio budget at 48kHz

## Contrapunk WASM API (what the adapter calls)

From `wasm/src/lib.rs`:

- `Engine::new()` → create engine
- `set_key("C")`, `set_mode("StrictCounterpoint")`, `set_scale_mode("Ionian")`, etc.
- `note_on(u8) -> Vec<u8>` → feed note, get back original + harmony notes
- `note_off(u8) -> Vec<u8>` → release note, get back notes to release
- `get_state() -> JsValue` → read-only introspection for UI sync

**Limitation:** No velocity or channel handling in harmony engine — adapter must manage externally.

## Integration Paths (ordered by feasibility)

### Path A: Wait for Device SDK (safest)
- `loading-devices-at-runtime.md` describes planned runtime device loading
- Devices would be standalone packages with manifest.json + processor + editor
- No timeline — post-1.0 (Q3 2026 target for 1.0)

### Path B: Fork and PR (heavy, needs buy-in)
- Fork openDAW monorepo, add all 5 layers
- Submit small focused PRs (André rejects large PRs)
- Book Calendly call first: calendly.com/andremichelle/opendaw-on-tour
- AGPL license implications for combined work

### Path C: Headless SDK (most realistic near-term)
- `npm install @opendaw/studio-sdk` (LGPL, compatible with MIT)
- Build standalone app using openDAW's audio engine + Contrapunk's WASM
- Reference: naomiaro/opendaw-test has 14 demos + 22 docs
- No upstream dependency — Contrapunk stays MIT

### Path D: Werkstatt script prototype (quick POC)
- Werkstatt = scriptable audio effect device in openDAW
- Could prototype harmony generation as a script
- No WASM interop in scripting system though

## Recommended Approach

1. **Now**: Book Calendly with André Michelle, discuss integration interest
2. **Short-term**: Path C — headless SDK integration for Contrapunk Cloud
3. **Medium-term**: Path B — fork, add device, submit PRs with André's guidance
4. **Long-term**: Path A — standalone device package when runtime loading ships

## Reference Integration: TONE3000/NAM

TONE3000 integrated as a NeuralAmpDevice following the exact 5-layer pattern:
- Schema: `NeuralAmpDeviceBox.ts` with fields for input-gain, output-gain, model pointer
- Adapter: `NeuralAmpDeviceBoxAdapter.ts` with ValueMapping.decibel
- Processor: `NeuralAmpDeviceProcessor.ts` — singleton WASM module, per-instance handles
- Editor: `NeuralAmpDeviceEditor.tsx` with knobs + TONE3000 browse button
- Registration: `EffectFactories.ts` with `external: true` flag

Key patterns from NAM:
- Singleton WASM module shared across instances
- Lazy loading (only on first device creation)
- Content-addressable model deduplication
- OPFS caching for offline support

## Key Files in openDAW

- Device creation guide: `packages/app/studio/public/manuals/creating-a-device.md`
- Runtime loading plan: `plans/loading-devices-at-runtime.md`
- NAM integration plan: `plans/nam-integration.md`
- MIDI effect base: `packages/studio/core-processors/src/MidiEffectProcessor.ts`
- Pitch device (reference): `packages/studio/core-processors/src/devices/midi-effects/PitchDeviceProcessor.ts`
- Arpeggio device (reference): `packages/studio/core-processors/src/devices/midi-effects/ArpeggioDeviceProcessor.ts`
- Device factory: `packages/studio/forge-boxes/src/schema/std/DeviceFactory.ts`
- MIDI chain wiring: `packages/studio/core-processors/src/MidiDeviceChain.ts`
- Engine processor: `packages/studio/core-processors/src/EngineProcessor.ts`

## Contacts

- **André Michelle**: andre.michelle@opendaw.org / calendly.com/andremichelle/opendaw-on-tour
- **Discord**: discord.gg/ZRm8du7vn4
- **@Chaosmeister**: Most active external contributor
- **@naomiaro**: Community SDK docs and demos

## Licensing

- openDAW main repo: AGPL v3 (copyleft for combined works)
- `@opendaw/studio-sdk`: LGPL v3 (more permissive for library usage)
- `@opendaw/nam-wasm`: MIT
- Commercial license available from André
- Contrapunk is MIT — Path C (headless SDK, LGPL) is safest for license compatibility
