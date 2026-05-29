//! Drummer style selection.

/// High-level groove family.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum Style {
    Rock,
    HalfTime,
    FourOnFloor,
}

impl Default for Style {
    fn default() -> Self {
        Self::Rock
    }
}

impl Style {
    pub fn as_u8(self) -> u8 {
        match self {
            Self::Rock => 0,
            Self::HalfTime => 1,
            Self::FourOnFloor => 2,
        }
    }

    pub fn from_u8(value: u8) -> Self {
        match value {
            1 => Self::HalfTime,
            2 => Self::FourOnFloor,
            _ => Self::Rock,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Rock => "rock",
            Self::HalfTime => "half-time",
            Self::FourOnFloor => "four-on-floor",
        }
    }

    pub fn parse(value: &str) -> Self {
        match value.trim().to_ascii_lowercase().as_str() {
            "half-time" | "halftime" | "half_time" => Self::HalfTime,
            "four-on-floor" | "four_on_floor" | "four" | "disco" => Self::FourOnFloor,
            _ => Self::Rock,
        }
    }
}
