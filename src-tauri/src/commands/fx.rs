//! Tauri commands for built-in FX blocks.
//!
//! Each command writes an atomic on the corresponding params struct
//! shared with the audio callback; changes take effect on the next
//! buffer.

use serde::Serialize;
use tauri::State;

use crate::state::AppState;

#[derive(Serialize)]
pub struct ReverbState {
    pub enabled: bool,
    pub mix: f32,
    pub room_size: f32,
    pub damping: f32,
}

#[tauri::command]
pub fn get_reverb_state(state: State<AppState>) -> ReverbState {
    let p = &state.reverb_params;
    ReverbState {
        enabled: p.enabled(),
        mix: p.mix(),
        room_size: p.room_size(),
        damping: p.damping(),
    }
}

#[tauri::command]
pub fn set_reverb_enabled(enabled: bool, state: State<AppState>) {
    state.reverb_params.set_enabled(enabled);
}

#[tauri::command]
pub fn set_reverb_mix(value: f32, state: State<AppState>) {
    state.reverb_params.set_mix(value);
}

#[tauri::command]
pub fn set_reverb_room_size(value: f32, state: State<AppState>) {
    state.reverb_params.set_room_size(value);
}

#[tauri::command]
pub fn set_reverb_damping(value: f32, state: State<AppState>) {
    state.reverb_params.set_damping(value);
}

#[derive(Serialize)]
pub struct DelayState {
    pub enabled: bool,
    pub mix: f32,
    pub time_ms: u32,
    pub feedback: f32,
}

#[tauri::command]
pub fn get_delay_state(state: State<AppState>) -> DelayState {
    let p = &state.delay_params;
    DelayState {
        enabled: p.enabled(),
        mix: p.mix(),
        time_ms: p.time_ms(),
        feedback: p.feedback(),
    }
}

#[tauri::command]
pub fn set_delay_enabled(enabled: bool, state: State<AppState>) {
    state.delay_params.set_enabled(enabled);
}

#[tauri::command]
pub fn set_delay_mix(value: f32, state: State<AppState>) {
    state.delay_params.set_mix(value);
}

#[tauri::command]
pub fn set_delay_time_ms(ms: u32, state: State<AppState>) {
    state.delay_params.set_time_ms(ms);
}

#[tauri::command]
pub fn set_delay_feedback(value: f32, state: State<AppState>) {
    state.delay_params.set_feedback(value);
}
