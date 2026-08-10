//! Tauri commands for per-voice output routing.
//!
//! Exposes the `voice_outputs` table from `AppState` so the UI can
//! configure where each voice sends notes: internal synth, a specific
//! external MIDI port, off, or defer to the global routing mode.

use std::sync::atomic::Ordering;

use serde::Serialize;
use tauri::State;

use crate::state::{AppState, VoiceOutputTarget, VoiceRouteId};

#[derive(Serialize)]
pub struct VoiceOutputAssignment {
    route: String,
    target: VoiceOutputTarget,
}

/// Set one stable musical part's output destination.
#[tauri::command]
pub fn set_voice_output(
    route: String,
    target: VoiceOutputTarget,
    state: State<AppState>,
) -> Result<(), String> {
    let route = VoiceRouteId::parse(&route)?;
    let changed = state
        .voice_outputs
        .lock()
        .map_err(|e| e.to_string())?
        .set(route, target);
    if changed {
        state.route_change_pending.store(true, Ordering::Release);
    }
    Ok(())
}

/// Temporarily send every part to the internal synth without replacing the
/// user's per-part assignments.
#[tauri::command]
pub fn set_all_voice_outputs_to_synth(enabled: bool, state: State<AppState>) -> Result<(), String> {
    let changed = state
        .voice_outputs
        .lock()
        .map_err(|e| e.to_string())?
        .set_all_to_synth(enabled);
    if changed {
        state.route_change_pending.store(true, Ordering::Release);
    }
    Ok(())
}

/// Return only non-default assignments. Missing routes resolve to Synth.
#[tauri::command]
pub fn get_voice_outputs(state: State<AppState>) -> Result<Vec<VoiceOutputAssignment>, String> {
    let outputs = state.voice_outputs.lock().map_err(|e| e.to_string())?;
    Ok(outputs
        .assignments()
        .map(|(route, target)| VoiceOutputAssignment {
            route: route.key(),
            target,
        })
        .collect())
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
    fn voice_route_keys_roundtrip_and_reject_bad_slots() {
        for key in [
            "input",
            "harmony:0",
            "harmony:7",
            "canon:3",
            "counterpoint:1",
            "pattern_low",
            "pattern_counter",
        ] {
            assert_eq!(VoiceRouteId::parse(key).unwrap().key(), key);
        }
        assert!(VoiceRouteId::parse("harmony:8").is_err());
        assert!(VoiceRouteId::parse("unknown:0").is_err());
    }

    #[test]
    fn route_table_defaults_to_synth_and_keeps_parts_independent() {
        let mut routes = crate::state::VoiceOutputRoutes::default();
        let canon = VoiceRouteId::Canon { voice: 0 };
        let pattern = VoiceRouteId::PatternLow;
        assert_eq!(routes.get(canon), VoiceOutputTarget::Synth);
        assert!(routes.set(canon, VoiceOutputTarget::MidiPort { port: 2 }));
        assert_eq!(routes.get(canon), VoiceOutputTarget::MidiPort { port: 2 });
        assert_eq!(routes.get(pattern), VoiceOutputTarget::Synth);
        assert!(!routes.set(canon, VoiceOutputTarget::MidiPort { port: 2 }));
        assert!(routes.set(canon, VoiceOutputTarget::Synth));
        assert_eq!(routes.get(canon), VoiceOutputTarget::Synth);
    }

    #[test]
    fn all_synth_override_preserves_user_assignments() {
        let mut routes = crate::state::VoiceOutputRoutes::default();
        let input = VoiceRouteId::Input;
        let canon = VoiceRouteId::Canon { voice: 0 };
        routes.set(input, VoiceOutputTarget::MidiPort { port: 2 });
        routes.set(canon, VoiceOutputTarget::Off);

        assert!(routes.set_all_to_synth(true));
        assert_eq!(routes.get(input), VoiceOutputTarget::Synth);
        assert_eq!(routes.get(canon), VoiceOutputTarget::Synth);
        assert!(!routes.has_external_target());

        assert!(routes.set_all_to_synth(false));
        assert_eq!(routes.get(input), VoiceOutputTarget::MidiPort { port: 2 });
        assert_eq!(routes.get(canon), VoiceOutputTarget::Off);
        assert!(routes.has_external_target());
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
