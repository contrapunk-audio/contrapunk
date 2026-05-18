//! Small feedback-matrix helpers.

/// 16-point Hadamard transform. Caller applies normalization.
pub fn hadamard16(input: &[f32; 16], out: &mut [f32; 16]) {
    *out = *input;
    let mut step = 1;
    while step < 16 {
        let jump = step * 2;
        let mut i = 0;
        while i < 16 {
            let mut j = 0;
            while j < step {
                let a = out[i + j];
                let b = out[i + j + step];
                out[i + j] = a + b;
                out[i + j + step] = a - b;
                j += 1;
            }
            i += jump;
        }
        step = jump;
    }
}
