//! Schroeder reverb (Phase 21.A5 v1).
//!
//! 4 parallel comb filters + 2 series allpass filters. The classic
//! 1962 design — small CPU footprint, slightly metallic tail. The
//! design-doc target is a 16-line FDN; that lands in A6 once we have
//! room and the audible difference matters. For A5 the goal is
//! "yes, there is a reverb tail when you let go of a note".

extern crate alloc;
use alloc::vec::Vec;

pub struct Reverb {
    combs: [Comb; 4],
    aps: [Allpass; 2],
    decay: f32,
    damping: f32,
    mix: f32,
}

impl Reverb {
    pub fn new(sample_rate: f32) -> Self {
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
        self.decay = d.clamp(0.0, 0.99);
        for c in self.combs.iter_mut() {
            c.feedback = self.decay;
        }
    }
    pub fn set_damping(&mut self, d: f32) {
        self.damping = d.clamp(0.0, 1.0);
        for c in self.combs.iter_mut() {
            c.damping = self.damping;
        }
    }
    pub fn set_mix(&mut self, m: f32) {
        self.mix = m.clamp(0.0, 1.0);
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
}
