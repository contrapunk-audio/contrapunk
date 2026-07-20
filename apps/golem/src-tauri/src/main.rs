//! Standalone Tauri host for Golem.
//!
//! The UI sends controls over Tauri commands. This backend owns cpal
//! input/output streams and keeps realtime audio on Rust threads.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::Duration;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{SampleFormat, Stream};
use golem_core::params::AtomicF32;
use golem_core::{ClockSnapshot, Engine, EngineParams, FollowInput, Follower, SharedParams, Style};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager, State};

struct GolemState {
    params: Arc<SharedParams>,
    runtime: Mutex<Option<RuntimeHandle>>,
}

impl GolemState {
    fn new() -> Self {
        Self {
            params: Arc::new(SharedParams::default()),
            runtime: Mutex::new(None),
        }
    }
}

struct RuntimeHandle {
    stop: Arc<AtomicBool>,
    follow: Arc<FollowAtomics>,
    join: Option<thread::JoinHandle<()>>,
}

impl RuntimeHandle {
    fn stop(mut self) {
        self.stop.store(true, Ordering::SeqCst);
        if let Some(join) = self.join.take() {
            let _ = join.join();
        }
    }
}

impl Drop for RuntimeHandle {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::SeqCst);
    }
}

#[derive(Clone, Debug, Deserialize)]
struct StartConfig {
    input_device: Option<String>,
    input_channel: Option<usize>,
}

#[derive(Clone, Debug, Deserialize)]
struct UiParams {
    bpm: f32,
    style: String,
    intensity: f32,
    complexity: f32,
    swing: f32,
    fill_amount: f32,
    follow_amount: f32,
    master_gain: f32,
}

impl UiParams {
    fn to_engine_params(&self) -> EngineParams {
        EngineParams {
            bpm: self.bpm,
            style: Style::parse(&self.style),
            intensity: self.intensity,
            complexity: self.complexity,
            swing: self.swing,
            fill_amount: self.fill_amount,
            follow_amount: self.follow_amount,
            master_gain: self.master_gain,
        }
    }
}

#[derive(Clone, Debug, Serialize)]
struct ParamsPayload {
    bpm: f32,
    style: String,
    intensity: f32,
    complexity: f32,
    swing: f32,
    fill_amount: f32,
    follow_amount: f32,
    master_gain: f32,
}

impl From<EngineParams> for ParamsPayload {
    fn from(params: EngineParams) -> Self {
        Self {
            bpm: params.bpm,
            style: params.style.as_str().to_string(),
            intensity: params.intensity,
            complexity: params.complexity,
            swing: params.swing,
            fill_amount: params.fill_amount,
            follow_amount: params.follow_amount,
            master_gain: params.master_gain,
        }
    }
}

#[derive(Clone, Debug, Serialize)]
struct EngineStatePayload {
    running: bool,
    params: ParamsPayload,
}

#[derive(Clone, Debug, Serialize)]
struct MeterPayload {
    rms: f32,
    onset: f32,
    density: f32,
    confidence: f32,
    running: bool,
    input_device: String,
    input_channel: usize,
    active_channel: usize,
    channel_levels: Vec<f32>,
    input_blocks: u64,
    raw_rms: f32,
    raw_peak: f32,
    raw_rms_db: f32,
    raw_peak_db: f32,
    normalized_energy: f32,
    energy_fast: f32,
    energy_slow: f32,
    noise_floor_db: f32,
    clipping: bool,
}

struct FollowAtomics {
    rms: AtomicF32,
    onset: AtomicF32,
    density: AtomicF32,
    confidence: AtomicF32,
    raw_rms: AtomicF32,
    raw_peak: AtomicF32,
    raw_rms_db: AtomicF32,
    raw_peak_db: AtomicF32,
    normalized_energy: AtomicF32,
    energy_fast: AtomicF32,
    energy_slow: AtomicF32,
    noise_floor_db: AtomicF32,
    clipping: AtomicBool,
    channel_count: AtomicUsize,
    active_channel: AtomicUsize,
    channel_levels: [AtomicF32; MAX_METER_CHANNELS],
    input_blocks: AtomicU64,
    input_device: String,
    input_channel: AtomicUsize,
}

impl FollowAtomics {
    fn new() -> Self {
        Self {
            rms: AtomicF32::new(0.0),
            onset: AtomicF32::new(0.0),
            density: AtomicF32::new(0.0),
            confidence: AtomicF32::new(0.0),
            raw_rms: AtomicF32::new(0.0),
            raw_peak: AtomicF32::new(0.0),
            raw_rms_db: AtomicF32::new(-120.0),
            raw_peak_db: AtomicF32::new(-120.0),
            normalized_energy: AtomicF32::new(0.0),
            energy_fast: AtomicF32::new(0.0),
            energy_slow: AtomicF32::new(0.0),
            noise_floor_db: AtomicF32::new(-80.0),
            clipping: AtomicBool::new(false),
            channel_count: AtomicUsize::new(0),
            active_channel: AtomicUsize::new(0),
            channel_levels: std::array::from_fn(|_| AtomicF32::new(0.0)),
            input_blocks: AtomicU64::new(0),
            input_device: String::new(),
            input_channel: AtomicUsize::new(0),
        }
    }

    fn with_input(input_device: String, input_channel: usize) -> Self {
        Self {
            input_device,
            input_channel: AtomicUsize::new(input_channel),
            ..Self::new()
        }
    }

    fn store_frame(&self, frame: InputFeatures) {
        self.input_blocks.fetch_add(1, Ordering::Relaxed);
        self.rms.store(frame.follow.guitar_rms);
        self.onset.store(frame.follow.onset_strength);
        self.density.store(frame.follow.strum_density);
        self.confidence.store(frame.follow.confidence);
        self.raw_rms.store(frame.follow.raw_rms);
        self.raw_peak.store(frame.follow.raw_peak);
        self.raw_rms_db.store(frame.follow.raw_rms_db);
        self.raw_peak_db.store(frame.follow.raw_peak_db);
        self.normalized_energy.store(frame.follow.normalized_energy);
        self.energy_fast.store(frame.follow.energy_fast);
        self.energy_slow.store(frame.follow.energy_slow);
        self.noise_floor_db.store(frame.follow.noise_floor_db);
        self.clipping
            .store(frame.follow.clipping, Ordering::Relaxed);
        self.channel_count.store(
            frame.channel_count.min(MAX_METER_CHANNELS),
            Ordering::Relaxed,
        );
        self.active_channel.store(
            frame
                .active_channel
                .min(MAX_METER_CHANNELS.saturating_sub(1)),
            Ordering::Relaxed,
        );
        for idx in 0..MAX_METER_CHANNELS {
            let value = if idx < frame.channel_count {
                frame.channel_levels[idx]
            } else {
                0.0
            };
            self.channel_levels[idx].store(value);
        }
    }

    fn snapshot(&self) -> FollowInput {
        FollowInput {
            guitar_rms: self.rms.load(),
            onset_strength: self.onset.load(),
            strum_density: self.density.load(),
            confidence: self.confidence.load(),
            raw_rms: self.raw_rms.load(),
            raw_peak: self.raw_peak.load(),
            raw_rms_db: self.raw_rms_db.load(),
            raw_peak_db: self.raw_peak_db.load(),
            normalized_energy: self.normalized_energy.load(),
            energy_fast: self.energy_fast.load(),
            energy_slow: self.energy_slow.load(),
            noise_floor_db: self.noise_floor_db.load(),
            clipping: self.clipping.load(Ordering::Relaxed),
        }
    }
}

fn meter_payload(follow: &FollowAtomics, running: bool) -> MeterPayload {
    let snap = follow.snapshot();
    let channel_count = follow
        .channel_count
        .load(Ordering::Relaxed)
        .min(MAX_METER_CHANNELS);
    let channel_levels = (0..channel_count)
        .map(|idx| follow.channel_levels[idx].load())
        .collect::<Vec<_>>();

    MeterPayload {
        rms: snap.guitar_rms,
        onset: snap.onset_strength,
        density: snap.strum_density,
        confidence: snap.confidence,
        running,
        input_device: follow.input_device.clone(),
        input_channel: follow.input_channel.load(Ordering::Relaxed),
        active_channel: follow.active_channel.load(Ordering::Relaxed),
        channel_levels,
        input_blocks: follow.input_blocks.load(Ordering::Relaxed),
        raw_rms: snap.raw_rms,
        raw_peak: snap.raw_peak,
        raw_rms_db: snap.raw_rms_db,
        raw_peak_db: snap.raw_peak_db,
        normalized_energy: snap.normalized_energy,
        energy_fast: snap.energy_fast,
        energy_slow: snap.energy_slow,
        noise_floor_db: snap.noise_floor_db,
        clipping: snap.clipping,
    }
}

#[tauri::command]
fn list_audio_inputs() -> Result<Vec<String>, String> {
    let host = cpal::default_host();
    let devices = host
        .input_devices()
        .map_err(|e| format!("Failed to enumerate input devices: {e}"))?;
    Ok(devices.map(|d| device_label(&d)).collect())
}

#[tauri::command]
fn get_engine_state(state: State<GolemState>) -> Result<EngineStatePayload, String> {
    let running = state.runtime.lock().map_err(|e| e.to_string())?.is_some();
    Ok(EngineStatePayload {
        running,
        params: state.params.snapshot().into(),
    })
}

#[tauri::command]
fn set_engine_params(
    params: UiParams,
    state: State<GolemState>,
) -> Result<EngineStatePayload, String> {
    state.params.set_snapshot(params.to_engine_params());
    get_engine_state(state)
}

#[tauri::command]
fn set_input_channel(channel: usize, state: State<GolemState>) -> Result<(), String> {
    if let Some(handle) = state.runtime.lock().map_err(|e| e.to_string())?.as_ref() {
        handle
            .follow
            .input_channel
            .store(channel, Ordering::Relaxed);
    }
    Ok(())
}

#[tauri::command]
fn start_engine(
    config: StartConfig,
    app: AppHandle,
    state: State<GolemState>,
) -> Result<EngineStatePayload, String> {
    let mut runtime = state.runtime.lock().map_err(|e| e.to_string())?;
    if runtime.is_some() {
        return Err("Golem is already running".into());
    }

    let input_device = config.input_device.clone().unwrap_or_default();
    let input_channel = config.input_channel.unwrap_or(0);
    let stop = Arc::new(AtomicBool::new(false));
    let follow = Arc::new(FollowAtomics::with_input(input_device, input_channel));
    let params = Arc::clone(&state.params);
    let stop_for_thread = Arc::clone(&stop);
    let follow_for_thread = Arc::clone(&follow);
    let app_for_thread = app.clone();

    let (ready_tx, ready_rx) = mpsc::channel::<Result<(), String>>();
    let join = thread::spawn(move || {
        let result = run_audio_thread(
            config,
            params,
            follow_for_thread,
            Arc::clone(&stop_for_thread),
            app_for_thread,
            ready_tx.clone(),
        );
        if let Err(err) = result {
            let _ = ready_tx.send(Err(err));
        }
    });

    match ready_rx.recv_timeout(Duration::from_secs(4)) {
        Ok(Ok(())) => {
            *runtime = Some(RuntimeHandle {
                stop,
                follow,
                join: Some(join),
            });
            Ok(EngineStatePayload {
                running: true,
                params: state.params.snapshot().into(),
            })
        }
        Ok(Err(err)) => {
            stop.store(true, Ordering::SeqCst);
            let _ = join.join();
            Err(err)
        }
        Err(err) => {
            stop.store(true, Ordering::SeqCst);
            let _ = join.join();
            Err(format!("Audio thread did not start: {err}"))
        }
    }
}

#[tauri::command]
fn stop_engine(state: State<GolemState>) -> Result<EngineStatePayload, String> {
    let handle = state.runtime.lock().map_err(|e| e.to_string())?.take();
    if let Some(handle) = handle {
        handle.stop();
    }
    Ok(EngineStatePayload {
        running: false,
        params: state.params.snapshot().into(),
    })
}

fn run_audio_thread(
    config: StartConfig,
    params: Arc<SharedParams>,
    follow: Arc<FollowAtomics>,
    stop: Arc<AtomicBool>,
    app: AppHandle,
    ready_tx: mpsc::Sender<Result<(), String>>,
) -> Result<(), String> {
    let host = cpal::default_host();

    let output_device = host
        .default_output_device()
        .ok_or_else(|| "No default audio output device".to_string())?;
    let output_config = output_device
        .default_output_config()
        .map_err(|e| format!("Default output config error: {e}"))?;

    let sample_rate = output_config.sample_rate();
    let channels = output_config.channels() as usize;
    let sample_format = output_config.sample_format();
    let stream_config: cpal::StreamConfig = output_config.into();
    let err_fn = |err: cpal::StreamError| eprintln!("[golem] audio stream error: {err}");

    eprintln!(
        "[golem] output device='{}' sample_rate={}Hz channels={} format={:?}",
        device_label(&output_device),
        sample_rate,
        channels,
        sample_format,
    );

    let output_stream = match sample_format {
        SampleFormat::F32 => build_output_stream_f32(
            &output_device,
            &stream_config,
            channels,
            sample_rate,
            Arc::clone(&params),
            Arc::clone(&follow),
            err_fn,
        ),
        SampleFormat::I16 => build_output_stream_i16(
            &output_device,
            &stream_config,
            channels,
            sample_rate,
            Arc::clone(&params),
            Arc::clone(&follow),
            err_fn,
        ),
        SampleFormat::U16 => build_output_stream_u16(
            &output_device,
            &stream_config,
            channels,
            sample_rate,
            Arc::clone(&params),
            Arc::clone(&follow),
            err_fn,
        ),
        other => return Err(format!("Unsupported output sample format: {other:?}")),
    }
    .map_err(|e| format!("Failed to build output stream: {e}"))?;

    let input_stream = build_input_stream(&host, config, Arc::clone(&follow))?;

    output_stream
        .play()
        .map_err(|e| format!("Failed to start output stream: {e}"))?;
    if let Some(stream) = input_stream.as_ref() {
        stream
            .play()
            .map_err(|e| format!("Failed to start input stream: {e}"))?;
    }

    let telemetry_stop = Arc::clone(&stop);
    let telemetry_follow = Arc::clone(&follow);
    let telemetry_app = app.clone();
    let telemetry = thread::spawn(move || {
        while !telemetry_stop.load(Ordering::SeqCst) {
            let _ = telemetry_app.emit("golem-meter", meter_payload(&telemetry_follow, true));
            thread::sleep(Duration::from_millis(100));
        }
        let _ = telemetry_app.emit("golem-meter", meter_payload(&telemetry_follow, false));
    });

    let _ = ready_tx.send(Ok(()));

    // Keep streams alive on this owner thread until stop is requested.
    while !stop.load(Ordering::SeqCst) {
        thread::sleep(Duration::from_millis(40));
    }

    drop(input_stream);
    drop(output_stream);
    let _ = telemetry.join();
    Ok(())
}

fn render_f32_block(
    engine: &mut Engine,
    params: &Arc<SharedParams>,
    follow: &Arc<FollowAtomics>,
    sample_rate: u32,
    sample_pos: &mut u64,
    data: &mut [f32],
    channels: usize,
) {
    for sample in data.iter_mut() {
        *sample = 0.0;
    }

    let frames = data.len() / channels.max(1);
    let snapshot = params.snapshot();
    engine.set_params(snapshot);
    engine.process(
        ClockSnapshot {
            sample_pos: *sample_pos,
            sample_rate,
            bpm: snapshot.bpm as f64,
            playing: true,
        },
        follow.snapshot(),
        data,
        channels,
    );
    *sample_pos = (*sample_pos).wrapping_add(frames as u64);
}

fn build_output_stream_f32(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    channels: usize,
    sample_rate: u32,
    params: Arc<SharedParams>,
    follow: Arc<FollowAtomics>,
    err_fn: fn(cpal::StreamError),
) -> Result<Stream, cpal::BuildStreamError> {
    let mut engine = Engine::new();
    engine.prepare(sample_rate, 1024);
    let mut sample_pos = 0u64;
    device.build_output_stream(
        config,
        move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
            render_f32_block(
                &mut engine,
                &params,
                &follow,
                sample_rate,
                &mut sample_pos,
                data,
                channels,
            );
        },
        err_fn,
        None,
    )
}

fn build_output_stream_i16(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    channels: usize,
    sample_rate: u32,
    params: Arc<SharedParams>,
    follow: Arc<FollowAtomics>,
    err_fn: fn(cpal::StreamError),
) -> Result<Stream, cpal::BuildStreamError> {
    let mut engine = Engine::new();
    engine.prepare(sample_rate, 1024);
    let mut sample_pos = 0u64;
    let channels = channels.max(1);
    let mut scratch = vec![0.0f32; MAX_CONVERTED_OUTPUT_SAMPLES.max(channels)];
    let chunk_samples = (scratch.len() / channels).max(1) * channels;
    device.build_output_stream(
        config,
        move |data: &mut [i16], _: &cpal::OutputCallbackInfo| {
            for chunk in data.chunks_mut(chunk_samples) {
                let scratch = &mut scratch[..chunk.len()];
                render_f32_block(
                    &mut engine,
                    &params,
                    &follow,
                    sample_rate,
                    &mut sample_pos,
                    scratch,
                    channels,
                );
                for (dst, src) in chunk.iter_mut().zip(scratch.iter()) {
                    *dst = (src.clamp(-1.0, 1.0) * i16::MAX as f32) as i16;
                }
            }
        },
        err_fn,
        None,
    )
}

fn build_output_stream_u16(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    channels: usize,
    sample_rate: u32,
    params: Arc<SharedParams>,
    follow: Arc<FollowAtomics>,
    err_fn: fn(cpal::StreamError),
) -> Result<Stream, cpal::BuildStreamError> {
    let mut engine = Engine::new();
    engine.prepare(sample_rate, 1024);
    let mut sample_pos = 0u64;
    let channels = channels.max(1);
    let mut scratch = vec![0.0f32; MAX_CONVERTED_OUTPUT_SAMPLES.max(channels)];
    let chunk_samples = (scratch.len() / channels).max(1) * channels;
    device.build_output_stream(
        config,
        move |data: &mut [u16], _: &cpal::OutputCallbackInfo| {
            for chunk in data.chunks_mut(chunk_samples) {
                let scratch = &mut scratch[..chunk.len()];
                render_f32_block(
                    &mut engine,
                    &params,
                    &follow,
                    sample_rate,
                    &mut sample_pos,
                    scratch,
                    channels,
                );
                for (dst, src) in chunk.iter_mut().zip(scratch.iter()) {
                    let normalized = src.clamp(-1.0, 1.0) * 0.5 + 0.5;
                    *dst = (normalized * u16::MAX as f32) as u16;
                }
            }
        },
        err_fn,
        None,
    )
}

fn device_label(device: &cpal::Device) -> String {
    device
        .description()
        .map(|description| description.name().to_string())
        .unwrap_or_default()
}

fn build_input_stream(
    host: &cpal::Host,
    config: StartConfig,
    follow: Arc<FollowAtomics>,
) -> Result<Option<Stream>, String> {
    let input_device = match config
        .input_device
        .as_deref()
        .filter(|s| !s.trim().is_empty())
    {
        Some(name) => host
            .input_devices()
            .map_err(|e| format!("Failed to enumerate input devices: {e}"))?
            .find(|device| device_label(device) == name)
            .or_else(|| host.default_input_device()),
        None => host.default_input_device(),
    };

    let Some(input_device) = input_device else {
        eprintln!("[golem] no input device; drummer will run without guitar-follow");
        return Ok(None);
    };

    let input_config = input_device
        .default_input_config()
        .map_err(|e| format!("Default input config error: {e}"))?;

    let input_sample_rate = input_config.sample_rate();
    let input_channels = input_config.channels() as usize;
    let input_format = input_config.sample_format();
    let input_channel = config
        .input_channel
        .unwrap_or(0)
        .min(input_channels.saturating_sub(1));
    let stream_config = cpal::StreamConfig {
        channels: input_config.channels(),
        sample_rate: input_sample_rate,
        buffer_size: cpal::BufferSize::Fixed(128),
    };

    eprintln!(
        "[golem] input-follow: device='{}' channel={} sample_rate={}Hz channels={} format={:?}",
        device_label(&input_device),
        input_channel,
        input_sample_rate,
        input_channels,
        input_format,
    );

    let stream = match input_format {
        SampleFormat::F32 => {
            let mut extractor = GuitarFeatureExtractor::new(input_sample_rate);
            input_device.build_input_stream(
                &stream_config,
                move |data: &[f32], _: &cpal::InputCallbackInfo| {
                    let channel = follow
                        .input_channel
                        .load(Ordering::Relaxed)
                        .min(input_channels.saturating_sub(1));
                    let features = extractor.process_interleaved_f32(data, input_channels, channel);
                    follow.store_frame(features);
                },
                |err| eprintln!("[golem] input stream error: {err}"),
                None,
            )
        }
        SampleFormat::I16 => {
            let mut extractor = GuitarFeatureExtractor::new(input_sample_rate);
            input_device.build_input_stream(
                &stream_config,
                move |data: &[i16], _: &cpal::InputCallbackInfo| {
                    let channel = follow
                        .input_channel
                        .load(Ordering::Relaxed)
                        .min(input_channels.saturating_sub(1));
                    let features = extractor.process_interleaved_i16(data, input_channels, channel);
                    follow.store_frame(features);
                },
                |err| eprintln!("[golem] input stream error: {err}"),
                None,
            )
        }
        SampleFormat::U16 => {
            let mut extractor = GuitarFeatureExtractor::new(input_sample_rate);
            input_device.build_input_stream(
                &stream_config,
                move |data: &[u16], _: &cpal::InputCallbackInfo| {
                    let channel = follow
                        .input_channel
                        .load(Ordering::Relaxed)
                        .min(input_channels.saturating_sub(1));
                    let features = extractor.process_interleaved_u16(data, input_channels, channel);
                    follow.store_frame(features);
                },
                |err| eprintln!("[golem] input stream error: {err}"),
                None,
            )
        }
        other => return Err(format!("Unsupported input sample format: {other:?}")),
    }
    .map_err(|e| format!("Failed to build input stream: {e}"))?;

    Ok(Some(stream))
}

/// Golem-specific realtime input feature extractor.
///
/// Keep this intentionally simple for v0.1: no pitch tracking, MIDI events,
/// heap allocation, or locks in the cpal input callback. The drummer only
/// needs RMS/onset/density to follow the player's energy.
const MAX_INPUT_FRAMES: usize = 8192;
const MAX_CONVERTED_OUTPUT_SAMPLES: usize = 16_384;
const MAX_METER_CHANNELS: usize = 16;

struct InputFeatures {
    follow: FollowInput,
    channel_levels: [f32; MAX_METER_CHANNELS],
    channel_count: usize,
    active_channel: usize,
}

impl Default for InputFeatures {
    fn default() -> Self {
        Self {
            follow: FollowInput::default(),
            channel_levels: [0.0; MAX_METER_CHANNELS],
            channel_count: 0,
            active_channel: 0,
        }
    }
}

struct GuitarFeatureExtractor {
    follower: Follower,
    mono: [f32; MAX_INPUT_FRAMES],
}

impl GuitarFeatureExtractor {
    fn new(sample_rate: u32) -> Self {
        Self {
            follower: Follower::new(sample_rate),
            mono: [0.0; MAX_INPUT_FRAMES],
        }
    }

    fn process_interleaved_f32(
        &mut self,
        input: &[f32],
        channels: usize,
        channel: usize,
    ) -> InputFeatures {
        if channels == 0 || input.is_empty() {
            return InputFeatures::default();
        }

        let frames = (input.len() / channels).min(MAX_INPUT_FRAMES);
        let (levels, channel_count, active_channel) =
            choose_active_channel(channels, channel, frames, |frame_idx, ch| {
                input[frame_idx * channels + ch]
            });
        for (idx, frame) in input.chunks(channels).take(frames).enumerate() {
            self.mono[idx] = frame.get(active_channel).copied().unwrap_or(0.0);
        }
        self.process_mono(frames, levels, channel_count, active_channel)
    }

    fn process_interleaved_i16(
        &mut self,
        input: &[i16],
        channels: usize,
        channel: usize,
    ) -> InputFeatures {
        if channels == 0 || input.is_empty() {
            return InputFeatures::default();
        }

        let frames = (input.len() / channels).min(MAX_INPUT_FRAMES);
        let (levels, channel_count, active_channel) =
            choose_active_channel(channels, channel, frames, |frame_idx, ch| {
                input[frame_idx * channels + ch] as f32 / i16::MAX as f32
            });
        for (idx, frame) in input.chunks(channels).take(frames).enumerate() {
            self.mono[idx] =
                frame.get(active_channel).copied().unwrap_or(0) as f32 / i16::MAX as f32;
        }
        self.process_mono(frames, levels, channel_count, active_channel)
    }

    fn process_interleaved_u16(
        &mut self,
        input: &[u16],
        channels: usize,
        channel: usize,
    ) -> InputFeatures {
        if channels == 0 || input.is_empty() {
            return InputFeatures::default();
        }

        let frames = (input.len() / channels).min(MAX_INPUT_FRAMES);
        let (levels, channel_count, active_channel) =
            choose_active_channel(channels, channel, frames, |frame_idx, ch| {
                let x = input[frame_idx * channels + ch];
                (x as f32 / u16::MAX as f32) * 2.0 - 1.0
            });
        for (idx, frame) in input.chunks(channels).take(frames).enumerate() {
            let x = frame.get(active_channel).copied().unwrap_or(u16::MAX / 2);
            self.mono[idx] = (x as f32 / u16::MAX as f32) * 2.0 - 1.0;
        }
        self.process_mono(frames, levels, channel_count, active_channel)
    }

    fn process_mono(
        &mut self,
        frames: usize,
        channel_levels: [f32; MAX_METER_CHANNELS],
        channel_count: usize,
        active_channel: usize,
    ) -> InputFeatures {
        if frames == 0 {
            return InputFeatures::default();
        }

        let follow = self
            .follower
            .process_interleaved_f32(&self.mono[..frames], 1, 0, 1.0);

        InputFeatures {
            follow,
            channel_levels,
            channel_count,
            active_channel,
        }
    }
}

fn choose_active_channel<F>(
    channels: usize,
    preferred_channel: usize,
    frames: usize,
    mut sample_at: F,
) -> ([f32; MAX_METER_CHANNELS], usize, usize)
where
    F: FnMut(usize, usize) -> f32,
{
    let channel_count = channels.min(MAX_METER_CHANNELS);
    let mut sums = [0.0f32; MAX_METER_CHANNELS];
    if frames == 0 || channel_count == 0 {
        return ([0.0; MAX_METER_CHANNELS], 0, 0);
    }

    for frame_idx in 0..frames {
        for ch in 0..channel_count {
            let x = sample_at(frame_idx, ch);
            sums[ch] += x * x;
        }
    }

    let mut levels = [0.0f32; MAX_METER_CHANNELS];
    for ch in 0..channel_count {
        levels[ch] = (sums[ch] / frames as f32).sqrt().clamp(0.0, 1.0);
    }

    let active_channel = preferred_channel.min(channel_count - 1);
    (levels, channel_count, active_channel)
}

fn main() {
    tauri::Builder::default()
        .manage(GolemState::new())
        .setup(|app| {
            let state = app.state::<GolemState>();
            eprintln!(
                "[golem] standalone host ready: {:?}",
                state.params.snapshot()
            );
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            list_audio_inputs,
            get_engine_state,
            set_engine_params,
            set_input_channel,
            start_engine,
            stop_engine,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Golem");
}
