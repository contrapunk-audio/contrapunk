# Run Contrapunk side-by-side with a DAW

Contrapunk is a standalone app, not a DAW plugin (yet — see #9). To use it alongside Logic Pro / Ableton Live / FL Studio / Reaper / Bitwig, you bridge MIDI and (optionally) audio between the two processes using virtual loopback devices.

This guide covers the **side-by-side** scenario: Contrapunk's harmony voices feed your DAW's instrument tracks via virtual MIDI, and you optionally route the DAW's audio output back into Contrapunk for the built-in synth or future analysis features.

If you just want Contrapunk → standalone plugin (no DAW), use [IAC_PLUGIN_SETUP.md](./IAC_PLUGIN_SETUP.md) instead. Same MIDI mechanics, simpler audio path.

## macOS

### MIDI bridge: IAC Driver

1. Open **Audio MIDI Setup** (`/Applications/Utilities/Audio MIDI Setup.app`).
2. `Window` → `Show MIDI Studio`.
3. Double-click **IAC Driver** → tick **Device is online**.
4. The default "Bus 1" is enough for one-track routing. Add more buses (`+`) if you want per-voice port-based routing.

### Audio bridge: BlackHole 2ch

[BlackHole](https://existential.audio/blackhole/) is a free, open-source virtual audio loopback driver. The 2-channel build is the right pick for stereo bridging.

1. Install BlackHole 2ch via Homebrew: `brew install --cask blackhole-2ch`. Or download from [existential.audio](https://existential.audio/blackhole/).
2. In Audio MIDI Setup → MIDI Studio is now Audio devices: confirm "BlackHole 2ch" appears.
3. (Optional, for hearing your DAW *and* Contrapunk simultaneously) create an **Aggregate Device** combining BlackHole 2ch with your audio interface:
   - Audio MIDI Setup → click `+` bottom-left → **Create Aggregate Device**.
   - Check "BlackHole 2ch" + your interface.
   - Set the aggregate device as your system output if you want Contrapunk and the DAW playing into the same monitor.

### Contrapunk-side wiring

1. Launch Contrapunk.
2. **MIDI Output** picker → select `IAC Driver Bus 1` (and additional buses if you set them up).
3. **MIDI Input** picker → either a hardware controller, or `IAC Driver Bus 1` if you want to chain through another tool.
4. **Audio Output** (built-in synth) → leave as the system default unless you've created the Aggregate Device above.

### DAW-side wiring (Logic Pro)

1. Logic Pro → Preferences → MIDI → Inputs → confirm `IAC Driver Bus 1` is enabled.
2. Create a software-instrument track.
3. In the track header, set **MIDI In** → `IAC Driver Bus 1`. Logic will now receive Contrapunk's harmony notes.
4. Arm the track for recording / monitoring as usual.

### DAW-side wiring (Ableton Live)

1. Preferences → Link/Tempo/MIDI → MIDI tab.
2. Under **Input Ports**, find `IAC Driver (Bus 1)` and turn **Track** on (you can also enable Remote if you want CCs mapping live).
3. Create a MIDI track. Set its `MIDI From` to `IAC Driver (Bus 1)` and the channel to match what Contrapunk emits (default: channel 1 for channel-based, or the per-voice channel for MPE).

### DAW-side wiring (Reaper)

1. Options → Preferences → Audio → MIDI Devices.
2. Right-click `IAC Driver Bus 1` → Enable input.
3. Insert a new track. Set its MIDI input to `IAC Driver Bus 1, all channels`.
4. Record-arm.

## Windows

### MIDI bridge: loopMIDI

[loopMIDI](https://www.tobias-erichsen.de/software/loopmidi.html) is the standard free virtual MIDI cable on Windows.

1. Install loopMIDI from tobias-erichsen.de.
2. Launch it; click `+` to create a port named e.g. `Contrapunk Out`. Add a second port `Contrapunk Out 2` for per-voice routing if needed.
3. In Contrapunk, pick `Contrapunk Out` as the MIDI output port.
4. In your DAW (FL Studio, Cubase, Bitwig, Reaper), enable the same `Contrapunk Out` port as a MIDI input device in your audio preferences, then route a MIDI track from it.

### Audio bridge: VB-Audio Virtual Cable

[VB-Cable](https://vb-audio.com/Cable/) by VB-Audio is the loopMIDI equivalent for audio.

1. Install VB-Cable from vb-audio.com (free, donation-ware).
2. In your DAW audio output settings, route the audio output to "CABLE Input (VB-Audio Virtual Cable)".
3. In Contrapunk, if you want to consume that audio, pick "CABLE Output" as audio input. (Currently only useful if you're using the guitar-input pipeline — Contrapunk doesn't yet do general audio analysis on system input.)
4. For monitoring both Contrapunk and the DAW: use VoiceMeeter (also from VB-Audio) as a software mixer.

## Linux

### MIDI bridge: ALSA virtual port or `a2jmidid`

1. Modern desktops with PipeWire: ALSA-level virtual MIDI ports auto-appear. You can also create one explicitly with `modprobe snd-virmidi`.
2. Alternative: install `a2jmidid` and run `a2jmidid -e` to bridge JACK MIDI ↔ ALSA.
3. Contrapunk and your DAW (Ardour, Bitwig Linux, Reaper Linux) both pick the virtual port from their MIDI device pickers.

### Audio bridge: JACK or PipeWire

Both PipeWire (newer) and JACK provide arbitrary inter-application audio routing. Use `qjackctl` (JACK) or `pwvucontrol` / `helvum` (PipeWire) to draw connections between Contrapunk's audio output and your DAW's input.

## Tips and pitfalls

- **Latency**: virtual MIDI is essentially zero-latency; virtual audio bridges (BlackHole, VB-Cable) add ~1-2 buffer periods. Don't compensate manually; let the DAW's automatic plugin-delay compensation handle it.
- **Stuck notes**: if Contrapunk crashes or you yank a USB MIDI cable mid-phrase, send a CC 123 (All Notes Off) from your DAW to Contrapunk to recover — Contrapunk handles CC 123 as a panic drain since v1.2.x. Or restart routing.
- **Per-voice routing**: Contrapunk's per-voice routing table can send each harmony voice (soprano / alto / tenor / bass) to a different MIDI port. Set up multiple IAC / loopMIDI buses, then assign in Contrapunk's Voice Routing panel.
- **MPE mode**: if you want Contrapunk to drive an MPE-capable instrument (Equator, Pigments, MPE-mode Kontakt), turn on **MPE / per-string channels** in Contrapunk's Routing settings. The instrument needs MPE mode enabled too.
- **Audio feedback loops**: if you route Contrapunk's audio out → DAW → BlackHole → Contrapunk in, you'll create a feedback loop. Mute Contrapunk's built-in synth if you only want DAW-rendered audio.

## What this guide does NOT cover

- **Audio-rate sidechain triggers** (Contrapunk reacts to DAW audio peaks). Planned for a future Contrapunk release; see issue #99 for status. For now Contrapunk only consumes MIDI input, not audio.
- **Tempo sync between DAW and Contrapunk's transport**. Manual right now (set the same BPM in both). Ableton Link sync is being researched but the canonical Rust binding has a GPL license that conflicts with Contrapunk's MIT posture — see `.planning/research/CLEAN-ROOM-CANDIDATES.md` for the long-term plan.
- **Plugin-format Contrapunk** (VST3 / AU / CLAP). The plugin shell exists but isn't shipping yet. See issue #9 for status.

## Compatibility notes

| DAW | Tested | Notes |
|---|---|---|
| Logic Pro 11 | yes | IAC Driver works out of the box. MPE auto-recognized by ESP / SC standalone instruments. |
| Ableton Live 12 | yes | Enable each IAC bus in Preferences. MPE requires a Live 12 MPE-aware instrument or a Max for Live device. |
| Reaper 7 | yes | Most flexible — enable per port, per channel. Custom routing via ReaRoute on Windows. |
| Bitwig Studio 5 | yes | Native MPE support. |
| FL Studio | partial | Windows-only via loopMIDI. macOS via Wine is fragile; not recommended. |

If your DAW isn't listed, the MIDI bridge mechanism is universal — every DAW has a "MIDI input device" picker.
