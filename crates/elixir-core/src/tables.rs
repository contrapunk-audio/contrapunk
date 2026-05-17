//! Wavetable storage and lookup primitives.
//!
//! Phase 21.A1 ships a single 2048-sample sine table; later phases add
//! multi-frame wavetables, spectral mip-mapping, and Catmull-Rom
//! interpolation. The fixed-point phase format is the long-term shape
//! described in `ELIXIR-DESIGN.md` §4, so this scaffolding survives all
//! the way to A6.

use core::f32::consts::TAU;

/// Top bits of a phase that index a wavetable sample. 11 bits → 2048
/// samples per frame, matching the design doc.
pub const TABLE_BITS: u32 = 11;

/// Samples per wavetable frame.
pub const TABLE_SIZE: usize = 1 << TABLE_BITS as usize;

/// Mask for wrapping a sample index.
pub const TABLE_MASK: usize = TABLE_SIZE - 1;

/// Bottom bits of a phase that hold the interpolation fraction.
pub const FRAC_BITS: u32 = 32 - TABLE_BITS;

/// Mask for extracting the fraction.
pub const FRAC_MASK: u32 = (1u32 << FRAC_BITS) - 1;

/// Inverse of `1 << FRAC_BITS` as `f32` — used to convert the fixed-point
/// fraction to a `[0, 1)` float without a divide in the hot loop.
pub const FRAC_TO_F32: f32 = 1.0 / ((1u32 << FRAC_BITS) as f32);

/// A 2048-sample sine table. Owned by [`crate::Engine`] for now; future
/// phases promote it to an `Arc<WavetableData>` swapped via `arc-swap`.
pub struct SineTable {
    data: [f32; TABLE_SIZE],
}

impl SineTable {
    /// Build the sine table at runtime. Cost is ~2048 `sinf` calls — a
    /// one-shot ~50 µs at engine construction, never on the audio thread.
    pub fn new() -> Self {
        let mut data = [0.0f32; TABLE_SIZE];
        let n = TABLE_SIZE as f32;
        let mut i = 0;
        while i < TABLE_SIZE {
            let phase = TAU * (i as f32) / n;
            data[i] = libm::sinf(phase);
            i += 1;
        }
        Self { data }
    }

    /// Read the table with linear interpolation. Retained for tests
    /// and comparison; the production hot-path uses [`Self::lookup_catmull`].
    #[inline]
    pub fn lookup_linear(&self, phase: u32) -> f32 {
        let idx = (phase >> FRAC_BITS) as usize;
        let frac = (phase & FRAC_MASK) as f32 * FRAC_TO_F32;
        let s0 = self.data[idx];
        let s1 = self.data[(idx + 1) & TABLE_MASK];
        s0 + (s1 - s0) * frac
    }

    /// Read the table with four-point Catmull-Rom interpolation. This is
    /// the production lookup — see `ELIXIR-DESIGN.md` §4. Error vs ideal
    /// sin(2π·phase) is below -120 dBFS at 2048 samples per period.
    #[inline]
    pub fn lookup_catmull(&self, phase: u32) -> f32 {
        let idx = (phase >> FRAC_BITS) as usize;
        let t = (phase & FRAC_MASK) as f32 * FRAC_TO_F32;
        let p0 = self.data[(idx + TABLE_SIZE - 1) & TABLE_MASK];
        let p1 = self.data[idx];
        let p2 = self.data[(idx + 1) & TABLE_MASK];
        let p3 = self.data[(idx + 2) & TABLE_MASK];
        let t2 = t * t;
        let t3 = t2 * t;
        0.5 * ((2.0 * p1)
            + (-p0 + p2) * t
            + (2.0 * p0 - 5.0 * p1 + 4.0 * p2 - p3) * t2
            + (-p0 + 3.0 * p1 - 3.0 * p2 + p3) * t3)
    }

    /// Raw table access for tests and future spectral-mip-map building.
    #[inline]
    pub fn as_slice(&self) -> &[f32] {
        &self.data
    }
}

impl Default for SineTable {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn table_size_matches_bits() {
        assert_eq!(TABLE_SIZE, 2048);
        assert_eq!(FRAC_BITS, 21);
    }

    #[test]
    fn sine_table_bounds() {
        let t = SineTable::new();
        for &v in t.as_slice() {
            assert!((-1.0..=1.0).contains(&v));
        }
    }

    #[test]
    fn sine_table_known_values() {
        let t = SineTable::new();
        assert!((t.as_slice()[0] - 0.0).abs() < 1e-6);
        let quarter = t.as_slice()[TABLE_SIZE / 4];
        assert!((quarter - 1.0).abs() < 1e-3);
        let half = t.as_slice()[TABLE_SIZE / 2];
        assert!(half.abs() < 1e-3);
    }

    #[test]
    fn lookup_at_index_zero() {
        let t = SineTable::new();
        assert!((t.lookup_linear(0) - 0.0).abs() < 1e-6);
        assert!((t.lookup_catmull(0) - 0.0).abs() < 1e-6);
    }

    #[test]
    fn catmull_beats_linear_on_sine() {
        let t = SineTable::new();
        // Sample sub-table points (between integer indices) and compare
        // against the analytic sin value.
        let mut lin_err = 0.0f64;
        let mut cat_err = 0.0f64;
        let n = 4096u32;
        for k in 0..n {
            let phase = ((k as u64 * (1u64 << 32) / n as u64) as u32).wrapping_add(0x000F_FF80);
            let analytic =
                libm::sinf(core::f32::consts::TAU * (phase as f64 / (1u64 << 32) as f64) as f32);
            lin_err += (t.lookup_linear(phase) as f64 - analytic as f64).powi(2);
            cat_err += (t.lookup_catmull(phase) as f64 - analytic as f64).powi(2);
        }
        let lin_rms = (lin_err / n as f64).sqrt();
        let cat_rms = (cat_err / n as f64).sqrt();
        // Catmull must reduce sub-sample error vs linear. With a
        // 2048-sample sine table both methods are already extremely
        // accurate, so we only require a meaningful, repeatable
        // improvement — not orders of magnitude.
        assert!(
            cat_rms < lin_rms * 0.5,
            "expected catmull < 50% of linear RMS: cat={cat_rms}, lin={lin_rms}"
        );
        // Absolute Catmull error must be tiny vs full-scale.
        let cat_dbfs = 20.0 * cat_rms.log10();
        assert!(
            cat_dbfs < -100.0,
            "catmull RMS error {cat_dbfs} dBFS not below -100 dBFS"
        );
    }
}
