//! Cpal output stream that drives the app's transport clock and
//! synthesizes metronome clicks.
//!
//! # Model
//!
//! A single cpal output stream runs for the life of the application.
//! The stream's callback:
//!   1. Ticks the shared [`Transport`] by `frames`.
//!   2. If a beat boundary was crossed, enqueues a `BeatCrossing` for
//!      the UI event forwarder and (if metronome is enabled) kicks
//!      off a short decaying-sine click.
//!   3. Mixes any active click into the output buffer; silence otherwise.
//!
//! # Thread ownership
//!
//! `cpal::Stream` is `!Send` on several platforms. A dedicated
//! audio-owner thread holds the stream and parks itself so the
//! stream's OS-driven callback runs in the background for the process
//! lifetime. Dropping the app terminates the process and the stream.

use std::f32::consts::TAU;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc};
use std::thread;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::SampleFormat;
use serde::Serialize;
use tauri::{AppHandle, Emitter};

use contrapunk::chain::{
    AudioBlock, BlockDescriptor, Chain, ChainCommandConsumer, ChainCommander, ElixirSynthBlock,
    ELIXIR_EVENT_QUEUE_CAPACITY,
};
use contrapunk::elixir::{synth_event_channel, SynthEventReceiver, SynthParams};
use contrapunk::fx::{Delay, DelayParams, Reverb, ReverbParams};
use contrapunk::transport::{BeatCrossing, Transport};
use ringbuf::traits::{Producer, Split};
use ringbuf::HeapRb;

/// Payload for the `beat-update` Tauri event.
#[derive(Clone, Serialize)]
struct BeatUpdatePayload {
    total_beat: u64,
    beat_in_bar: u8,
    bar: u64,
    bpm: f64,
    running: bool,
}

/// Click-envelope duration in seconds. ~15 ms feels percussive without
/// smearing adjacent beats at high tempo.
const CLICK_SECS: f32 = 0.015;
/// Peak amplitude of the click. Deliberately conservative so there's
/// headroom once harmony audio flows through the same stream later.
const CLICK_GAIN: f32 = 0.30;
/// Frequency used for the downbeat (beat 0 in the bar).
const CLICK_FREQ_DOWNBEAT: f32 = 900.0;
/// Frequency used for other beats.
const CLICK_FREQ_OFFBEAT: f32 = 600.0;

/// Per-stream click synthesis state. Lives inside the cpal callback
/// closure, mutated only on the audio thread.
struct Click {
    active: bool,
    freq: f32,
    phase: f32,
    samples_remaining: u32,
    total: u32,
}

impl Click {
    fn new(total: u32) -> Self {
        Self {
            active: false,
            freq: 0.0,
            phase: 0.0,
            samples_remaining: 0,
            total,
        }
    }

    fn trigger(&mut self, freq: f32) {
        self.active = true;
        self.freq = freq;
        self.phase = 0.0;
        self.samples_remaining = self.total;
    }

    /// Render the next sample of the click envelope. Returns 0.0 when
    /// inactive. Advances phase and decrements the remaining counter.
    fn next_sample(&mut self, sample_rate: f32) -> f32 {
        if !self.active || self.samples_remaining == 0 {
            return 0.0;
        }
        let decay = self.samples_remaining as f32 / self.total as f32;
        let v = self.phase.sin() * CLICK_GAIN * decay;
        self.phase += TAU * self.freq / sample_rate;
        if self.phase > TAU {
            self.phase -= TAU;
        }
        self.samples_remaining -= 1;
        if self.samples_remaining == 0 {
            self.active = false;
        }
        v
    }
}

/// Start the audio clock. Spawns a dedicated thread that owns the cpal
/// stream for the app lifetime and a forwarding thread that drains
/// beat crossings to the Tauri event bus.
///
/// `metronome_enabled` — read on the audio thread at each beat crossing
/// to decide whether to trigger a click. Toggled via Tauri command
/// from the UI.
pub fn start(
    app_handle: AppHandle,
    transport: Arc<Transport>,
    metronome_enabled: Arc<AtomicBool>,
    synth_params: Arc<SynthParams>,
    synth_rx: Option<SynthEventReceiver>,
    reverb_params: Arc<ReverbParams>,
    delay_params: Arc<DelayParams>,
) -> Result<Arc<ChainCommander>, String> {
    let (tx, rx) = mpsc::channel::<BeatCrossing>();

    // Build the chain-command queue upfront so we can register the
    // initial (startup) blocks in the commander's mirror before the
    // audio thread starts consuming.
    let (commander, chain_rx) = ChainCommander::new_split();
    let commander = Arc::new(commander);
    commander.register_initial_blocks(vec![
        initial_synth_descriptor(),
        BlockDescriptor {
            type_id: "builtin.delay".into(),
            name: "Delay".into(),
        },
        BlockDescriptor {
            type_id: "builtin.reverb".into(),
            name: "Reverb".into(),
        },
    ]);

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
            match build_and_run_stream(
                transport,
                metronome_enabled,
                synth_params,
                synth_rx,
                reverb_params,
                delay_params,
                chain_rx,
                tx,
            ) {
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
        .map_err(|e| format!("audio-clock thread died: {}", e))??;
    Ok(commander)
}

fn initial_synth_descriptor() -> BlockDescriptor {
    BlockDescriptor {
        type_id: "builtin.synth".into(),
        name: "Sine".into(),
    }
}

fn make_synth_block(
    params: Arc<SynthParams>,
    events: SynthEventReceiver,
    sample_rate: u32,
) -> Box<dyn AudioBlock> {
    let queue = HeapRb::new(ELIXIR_EVENT_QUEUE_CAPACITY);
    let (mut tx, rx) = queue.split();
    let source_fault = events.fault_flag();
    let event_fault = Arc::new(AtomicBool::new(false));
    let bridge_fault = Arc::clone(&event_fault);
    let _ = thread::Builder::new()
        .name("elixir-event-bridge".into())
        .spawn(move || {
            while let Ok(event) = events.recv() {
                if source_fault.swap(false, Ordering::AcqRel) {
                    while events.try_recv().is_ok() {}
                    bridge_fault.store(true, Ordering::Release);
                } else if tx.try_push(event).is_err() {
                    bridge_fault.store(true, Ordering::Release);
                }
            }
            bridge_fault.store(true, Ordering::Release);
        });
    Box::new(ElixirSynthBlock::new_with_event_consumer(
        sample_rate,
        params,
        rx,
        event_fault,
    ))
}

fn build_and_run_stream(
    transport: Arc<Transport>,
    metronome_enabled: Arc<AtomicBool>,
    synth_params: Arc<SynthParams>,
    synth_rx: Option<SynthEventReceiver>,
    reverb_params: Arc<ReverbParams>,
    delay_params: Arc<DelayParams>,
    chain_rx: ChainCommandConsumer,
    crossing_tx: mpsc::Sender<BeatCrossing>,
) -> Result<cpal::Stream, String> {
    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .ok_or_else(|| "No default audio output device".to_string())?;

    let config = device
        .default_output_config()
        .map_err(|e| format!("Default output config error: {}", e))?;
    let sample_rate = config.sample_rate();
    let channels = config.channels();

    transport.set_sample_rate(sample_rate);
    eprintln!(
        "[audio-clock] cpal stream: {}Hz, {} channels, format={:?}",
        sample_rate,
        channels,
        config.sample_format()
    );

    let sample_format = config.sample_format();
    let stream_config: cpal::StreamConfig = config.into();
    let click_total_samples = (sample_rate as f32 * CLICK_SECS) as u32;

    // Build the synth-event receiver — only wired on the F32 path below.
    // If no rx was provided (non-default AppState), we still run with an
    // empty one.
    let synth_rx = synth_rx.unwrap_or_else(|| {
        let (_tx, rx) = synth_event_channel();
        rx
    });

    let err_fn = |err: cpal::StreamError| {
        eprintln!("[audio-clock] stream error: {}", err);
    };

    let stream = match sample_format {
        SampleFormat::F32 => {
            let transport = Arc::clone(&transport);
            let metronome_enabled = Arc::clone(&metronome_enabled);
            let crossing_tx = crossing_tx.clone();
            let mut click = Click::new(click_total_samples);
            let sr_f = sample_rate as f32;

            // Build the audio chain. Order: synth first, then FX
            // blocks that shape its output in place. CLAP / VST3
            // plugin blocks are also pushed here once their hosts
            // exist.
            let synth = make_synth_block(synth_params, synth_rx, sample_rate);
            let reverb = Reverb::new(reverb_params, sample_rate);
            let delay = Delay::with_transport(delay_params, Arc::clone(&transport), sample_rate);
            let mut chain = Chain::with_queue(sample_rate, chain_rx);
            chain.push(synth);
            chain.push(Box::new(delay));
            chain.push(Box::new(reverb));

            device.build_output_stream(
                &stream_config,
                move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                    let frames = data.len() as u32 / channels as u32;

                    if let Some(crossing) = transport.advance(frames) {
                        let _ = crossing_tx.send(crossing);
                        if metronome_enabled.load(Ordering::Relaxed) {
                            let freq = if crossing.beat_in_bar == 0 {
                                CLICK_FREQ_DOWNBEAT
                            } else {
                                CLICK_FREQ_OFFBEAT
                            };
                            click.trigger(freq);
                        }
                    }

                    // Chain runs every block in order on the shared buffer.
                    // Synth (first block) writes audio; future FX blocks
                    // shape it in place.
                    chain.process(data, channels as usize);

                    // Metronome click mixes on top of the whole chain.
                    for frame in data.chunks_mut(channels as usize) {
                        let v = click.next_sample(sr_f);
                        if v != 0.0 {
                            for s in frame.iter_mut() {
                                *s += v;
                            }
                        }
                    }
                },
                err_fn,
                None,
            )
        }
        SampleFormat::I16 => {
            // Clicks not implemented for non-F32 formats. Transport
            // still ticks; stream outputs silence.
            let transport = Arc::clone(&transport);
            let crossing_tx = crossing_tx.clone();
            device.build_output_stream(
                &stream_config,
                move |data: &mut [i16], _: &cpal::OutputCallbackInfo| {
                    let frames = data.len() as u32 / channels as u32;
                    if let Some(crossing) = transport.advance(frames) {
                        let _ = crossing_tx.send(crossing);
                    }
                    for s in data.iter_mut() {
                        *s = 0;
                    }
                },
                err_fn,
                None,
            )
        }
        SampleFormat::U16 => {
            let transport = Arc::clone(&transport);
            let crossing_tx = crossing_tx.clone();
            device.build_output_stream(
                &stream_config,
                move |data: &mut [u16], _: &cpal::OutputCallbackInfo| {
                    let frames = data.len() as u32 / channels as u32;
                    if let Some(crossing) = transport.advance(frames) {
                        let _ = crossing_tx.send(crossing);
                    }
                    for s in data.iter_mut() {
                        *s = u16::MAX / 2;
                    }
                },
                err_fn,
                None,
            )
        }
        other => return Err(format!("Unsupported sample format: {:?}", other)),
    }
    .map_err(|e| format!("Failed to build output stream: {}", e))?;

    stream
        .play()
        .map_err(|e| format!("Failed to start output stream: {}", e))?;

    Ok(stream)
}
