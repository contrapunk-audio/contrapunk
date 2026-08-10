# Deterministic performance contract

For one Contrapunk release, the same initial configuration, ordered MIDI/transport events, and starting runtime state produce the same semantic NoteOn, NoteOff, sustain, routing, and final ownership state.

This contract does **not** promise sample-identical floating-point audio, identical OS/device delivery jitter, or real-time delivery after an arbitrarily long stall.

## Ordering

- Companion decisions use a fixed `1/256`-beat grid, independent of audio block and router polling boundaries.
- One callback processes at most 256 grid slots; remaining backlog stays ordered for later callbacks.
- Tauri interleaves each grid slot in this order: loop Companion, due loop source events, live Companion.
- Looper events retain their exact fixed-point scheduled beat within the grid delivery window.
- Held-note configuration replay is ordered by pitch, then source channel, and retains original velocity.
- Route, sustain, MIDI-bend, and panic drains use canonical target/channel/note order.

## Ownership

- Repeated pitches keep FIFO-generated frames instead of overwriting one pitch entry.
- MIDI-channel ownership distinguishes equal pitches on separate MPE channels.
- Live and loop routes share musical route identities but retain separate origin counts.
- Release and panic paths are idempotent and finish with no routed notes, sustain owners, scheduled loop sources, synth voices, or MIDI Slide voices.
- Editing one route drains that route only; the global All to Synth override intentionally drains every route.
- Stale harmony releases use exact recorded targets/channels rather than a broadcast NoteOff.

## Reset and snapshot

Desktop **Reset performance** stops and rewinds transport, preserves configuration and the recorded loop take, stops loop playback, recreates clean harmony history, clears Companion/Phrase runtime, and requests an exact output drain. Press Play to begin the next run from beat zero.

`get_performance_snapshot` returns schema version 1 with the app version, harmony, Companion, Slide, routing, transport, synth, detune, and loop-buffer configuration. Route assignments are canonicalized.

## MIDI wire validation

`OutputRouter::recording()` captures the stable device port and exact bytes at the OS MIDI boundary without requiring hardware. Fresh-run tests compare these wire traces. Actual driver/device latency remains outside the contract.

## Compatibility

Selectable `RandomBelow` modes were removed. Legacy values migrate deterministically:

- `random_below` / mode slot 4 → Contrary Motion
- `random_below_no_seconds` / mode slot 5 → Strict Counterpoint

The plug-in keeps the obsolete parameter indices reserved so existing DAW automation does not shift. The product editor reports their deterministic replacement names.

## Executable contract

The versioned typed trace and lifecycle regressions live in:

- `crates/contrapunk-harmony/tests/deterministic_performance.rs`
- `src-tauri/src/commands/engine_determinism_tests.rs`
- `src-tauri/src/commands/determinism.rs`
- `crates/contrapunk-midi/src/output.rs`

Run:

```bash
cargo test -p contrapunk-harmony --tests
cargo test -p contrapunk-tauri deterministic_performance_tests
```
