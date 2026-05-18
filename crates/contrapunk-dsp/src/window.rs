//! Windowing and interpolation helpers.

use core::f32::consts::PI;

/// Equal-power dry/wet coefficients for a `0..1` mix value.
#[inline]
pub fn equal_power(mix: f32) -> (f32, f32) {
    let m = mix.clamp(0.0, 1.0);
    (libm::cosf(m * PI * 0.5), libm::sinf(m * PI * 0.5))
}

/// Fractional part in `[0, 1)` using `libm` for no_std builds.
#[inline]
pub fn frac01(x: f32) -> f32 {
    x - libm::floorf(x)
}

/// Fractional circular-buffer read with linear interpolation.
#[inline]
pub fn frac_read(buf: &[f32], write: usize, delay: f32) -> f32 {
    let len = buf.len();
    if len == 0 {
        return 0.0;
    }
    let d = delay.clamp(1.0, (len.saturating_sub(2)) as f32);
    let whole = libm::floorf(d) as usize;
    let frac = d - whole as f32;
    let i0 = (write + len - whole) % len;
    let i1 = (write + len - whole - 1) % len;
    buf[i0] * (1.0 - frac) + buf[i1] * frac
}
