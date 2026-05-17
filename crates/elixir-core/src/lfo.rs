//! Low-frequency oscillator (Phase 21.A3).
//!
//! Same fixed-point phase trick as the audio oscillator, but rendered
//! at the *control rate* — one sample per audio block, broadcast to
//! every voice that subscribes. A later A3 follow-up adds custom-
//! waveform (breakpoint) LFOs and the four random-LFO flavours from
//! the design doc.

use crate::tables::SineTable;

/// Sine LFO with a base rate plus an additive rate-modulation input.
pub struct Lfo {
    phase: u32,
    base_rate_hz: f32,
    /// Per-block additive rate offset in Hz, supplied by the mod matrix.
    rate_mod_hz: f32,
    sample_rate: f32,
}

impl Lfo {
    pub const fn new() -> Self {
        Self {
            phase: 0,
            base_rate_hz: 5.0,
            rate_mod_hz: 0.0,
            sample_rate: 48_000.0,
        }
    }

    pub fn set_sample_rate(&mut self, sr: f32) {
        self.sample_rate = sr.max(1.0);
    }
    pub fn set_rate_hz(&mut self, hz: f32) {
        self.base_rate_hz = hz.max(0.0);
    }
    pub fn base_rate_hz(&self) -> f32 {
        self.base_rate_hz
    }

    /// Set the per-block rate modulation in Hz. Cleared each block by
    /// the engine before mod evaluation, then this is summed from every
    /// route targeting `ModDest::LfoRate`.
    pub fn set_rate_mod_hz(&mut self, hz: f32) {
        self.rate_mod_hz = hz;
    }

    pub fn reset_phase(&mut self) {
        self.phase = 0;
    }

    /// Advance the LFO by `frames` samples at the current effective
    /// rate and return the value AT THE START of the block (before
    /// advancement). Returned value is bipolar (-1.0..1.0).
    pub fn tick_block(&mut self, table: &SineTable, frames: usize) -> f32 {
        let value = table.lookup_catmull(self.phase);
        let effective_hz = (self.base_rate_hz + self.rate_mod_hz).max(0.0);
        let ratio = (effective_hz / self.sample_rate).clamp(0.0, 0.49);
        let inc = (ratio * (1u64 << 32) as f32) as u32;
        let total = inc.wrapping_mul(frames as u32);
        self.phase = self.phase.wrapping_add(total);
        value
    }
}

impl Default for Lfo {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lfo_default_oscillates_around_zero() {
        let table = SineTable::new();
        let mut lfo = Lfo::new();
        lfo.set_sample_rate(48_000.0);
        lfo.set_rate_hz(5.0);
        let mut peak_pos = -1.0f32;
        let mut peak_neg = 1.0f32;
        // 200 blocks of 64 samples = ~0.267s, well over one cycle at 5Hz.
        for _ in 0..200 {
            let v = lfo.tick_block(&table, 64);
            if v > peak_pos {
                peak_pos = v;
            }
            if v < peak_neg {
                peak_neg = v;
            }
        }
        assert!(peak_pos > 0.9, "LFO peak positive {peak_pos} below 0.9");
        assert!(peak_neg < -0.9, "LFO peak negative {peak_neg} above -0.9");
    }

    #[test]
    fn rate_mod_changes_effective_frequency() {
        let table = SineTable::new();
        let mut lfo = Lfo::new();
        lfo.set_sample_rate(48_000.0);
        lfo.set_rate_hz(0.5); // very slow
        lfo.set_rate_mod_hz(10.0); // mod pushes to ~10.5 Hz
                                   // 0.5s of 64-sample blocks = 375 blocks; at 10.5Hz we should see ≥ 4 cycles
        let mut crossings = 0u32;
        let mut prev = lfo.tick_block(&table, 64);
        for _ in 0..380 {
            let v = lfo.tick_block(&table, 64);
            if (prev >= 0.0) != (v >= 0.0) {
                crossings += 1;
            }
            prev = v;
        }
        // 4 cycles = 8 zero-crossings minimum
        assert!(crossings >= 8, "expected >= 8 crossings, got {crossings}");
    }
}
