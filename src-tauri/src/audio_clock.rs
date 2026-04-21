//! Cpal silent-output stream that drives the app's transport clock.
//!
//! # Model
//!
//! A single cpal output stream runs for the life of the application.
//! The stream is silent (fills buffers with zeros) — its only job is to
//! tick an [`Arc<Transport>`] on a deterministic schedule. Later work
//! (metronome, plugin hosting) can repurpose the same callback to
//! actually produce audio.
//!
//! The callback also detects beat-boundary crossings and pushes them
//! onto an `mpsc::Sender<BeatCrossing>`. A separate forwarding thread
//! pulls crossings and emits `beat-update` Tauri events for the UI.
//!
//! # Thread ownership
//!
//! `cpal::Stream` is `!Send` on several platforms. Rather than store it
//! in Tauri's managed state (which requires `Send + Sync`), we hand it
//! to a dedicated audio-owner thread that parks itself and lets the
//! stream's OS-driven callback run in the background for the process
//! lifetime. Dropping the app terminates the process and the stream
//! along with it.

use std::sync::{mpsc, Arc};
use std::thread;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::SampleFormat;
use serde::Serialize;
use tauri::{AppHandle, Emitter};

use contrapunk::transport::{BeatCrossing, Transport};

/// Payload for the `beat-update` Tauri event.
#[derive(Clone, Serialize)]
struct BeatUpdatePayload {
    total_beat: u64,
    beat_in_bar: u8,
    bar: u64,
    bpm: f64,
    running: bool,
}

/// Start the audio clock. Spawns a dedicated thread that owns the cpal
/// stream for the app lifetime and a forwarding thread that drains
/// beat crossings to the Tauri event bus.
///
/// Returns only after the stream has been built and started; any cpal
/// setup errors bubble out as `Result::Err(String)`. Once this returns
/// Ok, the transport will tick whenever its `running` flag is true.
pub fn start(app_handle: AppHandle, transport: Arc<Transport>) -> Result<(), String> {
    let (tx, rx) = mpsc::channel::<BeatCrossing>();

    // Forwarding thread: pulls BeatCrossings and emits Tauri events.
    {
        let handle = app_handle.clone();
        let transport = Arc::clone(&transport);
        thread::spawn(move || {
            while let Ok(crossing) = rx.recv() {
                let _ = handle.emit(
                    "beat-update",
                    BeatUpdatePayload {
                        total_beat: crossing.total_beat,
                        beat_in_bar: crossing.beat_in_bar,
                        bar: crossing.bar,
                        bpm: transport.bpm(),
                        running: transport.is_running(),
                    },
                );
            }
        });
    }

    // Audio-owner thread: builds the cpal stream, keeps it alive forever.
    let (ready_tx, ready_rx) = mpsc::channel::<Result<(), String>>();
    {
        let transport = Arc::clone(&transport);
        thread::spawn(move || {
            match build_and_run_stream(transport, tx) {
                Ok(_stream) => {
                    let _ = ready_tx.send(Ok(()));
                    // Park the thread to keep the Stream alive. cpal's
                    // callback runs on its own driver thread; this
                    // thread just holds the Stream so it isn't dropped.
                    loop {
                        thread::park();
                    }
                }
                Err(e) => {
                    let _ = ready_tx.send(Err(e));
                }
            }
        });
    }

    ready_rx
        .recv()
        .map_err(|e| format!("audio-clock thread died: {}", e))?
}

/// Build the cpal output stream and start it. Returns the live `Stream`,
/// which must be kept in scope or the audio callback stops.
fn build_and_run_stream(
    transport: Arc<Transport>,
    crossing_tx: mpsc::Sender<BeatCrossing>,
) -> Result<cpal::Stream, String> {
    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .ok_or_else(|| "No default audio output device".to_string())?;

    let config = device
        .default_output_config()
        .map_err(|e| format!("Default output config error: {}", e))?;
    let sample_rate = config.sample_rate().0;
    let channels = config.channels();

    // Sync the transport's sample rate with the actual device rate.
    transport.set_sample_rate(sample_rate);
    eprintln!(
        "[audio-clock] cpal stream: {}Hz, {} channels, format={:?}",
        sample_rate,
        channels,
        config.sample_format()
    );

    let sample_format = config.sample_format();
    let stream_config: cpal::StreamConfig = config.into();

    // The callback must not allocate or lock. Our crossing_tx.send()
    // uses std::sync::mpsc which does allocate on occasion — for M2
    // this is acceptable because crossings occur ≤1× per buffer and
    // the queue is drained at UI rate. Swap for rtrb if dropouts.
    let advance_cb = {
        let transport = Arc::clone(&transport);
        let crossing_tx = crossing_tx;
        move |frames: u32| {
            if let Some(crossing) = transport.advance(frames) {
                let _ = crossing_tx.send(crossing);
            }
        }
    };

    let err_fn = |err: cpal::StreamError| {
        eprintln!("[audio-clock] stream error: {}", err);
    };

    let stream = match sample_format {
        SampleFormat::F32 => device.build_output_stream(
            &stream_config,
            {
                let cb = advance_cb;
                move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                    let frames = data.len() as u32 / channels as u32;
                    for s in data.iter_mut() {
                        *s = 0.0;
                    }
                    cb(frames);
                }
            },
            err_fn,
            None,
        ),
        SampleFormat::I16 => device.build_output_stream(
            &stream_config,
            {
                let cb = advance_cb;
                move |data: &mut [i16], _: &cpal::OutputCallbackInfo| {
                    let frames = data.len() as u32 / channels as u32;
                    for s in data.iter_mut() {
                        *s = 0;
                    }
                    cb(frames);
                }
            },
            err_fn,
            None,
        ),
        SampleFormat::U16 => device.build_output_stream(
            &stream_config,
            {
                let cb = advance_cb;
                move |data: &mut [u16], _: &cpal::OutputCallbackInfo| {
                    let frames = data.len() as u32 / channels as u32;
                    for s in data.iter_mut() {
                        *s = u16::MAX / 2;
                    }
                    cb(frames);
                }
            },
            err_fn,
            None,
        ),
        other => return Err(format!("Unsupported sample format: {:?}", other)),
    }
    .map_err(|e| format!("Failed to build output stream: {}", e))?;

    stream
        .play()
        .map_err(|e| format!("Failed to start output stream: {}", e))?;

    // Transport stays stopped until the user hits Play. The callback
    // ticks immediately but `advance()` is a no-op until then.
    Ok(stream)
}
