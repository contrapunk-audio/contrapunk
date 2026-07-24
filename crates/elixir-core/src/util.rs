//! Pitch helper shared with Contrapunk.

pub use contrapunk_dsp::pitch::midi_to_freq;

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
