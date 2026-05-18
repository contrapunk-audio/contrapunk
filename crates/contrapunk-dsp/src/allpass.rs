//! All-pass filter primitives.

#[derive(Clone, Copy, Debug)]
pub struct Allpass1 {
    z: f32,
}

impl Allpass1 {
    pub const fn new() -> Self {
        Self { z: 0.0 }
    }

    pub fn reset(&mut self) {
        self.z = 0.0;
    }

    #[inline]
    pub fn tick(&mut self, x: f32, a: f32) -> f32 {
        let y = -a * x + self.z;
        self.z = x + a * y;
        y
    }
}

impl Default for Allpass1 {
    fn default() -> Self {
        Self::new()
    }
}
