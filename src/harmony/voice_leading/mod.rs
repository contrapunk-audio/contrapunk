//! Voice leading module: rule checkers, style presets, and chord-level voicing.

pub mod rules;
pub mod styles;
pub mod suspension;
pub mod voicer;

pub use rules::{
    check_motion_independence, check_parallel_fifths, check_parallel_octaves, check_spacing,
    check_voice_crossing, interval_class,
};
pub use styles::{StyleRules, VoiceLeadingStyle};
pub use suspension::SuspensionState;
pub use voicer::{revoice_chord, VoiceAnchor, VoiceRegister};
