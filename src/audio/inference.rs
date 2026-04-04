//! Pure Rust CNN inference for the Contrapunk guitar classifier.
//!
//! Implements forward-pass inference for the trained Pure CNN model that
//! classifies guitar audio (as mel-spectrograms) into 138 string+fret classes
//! (6 strings x 23 frets). No external ML dependencies -- all tensor
//! operations are hand-written.
//!
//! # Model Architecture
//!
//! ```text
//! Input: (1, 1, 64, 94) -- batch=1, channels=1, n_mels=64, time_frames=94
//!
//! Conv2d(1->32, 3x3, pad=1)  -> BN(32) -> ReLU -> MaxPool2d(2)
//! Conv2d(32->64, 3x3, pad=1) -> BN(64) -> ReLU -> MaxPool2d(2)
//! Conv2d(64->128, 3x3, pad=1) -> BN(128) -> ReLU -> MaxPool2d(2)
//! Conv2d(128->256, 3x3, pad=1) -> BN(256) -> ReLU -> AdaptiveAvgPool2d(1,1)
//! Flatten -> Linear(256->138)
//! ```
//!
//! # Tensor Layout
//!
//! All tensors are stored as flat `Vec<f32>` in row-major (C-contiguous) order:
//!
//! - **Feature maps** `(C, H, W)`: element at `(c, h, w)` is at index `c * H * W + h * W + w`
//! - **Conv weights** `(out_c, in_c, kH, kW)`: index `oc * in_c * kH * kW + ic * kH * kW + kh * kW + kw`
//! - **Linear weights** `(out_features, in_features)`: index `o * in_features + i`

// ============================================================================
// Label mapping: class_id -> (string_idx, fret, midi_note)
// ============================================================================

/// Standard guitar tuning open-string MIDI notes.
/// String 0 = low E (E2 = 40), String 5 = high E (E4 = 64).
const STRING_OPEN_MIDI: [u8; 6] = [40, 45, 50, 55, 59, 64];

/// Number of frets per string (0 = open, 1..22 = fretted).
const FRETS_PER_STRING: usize = 23;

/// Total number of output classes: 6 strings x 23 fret positions = 138.
pub const NUM_CLASSES: usize = 6 * FRETS_PER_STRING;

/// Input mel-spectrogram dimensions.
pub const MEL_BINS: usize = 64;
pub const TIME_FRAMES: usize = 94;

/// BatchNorm epsilon (PyTorch default).
const BN_EPS: f32 = 1e-5;

/// Decode a class_id (0..137) into `(string_idx, fret, midi_note)`.
///
/// Mapping: `class_id = string_idx * 23 + fret`
#[inline]
pub fn decode_class(class_id: usize) -> (usize, usize, u8) {
    let string_idx = class_id / FRETS_PER_STRING;
    let fret = class_id % FRETS_PER_STRING;
    let midi_note = STRING_OPEN_MIDI[string_idx] + fret as u8;
    (string_idx, fret, midi_note)
}

// ============================================================================
// Prediction result
// ============================================================================

/// Result of running the guitar classifier on a single mel-spectrogram.
#[derive(Debug, Clone)]
pub struct GuitarPrediction {
    /// Predicted class index (0..137).
    pub class_id: usize,
    /// Guitar string index (0 = low E, 5 = high E).
    pub string_idx: usize,
    /// Fret number (0 = open string, 1..22).
    pub fret: usize,
    /// Softmax confidence for the predicted class.
    pub confidence: f32,
    /// MIDI note number for the predicted class.
    pub midi_note: u8,
    /// Top 3 predictions as `(class_id, confidence)`.
    pub top3: [(usize, f32); 3],
}

// ============================================================================
// Classifier weights
// ============================================================================

/// All model weights stored as flat f32 vectors.
///
/// Weight shapes (PyTorch convention, row-major):
/// - Conv weight: `(out_c, in_c, kH, kW)`
/// - Conv bias: `(out_c,)`
/// - BN gamma (weight): `(channels,)`
/// - BN beta (bias): `(channels,)`
/// - BN running_mean: `(channels,)`
/// - BN running_var: `(channels,)`
/// - Linear weight: `(out_features, in_features)`
/// - Linear bias: `(out_features,)`
struct ClassifierWeights {
    // Block 1: Conv2d(1->32, 3x3) + BN(32)
    conv1_w: Vec<f32>,
    conv1_b: Vec<f32>,
    bn1_w: Vec<f32>,
    bn1_b: Vec<f32>,
    bn1_mean: Vec<f32>,
    bn1_var: Vec<f32>,

    // Block 2: Conv2d(32->64, 3x3) + BN(64)
    conv2_w: Vec<f32>,
    conv2_b: Vec<f32>,
    bn2_w: Vec<f32>,
    bn2_b: Vec<f32>,
    bn2_mean: Vec<f32>,
    bn2_var: Vec<f32>,

    // Block 3: Conv2d(64->128, 3x3) + BN(128)
    conv3_w: Vec<f32>,
    conv3_b: Vec<f32>,
    bn3_w: Vec<f32>,
    bn3_b: Vec<f32>,
    bn3_mean: Vec<f32>,
    bn3_var: Vec<f32>,

    // Block 4: Conv2d(128->256, 3x3) + BN(256)
    conv4_w: Vec<f32>,
    conv4_b: Vec<f32>,
    bn4_w: Vec<f32>,
    bn4_b: Vec<f32>,
    bn4_mean: Vec<f32>,
    bn4_var: Vec<f32>,

    // Fully connected: Linear(256->138)
    fc_w: Vec<f32>,
    fc_b: Vec<f32>,
}

// ============================================================================
// GuitarClassifier
// ============================================================================

/// Pure-Rust CNN inference engine for the guitar string+fret classifier.
///
/// Holds all model weights in memory and runs the forward pass on
/// mel-spectrograms to produce class predictions.
pub struct GuitarClassifier {
    weights: ClassifierWeights,
}

impl GuitarClassifier {
    /// Load model weights from a raw byte buffer.
    ///
    /// Intended for use with `include_bytes!()` to embed the weights at
    /// compile time in the final binary.
    ///
    /// # Binary format
    ///
    /// ```text
    /// [4 bytes: magic "GCNN"]
    /// [4 bytes: u32 version = 1]
    /// [4 bytes: u32 n_classes]
    /// [4 bytes: u32 n_layers]
    /// For each layer:
    ///   [4 bytes: u32 name_len]
    ///   [name_len bytes: layer name as UTF-8]
    ///   [4 bytes: u32 n_elements]
    ///   [n_elements * 4 bytes: f32 values little-endian row-major]
    /// ```
    ///
    /// # Panics
    ///
    /// Panics if the data is malformed, has the wrong magic/version, or
    /// is missing expected layers.
    pub fn from_bytes(data: &[u8]) -> Self {
        assert!(data.len() >= 16, "Weight data too short for header");

        // -- Parse header --
        assert_eq!(&data[0..4], b"GCNN", "Bad magic bytes");
        let version = read_u32_le(&data[4..8]);
        assert_eq!(version, 1, "Unsupported weight format version {}", version);
        let n_classes = read_u32_le(&data[8..12]) as usize;
        assert_eq!(
            n_classes, NUM_CLASSES,
            "Weight file has {} classes, expected {}",
            n_classes, NUM_CLASSES
        );
        let n_layers = read_u32_le(&data[12..16]) as usize;

        // -- Read all layers into a map --
        let mut offset = 16usize;
        let mut layers: Vec<(String, Vec<f32>)> = Vec::with_capacity(n_layers);
        for _ in 0..n_layers {
            let name_len = read_u32_le(&data[offset..offset + 4]) as usize;
            offset += 4;
            let name = std::str::from_utf8(&data[offset..offset + name_len])
                .expect("Invalid UTF-8 in layer name")
                .to_string();
            offset += name_len;
            let n_elements = read_u32_le(&data[offset..offset + 4]) as usize;
            offset += 4;
            let byte_len = n_elements * 4;
            let values = read_f32_slice(&data[offset..offset + byte_len], n_elements);
            offset += byte_len;
            layers.push((name, values));
        }

        // -- Helper: find a layer by name --
        let find = |name: &str| -> Vec<f32> {
            layers
                .iter()
                .find(|(n, _)| n == name)
                .unwrap_or_else(|| panic!("Missing weight layer: {}", name))
                .1
                .clone()
        };

        GuitarClassifier {
            weights: ClassifierWeights {
                // Block 1: features.0 = Conv2d, features.1 = BN
                conv1_w: find("features.0.weight"),
                conv1_b: find("features.0.bias"),
                bn1_w: find("features.1.weight"),
                bn1_b: find("features.1.bias"),
                bn1_mean: find("features.1.running_mean"),
                bn1_var: find("features.1.running_var"),

                // Block 2: features.4 = Conv2d, features.5 = BN
                conv2_w: find("features.4.weight"),
                conv2_b: find("features.4.bias"),
                bn2_w: find("features.5.weight"),
                bn2_b: find("features.5.bias"),
                bn2_mean: find("features.5.running_mean"),
                bn2_var: find("features.5.running_var"),

                // Block 3: features.8 = Conv2d, features.9 = BN
                conv3_w: find("features.8.weight"),
                conv3_b: find("features.8.bias"),
                bn3_w: find("features.9.weight"),
                bn3_b: find("features.9.bias"),
                bn3_mean: find("features.9.running_mean"),
                bn3_var: find("features.9.running_var"),

                // Block 4: features.12 = Conv2d, features.13 = BN
                conv4_w: find("features.12.weight"),
                conv4_b: find("features.12.bias"),
                bn4_w: find("features.13.weight"),
                bn4_b: find("features.13.bias"),
                bn4_mean: find("features.13.running_mean"),
                bn4_var: find("features.13.running_var"),

                // FC: classifier.2 = Linear
                fc_w: find("classifier.2.weight"),
                fc_b: find("classifier.2.bias"),
            },
        }
    }

    /// Create a classifier with all-zero weights (for testing / placeholder).
    fn with_zero_weights() -> Self {
        GuitarClassifier {
            weights: ClassifierWeights {
                conv1_w: vec![0.0; 32 * 1 * 3 * 3],
                conv1_b: vec![0.0; 32],
                bn1_w: vec![1.0; 32], // gamma = 1 so BN is identity when mean=0, var=1
                bn1_b: vec![0.0; 32],
                bn1_mean: vec![0.0; 32],
                bn1_var: vec![1.0; 32],

                conv2_w: vec![0.0; 64 * 32 * 3 * 3],
                conv2_b: vec![0.0; 64],
                bn2_w: vec![1.0; 64],
                bn2_b: vec![0.0; 64],
                bn2_mean: vec![0.0; 64],
                bn2_var: vec![1.0; 64],

                conv3_w: vec![0.0; 128 * 64 * 3 * 3],
                conv3_b: vec![0.0; 128],
                bn3_w: vec![1.0; 128],
                bn3_b: vec![0.0; 128],
                bn3_mean: vec![0.0; 128],
                bn3_var: vec![1.0; 128],

                conv4_w: vec![0.0; 256 * 128 * 3 * 3],
                conv4_b: vec![0.0; 256],
                bn4_w: vec![1.0; 256],
                bn4_b: vec![0.0; 256],
                bn4_mean: vec![0.0; 256],
                bn4_var: vec![1.0; 256],

                fc_w: vec![0.0; NUM_CLASSES * 256],
                fc_b: vec![0.0; NUM_CLASSES],
            },
        }
    }

    /// Run inference on a mel-spectrogram.
    ///
    /// # Arguments
    ///
    /// * `mel_spectrogram` - flat f32 slice of shape `(64, 94)` in row-major order
    ///   (64 mel bins x 94 time frames). Must have exactly `64 * 94 = 6016` elements.
    ///
    /// # Panics
    ///
    /// Panics if the input length is not `64 * 94`.
    pub fn predict(&self, mel_spectrogram: &[f32]) -> GuitarPrediction {
        assert_eq!(
            mel_spectrogram.len(),
            MEL_BINS * TIME_FRAMES,
            "Expected mel-spectrogram of size {} ({}x{}), got {}",
            MEL_BINS * TIME_FRAMES,
            MEL_BINS,
            TIME_FRAMES,
            mel_spectrogram.len()
        );

        let w = &self.weights;

        // Input is (1, H=64, W=94) -- single channel
        let (mut h, mut w_dim) = (MEL_BINS, TIME_FRAMES);

        // ---- Block 1: Conv(1->32) + BN + ReLU + MaxPool ----
        let mut x = conv2d(
            mel_spectrogram,
            &w.conv1_w,
            &w.conv1_b,
            1,
            32,
            h,
            w_dim,
            3,
            1,
        );
        x = batchnorm(
            &x,
            &w.bn1_w,
            &w.bn1_b,
            &w.bn1_mean,
            &w.bn1_var,
            32,
            h,
            w_dim,
            BN_EPS,
        );
        relu(&mut x);
        x = maxpool2d(&x, 32, h, w_dim);
        h /= 2; // 32
        w_dim /= 2; // 47

        // ---- Block 2: Conv(32->64) + BN + ReLU + MaxPool ----
        x = conv2d(&x, &w.conv2_w, &w.conv2_b, 32, 64, h, w_dim, 3, 1);
        x = batchnorm(
            &x,
            &w.bn2_w,
            &w.bn2_b,
            &w.bn2_mean,
            &w.bn2_var,
            64,
            h,
            w_dim,
            BN_EPS,
        );
        relu(&mut x);
        x = maxpool2d(&x, 64, h, w_dim);
        h /= 2; // 16
        w_dim /= 2; // 23

        // ---- Block 3: Conv(64->128) + BN + ReLU + MaxPool ----
        x = conv2d(&x, &w.conv3_w, &w.conv3_b, 64, 128, h, w_dim, 3, 1);
        x = batchnorm(
            &x,
            &w.bn3_w,
            &w.bn3_b,
            &w.bn3_mean,
            &w.bn3_var,
            128,
            h,
            w_dim,
            BN_EPS,
        );
        relu(&mut x);
        x = maxpool2d(&x, 128, h, w_dim);
        h /= 2; // 8
        w_dim /= 2; // 11

        // ---- Block 4: Conv(128->256) + BN + ReLU + AdaptiveAvgPool(1,1) ----
        x = conv2d(&x, &w.conv4_w, &w.conv4_b, 128, 256, h, w_dim, 3, 1);
        x = batchnorm(
            &x,
            &w.bn4_w,
            &w.bn4_b,
            &w.bn4_mean,
            &w.bn4_var,
            256,
            h,
            w_dim,
            BN_EPS,
        );
        relu(&mut x);
        x = global_avg_pool(&x, 256, h, w_dim);
        // x is now (256,)

        // ---- FC: Linear(256->138) ----
        let logits = linear(&x, &w.fc_w, &w.fc_b, 256, NUM_CLASSES);

        // ---- Decode prediction ----
        let probs = softmax(&logits);
        let top3 = top_k(&probs, 3);
        let class_id = top3[0].0;
        let confidence = top3[0].1;
        let (string_idx, fret, midi_note) = decode_class(class_id);

        GuitarPrediction {
            class_id,
            string_idx,
            fret,
            confidence,
            midi_note,
            top3: [top3[0], top3[1], top3[2]],
        }
    }
}

// ============================================================================
// Tensor operations
// ============================================================================

/// 2D convolution with zero-padding.
///
/// # Tensor layout
///
/// - `input`: shape `(in_c, H, W)`, flat row-major
/// - `weight`: shape `(out_c, in_c, kH, kW)`, flat row-major
/// - `bias`: shape `(out_c,)`
/// - Output: shape `(out_c, H, W)` (same spatial size when `padding = kernel/2`)
///
/// With `padding=1` and `kernel=3`, the output has the same spatial dimensions
/// as the input.
fn conv2d(
    input: &[f32],
    weight: &[f32],
    bias: &[f32],
    in_c: usize,
    out_c: usize,
    h: usize,
    w: usize,
    kernel: usize,
    padding: usize,
) -> Vec<f32> {
    let out_h = h + 2 * padding - kernel + 1;
    let out_w = w + 2 * padding - kernel + 1;
    let mut output = vec![0.0f32; out_c * out_h * out_w];

    let hw = h * w;
    let kk = kernel * kernel;
    let ic_kk = in_c * kk;

    for oc in 0..out_c {
        let bias_val = bias[oc];
        let oc_offset = oc * ic_kk;

        for oh in 0..out_h {
            for ow in 0..out_w {
                let mut sum = bias_val;

                for ic in 0..in_c {
                    let in_base = ic * hw;
                    let w_base = oc_offset + ic * kk;

                    for kh in 0..kernel {
                        let ih = oh + kh;
                        // Subtract padding to get the actual input coordinate.
                        // If ih < padding or ih >= h + padding, we are in the
                        // zero-padded region.
                        let ih_actual = ih as isize - padding as isize;
                        if ih_actual < 0 || ih_actual >= h as isize {
                            continue;
                        }
                        let ih_actual = ih_actual as usize;

                        for kw in 0..kernel {
                            let iw = ow + kw;
                            let iw_actual = iw as isize - padding as isize;
                            if iw_actual < 0 || iw_actual >= w as isize {
                                continue;
                            }
                            let iw_actual = iw_actual as usize;

                            let in_val = input[in_base + ih_actual * w + iw_actual];
                            let w_val = weight[w_base + kh * kernel + kw];
                            sum += in_val * w_val;
                        }
                    }
                }

                output[oc * out_h * out_w + oh * out_w + ow] = sum;
            }
        }
    }

    output
}

/// Batch normalization in inference mode.
///
/// Uses running statistics (not batch statistics). For each channel:
/// ```text
/// y = gamma * (x - running_mean) / sqrt(running_var + eps) + beta
/// ```
///
/// # Tensor layout
///
/// - `input`: shape `(C, H, W)`, flat row-major
/// - `gamma`, `beta`, `running_mean`, `running_var`: shape `(C,)`
/// - Output: same shape as input `(C, H, W)`
fn batchnorm(
    input: &[f32],
    gamma: &[f32],
    beta: &[f32],
    running_mean: &[f32],
    running_var: &[f32],
    channels: usize,
    h: usize,
    w: usize,
    eps: f32,
) -> Vec<f32> {
    let hw = h * w;
    let mut output = vec![0.0f32; channels * hw];

    for c in 0..channels {
        let g = gamma[c];
        let b = beta[c];
        let mean = running_mean[c];
        // Pre-compute the scale factor: gamma / sqrt(var + eps)
        let inv_std = g / (running_var[c] + eps).sqrt();
        let offset = c * hw;

        for i in 0..hw {
            output[offset + i] = (input[offset + i] - mean) * inv_std + b;
        }
    }

    output
}

/// ReLU activation (in-place).
///
/// Clamps all negative values to zero: `x = max(0, x)`.
fn relu(input: &mut [f32]) {
    for v in input.iter_mut() {
        if *v < 0.0 {
            *v = 0.0;
        }
    }
}

/// Max pooling with a 2x2 kernel and stride 2.
///
/// Reduces spatial dimensions by half (floor division for odd sizes).
///
/// # Tensor layout
///
/// - `input`: shape `(C, H, W)`, flat row-major
/// - Output: shape `(C, H/2, W/2)`
fn maxpool2d(input: &[f32], channels: usize, h: usize, w: usize) -> Vec<f32> {
    let out_h = h / 2;
    let out_w = w / 2;
    let mut output = vec![f32::NEG_INFINITY; channels * out_h * out_w];

    let in_hw = h * w;
    let out_hw = out_h * out_w;

    for c in 0..channels {
        let in_base = c * in_hw;
        let out_base = c * out_hw;

        for oh in 0..out_h {
            let ih = oh * 2;
            for ow in 0..out_w {
                let iw = ow * 2;

                // 2x2 pool window
                let v00 = input[in_base + ih * w + iw];
                let v01 = input[in_base + ih * w + iw + 1];
                let v10 = input[in_base + (ih + 1) * w + iw];
                let v11 = input[in_base + (ih + 1) * w + iw + 1];

                let max_val = v00.max(v01).max(v10).max(v11);
                output[out_base + oh * out_w + ow] = max_val;
            }
        }
    }

    output
}

/// Adaptive average pooling to (1, 1) -- global average pool.
///
/// Computes the mean of all spatial elements for each channel, producing
/// a vector of length `channels`.
///
/// # Tensor layout
///
/// - `input`: shape `(C, H, W)`, flat row-major
/// - Output: shape `(C,)`
fn global_avg_pool(input: &[f32], channels: usize, h: usize, w: usize) -> Vec<f32> {
    let hw = h * w;
    let hw_f = hw as f32;
    let mut output = vec![0.0f32; channels];

    for c in 0..channels {
        let offset = c * hw;
        let mut sum = 0.0f32;
        for i in 0..hw {
            sum += input[offset + i];
        }
        output[c] = sum / hw_f;
    }

    output
}

/// Fully-connected (linear) layer: `y = x * W^T + b`.
///
/// # Tensor layout
///
/// - `input`: shape `(in_features,)`
/// - `weight`: shape `(out_features, in_features)`, row-major (PyTorch convention)
/// - `bias`: shape `(out_features,)`
/// - Output: shape `(out_features,)`
fn linear(
    input: &[f32],
    weight: &[f32],
    bias: &[f32],
    in_features: usize,
    out_features: usize,
) -> Vec<f32> {
    let mut output = vec![0.0f32; out_features];

    for o in 0..out_features {
        let mut sum = bias[o];
        let w_offset = o * in_features;
        for i in 0..in_features {
            sum += weight[w_offset + i] * input[i];
        }
        output[o] = sum;
    }

    output
}

/// Softmax over a 1-D logit vector.
///
/// Uses the numerically stable variant: subtract max before exponentiating.
fn softmax(logits: &[f32]) -> Vec<f32> {
    let max_val = logits.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
    let exps: Vec<f32> = logits.iter().map(|&v| (v - max_val).exp()).collect();
    let sum: f32 = exps.iter().sum();
    exps.into_iter().map(|e| e / sum).collect()
}

/// Return the index of the maximum value.
pub fn argmax(logits: &[f32]) -> usize {
    let mut best_idx = 0;
    let mut best_val = f32::NEG_INFINITY;
    for (i, &v) in logits.iter().enumerate() {
        if v > best_val {
            best_val = v;
            best_idx = i;
        }
    }
    best_idx
}

/// Return the top-k `(index, value)` pairs sorted by descending value.
fn top_k(values: &[f32], k: usize) -> Vec<(usize, f32)> {
    let mut indexed: Vec<(usize, f32)> = values.iter().enumerate().map(|(i, &v)| (i, v)).collect();
    // Partial sort: we only need the top k, but for k=3 a full sort is fine.
    indexed.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    indexed.truncate(k);
    indexed
}

// ============================================================================
// Binary parsing helpers
// ============================================================================

/// Read a little-endian u32 from a 4-byte slice.
#[inline]
fn read_u32_le(bytes: &[u8]) -> u32 {
    u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]])
}

/// Read `n` little-endian f32 values from a byte slice.
fn read_f32_slice(bytes: &[u8], n: usize) -> Vec<f32> {
    let mut out = Vec::with_capacity(n);
    for i in 0..n {
        let start = i * 4;
        let val = f32::from_le_bytes([
            bytes[start],
            bytes[start + 1],
            bytes[start + 2],
            bytes[start + 3],
        ]);
        out.push(val);
    }
    out
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_test_zero_weights() {
        // With all-zero weights (and identity BN), a zero input should
        // produce all-zero logits, yielding uniform softmax (1/138 each).
        let classifier = GuitarClassifier::with_zero_weights();
        let input = vec![0.0f32; MEL_BINS * TIME_FRAMES];
        let pred = classifier.predict(&input);

        // Class ID should be valid
        assert!(pred.class_id < NUM_CLASSES);
        assert!(pred.string_idx < 6);
        assert!(pred.fret < FRETS_PER_STRING);

        // Confidence should be roughly 1/138 for uniform distribution
        let expected_uniform = 1.0 / NUM_CLASSES as f32;
        assert!(
            (pred.confidence - expected_uniform).abs() < 1e-4,
            "Expected ~{}, got {}",
            expected_uniform,
            pred.confidence
        );
    }

    #[test]
    fn smoke_test_nonzero_input() {
        // Verify that a non-zero input doesn't panic even with zero weights.
        let classifier = GuitarClassifier::with_zero_weights();
        let input: Vec<f32> = (0..MEL_BINS * TIME_FRAMES)
            .map(|i| (i as f32) * 0.001)
            .collect();
        let pred = classifier.predict(&input);
        assert!(pred.class_id < NUM_CLASSES);
    }

    #[test]
    fn test_decode_class() {
        // String 0 (E2), fret 0 -> MIDI 40
        assert_eq!(decode_class(0), (0, 0, 40));
        // String 0 (E2), fret 22 -> MIDI 62
        assert_eq!(decode_class(22), (0, 22, 62));
        // String 1 (A2), fret 0 -> MIDI 45
        assert_eq!(decode_class(23), (1, 0, 45));
        // String 5 (E4), fret 0 -> MIDI 64
        assert_eq!(decode_class(115), (5, 0, 64));
        // String 5 (E4), fret 22 -> MIDI 86
        assert_eq!(decode_class(137), (5, 22, 86));
    }

    #[test]
    fn test_argmax() {
        assert_eq!(argmax(&[1.0, 3.0, 2.0]), 1);
        assert_eq!(argmax(&[-5.0, -1.0, -3.0]), 1);
        assert_eq!(argmax(&[0.0]), 0);
    }

    #[test]
    fn test_softmax() {
        let probs = softmax(&[1.0, 2.0, 3.0]);
        let sum: f32 = probs.iter().sum();
        assert!(
            (sum - 1.0).abs() < 1e-6,
            "Softmax should sum to 1, got {}",
            sum
        );
        // Values should be in ascending order
        assert!(probs[0] < probs[1]);
        assert!(probs[1] < probs[2]);
    }

    #[test]
    fn test_relu() {
        let mut data = vec![-2.0, -1.0, 0.0, 1.0, 2.0];
        relu(&mut data);
        assert_eq!(data, vec![0.0, 0.0, 0.0, 1.0, 2.0]);
    }

    #[test]
    fn test_maxpool2d_simple() {
        // 1 channel, 4x4 input -> 2x2 output
        #[rustfmt::skip]
        let input = vec![
            1.0, 2.0, 3.0, 4.0,
            5.0, 6.0, 7.0, 8.0,
            9.0, 10.0, 11.0, 12.0,
            13.0, 14.0, 15.0, 16.0,
        ];
        let output = maxpool2d(&input, 1, 4, 4);
        assert_eq!(output, vec![6.0, 8.0, 14.0, 16.0]);
    }

    #[test]
    fn test_global_avg_pool() {
        // 2 channels, 2x2 each
        let input = vec![
            1.0, 2.0, 3.0, 4.0, // channel 0: mean = 2.5
            10.0, 20.0, 30.0, 40.0, // channel 1: mean = 25.0
        ];
        let output = global_avg_pool(&input, 2, 2, 2);
        assert!((output[0] - 2.5).abs() < 1e-6);
        assert!((output[1] - 25.0).abs() < 1e-6);
    }

    #[test]
    fn test_linear_simple() {
        // 2 inputs -> 3 outputs
        let input = vec![1.0, 2.0];
        // Weight (3, 2): each row is one output neuron's weights
        let weight = vec![
            1.0, 0.0, // out 0: 1*1 + 0*2 = 1
            0.0, 1.0, // out 1: 0*1 + 1*2 = 2
            1.0, 1.0, // out 2: 1*1 + 1*2 = 3
        ];
        let bias = vec![0.5, -0.5, 0.0];
        let output = linear(&input, &weight, &bias, 2, 3);
        assert!((output[0] - 1.5).abs() < 1e-6);
        assert!((output[1] - 1.5).abs() < 1e-6);
        assert!((output[2] - 3.0).abs() < 1e-6);
    }

    #[test]
    fn test_batchnorm_identity() {
        // With gamma=1, beta=0, mean=0, var=1, BN should be (approximately) identity.
        let input = vec![1.0, 2.0, 3.0, 4.0];
        let gamma = vec![1.0];
        let beta = vec![0.0];
        let mean = vec![0.0];
        let var = vec![1.0];
        let output = batchnorm(&input, &gamma, &beta, &mean, &var, 1, 2, 2, BN_EPS);
        for (a, b) in output.iter().zip(input.iter()) {
            assert!((a - b).abs() < 1e-4, "Expected ~{}, got {}", b, a);
        }
    }

    #[test]
    fn test_conv2d_bias_only() {
        // With zero weights, conv output should equal the bias for each channel.
        let input = vec![1.0; 1 * 4 * 4]; // 1 channel, 4x4
        let weight = vec![0.0; 2 * 1 * 3 * 3]; // 2 output channels, all zero
        let bias = vec![5.0, -3.0];
        let output = conv2d(&input, &weight, &bias, 1, 2, 4, 4, 3, 1);
        // Output shape: (2, 4, 4)
        assert_eq!(output.len(), 2 * 4 * 4);
        // Channel 0 should all be 5.0
        for i in 0..16 {
            assert!((output[i] - 5.0).abs() < 1e-6);
        }
        // Channel 1 should all be -3.0
        for i in 16..32 {
            assert!((output[i] - (-3.0)).abs() < 1e-6);
        }
    }

    #[test]
    #[should_panic(expected = "Bad magic bytes")]
    fn test_from_bytes_bad_magic() {
        GuitarClassifier::from_bytes(&[0u8; 16]);
    }

    #[test]
    #[should_panic(expected = "Expected mel-spectrogram")]
    fn test_wrong_input_size_panics() {
        let classifier = GuitarClassifier::with_zero_weights();
        let bad_input = vec![0.0f32; 100]; // wrong size
        classifier.predict(&bad_input);
    }

    #[test]
    fn test_from_bytes_real_weights() {
        // Load real exported weights from the .bin file
        let weights_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("ml/training/round_01/weights/guitar_classifier.bin");
        let data = std::fs::read(&weights_path).unwrap_or_else(|e| {
            panic!(
                "Failed to read weight file at {}: {}. Run `python ml/training/export_weights_bin.py` first.",
                weights_path.display(),
                e
            )
        });

        let classifier = GuitarClassifier::from_bytes(&data);

        // Run inference on a zeros input
        let input = vec![0.0f32; MEL_BINS * TIME_FRAMES];
        let pred = classifier.predict(&input);

        // Output should have a valid class
        assert!(
            pred.class_id < NUM_CLASSES,
            "class_id {} out of range",
            pred.class_id
        );
        assert!(pred.string_idx < 6);
        assert!(pred.fret < FRETS_PER_STRING);

        // With real weights, softmax output should NOT be uniform --
        // the confidence for the top class should differ from 1/138.
        let uniform = 1.0 / NUM_CLASSES as f32;
        assert!(
            (pred.confidence - uniform).abs() > 1e-6,
            "Real weights should produce non-uniform output, got confidence {} (uniform = {})",
            pred.confidence,
            uniform
        );

        // Verify top3 are valid and sorted
        assert!(pred.top3[0].1 >= pred.top3[1].1);
        assert!(pred.top3[1].1 >= pred.top3[2].1);
        for &(cls, conf) in &pred.top3 {
            assert!(cls < NUM_CLASSES, "top3 class {} out of range", cls);
            assert!(
                conf >= 0.0 && conf <= 1.0,
                "top3 confidence {} out of range",
                conf
            );
        }
    }

    #[test]
    fn test_binary_parsing_helpers() {
        // read_u32_le
        assert_eq!(read_u32_le(&[0x01, 0x00, 0x00, 0x00]), 1);
        assert_eq!(read_u32_le(&[0xFF, 0x00, 0x00, 0x00]), 255);
        assert_eq!(read_u32_le(&[0x00, 0x01, 0x00, 0x00]), 256);

        // read_f32_slice
        let one_bytes = 1.0f32.to_le_bytes();
        let two_bytes = 2.0f32.to_le_bytes();
        let mut buf = Vec::new();
        buf.extend_from_slice(&one_bytes);
        buf.extend_from_slice(&two_bytes);
        let vals = read_f32_slice(&buf, 2);
        assert_eq!(vals.len(), 2);
        assert!((vals[0] - 1.0).abs() < 1e-10);
        assert!((vals[1] - 2.0).abs() < 1e-10);
    }
}
