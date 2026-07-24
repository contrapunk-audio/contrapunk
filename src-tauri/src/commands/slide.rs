use contrapunk::slide::SlideConfig;
use tauri::State;

use crate::state::AppState;

#[tauri::command]
pub fn get_slide_config(state: State<AppState>) -> SlideConfig {
    *state
        .slide_config
        .lock()
        .unwrap_or_else(|error| error.into_inner())
}

#[tauri::command]
pub fn set_slide_config(config: SlideConfig, state: State<AppState>) -> Result<(), String> {
    if !config.validate() {
        return Err("Invalid Slide configuration".into());
    }
    *state
        .slide_config
        .lock()
        .unwrap_or_else(|error| error.into_inner()) = config;
    Ok(())
}
