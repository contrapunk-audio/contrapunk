//! Fixed-capacity delay-line primitives.

extern crate alloc;
use alloc::vec::Vec;

use crate::window::frac_read;

/// Stereo delay line with one shared write head and fractional reads.
/// Allocates at construction only.
pub struct StereoDelayLine {
    left: Vec<f32>,
    right: Vec<f32>,
    write: usize,
}

impl StereoDelayLine {
    pub fn new_power_of_two(max_delay_samples: usize) -> Self {
        let n = max_delay_samples.max(4).next_power_of_two();
        let mut left = Vec::with_capacity(n);
        left.resize(n, 0.0);
        let mut right = Vec::with_capacity(n);
        right.resize(n, 0.0);
        Self {
            left,
            right,
            write: 0,
        }
    }

    #[inline]
    pub fn tick(
        &mut self,
        in_l: f32,
        in_r: f32,
        delay_l: f32,
        delay_r: f32,
        feedback: f32,
    ) -> (f32, f32) {
        let out_l = frac_read(&self.left, self.write, delay_l);
        let out_r = frac_read(&self.right, self.write, delay_r);
        self.left[self.write] = in_l + out_l * feedback;
        self.right[self.write] = in_r + out_r * feedback;
        self.write = (self.write + 1) & (self.left.len() - 1);
        (out_l, out_r)
    }
}
