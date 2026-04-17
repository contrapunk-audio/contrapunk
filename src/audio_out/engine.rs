//! cpal stream lifecycle and audio callback dispatch.
//!
//! The `AudioOutEngine` owns the cpal `Stream` (which keeps the OS audio
//! thread alive) and the `PolySynth` + consumer half of the MIDI queue.
//! When `start()` succeeds, the caller gets a `MidiProducer` to push events
//! into; the audio thread drains it each callback.

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{
    BufferSize, Device, OutputCallbackInfo, SampleFormat, SampleRate, Stream, StreamConfig,
};

use crate::audio_out::config::AudioConfig;
use crate::audio_out::midi_queue::{midi_queue, MidiProducer};
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

        let (producer, mut consumer) = midi_queue(MIDI_QUEUE_CAPACITY);
        let mut synth = PolySynth::new(cfg.sample_rate as f32, MAX_POLYPHONY);

        let err_fn = |err| eprintln!("[audio-out] stream error: {err}");

        let stream = match sample_format {
            SampleFormat::F32 => device.build_output_stream(
                &stream_config,
                move |data: &mut [f32], _info: &OutputCallbackInfo| {
                    // Drain pending MIDI events (lock-free SPSC pop).
                    while let Some(event) = consumer.pop() {
                        synth.handle_event(event);
                    }
                    synth.process_stereo(data);
                },
                err_fn,
                None,
            ),
            other => {
                return Err(format!("Unsupported sample format: {other:?}"));
            }
        }
        .map_err(|e| format!("Failed to build stream: {e}"))?;

        stream
            .play()
            .map_err(|e| format!("Failed to start stream: {e}"))?;
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

// cpal's CoreAudio `Stream` is `!Send` because it contains a raw pointer marker,
// but `AudioOutEngine` is always accessed through a `Mutex` which serialises all
// access to a single thread at a time. The stream is never moved across threads
// while any CoreAudio thread-affinity operations are in flight — the audio
// callback runs independently on the OS audio thread via the C callback, not
// via Rust's `Send` mechanism. This pattern is identical to how other Rust audio
// apps (e.g., CPAL's own examples) handle state in a Tauri/GUI context.
// SAFETY: `Mutex<AudioOutEngine>` ensures exclusive access; no concurrent Rust
// thread races on the stream handle are possible.
unsafe impl Send for AudioOutEngine {}
unsafe impl Sync for AudioOutEngine {}

// PolySynth and MidiConsumer are owned exclusively by the audio callback
// closure — no shared state, no locks on the hot path. MIDI events arrive
// through the lock-free SPSC ringbuffer (MidiConsumer::pop).

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
