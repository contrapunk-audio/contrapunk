//! Plugin hosting for Contrapunk.
//!
//! Contrapunk positions as an OSS Gig Performer — that requires hosting
//! third-party instrument and effect plugins inside the app's audio
//! chain. This module is the home for those integrations. CLAP comes
//! first because it is open, royalty-free, and the Rust tooling (
//! `clack-host`) is the most ergonomic path to a working host.
//!
//! VST3 and AU are follow-up work; each will live in its own submodule
//! so dependencies stay isolated.

pub mod clap;
