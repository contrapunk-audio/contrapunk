# Route the Contrapunk desktop app into a standalone instrument

Contrapunk's desktop app can send generated MIDI to a standalone software instrument without a DAW. On macOS, connect the two applications with an IAC Driver bus. The equivalent Windows workflow uses loopMIDI.

Use this workflow when you want desktop-only routing, sound, or configuration. Logic users who want an in-DAW workflow should follow [DAW_SIDE_BY_SIDE.md](./DAW_SIDE_BY_SIDE.md) instead.

## Prerequisites

- macOS
- A standalone instrument application, such as Arturia Analog Lab V, Kontakt, Pianoteq, or another instrument with MIDI-input settings
- An audio interface or system audio output for monitoring the instrument

## 1. Enable an IAC bus

1. Open **Audio MIDI Setup** from `/Applications/Utilities`.
2. Choose **Window → Show MIDI Studio**.
3. Open **IAC Driver**.
4. Enable **Device is online**.
5. Keep the default **Bus 1**, or add more buses if different Contrapunk voices need different instruments.
6. Click **Apply**.

## 2. Configure the standalone instrument

1. Launch the instrument application.
2. Open its **Audio & MIDI** settings.
3. Enable **IAC Driver Bus 1** as a MIDI input.
4. Select the audio interface/output you want to monitor.
5. Load a sound.

## 3. Configure Contrapunk

1. Launch Contrapunk.
2. In **Input**, choose a MIDI controller, Computer Keyboard, or Guitar Audio.
3. Open **Output → Per-voice routing**.
4. Choose the number of voices.
5. Set each audible voice to **IAC Driver Bus 1**. Choose **Internal Synth** to keep a voice inside Contrapunk, or **Off** to silence it.
6. Choose an arrangement or configure Harmony/Companion manually.
7. Start routing and play.

The instrument should receive the melody and generated harmony. Contrapunk owns matching releases for every emitted note; use its panic/reset control if an external application is interrupted mid-phrase.

## Multiple instruments

Create one IAC bus per destination, then assign each voice to the desired bus in **Per-voice routing**. Enable each bus in its target instrument.

```text
Voice 1 → IAC Bus 1 → bass instrument
Voice 2 → IAC Bus 2 → pad instrument
Voice 3 → IAC Bus 3 → lead instrument
```

This is port-based routing: each selected Contrapunk output is a separate MIDI destination. If one instrument should play the full arrangement, route every voice to the same bus.

## Computer Keyboard input

| Row | Natural notes | Accidentals |
|---|---|---|
| Lower | `Z X C V B N M` | `S D G H J` |
| Upper | `Q W E R T Y U I` | `2 3 5 6 7` |

Use `-` and `+` to shift the base octave.

## Troubleshooting

**No MIDI activity**

- Confirm IAC Driver is online.
- Confirm the standalone instrument has the same bus enabled as an input.
- Confirm each required Contrapunk voice is assigned to that bus rather than Internal Synth or Off.
- Start/enable routing and monitor the instrument.

**Only one voice plays**

- Check every voice in **Per-voice routing**.
- Set the instrument to receive all channels if it applies a channel filter.

**Stuck note after a crash or disconnected device**

- Use Contrapunk's panic/reset control or restart routing.
- If necessary, send MIDI CC 123 (All Notes Off) to the instrument.

**High guitar latency**

- Use a dry, clean, monophonic input.
- Reduce the audio-interface buffer only if the system remains stable.
- Avoid software monitoring the same guitar through multiple paths.

**Feedback or doubled sound**

- If using the standalone instrument for sound, route Contrapunk voices to IAC and disable the same voices' Internal Synth output.
- Check that the instrument's audio is not routed back into Contrapunk's guitar input.

## Windows

Install [loopMIDI](https://www.tobias-erichsen.de/software/loopmidi.html), create one or more virtual ports, and use them in place of IAC buses. The per-voice routing steps are otherwise the same.
