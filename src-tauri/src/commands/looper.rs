//! Volatile one-slot MIDI looper commands.

use tauri::State;

use contrapunk_companion::LoopStatus;

use crate::state::AppState;

#[tauri::command]
pub fn looper_press(state: State<AppState>) -> Result<LoopStatus, String> {
    press(&state)
}

fn press(state: &AppState) -> Result<LoopStatus, String> {
    let (beats_per_bar, _) = state.transport.time_signature();
    let mut looper = state.looper.lock().map_err(|error| error.to_string())?;
    let outcome = looper.press(
        state.transport.total_beats(),
        beats_per_bar,
        state.transport.is_running(),
    );
    if outcome.reset_transport {
        state.transport.reset();
        looper.accept_discontinuity_revision(state.transport.discontinuity_revision());
    }
    if outcome.start_transport {
        state.transport.play();
    }
    Ok(looper.status())
}

#[tauri::command]
pub fn looper_clear(state: State<AppState>) -> Result<LoopStatus, String> {
    let mut looper = state.looper.lock().map_err(|error| error.to_string())?;
    looper.clear();
    Ok(looper.status())
}

#[tauri::command]
pub fn looper_status(state: State<AppState>) -> Result<LoopStatus, String> {
    let looper = state.looper.lock().map_err(|error| error.to_string())?;
    Ok(looper.status())
}

#[cfg(test)]
mod tests {
    use super::*;
    use contrapunk_companion::{LoopState, LoopStatusState};

    #[test]
    fn stopped_press_resets_starts_and_arms_one_bar_count_in() {
        let state = AppState::new();
        state.transport.set_time_signature(3, 4);
        state.transport.play();
        let _ = state.transport.advance(48_000);
        state.transport.stop();

        let status = press(&state).unwrap();

        assert!(state.transport.is_running());
        assert_eq!(state.transport.total_beats(), 0.0);
        assert_eq!(status.state, LoopStatusState::Armed);
        assert_eq!(
            state.looper.lock().unwrap().state(),
            LoopState::Armed {
                start_beat_us: 3_000_000,
                beats_per_bar_us: 3_000_000,
            }
        );
    }

    #[test]
    fn clear_is_volatile_and_returns_empty_status() {
        let state = AppState::new();
        state.transport.play();
        press(&state).unwrap();
        let mut looper = state.looper.lock().unwrap();
        looper.clear();
        assert_eq!(looper.status().state, LoopStatusState::Empty);
        assert!(!looper.status().has_loop);
    }
}
