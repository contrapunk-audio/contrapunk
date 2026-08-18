//! Tauri commands for the role-aware Elixir foundations controls.

use serde::Serialize;
use tauri::State;

use contrapunk::elixir::{RolePatch, RolePatchState, MIX_GROUP_COUNT};

use crate::state::AppState;

#[derive(Serialize)]
pub struct SynthState {
    pub enabled: bool,
    pub master_gain: f32,
    pub mix_gains: [f32; MIX_GROUP_COUNT],
    pub role_patches: [RolePatchState; MIX_GROUP_COUNT],
}

#[tauri::command]
pub fn get_synth_state(state: State<AppState>) -> SynthState {
    let params = &state.synth_params;
    SynthState {
        enabled: params.enabled(),
        master_gain: params.master_gain(),
        mix_gains: params.mix_gains(),
        role_patches: params.role_patches().map(RolePatchState::from),
    }
}

#[tauri::command]
pub fn set_synth_enabled(enabled: bool, state: State<AppState>) {
    state.synth_params.set_enabled(enabled);
}

#[tauri::command]
pub fn set_synth_master_gain(value: f32, state: State<AppState>) {
    state.synth_params.set_master_gain(value);
}

#[tauri::command]
pub fn set_synth_mix_gain(group: usize, value: f32, state: State<AppState>) -> Result<(), String> {
    if group >= MIX_GROUP_COUNT || !value.is_finite() {
        return Err("invalid synth mix gain".into());
    }
    state.synth_params.set_mix_gain(group, value);
    Ok(())
}

#[tauri::command]
pub fn set_synth_role_patch(
    group: usize,
    patch: RolePatchState,
    state: State<AppState>,
) -> Result<(), String> {
    if group >= MIX_GROUP_COUNT {
        return Err("invalid synth role".into());
    }
    if state
        .synth_params
        .set_role_patch(group, RolePatch::from(patch))
    {
        Ok(())
    } else {
        Err("invalid synth role".into())
    }
}
