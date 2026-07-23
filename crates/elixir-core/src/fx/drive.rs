//! Soft-clip distortion (Phase 21.A5).
//!
//! Pointwise nonlinearity: `out = mix * tanh(drive * in) + (1 - mix) * in`.
//! Stateless (no buffer state), so the audio thread cost is trivial.
//! Higher-quality bandlimited drive lands in A6 with the wave-folder /
//! bit-crush / downsample family from the design doc.

#[derive(Clone, Copy, Debug)]
pub struct Drive {
    /// Pre-gain multiplier into the nonlinearity. `1.0` is unity;
    /// audible breakup typically starts around 2–4.
    pub drive: f32,
    /// Wet/dry crossfade, `0..1`.
    pub mix: f32,
}

impl Drive {
    pub const fn new() -> Self {
        Self {
            drive: 1.0,
            mix: 1.0,
        }
    }

    pub fn with_drive(drive: f32) -> Self {
        let mut value = Self::new();
        value.set_drive(drive);
        value
    }

    pub fn set_drive(&mut self, drive: f32) {
        if drive.is_finite() {
            self.drive = drive;
        }
    }

    pub fn set_mix(&mut self, mix: f32) {
        crate::util::set_finite_clamped(&mut self.mix, mix, 0.0, 1.0);
    }

    pub fn process_inplace(&mut self, buf: &mut [f32]) {
        let drive = if self.drive.is_finite() {
            self.drive.max(0.0)
        } else {
            1.0
        };
        let mix = if self.mix.is_finite() {
            self.mix.clamp(0.0, 1.0)
        } else {
            0.0
        };
        let dry_w = 1.0 - mix;
        for s in buf.iter_mut() {
            let driven = libm::tanhf(*s * drive);
            *s = dry_w * *s + mix * driven;
        }
    }
}

impl Default for Drive {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unity_drive_pass_through_at_zero_mix() {
        let mut d = Drive::new();
        d.mix = 0.0;
        let mut buf = [0.5f32, -0.3, 0.8, -0.9];
        let orig = buf;
        d.process_inplace(&mut buf);
        for (a, b) in buf.iter().zip(orig.iter()) {
            assert!((a - b).abs() < 1e-6);
        }
    }

    #[test]
    fn high_drive_clips_at_unity() {
        let mut d = Drive::with_drive(20.0);
        let mut buf = [1.5f32; 4];
        d.process_inplace(&mut buf);
        for s in buf.iter() {
            assert!(s.abs() < 1.0001);
        }
    }
}
