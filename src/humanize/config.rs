//! Configuration types for the humanization system.
//!
//! This module defines the configuration structure for humanization effects
//! and the wrapper type for humanized notes with their computed offsets.

use serde::{Deserialize, Serialize};
use wmidi::{Channel, Note, Velocity};

/// Master configuration for all humanization parameters.
///
/// Controls timing jitter, velocity variation, swing/groove, and tempo settings.
/// All parameters can be adjusted during playback without restarting.
///
/// # Parameter Ranges
///
/// | Parameter | Typical Range | Effect |
/// |-----------|---------------|--------|
/// | `jitter_min_ms` / `jitter_max_ms` | 1-30ms | Subtle: 1-10ms, Loose: 10-30ms |
/// | `velocity_variation` | 5-25 | Subtle: 5-10, Expressive: 15-25 |
/// | `swing_amount` | 0.0-0.5 | Straight: 0.0, Light swing: 0.2, Jazz: 0.4-0.5 |
/// | `duration_variation_ms` | 0-50ms | Note length extension for legato feel |
///
/// # Example
///
/// ```ignore
/// use contrapunk::humanize::HumanizeConfig;
///
/// let mut config = HumanizeConfig::default();
/// config.enabled = true;
/// config.jitter_enabled = true;
/// config.jitter_max_ms = 15;  // Subtle timing variation
/// config.velocity_enabled = true;
/// config.velocity_variation = 12;  // +/- 12 velocity units
/// config.swing_enabled = true;
/// config.swing_amount = 0.3;  // Medium swing feel
/// ```
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HumanizeConfig {
    /// Master toggle for all humanization effects.
    ///
    /// When `false`, notes pass through unchanged regardless of other settings.
    pub enabled: bool,

    /// Timing jitter toggle.
    ///
    /// When enabled, each note-on receives a random delay between
    /// `jitter_min_ms` and `jitter_max_ms`.
    pub jitter_enabled: bool,

    /// Minimum jitter in milliseconds.
    ///
    /// Lower bound for random timing offset. Typical values: 1-5ms.
    pub jitter_min_ms: u16,

    /// Maximum jitter in milliseconds.
    ///
    /// Upper bound for random timing offset. Typical values: 10-30ms.
    /// Values above 50ms may sound noticeably late.
    pub jitter_max_ms: u16,

    /// Velocity variation toggle.
    ///
    /// When enabled, note velocities are randomly adjusted by +/- `velocity_variation`.
    pub velocity_enabled: bool,

    /// Range +/- for velocity variation (0-127 scale).
    ///
    /// The actual velocity will be clamped to 1-127 after variation.
    /// Typical values: 5-20 for subtle dynamics, 20-40 for expressive.
    pub velocity_variation: u8,

    /// Duration variation toggle.
    ///
    /// When enabled, note-off events are delayed by a random amount up to
    /// `duration_variation_ms`, creating a more legato feel.
    pub duration_enabled: bool,

    /// Maximum duration extension in milliseconds.
    ///
    /// How much longer notes can ring out. Applied to note-off timing.
    /// Typical values: 10-50ms. Higher values create overlap between notes.
    pub duration_variation_ms: u16,

    /// Swing toggle.
    ///
    /// When enabled, off-beat notes (those falling between beats) are delayed
    /// based on `swing_amount`, creating a shuffle or swing feel.
    pub swing_enabled: bool,

    /// Swing amount (0.0 = straight, 1.0 = full triplet swing).
    ///
    /// - 0.0: No swing, straight timing
    /// - 0.2-0.3: Light shuffle
    /// - 0.4-0.5: Jazz swing
    /// - 0.6+: Extreme swing (rarely used)
    pub swing_amount: f32,

    /// Tempo in beats per minute.
    ///
    /// Used by [`BeatClock`](super::BeatClock) for swing calculations and metronome.
    pub bpm: f64,

    /// Beats per bar (time signature numerator).
    ///
    /// Common values: 4 (4/4 time), 3 (3/4 waltz), 6 (6/8 compound).
    pub beats_per_bar: u8,

    /// Beat unit (time signature denominator).
    ///
    /// Common values: 4 (quarter note), 8 (eighth note).
    pub beat_unit: u8,

    /// Whether to enable metronome clicks.
    ///
    /// When enabled, the [`Metronome`](super::Metronome) generates click
    /// notes on beat boundaries.
    pub metronome_enabled: bool,

    /// Output port index for metronome clicks.
    ///
    /// `None` means use the first available output port.
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
///
/// Wraps a MIDI note with computed timing and velocity offsets.
/// The [`Humanizer`](super::Humanizer) produces these, and the
/// [`DelayQueue`](super::DelayQueue) schedules them for later dispatch.
///
/// # Fields
///
/// - `delay_ms`: Total delay before sending (jitter + swing)
/// - `duration_delta_ms`: How much longer the note should ring
/// - `velocity`: Adjusted velocity (may differ from original)
///
/// # Example Flow
///
/// ```text
/// Note-On (C4, vel=100)
///     |
///     v
/// Humanizer.humanize_note_on()
///     |
///     v
/// HumanizedNote {
///     note: C4,
///     velocity: 94,        // vel=100 - 6 (random variation)
///     delay_ms: 12,        // 8ms jitter + 4ms swing
///     duration_delta_ms: 5 // extend note-off by 5ms
/// }
/// ```
#[derive(Clone, Debug)]
pub struct HumanizedNote {
    /// The MIDI note (pitch).
    pub note: Note,

    /// The MIDI channel (0-15).
    pub channel: Channel,

    /// The velocity after humanization (may differ from input).
    pub velocity: Velocity,

    /// Combined jitter + swing delay in milliseconds.
    ///
    /// The note should be sent this many milliseconds after the original
    /// note-on was received. For note-off events, this also includes
    /// the duration delta.
    pub delay_ms: u16,

    /// Duration delta in milliseconds (positive = extend note-off).
    ///
    /// Applied to note-off events to make notes ring slightly longer
    /// than their natural duration.
    pub duration_delta_ms: i16,

    /// Output port index for MIDI routing.
    pub port: usize,

    /// Harmony voice index (0 = melody, 1+ = harmony voices).
    ///
    /// Used for per-voice audio synth routing and future per-voice plugin
    /// chains. Distinct from `port` which is the physical MIDI output index.
    pub voice_index: u8,

    /// Whether this is a note-off event.
    ///
    /// Note-off events use the humanization record from the corresponding
    /// note-on to ensure matching timing characteristics.
    pub is_note_off: bool,
}
