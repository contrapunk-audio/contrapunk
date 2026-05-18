//! Shared DSP primitives for Contrapunk and Elixir.
//!
//! This crate intentionally contains host-agnostic building blocks only:
//! no `AudioBlock`, no atomics, no `Transport`, no UI/plugin/Tauri glue.
//! Higher-level crates wrap these primitives for each surface.

#![cfg_attr(not(any(test, feature = "std")), no_std)]

extern crate alloc;

pub mod allpass;
pub mod delay_line;
pub mod matrix;
pub mod pitch;
pub mod sat;
pub mod window;
