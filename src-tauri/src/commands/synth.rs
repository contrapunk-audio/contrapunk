//! Tauri commands for the built-in synth parameters.
//!
//! Each command writes an atomic on [`SynthParams`] shared with the
//! audio callback; changes take effect on the next buffer.

use serde::Serialize;
use tauri::State;

use contrapunk::synth::params::MIX_GROUP_COUNT;
use contrapunk::synth::Waveform;

use crate::state::AppState;

/// Snapshot shape for the UI.
#[derive(Serialize)]
pub struct SynthState {
    pub enabled: bool,
    pub waveform: u8,
    pub attack_ms: u32,
    pub decay_ms: u32,
    pub sustain: f32,
    pub release_ms: u32,
    pub cutoff_hz: u32,
    pub resonance: f32,
    pub master_gain: f32,
    pub mix_gains: [f32; 4],
}

#[tauri::command]
pub fn get_synth_state(state: State<AppState>) -> SynthState {
    let p = &state.synth_params;
    SynthState {
        enabled: p.enabled(),
        waveform: p.waveform() as u8,
        attack_ms: (p.attack_secs() * 1000.0) as u32,
        decay_ms: (p.decay_secs() * 1000.0) as u32,
        sustain: p.sustain_level(),
        release_ms: (p.release_secs() * 1000.0) as u32,
        cutoff_hz: p.cutoff_hz() as u32,
        resonance: p.resonance(),
        master_gain: p.master_gain(),
        mix_gains: p.mix_gains(),
    }
}

#[tauri::command]
pub fn set_synth_enabled(enabled: bool, state: State<AppState>) {
    state.synth_params.set_enabled(enabled);
}

#[tauri::command]
pub fn set_synth_waveform(value: u8, state: State<AppState>) {
    state.synth_params.set_waveform(Waveform::from_u8(value));
}

#[tauri::command]
pub fn set_synth_attack_ms(ms: u32, state: State<AppState>) {
    state.synth_params.set_attack_ms(ms);
}

#[tauri::command]
pub fn set_synth_decay_ms(ms: u32, state: State<AppState>) {
    state.synth_params.set_decay_ms(ms);
}

#[tauri::command]
pub fn set_synth_sustain(level: f32, state: State<AppState>) {
    state.synth_params.set_sustain_level(level);
}

#[tauri::command]
pub fn set_synth_release_ms(ms: u32, state: State<AppState>) {
    state.synth_params.set_release_ms(ms);
}

#[tauri::command]
pub fn set_synth_cutoff_hz(hz: u32, state: State<AppState>) {
    state.synth_params.set_cutoff_hz(hz);
}

#[tauri::command]
pub fn set_synth_resonance(value: f32, state: State<AppState>) {
    state.synth_params.set_resonance(value);
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
