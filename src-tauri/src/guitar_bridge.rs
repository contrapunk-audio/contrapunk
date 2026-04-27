//! Bridge between cpal audio capture and the MIDI routing thread.
//!
//! Spawns an audio capture thread that feeds audio blocks through
//! GuitarInput::process_block(), converts MidiEvent to MIDI bytes,
//! and sends them via an mpsc::Sender<Vec<u8>>.
//!
//! Also sends signal info (RMS, pitch, state) via a separate channel
//! for UI feedback in the Tauri frontend.

use contrapunk::audio::guitar_input::{GuitarInput, GuitarInputConfig};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::{mpsc, Arc, Mutex};

/// Signal info emitted every audio block for UI feedback.
#[derive(Clone, Debug)]
pub struct GuitarSignalInfo {
    pub rms: f32,
    pub frequency: Option<f32>,
    pub clarity: f32,
    pub note_state: u8, // 0=Idle, 1=Attack, 2=Sustain, 3=Decay
}

pub struct GuitarBridge {
    stream: Option<cpal::Stream>,
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
        shared_config: Arc<Mutex<Option<GuitarInputConfig>>>,
        tx: mpsc::Sender<Vec<u8>>,
        signal_tx: Option<mpsc::Sender<GuitarSignalInfo>>,
    ) -> Result<Self, String> {
        eprintln!(
            "[guitar_bridge] Creating bridge: device='{}' channel={}",
            device_name, channel
        );
        let host = cpal::default_host();

        // Find device by name, or fall back to default
        let device = if device_name.is_empty() {
            host.default_input_device()
                .ok_or("No default audio input device")?
        } else {
            let found = host
                .input_devices()
                .map_err(|e| format!("Failed to enumerate audio devices: {}", e))?
                .find(|d| d.name().unwrap_or_default().contains(device_name));
            match found {
                Some(d) => d,
                None => {
                    eprintln!(
                        "[guitar_bridge] Device '{}' not found, using default",
                        device_name
                    );
                    host.default_input_device()
                        .ok_or("No default audio input device")?
                }
            }
        };

        eprintln!(
            "[guitar_bridge] Found device: {}",
            device.name().unwrap_or_default()
        );
        let supported_config = device
            .default_input_config()
            .map_err(|e| format!("No input config: {}", e))?;
        eprintln!(
            "[guitar_bridge] Config: {}ch {}Hz",
            supported_config.channels(),
            supported_config.sample_rate()
        );

        let sample_rate = supported_config.sample_rate() as usize;
        let channels = supported_config.channels() as usize;

        let mut actual_config = config;
        actual_config.sample_rate = sample_rate;

        // DSP state lives inside the cpal stream closure — no outer binding
        // needed, the closure owns the only Arc reference.
        let pipeline = Arc::new(Mutex::new(GuitarInput::new(actual_config)));

        // Request small buffer (128 samples = ~2.7ms at 48kHz) for lower latency.
        // Falls back to driver default if the device doesn't support it.
        let stream_config = cpal::StreamConfig {
            channels: supported_config.channels(),
            sample_rate: supported_config.sample_rate(),
            buffer_size: cpal::BufferSize::Fixed(128),
        };

        let stream = device
            .build_input_stream(
                &stream_config,
                move |data: &[f32], _: &cpal::InputCallbackInfo| {
                    // Extract target channel
                    let mono: Vec<f32> = data
                        .chunks(channels)
                        .map(|frame| frame.get(channel).copied().unwrap_or(0.0))
                        .collect();

                    // Process through DSP pipeline
                    let (events, pb_range) = {
                        let mut pipe = pipeline.lock().unwrap();

                        // Sync config from shared state every block so the
                        // debug window's edits take effect live (without a
                        // routing restart). Preserve `sample_rate` because
                        // it's tied to the cpal stream config and can't
                        // change without rebuilding the stream.
                        if let Ok(guard) = shared_config.lock() {
                            if let Some(ref new_config) = *guard {
                                let cfg = pipe.config_mut();
                                let preserved_sr = cfg.sample_rate;
                                *cfg = new_config.clone();
                                cfg.sample_rate = preserved_sr;
                            }
                        }

                        let pb_range = pipe.config_mut().pitch_bend_range;
                        let evts = pipe.process_block(&mono);

                        // Send signal info for UI feedback
                        if let Some(ref sig_tx) = signal_tx {
                            let info = GuitarSignalInfo {
                                rms: pipe.prev_rms(),
                                frequency: pipe.last_debug_pitch.map(|(f, _)| f),
                                clarity: pipe.last_debug_pitch.map(|(_, c)| c).unwrap_or(0.0),
                                note_state: pipe.note_state_name(),
                            };
                            let _ = sig_tx.send(info);
                        }

                        (evts, pb_range)
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
            )
            .map_err(|e| format!("Failed to build audio stream: {}", e))?;

        Ok(Self {
            stream: Some(stream),
        })
    }

    /// Start audio capture.
    pub fn start(&self) -> Result<(), String> {
        if let Some(ref stream) = self.stream {
            stream
                .play()
                .map_err(|e| format!("Failed to start audio: {}", e))
        } else {
            Err("No audio stream".into())
        }
    }
}
