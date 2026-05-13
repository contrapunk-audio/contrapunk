//! Stereo delay with feedback.
//!
//! Per-channel circular delay buffers with a shared delay time and
//! feedback coefficient. Wet signal = delayed sample; dry/wet crossfade
//! via `mix`.
//!
//! # Params (lock-free atomics)
//!
//! - `mix`: 0..1 dry/wet
//! - `time_ms`: 10..2000 ms delay time (clamped) — used in Free mode
//! - `feedback`: 0..0.95 feedback amount (clamped below 1.0 to avoid runaway)
//! - `enabled`: when false `process()` is a pass-through
//! - `sync_enabled`: when true, the effective delay time is derived
//!   from the transport BPM and `subdivision`, ignoring `time_ms`
//! - `subdivision`: musical subdivision index (see `Subdivision`)

use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU8, Ordering};
use std::sync::Arc;

use contrapunk_transport::Transport;

use crate::chain::AudioBlock;

/// Musical subdivisions for the tempo-synced delay. Indices match the
/// stored atomic encoding — never renumber once shipped (state file
/// compatibility). Beat fractions assume a quarter-note beat unit.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Subdivision {
    /// 1/4 (one beat).
    Quarter = 0,
    /// 1/8 dotted (3/16 = 0.75 beats).
    EighthDotted = 1,
    /// 1/8 (half a beat).
    Eighth = 2,
    /// 1/8 triplet (1/3 of a beat).
    EighthTriplet = 3,
    /// 1/16 (quarter of a beat).
    Sixteenth = 4,
    /// 1/16 triplet (1/6 of a beat).
    SixteenthTriplet = 5,
}

impl Subdivision {
    /// Length of one tap in beats (quarter-note units).
    pub fn beats(self) -> f32 {
        match self {
            Subdivision::Quarter => 1.0,
            Subdivision::EighthDotted => 0.75,
            Subdivision::Eighth => 0.5,
            Subdivision::EighthTriplet => 1.0 / 3.0,
            Subdivision::Sixteenth => 0.25,
            Subdivision::SixteenthTriplet => 1.0 / 6.0,
        }
    }

    pub fn from_u8(v: u8) -> Self {
        match v {
            0 => Subdivision::Quarter,
            1 => Subdivision::EighthDotted,
            2 => Subdivision::Eighth,
            3 => Subdivision::EighthTriplet,
            4 => Subdivision::Sixteenth,
            5 => Subdivision::SixteenthTriplet,
            _ => Subdivision::Quarter,
        }
    }

    /// Canonical wire-format identifier used by Tauri / WASM commands
    /// and the persisted UI state. Stable across releases.
    pub fn as_str(self) -> &'static str {
        match self {
            Subdivision::Quarter => "1/4",
            Subdivision::EighthDotted => "1/8d",
            Subdivision::Eighth => "1/8",
            Subdivision::EighthTriplet => "1/8t",
            Subdivision::Sixteenth => "1/16",
            Subdivision::SixteenthTriplet => "1/16t",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "1/4" => Some(Subdivision::Quarter),
            "1/8d" => Some(Subdivision::EighthDotted),
            "1/8" => Some(Subdivision::Eighth),
            "1/8t" => Some(Subdivision::EighthTriplet),
            "1/16" => Some(Subdivision::Sixteenth),
            "1/16t" => Some(Subdivision::SixteenthTriplet),
            _ => None,
        }
    }
}

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
    sync_enabled: AtomicBool,
    subdivision: AtomicU8,
}

impl Default for DelayParams {
    fn default() -> Self {
        Self {
            enabled: AtomicBool::new(false),
            mix_ppt: AtomicU32::new(300),      // 0.30
            time_ms: AtomicU32::new(375),      // dotted 8th @ 120 bpm
            feedback_ppt: AtomicU32::new(350), // 0.35
            sync_enabled: AtomicBool::new(false),
            subdivision: AtomicU8::new(Subdivision::EighthDotted as u8),
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
    pub fn sync_enabled(&self) -> bool {
        self.sync_enabled.load(Ordering::Relaxed)
    }
    pub fn subdivision(&self) -> Subdivision {
        Subdivision::from_u8(self.subdivision.load(Ordering::Relaxed))
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
    pub fn set_sync_enabled(&self, v: bool) {
        self.sync_enabled.store(v, Ordering::Relaxed);
    }
    pub fn set_subdivision(&self, s: Subdivision) {
        self.subdivision.store(s as u8, Ordering::Relaxed);
    }

    /// Effective delay length in milliseconds. In sync mode, derived
    /// from the transport BPM and the selected subdivision; otherwise
    /// the free-running `time_ms` value. Clamped to the buffer's
    /// `MAX_DELAY_SECS`.
    pub fn effective_time_ms(&self, transport: &Transport) -> u32 {
        if self.sync_enabled() {
            let bpm = transport.bpm().max(1.0);
            let beats = self.subdivision().beats() as f64;
            let ms = beats * 60_000.0 / bpm;
            (ms.round() as u32).clamp(1, (MAX_DELAY_SECS * 1000.0) as u32)
        } else {
            self.time_ms()
        }
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
    transport: Option<Arc<Transport>>,
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
            transport: None,
            lines,
            sample_rate,
            name: "Delay".to_string(),
        }
    }

    /// Build a Delay that knows about the transport, so the sync mode
    /// can derive tap length from BPM. Pass-through to `new` otherwise.
    pub fn with_transport(
        params: Arc<DelayParams>,
        transport: Arc<Transport>,
        sample_rate: u32,
    ) -> Self {
        let mut d = Self::new(params, sample_rate);
        d.transport = Some(transport);
        d
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
        // Sync mode uses the transport BPM if available; otherwise the
        // free-running time_ms value. Hard re-tap on BPM change — the
        // new tap takes effect on the next block. Buffer contents are
        // preserved; only the read offset moves, so the existing tail
        // continues to play out at the old length until decayed.
        let time_ms = match &self.transport {
            Some(t) if self.params.sync_enabled() => self.params.effective_time_ms(t),
            _ => self.params.time_ms(),
        };
        let delay_samples = ((time_ms as f32 / 1000.0) * self.sample_rate as f32) as usize;

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

    #[test]
    fn subdivision_round_trips_through_str() {
        for s in [
            Subdivision::Quarter,
            Subdivision::EighthDotted,
            Subdivision::Eighth,
            Subdivision::EighthTriplet,
            Subdivision::Sixteenth,
            Subdivision::SixteenthTriplet,
        ] {
            assert_eq!(Subdivision::from_str(s.as_str()), Some(s));
            assert_eq!(Subdivision::from_u8(s as u8), s);
        }
        // Bogus tags fall back to None (caller keeps current setting).
        assert_eq!(Subdivision::from_str("1/2"), None);
        // Bogus indices fall back to Quarter — never crash on a junk byte.
        assert_eq!(Subdivision::from_u8(99), Subdivision::Quarter);
    }

    #[test]
    fn effective_time_ms_matches_daw_subdivisions_at_120bpm() {
        let p = DelayParams::default();
        p.set_sync_enabled(true);
        let t = Transport::new(48_000);
        // Transport defaults to 120 bpm → 1 beat = 500 ms.
        let cases = [
            (Subdivision::Quarter, 500u32),
            (Subdivision::EighthDotted, 375u32),
            (Subdivision::Eighth, 250u32),
            (Subdivision::Sixteenth, 125u32),
        ];
        for (sub, expected) in cases {
            p.set_subdivision(sub);
            let actual = p.effective_time_ms(&t);
            assert_eq!(
                actual, expected,
                "subdivision {:?} at 120 bpm should be {} ms, got {}",
                sub, expected, actual
            );
        }
        // Triplets round to the nearest ms; check they land on Ableton's
        // canonical values (167 / 83).
        p.set_subdivision(Subdivision::EighthTriplet);
        assert_eq!(p.effective_time_ms(&t), 167);
        p.set_subdivision(Subdivision::SixteenthTriplet);
        assert_eq!(p.effective_time_ms(&t), 83);
    }

    #[test]
    fn effective_time_ms_tracks_bpm_changes() {
        let p = DelayParams::default();
        p.set_sync_enabled(true);
        p.set_subdivision(Subdivision::Quarter);
        let t = Transport::new(48_000);
        // 120 bpm → 1/4 = 500 ms; 60 bpm → 1/4 = 1000 ms.
        assert_eq!(p.effective_time_ms(&t), 500);
        t.set_bpm(60.0);
        assert_eq!(p.effective_time_ms(&t), 1000);
        t.set_bpm(240.0);
        assert_eq!(p.effective_time_ms(&t), 250);
    }

    #[test]
    fn sync_disabled_keeps_free_time_ms() {
        let p = DelayParams::default();
        p.set_sync_enabled(false);
        p.set_time_ms(750);
        let t = Transport::new(48_000);
        // Even at 60 bpm, free mode ignores the transport entirely.
        t.set_bpm(60.0);
        assert_eq!(p.effective_time_ms(&t), 750);
    }
}
