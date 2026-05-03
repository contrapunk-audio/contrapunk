//! Pattern lane — rhythmic gate on held inputs.
//!
//! Currently a placeholder that re-exports the existing `PatternConfig`
//! and `PatternInputMode` from `crate::state`. Increment 1.2 moves the
//! type definitions here; until then this exists so other companion
//! modules can refer to `crate::companion::CompanionPattern` even though
//! the algorithm still lives in state.rs.

#[allow(unused_imports)]
pub use crate::state::{PatternConfig as CompanionPattern, PatternInputMode};
