//! Tauri commands for per-voice output routing.
//!
//! Exposes the `voice_outputs` table from `AppState` so the UI can
//! configure where each voice sends notes: internal synth, a specific
//! external MIDI port, off, or defer to the global routing mode.

use tauri::State;

use crate::state::{AppState, VoiceOutputTarget, MAX_VOICES};

/// Set the output destination for a single voice by index (0-based).
#[tauri::command]
pub fn set_voice_output(
    voice_idx: usize,
    target: VoiceOutputTarget,
    state: State<AppState>,
) -> Result<(), String> {
    if voice_idx >= MAX_VOICES {
        return Err(format!(
            "voice_idx {} out of range (max {})",
            voice_idx,
            MAX_VOICES - 1
        ));
    }
    let mut outputs = state.voice_outputs.lock().map_err(|e| e.to_string())?;
    outputs[voice_idx] = target;
    Ok(())
}

/// Return the current voice-output routing table as a Vec of length
/// MAX_VOICES. Each entry is the destination for that voice index.
#[tauri::command]
pub fn get_voice_outputs(state: State<AppState>) -> Result<Vec<VoiceOutputTarget>, String> {
    let outputs = state.voice_outputs.lock().map_err(|e| e.to_string())?;
    Ok(outputs.clone())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn voice_output_target_roundtrips_through_serde() {
        let cases = [
            VoiceOutputTarget::Synth,
            VoiceOutputTarget::MidiPort { port: 3 },
            VoiceOutputTarget::Off,
        ];
        for t in cases {
            let json = serde_json::to_string(&t).unwrap();
            let back: VoiceOutputTarget = serde_json::from_str(&json).unwrap();
            assert_eq!(t, back, "failed roundtrip: {json}");
        }
    }

    #[test]
    fn voice_output_target_default_is_synth() {
        assert_eq!(VoiceOutputTarget::default(), VoiceOutputTarget::Synth);
    }

    #[test]
    fn voice_output_target_json_shape_is_tagged() {
        // Frontend contract: `{ kind: "midi_port", port: 2 }` and similar.
        // Lock the shape so type changes break the test instead of silently
        // breaking the TS adapter.
        assert_eq!(
            serde_json::to_string(&VoiceOutputTarget::Synth).unwrap(),
            r#"{"kind":"synth"}"#
        );
        assert_eq!(
            serde_json::to_string(&VoiceOutputTarget::MidiPort { port: 2 }).unwrap(),
            r#"{"kind":"midi_port","port":2}"#
        );
        assert_eq!(
            serde_json::to_string(&VoiceOutputTarget::Off).unwrap(),
            r#"{"kind":"off"}"#
        );
    }
}
