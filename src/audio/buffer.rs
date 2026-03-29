//! Optimized buffer management for guitar pitch detection.
//!
//! This module provides a dual-buffer analysis strategy and overlap management
//! for real-time pitch detection of guitar signals. Guitar strings span a wide
//! frequency range (E2 ~82Hz to E4 ~330Hz for standard tuning), requiring
//! different buffer sizes for accurate detection at different frequencies.
//!
//! # Components
//!
//! - [`RingBuffer`] -- Fixed-capacity circular buffer with no heap allocation
//!   after construction.
//! - [`OverlapManager`] -- Produces overlapping analysis frames (50% or 75%)
//!   from a continuous audio stream.
//! - [`DualBufferAnalyzer`] -- Routes audio to fast (512-sample) and slow
//!   (1024-sample) analysis paths for high and low guitar strings respectively.

// ---------------------------------------------------------------------------
// RingBuffer
// ---------------------------------------------------------------------------

/// A fixed-capacity circular buffer that avoids heap allocation after
/// construction.
///
/// All storage is allocated once in [`RingBuffer::new`]. Subsequent
/// [`push`](RingBuffer::push) calls never allocate. Elements are always
/// stored in FIFO order so that [`as_slice`](RingBuffer::as_slice) returns
/// a contiguous, oldest-to-newest view in O(1).
#[derive(Debug, Clone)]
pub struct RingBuffer<T> {
    buf: Vec<T>,
    cap: usize,
    count: usize,
}

impl<T: Clone + Default> RingBuffer<T> {
    /// Create a new ring buffer with the given fixed capacity.
    ///
    /// # Panics
    ///
    /// Panics if `capacity` is 0.
    pub fn new(capacity: usize) -> Self {
        assert!(capacity > 0, "RingBuffer capacity must be > 0");
        Self {
            buf: vec![T::default(); capacity],
            cap: capacity,
            count: 0,
        }
    }

    /// Push a value into the buffer. When the buffer is full the oldest
    /// element is evicted.
    pub fn push(&mut self, value: T) {
        if self.count < self.cap {
            // Buffer not yet full -- simple append.
            self.buf[self.count] = value;
            self.count += 1;
        } else {
            // Buffer full -- rotate left by one (evict oldest) and place
            // the new value at the end. `rotate_left` compiles to an
            // efficient memmove on contiguous memory.
            self.buf.rotate_left(1);
            self.buf[self.cap - 1] = value;
        }
    }

    /// Return the buffer contents in FIFO order (oldest to newest).
    ///
    /// Because the internal storage is always kept in logical order this
    /// is an O(1) slice operation with no copying.
    pub fn as_slice(&self) -> &[T] {
        &self.buf[..self.count]
    }

    /// Number of elements currently stored.
    pub fn len(&self) -> usize {
        self.count
    }

    /// Whether the buffer is empty.
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }
}

// ---------------------------------------------------------------------------
// OverlapManager
// ---------------------------------------------------------------------------

/// Manages overlapping analysis frames from a continuous audio stream.
///
/// At 75% overlap with a 1024-sample buffer the hop size is 256 samples,
/// yielding a new frame approximately every 5.8 ms at 44.1 kHz.
///
/// At 50% overlap the hop is 512 samples (~11.6 ms at 44.1 kHz).
#[derive(Debug, Clone)]
pub struct OverlapManager {
    /// Internal ring buffer holding incoming samples.
    ring: RingBuffer<f32>,
    /// Total analysis frame size.
    buffer_size: usize,
    /// Number of new samples between consecutive frames.
    hop: usize,
    /// Samples accumulated since the last emitted frame.
    samples_since_frame: usize,
}

impl OverlapManager {
    /// Create a new overlap manager.
    ///
    /// # Panics
    ///
    /// Panics if `overlap_percent` is not 50 or 75, or if `buffer_size` is 0.
    pub fn new(buffer_size: usize, overlap_percent: u8) -> Self {
        assert!(buffer_size > 0, "buffer_size must be > 0");
        assert!(
            overlap_percent == 50 || overlap_percent == 75,
            "overlap_percent must be 50 or 75, got {overlap_percent}"
        );
        let hop = match overlap_percent {
            75 => buffer_size / 4,
            50 => buffer_size / 2,
            _ => unreachable!(),
        };
        Self {
            ring: RingBuffer::new(buffer_size),
            buffer_size,
            hop,
            samples_since_frame: 0,
        }
    }

    /// Feed incoming audio samples and collect any completed analysis frames.
    ///
    /// Each returned `Vec<f32>` is a full-length frame of `buffer_size`
    /// samples, produced every [`hop_size`](Self::hop_size) new samples once
    /// the internal ring buffer has been primed (filled to capacity).
    pub fn feed(&mut self, samples: &[f32]) -> Vec<Vec<f32>> {
        let mut frames = Vec::new();
        for &s in samples {
            self.ring.push(s);
            self.samples_since_frame += 1;

            // Emit a frame every hop samples, but only once the ring is full.
            if self.ring.len() == self.buffer_size && self.samples_since_frame >= self.hop {
                frames.push(self.ring.as_slice().to_vec());
                self.samples_since_frame = 0;
            }
        }
        frames
    }

    /// The hop size (number of new samples between consecutive frames).
    pub fn hop_size(&self) -> usize {
        self.hop
    }

    /// Estimated number of analysis frames per second at the given sample rate.
    pub fn frames_per_second(&self, sample_rate: usize) -> f32 {
        sample_rate as f32 / self.hop as f32
    }
}

// ---------------------------------------------------------------------------
// DualBufferAnalyzer
// ---------------------------------------------------------------------------

/// Frequency threshold separating the fast and slow analysis paths.
/// G3 = 196 Hz.  Signals with a spectral centroid at or above this
/// frequency are routed to the fast (512-sample) path.
const HIGH_STRING_THRESHOLD_HZ: f32 = 196.0;

/// A buffer that is ready for analysis.
#[derive(Debug, Clone, PartialEq)]
pub enum BufferReady {
    /// A 512-sample frame suitable for high-string (>= G3) analysis.
    FastBuffer(Vec<f32>),
    /// A 1024-sample frame suitable for low-string (< G3) analysis.
    SlowBuffer(Vec<f32>),
}

/// Dual-buffer analyzer that maintains two parallel analysis paths optimized
/// for different frequency ranges of the guitar.
///
/// * **Fast path** -- 512 samples.  Can resolve frequencies down to ~172 Hz
///   at 44.1 kHz (`44100 / 256 ~ 172`).  Suitable for G3 (196 Hz) and above.
/// * **Slow path** -- 1024 samples.  Resolves down to ~86 Hz at 44.1 kHz,
///   covering the lowest guitar string (E2, 82.4 Hz) comfortably.
///
/// Because the fast buffer is half the size of the slow buffer it fills --
/// and therefore produces frames -- at twice the rate.
#[derive(Debug, Clone)]
pub struct DualBufferAnalyzer {
    fast_buf: Vec<f32>,
    slow_buf: Vec<f32>,
    fast_pos: usize,
    slow_pos: usize,
    sample_rate: usize,
}

impl DualBufferAnalyzer {
    /// Fast-path buffer length.
    pub const FAST_SIZE: usize = 512;
    /// Slow-path buffer length.
    pub const SLOW_SIZE: usize = 1024;

    /// Create a new analyzer for the given sample rate.
    pub fn new(sample_rate: usize) -> Self {
        Self {
            fast_buf: vec![0.0; Self::FAST_SIZE],
            slow_buf: vec![0.0; Self::SLOW_SIZE],
            fast_pos: 0,
            slow_pos: 0,
            sample_rate,
        }
    }

    /// Feed a single sample into both analysis paths.
    ///
    /// Returns a list of buffers that are now ready for analysis.  The fast
    /// buffer will be ready twice as often as the slow buffer.
    pub fn feed(&mut self, sample: f32) -> Vec<BufferReady> {
        let mut ready = Vec::new();

        // Fast path
        self.fast_buf[self.fast_pos] = sample;
        self.fast_pos += 1;
        if self.fast_pos == Self::FAST_SIZE {
            ready.push(BufferReady::FastBuffer(self.fast_buf.clone()));
            self.fast_pos = 0;
        }

        // Slow path
        self.slow_buf[self.slow_pos] = sample;
        self.slow_pos += 1;
        if self.slow_pos == Self::SLOW_SIZE {
            ready.push(BufferReady::SlowBuffer(self.slow_buf.clone()));
            self.slow_pos = 0;
        }

        ready
    }

    /// Determine whether a buffer likely contains a high-frequency signal by
    /// computing the spectral centroid over significant DFT bins.
    ///
    /// Returns `true` when the centroid is at or above
    /// [`HIGH_STRING_THRESHOLD_HZ`] (196 Hz / G3), meaning the fast-path
    /// result is likely sufficient.  Otherwise the caller should wait for
    /// the slow path.
    pub fn is_high_frequency(buffer: &[f32], sample_rate: usize) -> bool {
        let centroid = spectral_centroid(buffer, sample_rate);
        centroid >= HIGH_STRING_THRESHOLD_HZ
    }

    /// The sample rate this analyzer was constructed with.
    pub fn sample_rate(&self) -> usize {
        self.sample_rate
    }
}

// ---------------------------------------------------------------------------
// Spectral centroid (internal)
// ---------------------------------------------------------------------------

/// Compute the spectral centroid of a real-valued signal buffer.
///
/// Only DFT bins whose magnitude exceeds 10% of the peak bin magnitude
/// are included, which prevents spectral leakage from low-energy bins
/// dragging the centroid away from the true dominant frequency.
///
/// ```text
/// centroid = Sum(f_k * |X_k|) / Sum(|X_k|)   for |X_k| > 0.1 * max|X_k|
/// ```
fn spectral_centroid(buffer: &[f32], sample_rate: usize) -> f32 {
    let n = buffer.len();
    if n == 0 {
        return 0.0;
    }
    let half = n / 2;

    // First pass: compute magnitudes and find the peak.
    let mut magnitudes: Vec<f64> = Vec::with_capacity(half);
    let mut peak_mag: f64 = 0.0;

    for k in 1..=half {
        let mut re: f64 = 0.0;
        let mut im: f64 = 0.0;
        let angle_step = 2.0 * std::f64::consts::PI * k as f64 / n as f64;
        for (i, &s) in buffer.iter().enumerate() {
            let angle = angle_step * i as f64;
            re += s as f64 * angle.cos();
            im -= s as f64 * angle.sin();
        }
        let mag = (re * re + im * im).sqrt();
        if mag > peak_mag {
            peak_mag = mag;
        }
        magnitudes.push(mag);
    }

    if peak_mag < 1e-12 {
        return 0.0;
    }

    // Second pass: centroid using only significant bins (> 10% of peak).
    let threshold = peak_mag * 0.1;
    let mut weighted_sum: f64 = 0.0;
    let mut magnitude_sum: f64 = 0.0;

    for (idx, &mag) in magnitudes.iter().enumerate() {
        if mag > threshold {
            let k = idx + 1;
            let freq = k as f64 * sample_rate as f64 / n as f64;
            weighted_sum += freq * mag;
            magnitude_sum += mag;
        }
    }

    if magnitude_sum < 1e-12 {
        0.0
    } else {
        (weighted_sum / magnitude_sum) as f32
    }
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // -----------------------------------------------------------------------
    // RingBuffer tests
    // -----------------------------------------------------------------------

    #[test]
    fn ring_buffer_maintains_fifo_order() {
        let mut rb = RingBuffer::new(4);
        rb.push(1);
        rb.push(2);
        rb.push(3);
        assert_eq!(rb.as_slice(), &[1, 2, 3]);
        assert_eq!(rb.len(), 3);
    }

    #[test]
    fn ring_buffer_wraps_correctly_at_capacity() {
        let mut rb = RingBuffer::new(3);
        rb.push(1);
        rb.push(2);
        rb.push(3);
        assert_eq!(rb.as_slice(), &[1, 2, 3]);

        // Pushing a 4th element should evict the oldest (1).
        rb.push(4);
        assert_eq!(rb.as_slice(), &[2, 3, 4]);
        assert_eq!(rb.len(), 3);

        // Continue wrapping.
        rb.push(5);
        assert_eq!(rb.as_slice(), &[3, 4, 5]);

        rb.push(6);
        rb.push(7);
        assert_eq!(rb.as_slice(), &[5, 6, 7]);
    }

    #[test]
    fn ring_buffer_single_capacity() {
        let mut rb = RingBuffer::new(1);
        rb.push(10);
        assert_eq!(rb.as_slice(), &[10]);
        rb.push(20);
        assert_eq!(rb.as_slice(), &[20]);
    }

    // -----------------------------------------------------------------------
    // DualBufferAnalyzer tests
    // -----------------------------------------------------------------------

    #[test]
    fn dual_buffer_fast_frames_at_2x_slow_rate() {
        let mut analyzer = DualBufferAnalyzer::new(44100);
        let mut fast_count = 0usize;
        let mut slow_count = 0usize;

        // Feed exactly 2 * SLOW_SIZE samples (= 2048 = 4 * FAST_SIZE).
        let total_samples = DualBufferAnalyzer::SLOW_SIZE * 2;
        for i in 0..total_samples {
            let sample = (i as f32 * 0.01).sin();
            for ready in analyzer.feed(sample) {
                match ready {
                    BufferReady::FastBuffer(_) => fast_count += 1,
                    BufferReady::SlowBuffer(_) => slow_count += 1,
                }
            }
        }

        // 2048 / 512 = 4 fast frames, 2048 / 1024 = 2 slow frames.
        assert_eq!(fast_count, 4, "expected 4 fast frames");
        assert_eq!(slow_count, 2, "expected 2 slow frames");
        assert_eq!(
            fast_count,
            slow_count * 2,
            "fast frames should be 2x slow frames"
        );
    }

    #[test]
    fn dual_buffer_frame_sizes_correct() {
        let mut analyzer = DualBufferAnalyzer::new(44100);
        let total = DualBufferAnalyzer::SLOW_SIZE;
        for i in 0..total {
            let sample = (i as f32 * 0.01).sin();
            for ready in analyzer.feed(sample) {
                match &ready {
                    BufferReady::FastBuffer(v) => {
                        assert_eq!(v.len(), DualBufferAnalyzer::FAST_SIZE);
                    }
                    BufferReady::SlowBuffer(v) => {
                        assert_eq!(v.len(), DualBufferAnalyzer::SLOW_SIZE);
                    }
                }
            }
        }
    }

    // -----------------------------------------------------------------------
    // OverlapManager tests
    // -----------------------------------------------------------------------

    #[test]
    fn overlap_75_produces_frames_every_hop() {
        let buffer_size = 1024;
        let mut om = OverlapManager::new(buffer_size, 75);
        assert_eq!(om.hop_size(), 256);

        // Prime the ring buffer by feeding buffer_size samples.
        let prime: Vec<f32> = (0..buffer_size).map(|i| i as f32).collect();
        let frame_count = om.feed(&prime).len();

        // After priming we should have gotten one frame (at sample 1024).
        assert_eq!(frame_count, 1, "one frame after priming");

        // Feed exactly one hop worth of samples -- should produce exactly
        // one more frame.
        let hop_chunk: Vec<f32> = (0..256).map(|i| i as f32).collect();
        let frames = om.feed(&hop_chunk);
        assert_eq!(frames.len(), 1, "one frame per hop after priming");
        assert_eq!(frames[0].len(), buffer_size);
    }

    #[test]
    fn overlap_50_produces_frames_every_half_buffer() {
        let buffer_size = 1024;
        let mut om = OverlapManager::new(buffer_size, 50);
        assert_eq!(om.hop_size(), 512);

        // Prime.
        let prime: Vec<f32> = (0..buffer_size).map(|i| i as f32).collect();
        let frames = om.feed(&prime);
        assert_eq!(frames.len(), 1, "one frame after priming");

        // Feed half-buffer more samples.
        let hop_chunk: Vec<f32> = (0..512).map(|i| i as f32).collect();
        let frames = om.feed(&hop_chunk);
        assert_eq!(frames.len(), 1, "one frame per half-buffer hop");
    }

    #[test]
    fn overlap_75_multiple_frames() {
        let buffer_size = 1024;
        let mut om = OverlapManager::new(buffer_size, 75);

        // Feed 1024 + 4*256 = 2048 samples total.
        let samples: Vec<f32> = (0..2048).map(|i| i as f32).collect();
        let frames = om.feed(&samples);

        // First frame at sample 1024, then one each at 1280, 1536, 1792,
        // 2048.  Total = 5 frames.
        assert_eq!(frames.len(), 5);
    }

    // -----------------------------------------------------------------------
    // Hop size and frame rate tests
    // -----------------------------------------------------------------------

    #[test]
    fn hop_sizes_correct() {
        let om75 = OverlapManager::new(1024, 75);
        assert_eq!(om75.hop_size(), 256);

        let om50 = OverlapManager::new(1024, 50);
        assert_eq!(om50.hop_size(), 512);
    }

    #[test]
    fn frames_per_second_44100() {
        let om75 = OverlapManager::new(1024, 75);
        let fps = om75.frames_per_second(44100);
        // 44100 / 256 = 172.265625
        assert!(
            (fps - 172.265_63).abs() < 0.01,
            "44100/256 ~ 172.27, got {fps}"
        );

        let om50 = OverlapManager::new(1024, 50);
        let fps = om50.frames_per_second(44100);
        // 44100 / 512 = 86.1328125
        assert!(
            (fps - 86.132_81).abs() < 0.01,
            "44100/512 ~ 86.13, got {fps}"
        );
    }

    #[test]
    fn frames_per_second_48000() {
        let om75 = OverlapManager::new(1024, 75);
        let fps = om75.frames_per_second(48000);
        // 48000 / 256 = 187.5
        assert!((fps - 187.5).abs() < 0.01, "48000/256 = 187.5, got {fps}");

        let om50 = OverlapManager::new(1024, 50);
        let fps = om50.frames_per_second(48000);
        // 48000 / 512 = 93.75
        assert!((fps - 93.75).abs() < 0.01, "48000/512 = 93.75, got {fps}");
    }

    // -----------------------------------------------------------------------
    // Spectral centroid routing
    // -----------------------------------------------------------------------

    #[test]
    fn spectral_centroid_high_frequency_detection() {
        // Generate a 440 Hz sine wave (A4, well above G3/196 Hz).
        let sr = 44100;
        let n = 512;
        let buf: Vec<f32> = (0..n)
            .map(|i| {
                let t = i as f32 / sr as f32;
                (2.0 * std::f32::consts::PI * 440.0 * t).sin()
            })
            .collect();
        assert!(
            DualBufferAnalyzer::is_high_frequency(&buf, sr),
            "440 Hz should be detected as high frequency"
        );
    }

    #[test]
    fn spectral_centroid_low_frequency_detection() {
        // Generate an 82.4 Hz sine wave (E2, the lowest guitar string).
        let sr = 44100;
        let n = 1024;
        let buf: Vec<f32> = (0..n)
            .map(|i| {
                let t = i as f32 / sr as f32;
                (2.0 * std::f32::consts::PI * 82.4 * t).sin()
            })
            .collect();
        assert!(
            !DualBufferAnalyzer::is_high_frequency(&buf, sr),
            "82.4 Hz should be detected as low frequency"
        );
    }
}
