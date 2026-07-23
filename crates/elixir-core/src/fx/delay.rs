//! Stereo ping-pong delay (Phase 21.A5).
//!
//! Pre-allocated delay lines (one per channel). Each side reads from
//! the *other* side's tap before writing, producing the bouncing
//! stereo effect. Feedback ≤ 0.99 to avoid runaway resonance.

extern crate alloc;
use alloc::vec::Vec;

use crate::util::set_finite_clamped;

pub struct Delay {
    buf_l: Vec<f32>,
    buf_r: Vec<f32>,
    write: usize,
    delay_samples: usize,
    feedback: f32,
    mix: f32,
}

impl Delay {
    /// `max_delay_samples` defines the upper bound of delay time; the
    /// current delay is initialised at half of that.
    pub fn new(max_delay_samples: usize) -> Self {
        let cap = max_delay_samples.max(1);
        let mut buf_l = Vec::with_capacity(cap);
        let mut buf_r = Vec::with_capacity(cap);
        buf_l.resize(cap, 0.0);
        buf_r.resize(cap, 0.0);
        Self {
            buf_l,
            buf_r,
            write: 0,
            delay_samples: cap / 2,
            feedback: 0.4,
            mix: 0.3,
        }
    }

    pub fn set_delay_samples(&mut self, n: usize) {
        self.delay_samples = n.clamp(1, self.buf_l.len() - 1);
    }
    pub fn set_delay_secs(&mut self, secs: f32, sample_rate: f32) {
        if secs.is_finite() && sample_rate.is_finite() {
            let n = (secs.max(0.0) * sample_rate.max(1.0)) as usize;
            self.set_delay_samples(n);
        }
    }
    pub fn set_feedback(&mut self, f: f32) {
        set_finite_clamped(&mut self.feedback, f, 0.0, 0.99);
    }
    pub fn set_mix(&mut self, m: f32) {
        set_finite_clamped(&mut self.mix, m, 0.0, 1.0);
    }
    pub fn delay_samples(&self) -> usize {
        self.delay_samples
    }
    pub fn feedback(&self) -> f32 {
        self.feedback
    }
    pub fn mix(&self) -> f32 {
        self.mix
    }

    /// Process an interleaved stereo buffer in-place. Channels < 2 falls
    /// back to mono (writes left-channel only).
    pub fn process_inplace(&mut self, buf: &mut [f32], channels: usize) {
        if channels == 0 || self.buf_l.is_empty() {
            return;
        }
        let n = self.buf_l.len();
        let frames = buf.len() / channels;
        let mix = self.mix;
        let dry_w = 1.0 - mix;
        let fb = self.feedback;
        for f in 0..frames {
            let read = (self.write + n - self.delay_samples) % n;
            let dl = self.buf_l[read];
            let dr = self.buf_r[read];
            let base = f * channels;
            let in_l = buf[base];
            let in_r = if channels >= 2 { buf[base + 1] } else { in_l };

            // Ping-pong: cross-feed delayed taps.
            self.buf_l[self.write] = in_l + dr * fb;
            self.buf_r[self.write] = in_r + dl * fb;

            buf[base] = dry_w * in_l + mix * dl;
            if channels >= 2 {
                buf[base + 1] = dry_w * in_r + mix * dr;
            }

            self.write = (self.write + 1) % n;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn impulse_appears_after_delay() {
        let mut d = Delay::new(1024);
        d.set_delay_samples(64);
        d.set_feedback(0.0);
        d.set_mix(1.0);
        let mut buf = [0.0f32; 256 * 2];
        buf[0] = 1.0; // impulse left
        buf[1] = 1.0; // impulse right
        d.process_inplace(&mut buf, 2);
        // 64-sample delay × stereo interleave → impulse echoes back
        // at sample index 64 on left
        let echo_l = buf[64 * 2];
        let echo_r = buf[64 * 2 + 1];
        assert!(echo_l.abs() > 0.5, "left echo too quiet: {echo_l}");
        assert!(echo_r.abs() > 0.5, "right echo too quiet: {echo_r}");
    }

    #[test]
    fn setters_are_observable_for_ui_snapshots() {
        let mut d = Delay::new(2048);
        d.set_delay_samples(100);
        d.set_feedback(0.7);
        d.set_mix(0.25);
        assert_eq!(d.delay_samples(), 100);
        assert!((d.feedback() - 0.7).abs() < 1e-6);
        assert!((d.mix() - 0.25).abs() < 1e-6);
    }

    #[test]
    fn feedback_creates_repeating_echoes() {
        let mut d = Delay::new(2048);
        d.set_delay_samples(100);
        d.set_feedback(0.7);
        d.set_mix(1.0);
        let mut buf = [0.0f32; 600 * 2];
        buf[0] = 1.0;
        buf[1] = 1.0;
        d.process_inplace(&mut buf, 2);
        // expect multiple echoes
        let mut peaks = 0;
        let mut prev = 0.0f32;
        for s in buf.iter().step_by(2) {
            if s.abs() > 0.1 && prev.abs() <= 0.1 {
                peaks += 1;
            }
            prev = *s;
        }
        assert!(peaks >= 3, "expected ≥3 echoes, got {peaks}");
    }
}
