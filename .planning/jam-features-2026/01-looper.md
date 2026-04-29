# Wk 1 Feature — Looper / Phrase Recorder

**Ship by**: Thu Apr 30, 2026 EOD
**Live for jam**: Fri May 1
**Risk**: Medium (depends on transport/pattern infra stability)
**Pairs with**: Koji Kondo themed jam — see `website/.planning/jam-2026/composers/01-kondo.md`

---

## Minimum shippable definition (lock Monday)

A user can:
1. Press a "Loop" button (or hit a hotkey) while playing notes
2. Capture the next 1, 2, 4, or 8 bars of MIDI input as a loop buffer (length picker UI)
3. Hear that loop play back through the harmony engine on subsequent transport cycles
4. Improvise *additional* notes on top of the loop while the engine harmonizes both the looped material AND the live notes
5. Press "Stop loop" to clear

That's it. No multi-track loops, no per-track mute/solo, no overdub layers, no quantize correction. **Single loop, single takes, single playback. MVP.**

---

## Why it fits Kondo

Kondo's whole compositional method is motif-as-foundation. You need to be able to lay down a 2-bar Mario-style motif and improvise a counter-melody on top to authentically emulate that style. Without a looper this is impossible in real-time solo improvisation.

---

## Files to touch

| File | Change |
|---|---|
| `crates/contrapunk-transport/src/lib.rs` | Add `loop_buffer: Option<LoopBuffer>` to transport state; on each transport tick, replay loop buffer events when at the right beat |
| `crates/contrapunk-midi/src/lib.rs` | Tap into MIDI input stream to copy events into LoopBuffer when recording |
| `src/router.rs` | Route looped-back MIDI events through harmony engine same as live input |
| `src/lib.rs` (or wherever AppState lives) | Add `LooperConfig { recording: bool, length_bars: u8, buffer: Option<LoopBuffer> }` |
| `ui/src/lib/components/LooperPanel.svelte` | New UI: Loop button, length picker (1/2/4/8 bars), Stop button, visual indicator that loop is playing |
| `ui/src/lib/stores/looper.ts` | Frontend store; dispatches `start_recording`, `stop_recording`, `clear_loop` to backend via Tauri command |
| `src-tauri/src/commands/looper.rs` | Tauri command handlers — `start_loop`, `stop_loop`, `clear_loop` |
| `wasm/src/lib.rs` | WASM bindings so the browser version (`app.contrapunk.com`) gets the same feature |

---

## Day-by-day plan

### Mon (Apr 27)
- Lock the spec above (this doc) — no scope additions after EOD
- Read `crates/contrapunk-transport/` to understand current transport tick model
- Sketch `LoopBuffer` data structure (Vec<(beat_offset, MidiEvent)>?)
- Decide: do we quantize captured events to nearest 16th, or store raw timestamps? **Default: raw timestamps, no quantize correction in MVP**

### Tue (Apr 28)
- Implement `LoopBuffer` + capture logic in `contrapunk-transport`
- Implement playback (transport tick → emit looped events when at the right beat offset)
- Backend unit test: capture 4 events over 1 bar, verify they replay in next transport cycle

### Wed (Apr 29) — **CHECKPOINT DAY**
- Wire to harmony engine via router (looped events should harmonize same as live)
- Build Tauri commands + WASM bindings
- **Wednesday checkpoint: is the backend round-trip working?** If not, descope per below

### Thu (Apr 30)
- Build Svelte UI: Loop button, length picker, indicator
- Browser test in `app.contrapunk.com` build
- Native test in Tauri build
- Record demo video (30 sec showing Kondo-style motif + counter-melody)
- **Merge to main**, deploy web build, smoke test
- Brief documentation paragraph for the website's `/jam/1` page

### Fri (May 1)
- Jam goes live; monitor for bug reports in Discord; hotfix small things only

---

## Wednesday descope option (if 60% checkpoint fails)

If by Wed EOD the backend loop capture+replay is not working end-to-end, ship **a stripped version**:
- Remove the length picker UI — just "Loop the last 2 bars" button (fixed length)
- Remove Stop button — looper auto-clears after 8 cycles or on app reload
- Don't ship to native (Tauri) — web only, simpler deploy

If even THIS isn't ready by Thu EOD, drop to plugin-curation week.

---

## Acceptance criteria

- [ ] User can record a 1, 2, 4, or 8 bar loop via UI button
- [ ] Loop plays back at the same tempo as the transport (no drift)
- [ ] Live notes layered on top harmonize alongside the looped material
- [ ] Loop visually indicates "I'm recording" / "I'm playing" / "I'm stopped" states
- [ ] Works in both desktop (Tauri) and web (`app.contrapunk.com`) builds
- [ ] No audio glitches when starting/stopping loop mid-bar
- [ ] No regressions in existing pattern sequencer / harmony / transport behavior

---

## Demo for the jam (30-60 sec video)

Script:
1. Open `app.contrapunk.com`
2. Pick mode: Diatonic Thirds, scale: C major
3. Hit "Loop" with length = 2 bars
4. Play a 4-note Mario-ish motif (e.g., E E E C E G)
5. Loop captures it, plays it back
6. Improvise a counter-melody on top while the loop continues
7. Hit "Stop loop" — clean exit

Save as `cover/demos/01-looper.mp4` in website repo. Embed in `/jam/1` page + post in Discord Friday morning.

---

## Risks (known unknowns)

- **Transport timing**: if transport tick isn't reliable enough, loop will drift. Mitigation: stress-test with 8-bar loops at 120 BPM for 5 minutes before merging.
- **MIDI input forking**: capturing input for the loop while ALSO sending to harmony engine — be careful not to double-process events. Mitigation: use a clear `LoopMidiSource` enum tag on captured events.
- **WASM perf**: replaying many events per tick in the browser might glitch audio. Mitigation: profile in browser DevTools before claiming victory.
- **Existing pattern sequencer overlap**: pattern seq just shipped; looper is conceptually similar but distinct. Make sure they don't fight for the transport tick.

---

## After ship — feeds Wk 6

The Looper is a hard dependency for Wk 6's Motif Transposer (which lets you transpose a recorded loop's pitch as a unit). Treat the LoopBuffer data structure as production-grade — it'll be extended (not rewritten) in Wk 6.

If Wk 1 ships cleanly, Wk 6 takes ~3 days instead of 5-7. If Wk 1 is a hack, Wk 6 will be brutal.
