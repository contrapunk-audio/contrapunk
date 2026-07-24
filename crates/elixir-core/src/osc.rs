//! Fixed sine oscillator used by every Elixir voice.

/// One phase-accumulating sine oscillator.
pub(crate) struct Oscillator {
    phase: f32,
    phase_step: f32,
}

impl Oscillator {
    pub const fn new() -> Self {
        Self {
            phase: 0.0,
            phase_step: 0.0,
        }
    }

    pub fn start(&mut self, frequency_hz: f32, sample_rate: f32) {
        self.phase = 0.0;
        self.retune(frequency_hz, sample_rate);
    }

    pub fn retune(&mut self, frequency_hz: f32, sample_rate: f32) {
        self.phase_step = core::f32::consts::TAU * frequency_hz / sample_rate;
    }

    #[inline]
    pub fn tick(&mut self) -> f32 {
        let sample = libm::sinf(self.phase);
        self.phase += self.phase_step;
        if self.phase >= core::f32::consts::TAU {
            self.phase -= core::f32::consts::TAU;
        }
        sample
    }
}
