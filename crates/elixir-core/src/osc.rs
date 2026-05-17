//! Wavetable oscillator (Phase 21.A1).
//!
//! Fixed-point phase accumulator, linear-interpolated table lookup. Both
//! design choices match `ELIXIR-DESIGN.md` §4. A1.4 will upgrade the
//! interp to four-point Catmull-Rom; A1.5+ will add spectral mip-maps.

use crate::tables::SineTable;

/// Single-frame wavetable oscillator. A1.1 is wired to a sine table; the
/// table is passed in on `tick` so future phases can hot-swap multi-frame
/// `WavetableData` without changing this struct.
pub struct Oscillator {
    phase: u32,
    phase_inc: u32,
}

impl Oscillator {
    pub const fn new() -> Self {
        Self {
            phase: 0,
            phase_inc: 0,
        }
    }

    /// Configure the oscillator's pitch.
    ///
    /// `freq_hz` is the target output frequency, `sample_rate` is the
    /// host device's audio rate. Internally stored as a 32-bit fixed-
    /// point phase increment: a full circle = `2^32` ticks.
    pub fn set_frequency(&mut self, freq_hz: f32, sample_rate: f32) {
        let sr = sample_rate.max(1.0);
        let ratio = (freq_hz / sr).clamp(0.0, 0.499_999); // Nyquist guard
        let inc = ratio * ((1u64 << 32) as f32);
        self.phase_inc = inc as u32;
    }

    /// Reset the phase to zero. Called on note-on to give a deterministic
    /// starting point — important for A/B parity tests later.
    pub fn reset_phase(&mut self) {
        self.phase = 0;
    }

    /// Produce one sample and advance the phase. Inlined in tight loops.
    /// Uses four-point Catmull-Rom interpolation against the wavetable —
    /// see `ELIXIR-DESIGN.md` §4.
    #[inline]
    pub fn tick(&mut self, table: &SineTable) -> f32 {
        let sample = table.lookup_catmull(self.phase);
        self.phase = self.phase.wrapping_add(self.phase_inc);
        sample
    }
}

impl Default for Oscillator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_oscillator_silent() {
        let table = SineTable::new();
        let mut osc = Oscillator::new();
        for _ in 0..16 {
            assert_eq!(osc.tick(&table), 0.0);
        }
    }

    #[test]
    fn set_frequency_produces_oscillation() {
        let table = SineTable::new();
        let mut osc = Oscillator::new();
        osc.set_frequency(1_000.0, 48_000.0);
        let mut peak = 0.0f32;
        for _ in 0..48 {
            let s = osc.tick(&table);
            if s.abs() > peak {
                peak = s.abs();
            }
        }
        assert!(peak > 0.5, "expected a meaningful peak, got {peak}");
    }

    #[test]
    fn reset_phase_returns_to_zero_crossing() {
        let table = SineTable::new();
        let mut osc = Oscillator::new();
        osc.set_frequency(440.0, 48_000.0);
        // advance some samples
        for _ in 0..100 {
            let _ = osc.tick(&table);
        }
        osc.reset_phase();
        assert!((osc.tick(&table)).abs() < 1e-6);
    }
}
