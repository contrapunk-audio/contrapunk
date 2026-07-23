//! Small math helpers.
//!
//! Generic DSP helpers live in `contrapunk-dsp`; this module preserves
//! the original Elixir import path while avoiding duplicate math.

pub use contrapunk_dsp::pitch::midi_to_freq;

#[inline]
pub(crate) fn finite_or(value: f32, fallback: f32) -> f32 {
    if value.is_finite() {
        value
    } else {
        fallback
    }
}

#[inline]
pub(crate) fn set_finite_clamped(target: &mut f32, value: f32, min: f32, max: f32) {
    if value.is_finite() {
        *target = value.clamp(min, max);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a4_is_440() {
        assert!((midi_to_freq(69) - 440.0).abs() < 1e-3);
    }

    #[test]
    fn c4_is_about_261() {
        assert!((midi_to_freq(60) - 261.625_5).abs() < 1e-2);
    }

    #[test]
    fn a5_is_880() {
        assert!((midi_to_freq(81) - 880.0).abs() < 1e-2);
    }
}
