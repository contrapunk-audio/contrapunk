//! Stereo delay with feedback.
//!
//! Per-channel circular delay buffers with a shared delay time and
//! feedback coefficient. Wet signal = delayed sample; dry/wet crossfade
//! via `mix`.
//!
//! # Params (lock-free atomics)
//!
//! - `mix`: 0..1 dry/wet
//! - `time_ms`: 10..2000 ms delay time (clamped)
//! - `feedback`: 0..0.95 feedback amount (clamped below 1.0 to avoid runaway)
//! - `enabled`: when false `process()` is a pass-through

use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Arc;

use crate::chain::AudioBlock;

/// Maximum supported delay time in seconds. Buffers are sized for this
/// at construction and reused across `time_ms` changes.
const MAX_DELAY_SECS: f32 = 2.0;
/// Max channels we support on the delay (stereo DAW typical).
const MAX_CHANNELS: usize = 8;
/// Cap feedback below 1.0 so the tail always decays.
const MAX_FEEDBACK: f32 = 0.95;

pub struct DelayParams {
    enabled: AtomicBool,
    mix_ppt: AtomicU32,
    time_ms: AtomicU32,
    feedback_ppt: AtomicU32,
}

impl Default for DelayParams {
    fn default() -> Self {
        Self {
            enabled: AtomicBool::new(false),
            mix_ppt: AtomicU32::new(300),      // 0.30
            time_ms: AtomicU32::new(375),      // dotted 8th @ 120 bpm
            feedback_ppt: AtomicU32::new(350), // 0.35
        }
    }
}

impl DelayParams {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn enabled(&self) -> bool {
        self.enabled.load(Ordering::Relaxed)
    }
    pub fn mix(&self) -> f32 {
        self.mix_ppt.load(Ordering::Relaxed) as f32 / 1000.0
    }
    pub fn time_ms(&self) -> u32 {
        self.time_ms.load(Ordering::Relaxed)
    }
    pub fn feedback(&self) -> f32 {
        self.feedback_ppt.load(Ordering::Relaxed) as f32 / 1000.0
    }

    pub fn set_enabled(&self, v: bool) {
        self.enabled.store(v, Ordering::Relaxed);
    }
    pub fn set_mix(&self, v: f32) {
        self.mix_ppt
            .store((v.clamp(0.0, 1.0) * 1000.0) as u32, Ordering::Relaxed);
    }
    pub fn set_time_ms(&self, ms: u32) {
        let clamped = ms.clamp(10, (MAX_DELAY_SECS * 1000.0) as u32);
        self.time_ms.store(clamped, Ordering::Relaxed);
    }
    pub fn set_feedback(&self, v: f32) {
        self.feedback_ppt.store(
            (v.clamp(0.0, MAX_FEEDBACK) * 1000.0) as u32,
            Ordering::Relaxed,
        );
    }
}

/// Fixed-capacity circular buffer for one channel of delay.
struct DelayLine {
    buffer: Vec<f32>,
    write_idx: usize,
}

impl DelayLine {
    fn new(capacity: usize) -> Self {
        Self {
            buffer: vec![0.0; capacity.max(1)],
            write_idx: 0,
        }
    }

    fn reset(&mut self) {
        for s in &mut self.buffer {
            *s = 0.0;
        }
        self.write_idx = 0;
    }

    /// Read the sample `delay_samples` ago (clamped to buffer capacity),
    /// then write `input` at the current write head. Returns the delayed
    /// sample.
    #[inline]
    fn step(&mut self, input: f32, delay_samples: usize) -> f32 {
        let cap = self.buffer.len();
        let d = delay_samples.max(1).min(cap - 1);
        let read_idx = (self.write_idx + cap - d) % cap;
        let out = self.buffer[read_idx];
        self.buffer[self.write_idx] = input;
        self.write_idx += 1;
        if self.write_idx >= cap {
            self.write_idx = 0;
        }
        out
    }
}

pub struct Delay {
    params: Arc<DelayParams>,
    lines: Vec<DelayLine>,
    sample_rate: u32,
    name: String,
}

impl Delay {
    pub fn new(params: Arc<DelayParams>, sample_rate: u32) -> Self {
        let cap = (sample_rate as f32 * MAX_DELAY_SECS) as usize + 1;
        let lines = (0..MAX_CHANNELS).map(|_| DelayLine::new(cap)).collect();
        Self {
            params,
            lines,
            sample_rate,
            name: "Delay".to_string(),
        }
    }

    pub fn params(&self) -> Arc<DelayParams> {
        Arc::clone(&self.params)
    }

    fn resize_buffers(&mut self, sample_rate: u32) {
        let cap = (sample_rate as f32 * MAX_DELAY_SECS) as usize + 1;
        for line in self.lines.iter_mut() {
            line.buffer.clear();
            line.buffer.resize(cap, 0.0);
            line.write_idx = 0;
        }
    }
}

impl AudioBlock for Delay {
    fn name(&self) -> &str {
        &self.name
    }

    fn type_id(&self) -> &str {
        "builtin.delay"
    }

    fn process(&mut self, buffer: &mut [f32], channels: usize) {
        if channels == 0 {
            return;
        }
        if !self.params.enabled() {
            return;
        }

        let mix = self.params.mix();
        let dry = 1.0 - mix;
        let wet = mix;
        let feedback = self.params.feedback().min(MAX_FEEDBACK);
        let delay_samples =
            ((self.params.time_ms() as f32 / 1000.0) * self.sample_rate as f32) as usize;

        let use_channels = channels.min(self.lines.len());
        let frames = buffer.len() / channels;
        for i in 0..frames {
            let base = i * channels;
            for ch in 0..use_channels {
                let dry_sample = buffer[base + ch];
                // Feedback: write (input + feedback * previous delayed).
                // Read before write so current sample feeds on the next tap.
                let line = &mut self.lines[ch];
                let cap = line.buffer.len();
                let d = delay_samples.max(1).min(cap - 1);
                let read_idx = (line.write_idx + cap - d) % cap;
                let delayed = line.buffer[read_idx];
                line.buffer[line.write_idx] = dry_sample + delayed * feedback;
                line.write_idx += 1;
                if line.write_idx >= cap {
                    line.write_idx = 0;
                }
                buffer[base + ch] = dry_sample * dry + delayed * wet;
            }
        }
    }

    fn reset(&mut self) {
        for line in self.lines.iter_mut() {
            line.reset();
        }
    }

    fn set_sample_rate(&mut self, sample_rate: u32) {
        if sample_rate != self.sample_rate {
            self.sample_rate = sample_rate;
            self.resize_buffers(sample_rate);
        }
    }

    fn enabled(&self) -> bool {
        self.params.enabled()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mk_delay() -> Delay {
        Delay::new(Arc::new(DelayParams::default()), 48_000)
    }

    #[test]
    fn bypass_when_disabled() {
        let mut d = mk_delay();
        let mut buf = [0.5f32; 256];
        d.process(&mut buf, 2);
        assert!(buf.iter().all(|&x| x == 0.5));
    }

    #[test]
    fn impulse_produces_echo() {
        let p = Arc::new(DelayParams::default());
        p.set_enabled(true);
        p.set_mix(1.0); // full wet so echo is isolated
        p.set_time_ms(100);
        p.set_feedback(0.0);
        let mut d = Delay::new(Arc::clone(&p), 48_000);
        // 1 sec of silence with a single impulse at t=0 on each channel.
        let frames = 48_000;
        let channels = 2;
        let mut buf = vec![0.0f32; frames * channels];
        buf[0] = 1.0;
        buf[1] = 1.0;
        d.process(&mut buf, channels);
        // Expect a nonzero sample approximately 100 ms later = 4800 frames in.
        let echo_frame = 4800;
        let echo_sample = buf[echo_frame * channels].abs();
        assert!(
            echo_sample > 0.5,
            "expected an echo at ~100ms, got {}",
            echo_sample
        );
    }

    #[test]
    fn feedback_creates_multiple_echoes() {
        let p = Arc::new(DelayParams::default());
        p.set_enabled(true);
        p.set_mix(1.0);
        p.set_time_ms(50);
        p.set_feedback(0.7);
        let mut d = Delay::new(Arc::clone(&p), 48_000);
        let frames = 48_000;
        let channels = 2;
        let mut buf = vec![0.0f32; frames * channels];
        buf[0] = 1.0;
        buf[1] = 1.0;
        d.process(&mut buf, channels);
        // At least 4 echo peaks should be present across 1 sec with
        // 50 ms taps and 0.7 feedback.
        let tap_frames = 2400;
        let mut peaks = 0;
        for k in 1..=6 {
            let idx = k * tap_frames;
            if idx < frames && buf[idx * channels].abs() > 0.05 {
                peaks += 1;
            }
        }
        assert!(peaks >= 4, "expected ≥4 decaying echoes, saw {}", peaks);
    }

    #[test]
    fn reset_clears_tail() {
        let p = Arc::new(DelayParams::default());
        p.set_enabled(true);
        p.set_mix(1.0);
        p.set_feedback(0.5);
        let mut d = Delay::new(Arc::clone(&p), 48_000);
        let mut buf = vec![0.0f32; 4096];
        buf[0] = 1.0;
        d.process(&mut buf, 2);
        d.reset();
        let mut silence = vec![0.0f32; 8192];
        d.process(&mut silence, 2);
        let peak = silence.iter().fold(0.0f32, |a, b| a.max(b.abs()));
        assert!(peak < 1e-6, "expected silence after reset, got {}", peak);
    }

    #[test]
    fn mix_zero_is_pure_dry() {
        let p = Arc::new(DelayParams::default());
        p.set_enabled(true);
        p.set_mix(0.0);
        let mut d = Delay::new(Arc::clone(&p), 48_000);
        let mut buf = vec![0.25f32; 256];
        d.process(&mut buf, 2);
        assert!(
            buf.iter().all(|&x| (x - 0.25).abs() < 1e-6),
            "mix=0 should pass dry through unchanged"
        );
    }

    #[test]
    fn feedback_clamps_below_one() {
        let p = Arc::new(DelayParams::default());
        p.set_feedback(2.0);
        assert!(p.feedback() <= MAX_FEEDBACK);
    }
}
