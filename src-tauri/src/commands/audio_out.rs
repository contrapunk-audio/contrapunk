//! Tauri commands for audio output.
//!
//! Thin wrappers around `contrapunk::audio_out::AudioOutEngine`. The engine
//! instance lives in `AppState`. Starting audio output creates the cpal
//! stream and stores the resulting MidiProducer in AppState so the router
//! thread can pick it up (via Task 9's wiring).

use contrapunk::audio_out::AudioConfig;
use serde::Serialize;
use tauri::State;

use crate::state::AppState;

#[derive(Clone, Debug, Serialize)]
pub struct AudioDeviceInfoJs {
    pub name: String,
    pub is_default: bool,
}

/// Enumerate available output devices. Returns empty Vec if no audio.
#[tauri::command]
pub fn list_audio_output_devices() -> Vec<AudioDeviceInfoJs> {
    contrapunk::audio_out::AudioOutEngine::list_output_devices()
        .into_iter()
        .map(|d| AudioDeviceInfoJs {
            name: d.name,
            is_default: d.is_default,
        })
        .collect()
}

/// Start the audio output engine.
#[tauri::command]
pub fn start_audio_output(
    device_id: Option<String>,
    sample_rate: Option<u32>,
    buffer_size: Option<u32>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let mut engine = state.audio_out.lock().map_err(|e| e.to_string())?;
    let mut producer_slot = state.audio_out_producer.lock().map_err(|e| e.to_string())?;

    let mut cfg = AudioConfig::default();
    if let Some(id) = device_id {
        cfg.device_id = Some(id);
    }
    if let Some(sr) = sample_rate {
        cfg.sample_rate = sr;
    }
    if let Some(bs) = buffer_size {
        cfg.buffer_size = bs;
    }

    let producer = engine.start(cfg)?;
    *producer_slot = Some(producer);
    // NOTE: If routing is already running, the producer sits idle in AppState
    // until the router restarts. Users must stop & restart routing to pick up
    // the new audio output. v1 limitation — revisit in sub-project 4 when the
    // Routing tab lands.
    Ok(())
}

/// Stop the audio output engine.
#[tauri::command]
pub fn stop_audio_output(state: State<'_, AppState>) -> Result<(), String> {
    let mut engine = state.audio_out.lock().map_err(|e| e.to_string())?;
    let mut producer_slot = state.audio_out_producer.lock().map_err(|e| e.to_string())?;
    engine.stop();
    *producer_slot = None;
    Ok(())
}

/// Whether audio output is currently running.
#[tauri::command]
pub fn is_audio_output_running(state: State<'_, AppState>) -> Result<bool, String> {
    let engine = state.audio_out.lock().map_err(|e| e.to_string())?;
    Ok(engine.is_running())
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_device_info_serializes() {
        let info = super::AudioDeviceInfoJs {
            name: "Test".to_string(),
            is_default: true,
        };
        let json = serde_json::to_string(&info).expect("serialize");
        assert!(json.contains("\"name\":\"Test\""));
        assert!(json.contains("\"is_default\":true"));
    }
}
