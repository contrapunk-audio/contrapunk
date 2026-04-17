//! Humanization engine for adding human-like timing variation.
//!
//! This module provides timing jitter, velocity variation, groove/swing,
//! and beat clock tracking to make generated harmony notes feel more natural.
//!
//! # Overview
//!
//! The humanization system sits between the harmony engine and MIDI output,
//! adding subtle imperfections that make computer-generated harmonies feel
//! more like they were played by a human musician.
//!
//! ```text
//! Harmony Engine --> Humanizer --> Delay Queue --> MIDI Output
//!                        |              |
//!                   [BeatClock]    [Scheduled events]
//!                   [Config]       [Timing offsets]
//! ```
//!
//! # Main Types
//!
//! - [`BeatClock`] - Tracks musical time (BPM, beat position, bar boundaries)
//! - [`Humanizer`] - Applies humanization effects to notes
//! - [`HumanizeConfig`] - Configuration for humanization parameters
//! - [`HumanizedNote`] - A note with computed timing/velocity offsets
//! - [`DelayQueue`] - Schedules delayed note events for later dispatch
//! - [`Metronome`] - Optional audible click track synchronized to beat clock
//!
//! # Humanization Effects
//!
//! | Effect | Description | Typical Range |
//! |--------|-------------|---------------|
//! | Timing jitter | Random delay on note attacks | 1-30ms |
//! | Velocity variation | Random velocity offset | +/-10-20% |
//! | Swing/groove | Off-beat timing shift | 0.0-0.5 |
//! | Duration variation | Note length extension | 0-50ms |
//!
//! # Usage
//!
//! Humanization is applied after harmony generation, before notes are sent
//! to MIDI outputs. The melody note (voice index 0) bypasses humanization;
//! only harmony notes (voice index 1+) are affected, preserving the player's
//! original timing while adding variation to the generated parts.
//!
//! ```ignore
//! use contrapunk::humanize::{Humanizer, HumanizeConfig, BeatClock};
//!
//! let config = HumanizeConfig::default();
//! let mut humanizer = Humanizer::new(config);
//!
//! // Tick the beat clock on each frame
//! humanizer.tick(current_time_ms);
//!
//! // Humanize a note-on event
//! let humanized = humanizer.humanize_note_on(note, channel, velocity, port);
//! // humanized.delay_ms contains the computed timing offset
//! ```

pub mod beat_clock;
pub mod config;
pub mod engine;
pub mod groove;
pub mod metronome;
pub mod scheduler;

pub use beat_clock::BeatClock;
pub use config::{HumanizeConfig, HumanizedNote};
pub use engine::Humanizer;
pub use groove::{all_templates, GrooveTemplate};
pub use metronome::Metronome;
pub use scheduler::DelayQueue;
