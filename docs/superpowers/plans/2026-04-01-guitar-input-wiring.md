# Guitar Input Wiring Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Wire the DSP guitar input pipeline into Contrapunk's Tauri routing thread so guitar audio feeds through the harmony engine and outputs on MIDI ports.

**Architecture:** Guitar audio captured via cpal on a background thread, processed by `GuitarInput::process_block()` into `MidiEvent`s, converted to MIDI bytes, sent via mpsc channel to the existing routing thread where `process_midi_message()` feeds them into `HarmonyEngine`. No changes to the harmony engine or output router.

**Tech Stack:** Rust, cpal (audio capture), Tauri 2, mpsc channels, contrapunk::audio::guitar_input

---

## File Structure

### Create
- `src-tauri/src/guitar_bridge.rs` — Bridges audio capture → GuitarInput DSP → mpsc MIDI bytes
- `src-tauri/src/commands/guitar.rs` — Tauri commands for guitar input control

### Modify
- `src-tauri/src/commands/engine.rs` — Add guitar input channel to routing thread
- `src-tauri/src/state.rs` — Add guitar config to AppState
- `src-tauri/src/main.rs` — Register new commands
- `ui/src/lib/adapter/tauri.ts` — Add guitar-specific IPC calls
- `ui/src/lib/adapter/types.ts` — Add guitar methods to adapter interface

---

## Task 1: MidiEvent → MIDI Bytes Converter

**Files:**
- Modify: `src/audio/guitar_input.rs`

- [ ] **Step 1: Add `to_midi_bytes()` method to MidiEvent**

```rust
impl MidiEvent {
    /// Convert to raw MIDI bytes for sending through mpsc channel.
    /// Returns empty vec for informational-only events.
    pub fn to_midi_bytes(&self, pitch_bend_range: u8) -> Vec<u8> {
        match self {
            MidiEvent::NoteOn { channel, note, velocity } => {
                vec![0x90 | (channel & 0x0F), *note & 0x7F, *velocity & 0x7F]
            }
            MidiEvent::NoteOff { channel, note, velocity } => {
                vec![0x80 | (channel & 0x0F), *note & 0x7F, *velocity & 0x7F]
            }
            MidiEvent::PitchBend { channel, cents } => {
                let value = cents_to_midi_pitch_bend(*cents, pitch_bend_range);
                let lsb = (value & 0x7F) as u8;
                let msb = ((value >> 7) & 0x7F) as u8;
                vec![0xE0 | (channel & 0x0F), lsb, msb]
            }
            MidiEvent::MidiPitchBend { channel, value } => {
                let lsb = (*value & 0x7F) as u8;
                let msb = ((*value >> 7) & 0x7F) as u8;
                vec![0xE0 | (channel & 0x0F), lsb, msb]
            }
            MidiEvent::CC { channel, controller, value } => {
                vec![0xB0 | (channel & 0x0F), *controller & 0x7F, *value & 0x7F]
            }
            MidiEvent::ChannelPressure { channel, pressure } => {
                vec![0xD0 | (channel & 0x0F), *pressure & 0x7F]
            }
            MidiEvent::VibratoStatus { .. } => {
                vec![] // Informational only, no MIDI bytes
            }
        }
    }
}
```

- [ ] **Step 2: Add tests**

```rust
#[test]
fn midi_event_to_bytes_note_on() {
    let event = MidiEvent::NoteOn { channel: 1, note: 60, velocity: 100 };
    assert_eq!(event.to_midi_bytes(2), vec![0x91, 60, 100]);
}

#[test]
fn midi_event_to_bytes_note_off() {
    let event = MidiEvent::NoteOff { channel: 0, note: 64, velocity: 0 };
    assert_eq!(event.to_midi_bytes(2), vec![0x80, 64, 0]);
}

#[test]
fn midi_event_to_bytes_pitch_bend_center() {
    let event = MidiEvent::MidiPitchBend { channel: 2, value: 8192 };
    let bytes = event.to_midi_bytes(2);
    assert_eq!(bytes[0], 0xE2);
    assert_eq!(bytes.len(), 3);
}

#[test]
fn midi_event_to_bytes_cc74() {
    let event = MidiEvent::CC { channel: 3, controller: 74, value: 100 };
    assert_eq!(event.to_midi_bytes(2), vec![0xB3, 74, 100]);
}

#[test]
fn midi_event_to_bytes_vibrato_is_empty() {
    let event = MidiEvent::VibratoStatus { active: true, rate_hz: 5.0, depth_cents: 30.0 };
    assert!(event.to_midi_bytes(2).is_empty());
}
```

- [ ] **Step 3: Verify**

Run: `cargo test --lib audio::guitar_input::tests::midi_event_to_bytes`
Expected: 5 new tests pass.

- [ ] **Step 4: Commit**

```bash
git add src/audio/guitar_input.rs
git commit -m "feat: add MidiEvent::to_midi_bytes() for MIDI channel output"
```

---

## Task 2: Guitar Bridge Module

**Files:**
- Create: `src-tauri/src/guitar_bridge.rs`

- [ ] **Step 1: Create the guitar bridge**

This module spawns a cpal audio capture thread and feeds audio blocks through `GuitarInput`, converting events to MIDI bytes and sending them via an mpsc channel.

```rust
//! Bridge between cpal audio capture and the MIDI routing thread.
//!
//! Spawns an audio capture thread that feeds audio blocks through
//! GuitarInput::process_block(), converts MidiEvent to MIDI bytes,
//! and sends them via an mpsc::Sender<Vec<u8>>.

use contrapunk::audio::guitar_input::{GuitarInput, GuitarInputConfig, GuitarCalibration, MidiEvent};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::{mpsc, Arc, Mutex};

pub struct GuitarBridge {
    stream: Option<cpal::Stream>,
    pipeline: Arc<Mutex<GuitarInput>>,
}

impl GuitarBridge {
    /// Create a new guitar bridge.
    ///
    /// `device_name`: audio device name (e.g., "Audient iD14"), or empty for default
    /// `channel`: audio channel index (0-based)
    /// `config`: GuitarInput configuration
    /// `tx`: mpsc sender for MIDI bytes (same channel type as physical MIDI input)
    pub fn new(
        device_name: &str,
        channel: usize,
        config: GuitarInputConfig,
        tx: mpsc::Sender<Vec<u8>>,
    ) -> Result<Self, String> {
        let host = cpal::default_host();

        // Find device by name, or use default
        let device = if device_name.is_empty() {
            host.default_input_device()
                .ok_or("No default audio input device")?
        } else {
            host.input_devices()
                .map_err(|e| format!("Failed to enumerate audio devices: {}", e))?
                .find(|d| d.name().unwrap_or_default().contains(device_name))
                .ok_or_else(|| format!("Audio device '{}' not found", device_name))?
        };

        let supported_config = device.default_input_config()
            .map_err(|e| format!("No input config: {}", e))?;

        let sample_rate = supported_config.sample_rate().0 as usize;
        let channels = supported_config.channels() as usize;
        let pb_range = config.pitch_bend_range;

        let mut actual_config = config;
        actual_config.sample_rate = sample_rate;

        let pipeline = Arc::new(Mutex::new(GuitarInput::new(actual_config)));

        let pipeline_c = Arc::clone(&pipeline);
        let stream_config: cpal::StreamConfig = supported_config.into();

        let stream = device.build_input_stream(
            &stream_config,
            move |data: &[f32], _: &cpal::InputCallbackInfo| {
                // Extract target channel
                let mono: Vec<f32> = data.chunks(channels)
                    .map(|frame| frame.get(channel).copied().unwrap_or(0.0))
                    .collect();

                // Process through DSP pipeline
                let events = {
                    let mut pipe = pipeline_c.lock().unwrap();
                    pipe.process_block(&mono)
                };

                // Convert events to MIDI bytes and send
                for event in events {
                    let bytes = event.to_midi_bytes(pb_range);
                    if !bytes.is_empty() {
                        let _ = tx.send(bytes);
                    }
                }
            },
            |err| eprintln!("Guitar audio error: {}", err),
            None,
        ).map_err(|e| format!("Failed to build audio stream: {}", e))?;

        Ok(Self {
            stream: Some(stream),
            pipeline,
        })
    }

    /// Start audio capture.
    pub fn start(&self) -> Result<(), String> {
        if let Some(ref stream) = self.stream {
            stream.play().map_err(|e| format!("Failed to start audio: {}", e))
        } else {
            Err("No audio stream".into())
        }
    }

    /// Stop audio capture.
    pub fn stop(&mut self) {
        self.stream = None;
    }

    /// Set calibration data on the pipeline.
    pub fn set_calibration(&self, cal: GuitarCalibration) {
        let mut pipe = self.pipeline.lock().unwrap();
        pipe.set_calibration(cal);
    }

    /// Get a clone of the pipeline for status queries.
    pub fn pipeline(&self) -> Arc<Mutex<GuitarInput>> {
        Arc::clone(&self.pipeline)
    }
}
```

- [ ] **Step 2: Add module to Tauri**

In `src-tauri/src/main.rs` (or `lib.rs`), add:
```rust
mod guitar_bridge;
```

- [ ] **Step 3: Verify compilation**

Run: `cd src-tauri && cargo check`
Expected: Compiles without errors.

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/guitar_bridge.rs src-tauri/src/main.rs
git commit -m "feat: add GuitarBridge — cpal audio capture → DSP → MIDI bytes"
```

---

## Task 3: Integrate into Routing Thread

**Files:**
- Modify: `src-tauri/src/commands/engine.rs`
- Modify: `src-tauri/src/state.rs`

- [ ] **Step 1: Add guitar state to AppState**

In `src-tauri/src/state.rs`, add:
```rust
use contrapunk::audio::guitar_input::GuitarInputConfig;

pub struct AppState {
    // ... existing fields ...
    pub guitar_config: Mutex<Option<GuitarInputConfig>>,
    pub guitar_device: Mutex<String>,
    pub guitar_channel: Mutex<usize>,
}
```

Update `Default` impl to include the new fields.

- [ ] **Step 2: Modify `start_routing()` to detect Guitar Audio input**

The Guitar Audio virtual input uses a sentinel value. When `input_idx` matches the sentinel, spawn the guitar bridge instead of connecting physical MIDI.

In `start_routing()`:
```rust
const GUITAR_AUDIO_SENTINEL: usize = usize::MAX - 2;

// In the routing thread spawn, before connect_input:
let is_guitar = input_idx == GUITAR_AUDIO_SENTINEL;
```

- [ ] **Step 3: Modify `run_tauri_router()` to support guitar input**

Add a parameter for the guitar bridge and create a second receive path:

```rust
// At the start of run_tauri_router:
let (tx, rx) = mpsc::channel::<Vec<u8>>();

let _midi_conn;
let _guitar_bridge;

if is_guitar {
    // Guitar Audio mode: spawn cpal capture → DSP → same tx channel
    let device_name = { state.guitar_device.lock().unwrap().clone() };
    let channel = { *state.guitar_channel.lock().unwrap() };
    let config = {
        state.guitar_config.lock().unwrap()
            .clone()
            .unwrap_or_default()
    };

    let bridge = GuitarBridge::new(&device_name, channel, config, tx.clone())
        .map_err(|e| format!("Guitar bridge error: {}", e))?;
    bridge.start()?;
    _guitar_bridge = Some(bridge);
    _midi_conn = None;
} else {
    // Physical MIDI mode: existing behavior
    _midi_conn = Some(connect_input(input_port, tx)?);
    _guitar_bridge = None;
}

// The rest of the routing loop is UNCHANGED
// rx.recv_timeout(5ms) receives from EITHER physical MIDI or guitar DSP
```

The key insight: both paths send `Vec<u8>` MIDI bytes through the SAME `tx` channel. The routing loop doesn't need to know the source.

- [ ] **Step 4: Verify compilation**

Run: `cd src-tauri && cargo check`
Expected: Compiles.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/commands/engine.rs src-tauri/src/state.rs
git commit -m "feat: integrate guitar bridge into routing thread"
```

---

## Task 4: Tauri Commands for Guitar Control

**Files:**
- Create: `src-tauri/src/commands/guitar.rs`
- Modify: `src-tauri/src/main.rs`

- [ ] **Step 1: Create guitar commands**

```rust
use tauri::State;
use crate::state::AppState;
use contrapunk::audio::guitar_input::GuitarInputConfig;

#[tauri::command]
pub fn set_guitar_device(
    device_name: String,
    channel: usize,
    state: State<AppState>,
) -> Result<(), String> {
    *state.guitar_device.lock().unwrap() = device_name;
    *state.guitar_channel.lock().unwrap() = channel;
    Ok(())
}

#[tauri::command]
pub fn set_guitar_config(
    latency_ms: f32,
    gain: f32,
    string_confidence: f32,
    bends: bool,
    legato: bool,
    slides: bool,
    vibrato: bool,
    state: State<AppState>,
) -> Result<(), String> {
    let sample_rate = 48000; // default, updated by bridge on start
    let config = GuitarInputConfig {
        buffer_size: GuitarInputConfig::buffer_size_for_latency(latency_ms, sample_rate),
        sample_rate,
        onset_threshold: 0.015,
        string_confidence_min: string_confidence,
        bends_enabled: bends,
        legato_enabled: legato,
        slides_enabled: slides,
        vibrato_detection: vibrato,
        vibrato_passthrough: true,
        filter_enabled: false,
        min_clarity: 0.40,
        cooldown_samples: sample_rate / 10,
        n_harmonics: 6,
        input_gain: gain,
        flux_threshold: 0.5,
        per_string_channels: true,
        pitch_bend_range: 2,
        pressure_enabled: true,
        pressure_hold: 0.3,
        brightness_enabled: true,
        ..GuitarInputConfig::default()
    };
    *state.guitar_config.lock().unwrap() = Some(config);
    Ok(())
}

#[tauri::command]
pub fn list_audio_devices() -> Result<Vec<String>, String> {
    let host = cpal::default_host();
    let devices = host.input_devices()
        .map_err(|e| format!("Failed to enumerate: {}", e))?;
    Ok(devices.map(|d| d.name().unwrap_or_default()).collect())
}
```

- [ ] **Step 2: Register commands**

In `src-tauri/src/main.rs`, add to the invoke_handler:
```rust
commands::guitar::set_guitar_device,
commands::guitar::set_guitar_config,
commands::guitar::list_audio_devices,
```

Add `pub mod guitar;` to `src-tauri/src/commands/mod.rs`.

- [ ] **Step 3: Verify compilation**

Run: `cd src-tauri && cargo check`

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/commands/guitar.rs src-tauri/src/commands/mod.rs src-tauri/src/main.rs
git commit -m "feat: add Tauri commands for guitar device/config/enumeration"
```

---

## Task 5: Wire UI Adapter to Backend

**Files:**
- Modify: `ui/src/lib/adapter/types.ts`
- Modify: `ui/src/lib/adapter/tauri.ts`
- Modify: `ui/src/lib/stores/guitar.svelte.ts`
- Modify: `ui/src/lib/components/GuitarInputPanel.svelte`

- [ ] **Step 1: Add guitar methods to adapter interface**

In `types.ts`:
```typescript
listAudioDevices(): Promise<string[]>;
setGuitarDevice(deviceName: string, channel: number): Promise<void>;
setGuitarConfig(config: GuitarConfig): Promise<void>;
```

- [ ] **Step 2: Implement in Tauri adapter**

In `tauri.ts`:
```typescript
async listAudioDevices(): Promise<string[]> {
    return invoke('list_audio_devices');
}

async setGuitarDevice(deviceName: string, channel: number): Promise<void> {
    await invoke('set_guitar_device', { deviceName, channel });
}

async setGuitarConfig(config: GuitarConfig): Promise<void> {
    await invoke('set_guitar_config', {
        latencyMs: config.latencyMs,
        gain: config.gain,
        stringConfidence: config.stringConfidence,
        bends: config.bends,
        legato: config.legato,
        slides: config.slides,
        vibrato: config.vibrato,
    });
}
```

- [ ] **Step 3: Wire guitar store to adapter**

In `guitar.svelte.ts`, add methods that call the adapter:
```typescript
async syncConfig() {
    const adapter = getAdapter();
    await adapter.setGuitarConfig({
        latencyMs: this.latencyMs,
        gain: this.gain,
        stringConfidence: this.stringConfidence,
        bends: this.bendsEnabled,
        legato: this.legatoEnabled,
        slides: this.slidesEnabled,
        vibrato: this.vibratoEnabled,
    });
}

async syncDevice() {
    const adapter = getAdapter();
    await adapter.setGuitarDevice(this.selectedDeviceId, this.selectedChannel);
}
```

- [ ] **Step 4: Wire panel controls to sync**

In `GuitarInputPanel.svelte`, call `guitar.syncConfig()` when toggles change and `guitar.syncDevice()` when device/channel changes.

- [ ] **Step 5: Verify build**

Run: `cd ui && npm run build`

- [ ] **Step 6: Commit**

```bash
git add ui/src/lib/adapter/types.ts ui/src/lib/adapter/tauri.ts ui/src/lib/stores/guitar.svelte.ts ui/src/lib/components/GuitarInputPanel.svelte
git commit -m "feat: wire guitar UI controls to Tauri backend commands"
```

---

## Task 6: End-to-End Test

- [ ] **Step 1: Build the Tauri app**

```bash
cd src-tauri && cargo build
```

- [ ] **Step 2: Test the flow**

1. Run the Tauri dev server: `cd ui && npm run tauri dev`
2. In the UI, select "Guitar Audio" from the input dropdown
3. Select your Audient iD14 and channel
4. Select MIDI output ports
5. Click Start
6. Pluck your guitar
7. Verify: detected notes appear on the piano (green), harmonies appear (orange), MIDI output ports receive notes

- [ ] **Step 3: Commit any fixes**

```bash
git add -A && git commit -m "fix: end-to-end guitar input wiring fixes"
```
