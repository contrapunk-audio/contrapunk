# Guitar Audio Input Integration — Design Spec

## What

Wire the DSP guitar input pipeline into Contrapunk's harmony engine so users can play guitar and hear real-time harmonies. Guitar audio → pitch/string detection → MIDI events → harmony engine → harmonized output on MIDI ports.

## Why

This is Contrapunk's core promise as an improvisation companion. The DSP pipeline (93 tests, full MPE output) and the UI panel are built. The missing piece is connecting them.

## Design Decisions

| Decision | Choice |
|----------|--------|
| UI placement | Option B: input dropdown + dedicated Guitar Input panel in left column |
| Panel controls | Audio device + channel selector, 3 dials (latency/gain/string), 4 technique toggles, TUNE + CALIBRATE |
| Piano display | Unchanged — green input, orange harmony |
| Platform | Both Tauri + WASM from start (abstract audio capture layer) |
| MIDI output | Full MPE: per-string channels 2-7, CC74 brightness, pressure envelope, 14-bit pitch bend |
| Calibration | TUNE + CALIBRATE = noise floor + 3-pass tuner + capture profiles + set gain |
| Channel selection | User picks audio device + channel number (e.g., Audient iD14, channel 2) |

## Architecture

```
                    ┌─────────────────────────────┐
                    │   Audio Capture Layer        │
                    │   (cpal for Tauri,           │
                    │    Web Audio API for WASM)   │
                    └──────────┬──────────────────┘
                               │ f32 audio blocks
                               ▼
                    ┌─────────────────────────────┐
                    │   GuitarInput DSP Pipeline   │
                    │   (pure Rust, WASM-compat)   │
                    │   src/audio/guitar_input.rs  │
                    └──────────┬──────────────────┘
                               │ Vec<MidiEvent>
                               ▼
                    ┌─────────────────────────────┐
                    │   MidiEvent → MIDI Bytes     │
                    │   Convert to wmidi format    │
                    └──────────┬──────────────────┘
                               │ raw MIDI bytes
                               ▼
                    ┌─────────────────────────────┐
                    │   Same mpsc channel as       │
                    │   physical MIDI input        │
                    └──────────┬──────────────────┘
                               │
                               ▼
                    ┌─────────────────────────────┐
                    │   HarmonyEngine              │
                    │   (unchanged — processes     │
                    │    NoteOn/NoteOff as always)  │
                    └──────────┬──────────────────┘
                               │
                               ▼
                    ┌─────────────────────────────┐
                    │   OutputRouter → MIDI Ports   │
                    └─────────────────────────────┘
```

## What Exists Already

### Built
- `src/audio/guitar_input.rs` — full DSP pipeline (93 tests, MPE output)
- `examples/guitar_input_demo.rs` — working standalone demo with tuner + calibration + live detection
- `ui/src/lib/components/GuitarInputPanel.svelte` — UI panel with dials, toggles, calibrate button
- `ui/src/lib/stores/guitar.svelte.ts` — reactive store for guitar state
- `ui/src/lib/components/MidiDevices.svelte` — Guitar Audio in dropdown (VIRTUAL_GUITAR_AUDIO sentinel)
- Audio device + channel selector (being built by agent)

### Needs Building
1. **Audio capture abstraction** — trait/interface that both cpal (Tauri) and Web Audio API (WASM) implement
2. **GuitarInput → MIDI bytes converter** — translate MidiEvent enum to wmidi bytes
3. **Integration into routing thread** — when Guitar Audio is selected, spawn audio capture + DSP thread instead of connecting physical MIDI
4. **WASM audio bridge** — Web Audio API AudioWorklet feeding audio blocks to the WASM GuitarInput
5. **Calibration UI flow** — trigger tuner from the panel, show progress, save profile
6. **Live state updates** — feed detection results back to the UI store for display

## Audio Capture Abstraction

```rust
pub trait AudioCapture: Send {
    fn start(&mut self, callback: Box<dyn FnMut(&[f32]) + Send>) -> Result<(), String>;
    fn stop(&mut self) -> Result<(), String>;
    fn sample_rate(&self) -> usize;
    fn channel_count(&self) -> usize;
}

// Tauri implementation
pub struct CpalCapture { device_name: String, channel: usize, ... }

// WASM implementation
pub struct WebAudioCapture { context: AudioContext, worklet: AudioWorkletNode, ... }
```

## MidiEvent → MIDI Bytes

```rust
impl MidiEvent {
    pub fn to_midi_bytes(&self) -> Vec<u8> {
        match self {
            MidiEvent::NoteOn { channel, note, velocity } => {
                vec![0x90 | channel, *note, *velocity]
            }
            MidiEvent::NoteOff { channel, note, velocity } => {
                vec![0x80 | channel, *note, *velocity]
            }
            MidiEvent::PitchBend { channel, cents } => {
                let value = cents_to_midi_pitch_bend(*cents, 2);
                let lsb = (value & 0x7F) as u8;
                let msb = ((value >> 7) & 0x7F) as u8;
                vec![0xE0 | channel, lsb, msb]
            }
            MidiEvent::CC { channel, controller, value } => {
                vec![0xB0 | channel, *controller, *value]
            }
            MidiEvent::ChannelPressure { channel, pressure } => {
                vec![0xD0 | channel, *pressure]
            }
            _ => vec![], // VibratoStatus, MidiPitchBend are informational
        }
    }
}
```

## Routing Thread Integration (Tauri)

In `src-tauri/src/commands/engine.rs`, the `start_routing()` function currently connects a physical MIDI input. When Guitar Audio is selected:

```rust
if is_guitar_audio_input {
    // Instead of connect_input(), start audio capture
    let mut capture = CpalCapture::new(device_name, channel);
    let mut guitar = GuitarInput::new(config);

    if let Some(cal) = load_calibration() {
        guitar.set_calibration(cal);
    }

    capture.start(Box::new(move |audio_block| {
        let events = guitar.process_block(audio_block);
        for event in events {
            let bytes = event.to_midi_bytes();
            if !bytes.is_empty() {
                tx.send(bytes).ok();  // same channel as physical MIDI
            }
        }
    }));
} else {
    // Existing physical MIDI connection
    connect_input(input_idx, tx);
}
```

## WASM Audio Bridge

For the browser, use AudioWorklet to capture audio and feed it to the WASM GuitarInput:

```javascript
// In the adapter/wasm.ts
class GuitarAudioProcessor extends AudioWorkletProcessor {
    process(inputs) {
        const audio = inputs[0][selectedChannel];
        // Send to main thread via port
        this.port.postMessage(audio);
        return true;
    }
}

// Main thread receives audio, feeds to WASM GuitarInput
processor.port.onmessage = (e) => {
    const events = wasmGuitarInput.process_block(e.data);
    for (const event of events) {
        handleMidiEvent(event);  // inject into harmony engine
    }
};
```

## Calibration UI Flow

When user clicks "TUNE + CALIBRATE":
1. UI sends command to backend: `start_guitar_calibration(device, channel)`
2. Backend starts audio capture + calibration state machine
3. Backend sends progress events to UI: "Measuring noise floor...", "Pluck E2...", "E2 tuned (+2c)"
4. UI shows progress in the GuitarInputPanel (replace status line with calibration progress)
5. On complete: backend saves profile, UI shows "Calibrated" status
6. Guitar store updates: `calibrated = true`, gain/latency from calibration data

## UI State Flow

```
GuitarInputStore
  ├── Config → sent to backend on change
  │   latencyMs, gain, stringConfidence, techniques
  │
  ├── Device → sent to backend
  │   selectedDeviceId, selectedChannel
  │
  ├── Detection ← received from backend at ~30fps
  │   currentNote, currentString, currentFret, confidence, velocity
  │
  └── Calibration ← received from backend during calibration
      calibrating, calibrationStep, calibrationProgress
```

## Not in Scope
- Modulator/envelope architecture (separate phase, plan exists)
- Guitar fretboard visualization (piano stays as-is)
- Polyphonic detection (monophonic only)
- External controller routing (expression pedal, breath)
