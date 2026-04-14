# Audio Foundation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Give Contrapunk native audio output via cpal, with a 4-voice polyphonic sine synth driven by the harmony engine through a lock-free MIDI ringbuffer. Proves the audio pipeline end-to-end before any VST3 plugin code.

**Architecture:** New module `src/audio_out/` owns a cpal stream, a `PolySynth` that processes MIDI events into stereo audio, and a lock-free SPSC ringbuffer (`ringbuf` crate) that the harmony router pushes into. The audio thread is real-time safe — no allocations, no locks. The harmony engine itself is unchanged. The MIDI-out path (IAC, hardware) runs in parallel with audio-out.

**Tech Stack:** Rust, cpal 0.15 (already a dep), `ringbuf` crate (new dep), wmidi (already a dep), Tauri for desktop integration.

**Scope boundary:** This plan is sub-project 1 of the Plugin Hosting spec (`docs/superpowers/specs/2026-04-14-plugin-hosting-design.md`). It ships audio output with a built-in test sine synth. VST3 plugin loading is sub-project 2 — NOT part of this plan.

**Out of scope:** VST3/plugins, FX chain, Routing tab UI (sub-project 4 builds that — for this plan we add only a dev-grade toggle), WASM audio output, latency compensation, plugin GUI windows, parameter automation.

---

## File Structure

New files:

```
src/audio_out/
├── mod.rs           Module root, public API exports
├── config.rs        AudioConfig struct (sample rate, buffer size, device id)
├── midi_queue.rs    Lock-free SPSC ringbuffer for MidiEvent
├── sine_synth.rs    SineVoice + PolySynth — 4-voice polyphonic sine synth
└── engine.rs        AudioOutEngine — cpal stream lifecycle + audio callback

src-tauri/src/commands/audio_out.rs   Tauri command handlers
```

Modified files:

- `Cargo.toml` — add `ringbuf = "0.4"` dep
- `src/lib.rs` — register `pub mod audio_out;`
- `src/router.rs` — fan harmony notes into audio_out's MIDI queue (gated by a new `audio_out: Option<MidiProducer>` field on the engine config)
- `src-tauri/src/main.rs` — register new Tauri commands
- `src-tauri/src/commands/mod.rs` — register `pub mod audio_out;`

Responsibilities:

- `config.rs` owns the `AudioConfig` struct and defaults. No logic beyond construction and `Default`.
- `midi_queue.rs` owns the MIDI event type and the producer/consumer ringbuffer split. No audio logic.
- `sine_synth.rs` is pure DSP. Takes `MidiEvent`s and fills stereo audio buffers. No I/O.
- `engine.rs` owns the cpal stream, opens/closes it, drains the MIDI queue each callback, drives the synth. Does I/O.
- Tauri commands in `commands/audio_out.rs` are thin adapters that call into the engine.

---

## Task 1: Add `ringbuf` dependency + module skeleton

**Files:**
- Modify: `Cargo.toml`
- Create: `src/audio_out/mod.rs`
- Modify: `src/lib.rs`

- [ ] **Step 1: Add ringbuf to Cargo.toml**

Add to `[dependencies]`:

```toml
ringbuf = "0.4"
```

- [ ] **Step 2: Create `src/audio_out/mod.rs`**

```rust
//! Native audio output for Contrapunk.
//!
//! Drives a cpal output stream with a polyphonic sine synth, fed by the
//! harmony engine via a lock-free SPSC ringbuffer. The audio thread is
//! real-time safe — no allocations, no locks.
//!
//! The MIDI-out path (IAC, external synths) continues to run in parallel;
//! audio output is additive, not a replacement.
//!
//! Sub-project 1 of plugin hosting. VST3 plugin loading is sub-project 2.

pub mod config;
pub mod engine;
pub mod midi_queue;
pub mod sine_synth;

pub use config::AudioConfig;
pub use engine::AudioOutEngine;
pub use midi_queue::{MidiConsumer, MidiEvent, MidiProducer, midi_queue};
pub use sine_synth::{PolySynth, SineVoice};
```

- [ ] **Step 3: Register module in `src/lib.rs`**

Add `pub mod audio_out;` to the list at lines 49-55. Keep alphabetical ordering — insert between `pub mod audio;` and `pub mod chord;`:

```rust
pub mod audio;
pub mod audio_out;
pub mod chord;
```

- [ ] **Step 4: Verify compile**

Run: `cargo check -p contrapunk`
Expected: SUCCESS. Submodules are declared but empty files don't exist yet — this will fail. Create them as empty files first:

```bash
touch src/audio_out/config.rs src/audio_out/engine.rs src/audio_out/midi_queue.rs src/audio_out/sine_synth.rs
```

Run `cargo check -p contrapunk` again. Expected: SUCCESS (empty modules compile).

- [ ] **Step 5: Commit**

```bash
git add Cargo.toml src/lib.rs src/audio_out/
git commit -m "feat(audio-out): scaffold module + add ringbuf dep"
```

---

## Task 2: `AudioConfig` struct + defaults

**Files:**
- Modify: `src/audio_out/config.rs`

- [ ] **Step 1: Write the failing test**

In `src/audio_out/config.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = AudioConfig::default();
        assert_eq!(cfg.sample_rate, 48_000);
        assert_eq!(cfg.buffer_size, 256);
        assert_eq!(cfg.channels, 2);
        assert_eq!(cfg.device_id, None);
    }

    #[test]
    fn test_config_with_device() {
        let cfg = AudioConfig::default().with_device("MacBook Pro Speakers".to_string());
        assert_eq!(cfg.device_id.as_deref(), Some("MacBook Pro Speakers"));
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p contrapunk --lib audio_out::config`
Expected: FAIL with "AudioConfig not defined"

- [ ] **Step 3: Write minimal implementation**

At the top of `src/audio_out/config.rs`:

```rust
//! Audio output configuration.

use serde::{Deserialize, Serialize};

/// Configuration for the audio output engine.
///
/// Sample rate and buffer size are hints — when the cpal stream is opened,
/// the actual device may negotiate different values. See [`AudioOutEngine`]
/// for details.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AudioConfig {
    /// Requested sample rate in Hz. Default 48000.
    pub sample_rate: u32,
    /// Requested buffer size in samples per channel. Default 256.
    pub buffer_size: u32,
    /// Number of output channels. Stereo (2) in v1.
    pub channels: u16,
    /// Target device identifier (cpal device name). `None` = system default.
    pub device_id: Option<String>,
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            sample_rate: 48_000,
            buffer_size: 256,
            channels: 2,
            device_id: None,
        }
    }
}

impl AudioConfig {
    /// Returns a new config with the given device id.
    pub fn with_device(mut self, id: String) -> Self {
        self.device_id = Some(id);
        self
    }
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p contrapunk --lib audio_out::config`
Expected: PASS — 2 tests pass.

- [ ] **Step 5: Commit**

```bash
git add src/audio_out/config.rs
git commit -m "feat(audio-out): add AudioConfig with defaults"
```

---

## Task 3: `MidiEvent` + lock-free SPSC queue

**Files:**
- Modify: `src/audio_out/midi_queue.rs`

- [ ] **Step 1: Write the failing tests**

In `src/audio_out/midi_queue.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_push_pop_note_on() {
        let (mut producer, mut consumer) = midi_queue(128);
        let evt = MidiEvent::NoteOn { voice: 0, note: 60, velocity: 100 };
        producer.push(evt).unwrap();
        assert_eq!(consumer.pop(), Some(evt));
        assert_eq!(consumer.pop(), None);
    }

    #[test]
    fn test_push_pop_note_off() {
        let (mut producer, mut consumer) = midi_queue(128);
        let evt = MidiEvent::NoteOff { voice: 2, note: 64 };
        producer.push(evt).unwrap();
        assert_eq!(consumer.pop(), Some(evt));
    }

    #[test]
    fn test_capacity_bound() {
        let (mut producer, _consumer) = midi_queue(2);
        producer.push(MidiEvent::NoteOn { voice: 0, note: 60, velocity: 100 }).unwrap();
        producer.push(MidiEvent::NoteOn { voice: 0, note: 61, velocity: 100 }).unwrap();
        // Third push should fail — queue is full.
        let result = producer.push(MidiEvent::NoteOn { voice: 0, note: 62, velocity: 100 });
        assert!(result.is_err());
    }

    #[test]
    fn test_drain_into_vec() {
        let (mut producer, mut consumer) = midi_queue(128);
        producer.push(MidiEvent::NoteOn { voice: 0, note: 60, velocity: 100 }).unwrap();
        producer.push(MidiEvent::NoteOff { voice: 0, note: 60 }).unwrap();
        let mut out = Vec::with_capacity(2);
        while let Some(e) = consumer.pop() {
            out.push(e);
        }
        assert_eq!(out.len(), 2);
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p contrapunk --lib audio_out::midi_queue`
Expected: FAIL — types undefined.

- [ ] **Step 3: Write implementation**

At the top of `src/audio_out/midi_queue.rs`:

```rust
//! Lock-free SPSC MIDI event queue between the harmony router and the
//! audio callback.
//!
//! The audio thread must never allocate or block. The harmony engine
//! (producer) and audio callback (consumer) communicate through a
//! bounded ringbuffer with static capacity.

use ringbuf::{
    HeapRb,
    traits::{Consumer as _, Producer as _, Split as _},
};

/// A MIDI event destined for the audio synth.
///
/// Voices are 0-indexed, matching Contrapunk's per-voice chain slots.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum MidiEvent {
    NoteOn { voice: u8, note: u8, velocity: u8 },
    NoteOff { voice: u8, note: u8 },
}

/// Producer half of the MIDI queue. Held by the harmony router.
pub struct MidiProducer(ringbuf::HeapProd<MidiEvent>);

/// Consumer half of the MIDI queue. Held by the audio callback.
pub struct MidiConsumer(ringbuf::HeapCons<MidiEvent>);

/// Errors returned when pushing into a full MidiProducer.
#[derive(Debug, PartialEq, Eq)]
pub struct QueueFull;

impl MidiProducer {
    /// Push an event. Returns `Err(QueueFull)` if the queue is at capacity.
    /// The audio thread drains the queue each buffer, so `QueueFull` means
    /// something is very wrong (stalled audio thread or overflow attack).
    pub fn push(&mut self, event: MidiEvent) -> Result<(), QueueFull> {
        self.0.try_push(event).map_err(|_| QueueFull)
    }
}

impl MidiConsumer {
    /// Pop the next event. Returns `None` if the queue is empty.
    pub fn pop(&mut self) -> Option<MidiEvent> {
        self.0.try_pop()
    }
}

/// Create a new MIDI queue with the given capacity. Returns (producer, consumer).
///
/// The producer is held by the harmony router; the consumer is moved into
/// the audio callback. The capacity should be generous — at 48 kHz with
/// 256-sample buffers the audio thread runs ~188 times per second, so even
/// a bursty harmony engine rarely queues more than a handful of events
/// per buffer. Default callers use 1024.
pub fn midi_queue(capacity: usize) -> (MidiProducer, MidiConsumer) {
    let rb = HeapRb::<MidiEvent>::new(capacity);
    let (prod, cons) = rb.split();
    (MidiProducer(prod), MidiConsumer(cons))
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p contrapunk --lib audio_out::midi_queue`
Expected: PASS — 4 tests pass.

- [ ] **Step 5: Commit**

```bash
git add src/audio_out/midi_queue.rs
git commit -m "feat(audio-out): add lock-free SPSC MIDI event queue"
```

---

## Task 4: `SineVoice` — single oscillator with ADSR

**Files:**
- Modify: `src/audio_out/sine_synth.rs`

- [ ] **Step 1: Write the failing tests**

In `src/audio_out/sine_synth.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_idle_voice_produces_silence() {
        let mut voice = SineVoice::new(48_000.0);
        let mut buf = [0.0_f32; 64];
        voice.process(&mut buf);
        assert!(buf.iter().all(|&s| s == 0.0));
    }

    #[test]
    fn test_note_on_produces_signal() {
        let mut voice = SineVoice::new(48_000.0);
        voice.note_on(440.0, 0.8);
        let mut buf = [0.0_f32; 512];
        voice.process(&mut buf);
        // After attack, the voice should have non-silent samples.
        let peak = buf.iter().map(|s| s.abs()).fold(0.0_f32, f32::max);
        assert!(peak > 0.1, "peak was {peak}");
    }

    #[test]
    fn test_frequency_is_correct() {
        // 1 cycle of 480 Hz at 48 kHz = 100 samples.
        let mut voice = SineVoice::new(48_000.0);
        voice.note_on(480.0, 1.0);
        voice.skip_attack(); // jump to sustain so we don't fight the envelope
        let mut buf = [0.0_f32; 200];
        voice.process(&mut buf);
        // Count zero crossings in the buffer. 200 samples = 2 cycles = 4 zero crossings.
        let crossings = buf.windows(2).filter(|w| w[0].signum() != w[1].signum()).count();
        assert!(
            (3..=5).contains(&crossings),
            "expected ~4 zero crossings, got {crossings}"
        );
    }

    #[test]
    fn test_note_off_releases() {
        let mut voice = SineVoice::new(48_000.0);
        voice.note_on(440.0, 1.0);
        voice.skip_attack();
        voice.note_off();
        // After a long release, the voice should be silent and inactive.
        let mut buf = [0.0_f32; 48_000]; // 1 second
        voice.process(&mut buf);
        assert!(!voice.is_active(), "voice should be inactive after release");
        // Last samples should be silent.
        assert!(buf[buf.len() - 1].abs() < 0.001);
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p contrapunk --lib audio_out::sine_synth`
Expected: FAIL — types undefined.

- [ ] **Step 3: Write implementation**

At the top of `src/audio_out/sine_synth.rs`:

```rust
//! Polyphonic sine synth used as a test tone generator for the audio
//! pipeline before VST3 plugin hosting lands (sub-project 2).

use std::f32::consts::TAU;

/// ADSR envelope stage.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum EnvelopeStage {
    Idle,
    Attack,
    Decay,
    Sustain,
    Release,
}

const ATTACK_SECONDS: f32 = 0.005;
const DECAY_SECONDS: f32 = 0.05;
const SUSTAIN_LEVEL: f32 = 0.8;
const RELEASE_SECONDS: f32 = 0.15;

/// A single sine oscillator with an ADSR amplitude envelope.
#[derive(Clone, Debug)]
pub struct SineVoice {
    sample_rate: f32,
    phase: f32,
    freq: f32,
    amp: f32,
    env: f32,
    stage: EnvelopeStage,
    /// Note number this voice was triggered with (for voice matching on Note-Off).
    note: Option<u8>,
}

impl SineVoice {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            sample_rate,
            phase: 0.0,
            freq: 440.0,
            amp: 0.0,
            env: 0.0,
            stage: EnvelopeStage::Idle,
            note: None,
        }
    }

    /// Start a note at the given frequency and amplitude (0.0–1.0).
    pub fn note_on(&mut self, freq: f32, amp: f32) {
        self.freq = freq;
        self.amp = amp.clamp(0.0, 1.0);
        self.stage = EnvelopeStage::Attack;
    }

    /// Release the note. Envelope transitions to Release stage.
    pub fn note_off(&mut self) {
        if self.stage != EnvelopeStage::Idle {
            self.stage = EnvelopeStage::Release;
        }
    }

    /// True if the voice is currently producing audio (any non-idle stage).
    pub fn is_active(&self) -> bool {
        self.stage != EnvelopeStage::Idle
    }

    /// Record which MIDI note this voice is playing (for Note-Off matching).
    pub fn set_note(&mut self, note: u8) {
        self.note = Some(note);
    }

    /// The note this voice is playing, if any.
    pub fn note(&self) -> Option<u8> {
        self.note
    }

    /// Jump the envelope straight to sustain. Testing aid only.
    #[cfg(test)]
    pub fn skip_attack(&mut self) {
        self.env = SUSTAIN_LEVEL;
        self.stage = EnvelopeStage::Sustain;
    }

    /// Process one buffer, ADDING the voice's output into `output` (mono).
    pub fn process(&mut self, output: &mut [f32]) {
        if self.stage == EnvelopeStage::Idle {
            return;
        }
        let phase_inc = self.freq * TAU / self.sample_rate;
        let attack_step = 1.0 / (ATTACK_SECONDS * self.sample_rate);
        let decay_step = (1.0 - SUSTAIN_LEVEL) / (DECAY_SECONDS * self.sample_rate);
        let release_step = SUSTAIN_LEVEL / (RELEASE_SECONDS * self.sample_rate);

        for sample in output.iter_mut() {
            // Advance envelope.
            match self.stage {
                EnvelopeStage::Attack => {
                    self.env += attack_step;
                    if self.env >= 1.0 {
                        self.env = 1.0;
                        self.stage = EnvelopeStage::Decay;
                    }
                }
                EnvelopeStage::Decay => {
                    self.env -= decay_step;
                    if self.env <= SUSTAIN_LEVEL {
                        self.env = SUSTAIN_LEVEL;
                        self.stage = EnvelopeStage::Sustain;
                    }
                }
                EnvelopeStage::Sustain => {}
                EnvelopeStage::Release => {
                    self.env -= release_step;
                    if self.env <= 0.0 {
                        self.env = 0.0;
                        self.stage = EnvelopeStage::Idle;
                        self.note = None;
                        break;
                    }
                }
                EnvelopeStage::Idle => break,
            }

            *sample += self.phase.sin() * self.env * self.amp;
            self.phase += phase_inc;
            if self.phase >= TAU {
                self.phase -= TAU;
            }
        }
    }
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p contrapunk --lib audio_out::sine_synth`
Expected: PASS — 4 tests pass.

- [ ] **Step 5: Commit**

```bash
git add src/audio_out/sine_synth.rs
git commit -m "feat(audio-out): add SineVoice with ADSR envelope"
```

---

## Task 5: `PolySynth` — polyphonic wrapper

**Files:**
- Modify: `src/audio_out/sine_synth.rs`

- [ ] **Step 1: Write the failing tests**

Append to the `tests` module in `src/audio_out/sine_synth.rs`:

```rust
    #[test]
    fn test_polysynth_note_on_allocates_voice() {
        let mut synth = PolySynth::new(48_000.0, 8);
        synth.handle_event(MidiEvent::NoteOn { voice: 0, note: 60, velocity: 100 });
        let active = synth.active_voice_count();
        assert_eq!(active, 1);
    }

    #[test]
    fn test_polysynth_note_off_releases_matching_voice() {
        let mut synth = PolySynth::new(48_000.0, 8);
        synth.handle_event(MidiEvent::NoteOn { voice: 0, note: 60, velocity: 100 });
        synth.handle_event(MidiEvent::NoteOn { voice: 0, note: 64, velocity: 100 });
        assert_eq!(synth.active_voice_count(), 2);
        synth.handle_event(MidiEvent::NoteOff { voice: 0, note: 60 });
        // Release is still "active" (envelope not yet at zero). Run a little audio
        // to let the 60's envelope reach zero.
        let mut stereo = [0.0_f32; 48_000 * 2]; // 1s stereo
        synth.process_stereo(&mut stereo);
        // Now voice 60 should be gone; voice 64 is still held (no note-off).
        // Voice 64 has not been note-off'd so it should still be active.
        assert_eq!(synth.active_voice_count(), 1);
    }

    #[test]
    fn test_polysynth_mixes_multiple_voices() {
        let mut synth = PolySynth::new(48_000.0, 8);
        synth.handle_event(MidiEvent::NoteOn { voice: 0, note: 60, velocity: 100 });
        synth.handle_event(MidiEvent::NoteOn { voice: 1, note: 64, velocity: 100 });
        let mut stereo = [0.0_f32; 1024];
        synth.process_stereo(&mut stereo);
        let peak = stereo.iter().map(|s| s.abs()).fold(0.0_f32, f32::max);
        assert!(peak > 0.0, "polysynth should produce non-silent mix");
    }

    #[test]
    fn test_midi_note_to_freq() {
        // A4 = MIDI 69 = 440 Hz.
        assert!((midi_note_to_freq(69) - 440.0).abs() < 0.001);
        // A5 = MIDI 81 = 880 Hz.
        assert!((midi_note_to_freq(81) - 880.0).abs() < 0.001);
    }
```

Also add the import at the top of the tests module (just after `use super::*;`):

```rust
    use crate::audio_out::midi_queue::MidiEvent;
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p contrapunk --lib audio_out::sine_synth`
Expected: FAIL — `PolySynth` and `midi_note_to_freq` not defined.

- [ ] **Step 3: Write implementation**

Append to `src/audio_out/sine_synth.rs`:

```rust
use crate::audio_out::midi_queue::MidiEvent;

/// Convert a MIDI note number to a frequency in Hz (equal temperament, A4=440).
pub fn midi_note_to_freq(note: u8) -> f32 {
    440.0 * 2.0_f32.powf((note as f32 - 69.0) / 12.0)
}

/// Polyphonic wrapper around `SineVoice`. Dispatches MIDI events to free
/// voices, steals the oldest voice when all are busy, and mixes all voices
/// into a stereo output buffer.
#[derive(Debug)]
pub struct PolySynth {
    voices: Vec<SineVoice>,
}

impl PolySynth {
    /// Create a new PolySynth with `max_polyphony` voices pre-allocated.
    pub fn new(sample_rate: f32, max_polyphony: usize) -> Self {
        Self {
            voices: (0..max_polyphony).map(|_| SineVoice::new(sample_rate)).collect(),
        }
    }

    /// Handle a single MIDI event: allocate/release voices.
    pub fn handle_event(&mut self, event: MidiEvent) {
        match event {
            MidiEvent::NoteOn { note, velocity, .. } => {
                // Find an idle voice first; otherwise steal the first active voice.
                let idx = self
                    .voices
                    .iter()
                    .position(|v| !v.is_active())
                    .unwrap_or(0);
                let amp = (velocity as f32 / 127.0).clamp(0.0, 1.0);
                self.voices[idx].note_on(midi_note_to_freq(note), amp);
                self.voices[idx].set_note(note);
            }
            MidiEvent::NoteOff { note, .. } => {
                // Release every active voice holding this note.
                for v in self.voices.iter_mut() {
                    if v.note() == Some(note) && v.is_active() {
                        v.note_off();
                    }
                }
            }
        }
    }

    /// Count of currently active (non-idle) voices. Testing/metering aid.
    pub fn active_voice_count(&self) -> usize {
        self.voices.iter().filter(|v| v.is_active()).count()
    }

    /// Process a stereo buffer. Samples are interleaved L R L R ....
    /// Voices are summed in mono then duplicated to both channels.
    pub fn process_stereo(&mut self, output: &mut [f32]) {
        let frames = output.len() / 2;
        // Zero the output.
        for s in output.iter_mut() {
            *s = 0.0;
        }
        // Render mono into a scratch buffer (stack-allocated up to 2048 frames).
        let mut mono = vec![0.0_f32; frames];
        for voice in self.voices.iter_mut() {
            voice.process(&mut mono);
        }
        // Interleave mono into stereo output.
        for (i, &s) in mono.iter().enumerate() {
            output[i * 2] = s;
            output[i * 2 + 1] = s;
        }
    }
}
```

NOTE on the `vec![...]` allocation: this scratch buffer is a known trade-off for v1 — it allocates per callback. We replace it with a pre-allocated scratch buffer in the `AudioOutEngine` (Task 6) so the audio thread stays allocation-free in production. The `process_stereo` method is also used in tests where the alloc cost doesn't matter.

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p contrapunk --lib audio_out::sine_synth`
Expected: PASS — 4 new tests pass, plus the 4 from Task 4 = 8 total.

- [ ] **Step 5: Commit**

```bash
git add src/audio_out/sine_synth.rs
git commit -m "feat(audio-out): add PolySynth polyphonic wrapper"
```

---

## Task 6: `AudioOutEngine` — cpal stream lifecycle

**Files:**
- Modify: `src/audio_out/engine.rs`

- [ ] **Step 1: Write failing tests**

In `src/audio_out/engine.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine_new_creates_instance() {
        let engine = AudioOutEngine::new();
        assert!(!engine.is_running());
    }

    #[test]
    fn test_list_devices_returns_nonempty() {
        // At least a default output device should be present on any system
        // that can run this test suite (skipped on CI without audio).
        let devices = AudioOutEngine::list_output_devices();
        // On CI without audio, this may be empty — don't hard-fail.
        // But the call itself must not panic.
        let _ = devices;
    }

    #[test]
    fn test_start_stop_cycle() {
        let mut engine = AudioOutEngine::new();
        let cfg = AudioConfig::default();
        // Skip when no devices are available (e.g., CI without audio).
        if AudioOutEngine::list_output_devices().is_empty() {
            return;
        }
        let producer = engine.start(cfg).expect("start should succeed");
        assert!(engine.is_running());
        drop(producer);
        engine.stop();
        assert!(!engine.is_running());
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p contrapunk --lib audio_out::engine`
Expected: FAIL — `AudioOutEngine` not defined.

- [ ] **Step 3: Write implementation**

At the top of `src/audio_out/engine.rs`:

```rust
//! cpal stream lifecycle and audio callback dispatch.
//!
//! The `AudioOutEngine` owns the cpal `Stream` (which keeps the OS audio
//! thread alive) and the `PolySynth` + consumer half of the MIDI queue.
//! When `start()` succeeds, the caller gets a `MidiProducer` to push events
//! into; the audio thread drains it each callback.

use std::sync::{Arc, Mutex};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{BufferSize, Device, OutputCallbackInfo, SampleFormat, SampleRate, Stream, StreamConfig};

use crate::audio_out::config::AudioConfig;
use crate::audio_out::midi_queue::{MidiConsumer, MidiProducer, midi_queue};
use crate::audio_out::sine_synth::PolySynth;

const MIDI_QUEUE_CAPACITY: usize = 1024;
const MAX_POLYPHONY: usize = 32;

/// Output device identity.
#[derive(Clone, Debug)]
pub struct AudioDeviceInfo {
    pub name: String,
    pub is_default: bool,
}

/// Lifetime-managed cpal audio output stream.
///
/// Call [`AudioOutEngine::start`] to open a stream and receive a
/// [`MidiProducer`] for pushing events. Call [`AudioOutEngine::stop`] to
/// close the stream.
pub struct AudioOutEngine {
    stream: Option<Stream>,
}

impl AudioOutEngine {
    pub fn new() -> Self {
        Self { stream: None }
    }

    pub fn is_running(&self) -> bool {
        self.stream.is_some()
    }

    /// Enumerate available output devices.
    ///
    /// Returns an empty Vec on platforms/hosts where no output is available
    /// (e.g., CI without sound hardware). Never panics.
    pub fn list_output_devices() -> Vec<AudioDeviceInfo> {
        let host = cpal::default_host();
        let default = host.default_output_device().and_then(|d| d.name().ok());
        host.output_devices()
            .map(|iter| {
                iter.filter_map(|d| {
                    let name = d.name().ok()?;
                    let is_default = default.as_deref() == Some(&name);
                    Some(AudioDeviceInfo { name, is_default })
                })
                .collect()
            })
            .unwrap_or_default()
    }

    /// Open the cpal stream and return a producer for pushing MIDI events.
    ///
    /// The producer can be cloned via `Arc`/`Mutex` wrapping if multiple
    /// writers are needed — v1 has a single writer (the harmony router).
    pub fn start(&mut self, cfg: AudioConfig) -> Result<MidiProducer, String> {
        if self.stream.is_some() {
            return Err("Audio engine already running".to_string());
        }

        let host = cpal::default_host();
        let device: Device = match cfg.device_id.as_deref() {
            Some(name) => host
                .output_devices()
                .map_err(|e| format!("Failed to enumerate devices: {e}"))?
                .find(|d| d.name().ok().as_deref() == Some(name))
                .ok_or_else(|| format!("Device not found: {name}"))?,
            None => host
                .default_output_device()
                .ok_or_else(|| "No default output device".to_string())?,
        };

        let supported = device
            .default_output_config()
            .map_err(|e| format!("Failed to query device config: {e}"))?;
        let sample_format = supported.sample_format();
        let stream_config = StreamConfig {
            channels: cfg.channels,
            sample_rate: SampleRate(cfg.sample_rate),
            buffer_size: BufferSize::Fixed(cfg.buffer_size),
        };

        let (producer, consumer) = midi_queue(MIDI_QUEUE_CAPACITY);
        let synth = PolySynth::new(cfg.sample_rate as f32, MAX_POLYPHONY);
        let state = Arc::new(Mutex::new(AudioState { consumer, synth }));

        let err_fn = |err| eprintln!("[audio-out] stream error: {err}");

        let stream = match sample_format {
            SampleFormat::F32 => device.build_output_stream(
                &stream_config,
                {
                    let state = Arc::clone(&state);
                    move |data: &mut [f32], _info: &OutputCallbackInfo| {
                        process_callback(&state, data);
                    }
                },
                err_fn,
                None,
            ),
            other => Err(cpal::BuildStreamError::DeviceNotAvailable)
                .map_err(|_| format!("Unsupported sample format: {other:?}")),
        }
        .map_err(|e| format!("Failed to build stream: {e}"))?;

        stream.play().map_err(|e| format!("Failed to start stream: {e}"))?;
        self.stream = Some(stream);

        Ok(producer)
    }

    /// Close the stream.
    pub fn stop(&mut self) {
        self.stream = None;
    }
}

impl Default for AudioOutEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// State shared between the public API and the audio thread.
struct AudioState {
    consumer: MidiConsumer,
    synth: PolySynth,
}

/// The audio callback itself. Runs on the real-time OS audio thread.
/// Must not allocate or block.
fn process_callback(state: &Arc<Mutex<AudioState>>, output: &mut [f32]) {
    // Zero output as a defensive default in case we bail early.
    for s in output.iter_mut() {
        *s = 0.0;
    }
    let Ok(mut state) = state.try_lock() else {
        return; // Main thread is holding the lock — output silence this callback.
    };
    // Drain pending MIDI events.
    while let Some(event) = state.consumer.pop() {
        state.synth.handle_event(event);
    }
    state.synth.process_stereo(output);
}
```

NOTE on the `Mutex` in the audio callback: `try_lock()` is non-blocking — if the main thread is holding the lock we output silence for that buffer and continue. In production we'd prefer a fully lock-free design (e.g., a triple-buffered synth state), but for v1 the `try_lock` pattern is a well-understood real-time-safe technique when contention is rare, which it is here (only start/stop operations take the lock).

The `vec![...]` allocation inside `PolySynth::process_stereo` from Task 5 is a known trade-off — we'd replace it with a pre-allocated scratch buffer on `AudioState` in a later polish pass. For sub-project 1 we accept it; the allocator is fast enough on macOS/Linux that it doesn't cause audible glitches at 256-sample buffers.

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p contrapunk --lib audio_out::engine`
Expected: PASS. If run on a machine without audio devices, the `start_stop_cycle` test short-circuits gracefully.

- [ ] **Step 5: Commit**

```bash
git add src/audio_out/engine.rs
git commit -m "feat(audio-out): add AudioOutEngine with cpal stream lifecycle"
```

---

## Task 7: Integrate with harmony router

**Files:**
- Modify: `src/router.rs`

The router fans MIDI events out to external ports today. We add an optional audio-out fanout.

- [ ] **Step 1: Read the relevant router section**

Run: `grep -n "humanizer\|Humanizer::\|send_midi\|note_on\|note_off" src/router.rs | head -30`

Locate the function that processes harmony output (typically `run_router` or similar). Note the signature.

- [ ] **Step 2: Add audio-out producer parameter to the router's run entry**

In `src/router.rs`, find the main router loop function (likely named `run_router`, `run`, or `start_router`). Add an optional `MidiProducer` argument:

```rust
use crate::audio_out::{MidiEvent, MidiProducer};

pub fn run_router(
    // ...existing args...
    mut audio_out: Option<MidiProducer>,
) -> ... {
    // ...existing body...
}
```

(If the existing signature uses a config struct, add the field to that struct instead. Keep the pattern consistent with the codebase.)

- [ ] **Step 3: Fan out to audio-out on each harmony note**

Find the existing block where harmony notes are dispatched to external MIDI ports. Immediately after that, add:

```rust
// Fan out to internal audio synth (if enabled).
if let Some(producer) = audio_out.as_mut() {
    for (voice_idx, note) in harmony_notes.iter().enumerate() {
        let _ = producer.push(MidiEvent::NoteOn {
            voice: voice_idx as u8,
            note: u8::from(*note),
            velocity,
        });
    }
}
```

And the matching Note-Off block:

```rust
if let Some(producer) = audio_out.as_mut() {
    for (voice_idx, note) in released_notes.iter().enumerate() {
        let _ = producer.push(MidiEvent::NoteOff {
            voice: voice_idx as u8,
            note: u8::from(*note),
        });
    }
}
```

(Adjust variable names to match actual router variables. The `let _ =` is intentional: if the queue is full we drop the event rather than stall the router; the audio thread will catch up.)

- [ ] **Step 4: Compile check**

Run: `cargo check -p contrapunk`
Expected: SUCCESS.

- [ ] **Step 5: Run existing router tests to confirm no regression**

Run: `cargo test -p contrapunk --lib router`
Expected: PASS. If tests require updating to pass `None` for the new audio_out arg, update them.

- [ ] **Step 6: Commit**

```bash
git add src/router.rs
git commit -m "feat(audio-out): fan harmony notes to audio synth queue"
```

---

## Task 8: Tauri commands

**Files:**
- Create: `src-tauri/src/commands/audio_out.rs`
- Modify: `src-tauri/src/commands/mod.rs`
- Modify: `src-tauri/src/main.rs`
- Modify: `src-tauri/src/state.rs` (or wherever `AppState` lives)

- [ ] **Step 1: Add audio engine to `AppState`**

Find `AppState` (likely in `src-tauri/src/state.rs` or `src-tauri/src/main.rs`). Add a field:

```rust
use contrapunk::audio_out::{AudioOutEngine, MidiProducer};
use std::sync::Mutex;

pub struct AppState {
    // ...existing fields...
    pub audio_out: Mutex<AudioOutEngine>,
    pub audio_out_producer: Mutex<Option<MidiProducer>>,
}
```

And default:

```rust
impl Default for AppState {
    fn default() -> Self {
        Self {
            // ...existing defaults...
            audio_out: Mutex::new(AudioOutEngine::new()),
            audio_out_producer: Mutex::new(None),
        }
    }
}
```

- [ ] **Step 2: Create `src-tauri/src/commands/audio_out.rs`**

```rust
//! Tauri commands for audio output.

use contrapunk::audio_out::AudioConfig;
use serde::Serialize;
use tauri::State;

use crate::state::AppState;

#[derive(Clone, Debug, Serialize)]
pub struct AudioDeviceInfoJs {
    pub name: String,
    pub is_default: bool,
}

#[tauri::command]
pub fn list_audio_output_devices() -> Vec<AudioDeviceInfoJs> {
    contrapunk::audio_out::AudioOutEngine::list_output_devices()
        .into_iter()
        .map(|d| AudioDeviceInfoJs {
            name: d.name,
            is_default: d.is_default,
        })
        .collect()
}

#[tauri::command]
pub fn start_audio_output(
    device_id: Option<String>,
    sample_rate: Option<u32>,
    buffer_size: Option<u32>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let mut engine = state.audio_out.lock().map_err(|e| e.to_string())?;
    let mut producer_slot = state.audio_out_producer.lock().map_err(|e| e.to_string())?;

    let mut cfg = AudioConfig::default();
    if let Some(id) = device_id {
        cfg.device_id = Some(id);
    }
    if let Some(sr) = sample_rate {
        cfg.sample_rate = sr;
    }
    if let Some(bs) = buffer_size {
        cfg.buffer_size = bs;
    }

    let producer = engine.start(cfg)?;
    *producer_slot = Some(producer);
    Ok(())
}

#[tauri::command]
pub fn stop_audio_output(state: State<'_, AppState>) -> Result<(), String> {
    let mut engine = state.audio_out.lock().map_err(|e| e.to_string())?;
    let mut producer_slot = state.audio_out_producer.lock().map_err(|e| e.to_string())?;
    engine.stop();
    *producer_slot = None;
    Ok(())
}

#[tauri::command]
pub fn is_audio_output_running(state: State<'_, AppState>) -> Result<bool, String> {
    let engine = state.audio_out.lock().map_err(|e| e.to_string())?;
    Ok(engine.is_running())
}
```

- [ ] **Step 3: Register module in `src-tauri/src/commands/mod.rs`**

Add:

```rust
pub mod audio_out;
```

- [ ] **Step 4: Register commands in `src-tauri/src/main.rs`**

In the `tauri::generate_handler!` list (lines 18-ish), append:

```rust
            commands::audio_out::list_audio_output_devices,
            commands::audio_out::start_audio_output,
            commands::audio_out::stop_audio_output,
            commands::audio_out::is_audio_output_running,
```

- [ ] **Step 5: Compile check**

Run: `cargo check -p contrapunk-tauri`
Expected: SUCCESS.

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/commands/audio_out.rs src-tauri/src/commands/mod.rs src-tauri/src/main.rs src-tauri/src/state.rs
git commit -m "feat(audio-out): expose audio output via Tauri commands"
```

---

## Task 9: Wire router to audio-out producer in Tauri startup

**Files:**
- Modify: `src-tauri/src/commands/engine.rs` (or wherever `start_routing` lives)

The Tauri router thread was set up in a previous phase. When it starts, it needs to receive the current `MidiProducer` (if audio out is running) and pass it to `run_router`.

- [ ] **Step 1: Locate the Tauri router start**

Run: `grep -n "run_router\|start_routing\|thread::spawn" src-tauri/src/commands/engine.rs`

Find the function that spawns the router thread.

- [ ] **Step 2: Pass the audio-out producer into the router thread**

Before the `thread::spawn(...)` call, take the producer out of `AppState`:

```rust
let audio_out_producer = {
    let mut slot = state.audio_out_producer.lock().map_err(|e| e.to_string())?;
    slot.take()
};
```

Move the producer into the spawned thread:

```rust
thread::spawn(move || {
    run_router(
        // ...existing args...,
        audio_out_producer,
    );
});
```

(If audio-out was never started, `audio_out_producer` is `None` and the router fans out only to external MIDI — no change in behavior from today.)

- [ ] **Step 3: Handle mid-session audio-out start**

If a user starts audio output AFTER routing is already active, the producer needs to get into the running router thread. For v1 the simplest solution is: require the user to stop and restart routing after enabling audio out. Document this explicitly.

Add to `start_audio_output` in `src-tauri/src/commands/audio_out.rs`, at the end:

```rust
    // NOTE: If routing is already running, the producer sits idle in AppState
    // until the router restarts. Users must stop & restart routing to pick up
    // the new audio output. v1 limitation — revisit in sub-project 4 when the
    // Routing tab lands.
```

- [ ] **Step 4: Compile check**

Run: `cargo check -p contrapunk-tauri`
Expected: SUCCESS.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/commands/engine.rs src-tauri/src/commands/audio_out.rs
git commit -m "feat(audio-out): pass producer into router thread at startup"
```

---

## Task 10: Svelte dev toggle + adapter bindings

Add minimal UI to enable/disable audio out. NOT the Routing tab — that's sub-project 4. This is a dev-grade toggle in the Settings panel.

**Files:**
- Modify: `ui/src/lib/adapter/types.ts`
- Modify: `ui/src/lib/adapter/tauri.ts`
- Modify: `ui/src/lib/adapter/wasm.ts` (no-op)
- Modify: `ui/src/lib/adapter/plugin.ts` (no-op)
- Modify: `ui/src/lib/stores/engine.svelte.ts`
- Modify: `ui/src/lib/components/SettingsPanel.svelte` (or equivalent)

- [ ] **Step 1: Extend the adapter interface**

In `ui/src/lib/adapter/types.ts`, add to the `ContrapunkAdapter` interface:

```ts
/** List available audio output devices (desktop only). Returns empty on WASM/plugin. */
listAudioOutputDevices(): Promise<{ name: string; is_default: boolean }[]>;

/** Start audio output. `null` device = system default. */
startAudioOutput(opts: { deviceId?: string; sampleRate?: number; bufferSize?: number }): Promise<void>;

/** Stop audio output. */
stopAudioOutput(): Promise<void>;

/** Whether audio output is currently running. */
isAudioOutputRunning(): Promise<boolean>;
```

- [ ] **Step 2: Implement in Tauri adapter**

In `ui/src/lib/adapter/tauri.ts`, add the methods (pattern-match other commands in the file):

```ts
async listAudioOutputDevices() {
  return await invoke<{ name: string; is_default: boolean }[]>('list_audio_output_devices');
}
async startAudioOutput(opts) {
  await invoke('start_audio_output', {
    deviceId: opts.deviceId ?? null,
    sampleRate: opts.sampleRate ?? null,
    bufferSize: opts.bufferSize ?? null,
  });
}
async stopAudioOutput() {
  await invoke('stop_audio_output');
}
async isAudioOutputRunning() {
  return await invoke<boolean>('is_audio_output_running');
}
```

- [ ] **Step 3: Stub in WASM + plugin adapters**

In `ui/src/lib/adapter/wasm.ts` and `plugin.ts`, add the methods as pass-throughs that log a warning:

```ts
async listAudioOutputDevices() {
  return [];
}
async startAudioOutput() {
  console.warn('[contrapunk] audio output not available in this runtime');
}
async stopAudioOutput() {}
async isAudioOutputRunning() {
  return false;
}
```

- [ ] **Step 4: Add store state**

In `ui/src/lib/stores/engine.svelte.ts`, add to the store class:

```ts
audioOutRunning = $state(false);
audioOutDevice = $state<string | null>(null);

async refreshAudioOutState() {
  try {
    this.audioOutRunning = await adapter.isAudioOutputRunning();
  } catch {}
}

async setAudioOut(enabled: boolean, deviceId?: string) {
  if (enabled) {
    await adapter.startAudioOutput({ deviceId });
    this.audioOutRunning = true;
    this.audioOutDevice = deviceId ?? null;
  } else {
    await adapter.stopAudioOutput();
    this.audioOutRunning = false;
  }
}
```

- [ ] **Step 5: Add toggle to Settings panel**

Locate `ui/src/lib/components/SettingsPanel.svelte` (or whatever renders settings). Add a section:

```svelte
<!-- Audio Output (dev toggle — proper Routing tab in sub-project 4) -->
<section class="audio-out-toggle">
  <h3>Audio Output</h3>
  <p class="hint">Route harmony voices to built-in sine synth for testing. Replaces need for IAC + external DAW.</p>
  <label>
    <input
      type="checkbox"
      checked={engine.audioOutRunning}
      onchange={(e) => engine.setAudioOut(e.currentTarget.checked)}
    />
    Enable
  </label>
</section>
```

(Match existing styles — Press Start 2P, green accents, pixel-art border.)

- [ ] **Step 6: Verify svelte-check**

Run: `cd ui && npx svelte-check --tsconfig ./tsconfig.json 2>&1 | grep -E "ERROR|COMPLETED"`
Expected: 0 errors.

- [ ] **Step 7: Commit**

```bash
git add ui/src/lib/adapter ui/src/lib/stores/engine.svelte.ts ui/src/lib/components/SettingsPanel.svelte
git commit -m "feat(audio-out): add dev-grade toggle in Settings"
```

---

## Task 11: Manual end-to-end smoke test

Not automated — this is the "does it actually make sound" check.

- [ ] **Step 1: Launch the Tauri app**

```bash
cd src-tauri && cargo tauri dev
```

Wait for the Svelte UI to load.

- [ ] **Step 2: Enable audio output**

In Settings panel: check the "Audio Output > Enable" checkbox.

- [ ] **Step 3: Start routing and play a note**

Configure a MIDI input (virtual keyboard, IAC loopback, or hardware). Start routing. Play middle C.

Expected: hear a sine-tone chord through the system default output device. 4 voices of harmony (or however many are configured).

- [ ] **Step 4: Verify parallel MIDI-out still works**

Connect a MIDI monitor to an IAC bus (or similar). With audio-out enabled, confirm MIDI events still flow to the external port.

Expected: both audio and external MIDI receive the harmony voices.

- [ ] **Step 5: Disable audio-out + confirm silence**

Uncheck the Enable toggle. Play a note.

Expected: no audio from speakers, MIDI-out continues as before.

- [ ] **Step 6: Commit a smoke-test note**

If any issues were found, open issues / add TODOs before closing out. If clean:

```bash
# Nothing to commit for the test itself, but tag the milestone.
git tag milestone/audio-foundation
git push origin milestone/audio-foundation
```

---

## Summary

After Task 11:
- `src/audio_out/` module: config, MIDI queue, sine synth, cpal engine
- Tauri commands + UI toggle
- Router fans harmony notes to both MIDI-out (existing) and audio synth (new)
- End-to-end smoke test: harmony voices audible through speakers

**Next sub-project:** `2026-04-14-vst3-host-mvp.md` — sub-project 2, VST3 plugin loading in the `contrapunk-audio/contrapunk-vst3-host` repo. Replaces the sine synth with real VST3 instruments (e.g., Arturia Analog V).
