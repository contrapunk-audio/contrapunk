//! Tauri commands for transport control (play, stop, BPM, time sig,
//! metronome click).

use std::sync::atomic::Ordering;

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
    pub metronome_enabled: bool,
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
        metronome_enabled: state.metronome_enabled.load(Ordering::Relaxed),
    }
}

#[tauri::command]
pub fn set_metronome_enabled(enabled: bool, state: State<AppState>) {
    state.metronome_enabled.store(enabled, Ordering::Relaxed);
}

#[tauri::command]
pub fn transport_play(state: State<AppState>) {
    state.transport.play();
}

fn stop_transport(state: &AppState) {
    state.transport.stop();
    crate::commands::engine::request_all_notes_off(state);
}

#[tauri::command]
pub fn transport_stop(state: State<AppState>) {
    stop_transport(&state);
}

fn reset_transport(state: &AppState) {
    state.transport.reset();
    crate::commands::engine::request_all_notes_off(state);
}

#[tauri::command]
pub fn transport_reset(state: State<AppState>) {
    reset_transport(&state);
}

#[tauri::command]
pub fn set_bpm(bpm: f64, state: State<AppState>) {
    state.transport.set_bpm(bpm);
}

#[tauri::command]
pub fn set_time_signature(beats_per_bar: u8, beat_unit: u8, state: State<AppState>) {
    state.transport.set_time_signature(beats_per_bar, beat_unit);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc;
    use std::time::Duration;

    #[test]
    fn stop_freezes_clock_and_requests_router_all_notes_off() {
        let state = AppState::new();
        state.transport.play();
        let (tx, rx) = mpsc::channel();
        *state.router_tx.lock().unwrap() = Some(tx);

        stop_transport(&state);

        assert!(!state.transport.is_running());
        assert_eq!(
            rx.recv_timeout(Duration::from_millis(50)).unwrap(),
            vec![0xB0, 123, 0]
        );
    }

    #[test]
    fn reset_rewinds_clock_and_requests_router_all_notes_off() {
        let state = AppState::new();
        state.transport.play();
        let _ = state.transport.advance(48_000);
        let (tx, rx) = mpsc::channel();
        *state.router_tx.lock().unwrap() = Some(tx);

        reset_transport(&state);

        assert_eq!(state.transport.sample_pos(), 0);
        assert_eq!(
            rx.recv_timeout(Duration::from_millis(50)).unwrap(),
            vec![0xB0, 123, 0]
        );
    }
}
