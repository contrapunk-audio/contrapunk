//! Microphone and audio analysis configuration types.

use serde::{Deserialize, Serialize};

// ── Onset Detection Defaults ────────────────────────────────────────

/// HFC threshold ratio: onset fires when HFC > mean * this ratio.
pub const DEFAULT_HFC_THRESHOLD_RATIO: f32 = 3.0;

/// Spectral flux threshold ratio: onset fires when flux > mean * this ratio.
pub const DEFAULT_FLUX_THRESHOLD_RATIO: f32 = 2.5;

/// Amplitude slope threshold: onset fires when frame energy jump exceeds this.
pub const DEFAULT_AMPLITUDE_SLOPE_THRESHOLD: f32 = 2.0;

/// Number of frames to keep in the running history for adaptive thresholds.
pub const DEFAULT_HISTORY_LEN: usize = 20;

/// Configuration for microphone input.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MicConfig {
    /// Volume threshold (0.0-1.0) - signals below this are ignored
    pub volume_threshold: f32,
    /// Confidence threshold (0.0-1.0) - detections below this are ignored
    pub confidence_threshold: f32,
    /// Buffer size in samples (affects latency vs accuracy tradeoff)
    pub buffer_size: usize,
    /// Minimum MIDI note to detect (C2 = 36)
    pub min_note: u8,
    /// Maximum MIDI note to detect (C6 = 84)
    pub max_note: u8,
    /// Hysteresis threshold in cents to prevent vibrato triggering note changes
    pub hysteresis_cents: i8,
    /// Minimum note duration in ms before allowing note change
    pub min_note_duration_ms: u32,
}

impl Default for MicConfig {
    fn default() -> Self {
        Self {
            volume_threshold: 0.01,    // -40dB
            confidence_threshold: 0.7, // 70% clarity
            buffer_size: 2048,         // ~46ms at 44.1kHz
            min_note: 36,              // C2
            max_note: 84,              // C6
            hysteresis_cents: 40,      // 40 cents threshold
            min_note_duration_ms: 50,  // 50ms minimum
        }
    }
}
