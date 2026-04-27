//! Guitar→MIDI pipeline rewrite.
//!
//! See [issue #82](https://github.com/contrapunk-audio/contrapunk/issues/82)
//! for the design context. This module is the home for the rewritten
//! pipeline. The legacy implementation continues to live in
//! [`crate::audio::guitar_input`] and is the runtime default; new stages
//! land here under [`new`] and become opt-in via runtime config.
//!
//! Phase 0 (this commit): trait skeleton only — [`stages`] defines the
//! per-stage contracts the rewritten pipeline composes, [`new`] is empty
//! and will be filled in by subsequent phases. The legacy code is
//! untouched.

pub mod new;
pub mod stages;
