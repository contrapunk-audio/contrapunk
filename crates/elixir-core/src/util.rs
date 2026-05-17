//! Small math helpers.
//!
//! Kept separate so A2+ can extend without churning the engine surface.

/// Convert MIDI note number → frequency in Hz.
///
/// A4 = note 69 = 440 Hz; each semitone is `2^(1/12)`.
#[inline]
pub fn midi_to_freq(note: u8) -> f32 {
    440.0 * libm::powf(2.0, (note as f32 - 69.0) / 12.0)
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
