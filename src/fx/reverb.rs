//! Freeverb reverb block.
//!
//! Classic Schroeder/Moorer algorithm with 8 parallel comb filters
//! feeding 4 serial allpass filters per channel. Public-domain design
//! originally by Jezar Wakefield (2000). Implementation below is a
//! pragmatic port: mono→stereo (both channels get the same wet signal
//! with a slight stereo-spread tap offset).
//!
//! # Params (lock-free atomics)
//!
//! - `mix`: 0..1 dry/wet
//! - `room_size`: 0..1 comb feedback; higher = longer tail
//! - `damping`: 0..1 high-freq damping in the feedback loop; higher
//!   = darker tail
//! - `width`: 0..1 stereo width (currently mono wet — reserved for
//!   future stereo spread)
//! - `enabled`: atomic bool, when false `process()` is a pass-through

use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Arc;

use crate::chain::AudioBlock;

/// Reference sample rate for the comb/allpass tap lengths. At other
/// sample rates we scale the tap lengths proportionally.
const REF_SR: f32 = 44_100.0;

/// Comb filter tap lengths at the reference sample rate (Freeverb).
const COMB_TAPS: [usize; 8] = [1116, 1188, 1277, 1356, 1422, 1491, 1557, 1617];
/// Allpass filter tap lengths.
const ALLPASS_TAPS: [usize; 4] = [556, 441, 341, 225];
/// Allpass feedback coefficient (Freeverb classic).
const ALLPASS_FB: f32 = 0.5;
/// Comb-filter input gain to keep things from clipping.
const FIXED_GAIN: f32 = 0.015;

pub struct ReverbParams {
    enabled: AtomicBool,
    /// 0..1000 as parts per thousand.
    mix_ppt: AtomicU32,
    room_size_ppt: AtomicU32,
    damping_ppt: AtomicU32,
}

impl Default for ReverbParams {
    fn default() -> Self {
        Self {
            // FTUX: reverb on by default so the first note sounds like
            // an instrument, not a dry sine. Mix kept conservative so it
            // doesn't smear the contrapuntal lines.
            enabled: AtomicBool::new(true),
            mix_ppt: AtomicU32::new(250),       // 0.25 — subtle wash
            room_size_ppt: AtomicU32::new(550), // 0.55 — medium space
            damping_ppt: AtomicU32::new(500),   // 0.50
        }
    }
}

impl ReverbParams {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn enabled(&self) -> bool {
        self.enabled.load(Ordering::Relaxed)
    }
    pub fn mix(&self) -> f32 {
        self.mix_ppt.load(Ordering::Relaxed) as f32 / 1000.0
    }
    pub fn room_size(&self) -> f32 {
        self.room_size_ppt.load(Ordering::Relaxed) as f32 / 1000.0
    }
    pub fn damping(&self) -> f32 {
        self.damping_ppt.load(Ordering::Relaxed) as f32 / 1000.0
    }

    pub fn set_enabled(&self, v: bool) {
        self.enabled.store(v, Ordering::Relaxed);
    }
    pub fn set_mix(&self, v: f32) {
        self.mix_ppt
            .store((v.clamp(0.0, 1.0) * 1000.0) as u32, Ordering::Relaxed);
    }
    pub fn set_room_size(&self, v: f32) {
        self.room_size_ppt
            .store((v.clamp(0.0, 1.0) * 1000.0) as u32, Ordering::Relaxed);
    }
    pub fn set_damping(&self, v: f32) {
        self.damping_ppt
            .store((v.clamp(0.0, 1.0) * 1000.0) as u32, Ordering::Relaxed);
    }
}

/// One comb filter with a simple 1-pole LPF in the feedback loop.
struct Comb {
    buffer: Vec<f32>,
    idx: usize,
    lp_state: f32,
}

impl Comb {
    fn new(size: usize) -> Self {
        Self {
            buffer: vec![0.0; size.max(1)],
            idx: 0,
            lp_state: 0.0,
        }
    }

    fn resize(&mut self, size: usize) {
        self.buffer.clear();
        self.buffer.resize(size.max(1), 0.0);
        self.idx = 0;
        self.lp_state = 0.0;
    }

    fn reset(&mut self) {
        for s in &mut self.buffer {
            *s = 0.0;
        }
        self.idx = 0;
        self.lp_state = 0.0;
    }

    #[inline]
    fn process(&mut self, input: f32, feedback: f32, damp: f32) -> f32 {
        let out = self.buffer[self.idx];
        // One-pole LPF on feedback
        self.lp_state = out * (1.0 - damp) + self.lp_state * damp;
        self.buffer[self.idx] = input + self.lp_state * feedback;
        self.idx += 1;
        if self.idx >= self.buffer.len() {
            self.idx = 0;
        }
        out
    }
}

/// Schroeder allpass with fixed coefficient.
struct Allpass {
    buffer: Vec<f32>,
    idx: usize,
}

impl Allpass {
    fn new(size: usize) -> Self {
        Self {
            buffer: vec![0.0; size.max(1)],
            idx: 0,
        }
    }

    fn resize(&mut self, size: usize) {
        self.buffer.clear();
        self.buffer.resize(size.max(1), 0.0);
        self.idx = 0;
    }

    fn reset(&mut self) {
        for s in &mut self.buffer {
            *s = 0.0;
        }
        self.idx = 0;
    }

    #[inline]
    fn process(&mut self, input: f32) -> f32 {
        let buf_out = self.buffer[self.idx];
        let out = -input + buf_out;
        self.buffer[self.idx] = input + buf_out * ALLPASS_FB;
        self.idx += 1;
        if self.idx >= self.buffer.len() {
            self.idx = 0;
        }
        out
    }
}

pub struct Reverb {
    params: Arc<ReverbParams>,
    combs: [Comb; 8],
    allpasses: [Allpass; 4],
    sample_rate: u32,
    name: String,
}

impl Reverb {
    pub fn new(params: Arc<ReverbParams>, sample_rate: u32) -> Self {
        let scale = sample_rate as f32 / REF_SR;
        let combs = [
            Comb::new((COMB_TAPS[0] as f32 * scale) as usize),
            Comb::new((COMB_TAPS[1] as f32 * scale) as usize),
            Comb::new((COMB_TAPS[2] as f32 * scale) as usize),
            Comb::new((COMB_TAPS[3] as f32 * scale) as usize),
            Comb::new((COMB_TAPS[4] as f32 * scale) as usize),
            Comb::new((COMB_TAPS[5] as f32 * scale) as usize),
            Comb::new((COMB_TAPS[6] as f32 * scale) as usize),
            Comb::new((COMB_TAPS[7] as f32 * scale) as usize),
        ];
        let allpasses = [
            Allpass::new((ALLPASS_TAPS[0] as f32 * scale) as usize),
            Allpass::new((ALLPASS_TAPS[1] as f32 * scale) as usize),
            Allpass::new((ALLPASS_TAPS[2] as f32 * scale) as usize),
            Allpass::new((ALLPASS_TAPS[3] as f32 * scale) as usize),
        ];
        Self {
            params,
            combs,
            allpasses,
            sample_rate,
            name: "Reverb".to_string(),
        }
    }

    pub fn params(&self) -> Arc<ReverbParams> {
        Arc::clone(&self.params)
    }

    /// Resize comb/allpass buffers when the sample rate changes. Safe
    /// to call from the audio thread because `Vec::resize` to the same
    /// or smaller size doesn't allocate (we pre-size on startup and
    /// only rescale proportionally). Note: `Vec::clear` + `resize`
    /// DOES allocate if growing; since this is only called at startup
    /// via `set_sample_rate` it's acceptable.
    fn resize_buffers(&mut self, sample_rate: u32) {
        let scale = sample_rate as f32 / REF_SR;
        for i in 0..8 {
            self.combs[i].resize((COMB_TAPS[i] as f32 * scale) as usize);
        }
        for i in 0..4 {
            self.allpasses[i].resize((ALLPASS_TAPS[i] as f32 * scale) as usize);
        }
    }
}

impl AudioBlock for Reverb {
    fn name(&self) -> &str {
        &self.name
    }

    fn type_id(&self) -> &str {
        "builtin.reverb"
    }

    fn process(&mut self, buffer: &mut [f32], channels: usize) {
        if channels == 0 {
            return;
        }
        if !self.params.enabled() {
            return; // pass-through
        }

        let mix = self.params.mix();
        let dry = 1.0 - mix;
        let wet = mix;
        let feedback = 0.28 + self.params.room_size() * 0.7; // 0.28..0.98
        let damp = self.params.damping() * 0.4; // 0..0.4 on the 1-pole

        let frames = buffer.len() / channels;
        for i in 0..frames {
            let base = i * channels;
            // Mix down channels to a mono input so a single reverb
            // network processes the sum (Freeverb does this). Send
            // the wet signal back to both channels.
            let mut input = 0.0;
            for ch in 0..channels {
                input += buffer[base + ch];
            }
            input *= FIXED_GAIN / (channels as f32);

            // Parallel combs
            let mut out = 0.0;
            for c in &mut self.combs {
                out += c.process(input, feedback, damp);
            }
            // Serial allpasses
            for a in &mut self.allpasses {
                out = a.process(out);
            }

            // Mix dry+wet back into each channel.
            for ch in 0..channels {
                let dry_sample = buffer[base + ch];
                buffer[base + ch] = dry_sample * dry + out * wet;
            }
        }
    }

    fn reset(&mut self) {
        for c in &mut self.combs {
            c.reset();
        }
        for a in &mut self.allpasses {
            a.reset();
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

    #[test]
    fn bypass_when_disabled() {
        // Reverb default is enabled (FTUX) — explicitly disable here
        // to validate the bypass path.
        let r_params = Arc::new(ReverbParams::default());
        r_params.set_enabled(false);
        let mut r = Reverb::new(Arc::clone(&r_params), 48_000);
        let mut buf = [0.5f32; 256];
        r.process(&mut buf, 2);
        assert!(buf.iter().all(|&x| x == 0.5));
    }

    #[test]
    fn enabled_reverb_modifies_signal() {
        let r_params = Arc::new(ReverbParams::default());
        r_params.set_enabled(true);
        r_params.set_mix(1.0); // 100% wet so we see the wet path
        let mut r = Reverb::new(Arc::clone(&r_params), 48_000);
        let mut buf = vec![0.0f32; 4096];
        // Put a single impulse in the middle of the buffer.
        buf[1000] = 1.0;
        r.process(&mut buf, 2);
        // After processing, plenty of nonzero tail samples should exist.
        let tail_energy: f32 = buf[1500..].iter().map(|x| x.abs()).sum();
        assert!(
            tail_energy > 0.01,
            "expected reverb tail energy > 0.01, got {}",
            tail_energy
        );
    }

    #[test]
    fn reset_clears_tail() {
        let r_params = Arc::new(ReverbParams::default());
        r_params.set_enabled(true);
        r_params.set_mix(1.0);
        let mut r = Reverb::new(Arc::clone(&r_params), 48_000);
        let mut buf = vec![0.0f32; 2048];
        buf[500] = 1.0;
        r.process(&mut buf, 2);
        r.reset();
        // Subsequent processing of silence should be silent.
        let mut silence = vec![0.0f32; 4096];
        r.process(&mut silence, 2);
        let peak = silence.iter().fold(0.0f32, |a, b| a.max(b.abs()));
        assert!(
            peak < 1e-6,
            "expected silence after reset, got peak {}",
            peak
        );
    }

    #[test]
    fn mix_zero_is_pure_dry() {
        let r_params = Arc::new(ReverbParams::default());
        r_params.set_enabled(true);
        r_params.set_mix(0.0);
        let mut r = Reverb::new(Arc::clone(&r_params), 48_000);
        let mut buf = vec![0.25f32; 256];
        r.process(&mut buf, 2);
        assert!(
            buf.iter().all(|&x| (x - 0.25).abs() < 1e-6),
            "mix=0 should pass dry through unchanged"
        );
    }
}
