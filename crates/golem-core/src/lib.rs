//! Golem core — audio-native adaptive drummer engine.
//!
//! This crate intentionally has no Tauri, cpal, Contrapunk MIDI-router,
//! or UI dependency. Hosts provide clock/follow snapshots and receive
//! generated drum audio in an interleaved output buffer.

pub mod dynamics;
pub mod engine;
pub mod events;
pub mod follow;
pub mod params;
pub mod style;

pub use dynamics::{AdaptiveDynamics, DrummerIntent};
pub use engine::{ClockSnapshot, Engine};
pub use events::{Articulation, DrumHit, DrumPiece};
pub use follow::{
    amp_to_db, FollowInput, Follower, InputCalibration, PlayerFeatures, RawInputLevel,
};
pub use params::{EngineParams, SharedParams};
pub use style::Style;
