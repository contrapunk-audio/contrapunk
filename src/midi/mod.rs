#[cfg(not(target_arch = "wasm32"))]
pub mod input;
#[cfg(not(target_arch = "wasm32"))]
pub mod output;
#[cfg(not(target_arch = "wasm32"))]
pub mod ports;

#[cfg(target_arch = "wasm32")]
pub mod web;
