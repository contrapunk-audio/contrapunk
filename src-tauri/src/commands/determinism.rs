//! Versioned deterministic-performance snapshot and clean runtime reset.

use std::sync::atomic::Ordering;

use contrapunk::elixir::SynthEvent;
use contrapunk::harmony::{ExplicitIntervalMap, TuningConfig};
use contrapunk_companion::{CompanionState, LoopBuffer};
use serde::Serialize;
use tauri::State;

use crate::state::{AppState, VoiceOutputTarget};

pub const PERFORMANCE_SNAPSHOT_SCHEMA_VERSION: u16 = 1;

#[derive(Serialize)]
pub struct PerformanceSnapshot {
    pub schema_version: u16,
    pub app_version: &'static str,
    pub engine: EngineSnapshot,
    pub companion: CompanionState,
    pub slide: contrapunk::slide::SlideConfig,
    pub routing: RoutingSnapshot,
    pub transport: TransportSnapshot,
    pub synth: SynthSnapshot,
    pub detune_cents: i32,
    pub loop_buffer: Option<LoopBuffer>,
}

#[derive(Serialize)]
pub struct EngineSnapshot {
    pub key: String,
    pub mode: String,
    pub scale_mode: String,
    pub octave_mode: String,
    pub octave_intensity: f32,
    pub voice_count: usize,
    pub voice_position: usize,
    pub voice_leading_enabled: bool,
    pub voice_leading_style: String,
    pub interchange_enabled: bool,
    pub borrowing_range: u8,
    pub auto_key: bool,
    pub counterpoint_species: String,
    pub counterpoint_strictness: String,
    pub explicit_interval_map: ExplicitIntervalMap,
    pub tuning: TuningConfig,
    pub suppress_bass_register: bool,
    pub bass_register_threshold: u8,
}

#[derive(Serialize)]
pub struct RoutingSnapshot {
    pub mode: String,
    pub all_to_synth: bool,
    pub assignments: Vec<RouteSnapshot>,
}

#[derive(Serialize)]
pub struct RouteSnapshot {
    pub route: String,
    pub target: VoiceOutputTarget,
}

#[derive(Serialize)]
pub struct TransportSnapshot {
    pub running: bool,
    pub bpm: f64,
    pub beats_per_bar: u8,
    pub beat_unit: u8,
}

#[derive(Serialize)]
pub struct SynthSnapshot {
    pub enabled: bool,
    pub master_gain: f32,
    pub mix_gains: [f32; contrapunk::elixir::MIX_GROUP_COUNT],
}

pub(crate) fn snapshot(state: &AppState) -> Result<PerformanceSnapshot, String> {
    let engine = {
        let engine = state.engine.lock().map_err(|error| error.to_string())?;
        EngineSnapshot {
            key: format!("{}", engine.key()),
            mode: format!("{:?}", engine.mode()),
            scale_mode: format!("{:?}", engine.scale_mode()),
            octave_mode: format!("{:?}", engine.octave_mode()),
            octave_intensity: engine.octave_intensity(),
            voice_count: engine.voice_count(),
            voice_position: engine.voice_position(),
            voice_leading_enabled: engine.voice_leading_enabled(),
            voice_leading_style: format!("{:?}", engine.voice_leading_style()),
            interchange_enabled: engine.interchange_enabled(),
            borrowing_range: engine.borrowing_range(),
            auto_key: engine.auto_key(),
            counterpoint_species: format!("{:?}", engine.counterpoint_species()),
            counterpoint_strictness: format!("{:?}", engine.counterpoint_strictness()),
            explicit_interval_map: engine.explicit_interval_map().clone(),
            tuning: engine.tuning_config(),
            suppress_bass_register: engine.suppress_bass_register(),
            bass_register_threshold: engine.bass_register_threshold(),
        }
    };
    let companion = state
        .companion
        .lock()
        .map_err(|error| error.to_string())?
        .save();
    let slide = *state
        .slide_config
        .lock()
        .map_err(|error| error.to_string())?;
    let routing = {
        let routes = state
            .voice_outputs
            .lock()
            .map_err(|error| error.to_string())?;
        RoutingSnapshot {
            mode: format!(
                "{:?}",
                *state
                    .routing_mode
                    .lock()
                    .map_err(|error| error.to_string())?
            ),
            all_to_synth: routes.all_to_synth(),
            assignments: routes
                .assignments()
                .map(|(route, target)| RouteSnapshot {
                    route: route.key(),
                    target,
                })
                .collect(),
        }
    };
    let (beats_per_bar, beat_unit) = state.transport.time_signature();
    let transport = TransportSnapshot {
        running: state.transport.is_running(),
        bpm: state.transport.bpm(),
        beats_per_bar,
        beat_unit,
    };
    let synth = SynthSnapshot {
        enabled: state.synth_params.enabled(),
        master_gain: state.synth_params.master_gain(),
        mix_gains: state.synth_params.mix_gains(),
    };
    let loop_buffer = state
        .looper
        .lock()
        .map_err(|error| error.to_string())?
        .buffer()
        .cloned();

    Ok(PerformanceSnapshot {
        schema_version: PERFORMANCE_SNAPSHOT_SCHEMA_VERSION,
        app_version: env!("CARGO_PKG_VERSION"),
        engine,
        companion,
        slide,
        routing,
        transport,
        synth,
        detune_cents: state.detune_cents.load(Ordering::Relaxed),
        loop_buffer,
    })
}

pub(crate) fn reset(state: &AppState) -> Result<(), String> {
    state.transport.stop();
    state.transport.reset();
    state
        .looper
        .lock()
        .map_err(|error| error.to_string())?
        .transport_discontinuity();
    state
        .companion
        .lock()
        .map_err(|error| error.to_string())?
        .reset_runtime();
    {
        let mut engine = state.engine.lock().map_err(|error| error.to_string())?;
        *engine = engine.fork_clean_runtime();
    }
    if let Err(error) = state.synth_tx.send(SynthEvent::AllNotesOff) {
        eprintln!("[determinism] could not immediately silence synth during reset: {error}");
    }
    state
        .route_changes
        .lock()
        .map_err(|error| error.to_string())?
        .mark_all();
    state
        .performance_reset_revision
        .fetch_add(1, Ordering::AcqRel);
    Ok(())
}

#[tauri::command]
pub fn get_performance_snapshot(state: State<AppState>) -> Result<PerformanceSnapshot, String> {
    snapshot(&state)
}

#[tauri::command]
pub fn reset_performance(state: State<AppState>) -> Result<PerformanceSnapshot, String> {
    reset(&state)?;
    snapshot(&state)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::VoiceRouteId;
    use contrapunk::harmony::{HarmonyMode, Key};
    use contrapunk_companion::{InputOrigin, LoopMidiEvent, LoopState, OriginMidiEvent};
    use wmidi::Note;

    #[test]
    fn snapshot_is_versioned_and_routes_are_canonical() {
        let state = AppState::new();
        {
            let mut routes = state.voice_outputs.lock().unwrap();
            routes.set(VoiceRouteId::PatternCounter, VoiceOutputTarget::Off);
            routes.set(VoiceRouteId::Input, VoiceOutputTarget::Off);
            routes.set(
                VoiceRouteId::Harmony { slot: 2 },
                VoiceOutputTarget::MidiPort { port: 7 },
            );
        }

        let value = serde_json::to_value(snapshot(&state).unwrap()).unwrap();
        assert_eq!(value["schema_version"], 1);
        assert_eq!(
            value["routing"]["assignments"]
                .as_array()
                .unwrap()
                .iter()
                .map(|route| route["route"].as_str().unwrap())
                .collect::<Vec<_>>(),
            ["input", "harmony:2", "pattern_counter"]
        );
    }

    #[test]
    fn reset_preserves_the_take_but_stops_and_rewinds_loop_playback() {
        let state = AppState::new();
        {
            let mut looper = state.looper.lock().unwrap();
            looper.press(0.0, 4, true);
            looper.capture(
                OriginMidiEvent {
                    origin: InputOrigin::Live,
                    event: LoopMidiEvent::NoteOn {
                        note: 60,
                        velocity: 88,
                        channel: 0,
                    },
                    scheduled_beat_us: None,
                },
                4.0,
            );
            looper.capture(
                OriginMidiEvent {
                    origin: InputOrigin::Live,
                    event: LoopMidiEvent::NoteOff {
                        note: 60,
                        velocity: 17,
                        channel: 0,
                    },
                    scheduled_beat_us: None,
                },
                4.5,
            );
            looper.press(4.5, 4, true);
            looper.tick(8.0);
            assert!(matches!(looper.state(), LoopState::Playing { .. }));
        }

        reset(&state).unwrap();

        let looper = state.looper.lock().unwrap();
        assert_eq!(looper.state(), LoopState::Stopped);
        assert!(looper.buffer().is_some());
    }

    #[test]
    fn reset_rewinds_transport_and_recreates_clean_harmony_history() {
        let state = AppState::new();
        state.transport.play();
        let _ = state.transport.advance(48_000);
        let expected = {
            let mut engine = state.engine.lock().unwrap();
            engine.set_key(Key::C);
            engine.set_mode(HarmonyMode::ContraryMotion);
            engine.set_voice_count(3);
            engine.harmonize_note_on(Note::try_from(60).unwrap());
            let mut clean = engine.fork_clean_runtime();
            clean.harmonize_note_on(Note::try_from(64).unwrap())
        };
        let before_revision = state.performance_reset_revision.load(Ordering::Acquire);

        reset(&state).unwrap();

        let actual = state
            .engine
            .lock()
            .unwrap()
            .harmonize_note_on(Note::try_from(64).unwrap());
        assert_eq!(actual, expected);
        assert_eq!(state.transport.sample_pos(), 0);
        assert!(!state.transport.is_running());
        assert!(state.route_changes.lock().unwrap().all());
        assert_eq!(
            state.performance_reset_revision.load(Ordering::Acquire),
            before_revision + 1
        );
    }
}
