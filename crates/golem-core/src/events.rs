//! Internal drum event types. These are not MIDI events.

/// Drum kit piece addressed by the drummer brain.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum DrumPiece {
    Kick,
    Snare,
    ClosedHat,
    OpenHat,
    Ride,
    Crash,
    TomLow,
    TomMid,
    TomHigh,
}

/// Hit articulation. v0.1 only renders a subset, but the shape is
/// sampler-ready.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum Articulation {
    Center,
    Rimshot,
    Ghost,
    Flam,
    Edge,
}

/// A scheduled drum hit inside the current audio block.
#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct DrumHit {
    pub piece: DrumPiece,
    pub articulation: Articulation,
    /// Normalized hit strength.
    pub velocity: f32,
    /// Frame offset within the current output buffer.
    pub offset_frames: u32,
}

impl Default for DrumHit {
    fn default() -> Self {
        Self {
            piece: DrumPiece::Kick,
            articulation: Articulation::Center,
            velocity: 0.0,
            offset_frames: 0,
        }
    }
}
