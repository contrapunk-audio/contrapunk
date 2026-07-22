# Route Contrapunk with a DAW

Contrapunk can run **inside a DAW** as a plug-in or **beside a DAW** as the desktop app. Use the plug-in path when your published release and host support it; use virtual MIDI when you need desktop-only features or explicit multi-port routing.

## Plug-in routing

### Logic Pro: MIDI controller or MIDI region

On a software-instrument channel strip:

1. Open the **MIDI FX** slot.
2. Choose **Audio Units → Contrapunk Audio → Contrapunk**.
3. Load your instrument after the MIDI FX slot.
4. Record-enable or monitor the track.
5. Play MIDI and confirm that the instrument receives the generated notes.

The regular **Contrapunk** component is MIDI-only. Logic does not send track audio to a MIDI FX slot.

### Logic Pro: live guitar audio

On the guitar audio track:

1. Set the track input to the correct interface channel.
2. Insert **Audio FX → Audio Units → Contrapunk Audio → Contrapunk Guitar**.
3. Enable input monitoring.
4. Confirm that the editor shows the incoming guitar signal. The component accepts mono or stereo input and passes the track audio through.
5. Create a software-instrument track.
6. Route the CoreMIDI source **Contrapunk Guitar MIDI Out** to that instrument through Logic's MIDI Environment or your preferred CoreMIDI routing method.

Do not route the virtual MIDI output back into the Guitar component. After installing a new component version, restart Logic and rescan Audio Units if its cache still shows old metadata.

For acceptance testing, play one dry sustained note and then a slow four-note phrase. Confirm uninterrupted guitar audio, pitch activity in the editor, and matching generated NoteOn/NoteOff events at the instrument.

### VST3 and CLAP hosts

Contrapunk's VST3/CLAP build emits generated MIDI/events for a downstream instrument. Put Contrapunk before the instrument or route its event output to another track according to the host's plug-in-routing model. MIDI-output support varies by DAW; if the host cannot route a plug-in's generated events, use the desktop/virtual-MIDI workflow below.

The DAW host owns transport, audio devices, and instrument sound. Controls that only make sense in the desktop app are capability-gated in the plug-in UI.

#### Windows: install and rescan in FL Studio

1. Close FL Studio and run `Contrapunk-Plugins-Windows-x64.exe` as administrator. The installer puts `Contrapunk.vst3` in `C:\Program Files\Common Files\VST3`; do not copy files out of that bundle.
2. Remove any older, manually copied `Contrapunk.vst3` bundles from custom plug-in folders so FL Studio sees only one copy.
3. Open **Options → Manage plugins**. The standard VST3 folder should already be scanned; add it to the search paths if it is absent.
4. When replacing the same displayed version with a test build, enable **Rescan previously verified plugins** and **Rescan plugins with errors**, then choose **Find installed plugins**.
5. Load **Contrapunk** as a generator. To drive another instrument with its generated MIDI, use FL Studio's plug-in event routing when available; otherwise use the loopMIDI workflow below.

Published Windows installers are currently unsigned, so Windows may show a SmartScreen warning. Download installers only from the official Contrapunk GitHub release or an explicitly linked Contrapunk Actions run.

## Desktop app beside a DAW

The desktop app sends harmony voices through a virtual MIDI port. The DAW receives that port on an instrument track.

```text
Controller or guitar → Contrapunk desktop → virtual MIDI → DAW instrument
```

### macOS: IAC Driver

1. Open **Audio MIDI Setup** in `/Applications/Utilities`.
2. Choose **Window → Show MIDI Studio**.
3. Open **IAC Driver**, enable **Device is online**, and create one or more buses.
4. In Contrapunk, select an IAC bus as the MIDI output.
5. In the DAW, enable that bus as a MIDI input and monitor a software-instrument track.

One bus is enough for channel-based routing. Create multiple buses only when using port-based per-voice routing.

#### Logic Pro

Enable the IAC bus in Logic's MIDI settings, then route it to a monitored software-instrument track. Use the Audio Unit workflow above when you do not need the desktop app.

#### Ableton Live

1. Open **Settings/Preferences → Link, Tempo & MIDI**.
2. Enable **Track** for the IAC input port.
3. Choose the IAC bus in the MIDI track's **MIDI From** selector.
4. Monitor the track and load an instrument.

#### Reaper

1. Open **Preferences → Audio → MIDI Devices**.
2. Enable the IAC bus as an input.
3. Record-arm a track and select the bus as its MIDI input.

### Windows: loopMIDI

1. Install [loopMIDI](https://www.tobias-erichsen.de/software/loopmidi.html).
2. Create a port such as `Contrapunk Out`.
3. Select it as Contrapunk's MIDI output.
4. Enable the same port as a DAW MIDI input and monitor an instrument track.

### Linux: ALSA, JACK, or PipeWire

Use an ALSA virtual MIDI port (`snd-virmidi`) or bridge JACK MIDI with `a2jmidid`. PipeWire/JACK patch-bay tools such as Helvum or qjackctl can connect the Contrapunk output to the DAW input.

## Optional audio loopback

Virtual audio is not required for MIDI harmony. Use it only when a workflow explicitly needs audio shared between applications.

- **macOS:** [BlackHole 2ch](https://existential.audio/blackhole/)
- **Windows:** [VB-Cable](https://vb-audio.com/Cable/)
- **Linux:** JACK or PipeWire

Avoid routing Contrapunk's monitored audio back into its own input; that creates a feedback loop.

## Routing notes

- **Channel-based routing:** one MIDI connection; channel 1 is the MPE master, channel 2 carries the melody, and later channels carry generated voices.
- **Port-based routing:** each voice uses a separate output port on channel 1.
- **Stuck-note recovery:** use Contrapunk's panic/reset control or restart routing. Configuration changes and transport stops also drain owned notes.
- **Tempo:** plug-in builds follow host transport where available. The desktop side-by-side workflow does not currently synchronize transport with the DAW; set the same BPM manually when needed.
- **Direct hosting:** the desktop app can host CLAP instruments/effects. Generic VST3 instrument hosting is parked and is not a current release feature.

## Compatibility status

| Host | Supported route |
|---|---|
| Logic Pro | Contrapunk MIDI FX AU; Contrapunk Guitar Audio FX AU; IAC side-by-side |
| Ableton Live | IAC/loopMIDI side-by-side; plug-in event routing depends on host support |
| Reaper | VST3/CLAP event routing or virtual MIDI |
| Bitwig Studio | CLAP/VST3 or virtual MIDI; verify the selected track/event routing |
| FL Studio | loopMIDI side-by-side; verify plug-in event routing in the target version |

If a host drops generated events, use virtual MIDI rather than adding an in-process instrument host to Contrapunk.
