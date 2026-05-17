//! State-variable filter (Phase 21.A4 v1).
//!
//! Zavalishin's TPT / ZDF formulation — `ELIXIR-DESIGN.md` §5. Splits
//! into [`SvfCoeffs`] (computed once per block from cutoff + resonance
//! + sample rate) and [`Svf`] (the two-sample state per voice). That
//! lets the engine compute coefficients once per block and apply them
//! across every active voice without redoing the `tanf` per voice.
//!
//! v1 ships the lowpass tap only. The same state easily produces
//! bandpass, highpass, and the dual-notch / band-peak blends — those
//! arrive in A4 follow-ups along with the analog-ladder and comb
//! topologies the design doc enumerates.

use core::f32::consts::PI;

/// Per-block coefficients derived from `(cutoff_hz, resonance, sample_rate)`.
#[derive(Clone, Copy, Debug)]
pub struct SvfCoeffs {
    pub a1: f32,
    pub a2: f32,
    pub a3: f32,
    pub k: f32,
}

impl SvfCoeffs {
    /// Identity coefficients — filter is bypass.
    pub const fn identity() -> Self {
        Self {
            a1: 1.0,
            a2: 0.0,
            a3: 0.0,
            k: 2.0,
        }
    }

    /// Compute coefficients for a target cutoff (Hz), resonance
    /// (`0..1`), and sample rate. Cutoff is clamped to `[20, fs * 0.49]`
    /// for stability.
    pub fn from_params(cutoff_hz: f32, resonance: f32, sample_rate: f32) -> Self {
        let sr = sample_rate.max(1.0);
        let nyq = sr * 0.5;
        let fc = cutoff_hz.clamp(20.0, nyq - 200.0);
        let g = libm::tanf(PI * fc / sr);
        let r = resonance.clamp(0.0, 0.99);
        // `k` maps 0→2 (no self-oscillation) down toward 0 at high R.
        let k = 2.0 - 2.0 * r;
        let denom = 1.0 + g * (g + k);
        let a1 = 1.0 / denom;
        let a2 = g * a1;
        let a3 = g * a2;
        Self { a1, a2, a3, k }
    }
}

/// Two-state SVF carrying nothing but `ic1eq` / `ic2eq`. The block-level
/// [`SvfCoeffs`] is passed in by reference each tick — typical use puts
/// one `Svf` per voice and one `SvfCoeffs` per engine.
#[derive(Clone, Copy, Debug, Default)]
pub struct Svf {
    ic1eq: f32,
    ic2eq: f32,
}

impl Svf {
    pub const fn new() -> Self {
        Self {
            ic1eq: 0.0,
            ic2eq: 0.0,
        }
    }

    pub fn reset(&mut self) {
        self.ic1eq = 0.0;
        self.ic2eq = 0.0;
    }

    /// Lowpass output. Returns the LP tap and updates state.
    #[inline]
    pub fn tick_lp(&mut self, x: f32, c: &SvfCoeffs) -> f32 {
        let v3 = x - self.ic2eq;
        let v1 = c.a1 * self.ic1eq + c.a2 * v3;
        let v2 = self.ic2eq + c.a2 * self.ic1eq + c.a3 * v3;
        self.ic1eq = 2.0 * v1 - self.ic1eq;
        self.ic2eq = 2.0 * v2 - self.ic2eq;
        v2
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rms(x: &[f32]) -> f32 {
        if x.is_empty() {
            return 0.0;
        }
        let s: f64 = x.iter().map(|v| (*v as f64).powi(2)).sum();
        (s / x.len() as f64).sqrt() as f32
    }

    #[test]
    fn lp_passes_low_frequencies() {
        let sr = 48_000.0f32;
        let c = SvfCoeffs::from_params(8_000.0, 0.0, sr);
        let mut svf = Svf::new();
        // 100 Hz sine — well below 8 kHz cutoff
        let mut buf = vec![0.0f32; 4800];
        let omega = 2.0 * PI * 100.0 / sr;
        for (i, s) in buf.iter_mut().enumerate() {
            *s = svf.tick_lp(libm::sinf(omega * i as f32), &c);
        }
        // After settling, RMS should be close to 0.707 (unit-amplitude
        // sine RMS); a 100 Hz signal through 8 kHz LP is barely attenuated.
        let settled = &buf[2400..];
        assert!(rms(settled) > 0.6, "100Hz LP rms too low: {}", rms(settled));
    }

    #[test]
    fn lp_attenuates_high_frequencies() {
        let sr = 48_000.0f32;
        let c = SvfCoeffs::from_params(500.0, 0.0, sr);
        let mut svf = Svf::new();
        // 8 kHz sine — far above 500 Hz cutoff
        let mut buf = vec![0.0f32; 4800];
        let omega = 2.0 * PI * 8_000.0 / sr;
        for (i, s) in buf.iter_mut().enumerate() {
            *s = svf.tick_lp(libm::sinf(omega * i as f32), &c);
        }
        let settled = &buf[2400..];
        // 8 kHz through 500 Hz LP, ~-24 dB attenuation expected
        assert!(
            rms(settled) < 0.15,
            "8kHz LP rms too high: {}",
            rms(settled)
        );
    }

    #[test]
    fn sweep_changes_amplitude_passed_through() {
        // Cutoff at 500 Hz attenuates a 2 kHz test signal; cutoff at
        // 8 kHz passes it through. RMS at the two cutoffs should
        // differ.
        let sr = 48_000.0f32;
        let omega = 2.0 * PI * 2_000.0 / sr;

        let c_low = SvfCoeffs::from_params(500.0, 0.0, sr);
        let mut svf_low = Svf::new();
        let mut buf_low = vec![0.0f32; 4800];
        for (i, s) in buf_low.iter_mut().enumerate() {
            *s = svf_low.tick_lp(libm::sinf(omega * i as f32), &c_low);
        }

        let c_high = SvfCoeffs::from_params(8_000.0, 0.0, sr);
        let mut svf_high = Svf::new();
        let mut buf_high = vec![0.0f32; 4800];
        for (i, s) in buf_high.iter_mut().enumerate() {
            *s = svf_high.tick_lp(libm::sinf(omega * i as f32), &c_high);
        }

        let r_low = rms(&buf_low[2400..]);
        let r_high = rms(&buf_high[2400..]);
        assert!(
            r_high > r_low * 2.0,
            "expected open cutoff to pass much more signal: low={r_low}, high={r_high}"
        );
    }

    #[test]
    fn high_resonance_amplifies_signal_near_cutoff() {
        let sr = 48_000.0f32;
        let omega = 2.0 * PI * 1_000.0 / sr;

        let c_flat = SvfCoeffs::from_params(1_000.0, 0.0, sr);
        let mut svf_flat = Svf::new();
        let mut buf_flat = vec![0.0f32; 4800];
        for (i, s) in buf_flat.iter_mut().enumerate() {
            *s = svf_flat.tick_lp(libm::sinf(omega * i as f32), &c_flat);
        }

        let c_reso = SvfCoeffs::from_params(1_000.0, 0.9, sr);
        let mut svf_reso = Svf::new();
        let mut buf_reso = vec![0.0f32; 4800];
        for (i, s) in buf_reso.iter_mut().enumerate() {
            *s = svf_reso.tick_lp(libm::sinf(omega * i as f32), &c_reso);
        }

        let r_flat = rms(&buf_flat[2400..]);
        let r_reso = rms(&buf_reso[2400..]);
        // High-Q at the cutoff frequency lifts the response.
        assert!(
            r_reso > r_flat * 1.2,
            "expected resonance peak: flat={r_flat}, reso={r_reso}"
        );
    }
}
