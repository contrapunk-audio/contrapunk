//! Reverbs.
//!
//! [`Reverb`] is the Phase 21.A5 Schroeder reverb: 4 parallel comb
//! filters + 2 series allpass filters. [`FdnReverb`] is the Phase 21.A6
//! 16-line feedback-delay-network reverb. Both allocate delay memory at
//! construction / sample-rate setup time, then process allocation-free.

extern crate alloc;
use alloc::vec::Vec;

use contrapunk_dsp::matrix::hadamard16;

use crate::util::{finite_or, set_finite_clamped};

pub struct Reverb {
    combs: [Comb; 4],
    aps: [Allpass; 2],
    decay: f32,
    damping: f32,
    mix: f32,
}

impl Reverb {
    pub fn new(sample_rate: f32) -> Self {
        let sample_rate = finite_or(sample_rate, 48_000.0).max(1.0);
        let r = sample_rate / 44_100.0;
        let comb_delays = [1116usize, 1188, 1277, 1356];
        let ap_delays = [225usize, 556];
        let mut combs: [Comb; 4] =
            core::array::from_fn(|i| Comb::new(((comb_delays[i] as f32) * r) as usize));
        let aps: [Allpass; 2] =
            core::array::from_fn(|i| Allpass::new(((ap_delays[i] as f32) * r) as usize));
        for c in combs.iter_mut() {
            c.feedback = 0.84;
            c.damping = 0.4;
        }
        Self {
            combs,
            aps,
            decay: 0.84,
            damping: 0.4,
            mix: 0.3,
        }
    }

    pub fn set_decay(&mut self, d: f32) {
        if d.is_finite() {
            self.decay = d.clamp(0.0, 0.99);
            for c in self.combs.iter_mut() {
                c.feedback = self.decay;
            }
        }
    }
    pub fn set_damping(&mut self, d: f32) {
        if d.is_finite() {
            self.damping = d.clamp(0.0, 1.0);
            for c in self.combs.iter_mut() {
                c.damping = self.damping;
            }
        }
    }
    pub fn set_mix(&mut self, m: f32) {
        set_finite_clamped(&mut self.mix, m, 0.0, 1.0);
    }
    pub fn decay(&self) -> f32 {
        self.decay
    }
    pub fn damping(&self) -> f32 {
        self.damping
    }
    pub fn mix(&self) -> f32 {
        self.mix
    }

    /// Stereo in-place processing. Mono-from-stereo summed for the wet
    /// path (true stereo decorrelation lands in the FDN variant).
    pub fn process_inplace(&mut self, buf: &mut [f32], channels: usize) {
        if channels == 0 {
            return;
        }
        let frames = buf.len() / channels;
        let mix = self.mix;
        let dry_w = 1.0 - mix;
        for f in 0..frames {
            let base = f * channels;
            let in_l = buf[base];
            let in_r = if channels >= 2 { buf[base + 1] } else { in_l };
            let mono = (in_l + in_r) * 0.5;

            let mut wet = 0.0f32;
            for c in self.combs.iter_mut() {
                wet += c.tick(mono);
            }
            wet *= 0.25;
            for ap in self.aps.iter_mut() {
                wet = ap.tick(wet);
            }

            buf[base] = dry_w * in_l + mix * wet;
            if channels >= 2 {
                buf[base + 1] = dry_w * in_r + mix * wet;
            }
        }
    }
}

/// 16-line feedback delay network reverb (Phase 21.A6).
///
/// This is a compact scalar FDN: prime-ish delay lengths, per-line
/// damping, and a normalized Hadamard feedback matrix. It is designed
/// as the desktop-quality path while the A5 Schroeder [`Reverb`] remains
/// available as a cheap fallback.
pub struct FdnReverb {
    sample_rate: f32,
    lines: [FdnLine; 16],
    feedback: [f32; 16],
    decay_seconds: f32,
    damping: f32,
    mix: f32,
    input_lp: f32,
    input_hp: f32,
    last_input: f32,
}

impl FdnReverb {
    pub fn new(sample_rate: f32) -> Self {
        let sr = finite_or(sample_rate, 48_000.0).max(1.0);
        // Canonical-ish mutually-prime lengths around 140-320 ms at
        // 48 kHz. The fractional design-doc values are rounded and
        // sample-rate scaled here; modulation drift can layer on later.
        let base = [
            6753usize, 9278, 7705, 11329, 8467, 10111, 12983, 15137, 16249, 17401, 18839, 19963,
            21317, 23167, 24989, 27103,
        ];
        let scale = sr / 48_000.0;
        let lines = core::array::from_fn(|i| FdnLine::new((base[i] as f32 * scale) as usize));
        let mut r = Self {
            sample_rate: sr,
            lines,
            feedback: [0.0; 16],
            decay_seconds: 2.8,
            damping: 0.35,
            mix: 0.35,
            input_lp: 0.0,
            input_hp: 0.0,
            last_input: 0.0,
        };
        r.recompute_feedback(sr);
        r
    }

    pub fn set_decay_seconds(&mut self, seconds: f32) {
        if seconds.is_finite() {
            self.decay_seconds = seconds.clamp(0.2, 20.0);
            self.recompute_feedback(self.sample_rate);
        }
    }
    pub fn sample_rate(&self) -> f32 {
        self.sample_rate
    }
    pub fn set_damping(&mut self, damping: f32) {
        set_finite_clamped(&mut self.damping, damping, 0.0, 1.0);
    }
    pub fn set_mix(&mut self, mix: f32) {
        set_finite_clamped(&mut self.mix, mix, 0.0, 1.0);
    }

    fn recompute_feedback(&mut self, sr: f32) {
        for (i, line) in self.lines.iter().enumerate() {
            let delay = line.delay_samples() as f32;
            self.feedback[i] = libm::powf(0.001, delay / (self.decay_seconds * sr).max(1.0));
        }
    }

    /// Stereo in-place processing. Input is folded to mono for the FDN;
    /// output is decorrelated by summing alternating delay-line groups.
    pub fn process_inplace(&mut self, buf: &mut [f32], channels: usize) {
        if channels == 0 {
            return;
        }
        let frames = buf.len() / channels;
        let wet = libm::sinf(self.mix * core::f32::consts::PI * 0.5);
        let dry = libm::cosf(self.mix * core::f32::consts::PI * 0.5);
        let mut outs = [0.0f32; 16];
        let mut mixed = [0.0f32; 16];
        for f in 0..frames {
            let base = f * channels;
            let in_l = buf[base];
            let in_r = if channels >= 2 { buf[base + 1] } else { in_l };
            let mono = (in_l + in_r) * 0.5;

            // Simple input low/high shaping so bright transients do not
            // dominate the feedback network forever.
            self.input_lp += (mono - self.input_lp) * 0.25;
            let hp = mono - self.last_input + 0.995 * self.input_hp;
            self.last_input = mono;
            self.input_hp = hp;
            let input = self.input_lp * 0.75 + hp * 0.25;

            for i in 0..16 {
                outs[i] = self.lines[i].read();
            }
            hadamard16(&outs, &mut mixed);
            let norm = 0.25; // 1/sqrt(16)
            for i in 0..16 {
                let fb = mixed[i] * norm * self.feedback[i];
                self.lines[i].write(input * 0.20 + fb, self.damping);
            }

            let mut wet_l = 0.0;
            let mut wet_r = 0.0;
            for i in 0..8 {
                wet_l += outs[i];
            }
            for i in 8..16 {
                wet_r += outs[i];
            }
            wet_l *= 0.125;
            wet_r *= 0.125;
            buf[base] = dry * in_l + wet * wet_l;
            if channels >= 2 {
                buf[base + 1] = dry * in_r + wet * wet_r;
            }
        }
    }
}

struct FdnLine {
    buf: Vec<f32>,
    write: usize,
    damped: f32,
    delay_samples: usize,
}

impl FdnLine {
    fn new(delay_samples: usize) -> Self {
        let delay_samples = delay_samples.max(4);
        let n = delay_samples.next_power_of_two();
        let mut buf = Vec::with_capacity(n);
        buf.resize(n, 0.0);
        Self {
            buf,
            write: 0,
            damped: 0.0,
            delay_samples,
        }
    }
    fn delay_samples(&self) -> usize {
        self.delay_samples
    }
    #[inline]
    fn read(&self) -> f32 {
        self.buf[self.write]
    }
    #[inline]
    fn write(&mut self, x: f32, damping: f32) {
        let d = damping.clamp(0.0, 0.99);
        self.damped = self.damped * d + x * (1.0 - d);
        self.buf[self.write] = self.damped;
        self.write = (self.write + 1) & (self.buf.len() - 1);
    }
}

struct Comb {
    buf: Vec<f32>,
    write: usize,
    feedback: f32,
    lp_state: f32,
    damping: f32,
}

impl Comb {
    fn new(delay_samples: usize) -> Self {
        let n = delay_samples.max(1);
        let mut buf = Vec::with_capacity(n);
        buf.resize(n, 0.0);
        Self {
            buf,
            write: 0,
            feedback: 0.7,
            lp_state: 0.0,
            damping: 0.5,
        }
    }

    #[inline]
    fn tick(&mut self, x: f32) -> f32 {
        let out = self.buf[self.write];
        let damped = out * (1.0 - self.damping) + self.lp_state * self.damping;
        self.lp_state = damped;
        self.buf[self.write] = x + damped * self.feedback;
        self.write = (self.write + 1) % self.buf.len();
        out
    }
}

struct Allpass {
    buf: Vec<f32>,
    write: usize,
    feedback: f32,
}

impl Allpass {
    fn new(delay_samples: usize) -> Self {
        let n = delay_samples.max(1);
        let mut buf = Vec::with_capacity(n);
        buf.resize(n, 0.0);
        Self {
            buf,
            write: 0,
            feedback: 0.5,
        }
    }

    #[inline]
    fn tick(&mut self, x: f32) -> f32 {
        let stored = self.buf[self.write];
        let out = -x + stored;
        self.buf[self.write] = x + stored * self.feedback;
        self.write = (self.write + 1) % self.buf.len();
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reverb_extends_decay_tail() {
        let mut r = Reverb::new(48_000.0);
        r.set_decay(0.9);
        r.set_mix(1.0); // pure wet so the tail is unambiguous
        let mut buf = [0.0f32; 48_000 * 2]; // 1 s
        buf[0] = 1.0;
        buf[1] = 1.0;
        r.process_inplace(&mut buf, 2);
        // At 500 ms we should still hear non-trivial reverb energy.
        let tail = &buf[24_000 * 2..24_000 * 2 + 1024];
        let rms: f32 = (tail.iter().map(|s| s * s).sum::<f32>() / tail.len() as f32).sqrt();
        assert!(rms > 1e-4, "reverb tail too quiet: {rms}");
    }

    #[test]
    fn schroeder_setters_are_observable_for_ui_snapshots() {
        let mut r = Reverb::new(48_000.0);
        r.set_decay(0.72);
        r.set_damping(0.33);
        r.set_mix(0.44);
        assert!((r.decay() - 0.72).abs() < 1e-6);
        assert!((r.damping() - 0.33).abs() < 1e-6);
        assert!((r.mix() - 0.44).abs() < 1e-6);
    }

    #[test]
    fn fdn_reverb_has_dense_late_tail() {
        let mut r = FdnReverb::new(48_000.0);
        r.set_decay_seconds(4.0);
        r.set_mix(1.0);
        let mut buf = vec![0.0f32; 96_000 * 2]; // 2 s stereo
        buf[0] = 1.0;
        buf[1] = 1.0;
        r.process_inplace(&mut buf, 2);
        let tail = &buf[48_000 * 2..48_000 * 2 + 4096];
        let rms: f32 = (tail.iter().map(|s| s * s).sum::<f32>() / tail.len() as f32).sqrt();
        assert!(rms > 1e-5, "fdn tail too quiet: {rms}");
        let stereo_diff: f32 = tail
            .chunks_exact(2)
            .map(|c| (c[0] - c[1]).abs())
            .sum::<f32>()
            / 2048.0;
        assert!(
            stereo_diff > 1e-6,
            "fdn output should be decorrelated: {stereo_diff}"
        );
    }
}
