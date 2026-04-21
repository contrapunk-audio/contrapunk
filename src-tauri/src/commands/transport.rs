//! Tauri commands for transport control (play, stop, BPM, time sig).

use serde::Serialize;
use tauri::State;

use crate::state::AppState;

/// Snapshot of current transport state for the UI.
#[derive(Serialize)]
pub struct TransportState {
    pub running: bool,
    pub bpm: f64,
    pub beats_per_bar: u8,
    pub beat_unit: u8,
    pub sample_rate: u32,
    pub sample_pos: u64,
    pub beat_position: f64,
    pub bar: u64,
}

#[tauri::command]
pub fn get_transport_state(state: State<AppState>) -> TransportState {
    let t = &state.transport;
    let (beats_per_bar, beat_unit) = t.time_signature();
    TransportState {
        running: t.is_running(),
        bpm: t.bpm(),
        beats_per_bar,
        beat_unit,
        sample_rate: t.sample_rate(),
        sample_pos: t.sample_pos(),
        beat_position: t.beat_position(),
        bar: t.bar(),
    }
}

#[tauri::command]
pub fn transport_play(state: State<AppState>) {
    state.transport.play();
}

#[tauri::command]
pub fn transport_stop(state: State<AppState>) {
    state.transport.stop();
}

#[tauri::command]
pub fn transport_reset(state: State<AppState>) {
    state.transport.reset();
}

#[tauri::command]
pub fn set_bpm(bpm: f64, state: State<AppState>) {
    state.transport.set_bpm(bpm);
}

#[tauri::command]
pub fn set_time_signature(beats_per_bar: u8, beat_unit: u8, state: State<AppState>) {
    state.transport.set_time_signature(beats_per_bar, beat_unit);
}
