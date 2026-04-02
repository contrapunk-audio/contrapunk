//! Bridge between cpal audio capture and the MIDI routing thread.
//!
//! Spawns an audio capture thread that feeds audio blocks through
//! GuitarInput::process_block(), converts MidiEvent to MIDI bytes,
//! and sends them via an mpsc::Sender<Vec<u8>>.

use contrapunk::audio::guitar_input::{GuitarCalibration, GuitarInput, GuitarInputConfig};
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

        // Find device by name, or fall back to default
        let device = if device_name.is_empty() {
            host.default_input_device()
                .ok_or("No default audio input device")?
        } else {
            let found = host.input_devices()
                .map_err(|e| format!("Failed to enumerate audio devices: {}", e))?
                .find(|d| d.name().unwrap_or_default().contains(device_name));
            match found {
                Some(d) => d,
                None => {
                    eprintln!("[guitar_bridge] Device '{}' not found, using default", device_name);
                    host.default_input_device()
                        .ok_or("No default audio input device")?
                }
            }
        };

        let supported_config = device
            .default_input_config()
            .map_err(|e| format!("No input config: {}", e))?;

        let sample_rate = supported_config.sample_rate().0 as usize;
        let channels = supported_config.channels() as usize;
        let pb_range = config.pitch_bend_range;

        let mut actual_config = config;
        actual_config.sample_rate = sample_rate;

        let pipeline = Arc::new(Mutex::new(GuitarInput::new(actual_config)));

        let pipeline_c = Arc::clone(&pipeline);
        let stream_config: cpal::StreamConfig = supported_config.into();

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
            )
            .map_err(|e| format!("Failed to build audio stream: {}", e))?;

        Ok(Self {
            stream: Some(stream),
            pipeline,
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
