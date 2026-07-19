//! Real-time-safe bridge between cpal capture and guitar-to-MIDI DSP.
//!
//! The cpal callback only deinterleaves samples into a bounded SPSC queue.
//! A named worker owns all allocating detector work, config synchronization,
//! calibration access, MIDI conversion, and UI signal updates.

use contrapunk::audio::guitar::GuitarCalibrationProfile;
use contrapunk::audio::guitar_input::{GuitarInput, GuitarInputConfig, MidiEvent};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use ringbuf::traits::{Consumer, Producer, Split};
use ringbuf::HeapRb;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;

const WORKER_BLOCK_SIZE: usize = 1024;
const MAX_QUEUED_AUDIO_MS: usize = 100;

fn audio_queue_capacity(sample_rate: usize) -> usize {
    sample_rate
        .saturating_mul(MAX_QUEUED_AUDIO_MS)
        .saturating_div(1_000)
        .max(128)
}

/// Signal info emitted by the worker for UI feedback.
#[derive(Clone, Debug)]
pub struct GuitarSignalInfo {
    pub rms: f32,
    pub frequency: Option<f32>,
    pub clarity: f32,
    pub note_state: u8,
}

pub struct GuitarBridge {
    stream: Option<cpal::Stream>,
    stop: Arc<AtomicBool>,
    worker: Option<JoinHandle<()>>,
    pipeline: Arc<Mutex<GuitarInput>>,
    live_pipeline_handle: Option<Arc<Mutex<Option<Arc<Mutex<GuitarInput>>>>>>,
}

fn send_events(tx: &mpsc::Sender<Vec<u8>>, events: Vec<MidiEvent>, pitch_bend_range: u8) {
    for event in events {
        #[cfg(debug_assertions)]
        match &event {
            MidiEvent::NoteOn { note, velocity, .. } => {
                eprintln!("[guitar-midi] NoteOn note={note} velocity={velocity}");
            }
            MidiEvent::NoteOff { note, .. } => {
                eprintln!("[guitar-midi] NoteOff note={note}");
            }
            _ => {}
        }
        let bytes = event.to_midi_bytes(pitch_bend_range);
        if !bytes.is_empty() {
            let _ = tx.send(bytes);
        }
    }
}

fn send_all_notes_off(tx: &mpsc::Sender<Vec<u8>>) {
    for channel in 0..16 {
        let _ = tx.send(vec![0xB0 | channel, 123, 0]);
    }
}

fn discard_if_overflow<C>(overflow: &AtomicBool, audio_rx: &mut C) -> bool
where
    C: Consumer<Item = f32>,
{
    if !overflow.swap(false, Ordering::AcqRel) {
        return false;
    }
    while audio_rx.try_pop().is_some() {}
    true
}

impl GuitarBridge {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        device_name: &str,
        channel: usize,
        config: GuitarInputConfig,
        shared_config: Arc<Mutex<Option<GuitarInputConfig>>>,
        calibration_profile: Option<GuitarCalibrationProfile>,
        live_pipeline_handle: Option<Arc<Mutex<Option<Arc<Mutex<GuitarInput>>>>>>,
        tx: mpsc::Sender<Vec<u8>>,
        signal_tx: Option<mpsc::Sender<GuitarSignalInfo>>,
    ) -> Result<Self, String> {
        eprintln!(
            "[guitar_bridge] Creating bridge: device='{}' channel={}",
            device_name, channel
        );
        let host = cpal::default_host();
        let device = if device_name.is_empty() {
            host.default_input_device()
                .ok_or("No default audio input device")?
        } else {
            host.input_devices()
                .map_err(|e| format!("Failed to enumerate audio devices: {e}"))?
                .find(|device| device.name().unwrap_or_default().contains(device_name))
                .or_else(|| host.default_input_device())
                .ok_or("No default audio input device")?
        };
        let supported_config = device
            .default_input_config()
            .map_err(|e| format!("No input config: {e}"))?;
        let sample_rate = supported_config.sample_rate() as usize;
        let channels = supported_config.channels() as usize;
        if channel >= channels {
            return Err(format!(
                "Input channel {} is unavailable on a {}-channel device",
                channel + 1,
                channels
            ));
        }

        eprintln!(
            "[guitar_bridge] Found device: {} ({}ch {}Hz)",
            device.name().unwrap_or_default(),
            channels,
            sample_rate
        );

        let mut actual_config = config;
        actual_config.sample_rate = sample_rate;
        let mut pipeline_inner = GuitarInput::new(actual_config.clone());
        if let Some(profile) = calibration_profile {
            pipeline_inner.set_calibration_profile(profile);
        }
        let pipeline = Arc::new(Mutex::new(pipeline_inner));
        let pipeline_for_worker = Arc::clone(&pipeline);

        // Cap queued audio at 100 ms. Overflow drops the backlog and forces
        // All Notes Off rather than replaying stale detector output.
        let audio_rb = HeapRb::<f32>::new(audio_queue_capacity(sample_rate));
        let (mut audio_tx, mut audio_rx) = audio_rb.split();
        let overflow = Arc::new(AtomicBool::new(false));
        let overflow_for_callback = Arc::clone(&overflow);
        let stop = Arc::new(AtomicBool::new(false));
        let stop_for_worker = Arc::clone(&stop);

        let worker = thread::Builder::new()
            .name("contrapunk-guitar".into())
            .spawn(move || {
                let mut block = vec![0.0; WORKER_BLOCK_SIZE];
                let mut applied_config = actual_config;

                while !stop_for_worker.load(Ordering::Acquire) {
                    if discard_if_overflow(&overflow, &mut audio_rx) {
                        let cleanup = {
                            let mut pipe = pipeline_for_worker
                                .lock()
                                .unwrap_or_else(|poisoned| poisoned.into_inner());
                            pipe.replace_config(applied_config.clone())
                        };
                        send_events(&tx, cleanup, applied_config.pitch_bend_range);
                        send_all_notes_off(&tx);
                        continue;
                    }

                    if let Some(mut next_config) =
                        shared_config.lock().ok().and_then(|guard| guard.clone())
                    {
                        next_config.sample_rate = sample_rate;
                        if next_config != applied_config {
                            let cleanup = {
                                let mut pipe = pipeline_for_worker
                                    .lock()
                                    .unwrap_or_else(|poisoned| poisoned.into_inner());
                                pipe.replace_config(next_config.clone())
                            };
                            send_events(&tx, cleanup, applied_config.pitch_bend_range);
                            applied_config = next_config;
                        }
                    }

                    let count = audio_rx.pop_slice(&mut block);
                    if count == 0 {
                        thread::sleep(Duration::from_millis(1));
                        continue;
                    }
                    // Overflow may race the pop. Never process a block that was
                    // already declared stale by the callback.
                    if discard_if_overflow(&overflow, &mut audio_rx) {
                        let cleanup = pipeline_for_worker
                            .lock()
                            .unwrap_or_else(|poisoned| poisoned.into_inner())
                            .replace_config(applied_config.clone());
                        send_events(&tx, cleanup, applied_config.pitch_bend_range);
                        send_all_notes_off(&tx);
                        continue;
                    }

                    let (events, info) = {
                        let mut pipe = pipeline_for_worker
                            .lock()
                            .unwrap_or_else(|poisoned| poisoned.into_inner());
                        let events = pipe.process_block(&block[..count]);
                        let info = GuitarSignalInfo {
                            rms: pipe.prev_rms(),
                            frequency: pipe.last_debug_pitch.map(|(frequency, _)| frequency),
                            clarity: pipe
                                .last_debug_pitch
                                .map(|(_, clarity)| clarity)
                                .unwrap_or(0.0),
                            note_state: pipe.note_state_name(),
                        };
                        (events, info)
                    };
                    // A full callback queue can be reported while DSP runs.
                    // Discard its events and reset rather than sending stale MIDI.
                    if discard_if_overflow(&overflow, &mut audio_rx) {
                        let cleanup = pipeline_for_worker
                            .lock()
                            .unwrap_or_else(|poisoned| poisoned.into_inner())
                            .replace_config(applied_config.clone());
                        send_events(&tx, cleanup, applied_config.pitch_bend_range);
                        send_all_notes_off(&tx);
                        continue;
                    }
                    send_events(&tx, events, applied_config.pitch_bend_range);
                    if let Some(signal_tx) = signal_tx.as_ref() {
                        let _ = signal_tx.send(info);
                    }
                }

                let cleanup = pipeline_for_worker
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner())
                    .replace_config(applied_config.clone());
                send_events(&tx, cleanup, applied_config.pitch_bend_range);
            })
            .map_err(|e| format!("Failed to start guitar worker: {e}"))?;

        let stream_config = cpal::StreamConfig {
            channels: supported_config.channels(),
            sample_rate: supported_config.sample_rate(),
            buffer_size: cpal::BufferSize::Fixed(128),
        };
        let stream = match device.build_input_stream(
            &stream_config,
            move |data: &[f32], _: &cpal::InputCallbackInfo| {
                for frame in data.chunks_exact(channels) {
                    if audio_tx.try_push(frame[channel]).is_err() {
                        overflow_for_callback.store(true, Ordering::Release);
                        break;
                    }
                }
            },
            |err| eprintln!("Guitar audio error: {err}"),
            None,
        ) {
            Ok(stream) => stream,
            Err(error) => {
                stop.store(true, Ordering::Release);
                let _ = worker.join();
                return Err(format!("Failed to build audio stream: {error}"));
            }
        };

        if let Err(error) = stream.play() {
            stop.store(true, Ordering::Release);
            let _ = worker.join();
            return Err(format!("Failed to start audio: {error}"));
        }

        // Publish only a stream that is already playing. Failed startup can
        // never leave calibration commands pointing at a zombie pipeline.
        if let Some(handle) = live_pipeline_handle.as_ref() {
            if let Ok(mut slot) = handle.lock() {
                *slot = Some(Arc::clone(&pipeline));
            }
        }

        Ok(Self {
            stream: Some(stream),
            stop,
            worker: Some(worker),
            pipeline,
            live_pipeline_handle,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn callback_queue_is_bounded_to_one_tenth_second() {
        assert_eq!(audio_queue_capacity(48_000), 4_800);
        assert_eq!(audio_queue_capacity(44_100), 4_410);
    }

    #[test]
    fn overflow_discards_every_queued_sample() {
        let rb = HeapRb::<f32>::new(4);
        let (mut tx, mut rx) = rb.split();
        tx.push_slice(&[1.0, 2.0, 3.0, 4.0]);
        let overflow = AtomicBool::new(true);

        assert!(discard_if_overflow(&overflow, &mut rx));
        assert!(rx.try_pop().is_none());
        assert!(!discard_if_overflow(&overflow, &mut rx));
    }
}

impl Drop for GuitarBridge {
    fn drop(&mut self) {
        // Stop the callback before the worker so no producer can enqueue after
        // shutdown begins. Join gives routing teardown observable completion.
        self.stream.take();
        self.stop.store(true, Ordering::Release);
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
        if let Some(handle) = self.live_pipeline_handle.as_ref() {
            if let Ok(mut slot) = handle.lock() {
                if slot
                    .as_ref()
                    .is_some_and(|published| Arc::ptr_eq(published, &self.pipeline))
                {
                    *slot = None;
                }
            }
        }
    }
}
