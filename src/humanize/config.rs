use serde::{Serialize, Deserialize};
use wmidi::{Channel, Note, Velocity};

/// Master configuration for all humanization parameters.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HumanizeConfig {
    /// Master toggle for all humanization.
    pub enabled: bool,

    /// Timing jitter toggle.
    pub jitter_enabled: bool,
    /// Minimum jitter in milliseconds.
    pub jitter_min_ms: u16,
    /// Maximum jitter in milliseconds.
    pub jitter_max_ms: u16,

    /// Velocity variation toggle.
    pub velocity_enabled: bool,
    /// Range +/- for velocity variation.
    pub velocity_variation: u8,

    /// Duration variation toggle.
    pub duration_enabled: bool,
    /// Maximum duration extension in milliseconds.
    pub duration_variation_ms: u16,

    /// Swing toggle.
    pub swing_enabled: bool,
    /// Swing amount (0.0 = none, 1.0 = full triplet swing).
    pub swing_amount: f32,

    /// Tempo in beats per minute.
    pub bpm: f64,
    /// Beats per bar (time signature numerator).
    pub beats_per_bar: u8,
    /// Beat unit (time signature denominator).
    pub beat_unit: u8,

    /// Whether to enable metronome clicks.
    pub metronome_enabled: bool,
    /// Output port index for metronome clicks (None = first output).
    pub metronome_output_port: Option<usize>,
}

impl Default for HumanizeConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            jitter_enabled: false,
            jitter_min_ms: 1,
            jitter_max_ms: 10,
            velocity_enabled: false,
            velocity_variation: 15,
            duration_enabled: false,
            duration_variation_ms: 20,
            swing_enabled: false,
            swing_amount: 0.0,
            bpm: 120.0,
            beats_per_bar: 4,
            beat_unit: 4,
            metronome_enabled: false,
            metronome_output_port: None,
        }
    }
}

/// A note after humanization has been applied.
#[derive(Clone, Debug)]
pub struct HumanizedNote {
    pub note: Note,
    pub channel: Channel,
    pub velocity: Velocity,
    /// Combined jitter + swing delay in milliseconds.
    pub delay_ms: u16,
    /// Duration delta in milliseconds (positive = extend note-off).
    pub duration_delta_ms: i16,
    /// Output port index.
    pub port: usize,
    /// Whether this is a note-off event.
    pub is_note_off: bool,
}
