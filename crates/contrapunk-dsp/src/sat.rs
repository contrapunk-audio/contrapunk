//! Saturation helpers.

#[inline]
pub fn quick_tanh(x: f32) -> f32 {
    let x = x.clamp(-8.0, 8.0);
    x * (27.0 + x * x) / (27.0 + 9.0 * x * x)
}
