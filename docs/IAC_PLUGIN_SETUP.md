# Route Contrapunk into a standalone plugin (macOS via IAC)

Contrapunk outputs MIDI. To hear its harmony voices, you route that MIDI into
any instrument plugin's standalone app via the macOS IAC Driver. No DAW
needed.

This guide covers the most common setup: one guitar/keyboard in,
multi-voice harmony out, one plugin standalone (e.g. Arturia Analog V,
NI Kontakt, Pianoteq).

## Prerequisites

- macOS
- A plugin with a standalone executable. Confirmed working:
    - Arturia V Collection (Analog V, Jup-8 V, etc.)
    - Native Instruments Kontakt (standalone)
    - Pianoteq
    - Most Arturia / u-he / Spectrasonics instruments
- An audio interface (e.g. Audient iD14, Scarlett, etc.) — for hearing the
  plugin output

## Step 1 — Enable IAC buses

1. Open **Audio MIDI Setup** (Spotlight search or
   `/Applications/Utilities/`).
2. `Window` → `Show MIDI Studio`.
3. Double-click the **IAC Driver** icon.
4. Tick **Device is online**.
5. The default "Bus 1" is enough for channel-based routing (recommended).
   If you plan to use port-based routing, click `+` to add `Bus 2`,
   `Bus 3`, ... one per voice you want to split out.
6. Click **Apply**.

## Step 2 — Launch the plugin standalone

Example with Analog V (any Arturia plugin works the same way):

1. Open `/Applications/Arturia/Analog V 3.app` (or the standalone
   wrapper for your chosen plugin).
2. Open the plugin's **Settings** / **Audio & MIDI** panel.
3. Under **MIDI Input**, enable `IAC Driver Bus 1` (or whatever bus
   you're targeting).
4. Under **Audio Output**, pick your audio interface.
5. Load a patch.

At this point the plugin is listening for MIDI on IAC Bus 1. Silence so
far — nothing is sending yet.

## Step 3 — Configure Contrapunk

1. Launch Contrapunk (`cargo tauri dev` during development, or the
   packaged app).
2. **Input (left column)**: pick one of
    - a hardware MIDI keyboard
    - `Computer Keyboard` (QWERTY input; see keybindings below)
    - `Guitar Audio` (cpal + onset/pitch detection)
3. **Outputs (left column)**: pick `IAC Driver Bus 1` in slot 1. If
   using port-based routing with separate IAC buses, pick a different
   bus in slots 2, 3, 4 as needed.
4. **Routing mode (center column → Voices section)**:
    - `Channel-based` (default, recommended): all voices on one bus,
      channels 2–9. Your plugin can listen on all channels or filter.
    - `Port-based`: one voice per bus, each on channel 1.
5. Pick a **key** and **harmony mode** (start with `DiatonicThirds`
   in `C Ionian`).
6. Hit **Start**.

Play a note. You should hear the plugin sounding melody + harmony.

## Computer Keyboard input (QWERTY)

Lower octave — `Z` through `M` row:

| Key | Note (C3=48 default) |
|---|---|
| `Z X C V B N M` | C, D, E, F, G, A, B |
| `S D G H J` | C#, D#, F#, G#, A# |

Upper octave — `Q` through `U` row:

| Key | Note (C4=60) |
|---|---|
| `Q W E R T Y U I` | C, D, E, F, G, A, B, C5 |
| `2 3 5 6 7` | C#, D#, F#, G#, A# |

`-` / `+` shifts the base octave.

## Routing-mode specifics

### Channel-based (recommended)

- All 8 voices go to **one** IAC bus.
- Voice 0 = MIDI channel 2, voice 1 = channel 3, up to voice 6 = channel 8.
- One DAW track or one plugin instance listening on all channels plays
  the whole chord.
- Simplest cabling; one plugin instance is enough.

### Port-based

- Each voice goes to its own IAC bus on channel 1.
- Needs N separate IAC buses and N plugin instances (or N DAW tracks).
- Use when you want each voice on a different patch (e.g. bass on
  a sub synth, upper voices on a pad).

## Troubleshooting

**No sound.** Check: IAC Driver is Online in Audio MIDI Setup, plugin
MIDI input is set to the correct IAC bus, plugin audio output points at
your interface and your volume is up.

**Stuck notes after changing key/mode.** Fixed as of commit `996f9a5` —
Contrapunk now emits MIDI All-Notes-Off on every port when the engine
config mutates.

**Latency.** Contrapunk uses a 128-sample cpal buffer for guitar input
(~2.7ms at 48kHz). End-to-end latency is mostly the plugin's audio
buffer + your interface driver. Lower the plugin's buffer in its audio
settings for tighter response.

**Wrong channel.** If you're on `Channel-based` and only hear one voice,
your plugin is probably filtering to channel 1 only. Either set it to
receive on **all channels** or switch Contrapunk to `Port-based`.

**Harmony voice plays weird note out of key.** As of commit `462eef1`,
out-of-scale input is snapped to the nearest scale note for harmony
purposes (melody stays chromatic). If you want chromatic approach notes
to produce borrowed harmony, enable **Modal Interchange** in the center
column.

## Multi-plugin / more flexibility

For more complex rigs (different plugin per voice, MIDI re-mapping,
controller routing), load plugins in a host:

- **Gig Performer** (commercial) — the reference product for this
  workflow.
- **Blue Cat PatchWork** (cheap) — plugin chainer.
- **MainStage** (free with Logic) — Mac-only.
- **Ableton Live / Bitwig / Reaper** (DAWs) — heavier but full-featured.

Contrapunk → IAC → host → plugins. Same routing pattern, the host
replaces the single standalone plugin.

## Windows note

Windows doesn't have IAC. Use **loopMIDI** (free) to create virtual
MIDI buses. The rest of the setup is identical — pick the loopMIDI port
as Contrapunk output, same port as plugin input.
