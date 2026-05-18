//! Pitch/frequency helpers.

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
    fn standard_tuning_points() {
        assert!((midi_to_freq(69) - 440.0).abs() < 1e-3);
        assert!((midi_to_freq(60) - 261.625_5).abs() < 1e-2);
        assert!((midi_to_freq(81) - 880.0).abs() < 1e-2);
    }
}
