//! A6 modulation/dynamics effects: chorus, flanger, phaser, compressor.
//!
//! These are intentionally small, allocation-at-construction processors
//! that fit the existing A5 `FxSlot::process_inplace` contract. They are
//! not the final SIMD-heavy design-doc implementations, but they provide
//! the user-facing FX family needed by A6 while keeping the audio callback
//! allocation-free after construction.

use core::f32::consts::PI;

use contrapunk_dsp::allpass::Allpass1;
use contrapunk_dsp::delay_line::StereoDelayLine;
use contrapunk_dsp::window::{equal_power, frac01};

use crate::util::{finite_or, set_finite_clamped};

/// Multi-tap style chorus implemented as a stereo modulated delay.
pub struct Chorus {
    sr: f32,
    delay: StereoDelayLine,
    phase: f32,
    rate_hz: f32,
    depth_ms: f32,
    base_ms: f32,
    mix: f32,
}

impl Chorus {
    pub fn new(sample_rate: f32) -> Self {
        let sr = finite_or(sample_rate, 48_000.0).max(1.0);
        Self {
            sr,
            delay: StereoDelayLine::new_power_of_two((sr * 0.080) as usize + 8),
            phase: 0.0,
            rate_hz: 0.35,
            depth_ms: 8.0,
            base_ms: 18.0,
            mix: 0.35,
        }
    }

    pub fn set_rate_hz(&mut self, hz: f32) {
        set_finite_clamped(&mut self.rate_hz, hz, 0.01, 8.0);
    }
    pub fn set_depth_ms(&mut self, ms: f32) {
        set_finite_clamped(&mut self.depth_ms, ms, 0.0, 40.0);
    }
    pub fn set_mix(&mut self, mix: f32) {
        set_finite_clamped(&mut self.mix, mix, 0.0, 1.0);
    }

    pub fn process_inplace(&mut self, buf: &mut [f32], channels: usize) {
        if channels == 0 {
            return;
        }
        let frames = buf.len() / channels;
        let (dry, wet) = equal_power(self.mix);
        let phase_inc = self.rate_hz / self.sr;
        for f in 0..frames {
            let base = f * channels;
            let in_l = buf[base];
            let in_r = if channels >= 2 { buf[base + 1] } else { in_l };
            let lfo_l = libm::sinf(2.0 * PI * self.phase);
            let lfo_r = libm::sinf(2.0 * PI * (self.phase + 0.25));
            self.phase = frac01(self.phase + phase_inc);
            let d_l = (self.base_ms + self.depth_ms * (0.5 + 0.5 * lfo_l)) * self.sr / 1000.0;
            let d_r = (self.base_ms + self.depth_ms * (0.5 + 0.5 * lfo_r)) * self.sr / 1000.0;
            let (dl, dr) = self.delay.tick(in_l, in_r, d_l, d_r, 0.0);
            buf[base] = dry * in_l + wet * dl;
            if channels >= 2 {
                buf[base + 1] = dry * in_r + wet * dr;
            }
        }
    }
}

/// Short modulated delay with feedback for classic flanging.
pub struct Flanger {
    sr: f32,
    delay: StereoDelayLine,
    phase: f32,
    rate_hz: f32,
    depth_ms: f32,
    base_ms: f32,
    feedback: f32,
    mix: f32,
}

impl Flanger {
    pub fn new(sample_rate: f32) -> Self {
        let sr = finite_or(sample_rate, 48_000.0).max(1.0);
        Self {
            sr,
            delay: StereoDelayLine::new_power_of_two((sr * 0.020) as usize + 8),
            phase: 0.0,
            rate_hz: 0.20,
            depth_ms: 3.0,
            base_ms: 2.0,
            feedback: 0.35,
            mix: 0.5,
        }
    }
    pub fn set_rate_hz(&mut self, hz: f32) {
        set_finite_clamped(&mut self.rate_hz, hz, 0.01, 10.0);
    }
    pub fn set_depth_ms(&mut self, ms: f32) {
        set_finite_clamped(&mut self.depth_ms, ms, 0.0, 10.0);
    }
    pub fn set_feedback(&mut self, fb: f32) {
        set_finite_clamped(&mut self.feedback, fb, -0.95, 0.95);
    }
    pub fn set_mix(&mut self, mix: f32) {
        set_finite_clamped(&mut self.mix, mix, 0.0, 1.0);
    }

    pub fn process_inplace(&mut self, buf: &mut [f32], channels: usize) {
        if channels == 0 {
            return;
        }
        let frames = buf.len() / channels;
        let (dry, wet) = equal_power(self.mix);
        let phase_inc = self.rate_hz / self.sr;
        for f in 0..frames {
            let base = f * channels;
            let in_l = buf[base];
            let in_r = if channels >= 2 { buf[base + 1] } else { in_l };
            let lfo = 0.5 + 0.5 * libm::sinf(2.0 * PI * self.phase);
            self.phase = frac01(self.phase + phase_inc);
            let d = (self.base_ms + self.depth_ms * lfo) * self.sr / 1000.0;
            let (dl, dr) = self.delay.tick(in_l, in_r, d, d + 0.7, self.feedback);
            buf[base] = dry * in_l + wet * dl;
            if channels >= 2 {
                buf[base + 1] = dry * in_r + wet * dr;
            }
        }
    }
}

/// Twelve-stage stereo all-pass phaser with an internal LFO.
pub struct Phaser {
    sr: f32,
    left: [Allpass1; 12],
    right: [Allpass1; 12],
    phase: f32,
    rate_hz: f32,
    depth: f32,
    feedback: f32,
    fb_l: f32,
    fb_r: f32,
    mix: f32,
}

impl Phaser {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            sr: finite_or(sample_rate, 48_000.0).max(1.0),
            left: [Allpass1::new(); 12],
            right: [Allpass1::new(); 12],
            phase: 0.0,
            rate_hz: 0.25,
            depth: 0.85,
            feedback: 0.55,
            fb_l: 0.0,
            fb_r: 0.0,
            mix: 0.5,
        }
    }
    pub fn set_rate_hz(&mut self, hz: f32) {
        set_finite_clamped(&mut self.rate_hz, hz, 0.01, 8.0);
    }
    pub fn set_depth(&mut self, depth: f32) {
        set_finite_clamped(&mut self.depth, depth, 0.0, 1.0);
    }
    pub fn set_feedback(&mut self, fb: f32) {
        set_finite_clamped(&mut self.feedback, fb, 0.0, 0.95);
    }
    pub fn set_mix(&mut self, mix: f32) {
        set_finite_clamped(&mut self.mix, mix, 0.0, 1.0);
    }

    pub fn process_inplace(&mut self, buf: &mut [f32], channels: usize) {
        if channels == 0 {
            return;
        }
        let frames = buf.len() / channels;
        let (dry, wet) = equal_power(self.mix);
        let phase_inc = self.rate_hz / self.sr;
        for f in 0..frames {
            let base = f * channels;
            let in_l = buf[base];
            let in_r = if channels >= 2 { buf[base + 1] } else { in_l };
            let sweep = 0.5 + 0.5 * libm::sinf(2.0 * PI * self.phase);
            self.phase = frac01(self.phase + phase_inc);
            // Map sweep to a stable first-order all-pass coefficient.
            let a = (0.05 + self.depth * (0.85 * sweep)).clamp(0.02, 0.92);
            let mut yl = in_l + self.fb_l * self.feedback;
            let mut yr = in_r + self.fb_r * self.feedback;
            for ap in self.left.iter_mut() {
                yl = ap.tick(yl, a);
            }
            for ap in self.right.iter_mut() {
                yr = ap.tick(yr, a * 0.97);
            }
            self.fb_l = yl;
            self.fb_r = yr;
            buf[base] = dry * in_l + wet * yl;
            if channels >= 2 {
                buf[base + 1] = dry * in_r + wet * yr;
            }
        }
    }
}

/// RMS-envelope compressor with downward compression and makeup gain.
pub struct Compressor {
    sr: f32,
    env: f32,
    threshold_db: f32,
    ratio: f32,
    attack_ms: f32,
    release_ms: f32,
    makeup_db: f32,
    mix: f32,
}

impl Compressor {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            sr: finite_or(sample_rate, 48_000.0).max(1.0),
            env: 0.0,
            threshold_db: -18.0,
            ratio: 4.0,
            attack_ms: 8.0,
            release_ms: 120.0,
            makeup_db: 4.0,
            mix: 1.0,
        }
    }
    pub fn set_threshold_db(&mut self, db: f32) {
        set_finite_clamped(&mut self.threshold_db, db, -60.0, 0.0);
    }
    pub fn set_ratio(&mut self, ratio: f32) {
        set_finite_clamped(&mut self.ratio, ratio, 1.0, 40.0);
    }
    pub fn set_attack_ms(&mut self, ms: f32) {
        set_finite_clamped(&mut self.attack_ms, ms, 0.1, 500.0);
    }
    pub fn set_release_ms(&mut self, ms: f32) {
        set_finite_clamped(&mut self.release_ms, ms, 1.0, 2000.0);
    }
    pub fn set_makeup_db(&mut self, db: f32) {
        set_finite_clamped(&mut self.makeup_db, db, -24.0, 24.0);
    }
    pub fn set_mix(&mut self, mix: f32) {
        set_finite_clamped(&mut self.mix, mix, 0.0, 1.0);
    }

    pub fn process_inplace(&mut self, buf: &mut [f32], channels: usize) {
        if channels == 0 {
            return;
        }
        let frames = buf.len() / channels;
        let attack = libm::expf(-1.0 / (self.attack_ms * 0.001 * self.sr).max(1.0));
        let release = libm::expf(-1.0 / (self.release_ms * 0.001 * self.sr).max(1.0));
        let makeup = libm::powf(10.0, self.makeup_db / 20.0);
        for f in 0..frames {
            let base = f * channels;
            let in_l = buf[base];
            let in_r = if channels >= 2 { buf[base + 1] } else { in_l };
            let level = libm::sqrtf((in_l * in_l + in_r * in_r) * 0.5);
            let coeff = if level > self.env { attack } else { release };
            self.env = coeff * self.env + (1.0 - coeff) * level;
            let env_db = 20.0 * libm::log10f(self.env.max(1.0e-9));
            let over = env_db - self.threshold_db;
            let gain_db = if over > 0.0 {
                -over * (1.0 - 1.0 / self.ratio)
            } else {
                0.0
            };
            let gain = libm::powf(10.0, gain_db / 20.0) * makeup;
            let out_l = in_l * gain;
            let out_r = in_r * gain;
            buf[base] = in_l + (out_l - in_l) * self.mix;
            if channels >= 2 {
                buf[base + 1] = in_r + (out_r - in_r) * self.mix;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rms(x: &[f32]) -> f32 {
        if x.is_empty() {
            return 0.0;
        }
        (x.iter().map(|v| v * v).sum::<f32>() / x.len() as f32).sqrt()
    }

    #[test]
    fn chorus_spreads_mono_to_stereo() {
        let mut c = Chorus::new(48_000.0);
        c.set_mix(1.0);
        let mut buf = vec![0.0f32; 48_000];
        for f in 0..24_000 {
            buf[f * 2] = libm::sinf(2.0 * PI * 220.0 * f as f32 / 48_000.0);
            buf[f * 2 + 1] = buf[f * 2];
        }
        c.process_inplace(&mut buf, 2);
        let diff: f32 = buf
            .chunks_exact(2)
            .map(|c| (c[0] - c[1]).abs())
            .sum::<f32>()
            / 24_000.0;
        assert!(diff > 1.0e-4, "chorus did not decorrelate stereo: {diff}");
    }

    #[test]
    fn flanger_feedback_changes_signal_energy() {
        let mut f = Flanger::new(48_000.0);
        f.set_mix(1.0);
        f.set_feedback(0.7);
        let mut buf = vec![0.0f32; 48_000];
        buf[0] = 1.0;
        buf[1] = 1.0;
        f.process_inplace(&mut buf, 2);
        assert!(rms(&buf[1000..]) > 1.0e-4);
    }

    #[test]
    fn compressor_reduces_hot_signal() {
        let mut c = Compressor::new(48_000.0);
        c.set_threshold_db(-24.0);
        c.set_ratio(8.0);
        c.set_makeup_db(0.0);
        let mut buf = vec![0.75f32; 48_000];
        let before = rms(&buf);
        c.process_inplace(&mut buf, 2);
        let after = rms(&buf[24_000..]);
        assert!(
            after < before * 0.75,
            "compressor not reducing level: before={before} after={after}"
        );
    }
}
