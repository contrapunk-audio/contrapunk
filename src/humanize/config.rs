//! Configuration types for the humanization system.
//!
//! This module defines the configuration structure for humanization effects
//! and the wrapper type for humanized notes with their computed offsets.

use serde::{Deserialize, Serialize};
use wmidi::{Channel, Note, Velocity};

// ---------------------------------------------------------------------------
// Metronome types
// ---------------------------------------------------------------------------

/// Accent level for a single beat in the metronome pattern.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum AccentLevel {
    /// Full accent (typically beat 1).
    Accent,
    /// Normal click.
    Normal,
    /// Soft ghost click.
    Ghost,
    /// Silent — skip this beat.
    Mute,
}

/// Subdivision depth for metronome clicks.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum MetronomeSubdivision {
    /// Quarter-note clicks only (whole beats).
    None,
    /// Add 8th-note subdivision clicks between beats.
    Eighth,
    /// Add 16th-note subdivision clicks between beats.
    Sixteenth,
}

impl Default for MetronomeSubdivision {
    fn default() -> Self {
        Self::None
    }
}

/// Sound set for metronome clicks (MIDI percussion note mappings).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum MetronomeSound {
    /// High/low woodblock (notes 76/77).
    Woodblock,
    /// Side stick / rimshot (notes 37/37 with velocity differentiation).
    Rimshot,
    /// Cowbell (notes 56/56 with velocity differentiation).
    Cowbell,
    /// Closed hi-hat (notes 42/42 with velocity differentiation).
    HiHat,
}

impl Default for MetronomeSound {
    fn default() -> Self {
        Self::Woodblock
    }
}

impl MetronomeSound {
    /// Returns (accent_note, normal_note, subdivision_note) MIDI note numbers.
    pub fn midi_notes(&self) -> (u8, u8, u8) {
        match self {
            Self::Woodblock => (76, 77, 42),
            Self::Rimshot => (37, 37, 42),
            Self::Cowbell => (56, 56, 42),
            Self::HiHat => (42, 42, 42),
        }
    }
}

// ---------------------------------------------------------------------------
// Velocity types
// ---------------------------------------------------------------------------

/// Velocity variation distribution shape.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum VelocityDistribution {
    /// Flat random ±variation (original behavior).
    Uniform,
    /// Bell curve centered on original velocity.
    Gaussian,
    /// Asymmetric: more likely softer, occasionally louder.
    Pareto,
}

impl Default for VelocityDistribution {
    fn default() -> Self {
        Self::Uniform
    }
}

// ---------------------------------------------------------------------------
// Per-voice overrides
// ---------------------------------------------------------------------------

/// Per-voice scaling multipliers for humanization parameters.
/// All default to 1.0 (no override).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PerVoiceOverride {
    /// Multiplier for jitter (0.0 = no jitter, 1.0 = full, 2.0 = double).
    #[serde(default = "one")]
    pub jitter_scale: f32,
    /// Multiplier for swing amount.
    #[serde(default = "one")]
    pub swing_scale: f32,
    /// Multiplier for velocity variation.
    #[serde(default = "one")]
    pub velocity_scale: f32,
    /// Multiplier for duration variation.
    #[serde(default = "one")]
    pub duration_scale: f32,
}

fn one() -> f32 {
    1.0
}

impl Default for PerVoiceOverride {
    fn default() -> Self {
        Self {
            jitter_scale: 1.0,
            swing_scale: 1.0,
            velocity_scale: 1.0,
            duration_scale: 1.0,
        }
    }
}

// ---------------------------------------------------------------------------
// Swing types
// ---------------------------------------------------------------------------

/// How swing delay is interpolated from the swing amount.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum SwingCurve {
    /// Proportional delay: delay = amount * half-beat duration.
    Linear,
    /// Snap toward triplet feel (2:1 at amount=0.33, 3:1 at amount=0.5).
    Triplet,
    /// Gentle onset, steep at high values (Logic Pro style).
    Logarithmic,
}

impl Default for SwingCurve {
    fn default() -> Self {
        Self::Linear
    }
}

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
    ///
    /// Legacy field: when `swing_8th_amount` and `swing_16th_amount` are both
    /// zero (the default), this value is used as the 8th-note swing amount
    /// for backward compatibility.
    pub swing_amount: f32,

    /// Swing amount for 8th-note offbeats (0.0-1.0).
    ///
    /// When non-zero, overrides `swing_amount` for 8th-note swing.
    #[serde(default)]
    pub swing_8th_amount: f32,

    /// Swing amount for 16th-note offbeats (0.0-1.0).
    #[serde(default)]
    pub swing_16th_amount: f32,

    /// Swing interpolation curve.
    #[serde(default)]
    pub swing_curve: SwingCurve,

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

    /// Metronome subdivision depth (None / Eighth / Sixteenth).
    #[serde(default)]
    pub metronome_subdivision: MetronomeSubdivision,

    /// Per-beat accent pattern. Length should match `beats_per_bar`.
    ///
    /// Default: `[Accent, Normal, Normal, Normal]` for 4/4 time.
    #[serde(default = "default_accent_pattern")]
    pub metronome_accent_pattern: Vec<AccentLevel>,

    /// Volume multiplier for metronome clicks (0.0-1.0).
    ///
    /// Applied to all click velocities before clamping to 1-127.
    #[serde(default = "default_metronome_volume")]
    pub metronome_volume: f32,

    /// Sound set for metronome clicks.
    #[serde(default)]
    pub metronome_sound: MetronomeSound,

    /// Number of count-in bars before routing starts (0 = disabled).
    #[serde(default)]
    pub metronome_count_in_bars: u8,

    /// Velocity variation distribution shape.
    #[serde(default)]
    pub velocity_distribution: VelocityDistribution,

    /// Accent strength on strong beats (0.0 = disabled, 1.0 = full).
    /// Boosts velocity on beats 0 and 2 (in 4/4) by up to 30%.
    #[serde(default)]
    pub accent_strength: f32,

    /// Per-voice humanization scale multipliers.
    /// Index matches voice_index (0 = melody, 1+ = harmony voices).
    /// Empty vec = all voices use global settings (scale 1.0).
    #[serde(default)]
    pub per_voice_overrides: Vec<PerVoiceOverride>,

    /// Currently active groove template, if any. Set by `GrooveTemplate::apply()`,
    /// cleared to `None` when the user manually adjusts parameters.
    #[serde(default)]
    pub groove_template: Option<super::groove::GrooveTemplate>,
}

static DEFAULT_VOICE_OVERRIDE: PerVoiceOverride = PerVoiceOverride {
    jitter_scale: 1.0,
    swing_scale: 1.0,
    velocity_scale: 1.0,
    duration_scale: 1.0,
};

impl HumanizeConfig {
    /// Returns the per-voice override for the given voice index.
    /// Falls back to a static default (all 1.0) if the index is out of bounds.
    pub fn voice_override(&self, voice_index: u8) -> &PerVoiceOverride {
        self.per_voice_overrides
            .get(voice_index as usize)
            .unwrap_or(&DEFAULT_VOICE_OVERRIDE)
    }
}

fn default_accent_pattern() -> Vec<AccentLevel> {
    vec![
        AccentLevel::Accent,
        AccentLevel::Normal,
        AccentLevel::Normal,
        AccentLevel::Normal,
    ]
}

fn default_metronome_volume() -> f32 {
    1.0
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
            swing_8th_amount: 0.0,
            swing_16th_amount: 0.0,
            swing_curve: SwingCurve::default(),
            bpm: 120.0,
            beats_per_bar: 4,
            beat_unit: 4,
            metronome_enabled: false,
            metronome_output_port: None,
            metronome_subdivision: MetronomeSubdivision::default(),
            metronome_accent_pattern: default_accent_pattern(),
            metronome_volume: 1.0,
            metronome_sound: MetronomeSound::default(),
            metronome_count_in_bars: 0,
            velocity_distribution: VelocityDistribution::default(),
            accent_strength: 0.0,
            per_voice_overrides: Vec::new(),
            groove_template: None,
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
